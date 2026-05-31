use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use lazy_static::lazy_static;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModNameTranslation {
    pub unique_id: String,
    pub original_name: String,
    pub translated_name: String,
}

lazy_static! {
    static ref BUILTIN_DICT: HashMap<String, String> = {
        let json_str = include_str!("../dictionaries/builtin_dict.json");
        let map: HashMap<String, String> = serde_json::from_str(json_str).unwrap_or_default();
        map
    };

    static ref NAME_DICT: HashMap<String, String> = {
        let json_str = include_str!("../dictionaries/name_dict.json");
        let map: HashMap<String, String> = serde_json::from_str(json_str).unwrap_or_default();
        map
    };
}

fn get_builtin_dict() -> HashMap<String, String> {
    BUILTIN_DICT.clone()
}

fn get_name_dict() -> HashMap<String, String> {
    NAME_DICT.clone()
}

fn strip_bom(content: &str) -> &str {
    content.strip_prefix('\u{FEFF}').unwrap_or(content)
}

fn strip_trailing_commas(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_string = false;
    let mut escape_next = false;

    while i < len {
        let c = chars[i];

        if escape_next {
            result.push(c);
            escape_next = false;
            i += 1;
            continue;
        }

        if in_string {
            result.push(c);
            if c == '\\' {
                escape_next = true;
            } else if c == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }

        if c == '"' {
            in_string = true;
            result.push(c);
            i += 1;
            continue;
        }

        if c == ',' {
            let mut j = i + 1;
            while j < len && chars[j].is_whitespace() {
                j += 1;
            }
            if j < len && (chars[j] == '}' || chars[j] == ']') {
                i += 1;
                continue;
            }
        }

        result.push(c);
        i += 1;
    }

    result
}

fn write_manifest_name(folder_path: &str, new_name: &str) -> Result<(), String> {
    let path = Path::new(folder_path).join("manifest.json");
    if !path.exists() {
        return Err(format!("manifest.json not found in {}", folder_path));
    }

    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let cleaned = strip_trailing_commas(strip_bom(&content));
    let mut json: Value = serde_json::from_str(&cleaned).map_err(|e| e.to_string())?;

    if let Some(obj) = json.as_object_mut() {
        obj.insert("Name".to_string(), Value::String(new_name.to_string()));
    }

    let output = serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?;
    fs::write(&path, output).map_err(|e| e.to_string())
}

fn get_translation_file_path() -> PathBuf {
    let app_data = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    let svl_dir = app_data.join("SVL");
    fs::create_dir_all(&svl_dir).ok();
    svl_dir.join("mod_name_translations.json")
}

fn load_translations() -> HashMap<String, ModNameTranslation> {
    let path = get_translation_file_path();
    if !path.exists() {
        return HashMap::new();
    }
    let content = fs::read_to_string(&path).unwrap_or_default();
    let list: Vec<ModNameTranslation> = serde_json::from_str(&content).unwrap_or_default();
    list.into_iter().map(|t| (t.unique_id.clone(), t)).collect()
}

