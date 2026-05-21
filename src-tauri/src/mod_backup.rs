use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use chrono::Utc;

fn is_safe_to_delete(target: &Path, parent_dir: &Path) -> bool {
    let target_canon = target.canonicalize().unwrap_or_else(|_| target.to_path_buf());
    let parent_canon = parent_dir.canonicalize().unwrap_or_else(|_| parent_dir.to_path_buf());

    if target_canon == parent_canon {
        eprintln!("[BACKUP SAFETY] BLOCKED: target is parent directory: {}", target.display());
        return false;
    }

    if !target_canon.starts_with(&parent_canon) {
        eprintln!("[BACKUP SAFETY] BLOCKED: target outside expected dir: {}", target.display());
        return false;
    }

    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModBackupInfo {
    pub backup_name: String,
    pub mod_name: String,
    pub mod_unique_id: String,
    pub backup_path: String,
    pub created_at: String,
    pub size_mb: f64,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModBackupResult {
    pub success: bool,
    pub backup_path: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModRestoreResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModBackupList {
    pub backups: Vec<ModBackupInfo>,
    pub total_backups: usize,
    pub total_size_mb: f64,
}

fn get_backup_dir() -> Result<PathBuf, String> {
    let app_data = dirs::config_dir()
        .ok_or("Cannot find app data directory".to_string())?;
    let backup_dir = app_data.join("svl").join("mod-backups");
    Ok(backup_dir)
}

fn calculate_dir_size(path: &PathBuf) -> u64 {
    let mut total = 0;
    if path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    total += calculate_dir_size(&p);
                } else if let Ok(metadata) = fs::metadata(&p) {
                    total += metadata.len();
                }
            }
        }
    }
    total
}

fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> Result<(), String> {
    if !dst.exists() {
        fs::create_dir_all(dst).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    for entry in fs::read_dir(src).map_err(|e| format!("Failed to read directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path).map_err(|e| format!("Failed to copy file: {}", e))?;
        }
    }
    Ok(())
}

fn read_mod_info(mod_path: &PathBuf) -> (String, String, String) {
    let manifest_path = mod_path.join("manifest.json");
    if manifest_path.exists() {
        if let Ok(content) = fs::read_to_string(&manifest_path) {
            if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&content) {
                let name = manifest["Name"].as_str().unwrap_or("Unknown").to_string();
                let unique_id = manifest["UniqueID"].as_str().unwrap_or("").to_string();
                let version = manifest["Version"].as_str().unwrap_or("Unknown").to_string();
                return (name, unique_id, version);
            }
        }
    }
    (mod_path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string(), String::new(), "Unknown".to_string())
}

#[tauri::command]
pub fn backup_mod_before_update(
    mod_path: String,
    custom_backup_dir: Option<String>,
) -> Result<ModBackupResult, String> {
    let path = PathBuf::from(&mod_path);

    if !path.exists() {
        return Err("Mod folder does not exist".to_string());
    }

    let (name, unique_id, version) = read_mod_info(&path);

    let is_custom = custom_backup_dir.is_some();
    let backup_dir = if let Some(custom) = &custom_backup_dir {
        let p = PathBuf::from(custom);
        fs::create_dir_all(&p).map_err(|e| format!("Failed to create custom backup directory: {}", e))?;
        p
    } else {
        let dir = get_backup_dir()?;
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create backup directory: {}", e))?;
        dir
    };

    let sanitized_name = unique_id.replace('.', "_").replace(" ", "_");
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("{}_{}", timestamp, sanitized_name);
    let backup_path = backup_dir.join(&backup_name);

    copy_dir_recursive(&path, &backup_path)?;

    let size_bytes = calculate_dir_size(&backup_path);
    let size_mb = size_bytes as f64 / (1024.0 * 1024.0);

    let meta_path = backup_path.join(".svl_backup_meta.json");
    let meta = serde_json::json!({
        "mod_name": name,
        "unique_id": unique_id,
        "version": version,
        "created_at": Utc::now().to_rfc3339(),
        "size_mb": size_mb,
        "original_path": mod_path,
        "custom_dir": is_custom,
    });
    fs::write(&meta_path, serde_json::to_string_pretty(&meta).unwrap())
        .unwrap_or_default();

    Ok(ModBackupResult {
        success: true,
        backup_path: backup_path.to_string_lossy().to_string(),
        message: format!("Backed up '{}' ({} MB)", name, size_mb),
    })
}

#[tauri::command]
pub fn restore_mod_from_backup(
    backup_path: String,
    target_mod_path: String,
) -> Result<ModRestoreResult, String> {
    let src = PathBuf::from(&backup_path);

    if !src.exists() {
        return Err("Backup folder does not exist".to_string());
    }

    let target = PathBuf::from(&target_mod_path);

    let mods_parent = target
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    if target.exists() {
        if !is_safe_to_delete(&target, &mods_parent) {
            return Err(format!("安全拦截: 不允许删除路径 {}", target.display()));
        }
        fs::remove_dir_all(&target).map_err(|e| format!("Failed to remove current mod: {}", e))?;
    }

    copy_dir_recursive(&src, &target)?;

    Ok(ModRestoreResult {
        success: true,
        message: format!("Restored mod from backup"),
    })
}

#[tauri::command]
pub fn list_mod_backups(
    mod_unique_id: Option<String>,
) -> Result<ModBackupList, String> {
    let backup_dir = get_backup_dir()?;

    if !backup_dir.exists() {
        return Ok(ModBackupList {
            backups: Vec::new(),
            total_backups: 0,
            total_size_mb: 0.0,
        });
    }

    let mut backups = Vec::new();
    let mut total_size = 0u64;

    for entry in fs::read_dir(&backup_dir).map_err(|e| format!("Failed to read backup directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let meta_path = path.join(".svl_backup_meta.json");
        if !meta_path.exists() {
            continue;
        }

        if let Ok(content) = fs::read_to_string(&meta_path) {
            if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&content) {
                let uid = meta["unique_id"].as_str().unwrap_or("").to_string();

                if let Some(filter_id) = &mod_unique_id {
                    if uid != *filter_id {
                        continue;
                    }
                }

                let size_bytes = calculate_dir_size(&path);
                total_size += size_bytes;

                backups.push(ModBackupInfo {
                    backup_name: path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string(),
                    mod_name: meta["mod_name"].as_str().unwrap_or("Unknown").to_string(),
                    mod_unique_id: uid,
                    backup_path: path.to_string_lossy().to_string(),
                    created_at: meta["created_at"].as_str().unwrap_or("Unknown").to_string(),
                    size_mb: size_bytes as f64 / (1024.0 * 1024.0),
                    version: meta["version"].as_str().unwrap_or("Unknown").to_string(),
                });
            }
        }
    }

    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let total_backups = backups.len();

    Ok(ModBackupList {
        backups,
        total_backups,
        total_size_mb: total_size as f64 / (1024.0 * 1024.0),
    })
}

#[tauri::command]
pub fn delete_mod_backup(
    backup_path: String,
) -> Result<ModRestoreResult, String> {
    let path = PathBuf::from(&backup_path);

    if !path.exists() {
        return Err("Backup does not exist".to_string());
    }

    fs::remove_dir_all(&path).map_err(|e| format!("Failed to delete backup: {}", e))?;

    Ok(ModRestoreResult {
        success: true,
        message: "Backup deleted".to_string(),
    })
}
