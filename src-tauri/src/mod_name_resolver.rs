use std::collections::HashMap;
use std::sync::OnceLock;

use crate::compatibility_list::get_mod_metadata;

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

    unique_id.to_string()
}

fn get_name_dict() -> &'static HashMap<String, String> {
    static NAME_DICT: OnceLock<HashMap<String, String>> = OnceLock::new();
    NAME_DICT.get_or_init(|| {
        let dict_str = include_str!("../mod_name_dict.json");
        serde_json::from_str(dict_str).unwrap_or_default()
    })
}
