use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::mod_name_resolver::resolve_mod_name;

pub const NEXUS_API_BASE: &str = "https://api.nexusmods.com/v1";
pub const STARDEW_GAME_ID: &str = "stardewvalley";
const USER_AGENT: &str = "SVL-Stardew-Valley-Launcher/1.0";

static MOD_DICT: OnceLock<HashMap<String, String>> = OnceLock::new();

fn get_mod_dict() -> &'static HashMap<String, String> {
    MOD_DICT.get_or_init(|| {
        let dict_str = include_str!("../mod_dict.json");
        serde_json::from_str(dict_str).unwrap_or_default()
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexusApiVerification {
    pub success: bool,
    pub is_premium: bool,
    pub user_name: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NxmLinkInfo {
    pub game_id: String,
    pub mod_id: String,
    pub file_id: String,
    pub original_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModUpdateInfo {
    pub unique_id: String,
    pub name: String,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub nexus_mod_id: Option<String>,
    pub has_update: bool,
    pub download_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexusModInfo {
    pub name: String,
    pub summary: String,
    pub version: String,
    pub author: String,
    pub picture_url: Option<String>,
    pub downloads: u64,
    pub endorsements: u64,
    pub is_endorsed: bool,
    pub mod_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexusFileDownloadInfo {
    pub file_id: String,
    pub name: String,
    pub version: String,
    pub size: u64,
    pub upload_time: String,
    pub download_url: Option<String>,
    pub is_premium_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NxmProtocolResult {
    pub success: bool,
    pub message: String,
}

pub fn build_nexus_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .unwrap_or_else(|_| reqwest::blocking::Client::new())
}

pub fn build_nexus_async_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

pub fn add_nexus_headers(request: reqwest::blocking::RequestBuilder, api_key: &str) -> reqwest::blocking::RequestBuilder {
    request
        .header("apikey", api_key)
        .header("User-Agent", USER_AGENT)
}

pub fn add_nexus_async_headers(request: reqwest::RequestBuilder, api_key: &str) -> reqwest::RequestBuilder {
    request
        .header("apikey", api_key)
        .header("User-Agent", USER_AGENT)
}

#[tauri::command]
pub async fn verify_nexus_api_key(api_key: String) -> Result<NexusApiVerification, String> {
    let client = build_nexus_async_client();

    let response = add_nexus_async_headers(
        client.get(format!("{}/users/validate.json", NEXUS_API_BASE)),
        &api_key
    )
    .send()
    .await
    .map_err(|e| format!("API 请求失败: {}", e))?;

    if response.status().is_success() {
        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("解析响应失败: {}", e))?;

        Ok(NexusApiVerification {
            success: true,
            is_premium: body["is_premium"].as_bool().unwrap_or(false),
            user_name: body["name"].as_str().map(|s| s.to_string()),
            message: None,
        })
    } else {
        let status = response.status();
        Ok(NexusApiVerification {
            success: false,
            is_premium: false,
            user_name: None,
            message: Some(format!("API Key 无效 (状态码: {})", status)),
        })
    }
}

#[tauri::command]
pub fn parse_nxm_link(nxm_url: String) -> Result<NxmLinkInfo, String> {
    let url = nxm_url.trim();
    
    if !url.starts_with("nxm://") {
        return Err("无效的 NXM 链接格式".to_string());
    }

    let parts: Vec<&str> = url.trim_start_matches("nxm://").split('/').collect();
    
    if parts.len() < 3 {
        return Err("NXM 链接格式不完整".to_string());
    }

    let game_id = parts[0].to_string();
    let mod_id = parts[1].to_string();
    let file_id = parts[2].to_string();

    Ok(NxmLinkInfo {
        game_id,
        mod_id,
        file_id,
        original_url: url.to_string(),
    })
}

#[tauri::command]
pub fn handle_nxm_link(nxm_url: String) -> Result<NxmLinkInfo, String> {
    let info = parse_nxm_link(nxm_url.clone())?;
    
    Ok(info)
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn register_nxm_protocol() -> Result<NxmProtocolResult, String> {
    use std::process::Command;
    
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("获取可执行文件路径失败: {}", e))?;
    
    let exe_path_str = exe_path.to_string_lossy();
    
    let default_icon_cmd = format!("\"{}\",0", exe_path_str);
    let open_command = format!("\"{}\" \"%1\"", exe_path_str);
    
    let reg_commands: Vec<Vec<&str>> = vec![
        vec!["add", "HKCU\\Software\\Classes\\nxm", "/ve", "/d", "Nexus Mods Link", "/f"],
        vec!["add", "HKCU\\Software\\Classes\\nxm\\DefaultIcon", "/ve", "/d", &default_icon_cmd, "/f"],
        vec!["add", "HKCU\\Software\\Classes\\nxm\\shell", "/f"],
        vec!["add", "HKCU\\Software\\Classes\\nxm\\shell\\open", "/f"],
        vec!["add", "HKCU\\Software\\Classes\\nxm\\shell\\open\\command", "/ve", "/d", &open_command, "/f"],
    ];
    
    for args in &reg_commands {
        let output = Command::new("reg")
            .args(args)
            .output()
            .map_err(|e| format!("执行注册表命令失败: {}", e))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("注册表写入失败: {}", stderr));
        }
    }
    
    Ok(NxmProtocolResult {
        success: true,
        message: "NXM 协议注册成功".to_string(),
    })
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub fn register_nxm_protocol() -> Result<NxmProtocolResult, String> {
    Ok(NxmProtocolResult {
        success: false,
        message: "当前平台不支持 NXM 协议注册".to_string(),
    })
}

#[tauri::command]
pub async fn check_mod_updates(
    api_key: String,
    mods_data: Vec<serde_json::Value>,
) -> Result<Vec<ModUpdateInfo>, String> {
    let client = build_nexus_async_client();
    let mut updates = Vec::new();
    let batch_size = 10;

    for (batch_idx, batch) in mods_data.chunks(batch_size).enumerate() {
        eprintln!("[check_mod_updates] Processing batch {}/{} ({} mods)", batch_idx + 1, (mods_data.len() + batch_size - 1) / batch_size, batch.len());

        for mod_entry in batch {
            let unique_id = mod_entry["unique_id"].as_str().unwrap_or("").to_string();
            let name = mod_entry["name"].as_str().unwrap_or("Unknown").to_string();
            let current_version = mod_entry["version"].as_str().unwrap_or("1.0.0").to_string();
            let nexus_mod_id = mod_entry["nexus_mod_id"].as_str().map(|s| s.to_string());

            if let Some(mod_id) = &nexus_mod_id {
                match get_nexus_mod_info_async(&client, &api_key, mod_id).await {
                    Ok(mod_info) => {
                        let has_update = mod_info.version != current_version;
                        updates.push(ModUpdateInfo {
                            unique_id,
                            name,
                            current_version,
                            latest_version: Some(mod_info.version),
                            nexus_mod_id: Some(mod_id.clone()),
                            has_update,
                            download_url: Some(format!(
                                "https://www.nexusmods.com/stardewvalley/mods/{}",
                                mod_id
                            )),
                        });
                    }
                    Err(e) => {
                        eprintln!("[check_mod_updates] Failed to fetch info for {}: {}", mod_id, e);
                        updates.push(ModUpdateInfo {
                            unique_id,
                            name,
                            current_version,
                            latest_version: None,
                            nexus_mod_id: Some(mod_id.clone()),
                            has_update: false,
                            download_url: Some(format!(
                                "https://www.nexusmods.com/stardewvalley/mods/{}",
                                mod_id
                            )),
                        });
                    }
                }
            }
        }

        tokio::task::yield_now().await;
    }

    updates.sort_by(|a, b| b.has_update.cmp(&a.has_update));
    Ok(updates)
}

#[tauri::command]
pub fn endorse_mod(api_key: String, mod_id: String) -> Result<bool, String> {
    let client = build_nexus_client();

    let response = add_nexus_headers(
        client.post(format!(
            "{}/games/{}/mods/{}/endorse.json",
            NEXUS_API_BASE, STARDEW_GAME_ID, mod_id
        )),
        &api_key
    )
    .send()
    .map_err(|e| format!("支持请求失败: {}", e))?;

    if response.status().is_success() {
        Ok(true)
    } else {
        Err(format!("支持失败 (状态码: {})", response.status()))
    }
}

#[tauri::command]
pub fn get_nexus_mod_files(
    api_key: String,
    mod_id: String,
) -> Result<Vec<NexusFileDownloadInfo>, String> {
    let client = build_nexus_client();

    let response = add_nexus_headers(
        client.get(format!(
            "{}/games/{}/mods/{}/files.json",
            NEXUS_API_BASE, STARDEW_GAME_ID, mod_id
        )),
        &api_key
    )
    .send()
    .map_err(|e| format!("获取文件列表失败: {}", e))?;

    if response.status().is_success() {
        let files: Vec<serde_json::Value> = response
            .json()
            .map_err(|e| format!("解析文件列表失败: {}", e))?;

        let result = files
            .into_iter()
            .map(|f| NexusFileDownloadInfo {
                file_id: f["file_id"].as_str().unwrap_or("").to_string(),
                name: f["name"].as_str().unwrap_or("").to_string(),
                version: f["version"].as_str().unwrap_or("").to_string(),
                size: f["size"].as_u64().unwrap_or(0),
                upload_time: f["uploaded_timestamp"].as_str().unwrap_or("").to_string(),
                download_url: None,
                is_premium_only: false,
            })
            .collect();

        Ok(result)
    } else {
        Err(format!("获取文件列表失败 (状态码: {})", response.status()))
    }
}

#[tauri::command]
pub async fn get_nexus_download_url(
    unique_id: String,
    api_key: String,
    mod_folder_path: Option<String>,
) -> Result<String, String> {
    eprintln!("[get_nexus_download_url] === 开始解析 UniqueID: {} ===", unique_id);

    // Step 1: Check mod_dict.json for direct UniqueID match
    let dict = get_mod_dict();
    if let Some(mod_id) = dict.get(&unique_id) {
        let url = format!("https://www.nexusmods.com/stardewvalley/mods/{}", mod_id);
        eprintln!("[get_nexus_download_url] 映射表直接命中: {} → {}", unique_id, url);
        return Ok(url);
    }

    // Also check manifest.json for Nexus UpdateKey if folder path provided
    if let Some(ref folder_path) = mod_folder_path {
        if let Some(nexus_id) = extract_nexus_id_from_manifest(folder_path) {
            let url = format!("https://www.nexusmods.com/stardewvalley/mods/{}", nexus_id);
            eprintln!("[get_nexus_download_url] manifest.json UpdateKey 命中: {} → {}", unique_id, url);
            return Ok(url);
        }
    }

    // Step 2: Try Nexus Mods API search (if API key available)
    if !api_key.is_empty() {
        let real_name = resolve_mod_name(&unique_id);
        let simplified = simplify_mod_name(&real_name);
        eprintln!("[get_nexus_download_url] API 搜索: UniqueID={} → 名称='{}'", unique_id, simplified);

        // Try wildcard search first
        let wildcard = format!("*{}*", simplified);
        match query_graphql_mods(&api_key, &wildcard).await {
            Ok(url) => {
                eprintln!("[get_nexus_download_url] API 通配符搜索成功: {}", url);
                return Ok(url);
            }
            Err(e) => eprintln!("[get_nexus_download_url] API 通配符搜索失败: {}", e),
        }

        // Try exact search
        match query_graphql_mods(&api_key, &simplified).await {
            Ok(url) => {
                eprintln!("[get_nexus_download_url] API 精确搜索成功: {}", url);
                return Ok(url);
            }
            Err(e) => eprintln!("[get_nexus_download_url] API 精确搜索失败: {}", e),
        }
    } else {
        eprintln!("[get_nexus_download_url] 未提供 API Key，跳过 API 搜索");
    }

    // Step 3: Fallback to search URL using resolved real name
    let real_name = resolve_mod_name(&unique_id);
    let search_name = simplify_mod_name(&real_name);
    let search_url = build_nexus_search_url(&search_name);
    eprintln!("[get_nexus_download_url] 降级为搜索链接: {} (名称: '{}')", search_url, real_name);
    Ok(search_url)
}

fn extract_nexus_id_from_manifest(folder_path: &str) -> Option<String> {
    let path = PathBuf::from(folder_path);
    let manifest_path = path.join("manifest.json");

    if !manifest_path.exists() {
        eprintln!("[extract_nexus_id] manifest.json 不存在: {}", manifest_path.display());
        return None;
    }

    let content = match fs::read_to_string(&manifest_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[extract_nexus_id] 读取 manifest.json 失败: {}", e);
            return None;
        }
    };

    let manifest: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[extract_nexus_id] 解析 manifest.json 失败: {}", e);
            return None;
        }
    };

    if let Some(update_keys) = manifest.get("UpdateKeys").and_then(|v| v.as_array()) {
        for key in update_keys {
            if let Some(key_str) = key.as_str() {
                if key_str.starts_with("Nexus:") {
                    let raw_id = key_str.trim_start_matches("Nexus:").trim();
                    let digits: String = raw_id.chars().filter(|c| c.is_ascii_digit()).collect();
                    if !digits.is_empty() {
                        eprintln!("[extract_nexus_id] 从 UpdateKey '{}' 提取到 ID: {}", key_str, digits);
                        return Some(digits);
                    }
                }
            }
        }
    }

    None
}

fn simplify_mod_name(name: &str) -> String {
    let stop_words = [
        "Stardew Valley",
        "stardew valley",
        "Stardew",
        "stardew",
        "Valley",
        "valley",
        "SV",
        "SVE",
        "SDV",
        "for Stardew Valley",
        "for Stardew",
        "- Stardew Valley",
        "(Stardew Valley)",
    ];

    let mut result = name.to_string();

    for stop_word in &stop_words {
        result = result.replace(stop_word, "");
    }

    result = result
        .replace("  ", " ")
        .trim_matches(|c: char| c == ' ' || c == '-' || c == '_' || c == ',')
        .to_string();

    if result.is_empty() {
        name.to_string()
    } else {
        result
    }
}

fn build_nexus_search_url(search_name: &str) -> String {
    let cleaned: String = search_name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { ' ' })
        .collect();
    let trimmed = cleaned.trim();
    format!(
        "https://www.nexusmods.com/stardewvalley/mods/search?search={}",
        urlencoding::encode(trimmed)
    )
}

