use crate::screenshot::{self, CapturedImage};
use crate::WorkerTask;
use cairo;
use gdk4;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Sender;

/// Internal state tracked during area selection.
struct OverlayState {
    start: Option<(f64, f64)>,
    current: Option<(f64, f64)>,
    frozen_image: CapturedImage,
}

/// Show the "frozen" screen overlay, let the user select a region, and crop it.
pub fn show_overlay(app: &gtk4::Application, worker_tx: Sender<WorkerTask>, frozen: CapturedImage) {
    let width = frozen.width;
    let height = frozen.height;

    let window = gtk4::Window::builder()
        .application(app)
        .decorated(false)
        .title("ClipSnap Overlay")
        .default_width(width as i32)
        .default_height(height as i32)
        .fullscreened(true)
        .build();
    
    // Transparent background for the window
    window.add_css_class("clipsnap-overlay");

    let provider = gtk4::CssProvider::new();
    provider.load_from_data(".clipsnap-overlay { background: transparent; }");
    if let Some(display) = gdk4::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    window.set_cursor(gdk4::Cursor::from_name("crosshair", None).as_ref());

    let state = Rc::new(RefCell::new(OverlayState {
        start: None,
        current: None,
        frozen_image: frozen.clone(),
    }));

    let drawing_area = gtk4::DrawingArea::new();
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);

    let frozen_surface = {
        let mut surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width as i32, height as i32).expect("Failed to create surface");
        {
            let mut data = surface.data().expect("Failed to get surface data");
            data.copy_from_slice(&frozen.data);
        }
        surface
    };

    let state_draw = state.clone();
    drawing_area.set_draw_func(move |_da, cr, _w, _h| {
        let st = state_draw.borrow();

        // 1. Draw the "frozen" background
        cr.set_source_surface(&frozen_surface, 0.0, 0.0).expect("Draw surface failed");
        let _ = cr.paint();

        // 2. Dim the entire screen slightly
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.3);
        let _ = cr.paint();

        // 3. Cut out and highlight the selected region
        if let (Some(start), Some(cur)) = (st.start, st.current) {
            let x = start.0.min(cur.0);
            let y = start.1.min(cur.1);
            let w = (start.0 - cur.0).abs();
            let h = (start.1 - cur.1).abs();

            if w > 1.0 && h > 1.0 {
                // Clear the selection area (reveal the "frozen" background behind the dimming layer)
                cr.set_operator(cairo::Operator::Source);
                cr.set_source_surface(&frozen_surface, 0.0, 0.0).expect("Draw source failed");
                cr.rectangle(x, y, w, h);
                let _ = cr.fill();

                // Draw premium border
                cr.set_operator(cairo::Operator::Over);
                cr.set_source_rgba(0.2, 0.6, 1.0, 0.9); // Modern blue
                cr.set_line_width(2.0);
                cr.rectangle(x, y, w, h);
                let _ = cr.stroke();
                
                // Show dimensions
                let label = format!("{} × {}", w as i32, h as i32);
                cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
                cr.set_font_size(14.0);
                cr.move_to(x + 5.0, if y > 20.0 { y - 5.0 } else { y + h + 20.0 });
                let _ = cr.show_text(&label);
            }
        }
    });

    // Keyboard ESC to cancel
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

    // Mouse drag for selection
    let drag = gtk4::GestureDrag::new();
    let state_drag = state.clone();
    let da_drag = drawing_area.clone();
    
    drag.connect_drag_begin(move |_, x, y| {
        let mut st = state_drag.borrow_mut();
        st.start = Some((x, y));
        st.current = Some((x, y));
        da_drag.queue_draw();
    });

    let state_update = state.clone();
    let da_update = drawing_area.clone();
    drag.connect_drag_update(move |_, ox, oy| {
        let mut st = state_update.borrow_mut();
        if let Some(start) = st.start {
            st.current = Some((start.0 + ox, start.1 + oy));
        }
        da_update.queue_draw();
    });

    let win_end = window.clone();
    let state_end = state.clone();
    drag.connect_drag_end(move |_, ox, oy| {
        let st = state_end.borrow();
        if let Some(start) = st.start {
            let sel_x = start.0.min(start.0 + ox) as i32;
            let sel_y = start.1.min(start.1 + oy) as i32;
            let sel_w = (ox.abs()) as u32;
            let sel_h = (oy.abs()) as u32;

            if sel_w > 10 && sel_h > 10 {
                // Crop from memory and send to worker
                match screenshot::crop_bgra_to_rgba(
                    &st.frozen_image.data,
                    st.frozen_image.width,
                    st.frozen_image.height,
                    sel_x,
                    sel_y,
                    sel_w,
                    sel_h,
                ) {
                    Ok((rgba, w, h)) => {
                        let _ = worker_tx.send(WorkerTask::ProcessScreenshot { rgba_pixels: rgba, width: w, height: h });
                    }
                    Err(e) => log::error!("Crop failed: {}", e),
                }
            }
        }
        win_end.close();
    });

    drawing_area.add_controller(drag);
    window.set_child(Some(&drawing_area));
    window.present();
}
