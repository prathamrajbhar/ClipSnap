mod clipboard;
mod config;
mod database;
mod hotkeys;
mod models;
mod notifications;
mod screenshot;
mod ui;

use config::Config;
use database::Database;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use gtk4::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureMode {
    StandardImage,
    ExtractText,
    DecodeQR,
}

/// Tasks handled by the background worker thread.
pub enum WorkerTask {
    ProcessScreenshot {
        rgba_pixels: Vec<u8>,
        width: u32,
        height: u32,
        mode: CaptureMode,
    },
    ProcessClipboardImage {
        rgba_pixels: Vec<u8>,
        width: u32,
        height: u32,
    },
}

fn main() {
    // ── Logging ─────────────────────────────────────
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    log::info!("ClipSnap starting…");

    // ── Configuration ───────────────────────────────
    let config = Config::load_or_create_default().expect("Failed to load configuration");

    // ── Database ────────────────────────────────────
    let db_path = config.resolved_db_path();
    let db = Arc::new(Mutex::new(
        Database::new(&db_path).expect("Failed to initialise database"),
    ));

    // Run maintenance on startup
    {
        let db = db.lock().unwrap();
        if config.history.auto_cleanup {
            let _ = db.cleanup_old_entries(config.history.retention_days);
            let _ = db.enforce_max_entries(config.history.max_entries);
        }
    }

    // ── Shared state for deduplication ───────────────
    let last_text_hash: Arc<Mutex<Option<u64>>> = Arc::new(Mutex::new(None));
    let last_image_hash: Arc<Mutex<Option<u64>>> = Arc::new(Mutex::new(None));

    // ── Background Worker ───────────────────────────
    let (worker_tx, worker_rx) = mpsc::channel::<WorkerTask>();
    let db_worker = db.clone();
    let clipboard_worker = Arc::new(Mutex::new(
        arboard::Clipboard::new().expect("Failed to initialise clipboard for worker"),
    ));

    let lih_worker = last_image_hash.clone();
    std::thread::Builder::new()
        .name("background-worker".into())
        .spawn(move || {
            while let Ok(task) = worker_rx.recv() {
                match task {
                    WorkerTask::ProcessScreenshot { rgba_pixels, width, height, mode } => {
                        log::info!("Worker: Processing screenshot ({}x{}) mode={:?}", width, height, mode);
                        process_image(&rgba_pixels, width, height, mode, &db_worker, &clipboard_worker, &lih_worker);
                    }
                    WorkerTask::ProcessClipboardImage { rgba_pixels, width, height } => {
                        log::info!("Worker: Processing clipboard image ({}x{})", width, height);
                        process_image(&rgba_pixels, width, height, CaptureMode::StandardImage, &db_worker, &clipboard_worker, &lih_worker);
                    }
                }
            }
        })
        .expect("Failed to spawn worker thread");



    // ── Clipboard Monitoring Thread (Updated) ───────
    let clipboard = Arc::new(Mutex::new(
        arboard::Clipboard::new().expect("Failed to initialise clipboard"),
    ));

    {
        let db_monitor = db.clone();
        let cb_monitor = clipboard.clone();
        let lth = last_text_hash.clone();
        let lih = last_image_hash.clone();
        let wt_monitor = worker_tx.clone();
        std::thread::Builder::new()
            .name("clipboard-monitor".into())
            .spawn(move || {
                clipboard::monitor_clipboard(cb_monitor, db_monitor, lth, lih, wt_monitor);
            })
            .expect("Failed to spawn clipboard monitor thread");
    }

    // ── GTK Application ─────────────────────────────
    let app = gtk4::Application::builder()
        .application_id("com.clipsnap.daemon")
        .build();

    let config = Arc::new(config);
    let db_activate = db.clone();
    let cb_activate = clipboard.clone();
    let config_activate = config.clone();
    let worker_tx_activate = worker_tx.clone();

    app.connect_activate(move |app| {
        let hotkey_manager = match GlobalHotKeyManager::new() {
            Ok(m) => m,
            Err(e) => {
                log::error!("Failed to initialise hotkey manager: {}", e);
                return;
            }
        };

        let screenshot_hk = hotkeys::parse_hotkey(&config_activate.shortcuts.screenshot).expect("Invalid screenshot shortcut");
        let history_hk = hotkeys::parse_hotkey(&config_activate.shortcuts.history).expect("Invalid history shortcut");
        let extract_text_hk = hotkeys::parse_hotkey(&config_activate.shortcuts.extract_text).expect("Invalid extract_text shortcut");
        let decode_qr_hk = hotkeys::parse_hotkey(&config_activate.shortcuts.decode_qr).expect("Invalid decode_qr shortcut");

        let screenshot_id = screenshot_hk.id();
        let history_id = history_hk.id();
        let extract_text_id = extract_text_hk.id();
        let decode_qr_id = decode_qr_hk.id();

        let _ = hotkey_manager.register(screenshot_hk);
        let _ = hotkey_manager.register(history_hk);
        let _ = hotkey_manager.register(extract_text_hk);
        let _ = hotkey_manager.register(decode_qr_hk);

        let app_weak = app.downgrade();
        let db_hotkey = db_activate.clone();
        let cb_hotkey = cb_activate.clone();
        let wt_hotkey = worker_tx_activate.clone();
        
        let hold_guard = app.hold();
        glib::timeout_add_local(Duration::from_millis(100), move || {
            let _hold = &hold_guard;
            let _keep = &hotkey_manager;

            while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                if event.state == HotKeyState::Pressed {
                    // --- Added check for modal dialogs to prevent hangs ---
                    if let Some(app) = app_weak.upgrade() {
                        let windows = app.windows();
                        let is_any_modal = windows.iter().any(|w| w.is_modal());
                        if is_any_modal {
                            log::warn!("Hotkey ignored: modal dialog is active.");
                            continue;
                        }
                    }

                    if event.id == screenshot_id || event.id == extract_text_id || event.id == decode_qr_id {
                        let mode = if event.id == screenshot_id { 
                            CaptureMode::StandardImage 
                        } else if event.id == extract_text_id {
                            CaptureMode::ExtractText
                        } else {
                            CaptureMode::DecodeQR
                        };
                        
                        // --- Multi-Monitor: Calculate total virtual desktop bounds ---
                        let display = gdk4::Display::default().expect("Failed to get default display");
                        let monitors = display.monitors();
                        
                        let mut min_x = i32::MAX;
                        let mut min_y = i32::MAX;
                        let mut max_x = i32::MIN;
                        let mut max_y = i32::MIN;

                        for i in 0..monitors.n_items() {
                            if let Some(monitor) = monitors.item(i).and_then(|obj| obj.downcast::<gdk4::Monitor>().ok()) {
                                let geometry = monitor.geometry();
                                let scale = monitor.scale_factor();
                                
                                // Geometry is in logical pixels. Capture needs device pixels.
                                let x = geometry.x() * scale;
                                let y = geometry.y() * scale;
                                let w = geometry.width() * scale;
                                let h = geometry.height() * scale;

                                min_x = min_x.min(x);
                                min_y = min_y.min(y);
                                max_x = max_x.max(x + w);
                                max_y = max_y.max(y + h);
                            }
                        }

                        let total_w = (max_x - min_x) as u32;
                        let total_h = (max_y - min_y) as u32;

                        log::info!("Hotkey: Freezing virtual desktop for {:?} ({}x{} at {},{})...", mode, total_w, total_h, min_x, min_y);
                        
                        match screenshot::capture_entire_screen(min_x, min_y, total_w, total_h) {
                            Ok(frozen_image) => {
                                if let Some(app) = app_weak.upgrade() {
                                    ui::overlay::show_overlay(&app, wt_hotkey.clone(), frozen_image, mode);
                                }
                            }
                            Err(e) => {
                                log::error!("Capture failed: {}", e);
                                notifications::notify_screenshot_error(&format!("Capture error: {}", e));
                            }
                        }
                    } else if event.id == history_id {
                        if let Some(app) = app_weak.upgrade() {
                            ui::history_dialog::show_history(&app, db_hotkey.clone(), cb_hotkey.clone());
                        }
                    }
                }
            }
            glib::ControlFlow::Continue
        });

        log::info!("ClipSnap ready");
    });

    let exit_code = app.run();
    log::info!("ClipSnap exiting with code {:?}", exit_code);
}

