use crate::database::Database;
use crate::WorkerTask;
use anyhow::{Context, Result};
use arboard::{Clipboard, ImageData};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::sync::mpsc::Sender;
use x11rb::connection::Connection;
use x11rb::protocol::xfixes::{self, ConnectionExt as _};
use x11rb::protocol::xproto::ConnectionExt as _;
use x11rb::rust_connection::RustConnection;

/// Set an image (RGBA pixels) to the system clipboard.
pub fn set_clipboard_image(clipboard: &mut Clipboard, rgba: &[u8], width: usize, height: usize) -> Result<()> {
    let img_data = ImageData {
        width,
        height,
        bytes: rgba.to_vec().into(),
    };
    clipboard
        .set_image(img_data)
        .map_err(|e| anyhow::anyhow!("Failed to set clipboard image: {}", e))?;
    Ok(())
}

/// Set text to the system clipboard.
pub fn set_clipboard_text(clipboard: &mut Clipboard, text: &str) -> Result<()> {
    clipboard
        .set_text(text.to_string())
        .map_err(|e| anyhow::anyhow!("Failed to set clipboard text: {}", e))?;
    Ok(())
}

/// Calculate a hash of data for deduplication.
pub fn calculate_hash(data: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

/// Run clipboard monitoring in a background thread using X11 events.
pub fn monitor_clipboard(
    clipboard: Arc<Mutex<Clipboard>>,
    db: Arc<Mutex<Database>>,
    last_text_hash: Arc<Mutex<Option<u64>>>,
    last_image_hash: Arc<Mutex<Option<u64>>>,
    worker_tx: Sender<WorkerTask>,
) {
    log::info!("Clipboard monitoring: Initializing X11 SelectionNotify listener");

    // Try to set up X11 event-driven monitoring
    match setup_x11_listener() {
        Ok(conn) => {
            log::info!("Clipboard monitoring: Using event-driven X11 listener");
            loop {
                match conn.wait_for_event() {
                    Ok(event) => {
                        if let x11rb::protocol::Event::XfixesSelectionNotify(_) = event {
                            check_clipboard(&clipboard, &db, &last_text_hash, &last_image_hash, &worker_tx);
                        }
                    }
                    Err(e) => {
                        log::error!("X11 event loop error: {}. Falling back to polling.", e);
                        run_polling_loop(clipboard, db, last_text_hash, last_image_hash, worker_tx);
                        break;
                    }
                }
            }
        }
        Err(e) => {
            log::warn!("Failed to setup X11 listener: {}. Falling back to polling.", e);
            run_polling_loop(clipboard, db, last_text_hash, last_image_hash, worker_tx);
        }
    }
}

fn setup_x11_listener() -> Result<RustConnection> {
    let (conn, screen_num) = RustConnection::connect(None).context("X11 connect failed")?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    // Request XFixes extension
    let xfixes_version = conn.xfixes_query_version(4, 0)?.reply()?;
    if xfixes_version.major_version < 4 {
        return Err(anyhow::anyhow!("XFixes version too old"));
    }

    // Get the CLIPBOARD atom
    let clipboard_atom = conn.intern_atom(false, b"CLIPBOARD")?.reply()?.atom;

    // Listen for selection changes
    conn.xfixes_select_selection_input(
        root,
        clipboard_atom,
        xfixes::SelectionEventMask::SET_SELECTION_OWNER |
        xfixes::SelectionEventMask::SELECTION_WINDOW_DESTROY |
        xfixes::SelectionEventMask::SELECTION_CLIENT_CLOSE,
    )?;
    conn.flush()?;

    Ok(conn)
}

fn check_clipboard(
    clipboard: &Arc<Mutex<Clipboard>>,
    db: &Arc<Mutex<Database>>,
    last_text_hash: &Arc<Mutex<Option<u64>>>,
    last_image_hash: &Arc<Mutex<Option<u64>>>,
    worker_tx: &Sender<WorkerTask>,
) {
    let mut cb = match clipboard.lock() {
        Ok(c) => c,
        Err(_) => return,
    };

    // --- Check text ---
    if let Ok(text) = cb.get_text() {
        if !text.is_empty() {
            let hash = calculate_hash(text.as_bytes());
            let is_new = {
                let last = last_text_hash.lock().unwrap();
                *last != Some(hash)
            };
            if is_new {
                if let Ok(db) = db.lock() {
                    let _ = db.insert_text(&text);
                }
                *last_text_hash.lock().unwrap() = Some(hash);
            }
        }
    }

    // --- Check image ---
    if let Ok(img) = cb.get_image() {
        if !img.bytes.is_empty() {
            let hash = calculate_hash(&img.bytes);
            let is_new = {
                let last = last_image_hash.lock().unwrap();
                *last != Some(hash)
            };
            if is_new {
                // Offload heavy processing to worker
                let _ = worker_tx.send(WorkerTask::ProcessClipboardImage {
                    rgba_pixels: img.bytes.to_vec(),
                    width: img.width as u32,
                    height: img.height as u32,
                });
                *last_image_hash.lock().unwrap() = Some(hash);
            }
        }
    }
}

/// Fallback polling loop if X11 events are unavailable.
fn run_polling_loop(
    clipboard: Arc<Mutex<Clipboard>>,
    db: Arc<Mutex<Database>>,
    last_text_hash: Arc<Mutex<Option<u64>>>,
    last_image_hash: Arc<Mutex<Option<u64>>>,
    worker_tx: Sender<WorkerTask>,
) {
    loop {
        std::thread::sleep(Duration::from_millis(1000));
        check_clipboard(&clipboard, &db, &last_text_hash, &last_image_hash, &worker_tx);
    }
}
