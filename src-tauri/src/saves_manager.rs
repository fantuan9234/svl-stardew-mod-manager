use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use chrono::{DateTime, Local};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveInfo {
    pub name: String,
    pub farm_name: String,
    pub farm_type: String,
    pub hours_played: u64,
    pub last_modified: String,
    pub save_path: String,
    pub backup_count: usize,
    pub linked_profile: Option<String>,
    pub character_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub name: String,
    pub original_name: String,
    pub backup_time: String,
    pub backup_path: String,
    pub size_mb: f64,
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

fn get_saves_directory() -> Option<PathBuf> {
    if let Some(appdata) = std::env::var("APPDATA").ok() {
        let saves = PathBuf::from(appdata).join("StardewValley").join("Saves");
        if saves.exists() {
            return Some(saves);
        }
    }
    None
}

fn get_bindings_path() -> Option<PathBuf> {
    if let Some(appdata) = std::env::var("APPDATA").ok() {
        let saves_dir = PathBuf::from(appdata).join("StardewValley").join("Saves");
        if !saves_dir.exists() {
            let _ = fs::create_dir_all(&saves_dir);
        }
        Some(saves_dir.join("svl-profile-bindings.json"))
    } else {
        None
    }
}

fn load_bindings() -> HashMap<String, String> {
    if let Some(bindings_path) = get_bindings_path() {
        if bindings_path.exists() {
            if let Ok(content) = fs::read_to_string(&bindings_path) {
                if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&content) {
                    return map;
                }
            }
        }
    }
    HashMap::new()
}

fn save_bindings(bindings: &HashMap<String, String>) -> Result<(), String> {
    let bindings_path = get_bindings_path().ok_or("Cannot get bindings path")?;
    let json = serde_json::to_string_pretty(bindings)
        .map_err(|e| format!("Failed to serialize bindings: {}", e))?;
    println!("[SVL Debug] Writing bindings to: {:?}", bindings_path);
    println!("[SVL Debug] Bindings content: {}", json);
    fs::write(&bindings_path, json)
        .map_err(|e| format!("Failed to write bindings: {}", e))?;
    println!("[SVL Debug] Bindings file written successfully");
    Ok(())
}

