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
        m.insert("Aerin.MoreFish", 2413);
        m.insert("Tanpoponoko.SeasonalOutfits", 10220);
        m.insert("Rose.craftables", 11062);
        m.insert("Pathoschild.Automate", 1063);
        m.insert("CantorsDust.LookupAnything", 541);
        m.insert("Pathoschild.TractorMod", 1401);
        m.insert("Pathoschild.ChestsAnywhere", 518);
        m.insert("Bouhm.NpcMapLocations", 239);
        m.insert("MizzionRpg.ProjectR", 11311);
        m.insert("furyx639.BetterFarmAnimalVariety", 3296);
        m.insert("ConsoleCommands", 2400);
        m.insert("SaveBackup", 2400);
        m.insert("PurplingCat.QuestFramework", 6414);
        m.insert("QuestFramework", 6414);
        m.insert("Quest Framework", 6414);
        m.insert("PurplingCat.NPCAdventures", 4582);
        m.insert("NPC Adventures", 4582);
        m.insert("Pahimabata.DistantLands", 11748);
        m.insert("Distant Lands", 11748);
        m.insert("BBR.VanillaProfessionsRevised", 15458);
        m.insert("Vanilla Professions Revised", 15458);
        m.insert("leclair.gamemode", 12796);
        m.insert("Game Mode Mod", 12796);
        m.insert("Miguel.GiftTasteHelper", 7746);
        m.insert("Gift Taste Helper", 7746);
        m.insert("Bouhm.NightOwl", 1222);
        m.insert("Night Owl", 1222);
        m.insert("Platonymous.Toolkit", 10533);
        m.insert("Platonymous.ModSettings", 10533);
        m.insert("PeacefulEnd.Core", 20078);
        m.insert("PeacefulEnd.FashionSense", 9960);
        m.insert("PeacefulEnd.VibrantPastoralRecolour", 13794);
        m.insert("DaisyNiko.Tilesheets", 14687);
        m.insert("Entoarox.FasterPathing", 1852);
        m.insert("Faster Pathing", 1852);
        m.insert("Bouhm.StardewCrops", 2425);
        m.insert("spacechase0.MachineControlPanel", 16243);
        m.insert("spacechase0.SkillLevelCodes", 15530);
        m.insert("spacechase0.BetterMeteorShowers", 15530);
        m.insert("Aedenthorn.Climbing", 16187);
        m.insert("Aedenthorn.JojaBank", 15798);
        m.insert("Aedenthorn.NoclipMode", 15465);
        m.insert("Aedenthorn.PredictiveMods", 15465);
        m.insert("Teh.PersianFarms", 8175);
        m.insert("Teh.BetterRanching", 8526);
        m.insert("Better Ranching", 8526);
        m.insert("Teh.FestivalOverhaul", 6273);
        m.insert("Festival Overhaul", 6273);
        m.insert("Teh.Core", 9509);
        m.insert("YTSC.WinterStarExterior", 14566);
        m.insert("LemurKat.MultipleSpouses", 12004);
        m.insert("Multiple Spouses", 12004);
        m.insert("Entoarox.AdvancedLocationLoader", 2270);
        m.insert("Advanced Location Loader", 2270);
        m.insert("Entoarox.EntoaroxFramework", 2270);
        m.insert("Alphablackdisco.WalkOfLife", 8304);
        m.insert("Walk of Life", 8304);
        m.insert("Mushymo.PonyRider", 10979);
        m.insert("Pony Rider", 10979);
        m.insert("Mushymo.RanchingTool", 10979);
        m.insert("FlashShifter.SVECode", 3753);
        m.insert("FlashShifter.SVEMap", 3753);
        m.insert("FlashShifter.SVENPC", 3753);
        m.insert("FlashShifter.SVEEvent", 3753);
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
        m.insert("QuestFramework", 6414);
        m.insert("Quest Framework", 6414);
        m.insert("NPC Adventures", 4582);
        m.insert("Distant Lands", 11748);
        m.insert("Vanilla Professions Revised", 15458);
        m.insert("Game Mode Mod", 12796);
        m.insert("Gift Taste Helper", 7746);
        m.insert("Night Owl", 1222);
        m.insert("Faster Pathing", 1852);
        m.insert("Better Ranching", 8526);
        m.insert("Festival Overhaul", 6273);
        m.insert("Multiple Spouses", 12004);
        m.insert("Advanced Location Loader", 2270);
        m.insert("Walk of Life", 8304);
        m.insert("Pony Rider", 10979);
        m
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexusLinkResult {
    pub url: String,
    pub method: String,
    pub mod_id: Option<String>,
}

fn builtin_dict_lookup(key: &str) -> Option<u64> {
    BUILTIN_DICT
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(key))
        .map(|(_, &v)| v)
}

fn folder_name_dict_lookup(key: &str) -> Option<u64> {
    FOLDER_NAME_DICT
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(key))
        .map(|(_, &v)| v)
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

    if let Some(nexus_id) = builtin_dict_lookup(unique_id) {
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
        if let Some(nexus_id) = builtin_dict_lookup(name) {
            let url = format!("https://www.nexusmods.com/stardewvalley/mods/{}", nexus_id);
            return NexusLinkResult {
                url,
                method: "builtin_dict_name".to_string(),
                mod_id: Some(nexus_id.to_string()),
            };
        }

        if let Some(nexus_id) = folder_name_dict_lookup(name) {
            let url = format!("https://www.nexusmods.com/stardewvalley/mods/{}", nexus_id);
            return NexusLinkResult {
                url,
                method: "folder_name_dict".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_nexus_link_builtin_dict_exact_match() {
        let result = build_nexus_link("Pathoschild.ContentPatcher", None, None);
        assert_eq!(result.url, "https://www.nexusmods.com/stardewvalley/mods/1915");
        assert_eq!(result.method, "builtin_dict");
    }

    #[test]
    fn test_build_nexus_link_builtin_dict_case_insensitive() {
        let result = build_nexus_link("pathoschild.contentpatcher", None, None);
        assert_eq!(result.url, "https://www.nexusmods.com/stardewvalley/mods/1915",
            "Should find mod with case-insensitive match");
    }

    #[test]
    fn test_build_nexus_link_nexus_mod_id_priority() {
        let result = build_nexus_link("SomeMod", None, Some(9999));
        assert_eq!(result.url, "https://www.nexusmods.com/stardewvalley/mods/9999");
        assert_eq!(result.method, "manifest_update_keys");
    }

    #[test]
    fn test_build_nexus_link_search_fallback() {
        let result = build_nexus_link("NonExistent.Mod", Some("Some Weird Mod"), None);
        assert!(result.url.contains("search"));
        assert_eq!(result.method, "search");
    }

    #[test]
    fn test_build_nexus_link_mod_name_lookup() {
        let result = build_nexus_link("NonExistent.ID", Some("Content Patcher"), None);
        assert_eq!(result.url, "https://www.nexusmods.com/stardewvalley/mods/1915");
        assert_eq!(result.method, "folder_name_dict");
    }
}
