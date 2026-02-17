use anyhow::{Context, Result};
use image::{ImageBuffer, ImageFormat, RgbaImage};
use std::io::Cursor;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{self, ConnectionExt as _};
use x11rb::rust_connection::RustConnection;

/// Capture a specific region of the screen via X11 and return (BGRA data, width, height).
/// Enhanced with better error handling and coordinate validation.
pub fn capture_region(x: i32, y: i32, w: u32, h: u32) -> Result<(Vec<u8>, u32, u32)> {
    // Validate input parameters
    if w == 0 || h == 0 {
        return Err(anyhow::anyhow!("Invalid capture dimensions: {}x{}", w, h));
    }
    
    let (conn, screen_num) =
        RustConnection::connect(None).context("Failed to connect to X11 display")?;
    let screen = &conn.setup().roots[screen_num];

    // Validate coordinates are within screen bounds
    let screen_width = screen.width_in_pixels as i32;
    let screen_height = screen.height_in_pixels as i32;
    
    if x < 0 || y < 0 || x >= screen_width || y >= screen_height {
        return Err(anyhow::anyhow!(
            "Capture coordinates ({}, {}) are outside screen bounds ({}x{})", 
            x, y, screen_width, screen_height
        ));
    }

    // Clamp dimensions to screen bounds to prevent X11 errors
    let actual_w = std::cmp::min(w, (screen_width - x) as u32);
    let actual_h = std::cmp::min(h, (screen_height - y) as u32);

    // Use a small delay to ensure any compositor effects are settled
    std::thread::sleep(std::time::Duration::from_millis(50));

    let reply = conn
        .get_image(
            xproto::ImageFormat::Z_PIXMAP,
            screen.root,
            x as i16,
            y as i16,
            actual_w as u16,
            actual_h as u16,
            u32::MAX,
        )
        .context("get_image request failed")?
        .reply()
        .context("get_image reply failed")?;

    let mut data = reply.data;
    
    // Validate data size matches expected
    let expected_size = (actual_w * actual_h * 4) as usize;
    if data.len() != expected_size {
        log::warn!("Data size mismatch: got {}, expected {}", data.len(), expected_size);
    }
    
    // X11 on little-endian returns BGRX (32-bit pixels). Set alpha to 255 → BGRA.
    for chunk in data.chunks_exact_mut(4) {
        chunk[3] = 255;
    }

    Ok((data, actual_w, actual_h))
}

/// Convert BGRA pixel data to RGBA.
pub fn bgra_to_rgba(bgra: &[u8]) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(bgra.len());
    for chunk in bgra.chunks_exact(4) {
        rgba.push(chunk[2]); // R
        rgba.push(chunk[1]); // G
        rgba.push(chunk[0]); // B
        rgba.push(chunk[3]); // A
    }
    rgba
}

/// Encode RGBA pixel data to PNG bytes.
pub fn encode_png(rgba_pixels: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    let img: RgbaImage = ImageBuffer::from_raw(width, height, rgba_pixels.to_vec())
        .context("Failed to create image buffer – size mismatch")?;

    let mut png_bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)
        .context("Failed to encode PNG")?;

    Ok(png_bytes)
}

/// Create a thumbnail from PNG bytes. Returns PNG thumbnail bytes.
/// Enhanced with better quality settings and error handling.
pub fn create_thumbnail(png_bytes: &[u8], max_size: u32) -> Result<Vec<u8>> {
    let img = image::load_from_memory(png_bytes).context("Failed to decode PNG for thumbnail")?;
    
    // Use high-quality Lanczos3 filter for better thumbnails
    let thumbnail = img.resize(
        max_size,
        max_size,
        image::imageops::FilterType::Lanczos3,
    );

    let mut thumb_bytes: Vec<u8> = Vec::new();
    thumbnail
        .write_to(&mut Cursor::new(&mut thumb_bytes), ImageFormat::Png)
        .context("Failed to encode thumbnail PNG")?;

    Ok(thumb_bytes)
}

/// Get screen information for better coordinate mapping
pub fn get_screen_info() -> Result<(i32, i32, u32, u32)> {
    let (conn, screen_num) =
        RustConnection::connect(None).context("Failed to connect to X11 display")?;
    let screen = &conn.setup().roots[screen_num];
    
    Ok((0, 0, screen.width_in_pixels as u32, screen.height_in_pixels as u32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bgra_to_rgba() {
        let bgra = vec![10, 20, 30, 255, 40, 50, 60, 255];
        let rgba = bgra_to_rgba(&bgra);
        assert_eq!(rgba, vec![30, 20, 10, 255, 60, 50, 40, 255]);
    }

    #[test]
    fn test_encode_png() {
        // 2×2 red RGBA image
        let pixels = vec![
            255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255,
        ];
        let png = encode_png(&pixels, 2, 2).unwrap();
        assert!(!png.is_empty());
        // PNG magic bytes
        assert_eq!(&png[0..4], &[0x89, 0x50, 0x4E, 0x47]);
    }

    #[test]
    fn test_create_thumbnail() {
        // Create a small valid PNG first
        let pixels = vec![255u8; 10 * 10 * 4];
        let png = encode_png(&pixels, 10, 10).unwrap();
        let thumb = create_thumbnail(&png, 5).unwrap();
        assert!(!thumb.is_empty());
    }
}
