use image::io::Reader as ImageReader;
use std::path::Path;

pub fn get_dimensions(path: &Path) -> Result<(Option<u32>, Option<u32>), String> {
    match ImageReader::open(path) {
        Ok(reader) => {
            match reader.into_dimensions() {
                Ok((width, height)) => Ok((Some(width), Some(height))),
                Err(_) => Ok((None, None)),
            }
        }
        Err(_) => Ok((None, None)),
    }
}

pub fn decode_to_rgb(path: &Path) -> Result<image::RgbImage, String> {
    let img = ImageReader::open(path)
        .map_err(|e| format!("Failed to open image: {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode image: {}", e))?;
    
    Ok(img.to_rgb8())
}

pub fn generate_thumbnail(path: &Path, size: u32) -> Result<image::RgbImage, String> {
    let img = decode_to_rgb(path)?;
    let (w, h) = (img.width(), img.height());
    let scale = size as f64 / std::cmp::max(w, h) as f64;
    let new_w = (w as f64 * scale).round() as u32;
    let new_h = (h as f64 * scale).round() as u32;
    let thumbnail = image::imageops::resize(
        &img,
        new_w,
        new_h,
        image::imageops::FilterType::Lanczos3,
    );
    Ok(thumbnail)
}