fn parse_savegame_info(folder_path: &PathBuf) -> (String, String, u64) {
    let mut character_name = String::new();
    let mut farm_name = String::new();
    let mut hours_played: u64 = 0;

    if let Ok(entries) = fs::read_dir(folder_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                if file_name.ends_with("_info") || file_name == "SaveGameInfo" {
                    if let Ok(content) = fs::read_to_string(&path) {
                        // Try JSON first
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(name) = value.get("name").and_then(|v| v.as_str()) {
                                character_name = name.to_string();
                            }
                            if let Some(farm) = value.get("farmName").and_then(|v| v.as_str()) {
                                farm_name = farm.to_string();
                            }
                            if let Some(hours) = value.get("secondsPlayed").and_then(|v| v.as_u64()) {
                                hours_played = hours / 3600;
                            }
                        } else {
                            // Fallback to XML parsing using regex-like approach
                            // Extract <name> tag (first occurrence in Farmer section)
                            // Note: <Farmer> tag may have attributes like xmlns:xsi, so search for "<Farmer" not "<Farmer>"
                            if let Some(pos) = content.find("<Farmer") {
                                // Find the closing '>' of the Farmer tag
                                if let Some(tag_end) = content[pos..].find('>') {
                                    let farmer_section = &content[pos + tag_end + 1..];
                                    if let Some(name_start) = farmer_section.find("<name>") {
                                        if let Some(name_end) = farmer_section[name_start..].find("</name>") {
                                            let name_value = &farmer_section[name_start + 6..name_start + name_end];
                                            if !name_value.is_empty() {
                                                character_name = name_value.to_string();
                                            }
                                        }
                                    }
                                }
                            }
                            
                            // Extract <farmName> tag
                            if let Some(farm_start) = content.find("<farmName>") {
                                if let Some(farm_end) = content[farm_start..].find("</farmName>") {
                                    farm_name = content[farm_start + 10..farm_start + farm_end].to_string();
                                }
                            }
                            
                            // Extract <secondsPlayed> or <millisecondsPlayed> tag
                            if let Some(secs_start) = content.find("<secondsPlayed>") {
                                if let Some(secs_end) = content[secs_start..].find("</secondsPlayed>") {
                                    let secs_str = &content[secs_start + 15..secs_start + secs_end];
                                    if let Ok(secs) = secs_str.parse::<u64>() {
                                        hours_played = secs / 3600;
                                    }
                                }
                            } else if let Some(ms_start) = content.find("<millisecondsPlayed>") {
                                if let Some(ms_end) = content[ms_start..].find("</millisecondsPlayed>") {
                                    let ms_str = &content[ms_start + 20..ms_start + ms_end];
                                    if let Ok(ms) = ms_str.parse::<u64>() {
                                        hours_played = ms / 3600000;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    (character_name, farm_name, hours_played)
}

fn parse_save_folder(folder_path: &PathBuf) -> Option<SaveInfo> {
    let folder_name = folder_path.file_name()?.to_string_lossy().to_string();
    let parts: Vec<&str> = folder_name.split('_').collect();
    if parts.len() < 2 {
        return None;
    }

    let (character_name, farm_name, hours_played) = parse_savegame_info(folder_path);

    let display_character = if character_name.is_empty() {
        parts[0].to_string()
    } else {
        character_name
    };

    let display_farm = if farm_name.is_empty() {
        folder_name.clone()
    } else {
        farm_name
    };

    let farm_type = "Standard".to_string();

    let last_modified = {
        let mut latest_time = String::new();
        if let Ok(entries) = fs::read_dir(folder_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(metadata) = path.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            let datetime: DateTime<Local> = modified.into();
                            let formatted = datetime.format("%Y-%m-%d %H:%M").to_string();
                            if formatted > latest_time {
                                latest_time = formatted;
                            }
                        }
                    }
                }
            }
        }
        if latest_time.is_empty() {
            if let Ok(metadata) = folder_path.metadata() {
                if let Ok(modified) = metadata.modified() {
                    let datetime: DateTime<Local> = modified.into();
                    datetime.format("%Y-%m-%d %H:%M").to_string()
                } else {
                    "Unknown".to_string()
                }
            } else {
                "Unknown".to_string()
            }
        } else {
            latest_time
        }
    };

    let backup_count = count_backups_for_save(folder_path);

    let bindings = load_bindings();
    let linked_profile = bindings.get(&folder_name).cloned();

    Some(SaveInfo {
        name: display_character.clone(),
        farm_name: display_farm,
        farm_type,
        hours_played,
        last_modified,
        save_path: folder_path.to_string_lossy().to_string(),
        backup_count,
        linked_profile,
        character_name: display_character,
    })
}

fn count_backups_for_save(save_path: &PathBuf) -> usize {
    let backup_dir = save_path.join("SVL_Backups");
    if !backup_dir.exists() {
        return 0;
    }
    fs::read_dir(&backup_dir).map_or(0, |entries| entries.count())
}

#[tauri::command]
pub fn scan_saves() -> Result<Vec<SaveInfo>, String> {
    let saves_dir = get_saves_directory().ok_or("Save directory not found, please ensure the game has been run at least once")?;

    let mut saves = Vec::new();

    if let Ok(entries) = fs::read_dir(&saves_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(save_info) = parse_save_folder(&path) {
                    saves.push(save_info);
                }
            }
        }
    }

    saves.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

    Ok(saves)
}

