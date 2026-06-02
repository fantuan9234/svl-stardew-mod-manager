use crate::save_editor::error::{Result, SaveEditorError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingInfo {
    pub index: usize,
    pub location: String,
    pub building_type: String,
    pub tile_x: i32,
    pub tile_y: i32,
    pub upgrade_level: i32,
    pub max_occupants: i32,
    pub current_occupants: i32,
    pub raw_xml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingList {
    pub buildings: Vec<BuildingInfo>,
}

pub fn parse(xml: &str) -> Result<BuildingList> {
    let mut buildings = Vec::new();
    let mut index = 0;

    let loc_start = xml
        .find("<locations>")
        .ok_or_else(|| SaveEditorError::InvalidStructure("missing <locations>".to_string()))?
        + "<locations>".len();
    let loc_end_rel = xml[loc_start..]
        .find("</locations>")
        .ok_or_else(|| SaveEditorError::InvalidStructure("unclosed <locations>".to_string()))?;
    let loc_block = &xml[loc_start..loc_start + loc_end_rel];

    let mut loc_cursor = 0;
    while let Some(gl_idx) = loc_block[loc_cursor..].find("<GameLocation>") {
        let gl_abs = loc_cursor + gl_idx + "<GameLocation>".len();
        let gl_close_rel = loc_block[gl_abs..].find("</GameLocation>").unwrap_or(0);
        if gl_close_rel == 0 {
            break;
        }
        let gl_block = &loc_block[gl_abs..gl_abs + gl_close_rel];
        let location_name = extract_tag(gl_block, "name").unwrap_or_default();

        if let Some(b_start) = gl_block.find("<buildings>") {
            let b_abs = b_start + "<buildings>".len();
            if let Some(b_end_rel) = gl_block[b_abs..].find("</buildings>") {
                let b_block = &gl_block[b_abs..b_abs + b_end_rel];

                let mut b_cursor = 0;
                while let Some(s_idx) = b_block[b_cursor..].find("<Building>") {
                    let abs_idx = b_cursor + s_idx + "<Building>".len();
                    if let Some(e_idx) = b_block[abs_idx..].find("</Building>") {
                        let inner = &b_block[abs_idx..abs_idx + e_idx];
                        let raw = format!("<Building>{}</Building>", inner);
                        buildings.push(BuildingInfo {
                            index,
                            location: location_name.clone(),
                            building_type: extract_tag(inner, "buildingType").unwrap_or_default(),
                            tile_x: extract_tag(inner, "tileX")
                                .and_then(|s| s.parse().ok())
                                .unwrap_or(0),
                            tile_y: extract_tag(inner, "tileY")
                                .and_then(|s| s.parse().ok())
                                .unwrap_or(0),
                            upgrade_level: extract_tag(inner, "buildingType")
                                .as_ref()
                                .map(|_| {
                                    extract_tag(inner, "daysOfConstructionLeft")
                                        .and_then(|s| s.parse().ok())
                                        .unwrap_or(0)
                                })
                                .unwrap_or(0),
                            max_occupants: extract_tag(inner, "maxOccupants")
                                .and_then(|s| s.parse().ok())
                                .unwrap_or(0),
                            current_occupants: extract_tag(inner, "currentOccupants")
                                .and_then(|s| s.parse().ok())
                                .unwrap_or(0),
                            raw_xml: raw,
                        });
                        index += 1;
                        b_cursor = abs_idx + e_idx + "</Building>".len();
                    } else {
                        break;
                    }
                }
            }
        }

        loc_cursor = gl_abs + gl_close_rel + "</GameLocation>".len();
    }

    Ok(BuildingList { buildings })
}

pub fn apply(xml: &str, list: &BuildingList) -> Result<String> {
    let loc_start = xml
        .find("<locations>")
        .ok_or_else(|| SaveEditorError::InvalidStructure("missing <locations>".to_string()))?;
    let loc_end_rel = xml[loc_start..]
        .find("</locations>")
        .ok_or_else(|| SaveEditorError::InvalidStructure("unclosed <locations>".to_string()))?
        + "</locations>".len();
    let loc_end = loc_start + loc_end_rel;

    let mut new_loc = String::from("<locations>");
    let mut building_idx = 0;
    let prefix = &xml[..loc_start];
    let loc_block = &xml[loc_start + "<locations>".len()..loc_end - "</locations>".len()];

    let mut loc_cursor = 0;
    while let Some(gl_idx) = loc_block[loc_cursor..].find("<GameLocation>") {
        let gl_abs = loc_cursor + gl_idx;
        let gl_close_rel = loc_block[gl_abs + "<GameLocation>".len()..]
            .find("</GameLocation>")
            .unwrap_or(0);
        if gl_close_rel == 0 {
            break;
        }
        let gl_block = &loc_block[gl_abs..gl_abs + "<GameLocation>".len() + gl_close_rel + "</GameLocation>".len()];

        let mut new_gl = String::from("<GameLocation>");
        if let Some(name_close) = find_close_tag(gl_block, "name") {
            new_gl.push_str(&gl_block[..name_close]);
        }

        if let Some(b_start) = gl_block.find("<buildings>") {
            let b_abs = b_start + "<buildings>".len();
            if let Some(b_end_rel) = gl_block[b_abs..].find("</buildings>") {
                let before = &gl_block[..b_abs];
                let after_rel = b_abs + b_end_rel + "</buildings>".len();
                let after = &gl_block[after_rel..];
                let b_block = &gl_block[b_abs..b_abs + b_end_rel];

                let mut new_buildings = String::new();
                let mut b_cursor = 0;
                while let Some(s_idx) = b_block[b_cursor..].find("<Building>") {
                    let abs_idx = b_cursor + s_idx + "<Building>".len();
                    if let Some(e_idx) = b_block[abs_idx..].find("</Building>") {
                        if building_idx < list.buildings.len() {
                            new_buildings.push_str(&serialize_building(&list.buildings[building_idx]));
                            building_idx += 1;
                        }
                        b_cursor = abs_idx + e_idx + "</Building>".len();
                    } else {
                        break;
                    }
                }

                new_gl.push_str(before);
                new_gl.push_str(&new_buildings);
                new_gl.push_str("</buildings>");
                new_gl.push_str(after);
            } else {
                new_gl.push_str(gl_block);
            }
        } else {
            new_gl.push_str(gl_block);
        }

        new_loc.push_str(&new_gl);
        loc_cursor = gl_abs + gl_block.len();
    }
    new_loc.push_str("</locations>");

    Ok(format!("{}{}{}", prefix, new_loc, &xml[loc_end..]))
}

fn serialize_building(b: &BuildingInfo) -> String {
    let mut out = String::from("<Building>");
    out.push_str(&format!(
        "<buildingType>{}</buildingType>",
        escape(&b.building_type)
    ));
    out.push_str(&format!("<tileX>{}</tileX>", b.tile_x));
    out.push_str(&format!("<tileY>{}</tileY>", b.tile_y));
    if b.max_occupants > 0 {
        out.push_str(&format!("<maxOccupants>{}</maxOccupants>", b.max_occupants));
    }
    if b.current_occupants > 0 {
        out.push_str(&format!(
            "<currentOccupants>{}</currentOccupants>",
            b.current_occupants
        ));
    }
    out.push_str("</Building>");
    out
}

fn find_close_tag(xml: &str, tag: &str) -> Option<usize> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)? + open.len();
    let end_rel = xml[start..].find(&close)?;
    Some(start + end_rel + close.len())
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
<locations>
<GameLocation>
<name>Farm</name>
<buildings>
<Building><buildingType>Cabin</buildingType><tileX>10</tileX><tileY>20</tileY><maxOccupants>4</maxOccupants></Building>
<Building><buildingType>Coop</buildingType><tileX>30</tileX><tileY>40</tileY><maxOccupants>4</maxOccupants></Building>
</buildings>
</GameLocation>
<GameLocation>
<name>IslandFarm</name>
<buildings>
<Building><buildingType>Barn</buildingType><tileX>5</tileX><tileY>5</tileY><maxOccupants>4</maxOccupants></Building>
</buildings>
</GameLocation>
</locations>
</SaveGame>"#;

    #[test]
    fn test_parse_buildings() {
        let list = parse(SAMPLE).unwrap();
        assert_eq!(list.buildings.len(), 3);
        assert_eq!(list.buildings[0].location, "Farm");
        assert_eq!(list.buildings[0].building_type, "Cabin");
        assert_eq!(list.buildings[0].tile_x, 10);
        assert_eq!(list.buildings[1].building_type, "Coop");
        assert_eq!(list.buildings[2].location, "IslandFarm");
    }

    #[test]
    fn test_apply_building_tile_move() {
        let mut list = parse(SAMPLE).unwrap();
        list.buildings[0].tile_x = 99;
        list.buildings[0].tile_y = 88;
        let updated = apply(SAMPLE, &list).unwrap();
        assert!(updated.contains("<tileX>99</tileX>"));
        assert!(updated.contains("<tileY>88</tileY>"));
        assert!(updated.contains("<buildingType>Cabin</buildingType>"));
    }
}
