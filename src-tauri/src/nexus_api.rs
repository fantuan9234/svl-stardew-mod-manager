use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use tauri::Emitter;

use crate::mod_name_resolver::resolve_mod_name;
use crate::app_logger::{log_info, get_svl_data_dir};

pub const NEXUS_API_BASE: &str = "https://api.nexusmods.com/v1";
pub const STARDEW_GAME_ID: &str = "stardewvalley";
const USER_AGENT: &str = "SVL-Stardew-Valley-Launcher/1.0";

static MOD_DICT: OnceLock<HashMap<String, String>> = OnceLock::new();

const NEXUS_GRAPHQL_URL: &str = "https://api.nexusmods.com/v2/graphql";

fn get_mod_dict() -> &'static HashMap<String, String> {
    MOD_DICT.get_or_init(|| {
        let dict_str = include_str!("../mod_dict.json");
        serde_json::from_str(dict_str).unwrap_or_default()
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexusModSearchResult {
    pub mod_id: String,
    pub name: String,
    pub summary: String,
    pub version: String,
    pub author: String,
    pub picture_url: Option<String>,
    pub downloads: u64,
    pub endorsements: u64,
    pub uploaded_time: String,
    pub nexus_url: String,
    pub size: u64,
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
    pub category_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDownloadResult {
    pub success: bool,
    pub mod_name: String,
    pub mod_version: String,
    pub message: String,
    pub file_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NxmProtocolResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDiagnosticResult {
    pub target: String,
    pub reachable: bool,
    pub response_time_ms: Option<u64>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn diagnose_network() -> Result<Vec<NetworkDiagnosticResult>, String> {
    let targets = vec![
        ("Nexus API", "https://api.nexusmods.com"),
        ("Nexus CDN", "https://cdn.nexusmods.com"),
        ("SMAPI", "https://smapi.io"),
    ];

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .connect_timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let mut results = Vec::new();

    for (name, url) in targets {
        let start = std::time::Instant::now();
        match client.get(url).send().await {
            Ok(resp) => {
                let elapsed = start.elapsed().as_millis() as u64;
                results.push(NetworkDiagnosticResult {
                    target: name.to_string(),
                    reachable: resp.status().is_success() || resp.status().is_redirection(),
                    response_time_ms: Some(elapsed),
                    error: None,
                });
            }
            Err(e) => {
                let elapsed = start.elapsed().as_millis() as u64;
                results.push(NetworkDiagnosticResult {
                    target: name.to_string(),
                    reachable: false,
                    response_time_ms: Some(elapsed),
                    error: Some(e.to_string()),
                });
            }
        }
    }

    Ok(results)
}

#[tauri::command]
pub async fn test_nexus_connection() -> Result<NetworkDiagnosticResult, String> {
    let client = build_verify_client();
    let start = std::time::Instant::now();
    match client.get(format!("{}/users/validate.json", NEXUS_API_BASE)).send().await {
        Ok(resp) => {
            let elapsed = start.elapsed().as_millis() as u64;
            Ok(NetworkDiagnosticResult {
                target: "Nexus API".to_string(),
                reachable: resp.status().is_success() || resp.status().is_redirection(),
                response_time_ms: Some(elapsed),
                error: None,
            })
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            Ok(NetworkDiagnosticResult {
                target: "Nexus API".to_string(),
                reachable: false,
                response_time_ms: Some(elapsed),
                error: Some(e.to_string()),
            })
        }
    }
}

pub fn build_nexus_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(15))
        .connect_timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_else(|_| reqwest::blocking::Client::new())
}

pub fn build_nexus_async_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(15))
        .connect_timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

fn build_verify_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(10))
        .connect_timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

fn build_download_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .connect_timeout(std::time::Duration::from_secs(10))
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
    let client = build_verify_client();
    let max_retries = 2;

    for attempt in 0..=max_retries {
        match add_nexus_async_headers(
            client.get(format!("{}/users/validate.json", NEXUS_API_BASE)),
            &api_key
        )
        .send()
        .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    let body: serde_json::Value = response
                        .json()
                        .await
                        .map_err(|e| format!("解析响应失败: {}", e))?;

                    return Ok(NexusApiVerification {
                        success: true,
                        is_premium: body["is_premium"].as_bool().unwrap_or(false),
                        user_name: body["name"].as_str().map(|s| s.to_string()),
                        message: None,
                    });
                } else {
                    let status = response.status();
                    return Ok(NexusApiVerification {
                        success: false,
                        is_premium: false,
                        user_name: None,
                        message: Some(format!("API Key 无效 (状态码: {})", status)),
                    });
                }
            }
            Err(e) if attempt < max_retries => {
                eprintln!("[verify_nexus_api_key] 尝试 {}/{} 失败: {}, 1秒后重试", attempt + 1, max_retries + 1, e);
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }
            Err(e) => {
                return Err(format!("API 请求失败（已重试{}次）: {}", max_retries, e));
            }
        }
    }

    Err("验证失败：未知错误".to_string())
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
        use std::os::windows::process::CommandExt;
        let output = Command::new("reg")
            .args(args)
            .creation_flags(0x08000000)
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
                        let has_update = {
                            let nexus_ver = mod_info.version.strip_prefix('v').unwrap_or(&mod_info.version);
                            let local_ver = current_version.strip_prefix('v').unwrap_or(&current_version);
                            let result = crate::update_checker::compare_versions(nexus_ver, local_ver) > 0;
                            log_info("NexusUpdateCheck", &format!(
                                "{}: local={} nexus={} has_update={}",
                                unique_id, local_ver, nexus_ver, result
                            ));
                            result
                        };
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
pub async fn endorse_mod(api_key: String, mod_id: String) -> Result<bool, String> {
    let client = build_nexus_async_client();

    let response = add_nexus_async_headers(
        client.post(format!(
            "{}/games/{}/mods/{}/endorse.json",
            NEXUS_API_BASE, STARDEW_GAME_ID, mod_id
        )),
        &api_key
    )
    .send()
    .await
    .map_err(|e| format!("支持请求失败: {}", e))?;

    if response.status().is_success() {
        Ok(true)
    } else {
        Err(format!("支持失败 (状态码: {})", response.status()))
    }
}

