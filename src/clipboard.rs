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
/// Polls the clipboard every 750ms (adaptive) and stores new content to the database.
pub fn monitor_clipboard(
    clipboard: Arc<Mutex<Clipboard>>,
    db: Arc<Mutex<Database>>,
    last_text_hash: Arc<Mutex<Option<u64>>>,
    last_image_hash: Arc<Mutex<Option<u64>>>,
) {
    log::info!("Clipboard monitoring started");

    let mut no_change_count = 0u32;
    
    loop {
        // Adaptive polling: slow down if no changes detected
        let poll_interval = if no_change_count > 5 {
            1000 // 1 second when idle
        } else {
            750  // 750ms normally
        };
        std::thread::sleep(Duration::from_millis(poll_interval));

        let mut cb = match clipboard.lock() {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to lock clipboard: {}", e);
                continue;
            }
        };

        let mut changed = false;

        // --- Check text ---
        if let Ok(text) = cb.get_text() {
            if !text.is_empty() {
                let hash = calculate_hash(text.as_bytes());
                let is_new = {
                    let last = last_text_hash.lock().unwrap();
                    *last != Some(hash)
                };
                if is_new {
                    changed = true;
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
                    changed = true;
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

        // Update adaptive polling counter
        if changed {
            no_change_count = 0;
        } else {
            no_change_count = no_change_count.saturating_add(1);
        }
    }
}
