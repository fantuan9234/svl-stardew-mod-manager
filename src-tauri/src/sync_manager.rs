use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_dialog::FilePath;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPackage {
    pub version: String,
    pub host_name: String,
    pub profile_name: String,
    pub created_at: String,
    pub mods: Vec<SyncModEntry>,
    pub configs: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncModEntry {
    pub name: String,
    pub unique_id: String,
    pub version: String,
    pub author: String,
    pub url: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncDiff {
    pub missing_mods: Vec<SyncModEntry>,
    pub version_mismatch: Vec<VersionMismatch>,
    pub extra_mods: Vec<SyncModEntry>,
    pub config_diffs: Vec<ConfigDiff>,
    pub total_changes: usize,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSyncDiff {
    pub local_missing: Vec<SyncModEntry>,
    pub remote_missing: Vec<SyncModEntry>,
    pub version_mismatch: Vec<VersionMismatch>,
    pub common_mods: Vec<SyncModEntry>,
    pub total_changes: usize,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionMismatch {
    pub mod_entry: SyncModEntry,
    pub current_version: String,
    pub required_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDiff {
    pub mod_name: String,
    pub config_file: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncApplyResult {
    pub success: bool,
    pub applied_mods: Vec<String>,
    pub failed_mods: Vec<String>,
    pub configs_applied: Vec<String>,
    pub message: String,
}

#[tauri::command]
pub fn export_sync_package(
    app: tauri::AppHandle,
    game_path: String,
    profile_name: String,
    host_name: String,
) -> Result<String, String> {
    let profile = crate::profiles::load_profile(&game_path, &profile_name)?;

    let all_mods = crate::profiles::scan_mods_for_profiles(&game_path);
    let mod_map: HashMap<String, &crate::profiles::ModInfo> = all_mods.iter()
        .map(|m| (m.unique_id.clone(), m))
        .collect();

    let mut sync_mods = Vec::new();

    for unique_id in &profile.enabled_mod_ids {
        if let Some(mod_info) = mod_map.get(unique_id) {
            let mods_path = std::path::PathBuf::from(&game_path).join("Mods");
            let url = find_mod_url(&mods_path, &mod_info.name, unique_id);

            sync_mods.push(SyncModEntry {
                name: mod_info.name.clone(),
                unique_id: mod_info.unique_id.clone(),
                version: mod_info.version.clone(),
                author: mod_info.author.clone(),
                url,
                enabled: true,
            });
        } else {
            sync_mods.push(SyncModEntry {
                name: unique_id.clone(),
                unique_id: unique_id.clone(),
                version: "0.0.0".to_string(),
                author: String::new(),
                url: None,
                enabled: true,
            });
        }
    }

    sync_mods.sort_by(|a, b| a.name.cmp(&b.name));

    let sync_package = SyncPackage {
        version: "2.0.0".to_string(),
        host_name,
        profile_name: profile_name.clone(),
        created_at: chrono::Utc::now().to_rfc3339(),
        mods: sync_mods,
        configs: HashMap::new(),
    };

    let result = app.dialog().file()
        .set_title("Save sync package")
        .add_filter("SVL Sync", &["svl_sync"])
        .blocking_save_file();

    let export_path = match result {
        Some(FilePath::Path(p)) => {
            let s = p.to_string_lossy().to_string();
            if s.ends_with(".svl_sync") { s } else { format!("{}.svl_sync", s) }
        }
        Some(FilePath::Url(u)) => u.to_string(),
        None => return Err("Cancelled".to_string()),
    };

    let json_content = serde_json::to_string_pretty(&sync_package)
        .map_err(|e| format!("Failed to serialize: {}", e))?;

    fs::write(&export_path, json_content)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(export_path)
}

fn find_mod_url(mods_path: &std::path::PathBuf, _mod_name: &str, unique_id: &str) -> Option<String> {
    find_mod_url_recursive(mods_path, unique_id)
}

fn find_mod_url_recursive(mods_path: &std::path::PathBuf, unique_id: &str) -> Option<String> {
    if let Ok(entries) = fs::read_dir(mods_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let folder_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if folder_name.starts_with('.') && !folder_name.starts_with("..") {
                continue;
            }
            let manifest_path = path.join("manifest.json");
            if manifest_path.exists() {
                if let Ok(content) = fs::read_to_string(&manifest_path) {
                    if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&content) {
                        let uid = manifest["UniqueID"].as_str().unwrap_or("");
                        if uid == unique_id {
                            return manifest.get("UpdateKeys")
                                .and_then(|v| v.as_array())
                                .and_then(|arr| arr.first())
                                .and_then(|v| v.as_str())
                                .map(|s| {
                                    if s.starts_with("Nexus:") {
                                        format!("https://www.nexusmods.com/stardewvalley/mods/{}", &s[7..])
                                    } else {
                                        s.to_string()
                                    }
                                });
                        }
                    }
                }
            }
            if let Some(url) = find_mod_url_recursive(&path, unique_id) {
                return Some(url);
            }
        }
    }
    None
}

#[tauri::command]
pub fn compare_sync_diff(
    sync_file_path: String,
    game_path: String,
    profile_name: String,
) -> Result<ProfileSyncDiff, String> {
    let content = fs::read_to_string(&sync_file_path)
        .map_err(|e| format!("Failed to read sync file: {}", e))?;

    let sync_package: SyncPackage = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse sync file: {}", e))?;

    let profile = crate::profiles::load_profile(&game_path, &profile_name)?;
    let all_mods = crate::profiles::scan_mods_for_profiles(&game_path);
    let mod_map: HashMap<String, &crate::profiles::ModInfo> = all_mods.iter()
        .map(|m| (m.unique_id.clone(), m))
        .collect();

    let remote_mods: HashMap<&str, SyncModEntry> = sync_package.mods.iter()
        .map(|m| (m.unique_id.as_str(), m.clone()))
        .collect();

    let mut local_missing = Vec::new();
    let mut remote_missing = Vec::new();
    let mut version_mismatch = Vec::new();
    let mut common_mods = Vec::new();

    let enabled_set: std::collections::HashSet<&str> = profile.enabled_mod_ids.iter().map(|s| s.as_str()).collect();

    for unique_id in &profile.enabled_mod_ids {
        match remote_mods.get(unique_id.as_str()) {
            Some(remote_mod) => {
                if let Some(local_info) = mod_map.get(unique_id) {
                    if local_info.version != remote_mod.version {
                        version_mismatch.push(VersionMismatch {
                            mod_entry: remote_mod.clone(),
                            current_version: local_info.version.clone(),
                            required_version: remote_mod.version.clone(),
                        });
                    } else {
                        common_mods.push(remote_mod.clone());
                    }
                } else {
                    common_mods.push(remote_mod.clone());
                }
            }
            None => {
                if let Some(local_info) = mod_map.get(unique_id) {
                    remote_missing.push(SyncModEntry {
                        name: local_info.name.clone(),
                        unique_id: local_info.unique_id.clone(),
                        version: local_info.version.clone(),
                        author: local_info.author.clone(),
                        url: None,
                        enabled: true,
                    });
                }
            }
        }
    }

    for remote_mod in &sync_package.mods {
        if !enabled_set.contains(remote_mod.unique_id.as_str()) {
            local_missing.push(remote_mod.clone());
        }
    }

    local_missing.sort_by(|a, b| a.name.cmp(&b.name));
    remote_missing.sort_by(|a, b| a.name.cmp(&b.name));

    let total_changes = local_missing.len() + remote_missing.len() + version_mismatch.len();

    let summary = if total_changes == 0 {
        "Profiles are fully matched, no sync needed".to_string()
    } else {
        format!("Found {} differences: {} local missing, {} remote missing, {} version mismatches",
            total_changes, local_missing.len(), remote_missing.len(), version_mismatch.len())
    };

    Ok(ProfileSyncDiff {
        local_missing,
        remote_missing,
        version_mismatch,
        common_mods,
        total_changes,
        summary,
    })
}

#[tauri::command]
pub fn export_sync_environment(
    game_path: String,
    host_name: String,
    export_path: String,
) -> Result<String, String> {
    let mods_path = PathBuf::from(&game_path).join("Mods");

    if !mods_path.exists() {
        return Err("Mods folder does not exist".to_string());
    }

    let all_mods = crate::profiles::scan_mods_for_profiles(&game_path);

    let mut sync_package = SyncPackage {
        version: "2.0.0".to_string(),
        host_name,
        profile_name: String::new(),
        created_at: chrono::Utc::now().to_rfc3339(),
        mods: Vec::new(),
        configs: HashMap::new(),
    };

    for mod_info in &all_mods {
        let mod_path = PathBuf::from(&mod_info.folder_path);

        let folder_name = mod_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        let is_disabled = folder_name.starts_with('.') && !folder_name.starts_with("..");

        let url = find_mod_url_recursive(&mods_path, &mod_info.unique_id);

        let config_path = mod_path.join("config.json");
        if config_path.exists() {
            let config_content = fs::read_to_string(&config_path).unwrap_or_default();
            sync_package.configs.insert(mod_info.unique_id.clone(), config_content);
        }

        sync_package.mods.push(SyncModEntry {
            name: mod_info.name.clone(),
            unique_id: mod_info.unique_id.clone(),
            version: mod_info.version.clone(),
            author: mod_info.author.clone(),
            url,
            enabled: !is_disabled,
        });
    }

    let json_content = serde_json::to_string_pretty(&sync_package)
        .map_err(|e| format!("Failed to serialize: {}", e))?;

    fs::write(&export_path, json_content)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(export_path)
}

#[tauri::command]
pub fn import_sync_environment(import_path: String, game_path: String) -> Result<SyncDiff, String> {
    let content = fs::read_to_string(&import_path)
        .map_err(|e| format!("Failed to read sync file: {}", e))?;

    let sync_package: SyncPackage = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse sync file: {}", e))?;

    let all_mods = crate::profiles::scan_mods_for_profiles(&game_path);
    let mut installed_mods: HashMap<String, (String, PathBuf)> = HashMap::new();

    for mod_info in &all_mods {
        installed_mods.insert(
            mod_info.unique_id.clone(),
            (mod_info.version.clone(), PathBuf::from(&mod_info.folder_path)),
        );
    }

    let mut missing_mods = Vec::new();
    let mut version_mismatch = Vec::new();
    let mut config_diffs = Vec::new();

    for mod_entry in &sync_package.mods {
        if let Some((current_version, _)) = installed_mods.get(&mod_entry.unique_id) {
            if current_version != &mod_entry.version {
                version_mismatch.push(VersionMismatch {
                    mod_entry: mod_entry.clone(),
                    current_version: current_version.clone(),
                    required_version: mod_entry.version.clone(),
                });
            }
        } else {
            missing_mods.push(mod_entry.clone());
        }
    }

    let extra_mods: Vec<SyncModEntry> = installed_mods
        .keys()
        .filter(|id| !sync_package.mods.iter().any(|m| &m.unique_id == *id))
        .filter_map(|id| {
            installed_mods.get(id).map(|(version, _)| SyncModEntry {
                name: id.clone(),
                unique_id: id.clone(),
                version: version.clone(),
                author: String::new(),
                url: None,
                enabled: true,
            })
        })
        .collect();

    for mod_entry in &sync_package.mods {
        if let Some(config_content) = sync_package.configs.get(&mod_entry.unique_id) {
            if let Some((_, mod_path)) = installed_mods.get(&mod_entry.unique_id) {
                let config_path = mod_path.join("config.json");

                let status = if config_path.exists() {
                    let existing = fs::read_to_string(&config_path).unwrap_or_default();
                    if &existing == config_content {
                        "matched".to_string()
                    } else {
                        "mismatch".to_string()
                    }
                } else {
                    "missing".to_string()
                };

                config_diffs.push(ConfigDiff {
                    mod_name: mod_entry.name.clone(),
                    config_file: mod_entry.unique_id.clone(),
                    status,
                });
            }
        }
    }

    let total_changes = missing_mods.len() + version_mismatch.len() +
        config_diffs.iter().filter(|c| c.status != "matched").count();

    let summary = if total_changes == 0 {
        "Environment fully matched, no sync needed".to_string()
    } else {
        format!("Found {} differences: {} missing MODs, {} version mismatches, {} config differences",
            total_changes,
            missing_mods.len(),
            version_mismatch.len(),
            config_diffs.iter().filter(|c| c.status != "matched").count())
    };

    Ok(SyncDiff {
        missing_mods,
        version_mismatch,
        extra_mods,
        config_diffs,
        total_changes,
        summary,
    })
}

#[tauri::command]
pub fn apply_sync_environment(
    sync_package_path: String,
    game_path: String,
) -> Result<SyncApplyResult, String> {
    let content = fs::read_to_string(&sync_package_path)
        .map_err(|e| format!("Failed to read sync file: {}", e))?;

    let sync_package: SyncPackage = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse sync file: {}", e))?;

    let mods_path = PathBuf::from(&game_path).join("Mods");

    if !mods_path.exists() {
        fs::create_dir_all(&mods_path)
            .map_err(|e| format!("Failed to create Mods folder: {}", e))?;
    }

    let all_mods = crate::profiles::scan_mods_for_profiles(&game_path);
    let installed_map: HashMap<String, PathBuf> = all_mods.iter()
        .map(|m| (m.unique_id.clone(), PathBuf::from(&m.folder_path)))
        .collect();

    let mut applied_mods = Vec::new();
    let mut failed_mods = Vec::new();
    let mut configs_applied = Vec::new();

    for mod_entry in &sync_package.mods {
        if let Some(mod_path) = installed_map.get(&mod_entry.unique_id) {
            applied_mods.push(mod_entry.name.clone());

            if let Some(config_content) = sync_package.configs.get(&mod_entry.unique_id) {
                let config_path = mod_path.join("config.json");
                fs::write(&config_path, config_content)
                    .map_err(|e| format!("Failed to write config file: {}", e))?;

                configs_applied.push(mod_entry.name.clone());
            }
        } else {
            if let Some(ref url) = mod_entry.url {
                failed_mods.push(format!("{} (download: {})", mod_entry.name, url));
            } else {
                failed_mods.push(mod_entry.name.clone());
            }
        }
    }

    let message = if failed_mods.is_empty() {
        format!("Successfully synced {} MODs and {} config files", applied_mods.len(), configs_applied.len())
    } else {
        format!("Partial sync: {} succeeded, {} need manual download", applied_mods.len(), failed_mods.len())
    };

    Ok(SyncApplyResult {
        success: failed_mods.is_empty(),
        applied_mods,
        failed_mods,
        configs_applied,
        message,
    })
}

#[tauri::command]
pub fn open_save_dialog(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let result = app.dialog().file()
        .set_title("Save sync environment file")
        .add_filter("SVL Sync", &["svl_sync"])
        .blocking_save_file();

    if let Some(path) = result {
        let path_str = match path {
            FilePath::Path(p) => p.to_string_lossy().to_string(),
            FilePath::Url(u) => u.to_string(),
        };
        if !path_str.ends_with(".svl_sync") {
            Ok(Some(format!("{}.svl_sync", path_str)))
        } else {
            Ok(Some(path_str))
        }
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn open_open_dialog(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let result = app.dialog().file()
        .set_title("Select sync environment file")
        .add_filter("SVL Sync", &["svl_sync"])
        .blocking_pick_file();

    if let Some(path) = result {
        let path_str = match path {
            FilePath::Path(p) => p.to_string_lossy().to_string(),
            FilePath::Url(u) => u.to_string(),
        };
        Ok(Some(path_str))
    } else {
        Ok(None)
    }
}
