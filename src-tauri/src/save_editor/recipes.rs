use crate::save_editor::error::{Result, SaveEditorError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeInfo {
    pub index: usize,
    pub name: String,
    pub unlocked: bool,
    pub times_crafted: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeData {
    pub cooking: Vec<RecipeInfo>,
    pub crafting: Vec<RecipeInfo>,
}

fn extract_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    Some(xml[start..end].to_string())
}

fn parse_recipe_list(xml: &str, container_tag: &str) -> Result<Vec<RecipeInfo>> {
    let start_tag = format!("<{}>", container_tag);
    let end_tag = format!("</{}>", container_tag);

    let block_start = xml
        .find(&start_tag)
        .ok_or_else(|| SaveEditorError::InvalidStructure(format!("missing <{}>", container_tag)))?
        + start_tag.len();
    let block_end_rel = xml[block_start..]
        .find(&end_tag)
        .ok_or_else(|| SaveEditorError::InvalidStructure(format!("unclosed <{}>", container_tag)))?;
    let block = &xml[block_start..block_start + block_end_rel];

    let mut recipes = Vec::new();
    let mut cursor = 0;
    let mut index = 0;

    while let Some(idx) = block[cursor..].find("<item>") {
        let abs = cursor + idx + "<item>".len();
        if let Some(end) = block[abs..].find("</item>") {
            let inner = &block[abs..abs + end];
            let name = extract_tag(inner, "string").unwrap_or_default();
            let val = extract_tag(inner, "int").and_then(|s| s.parse().ok()).unwrap_or(0);
            recipes.push(RecipeInfo {
                index,
                name,
                unlocked: val > 0,
                times_crafted: val,
            });
            index += 1;
            cursor = abs + end + "</item>".len();
        } else {
            break;
        }
    }

    Ok(recipes)
}

pub fn parse(xml: &str) -> Result<RecipeData> {
    let cooking = parse_recipe_list(xml, "cookingRecipes")?;
    let crafting = parse_recipe_list(xml, "craftingRecipes")?;
    Ok(RecipeData { cooking, crafting })
}

fn replace_int_in_item(xml: &str, target_index: usize) -> String {
    // 找到第 target_index 个 <item> 中的 <int> 替换
    let mut result = xml.to_string();
    let mut count = 0;
    let mut cursor = 0;

    while let Some(idx) = result[cursor..].find("<item>") {
        let abs = cursor + idx + "<item>".len();
        if let Some(end) = result[abs..].find("</item>") {
            if count == target_index {
                let item_end = abs + end;
                // 找到这个 <item> 内的 <int>X</int>
                let open_int = "<int";
                let close_int = "</int>";
                if let Some(int_start_rel) = result[abs..item_end].find(open_int) {
                    let int_start = abs + int_start_rel;
                    // 跳过属性
                    let gt_pos = result[int_start..].find('>').unwrap() + int_start;
                    if let Some(close_rel) = result[gt_pos..item_end].find(close_int) {
                        let val_start = gt_pos + 1;
                        let val_end = gt_pos + close_rel;
                        // result[val_start..val_end] 是值
                        // 我们不改 val，由调用方设置
                        return result;
                    }
                }
                break;
            }
            count += 1;
            cursor = abs + end + "</item>".len();
        } else {
            break;
        }
    }
    result
}

/// 解析 <int> 标签的位置
fn find_int_value_range(xml: &str, item_start: usize, item_end: usize) -> Option<(usize, usize)> {
    let open_int = "<int";
    let close_int = "</int>";
    let block = &xml[item_start..item_end];
    let int_start_rel = block.find(open_int)?;
    let int_start = item_start + int_start_rel;
    let gt_pos = xml[int_start..].find('>')? + int_start;
    if let Some(close_rel) = xml[gt_pos..item_end].find(close_int) {
        let val_start = gt_pos + 1;
        let val_end = gt_pos + close_rel;
        Some((val_start, val_end))
    } else {
        None
    }
}

fn apply_recipe_list(xml: &str, recipes: &[RecipeInfo], container_tag: &str) -> Result<String> {
    let start_tag = format!("<{}>", container_tag);
    let end_tag = format!("</{}>", container_tag);

    let block_start = xml
        .find(&start_tag)
        .ok_or_else(|| SaveEditorError::InvalidStructure(format!("missing <{}>", container_tag)))?
        + start_tag.len();
    let block_end = xml[block_start..]
        .find(&end_tag)
        .ok_or_else(|| SaveEditorError::InvalidStructure(format!("unclosed <{}>", container_tag)))?
        + block_start;
    let block = &xml[block_start..block_end];

    // 收集所有 <item> 范围
    let mut item_ranges: Vec<(usize, usize)> = Vec::new();
    let mut cursor = 0;
    while let Some(idx) = block[cursor..].find("<item>") {
        let abs = cursor + idx + "<item>".len();
        if let Some(end) = block[abs..].find("</item>") {
            item_ranges.push((block_start + abs, block_start + abs + end));
            cursor = abs + end + "</item>".len();
        } else {
            break;
        }
    }

    // 从右到左替换
    let mut updates: Vec<usize> = (0..recipes.len()).filter(|i| *i < item_ranges.len()).collect();
    updates.sort_by_key(|i| std::cmp::Reverse(*i));

    let mut result = xml.to_string();
    for i in updates {
        let (start, end) = item_ranges[i];
        if let Some((vs, ve)) = find_int_value_range(&result, start, end) {
            let val = if recipes[i].unlocked { recipes[i].times_crafted.max(1) } else { 0 };
            result.replace_range(vs..ve, &val.to_string());
        }
    }

    Ok(result)
}

pub fn apply(xml: &str, data: &RecipeData) -> Result<String> {
    let result = apply_recipe_list(xml, &data.cooking, "cookingRecipes")?;
    apply_recipe_list(&result, &data.crafting, "craftingRecipes")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_XML: &str = r#"<SaveGame>
<player>
<cookingRecipes>
<item><key><string>Fried Egg</string></key><value><int>1</int></value></item>
<item><key><string>Omelet</string></key><value><int>0</int></value></item>
</cookingRecipes>
<craftingRecipes>
<item><key><string>Wood Fence</string></key><value><int>5</int></value></item>
<item><key><string>Torch</string></key><value><int>1</int></value></item>
</craftingRecipes>
</player>
</SaveGame>"#;

    #[test]
    fn test_parse_recipes() {
        let data = parse(SAMPLE_XML).unwrap();
        assert_eq!(data.cooking.len(), 2);
        assert_eq!(data.cooking[0].name, "Fried Egg");
        assert!(data.cooking[0].unlocked);
        assert!(!data.cooking[1].unlocked);
        assert_eq!(data.crafting.len(), 2);
        assert_eq!(data.crafting[0].times_crafted, 5);
    }

    #[test]
    fn test_apply_recipes() {
        let mut data = parse(SAMPLE_XML).unwrap();
        data.cooking[1].unlocked = true;
        data.cooking[1].times_crafted = 1;
        let updated = apply(SAMPLE_XML, &data).unwrap();
        // Omelet 应被解锁
        let reparsed = parse(&updated).unwrap();
        assert!(reparsed.cooking[1].unlocked);
        assert_eq!(reparsed.cooking[1].times_crafted, 1);
    }
}