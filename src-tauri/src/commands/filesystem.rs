use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DriveInfo {
    pub name: String,
    pub path: String,
    pub total_space: u64,
    pub available_space: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DirectoryContent {
    pub path: String,
    pub directories: Vec<DirectoryEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DirectoryEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

#[tauri::command]
pub fn list_drives() -> Result<Vec<DriveInfo>, String> {
    let mut drives = Vec::new();
    
    #[cfg(target_os = "windows")]
    {
        // Windows: list all drive letters
        for letter in b'A'..=b'Z' {
            let drive_path = format!("{}:\\", letter as char);
            let path = PathBuf::from(&drive_path);
            if path.exists() {
                if let Ok(metadata) = fs::metadata(&path) {
                    drives.push(DriveInfo {
                        name: format!("{}:", letter as char),
                        path: drive_path,
                        total_space: 0,
                        available_space: 0,
                    });
                }
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // macOS: list /Volumes
        let volumes_path = PathBuf::from("/Volumes");
        if let Ok(entries) = fs::read_dir(&volumes_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    drives.push(DriveInfo {
                        name: name.clone(),
                        path: path.to_string_lossy().to_string(),
                        total_space: 0,
                        available_space: 0,
                    });
                }
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        // Linux: list /media and /mnt
        for mount_point in &["/media", "/mnt"] {
            let path = PathBuf::from(mount_point);
            if path.exists() {
                if let Ok(entries) = fs::read_dir(&path) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            let name = path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("")
                                .to_string();
                            drives.push(DriveInfo {
                                name: name.clone(),
                                path: path.to_string_lossy().to_string(),
                                total_space: 0,
                                available_space: 0,
                            });
                        }
                    }
                }
            }
        }
    }
    
    Ok(drives)
}

#[tauri::command]
pub fn list_directory(path: String) -> Result<DirectoryContent, String> {
    let path_buf = PathBuf::from(&path);
    
    if !path_buf.exists() {
        return Err(format!("Directory does not exist: {}", path));
    }
    
    if !path_buf.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }
    
    let mut directories = Vec::new();
    
    match fs::read_dir(&path_buf) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    let name = entry.file_name()
                        .to_string_lossy()
                        .to_string();
                    
                    // Skip hidden directories
                    if name.starts_with('.') {
                        continue;
                    }
                    
                    directories.push(DirectoryEntry {
                        name,
                        path: entry_path.to_string_lossy().to_string(),
                        is_dir: true,
                    });
                }
            }
        }
        Err(e) => {
            return Err(format!("Failed to read directory: {}", e));
        }
    }
    
    // Sort directories by name
    directories.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    
    Ok(DirectoryContent {
        path,
        directories,
    })
}
