use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModConfigSchema {
    pub mod_name: String,
    pub unique_id: String,
    pub config_path: String,
    pub fields: Vec<ConfigField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigField {
    pub key: String,
    pub value: ConfigValue,
    pub field_type: ConfigFieldType,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ConfigValue {
    Bool(bool),
    String(String),
    Number(f64),
    Array(Vec<ConfigValue>),
    Object(HashMap<String, ConfigValue>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigFieldType {
    Bool,
    String,
    Number,
    Array,
    Object,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigField {
    pub key: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModConfigListItem {
    pub mod_name: String,
    pub unique_id: String,
    pub folder_path: String,
    pub config_path: String,
    pub field_count: usize,
    pub has_config: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModConfigListResult {
    pub configs: Vec<ModConfigListItem>,
    pub total_mods_with_config: usize,
    pub total_mods_scanned: usize,
}

fn infer_field_type(value: &serde_json::Value) -> ConfigFieldType {
    match value {
        serde_json::Value::Bool(_) => ConfigFieldType::Bool,
        serde_json::Value::String(_) => ConfigFieldType::String,
        serde_json::Value::Number(_) => ConfigFieldType::Number,
        serde_json::Value::Array(_) => ConfigFieldType::Array,
        serde_json::Value::Object(_) => ConfigFieldType::Object,
        _ => ConfigFieldType::String,
    }
}

fn json_value_to_config_value(value: &serde_json::Value) -> ConfigValue {
    match value {
        serde_json::Value::Bool(b) => ConfigValue::Bool(*b),
        serde_json::Value::String(s) => ConfigValue::String(s.clone()),
        serde_json::Value::Number(n) => ConfigValue::Number(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::Array(arr) => ConfigValue::Array(arr.iter().map(json_value_to_config_value).collect()),
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k.clone(), json_value_to_config_value(v));
            }
            ConfigValue::Object(map)
        }
        _ => ConfigValue::String("".to_string()),
    }
}

fn config_value_to_json_value(value: &ConfigValue) -> serde_json::Value {
    match value {
        ConfigValue::Bool(b) => serde_json::Value::Bool(*b),
        ConfigValue::String(s) => serde_json::Value::String(s.clone()),
        ConfigValue::Number(n) => serde_json::Value::Number(serde_json::Number::from_f64(*n).unwrap_or(serde_json::Number::from(0))),
        ConfigValue::Array(arr) => serde_json::Value::Array(arr.iter().map(config_value_to_json_value).collect()),
        ConfigValue::Object(obj) => {
            let mut map = serde_json::Map::new();
            for (k, v) in obj {
                map.insert(k.clone(), config_value_to_json_value(v));
            }
            serde_json::Value::Object(map)
        }
    }
}

fn read_manifest_info(mod_path: &PathBuf) -> (String, String) {
    let manifest_path = mod_path.join("manifest.json");
    if manifest_path.exists() {
        if let Ok(content) = fs::read_to_string(&manifest_path) {
            if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&content) {
                let name = manifest["Name"].as_str().unwrap_or("Unknown").to_string();
                let unique_id = manifest["UniqueID"].as_str().unwrap_or("").to_string();
                return (name, unique_id);
            }
        }
    }
    (mod_path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string(), String::new())
}

#[tauri::command]
pub fn read_mod_config(mod_path: String) -> Result<ModConfigSchema, String> {
    let path = PathBuf::from(&mod_path);
    let (name, unique_id) = read_manifest_info(&path);

    let config_path = path.join("config.json");

    if !config_path.exists() {
        return Err(format!("No config.json found for {}", name));
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config.json: {}", e))?;

    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config.json: {}", e))?;

    let mut fields = Vec::new();

    if let serde_json::Value::Object(obj) = json {
        for (key, value) in obj {
            fields.push(ConfigField {
                key,
                value: json_value_to_config_value(&value),
                field_type: infer_field_type(&value),
                description: String::new(),
            });
        }
    }

    Ok(ModConfigSchema {
        mod_name: name,
        unique_id,
        config_path: config_path.to_string_lossy().to_string(),
        fields,
    })
}

#[tauri::command]
pub fn update_mod_config(
    mod_path: String,
    updates: Vec<UpdateConfigField>,
) -> Result<UpdateConfigResult, String> {
    let path = PathBuf::from(&mod_path);
    let config_path = path.join("config.json");

    if !config_path.exists() {
        return Err("config.json not found".to_string());
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config.json: {}", e))?;

    let mut json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config.json: {}", e))?;

    let update_count = updates.len();

    for update in updates {
        if let Some(obj) = json.as_object_mut() {
            obj.insert(update.key, update.value);
        }
    }

    let new_content = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, new_content)
        .map_err(|e| format!("Failed to write config.json: {}", e))?;

    Ok(UpdateConfigResult {
        success: true,
        message: format!("Updated {} fields in config.json", update_count),
    })
}

#[tauri::command]
pub fn list_mod_configs(mods_path: String) -> Result<ModConfigListResult, String> {
    let mods_dir = PathBuf::from(&mods_path);

    if !mods_dir.exists() || !mods_dir.is_dir() {
        return Err("Mods directory does not exist".to_string());
    }

    let mut configs = Vec::new();
    let mut total_scanned = 0usize;

    let entries = fs::read_dir(&mods_dir)
        .map_err(|e| format!("Failed to read Mods directory: {}", e))?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let mod_dir = entry.path();
        if !mod_dir.is_dir() {
            continue;
        }

        total_scanned += 1;

        if let Ok(sub_entries) = fs::read_dir(&mod_dir) {
            for sub_entry in sub_entries.flatten() {
                let sub_path = sub_entry.path();
                if sub_path.is_dir() {
                    let config_path = sub_path.join("config.json");
                    if config_path.exists() {
                        let (name, unique_id) = read_manifest_info(&sub_path);
                        let field_count = if let Ok(content) = fs::read_to_string(&config_path) {
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                                json.as_object().map(|o| o.len()).unwrap_or(0)
                            } else {
                                0
                            }
                        } else {
                            0
                        };

                        configs.push(ModConfigListItem {
                            mod_name: name,
                            unique_id,
                            folder_path: sub_path.to_string_lossy().to_string(),
                            config_path: config_path.to_string_lossy().to_string(),
                            field_count,
                            has_config: true,
                        });
                    }
                }
            }
        }

        let config_path = mod_dir.join("config.json");
        if config_path.exists() {
            let (name, unique_id) = read_manifest_info(&mod_dir);
            let field_count = if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    json.as_object().map(|o| o.len()).unwrap_or(0)
                } else {
                    0
                }
            } else {
                0
            };

            if !configs.iter().any(|c: &ModConfigListItem| c.folder_path == mod_dir.to_string_lossy()) {
                configs.push(ModConfigListItem {
                    mod_name: name,
                    unique_id,
                    folder_path: mod_dir.to_string_lossy().to_string(),
                    config_path: config_path.to_string_lossy().to_string(),
                    field_count,
                    has_config: true,
                });
            }
        }
    }

    configs.sort_by(|a, b| a.mod_name.to_lowercase().cmp(&b.mod_name.to_lowercase()));

    let total_with_config = configs.len();

    Ok(ModConfigListResult {
        configs,
        total_mods_with_config: total_with_config,
        total_mods_scanned: total_scanned,
    })
}
