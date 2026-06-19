use crate::save_editor::error::{Result, SaveEditorError};
use crate::save_editor::{SkillInfo, SkillSet};

const SKILL_NAMES: &[&str] = &["Farming", "Fishing", "Foraging", "Mining", "Combat"];

/// 经验值阈值表：索引 = 等级，值 = 该等级所需的最小经验
const LEVEL_THRESHOLDS: &[i32] = &[0, 100, 380, 770, 1300, 2150, 3300, 4800, 6900, 10000, 15000];

fn skill_level_field(name: &str) -> String {
    let mut chars = name.chars();
    let first = chars.next().unwrap().to_lowercase().to_string();
    format!("{}{}Level", first, chars.as_str())
}

fn exp_to_level(exp: i32) -> i32 {
    LEVEL_THRESHOLDS
        .iter()
        .enumerate()
        .rev()
        .find(|(_, &t)| exp >= t)
        .map(|(i, _)| i as i32)
        .unwrap_or(0)
}

fn exp_for_level(level: i32) -> i32 {
    LEVEL_THRESHOLDS
        .get(level as usize)
        .copied()
        .unwrap_or(0)
}

/// 解析星露谷存档中的技能数据。
///
/// 星露谷存档的技能存储格式：
/// ```xml
/// <experiencePoints><int>15000</int><int>15000</int>...</experiencePoints>
/// <farmingLevel>10</farmingLevel>
/// <miningLevel>10</miningLevel>
/// ...
/// ```
pub fn parse(xml: &str) -> Result<SkillSet> {
    let exps = parse_experience_points(xml)?;

    let mut skills = Vec::new();
    for (i, &name) in SKILL_NAMES.iter().enumerate() {
        let field = skill_level_field(name);
        let direct_level = extract_tag_i32(xml, &field).unwrap_or(0);
        let exp = exps.get(i).copied().unwrap_or(0);
        let computed_level = exp_to_level(exp);
        let level = direct_level.max(computed_level);

        skills.push(SkillInfo {
            name: name.to_string(),
            level,
            experience: exp,
        });
    }

    Ok(SkillSet { skills })
}

fn parse_experience_points(xml: &str) -> Result<Vec<i32>> {
    let tag_start = xml
        .find("<experiencePoints")
        .ok_or_else(|| SaveEditorError::InvalidStructure("missing <experiencePoints>".to_string()))?;

    let content_start = xml[tag_start..]
        .find('>')
        .map(|p| tag_start + p + 1)
        .ok_or_else(|| SaveEditorError::InvalidStructure("malformed <experiencePoints>".to_string()))?;

    let content_end = xml[content_start..]
        .find("</experiencePoints>")
        .map(|p| content_start + p)
        .ok_or_else(|| SaveEditorError::InvalidStructure("unclosed <experiencePoints>".to_string()))?;

    let block = &xml[content_start..content_end];
    let mut values = Vec::new();
    let mut cursor = 0;

    while let Some(idx) = block[cursor..].find("<int") {
        let abs = cursor + idx;
        let val_start = block[abs..].find('>').map(|p| abs + p + 1).unwrap_or(abs);
        if let Some(val_end) = block[val_start..].find("</int>").map(|p| val_start + p) {
            let val_str = &block[val_start..val_end];
            if let Ok(v) = val_str.trim().parse::<i32>() {
                values.push(v);
            }
            cursor = val_end + "</int>".len() - idx;
        } else {
            break;
        }
    }

    Ok(values)
}

fn extract_tag_i32(xml: &str, tag: &str) -> Option<i32> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    xml[start..end].trim().parse().ok()
}

/// 将技能修改应用到存档 XML。
pub fn apply(xml: &str, set: &SkillSet) -> Result<String> {
    let mut result = apply_experience_points(xml, set)?;

    for skill in &set.skills {
        let field = skill_level_field(&skill.name);
        result = replace_tag_value(&result, &field, skill.level)?;
    }

    Ok(result)
}

