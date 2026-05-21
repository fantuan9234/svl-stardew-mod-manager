use serde::{Deserialize, Serialize};
use std::collections::HashMap;

lazy_static::lazy_static! {
    pub static ref BUILTIN_DICT: HashMap<&'static str, u64> = {
        let mut m = HashMap::new();
        m.insert("Pathoschild.ContentPatcher", 1915);
        m.insert("Esca.FarmTypeManager", 3231);
        m.insert("Stardew Valley Expanded Farm Type Manager", 3231);
        m.insert("Farm Type Manager", 3231);
        m.insert("FlashShifter.StardewValleyExpandedCP", 3753);
        m.insert("Pathoschild.SMAPI", 2400);
        m.insert("spacechase0.SpaceCore", 1348);
        m.insert("spacechase0.JsonAssets", 1720);
        m.insert("Pathoschild.FashionSense", 9960);
        m.insert("spacechase0.GenericModConfigMenu", 5098);
        m.insert("furyx639.DynamicGameAssets", 13519);
        m.insert("furyx639.ExpandedPreconditionsUtility", 11229);
        m.insert("Candace.AkaiRinRidgesideVillage", 3562);
        m.insert("MizzionRpg.CustomNpcExclusions", 7103);
        m.insert("furyx639.CustomCompanions", 13519);
        m.insert("Aerin.MoreFish", 2413);
        m.insert("Tanpoponoko.SeasonalOutfits", 10220);
        m.insert("Rose.craftables", 11062);
        m.insert("Pathoschild.Automate", 1063);
        m.insert("CantorsDust.LookupAnything", 541);
        m.insert("Pathoschild.TractorMod", 541);
        m.insert("Pathoschild.ChestsAnywhere", 518);
        m.insert("Bouhm.NpcMapLocations", 239);
        m.insert("MizzionRpg.ProjectR", 11311);
        m.insert("furyx639.BetterFarmAnimalVariety", 11229);
        m.insert("ConsoleCommands", 2400);
        m.insert("SaveBackup", 2400);
        m
    };
    
    // 文件夹名 -> Nexus ID 映射（用于 SVE 等根 manifest 无 UpdateKeys 的情况）
    pub static ref FOLDER_NAME_DICT: HashMap<&'static str, u64> = {
        let mut m = HashMap::new();
        m.insert("Stardew Valley Expanded", 3753);
        m.insert("StardewValleyExpanded", 3753);
        m.insert("Farm Type Manager", 3231);
        m.insert("Ridgeside Village", 3562);
        m.insert("Custom NPC Exclusions", 7103);
        m.insert("Automate", 1063);
        m.insert("LookupAnything", 541);
        m.insert("Content Patcher", 1915);
        m.insert("Generic Mod Config Menu", 5098);
        m.insert("SpaceCore", 1348);
        m.insert("Json Assets", 1720);
        m.insert("Fashion Sense", 9960);
        m.insert("Dynamic Game Assets", 13519);
        m.insert("Expanded Preconditions Utility", 11229);
        m
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexusLinkResult {
    pub url: String,
    pub method: String,
    pub mod_id: Option<String>,
}

pub fn build_nexus_link(unique_id: &str, mod_name: Option<&str>, nexus_mod_id: Option<u64>) -> NexusLinkResult {
    if let Some(nexus_id) = nexus_mod_id {
        let url = format!("https://www.nexusmods.com/stardewvalley/mods/{}", nexus_id);
        return NexusLinkResult {
            url,
            method: "manifest_update_keys".to_string(),
            mod_id: Some(nexus_id.to_string()),
        };
    }

    if let Some(nexus_id) = BUILTIN_DICT.get(unique_id) {
        let url = format!("https://www.nexusmods.com/stardewvalley/mods/{}", nexus_id);
        return NexusLinkResult {
            url,
            method: "builtin_dict".to_string(),
            mod_id: Some(nexus_id.to_string()),
        };
    }

    if let Some(nexus_id) = crate::smapi_data::get_mod_nexus_id(unique_id) {
        let url = format!("https://www.nexusmods.com/stardewvalley/mods/{}", nexus_id);
        return NexusLinkResult {
            url,
            method: "smapi_data".to_string(),
            mod_id: Some(nexus_id.to_string()),
        };
    }

    if let Some(name) = mod_name {
        if let Some(nexus_id) = BUILTIN_DICT.get(name) {
            let url = format!("https://www.nexusmods.com/stardewvalley/mods/{}", nexus_id);
            return NexusLinkResult {
                url,
                method: "builtin_dict_name".to_string(),
                mod_id: Some(nexus_id.to_string()),
            };
        }

        if let Some(nexus_id) = crate::smapi_data::get_nexus_id_by_name(name) {
            let url = format!("https://www.nexusmods.com/stardewvalley/mods/{}", nexus_id);
            return NexusLinkResult {
                url,
                method: "smapi_data_name".to_string(),
                mod_id: Some(nexus_id.to_string()),
            };
        }
    }

    let search_name = mod_name.unwrap_or(unique_id);
    let decoded_name = urlencoding::decode(search_name)
        .unwrap_or_else(|_| search_name.into());
    let search_url = format!(
        "https://www.nexusmods.com/stardewvalley/mods/search?search={}",
        urlencoding::encode(&decoded_name)
    );
    NexusLinkResult {
        url: search_url,
        method: "search".to_string(),
        mod_id: None,
    }
}

#[tauri::command]
pub fn get_nexus_link(unique_id: String, mod_name: Option<String>, nexus_mod_id: Option<u64>) -> NexusLinkResult {
    build_nexus_link(&unique_id, mod_name.as_deref(), nexus_mod_id)
}
