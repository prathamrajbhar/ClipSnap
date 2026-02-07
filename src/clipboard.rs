use crate::database::Database;
use crate::screenshot;
use anyhow::Result;
use arboard::{Clipboard, ImageData};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Set an image (RGBA pixels) to the system clipboard.
pub fn set_clipboard_image(clipboard: &mut Clipboard, rgba: &[u8], width: usize, height: usize) -> Result<()> {
    let img_data = ImageData {
        width,
        height,
        bytes: rgba.to_vec().into(),
    };
    clipboard
        .set_image(img_data)
        .map_err(|e| anyhow::anyhow!("Failed to set clipboard image: {}", e))?;
    Ok(())
}

/// Set text to the system clipboard.
pub fn set_clipboard_text(clipboard: &mut Clipboard, text: &str) -> Result<()> {
    clipboard
        .set_text(text.to_string())
        .map_err(|e| anyhow::anyhow!("Failed to set clipboard text: {}", e))?;
    Ok(())
}

/// Calculate a hash of data for deduplication.
fn calculate_hash(data: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

/// Run clipboard monitoring in a background thread.
/// Polls the clipboard every 500ms and stores new content to the database.
pub fn monitor_clipboard(
    clipboard: Arc<Mutex<Clipboard>>,
    db: Arc<Mutex<Database>>,
    last_text_hash: Arc<Mutex<Option<u64>>>,
    last_image_hash: Arc<Mutex<Option<u64>>>,
) {
    log::info!("Clipboard monitoring started");

    loop {
        std::thread::sleep(Duration::from_millis(500));

        let mut cb = match clipboard.lock() {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to lock clipboard: {}", e);
                continue;
            }
        };

        // --- Check text ---
        if let Ok(text) = cb.get_text() {
            if !text.is_empty() {
                let hash = calculate_hash(text.as_bytes());
                let is_new = {
                    let last = last_text_hash.lock().unwrap();
                    *last != Some(hash)
                };
                if is_new {
                    if let Ok(db) = db.lock() {
                        if db.insert_text(&text).is_ok() {
                            log::debug!("Stored text clipboard entry ({} bytes)", text.len());
                        }
                    }
                    *last_text_hash.lock().unwrap() = Some(hash);
                }
            }
        }

        // --- Check image ---
        if let Ok(img) = cb.get_image() {
            if !img.bytes.is_empty() {
                let hash = calculate_hash(&img.bytes);
                let is_new = {
                    let last = last_image_hash.lock().unwrap();
                    *last != Some(hash)
                };
                if is_new {
                    // Convert RGBA to PNG and generate thumbnail
                    let width = img.width as u32;
                    let height = img.height as u32;
                    if let Ok(png) = screenshot::encode_png(&img.bytes, width, height) {
                        let thumb = screenshot::create_thumbnail(&png, 150).unwrap_or_default();
                        if let Ok(db) = db.lock() {
                            if db.insert_image(&png, &thumb).is_ok() {
                                log::debug!("Stored image clipboard entry ({}×{})", width, height);
                            }
                        }
                    }
                    *last_image_hash.lock().unwrap() = Some(hash);
                }
            }
        }
    }
}