async fn query_graphql_mods(
    api_key: &str,
    name_value: &str,
) -> Result<String, String> {
    eprintln!("[query_graphql_mods] 搜索名称: {}", name_value);

    let graphql_query = r#"
        query SearchMods($nameFilter: [BaseFilterValueEqualsWildcard!], $gameIdFilter: [BaseFilterValue!]) {
            mods(filter: { name: $nameFilter, gameId: $gameIdFilter }) {
                totalCount
                nodes {
                    modId
                    gameId
                }
            }
        }
    "#;

    let request_body = serde_json::json!({
        "query": graphql_query,
        "variables": {
            "nameFilter": [{"op": "EQUALS", "value": name_value}],
            "gameIdFilter": [{"op": "EQUALS", "value": "1303"}]
        }
    });

    let client = build_nexus_async_client();

    let response = client
        .post("https://api.nexusmods.com/v2/graphql")
        .header("Content-Type", "application/json")
        .header("User-Agent", USER_AGENT)
        .header("apikey", api_key)
        .body(request_body.to_string())
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let status = response.status();
    eprintln!("[query_graphql_mods] HTTP 状态码: {}", status);

    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_default();
        eprintln!("[query_graphql_mods] 错误响应: {}", error_body);
        return Err(format!("HTTP {}", status));
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    eprintln!("[query_graphql_mods] 响应: {}", body);

    if let Some(errors) = body.get("errors") {
        eprintln!("[query_graphql_mods] GraphQL 错误: {}", errors);
        return Err("GRAPHQL_ERROR".to_string());
    }

    if let Some(mods) = body.get("data")
        .and_then(|d| d.get("mods"))
        .and_then(|m| m.get("nodes"))
        .and_then(|n| n.as_array())
    {
        for mod_entry in mods {
            if let Some(mod_id) = mod_entry.get("modId").and_then(|id| id.as_u64()) {
                let mod_game_id = mod_entry.get("gameId").and_then(|g| g.as_u64()).unwrap_or(1303);
                if mod_game_id == 1303 {
                    let url = format!("https://www.nexusmods.com/stardewvalley/mods/{}", mod_id);
                    eprintln!("[query_graphql_mods] 找到 MOD: {}", url);
                    return Ok(url);
                }
            }
        }
    }

    Err("未找到匹配的 MOD".to_string())
}

