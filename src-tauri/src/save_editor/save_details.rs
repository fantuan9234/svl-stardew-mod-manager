use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SaveDetailedInfo {
    pub character_name: String,
    pub farm_name: String,
    pub game_version: String,
    pub farm_type: String,
    pub farm_type_id: i32,

    pub day_of_month: u32,
    pub current_season: String,
    pub year: u32,
    pub time_of_day: u32,
    pub days_played: u64,

    pub money: i64,
    pub total_money_earned: i64,
    pub gold: i64,

    pub health: i32,
    pub max_health: i32,
    pub stamina: i32,
    pub max_stamina: i32,

    pub deepest_mine_level: i32,
    pub deepest_skull_cavern_level: i32,
    pub grandpa_score: i32,
    pub has_finished_community_center: bool,
    pub has_joja_mart_run: bool,
    pub ginger_island_unlocked: bool,
    pub stardrops_found: i32,
    pub perfection_score: i32,
    pub perfection_waivers: i32,
    pub activated_golden_parrot: bool,
    pub treasure_totems_used: i32,
    pub times_fed_raccoons: i32,

    pub spouse: String,
    pub is_married: bool,
    pub friendship_count: usize,
    pub max_friendship_points: i32,
    pub max_friendship_npc: String,

    pub farming_level: i32,
    pub mining_level: i32,
    pub foraging_level: i32,
    pub fishing_level: i32,
    pub combat_level: i32,
    pub total_skill_levels: i32,

    pub building_count: usize,
    pub cabin_count: usize,
    pub item_count: usize,
    pub quest_count: usize,
    pub completed_quest_count: usize,
    pub cooking_recipes: usize,
    pub crafting_recipes: usize,
    pub recipes_known: usize,

    pub file_size_bytes: u64,
    pub raw_xml_size: usize,
}

pub fn parse_detailed_save<P: AsRef<Path>>(folder_path: P) -> Result<SaveDetailedInfo, String> {
    let folder_path = folder_path.as_ref();
    let main_save_path = locate_main_save_file(folder_path)?;
    let content = std::fs::read_to_string(&main_save_path)
        .map_err(|e| format!("Failed to read save: {}", e))?;
    let raw_xml_size = content.len();

    let player_start = find_block_start(&content, "player");
    let (mut info, player_block) = if let Some(ps) = player_start {
        let player_end = find_block_end(&content, ps, "player");
        let block = player_end
            .map(|end| content[ps..end].to_string())
            .unwrap_or_default();
        (parse_from_xml(&content, &block), block)
    } else {
        (parse_from_xml(&content, ""), String::new())
    };

    if let Ok(meta) = std::fs::metadata(folder_path) {
        info.file_size_bytes = get_dir_size(folder_path);
        let _ = meta.len();
    }
    info.raw_xml_size = raw_xml_size;

    let _ = player_block;
    Ok(info)
}

fn locate_main_save_file(folder_path: &Path) -> Result<std::path::PathBuf, String> {
    let folder_name = folder_path
        .file_name()
        .ok_or_else(|| "Cannot get folder name".to_string())?
        .to_string_lossy()
        .to_string();
    let entries = std::fs::read_dir(folder_path).map_err(|e| e.to_string())?;
    let paths: Vec<_> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect();
    for p in &paths {
        if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
            if name == folder_name {
                return Ok(p.clone());
            }
        }
    }
    for p in &paths {
        if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
            if name != "SaveGameInfo" && name.ends_with(&folder_name) {
                return Ok(p.clone());
            }
        }
    }
    Err("Main save file not found".to_string())
}

