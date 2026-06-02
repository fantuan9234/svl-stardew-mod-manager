use crate::save_editor::error::{Result, SaveEditorError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemInfo {
    pub index: usize,
    pub item_id: i32,
    pub stack: i32,
    pub name: String,
    pub quality: i32,
    pub raw_xml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub items: Vec<ItemInfo>,
}

pub fn parse(xml: &str) -> Result<Inventory> {
    let items_start = xml
        .find("<items>")
        .ok_or_else(|| SaveEditorError::InvalidStructure("missing <items>".to_string()))?
        + "<items>".len();
    let items_end_rel = xml[items_start..]
        .find("</items>")
        .ok_or_else(|| SaveEditorError::InvalidStructure("unclosed <items>".to_string()))?;
    let items_block = &xml[items_start..items_start + items_end_rel];

    let mut items = Vec::new();
    let mut cursor = 0;
    let mut index = 0;
    while let Some(s_idx) = items_block[cursor..].find("<Item>") {
        let abs_idx = cursor + s_idx + "<Item>".len();
        if let Some(e_idx) = items_block[abs_idx..].find("</Item>") {
            let inner = &items_block[abs_idx..abs_idx + e_idx];
            let raw = format!("<Item>{}</Item>", inner);
            items.push(ItemInfo {
                index,
                item_id: extract_tag(inner, "parentSheetIndex")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                stack: extract_tag(inner, "stack")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1),
                name: extract_tag(inner, "name").unwrap_or_default(),
                quality: extract_tag(inner, "quality")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                raw_xml: raw,
            });
            index += 1;
            cursor = abs_idx + e_idx + "</Item>".len();
        } else {
            break;
        }
    }

    Ok(Inventory { items })
}

pub fn apply(xml: &str, inv: &Inventory) -> Result<String> {
    let items_start = xml
        .find("<items>")
        .ok_or_else(|| SaveEditorError::InvalidStructure("missing <items>".to_string()))?;
    let items_end_rel = xml[items_start..]
        .find("</items>")
        .ok_or_else(|| SaveEditorError::InvalidStructure("unclosed <items>".to_string()))?
        + "</items>".len();
    let items_end = items_start + items_end_rel;

    let mut new_items = String::from("<items>");
    for item in &inv.items {
        new_items.push_str(&serialize_item(item));
    }
    new_items.push_str("</items>");

    Ok(format!(
        "{}{}{}",
        &xml[..items_start],
        new_items,
        &xml[items_end..]
    ))
}

fn serialize_item(item: &ItemInfo) -> String {
    let mut out = String::from("<Item>");
    out.push_str(&format!("<parentSheetIndex>{}</parentSheetIndex>", item.item_id));
    out.push_str(&format!("<stack>{}</stack>", item.stack));
    if !item.name.is_empty() {
        out.push_str(&format!("<name>{}</name>", escape(&item.name)));
    }
    if item.quality > 0 {
        out.push_str(&format!("<quality>{}</quality>", item.quality));
    }
    out.push_str("</Item>");
    out
}

pub fn update_item(inv: &mut Inventory, index: usize, stack: i32) -> Result<()> {
    let item = inv
        .items
        .iter_mut()
        .find(|i| i.index == index)
        .ok_or_else(|| {
            SaveEditorError::NotFound(format!("Item index {}", index))
        })?;
    item.stack = stack;
    Ok(())
}

pub fn remove_item(inv: &mut Inventory, index: usize) -> Result<()> {
    let pos = inv
        .items
        .iter()
        .position(|i| i.index == index)
        .ok_or_else(|| {
            SaveEditorError::NotFound(format!("Item index {}", index))
        })?;
    inv.items.remove(pos);
    Ok(())
}

pub fn add_item(inv: &mut Inventory, item_id: i32, stack: i32, name: String) {
    let new_index = inv.items.iter().map(|i| i.index).max().unwrap_or(0) + 1;
    inv.items.push(ItemInfo {
        index: new_index,
        item_id,
        stack,
        name,
        quality: 0,
        raw_xml: String::new(),
    });
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
<items>
<Item><parentSheetIndex>0</parentSheetIndex><stack>1</stack><name>Parsnip</name></Item>
<Item><parentSheetIndex>4</parentSheetIndex><stack>5</stack><name>Acorn</name></Item>
</items>
</player>
</SaveGame>"#;

    #[test]
    fn test_parse_items() {
        let inv = parse(SAMPLE).unwrap();
        assert_eq!(inv.items.len(), 2);
        assert_eq!(inv.items[0].item_id, 0);
        assert_eq!(inv.items[0].stack, 1);
        assert_eq!(inv.items[0].name, "Parsnip");
        assert_eq!(inv.items[1].item_id, 4);
        assert_eq!(inv.items[1].stack, 5);
    }

    #[test]
    fn test_update_item_stack() {
        let mut inv = parse(SAMPLE).unwrap();
        update_item(&mut inv, 0, 999).unwrap();
        let updated = apply(SAMPLE, &inv).unwrap();
        assert!(updated.contains("<stack>999</stack>"));
        assert!(updated.contains("<stack>5</stack>"));
        assert!(updated.contains("<parentSheetIndex>4</parentSheetIndex>"));
    }

    #[test]
    fn test_remove_item() {
        let mut inv = parse(SAMPLE).unwrap();
        remove_item(&mut inv, 0).unwrap();
        let updated = apply(SAMPLE, &inv).unwrap();
        assert!(!updated.contains("Parsnip"));
        assert!(updated.contains("Acorn"));
    }

    #[test]
    fn test_add_item() {
        let mut inv = parse(SAMPLE).unwrap();
        add_item(&mut inv, 24, 1, "Parsnip Seeds".to_string());
        let updated = apply(SAMPLE, &inv).unwrap();
        assert!(updated.contains("<parentSheetIndex>24</parentSheetIndex>"));
        assert!(updated.contains("<name>Parsnip Seeds</name>"));
    }
}
