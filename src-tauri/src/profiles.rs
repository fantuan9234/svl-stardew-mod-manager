use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use log::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub is_protected: bool,
    pub enabled_mod_ids: Vec<String>,
    pub created_at: String,
    pub last_used: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileListItem {
    pub name: String,
    pub is_protected: bool,
    pub is_active: bool,
    pub total_mods: usize,
    pub enabled_count: usize,
    pub created_at: String,
    pub last_used: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    pub unique_id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub is_required: bool,
    pub folder_path: String,
    pub folder_name: String,
}

pub fn get_profiles_dir(game_path: &str) -> Result<PathBuf, String> {
    let game_dir = PathBuf::from(game_path);
    let profiles_dir = game_dir.join("svl-profiles");

    if !profiles_dir.exists() {
        fs::create_dir_all(&profiles_dir)
            .map_err(|e| format!("Failed to create profiles directory: {}", e))?;
    }

    Ok(profiles_dir)
}

fn get_profile_file_path(profile_name: &str, profiles_dir: &PathBuf) -> PathBuf {
    profiles_dir.join(format!("{}.json", profile_name))
}

fn get_active_profile_file_path(game_path: &str) -> PathBuf {
    PathBuf::from(game_path).join("svl-profiles").join("_active.txt")
}

