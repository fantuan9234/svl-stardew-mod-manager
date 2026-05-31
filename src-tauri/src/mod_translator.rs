use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Emitter;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModTranslationStatus {
    pub mod_name: String,
    pub mod_path: String,
    pub status: String,
    pub total_entries: u32,
    pub translated_entries: u32,
    pub remaining_entries: u32,
    pub has_i18n: bool,
    pub has_target_lang: bool,
    pub default_file: Option<String>,
    pub target_file: Option<String>,
    pub file_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranslateFileResult {
    pub success: bool,
    pub file_path: String,
    pub message: String,
    pub backup_path: Option<String>,
}

#[tauri::command]
pub async fn scan_translatable_mods(
    game_path: String,
    target_lang: String,
) -> Result<Vec<ModTranslationStatus>, String> {
    let mods_dir = PathBuf::from(&game_path).join("Mods");
    if !mods_dir.exists() {
        return Err("Mods directory not found".into());
    }

    let lang_code = lang_name_to_code(&target_lang);
    let mut results: Vec<ModTranslationStatus> = Vec::new();

    let entries = fs::read_dir(&mods_dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let mod_dir = entry.path();
        if !mod_dir.is_dir() {
            continue;
        }

        let mod_name = mod_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if mod_name.starts_with('.') {
            continue;
        }

        if mod_dir.join("manifest.json").exists() {
            if is_chinese_translation_mod(&mod_dir) {
                continue;
            }
            let is_cp = is_content_patcher_pack(&mod_dir);
            let status = analyze_mod_translation_status(&mod_dir, &mod_name, lang_code, is_cp)?;
            if status.status != "no_i18n" || status.file_type != "none" {
                results.push(status);
            }
        } else {
            let sub_entries = fs::read_dir(&mod_dir).map_err(|e| e.to_string())?;
            for sub_entry in sub_entries {
                let sub_entry = sub_entry.map_err(|e| e.to_string())?;
                let sub_dir = sub_entry.path();
                if !sub_dir.is_dir() {
                    continue;
                }
                if sub_dir.join("manifest.json").exists() {
                    if is_chinese_translation_mod(&sub_dir) {
                        continue;
                    }
                    let is_cp = is_content_patcher_pack(&sub_dir);
                    let sub_name = sub_dir
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let status = analyze_mod_translation_status(&sub_dir, &sub_name, lang_code, is_cp)?;
                    if status.status != "no_i18n" || status.file_type != "none" {
                        results.push(status);
                    }
                }
            }
        }
    }

    results.sort_by(|a, b| a.mod_name.to_lowercase().cmp(&b.mod_name.to_lowercase()));
    Ok(results)
}

fn is_content_patcher_pack(mod_dir: &Path) -> bool {
    let manifest_path = mod_dir.join("manifest.json");
    if let Some(json) = read_json_file(&manifest_path) {
        if json.get("ContentPackFor").is_some() {
            return true;
        }
    }
    false
}

fn is_chinese_translation_mod(mod_dir: &Path) -> bool {
    let manifest_path = mod_dir.join("manifest.json");
    if let Some(json) = read_json_file(&manifest_path) {
        let name = json.get("Name").and_then(|v| v.as_str()).unwrap_or("");
        let uid = json.get("UniqueID").and_then(|v| v.as_str()).unwrap_or("");
        let name_lower = name.to_lowercase();
        let uid_lower = uid.to_lowercase();
        if name_lower.contains("chinese") || uid_lower.contains("chinese")
            || name_lower.contains("中文") || uid_lower.contains("中文")
            || name_lower.contains("汉化") || uid_lower.contains("汉化")
            || name_lower.contains("翻译") || uid_lower.contains("翻译")
        {
            return true;
        }
    }
    false
}

fn lang_name_to_code(lang: &str) -> &str {
    match lang {
        "简体中文" => "zh",
        "繁體中文" => "zh-tw",
        "日本語" => "ja",
        "한국어" => "ko",
        _ => "zh",
    }
}

fn strip_bom(content: &str) -> &str {
    content.strip_prefix('\u{FEFF}').unwrap_or(content)
}

fn strip_json_comments(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut in_string = false;
    let mut escape_next = false;
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut i = 0;

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

        if c == '/' && i + 1 < len {
            if chars[i + 1] == '/' {
                while i < len && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            }
            if chars[i + 1] == '*' {
                i += 2;
                while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                i += 2;
                continue;
            }
        }

        result.push(c);
        i += 1;
    }

    result
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
            while j < len && (chars[j] == ' ' || chars[j] == '\t' || chars[j] == '\n' || chars[j] == '\r') {
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

fn read_json_file(path: &Path) -> Option<Value> {
    let content = fs::read_to_string(path).ok()?;
    let cleaned = strip_bom(&content);
    let no_comments = strip_json_comments(cleaned);
    let no_trailing = strip_trailing_commas(&no_comments);
    serde_json::from_str(&no_trailing).ok()
}

fn count_i18n_entries(path: &Path) -> u32 {
    match read_json_file(path) {
        Some(Value::Object(map)) => map.len() as u32,
        _ => 0,
    }
}

fn analyze_mod_translation_status(
    mod_dir: &Path,
    mod_name: &str,
    lang_code: &str,
    is_content_patcher: bool,
) -> Result<ModTranslationStatus, String> {
    let i18n_dir = mod_dir.join("i18n");

    if i18n_dir.exists() && i18n_dir.is_dir() {
        return analyze_i18n_folder(mod_dir, mod_name, lang_code, &i18n_dir);
    }

    if is_content_patcher {
        return analyze_content_patcher_mod(mod_dir, mod_name);
    }

    Ok(make_no_i18n_status(mod_name, mod_dir))
}

fn analyze_i18n_folder(
    mod_dir: &Path,
    mod_name: &str,
    lang_code: &str,
    i18n_dir: &Path,
) -> Result<ModTranslationStatus, String> {
    let default_path = i18n_dir.join("default.json");
    let target_path = i18n_dir.join(format!("{}.json", lang_code));

    let default_json = read_json_file(&default_path);
    let target_json = read_json_file(&target_path);

    if let (Some(Value::Object(def_map)), Some(Value::Object(tgt_map))) = (&default_json, &target_json) {
        let default_total = def_map.len() as u32;
        if default_total == 0 {
            return Ok(make_no_i18n_status(mod_name, mod_dir));
        }

        let mut translated = 0u32;
        for (key, default_val) in def_map {
            if let Some(target_val) = tgt_map.get(key) {
                let dv = default_val.as_str().unwrap_or("").trim();
                let tv = target_val.as_str().unwrap_or("").trim();
                if !tv.is_empty() && tv != dv {
                    translated += 1;
                }
            }
        }

        let remaining = default_total.saturating_sub(translated);
        let status = if remaining == 0 {
            "completed".to_string()
        } else if translated > 0 {
            "partial".to_string()
        } else {
            "untranslated".to_string()
        };

        return Ok(ModTranslationStatus {
            mod_name: mod_name.to_string(),
            mod_path: mod_dir.to_string_lossy().to_string(),
            status,
            total_entries: default_total,
            translated_entries: translated,
            remaining_entries: remaining,
            has_i18n: true,
            has_target_lang: translated > 0,
            default_file: Some(default_path.to_string_lossy().to_string()),
            target_file: Some(target_path.to_string_lossy().to_string()),
            file_type: "i18n".to_string(),
        });
    }

    if let Some(Value::Object(def_map)) = &default_json {
        let default_total = def_map.len() as u32;
        if default_total > 0 {
            return Ok(ModTranslationStatus {
                mod_name: mod_name.to_string(),
                mod_path: mod_dir.to_string_lossy().to_string(),
                status: "untranslated".to_string(),
                total_entries: default_total,
                translated_entries: 0,
                remaining_entries: default_total,
                has_i18n: true,
                has_target_lang: false,
                default_file: Some(default_path.to_string_lossy().to_string()),
                target_file: None,
                file_type: "i18n".to_string(),
            });
        }
    }

    if let Some(Value::Object(tgt_map)) = &target_json {
        let tgt_total = tgt_map.len() as u32;
        if tgt_total > 0 {
            return Ok(ModTranslationStatus {
                mod_name: mod_name.to_string(),
                mod_path: mod_dir.to_string_lossy().to_string(),
                status: "completed".to_string(),
                total_entries: tgt_total,
                translated_entries: tgt_total,
                remaining_entries: 0,
                has_i18n: true,
                has_target_lang: true,
                default_file: None,
                target_file: Some(target_path.to_string_lossy().to_string()),
                file_type: "i18n".to_string(),
            });
        }
    }

    let entries = fs::read_dir(i18n_dir).map_err(|e| e.to_string())?;
    let mut source_file: Option<PathBuf> = None;
    let mut found_target: Option<PathBuf> = None;

    for entry in entries {
        if let Ok(e) = entry {
            let p = e.path();
            if p.is_file() && p.extension().map_or(false, |e| e == "json") {
                let name = p.file_stem().unwrap_or_default().to_string_lossy().to_lowercase();
                if name == lang_code {
                    found_target = Some(p);
                } else if source_file.is_none() {
                    source_file = Some(p);
                }
            }
        }
    }

    match (source_file, found_target) {
        (Some(ref src), Some(ref tgt)) => {
            let src_json = read_json_file(src);
            let tgt_json = read_json_file(tgt);
            if let (Some(Value::Object(def_map)), Some(Value::Object(tgt_map))) = (&src_json, &tgt_json) {
                let src_total = def_map.len() as u32;
                if src_total > 0 {
                    let mut translated = 0u32;
                    for (key, default_val) in def_map {
                        if let Some(target_val) = tgt_map.get(key) {
                            let dv = default_val.as_str().unwrap_or("").trim();
                            let tv = target_val.as_str().unwrap_or("").trim();
                            if !tv.is_empty() && tv != dv {
                                translated += 1;
                            }
                        }
                    }
                    let remaining = src_total.saturating_sub(translated);
                    let status = if remaining == 0 {
                        "completed".to_string()
                    } else if translated > 0 {
                        "partial".to_string()
                    } else {
                        "untranslated".to_string()
                    };

                    return Ok(ModTranslationStatus {
                        mod_name: mod_name.to_string(),
                        mod_path: mod_dir.to_string_lossy().to_string(),
                        status,
                        total_entries: src_total,
                        translated_entries: translated,
                        remaining_entries: remaining,
                        has_i18n: true,
                        has_target_lang: translated > 0,
                        default_file: Some(src.to_string_lossy().to_string()),
                        target_file: Some(tgt.to_string_lossy().to_string()),
                        file_type: "i18n".to_string(),
                    });
                }
            }
        }
        (Some(ref src), None) => {
            let src_total = count_i18n_entries(src);
            if src_total > 0 {
                return Ok(ModTranslationStatus {
                    mod_name: mod_name.to_string(),
                    mod_path: mod_dir.to_string_lossy().to_string(),
                    status: "untranslated".to_string(),
                    total_entries: src_total,
                    translated_entries: 0,
                    remaining_entries: src_total,
                    has_i18n: true,
                    has_target_lang: false,
                    default_file: Some(src.to_string_lossy().to_string()),
                    target_file: None,
                    file_type: "i18n".to_string(),
                });
            }
        }
        (None, Some(ref tgt)) => {
            let tgt_total = count_i18n_entries(tgt);
            if tgt_total > 0 {
                return Ok(ModTranslationStatus {
                    mod_name: mod_name.to_string(),
                    mod_path: mod_dir.to_string_lossy().to_string(),
                    status: "completed".to_string(),
                    total_entries: tgt_total,
                    translated_entries: tgt_total,
                    remaining_entries: 0,
                    has_i18n: true,
                    has_target_lang: true,
                    default_file: None,
                    target_file: Some(tgt.to_string_lossy().to_string()),
                    file_type: "i18n".to_string(),
                });
            }
        }
        (None, None) => {}
    }

    Ok(make_no_i18n_status(mod_name, mod_dir))
}

fn make_no_i18n_status(mod_name: &str, mod_dir: &Path) -> ModTranslationStatus {
    ModTranslationStatus {
        mod_name: mod_name.to_string(),
        mod_path: mod_dir.to_string_lossy().to_string(),
        status: "no_i18n".to_string(),
        total_entries: 0,
        translated_entries: 0,
        remaining_entries: 0,
        has_i18n: false,
        has_target_lang: false,
        default_file: None,
        target_file: None,
        file_type: "none".to_string(),
    }
}

fn analyze_content_patcher_mod(
    mod_dir: &Path,
    mod_name: &str,
) -> Result<ModTranslationStatus, String> {
    let content_path = mod_dir.join("content.json");
    if !content_path.exists() {
        return Ok(make_no_i18n_status(mod_name, mod_dir));
    }

    let total = count_all_changes_in_dir(mod_dir);
    if total == 0 {
        return Ok(make_no_i18n_status(mod_name, mod_dir));
    }

    Ok(ModTranslationStatus {
        mod_name: mod_name.to_string(),
        mod_path: mod_dir.to_string_lossy().to_string(),
        status: "untranslated".to_string(),
        total_entries: total,
        translated_entries: 0,
        remaining_entries: total,
        has_i18n: true,
        has_target_lang: false,
        default_file: Some(content_path.to_string_lossy().to_string()),
        target_file: None,
        file_type: "content".to_string(),
    })
}

fn count_all_changes_in_dir(mod_dir: &Path) -> u32 {
    let mut total: u32 = 0;
    if let Ok(entries) = fs::read_dir(mod_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += count_changes_in_subdir(&p);
            } else if p.extension().map_or(false, |e| e == "json") {
                let name = p.file_name().unwrap_or_default().to_string_lossy();
                if name != "manifest.json" && name != "config.json" {
                    total += count_changes_in_file(&p);
                }
            }
        }
    }
    total
}