fn apply_experience_points(xml: &str, set: &SkillSet) -> Result<String> {
    let ep_start = xml
        .find("<experiencePoints")
        .ok_or_else(|| SaveEditorError::InvalidStructure("missing <experiencePoints>".to_string()))?;

    let ep_content_start = xml[ep_start..]
        .find('>')
        .map(|p| ep_start + p + 1)
        .ok_or_else(|| SaveEditorError::InvalidStructure("malformed <experiencePoints>".to_string()))?;

    let ep_end = xml[ep_content_start..]
        .find("</experiencePoints>")
        .map(|p| ep_content_start + p)
        .ok_or_else(|| SaveEditorError::InvalidStructure("unclosed <experiencePoints>".to_string()))?;

    let block = &xml[ep_content_start..ep_end];
    let mut int_positions: Vec<(usize, usize)> = Vec::new();
    let mut cursor = 0;

    while let Some(idx) = block[cursor..].find("<int") {
        let abs = cursor + idx;
        let val_start = block[abs..].find('>').map(|p| abs + p + 1).unwrap_or(abs);
        if let Some(val_end) = block[val_start..].find("</int>").map(|p| val_start + p) {
            int_positions.push((ep_content_start + val_start, ep_content_start + val_end));
            cursor = val_end + "</int>".len() - idx;
        } else {
            break;
        }
    }

    // 从右到左替换，保持偏移量正确
    let mut result = xml.to_string();
    let mut updates: Vec<(usize, i32)> = set
        .skills
        .iter()
        .enumerate()
        .map(|(i, s)| (i, s.experience.max(exp_for_level(s.level))))
        .filter(|(i, _)| *i < int_positions.len())
        .collect();
    updates.sort_by_key(|(i, _)| std::cmp::Reverse(*i));

    for (i, val) in &updates {
        let (vs, ve) = int_positions[*i];
        result.replace_range(vs..ve, &val.to_string());
    }

    Ok(result)
}

