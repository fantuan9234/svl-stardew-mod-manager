use crate::save_editor::error::{Result, SaveEditorError};
use crate::save_editor::CharacterInfo;

pub fn parse_character(xml: &str) -> Result<CharacterInfo> {
    Ok(CharacterInfo {
        name: find_in_player(xml, "name").ok_or_else(|| {
            SaveEditorError::InvalidStructure("missing <name>".to_string())
        })?,
        farm_name: find_in_player(xml, "farmName").unwrap_or_default(),
        money: find_in_player(xml, "money")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        health: find_in_player(xml, "health")
            .and_then(|s| s.parse().ok())
            .unwrap_or(100),
        max_health: find_in_player(xml, "maxHealth")
            .and_then(|s| s.parse().ok())
            .unwrap_or(100),
        stamina: find_in_player(xml, "stamina")
            .and_then(|s| s.parse().ok())
            .unwrap_or(270),
        max_stamina: find_in_player(xml, "maxStamina")
            .and_then(|s| s.parse().ok())
            .unwrap_or(270),
        day_of_month: find_in_player(xml, "dayOfMonth")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1),
        current_season: find_in_player(xml, "currentSeason").unwrap_or_else(|| "spring".to_string()),
        year: find_in_player(xml, "year")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1),
        time_of_day: find_in_player(xml, "timeOfDay")
            .and_then(|s| s.parse().ok())
            .unwrap_or(600),
    })
}

pub fn apply_character_edits(xml: &str, info: &CharacterInfo) -> Result<String> {
    let mut result = xml.to_string();
    result = replace_in_player(&result, "name", &info.name)?;
    result = replace_in_player(&result, "farmName", &info.farm_name)?;
    result = replace_in_player(&result, "money", &info.money.to_string())?;
    result = replace_in_player(&result, "health", &info.health.to_string())?;
    result = replace_in_player(&result, "maxHealth", &info.max_health.to_string())?;
    result = replace_in_player(&result, "stamina", &info.stamina.to_string())?;
    result = replace_in_player(&result, "maxStamina", &info.max_stamina.to_string())?;
    result = replace_in_player(&result, "dayOfMonth", &info.day_of_month.to_string())?;
    result = replace_in_player(&result, "currentSeason", &info.current_season)?;
    result = replace_in_player(&result, "year", &info.year.to_string())?;
    result = replace_in_player(&result, "timeOfDay", &info.time_of_day.to_string())?;
    Ok(result)
}

fn find_in_player(xml: &str, tag: &str) -> Option<String> {
    let player_start = xml.find("<player>").or_else(|| {
        let p = xml.find("<player ")?;
        xml[p..].find('>').map(|o| p + o + 1)
    })?;
    let player_end_rel = xml[player_start..].find("</player>")?;
    let player_end = player_start + player_end_rel;
    let player_block = &xml[player_start..player_end];

    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let tag_start = player_block.find(&open)? + open.len();
    let tag_end = player_block[tag_start..].find(&close)? + tag_start;
    Some(player_block[tag_start..tag_end].to_string())
}

fn replace_in_player(xml: &str, tag: &str, value: &str) -> Result<String> {
    let player_start = xml
        .find("<player>")
        .or_else(|| {
            let p = xml.find("<player ")?;
            xml[p..].find('>').map(|o| p + o + 1)
        })
        .ok_or_else(|| {
            SaveEditorError::InvalidStructure("No <player> block found".to_string())
        })?;
    let player_end_rel = xml[player_start..].find("</player>").ok_or_else(|| {
        SaveEditorError::InvalidStructure("Unclosed <player> block".to_string())
    })?;
    let player_end = player_start + player_end_rel;

    let before = &xml[..player_start];
    let player_block = &xml[player_start..player_end];
    let after = &xml[player_end..];

    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);

    let new_player_block = if let Some(local_start) = player_block.find(&open) {
        let abs_start = local_start + open.len();
        if let Some(end_off) = player_block[abs_start..].find(&close) {
            let mut new_block = String::with_capacity(player_block.len());
            new_block.push_str(&player_block[..abs_start]);
            new_block.push_str(&escape_xml_text(value));
            new_block.push_str(&player_block[abs_start + end_off..]);
            new_block
        } else {
            return Err(SaveEditorError::InvalidStructure(format!(
                "Unclosed <{}> inside <player>",
                tag
            )));
        }
    } else {
        let mut new_block = String::with_capacity(
            player_block.len() + tag.len() * 2 + value.len() + 5,
        );
        new_block.push_str(player_block);
        new_block.push_str(&format!(
            "\n<{}>{}</{}>",
            tag,
            escape_xml_text(value),
            tag
        ));
        new_block
    };

    Ok(format!("{}{}{}", before, new_player_block, after))
}

fn escape_xml_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_XML: &str = r#"<SaveGame>
<player>
<name>TestFarmer</name>
<farmName>TestFarm</farmName>
<money>5000</money>
<maxHealth>100</maxHealth>
<health>80</health>
<maxStamina>270</maxStamina>
<stamina>200</stamina>
<dayOfMonth>15</dayOfMonth>
<currentSeason>summer</currentSeason>
<year>2</year>
<timeOfDay>1200</timeOfDay>
</player>
</SaveGame>"#;

    #[test]
    fn test_parse_character_info() {
        let info = parse_character(SAMPLE_XML).unwrap();
        assert_eq!(info.name, "TestFarmer");
        assert_eq!(info.farm_name, "TestFarm");
        assert_eq!(info.money, 5000);
        assert_eq!(info.max_health, 100);
        assert_eq!(info.health, 80);
        assert_eq!(info.stamina, 200);
        assert_eq!(info.max_stamina, 270);
        assert_eq!(info.day_of_month, 15);
        assert_eq!(info.current_season, "summer");
        assert_eq!(info.year, 2);
        assert_eq!(info.time_of_day, 1200);
    }

    #[test]
    fn test_apply_character_edits() {
        let mut info = parse_character(SAMPLE_XML).unwrap();
        info.money = 99999;
        info.name = "Renamed".to_string();
        info.day_of_month = 28;
        info.current_season = "winter".to_string();
        info.time_of_day = 2400;
        let updated = apply_character_edits(SAMPLE_XML, &info).unwrap();
        assert!(updated.contains("<money>99999</money>"));
        assert!(updated.contains("<name>Renamed</name>"));
        assert!(updated.contains("<farmName>TestFarm</farmName>"));
        assert!(updated.contains("<dayOfMonth>28</dayOfMonth>"));
        assert!(updated.contains("<currentSeason>winter</currentSeason>"));
        assert!(updated.contains("<timeOfDay>2400</timeOfDay>"));
    }
}