fn parse_from_xml(xml: &str, _player_block: &str) -> SaveDetailedInfo {
    let mut info = SaveDetailedInfo::default();

    info.character_name = find_tag(xml, "name").unwrap_or_default();
    info.farm_name = find_tag(xml, "farmName").unwrap_or_default();
    info.game_version = find_tag(xml, "gameVersion").unwrap_or_default();

    let farm_id_str = find_tag(xml, "whichFarm").unwrap_or_else(|| "0".to_string());
    info.farm_type_id = farm_id_str.parse().unwrap_or(0);
    info.farm_type = farm_type_name(info.farm_type_id).to_string();

    info.day_of_month = find_tag(xml, "dayOfMonth")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    info.current_season = find_tag(xml, "currentSeason").unwrap_or_default();
    info.year = find_tag(xml, "year").and_then(|s| s.parse().ok()).unwrap_or(0);
    info.time_of_day = find_tag(xml, "timeOfDay")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    info.days_played = find_tag(xml, "stats_DaysPlayed")
        .or_else(|| find_tag(xml, "DaysPlayed"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    info.money = find_tag(xml, "money")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    info.total_money_earned = find_tag(xml, "totalMoneyEarned")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    info.gold = info.money;

    info.health = find_tag_in_block(xml, "player", "health")
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    info.max_health = find_tag_in_block(xml, "player", "maxHealth")
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    info.stamina = find_tag_in_block(xml, "player", "stamina")
        .and_then(|s| s.parse().ok())
        .unwrap_or(270);
    info.max_stamina = find_tag_in_block(xml, "player", "maxStamina")
        .and_then(|s| s.parse().ok())
        .unwrap_or(270);

    info.farming_level = find_tag_in_block(xml, "player", "farmingLevel")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    info.mining_level = find_tag_in_block(xml, "player", "miningLevel")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    info.foraging_level = find_tag_in_block(xml, "player", "foragingLevel")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    info.fishing_level = find_tag_in_block(xml, "player", "fishingLevel")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    info.combat_level = find_tag_in_block(xml, "player", "combatLevel")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    info.total_skill_levels = info.farming_level
        + info.mining_level
        + info.foraging_level
        + info.fishing_level
        + info.combat_level;

    info.deepest_mine_level = find_tag(xml, "deepestMineLevel")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    info.deepest_skull_cavern_level = find_tag(xml, "deepestInSkullCavern")
        .or_else(|| find_tag(xml, "deepestSkullCavernLevel"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    info.grandpa_score = find_tag(xml, "grandpaScore")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    info.has_finished_community_center = find_tag(xml, "hasFinishedCommunityCenter")
        .map(|s| s == "true")
        .unwrap_or(false);
    info.has_joja_mart_run = find_tag(xml, "hasJojaRun")
        .map(|s| s == "true")
        .unwrap_or(false);
    info.activated_golden_parrot = find_tag(xml, "activatedGoldenParrot")
        .map(|s| s == "true")
        .unwrap_or(false);
    info.treasure_totems_used = find_tag(xml, "treasureTotemsUsed")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    info.times_fed_raccoons = find_tag(xml, "timesFedRaccoons")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    info.perfection_waivers = find_tag(xml, "perfectionWaivers")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    info.perfection_score = find_tag(xml, "perfectionScore")
        .or_else(|| find_tag(xml, "stats_PerfectionScore"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    info.stardrops_found = count_mail_flags_containing(xml, "Stardrop");

    info.ginger_island_unlocked = !find_tag(xml, "islandWest").unwrap_or_default().is_empty()
        || info.perfection_score > 0
        || !find_tag(xml, "mailReceived")
            .unwrap_or_default()
            .contains("Island_Resort")
        && find_tag(xml, "hasFinishedCommunityCenter")
            .map(|s| s == "true")
            .unwrap_or(false)
        || !find_tag(xml, "gingerIslandFarmhouseFixed")
            .unwrap_or_default()
            .is_empty()
        || find_tag(xml, "stats_ItemsCooked")
            .and_then(|s| s.parse::<i32>().ok())
            .map(|_| info.perfection_score)
            .unwrap_or(0)
            > 0
        && !find_tag(xml, "raccoonBundles")
            .unwrap_or_default()
            .is_empty();

    let spouse_name = find_tag_in_block(xml, "player", "spouse").unwrap_or_default();
    info.spouse = spouse_name;
    info.is_married = !info.spouse.is_empty()
        && info.spouse != "null"
        && info.spouse.to_lowercase() != "false"
        && info.spouse != "0";

    let (friendship_count, max_points, max_npc) = parse_friendships(xml);
    info.friendship_count = friendship_count;
    info.max_friendship_points = max_points;
    info.max_friendship_npc = max_npc;

    info.building_count = count_occurrences(xml, "<Building>");
    info.cabin_count = count_occurrences(xml, "Cabin");

    info.item_count = count_items(xml);

    let (quest_count, completed) = parse_quests(xml);
    info.quest_count = quest_count;
    info.completed_quest_count = completed;

    let (cooking, crafting) = parse_recipes(xml);
    info.cooking_recipes = cooking;
    info.crafting_recipes = crafting;
    info.recipes_known = cooking + crafting;

    info
}

fn farm_type_name(id: i32) -> &'static str {
    match id {
        0 => "Standard",
        1 => "Riverland",
        2 => "Forest",
        3 => "Hill-top",
        4 => "Wilderness",
        5 => "Four Corners",
        6 => "Beach",
        _ => "Custom",
    }
}

fn find_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let with_attr = format!("<{} ", tag);
    let start_rel = xml.find(&open).or_else(|| xml.find(&with_attr))?;
    let value_start = if xml[start_rel..].starts_with(&with_attr) {
        let gt = xml[start_rel..].find('>')?;
        start_rel + gt + 1
    } else {
        start_rel + open.len()
    };
    let close = format!("</{}>", tag);
    let end = xml[value_start..].find(&close)? + value_start;
    Some(xml[value_start..end].to_string())
}

fn find_tag_in_block(xml: &str, block: &str, tag: &str) -> Option<String> {
    let block_start = find_block_start(xml, block)?;
    let block_end = find_block_end(xml, block_start, block)?;
    let block_content = &xml[block_start..block_end];
    find_tag(block_content, tag)
}

fn find_block_start(xml: &str, block: &str) -> Option<usize> {
    let plain = format!("<{}>", block);
    let with_attr = format!("<{} ", block);
    let p = xml.find(&plain);
    let a = xml.find(&with_attr);
    match (p, a) {
        (Some(x), Some(y)) => Some(x.min(y)),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        (None, None) => None,
    }
}

fn find_block_end(xml: &str, start: usize, block: &str) -> Option<usize> {
    let close = format!("</{}>", block);
    let end = xml[start..].find(&close)? + start;
    Some(end)
}

fn count_occurrences(xml: &str, needle: &str) -> usize {
    let mut count = 0;
    let mut pos = 0;
    while let Some(idx) = xml[pos..].find(needle) {
        count += 1;
        pos += idx + needle.len();
    }
    count
}

fn count_items(xml: &str) -> usize {
    let upper = count_occurrences(xml, "<Item>");
    let lower = count_occurrences(xml, "<item>");
    upper + lower
}

fn count_mail_flags_containing(xml: &str, contains: &str) -> i32 {
    if let Some(mail) = find_tag(xml, "mailReceived") {
        let flags: Vec<&str> = mail
            .split(">")
            .filter(|s| s.contains(contains))
            .collect();
        flags.len() as i32
    } else {
        0
    }
}

fn parse_friendships(xml: &str) -> (usize, i32, String) {
    let mut count = 0;
    let mut max_points = 0;
    let mut max_npc = String::new();

    let start = match find_block_start(xml, "friendshipData") {
        Some(s) => s,
        None => return (0, 0, String::new()),
    };
    let end = match find_block_end(xml, start, "friendshipData") {
        Some(e) => e,
        None => return (0, 0, String::new()),
    };
    let block = &xml[start..end];

    let mut cursor = 0;
    while let Some(idx) = block[cursor..].find("<item>") {
        let abs = cursor + idx + "<item>".len();
        if let Some(end_rel) = block[abs..].find("</item>") {
            let inner = &block[abs..abs + end_rel];
            let npc = find_tag(inner, "string").unwrap_or_default();
            let points = find_tag(inner, "Points")
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);
            if !npc.is_empty() {
                count += 1;
                if points > max_points {
                    max_points = points;
                    max_npc = npc;
                }
            }
            cursor = abs + end_rel + "</item>".len();
        } else {
            break;
        }
    }
    (count, max_points, max_npc)
}

fn parse_quests(xml: &str) -> (usize, usize) {
    let start = match find_block_start(xml, "questLog") {
        Some(s) => s,
        None => return (0, 0),
    };
    let end = match find_block_end(xml, start, "questLog") {
        Some(e) => e,
        None => return (0, 0),
    };
    let block = &xml[start..end];

    let quest_open = "<Quest";
    let mut total = 0;
    let mut completed = 0;
    let mut pos = 0;
    while let Some(idx) = block[pos..].find(quest_open) {
        total += 1;
        let abs = pos + idx;
        if let Some(end_rel) = block[abs..].find(">") {
            let attr_end = abs + end_rel;
            if let Some(close_rel) = block[attr_end..].find("</Quest>") {
                let inner = &block[attr_end + 1..attr_end + close_rel];
                if find_tag(inner, "completed")
                    .map(|s| s == "true")
                    .unwrap_or(false)
                {
                    completed += 1;
                }
                pos = attr_end + close_rel + "</Quest>".len();
            } else {
                break;
            }
        } else {
            break;
        }
    }
    (total, completed)
}

fn parse_recipes(xml: &str) -> (usize, usize) {
    let cooking = count_in_vector_block(xml, "cookingRecipes");
    let crafting = count_in_vector_block(xml, "craftingRecipes");
    (cooking, crafting)
}

fn count_in_vector_block(xml: &str, block_name: &str) -> usize {
    let start = match find_block_start(xml, block_name) {
        Some(s) => s,
        None => return 0,
    };
    let end = match find_block_end(xml, start, block_name) {
        Some(e) => e,
        None => return 0,
    };
    let block = &xml[start..end];
    count_occurrences(block, "<item>")
}

fn get_dir_size(path: &Path) -> u64 {
    let mut size = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                size += get_dir_size(&p);
            } else if let Ok(meta) = p.metadata() {
                size += meta.len();
            }
        }
    }
    size
}