fn count_changes_in_subdir(dir: &Path) -> u32 {
    let mut total: u32 = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += count_changes_in_subdir(&p);
            } else if p.extension().map_or(false, |e| e == "json") {
                let name = p.file_name().unwrap_or_default().to_string_lossy();
                if name != "manifest.json" && name != "config.json" {
                    total += count_changes_in_file(&p);
                }
            }
        }
    }
    total
}

fn count_changes_in_file(path: &Path) -> u32 {
    if let Some(json) = read_json_file(path) {
        if let Some(changes) = json.get("Changes").and_then(|v| v.as_array()) {
            return changes.len() as u32;
        }
    }
    0
}

#[tauri::command]
pub async fn translate_mod_file(
    app: tauri::AppHandle,
    file_path: String,
    file_type: String,
    ai_config: AiConfig,
    target_lang: String,
) -> Result<TranslateFileResult, String> {
    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err("File not found".into());
    }

    let original_content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let cleaned_content = strip_bom(&original_content);
    let no_comments_content = strip_json_comments(cleaned_content);
    let no_trailing_content = strip_trailing_commas(&no_comments_content);

    let parsed: Value = serde_json::from_str(&no_trailing_content)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let is_i18n_flat = file_type == "i18n" && parsed.is_object();
    let is_content_changes = file_type == "content" && parsed.get("Changes").and_then(|v| v.as_array()).is_some();

    if is_i18n_flat {
        return translate_i18n_chunked(&app, &path, &parsed, &ai_config, &target_lang).await;
    }

    if is_content_changes {
        return translate_content_chunked(&app, &path, &parsed, &ai_config, &target_lang).await;
    }

    let content_str = serde_json::to_string(&parsed).map_err(|e| e.to_string())?;
    if content_str.len() > 200 * 1024 {
        return Err(format!(
            "File too large ({}KB), maximum 200KB for this file type",
            content_str.len() / 1024
        ));
    }

    let translated =
        call_ai_translate(&content_str, &file_type, &ai_config, &target_lang).await?;

    let translated_json: Value =
        serde_json::from_str(&translated).map_err(|e| format!("AI returned invalid JSON: {}. Raw response (first 200 chars): {}", e, &translated[..translated.len().min(200)]))?;

    let backup_path = format!("{}.svlbak", file_path);
    fs::copy(&path, &backup_path).map_err(|e| format!("Backup failed: {}", e))?;

    let new_file = if file_type == "content" {
        path.clone()
    } else {
        let lang_code = lang_name_to_code(&target_lang);
        let parent = path.parent().ok_or("Cannot determine parent directory")?;
        parent.join(format!("{}.json", lang_code))
    };
    let formatted =
        serde_json::to_string_pretty(&translated_json).map_err(|e| e.to_string())?;
    fs::write(&new_file, formatted).map_err(|e| format!("Write failed: {}", e))?;

    Ok(TranslateFileResult {
        success: true,
        file_path: file_path.clone(),
        message: "Translation successful".into(),
        backup_path: Some(backup_path),
    })
}

