use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveBackupInfo {
    pub backup_name: String,
    pub character_name: String,
    pub farm_name: String,
    pub save_folder: String,
    pub backup_path: String,
    pub created_at: String,
    pub size_mb: f64,
    pub source: String, // "save_editor" or "manual"
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveBackupList {
    pub backups: Vec<SaveBackupInfo>,
    pub total_backups: usize,
    pub total_size_mb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveBackupResult {
    pub success: bool,
    pub backup_path: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveRestoreResult {
    pub success: bool,
    pub message: String,
}

fn get_save_backup_dir() -> Result<PathBuf, String> {
    let dir = crate::app_logger::get_svl_data_dir().join("save-backups");
    Ok(dir)
}

#[tauri::command]
pub fn get_save_backup_dir_cmd() -> Result<String, String> {
    let dir = get_save_backup_dir()?;
    Ok(dir.to_string_lossy().to_string())
}

fn calculate_dir_size(path: &Path) -> u64 {
    let mut total = 0u64;
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

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
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

fn read_meta(path: &Path) -> Option<serde_json::Value> {
    let meta_path = path.join(".svl_save_backup_meta.json");
    let content = fs::read_to_string(&meta_path).ok()?;
    serde_json::from_str(&content).ok()
}

/// 创建存档备份（由存档编辑器在保存修改前自动调用）
/// 备份整个存档文件夹到 SVL 数据目录的 save-backups 子目录
pub fn create_save_backup_internal(
    save_folder: &Path,
    source: &str,
    note: &str,
) -> Result<SaveBackupResult, String> {
    if !save_folder.is_dir() {
        return Err("Save folder does not exist".to_string());
    }

    let folder_name = save_folder
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let backup_dir = get_save_backup_dir()?;
    fs::create_dir_all(&backup_dir).map_err(|e| format!("Failed to create backup directory: {}", e))?;

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("{}_{}", timestamp, folder_name);
    let backup_path = backup_dir.join(&backup_name);

    copy_dir_recursive(save_folder, &backup_path)?;

    // 读取角色名和农场名（如果可能）
    let (character_name, farm_name) = read_save_info(save_folder);

    let size_bytes = calculate_dir_size(&backup_path);
    let size_mb = size_bytes as f64 / (1024.0 * 1024.0);

    let meta = serde_json::json!({
        "character_name": character_name,
        "farm_name": farm_name,
        "save_folder": save_folder.to_string_lossy(),
        "created_at": Utc::now().to_rfc3339(),
        "size_mb": size_mb,
        "source": source,
        "note": note,
    });
    let meta_path = backup_path.join(".svl_save_backup_meta.json");
    fs::write(&meta_path, serde_json::to_string_pretty(&meta).unwrap_or_default())
        .map_err(|e| format!("Failed to write meta: {}", e))?;

    Ok(SaveBackupResult {
        success: true,
        backup_path: backup_path.to_string_lossy().to_string(),
        message: format!("Backed up save '{}' ({:.1} MB)", folder_name, size_mb),
    })
}

fn read_save_info(save_folder: &Path) -> (String, String) {
    // 尝试从 SaveGameInfo 中读取（XML 格式）
    let info_path = save_folder.join("SaveGameInfo");
    if let Ok(content) = fs::read_to_string(&info_path) {
        let character = extract_xml_tag(&content, "name").unwrap_or_default();
        let farm = extract_xml_tag(&content, "farmName").unwrap_or_default();
        return (character, farm);
    }
    (String::new(), String::new())
}

fn extract_xml_tag(content: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = content.find(&open)? + open.len();
    let end = content[start..].find(&close)? + start;
    Some(content[start..end].to_string())
}

#[tauri::command]
pub fn create_save_backup(
    save_folder: String,
    note: Option<String>,
) -> Result<SaveBackupResult, String> {
    let path = PathBuf::from(&save_folder);
    create_save_backup_internal(&path, "manual", note.as_deref().unwrap_or(""))
}

#[tauri::command]
pub fn list_save_file_backups() -> Result<SaveBackupList, String> {
    let backup_dir = get_save_backup_dir()?;

    let mut backups = Vec::new();
    let mut total_size = 0u64;

    if backup_dir.exists() {
        for entry in fs::read_dir(&backup_dir)
            .map_err(|e| format!("Failed to read backup directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let meta = read_meta(&path).unwrap_or_else(|| serde_json::json!({}));
            let size_bytes = calculate_dir_size(&path);
            total_size += size_bytes;

            backups.push(SaveBackupInfo {
                backup_name: path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string(),
                character_name: meta["character_name"].as_str().unwrap_or("").to_string(),
                farm_name: meta["farm_name"].as_str().unwrap_or("").to_string(),
                save_folder: meta["save_folder"].as_str().unwrap_or("").to_string(),
                backup_path: path.to_string_lossy().to_string(),
                created_at: meta["created_at"].as_str().unwrap_or("Unknown").to_string(),
                size_mb: size_bytes as f64 / (1024.0 * 1024.0),
                source: meta["source"].as_str().unwrap_or("manual").to_string(),
                note: meta["note"].as_str().unwrap_or("").to_string(),
            });
        }
    }

    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(SaveBackupList {
        total_backups: backups.len(),
        total_size_mb: total_size as f64 / (1024.0 * 1024.0),
        backups,
    })
}

#[tauri::command]
pub fn restore_save_file_backup(backup_path: String) -> Result<SaveRestoreResult, String> {
    let backup = PathBuf::from(&backup_path);
    if !backup.is_dir() {
        return Err("Backup directory does not exist".to_string());
    }

    let meta = read_meta(&backup).ok_or_else(|| "Backup metadata missing".to_string())?;
    let target = meta["save_folder"]
        .as_str()
        .ok_or_else(|| "Save folder path missing in metadata".to_string())?;
    let target_path = PathBuf::from(target);

    if !target_path.exists() {
        return Err(format!(
            "Original save folder no longer exists: {}",
            target_path.display()
        ));
    }

    // 简单安全检查：目标路径必须包含 Saves 子串
    if !target_path.to_string_lossy().contains("Saves") && !target_path.to_string_lossy().contains("Saves") {
        return Err("Safety check failed: target is not a save folder".to_string());
    }

    // 先把还原前的"当前存档"状态留一份副本到 SVL 数据目录，
    // 而不是 Saves 目录里——避免被游戏识别为新存档。
    let folder_name = target_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("save");
    let restore_history_dir = get_save_backup_dir()?.join("restore-history");
    if let Ok(()) = fs::create_dir_all(&restore_history_dir) {
        let history_backup = restore_history_dir.join(format!(
            "{}_before_restore_{}",
            folder_name,
            Utc::now().format("%Y%m%d_%H%M%S")
        ));
        let _ = copy_dir_recursive(&target_path, &history_backup);
    }

    // 删除目标文件夹中除 SVL_Backups 之外的所有内容
    if let Ok(entries) = fs::read_dir(&target_path) {
        for entry in entries.flatten() {
            let p = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("SVL_Backups") || name_str.starts_with(".") {
                continue;
            }
            if p.is_dir() {
                let _ = fs::remove_dir_all(&p);
            } else {
                let _ = fs::remove_file(&p);
            }
        }
    }

    // 复制备份内容到目标
    if let Ok(entries) = fs::read_dir(&backup) {
        for entry in entries.flatten() {
            let p = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with(".") {
                continue;
            }
            let dest = target_path.join(&name);
            if p.is_dir() {
                copy_dir_recursive(&p, &dest)?;
            } else {
                fs::copy(&p, &dest).map_err(|e| format!("Failed to copy: {}", e))?;
            }
        }
    }

    Ok(SaveRestoreResult {
        success: true,
        message: format!("Restored from backup"),
    })
}

#[tauri::command]
pub fn delete_save_file_backup(backup_path: String) -> Result<SaveRestoreResult, String> {
    let path = PathBuf::from(&backup_path);
    if !path.exists() {
        return Err("Backup does not exist".to_string());
    }

    if path.is_dir() {
        fs::remove_dir_all(&path).map_err(|e| format!("Failed to delete: {}", e))?;
    } else {
        fs::remove_file(&path).map_err(|e| format!("Failed to delete: {}", e))?;
    }

    Ok(SaveRestoreResult {
        success: true,
        message: "Backup deleted".to_string(),
    })
}
