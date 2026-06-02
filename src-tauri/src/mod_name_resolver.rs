use std::collections::HashMap;
use std::sync::OnceLock;

use crate::compatibility_list::get_mod_metadata;

fn add_spaces_to_camel_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 8);
    let chars: Vec<char> = s.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && c.is_uppercase() {
            let prev = chars[i - 1];
            let next_is_lower = chars.get(i + 1).map_or(true, |n| n.is_lowercase());
            if prev.is_lowercase() || (prev.is_uppercase() && next_is_lower) {
                result.push(' ');
            }
        }
        result.push(*c);
    }
    result
}

fn strip_author_prefix(unique_id: &str) -> String {
    if let Some(pos) = unique_id.find('.') {
        let suffix = &unique_id[pos + 1..];
        if !suffix.is_empty() {
            return suffix.to_string();
        }
    }
    unique_id.to_string()
}

pub fn resolve_mod_name(unique_id: &str) -> String {
    if let Some(metadata) = get_mod_metadata(unique_id) {
        if !metadata.name.is_empty() {
            return metadata.name;
        }
    }

    let name_dict = get_name_dict();
    if let Some(name) = name_dict.get(unique_id) {
        return name.clone();
    }

    for (uid, name) in name_dict {
        if uid.to_lowercase() == unique_id.to_lowercase() {
            return name.clone();
        }
    }

    let suffix = strip_author_prefix(unique_id);
    if suffix != unique_id {
        if let Some(metadata) = get_mod_metadata(&suffix) {
            if !metadata.name.is_empty() {
                return metadata.name;
            }
        }
        if let Some(name) = name_dict.get(&suffix) {
            return name.clone();
        }
        for (uid, name) in name_dict {
            if uid.to_lowercase() == suffix.to_lowercase() {
                return name.clone();
            }
        }
        let pretty = add_spaces_to_camel_case(&suffix);
        if pretty != suffix {
            return pretty;
        }
    } else {
        let pretty = add_spaces_to_camel_case(unique_id);
        if pretty != unique_id {
            return pretty;
        }
    }

    unique_id.to_string()
}

fn get_name_dict() -> &'static HashMap<String, String> {
    static NAME_DICT: OnceLock<HashMap<String, String>> = OnceLock::new();
    NAME_DICT.get_or_init(|| {
        let dict_str = include_str!("../mod_name_dict.json");
        serde_json::from_str(dict_str).unwrap_or_default()
    })
}
