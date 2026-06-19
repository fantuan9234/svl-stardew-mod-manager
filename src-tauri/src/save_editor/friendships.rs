use crate::save_editor::error::{Result, SaveEditorError};
use serde::{Deserialize, Serialize};

const HEART_SIZE: i32 = 250;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendshipInfo {
    pub index: usize,
    pub npc_name: String,
    pub points: i32,
    pub status: String,
    pub gifts_this_week: i32,
    pub gifts_today: i32,
    pub talked_to_today: bool,
    pub proposer: String,
    pub wedding_date: String,
    pub next_anniversary: String,
    pub meeting_since: String,
    pub countdown_to_wedding: i32,
    pub anniversaries: i32,
    pub roommate_marriage: bool,
    pub broken_up: bool,
    pub proposal_rejected: bool,
    pub family_perk: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendshipList {
    pub friendships: Vec<FriendshipInfo>,
}

fn extract_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    Some(xml[start..end].to_string())
}

fn extract_tag_with_xsi(xml: &str, tag: &str) -> Option<String> {
    // 支持 <tag xsi:type="...">value</tag>
    let with_attr_open = format!("<{} ", tag);
    if let Some(start_with_attr) = xml.find(&with_attr_open) {
        let gt_pos = xml[start_with_attr..].find('>')? + start_with_attr;
        let value_start = gt_pos + 1;
        let close = format!("</{}>", tag);
        let end = xml[value_start..].find(&close)? + value_start;
        return Some(xml[value_start..end].to_string());
    }
    extract_tag(xml, tag)
}

fn parse_bool(s: &str) -> bool {
    s == "true" || s == "True" || s == "1"
}

pub fn parse(xml: &str) -> Result<FriendshipList> {
    let fd_start = xml
        .find("<friendshipData>")
        .ok_or_else(|| SaveEditorError::InvalidStructure("missing <friendshipData>".to_string()))?
        + "<friendshipData>".len();
    let fd_end = xml[fd_start..]
        .find("</friendshipData>")
        .ok_or_else(|| SaveEditorError::InvalidStructure("unclosed <friendshipData>".to_string()))?
        + fd_start;
    let fd_block = &xml[fd_start..fd_end];

    let mut friendships = Vec::new();
    let mut cursor = 0;
    let mut index = 0;

    while let Some(item_idx) = fd_block[cursor..].find("<item>") {
        let abs_start = cursor + item_idx + "<item>".len();
        if let Some(item_end) = fd_block[abs_start..].find("</item>") {
            let inner = &fd_block[abs_start..abs_start + item_end];

            let npc_name = extract_tag(inner, "string")
                .or_else(|| extract_tag_with_xsi(inner, "string"))
                .unwrap_or_default();

            friendships.push(FriendshipInfo {
                index,
                npc_name,
                points: extract_tag(inner, "Points")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                status: extract_tag(inner, "Status").unwrap_or_default(),
                gifts_this_week: extract_tag(inner, "GiftsThisWeek")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                gifts_today: extract_tag(inner, "GiftsToday")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                talked_to_today: extract_tag(inner, "TalkedToToday")
                    .map(|s| parse_bool(&s))
                    .unwrap_or(false),
                proposer: extract_tag(inner, "Proposer").unwrap_or_default(),
                wedding_date: extract_tag(inner, "WeddingDate").unwrap_or_default(),
                next_anniversary: extract_tag(inner, "NextAnniversary").unwrap_or_default(),
                meeting_since: extract_tag(inner, "MeetingSince").unwrap_or_default(),
                countdown_to_wedding: extract_tag(inner, "CountdownToWedding")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                anniversaries: extract_tag(inner, "Anniversaries")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                roommate_marriage: extract_tag(inner, "RoommateMarriage")
                    .map(|s| parse_bool(&s))
                    .unwrap_or(false),
                broken_up: extract_tag(inner, "BrokenUp")
                    .map(|s| parse_bool(&s))
                    .unwrap_or(false),
                proposal_rejected: extract_tag(inner, "ProposalRejected")
                    .map(|s| parse_bool(&s))
                    .unwrap_or(false),
                family_perk: extract_tag(inner, "FamilyPerk")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
            });

            index += 1;
            cursor = abs_start + item_end + "</item>".len();
        } else {
            break;
        }
    }

    Ok(FriendshipList { friendships })
}

