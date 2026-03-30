/// Send a desktop notification for a successful screenshot,
/// showing the captured image as the notification icon or preview.
pub fn notify_screenshot_success(png_path: &std::path::Path) {
    let result = notify_rust::Notification::new()
        .appname("ClipSnap")
        .summary("📸 Screenshot Captured")
        .body("Image copied to clipboard and saved to history")
        .icon(&png_path.to_string_lossy())
        .hint(notify_rust::Hint::ImagePath(png_path.to_string_lossy().into_owned()))
        .timeout(3000)
        .urgency(notify_rust::Urgency::Low)
        .show();

    if let Err(e) = result {
        log::error!("Notification Error (Success): {}", e);
    }
}

/// Send a desktop notification for a failed screenshot.
pub fn notify_screenshot_error(msg: &str) {
    let result = notify_rust::Notification::new()
        .appname("ClipSnap")
        .summary("❌ Screenshot Failed")
        .body(&format!("Error: {}", msg))
        .icon("dialog-error")
        .timeout(4000)
        .urgency(notify_rust::Urgency::Normal)
        .show();

    if let Err(e) = result {
        log::error!("Notification Error (Error): {}", e);
    }
}

/// Send a desktop notification for successful text extraction (OCR).
pub fn notify_text_extraction(text: &str) {
    let preview = if text.len() > 100 {
        format!("{}...", &text[..100])
    } else {
        text.to_string()
    };

    let result = notify_rust::Notification::new()
        .appname("ClipSnap")
        .summary("🔤 Text Extracted")
        .body(&format!("Content: {}", preview))
        .icon("edit-copy")
        .timeout(3000)
        .urgency(notify_rust::Urgency::Low)
        .show();

    if let Err(e) = result {
        log::error!("Notification Error (OCR): {}", e);
    }
}
