use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use crate::mod_parser::ModInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModLoadOrder {
    pub unique_id: String,
    pub name: String,
    pub position: usize,
    pub layer: LoadOrderLayer,
    pub reason: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LoadOrderLayer {
    Core,
    Framework,
    Library,
    Content,
    Expansion,
    Override,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadOrderReport {
    pub ordered_mods: Vec<ModLoadOrder>,
    pub conflicts: Vec<String>,
    pub suggestions: Vec<String>,
    pub total_mods: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyLoadOrderResult {
    pub success: bool,
    pub message: String,
    pub moved_count: usize,
}

fn determine_layer(mod_info: &ModInfo) -> LoadOrderLayer {
    let name_lower = mod_info.name.to_lowercase();
    let id_lower = mod_info.unique_id.to_lowercase();

    if id_lower.contains("contentpatcher") || id_lower.contains("jsonassets") || id_lower.contains("genericmodconfigmenu") {
        return LoadOrderLayer::Framework;
    }

    if id_lower.contains("lib") || name_lower.contains("library") {
        return LoadOrderLayer::Library;
    }

    if name_lower.contains("expansion") || name_lower.contains("sve") || name_lower.contains("expansion mod") {
        return LoadOrderLayer::Expansion;
    }

    if name_lower.contains("override") || name_lower.contains("patch") {
        return LoadOrderLayer::Override;
    }

    if mod_info.is_content_pack {
        return LoadOrderLayer::Content;
    }

    LoadOrderLayer::Core
}

fn layer_priority(layer: &LoadOrderLayer) -> usize {
    match layer {
        LoadOrderLayer::Framework => 0,
        LoadOrderLayer::Library => 1,
        LoadOrderLayer::Core => 2,
        LoadOrderLayer::Content => 3,
        LoadOrderLayer::Expansion => 4,
        LoadOrderLayer::Override => 5,
    }
}

fn find_content_packs(mods: &[ModInfo]) -> HashMap<String, Vec<String>> {
    let mut packs: HashMap<String, Vec<String>> = HashMap::new();

    for m in mods {
        if let Some(parent_id) = &m.content_pack_for {
            if m.is_content_pack {
                packs.entry(parent_id.clone()).or_default().push(m.unique_id.clone());
            }
        }
    }

    packs
}

#[tauri::command]
pub fn calculate_optimal_load_order(mods: Vec<ModInfo>) -> Result<LoadOrderReport, String> {
    let mut ordered = Vec::new();
    let mut conflicts = Vec::new();
    let mut suggestions = Vec::new();

    let content_packs = find_content_packs(&mods);

    let mut scored_mods: Vec<(ModInfo, LoadOrderLayer, usize)> = mods.iter().map(|m| {
        let layer = determine_layer(m);
        let priority = layer_priority(&layer);
        (m.clone(), layer, priority)
    }).collect();

    scored_mods.sort_by(|a, b| {
        a.2.cmp(&b.2).then_with(|| a.0.name.cmp(&b.0.name))
    });

    let installed_ids: HashSet<String> = mods.iter().map(|m| m.unique_id.to_lowercase()).collect();

    for (i, (mod_info, layer, _priority)) in scored_mods.iter().enumerate() {
        let mut deps: Vec<String> = Vec::new();
        let reason;

        if let Some(packs) = content_packs.get(&mod_info.unique_id) {
            reason = format!("Framework/Library mod with {} content packs", packs.len());
            deps.extend(packs.clone());
        } else if mod_info.is_content_pack {
            let parent = mod_info.content_pack_for.as_deref().unwrap_or("unknown");
            reason = format!("Content pack for {}", parent);
            deps.push(parent.to_string());
        } else {
            let missing_deps: Vec<_> = mod_info.dependencies.iter()
                .filter(|d| !installed_ids.contains(&d.unique_id.to_lowercase()))
                .map(|d| d.unique_id.clone())
                .collect();

            if !missing_deps.is_empty() {
                conflicts.push(format!("{} is missing dependencies: {}", mod_info.name, missing_deps.join(", ")));
                suggestions.push(format!("Install missing dependencies for {} to optimize load order", mod_info.name));
            }

            reason = format!("{:?} layer mod", layer);
        }

        ordered.push(ModLoadOrder {
            unique_id: mod_info.unique_id.clone(),
            name: mod_info.name.clone(),
            position: i,
            layer: layer.clone(),
            reason,
            dependencies: deps,
        });
    }

    if !content_packs.is_empty() {
        suggestions.push(format!("{} content packs are properly bound to their parent mods", content_packs.values().map(|v| v.len()).sum::<usize>()));
    }

    Ok(LoadOrderReport {
        ordered_mods: ordered,
        conflicts,
        suggestions,
        total_mods: mods.len(),
    })
}

#[tauri::command]
pub fn apply_load_order(
    game_path: String,
    order: Vec<String>,
) -> Result<ApplyLoadOrderResult, String> {
    let mods_dir = PathBuf::from(&game_path).join("Mods");

    if !mods_dir.exists() {
        return Err("Mods directory not found".to_string());
    }

    let mut moved_count = 0;

    for (i, mod_id) in order.iter().enumerate() {
        let mod_path = mods_dir.join(mod_id);
        if !mod_path.exists() {
            continue;
        }

        let new_name = format!("{:03}_{}", i, mod_id);
        let new_path = mods_dir.join(&new_name);

        if new_path.exists() {
            continue;
        }

        fs::rename(&mod_path, &new_path)
            .map_err(|e| format!("Failed to rename {}: {}", mod_id, e))?;

        moved_count += 1;
    }

    Ok(ApplyLoadOrderResult {
        success: true,
        message: format!("Successfully reordered {} mods", moved_count),
        moved_count,
    })
}