async fn translate_i18n_chunked(
    app: &tauri::AppHandle,
    path: &Path,
    parsed: &Value,
    ai_config: &AiConfig,
    target_lang: &str,
) -> Result<TranslateFileResult, String> {
    let map = parsed.as_object().ok_or("Expected JSON object")?;
    let keys: Vec<String> = map.keys().cloned().collect();
    let total = keys.len();

    if total == 0 {
        return Err("No entries to translate".into());
    }

    let chunk_size = 150usize;
    let mut merged = serde_json::Map::new();
    let total_chunks = (total + chunk_size - 1) / chunk_size;

    for (chunk_idx, chunk_start) in (0..total).step_by(chunk_size).enumerate() {
        let chunk_end = (chunk_start + chunk_size).min(total);
        let chunk_keys: Vec<String> = keys[chunk_start..chunk_end].to_vec();

        let _ = app.emit("translate-progress", serde_json::json!({
            "phase": "i18n",
            "chunk_current": chunk_idx + 1,
            "chunk_total": total_chunks,
            "entry_current": chunk_end,
            "entry_total": total,
            "current_keys": chunk_keys.iter().take(5).cloned().collect::<Vec<_>>(),
            "first_key": chunk_keys.first().unwrap_or(&String::new()),
        }));

        let mut chunk_obj = serde_json::Map::new();
        for key in &chunk_keys {
            if let Some(val) = map.get(key) {
                chunk_obj.insert(key.clone(), val.clone());
            }
        }

        let chunk_json = serde_json::to_string(&Value::Object(chunk_obj))
            .map_err(|e| e.to_string())?;

        let chunk_file_type = "i18n";
        let translated = call_ai_translate(&chunk_json, chunk_file_type, ai_config, target_lang)
            .await
            .map_err(|e| format!("Chunk {}/{} failed: {}", chunk_idx + 1, total_chunks, e))?;

        let translated_json: Value = serde_json::from_str(&translated)
            .map_err(|e| format!(
                "Chunk {}/{}: AI returned invalid JSON: {}",
                chunk_idx + 1, total_chunks, e
            ))?;

        if let Some(obj) = translated_json.as_object() {
            for (k, v) in obj {
                merged.insert(k.clone(), v.clone());
            }
        }
    }

    let backup_path = format!("{}.svlbak", path.to_string_lossy());
    let lang_code = lang_name_to_code(target_lang);
    let parent = path.parent().ok_or("Cannot determine parent directory")?;
    let target_file = parent.join(format!("{}.json", lang_code));

    if target_file.exists() {
        fs::copy(&target_file, &backup_path).map_err(|e| format!("Backup failed: {}", e))?;
        if let Some(existing) = read_json_file(&target_file) {
            if let Some(existing_map) = existing.as_object() {
                for (k, v) in existing_map {
                    if !merged.contains_key(k) {
                        merged.insert(k.clone(), v.clone());
                    }
                }
            }
        }
    } else {
        fs::copy(path, &backup_path).map_err(|e| format!("Backup failed: {}", e))?;
    }

    let formatted = serde_json::to_string_pretty(&Value::Object(merged))
        .map_err(|e| e.to_string())?;
    fs::write(&target_file, formatted).map_err(|e| format!("Write failed: {}", e))?;

    Ok(TranslateFileResult {
        success: true,
        file_path: path.to_string_lossy().to_string(),
        message: format!("Translated {} entries in {} chunks", total, total_chunks),
        backup_path: Some(backup_path),
    })
}

