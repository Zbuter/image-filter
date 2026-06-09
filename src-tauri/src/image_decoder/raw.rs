use image::ImageFormat;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Extract embedded JPEG preview from RAW files using chunked scanning.
/// Reads the file in small chunks instead of loading the entire file into memory.
/// Returns the largest embedded JPEG found without full decode validation.
pub fn extract_embedded_jpeg(path: &Path) -> Result<Vec<u8>, String> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let file_size = file.metadata().map_err(|e| format!("Failed to get metadata: {}", e))?.len();

    const CHUNK_SIZE: usize = 64 * 1024;
    let mut jpeg_segments: Vec<(u64, u64)> = Vec::new();
    let mut file_pos: u64 = 0;
    let mut prev_last_byte: Option<u8> = None;
    let mut chunk = vec![0u8; CHUNK_SIZE];

    loop {
        let bytes_read = file.read(&mut chunk).map_err(|e| format!("Read error: {}", e))?;
        if bytes_read == 0 {
            break;
        }
        let data = &chunk[..bytes_read];

        let start = match prev_last_byte {
            Some(0xFF) if !data.is_empty() && data[0] == 0xD8 => 1,
            _ => 0,
        };

        let mut i = start;
        while i + 1 < data.len() {
            if data[i] == 0xFF && data[i + 1] == 0xD8 {
                let soi_file_pos = file_pos + i as u64;
                let mut j = i + 2;
                let mut found_eoi = false;
                while j + 1 < data.len() {
                    if data[j] == 0xFF && data[j + 1] == 0xD9 {
                        let eoi_file_pos = file_pos + j as u64 + 1;
                        let segment_len = eoi_file_pos - soi_file_pos + 1;
                        if segment_len > 1000 {
                            jpeg_segments.push((soi_file_pos, eoi_file_pos));
                        }
                        found_eoi = true;
                        break;
                    }
                    j += 1;
                }
                if !found_eoi {
                    // JPEG spans chunk boundary - scan forward from SOI
                    let scan_size = std::cmp::min(file_size - soi_file_pos, 20 * 1024 * 1024);
                    let mut buf = vec![0u8; scan_size as usize];
                    file.seek(SeekFrom::Start(soi_file_pos)).ok();
                    if let Ok(n) = file.read(&mut buf) {
                        for k in 2..n.saturating_sub(1) {
                            if buf[k] == 0xFF && buf[k + 1] == 0xD9 {
                                let segment_len = (k + 2) as u64;
                                if segment_len > 1000 {
                                    jpeg_segments.push((soi_file_pos, soi_file_pos + segment_len - 1));
                                }
                                break;
                            }
                        }
                    }
                    file.seek(SeekFrom::Start(file_pos + bytes_read as u64)).ok();
                }
                i += 2;
            } else {
                i += 1;
            }
        }

        prev_last_byte = Some(data[data.len() - 1]);
        file_pos += bytes_read as u64;
    }

    if jpeg_segments.is_empty() {
        return Err("No embedded JPEG found in RAW file".to_string());
    }

    // Return the largest JPEG segment without decode validation
    jpeg_segments.sort_by(|a, b| (b.1 - b.0).cmp(&(a.1 - a.0)));
    let (start, end) = jpeg_segments[0];
    let len = (end - start + 1) as usize;
    let mut jpeg_data = vec![0u8; len];
    file.seek(SeekFrom::Start(start)).ok();
    file.read_exact(&mut jpeg_data).map_err(|e| format!("Failed to read JPEG segment: {}", e))?;

    Ok(jpeg_data)
}

#[allow(dead_code)]
pub fn decode_raw_to_rgb(path: &Path) -> Result<image::RgbImage, String> {
    let jpeg_data = extract_embedded_jpeg(path)?;
    let img = image::load_from_memory_with_format(&jpeg_data, ImageFormat::Jpeg)
        .map_err(|e| format!("Failed to decode embedded JPEG: {}", e))?;
    Ok(img.to_rgb8())
}

#[allow(dead_code)]
pub fn generate_raw_thumbnail(path: &Path, size: u32) -> Result<image::RgbImage, String> {
    let rgb = decode_raw_to_rgb(path)?;
    let (w, h) = (rgb.width(), rgb.height());
    let scale = size as f64 / std::cmp::max(w, h) as f64;
    let new_w = (w as f64 * scale).round() as u32;
    let new_h = (h as f64 * scale).round() as u32;
    let thumbnail = image::imageops::resize(
        &rgb,
        new_w,
        new_h,
        image::imageops::FilterType::Lanczos3,
    );
    Ok(thumbnail)
}

#[allow(dead_code)]
pub fn get_raw_dimensions(path: &Path) -> Result<(Option<u32>, Option<u32>), String> {
    match decode_raw_to_rgb(path) {
        Ok(img) => Ok((Some(img.width()), Some(img.height()))),
        Err(_) => Ok((None, None)),
    }
}
