use regex::Regex;
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TranslateFileResult {
    pub success: bool,
    pub file_path: String,
    pub message: String,
    pub backup_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UntranslatedEntry {
    pub key: String,
    pub source: String,
    pub current: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModTranslationDetail {
    pub mod_name: String,
    pub mod_path: String,
    pub default_file: Option<String>,
    pub target_file: Option<String>,
    pub entries: Vec<UntranslatedEntry>,
    pub total_entries: u32,
    pub untranslated_count: u32,
}

#[tauri::command]
pub async fn get_mod_untranslated_entries(
    mod_path: String,
    target_lang: String,
) -> Result<ModTranslationDetail, String> {
    let mod_dir = PathBuf::from(&mod_path);
    if !mod_dir.exists() {
        return Err(format!("Mod directory not found: {}", mod_path));
    }

    let mod_name = mod_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| mod_path.clone());

    let lang_code = lang_name_to_code(&target_lang);
    let i18n_dir = mod_dir.join("i18n");
    let default_path = i18n_dir.join("default.json");
    let target_path = i18n_dir.join(format!("{}.json", lang_code));

    if !default_path.exists() {
        return Err(format!("No default.json found at {}", default_path.display()));
    }

    let default_json = read_json_file(&default_path)
        .ok_or_else(|| format!("Failed to read {}", default_path.display()))?;
    let target_json = read_json_file(&target_path);

    let def_map = default_json.as_object()
        .ok_or_else(|| "default.json is not a JSON object".to_string())?;
    let tgt_map = target_json.as_ref().and_then(|v| v.as_object());

    let mut entries: Vec<UntranslatedEntry> = Vec::new();
    let total = def_map.len() as u32;

    for (key, default_val) in def_map {
        let source = default_val.as_str().unwrap_or("").to_string();
        if is_skippable_entry(&source) {
            continue;
        }

        let current = match tgt_map.and_then(|m| m.get(key)) {
            Some(v) => v.as_str().unwrap_or("").to_string(),
            None => String::new(),
        };
        let trimmed_current = current.trim();
        let trimmed_source = source.trim();

        let status = if trimmed_current.is_empty() {
            "untranslated"
        } else if trimmed_current == trimmed_source {
            "same_as_source"
        } else {
            "translated"
        };

        if status == "untranslated" || status == "same_as_source" {
            entries.push(UntranslatedEntry {
                key: key.clone(),
                source,
                current,
                status: status.to_string(),
            });
        }
    }

    Ok(ModTranslationDetail {
        mod_name,
        mod_path,
        default_file: Some(default_path.to_string_lossy().to_string()),
        target_file: target_json.as_ref().map(|_| target_path.to_string_lossy().to_string()),
        total_entries: total,
        untranslated_count: entries.len() as u32,
        entries,
    })
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

fn is_symbol_only(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return true;
    }
    if looks_like_animal_sound(trimmed) {
        return true;
    }
    if looks_like_display_string(trimmed) {
        return true;
    }
    if is_format_template(trimmed) {
        return true;
    }
    if is_pure_mod_variable(trimmed) {
        return true;
    }
    if looks_like_fictional_language(trimmed) {
        return true;
    }
    let mut consecutive_alpha = 0usize;
    for c in trimmed.chars() {
        if c.is_alphabetic() {
            consecutive_alpha += 1;
            if consecutive_alpha >= 2 {
                return false;
            }
        } else {
            consecutive_alpha = 0;
        }
        let cp = c as u32;
        if (0x4E00..=0x9FFF).contains(&cp)
            || (0x3040..=0x309F).contains(&cp)
            || (0x30A0..=0x30FF).contains(&cp)
            || (0xAC00..=0xD7AF).contains(&cp)
            || (0x3400..=0x4DBF).contains(&cp)
            || (0x20000..=0x2A6DF).contains(&cp)
        {
            return false;
        }
    }
    true
}

fn is_format_template(s: &str) -> bool {
    let trimmed = s.trim();
    if !trimmed.contains("{{") {
        return false;
    }
    let re = match Regex::new(r"\{\{[^}]*\}\}") {
        Ok(r) => r,
        Err(_) => return false,
    };
    let stripped = re.replace_all(trimmed, " ").to_string();
    for c in stripped.chars() {
        if c.is_alphabetic() {
            return false;
        }
    }
    true
}

fn is_skippable_entry(s: &str) -> bool {
    is_symbol_only(s) || is_format_template(s)
}

fn contains_cjk(s: &str) -> bool {
    s.chars().any(|c| {
        let cp = c as u32;
        (0x4E00..=0x9FFF).contains(&cp)
            || (0x3040..=0x309F).contains(&cp)
            || (0x30A0..=0x30FF).contains(&cp)
            || (0xAC00..=0xD7AF).contains(&cp)
            || (0x3400..=0x4DBF).contains(&cp)
            || (0x20000..=0x2A6DF).contains(&cp)
    })
}

fn looks_like_fictional_language(s: &str) -> bool {
    let trimmed = s.trim();

    let has_emote = match Regex::new(r"\$[a-zA-Z0-9]") {
        Ok(r) => r.is_match(trimmed),
        Err(_) => false,
    };
    if has_emote {
        return looks_like_elves_with_emote(trimmed);
    }

    looks_like_quoted_elves(trimmed)
}

fn looks_like_elves_with_emote(s: &str) -> bool {
    let raw_words: Vec<&str> = s.split_whitespace().collect();
    let clean_words: Vec<String> = raw_words
        .iter()
        .map(|w| w.chars().filter(|c| c.is_ascii_alphabetic()).collect::<String>())
        .filter(|w| w.len() >= 2)
        .collect();
    if clean_words.len() < 3 {
        return false;
    }

    let common_en = [
        "the", "a", "an", "is", "are", "was", "were", "of", "in", "to", "and", "or",
        "for", "with", "on", "at", "by", "from", "as", "this", "that", "it", "its",
        "you", "your", "we", "they", "he", "she", "i", "my", "our", "their", "his",
        "her", "be", "been", "do", "does", "did", "have", "has", "had", "will", "would",
        "can", "could", "should", "may", "might", "must", "shall",
        "not", "no", "yes", "if", "but", "all", "some", "any", "more", "less",
        "one", "two", "three", "four", "five",
        "after", "before", "during", "while", "when", "where", "what", "who", "how", "why",
        "time", "level", "weather", "change", "force", "game", "force",
        "pet", "pets", "auto", "water", "bowl", "bowls", "few", "hide", "show",
        "tomorrow", "today", "yesterday",
    ];
    let en_suffixes = [
        "tion", "sion", "ment", "ness", "ity", "ous", "ive", "ful", "less",
        "able", "ible", "ize", "ise", "ing", "ed", "ly", "er", "est", "en",
    ];
    for w in &clean_words {
        let lower = w.to_lowercase();
        if common_en.contains(&lower.as_str()) {
            return false;
        }
        for suf in &en_suffixes {
            if lower.len() > suf.len() + 2 && lower.ends_with(suf) {
                return false;
            }
        }
    }

    let mut total_alpha = 0usize;
    let mut total_vowel = 0usize;
    for w in &clean_words {
        for c in w.chars() {
            if c.is_ascii_alphabetic() {
                total_alpha += 1;
                if matches!(c, 'a' | 'e' | 'i' | 'o' | 'u') {
                    total_vowel += 1;
                }
            }
        }
    }
    if total_alpha < 10 {
        return false;
    }
    let vowel_ratio = total_vowel as f64 / total_alpha as f64;
    if !(0.25..=0.55).contains(&vowel_ratio) {
        return false;
    }

    let mut has_consonant_cluster = false;
    for w in &clean_words {
        let mut count = 0usize;
        for c in w.chars() {
            if !matches!(c, 'a' | 'e' | 'i' | 'o' | 'u') {
                count += 1;
                if count >= 2 {
                    has_consonant_cluster = true;
                    break;
                }
            } else {
                count = 0;
            }
        }
        if has_consonant_cluster {
            break;
        }
    }
    if !has_consonant_cluster {
        return false;
    }

    true
}

fn looks_like_quoted_elves(s: &str) -> bool {
    let cleaned: String = s
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect();
    let raw_words: Vec<&str> = cleaned.split_whitespace().collect();
    let clean_words: Vec<String> = raw_words
        .iter()
        .filter(|w| w.chars().all(|c| c.is_ascii_alphabetic()))
        .map(|w| w.to_string())
        .filter(|w| w.len() >= 2)
        .collect();
    if clean_words.len() < 4 {
        return false;
    }

    let first = &clean_words[0];
    if first.len() > 6 || first.len() < 2 {
        return false;
    }
    if !first.chars().all(|c| c.is_ascii_uppercase()) {
        return false;
    }

    for w in &clean_words[1..] {
        if w.len() > 7 {
            return false;
        }
    }

    let common_en = [
        "the", "a", "an", "is", "are", "was", "were", "of", "in", "to", "and", "or",
        "for", "with", "on", "at", "by", "from", "as", "this", "that", "it", "its",
        "you", "your", "we", "they", "he", "she", "i", "my", "our", "their", "his",
        "her", "be", "been", "do", "does", "did", "have", "has", "had", "will", "would",
        "can", "could", "should", "may", "might", "must", "shall",
        "not", "no", "yes", "if", "but", "all", "some", "any", "more", "less",
        "one", "two", "three", "four", "five",
        "after", "before", "during", "while", "when", "where", "what", "who", "how", "why",
        "time", "level", "weather", "change", "force", "game",
        "pet", "pets", "auto", "water", "bowl", "bowls", "few", "hide", "show",
        "tomorrow", "today", "yesterday",
    ];
    for w in &clean_words {
        if common_en.contains(&w.to_lowercase().as_str()) {
            return false;
        }
    }

    let mut total_alpha = 0usize;
    let mut total_vowel = 0usize;
    for w in &clean_words {
        for c in w.chars() {
            if c.is_ascii_alphabetic() {
                total_alpha += 1;
                if matches!(c, 'a' | 'e' | 'i' | 'o' | 'u') {
                    total_vowel += 1;
                }
            }
        }
    }
    if total_alpha < 14 {
        return false;
    }
    let vowel_ratio = total_vowel as f64 / total_alpha as f64;
    if vowel_ratio > 0.5 {
        return false;
    }

    let mut has_consonant_cluster = false;
    for w in &clean_words {
        let mut count = 0usize;
        for c in w.chars() {
            if !matches!(c, 'a' | 'e' | 'i' | 'o' | 'u') {
                count += 1;
                if count >= 2 {
                    has_consonant_cluster = true;
                    break;
                }
            } else {
                count = 0;
            }
        }
        if has_consonant_cluster {
            break;
        }
    }
    if !has_consonant_cluster {
        return false;
    }

    true
}

fn is_pure_mod_variable(s: &str) -> bool {
    let trimmed = s.trim();
    let has_token = match Regex::new(r"\$\w|%") {
        Ok(r) => r.is_match(trimmed),
        Err(_) => false,
    };
    if !has_token {
        return false;
    }
    let re_dollar = match Regex::new(r"\$\w+") {
        Ok(r) => r,
        Err(_) => return false,
    };
    let re_brace = match Regex::new(r"\{\{[^}]*\}\}") {
        Ok(r) => r,
        Err(_) => return false,
    };
    let stripped_dollar = re_dollar.replace_all(trimmed, " ");
    let stripped = re_brace.replace_all(&stripped_dollar, " ");
    let alpha_count = stripped.chars().filter(|c| c.is_ascii_alphabetic()).count();
    alpha_count < 8
}

fn is_punctuation_with_smapi(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = s[i..].chars().next().unwrap();
        if c == '$' {
            i += c.len_utf8();
            let mut saw_alnum = false;
            while i < bytes.len() {
                let nc = s[i..].chars().next().unwrap();
                if nc.is_ascii_alphanumeric() {
                    saw_alnum = true;
                    i += nc.len_utf8();
                } else {
                    break;
                }
            }
            if !saw_alnum {
                return false;
            }
        } else if c == '#' || c == '!' || c == '?' || c == '.' || c == '*' || c == '$' {
            i += c.len_utf8();
        } else if !c.is_alphabetic() {
            i += c.len_utf8();
        } else {
            return false;
        }
    }
    true
}

