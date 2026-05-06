use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

const SMAPI_MODS_JSON_URL: &str = "https://raw.githubusercontent.com/Pathoschild/SmapiCompatibilityList/develop/data/mods.jsonc";
const MOD_DICT_FILENAME: &str = "mod_dict.json";
const UPDATE_TIMESTAMP_FILENAME: &str = "mod_dict_update_time.txt";
const AUTO_UPDATE_INTERVAL_DAYS: i64 = 7;

static MOD_DICT_CACHE: OnceLock<HashMap<String, String>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDictUpdateResult {
    pub success: bool,
    pub new_entries: usize,
    pub total_entries: usize,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
struct SmapiModEntry {
    #[serde(rename = "UniqueID")]
    unique_id: Option<String>,
    #[serde(rename = "NexusID")]
    nexus_id: Option<serde_json::Value>,
    #[serde(rename = "Name")]
    name: Option<String>,
}

#[tauri::command]
pub async fn update_mod_dict() -> Result<ModDictUpdateResult, String> {
    eprintln!("[update_mod_dict] Starting MOD dictionary update from SMAPI official list");

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

    let mods_array = mods_json["Mods"]
        .as_array()
        .ok_or("JSON 格式错误：未找到 Mods 数组")?;

    let mut new_mapping = HashMap::new();
    let mut parsed_count = 0;

    for mod_entry in mods_array {
        let smapi_entry: SmapiModEntry = match serde_json::from_value(mod_entry.clone()) {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("[update_mod_dict] 解析条目失败: {}", e);
                continue;
            }
        };

        if let Some(unique_id) = smapi_entry.unique_id {
            if let Some(ref nexus_value) = smapi_entry.nexus_id {
                if let Some(nexus_id) = extract_nexus_id(nexus_value) {
                    new_mapping.insert(unique_id.clone(), nexus_id);
                    parsed_count += 1;
                }
            }
            if let Some(name) = smapi_entry.name {
                if let Some(ref nexus_value) = smapi_entry.nexus_id {
                    if let Some(nexus_id) = extract_nexus_id(nexus_value) {
                        new_mapping.insert(name.clone(), nexus_id);
                    }
                }
            }
        }
    }

    eprintln!("[update_mod_dict] Parsed {} entries from SMAPI mods.json", parsed_count);

    let local_dict_path = get_mod_dict_path();
    let existing_dict = load_local_dict(&local_dict_path);

    let mut merged_dict = existing_dict.clone();
    let mut new_entries_count = 0;

    for (key, nexus_id) in &new_mapping {
        if !merged_dict.contains_key(key) {
            new_entries_count += 1;
        }
        merged_dict.insert(key.clone(), nexus_id.clone());
    }

    let dict_json = serde_json::to_string_pretty(&merged_dict)
        .map_err(|e| format!("序列化字典失败: {}", e))?;

    fs::write(&local_dict_path, &dict_json)
        .map_err(|e| format!("写入本地字典失败: {}", e))?;

    eprintln!("[update_mod_dict] Saved {} entries to {:?}", merged_dict.len(), local_dict_path);

    save_update_timestamp();

    MOD_DICT_CACHE.set(merged_dict.clone()).ok();

    Ok(ModDictUpdateResult {
        success: true,
        new_entries: new_entries_count,
        total_entries: merged_dict.len(),
        message: format!("成功更新字典，新增 {} 个条目", new_entries_count),
    })
}

fn strip_jsonc_comments(content: &str) -> String {
    let mut result = String::new();
    let mut in_string = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut prev_char = '\0';

    for c in content.chars() {
        if in_line_comment {
            if c == '\n' {
                in_line_comment = false;
                result.push(c);
            }
            continue;
        }

        if in_block_comment {
            if c == '/' && prev_char == '*' {
                in_block_comment = false;
            }
            prev_char = c;
            continue;
        }

        if !in_string {
            if c == '/' && prev_char == '/' {
                result.pop();
                in_line_comment = true;
                prev_char = c;
                continue;
            }
            if c == '*' && prev_char == '/' {
                result.pop();
                in_block_comment = true;
                prev_char = c;
                continue;
            }
        }

        if c == '"' && prev_char != '\\' {
            in_string = !in_string;
        }

        result.push(c);
        prev_char = c;
    }

    result
}

fn extract_nexus_id(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::Number(n) => {
            let num_str = n.to_string();
            if num_str.chars().all(|c| c.is_ascii_digit()) && !num_str.is_empty() {
                Some(num_str)
            } else {
                None
            }
        }
        serde_json::Value::String(s) => {
            let cleaned: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
            if !cleaned.is_empty() {
                Some(cleaned)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn get_mod_dict_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    path.pop();
    path.push("assets");
    
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    
    path.push(MOD_DICT_FILENAME);
    path
}

fn load_local_dict(path: &PathBuf) -> HashMap<String, String> {
    if !path.exists() {
        eprintln!("[update_mod_dict] Local dict not found, starting with empty dict");
        return HashMap::new();
    }

    match fs::read_to_string(path) {
        Ok(content) => {
            match serde_json::from_str::<HashMap<String, String>>(&content) {
                Ok(dict) => {
                    let validated = validate_dict(&dict);
                    eprintln!("[update_mod_dict] Loaded {} valid entries from local dict", validated.len());
                    validated
                }
                Err(e) => {
                    eprintln!("[update_mod_dict] Failed to parse local dict: {}", e);
                    HashMap::new()
                }
            }
        }
        Err(e) => {
            eprintln!("[update_mod_dict] Failed to read local dict: {}", e);
            HashMap::new()
        }
    }
}

fn validate_dict(dict: &HashMap<String, String>) -> HashMap<String, String> {
    dict.iter()
        .filter(|(_, v)| {
            !v.is_empty() && v.chars().all(|c| c.is_ascii_digit())
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

fn save_update_timestamp() {
    let timestamp = chrono::Utc::now().timestamp();
    let mut path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    path.pop();
    path.push("assets");
    path.push(UPDATE_TIMESTAMP_FILENAME);

    if let Err(e) = fs::write(&path, timestamp.to_string()) {
        eprintln!("[update_mod_dict] Failed to save update timestamp: {}", e);
    }
}

pub fn get_last_update_timestamp() -> Option<i64> {
    let mut path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    path.pop();
    path.push("assets");
    path.push(UPDATE_TIMESTAMP_FILENAME);

    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(timestamp) = content.trim().parse::<i64>() {
                return Some(timestamp);
            }
        }
    }
    None
}

pub fn should_auto_update() -> bool {
    match get_last_update_timestamp() {
        Some(timestamp) => {
            let now = chrono::Utc::now().timestamp();
            let seconds_in_7_days = AUTO_UPDATE_INTERVAL_DAYS * 24 * 60 * 60;
            (now - timestamp) > seconds_in_7_days
        }
        None => true,
    }
}

pub async fn auto_update_mod_dict() {
    if !should_auto_update() {
        eprintln!("[auto_update_mod_dict] Dictionary is up to date, skipping");
        return;
    }

    eprintln!("[auto_update_mod_dict] Auto-update triggered (7+ days since last update)");

    match update_mod_dict().await {
        Ok(result) => {
            eprintln!("[auto_update_mod_dict] Auto-update successful: {}", result.message);
        }
        Err(e) => {
            eprintln!("[auto_update_mod_dict] Auto-update failed: {}", e);
        }
    }
}
