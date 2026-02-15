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