async fn translate_content_chunked(
    app: &tauri::AppHandle,
    path: &Path,
    parsed: &Value,
    ai_config: &AiConfig,
    target_lang: &str,
) -> Result<TranslateFileResult, String> {
    let changes = parsed.get("Changes").and_then(|v| v.as_array())
        .ok_or("No Changes array found")?;
    let total = changes.len();

    if total == 0 {
        return Err("No Changes entries to translate".into());
    }

    let chunk_size = 20usize;
    let mut all_translated_changes = Vec::new();
    let total_chunks = (total + chunk_size - 1) / chunk_size;

    for (chunk_idx, chunk_start) in (0..total).step_by(chunk_size).enumerate() {
        let chunk_end = (chunk_start + chunk_size).min(total);

        let _ = app.emit("translate-progress", serde_json::json!({
            "phase": "content",
            "chunk_current": chunk_idx + 1,
            "chunk_total": total_chunks,
            "entry_current": chunk_end,
            "entry_total": total,
            "first_key": format!("Change #{}", chunk_start + 1),
        }));

        let chunk_changes = &changes[chunk_start..chunk_end];

        let mut chunk_obj = serde_json::Map::new();
        if let Some(format) = parsed.get("Format") {
            chunk_obj.insert("Format".to_string(), format.clone());
        }
        chunk_obj.insert("Changes".to_string(), Value::Array(chunk_changes.to_vec()));

        let chunk_json = serde_json::to_string(&Value::Object(chunk_obj))
            .map_err(|e| e.to_string())?;

        let translated = call_ai_translate(&chunk_json, "content", ai_config, target_lang)
            .await
            .map_err(|e| format!("Chunk {}/{} failed: {}", chunk_idx + 1, total_chunks, e))?;

        let translated_json: Value = serde_json::from_str(&translated)
            .map_err(|e| format!(
                "Chunk {}/{}: AI returned invalid JSON: {}",
                chunk_idx + 1, total_chunks, e
            ))?;

        if let Some(arr) = translated_json.get("Changes").and_then(|v| v.as_array()) {
            all_translated_changes.extend(arr.iter().cloned());
        }
    }

    let backup_path = format!("{}.svlbak", path.to_string_lossy());
    fs::copy(path, &backup_path).map_err(|e| format!("Backup failed: {}", e))?;

    let mut result_obj = serde_json::Map::new();
    if let Some(format) = parsed.get("Format") {
        result_obj.insert("Format".to_string(), format.clone());
    }
    result_obj.insert("Changes".to_string(), Value::Array(all_translated_changes));

    let formatted = serde_json::to_string_pretty(&Value::Object(result_obj))
        .map_err(|e| e.to_string())?;
    fs::write(path, formatted).map_err(|e| format!("Write failed: {}", e))?;

    Ok(TranslateFileResult {
        success: true,
        file_path: path.to_string_lossy().to_string(),
        message: format!("Translated {} changes in {} chunks", total, total_chunks),
        backup_path: Some(backup_path),
    })
}

