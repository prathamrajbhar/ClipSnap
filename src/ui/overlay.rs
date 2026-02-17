use crate::{clipboard, database::Database, notifications, screenshot};
use arboard::Clipboard;
use cairo;
use gdk4;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// Internal state tracked during area selection.
struct OverlayState {
    start: Option<(f64, f64)>,
    current: Option<(f64, f64)>,
}

/// Show a transparent overlay, let the user select an area, and capture it.
pub fn show_overlay(app: &gtk4::Application, db: Arc<Mutex<Database>>, clipboard: Arc<Mutex<Clipboard>>) {
    // --- 1. Calculate total bounding box of all monitors with proper scaling ---
    let mut min_x = 0;
    let mut min_y = 0;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut scale_factor = 1.0;

    if let Some(display) = gdk4::Display::default() {
        let monitors = display.monitors();
        for i in 0..monitors.n_items() {
            if let Some(monitor) = monitors.item(i).and_then(|m| m.downcast::<gdk4::Monitor>().ok()) {
                let geom = monitor.geometry();
                scale_factor = monitor.scale_factor() as f64;
                min_x = min_x.min(geom.x());
                min_y = min_y.min(geom.y());
                max_x = max_x.max(geom.x() + geom.width());
                max_y = max_y.max(geom.y() + geom.height());
            }
        }
    }

    let total_width = max_x - min_x;
    let total_height = max_y - min_y;

    let window = gtk4::Window::builder()
        .application(app)
        .decorated(false)
        .title("ClipSnap Overlay")
        .default_width(total_width)
        .default_height(total_height)
        .fullscreened(true)
        .build();
    window.add_css_class("clipsnap-overlay");

    // Enhanced CSS for premium look with smooth animations
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(
        ".clipsnap-overlay {
            background-color: transparent;
            transition: all 200ms ease-in-out;
        }
        .selection-area {
            border-radius: 3px;
            box-shadow: 0 0 20px rgba(0, 0, 0, 0.3);
            transition: all 150ms ease-out;
        }"
    );
    if let Some(display) = gdk4::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    // Premium precision cursor
    let cursor = gdk4::Cursor::from_name("crosshair", None)
        .or_else(|| gdk4::Cursor::from_name("cross", None));
    window.set_cursor(cursor.as_ref());

    // --- 2. State shared between closures ---
    let state = Rc::new(RefCell::new(OverlayState {
        start: None,
        current: None,
    }));

    // --- 3. Drawing area ---
    let drawing_area = gtk4::DrawingArea::new();
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);

    // Draw callback: Dim the screen and "cut out" the selection
    let state_draw = state.clone();
    drawing_area.set_draw_func(move |_da, cr, _w, _h| {
        let st = state_draw.borrow();

        // 1. Premium overlay background with subtle gradient
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.5);
        cr.set_operator(cairo::Operator::Source);
        let _ = cr.paint();

        // 2. "Cut out" the selected region with enhanced visuals
        if let (Some(start), Some(cur)) = (st.start, st.current) {
            let sel_x = start.0.min(cur.0);
            let sel_y = start.1.min(cur.1);
            let sel_w = (start.0 - cur.0).abs();
            let sel_h = (start.1 - cur.1).abs();

            if sel_w > 1.0 && sel_h > 1.0 {
                // Clear the selection area (make it fully transparent)
                cr.set_operator(cairo::Operator::Clear);
                cr.rectangle(sel_x, sel_y, sel_w, sel_h);
                let _ = cr.fill();

                // Premium selection border with subtle glow effect
                cr.set_operator(cairo::Operator::Over);
                
                // Outer glow (shadow)
                cr.set_source_rgba(0.0, 0.5, 1.0, 0.3);
                cr.set_line_width(6.0);
                cr.rectangle(sel_x - 1.0, sel_y - 1.0, sel_w + 2.0, sel_h + 2.0);
                let _ = cr.stroke();
                
                // Main border - modern blue accent
                cr.set_source_rgba(0.2, 0.6, 1.0, 0.9);
                cr.set_line_width(2.0);
                cr.rectangle(sel_x, sel_y, sel_w, sel_h);
                let _ = cr.stroke();

                // Corner indicators for precision
                let corner_size = 8.0;
                cr.set_source_rgba(0.2, 0.6, 1.0, 1.0);
                cr.set_line_width(2.5);
                
                // Top-left corner
                cr.move_to(sel_x, sel_y + corner_size);
                cr.line_to(sel_x, sel_y);
                cr.line_to(sel_x + corner_size, sel_y);
                
                // Top-right corner
                cr.move_to(sel_x + sel_w - corner_size, sel_y);
                cr.line_to(sel_x + sel_w, sel_y);
                cr.line_to(sel_x + sel_w, sel_y + corner_size);
                
                // Bottom-left corner
                cr.move_to(sel_x, sel_y + sel_h - corner_size);
                cr.line_to(sel_x, sel_y + sel_h);
                cr.line_to(sel_x + corner_size, sel_y + sel_h);
                
                // Bottom-right corner
                cr.move_to(sel_x + sel_w - corner_size, sel_y + sel_h);
                cr.line_to(sel_x + sel_w, sel_y + sel_h);
                cr.line_to(sel_x + sel_w, sel_y + sel_h - corner_size);
                
                let _ = cr.stroke();

                // Enhanced dimensions label with background
                let label = format!("{} × {}", sel_w as i32, sel_h as i32);
                cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
                cr.set_font_size(13.0);
                
                let text_extents = cr.text_extents(&label).unwrap();
                let label_x = sel_x + 8.0;
                let label_y = if sel_y > 30.0 { sel_y - 8.0 } else { sel_y + sel_h + 20.0 };
                
                // Background for text
                cr.set_source_rgba(0.0, 0.0, 0.0, 0.8);
                cr.rectangle(label_x - 4.0, label_y - text_extents.height() - 2.0, 
                           text_extents.width() + 8.0, text_extents.height() + 6.0);
                let _ = cr.fill();
                
                // Text
                cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
                cr.move_to(label_x, label_y);
                let _ = cr.show_text(&label);
            }
        }
    });

    // --- 4. Keyboard: ESC cancels ---
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

    // --- 5. Mouse drag for area selection ---
    let drag = gtk4::GestureDrag::new();
    drag.set_button(1); // left mouse button

    let state_begin = state.clone();
    let da_begin = drawing_area.clone();
    drag.connect_drag_begin(move |_gesture, x, y| {
        let mut st = state_begin.borrow_mut();
        st.start = Some((x, y));
        st.current = Some((x, y));
        da_begin.queue_draw();
    });

    let state_update = state.clone();
    let da_update = drawing_area.clone();
    drag.connect_drag_update(move |_gesture, offset_x, offset_y| {
        let mut st = state_update.borrow_mut();
        if let Some(start) = st.start {
            st.current = Some((start.0 + offset_x, start.1 + offset_y));
        }
        da_update.queue_draw();
    });

    // drag-end → capture selection
    let state_end = state.clone();
    let win_end = window.clone();
    drag.connect_drag_end(move |_gesture, offset_x, offset_y| {
        let st = state_end.borrow();

        if let Some(start) = st.start {
            let end_x = start.0 + offset_x;
            let end_y = start.1 + offset_y;

            let sel_w = ((start.0 - end_x).abs() * scale_factor) as u32;
            let sel_h = ((start.1 - end_y).abs() * scale_factor) as u32;

            // Improved coordinate mapping with proper scaling
            let local_x = start.0.min(end_x);
            let local_y = start.1.min(end_y);
            let global_x = ((local_x * scale_factor) as i32) + min_x;
            let global_y = ((local_y * scale_factor) as i32) + min_y;

            // Close overlay immediately
            win_end.close();

            // Minimum selection size guard (account for scaling)
            if sel_w < 10 || sel_h < 10 {
                return;
            }

            // --- Capture the region with proper delay ---
            let db = db.clone();
            let clipboard = clipboard.clone();
            
            // Increased delay to ensure overlay is completely gone (300ms for safety)
            // This prevents any overlay artifacts from appearing in screenshots
            glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
                match screenshot::capture_region(global_x, global_y, sel_w, sel_h) {
                    Ok((raw_bgra, width, height)) => {
                        let rgba = screenshot::bgra_to_rgba(&raw_bgra);
                        
                        // Encode to PNG
                        match screenshot::encode_png(&rgba, width, height) {
                            Ok(png_bytes) => {
                                let thumb = screenshot::create_thumbnail(&png_bytes, 150).unwrap_or_default();

                                // Copy to shared clipboard
                                if let Ok(mut cb) = clipboard.lock() {
                                    if let Err(e) = clipboard::set_clipboard_image(&mut cb, &rgba, width as usize, height as usize) {
                                        log::error!("Failed to copy to clipboard: {}", e);
                                        notifications::notify_screenshot_error("Clipboard copy failed");
                                    }
                                }

                                // Store in database
                                if let Ok(db) = db.lock() {
                                    if let Err(e) = db.insert_image(&png_bytes, &thumb) {
                                        log::error!("Failed to save screenshot: {}", e);
                                    }
                                }

                                let tmp_path = std::env::temp_dir().join("clipsnap_last.png");
                                let _ = std::fs::write(&tmp_path, &png_bytes);
                                notifications::notify_screenshot_success(&tmp_path);
                            }
                            Err(e) => {
                                log::error!("PNG encoding failed: {}", e);
                                notifications::notify_screenshot_error("PNG encoding failed");
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Region capture failed: {}", e);
                        notifications::notify_screenshot_error(&format!("Capture failed: {}", e));
                    }
                }
            });
        }
    });

    drawing_area.add_controller(drag);
    window.set_child(Some(&drawing_area));
    window.present();
}
