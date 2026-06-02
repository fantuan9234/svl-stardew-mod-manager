use crate::save_editor::error::{Result, SaveEditorError};
use crate::save_editor::{SkillInfo, SkillSet};

pub fn parse(xml: &str) -> Result<SkillSet> {
    let skills_start = xml
        .find("<skills>")
        .ok_or_else(|| SaveEditorError::InvalidStructure("missing <skills>".to_string()))?
        + "<skills>".len();
    let skills_end_rel = xml[skills_start..]
        .find("</skills>")
        .ok_or_else(|| SaveEditorError::InvalidStructure("unclosed <skills>".to_string()))?;
    let skills_block = &xml[skills_start..skills_start + skills_end_rel];

    let mut skills = Vec::new();
    let mut cursor = 0;
    while let Some(s_idx) = skills_block[cursor..].find("<Skill>") {
        let abs_idx = cursor + s_idx + "<Skill>".len();
        if let Some(e_idx) = skills_block[abs_idx..].find("</Skill>") {
            let skill_xml = &skills_block[abs_idx..abs_idx + e_idx];
            let name = extract_tag(skill_xml, "Name").unwrap_or_default();
            let level = extract_tag(skill_xml, "Level")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let exp = extract_tag(skill_xml, "Experience")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            skills.push(SkillInfo {
                name,
                level,
                experience: exp,
            });
            cursor = abs_idx + e_idx + "</Skill>".len();
        } else {
            break;
        }
    }

    Ok(SkillSet { skills })
}

pub fn apply(xml: &str, set: &SkillSet) -> Result<String> {
    let new_skills = build_skills_block(&set.skills);
    let skills_start = xml
        .find("<skills>")
        .ok_or_else(|| SaveEditorError::InvalidStructure("missing <skills>".to_string()))?;
    let skills_end_rel = xml[skills_start..]
        .find("</skills>")
        .ok_or_else(|| SaveEditorError::InvalidStructure("unclosed <skills>".to_string()))?
        + "</skills>".len();
    let skills_end = skills_start + skills_end_rel;

    Ok(format!(
        "{}{}{}",
        &xml[..skills_start],
        new_skills,
        &xml[skills_end..]
    ))
}

fn build_skills_block(skills: &[SkillInfo]) -> String {
    let mut out = String::from("<skills>");
    for s in skills {
        out.push_str(&format!(
            "<Skill><Name>{}</Name><Level>{}</Level><Experience>{}</Experience></Skill>",
            escape(&s.name),
            s.level,
            s.experience
        ));
    }
    out.push_str("</skills>");
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
<skills>
<Skill><Name>Farming</Name><Level>5</Level><Experience>2500</Experience></Skill>
<Skill><Name>Mining</Name><Level>3</Level><Experience>1500</Experience></Skill>
</skills>
</player>
</SaveGame>"#;

    #[test]
    fn test_parse_skills() {
        let set = parse(SAMPLE).unwrap();
        assert_eq!(set.skills.len(), 2);
        assert_eq!(set.skills[0].name, "Farming");
        assert_eq!(set.skills[0].level, 5);
        assert_eq!(set.skills[0].experience, 2500);
        assert_eq!(set.skills[1].name, "Mining");
        assert_eq!(set.skills[1].level, 3);
    }

    #[test]
    fn test_apply_skill_edits() {
        let mut set = parse(SAMPLE).unwrap();
        set.skills[0].level = 10;
        let updated = apply(SAMPLE, &set).unwrap();
        assert!(updated.contains("<Level>10</Level>"));
        assert!(updated.contains("<Experience>1500</Experience>"));
    }
}