fn process_image(
    rgba_pixels: &[u8],
    width: u32,
    height: u32,
    mode: CaptureMode,
    db_worker: &Arc<Mutex<Database>>,
    clipboard_worker: &Arc<Mutex<arboard::Clipboard>>,
    last_image_hash: &Arc<Mutex<Option<u64>>>,
) {
    if mode == CaptureMode::ExtractText {
        log::info!("Worker: Performing OCR...");
        match screenshot::encode_png(rgba_pixels, width, height) {
            Ok(png_bytes) => {
                let dynamic_image = match image::load_from_memory(&png_bytes) {
                    Ok(img) => img,
                    Err(e) => {
                        log::error!("Worker: Failed to load image for OCR: {}", e);
                        return;
                    }
                };

                let img = match rusty_tesseract::Image::from_dynamic_image(&dynamic_image) {
                    Ok(i) => i,
                    Err(e) => {
                        log::error!("Worker: Failed to create tesseract image: {}", e);
                        return;
                    }
                };
                
                let mut args = rusty_tesseract::Args::default();
                args.psm = Some(6); // Assume a single uniform block of text. Often better for snippets.
                
                match rusty_tesseract::image_to_string(&img, &args) {
                    Ok(text) => {
                        let trimmed = text.trim();
                        if trimmed.is_empty() {
                            log::warn!("Worker: OCR completed but no text was detected.");
                            notifications::notify_screenshot_error("No text detected in selected area");
                            return;
                        }

                        // Save to clipboard
                        if let Ok(mut cb) = clipboard_worker.lock() {
                            let _ = clipboard::set_clipboard_text(&mut cb, trimmed);
                        }

                        // Save to DB
                        if let Ok(db) = db_worker.lock() {
                            let _ = db.insert_text(trimmed);
                        }

                        notifications::notify_text_extraction(trimmed);
                        log::info!("Worker: OCR Success: {} characters extracted", trimmed.len());
                    }
                    Err(e) => {
                        log::error!("Worker: OCR failed: {}", e);
                        notifications::notify_screenshot_error(&format!("OCR Error: {}", e));
                    }
                }
            }
            Err(e) => log::error!("Worker: Encoding for OCR failed: {}", e),
        }
        return;
    }

    if mode == CaptureMode::DecodeQR {
        log::info!("Worker: Decoding QR Code...");
        match screenshot::encode_png(rgba_pixels, width, height) {
            Ok(png_bytes) => {
                let dynamic_image = match image::load_from_memory(&png_bytes) {
                    Ok(img) => img,
                    Err(e) => {
                        log::error!("Worker: Failed to load image for QR: {}", e);
                        return;
                    }
                };

                // rqrr needs a grayscale image
                let luma_img = dynamic_image.to_luma8();
                let mut img = rqrr::PreparedImage::prepare(luma_img);
                let grids = img.detect_grids();

                if grids.is_empty() {
                    log::warn!("Worker: No QR code detected.");
                    notifications::notify_screenshot_error("No QR Code detected in selection.");
                    return;
                }

                // Decode the first grid found
                let (_meta, content) = match grids[0].decode() {
                    Ok(res) => res,
                    Err(e) => {
                        log::error!("Worker: QR decode failed: {}", e);
                        notifications::notify_screenshot_error("Failed to decode QR Code content.");
                        return;
                    }
                };

                if content.is_empty() {
                    notifications::notify_screenshot_error("Failed to decode QR Code content.");
                    return;
                }

                // Save to clipboard
                if let Ok(mut cb) = clipboard_worker.lock() {
                    let _ = clipboard::set_clipboard_text(&mut cb, &content);
                }

                // Save to DB
                if let Ok(db) = db_worker.lock() {
                    let _ = db.insert_text(&content);
                }

                notifications::notify_text_extraction(&content); // Reuse same notification for QR result
                log::info!("Worker: QR Decode Success: {} chars", content.len());
            }
            Err(e) => log::error!("Worker: Encoding for QR failed: {}", e),
        }
        return;
    }

    match screenshot::encode_png(rgba_pixels, width, height) {
        Ok(png_bytes) => {
            let thumb = screenshot::create_thumbnail(&png_bytes, 150).unwrap_or_default();
            
            // Save to DB
            if let Ok(db) = db_worker.lock() {
                if let Err(e) = db.insert_image(&png_bytes, &thumb) {
                    log::error!("Worker: Failed to save to DB: {}", e);
                }
            }

            // If it was a screenshot, also copy to clipboard and notify
            if mode == CaptureMode::StandardImage {
                // Update the shared hash BEFORE setting to clipboard to prevent monitor loopback
                let hash = clipboard::calculate_hash(rgba_pixels);
                if let Ok(mut lih) = last_image_hash.lock() {
                    *lih = Some(hash);
                }

                if let Ok(mut cb) = clipboard_worker.lock() {
                    let _ = clipboard::set_clipboard_image(&mut cb, rgba_pixels, width as usize, height as usize);
                }
                // Save a temporary copy for the notification preview
                let cache_dir = dirs::cache_dir().unwrap_or_else(|| std::env::temp_dir()).join("clipsnap");
                let _ = std::fs::create_dir_all(&cache_dir);
                let tmp_path = cache_dir.join("last_screenshot.png");
                
                if let Err(e) = std::fs::write(&tmp_path, &png_bytes) {
                    log::error!("Worker: Failed to write preview file: {}", e);
                }
                notifications::notify_screenshot_success(&tmp_path);
            }
        }
        Err(e) => log::error!("Worker: Encoding failed: {}", e),
    }
}