#[tauri::command]
pub fn backup_save(save_path: String, backup_dir: String) -> Result<SaveBackupResult, String> {
    let save_path = PathBuf::from(&save_path);
    if !save_path.exists() {
        return Err("Save path does not exist".to_string());
    }

    let backup_base = PathBuf::from(&backup_dir);
    if !backup_base.exists() {
        fs::create_dir_all(&backup_base).map_err(|e| format!("Failed to create backup directory: {}", e))?;
    }

    let save_name = save_path
        .file_name()
        .ok_or("Cannot get save name")?
        .to_string_lossy()
        .to_string();

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("{}_{}", save_name, timestamp);
    let backup_path = backup_base.join(&backup_name);

    fs_extra::dir::copy(
        &save_path,
        &backup_base,
        &fs_extra::dir::CopyOptions::new().overwrite(true).content_only(false),
    )
    .map_err(|e| format!("Backup failed: {}", e))?;

    let copied_dir = backup_base.join(&save_name);
    if copied_dir.exists() && copied_dir != backup_path {
        fs::rename(&copied_dir, &backup_path)
            .map_err(|e| format!("Failed to rename backup: {}", e))?;
    }

    Ok(SaveBackupResult {
        success: true,
        backup_path: backup_path.to_string_lossy().to_string(),
        message: format!("Save backed up to: {}", backup_path.display()),
    })
}

#[tauri::command]
pub fn restore_save(backup_path: String, saves_dir: String) -> Result<SaveRestoreResult, String> {
    let backup = PathBuf::from(&backup_path);
    if !backup.exists() {
        return Err("Backup path does not exist".to_string());
    }

    let saves = PathBuf::from(&saves_dir);
    if !saves.exists() {
        return Err("Saves directory does not exist".to_string());
    }

    let save_name = extract_original_save_name(&backup)?;
    let target_path = saves.join(&save_name);

    if target_path.exists() {
        fs::remove_dir_all(&target_path)
            .map_err(|e| format!("Failed to remove existing save: {}", e))?;
    }

    fs_extra::dir::copy(
        &backup,
        &saves,
        &fs_extra::dir::CopyOptions::new().overwrite(true).content_only(false),
    )
    .map_err(|e| format!("Restore failed: {}", e))?;

    Ok(SaveRestoreResult {
        success: true,
        message: format!("Save restored from backup: {}", backup_path),
    })
}

fn extract_original_save_name(backup_path: &PathBuf) -> Result<String, String> {
    let name = backup_path
        .file_name()
        .ok_or("Cannot get backup name")?
        .to_string_lossy()
        .to_string();

    let name_without_ext = name.trim_end_matches(".disabled");

    if let Some(pos) = name_without_ext.rfind("_") {
        let before_underscore = &name_without_ext[..pos];
        if let Some(pos2) = before_underscore.rfind("_") {
            let potential_timestamp = &name_without_ext[pos2 + 1..];
            let parts: Vec<&str> = potential_timestamp.split('_').collect();
            if parts.len() == 2 && parts[0].len() == 8 && parts[1].len() == 6 {
                if parts[0].chars().all(|c| c.is_ascii_digit())
                    && parts[1].chars().all(|c| c.is_ascii_digit())
                {
                    return Ok(name_without_ext[..pos2].to_string());
                }
            }
        }
    }

    Ok(name)
}

#[tauri::command]
pub fn list_save_backups(save_path: String) -> Result<Vec<BackupInfo>, String> {
    let save_path = PathBuf::from(&save_path);
    if !save_path.exists() {
        return Err("Save path does not exist".to_string());
    }

    let backup_dir = save_path.join("SVL_Backups");
    if !backup_dir.exists() {
        return Ok(vec![]);
    }

    let mut backups = Vec::new();

    if let Ok(entries) = fs::read_dir(&backup_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let size = get_dir_size(&path);
                let size_mb = size as f64 / (1024.0 * 1024.0);

                let backup_time = if let Ok(metadata) = path.metadata() {
                    if let Ok(created) = metadata.created() {
                        let datetime: DateTime<Local> = created.into();
                        datetime.format("%Y-%m-%d %H:%M").to_string()
                    } else {
                        "Unknown".to_string()
                    }
                } else {
                    "Unknown".to_string()
                };

                backups.push(BackupInfo {
                    name: name.clone(),
                    original_name: save_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    backup_time,
                    backup_path: path.to_string_lossy().to_string(),
                    size_mb,
                });
            }
        }
    }

    backups.sort_by(|a, b| b.backup_time.cmp(&a.backup_time));

    Ok(backups)
}

fn get_dir_size(path: &PathBuf) -> u64 {
    let mut size = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                size += get_dir_size(&path);
            } else if let Ok(metadata) = path.metadata() {
                size += metadata.len();
            }
        }
    }
    size
}

