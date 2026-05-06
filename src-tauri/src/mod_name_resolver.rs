use std::collections::HashMap;
use std::sync::OnceLock;

use crate::compatibility_list::get_mod_metadata;

pub fn resolve_mod_name(unique_id: &str) -> String {
    if let Some(metadata) = get_mod_metadata(unique_id) {
        if !metadata.name.is_empty() {
            return metadata.name;
        }
    }

    let dict = get_mod_dict();
    if let Some(name) = dict.get(unique_id) {
        return name.clone();
    }

                                                         for (uid, name) in dict {
        if uid.to_lowercase() == unique_id.to_lowercase() {
            return name.clone();
        }
    }

    unique_id.to_string()
}

fn get_mod_dict() -> &'static HashMap<String, String> {
    static MOD_DICT: OnceLock<HashMap<String, String>> = OnceLock::new();
    MOD_DICT.get_or_init(|| {
        let dict_str = include_str!("../mod_dict.json");
        serde_json::from_str(dict_str).unwrap_or_default()
    })
}