#[tauri::command]
pub async fn get_nexus_mod_files(
    api_key: String,
    mod_id: String,
) -> Result<Vec<NexusFileDownloadInfo>, String> {
    let client = build_nexus_async_client();

    let response = add_nexus_async_headers(
        client.get(format!(
            "{}/games/{}/mods/{}/files.json",
            NEXUS_API_BASE, STARDEW_GAME_ID, mod_id
        )),
        &api_key
    )
    .send()
    .await
    .map_err(|e| format!("获取文件列表失败: {}", e))?;

    if response.status().is_success() {
        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("解析文件列表失败: {}", e))?;

        let files_arr = body.get("files")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let result = files_arr
            .into_iter()
            .map(|f| NexusFileDownloadInfo {
                file_id: f["file_id"].as_u64().unwrap_or(0).to_string(),
                name: f["file_name"].as_str().unwrap_or("").to_string(),
                version: f["version"].as_str().unwrap_or("").to_string(),
                size: f["size_in_bytes"].as_u64().unwrap_or(0),
                upload_time: f["uploaded_time"].as_str().unwrap_or("").to_string(),
                download_url: None,
                is_premium_only: f["is_premium"].as_bool().unwrap_or(false),
                category_id: f["category_id"].as_i64().unwrap_or(1),
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
        eprintln!("[get_nexus_download_url] 映射表直接命中: {} -> {}", unique_id, url);
        return Ok(url);
    }

    // Also check manifest.json for Nexus UpdateKey if folder path provided
    if let Some(ref folder_path) = mod_folder_path {
        if let Some(nexus_id) = extract_nexus_id_from_manifest(folder_path) {
            let url = format!("https://www.nexusmods.com/stardewvalley/mods/{}", nexus_id);
            eprintln!("[get_nexus_download_url] manifest.json UpdateKey 命中: {} -> {}", unique_id, url);
            return Ok(url);
        }
    }

    // Step 2: Try Nexus Mods API search (if API key available)
    if !api_key.is_empty() {
        let real_name = resolve_mod_name(&unique_id);
        let simplified = simplify_mod_name(&real_name);
        eprintln!("[get_nexus_download_url] API 搜索: UniqueID={} -> 名称='{}'", unique_id, simplified);

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

    let normalized = crate::mod_parser::normalize_smart_quotes(&content);
    let cleaned = crate::mod_parser::remove_trailing_commas(&normalized);
    let cleaned = cleaned.strip_prefix('\u{FEFF}').unwrap_or(&cleaned);

    let manifest: serde_json::Value = match serde_json::from_str(cleaned) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[extract_nexus_id] 解析 manifest.json 失败: {}", e);
            return None;
        }
    };

    if let Some(update_keys) = manifest.get("UpdateKeys").and_then(|v| v.as_array()) {
        for key in update_keys {
            let key_str = if let Some(s) = key.as_str() {
                s.to_string()
            } else if let Some(n) = key.as_i64() {
                return Some(n.to_string());
            } else {
                continue;
            };

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
        .post(NEXUS_GRAPHQL_URL)
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

fn extract_mods_array(body: &serde_json::Value) -> Vec<serde_json::Value> {
    if let Some(arr) = body.as_array() {
        if !arr.is_empty() {
            eprintln!("[extract_mods_array] 响应是数组，长度: {}", arr.len());
            if let Some(obj) = arr[0].as_object() {
                let keys: Vec<_> = obj.keys().cloned().collect();
                eprintln!("[extract_mods_array] 第一个元素的 keys: {:?}", keys);
                eprintln!("[extract_mods_array] 第一个元素完整内容: {}", serde_json::to_string_pretty(&arr[0]).unwrap_or_default());
            }
        }
        arr.clone()
    } else if let Some(obj) = body.as_object() {
        let keys: Vec<_> = obj.keys().cloned().collect();
        eprintln!("[extract_mods_array] 响应是对象，keys: {:?}", keys);
        eprintln!("[extract_mods_array] 完整响应: {}", serde_json::to_string_pretty(body).unwrap_or_default());
        if let Some(arr) = obj.get("mods").and_then(|v| v.as_array()) {
            arr.clone()
        } else if let Some(arr) = obj.get("nodes").and_then(|v| v.as_array()) {
            arr.clone()
        } else if let Some(arr) = obj.get("data").and_then(|v| v.as_array()) {
            arr.clone()
        } else {
            for (key, val) in obj {
                if val.is_array() {
                    eprintln!("[extract_mods_array] 找到键: {}", key);
                    let arr = val.as_array().unwrap();
                    if !arr.is_empty() {
                        if let Some(first) = arr[0].as_object() {
                            let keys: Vec<_> = first.keys().cloned().collect();
                            eprintln!("[extract_mods_array] 数组元素 keys: {:?}", keys);
                            eprintln!("[extract_mods_array] 第一个数组元素: {}", serde_json::to_string_pretty(&arr[0]).unwrap_or_default());
                        }
                    }
                    return arr.clone();
                }
            }
            vec![]
        }
    } else {
        vec![]
    }
}

/// Extract mod IDs from updated.json summary response
fn extract_mod_ids_from_updated(body: &serde_json::Value) -> Vec<u64> {
    if let Some(arr) = body.as_array() {
        arr.iter()
            .filter_map(|item| item.get("mod_id").and_then(|v| v.as_u64()))
            .collect()
    } else if let Some(obj) = body.as_object() {
        // Try to find array in nested structure
        for (_key, val) in obj {
            if let Some(arr) = val.as_array() {
                return arr.iter()
                    .filter_map(|item| item.get("mod_id").and_then(|v| v.as_u64()))
                    .collect();
            }
        }
        vec![]
    } else {
        vec![]
    }
}

fn parse_mod_search_result(mod_data: &serde_json::Value) -> NexusModSearchResult {
    // Log the raw data to debug field name mapping (only first result to avoid spam)
    if let Some(obj) = mod_data.as_object() {
        let keys: Vec<_> = obj.keys().cloned().collect();
        eprintln!("[parse_mod_search_result] MOD 元素，键: {:?}", keys);
    }

    // The updated.json endpoint returns objects with these exact field names:
    // mod_id, name, summary, picture_url, downloads, endorsements, uploaded_time,
    // author, version, etc.
    // But let's try all possible variants to be safe.

    let mod_id = mod_data.get("mod_id")
        .or_else(|| mod_data.get("modId"))
        .or_else(|| mod_data.get("id"))
        .or_else(|| mod_data.get("game_mod_id"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0).to_string();

    let name = mod_data.get("name")
        .or_else(|| mod_data.get("mod_name"))
        .or_else(|| mod_data.get("title"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let summary = mod_data.get("summary")
        .or_else(|| mod_data.get("description"))
        .or_else(|| mod_data.get("short_description"))
        .or_else(|| mod_data.get("blurb"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let version = mod_data.get("version")
        .or_else(|| mod_data.get("current_version"))
        .or_else(|| mod_data.get("latest_version"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let author = mod_data.get("author")
        .or_else(|| mod_data.get("uploaded_by"))
        .or_else(|| mod_data.get("user_name"))
        .or_else(|| mod_data.get("author_name"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let picture_url = mod_data.get("picture_url")
        .or_else(|| mod_data.get("thumbnailUrl"))
        .or_else(|| mod_data.get("image_url"))
        .or_else(|| mod_data.get("picture"))
        .or_else(|| mod_data.get("thumbnail"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let downloads = mod_data.get("downloads")
        .or_else(|| mod_data.get("total_downloads"))
        .or_else(|| mod_data.get("download_count"))
        .or_else(|| mod_data.get("downloadTotal"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let endorsements = mod_data.get("endorsements")
        .or_else(|| mod_data.get("endorsement_count"))
        .or_else(|| mod_data.get("endorsement_total"))
        .or_else(|| mod_data.get("endorsementTotal"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let uploaded_time = mod_data.get("uploaded_time")
        .or_else(|| mod_data.get("updatedAt"))
        .or_else(|| mod_data.get("createdAt"))
        .or_else(|| mod_data.get("last_updated"))
        .or_else(|| mod_data.get("date_added"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    NexusModSearchResult {
        mod_id: mod_id.clone(),
        name,
        summary,
        version,
        author,
        picture_url,
        downloads,
        endorsements,
        uploaded_time,
        nexus_url: format!("https://www.nexusmods.com/stardewvalley/mods/{}", mod_id),
        size: 0,
    }
}

fn expand_mod_abbreviation(query: &str) -> String {
    let lower = query.trim().to_lowercase();
    let abbreviation_map: &[(&str, &str)] = &[
        ("sve", "Stardew Valley Expanded"),
        ("svei", "Stardew Valley Expanded"),
        ("sv expanded", "Stardew Valley Expanded"),
        ("cp", "Content Patcher"),
        ("gmcm", "Generic Mod Config Menu"),
        ("rsv", "Ridgeside Village"),
        ("ridgeside", "Ridgeside Village"),
        ("es", "East Scarp"),
        ("ja", "Json Assets"),
        ("dga", "Dynamic Game Assets"),
        ("epu", "Expanded Preconditions Utility"),
        ("cc", "Custom Companions"),
        ("bfav", "Better Farm Animal Variety"),
        ("npm", "NPC Map Locations"),
        ("npcmap", "NPC Map Locations"),
        ("ftm", "Farm Type Manager"),
        ("la", "Lookup Anything"),
        ("cjb", "CJB"),
        ("cjbc", "CJB Cheats Menu"),
        ("cjbi", "CJB Item Spawner"),
        ("cjbs", "CJB Show Item Sell Price"),
        ("ppja", "PPJA"),
        ("artisan valley", "Artisan Valley"),
        ("json assets", "Json Assets"),
        ("content patcher", "Content Patcher"),
        ("spacecore", "SpaceCore"),
        ("sc", "SpaceCore"),
    ];

    let exact_match = lower.as_str();
    for (abbr, full) in abbreviation_map {
        if exact_match == *abbr {
            eprintln!("[expand_mod_abbreviation] '{}' -> '{}'", query, full);
            return full.to_string();
        }
    }

    query.trim().to_string()
}

async fn search_nexus_website(
    client: &reqwest::Client,
    query: &str,
) -> Result<Vec<NexusModSearchResult>, String> {
    let url = format!(
        "https://www.nexusmods.com/stardewvalley/ajax/search?term={}&game_id=1303",
        urlencoding::encode(query)
    );
    eprintln!("[search_nexus_website] 请求: {}", url);

    let response = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("网站搜索请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("网站搜索失败 (状态码: {})", response.status()));
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析网站搜索响应失败: {}", e))?;

    let mut results: Vec<NexusModSearchResult> = Vec::new();

    if let Some(mods_arr) = body.get("MODS").and_then(|m| m.as_array()) {
        for mod_item in mods_arr {
            let mod_id = mod_item.get("mod_id")
                .and_then(|v| v.as_str())
                .or_else(|| mod_item.get("id").and_then(|v| v.as_str()))
                .unwrap_or("0");

            let name = mod_item.get("mod_name")
                .and_then(|v| v.as_str())
                .or_else(|| mod_item.get("name").and_then(|v| v.as_str()))
                .unwrap_or("");

            let summary = mod_item.get("summary")
                .and_then(|v| v.as_str())
                .or_else(|| mod_item.get("description").and_then(|v| v.as_str()))
                .unwrap_or("");

            let author = mod_item.get("author")
                .and_then(|v| v.as_str())
                .or_else(|| mod_item.get("username").and_then(|v| v.as_str()))
                .unwrap_or("");

            let picture_url = mod_item.get("image")
                .and_then(|v| v.as_str())
                .or_else(|| mod_item.get("thumbnail").and_then(|v| v.as_str()))
                .map(|s| {
                    if s.starts_with("http") {
                        s.to_string()
                    } else {
                        format!("https://www.nexusmods.com{}", s)
                    }
                });

            let downloads = mod_item.get("downloads")
                .and_then(|v| v.as_str())
                .and_then(|s| s.replace(",", "").parse::<u64>().ok())
                .or_else(|| mod_item.get("downloads").and_then(|v| v.as_u64()))
                .unwrap_or(0);

            let endorsements = mod_item.get("endorsements")
                .and_then(|v| v.as_str())
                .and_then(|s| s.replace(",", "").parse::<u64>().ok())
                .or_else(|| mod_item.get("endorsements").and_then(|v| v.as_u64()))
                .unwrap_or(0);

            let uploaded_time = mod_item.get("date")
                .and_then(|v| v.as_str())
                .or_else(|| mod_item.get("uploaded_time").and_then(|v| v.as_str()))
                .unwrap_or("");

            results.push(NexusModSearchResult {
                mod_id: mod_id.to_string(),
                name: name.to_string(),
                summary: summary.to_string(),
                version: String::new(),
                author: author.to_string(),
                picture_url,
                downloads,
                endorsements,
                uploaded_time: uploaded_time.to_string(),
                nexus_url: format!("https://www.nexusmods.com/stardewvalley/mods/{}", mod_id),
                size: 0,
            });
        }
    }

    eprintln!("[search_nexus_website] 网站搜索返回 {} 个结果", results.len());
    Ok(results)
}

async fn search_graphql_mods(
    client: &reqwest::Client,
    api_key: &str,
    query: &str,
) -> Result<Vec<NexusModSearchResult>, String> {
    eprintln!("[search_graphql_mods] 搜索: {}", query);

    let wildcard_value = format!("*{}*", query);

    let graphql_query = r#"
        query SearchMods($nameFilter: [BaseFilterValueEqualsWildcard!], $gameIdFilter: [BaseFilterValue!]) {
            mods(filter: { name: $nameFilter, gameId: $gameIdFilter }) {
                totalCount
                nodes {
                    modId
                    gameId
                    name
                }
            }
        }
    "#;

    let request_body = serde_json::json!({
        "query": graphql_query,
        "variables": {
            "nameFilter": [{"op": "WILDCARD", "value": wildcard_value}],
            "gameIdFilter": [{"op": "EQUALS", "value": "1303"}]
        }
    });

    let response = client
        .post(NEXUS_GRAPHQL_URL)
        .header("Content-Type", "application/json")
        .header("User-Agent", USER_AGENT)
        .header("apikey", api_key)
        .body(request_body.to_string())
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("GraphQL 请求失败: {}", e))?;

    let status = response.status();
    eprintln!("[search_graphql_mods] HTTP 状态码: {}", status);

    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_default();
        eprintln!("[search_graphql_mods] 错误响应: {}", error_body);
        return Err(format!("HTTP {}", status));
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    if let Some(errors) = body.get("errors") {
        eprintln!("[search_graphql_mods] GraphQL 错误: {}", errors);
        return Err("GRAPHQL_ERROR".to_string());
    }

    let nodes = body.get("data")
        .and_then(|d| d.get("mods"))
        .and_then(|m| m.get("nodes"))
        .and_then(|n| n.as_array())
        .cloned()
        .unwrap_or_default();

    eprintln!("[search_graphql_mods] GraphQL 返回 {} 个结果", nodes.len());

    let mut mod_ids: Vec<u64> = Vec::new();
    let mut graphql_names: std::collections::HashMap<u64, String> = std::collections::HashMap::new();

    for node in &nodes {
        if let Some(mod_id) = node.get("modId").and_then(|v| v.as_u64()) {
            let game_id = node.get("gameId").and_then(|v| v.as_u64()).unwrap_or(0);
            if game_id == 1303 || game_id == 0 {
                mod_ids.push(mod_id);
                if let Some(name) = node.get("name").and_then(|v| v.as_str()) {
                    graphql_names.insert(mod_id, name.to_string());
                }
            }
        }
    }

    mod_ids.truncate(20);

    let mut results: Vec<NexusModSearchResult> = Vec::new();

    for mod_id in &mod_ids {
        let url = format!("{}/games/stardewvalley/mods/{}.json", NEXUS_API_BASE, mod_id);

        match client
            .get(&url)
            .header("User-Agent", USER_AGENT)
            .header("apikey", api_key)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<serde_json::Value>().await {
                    Ok(mod_data) => {
                        let mut result = parse_mod_search_result(&mod_data);
                        if result.name.is_empty() {
                            if let Some(gql_name) = graphql_names.get(mod_id) {
                                result.name = gql_name.clone();
                            }
                        }
                        results.push(result);
                    }
                    Err(e) => eprintln!("[search_graphql_mods] 解析 mod {} 数据失败: {}", mod_id, e),
                }
            }
            Ok(resp) => {
                eprintln!("[search_graphql_mods] 获取 mod {} 详情失败: {}", mod_id, resp.status());
            }
            Err(e) => {
                eprintln!("[search_graphql_mods] 请求 mod {} 详情失败: {}", mod_id, e);
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    }

    eprintln!("[search_graphql_mods] 最终返回 {} 个结果", results.len());
    Ok(results)
}

#[tauri::command]
pub async fn search_nexus_mods(
    api_key: String,
    query: String,
    page: u32,
    category: Option<String>,
) -> Result<(Vec<NexusModSearchResult>, u32), String> {
    eprintln!("[search_nexus_mods] 搜索: query='{}', page={}, category={:?}", query, page, category);

    let client = build_nexus_async_client();

    let trimmed = query.trim();
    let is_mod_id = !trimmed.is_empty() && trimmed.parse::<u64>().is_ok();

    if is_mod_id {
        let mod_id = trimmed.to_string();
        let url = format!("{}/games/stardewvalley/mods/{}.json", NEXUS_API_BASE, mod_id);
        let response = client
            .get(&url)
            .header("User-Agent", USER_AGENT)
            .header("apikey", &api_key)
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await
            .map_err(|e| format!("搜索请求失败: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            eprintln!("[search_nexus_mods] API 错误: {} - {}", status, body);
            return Err(format!("未找到 MOD ID 为 {} 的模组", mod_id));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("解析搜索响应失败: {}", e))?;

        let mod_data = &body;
        let result = NexusModSearchResult {
            mod_id: mod_data.get("mod_id").and_then(|v| v.as_u64()).unwrap_or(0).to_string(),
            name: mod_data.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            summary: mod_data.get("summary").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            version: mod_data.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            author: mod_data.get("author").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            picture_url: mod_data.get("picture_url").and_then(|v| v.as_str()).map(|s| s.to_string()),
            downloads: mod_data.get("downloads").and_then(|v| v.as_u64()).unwrap_or(0),
            endorsements: mod_data.get("endorsements").and_then(|v| v.as_u64()).unwrap_or(0),
            uploaded_time: mod_data.get("uploaded_time").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            nexus_url: format!("https://www.nexusmods.com/stardewvalley/mods/{}", mod_id),
            size: 0,
        };

        Ok((vec![result], 1))
    } else if trimmed.is_empty() {
        let url_str = format!(
            "{}/games/stardewvalley/mods/updated.json?period=1w",
            NEXUS_API_BASE
        );

        eprintln!("[search_nexus_mods] 空搜索：获取最新 MOD: {}", url_str);

        let response = client
            .get(url_str.as_str())
            .header("User-Agent", USER_AGENT)
            .header("apikey", &api_key)
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await
            .map_err(|e| format!("搜索请求失败: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            eprintln!("[search_nexus_mods] API 错误: {} - {}", status, body);
            return Err(format!("搜索失败 (状态码: {})", status));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("解析搜索响应失败: {}", e))?;

        let mods_arr = extract_mods_array(&body);

        let mut results: Vec<NexusModSearchResult> = mods_arr.iter().map(|mod_data| {
            parse_mod_search_result(mod_data)
        }).collect();

        results.sort_by(|a, b| b.endorsements.cmp(&a.endorsements));
        results.truncate(20);

        eprintln!("[search_nexus_mods] 获取到 {} 个 MOD", results.len());
        Ok((results, 1))
    } else {
        let search_term = expand_mod_abbreviation(trimmed);
        eprintln!("[search_nexus_mods] 使用 GraphQL 搜索: '{}' -> 展开为 '{}'", trimmed, search_term);

        match search_graphql_mods(&client, &api_key, &search_term).await {
            Ok(results) if !results.is_empty() => {
                let total_pages = if results.len() > 20 { 2 } else { 1 };
                eprintln!("[search_nexus_mods] GraphQL 搜索返回 {} 个结果", results.len());
                Ok((results, total_pages))
            }
            Ok(results) => {
                eprintln!("[search_nexus_mods] GraphQL 返回 0 结果，尝试网站搜索");
                match search_nexus_website(&client, trimmed).await {
                    Ok(web_results) if !web_results.is_empty() => {
                        eprintln!("[search_nexus_mods] 网站搜索返回 {} 个结果", web_results.len());
                        Ok((web_results, 1))
                    }
                    _ => {
                        eprintln!("[search_nexus_mods] 网站搜索也无结果，返回 GraphQL 空结果");
                        Ok((results, 1))
                    }
                }
            }
            Err(e) => {
                eprintln!("[search_nexus_mods] GraphQL 搜索失败: {}, 尝试网站搜索", e);
                match search_nexus_website(&client, trimmed).await {
                    Ok(web_results) if !web_results.is_empty() => {
                        eprintln!("[search_nexus_mods] 网站搜索返回 {} 个结果", web_results.len());
                        Ok((web_results, 1))
                    }
                    Ok(_) => {
                        eprintln!("[search_nexus_mods] 网站搜索也无结果");
                        Err(format!("未找到与 '{}' 相关的模组", trimmed))
                    }
                    Err(web_err) => {
                        eprintln!("[search_nexus_mods] 网站搜索也失败: {}", web_err);
                        Err(format!("搜索失败: GraphQL 和网站搜索均不可用"))
                    }
                }
            }
        }
    }
}



async fn fetch_nexus_mods_by_period(
    api_key: &str,
    period: &str,
    sort_by: &str,
    limit: usize,
    label: &str,
) -> Result<Vec<NexusModSearchResult>, String> {
    let client = build_nexus_async_client();

    let url = format!(
        "{}/games/stardewvalley/mods/updated.json?period={}",
        NEXUS_API_BASE,
        period
    );
    eprintln!("[{}] 请求: {}", label, url);

    let response = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .header("apikey", api_key)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("获取MOD失败: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        eprintln!("[{}] API 错误: {} - {}", label, status, body);
        return Err(format!("获取MOD失败 (状态码: {})", status));
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    let mod_ids = extract_mod_ids_from_updated(&body);
    eprintln!("[{}] 获取到 {} 个 MOD ID", label, mod_ids.len());

    if mod_ids.is_empty() {
        return Err("未获取到 MOD 列表".to_string());
    }

    let ids_to_fetch: Vec<u64> = mod_ids.into_iter().take(limit).collect();

    let mut handles: Vec<tokio::task::JoinHandle<Option<NexusModSearchResult>>> = Vec::new();

    for mod_id in ids_to_fetch {
        let client_clone = client.clone();
        let api_key_clone = api_key.to_string();
        handles.push(tokio::spawn(async move {
            let url = format!("{}/games/stardewvalley/mods/{}.json", NEXUS_API_BASE, mod_id);
            match client_clone
                .get(&url)
                .header("User-Agent", USER_AGENT)
                .header("apikey", &api_key_clone)
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<serde_json::Value>().await {
                        Ok(mod_data) => Some(parse_mod_search_result(&mod_data)),
                        Err(e) => {
                            eprintln!("[fetch_nexus_mods] 解析 mod {} 数据失败: {}", mod_id, e);
                            None
                        }
                    }
                }
                Ok(resp) => {
                    eprintln!("[fetch_nexus_mods] 获取 mod {} 详情失败: {}", mod_id, resp.status());
                    None
                }
                Err(e) => {
                    eprintln!("[fetch_nexus_mods] 请求 mod {} 详情失败: {}", mod_id, e);
                    None
                }
            }
        }));
    }

    let mut results: Vec<NexusModSearchResult> = Vec::new();
    for handle in handles {
        if let Ok(Some(mod_result)) = handle.await {
            results.push(mod_result);
        }
    }

    match sort_by {
        "uploaded_time" => {
            results.sort_by(|a, b| b.uploaded_time.cmp(&a.uploaded_time));
        }
        "downloads" => {
            results.sort_by(|a, b| b.downloads.cmp(&a.downloads));
        }
        _ => {
            results.sort_by(|a, b| b.endorsements.cmp(&a.endorsements));
        }
    }

    eprintln!("[{}] 获取到 {} 个MOD", label, results.len());
    Ok(results)
}

#[tauri::command]
pub async fn get_trending_nexus_mods(
    api_key: String,
) -> Result<Vec<NexusModSearchResult>, String> {
    fetch_nexus_mods_by_period(&api_key, "1w", "endorsements", 20, "trending").await
}

#[tauri::command]
pub async fn get_recently_updated_nexus_mods(
    api_key: String,
) -> Result<Vec<NexusModSearchResult>, String> {
    fetch_nexus_mods_by_period(&api_key, "1d", "uploaded_time", 20, "recently_updated").await
}

#[tauri::command]
pub async fn get_monthly_top_nexus_mods(
    api_key: String,
) -> Result<Vec<NexusModSearchResult>, String> {
    fetch_nexus_mods_by_period(&api_key, "1m", "endorsements", 20, "monthly_top").await
}

#[tauri::command]
pub async fn browse_nexus_category(
    api_key: String,
    category_id: String,
) -> Result<Vec<NexusModSearchResult>, String> {
    let client = build_nexus_async_client();
    eprintln!("[browse_nexus_category] 浏览分类: category_id={}", category_id);

    let graphql_query = r#"
        query BrowseCategory($categoryIdFilter: [BaseFilterValue!], $gameIdFilter: [BaseFilterValue!]) {
            mods(filter: { categoryId: $categoryIdFilter, gameId: $gameIdFilter }) {
                totalCount
                nodes {
                    modId
                    gameId
                    name
                }
            }
        }
    "#;

    let request_body = serde_json::json!({
        "query": graphql_query,
        "variables": {
            "categoryIdFilter": [{"op": "EQUALS", "value": category_id}],
            "gameIdFilter": [{"op": "EQUALS", "value": "1303"}]
        }
    });

    let response = client
        .post(NEXUS_GRAPHQL_URL)
        .header("Content-Type", "application/json")
        .header("User-Agent", USER_AGENT)
        .header("apikey", &api_key)
        .body(request_body.to_string())
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("GraphQL 请求失败: {}", e))?;

    let status = response.status();
    eprintln!("[browse_nexus_category] GraphQL HTTP 状态码: {}", status);

    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_default();
        eprintln!("[browse_nexus_category] 错误响应: {}", error_body);
        return Err(format!("获取分类MOD失败 (状态码: {})", status));
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    if let Some(errors) = body.get("errors") {
        eprintln!("[browse_nexus_category] GraphQL 错误: {}", errors);
    }

    let nodes = body.get("data")
        .and_then(|d| d.get("mods"))
        .and_then(|m| m.get("nodes"))
        .and_then(|n| n.as_array())
        .cloned()
        .unwrap_or_default();

    let total = body.get("data")
        .and_then(|d| d.get("mods"))
        .and_then(|m| m.get("totalCount"))
        .and_then(|t| t.as_u64())
        .unwrap_or(0);

    eprintln!("[browse_nexus_category] GraphQL 返回 {} 个结果(总计 {})", nodes.len(), total);

    let mut mod_ids: Vec<u64> = Vec::new();
    for node in &nodes {
        if let Some(mod_id) = node.get("modId").and_then(|v| v.as_u64()) {
            mod_ids.push(mod_id);
        }
    }

    mod_ids.truncate(20);

    let mut handles: Vec<tokio::task::JoinHandle<Option<NexusModSearchResult>>> = Vec::new();
    for mod_id in mod_ids {
        let client_clone = client.clone();
        let api_key_clone = api_key.clone();
        handles.push(tokio::spawn(async move {
            let url = format!("{}/games/stardewvalley/mods/{}.json", NEXUS_API_BASE, mod_id);
            match client_clone
                .get(&url)
                .header("User-Agent", USER_AGENT)
                .header("apikey", &api_key_clone)
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<serde_json::Value>().await {
                        Ok(mod_data) => Some(parse_mod_search_result(&mod_data)),
                        Err(_) => None,
                    }
                }
                _ => None,
            }
        }));
    }

    let mut results: Vec<NexusModSearchResult> = Vec::new();
    for handle in handles {
        if let Ok(Some(mod_result)) = handle.await {
            results.push(mod_result);
        }
    }

    results.sort_by(|a, b| b.endorsements.cmp(&a.endorsements));

    eprintln!("[browse_nexus_category] 分类 {} 找到 {} 个MOD", category_id, results.len());
    Ok(results)
}

#[tauri::command]
pub async fn get_nexus_categories(
    api_key: String,
) -> Result<Vec<serde_json::Value>, String> {
    let client = build_nexus_async_client();
    eprintln!("[get_nexus_categories] 获取 N 网分类列表");

    let response = match client
        .get(format!("{}/games/stardewvalley/categories", NEXUS_API_BASE))
        .header("User-Agent", USER_AGENT)
        .header("apikey", &api_key)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("[get_nexus_categories] API 请求失败: {}", e);
            return Err(format!("获取分类失败: {}", e));
        }
    };

    if !response.status().is_success() {
        let status = response.status();
        eprintln!("[get_nexus_categories] API 返回错误: {}", status);
        return Err(format!("API 返回错误: {}", status));
    }

    let body: serde_json::Value = match response.json().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("[get_nexus_categories] 解析响应失败: {}", e);
            return Err(format!("解析响应失败: {}", e));
        }
    };

    eprintln!("[get_nexus_categories] 响应类型: {}", if body.is_array() { "array" } else if body.is_object() { "object" } else { "other" });

    let categories = if let Some(arr) = body.as_array() {
        arr.clone()
    } else if let Some(obj) = body.as_object() {
        if let Some(cats) = obj.get("categories").and_then(|c| c.as_array()) {
            cats.clone()
        } else if let Some(cats) = obj.get("data").and_then(|c| c.as_array()) {
            // Some APIs return data in a "data" field
            cats.clone()
        } else {
            eprintln!("[get_nexus_categories] 响应对象中没有 categories 或 data 字段");
            return Ok(vec![]);
        }
    } else {
        eprintln!("[get_nexus_categories] 响应格式未知");
        return Ok(vec![]);
    };

    if categories.is_empty() {
        eprintln!("[get_nexus_categories] API 返回空分类");
        eprintln!("[get_nexus_categories] 原始响应内容: {}", serde_json::to_string_pretty(&body).unwrap_or_else(|_| "无法解析".to_string()));
        return Ok(vec![]);
    }

    eprintln!("[get_nexus_categories] 获取到 {} 个分类", categories.len());
    Ok(categories)
}

fn find_and_normalize_download(temp_dir: &PathBuf) -> Option<PathBuf> {
    if let Ok(entries) = fs::read_dir(temp_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if !p.is_file() {
                continue;
            }
            let file_name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            eprintln!("[find_and_normalize] 检查文件: {}", file_name);

            let clean_name = extract_filename_from_url(file_name);
            let clean_path = temp_dir.join(&clean_name);

            if clean_path != p {
                eprintln!("[find_and_normalize] 文件名含URL参数，重命名: {} -> {}", file_name, clean_name);
                if let Err(e) = fs::rename(&p, &clean_path) {
                    eprintln!("[find_and_normalize] 重命名失败: {} (将使用原路径)", e);
                    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                    if ext == "zip" || ext == "7z" || ext == "rar" {
                        return Some(p);
                    }
                    continue;
                }
            }

            let ext = clean_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            if ext == "zip" || ext == "7z" || ext == "rar" {
                eprintln!("[find_and_normalize] 找到下载文件: {}", clean_path.display());
                return Some(clean_path);
            }
        }
    }
    eprintln!("[find_and_normalize] 临时目录中未找到下载文件");
    None
}

/// 打开内置 N 网浏览器窗口
/// WebView2 会继承 Edge 浏览器的登录状态（cookies），因此用户已登录的话无需再次登录
/// on_download 拦截下载事件，设置下载路径后由 WebView2 自己下载，完成后自动安装
#[tauri::command]
pub async fn open_nexus_browser(
    app: tauri::AppHandle,
    initial_url: Option<String>,
) -> Result<bool, String> {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tauri::Manager;
    use tauri::webview::DownloadEvent;

    eprintln!("[nexus_browser] 打开 N 网浏览器, initial_url={:?}", initial_url);

    let start_url = initial_url.clone().unwrap_or_else(|| {
        "https://www.nexusmods.com/stardewvalley".to_string()
    });

    if let Some(existing) = app.get_webview_window("nexus_browser") {
        eprintln!("[nexus_browser] 浏览器窗口已存在，导航到: {}", start_url);
        match start_url.parse::<tauri::Url>() {
            Ok(url) => { let _ = existing.navigate(url); }
            Err(e) => { eprintln!("[nexus_browser] URL 解析失败: {}", e); }
        }
        let _ = existing.set_focus();
        return Ok(true);
    }

    let mods_path = {
        if let Some((detected_path, _method)) = crate::smapi::find_game_path() {
            detected_path.join("Mods").to_string_lossy().to_string()
        } else {
            let fallback = get_svl_data_dir().join("Mods");
            let _ = fs::create_dir_all(&fallback);
            fallback.to_string_lossy().to_string()
        }
    };

    let temp_dir = PathBuf::from(&mods_path).join(".temp_nexus_download");
    if !temp_dir.exists() {
        let _ = fs::create_dir_all(&temp_dir);
    }

    let _app_dl = app.clone();
    let temp_dir_for_dl = temp_dir.clone();
    let mods_path_for_dl = mods_path.clone();
    let _app_install = app.clone();
    let download_finished_flag: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let download_success_flag: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let downloaded_file_path: Arc<std::sync::Mutex<Option<String>>> = Arc::new(std::sync::Mutex::new(None));
    let finished_flag_dl = download_finished_flag.clone();
    let success_flag_dl = download_success_flag.clone();
    let file_path_dl = downloaded_file_path.clone();
    let app_progress = app.clone();

    let auto_download_script = include_str!("../auto_download.js");

    let _webview_window = tauri::WebviewWindowBuilder::new(
        &app,
        "nexus_browser",
        tauri::WebviewUrl::External(start_url.parse().map_err(|e| format!("URL 解析失败: {}", e))?),
    )
    .title("Nexus Mods 浏览器 - 下载的模组将自动安装")
    .inner_size(1100.0, 750.0)
    .visible(true)
    .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
    .initialization_script(auto_download_script)
    .on_navigation(move |url| {
        let url_str = url.as_str();
        eprintln!("[nexus_browser] 导航: {}", url_str);
        true
    })
    .on_download(move |_webview, event| {
        match event {
            DownloadEvent::Requested { url, destination } => {
                let url_str = url.to_string();
                eprintln!("[nexus_browser] on_download Requested: {}", url_str);
                let file_name = extract_filename_from_url(&url_str);
                let safe_name = file_name.replace(|c: char| !c.is_alphanumeric() && c != '.' && c != '-' && c != '_', "_");
                if safe_name.is_empty() || safe_name == "_" {
                    eprintln!("[nexus_browser] 无法提取文件名，忽略");
                    return true;
                }
                let dest = temp_dir_for_dl.join(&safe_name);
                eprintln!("[nexus_browser] 拦截下载，重定向到: {}", dest.display());
                *destination = dest.clone();
                if let Ok(mut fp) = file_path_dl.lock() {
                    *fp = Some(dest.to_string_lossy().to_string());
                }
                let _ = app_progress.emit("mod-install-progress", serde_json::json!({
                    "step": "downloading_file",
                    "mod_name": "Nexus Mod",
                    "message": "正在下载文件...",
                }));
            }
            DownloadEvent::Finished { url, path, success } => {
                eprintln!("[nexus_browser] Download finished: {} success={}", url, success);
                if success {
                    if let Some(p) = path {
                        eprintln!("[nexus_browser] 下载完成: {}", p.display());
                        if let Ok(mut fp) = file_path_dl.lock() {
                            *fp = Some(p.to_string_lossy().to_string());
                        }
                    }
                    success_flag_dl.store(true, Ordering::SeqCst);
                }
                finished_flag_dl.store(true, Ordering::SeqCst);
            }
            _ => {}
        }
        true
    })
    .build()
    .map_err(|e| format!("创建浏览器窗口失败: {}", e))?;

    let app_wait = app.clone();
    let finished_flag_wait = download_finished_flag.clone();
    let success_flag_wait = download_success_flag.clone();
    let temp_dir_wait = temp_dir.clone();
    let mods_path_wait = mods_path_for_dl.clone();

    tokio::spawn(async move {
        for _ in 0..1200 {
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            if finished_flag_wait.load(Ordering::SeqCst) {
                break;
            }
        }

        if !success_flag_wait.load(Ordering::SeqCst) {
            return;
        }

        let _ = app_wait.emit("mod-install-progress", serde_json::json!({
            "step": "installing",
            "mod_name": "Nexus Mod",
            "message": "正在安装模组...",
        }));

        let archive_path = find_and_normalize_download(&temp_dir_wait);

        if let Some(archive) = archive_path {
            let path_str = archive.to_string_lossy().to_string();
            let result = crate::mod_installer::install_mod_from_archive_blocking(
                app_wait.clone(),
                path_str.clone(),
                mods_path_wait.clone(),
                None,
            );

            match result {
                Ok(install_result) => {
                    let mod_name = install_result.mod_name.as_deref().unwrap_or("Nexus Mod");
                    eprintln!("[nexus_browser] 安装成功: {}", mod_name);

                    let mods_dir = PathBuf::from(&mods_path_wait);
                    let installed_path = mods_dir.join(mod_name);
                    let verified = installed_path.exists() && installed_path.join("manifest.json").exists();
                    eprintln!("[nexus_browser] 安装验证: path={}, exists={}", installed_path.display(), verified);

                    let game_path = mods_dir.parent().map(|p| p.to_string_lossy().to_string());
                    let scan_found = game_path.as_ref().map_or(false, |gp| {
                        match crate::mod_parser::scan_mods(Some(gp.clone())) {
                            Ok(mods) => {
                                let found = mods.iter().any(|m| m.folder_path.contains(&mod_name));
                                eprintln!("[nexus_browser] scan_mods 验证: scanned={}, found_mod={}", mods.len(), found);
                                found
                            }
                            Err(e) => {
                                eprintln!("[nexus_browser] scan_mods 验证失败: {}", e);
                                false
                            }
                        }
                    });

                    let _ = app_wait.emit("mod-install-progress", serde_json::json!({
                        "step": "completed",
                        "mod_name": mod_name,
                        "mods_path": mods_path_wait,
                        "installed_path": installed_path.to_string_lossy().to_string(),
                        "verified": verified,
                        "scan_found": scan_found,
                        "message": format!("{} 安装成功! 路径: {}", mod_name, installed_path.display()),
                    }));
                }
                Err(e) => {
                    eprintln!("[nexus_browser] 安装失败: {}", e);
                    let _ = app_wait.emit("mod-install-progress", serde_json::json!({
                        "step": "error",
                        "mod_name": "Nexus Mod",
                        "message": format!("安装失败: {}", e),
                    }));
                }
            }
        } else {
            eprintln!("[nexus_browser] 未找到下载文件");
        }

        let _ = fs::remove_dir_all(&temp_dir_wait);
    });

    Ok(true)
}

/// 关闭内置 N 网浏览器窗口
#[tauri::command]
pub async fn close_nexus_browser(app: tauri::AppHandle) -> Result<bool, String> {
    use tauri::Manager;
    if let Some(webview_window) = app.get_webview_window("nexus_browser") {
        let _ = webview_window.close();
        eprintln!("[nexus_browser] 已关闭内置浏览器");
        Ok(true)
    } else {
        Ok(false)
    }
}

/// 通过内置浏览器下载并安装 N 网模组（唯一指定文件）
/// 
/// 原理：
/// 1. 打开内置浏览器窗口到 N 网下载页面
/// 2. WebView2 继承 Edge 浏览器的登录状态（cookies）
/// 3. 使用 initialization_script 自动点击 "Manual Download" 按钮
/// 4. 使用 on_download 拦截下载事件，设置下载路径到临时目录，由 WebView2 自己下载
/// 5. 等待下载完成后，解压安装到 Mods 目录
/// 6. 关闭浏览器窗口
async fn download_mod_via_webview(
    app: &tauri::AppHandle,
    mod_id: &str,
    file_id: &str,
    mod_name: &str,
    mods_path: &str,
    old_unique_id: Option<String>,
) -> Result<ModDownloadResult, String> {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tauri::Manager;
    use tauri::webview::DownloadEvent;

    eprintln!("[nexus_webview] 开始通过内置浏览器下载 mod_id={}, file_id={}, mod_name={}", mod_id, file_id, mod_name);

    let download_page_url = format!(
        "https://www.nexusmods.com/stardewvalley/mods/{}?tab=files&file_id={}",
        mod_id, file_id
    );

    let temp_dir = PathBuf::from(mods_path).join(".temp_nexus_download");
    if temp_dir.exists() {
        let _ = fs::remove_dir_all(&temp_dir);
    }
    fs::create_dir_all(&temp_dir).map_err(|e| format!("创建临时文件夹失败: {}", e))?;

    let download_finished = Arc::new(AtomicBool::new(false));
    let download_success = Arc::new(AtomicBool::new(false));
    let downloaded_file_path: Arc<std::sync::Mutex<Option<String>>> = Arc::new(std::sync::Mutex::new(None));

    let finished_flag_dl = download_finished.clone();
    let success_flag_dl = download_success.clone();
    let file_path_dl = downloaded_file_path.clone();
    let app_dl = app.clone();
    let temp_dir_for_dl = temp_dir.clone();
    let mod_name_for_dl = mod_name.to_string();

    if let Some(existing) = app.get_webview_window("nexus_download") {
        let _ = existing.close();
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    }

    let _ = app.emit("mod-install-progress", serde_json::json!({
        "step": "downloading",
        "mod_name": mod_name,
        "message": format!("正在打开下载页面，请稍候.."),
    }));

    let mut auto_download_script = include_str!("../auto_download.js").to_string();
    auto_download_script = auto_download_script.replace("\"SVL_TARGET_MOD_ID\"", mod_id);
    auto_download_script = auto_download_script.replace("\"SVL_TARGET_FILE_ID\"", file_id);

    let _webview_window = tauri::WebviewWindowBuilder::new(
        app,
        "nexus_download",
        tauri::WebviewUrl::External(download_page_url.parse().map_err(|e| format!("URL 解析失败: {}", e))?),
    )
    .title(format!("Nexus Mods 下载 - {} - 自动获取中..", mod_name))
    .inner_size(1000.0, 700.0)
    .visible(true)
    .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
    .initialization_script(auto_download_script)
    .on_navigation(move |url| {
        let url_str = url.as_str();
        eprintln!("[nexus_webview] 导航: {}", url_str);
        true
    })
    .on_download(move |_webview, event| {
        match event {
            DownloadEvent::Requested { url, destination } => {
                let url_str = url.to_string();
                eprintln!("[nexus_webview] on_download Requested: {}", url_str);
                let file_name = extract_filename_from_url(&url_str);
                let safe_name = file_name.replace(|c: char| !c.is_alphanumeric() && c != '.' && c != '-' && c != '_', "_");
                if safe_name.is_empty() || safe_name == "_" {
                    eprintln!("[nexus_webview] 无法提取文件名，忽略");
                    return true;
                }
                let dest = temp_dir_for_dl.join(&safe_name);
                eprintln!("[nexus_webview] 拦截下载，重定向到: {}", dest.display());
                *destination = dest.clone();
                if let Ok(mut fp) = file_path_dl.lock() {
                    *fp = Some(dest.to_string_lossy().to_string());
                }
                let _ = app_dl.emit("mod-install-progress", serde_json::json!({
                    "step": "downloading_file",
                    "mod_name": &mod_name_for_dl,
                    "message": "正在下载文件...",
                }));
            }
            DownloadEvent::Finished { url, path, success } => {
                eprintln!("[nexus_webview] Download finished: {} success={}", url, success);
                if success {
                    if let Some(p) = path {
                        eprintln!("[nexus_webview] 下载完成: {}", p.display());
                        if let Ok(mut fp) = file_path_dl.lock() {
                            *fp = Some(p.to_string_lossy().to_string());
                        }
                    }
                    success_flag_dl.store(true, Ordering::SeqCst);
                }
                finished_flag_dl.store(true, Ordering::SeqCst);
            }
            _ => {}
        }
        true
    })
    .build()
    .map_err(|e| format!("创建浏览器窗口失败: {}", e))?;

    for _ in 0..1200 {
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        if download_finished.load(Ordering::SeqCst) {
            break;
        }
    }

    let _ = app.emit("mod-install-progress", serde_json::json!({
        "step": "installing",
        "mod_name": mod_name,
        "message": "正在安装模组...",
    }));

    if let Some(wv) = app.get_webview_window("nexus_download") {
        let _ = wv.close();
    }

    if !download_success.load(Ordering::SeqCst) {
        let _ = fs::remove_dir_all(&temp_dir);
        if !download_finished.load(Ordering::SeqCst) {
            return Err("下载超时：请确保已在内置浏览器中登录 N 网账号".to_string());
        } else {
            return Err("下载失败：文件未能成功下载，请重试".to_string());
        }
    }

    let archive_path = find_and_normalize_download(&temp_dir);

    match archive_path {
        Some(archive) => {
            let path_str = archive.to_string_lossy().to_string();
            let result = crate::mod_installer::install_mod_from_archive_blocking(
                app.clone(),
                path_str.clone(),
                mods_path.to_string(),
                old_unique_id,
            );

            let _ = fs::remove_dir_all(&temp_dir);

            match result {
                Ok(install_result) => {
                    eprintln!("[nexus_webview] 安装成功: {}", install_result.mod_name.as_deref().unwrap_or("unknown"));
                    Ok(ModDownloadResult {
                        success: true,
                        mod_name: install_result.mod_name.unwrap_or_else(|| mod_name.to_string()),
                        mod_version: String::new(),
                        message: install_result.message,
                        file_size: 0,
                    })
                }
                Err(e) => {
                    eprintln!("[nexus_webview] 安装失败: {}", e);
                    Ok(ModDownloadResult {
                        success: false,
                        mod_name: mod_name.to_string(),
                        mod_version: String::new(),
                        message: format!("下载成功但安装失败: {}", e),
                        file_size: 0,
                    })
                }
            }
        }
        None => {
            let _ = fs::remove_dir_all(&temp_dir);
            Err("下载失败：未获取到下载文件路径".to_string())
        }
    }
}

/// 从页面 HTML 中提取 CDN 下载链接
fn extract_cdn_url_from_html(html: &str) -> Option<String> {
    // Pattern 1: data-url attribute on download buttons
    if let Some(caps) = regex::Regex::new(r#"data-url="([^"]*nexus-cdn[^"]*)""#)
        .ok()
        .and_then(|re| re.captures(html))
    {
        return Some(caps[1].to_string());
    }

    // Pattern 2: href on download links
    if let Some(caps) = regex::Regex::new(r#"href="([^"]*supporter-files[^"]*)""#)
        .ok()
        .and_then(|re| re.captures(html))
    {
        return Some(caps[1].to_string());
    }

    // Pattern 3: Any nexus-cdn URL in the page
    if let Some(caps) = regex::Regex::new(r#"(https://[^"'\s<>]*nexus-cdn[^"'\s<>]*)"#)
        .ok()
        .and_then(|re| re.captures(html))
    {
        return Some(caps[1].to_string());
    }

    None
}

/// 直接从 CDN 链接下载并安装模组（由内置浏览器捕获 CDN 链接后调用）
#[tauri::command]
pub async fn download_mod_from_cdn_link(
    app: tauri::AppHandle,
    cdn_link: String,
    mods_path: Option<String>,
) -> Result<ModDownloadResult, String> {
    let mods_path = match mods_path {
        Some(path) => path,
        None => {
            if let Some((detected_path, _method)) = crate::smapi::find_game_path() {
                detected_path.join("Mods").to_string_lossy().to_string()
            } else {
                let default_paths = [
                    r"C:\Program Files (x86)\Steam\steamapps\common\Stardew Valley",
                    r"C:\Program Files\Steam\steamapps\common\Stardew Valley",
                    r"D:\steam\steamapps\common\Stardew Valley",
                    r"C:\GOG Games\Stardew Valley",
                ];
                let mut found = None;
                for default in &default_paths {
                    let path = PathBuf::from(default);
                    if path.exists() {
                        found = Some(path);
                        break;
                    }
                }
                match found {
                    Some(p) => p.join("Mods").to_string_lossy().to_string(),
                    None => return Err("未找到星露谷物语安装目录".to_string()),
                }
            }
        }
    };

    eprintln!("[download_mod_from_cdn_link] 从 CDN 链接下载: {}", cdn_link);

    let _ = app.emit("mod-install-progress", serde_json::json!({
        "step": "downloading",
        "mod_name": "Nexus Mod",
        "message": "正在从 CDN 下载模组...",
    }));

    let client = build_download_client();

    let temp_dir = PathBuf::from(&mods_path).join(".temp_nexus_download");
    if temp_dir.exists() {
        let _ = fs::remove_dir_all(&temp_dir);
    }
    fs::create_dir_all(&temp_dir).map_err(|e| format!("创建临时文件夹失败: {}", e))?;

    let file_name = extract_filename_from_url(&cdn_link);

    let lower = file_name.to_lowercase();
    let temp_file_path = if lower.ends_with(".zip") || lower.ends_with(".7z") || lower.ends_with(".rar") {
        temp_dir.join(&file_name)
    } else {
        temp_dir.join(format!("{}.zip", file_name))
    };

    let _ = app.emit("mod-install-progress", serde_json::json!({
        "step": "downloading_file",
        "mod_name": "Nexus Mod",
        "message": "正在下载文件...",
    }));

    download_file_with_progress(&client, &cdn_link, &temp_file_path, &app, "Nexus Mod").await?;

    let _ = app.emit("mod-install-progress", serde_json::json!({
        "step": "installing",
        "mod_name": "Nexus Mod",
        "message": "正在安装模组...",
    }));

    let archive_path = temp_file_path.to_string_lossy().to_string();
    let result = crate::mod_installer::install_mod_from_archive_blocking(
        app.clone(),
        archive_path,
        mods_path.clone(),
        None,
    );

    let _ = fs::remove_file(&temp_file_path);
    let _ = fs::remove_dir_all(&temp_dir);

    match result {
        Ok(install_result) => {
            eprintln!("[download_mod_from_cdn_link] 安装成功: {}", install_result.mod_name.as_deref().unwrap_or("unknown"));
            Ok(ModDownloadResult {
                success: true,
                mod_name: install_result.mod_name.unwrap_or_else(|| "Nexus Mod".to_string()),
                mod_version: String::new(),
                message: install_result.message,
                file_size: 0,
            })
        }
        Err(e) => {
            eprintln!("[download_mod_from_cdn_link] 安装失败: {}", e);
            Ok(ModDownloadResult {
                success: false,
                mod_name: "Nexus Mod".to_string(),
                mod_version: String::new(),
                message: format!("下载成功但安装失败: {}", e),
                file_size: 0,
            })
        }
    }
}

#[tauri::command]
pub async fn download_mod_from_nexus(
    app: tauri::AppHandle,
    mod_id: String,
    api_key: String,
    mods_path: Option<String>,
    file_id: Option<String>,
    old_unique_id: Option<String>,
) -> Result<ModDownloadResult, String> {
    let mods_path = match mods_path {
        Some(path) => path,
        None => {
            if let Some((detected_path, _method)) = crate::smapi::find_game_path() {
                detected_path.join("Mods").to_string_lossy().to_string()
            } else {
                let default_paths = [
                    r"C:\Program Files (x86)\Steam\steamapps\common\Stardew Valley",
                    r"C:\Program Files\Steam\steamapps\common\Stardew Valley",
                    r"D:\steam\steamapps\common\Stardew Valley",
                    r"C:\GOG Games\Stardew Valley",
                ];
                let mut found = None;
                for default in &default_paths {
                    let path = PathBuf::from(default);
                    if path.exists() {
                        found = Some(path);
                        break;
                    }
                }
                match found {
                    Some(p) => p.join("Mods").to_string_lossy().to_string(),
                    None => return Err("未找到星露谷物语安装目录，请先在模组管理中设置游戏路径".to_string()),
                }
            }
        }
    };

    eprintln!("[download_mod_from_nexus] 开始下载 mod_id={}, mods_path={}", mod_id, mods_path);

    let client = build_nexus_async_client();
    let mod_info = get_nexus_mod_info_async(&client, &api_key, &mod_id).await?;
    let mod_name = mod_info.name.clone();

    let files = get_mod_files_via_api(&client, &api_key, &mod_id).await?;
    if files.is_empty() {
        return Err(format!("模组 '{}' 没有可用的下载文件", mod_name));
    }

    let target_file = match file_id {
        Some(fid) => files.into_iter().find(|f| f.file_id == fid)
            .ok_or_else(|| "指定的文件不存在".to_string())?,
        None => {
            let non_premium: Vec<_> = files.into_iter()
                .filter(|f| !f.is_premium_only)
                .collect();

            let main_files: Vec<_> = non_premium.iter()
                .filter(|f| f.category_id == 1)
                .cloned()
                .collect();
            if !main_files.is_empty() {
                let mut sorted = main_files;
                sorted.sort_by(|a, b| b.upload_time.cmp(&a.upload_time));
                sorted.into_iter().next()
                    .ok_or_else(|| "该MOD没有可下载的文件".to_string())?
            } else {
                let update_files: Vec<_> = non_premium.iter()
                    .filter(|f| f.category_id == 2)
                    .cloned()
                    .collect();
                if !update_files.is_empty() {
                    let mut sorted = update_files;
                    sorted.sort_by(|a, b| b.upload_time.cmp(&a.upload_time));
                    sorted.into_iter().next()
                        .ok_or_else(|| "该MOD没有可下载的文件".to_string())?
                } else {
                    let mut sorted = non_premium;
                    sorted.sort_by(|a, b| b.upload_time.cmp(&a.upload_time));
                    sorted.into_iter().next()
                        .ok_or_else(|| "该MOD没有可下载的文件".to_string())?
                }
            }
        }
    };

    log_info("NexusDownload", &format!(
        "Selected file: {} (file_id={}, category={}, version={})",
        target_file.name, target_file.file_id, target_file.category_id, target_file.version
    ));

    if !api_key.is_empty() {
        let client_dl = build_nexus_async_client();
        match get_nexus_file_download_link(&client_dl, &api_key, &mod_id, &target_file.file_id).await {
            Ok(cdn_uri) => {
                eprintln!("[download_mod_from_nexus] API 直接下载: mod_id={}, file_id={}, cdn_uri={}", mod_id, target_file.file_id, cdn_uri);
                let _ = app.emit("mod-install-progress", serde_json::json!({
                    "step": "downloading",
                    "mod_name": mod_name,
                    "message": "正在通过 API 直接下载模组...",
                }));

                let temp_dir = PathBuf::from(&mods_path).join(".temp_nexus_download");
                if temp_dir.exists() { let _ = fs::remove_dir_all(&temp_dir); }
                fs::create_dir_all(&temp_dir).map_err(|e| format!("创建临时文件夹失败: {}", e))?;

                let file_name = extract_filename_from_url(&cdn_uri);
                let lower = file_name.to_lowercase();
                let temp_file_path = if lower.ends_with(".zip") || lower.ends_with(".7z") || lower.ends_with(".rar") {
                    temp_dir.join(&file_name)
                } else {
                    temp_dir.join(format!("{}.zip", mod_name.replace(|c: char| !c.is_alphanumeric() && c != '.' && c != '-' && c != '_', "_")))
                };

                let dl_client = build_download_client();
                match download_file_with_progress(&dl_client, &cdn_uri, &temp_file_path, &app, &mod_name).await {
                    Ok(()) => {
                        let _ = app.emit("mod-install-progress", serde_json::json!({
                            "step": "installing",
                            "mod_name": mod_name,
                            "message": "正在安装模组...",
                        }));

                        let archive_path = temp_file_path.to_string_lossy().to_string();
                        let result = crate::mod_installer::install_mod_from_archive_blocking(
                            app.clone(),
                            archive_path,
                            mods_path.clone(),
                            old_unique_id.clone(),
                        );

                        let _ = fs::remove_file(&temp_file_path);
                        let _ = fs::remove_dir_all(&temp_dir);

                        match result {
                            Ok(install_result) => {
                                eprintln!("[download_mod_from_nexus] API 下载安装成功: {}", install_result.mod_name.as_deref().unwrap_or("unknown"));
                                return Ok(ModDownloadResult {
                                    success: true,
                                    mod_name: install_result.mod_name.unwrap_or_else(|| mod_name.clone()),
                                    mod_version: target_file.version.clone(),
                                    message: install_result.message,
                                    file_size: 0,
                                });
                            }
                            Err(e) => {
                                eprintln!("[download_mod_from_nexus] API 下载安装失败: {}, 降级为 WebView 下载", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[download_mod_from_nexus] API 下载文件失败: {}, 降级为 WebView 下载", e);
                    }
                }

                let _ = fs::remove_dir_all(&temp_dir);
            }
            Err(e) => {
                eprintln!("[download_mod_from_nexus] 获取 CDN 链接失败: {}, 降级为 WebView 下载", e);
            }
        }
    } else {
        eprintln!("[download_mod_from_nexus] 无 API Key，使用 WebView 下载");
    }

    download_mod_via_webview(&app, &mod_id, &target_file.file_id, &mod_name, &mods_path, old_unique_id).await
}

async fn get_mod_files_via_api(
    client: &reqwest::Client,
    api_key: &str,
    mod_id: &str,
) -> Result<Vec<NexusFileDownloadInfo>, String> {
    let response = add_nexus_async_headers(
        client.get(format!(
            "{}/games/{}/mods/{}/files.json",
            NEXUS_API_BASE, STARDEW_GAME_ID, mod_id
        )),
        api_key,
    )
    .send()
    .await
    .map_err(|e| format!("获取文件列表失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("获取文件列表失败 (状态码: {})", response.status()));
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析文件列表失败: {}", e))?;

    let files_arr = body.get("files")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    Ok(files_arr.into_iter().map(|f| NexusFileDownloadInfo {
        file_id: f["file_id"].as_u64().unwrap_or(0).to_string(),
        name: f["file_name"].as_str().unwrap_or("").to_string(),
        version: f["version"].as_str().unwrap_or("").to_string(),
        size: f["size_in_bytes"].as_u64().unwrap_or(0),
        upload_time: f["uploaded_time"].as_str().unwrap_or("").to_string(),
        download_url: None,
        is_premium_only: f["is_premium"].as_bool().unwrap_or(false),
        category_id: f["category_id"].as_i64().unwrap_or(1),
    }).collect())
}

async fn get_nexus_file_download_link(
    client: &reqwest::Client,
    api_key: &str,
    mod_id: &str,
    file_id: &str,
) -> Result<String, String> {
    let response = add_nexus_async_headers(
        client.get(format!(
            "{}/games/{}/mods/{}/files/{}/download-link.json",
            NEXUS_API_BASE, STARDEW_GAME_ID, mod_id, file_id
        )),
        api_key,
    )
    .send()
    .await
    .map_err(|e| format!("获取下载链接失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("获取下载链接失败 (状态码: {})", response.status()));
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析下载链接失败: {}", e))?;

    let links = body.as_array()
        .ok_or_else(|| "下载链接响应格式错误，期望数组".to_string())?;

    let first_link = links.first()
        .ok_or_else(|| "下载链接列表为空".to_string())?;

    let uri = first_link["URI"]
        .as_str()
        .ok_or_else(|| "下载响应中未找到 URI".to_string())?
        .to_string();

    Ok(uri)
}

async fn download_file_with_progress(
    client: &reqwest::Client,
    url: &str,
    dest: &PathBuf,
    app: &tauri::AppHandle,
    mod_name: &str,
) -> Result<(), String> {
    let response = client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .map_err(|e| format!("下载请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("下载失败 (状态码: {})", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut file = fs::File::create(dest).map_err(|e| format!("创建文件失败: {}", e))?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("下载数据块失败: {}", e))?;
        std::io::Write::write_all(&mut file, &chunk).map_err(|e| format!("写入文件失败: {}", e))?;
        downloaded += chunk.len() as u64;

        if total_size > 0 {
            let percent = ((downloaded as f64 / total_size as f64) * 100.0) as u32;
            let _ = app.emit("mod-install-progress", serde_json::json!({
                "step": "download_progress",
                "mod_name": mod_name,
                "message": format!("已下载 {}%", percent),
                "percent": percent,
            }));
        }
    }

    Ok(())
}

fn extract_filename_from_url(url_str: &str) -> String {
    let no_query = url_str.split('?').next().unwrap_or(url_str);
    let no_fragment = no_query.split('#').next().unwrap_or(no_query);
    no_fragment.split('/').last().unwrap_or("mod_download.zip").to_string()
}

fn sanitize_file_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' || c == '.' { c } else { '_' })
        .collect::<String>()
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_extract_nexus_id_numeric_update_key() {
        let tmp = tempfile::tempdir().unwrap();
        let mod_dir = tmp.path().join("TestMod");
        std::fs::create_dir_all(&mod_dir).unwrap();
        std::fs::write(mod_dir.join("manifest.json"), r#"{
            "Name": "Test Mod",
            "UniqueID": "Test.Mod",
            "Version": "1.0.0",
            "UpdateKeys": [1915]
        }"#).unwrap();

        let result = extract_nexus_id_from_manifest(mod_dir.to_str().unwrap());
        assert_eq!(result, Some("1915".to_string()),
            "Numeric UpdateKeys should be handled like 'Nexus:1915'");
    }

    #[test]
    fn test_extract_nexus_id_smart_quotes() {
        let tmp = tempfile::tempdir().unwrap();
        let mod_dir = tmp.path().join("TestMod");
        std::fs::create_dir_all(&mod_dir).unwrap();
        let smart_quote_manifest = format!("{{
            \u{201C}Name\u{201D}: \u{201C}Test Mod\u{201D},
            \u{201C}UniqueID\u{201D}: \u{201C}Test.Mod\u{201D},
            \u{201C}Version\u{201D}: \u{201C}1.0.0\u{201D},
            \u{201C}UpdateKeys\u{201D}: [\u{201C}Nexus:1915\u{201D}]
        }}");
        std::fs::write(mod_dir.join("manifest.json"), smart_quote_manifest).unwrap();

        let result = extract_nexus_id_from_manifest(mod_dir.to_str().unwrap());
        assert_eq!(result, Some("1915".to_string()),
            "Should parse manifest with smart quotes and extract Nexus ID");
    }

    #[test]
    fn test_extract_nexus_id_string_update_key() {
        let tmp = tempfile::tempdir().unwrap();
        let mod_dir = tmp.path().join("TestMod");
        std::fs::create_dir_all(&mod_dir).unwrap();
        std::fs::write(mod_dir.join("manifest.json"), r#"{
            "Name": "Test Mod",
            "UniqueID": "Test.Mod",
            "Version": "1.0.0",
            "UpdateKeys": ["Nexus:1915"]
        }"#).unwrap();

        let result = extract_nexus_id_from_manifest(mod_dir.to_str().unwrap());
        assert_eq!(result, Some("1915".to_string()),
            "String Nexus UpdateKeys should work");
    }
}

