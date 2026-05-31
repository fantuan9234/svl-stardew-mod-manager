use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use log::info;

fn get_saves_bindings_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA").ok().map(|appdata| {
            PathBuf::from(appdata)
                .join("StardewValley")
                .join("Saves")
                .join("svl-profile-bindings.json")
        })
    }

    #[cfg(target_os = "linux")]
    {
        std::env::var("HOME").ok().map(|home| {
            PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("StardewValley")
                .join("Saves")
                .join("svl-profile-bindings.json")
        })
    }

    #[cfg(target_os = "macos")]
    {
        std::env::var("HOME").ok().map(|home| {
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("StardewValley")
                .join("Saves")
                .join("svl-profile-bindings.json")
        })
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        None::<PathBuf>
    }
}

/// SMAPI必需前置mod列表（这些mod是运行大多数mod的基础框架，不能被禁用）
/// 参考：https://www.nexusmods.com/stardewvalley/mods/ 和 SMAPI官方文档
const ESSENTIAL_MOD_IDS: &[&str] = &[
    // SMAPI核心（虽然SMAPI本身不是mod，但这些是必需的）
    "SMAPI", // SMAPI本身
    
    // 内容补丁框架 - 99%的内容mod都需要这个
    "Pathoschild.ContentPatcher", // Content Patcher (Nexus ID: 1915)
    
    // 通用框架mod - 大量mod依赖
    "spacechase0.SpaceCore", // SpaceCore (Nexus ID: 6114)
    "furyx639.ExpandedPreconditionsUtility", // Expanded Preconditions Utility - EPU (Nexus ID: 9250)
    "stardewvalleyexpanded", // Stardew Valley Expanded的前置
    
    // JSON和资产加载
    "spacechase0.JsonAssets", // Json Assets (JA) (Nexus ID: 1720)
    
    // 菜单和UI框架
    "Omegasis.LevelAutomaticSave", // Level Automatic Save (Nexus ID: 5129)
    "Cherry.Coc", // Common UI Components
    
    // 多语言支持
    "furyx639.Grammaticus", // Grammaticus (Nexus ID: 13567)
];

/// 检查mod是否是必需前置mod
fn is_essential_mod(mod_id: &str) -> bool {
    ESSENTIAL_MOD_IDS.iter().any(|&id| id.eq_ignore_ascii_case(mod_id))
}