#[tauri::command]
pub fn link_save_to_profile(save_path: String, profile_name: String) -> Result<bool, String> {
    let save_path_buf = PathBuf::from(&save_path);
    if !save_path_buf.exists() {
        return Err("Save path does not exist".to_string());
    }

    let folder_name = save_path_buf
        .file_name()
        .ok_or("Cannot get save folder name")?
        .to_string_lossy()
        .to_string();

    println!("[SVL Debug] Linking save '{}' to profile '{}'", folder_name, profile_name);

    let mut bindings = load_bindings();
    bindings.insert(folder_name.clone(), profile_name.clone());
    save_bindings(&bindings)?;

    println!("[SVL Debug] Successfully linked save '{}' to profile '{}'", folder_name, profile_name);
    Ok(true)
}

#[tauri::command]
pub fn unlink_save_from_profile(save_path: String) -> Result<bool, String> {
    let save_path_buf = PathBuf::from(&save_path);
    if !save_path_buf.exists() {
        return Err("Save path does not exist".to_string());
    }

    let folder_name = save_path_buf
        .file_name()
        .ok_or("Cannot get save folder name")?
        .to_string_lossy()
        .to_string();

    println!("[SVL Debug] Unlinking save '{}' from profile", folder_name);

    let mut bindings = load_bindings();
    bindings.remove(&folder_name);
    save_bindings(&bindings)?;

    println!("[SVL Debug] Successfully unlinked save '{}'", folder_name);
    Ok(true)
}

#[tauri::command]
pub fn get_save_profile_binding(save_path: String) -> Result<Option<String>, String> {
    let save_path_buf = PathBuf::from(&save_path);
    let folder_name = save_path_buf
        .file_name()
        .ok_or("Cannot get save folder name")?
        .to_string_lossy()
        .to_string();

    let bindings = load_bindings();
    let result = bindings.get(&folder_name).cloned();

    println!("[SVL Debug] Getting profile binding for save '{}': {:?}", folder_name, result);
    Ok(result)
}

#[tauri::command]
pub fn launch_game_with_save_profile(
    game_path: String,
    save_path: String,
    app: tauri::AppHandle,
) -> Result<crate::smapi_launcher::LaunchResult, String> {
    let save_path_buf = PathBuf::from(&save_path);
    let folder_name = save_path_buf
        .file_name()
        .ok_or("Cannot get save folder name")?
        .to_string_lossy()
        .to_string();

    let bindings = load_bindings();
    let profile_name = bindings.get(&folder_name);

    println!("[saves_manager] Launch with save profile:");
    println!("[saves_manager]   Save folder: {}", folder_name);
    println!("[saves_manager]   Linked profile: {:?}", profile_name);

    if let Some(profile) = profile_name {
        println!("[saves_manager] Applying profile: {}", profile);
        match crate::profiles::apply_profile(&game_path, profile) {
            Ok(applied_profile) => {
                println!("[saves_manager] Profile applied successfully: {}", applied_profile.name);
                println!("[saves_manager] Enabled mods count: {}", applied_profile.enabled_mod_ids.len());
            }
            Err(e) => {
                println!("[saves_manager] Failed to apply profile: {}", e);
                return Err(format!("Failed to apply profile: {}", e));
            }
        }
    } else {
        println!("[saves_manager] No profile linked for this save");
    }

    crate::smapi_launcher::launch_game(game_path, app)
}

#[tauri::command]
pub fn open_save_location() -> Result<bool, String> {
    let saves_dir = get_saves_directory().ok_or("Save directory not found")?;
    tauri_plugin_opener::open_path(&saves_dir, Option::<&str>::None)
        .map_err(|e| format!("Failed to open save location: {}", e))?;
    Ok(true)
}

#[tauri::command]
pub fn open_backup_dialog() -> Result<String, String> {
    use std::env;
    if let Some(desktop) = env::var("USERPROFILE").ok() {
        let default_backup = PathBuf::from(desktop).join("SVL_SaveBackups");
        return Ok(default_backup.to_string_lossy().to_string());
    }
    Err("Cannot get default backup path".to_string())
}