fn replace_tag_value(xml: &str, tag: &str, val: &str) -> String {
    let open_with_attr = format!("<{} ", tag);
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);

    // 优先替换带属性的版本
    if let Some(start) = xml.find(&open_with_attr) {
        let gt_pos = xml[start..].find('>').unwrap() + start;
        let value_start = gt_pos + 1;
        if let Some(end_rel) = xml[value_start..].find(&close) {
            let end = value_start + end_rel + close.len();
            return format!("{}{}{}{}", &xml[..value_start], val, close, &xml[end..]);
        }
    }
    // 降级使用普通版本
    if let Some(start) = xml.find(&open) {
        let value_start = start + open.len();
        if let Some(end_rel) = xml[value_start..].find(&close) {
            let end = value_start + end_rel + close.len();
            return format!("{}{}{}{}", &xml[..value_start], val, close, &xml[end..]);
        }
    }
    xml.to_string()
}

pub fn apply(xml: &str, list: &FriendshipList) -> Result<String> {
    let fd_start = xml
        .find("<friendshipData>")
        .ok_or_else(|| SaveEditorError::InvalidStructure("missing <friendshipData>".to_string()))?
        + "<friendshipData>".len();
    let fd_end = xml[fd_start..]
        .find("</friendshipData>")
        .ok_or_else(|| SaveEditorError::InvalidStructure("unclosed <friendshipData>".to_string()))?
        + fd_start;
    let fd_block = &xml[fd_start..fd_end];

    let mut item_positions: Vec<(usize, usize)> = Vec::new();
    let mut cursor = 0;
    while let Some(idx) = fd_block[cursor..].find("<item>") {
        let abs = cursor + idx + "<item>".len();
        if let Some(end) = fd_block[abs..].find("</item>") {
            item_positions.push((fd_start + abs, fd_start + abs + end));
            cursor = abs + end + "</item>".len();
        } else {
            break;
        }
    }

    // 从右到左替换避免偏移问题
    let mut updates: Vec<usize> = (0..list.friendships.len()).filter(|i| *i < item_positions.len()).collect();
    updates.sort_by_key(|i| std::cmp::Reverse(*i));

    let mut result = xml.to_string();
    for i in updates {
        let (start, end) = item_positions[i];
        let inner = &result[start..end];
        let mut new_inner = inner.to_string();

        let f = &list.friendships[i];
        new_inner = replace_tag_value(&new_inner, "Points", &f.points.to_string());
        new_inner = replace_tag_value(&new_inner, "Status", &f.status);
        new_inner = replace_tag_value(&new_inner, "GiftsThisWeek", &f.gifts_this_week.to_string());
        new_inner = replace_tag_value(&new_inner, "GiftsToday", &f.gifts_today.to_string());
        new_inner = replace_tag_value(
            &new_inner,
            "TalkedToToday",
            if f.talked_to_today { "true" } else { "false" },
        );
        new_inner = replace_tag_value(&new_inner, "Proposer", &f.proposer);
        new_inner = replace_tag_value(&new_inner, "WeddingDate", &f.wedding_date);
        new_inner = replace_tag_value(&new_inner, "NextAnniversary", &f.next_anniversary);
        new_inner = replace_tag_value(&new_inner, "MeetingSince", &f.meeting_since);
        new_inner = replace_tag_value(&new_inner, "CountdownToWedding", &f.countdown_to_wedding.to_string());
        new_inner = replace_tag_value(&new_inner, "Anniversaries", &f.anniversaries.to_string());
        new_inner = replace_tag_value(
            &new_inner,
            "RoommateMarriage",
            if f.roommate_marriage { "true" } else { "false" },
        );
        new_inner = replace_tag_value(
            &new_inner,
            "BrokenUp",
            if f.broken_up { "true" } else { "false" },
        );
        new_inner = replace_tag_value(
            &new_inner,
            "ProposalRejected",
            if f.proposal_rejected { "true" } else { "false" },
        );
        new_inner = replace_tag_value(&new_inner, "FamilyPerk", &f.family_perk.to_string());

        result.replace_range(start..end, &new_inner);
    }

    Ok(result)
}

