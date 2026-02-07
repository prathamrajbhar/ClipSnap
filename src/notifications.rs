/// Send a desktop notification for a successful screenshot,
/// showing the captured image as the notification icon.
pub fn notify_screenshot_success(png_path: &std::path::Path) {
    let _ = notify_rust::Notification::new()
        .summary("Screenshot Captured")
        .body("Image copied to clipboard")
        .icon(&png_path.to_string_lossy())
        .timeout(3000)
        .show();
}

/// Send a desktop notification for a failed screenshot.
pub fn notify_screenshot_error(msg: &str) {
    let _ = notify_rust::Notification::new()
        .summary("Screenshot Failed")
        .body(msg)
        .icon("dialog-error")
        .timeout(3000)
        .show();
}

/// Send a desktop notification when an item is restored from history.
pub fn notify_clipboard_restored() {
    let _ = notify_rust::Notification::new()
        .summary("Clipboard Restored")
        .body("Item copied from history")
        .icon("edit-paste")
        .timeout(2000)
        .show();
}

/// Send a desktop notification that the daemon is running.
pub fn notify_daemon_started() {
    let _ = notify_rust::Notification::new()
        .summary("ClipSnap Running")
        .body("Screenshot: Ctrl+Alt+S  •  History: Alt+H")
        .icon("accessories-clipboard")
        .timeout(3000)
        .show();
}
