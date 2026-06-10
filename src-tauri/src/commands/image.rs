use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use crate::image_decoder;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageInfo {
    pub name: String,
    pub path: String,
    pub extension: String,
    pub size: u64,
    pub modified: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_path: Option<String>,
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

const RAW_EXTENSIONS: &[&str] = &[
    "cr2", "cr3", "nef", "arw", "dng", "orf", "rw2", "pef", "srw", "raf",
];

fn is_raw_format(extension: &str) -> bool {
    RAW_EXTENSIONS.contains(&extension)
}

/// Get the stem (filename without extension) for pairing
fn file_stem(name: &str) -> &str {
    name.rsplit_once('.').map(|(stem, _)| stem).unwrap_or(name)
}

#[tauri::command]
pub fn scan_images(directories: Vec<String>) -> Result<Vec<ImageInfo>, String> {
    // Collect all images from all directories
    let mut all_images: Vec<ImageInfo> = Vec::new();

    for directory in &directories {
        let path_buf = PathBuf::from(directory);

        if !path_buf.exists() || !path_buf.is_dir() {
            continue; // Skip invalid directories silently
        }

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

                            let modified = metadata.as_ref()
                                    .and_then(|m| m.modified().ok())
                                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                    .map(|d| d.as_secs())
                                    .unwrap_or(0);
                            all_images.push(ImageInfo {
                                name,
                                path: entry_path.to_string_lossy().to_string(),
                                extension,
                                size,
                                modified,
                                width: None,
                                height: None,
                                raw_path: None,
                            });
                        }
                    }
                }
            }
            Err(_) => continue,
        }
    }

    // Build pairing index: stem -> (jpg_entry_index, raw_entry_index)
    // Group by lowercase stem to handle case-insensitive matching
    let mut stem_map: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, img) in all_images.iter().enumerate() {
        let stem = file_stem(&img.name).to_lowercase();
        stem_map.entry(stem).or_default().push(i);
    }

    // For each group with both RAW and non-RAW, keep the non-RAW as primary
    // and attach the RAW path. Pure RAW entries (no JPG pair) stay as-is.
    let mut result: Vec<ImageInfo> = Vec::new();
    let mut skip_indices: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for (_stem, indices) in &stem_map {
        let mut raw_indices = Vec::new();
        let mut non_raw_indices = Vec::new();

        for &idx in indices {
            if is_raw_format(&all_images[idx].extension) {
                raw_indices.push(idx);
            } else {
                non_raw_indices.push(idx);
            }
        }

        if !non_raw_indices.is_empty() && !raw_indices.is_empty() {
            // Pair: use first non-RAW as primary, attach first RAW path
            let primary_idx = non_raw_indices[0];
            let raw_idx = raw_indices[0];
            let mut primary = all_images[primary_idx].clone();
            primary.raw_path = Some(all_images[raw_idx].path.clone());
            result.push(primary);

            // Mark RAW as consumed
            skip_indices.insert(raw_idx);
            // If multiple non-RAW, keep them too (but only first gets the RAW pair)
            for &idx in &non_raw_indices[1..] {
                result.push(all_images[idx].clone());
            }
        } else {
            // No pairing possible, keep all entries as-is
            for &idx in indices {
                result.push(all_images[idx].clone());
            }
        }
    }

    result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(result)
}

#[tauri::command]
pub fn get_raw_preview(path: String) -> Result<String, String> {
    let path_buf = PathBuf::from(&path);

    if !path_buf.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    let jpeg_data = image_decoder::raw::extract_embedded_jpeg(&path_buf)?;
    use base64::Engine;
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&jpeg_data);

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
