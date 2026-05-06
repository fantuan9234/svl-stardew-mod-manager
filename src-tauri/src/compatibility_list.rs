/// SMAPI官方兼容性列表模块
/// 
/// 重要说明：SMAPI mods.json 不包含依赖关系信息，仅包含模组元数据。
/// 因此，本模块的设计目标是：
/// 1. 提供权威的模组元数据（名称、作者、Nexus ID等）
/// 2. 用于更新检测（对比本地版本与最新版本）
/// 3. 用于名称解析（解决模组显示名称不一致的问题）
/// 
/// 依赖关系解析仍然依赖本地manifest.json和硬编码补丁。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

const SMAPI_MODS_JSON_URL: &str = "https://raw.githubusercontent.com/Pathoschild/SmapiCompatibilityList/develop/data/mods.jsonc";
const COMPAT_CACHE_FILENAME: &str = "compatibility_cache.json";
const COMPAT_UPDATE_TIME_FILENAME: &str = "compatibility_update_time.txt";
const AUTO_UPDATE_INTERVAL_DAYS: i64 = 7;

static COMPAT_CACHE: OnceLock<HashMap<String, SmapiModMetadata>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmapiModMetadata {
    pub name: String,
    pub author: String,
    pub unique_id: String,
    pub nexus_id: Option<i64>,
    pub github_repo: Option<String>,
    pub status: Option<String>,
    pub broke_in: Option<String>,
    pub summary: Option<String>,
    pub content_pack_for: Option<String>,
    pub unofficial_update_url: Option<String>,
    pub unofficial_update_version: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CompatUpdateResult {
    pub success: bool,
    pub total_mods: usize,
    pub message: String,
    pub last_update: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct SmapiModEntryRaw {
    name: Option<String>,
    author: Option<String>,
    id: Option<String>,
    nexus: Option<Option<i64>>,
    github: Option<Option<String>>,
    status: Option<String>,
    #[serde(rename = "brokeIn")]
    broke_in: Option<String>,
    summary: Option<String>,
    #[serde(rename = "contentPackFor")]
    content_pack_for: Option<String>,
    #[serde(rename = "unofficialUpdate")]
    unofficial_update: Option<UnofficialUpdateRaw>,
}

#[derive(Debug, Clone, Deserialize)]
struct UnofficialUpdateRaw {
    version: Option<String>,
    url: Option<String>,
}

fn get_cache_dir() -> PathBuf {
    let mut path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    path.pop();
    path.push("assets");
    path
}

fn get_cache_path() -> PathBuf {
    get_cache_dir().join(COMPAT_CACHE_FILENAME)
}

fn get_update_time_path() -> PathBuf {
    get_cache_dir().join(COMPAT_UPDATE_TIME_FILENAME)
}

pub async fn fetch_and_cache_compatibility_list() -> Result<CompatUpdateResult, String> {
    eprintln!("[compatibility_list] Fetching SMAPI compatibility list");

    let client = reqwest::Client::builder()
        .user_agent("SVL-Stardew-Valley-Launcher/1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let response = client
        .get(SMAPI_MODS_JSON_URL)
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("下载失败 (状态码: {})", response.status()));
    }

    let body = response
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    let cleaned = strip_jsonc_comments(&body);

    let mods_json: serde_json::Value = serde_json::from_str(&cleaned)
        .map_err(|e| format!("解析 JSON 失败: {}", e))?;

    let mods_array = mods_json["mods"]
        .as_array()
        .ok_or("JSON 格式错误：未找到 mods 数组")?;

    let mut cache = HashMap::new();

    for mod_entry in mods_array {
        let raw: SmapiModEntryRaw = match serde_json::from_value(mod_entry.clone()) {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("[compatibility_list] Failed to parse mod entry: {}", e);
                continue;
            }
        };

        if let Some(unique_id) = raw.id {
            let unique_id = unique_id.split(',').next().unwrap_or(&unique_id).trim().to_string();

            let metadata = SmapiModMetadata {
                name: raw.name.unwrap_or_default(),
                author: raw.author.unwrap_or_default(),
                unique_id: unique_id.clone(),
                nexus_id: raw.nexus.flatten(),
                github_repo: raw.github.flatten(),
                status: raw.status,
                broke_in: raw.broke_in,
                summary: raw.summary,
                content_pack_for: raw.content_pack_for,
                unofficial_update_url: raw.unofficial_update.as_ref().and_then(|u| u.url.clone()),
                unofficial_update_version: raw.unofficial_update.as_ref().and_then(|u| u.version.clone()),
            };

            cache.insert(unique_id, metadata);
        }
    }

    eprintln!("[compatibility_list] Parsed {} mods from SMAPI compatibility list", cache.len());

    let cache_path = get_cache_path();
    if let Some(cache_dir) = cache_path.parent() {
        if !cache_dir.exists() {
            let _ = fs::create_dir_all(cache_dir);
        }
    }

    let cache_json = serde_json::to_string_pretty(&cache)
        .map_err(|e| format!("序列化缓存失败: {}", e))?;

    fs::write(&cache_path, &cache_json)
        .map_err(|e| format!("写入缓存失败: {}", e))?;

    let now = chrono::Utc::now().timestamp();
    let _ = fs::write(get_update_time_path(), now.to_string());

    COMPAT_CACHE.get_or_init(|| cache.clone());

    let last_update = Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());

    Ok(CompatUpdateResult {
        success: true,
        total_mods: cache.len(),
        message: format!("成功更新，缓存 {} 个模组元数据", cache.len()),
        last_update,
    })
}

