use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::image_decoder;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageInfo {
    pub name: String,
    pub path: String,
    pub extension: String,
    pub size: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportProgress {
    pub total: usize,
    pub completed: usize,
    pub failed: Vec<String>,
}

const IMAGE_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "webp", "tiff", "tif", "bmp",
    "cr2", "cr3", "nef", "arw", "dng", "orf", "rw2", "pef", "srw", "raf",
];

#[tauri::command]
pub fn scan_images(directory: String) -> Result<Vec<ImageInfo>, String> {
    let path_buf = PathBuf::from(&directory);
    
    if !path_buf.exists() {
        return Err(format!("Directory does not exist: {}", directory));
    }
    
    if !path_buf.is_dir() {
        return Err(format!("Path is not a directory: {}", directory));
    }
    
    let mut images = Vec::new();
    
    match fs::read_dir(&path_buf) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    let extension = entry_path.extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e.to_lowercase())
                        .unwrap_or_default();
                    
                    if IMAGE_EXTENSIONS.contains(&extension.as_str()) {
                        let name = entry.file_name()
                            .to_string_lossy()
                            .to_string();
                        
                        let metadata = entry.metadata().ok();
                        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                        
                        let (width, height) = (None, None);
                        
                        images.push(ImageInfo {
                            name,
                            path: entry_path.to_string_lossy().to_string(),
                            extension,
                            size,
                            width,
                            height,
                        });
                    }
                }
            }
        }
        Err(e) => {
            return Err(format!("Failed to read directory: {}", e));
        }
    }
    
    images.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    
    Ok(images)
}

#[tauri::command]
pub fn get_raw_preview(path: String) -> Result<String, String> {
    let path_buf = PathBuf::from(&path);
    
    if !path_buf.exists() {
        return Err(format!("File does not exist: {}", path));
    }
    
    let jpeg_data = image_decoder::raw::extract_embedded_jpeg(&path_buf)?;
    let base64_data = base64::encode(&jpeg_data);
    
    Ok(format!("data:image/jpeg;base64,{}", base64_data))
}

#[tauri::command]
pub fn export_images(sources: Vec<String>, target_dir: String) -> Result<ExportProgress, String> {
    let target_path = PathBuf::from(&target_dir);
    
    if !target_path.exists() {
        return Err(format!("Target directory does not exist: {}", target_dir));
    }
    
    let total = sources.len();
    let mut completed = 0;
    let mut failed = Vec::new();
    
    for source in &sources {
        let source_path = PathBuf::from(source);
        
        if !source_path.exists() {
            failed.push(source.clone());
            continue;
        }
        
        let file_name = source_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        let dest_path = target_path.join(file_name);
        
        match fs::copy(&source_path, &dest_path) {
            Ok(_) => completed += 1,
            Err(_) => failed.push(source.clone()),
        }
    }
    
    Ok(ExportProgress {
        total,
        completed,
        failed,
    })
}

fn is_raw_format(extension: &str) -> bool {
    matches!(
        extension,
        "cr2" | "cr3" | "nef" | "arw" | "dng" | "orf" | "rw2" | "pef" | "srw" | "raf"
    )
}