#[tauri::command]
pub async fn test_ai_connection(ai_config: AiConfig) -> Result<String, String> {
    let url = format!(
        "{}/chat/completions",
        ai_config.base_url.trim_end_matches('/')
    );

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", ai_config.api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": ai_config.model,
            "messages": [
                {"role": "user", "content": "Say hello in Chinese, reply with only the translation."}
            ],
            "max_tokens": 50,
            "temperature": 0.1
        }))
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error ({}): {}", status, &body[..body.len().min(300)]));
    }

    let response_json: Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let reply = response_json
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("Connection successful")
        .to_string();

    Ok(reply)
}

#[tauri::command]
pub async fn restore_translation_backup(file_path: String) -> Result<bool, String> {
    let backup_path = format!("{}.svlbak", file_path);
    let backup = PathBuf::from(&backup_path);
    if !backup.exists() {
        return Err("Backup file not found".into());
    }
    fs::copy(&backup, &file_path).map_err(|e| format!("Restore failed: {}", e))?;
    let _ = fs::remove_file(&backup);
    Ok(true)
}

async fn call_ai_translate(
    content: &str,
    file_type: &str,
    ai_config: &AiConfig,
    target_lang: &str,
) -> Result<String, String> {
    let lang_code = lang_name_to_code(target_lang);
    let i18n_instruction = if file_type == "i18n" {
        format!(
            "\n13. This is an i18n translation file for a Stardew Valley mod. The file uses language keys as JSON keys. Translate all string VALUES to {}. Keep all JSON keys exactly as they are. The result will be saved as {}.json in the i18n folder.",
            target_lang, lang_code
        )
    } else {
        String::new()
    };

    let system_prompt = format!(
        "You are a professional game mod translator for Stardew Valley. Translate all English text values in the JSON to {}. \
Rules:\
1. Keep all JSON keys exactly unchanged.\
2. Only translate string values that contain English text.\
3. Keep numbers, booleans, and null values unchanged.\
4. Preserve the exact JSON structure and nesting.\
5. Do not add or remove any fields.\
6. Return ONLY the translated JSON object, no explanations, no markdown code fences.\
7. Keep special formatting tokens like {{}}, <<>>, [ ] unchanged as they are placeholders.\
8. Keep file paths and URLs unchanged.\
9. Keep mod unique IDs and SMAPI-related identifiers unchanged.\
10. If a value is already in {}, keep it unchanged.\
11. Translate game terms naturally: e.g. 'Stardew Valley' can stay English, common mod terms like 'Content Patcher', 'SMAPI' should stay English.\
12. For config files, translate option descriptions and display names but keep technical values unchanged.{}",
        target_lang, target_lang, i18n_instruction
    );

    let file_type_desc = match file_type {
        "config" => "mod configuration file (translate display text and descriptions, keep technical values)",
        "manifest" => "mod manifest file (only translate the Description field, keep everything else)",
        "content" => "mod content file (translate all user-facing text like dialogue, item names, descriptions)",
        "i18n" => "mod i18n translation file (translate all string values, keep all keys unchanged)",
        _ => "mod file",
    };

    let user_prompt = format!(
        "Translate the following {} JSON to {}:\n\n{}",
        file_type_desc, target_lang, content
    );

    let url = format!(
        "{}/chat/completions",
        ai_config.base_url.trim_end_matches('/')
    );

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", ai_config.api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": ai_config.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            "temperature": 0.3
        }))
        .timeout(std::time::Duration::from_secs(86400))
        .send()
        .await
        .map_err(|e| format!("AI API request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!(
            "AI API error ({}): {}",
            status,
            &body[..body.len().min(300)]
        ));
    }

    let response_json: Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse AI response: {}", e))?;

    let translated = response_json
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .ok_or("Invalid AI response format")?;

    let trimmed = translated.trim();
    let cleaned = if trimmed.starts_with("```json") {
        trimmed
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim()
    } else if trimmed.starts_with("```") {
        trimmed
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
    } else {
        trimmed
    };

    Ok(cleaned.to_string())
}