fn replace_tag_value(xml: &str, tag: &str, value: i32) -> Result<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);

    let start = xml
        .find(&open)
        .ok_or_else(|| SaveEditorError::InvalidStructure(format!("missing <{}>", tag)))?;
    let content_start = start + open.len();
    let content_end = xml[content_start..]
        .find(&close)
        .ok_or_else(|| SaveEditorError::InvalidStructure(format!("unclosed <{}>", tag)))?;
    let end = content_start + content_end + close.len();

    Ok(format!(
        "{}{}{}{}",
        &xml[..content_start],
        value,
        close,
        &xml[end..]
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_XML: &str = r#"<SaveGame>
<player>
<experiencePoints><int>15000</int><int>15000</int><int>15000</int><int>15000</int><int>15000</int><int>0</int></experiencePoints>
<farmingLevel>10</farmingLevel>
<fishingLevel>10</fishingLevel>
<foragingLevel>10</foragingLevel>
<miningLevel>10</miningLevel>
<combatLevel>10</combatLevel>
<luckLevel>0</luckLevel>
</player>
</SaveGame>"#;

    #[test]
    fn test_parse_max_level() {
        let set = parse(SAMPLE_XML).unwrap();
        assert_eq!(set.skills.len(), 5);
        for skill in &set.skills {
            assert_eq!(skill.level, 10, "{} level should be 10", skill.name);
            assert_eq!(skill.experience, 15000, "{} exp should be 15000", skill.name);
        }
    }

    #[test]
    fn test_parse_zero_level() {
        let xml = r#"<SaveGame>
<player>
<experiencePoints><int>0</int><int>0</int><int>0</int><int>0</int><int>0</int></experiencePoints>
<farmingLevel>0</farmingLevel>
<fishingLevel>0</fishingLevel>
<foragingLevel>0</foragingLevel>
<miningLevel>0</miningLevel>
<combatLevel>0</combatLevel>
</player>
</SaveGame>"#;
        let set = parse(xml).unwrap();
        for skill in &set.skills {
            assert_eq!(skill.level, 0);
            assert_eq!(skill.experience, 0);
        }
    }

    #[test]
    fn test_level_from_experience_only() {
        // 经验值足够 10 级但 level 字段为 0
        let xml = r#"<SaveGame>
<player>
<experiencePoints><int>15000</int><int>0</int><int>0</int><int>0</int><int>0</int></experiencePoints>
<farmingLevel>0</farmingLevel>
<fishingLevel>0</fishingLevel>
<foragingLevel>0</foragingLevel>
<miningLevel>0</miningLevel>
<combatLevel>0</combatLevel>
</player>
</SaveGame>"#;
        let set = parse(xml).unwrap();
        assert_eq!(set.skills[0].level, 10); // 从经验值推导
        assert_eq!(set.skills[0].experience, 15000);
        assert_eq!(set.skills[1].level, 0);
    }

    #[test]
    fn test_level_from_field_when_exp_low() {
        // level 字段值高但经验值低（取 max）
        let xml = r#"<SaveGame>
<player>
<experiencePoints><int>0</int><int>0</int><int>0</int><int>0</int><int>0</int></experiencePoints>
<farmingLevel>10</farmingLevel>
<fishingLevel>0</fishingLevel>
<foragingLevel>0</foragingLevel>
<miningLevel>0</miningLevel>
<combatLevel>0</combatLevel>
</player>
</SaveGame>"#;
        let set = parse(xml).unwrap();
        assert_eq!(set.skills[0].level, 10);
        assert_eq!(set.skills[0].experience, 0);
    }

    #[test]
    fn test_exp_to_level_thresholds() {
        // 验证各等级的经验阈值
        assert_eq!(exp_to_level(0), 0);
        assert_eq!(exp_to_level(99), 0);
        assert_eq!(exp_to_level(100), 1);
        assert_eq!(exp_to_level(379), 1);
        assert_eq!(exp_to_level(380), 2);
        assert_eq!(exp_to_level(14999), 9);
        assert_eq!(exp_to_level(15000), 10);
        assert_eq!(exp_to_level(99999), 10);
    }

    #[test]
    fn test_apply_preserves_structure() {
        let mut set = parse(SAMPLE_XML).unwrap();
        set.skills[0].level = 5;
        set.skills[0].experience = 2500;
        let updated = apply(SAMPLE_XML, &set).unwrap();

        // 验证更新后的值
        let parsed = parse(&updated).unwrap();
        assert_eq!(parsed.skills[0].level, 5);
        assert_eq!(parsed.skills[0].experience, 2500);
    }

    #[test]
    fn test_apply_preserves_other_levels() {
        let mut set = parse(SAMPLE_XML).unwrap();
        set.skills[0].level = 3;
        set.skills[0].experience = 770; // 等级 3 对应的经验值
        let updated = apply(SAMPLE_XML, &set).unwrap();
        let parsed = parse(&updated).unwrap();
        for (i, skill) in parsed.skills.iter().enumerate() {
            if i == 0 {
                assert_eq!(skill.level, 3);
            } else {
                assert_eq!(skill.level, 10);
            }
        }
    }

    #[test]
    fn test_apply_preserves_luck_int() {
        // luck 对应的第6个 <int> 应保持不变
        let set = parse(SAMPLE_XML).unwrap();
        let updated = apply(SAMPLE_XML, &set).unwrap();
        assert!(updated.contains("<luckLevel>0</luckLevel>"));
        // 验证第6个 <int> 仍然是 0
        let exps = parse_experience_points(&updated).unwrap();
        assert_eq!(exps.len(), 6);
        assert_eq!(exps[5], 0); // luck
    }

    #[test]
    fn test_parse_with_xsi_type() {
        let xml = r#"<SaveGame>
<player>
<experiencePoints><int xsi:type="xsd:int">15000</int><int>15000</int><int>15000</int><int>15000</int><int>15000</int></experiencePoints>
<farmingLevel>10</farmingLevel>
<fishingLevel>10</fishingLevel>
<foragingLevel>10</foragingLevel>
<miningLevel>10</miningLevel>
<combatLevel>10</combatLevel>
</player>
</SaveGame>"#;
        let set = parse(xml).unwrap();
        for skill in &set.skills {
            assert_eq!(skill.level, 10);
            assert_eq!(skill.experience, 15000);
        }
    }

    #[test]
    fn test_exp_for_level() {
        assert_eq!(exp_for_level(0), 0);
        assert_eq!(exp_for_level(1), 100);
        assert_eq!(exp_for_level(5), 2150);
        assert_eq!(exp_for_level(10), 15000);
    }

    #[test]
    fn test_parse_partial_experience_points() {
        // 只有3个经验值
        let xml = r#"<SaveGame>
<player>
<experiencePoints><int>15000</int><int>15000</int><int>0</int></experiencePoints>
<farmingLevel>10</farmingLevel>
<fishingLevel>10</fishingLevel>
<foragingLevel>0</foragingLevel>
<miningLevel>0</miningLevel>
<combatLevel>0</combatLevel>
</player>
</SaveGame>"#;
        let set = parse(xml).unwrap();
        assert_eq!(set.skills[0].experience, 15000);
        assert_eq!(set.skills[1].experience, 15000);
        assert_eq!(set.skills[2].experience, 0); // 有对应 <int>
        assert_eq!(set.skills[3].experience, 0); // 无对应 <int>，默认 0
        assert_eq!(set.skills[4].experience, 0);
    }
}