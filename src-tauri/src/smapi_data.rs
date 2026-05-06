use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

const SMAPI_MODS_URL: &str = "https://raw.githubusercontent.com/Pathoschild/SmapiCompatibilityList/develop/data/mods.jsonc";
const CACHE_FILENAME: &str = "smapi_cache.json";

#[derive(Debug, Clone, Deserialize)]
struct SmapiModEntry {
    name: Option<String>,
    id: Option<String>,
    nexus: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
struct SmapiModsJson {
    mods: Vec<SmapiModEntry>,
}

static CACHE: Mutex<Option<HashMap<String, u64>>> = Mutex::new(None);
static NAME_CACHE: Mutex<Option<HashMap<String, u64>>> = Mutex::new(None);

fn get_cache_path() -> PathBuf {
    let app_data = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    app_data.join("StardewValley").join("SVL").join(CACHE_FILENAME)
}

fn load_cache_from_disk() -> Option<HashMap<String, u64>> {
    let path = get_cache_path();
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(&path).ok()?;
    let content = content.strip_prefix('\u{FEFF}').unwrap_or(&content);
    serde_json::from_str(content).ok()
}

fn save_cache_to_disk(cache: &HashMap<String, u64>) {
    if let Some(parent) = get_cache_path().parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(cache) {
        let _ = fs::write(get_cache_path(), json);
    }
}

fn strip_jsonc_comments(content: &str) -> String {
    let re = Regex::new(r"//.*|/\*[\s\S]*?\*/").unwrap();
    re.replace_all(content, "").to_string()
}

pub async fn init_smapi_cache() {
    eprintln!("[smapi_data] Initializing SMAPI data cache...");

    let client = match reqwest::Client::builder()
        .user_agent("SVL-Stardew-Valley-Launcher/1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[smapi_data] Failed to create HTTP client: {}", e);
            if let Some(cached) = load_cache_from_disk() {
                eprintln!("[smapi_data] Using cached data from disk");
                *CACHE.lock().unwrap() = Some(cached);
            }
            return;
        }
    };

    let response = match client.get(SMAPI_MODS_URL).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[smapi_data] Failed to download SMAPI mods list: {}", e);
            if let Some(cached) = load_cache_from_disk() {
                eprintln!("[smapi_data] Using cached data from disk");
                *CACHE.lock().unwrap() = Some(cached);
            }
            return;
        }
    };

    let body = match response.text().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("[smapi_data] Failed to read response body: {}", e);
            if let Some(cached) = load_cache_from_disk() {
                *CACHE.lock().unwrap() = Some(cached);
            }
            return;
        }
    };

    let cleaned = strip_jsonc_comments(&body);
    let cleaned = cleaned.strip_prefix('\u{FEFF}').unwrap_or(&cleaned);

    let mods_json: SmapiModsJson = match serde_json::from_str(cleaned) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[smapi_data] Failed to parse JSON: {}", e);
            if let Some(cached) = load_cache_from_disk() {
                *CACHE.lock().unwrap() = Some(cached);
            }
            return;
        }
    };

    let mut mods_map: HashMap<String, u64> = HashMap::new();
    let mut name_map: HashMap<String, u64> = HashMap::new();

    for mod_entry in mods_json.mods {
        if let Some(nexus_id) = mod_entry.nexus {
            if let Some(id) = mod_entry.id {
                mods_map.insert(id, nexus_id);
            }
            if let Some(name) = mod_entry.name {
                for alias in name.split(',').map(|s| s.trim()) {
                    if !alias.is_empty() {
                        name_map.insert(alias.to_string(), nexus_id);
                    }
                }
            }
        }
    }

    save_cache_to_disk(&mods_map);
    *CACHE.lock().unwrap() = Some(mods_map);
    *NAME_CACHE.lock().unwrap() = Some(name_map);

    eprintln!("[smapi_data] Cache initialized with {} mods, {} name aliases", 
        CACHE.lock().unwrap().as_ref().map(|c| c.len()).unwrap_or(0),
        NAME_CACHE.lock().unwrap().as_ref().map(|c| c.len()).unwrap_or(0));
}

pub fn get_mod_nexus_id(mod_unique_id: &str) -> Option<u64> {
    let cache = CACHE.lock().unwrap();
    if let Some(cached) = cache.as_ref() {
        if let Some(nexus_id) = cached.get(mod_unique_id) {
            return Some(*nexus_id);
        }
    }
    None
}

pub fn get_nexus_id_by_name(mod_name: &str) -> Option<u64> {
    let cache = NAME_CACHE.lock().unwrap();
    if let Some(cached) = cache.as_ref() {
        if let Some(nexus_id) = cached.get(mod_name) {
            return Some(*nexus_id);
        }
    }
    None
}

pub fn get_all_mod_ids() -> Vec<String> {
    let cache = CACHE.lock().unwrap();
    if let Some(cached) = cache.as_ref() {
        return cached.keys().cloned().collect();
    }
    Vec::new()
}