/// 获取所有必需前置mod的unique_id列表
#[tauri::command]
pub fn get_essential_mod_ids() -> Result<Vec<String>, String> {
    Ok(ESSENTIAL_MOD_IDS.iter().map(|s| s.to_string()).collect())
}

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
    let safe_name: String = profile_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else if c == ' ' || c == '(' || c == ')' {
                '_'
            } else {
                '\0'
            }
        })
        .filter(|c| *c != '\0')
        .collect();
    let safe_name = safe_name.replace("__", "_").trim_matches('_').to_string();
    if safe_name.is_empty() {
        return profiles_dir.join("default.json");
    }
    profiles_dir.join(format!("{}.json", safe_name))
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
            let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
            for m in full_mods {
                let folder_name = std::path::PathBuf::from(&m.folder_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let unique_id = if m.unique_id.is_empty() {
                    folder_name.clone()
                } else {
                    m.unique_id.clone()
                };
                let name = if m.name.is_empty() {
                    folder_name.clone()
                } else {
                    m.name.clone()
                };

                if !seen_ids.contains(&unique_id) {
                    seen_ids.insert(unique_id.clone());
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

/// 递归扫描Mods文件夹，获取所有mod的实际路径（包括已禁用的）
fn scan_all_mod_folders(mods_path: &PathBuf) -> Vec<(String, PathBuf)> {
    scan_all_mod_folders_with_depth(mods_path, 0)
}

fn scan_all_mod_folders_with_depth(mods_path: &PathBuf, depth: usize) -> Vec<(String, PathBuf)> {
    const MAX_RECURSION_DEPTH: usize = 5;

    if depth > MAX_RECURSION_DEPTH {
        return Vec::new();
    }

    let mut results = Vec::new();

    if let Ok(entries) = fs::read_dir(mods_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("manifest.json");
            if manifest_path.exists() {
                if let Ok(content) = fs::read_to_string(&manifest_path) {
                    let normalized = crate::mod_parser::normalize_smart_quotes(&content);
                    let no_comments = crate::mod_parser::strip_json_comments(&normalized);
                    let cleaned = crate::mod_parser::remove_trailing_commas(&no_comments);
                    let cleaned = cleaned.strip_prefix('\u{FEFF}').unwrap_or(&cleaned);
                    if let Ok(manifest) = serde_json::from_str::<crate::mod_parser::ModManifest>(cleaned) {
                        if let Some(uid) = &manifest.unique_id {
                            results.push((uid.clone(), path.clone()));
                        }
                    }
                }
            }

            let nested = scan_all_mod_folders_with_depth(&path, depth + 1);
            results.extend(nested);
        }
    }
    
    results
}

/// 获取主mod文件夹的路径（用于组mod）
fn get_root_mod_path(mods_path: &PathBuf, folder_path: &PathBuf) -> PathBuf {
    let mut current = folder_path.clone();
    loop {
        let parent = match current.parent() {
            Some(p) => p.to_path_buf(),
            None => return current,
        };
        if parent == *mods_path {
            return current;
        }
        current = parent;
    }
}

pub(crate) fn apply_profile_mod_states(game_path: &str, profile: &Profile) -> Result<(), String> {
    let mods_path = PathBuf::from(game_path).join("Mods");
    if !mods_path.exists() {
        return Err("Mods folder does not exist".to_string());
    }

    info!("[profiles] ========== APPLYING PROFILE MOD STATES ==========");
    info!("[profiles] Profile: {}, enabled_mod_ids count: {}", profile.name, profile.enabled_mod_ids.len());
    info!("[profiles] Enabled mod IDs:");
    for id in &profile.enabled_mod_ids {
        info!("[profiles]   - {}", id);
    }

    // 获取完整的模组信息（包括组mod信息）
    let full_mods = crate::mod_parser::scan_mods(Some(game_path.to_string()))
        .map_err(|e| format!("Failed to scan mods: {}", e))?;
    
    info!("[profiles] Scanned {} total mods", full_mods.len());
    for m in &full_mods {
        info!("[profiles]   Mod: {} ({}), is_group={}, folder_path={}", m.name, m.unique_id, m.is_group, m.folder_path);
        if m.is_group {
            info!("[profiles]     Sub-mods:");
            for sub in &m.sub_mods {
                info!("[profiles]       - {} ({})", sub.name, sub.unique_id);
            }
        }
    }
    
    // 识别所有组mod的主mod unique_id
    let mut group_main_mod_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    // 收集组mod的子mod unique_id
    let mut group_sub_mod_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    
    for m in &full_mods {
        if m.is_group && !m.sub_mods.is_empty() {
            // 主mod在档案中应该被跳过
            group_main_mod_ids.insert(m.unique_id.clone());
            // 子mod应该被正常处理
            for sub in &m.sub_mods {
                group_sub_mod_ids.insert(sub.unique_id.clone());
            }
        }
    }

    info!("[profiles] Group main mod IDs (to skip): {:?}", group_main_mod_ids);
    info!("[profiles] Group sub-mod IDs (to handle separately): {:?}", group_sub_mod_ids);

    // 直接扫描所有mod文件夹，获取实际路径（包括已禁用的）
    let all_mod_folders = scan_all_mod_folders(&mods_path);
    
    info!("[profiles] Scanned {} mod folders for path mapping", all_mod_folders.len());
    for (uid, path) in &all_mod_folders {
        info!("[profiles]   -> {} = {}", uid, path.display());
    }
    
    // 构建 unique_id -> 实际路径的映射
    let mut id_to_path: std::collections::HashMap<String, PathBuf> = std::collections::HashMap::new();
    for (uid, path) in all_mod_folders {
        id_to_path.insert(uid, path);
    }

    // 对于不在映射中的mod（可能已被禁用），尝试扫描带.前缀的文件夹
    for m in &full_mods {
        if !id_to_path.contains_key(&m.unique_id) {
            info!("[profiles] Mod {} not found in path mapping, searching disabled folders", m.unique_id);
            let root_path = get_root_mod_path(&mods_path, &PathBuf::from(&m.folder_path));
            let disabled_path = root_path.parent()
                .map(|p| p.join(format!(".{}", root_path.file_name().and_then(|n| n.to_str()).unwrap_or(""))));
            if let Some(disabled_path) = disabled_path {
                if disabled_path.exists() {
                    if let Ok(entries) = fs::read_dir(&disabled_path) {
                        for entry in entries.flatten() {
                            let sub_path = entry.path();
                            if sub_path.is_dir() {
                                let manifest_path = sub_path.join("manifest.json");
                                if manifest_path.exists() {
                                    if let Ok(content) = fs::read_to_string(&manifest_path) {
                                        let normalized = crate::mod_parser::normalize_smart_quotes(&content);
                                        let no_comments = crate::mod_parser::strip_json_comments(&normalized);
                                        let cleaned = crate::mod_parser::remove_trailing_commas(&no_comments);
                                        let cleaned = cleaned.strip_prefix('\u{FEFF}').unwrap_or(&cleaned);
                                        if let Ok(manifest) = serde_json::from_str::<crate::mod_parser::ModManifest>(cleaned) {
                                            if let Some(uid) = &manifest.unique_id {
                                                id_to_path.insert(uid.clone(), sub_path);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    info!("[profiles] Final id_to_path mapping contains {} entries", id_to_path.len());

    // 构建 should_enable 的完整集合：档案中的 mod + group主模组对应的所有sub_mods
    let mut full_enabled_set: std::collections::HashSet<String> = profile.enabled_mod_ids.iter().cloned().collect();

    // 如果档案中包含了group主模组，自动把它的sub_mods也加入启用集合
    for main_mod_id in &profile.enabled_mod_ids {
        if group_main_mod_ids.contains(main_mod_id) {
            if let Some(m) = full_mods.iter().find(|m| m.unique_id == *main_mod_id) {
                for sub in &m.sub_mods {
                    info!("[profiles] Auto-enabling sub-mod '{}' because parent group '{}' is enabled", sub.unique_id, main_mod_id);
                    full_enabled_set.insert(sub.unique_id.clone());
                }
            }
        }
    }

    info!("[profiles] Expanded enabled set to {} entries (including sub-mods of groups)", full_enabled_set.len());

    let enabled_set: std::collections::HashSet<&str> = full_enabled_set.iter().map(|s| s.as_str()).collect();

    info!("[profiles] Applying profile '{}' with {} enabled mods", profile.name, enabled_set.len());
    info!("[profiles] Group main mods (to skip): {:?}", group_main_mod_ids);
    info!("[profiles] Group sub-mod ids: {:?}", group_sub_mod_ids);
    info!("[profiles] Found {} mod folders", id_to_path.len());

    // 先处理组mod的子mod（只处理子mod，不处理主mod）
    for (uid, path) in &id_to_path {
        // 如果不是组mod的子mod，跳过
        if !group_sub_mod_ids.contains(uid.as_str()) {
            continue;
        }

        let should_enable = enabled_set.contains(uid.as_str());
        
        if !path.exists() {
            info!("[profiles] Skipping {} - path does not exist", uid);
            continue;
        }

        let folder_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let is_currently_disabled = folder_name.starts_with('.') && !folder_name.starts_with("..");
        info!("[profiles] Group sub-mod: {}, should_enable={}, folder_name='{}', is_disabled={}", uid, should_enable, folder_name, is_currently_disabled);

        if should_enable && is_currently_disabled {
            let clean_name = &folder_name[1..];
            let new_path = path.parent()
                .map(|p| p.join(clean_name))
                .unwrap_or_else(|| path.clone());
            if new_path.exists() && new_path != *path {
                info!("[profiles] Cannot enable {} - target exists", uid);
                continue;
            }
            info!("[profiles] Enabling {}: {} -> {}", uid, path.display(), new_path.display());
            if let Err(e) = fs::rename(path, &new_path) {
                info!("[profiles] Failed to enable mod {}: {}", uid, e);
            }
        } else if !should_enable && !is_currently_disabled {
            let new_name = format!(".{}", folder_name);
            let new_path = path.parent()
                .map(|p| p.join(&new_name))
                .unwrap_or_else(|| path.clone());
            if new_path.exists() {
                info!("[profiles] Cannot disable {} - target exists", uid);
                continue;
            }
            info!("[profiles] Disabling {}: {} -> {}", uid, path.display(), new_path.display());
            if let Err(e) = fs::rename(path, &new_path) {
                info!("[profiles] Failed to disable mod {}: {}", uid, e);
            }
        }
    }

    // 再处理非组mod和组mod的主mod（普通mod、必需的mod、以及组mod的主mod文件夹）
    // 注意：组mod的子mod已在上方处理，这里只跳过子mod，主mod需要正常处理
    info!("[profiles] === Processing non-group mods and group main mods ===");
    for mod_info in &full_mods {
        if group_sub_mod_ids.contains(&mod_info.unique_id) {
            info!("[profiles] Skipping group sub-mod (already processed): {} ({})", mod_info.unique_id, mod_info.name);
            continue;
        }

        let should_enable = enabled_set.contains(mod_info.unique_id.as_str());
        
        // 使用实际的文件夹路径
        let mod_path = id_to_path.get(&mod_info.unique_id);
        let mod_path = match mod_path {
            Some(p) => p.clone(),
            None => {
                // 可能已经被禁用了，尝试查找带.前缀的路径
                info!("[profiles] Mod {} path not found in mapping, trying disabled path", mod_info.unique_id);
                continue;
            }
        };

        info!("[profiles] Checking non-group mod: {} ({}), should_enable={}, actual_path={}", mod_info.unique_id, mod_info.name, should_enable, mod_path.display());

        if !mod_path.exists() {
            info!("[profiles]   Skipping - path does not exist: {}", mod_path.display());
            continue;
        }

        let folder_name = mod_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let is_currently_disabled = folder_name.starts_with('.') && !folder_name.starts_with("..");
        info!("[profiles]   folder_name='{}', is_disabled={}, is_required={}", folder_name, is_currently_disabled, mod_info.is_required);

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
        } else if !should_enable && !is_currently_disabled {
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
        } else {
            info!("[profiles]   No action needed (should_enable={}, is_disabled={}, is_required={})", should_enable, is_currently_disabled, mod_info.is_required);
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

    // 保存当前mod状态，以便退出档案时恢复
    let current_mods = crate::mod_parser::scan_mods(Some(game_path.to_string()))
        .map_err(|e| format!("应用档案前扫描当前mod失败: {}", e))?;
    let current_states: Vec<(String, bool)> = current_mods.iter()
        .map(|m| (m.unique_id.clone(), m.enabled))
        .collect();
    
    let states_dir = PathBuf::from(game_path).join("SVL_Data");
    if !states_dir.exists() {
        let _ = fs::create_dir_all(&states_dir);
    }
    let pre_profile_path = states_dir.join("pre_profile_state.json");
    if let Ok(json) = serde_json::to_string(&current_states) {
        let _ = fs::write(&pre_profile_path, json);
        info!("[profiles] Saved pre-profile state to {:?}", pre_profile_path);
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_profile_file_path_no_collision_with_imported_suffix() {
        let dir = PathBuf::from("/tmp/profiles");
        let path1 = get_profile_file_path("MyProfile", &dir);
        let path2 = get_profile_file_path("MyProfile (imported)", &dir);
        assert_ne!(path1, path2, "Different profile names should not map to same file. 'MyProfile' and 'MyProfile (imported)' must produce different filenames.");
    }

    #[test]
    fn test_get_profile_file_path_preserves_distinguishing_info() {
        let dir = PathBuf::from("/tmp/profiles");
        let path = get_profile_file_path("My Profile (imported)", &dir);
        let filename = path.file_stem().unwrap().to_str().unwrap();
        assert_ne!(filename, "MyProfile", "Imported profile should have a different filename than the original, not just strip all special chars");
    }

    #[test]
    fn test_get_profile_file_path_valid_filename() {
        let dir = PathBuf::from("/tmp/profiles");
        let path = get_profile_file_path("Test Profile", &dir);
        let filename = path.file_name().unwrap().to_str().unwrap();
        assert!(filename.ends_with(".json"), "Filename should end with .json");
        assert!(!filename.contains(' '), "Filename should not contain spaces");
    }
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
pub fn profile_clear_active(
    app: AppHandle,
    game_path: String,
) -> Result<bool, String> {
    // 读取保存的前置状态
    let states_dir = PathBuf::from(&game_path).join("SVL_Data");
    let pre_profile_path = states_dir.join("pre_profile_state.json");
    
    if pre_profile_path.exists() {
        if let Ok(content) = fs::read_to_string(&pre_profile_path) {
            if let Ok(states) = serde_json::from_str::<Vec<(String, bool)>>(&content) {
                info!("[profiles] Restoring {} mod states from pre-profile", states.len());
                
                // 应用前置状态
                let mods_path = PathBuf::from(&game_path).join("Mods");
                if mods_path.exists() {
                    let all_mod_folders = scan_all_mod_folders(&mods_path);
                    let mut id_to_path: std::collections::HashMap<String, PathBuf> = std::collections::HashMap::new();
                    for (uid, path) in all_mod_folders {
                        id_to_path.insert(uid, path);
                    }
                    
                    for (mod_id, should_enabled) in &states {
                        if let Some(mod_path) = id_to_path.get(mod_id) {
                            if !mod_path.exists() {
                                continue;
                            }
                            
                            let folder_name = mod_path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("")
                                .to_string();
                            
                            let is_currently_disabled = folder_name.starts_with('.') && !folder_name.starts_with("..");
                            
                            if *should_enabled && is_currently_disabled {
                                let clean_name = &folder_name[1..];
                                let new_path = mod_path.parent()
                                    .map(|p| p.join(clean_name))
                                    .unwrap_or_else(|| mod_path.clone());
                                if !new_path.exists() || new_path == *mod_path {
                                    let _ = fs::rename(mod_path, &new_path);
                                    info!("[profiles] Restored mod {} (enabled)", mod_id);
                                }
                            } else if !should_enabled && !is_currently_disabled {
                                let new_name = format!(".{}", folder_name);
                                let new_path = mod_path.parent()
                                    .map(|p| p.join(&new_name))
                                    .unwrap_or_else(|| mod_path.clone());
                                if !new_path.exists() {
                                    let _ = fs::rename(mod_path, &new_path);
                                    info!("[profiles] Restored mod {} (disabled)", mod_id);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    let active_path = get_active_profile_file_path(&game_path);
    if active_path.exists() {
        fs::remove_file(&active_path)
            .map_err(|e| format!("Failed to clear active profile: {}", e))?;
    }
    
    // 删除前置状态文件
    if pre_profile_path.exists() {
        let _ = fs::remove_file(&pre_profile_path);
    }

    let _ = app.emit("profile-changed", "");

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
    let bindings_path = get_saves_bindings_path();
    if let Some(path) = bindings_path {
        if path.exists() {
            let content = fs::read_to_string(&path)
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
    let bindings_path = get_saves_bindings_path()
        .ok_or("Cannot determine saves directory path")?;

    let saves_dir = bindings_path.parent()
        .ok_or("Cannot determine saves directory")?;
    if !saves_dir.exists() {
        fs::create_dir_all(saves_dir)
            .map_err(|e| format!("Failed to create saves directory: {}", e))?;
    }

    let mut bindings: HashMap<String, String> = if bindings_path.exists() {
        let content = fs::read_to_string(&bindings_path)
            .map_err(|e| format!("Failed to read bindings: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("解析档案绑定文件失败 ({}): {}", bindings_path.display(), e))?
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
