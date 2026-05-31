use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use chrono::Utc;
use tauri::{AppHandle, Emitter};

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
    let backup_dir = crate::app_logger::get_svl_data_dir().join("mod-backups");
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
    let meta_json = serde_json::to_string_pretty(&meta)
        .map_err(|e| format!("备份元数据序列化失败: {}", e))?;
    fs::write(&meta_path, meta_json)
        .map_err(|e| format!("写入备份元数据失败: {}", e))?;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModSnapshotInfo {
    pub snapshot_name: String,
    pub created_at: String,
    pub mod_count: usize,
    pub size_mb: f64,
    pub label: String,
    pub snapshot_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModSnapshotList {
    pub snapshots: Vec<ModSnapshotInfo>,
    pub total_snapshots: usize,
    pub total_size_mb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotResult {
    pub success: bool,
    pub message: String,
}

fn get_snapshot_dir() -> Result<PathBuf, String> {
    let snapshot_dir = crate::app_logger::get_svl_data_dir().join("mod-snapshots");
    Ok(snapshot_dir)
}

#[tauri::command]
pub fn create_snapshot(
    mods_path: String,
    label: String,
) -> Result<SnapshotResult, String> {
    let mods_dir = PathBuf::from(&mods_path);

    if !mods_dir.exists() || !mods_dir.is_dir() {
        return Err("Mods directory does not exist".to_string());
    }

    let snapshot_dir = get_snapshot_dir()?;
    fs::create_dir_all(&snapshot_dir)
        .map_err(|e| format!("Failed to create snapshot directory: {}", e))?;

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let sanitized_label = if label.is_empty() {
        timestamp.to_string()
    } else {
        format!("{}_{}", timestamp, label.replace(' ', "_").replace('/', "_").replace('\\', "_"))
    };
    let snapshot_path = snapshot_dir.join(&sanitized_label);

    fs::create_dir_all(&snapshot_path)
        .map_err(|e| format!("Failed to create snapshot folder: {}", e))?;

    let mut mod_count = 0usize;

    let entries = fs::read_dir(&mods_dir)
        .map_err(|e| format!("Failed to read Mods directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let mod_dir = entry.path();
        if !mod_dir.is_dir() {
            continue;
        }

        let dir_name = mod_dir.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
        let dest = snapshot_path.join(dir_name);
        copy_dir_recursive(&mod_dir, &dest)?;
        mod_count += 1;
    }

    let size_bytes = calculate_dir_size(&snapshot_path);
    let size_mb = size_bytes as f64 / (1024.0 * 1024.0);

    let meta_path = snapshot_path.join(".svl_snapshot_meta.json");
    let meta = serde_json::json!({
        "label": sanitized_label,
        "created_at": Utc::now().to_rfc3339(),
        "mod_count": mod_count,
        "size_mb": size_mb,
        "original_mods_path": mods_path,
    });
    let meta_json = serde_json::to_string_pretty(&meta)
        .map_err(|e| format!("Failed to write snapshot metadata: {}", e))?;
    fs::write(&meta_path, meta_json)
        .map_err(|e| format!("Failed to write snapshot metadata: {}", e))?;

    Ok(SnapshotResult {
        success: true,
        message: format!("Snapshot created: {} mods, {:.1} MB", mod_count, size_mb),
    })
}

#[tauri::command]
pub fn list_snapshots() -> Result<ModSnapshotList, String> {
    let snapshot_dir = get_snapshot_dir()?;

    if !snapshot_dir.exists() {
        return Ok(ModSnapshotList {
            snapshots: Vec::new(),
            total_snapshots: 0,
            total_size_mb: 0.0,
        });
    }

    let mut snapshots = Vec::new();
    let mut total_size = 0.0f64;

    let entries = fs::read_dir(&snapshot_dir)
        .map_err(|e| format!("Failed to read snapshot directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let meta_path = path.join(".svl_snapshot_meta.json");
        if !meta_path.exists() {
            continue;
        }

        if let Ok(content) = fs::read_to_string(&meta_path) {
            if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&content) {
                let mod_count = meta["mod_count"].as_u64().unwrap_or(0) as usize;
                let size_mb = meta["size_mb"].as_f64().unwrap_or(0.0);
                total_size += size_mb;

                snapshots.push(ModSnapshotInfo {
                    snapshot_name: path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string(),
                    created_at: meta["created_at"].as_str().unwrap_or("Unknown").to_string(),
                    mod_count,
                    size_mb,
                    label: meta["label"].as_str().unwrap_or("").to_string(),
                    snapshot_path: path.to_string_lossy().to_string(),
                });
            }
        }
    }

    snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let total_snapshots = snapshots.len();

    Ok(ModSnapshotList {
        snapshots,
        total_snapshots,
        total_size_mb: total_size,
    })
}

#[tauri::command]
pub fn restore_snapshot(
    snapshot_name: String,
    mods_path: String,
    app: AppHandle,
) -> Result<SnapshotResult, String> {
    let snapshot_dir = get_snapshot_dir()?;
    let snapshot_path = snapshot_dir.join(&snapshot_name);

    if !snapshot_path.exists() {
        return Err("Snapshot does not exist".to_string());
    }

    let mods_dir = PathBuf::from(&mods_path);

    let entries = fs::read_dir(&mods_dir)
        .map_err(|e| format!("Failed to read Mods directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        if path.is_dir() {
            if is_safe_to_delete(&path, &mods_dir) {
                fs::remove_dir_all(&path)
                    .map_err(|e| format!("Failed to remove mod: {}", e))?;
            } else {
                eprintln!("[SNAPSHOT SAFETY] Skipping unsafe path: {}", path.display());
            }
        }
    }

    let snapshot_entries = fs::read_dir(&snapshot_path)
        .map_err(|e| format!("Failed to read snapshot: {}", e))?;

    let mut restored = 0usize;

    for entry in snapshot_entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        let name = entry.file_name();

        let name_str = name.to_string_lossy();
        if name_str.starts_with(".svl_") {
            continue;
        }

        if path.is_dir() {
            let dest = mods_dir.join(&*name);
            copy_dir_recursive(&path, &dest)?;
            restored += 1;
        }
    }

    let _ = app.emit("mods-changed", ());

    Ok(SnapshotResult {
        success: true,
        message: format!("Restored {} mods from snapshot", restored),
    })
}

#[tauri::command]
pub fn delete_snapshot(
    snapshot_name: String,
) -> Result<SnapshotResult, String> {
    let snapshot_dir = get_snapshot_dir()?;
    let snapshot_path = snapshot_dir.join(&snapshot_name);

    if !snapshot_path.exists() {
        return Err("Snapshot does not exist".to_string());
    }

    if !is_safe_to_delete(&snapshot_path, &snapshot_dir) {
        return Err("Safety check failed: cannot delete snapshot path".to_string());
    }

    fs::remove_dir_all(&snapshot_path)
        .map_err(|e| format!("Failed to delete snapshot: {}", e))?;

    Ok(SnapshotResult {
        success: true,
        message: "Snapshot deleted".to_string(),
    })
}