fn looks_like_animal_sound(s: &str) -> bool {
    if !s.contains('*') {
        return false;
    }
    let chars: Vec<(usize, char)> = s.char_indices().collect();
    for (i, (start, c)) in chars.iter().enumerate() {
        if *c != '*' {
            continue;
        }
        let end_rel = chars[i + 1..]
            .iter()
            .find(|(_, ch)| *ch == '*')
            .map(|(pos, _)| *pos);
        let end_pos = match end_rel {
            Some(p) => p,
            None => return false,
        };
        let content_start = start + c.len_utf8();
        if end_pos <= content_start {
            continue;
        }
        let content = &s[content_start..end_pos];
        if content.is_empty() || content.len() > 5 {
            continue;
        }
        if !content.chars().all(|ch| ch.is_ascii_alphabetic() && ch.is_ascii_lowercase()) {
            continue;
        }
        let before = &s[..*start];
        let after = &s[end_pos + 1..];
        if is_punctuation_with_smapi(before) && is_punctuation_with_smapi(after) {
            return true;
        }
    }
    false
}

fn looks_like_display_string(s: &str) -> bool {
    if s.len() > 20 {
        return false;
    }
    let quote_chars = ['"', '\u{201C}', '\u{201D}', '\'', '\u{2018}', '\u{2019}'];
    let stripped = s.trim_matches(|c: char| quote_chars.contains(&c));
    if stripped == s || stripped.is_empty() {
        return false;
    }
    let word_count = stripped.split_whitespace().count();
    if word_count == 0 || word_count > 3 {
        return false;
    }
    if stripped.len() > 15 {
        return false;
    }
    stripped
        .chars()
        .all(|c| c.is_ascii_uppercase() || c == ' ' || c == '-' || c == '!' || c == '?')
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
            let dv = default_val.as_str().unwrap_or("").trim();
            let is_symbol = is_skippable_entry(dv);
            if let Some(target_val) = tgt_map.get(key) {
                let tv = target_val.as_str().unwrap_or("").trim();
                if !tv.is_empty() && (tv != dv || is_symbol) {
                    translated += 1;
                }
            } else if is_symbol {
                translated += 1;
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
            let mut skippable_count = 0u32;
            for (_, default_val) in def_map {
                if is_skippable_entry(default_val.as_str().unwrap_or("")) {
                    skippable_count += 1;
                }
            }
            return Ok(ModTranslationStatus {
                mod_name: mod_name.to_string(),
                mod_path: mod_dir.to_string_lossy().to_string(),
                status: "untranslated".to_string(),
                total_entries: default_total,
                translated_entries: skippable_count,
                remaining_entries: default_total.saturating_sub(skippable_count),
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
                        let dv = default_val.as_str().unwrap_or("").trim();
                        let is_symbol = is_skippable_entry(dv);
                        if let Some(target_val) = tgt_map.get(key) {
                            let tv = target_val.as_str().unwrap_or("").trim();
                            if !tv.is_empty() && (tv != dv || is_symbol) {
                                translated += 1;
                            }
                        } else if is_symbol {
                            translated += 1;
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
                let mut skippable_count = 0u32;
                if let Some(Value::Object(map)) = read_json_file(src) {
                    for (_, val) in map {
                        if is_skippable_entry(val.as_str().unwrap_or("")) {
                            skippable_count += 1;
                        }
                    }
                }
                return Ok(ModTranslationStatus {
                    mod_name: mod_name.to_string(),
                    mod_path: mod_dir.to_string_lossy().to_string(),
                    status: "untranslated".to_string(),
                    total_entries: src_total,
                    translated_entries: skippable_count,
                    remaining_entries: src_total.saturating_sub(skippable_count),
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

    let translated_json: Value = parse_translated_json(&translated, 1, 1)?;

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
    let lang_code = lang_name_to_code(target_lang);
    let parent = path.parent().ok_or("Cannot determine parent directory")?;
    let target_file = parent.join(format!("{}.json", lang_code));

    let mut merged = serde_json::Map::new();

    if target_file.exists() {
        if let Some(existing) = read_json_file(&target_file) {
            if let Some(existing_map) = existing.as_object() {
                for (k, v) in existing_map {
                    merged.insert(k.clone(), v.clone());
                }
            }
        }
    }

    let keys_to_translate: Vec<String> = map.keys().cloned().filter(|k| {
        let source_val = map.get(k).and_then(|v| v.as_str()).unwrap_or("");
        if is_skippable_entry(source_val) {
            return false;
        }
        match merged.get(k).and_then(|v| v.as_str()) {
            None => true,
            Some(existing) => {
                let existing = existing.trim();
                if existing.is_empty() {
                    return true;
                }
                if existing == source_val.trim() {
                    return true;
                }
                !contains_cjk(existing)
            }
        }
    }).collect();
    let already_translated_count = map.len() - keys_to_translate.len();

    if already_translated_count > 0 {
        let _ = app.emit("translate-progress", serde_json::json!({
            "phase": "i18n",
            "chunk_current": 0,
            "chunk_total": 0,
            "entry_current": already_translated_count,
            "entry_total": map.len(),
            "skipped_entries": already_translated_count,
            "first_key": "resuming",
        }));
    }

    let total = keys_to_translate.len();
    if total == 0 {
        let backup_path = format!("{}.svlbak", path.to_string_lossy());
        if !path.with_extension("svlbak").exists() {
            fs::copy(path, &backup_path).map_err(|e| format!("Backup failed: {}", e))?;
        }
        return Ok(TranslateFileResult {
            success: true,
            file_path: path.to_string_lossy().to_string(),
            message: format!("All {} entries already translated", merged.len()),
            backup_path: Some(backup_path),
        });
    }

    let chunk_size = 100usize;
    let mut missing_keys: Vec<String> = Vec::new();
    let mut symbol_only_skipped: usize = 0;

    let mut ai_keys: Vec<String> = Vec::with_capacity(total);
    for k in &keys_to_translate {
        if let Some(val) = map.get(k) {
            let s = val.as_str().unwrap_or("");
            if is_symbol_only(s) {
                merged.insert(k.clone(), val.clone());
                symbol_only_skipped += 1;
                continue;
            }
        }
        ai_keys.push(k.clone());
    }
    let keys_to_translate = ai_keys;
    let total = keys_to_translate.len();
    let total_chunks = if total == 0 { 0 } else { (total + chunk_size - 1) / chunk_size };

    let backup_path = format!("{}.svlbak", path.to_string_lossy());
    if !path.with_extension("svlbak").exists() {
        if target_file.exists() {
            fs::copy(&target_file, &backup_path).map_err(|e| format!("Backup failed: {}", e))?;
        } else {
            fs::copy(path, &backup_path).map_err(|e| format!("Backup failed: {}", e))?;
        }
    }

    if total == 0 {
        if symbol_only_skipped > 0 {
            let formatted = serde_json::to_string_pretty(&Value::Object(merged.clone()))
                .map_err(|e| e.to_string())?;
            fs::write(&target_file, formatted).map_err(|e| format!("Write failed: {}", e))?;
            return Ok(TranslateFileResult {
                success: true,
                file_path: path.to_string_lossy().to_string(),
                message: format!(
                    "All {} entries already handled ({} symbol-only values preserved as-is, {} previously translated)",
                    merged.len(), symbol_only_skipped, already_translated_count
                ),
                backup_path: Some(backup_path),
            });
        }
        return Ok(TranslateFileResult {
            success: true,
            file_path: path.to_string_lossy().to_string(),
            message: format!("All {} entries already translated", merged.len()),
            backup_path: Some(backup_path),
        });
    }

    for (chunk_idx, chunk_start) in (0..total).step_by(chunk_size).enumerate() {
        let chunk_end = (chunk_start + chunk_size).min(total);
        let chunk_keys: Vec<String> = keys_to_translate[chunk_start..chunk_end].to_vec();

        let _ = app.emit("translate-progress", serde_json::json!({
            "phase": "i18n",
            "chunk_current": chunk_idx + 1,
            "chunk_total": total_chunks,
            "entry_current": already_translated_count + symbol_only_skipped + chunk_end,
            "entry_total": map.len(),
            "current_keys": chunk_keys.iter().take(5).cloned().collect::<Vec<_>>(),
            "first_key": chunk_keys.first().unwrap_or(&String::new()),
        }));

        let (translated_obj, mut chunk_missing) = translate_chunk_with_retry(
            app, &chunk_keys, map, "i18n", ai_config, target_lang, chunk_idx + 1, total_chunks
        ).await?;

        if let Some(obj) = translated_obj.as_object() {
            for key in &chunk_keys {
                if let Some(val) = obj.get(key) {
                    let s = val.as_str().unwrap_or("").trim().to_string();
                    if !s.is_empty() {
                        if let Some(default_val) = map.get(key) {
                            let dv = default_val.as_str().unwrap_or("").trim();
                            if s != dv {
                                merged.insert(key.clone(), Value::String(s));
                                continue;
                            }
                        }
                        merged.insert(key.clone(), val.clone());
                    } else {
                        chunk_missing.push(key.clone());
                    }
                } else {
                    chunk_missing.push(key.clone());
                }
            }
        }

        for k in &chunk_missing {
            if !missing_keys.contains(k) {
                missing_keys.push(k.clone());
            }
        }

        emit_chunk_samples(app, &chunk_keys, map, &merged, chunk_idx + 1, total_chunks);

        let formatted = serde_json::to_string_pretty(&Value::Object(merged.clone()))
            .map_err(|e| e.to_string())?;
        fs::write(&target_file, formatted).map_err(|e| format!("Write failed: {}", e))?;

        if chunk_missing.len() == chunk_keys.len() && !chunk_keys.is_empty() {
            return Err(format!(
                "Chunk {}/{}: AI failed to translate any of the {} keys in this batch. {} entries from previous chunks have been saved. Missing keys: {:?}",
                chunk_idx + 1, total_chunks, chunk_keys.len(), merged.len() - already_translated_count,
                chunk_keys.iter().take(5).cloned().collect::<Vec<_>>()
            ));
        }
    }

    let mut message = format!(
        "Translated {} entries in {} chunks ({} skipped as already done, {} symbol-only values preserved as-is)",
        total - missing_keys.len(), total_chunks, already_translated_count, symbol_only_skipped
    );
    if !missing_keys.is_empty() {
        message.push_str(&format!(
            ". {} keys were not translated by AI and remain in source language: {:?}",
            missing_keys.len(),
            missing_keys.iter().take(5).cloned().collect::<Vec<_>>()
        ));
    }

    Ok(TranslateFileResult {
        success: true,
        file_path: path.to_string_lossy().to_string(),
        message,
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

    let mut already_translated_count = 0usize;
    let mut all_translated_changes: Vec<Value> = Vec::new();

    let backup_path = format!("{}.svlbak", path.to_string_lossy());
    let bak_file = path.with_extension("svlbak");

    if !bak_file.exists() {
        fs::copy(path, &backup_path).map_err(|e| format!("Backup failed: {}", e))?;
    }

    if bak_file.exists() {
        if let Some(current_json) = read_json_file(path) {
            if let Some(current_changes) = current_json.get("Changes").and_then(|v| v.as_array()) {
                let original_json = read_json_file(&bak_file);
                let original_changes = original_json
                    .as_ref()
                    .and_then(|j| j.get("Changes"))
                    .and_then(|v| v.as_array());

                if let Some(orig_arr) = original_changes {
                    if current_changes.len() > orig_arr.len() {
                        already_translated_count = current_changes.len() - orig_arr.len();
                        all_translated_changes = current_changes.clone();
                    }
                } else {
                    if current_changes.len() > 0 {
                        all_translated_changes = current_changes.clone();
                    }
                }
            }
        }
    }

    let start_index = if all_translated_changes.len() > changes.len() {
        0
    } else {
        all_translated_changes.len().min(total)
    };

    if already_translated_count > 0 {
        let _ = app.emit("translate-progress", serde_json::json!({
            "phase": "content",
            "chunk_current": 0,
            "chunk_total": 0,
            "entry_current": already_translated_count,
            "entry_total": total,
            "skipped_entries": already_translated_count,
            "first_key": "resuming",
        }));
    }

    if start_index >= total {
        let mut result_obj = serde_json::Map::new();
        if let Some(format) = parsed.get("Format") {
            result_obj.insert("Format".to_string(), format.clone());
        }
        result_obj.insert("Changes".to_string(), Value::Array(all_translated_changes));

        let formatted = serde_json::to_string_pretty(&Value::Object(result_obj))
            .map_err(|e| e.to_string())?;
        fs::write(path, formatted).map_err(|e| format!("Write failed: {}", e))?;

        return Ok(TranslateFileResult {
            success: true,
            file_path: path.to_string_lossy().to_string(),
            message: format!("All {} changes already translated", total),
            backup_path: Some(backup_path),
        });
    }

    let chunk_size = 20usize;
    let remaining = total - start_index;
    let total_chunks = (remaining + chunk_size - 1) / chunk_size;
    let mut chunks_completed = 0usize;

    for (chunk_idx, chunk_start) in (start_index..total).step_by(chunk_size).enumerate() {
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

        let mut translated_str: Option<String> = None;
        let mut last_error: Option<String> = None;
        for retry in 0..3 {
            match call_ai_translate(&chunk_json, "content", ai_config, target_lang).await {
                Ok(t) => {
                    translated_str = Some(t);
                    break;
                }
                Err(e) => {
                    last_error = Some(e);
                    if retry < 2 {
                        tokio::time::sleep(std::time::Duration::from_millis(500 * (retry + 1) as u64)).await;
                    }
                }
            }
        }

        let translated = match translated_str {
            Some(t) => t,
            None => {
                let e = last_error.unwrap_or_else(|| "Unknown error".to_string());
                if chunks_completed > 0 {
                    let mut result_obj = serde_json::Map::new();
                    if let Some(format) = parsed.get("Format") {
                        result_obj.insert("Format".to_string(), format.clone());
                    }
                    result_obj.insert("Changes".to_string(), Value::Array(all_translated_changes.clone()));
                    if let Ok(formatted) = serde_json::to_string_pretty(&Value::Object(result_obj)) {
                        let _ = fs::write(path, formatted);
                    }
                }
                return Err(format!(
                    "Chunk {}/{} failed: {}. {} changes from {} completed chunks have been saved.",
                    chunk_idx + 1, total_chunks, e, all_translated_changes.len() - start_index, chunks_completed
                ));
            }
        };

        let translated_json = parse_translated_json(&translated, chunk_idx + 1, total_chunks)?;

        if let Some(arr) = translated_json.get("Changes").and_then(|v| v.as_array()) {
            all_translated_changes.extend(arr.iter().cloned());
        } else {
            return Err(format!(
                "Chunk {}/{}: AI response did not contain 'Changes' array. Got: {}",
                chunk_idx + 1, total_chunks,
                translated_json.to_string().chars().take(200).collect::<String>()
            ));
        }

        chunks_completed += 1;

        emit_content_chunk_samples(app, chunk_changes, &all_translated_changes, chunk_idx + 1, total_chunks);

        let mut result_obj = serde_json::Map::new();
        if let Some(format) = parsed.get("Format") {
            result_obj.insert("Format".to_string(), format.clone());
        }
        result_obj.insert("Changes".to_string(), Value::Array(all_translated_changes.clone()));

        let formatted = serde_json::to_string_pretty(&Value::Object(result_obj))
            .map_err(|e| e.to_string())?;
        fs::write(path, formatted).map_err(|e| format!("Write failed: {}", e))?;
    }

    Ok(TranslateFileResult {
        success: true,
        file_path: path.to_string_lossy().to_string(),
        message: format!("Translated {} changes in {} chunks ({} skipped as already done)", remaining, total_chunks, already_translated_count),
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

#[derive(Serialize)]
pub struct BackupEntry {
    backup_path: String,
    original_path: String,
    mod_name: String,
    relative_path: String,
    backup_time: u64,
}

#[tauri::command]
pub async fn scan_translation_backups(mods_dir: String) -> Result<Vec<BackupEntry>, String> {
    let mods_path = PathBuf::from(&mods_dir);
    if !mods_path.exists() {
        return Ok(vec![]);
    }
    let mut entries = Vec::new();
    scan_backups_recursive(&mods_path, &mods_path, &mut entries)?;
    entries.sort_by(|a, b| b.backup_time.cmp(&a.backup_time));
    Ok(entries)
}

fn scan_backups_recursive(dir: &Path, mods_root: &Path, entries: &mut Vec<BackupEntry>) -> Result<(), String> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).map_err(|e| format!("Read dir failed: {}", e))? {
        let entry = entry.map_err(|e| format!("Entry failed: {}", e))?;
        let path = entry.path();
        if path.is_dir() {
            scan_backups_recursive(&path, mods_root, entries)?;
        } else {
            let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            if file_name.ends_with(".svlbak") {
                let original_path = path.to_string_lossy().to_string().replace(".svlbak", "");
                let original = PathBuf::from(&original_path);
                if !original.exists() {
                    continue;
                }
                let rel = original.parent().and_then(|p| p.file_name()).unwrap_or_default().to_string_lossy().to_string();
                let file_rel = original.strip_prefix(mods_root).unwrap_or(&original).to_string_lossy().to_string();
                let metadata = fs::metadata(&path).ok();
                let backup_time = metadata.and_then(|m| m.modified().ok())
                    .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
                    .unwrap_or(0);
                entries.push(BackupEntry {
                    backup_path: path.to_string_lossy().to_string(),
                    original_path,
                    mod_name: rel,
                    relative_path: file_rel,
                    backup_time,
                });
            }
        }
    }
    Ok(())
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
            "\n15. This is an i18n translation file for a Stardew Valley mod. The file uses language keys as JSON keys. Translate all string VALUES to {}. Keep all JSON keys exactly as they are. The result will be saved as {}.json in the i18n folder.",
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
12. For config files, translate option descriptions and display names but keep technical values unchanged.\
13. TRANSLATE NPC, character, location, and creature names into natural {}. These are game content meant to be read by players, not technical identifiers. Examples: 'Mr. Raccoon' -> '浣熊先生', 'Mrs. Raccoon' -> '浣熊夫人', 'Pierre' -> '皮埃尔', 'Abigail' -> '阿比盖尔', 'Marnie' -> '玛妮', 'Wizard' -> '法师', 'Krobus' -> '克劳布斯'. When unsure, translate rather than leave English, as the player's reading experience is the priority.\
14. For format strings containing placeholders such as {{percent}}, {{name}}, {{value}} etc., keep the placeholders EXACTLY unchanged, but you may translate the surrounding literal text (e.g. '{{percent}}% complete' can become '完成 {{percent}}%').{}",
        target_lang, target_lang, target_lang, i18n_instruction
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
    let body = serde_json::json!({
        "model": ai_config.model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt}
        ],
        "temperature": 0.3,
        "response_format": {"type": "json_object"}
    });
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", ai_config.api_key))
        .header("Content-Type", "application/json")
        .json(&body)
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

    let cleaned = extract_json_from_response(translated);
    Ok(cleaned)
}

fn extract_json_from_response(raw: &str) -> String {
    let trimmed = raw.trim();

    if trimmed.starts_with("```json") {
        return trimmed
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim()
            .to_string();
    }
    if trimmed.starts_with("```") {
        return trimmed
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
            .to_string();
    }

    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            if end > start {
                return trimmed[start..=end].to_string();
            }
        }
    }

    if let Some(start) = trimmed.find('[') {
        if let Some(end) = trimmed.rfind(']') {
            if end > start {
                return trimmed[start..=end].to_string();
            }
        }
    }

    trimmed.to_string()
}

fn parse_translated_json(translated: &str, chunk_idx: usize, total_chunks: usize) -> Result<Value, String> {
    let cleaned = extract_json_from_response(translated);

    match serde_json::from_str::<Value>(&cleaned) {
        Ok(v) => Ok(v),
        Err(e) => {
            let len = cleaned.len();
            let preview_end = (300).min(len);
            Err(format!(
                "Chunk {}/{}: AI returned invalid JSON: {}. Response length: {} chars. First {} chars: {}",
                chunk_idx, total_chunks, e, len, preview_end, &cleaned[..preview_end]
            ))
        }
    }
}

async fn translate_chunk_with_retry(
    app: &tauri::AppHandle,
    chunk_keys: &[String],
    source_map: &serde_json::Map<String, Value>,
    file_type: &str,
    ai_config: &AiConfig,
    target_lang: &str,
    chunk_idx: usize,
    total_chunks: usize,
) -> Result<(Value, Vec<String>), String> {
    let mut current_keys: Vec<String> = chunk_keys.to_vec();
    let mut last_error: Option<String> = None;
    let mut last_translated: Option<Value> = None;
    let batch_sizes = [current_keys.len(), 50usize, 25usize];

    for (attempt, &batch_size) in batch_sizes.iter().enumerate() {
        if current_keys.is_empty() {
            break;
        }

        if current_keys.len() > batch_size {
            current_keys.truncate(batch_size);
        }

        if attempt > 0 {
            let _ = app.emit("translate-progress", serde_json::json!({
                "phase": "i18n",
                "chunk_current": chunk_idx,
                "chunk_total": total_chunks,
                "entry_current": 0,
                "entry_total": 0,
                "current_keys": current_keys.iter().take(5).cloned().collect::<Vec<_>>(),
                "first_key": format!("retry #{} ({} keys)", attempt, current_keys.len()),
            }));
            tokio::time::sleep(std::time::Duration::from_millis(800 * attempt as u64)).await;
        }

        let mut chunk_obj = serde_json::Map::new();
        for key in &current_keys {
            if let Some(val) = source_map.get(key) {
                chunk_obj.insert(key.clone(), val.clone());
            }
        }
        let chunk_json = serde_json::to_string(&Value::Object(chunk_obj))
            .map_err(|e| e.to_string())?;

        let mut call_success = false;
        for retry in 0..3 {
            match call_ai_translate(&chunk_json, file_type, ai_config, target_lang).await {
                Ok(translated) => {
                    call_success = true;
                    match parse_translated_json(&translated, chunk_idx, total_chunks) {
                        Ok(parsed) => {
                            last_translated = Some(parsed);
                            last_error = None;
                            break;
                        }
                        Err(e) => {
                            last_error = Some(e);
                        }
                    }
                }
                Err(e) => {
                    last_error = Some(e);
                    if retry < 2 {
                        tokio::time::sleep(std::time::Duration::from_millis(500 * (retry + 1) as u64)).await;
                    }
                }
            }
        }

        if !call_success && last_error.is_some() {
            if attempt == batch_sizes.len() - 1 {
                return Err(last_error.unwrap());
            }
            continue;
        }

        let translated_obj = match last_translated.take() {
            Some(v) => v,
            None => Value::Object(serde_json::Map::new()),
        };
        let obj = match translated_obj.as_object() {
            Some(o) => o,
            None => {
                if attempt == batch_sizes.len() - 1 {
                    return Err(format!(
                        "Chunk {}/{}: AI returned non-object JSON",
                        chunk_idx, total_chunks
                    ));
                }
                last_translated = Some(translated_obj);
                continue;
            }
        };

        let mut missing: Vec<String> = Vec::new();
        for key in &current_keys {
            match obj.get(key) {
                Some(val) => {
                    let s = val.as_str().unwrap_or("").trim();
                    if s.is_empty() {
                        missing.push(key.clone());
                    }
                }
                None => {
                    missing.push(key.clone());
                }
            }
        }

        if missing.is_empty() {
            return Ok((translated_obj, vec![]));
        }

        if attempt == batch_sizes.len() - 1 {
            return Ok((translated_obj, missing));
        }

        last_translated = Some(translated_obj);
        current_keys = missing;
    }

    Ok((last_translated.unwrap_or(Value::Object(serde_json::Map::new())), chunk_keys.to_vec()))
}

fn emit_chunk_samples(
    app: &tauri::AppHandle,
    chunk_keys: &[String],
    source_map: &serde_json::Map<String, Value>,
    merged: &serde_json::Map<String, Value>,
    chunk_idx: usize,
    total_chunks: usize,
) {
    let max_translated = 25usize;
    let max_missing = 10usize;
    let mut translated: Vec<Value> = Vec::new();
    let mut missing: Vec<Value> = Vec::new();

    for key in chunk_keys {
        if translated.len() >= max_translated && missing.len() >= max_missing {
            break;
        }
        let source_val = match source_map.get(key) {
            Some(v) => v,
            None => continue,
        };
        let source_str = source_val.as_str().unwrap_or("").to_string();
        if source_str.is_empty() {
            continue;
        }

        match merged.get(key) {
            Some(translated_val) => {
                let translated_str = translated_val.as_str().unwrap_or("").to_string();
                let trimmed = translated_str.trim();
                if !trimmed.is_empty() && trimmed != source_str.trim() {
                    if translated.len() < max_translated {
                        translated.push(serde_json::json!({
                            "key": key,
                            "source": source_str,
                            "translation": translated_str,
                        }));
                    }
                } else if missing.len() < max_missing {
                    missing.push(serde_json::json!({
                        "key": key,
                        "source": source_str,
                        "translation": "",
                    }));
                }
            }
            None => {
                if missing.len() < max_missing {
                    missing.push(serde_json::json!({
                        "key": key,
                        "source": source_str,
                        "translation": "",
                    }));
                }
            }
        }
    }

    if translated.is_empty() && missing.is_empty() {
        return;
    }

    let _ = app.emit("translate-sample", serde_json::json!({
        "chunk_index": chunk_idx,
        "total_chunks": total_chunks,
        "translated": translated,
        "missing": missing,
    }));
}

fn emit_content_chunk_samples(
    app: &tauri::AppHandle,
    chunk_changes: &[Value],
    all_translated: &[Value],
    chunk_idx: usize,
    total_chunks: usize,
) {
    let max_translated = 10usize;
    let start = all_translated.len().saturating_sub(chunk_changes.len());
    let mut translated: Vec<Value> = Vec::new();

    for (i, change) in chunk_changes.iter().enumerate() {
        if translated.len() >= max_translated {
            break;
        }
        let log_name = extract_change_log_name(change);
        let source_text = extract_change_text(change);
        if source_text.is_empty() {
            continue;
        }
        let translated_change = all_translated.get(start + i);
        let translation_text = translated_change
            .map(extract_change_text)
            .unwrap_or_default();
        if translation_text.is_empty() || translation_text == source_text {
            continue;
        }
        translated.push(serde_json::json!({
            "key": log_name,
            "source": source_text,
            "translation": translation_text,
        }));
    }

    if translated.is_empty() {
        return;
    }

    let _ = app.emit("translate-sample", serde_json::json!({
        "chunk_index": chunk_idx,
        "total_chunks": total_chunks,
        "translated": translated,
        "missing": Vec::<Value>::new(),
    }));
}

fn extract_change_log_name(change: &Value) -> String {
    change.get("LogName")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            change.get("Action")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| format!("Change#{}", change.get("__idx").and_then(|v| v.as_u64()).unwrap_or(0)))
}

fn extract_change_text(change: &Value) -> String {
    if let Some(when) = change.get("When") {
        if let Some(s) = when.get("Message").and_then(|v| v.as_str()) {
            if !s.is_empty() {
                return s.to_string();
            }
        }
    }
    for key in &["Text", "DisplayName", "DisplayText", "Name", "Description"] {
        if let Some(s) = change.get(key).and_then(|v| v.as_str()) {
            if !s.is_empty() {
                return s.to_string();
            }
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_symbol_only_pure_punctuation() {
        assert!(is_symbol_only("???"));
        assert!(is_symbol_only("!?!?"));
        assert!(is_symbol_only("..."));
        assert!(is_symbol_only("....$2"));
        assert!(is_symbol_only("!@#$%^&*()"));
    }

    #[test]
    fn is_symbol_only_sdv_emote_codes() {
        assert!(is_symbol_only("$1"));
        assert!(is_symbol_only("$e"));
        assert!(is_symbol_only("$1#$e#"));
        assert!(is_symbol_only("$h#"));
        assert!(is_symbol_only("$1#$e#...$4"));
    }

    #[test]
    fn is_symbol_only_short_single_letter() {
        assert!(is_symbol_only("e"));
        assert!(is_symbol_only(":D"));
        assert!(is_symbol_only(":)"));
        assert!(is_symbol_only("~"));
    }

    #[test]
    fn is_symbol_only_empty_and_whitespace() {
        assert!(is_symbol_only(""));
        assert!(is_symbol_only("   "));
        assert!(is_symbol_only("\t\n"));
    }

    #[test]
    fn is_symbol_only_real_text_should_not_match() {
        assert!(!is_symbol_only("Oh"));
        assert!(!is_symbol_only("Hi"));
        assert!(!is_symbol_only("Hello"));
        assert!(!is_symbol_only("Hmm..."));
        assert!(!is_symbol_only("Oh... I'm so tired"));
        assert!(!is_symbol_only("Hello $1 World"));
    }

    #[test]
    fn is_symbol_only_cjk_text_should_not_match() {
        assert!(!is_symbol_only("你好"));
        assert!(!is_symbol_only("星露谷"));
        assert!(!is_symbol_only("こんにちは"));
        assert!(!is_symbol_only("안녕하세요"));
    }

    #[test]
    fn is_symbol_only_unicode_alphabetic() {
        assert!(!is_symbol_only("café"));
        assert!(!is_symbol_only("naïve"));
        assert!(!is_symbol_only("Zürich"));
    }

    #[test]
    fn is_symbol_only_animal_sound_pattern() {
        assert!(is_symbol_only("*meep*"));
        assert!(is_symbol_only("*meep*!!!"));
        assert!(is_symbol_only("*meep*...$2"));
        assert!(is_symbol_only("*meep*!!!$0"));
        assert!(is_symbol_only("*woof*"));
        assert!(is_symbol_only("*hiss*!!!$1"));
        assert!(is_symbol_only("!*purr*$e#"));
        assert!(!is_symbol_only("*Hello World*"));
        assert!(!is_symbol_only("*important*"));
    }

    #[test]
    fn is_symbol_only_display_string() {
        assert!(is_symbol_only("\"CLOSED\""));
        assert!(is_symbol_only("\"OPEN\""));
        assert!(is_symbol_only("\"ON SALE\""));
        assert!(is_symbol_only("\"HELLO!\""));
        assert!(is_symbol_only("'SOLD'"));
        assert!(!is_symbol_only("\"Hello\""));
        assert!(!is_symbol_only("\"This is a sign with many words\""));
        assert!(!is_symbol_only("CLOSED"));
    }

    #[test]
    fn is_format_template_pure_placeholders() {
        assert!(is_format_template("{{chestName}}"));
        assert!(is_format_template("{{name}}"));
        assert!(is_format_template("{{value}}"));
    }

    #[test]
    fn is_format_template_placeholders_with_punctuation() {
        assert!(is_format_template("{{locationName}} #{{number}}"));
        assert!(is_format_template("{{name}} #{{number}}"));
        assert!(is_format_template("{{percent}}%"));
        assert!(is_format_template("{{a}} - {{b}}"));
    }

    #[test]
    fn is_format_template_real_text_should_not_match() {
        assert!(!is_format_template("Increase mastery level: {{currentLevel}}"));
        assert!(!is_format_template("Auto-Water Pet Bowls"));
        assert!(!is_format_template("Hide '{{timeFrozen}}' After a Few Seconds"));
        assert!(!is_format_template("The game forces tomorrow's weather to {{weather}}, so it can't be changed."));
        assert!(!is_format_template("Hello world"));
    }

    #[test]
    fn is_format_template_no_placeholder() {
        assert!(!is_format_template(""));
        assert!(!is_format_template("plain text"));
        assert!(!is_format_template("Mr. Raccoon"));
        assert!(!is_format_template("{{unclosed"));
    }

    #[test]
    fn is_skippable_entry_combined() {
        assert!(is_skippable_entry("{{chestName}}"));
        assert!(is_skippable_entry("{{locationName}} #{{number}}"));
        assert!(is_skippable_entry("$1"));
        assert!(is_skippable_entry("*meep*"));
        assert!(is_skippable_entry("\"CLOSED\""));
        assert!(!is_skippable_entry("Increase mastery level: {{currentLevel}}"));
        assert!(!is_skippable_entry("Auto-Water Pet Bowls"));
    }

    #[test]
    fn contains_cjk_detection() {
        assert!(contains_cjk("浣熊先生"));
        assert!(contains_cjk("增加 10% 经验"));
        assert!(contains_cjk("完成"));
        assert!(contains_cjk("皮埃尔"));
        assert!(!contains_cjk("Mr. Raccoon"));
        assert!(!contains_cjk("Auto-Water Pet Bowls"));
        assert!(!contains_cjk(""));
        assert!(!contains_cjk("Neen veniie anar"));
    }

    #[test]
    fn fictional_language_sve_elves() {
        assert!(looks_like_fictional_language("Neen veniie anar anaroore anrrima anarya ango angwil$1"));
        assert!(looks_like_fictional_language("Krobus nar harr veniie angwil$2"));
        assert!(looks_like_fictional_language("Naro siene anra naen vola harr$3"));
    }

    #[test]
    fn fictional_language_no_emote_code() {
        assert!(!looks_like_fictional_language("Neen veniie anar anroore"));
        assert!(!looks_like_fictional_language("hello there friend"));
    }

    #[test]
    fn fictional_language_normal_english_with_emote() {
        assert!(!looks_like_fictional_language("Hello $1 World"));
        assert!(!looks_like_fictional_language("Mr $2 Raccoon"));
    }

    #[test]
    fn fictional_language_normal_english_sentences() {
        assert!(!looks_like_fictional_language("Increase mastery level: {{currentLevel}}"));
        assert!(!looks_like_fictional_language("Auto-Water Pet Bowls"));
        assert!(!looks_like_fictional_language("Hide '{{timeFrozen}}' After a Few Seconds"));
        assert!(!looks_like_fictional_language("The game forces tomorrow's weather to {{weather}}, so it can't be changed."));
        assert!(!looks_like_fictional_language("Mr. Raccoon"));
    }

    #[test]
    fn is_symbol_only_includes_fictional_language() {
        assert!(is_symbol_only("Neen veniie anar anaroore anrrima anarya ango angwil$1"));
        assert!(is_symbol_only("Krobus nar harr veniie angwil$2"));
    }

    #[test]
    fn is_pure_mod_variable_patterns() {
        assert!(is_pure_mod_variable("…%noturn$7"));
        assert!(is_pure_mod_variable("$1"));
        assert!(is_pure_mod_variable("$e"));
        assert!(is_pure_mod_variable("%var$2"));
        assert!(is_pure_mod_variable("…%noturn$1"));
    }

    #[test]
    fn is_pure_mod_variable_real_text() {
        assert!(!is_pure_mod_variable("Hello $1 World"));
        assert!(!is_pure_mod_variable("The game forces tomorrow's weather to {{weather}}$1"));
        assert!(!is_pure_mod_variable("Neen veniie anar anaroore anrrima anarya ango angwil$1"));
        assert!(!is_pure_mod_variable("plain text"));
    }

    #[test]
    fn looks_like_quoted_elves_sve() {
        assert!(looks_like_fictional_language("\"ANKAR\"^^\"Huot hamp lonozzl doo duna tol\""));
        assert!(looks_like_fictional_language("ANKAR Huot hamp lonozzl doo duna tol"));
    }

    #[test]
    fn looks_like_quoted_elves_normal_english() {
        assert!(!looks_like_fictional_language("Hello World how are you"));
        assert!(!looks_like_fictional_language("The quick brown fox jumps"));
        assert!(!looks_like_fictional_language("My name is Bob Smith"));
    }

    #[test]
    fn is_symbol_only_handles_sve_remaining() {
        assert!(is_symbol_only("…%noturn$7"));
        assert!(is_symbol_only("\"ANKAR\"^^\"Huot hamp lonozzl doo duna tol\""));
    }
}