pub async fn init_compatibility_cache() {
    if COMPAT_CACHE.get().is_some() {
        eprintln!("[compatibility_list] Cache already initialized");
        return;
    }

    eprintln!("[compatibility_list] Initializing compatibility cache");

    let cache_path = get_cache_path();
    if cache_path.exists() {
        if let Ok(content) = fs::read_to_string(&cache_path) {
            if let Ok(cache) = serde_json::from_str::<HashMap<String, SmapiModMetadata>>(&content) {
                eprintln!("[compatibility_list] Loaded {} mods from local cache", cache.len());
                COMPAT_CACHE.get_or_init(|| cache);
                return;
            }
        }
    }

    eprintln!("[compatibility_list] Local cache not found, fetching from network");

    match fetch_and_cache_compatibility_list().await {
        Ok(result) => {
            eprintln!("[compatibility_list] Network fetch successful: {}", result.message);
        }
        Err(e) => {
            eprintln!("[compatibility_list] Network fetch failed: {}", e);
            COMPAT_CACHE.get_or_init(HashMap::new);
        }
    }
}

pub fn get_mod_metadata(mod_unique_id: &str) -> Option<SmapiModMetadata> {
    let cache = COMPAT_CACHE.get_or_init(HashMap::new);

    if let Some(metadata) = cache.get(mod_unique_id) {
        return Some(metadata.clone());
    }

    for (key, metadata) in cache {
        if key.to_lowercase() == mod_unique_id.to_lowercase() {
            return Some(metadata.clone());
        }
    }

    None
}

pub fn get_last_update_time() -> Option<String> {
    let path = get_update_time_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(timestamp) = content.trim().parse::<i64>() {
                let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(timestamp, 0);
                if let Some(dt) = dt {
                    return Some(dt.format("%Y-%m-%d %H:%M:%S").to_string());
                }
            }
        }
    }
    None
}

pub fn should_auto_update() -> bool {
    let path = get_update_time_path();
    if !path.exists() {
        return true;
    }

    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(timestamp) = content.trim().parse::<i64>() {
            let now = chrono::Utc::now().timestamp();
            let seconds_in_7_days = AUTO_UPDATE_INTERVAL_DAYS * 24 * 60 * 60;
            return (now - timestamp) > seconds_in_7_days;
        }
    }

    true
}

pub async fn auto_update_compatibility_list() {
    if !should_auto_update() {
        eprintln!("[compatibility_list] Auto-update skipped, cache is up to date");
        return;
    }

    eprintln!("[compatibility_list] Auto-update triggered (7+ days since last update)");

    match fetch_and_cache_compatibility_list().await {
        Ok(result) => {
            eprintln!("[compatibility_list] Auto-update successful: {}", result.message);
        }
        Err(e) => {
            eprintln!("[compatibility_list] Auto-update failed: {}", e);
        }
    }
}

fn strip_jsonc_comments(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut in_string = false;
    let mut escape_next = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if escape_next {
            result.push(c);
            escape_next = false;
            continue;
        }

        if c == '\\' && in_string {
            result.push(c);
            escape_next = true;
            continue;
        }

        if c == '"' {
            in_string = !in_string;
            result.push(c);
            continue;
        }

        if in_string {
            result.push(c);
            continue;
        }

        if c == '/' {
            if let Some(&next) = chars.peek() {
                if next == '/' {
                    chars.next();
                    while let Some(line_c) = chars.next() {
                        if line_c == '\n' {
                            result.push(line_c);
                            break;
                        }
                    }
                    continue;
                } else if next == '*' {
                    chars.next();
                    let mut prev = '\0';
                    while let Some(block_c) = chars.next() {
                        if prev == '*' && block_c == '/' {
                            break;
                        }
                        prev = block_c;
                    }
                    continue;
                }
            }
        }

        result.push(c);
    }

    result
}

#[tauri::command]
pub async fn update_compatibility_list() -> Result<CompatUpdateResult, String> {
    fetch_and_cache_compatibility_list().await
}

#[tauri::command]
pub fn get_compatibility_status() -> Result<serde_json::Value, String> {
    let last_update = get_last_update_time();
    let cache = COMPAT_CACHE.get_or_init(HashMap::new);

    Ok(serde_json::json!({
        "lastUpdate": last_update,
        "totalMods": cache.len(),
        "hasData": !cache.is_empty()
    }))
}