fn save_translations(translations: &HashMap<String, ModNameTranslation>) -> Result<(), String> {
    let path = get_translation_file_path();
    let list: Vec<&ModNameTranslation> = translations.values().collect();
    let json = serde_json::to_string_pretty(&list).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

pub fn lookup_name(unique_id: &str, original_name: &str) -> Option<String> {
    let uid_dict = get_builtin_dict();
    if let Some(name) = uid_dict.get(unique_id) {
        return Some(name.to_string());
    }

    let name_dict = get_name_dict();
    if let Some(name) = name_dict.get(original_name) {
        return Some(name.to_string());
    }

    let name_no_space: String = original_name.chars().filter(|c| !c.is_whitespace()).collect();
    if let Some(name) = name_dict.get(name_no_space.as_str()) {
        return Some(name.to_string());
    }

    let name_lower = original_name.to_lowercase();
    for (key, val) in &name_dict {
        if name_lower.contains(&key.to_lowercase()) {
            return Some(val.to_string());
        }
    }

    let name_no_space_lower = name_no_space.to_lowercase();
    for (key, val) in &name_dict {
        let key_no_space: String = key.chars().filter(|c| !c.is_whitespace()).collect();
        if name_no_space_lower.contains(&key_no_space.to_lowercase()) {
            return Some(val.to_string());
        }
    }

    None
}

pub fn extract_original_name(name: &str) -> String {
    let has_non_ascii = name.chars().any(|c| !c.is_ascii());
    if !has_non_ascii {
        return name.to_string();
    }
    if let Some(start) = name.rfind('(') {
        let after_open = &name[start + 1..];
        if let Some(end) = after_open.find(')') {
            let inner = &after_open[..end];
            if !inner.is_empty() && inner.chars().all(|c| c.is_ascii()) {
                return inner.to_string();
            }
        }
    }
    name.to_string()
}

fn is_already_translated(name: &str) -> bool {
    let has_non_ascii = name.chars().any(|c| !c.is_ascii());
    if !has_non_ascii {
        return false;
    }
    if let Some(start) = name.rfind('(') {
        let after_open = &name[start + 1..];
        if let Some(end) = after_open.find(')') {
            let inner = &after_open[..end];
            if !inner.is_empty() && inner.chars().all(|c| c.is_ascii()) {
                return true;
            }
        }
    }
    false
}

fn is_nested_translation(name: &str) -> bool {
    let count = name.matches('(').count();
    count > 1 && is_already_translated(name)
}

#[tauri::command]
pub fn get_mod_name_translations() -> Result<Vec<ModNameTranslation>, String> {
    let translations = load_translations();
    Ok(translations.into_values().collect())
}

#[tauri::command]
pub fn translate_mod_name(
    unique_id: String,
    original_name: String,
    folder_path: String,
) -> Result<ModNameTranslation, String> {
    if is_already_translated(&original_name) && !is_nested_translation(&original_name) {
        return Err("Already translated".into());
    }

    let real_original = extract_original_name(&original_name);
    let translated = match lookup_name(&unique_id, &real_original) {
        Some(t) => t,
        None => return Err(format!("No translation found for {}", real_original)),
    };

    let display_name = format!("{} ({})", translated, real_original);

    let translation = ModNameTranslation {
        unique_id,
        original_name: real_original,
        translated_name: translated,
    };

    if let Err(e) = write_manifest_name(&folder_path, &display_name) {
        eprintln!("Warning: failed to write manifest.json: {}", e);
    }

    let mut translations = load_translations();
    translations.insert(translation.unique_id.clone(), translation.clone());
    save_translations(&translations)?;

    Ok(translation)
}

#[tauri::command]
pub fn batch_translate_mod_names(
    mods: Vec<(String, String, String)>,
) -> Result<Vec<ModNameTranslation>, String> {
    let mut translations = load_translations();

    for (unique_id, original_name, folder_path) in &mods {
        if is_already_translated(original_name) && !is_nested_translation(original_name) {
            continue;
        }

        let real_original = extract_original_name(original_name);
        if let Some(translated) = lookup_name(unique_id, &real_original) {
            let display_name = format!("{} ({})", translated, real_original);

            let translation = ModNameTranslation {
                unique_id: unique_id.clone(),
                original_name: real_original,
                translated_name: translated,
            };

            if let Err(e) = write_manifest_name(folder_path, &display_name) {
                eprintln!("Warning: failed to write manifest.json for {}: {}", unique_id, e);
            }

            translations.insert(unique_id.clone(), translation);
        }
    }

    save_translations(&translations)?;
    Ok(translations.into_values().collect())
}

#[tauri::command]
pub fn delete_mod_name_translation(unique_id: String, folder_path: String) -> Result<bool, String> {
    let mut translations = load_translations();
    let original_name = if let Some(translation) = translations.remove(&unique_id) {
        Some(translation.original_name)
    } else {
        let manifest_path = Path::new(&folder_path).join("manifest.json");
        if manifest_path.exists() {
            if let Ok(content) = fs::read_to_string(&manifest_path) {
                let cleaned = strip_trailing_commas(strip_bom(&content));
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&cleaned) {
                    if let Some(name) = json["Name"].as_str() {
                        let extracted = extract_original_name(name);
                        if extracted != name {
                            Some(extracted)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    };

    if let Some(ref name) = original_name {
        if let Err(e) = write_manifest_name(&folder_path, name) {
            eprintln!("Warning: failed to restore manifest.json: {}", e);
        }
    }

    save_translations(&translations)?;
    Ok(true)
}

#[tauri::command]
pub fn clear_all_mod_name_translations(mods: Vec<(String, String)>) -> Result<bool, String> {
    let translations = load_translations();

    for (unique_id, folder_path) in &mods {
        let original_name = if let Some(translation) = translations.get(unique_id) {
            Some(translation.original_name.clone())
        } else {
            let manifest_path = Path::new(folder_path).join("manifest.json");
            if manifest_path.exists() {
                if let Ok(content) = fs::read_to_string(&manifest_path) {
                    let cleaned = strip_trailing_commas(strip_bom(&content));
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&cleaned) {
                        if let Some(name) = json["Name"].as_str() {
                            let extracted = extract_original_name(name);
                            if extracted != name {
                                Some(extracted)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(ref name) = original_name {
            if let Err(e) = write_manifest_name(folder_path, name) {
                eprintln!("Warning: failed to restore manifest.json for {}: {}", unique_id, e);
            }
        }
    }

    let path = get_translation_file_path();
    if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(true)
}
