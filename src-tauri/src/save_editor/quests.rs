use crate::save_editor::error::{Result, SaveEditorError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestInfo {
    pub index: usize,
    pub id: String,
    pub title: String,
    pub description: String,
    pub current_objective: String,
    pub money_reward: i32,
    pub completed: bool,
    pub days_left: i32,
    pub raw_xml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestLog {
    pub quests: Vec<QuestInfo>,
}

pub fn parse(xml: &str) -> Result<QuestLog> {
    let log_start = xml
        .find("<questLog>")
        .ok_or_else(|| SaveEditorError::InvalidStructure("missing <questLog>".to_string()))?
        + "<questLog>".len();
    let log_end_rel = xml[log_start..]
        .find("</questLog>")
        .ok_or_else(|| SaveEditorError::InvalidStructure("unclosed <questLog>".to_string()))?;
    let log_block = &xml[log_start..log_start + log_end_rel];

    let mut quests = Vec::new();
    let mut cursor = 0;
    let mut index = 0;
    while let Some(s_idx) = log_block[cursor..].find("<Quest>") {
        let abs_idx = cursor + s_idx + "<Quest>".len();
        if let Some(e_idx) = log_block[abs_idx..].find("</Quest>") {
            let inner = &log_block[abs_idx..abs_idx + e_idx];
            let raw = format!("<Quest>{}</Quest>", inner);
            quests.push(QuestInfo {
                index,
                id: extract_tag(inner, "id").unwrap_or_default(),
                title: extract_tag(inner, "title").unwrap_or_default(),
                description: extract_tag(inner, "description").unwrap_or_default(),
                current_objective: extract_tag(inner, "currentObjective").unwrap_or_default(),
                money_reward: extract_tag(inner, "moneyReward")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                completed: extract_tag(inner, "completed")
                    .map(|s| s == "true")
                    .unwrap_or(false),
                days_left: extract_tag(inner, "daysLeft")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                raw_xml: raw,
            });
            index += 1;
            cursor = abs_idx + e_idx + "</Quest>".len();
        } else {
            break;
        }
    }

    Ok(QuestLog { quests })
}

pub fn apply(xml: &str, log: &QuestLog) -> Result<String> {
    let log_start = xml
        .find("<questLog>")
        .ok_or_else(|| SaveEditorError::InvalidStructure("missing <questLog>".to_string()))?;
    let log_end_rel = xml[log_start..]
        .find("</questLog>")
        .ok_or_else(|| SaveEditorError::InvalidStructure("unclosed <questLog>".to_string()))?
        + "</questLog>".len();
    let log_end = log_start + log_end_rel;

    let mut new_log = String::from("<questLog>");
    for q in &log.quests {
        new_log.push_str(&serialize_quest(q));
    }
    new_log.push_str("</questLog>");

    Ok(format!(
        "{}{}{}",
        &xml[..log_start],
        new_log,
        &xml[log_end..]
    ))
}

fn serialize_quest(q: &QuestInfo) -> String {
    let mut out = String::from("<Quest>");
    if !q.id.is_empty() {
        out.push_str(&format!("<id>{}</id>", escape(&q.id)));
    }
    if !q.title.is_empty() {
        out.push_str(&format!("<title>{}</title>", escape(&q.title)));
    }
    if !q.description.is_empty() {
        out.push_str(&format!("<description>{}</description>", escape(&q.description)));
    }
    if !q.current_objective.is_empty() {
        out.push_str(&format!(
            "<currentObjective>{}</currentObjective>",
            escape(&q.current_objective)
        ));
    }
    if q.money_reward > 0 {
        out.push_str(&format!("<moneyReward>{}</moneyReward>", q.money_reward));
    }
    out.push_str(&format!(
        "<completed>{}</completed>",
        if q.completed { "true" } else { "false" }
    ));
    out.push_str("</Quest>");
    out
}

fn extract_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    Some(xml[start..end].to_string())
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"<SaveGame>
<player>
<questLog>
<Quest><id>1</id><title>Initiation</title><currentObjective>Reach level 1 in something</currentObjective><moneyReward>100</moneyReward><completed>false</completed><daysLeft>5</daysLeft></Quest>
<Quest><id>13</id><title>Deeper In The Mine</title><currentObjective></currentObjective><moneyReward>250</moneyReward><completed>true</completed></Quest>
</questLog>
</player>
</SaveGame>"#;

    #[test]
    fn test_parse_quests() {
        let log = parse(SAMPLE).unwrap();
        assert_eq!(log.quests.len(), 2);
        assert_eq!(log.quests[0].id, "1");
        assert_eq!(log.quests[0].title, "Initiation");
        assert_eq!(log.quests[0].money_reward, 100);
        assert!(!log.quests[0].completed);
        assert!(log.quests[1].completed);
    }

    #[test]
    fn test_apply_quest_completion() {
        let mut log = parse(SAMPLE).unwrap();
        log.quests[0].completed = true;
        let updated = apply(SAMPLE, &log).unwrap();
        assert!(updated.contains("<completed>true</completed>"));
        assert!(updated.contains("<title>Initiation</title>"));
    }

    #[test]
    fn test_apply_quest_money() {
        let mut log = parse(SAMPLE).unwrap();
        log.quests[0].money_reward = 9999;
        let updated = apply(SAMPLE, &log).unwrap();
        assert!(updated.contains("<moneyReward>9999</moneyReward>"));
    }
}
