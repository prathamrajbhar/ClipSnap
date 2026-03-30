use crate::screenshot::{self, CapturedImage};
use crate::WorkerTask;
use cairo;
use gdk4;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Sender;

/// Internal state tracked during area selection across all monitors.
struct OverlayState {
    start: Option<(f64, f64)>,        // Global logical coordinates
    current: Option<(f64, f64)>,      // Global logical coordinates
    frozen_image: CapturedImage,
    windows: Vec<gtk4::Window>,
    drawing_areas: Vec<gtk4::DrawingArea>,
}

impl OverlayState {
    fn queue_redraw(&self) {
        for da in &self.drawing_areas {
            da.queue_draw();
        }
    }

    fn close_all(&self) {
        for win in &self.windows {
            win.close();
        }
    }
}

/// Show synchronized overlays on all connected monitors.
pub fn show_overlay(app: &gtk4::Application, worker_tx: Sender<WorkerTask>, frozen: CapturedImage) {
    let display = gdk4::Display::default().expect("No display found");
    let monitors = display.monitors();
    
    // 1. Calculate Virtual Desktop Bounds in Logical Pixels
    let mut min_x_log = i32::MAX;
    let mut min_y_log = i32::MAX;
    for i in 0..monitors.n_items() {
        if let Some(monitor) = monitors.item(i).and_then(|obj| obj.downcast::<gdk4::Monitor>().ok()) {
            let geometry = monitor.geometry();
            min_x_log = min_x_log.min(geometry.x());
            min_y_log = min_y_log.min(geometry.y());
        }
    }

    // 2. Initialise Shared State
    let state = Rc::new(RefCell::new(OverlayState {
        start: None,
        current: None,
        frozen_image: frozen.clone(),
        windows: Vec::new(),
        drawing_areas: Vec::new(),
    }));

    // Create a shared surface from the frozen BGRA data (device pixels)
    let frozen_surface = {
        let mut surface = cairo::ImageSurface::create(
            cairo::Format::ARgb32, 
            frozen.width as i32, 
            frozen.height as i32
        ).expect("Failed to create surface");
        {
            let mut data = surface.data().expect("Failed to get surface data");
            data.copy_from_slice(&frozen.data);
        }
        Rc::new(surface)
    };

    // CSS for all overlay windows
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(".clipsnap-overlay { background: transparent; }");
    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // 3. Create a Window for each Monitor
    for i in 0..monitors.n_items() {
        let monitor = monitors.item(i).and_then(|obj| obj.downcast::<gdk4::Monitor>().ok()).unwrap();
        let geometry = monitor.geometry(); // Logical geometry of this monitor
        
        // Window setup
        let window = gtk4::Window::builder()
            .application(app)
            .decorated(false)
            .title(&format!("ClipSnap Overlay - Monitor {}", i))
            .default_width(geometry.width())
            .default_height(geometry.height())
            .can_focus(true)
            .build();
        
        window.add_css_class("clipsnap-overlay");
        window.set_cursor(gdk4::Cursor::from_name("crosshair", None).as_ref());
        
        // Fullscreen on this specific monitor
        window.fullscreen_on_monitor(&monitor);

        let drawing_area = gtk4::DrawingArea::new();
        drawing_area.set_hexpand(true);
        drawing_area.set_vexpand(true);

        // Drawing Function
        let state_draw = state.clone();
        let frozen_surf_draw = frozen_surface.clone();
        let mon_offset_x = geometry.x() as f64;
        let mon_offset_y = geometry.y() as f64;
        let global_min_x = min_x_log as f64;
        let global_min_y = min_y_log as f64;

        drawing_area.set_draw_func(move |da, cr, _w, _h| {
            let st = state_draw.borrow();
            let scale = da.scale_factor() as f64;

            // --- 1. Background image (frozen) ---
            cr.save().expect("Save failed");
            
            // Device pixel offset in the capture buffer for this monitor
            let dev_off_x = (mon_offset_x - global_min_x) * scale;
            let dev_off_y = (mon_offset_y - global_min_y) * scale;
            
            // Scale cr so that 1.0 unit = 1 logical pixel
            // But the surface is in device pixels, so we draw it at its offset
            cr.scale(1.0 / scale, 1.0 / scale);
            cr.set_source_surface(&*frozen_surf_draw, -dev_off_x, -dev_off_y).expect("Draw surface failed");
            let _ = cr.paint();
            cr.restore().expect("Restore failed");

            // --- 2. Dim the monitor ---
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.4);
            let _ = cr.paint();

            // --- 3. Highlight selection ---
            if let (Some(start), Some(cur)) = (st.start, st.current) {
                let g_x = start.0.min(cur.0); 
                let g_y = start.1.min(cur.1);
                let g_w = (start.0 - cur.0).abs();
                let g_h = (start.1 - cur.1).abs();

                if g_w > 1.0 && g_h > 1.0 {
                    // Translate global logical bounds to local logical bounds for this window
                    let l_x = g_x - mon_offset_x;
                    let l_y = g_y - mon_offset_y;

                    // Reveal "frozen" background in the selection area
                    cr.save().expect("Save highlight failed");
                    cr.set_operator(cairo::Operator::Source);
                    cr.scale(1.0 / scale, 1.0 / scale);
                    cr.set_source_surface(&*frozen_surf_draw, -dev_off_x, -dev_off_y).expect("Draw source failed");
                    cr.rectangle(l_x * scale, l_y * scale, g_w * scale, g_h * scale);
                    let _ = cr.fill();
                    cr.restore().expect("Restore highlight failed");

                    // Premium border (logical pixels)
                    cr.set_operator(cairo::Operator::Over);
                    cr.set_source_rgba(0.0, 0.6, 1.0, 0.9);
                    cr.set_line_width(2.0);
                    cr.rectangle(l_x, l_y, g_w, g_h);
                    let _ = cr.stroke();
                    
                    // Show dimensions (using the window's scale factor)
                    let label = format!("{} × {}", (g_w * scale) as i32, (g_h * scale) as i32);
                    cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
                    cr.set_font_size(14.0);
                    // Position label relative to LOCAL coordinates
                    cr.move_to(l_x + 5.0, if l_y > 20.0 { l_y - 5.0 } else { l_y + g_h + 20.0 });
                    let _ = cr.show_text(&label);
                }
            }
        });

        // Gestures for synchronization
        let drag = gtk4::GestureDrag::new();
        let state_drag = state.clone();
        drag.connect_drag_begin(move |_, x, y| {
            let mut st = state_drag.borrow_mut();
            // Convert locallogical offset to global logical offset
            st.start = Some((x + mon_offset_x, y + mon_offset_y));
            st.current = Some((x + mon_offset_x, y + mon_offset_y));
            st.queue_redraw();
        });

        let state_update = state.clone();
        drag.connect_drag_update(move |_, ox, oy| {
            let mut st = state_update.borrow_mut();
            if let Some(start) = st.start {
                st.current = Some((start.0 + ox, start.1 + oy));
                st.queue_redraw();
            }
        });

        let state_end = state.clone();
        let worker_tx_end = worker_tx.clone();
        let scale_end = monitor.scale_factor();
        drag.connect_drag_end(move |_, ox, oy| {
            let st = state_end.borrow();
            // All coordinates here are in GLOBAL LOGICAL space
            if let Some(start) = st.start {
                let g_x_log = start.0.min(start.0 + ox);
                let g_y_log = start.1.min(start.1 + oy);
                let g_w_log = ox.abs();
                let g_h_log = oy.abs();

                // Convert GLOBAL LOGICAL to GLOBAL DEVICE (capture buffer space)
                // Note: This simplified scale assumes the selection started on this monitor's scale
                // For perfect mixed DPI, we'd need to map based on the specific pixel clusters
                let x_dev = ((g_x_log - global_min_x as f64) * scale_end as f64) as i32;
                let y_dev = ((g_y_log - global_min_y as f64) * scale_end as f64) as i32;
                let w_dev = (g_w_log * scale_end as f64) as u32;
                let h_dev = (g_h_log * scale_end as f64) as u32;

                if w_dev > 5 && h_dev > 5 {
                    log::info!("Overlay: Cropping {}x{} at global dev {},{}", w_dev, h_dev, x_dev, y_dev);
                    match screenshot::crop_bgra_to_rgba(
                        &st.frozen_image.data,
                        st.frozen_image.width,
                        st.frozen_image.height,
                        x_dev,
                        y_dev,
                        w_dev,
                        h_dev,
                    ) {
                        Ok((rgba, w, h)) => {
                            let _ = worker_tx_end.send(WorkerTask::ProcessScreenshot { rgba_pixels: rgba, width: w, height: h });
                        }
                        Err(e) => log::error!("Crop failed: {}", e),
                    }
                }
            }
            st.close_all();
        });

        // Key controller (ESC)
        let state_key = state.clone();
        let key_ctl = gtk4::EventControllerKey::new();
        key_ctl.connect_key_pressed(move |_, key, _code, _mods| {
            if key == gdk4::Key::Escape {
                state_key.borrow().close_all();
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });

        drawing_area.add_controller(drag);
        window.add_controller(key_ctl);
        window.set_child(Some(&drawing_area));
        window.present();

        // Track windows/drawing areas for sync
        {
            let mut st = state.borrow_mut();
            st.windows.push(window);
            st.drawing_areas.push(drawing_area);
        }
    }
}
