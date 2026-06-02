pub mod error;
pub mod xml_utils;
pub mod save_file;
pub mod character;
pub mod skills;

use error::Result;
use save_file::SaveFile;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSummary {
    pub folder_path: String,
    pub character_name: String,
    pub farm_name: String,
    pub money: i64,
    pub current_date: String,
    pub play_time_hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterInfo {
    pub name: String,
    pub farm_name: String,
    pub money: i64,
    pub health: i32,
    pub max_health: i32,
    pub stamina: i32,
    pub max_stamina: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub level: i32,
    pub experience: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSet {
    pub skills: Vec<SkillInfo>,
}

#[tauri::command]
pub fn open_save_in_editor(save_path: String) -> std::result::Result<SaveSummary, String> {
    let folder = PathBuf::from(&save_path);
    let save = SaveFile::load(&folder).map_err(|e| e.to_string())?;
    Ok(SaveSummary {
        folder_path: save.folder_path.to_string_lossy().to_string(),
        character_name: save.character_name.clone(),
        farm_name: xml_utils::find_tag_value(&save.raw_xml, "farmName").unwrap_or_default(),
        money: xml_utils::find_tag_value(&save.raw_xml, "money")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        current_date: extract_current_date(&save.raw_xml),
        play_time_hours: 0,
    })
}

fn extract_current_date(xml: &str) -> String {
    let day = xml_utils::find_tag_value(xml, "dayOfMonth").unwrap_or_default();
    let season = xml_utils::find_tag_value(xml, "currentSeason").unwrap_or_default();
    let year = xml_utils::find_tag_value(xml, "year").unwrap_or_default();
    if day.is_empty() {
        String::new()
    } else {
        format!("Y{} {} D{}", year, season, day)
    }
}

#[tauri::command]
pub fn save_editor_load_character(
    save_path: String,
) -> std::result::Result<CharacterInfo, String> {
    let save = SaveFile::load(&PathBuf::from(&save_path)).map_err(|e| e.to_string())?;
    character::parse_character(&save.raw_xml).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_editor_save_character(
    save_path: String,
    info: CharacterInfo,
) -> std::result::Result<String, String> {
    let mut save = SaveFile::load(&PathBuf::from(&save_path)).map_err(|e| e.to_string())?;
    let backup_path = save.backup().map_err(|e| e.to_string())?;
    let new_xml = character::apply_character_edits(&save.raw_xml, &info)
        .map_err(|e| e.to_string())?;
    save.set_xml(new_xml);
    save.write().map_err(|e| e.to_string())?;
    Ok(backup_path.to_string_lossy().to_string())
}
