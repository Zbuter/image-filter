use image::ImageFormat;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use crate::image_decoder;

pub fn get_or_generate(image_path: &Path, extension: &str, size: u32) -> Result<PathBuf, String> {
    let cache_dir = get_cache_dir()?;
    
    // Generate cache key from path, modification time, and size
    let cache_key = generate_cache_key(image_path, size)?;
    let thumbnail_path = cache_dir.join(format!("{}_{}.jpg", cache_key, size));
    
    // Check if thumbnail exists
    if thumbnail_path.exists() {
        return Ok(thumbnail_path);
    }
    
    // Generate thumbnail
    let thumbnail = if is_raw_format(extension) {
        image_decoder::raw::generate_raw_thumbnail(image_path, size)?
    } else {
        image_decoder::regular::generate_thumbnail(image_path, size)?
    };
    
    // Save thumbnail
    thumbnail.save_with_format(&thumbnail_path, ImageFormat::Jpeg)
        .map_err(|e| format!("Failed to save thumbnail: {}", e))?;
    
    Ok(thumbnail_path)
}

fn get_cache_dir() -> Result<PathBuf, String> {
    let cache_dir = dirs::cache_dir()
        .ok_or("Could not determine cache directory")?
        .join("image-filter-thumbnails");
    
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("Failed to create cache directory: {}", e))?;
    }
    
    Ok(cache_dir)
}

fn generate_cache_key(image_path: &Path, size: u32) -> Result<String, String> {
    let metadata = fs::metadata(image_path)
        .map_err(|e| format!("Failed to read metadata: {}", e))?;
    
    let modified = metadata.modified()
        .map_err(|e| format!("Failed to get modification time: {}", e))?;
    
    let modified_time = modified.duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("Failed to convert time: {}", e))?
        .as_secs();
    
    let path_str = image_path.to_string_lossy();
    let input = format!("{}:{}:{}", path_str, modified_time, size);
    
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    
    Ok(hex::encode(result))
}

fn is_raw_format(extension: &str) -> bool {
    matches!(
        extension,
        "cr2" | "cr3" | "nef" | "arw" | "dng" | "orf" | "rw2" | "pef" | "srw" | "raf"
    )
}