pub fn load_profile(game_path: &str, profile_name: &str) -> Result<Profile, String> {
    let profiles_dir = get_profiles_dir(game_path)?;
    let profile_path = get_profile_file_path(profile_name, &profiles_dir);

    if !profile_path.exists() {
        return Err(format!("Profile '{}' does not exist", profile_name));
    }

    let content = fs::read_to_string(&profile_path)
        .map_err(|e| format!("Failed to read profile: {}", e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse profile: {}", e))
}

pub(crate) fn save_profile(profile: &Profile, game_path: &str) -> Result<(), String> {
    let profiles_dir = get_profiles_dir(game_path)?;
    let profile_path = get_profile_file_path(&profile.name, &profiles_dir);

    let json = serde_json::to_string_pretty(profile)
        .map_err(|e| format!("Failed to serialize profile: {}", e))?;

    fs::write(&profile_path, json)
        .map_err(|e| format!("Failed to write profile: {}", e))?;

    Ok(())
}

pub fn get_active_profile_name(game_path: &str) -> Option<String> {
    let active_path = get_active_profile_file_path(game_path);
    if active_path.exists() {
        if let Ok(content) = fs::read_to_string(&active_path) {
            let name = content.trim().to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}

pub(crate) fn set_active_profile_name(game_path: &str, profile_name: &str) -> Result<(), String> {
    let active_path = get_active_profile_file_path(game_path);
    if let Some(parent) = active_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create profiles directory: {}", e))?;
        }
    }
    fs::write(&active_path, profile_name)
        .map_err(|e| format!("Failed to save active profile: {}", e))?;
    Ok(())
}

pub fn scan_mods_for_profiles(game_path: &str) -> Vec<ModInfo> {
    match crate::mod_parser::scan_mods(Some(game_path.to_string())) {
        Ok(full_mods) => {
            info!("[profiles] scanned {} mods", full_mods.len());
            let mut result = Vec::new();
            for m in full_mods {
                let folder_name = std::path::PathBuf::from(&m.folder_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let unique_id = if m.unique_id.is_empty() {
                    folder_name.clone()
                } else {
                    m.unique_id
                };
                let name = if m.name.is_empty() {
                    folder_name.clone()
                } else {
                    m.name
                };

                if m.is_group && !m.sub_mods.is_empty() {
                    for sub in &m.sub_mods {
                        let sub_folder_name = std::path::PathBuf::from(&sub.folder_path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        result.push(ModInfo {
                            unique_id: if sub.unique_id.is_empty() {
                                sub_folder_name.clone()
                            } else {
                                sub.unique_id.clone()
                            },
                            name: if sub.name.is_empty() {
                                sub_folder_name.clone()
                            } else {
                                sub.name.clone()
                            },
                            version: if sub.version.is_empty() { "0.0.0".to_string() } else { sub.version.clone() },
                            author: if sub.author.is_empty() { "Unknown".to_string() } else { sub.author.clone() },
                            is_required: sub.is_required,
                            folder_path: sub.folder_path.clone(),
                            folder_name: sub_folder_name,
                        });
                    }
                } else {
                    result.push(ModInfo {
                        unique_id,
                        name,
                        version: if m.version.is_empty() { "0.0.0".to_string() } else { m.version.clone() },
                        author: if m.author.is_empty() { "Unknown".to_string() } else { m.author.clone() },
                        is_required: m.is_required,
                        folder_path: m.folder_path.clone(),
                        folder_name,
                    });
                }
            }
            result
        }
        Err(e) => {
            info!("[profiles] scan failed: {}", e);
            Vec::new()
        }
    }
}

pub(crate) fn apply_profile_mod_states(game_path: &str, profile: &Profile) -> Result<(), String> {
    let mods_path = PathBuf::from(game_path).join("Mods");
    if !mods_path.exists() {
        return Err("Mods folder does not exist".to_string());
    }

    let all_mods = scan_mods_for_profiles(game_path);
    let enabled_set: std::collections::HashSet<&str> = profile.enabled_mod_ids.iter().map(|s| s.as_str()).collect();

    info!("[profiles] Applying profile '{}' with {} enabled mods, found {} total mods", profile.name, enabled_set.len(), all_mods.len());
    for id in &enabled_set {
        info!("[profiles]   enabled: {}", id);
    }

    for mod_info in &all_mods {
        let should_enable = enabled_set.contains(mod_info.unique_id.as_str());
        let mod_path = PathBuf::from(&mod_info.folder_path);

        if !mod_path.exists() {
            info!("[profiles] Skipping {} - path does not exist: {}", mod_info.unique_id, mod_path.display());
            continue;
        }

        let folder_name = mod_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let is_currently_disabled = folder_name.starts_with('.') && !folder_name.starts_with("..");

        if should_enable && is_currently_disabled {
            let clean_name = &folder_name[1..];
            let new_path = mod_path.parent()
                .map(|p| p.join(clean_name))
                .unwrap_or_else(|| mod_path.clone());
            if new_path.exists() && new_path != mod_path {
                info!("[profiles] Cannot enable {} - target exists: {}", mod_info.unique_id, new_path.display());
                continue;
            }
            info!("[profiles] Enabling {}: {} -> {}", mod_info.unique_id, mod_path.display(), new_path.display());
            if let Err(e) = fs::rename(&mod_path, &new_path) {
                info!("[profiles] Failed to enable mod {}: {}", mod_info.unique_id, e);
            }
        } else if !should_enable && !is_currently_disabled && !mod_info.is_required {
            let new_name = format!(".{}", folder_name);
            let new_path = mod_path.parent()
                .map(|p| p.join(&new_name))
                .unwrap_or_else(|| mod_path.clone());
            if new_path.exists() {
                info!("[profiles] Cannot disable {} - target exists: {}", mod_info.unique_id, new_path.display());
                continue;
            }
            info!("[profiles] Disabling {}: {} -> {}", mod_info.unique_id, mod_path.display(), new_path.display());
            if let Err(e) = fs::rename(&mod_path, &new_path) {
                info!("[profiles] Failed to disable mod {}: {}", mod_info.unique_id, e);
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub fn profile_create(
    app: AppHandle,
    game_path: String,
    profile_name: String,
    enabled_mod_ids: Option<Vec<String>>,
) -> Result<Profile, String> {
    let profiles_dir = get_profiles_dir(&game_path)?;
    let profile_path = get_profile_file_path(&profile_name, &profiles_dir);

    if profile_path.exists() {
        return Err(format!("Profile '{}' already exists", profile_name));
    }

    let all_mods = scan_mods_for_profiles(&game_path);
    let enabled_ids = match enabled_mod_ids {
        Some(ids) => ids,
        None => all_mods.iter().map(|m| m.unique_id.clone()).collect(),
    };

    let now = chrono::Utc::now().to_rfc3339();

    let profile = Profile {
        name: profile_name,
        is_protected: false,
        enabled_mod_ids: enabled_ids,
        created_at: now.clone(),
        last_used: now,
    };

    save_profile(&profile, &game_path)?;

    let _ = app.emit("profile-changed", &profile.name);

    Ok(profile)
}

#[tauri::command]
pub fn profile_list(game_path: String) -> Result<Vec<ProfileListItem>, String> {
    let active_name = get_active_profile_name(&game_path);
    let profiles_dir = get_profiles_dir(&game_path)?;
    let all_mods = scan_mods_for_profiles(&game_path);
    let total_mod_count = all_mods.len();

    let mut profiles = Vec::new();

    if !profiles_dir.exists() {
        return Ok(profiles);
    }

    if let Ok(entries) = fs::read_dir(&profiles_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(profile) = serde_json::from_str::<Profile>(&content) {
                        let is_active = active_name.as_ref().map_or(false, |n| n == &profile.name);
                        let enabled_count = profile.enabled_mod_ids.len();

                        profiles.push(ProfileListItem {
                            name: profile.name,
                            is_protected: profile.is_protected,
                            is_active,
                            total_mods: total_mod_count,
                            enabled_count,
                            created_at: profile.created_at,
                            last_used: profile.last_used,
                        });
                    }
                }
            }
        }
    }

    profiles.sort_by(|a, b| {
        if a.is_protected && !b.is_protected {
            std::cmp::Ordering::Less
        } else if !a.is_protected && b.is_protected {
            std::cmp::Ordering::Greater
        } else {
            b.last_used.cmp(&a.last_used)
        }
    });

    Ok(profiles)
}

#[tauri::command]
pub fn profile_get_active(game_path: String) -> Result<Option<String>, String> {
    Ok(get_active_profile_name(&game_path))
}

pub fn apply_profile(game_path: &str, profile_name: &str) -> Result<Profile, String> {
    let profile = load_profile(game_path, profile_name)?;

    apply_profile_mod_states(game_path, &profile)?;

    let updated_profile = Profile {
        last_used: chrono::Utc::now().to_rfc3339(),
        ..profile
    };
    save_profile(&updated_profile, game_path)?;

    set_active_profile_name(game_path, profile_name)?;

    Ok(updated_profile)
}

#[tauri::command]
pub fn profile_switch(
    app: AppHandle,
    game_path: String,
    profile_name: String,
) -> Result<Profile, String> {
    let profile = apply_profile(&game_path, &profile_name)?;
    let _ = app.emit("profile-changed", &profile_name);
    Ok(profile)
}

#[tauri::command]
pub fn profile_delete(
    app: AppHandle,
    game_path: String,
    profile_name: String,
) -> Result<bool, String> {
    let profile = load_profile(&game_path, &profile_name)?;

    if profile.is_protected {
        return Err("Cannot delete a protected profile".to_string());
    }

    let active_name = get_active_profile_name(&game_path);
    if active_name.as_ref() == Some(&profile_name) {
        return Err("Cannot delete the active profile. Switch to another profile first.".to_string());
    }

    let profiles_dir = get_profiles_dir(&game_path)?;
    let profile_path = get_profile_file_path(&profile_name, &profiles_dir);

    if profile_path.exists() {
        fs::remove_file(&profile_path)
            .map_err(|e| format!("Failed to delete profile: {}", e))?;
    }

    let _ = app.emit("profile-changed", &profile_name);

    Ok(true)
}

#[tauri::command]
pub fn profile_update_mods(
    game_path: String,
    profile_name: String,
    enabled_mod_ids: Vec<String>,
) -> Result<Profile, String> {
    let mut profile = load_profile(&game_path, &profile_name)?;

    profile.enabled_mod_ids = enabled_mod_ids;
    profile.last_used = chrono::Utc::now().to_rfc3339();

    save_profile(&profile, &game_path)?;

    let active_name = get_active_profile_name(&game_path);
    if active_name.as_ref() == Some(&profile_name) {
        apply_profile_mod_states(&game_path, &profile)?;
    }

    Ok(profile)
}

#[tauri::command]
pub fn profile_toggle_mod(
    game_path: String,
    profile_name: String,
    mod_id: String,
    enabled: bool,
) -> Result<Profile, String> {
    let mut profile = load_profile(&game_path, &profile_name)?;

    if enabled {
        if !profile.enabled_mod_ids.contains(&mod_id) {
            profile.enabled_mod_ids.push(mod_id);
        }
    } else {
        profile.enabled_mod_ids.retain(|id| id != &mod_id);
    }

    profile.last_used = chrono::Utc::now().to_rfc3339();
    save_profile(&profile, &game_path)?;

    let active_name = get_active_profile_name(&game_path);
    if active_name.as_ref() == Some(&profile_name) {
        apply_profile_mod_states(&game_path, &profile)?;
    }

    Ok(profile)
}

#[tauri::command]
pub fn profile_get_mod_states(
    game_path: String,
    profile_name: String,
) -> Result<HashMap<String, bool>, String> {
    let profile = load_profile(&game_path, &profile_name)?;
    let all_mods = scan_mods_for_profiles(&game_path);

    let mut states = HashMap::new();
    let enabled_set: std::collections::HashSet<&str> = profile.enabled_mod_ids.iter().map(|s| s.as_str()).collect();

    for m in &all_mods {
        states.insert(m.unique_id.clone(), enabled_set.contains(m.unique_id.as_str()));
    }

    Ok(states)
}

#[tauri::command]
pub fn profile_clear_active(game_path: String) -> Result<bool, String> {
    let active_path = get_active_profile_file_path(&game_path);
    if active_path.exists() {
        fs::remove_file(&active_path)
            .map_err(|e| format!("Failed to clear active profile: {}", e))?;
    }
    Ok(true)
}

#[tauri::command]
pub fn profile_copy(
    game_path: String,
    from_profile: String,
    new_profile_name: String,
) -> Result<Profile, String> {
    let from = load_profile(&game_path, &from_profile)?;

    let profiles_dir = get_profiles_dir(&game_path)?;
    let new_path = get_profile_file_path(&new_profile_name, &profiles_dir);
    if new_path.exists() {
        return Err(format!("Profile '{}' already exists", new_profile_name));
    }

    let now = chrono::Utc::now().to_rfc3339();
    let new_profile = Profile {
        name: new_profile_name,
        is_protected: false,
        enabled_mod_ids: from.enabled_mod_ids.clone(),
        created_at: now.clone(),
        last_used: now,
    };

    save_profile(&new_profile, &game_path)?;

    Ok(new_profile)
}

#[tauri::command]
pub fn profile_export(
    game_path: String,
    profile_name: String,
    export_path: String,
) -> Result<bool, String> {
    let profile = load_profile(&game_path, &profile_name)?;

    let json = serde_json::to_string_pretty(&profile)
        .map_err(|e| format!("Failed to serialize profile: {}", e))?;

    let export_file = PathBuf::from(&export_path);
    if let Some(parent) = export_file.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create export directory: {}", e))?;
        }
    }

    fs::write(&export_file, json)
        .map_err(|e| format!("Failed to write export file: {}", e))?;

    Ok(true)
}

#[tauri::command]
pub fn profile_import(
    app: AppHandle,
    game_path: String,
    import_path: String,
) -> Result<Profile, String> {
    let import_file = PathBuf::from(&import_path);

    if !import_file.exists() {
        return Err(format!("Import file does not exist: {}", import_path));
    }

    let content = fs::read_to_string(&import_file)
        .map_err(|e| format!("Failed to read import file: {}", e))?;

    let mut profile: Profile = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse profile file: {}", e))?;

    let profiles_dir = get_profiles_dir(&game_path)?;
    let existing_path = get_profile_file_path(&profile.name, &profiles_dir);
    if existing_path.exists() {
        profile.name = format!("{} (imported)", profile.name);
    }

    let now = chrono::Utc::now().to_rfc3339();
    profile.is_protected = false;
    profile.created_at = now.clone();
    profile.last_used = now;

    save_profile(&profile, &game_path)?;

    let _ = app.emit("profile-changed", &profile.name);

    Ok(profile)
}

#[tauri::command]
pub fn profile_scan_mods(game_path: String) -> Result<Vec<ModInfo>, String> {
    let mods = scan_mods_for_profiles(&game_path);
    Ok(mods)
}

#[tauri::command]
pub fn get_profile_bindings() -> Result<HashMap<String, String>, String> {
    if let Some(appdata) = std::env::var("APPDATA").ok() {
        let bindings_path = PathBuf::from(appdata)
            .join("StardewValley")
            .join("Saves")
            .join("svl-profile-bindings.json");
        if bindings_path.exists() {
            let content = fs::read_to_string(&bindings_path)
                .map_err(|e| format!("Failed to read bindings: {}", e))?;
            let map: HashMap<String, String> = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse bindings: {}", e))?;
            return Ok(map);
        }
    }
    Ok(HashMap::new())
}

#[tauri::command]
pub fn set_profile_binding(save_folder_name: String, profile_name: Option<String>) -> Result<bool, String> {
    let bindings_path = if let Some(appdata) = std::env::var("APPDATA").ok() {
        let saves_dir = PathBuf::from(appdata).join("StardewValley").join("Saves");
        if !saves_dir.exists() {
            fs::create_dir_all(&saves_dir)
                .map_err(|e| format!("Failed to create saves directory: {}", e))?;
        }
        saves_dir.join("svl-profile-bindings.json")
    } else {
        return Err("Cannot determine APPDATA path".to_string());
    };

    let mut bindings: HashMap<String, String> = if bindings_path.exists() {
        let content = fs::read_to_string(&bindings_path)
            .map_err(|e| format!("Failed to read bindings: {}", e))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    };

    if let Some(name) = profile_name {
        bindings.insert(save_folder_name, name);
    } else {
        bindings.remove(&save_folder_name);
    }

    let json = serde_json::to_string_pretty(&bindings)
        .map_err(|e| format!("Failed to serialize bindings: {}", e))?;
    fs::write(&bindings_path, json)
        .map_err(|e| format!("Failed to write bindings: {}", e))?;

    Ok(true)
}
