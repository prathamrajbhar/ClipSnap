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

fn main() {
    // ── Logging ─────────────────────────────────────
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    log::info!("ClipSnap starting…");

    // ── Configuration ───────────────────────────────
    let config = Config::load_or_create_default().expect("Failed to load configuration");
    log::info!(
        "Config loaded – screenshot: {}, history: {}",
        config.shortcuts.screenshot,
        config.shortcuts.history,
    );

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

    // ── Clipboard Monitoring Thread ─────────────
    let clipboard = Arc::new(Mutex::new(
        arboard::Clipboard::new().expect("Failed to initialise clipboard"),
    ));

    {
        let db_monitor = db.clone();
        let cb_monitor = clipboard.clone();
        let lth = last_text_hash.clone();
        let lih = last_image_hash.clone();
        std::thread::Builder::new()
            .name("clipboard-monitor".into())
            .spawn(move || {
                clipboard::monitor_clipboard(cb_monitor, db_monitor, lth, lih);
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

    app.connect_activate(move |app| {
        // ── Global Hotkeys ──────────────────────────
        let hotkey_manager = match GlobalHotKeyManager::new() {
            Ok(m) => m,
            Err(e) => {
                log::error!("Failed to initialise hotkey manager: {}", e);
                eprintln!("ERROR: Failed to initialise global hotkeys: {}", e);
                return;
            }
        };

        let screenshot_hk = match hotkeys::parse_hotkey(&config_activate.shortcuts.screenshot) {
            Ok(hk) => hk,
            Err(e) => {
                log::error!("Invalid screenshot shortcut: {}", e);
                eprintln!("ERROR: Invalid screenshot shortcut – {}", e);
                return;
            }
        };
        let history_hk = match hotkeys::parse_hotkey(&config_activate.shortcuts.history) {
            Ok(hk) => hk,
            Err(e) => {
                log::error!("Invalid history shortcut: {}", e);
                eprintln!("ERROR: Invalid history shortcut – {}", e);
                return;
            }
        };

        let screenshot_id = screenshot_hk.id();
        let history_id = history_hk.id();

        if let Err(e) = hotkey_manager.register(screenshot_hk) {
            log::error!("Failed to register screenshot hotkey: {}", e);
            eprintln!(
                "WARNING: Could not register screenshot hotkey ({}). It may conflict with your DE.",
                config_activate.shortcuts.screenshot
            );
        } else {
            log::info!("Registered screenshot hotkey: {} (ID: {})", config_activate.shortcuts.screenshot, screenshot_id);
        }
        if let Err(e) = hotkey_manager.register(history_hk) {
            log::error!("Failed to register history hotkey: {}", e);
            eprintln!(
                "WARNING: Could not register history hotkey ({}). It may conflict with your DE.",
                config_activate.shortcuts.history
            );
        } else {
            log::info!("Registered history hotkey: {} (ID: {})", config_activate.shortcuts.history, history_id);
        }

        // ── Hotkey Polling (on GTK main loop) ───────
        let app_weak = app.downgrade();
        let db_hotkey = db_activate.clone();
        let cb_hotkey = cb_activate.clone();
        let hold_guard = app.hold();
        glib::timeout_add_local(Duration::from_millis(100), move || {
            let _hold = &hold_guard;
            let _keep = &hotkey_manager;

            while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                if event.state == HotKeyState::Pressed {
                    if event.id == screenshot_id {
                        log::info!("Screenshot hotkey pressed");
                        if let Some(ref app) = app_weak.upgrade() {
                            ui::overlay::show_overlay(app, db_hotkey.clone(), cb_hotkey.clone());
                        }
                    } else if event.id == history_id {
                        log::info!("History hotkey pressed");
                        if let Some(ref app) = app_weak.upgrade() {
                            ui::history_dialog::show_history(app, db_hotkey.clone(), cb_hotkey.clone());
                        }
                    }
                }
            }

            glib::ControlFlow::Continue
        });

        log::info!("ClipSnap ready");
    });

    // Run the GTK event loop (blocks until quit).
    let exit_code = app.run();
    log::info!("ClipSnap exiting with code {:?}", exit_code);
}
