use crate::models::{ContentType, HistoryEntry};
use crate::{clipboard, database::Database, notifications};
use arboard::Clipboard;
use enigo::{Enigo, Key};
use gdk4;
use gdk_pixbuf;
use glib;
use gtk4::prelude::*;
use std::sync::{Arc, Mutex};

/// Show the clipboard history dialog.
pub fn show_history(app: &gtk4::Application, db: Arc<Mutex<Database>>, clipboard: Arc<Mutex<Clipboard>>) {
    let window = gtk4::Window::builder()
        .application(app)
        .title("ClipSnap History")
        .default_width(600)
        .default_height(500)
        .resizable(true)
        .build();
    window.add_css_class("history-window");

    // ── Layout ──────────────────
    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    vbox.set_margin_start(16);
    vbox.set_margin_end(16);
    vbox.set_margin_top(16);
    vbox.set_margin_bottom(16);

    // Header with Title and Search
    let header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    
    let title_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    let title_label = gtk4::Label::builder()
        .label("ClipSnap History")
        .halign(gtk4::Align::Start)
        .build();
    title_label.add_css_class("title-1");
    title_box.append(&title_label);
    header_box.append(&title_box);

    let search_entry = gtk4::SearchEntry::builder()
        .placeholder_text("Search clipboard…")
        .hexpand(true)
        .build();
    header_box.append(&search_entry);

    let clear_button = gtk4::Button::builder()
        .icon_name("edit-clear-all-symbolic")
        .tooltip_text("Clear current history")
        .css_classes(["flat"])
        .build();
    header_box.append(&clear_button);
    
    vbox.append(&header_box);

    // --- Notebook (Tabs) ---
    let notebook = gtk4::Notebook::new();
    notebook.set_vexpand(true);
    
    // 1. Text Tab
    let text_scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .build();
    let text_flow = gtk4::FlowBox::builder()
        .max_children_per_line(1)
        .selection_mode(gtk4::SelectionMode::None)
        .row_spacing(8)
        .margin_start(8)
        .margin_end(8)
        .margin_top(8)
        .margin_bottom(8)
        .build();
    text_scrolled.set_child(Some(&text_flow));
    
    let text_tab_label = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    text_tab_label.append(&gtk4::Image::from_icon_name("edit-copy-symbolic"));
    text_tab_label.append(&gtk4::Label::new(Some("Text")));
    notebook.append_page(&text_scrolled, Some(&text_tab_label));

    // 2. Images Tab
    let img_scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .build();
    let img_flow = gtk4::FlowBox::builder()
        .max_children_per_line(4)
        .min_children_per_line(2)
        .selection_mode(gtk4::SelectionMode::None)
        .row_spacing(10)
        .column_spacing(10)
        .margin_start(10)
        .margin_end(10)
        .margin_top(10)
        .margin_bottom(10)
        .build();
    img_scrolled.set_child(Some(&img_flow));
    
    let img_tab_label = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    img_tab_label.append(&gtk4::Image::from_icon_name("image-x-generic-symbolic"));
    img_tab_label.append(&gtk4::Label::new(Some("Images")));
    notebook.append_page(&img_scrolled, Some(&img_tab_label));

    vbox.append(&notebook);

    // Footer Info
    let footer_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    let status_label = gtk4::Label::builder()
        .xalign(0.0)
        .css_classes(["dim-label"])
        .build();
    footer_box.append(&status_label);
    
    let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    footer_box.append(&spacer);
    
    let hint = gtk4::Label::builder()
        .label("Click to Copy & Paste   •   Esc to Close")
        .xalign(1.0)
        .css_classes(["dim-label"])
        .build();
    footer_box.append(&hint);
    vbox.append(&footer_box);

    window.set_child(Some(&vbox));

    // ── Load entries ────────────
    let db_load = db.clone();
    let win_ref = window.clone();
    let cb_ref = clipboard.clone();

    let populate = {
        let text_flow = text_flow.clone();
        let img_flow = img_flow.clone();
        let status_label = status_label.clone();
        
        move |query: &str| {
            // Clear both
            while let Some(child) = text_flow.first_child() { text_flow.remove(&child); }
            while let Some(child) = img_flow.first_child() { img_flow.remove(&child); }

            if let Ok(db) = db_load.lock() {
                // Populate Text
                let text_entries = if query.is_empty() {
                    db.get_recent_entries_by_type(50, ContentType::Text).unwrap_or_default()
                } else {
                    db.search_text(query).unwrap_or_default()
                };
                for (i, entry) in text_entries.iter().enumerate() {
                    if i > 0 {
                        let sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);
                        sep.add_css_class("divider");
                        text_flow.insert(&sep, -1);
                    }
                    text_flow.insert(&build_entry_widget(entry, &db_load, &win_ref, &cb_ref), -1);
                }

                // Populate Images
                let img_entries = if query.is_empty() {
                    db.get_recent_entries_by_type(50, ContentType::Image).unwrap_or_default()
                } else {
                    Vec::new()
                };
                for entry in &img_entries {
                    img_flow.insert(&build_entry_widget(entry, &db_load, &win_ref, &cb_ref), -1);
                }

                status_label.set_text(&format!("{} text, {} images", text_entries.len(), img_entries.len()));
            }
        }
    };

    // --- Clear All Logic ---
    let db_clear = db.clone();
    let notebook_clear = notebook.clone();
    let text_flow_clear = text_flow.clone();
    let img_flow_clear = img_flow.clone();
    let _win_clear = window.clone();
    
    clear_button.connect_clicked(move |_| {
        let current_page = notebook_clear.current_page();
        let content_type = if current_page == Some(0) { Some(ContentType::Text) } else { Some(ContentType::Image) };
        let active_flow = if current_page == Some(0) { text_flow_clear.clone() } else { img_flow_clear.clone() };

        // 1. Start smooth visual removal (fade out)
        active_flow.add_css_class("fade-out");

        // 2. Perform DB operation in background
        let db_async = db_clear.clone();
        
        // Wait for animation to finish (300ms) before clearing the UI
        glib::timeout_add_local_once(std::time::Duration::from_millis(320), move || {
            // UI Clear happens on main thread
            active_flow.remove_css_class("fade-out");
            
            // Manually clear the flowbox to be safe and instant
            while let Some(child) = active_flow.first_child() {
                active_flow.remove(&child);
            }

            // Background thread handles DB work
            std::thread::spawn(move || {
                if let Ok(db) = db_async.lock() {
                    let _ = db.clear_history(content_type);
                }
            });
        });
    });

    // CSS for divider and animations
    let provider_extra = gtk4::CssProvider::new();
    provider_extra.load_from_data("
        .divider { margin: 4px 0; opacity: 0.1; }
        .fade-out { 
            opacity: 0; 
            transition: opacity 300ms ease-out;
        }
    ");
    if let Some(display) = gdk4::Display::default() {
        gtk4::style_context_add_provider_for_display(&display, &provider_extra, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
    }

    // Initial population
    populate("");

    // ── Search ──────────────────
    let populate_search = populate.clone();
    search_entry.connect_search_changed(move |entry| {
        populate_search(&entry.text());
    });

    // ── Styles ──────────────────
    let provider = gtk4::CssProvider::new();
    provider.load_from_data("
        .history-window { background-color: @theme_bg_color; }
        .title-1 { font-size: 24px; font-weight: 800; margin-bottom: 4px; }
        .dim-label { opacity: 0.6; font-size: 13px; }
        
        card {
            background-color: @theme_bg_color;
            border-radius: 12px;
            padding: 16px;
            border: 1px solid alpha(@theme_fg_color, 0.1);
            transition: all 200ms ease;
        }
        card.text-card {
            min-width: 580px;
        }
        card:hover {
            background-color: alpha(@theme_fg_color, 0.05);
            border-color: alpha(@theme_fg_color, 0.2);
            box-shadow: 0 4px 12px alpha(black, 0.1);
        }

        notebook header {
            background-color: transparent;
            border: none;
            padding: 0 8px;
        }
        notebook tab {
            padding: 8px 16px;
            border-radius: 8px 8px 0 0;
            font-weight: 600;
        }
        notebook tab:checked {
            border-bottom: 2px solid @accent_color;
        }
    ");
    if let Some(display) = gdk4::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    // ── Keyboard: Esc closes ───
    let win_key = window.clone();
    let key_ctl = gtk4::EventControllerKey::new();
    key_ctl.connect_key_pressed(move |_, key, _code, _mods| {
        if key == gdk4::Key::Escape {
            win_key.close();
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    window.add_controller(key_ctl);

    window.present();
}

/// Build a GTK widget for a single history entry.
fn build_entry_widget(
    entry: &HistoryEntry,
    db: &Arc<Mutex<Database>>,
    window: &gtk4::Window,
    clipboard: &Arc<Mutex<Clipboard>>,
) -> gtk4::Widget {
    let card = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    card.add_css_class("card");
    card.set_cursor(Some(&gdk4::Cursor::from_name("pointer", None).unwrap()));

    match entry.content_type {
        ContentType::Image => {
            card.set_size_request(150, -1);
            // Thumbnail
            if let Some(thumb_bytes) = &entry.thumbnail {
                if let Some(pixbuf) = load_pixbuf_from_png(thumb_bytes) {
                    let texture = gdk4::Texture::for_pixbuf(&pixbuf);
                    let picture = gtk4::Picture::for_paintable(&texture);
                    picture.set_size_request(150, 110);
                    picture.set_can_shrink(false);
                    card.append(&picture);
                }
            }
        }
        ContentType::Text => {
            card.add_css_class("text-card");
            card.set_hexpand(true);
            let text = entry.text_content.as_deref().unwrap_or("");
            let preview = if text.len() > 150 {
                format!("{}…", &text[..text.char_indices().nth(150).map(|(i, _)| i).unwrap_or(text.len())])
            } else {
                text.to_string()
            };

            let label = gtk4::Label::new(Some(&preview));
            label.set_wrap(true);
            label.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
            label.set_xalign(0.0);
            label.set_max_width_chars(50);
            label.set_lines(3);
            label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            card.append(&label);
        }
    }

    // Card Footer (Type + Time)
    let footer = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    footer.add_css_class("dim-label");

    let ago = format_timestamp(entry.created_at);
    let ts_label = gtk4::Label::new(Some(&ago));
    footer.append(&ts_label);

    let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    footer.append(&spacer);

    if entry.content_type == ContentType::Image {
        let size_str = format_size(entry.file_size);
        let size_label = gtk4::Label::new(Some(&size_str));
        footer.append(&size_label);
    } else {
        let icon = gtk4::Image::from_icon_name("text-x-generic-symbolic");
        footer.append(&icon);
    }

    card.append(&footer);

    // Click handler – restore to clipboard
    let entry_id = entry.id;
    let content_type = entry.content_type.clone();
    let db_click = db.clone();
    let win_click = window.clone();

    // We store text/image data needed for the click handler
    let text_content = entry.text_content.clone();
    let image_data = entry.image_data.clone();
    let cb_click = clipboard.clone();

    let gesture = gtk4::GestureClick::new();
    gesture.connect_released(move |_gesture, _n, _x, _y| {
        restore_entry_to_clipboard(
            entry_id,
            &content_type,
            text_content.as_deref(),
            image_data.as_deref(),
            &db_click,
            &cb_click,
        );
        win_click.close();

        // --- Auto-Paste Simulation ---
        // Wait a short duration for the window to hide and focus to return to previous app.
        glib::timeout_add_local_once(std::time::Duration::from_millis(150), move || {
            use enigo::{Direction, Keyboard, Settings};
            if let Ok(mut enigo) = Enigo::new(&Settings::default()) {
                let _ = enigo.key(Key::Control, Direction::Press);
                let _ = enigo.key(Key::Unicode('v'), Direction::Click);
                let _ = enigo.key(Key::Control, Direction::Release);
            }
        });
    });
    card.add_controller(gesture);

    card.upcast()
}

/// Restore a history entry to the clipboard.
fn restore_entry_to_clipboard(
    id: i64,
    content_type: &ContentType,
    text_content: Option<&str>,
    image_data: Option<&[u8]>,
    db: &Arc<Mutex<Database>>,
    clipboard: &Arc<Mutex<Clipboard>>,
) {
    match content_type {
        ContentType::Text => {
            if let Some(text) = text_content {
                if let Ok(mut cb) = clipboard.lock() {
                    if let Err(e) = clipboard::set_clipboard_text(&mut cb, text) {
                        log::error!("Failed to restore text: {}", e);
                        return;
                    }
                }
            }
        }
        ContentType::Image => {
            // image_data might be None in the list (we don't always load full data)
            let img_bytes = if let Some(data) = image_data {
                data.to_vec()
            } else {
                // Fetch full data from DB
                let db = match db.lock() {
                    Ok(db) => db,
                    Err(_) => return,
                };
                match db.get_entry(id) {
                    Ok(Some(entry)) => match entry.image_data {
                        Some(d) => d,
                        None => return,
                    },
                    _ => return,
                }
            };

            // Decode PNG to RGBA
            match image::load_from_memory(&img_bytes) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let w = rgba.width() as usize;
                    let h = rgba.height() as usize;
                    if let Ok(mut cb) = clipboard.lock() {
                        if let Err(e) = clipboard::set_clipboard_image(&mut cb, rgba.as_raw(), w, h) {
                            log::error!("Failed to restore image: {}", e);
                            return;
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to decode image: {}", e);
                    return;
                }
            }
        }
    }

    notifications::notify_clipboard_restored();
}

/// Load a `gdk_pixbuf::Pixbuf` from PNG bytes.
fn load_pixbuf_from_png(png_bytes: &[u8]) -> Option<gdk_pixbuf::Pixbuf> {
    let stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from(png_bytes));
    gdk_pixbuf::Pixbuf::from_stream(&stream, None::<&gio::Cancellable>).ok()
}

/// Format a file size in human-friendly form.
fn format_size(bytes: i64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Format a Unix timestamp as a human-friendly "time ago" string.
fn format_timestamp(ts: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let diff = now - ts;

    if diff < 60 {
        "just now".into()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}
