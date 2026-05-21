use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::mod_parser::ModInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModStorageInfo {
    pub name: String,
    pub unique_id: String,
    pub folder_path: String,
    pub size_bytes: u64,
    pub size_formatted: String,
    pub file_count: u64,
    pub enabled: bool,
    pub is_content_pack: bool,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageAnalysisResult {
    pub mods: Vec<ModStorageInfo>,
    pub total_size_bytes: u64,
    pub total_size_formatted: String,
    pub total_mods: usize,
    pub enabled_size_bytes: u64,
    pub enabled_size_formatted: String,
    pub disabled_size_bytes: u64,
    pub disabled_size_formatted: String,
    pub largest_mod: Option<ModStorageInfo>,
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn calculate_dir_size(path: &PathBuf) -> (u64, u64) {
    let mut total_size: u64 = 0;
    let mut file_count: u64 = 0;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                let (sub_size, sub_count) = calculate_dir_size(&entry_path);
                total_size += sub_size;
                file_count += sub_count;
            } else if entry_path.is_file() {
                if let Ok(metadata) = fs::metadata(&entry_path) {
                    total_size += metadata.len();
                    file_count += 1;
                }
            }
        }
    }

    (total_size, file_count)
}

#[tauri::command]
pub fn analyze_mod_storage(mods: Vec<ModInfo>) -> Result<StorageAnalysisResult, String> {
    let mut storage_infos: Vec<ModStorageInfo> = Vec::new();
    let mut total_size: u64 = 0;
    let mut enabled_size: u64 = 0;
    let mut disabled_size: u64 = 0;

    for mod_info in &mods {
        let folder_path = PathBuf::from(&mod_info.folder_path);
        if !folder_path.exists() {
            continue;
        }

        let (size_bytes, file_count) = calculate_dir_size(&folder_path);

        total_size += size_bytes;
        if mod_info.enabled {
            enabled_size += size_bytes;
        } else {
            disabled_size += size_bytes;
        }

        storage_infos.push(ModStorageInfo {
            name: mod_info.name.clone(),
            unique_id: mod_info.unique_id.clone(),
            folder_path: mod_info.folder_path.clone(),
            size_bytes,
            size_formatted: format_size(size_bytes),
            file_count,
            enabled: mod_info.enabled,
            is_content_pack: mod_info.is_content_pack,
            version: mod_info.version.clone(),
        });
    }

    storage_infos.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

    let largest_mod = storage_infos.first().cloned();

    Ok(StorageAnalysisResult {
        total_size_bytes: total_size,
        total_size_formatted: format_size(total_size),
        total_mods: storage_infos.len(),
        enabled_size_bytes: enabled_size,
        enabled_size_formatted: format_size(enabled_size),
        disabled_size_bytes: disabled_size,
        disabled_size_formatted: format_size(disabled_size),
        largest_mod,
        mods: storage_infos,
    })
}