pub fn format_play_time(seconds: u64) -> String {
    if seconds == 0 {
        return "0h".to_string();
    }
    let total_hours = seconds / 3600;
    if total_hours < 1 {
        let m = seconds / 60;
        return format!("{}m", m);
    }
    let days = total_hours / 24;
    let hours = total_hours % 24;
    let minutes = (seconds % 3600) / 60;
    if days > 0 {
        if hours > 0 {
            format!("{}d {}h", days, hours)
        } else {
            format!("{}d", days)
        }
    } else if minutes > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}h", hours)
    }
}

pub fn format_money(amount: i64) -> String {
    let abs = amount.unsigned_abs();
    if abs >= 1_000_000 {
        format!("{:.2}M", amount as f64 / 1_000_000.0)
    } else if abs >= 10_000 {
        format!("{:.1}K", amount as f64 / 1_000.0)
    } else {
        amount.to_string()
    }
}

pub fn format_size_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn read_real_save() -> Option<String> {
        let path = r"d:\stardew mod mannager\stardew-mod-manager\114514_440125852\114514_440125852";
        fs::read_to_string(path).ok()
    }

    #[test]
    fn test_farm_type_name() {
        assert_eq!(farm_type_name(0), "Standard");
        assert_eq!(farm_type_name(1), "Riverland");
        assert_eq!(farm_type_name(2), "Forest");
        assert_eq!(farm_type_name(3), "Hill-top");
        assert_eq!(farm_type_name(4), "Wilderness");
        assert_eq!(farm_type_name(5), "Four Corners");
        assert_eq!(farm_type_name(6), "Beach");
        assert_eq!(farm_type_name(99), "Custom");
    }

    #[test]
    fn test_find_tag() {
        let xml = r#"<root><name>Test</name><gameVersion>1.6.15</gameVersion></root>"#;
        assert_eq!(find_tag(xml, "name").unwrap(), "Test");
        assert_eq!(find_tag(xml, "gameVersion").unwrap(), "1.6.15");
        assert!(find_tag(xml, "missing").is_none());
    }

    #[test]
    fn test_find_tag_with_attr() {
        let xml = r#"<Quest xsi:type="SocializeQuest"><id>9</id></Quest>"#;
        let inner = &xml[xml.find(">").unwrap() + 1..xml.rfind("</").unwrap()];
        let id = find_tag(inner, "id").unwrap();
        assert_eq!(id, "9");
    }

    #[test]
    fn test_count_occurrences() {
        let xml = "<a><b/><b/><b/></a>";
        assert_eq!(count_occurrences(xml, "<b/>"), 3);
    }

    #[test]
    fn test_format_money() {
        assert_eq!(format_money(500), "500");
        assert_eq!(format_money(15_000), "15.0K");
        assert_eq!(format_money(1_500_000), "1.50M");
    }

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size_bytes(500), "500 B");
        assert_eq!(format_size_bytes(2048), "2.0 KB");
        assert_eq!(format_size_bytes(5 * 1024 * 1024), "5.0 MB");
    }

    #[test]
    fn test_parse_real_save() {
        if let Some(content) = read_real_save() {
            assert!(!content.is_empty());
            let game_version = find_tag(&content, "gameVersion");
            assert_eq!(game_version, Some("1.6.15".to_string()));

            let which_farm = find_tag(&content, "whichFarm");
            assert_eq!(which_farm, Some("0".to_string()));

            let money = find_tag(&content, "money");
            assert_eq!(money, Some("114514".to_string()));
        }
    }

    #[test]
    fn test_parse_friendships() {
        let xml = r#"<root><friendshipData><item><string>Abigail</string><Points>250</Points></item><item><string>Haley</string><Points>1000</Points></item></friendshipData></root>"#;
        let (count, max, npc) = parse_friendships(xml);
        assert_eq!(count, 2);
        assert_eq!(max, 1000);
        assert_eq!(npc, "Haley");
    }

    #[test]
    fn test_parse_quests() {
        let xml = r#"<root><questLog><Quest><id>1</id><completed>true</completed></Quest><Quest><id>2</id><completed>false</completed></Quest><Quest><id>3</id><completed>true</completed></Quest></questLog></root>"#;
        let (total, completed) = parse_quests(xml);
        assert_eq!(total, 3);
        assert_eq!(completed, 2);
    }

    #[test]
    fn test_parse_recipes() {
        let xml = r#"<root><cookingRecipes><item><key>0</key><value>true</value></item><item><key>1</key><value>true</value></item></cookingRecipes><craftingRecipes><item><key>0</key><value>true</value></item></craftingRecipes></root>"#;
        let (cooking, crafting) = parse_recipes(xml);
        assert_eq!(cooking, 2);
        assert_eq!(crafting, 1);
    }
}
