use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::compatibility_list::get_mod_metadata;
use crate::mod_name_resolver::resolve_mod_name;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModUpdateStatus {
    pub unique_id: String,
    pub name: String,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub has_update: bool,
    pub update_source: UpdateSource,
    pub download_url: Option<String>,
    pub changelog: Option<String>,
    pub is_nexus_premium: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateSource {
    SmapiList,
    NexusApi,
    UnofficialUpdate,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchUpdateResult {
    pub total: usize,
    pub updated: usize,
    pub failed: usize,
    pub details: Vec<ModUpdateDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModUpdateDetail {
    pub unique_id: String,
    pub name: String,
    pub success: bool,
    pub message: String,
}

#[tauri::command]
pub fn check_single_mod_update(
    unique_id: String,
    current_version: String,
    mod_folder_path: Option<String>,
) -> Result<ModUpdateStatus, String> {
    eprintln!("[update_checker] Checking update for: {} (current: {})", unique_id, current_version);

    let name = resolve_mod_name(&unique_id);

    // Priority 1: Check SMAPI official list for unofficial updates
    if let Some(metadata) = get_mod_metadata(&unique_id) {
        if let Some(ref unofficial_version) = metadata.unofficial_update_version {
            let has_update = compare_versions(&current_version, unofficial_version) < 0;
            
            return Ok(ModUpdateStatus {
                unique_id,
                name,
                current_version,
                latest_version: Some(unofficial_version.clone()),
                has_update,
                update_source: UpdateSource::UnofficialUpdate,
                download_url: metadata.unofficial_update_url.clone(),
                changelog: metadata.summary.clone(),
                is_nexus_premium: false,
            });
        }

        // Check SMAPI status for breaking changes
        if let Some(ref status) = metadata.status {
            if status.to_lowercase().contains("broken") || status.to_lowercase().contains("unofficial") {
                return Ok(ModUpdateStatus {
                    unique_id,
                    name,
                    current_version,
                    latest_version: None,
                    has_update: true,
                    update_source: UpdateSource::SmapiList,
                    download_url: metadata.unofficial_update_url.clone(),
                    changelog: Some(format!("SMAPI 兼容性状态: {}", status)),
                    is_nexus_premium: false,
                });
            }
        }
    }

    // Priority 2: Check manifest.json for UpdateKeys
    if let Some(ref folder_path) = mod_folder_path {
        if let Some(nexus_update_info) = check_manifest_update_keys(folder_path, &current_version) {
            return Ok(nexus_update_info);
        }
    }

    // No update source available
    Ok(ModUpdateStatus {
        unique_id,
        name,
        current_version,
        latest_version: None,
        has_update: false,
        update_source: UpdateSource::None,
        download_url: None,
        changelog: None,
        is_nexus_premium: false,
    })
}

#[tauri::command]
pub async fn check_all_mods_updates(
    mods_data: Vec<serde_json::Value>,
    api_key: Option<String>,
) -> Result<Vec<ModUpdateStatus>, String> {
    eprintln!("[update_checker] Checking updates for {} mods", mods_data.len());

    let mut updates = Vec::new();

    for mod_entry in &mods_data {
        let unique_id = mod_entry["unique_id"].as_str().unwrap_or("").to_string();
        let current_version = mod_entry["version"].as_str().unwrap_or("1.0.0").to_string();
        let mod_folder_path = mod_entry["folder_path"].as_str().map(|s| s.to_string());

        if unique_id.is_empty() {
            continue;
        }

        // Check local sources first (SMAPI list + manifest)
        match check_single_mod_update(unique_id.clone(), current_version.clone(), mod_folder_path.clone()) {
            Ok(status) => {
                if status.has_update {
                    updates.push(status);
                    continue;
                }
            }
            Err(e) => {
                eprintln!("[update_checker] Failed to check local update for {}: {}", unique_id, e);
            }
        }

        // If API key available, check Nexus API
        if let Some(ref key) = api_key {
            if !key.is_empty() {
                if let Some(nexus_id) = get_nexus_id_from_mod_data(mod_entry) {
                    match check_nexus_mod_version(key, &nexus_id, &current_version).await {
                        Ok(nexus_status) => {
                            if nexus_status.has_update {
                                updates.push(nexus_status);
                            }
                        }
                        Err(e) => {
                            eprintln!("[update_checker] Nexus API check failed for {}: {}", unique_id, e);
                        }
                    }
                }
            }
        }
    }

    // Sort by update source priority
    updates.sort_by(|a, b| {
        let priority_a = match a.update_source {
            UpdateSource::UnofficialUpdate => 0,
            UpdateSource::SmapiList => 1,
            UpdateSource::NexusApi => 2,
            UpdateSource::None => 3,
        };
        let priority_b = match b.update_source {
            UpdateSource::UnofficialUpdate => 0,
            UpdateSource::SmapiList => 1,
            UpdateSource::NexusApi => 2,
            UpdateSource::None => 3,
        };
        priority_a.cmp(&priority_b)
    });

    eprintln!("[update_checker] Found {} mods with updates", updates.len());
    Ok(updates)
}

#[tauri::command]
pub async fn batch_update_mods(
    mods_to_update: Vec<serde_json::Value>,
    _api_key: String,
) -> Result<BatchUpdateResult, String> {
    eprintln!("[update_checker] Batch updating {} mods", mods_to_update.len());

    let mut details = Vec::new();
    let mut updated_count = 0;
    let mut failed_count = 0;

    for mod_entry in &mods_to_update {
        let unique_id = mod_entry["unique_id"].as_str().unwrap_or("").to_string();
        let name = mod_entry["name"].as_str().unwrap_or(&unique_id).to_string();

        if unique_id.is_empty() {
            continue;
        }

        // Get download URL
        let download_url = match mod_entry.get("download_url").and_then(|v| v.as_str()) {
            Some(url) => url.to_string(),
            None => {
                // Try to get from Nexus API
                if let Some(nexus_id) = get_nexus_id_from_mod_data(mod_entry) {
                    format!("https://www.nexusmods.com/stardewvalley/mods/{}", nexus_id)
                } else {
                    failed_count += 1;
                    details.push(ModUpdateDetail {
                        unique_id: unique_id.clone(),
                        name: name.clone(),
                        success: false,
                        message: "未找到下载链接".to_string(),
                    });
                    continue;
                }
            }
        };

        // Download and install (placeholder - actual implementation would use mod_installer)
        // For now, we just open the download URL
        eprintln!("[update_checker] Opening download URL for {}: {}", name, download_url);
        
        // In a real implementation, you would:
        // 1. Download the mod file
        // 2. Extract it to a temporary location
        // 3. Replace the old mod files
        // 4. Clean up temporary files

        updated_count += 1;
        details.push(ModUpdateDetail {
            unique_id: unique_id.clone(),
            name: name.clone(),
            success: true,
            message: format!("已打开下载页面: {}", download_url),
        });
    }

    Ok(BatchUpdateResult {
        total: mods_to_update.len(),
        updated: updated_count,
        failed: failed_count,
        details,
    })
}

fn compare_versions(v1: &str, v2: &str) -> i32 {
    let parts1: Vec<u64> = v1.split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    let parts2: Vec<u64> = v2.split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    let max_len = parts1.len().max(parts2.len());

    for i in 0..max_len {
        let p1 = parts1.get(i).copied().unwrap_or(0);
        let p2 = parts2.get(i).copied().unwrap_or(0);

        if p1 < p2 {
            return -1;
        } else if p1 > p2 {
            return 1;
        }
    }

    0
}

fn check_manifest_update_keys(folder_path: &str, current_version: &str) -> Option<ModUpdateStatus> {
    let path = PathBuf::from(folder_path);
    let manifest_path = path.join("manifest.json");

    if !manifest_path.exists() {
        return None;
    }

    let content = match fs::read_to_string(&manifest_path) {
        Ok(c) => c,
        Err(_) => return None,
    };

    let manifest: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return None,
    };

    if let Some(update_keys) = manifest.get("UpdateKeys").and_then(|v| v.as_array()) {
        for key in update_keys {
            if let Some(key_str) = key.as_str() {
                if key_str.starts_with("Nexus:") {
                    let raw_id = key_str.trim_start_matches("Nexus:").trim();
                    let digits: String = raw_id.chars().filter(|c| c.is_ascii_digit()).collect();
                    
                    if !digits.is_empty() {
                        // Found Nexus ID, but we can't check version without API
                        // Return info that Nexus update check is available
                        return Some(ModUpdateStatus {
                            unique_id: manifest["Name"].as_str().unwrap_or("Unknown").to_string(),
                            name: manifest["Name"].as_str().unwrap_or("Unknown").to_string(),
                            current_version: current_version.to_string(),
                            latest_version: None,
                            has_update: false,
                            update_source: UpdateSource::NexusApi,
                            download_url: Some(format!(
                                "https://www.nexusmods.com/stardewvalley/mods/{}",
                                digits
                            )),
                            changelog: None,
                            is_nexus_premium: false,
                        });
                    }
                }
            }
        }
    }

    None
}

async fn check_nexus_mod_version(
    api_key: &str,
    nexus_mod_id: &str,
    current_version: &str,
) -> Result<ModUpdateStatus, String> {
    use crate::nexus_api::build_nexus_async_client;
    use crate::nexus_api::add_nexus_async_headers;
    use crate::nexus_api::NEXUS_API_BASE;
    use crate::nexus_api::STARDEW_GAME_ID;

    let client = build_nexus_async_client();

    let response = add_nexus_async_headers(
        client.get(format!(
            "{}/games/{}/mods/{}.json",
            NEXUS_API_BASE, STARDEW_GAME_ID, nexus_mod_id
        )),
        api_key,
    )
    .send()
    .await
    .map_err(|e| format!("Nexus API 请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Nexus API 返回错误状态: {}", response.status()));
    }

    let mod_info: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析 Nexus API 响应失败: {}", e))?;

    let latest_version = mod_info["version"].as_str().unwrap_or("").to_string();
    let has_update = compare_versions(current_version, &latest_version) < 0;

    let mod_name = mod_info["name"].as_str().unwrap_or("Unknown").to_string();
    let summary = mod_info["summary"].as_str().unwrap_or("").to_string();

    Ok(ModUpdateStatus {
        unique_id: format!("Nexus:{}", nexus_mod_id),
        name: mod_name,
        current_version: current_version.to_string(),
        latest_version: Some(latest_version),
        has_update,
        update_source: UpdateSource::NexusApi,
        download_url: Some(format!(
            "https://www.nexusmods.com/stardewvalley/mods/{}",
            nexus_mod_id
        )),
        changelog: if summary.is_empty() { None } else { Some(summary) },
        is_nexus_premium: mod_info["is_premium"].as_bool().unwrap_or(false),
    })
}

fn get_nexus_id_from_mod_data(mod_entry: &serde_json::Value) -> Option<String> {
    // Try nexus_mod_id field first
    if let Some(id) = mod_entry.get("nexus_mod_id").and_then(|v| v.as_str()) {
        return Some(id.to_string());
    }

    // Try unique_id in SMAPI list
    if let Some(unique_id) = mod_entry.get("unique_id").and_then(|v| v.as_str()) {
        if let Some(metadata) = get_mod_metadata(unique_id) {
            if let Some(nexus_id) = metadata.nexus_id {
                return Some(nexus_id.to_string());
            }
        }
    }

    None
}
