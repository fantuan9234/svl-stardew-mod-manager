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
    pub nexus_mod_id: Option<String>,
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
pub async fn check_single_mod_update(
    unique_id: String,
    current_version: String,
    mod_folder_path: Option<String>,
    api_key: Option<String>,
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
                nexus_mod_id: metadata.nexus_id.map(|id| id.to_string()),
                changelog: metadata.summary.clone(),
                is_nexus_premium: false,
            });
        }

        // Check SMAPI main version against local version
        if let Some(ref main_version) = metadata.main_version {
            if compare_versions(&current_version, main_version) < 0 {
                let download_url = metadata.main_url.clone()
                    .or_else(|| metadata.nexus_id.map(|id| format!("https://www.nexusmods.com/stardewvalley/mods/{}", id)));
                
                return Ok(ModUpdateStatus {
                    unique_id,
                    name,
                    current_version,
                    latest_version: Some(main_version.clone()),
                    has_update: true,
                    update_source: UpdateSource::SmapiList,
                    download_url,
                    nexus_mod_id: metadata.nexus_id.map(|id| id.to_string()),
                    changelog: metadata.summary.clone(),
                    is_nexus_premium: false,
                });
            }
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
                    nexus_mod_id: metadata.nexus_id.map(|id| id.to_string()),
                    changelog: Some(format!("SMAPI 兼容性状态: {}", status)),
                    is_nexus_premium: false,
                });
            }
        }
    }

    // Priority 2: Check manifest.json for UpdateKeys
    if let Some(ref folder_path) = mod_folder_path {
        if let Some(nexus_update_info) = check_manifest_update_keys(folder_path, &current_version) {
            if nexus_update_info.has_update {
                return Ok(nexus_update_info);
            }
            if matches!(nexus_update_info.update_source, UpdateSource::NexusApi) && nexus_update_info.nexus_mod_id.is_some() {
                if let Some(ref key) = api_key {
                    if !key.is_empty() {
                        if let Some(ref nexus_id) = nexus_update_info.nexus_mod_id {
                            if let Ok(nexus_status) = check_nexus_mod_version(key, nexus_id, &current_version).await {
                                return Ok(nexus_status);
                            }
                        }
                    }
                }
                return Ok(nexus_update_info);
            }
            return Ok(nexus_update_info);
        }
    }

    // Priority 3: Try Nexus API if API key available
    if let Some(ref key) = api_key {
        if !key.is_empty() {
            let mod_data = serde_json::json!({
                "unique_id": unique_id,
                "name": name,
            });
            if let Some(nexus_id) = get_nexus_id_from_mod_data(&mod_data) {
                if let Ok(nexus_status) = check_nexus_mod_version(key, &nexus_id, &current_version).await {
                    return Ok(nexus_status);
                }
            }
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
        nexus_mod_id: None,
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
        match check_single_mod_update(unique_id.clone(), current_version.clone(), mod_folder_path.clone(), api_key.clone()).await {
            Ok(status) => {
                if status.has_update {
                    let mut enriched = false;
                    // For manifest-based Nexus updates without version info, try to enrich with API
                    if matches!(status.update_source, UpdateSource::NexusApi) && status.latest_version.is_none() {
                        if let Some(ref key) = api_key {
                            if !key.is_empty() {
                                if let Some(nexus_id) = status.nexus_mod_id.as_ref() {
                                    if let Ok(nexus_status) = check_nexus_mod_version(key, nexus_id, &current_version).await {
                                        enriched = true;
                                        if nexus_status.has_update {
                                            updates.push(nexus_status);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if !enriched {
                        updates.push(status);
                    }
                    continue;
                }

                if matches!(status.update_source, UpdateSource::NexusApi) && status.nexus_mod_id.is_some() && !status.has_update {
                    if let Some(ref key) = api_key {
                        if !key.is_empty() {
                            if let Some(nexus_id) = status.nexus_mod_id.as_ref() {
                                if let Ok(nexus_status) = check_nexus_mod_version(key, nexus_id, &current_version).await {
                                    if nexus_status.has_update {
                                        updates.push(nexus_status);
                                    }
                                    continue;
                                }
                            }
                        }
                    }
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
    app: tauri::AppHandle,
    mods_to_update: Vec<serde_json::Value>,
    api_key: String,
    mods_path: String,
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

        if let Some(nexus_id) = get_nexus_id_from_mod_data(mod_entry) {
            eprintln!("[update_checker] Auto-downloading {} from Nexus (mod_id={})", name, nexus_id);

            match crate::nexus_api::download_mod_from_nexus(
                app.clone(),
                nexus_id,
                api_key.clone(),
                Some(mods_path.clone()),
                None,
                Some(unique_id.clone()),
            ).await {
                Ok(download_result) => {
                    if download_result.success {
                        updated_count += 1;
                        details.push(ModUpdateDetail {
                            unique_id: unique_id.clone(),
                            name: name.clone(),
                            success: true,
                            message: download_result.message,
                        });
                    } else {
                        failed_count += 1;
                        details.push(ModUpdateDetail {
                            unique_id: unique_id.clone(),
                            name: name.clone(),
                            success: false,
                            message: download_result.message,
                        });
                    }
                }
                Err(e) => {
                    failed_count += 1;
                    details.push(ModUpdateDetail {
                        unique_id: unique_id.clone(),
                        name: name.clone(),
                        success: false,
                        message: format!("下载失败: {}", e),
                    });
                }
            }
        } else {
            failed_count += 1;
            details.push(ModUpdateDetail {
                unique_id: unique_id.clone(),
                name: name.clone(),
                success: false,
                message: "未找到 N 网 ID，无法自动下载".to_string(),
            });
        }
    }

    Ok(BatchUpdateResult {
        total: mods_to_update.len(),
        updated: updated_count,
        failed: failed_count,
        details,
    })
}

fn normalize_version(v: &str) -> String {
    let trimmed = v.trim().trim_start_matches('v').trim_start_matches('V');
    trimmed.to_string()
}

pub fn compare_versions(v1: &str, v2: &str) -> i32 {
    let parts1: Vec<u64> = normalize_version(v1).split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    let parts2: Vec<u64> = normalize_version(v2).split('.')
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
                        return Some(ModUpdateStatus {
                            unique_id: manifest["UniqueID"].as_str()
                                .or_else(|| manifest["Name"].as_str())
                                .unwrap_or("Unknown").to_string(),
                            name: manifest["Name"].as_str().unwrap_or("Unknown").to_string(),
                            current_version: current_version.to_string(),
                            latest_version: None,
                            has_update: false,
                            update_source: UpdateSource::NexusApi,
                            download_url: Some(format!(
                                "https://www.nexusmods.com/stardewvalley/mods/{}",
                                digits
                            )),
                            nexus_mod_id: Some(digits),
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_get_nexus_id_with_nexus_mod_id_field() {
        let mod_entry = json!({
            "unique_id": "SomeMod.ID",
            "name": "Some Mod",
            "nexus_mod_id": "1915"
        });
        let result = get_nexus_id_from_mod_data(&mod_entry);
        assert_eq!(result, Some("1915".to_string()));
    }

    #[test]
    fn test_get_nexus_id_missing_nexus_mod_id_and_no_smapi_metadata() {
        // Simulates current broken frontend: sends unique_id + name but no nexus_mod_id
        // Without populated SMAPI cache, this returns None -> batch update fails
        let mod_entry = json!({
            "unique_id": "NonExistent.Mod.xyz",
            "name": "Non Existent Mod"
        });
        assert!(
            mod_entry.get("nexus_mod_id").is_none(),
            "nexus_mod_id should be missing (simulating current frontend bug)"
        );
        let result = get_nexus_id_from_mod_data(&mod_entry);
        assert_eq!(result, None,
            "Without nexus_mod_id and without SMAPI metadata, batch update should fail for this mod"
        );
    }

    #[test]
    fn test_get_nexus_id_from_download_url_now_supported() {
        // After fix: download_url IS parsed as fallback
        let mod_entry = json!({
            "unique_id": "NonExistent.Mod.xyz",
            "name": "Non Existent Mod",
            "download_url": "https://www.nexusmods.com/stardewvalley/mods/1915"
        });
        let result = get_nexus_id_from_mod_data(&mod_entry);
        assert_eq!(result, Some("1915".to_string()),
            "download_url is now parsed as fallback for batch update"
        );
    }

    #[test]
    fn test_extract_nexus_id_from_download_url() {
        assert_eq!(
            extract_nexus_id_from_download_url("https://www.nexusmods.com/stardewvalley/mods/1915"),
            Some("1915".to_string())
        );
        assert_eq!(
            extract_nexus_id_from_download_url("https://www.nexusmods.com/stardewvalley/mods/2400?tab=files"),
            Some("2400".to_string())
        );
        assert_eq!(
            extract_nexus_id_from_download_url("not-a-nexus-url"),
            None
        );
        assert_eq!(
            extract_nexus_id_from_download_url(""),
            None
        );
    }

    #[test]
    fn test_get_nexus_id_falls_back_to_download_url() {
        // After fix: get_nexus_id_from_mod_data should extract nexus_id from download_url
        // even when nexus_mod_id field is missing
        let mod_entry = json!({
            "unique_id": "SomeMod.ID",
            "name": "Some Mod",
            "download_url": "https://www.nexusmods.com/stardewvalley/mods/1915"
        });
        let result = get_nexus_id_from_mod_data(&mod_entry);
        assert_eq!(result, Some("1915".to_string()),
            "Should extract nexus mod ID from download_url as fallback"
        );
    }
}

#[tauri::command]
pub async fn download_mod_update(
    app: tauri::AppHandle,
    nexus_mod_id: String,
    api_key: String,
    mods_path: String,
    old_unique_id: Option<String>,
) -> Result<String, String> {
    eprintln!("[download_mod_update] Downloading mod update for nexus_id={}", nexus_mod_id);

    if api_key.is_empty() {
        return Err("需要设置 Nexus API Key 才能下载 MOD".into());
    }

    let files = crate::nexus_api::get_nexus_mod_files(api_key.clone(), nexus_mod_id.clone()).await?;

    if files.is_empty() {
        return Err("该 MOD 没有可用的下载文件".into());
    }

    let target_file = files.into_iter()
        .filter(|f| !f.is_premium_only)
        .max_by_key(|f| f.upload_time.clone())
        .ok_or("该 MOD 的所有文件都需要付费会员".to_string())?;

    eprintln!("[download_mod_update] Downloading: {} (file_id={})", target_file.name, target_file.file_id);

    let result = crate::nexus_api::download_mod_from_nexus(
        app,
        nexus_mod_id,
        api_key,
        Some(mods_path),
        Some(target_file.file_id),
        old_unique_id,
    ).await?;

    if result.success {
        Ok(format!("已成功下载并安装更新: {}", result.message))
    } else {
        Err(format!("下载失败: {}", result.message))
    }
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
        nexus_mod_id: Some(nexus_mod_id.to_string()),
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

    // Fallback: extract nexus mod ID from download_url
    if let Some(url) = mod_entry.get("download_url").and_then(|v| v.as_str()) {
        if let Some(id) = extract_nexus_id_from_download_url(url) {
            return Some(id);
        }
    }

    None
}

fn extract_nexus_id_from_download_url(url: &str) -> Option<String> {
    let re = regex::Regex::new(r"nexusmods\.com/\w+/mods/(\d+)").ok()?;
    re.captures(url)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}
