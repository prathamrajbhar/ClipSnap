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
    // --- 1. Calculate total bounding box of all monitors ---
    let mut min_x = 0;
    let mut min_y = 0;
    let mut max_x = 0;
    let mut max_y = 0;

    if let Some(display) = gdk4::Display::default() {
        let monitors = display.monitors();
        for i in 0..monitors.n_items() {
            if let Some(monitor) = monitors.item(i).and_then(|m| m.downcast::<gdk4::Monitor>().ok()) {
                let geom = monitor.geometry();
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
        .build();
    window.add_css_class("overlay-window");

    // Position window at the top-left of the total bounding box
    // Note: GTK4 window positioning is sometimes compositor-dependent, 
    // but setting it as a fixed size overlay is the goal.
    // For many X11 compositors, this works well.

    // In GTK4, to make the window truly transparent (so we can see the desktop),
    // scoping it to .overlay-window.
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(".overlay-window { background-color: transparent; }");
    if let Some(display) = gdk4::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    // Crosshair cursor
    let cursor = gdk4::Cursor::from_name("crosshair", None);
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
    drawing_area.set_draw_func(move |da, cr, _w, _h| {
        let st = state_draw.borrow();

        // 1. Fill the entire area with semi-transparent black (dimmed look)
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.4);
        cr.set_operator(cairo::Operator::Source);
        let _ = cr.paint();

        // 2. "Cut out" the selected region if it exists
        if let (Some(start), Some(cur)) = (st.start, st.current) {
            let sel_x = start.0.min(cur.0);
            let sel_y = start.1.min(cur.1);
            let sel_w = (start.0 - cur.0).abs();
            let sel_h = (start.1 - cur.1).abs();

            if sel_w > 1.0 && sel_h > 1.0 {
                // Fetch theme color (e.g., Ubuntu orange)
                let context = da.style_context();
                // In GTK4, we can try to look up a named color or use the foreground color as a fallback
                // but usually the accent/selection color is more appropriate.
                // We'll use a standard accent color if possible, falling back to blue only as a last resort.
                let accent_color = context.lookup_color("accent_color")
                    .or_else(|| context.lookup_color("theme_selected_bg_color"))
                    .unwrap_or_else(|| gdk4::RGBA::new(0.0, 0.478, 1.0, 1.0));

                // Clear the selection area (make it fully transparent)
                cr.set_operator(cairo::Operator::Clear);
                cr.rectangle(sel_x, sel_y, sel_w, sel_h);
                let _ = cr.fill();

                // Draw the theme-colored border around the selection
                cr.set_operator(cairo::Operator::Over);
                cr.set_source_rgba(
                    accent_color.red() as f64,
                    accent_color.green() as f64,
                    accent_color.blue() as f64,
                    1.0
                );
                cr.set_line_width(2.5);
                cr.rectangle(sel_x, sel_y, sel_w, sel_h);
                let _ = cr.stroke();

                // Dimensions label
                let label = format!("{} × {}", sel_w as i32, sel_h as i32);
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.9);
                cr.set_font_size(14.0);
                let label_y = if sel_y > 20.0 { sel_y - 6.0 } else { sel_y + 18.0 };
                cr.move_to(sel_x + 4.0, label_y);
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

            let sel_w = (start.0 - end_x).abs() as u32;
            let sel_h = (start.1 - end_y).abs() as u32;

            // Map local to global coordinates
            let global_x = (start.0.min(end_x) as i32) + min_x;
            let global_y = (start.1.min(end_y) as i32) + min_y;

            // Close overlay immediately
            win_end.close();

            // Minimum selection size guard
            if sel_w < 5 || sel_h < 5 {
                return;
            }

            // --- Capture the region exactly as it is now ---
            let db = db.clone();
            let clipboard = clipboard.clone();
            
            // Note: We move the actual capture/processing to a closure to run after the window is hidden
            // Wait 100ms for the window to actually vanish from the compositor's view
            glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
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
