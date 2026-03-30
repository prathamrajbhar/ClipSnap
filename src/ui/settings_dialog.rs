use crate::config::Config;
use gdk4::ModifierType;
use gtk4::prelude::*;

/// Show the settings dialog.
pub fn show_settings(app: gtk4::Application) {
    let config = match Config::load_or_create_default() {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to load config for settings: {}", e);
            return;
        }
    };

    let window = gtk4::Window::builder()
        .application(&app)
        .title("ClipSnap Settings")
        .default_width(550)
        .default_height(600)
        .modal(true)
        .resizable(false)
        .build();

    let main_vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 18);
    main_vbox.set_margin_start(24);
    main_vbox.set_margin_end(24);
    main_vbox.set_margin_top(24);
    main_vbox.set_margin_bottom(24);

    let header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    
    if let Some(logo_pb) = crate::ui::load_logo_pixbuf(32) {
        let logo_img = gtk4::Image::from_pixbuf(Some(&logo_pb));
        logo_img.add_css_class("app-logo");
        header_box.append(&logo_img);
    }

    let title_label = gtk4::Label::builder()
        .label("Settings")
        .halign(gtk4::Align::Start)
        .css_classes(["title-1"])
        .build();
    header_box.append(&title_label);
    main_vbox.append(&header_box);

    let notebook = gtk4::Notebook::builder()
        .tab_pos(gtk4::PositionType::Top)
        .vexpand(true)
        .build();

    // --- Shortcuts Tab ---
    let shortcuts_grid = create_settings_grid();
    let entry_screenshot = create_entry(&config.shortcuts.screenshot);
    setup_shortcut_recording(&entry_screenshot);
    
    let entry_history = create_entry(&config.shortcuts.history);
    setup_shortcut_recording(&entry_history);
    
    let entry_extract = create_entry(&config.shortcuts.extract_text);
    setup_shortcut_recording(&entry_extract);
    
    let entry_qr = create_entry(&config.shortcuts.decode_qr);
    setup_shortcut_recording(&entry_qr);

    add_row(&shortcuts_grid, "Screenshot Region", &entry_screenshot, 0);
    add_row(&shortcuts_grid, "Open History", &entry_history, 1);
    add_row(&shortcuts_grid, "Extract Text (AI)", &entry_extract, 2);
    add_row(&shortcuts_grid, "Decode QR Code", &entry_qr, 3);
    
    notebook.append_page(&shortcuts_grid, Some(&gtk4::Label::new(Some("Shortcuts"))));

    // --- Capture Tab ---
    let capture_grid = create_settings_grid();
    let entry_format = create_entry(&config.capture.format);
    let adj_quality = gtk4::Adjustment::new(config.capture.quality as f64, 10.0, 100.0, 1.0, 5.0, 0.0);
    let spin_quality = gtk4::SpinButton::new(Some(&adj_quality), 1.0, 0);
    spin_quality.set_halign(gtk4::Align::End);

    let switch_dimensions = create_switch(config.capture.show_dimensions);

    add_row(&capture_grid, "Image Format", &entry_format, 0);
    add_row(&capture_grid, "JPEG Quality (10-100)", &spin_quality, 1);
    add_row(&capture_grid, "Show Area Dimensions", &switch_dimensions, 2);

    notebook.append_page(&capture_grid, Some(&gtk4::Label::new(Some("Capture"))));

    // --- History Tab ---
    let history_grid = create_settings_grid();
    let adj_max_entries = gtk4::Adjustment::new(config.history.max_entries as f64, 10.0, 1000.0, 10.0, 50.0, 0.0);
    let spin_max_entries = gtk4::SpinButton::new(Some(&adj_max_entries), 1.0, 0);
    spin_max_entries.set_halign(gtk4::Align::End);

    let adj_retention = gtk4::Adjustment::new(config.history.retention_days as f64, 1.0, 365.0, 1.0, 7.0, 0.0);
    let spin_retention = gtk4::SpinButton::new(Some(&adj_retention), 1.0, 0);
    spin_retention.set_halign(gtk4::Align::End);

    let switch_cleanup = create_switch(config.history.auto_cleanup);

    add_row(&history_grid, "Max History Entries", &spin_max_entries, 0);
    add_row(&history_grid, "Retention Days", &spin_retention, 1);
    add_row(&history_grid, "Auto-Cleanup Old Data", &switch_cleanup, 2);

    notebook.append_page(&history_grid, Some(&gtk4::Label::new(Some("History"))));

    // --- UI Tab ---
    let ui_grid = create_settings_grid();
    let adj_thumb = gtk4::Adjustment::new(config.ui.thumbnail_size as f64, 64.0, 512.0, 16.0, 64.0, 0.0);
    let spin_thumb = gtk4::SpinButton::new(Some(&adj_thumb), 1.0, 0);
    spin_thumb.set_halign(gtk4::Align::End);

    let adj_notif = gtk4::Adjustment::new(config.ui.notification_duration as f64, 1.0, 10.0, 1.0, 2.0, 0.0);
    let spin_notif = gtk4::SpinButton::new(Some(&adj_notif), 1.0, 0);
    spin_notif.set_halign(gtk4::Align::End);

    add_row(&ui_grid, "Thumbnail Size (px)", &spin_thumb, 0);
    add_row(&ui_grid, "Notification Duration (s)", &spin_notif, 1);

    notebook.append_page(&ui_grid, Some(&gtk4::Label::new(Some("UI"))));

    // --- Privacy Tab ---
    let privacy_grid = create_settings_grid();
    let switch_passwords = create_switch(config.privacy.exclude_passwords);
    add_row(&privacy_grid, "Exclude Passive Password Captures", &switch_passwords, 0);

    notebook.append_page(&privacy_grid, Some(&gtk4::Label::new(Some("Privacy"))));

    main_vbox.append(&notebook);

    // --- Save Button Section ---
    let footer_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    footer_box.set_margin_top(12);

    let warning_label = gtk4::Label::builder()
        .label("Note: Shortcut changes require restart.")
        .css_classes(["dim-label"])
        .valign(gtk4::Align::Center)
        .build();
    footer_box.append(&warning_label);

    let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    footer_box.append(&spacer);

    let save_button = gtk4::Button::builder()
        .label("Save Changes")
        .css_classes(["suggested-action"])
        .build();
    footer_box.append(&save_button);

    main_vbox.append(&footer_box);

    window.set_child(Some(&main_vbox));

    // --- Save Logic ---
    let win_save = window.clone();
    save_button.connect_clicked(move |_| {
        let mut new_config = match Config::load_or_create_default() {
            Ok(c) => c,
            Err(_) => return,
        };

        // Update Shortcuts
        new_config.shortcuts.screenshot = entry_screenshot.text().to_string();
        new_config.shortcuts.history = entry_history.text().to_string();
        new_config.shortcuts.extract_text = entry_extract.text().to_string();
        new_config.shortcuts.decode_qr = entry_qr.text().to_string();

        // Update Capture
        new_config.capture.format = entry_format.text().to_string();
        new_config.capture.quality = spin_quality.value() as u8;
        new_config.capture.show_dimensions = switch_dimensions.is_active();

        // Update History
        new_config.history.max_entries = spin_max_entries.value() as usize;
        new_config.history.retention_days = spin_retention.value() as i64;
        new_config.history.auto_cleanup = switch_cleanup.is_active();

        // Update UI
        new_config.ui.thumbnail_size = spin_thumb.value() as u32;
        new_config.ui.notification_duration = spin_notif.value() as u32;

        // Update Privacy
        new_config.privacy.exclude_passwords = switch_passwords.is_active();

        if let Err(e) = new_config.save() {
            log::error!("Failed to save config: {}", e);
        } else {
            log::info!("Settings saved successfully.");
        }

        win_save.close();
    });

    // Style
    let provider = gtk4::CssProvider::new();
    provider.load_from_data("
        .title-1 { font-size: 20px; font-weight: 800; }
        .dim-label { opacity: 0.6; font-size: 12px; }
        grid label { font-weight: 500; }
        notebook { border: 1px solid alpha(@theme_fg_color, 0.1); border-radius: 8px; }
        .app-logo { border-radius: 6px; }
    ");
    if let Some(display) = gdk4::Display::default() {
        gtk4::style_context_add_provider_for_display(&display, &provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
    }

    window.present();
}

fn setup_shortcut_recording(entry: &gtk4::Entry) {
    let controller = gtk4::EventControllerKey::new();
    let entry_weak = entry.downgrade();
    
    controller.connect_key_pressed(move |_ctrl, key, _code, mods| {
        let entry = match entry_weak.upgrade() {
            Some(e) => e,
            None => return glib::Propagation::Proceed,
        };

        // Filter out modifier-only presses
        if is_modifier(key) {
            return glib::Propagation::Proceed;
        }

        let mut parts = Vec::new();
        if mods.contains(ModifierType::CONTROL_MASK) {
            parts.push("Ctrl");
        }
        if mods.contains(ModifierType::ALT_MASK) {
            parts.push("Alt");
        }
        if mods.contains(ModifierType::SHIFT_MASK) {
            parts.push("Shift");
        }
        if mods.contains(ModifierType::SUPER_MASK) || mods.contains(ModifierType::META_MASK) {
            parts.push("Super");
        }

        if let Some(key_str) = format_key(key) {
            parts.push(&key_str);
            let shortcut_str = parts.join("+");
            entry.set_text(&shortcut_str);
            return glib::Propagation::Stop;
        }

        glib::Propagation::Proceed
    });

    entry.add_controller(controller);
    entry.set_editable(false); // Only allow setting via keypresses
    entry.set_cursor(Some(&gdk4::Cursor::from_name("pointer", None).unwrap()));
    entry.set_placeholder_text(Some("Press shortcut keys…"));
}

fn is_modifier(key: gdk4::Key) -> bool {
    matches!(
        key,
        gdk4::Key::Control_L
            | gdk4::Key::Control_R
            | gdk4::Key::Alt_L
            | gdk4::Key::Alt_R
            | gdk4::Key::Shift_L
            | gdk4::Key::Shift_R
            | gdk4::Key::Super_L
            | gdk4::Key::Super_R
            | gdk4::Key::Meta_L
            | gdk4::Key::Meta_R
    )
}

fn format_key(key: gdk4::Key) -> Option<String> {
    if let Some(name) = key.name() {
        match name.as_str() {
            "space" => Some("Space".into()),
            "Escape" => Some("Escape".into()),
            "Return" => Some("Enter".into()),
            "BackSpace" => Some("Backspace".into()),
            "Tab" => Some("Tab".into()),
            "Delete" => Some("Delete".into()),
            "Insert" => Some("Insert".into()),
            "Home" => Some("Home".into()),
            "End" => Some("End".into()),
            "Page_Up" => Some("PageUp".into()),
            "Page_Down" => Some("PageDown".into()),
            "Up" => Some("Up".into()),
            "Down" => Some("Down".into()),
            "Left" => Some("Left".into()),
            "Right" => Some("Right".into()),
            "Print" => Some("PrintScreen".into()),
            // Alphanumeric
            n if n.len() == 1 => Some(n.to_uppercase()),
            // F-keys
            n if n.starts_with('F') && n[1..].chars().all(|c| c.is_ascii_digit()) => Some(n.into()),
            _ => None,
        }
    } else {
        None
    }
}

fn create_settings_grid() -> gtk4::Grid {
    let grid = gtk4::Grid::builder()
        .row_spacing(18)
        .column_spacing(24)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(20)
        .build();
    grid
}

fn add_row(grid: &gtk4::Grid, label_text: &str, widget: &impl IsA<gtk4::Widget>, row: i32) {
    let label = gtk4::Label::builder()
        .label(label_text)
        .halign(gtk4::Align::Start)
        .build();
    grid.attach(&label, 0, row, 1, 1);
    grid.attach(widget, 1, row, 1, 1);
}

fn create_entry(text: &str) -> gtk4::Entry {
    gtk4::Entry::builder()
        .text(text)
        .hexpand(true)
        .halign(gtk4::Align::End)
        .width_request(200)
        .build()
}

fn create_switch(active: bool) -> gtk4::Switch {
    gtk4::Switch::builder()
        .active(active)
        .halign(gtk4::Align::End)
        .valign(gtk4::Align::Center)
        .build()
}