pub fn hearts_from_points(points: i32) -> i32 {
    points / HEART_SIZE
}

pub fn points_from_hearts(hearts: i32) -> i32 {
    hearts * HEART_SIZE
}

pub fn max_hearts_for_status(status: &str) -> i32 {
    match status {
        "Married" => 14,
        "Dating" => 11,
        _ => 10,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_XML: &str = r#"<SaveGame>
<player>
<friendshipData>
<item><key><string>Abigail</string></key><value><Friendship><Points>1250</Points><Status>Dating</Status><GiftsThisWeek>1</GiftsThisWeek><GiftsToday>0</GiftsToday><TalkedToToday>true</TalkedToToday><Proposer></Proposer><WeddingDate>0</WeddingDate><NextAnniversary>0</NextAnniversary><MeetingSince>0</MeetingSince><CountdownToWedding>0</CountdownToWedding><Anniversaries>0</Anniversaries><RoommateMarriage>false</RoommateMarriage><BrokenUp>false</BrokenUp><ProposalRejected>false</ProposalRejected><FamilyPerk>0</FamilyPerk></Friendship></value></item>
<item><key><string>Pierre</string></key><value><Friendship><Points>500</Points><Status>Friendly</Status><GiftsThisWeek>0</GiftsThisWeek><GiftsToday>0</GiftsToday><TalkedToToday>false</TalkedToToday><Proposer></Proposer><WeddingDate>0</WeddingDate><NextAnniversary>0</NextAnniversary><MeetingSince>0</MeetingSince><CountdownToWedding>0</CountdownToWedding><Anniversaries>0</Anniversaries><RoommateMarriage>false</RoommateMarriage><BrokenUp>false</BrokenUp><ProposalRejected>false</ProposalRejected><FamilyPerk>0</FamilyPerk></Friendship></value></item>
</friendshipData>
</player>
</SaveGame>"#;

    #[test]
    fn test_parse_friendships() {
        let list = parse(SAMPLE_XML).unwrap();
        assert_eq!(list.friendships.len(), 2);
        assert_eq!(list.friendships[0].npc_name, "Abigail");
        assert_eq!(list.friendships[0].points, 1250);
        assert_eq!(list.friendships[0].status, "Dating");
        assert!(list.friendships[0].talked_to_today);
        assert_eq!(list.friendships[1].npc_name, "Pierre");
        assert_eq!(list.friendships[1].points, 500);
        assert!(!list.friendships[1].talked_to_today);
    }

    #[test]
    fn test_apply_friendships() {
        let mut list = parse(SAMPLE_XML).unwrap();
        list.friendships[0].points = 3500;
        let updated = apply(SAMPLE_XML, &list).unwrap();
        assert!(updated.contains("<Points>3500</Points>"));
        assert!(updated.contains("<Points>500</Points>"));
    }

    #[test]
    fn test_hearts_conversion() {
        assert_eq!(hearts_from_points(0), 0);
        assert_eq!(hearts_from_points(250), 1);
        assert_eq!(hearts_from_points(1250), 5);
        assert_eq!(points_from_hearts(5), 1250);
        assert_eq!(max_hearts_for_status("Married"), 14);
        assert_eq!(max_hearts_for_status("Dating"), 11);
        assert_eq!(max_hearts_for_status("Friendly"), 10);
        assert_eq!(max_hearts_for_status(""), 10);
    }

    #[test]
    fn test_parse_xsi_type() {
        let xml = r#"<SaveGame>
<player>
<friendshipData>
<item><key><string xsi:type="xsd:string">Abigail</string></key><value><Friendship><Points>1250</Points></Friendship></value></item>
</friendshipData>
</player>
</SaveGame>"#;
        let list = parse(xml).unwrap();
        assert_eq!(list.friendships.len(), 1);
        assert_eq!(list.friendships[0].npc_name, "Abigail");
    }
}