use anyhow::{Context, Result};
use image::{ImageBuffer, ImageFormat, RgbaImage};
use std::io::Cursor;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{self, ConnectionExt as _};
use x11rb::rust_connection::RustConnection;

/// Result of a screen capture containing raw pixel data and dimensions.
#[derive(Clone)]
pub struct CapturedImage {
    pub data: Vec<u8>, // BGRA or RGBA
    pub width: u32,
    pub height: u32,
}

/// Capture the entire screen (all monitors) via X11.
/// Returns BGRA data and dimensions.
pub fn capture_entire_screen() -> Result<CapturedImage> {
    let (conn, screen_num) =
        RustConnection::connect(None).context("Failed to connect to X11 display")?;
    let screen = &conn.setup().roots[screen_num];

    let width = screen.width_in_pixels;
    let height = screen.height_in_pixels;

    let reply = conn
        .get_image(
            xproto::ImageFormat::Z_PIXMAP,
            screen.root,
            0,
            0,
            width,
            height,
            u32::MAX,
        )
        .context("get_image request failed for entire screen")?
        .reply()
        .context("get_image reply failed for entire screen")?;

    let mut data = reply.data;
    
    // Ensure alpha is set to 255 for BGRA
    for chunk in data.chunks_exact_mut(4) {
        chunk[3] = 255;
    }

    Ok(CapturedImage {
        data,
        width: width as u32,
        height: height as u32,
    })
}

/// Capture a specific region of the screen via X11.
#[allow(dead_code)]
pub fn capture_region(x: i32, y: i32, w: u32, h: u32) -> Result<(Vec<u8>, u32, u32)> {
    if w == 0 || h == 0 {
        return Err(anyhow::anyhow!("Invalid capture dimensions: {}x{}", w, h));
    }
    
    let (conn, screen_num) =
        RustConnection::connect(None).context("Failed to connect to X11 display")?;
    let screen = &conn.setup().roots[screen_num];

    let screen_width = screen.width_in_pixels as i32;
    let screen_height = screen.height_in_pixels as i32;
    
    let x = x.clamp(0, screen_width - 1);
    let y = y.clamp(0, screen_height - 1);
    let actual_w = std::cmp::min(w, (screen_width - x) as u32);
    let actual_h = std::cmp::min(h, (screen_height - y) as u32);

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
    for chunk in data.chunks_exact_mut(4) {
        chunk[3] = 255;
    }

    Ok((data, actual_w, actual_h))
}

/// Crop a region from BGRA data and return as RGBA.
pub fn crop_bgra_to_rgba(
    src_data: &[u8],
    src_w: u32,
    src_h: u32,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
) -> Result<(Vec<u8>, u32, u32)> {
    if w == 0 || h == 0 {
        return Err(anyhow::anyhow!("Invalid crop dimensions"));
    }

    let mut dest_rgba = Vec::with_capacity((w * h * 4) as usize);
    
    for row in 0..h {
        let src_y = y + row as i32;
        if src_y < 0 || src_y >= src_h as i32 {
            // Fill with black/transparent if out of bounds
            for _ in 0..w {
                dest_rgba.extend_from_slice(&[0, 0, 0, 255]);
            }
            continue;
        }

        for col in 0..w {
            let src_x = x + col as i32;
            if src_x < 0 || src_x >= src_w as i32 {
                dest_rgba.extend_from_slice(&[0, 0, 0, 255]);
                continue;
            }

            let src_idx = ((src_y as u32 * src_w + src_x as u32) * 4) as usize;
            if src_idx + 3 < src_data.len() {
                let b = src_data[src_idx];
                let g = src_data[src_idx + 1];
                let r = src_data[src_idx + 2];
                let a = src_data[src_idx + 3];
                dest_rgba.push(r);
                dest_rgba.push(g);
                dest_rgba.push(b);
                dest_rgba.push(a);
            } else {
                dest_rgba.extend_from_slice(&[0, 0, 0, 255]);
            }
        }
    }

    Ok((dest_rgba, w, h))
}

/// Convert BGRA pixel data to RGBA.
#[allow(dead_code)]
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

/// Create a thumbnail from PNG bytes.
pub fn create_thumbnail(png_bytes: &[u8], max_size: u32) -> Result<Vec<u8>> {
    let img = image::load_from_memory(png_bytes).context("Failed to decode PNG for thumbnail")?;
    
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