async fn get_nexus_mod_info_async(
    client: &reqwest::Client,
    api_key: &str,
    mod_id: &str,
) -> Result<NexusModInfo, String> {
    let response = add_nexus_async_headers(
        client.get(format!(
            "{}/games/{}/mods/{}.json",
            NEXUS_API_BASE, STARDEW_GAME_ID, mod_id
        )),
        api_key
    )
    .send()
    .await
    .map_err(|e| format!("获取 MOD 信息失败: {}", e))?;

    if response.status().is_success() {
        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("解析 MOD 信息失败: {}", e))?;

        Ok(NexusModInfo {
            name: body["name"].as_str().unwrap_or("").to_string(),
            summary: body["summary"].as_str().unwrap_or("").to_string(),
            version: body["version"].as_str().unwrap_or("").to_string(),
            author: body["author"].as_str().unwrap_or("").to_string(),
            picture_url: body["picture_url"].as_str().map(|s| s.to_string()),
            downloads: body["downloads"].as_u64().unwrap_or(0),
            endorsements: body["endorsements"].as_u64().unwrap_or(0),
            is_endorsed: body["is_endorsed"].as_bool().unwrap_or(false),
            mod_id: mod_id.to_string(),
        })
    } else {
        Err(format!("获取 MOD 信息失败 (状态码: {})", response.status()))
    }
}
