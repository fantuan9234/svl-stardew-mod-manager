use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use crate::mod_patches;
use crate::mod_name_resolver::resolve_mod_name;
use crate::mod_parser::ModInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictReport {
    pub mod_name: String,
    pub unique_id: String,
    pub conflict_type: ConflictType,
    pub description: String,
    pub severity: Severity,
    pub solution: String,
    pub affected_mods: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    MissingDependency,
    OptionalDependencyMissing,
    ContentPackConflict,
    Incompatibility,
    HardcodedPatch,
    AssetConflict,
    ContentPackTargetConflict,
    VersionConflict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[tauri::command]
pub fn check_conflicts(mods: Vec<ModInfo>) -> Result<Vec<ConflictReport>, String> {
    let mut conflicts = Vec::new();
    let mut reported_ids: HashSet<String> = HashSet::new();

    let installed_ids: HashSet<String> = mods.iter().map(|m| m.unique_id.to_lowercase()).collect();

    for mod_info in &mods {
        for dep in &mod_info.dependencies {
            let dep_id_lower = dep.unique_id.to_lowercase();
            if !installed_ids.contains(&dep_id_lower) {
                if dep.is_required {
                    if !reported_ids.contains(&dep_id_lower) {
                        let display_name = resolve_mod_name(&dep.unique_id);
                        conflicts.push(ConflictReport {
                            mod_name: mod_info.name.clone(),
                            unique_id: dep.unique_id.clone(),
                            conflict_type: ConflictType::MissingDependency,
                            description: format!("模组 '{}' 需要依赖 '{}'，但未安装", mod_info.name, display_name),
                            severity: Severity::Error,
                            solution: format!("请安装 '{}' 模组", display_name),
                            affected_mods: None,
                        });
                        reported_ids.insert(dep_id_lower);
                    }
                } else if !reported_ids.contains(&dep_id_lower) {
                    let display_name = resolve_mod_name(&dep.unique_id);
                    conflicts.push(ConflictReport {
                        mod_name: mod_info.name.clone(),
                        unique_id: dep.unique_id.clone(),
                        conflict_type: ConflictType::OptionalDependencyMissing,
                        description: format!("模组 '{}' 的可选依赖 '{}' 未安装", mod_info.name, display_name),
                        severity: Severity::Info,
                        solution: format!("如需该功能，请安装 '{}' 模组", display_name),
                        affected_mods: None,
                    });
                    reported_ids.insert(dep_id_lower);
                }
            }
        }
    }

    for mod_info in &mods {
        if let Some(patch) = mod_patches::get_missing_dependency(&mod_info.unique_id) {
            let patch_id_lower = patch.missing_id.to_lowercase();
            if !reported_ids.contains(&patch_id_lower) && !installed_ids.contains(&patch_id_lower) {
                let display_name = resolve_mod_name(&patch.missing_id);
                conflicts.push(ConflictReport {
                    mod_name: mod_info.name.clone(),
                    unique_id: patch.missing_id.clone(),
                    conflict_type: ConflictType::HardcodedPatch,
                    description: format!("[内置规则] '{}' 需要 '{}'（原因：{}）", mod_info.name, display_name, patch.reason),
                    severity: Severity::Error,
                    solution: format!("请安装 '{}' 模组", display_name),
                    affected_mods: None,
                });
                reported_ids.insert(patch_id_lower);
            }
        }
    }

    check_content_pack_conflicts(&mods, &mut conflicts);
    check_version_conflicts(&mods, &installed_ids, &mut conflicts);
    check_asset_conflicts(&mods, &mut conflicts);

    Ok(conflicts)
}

fn check_content_pack_conflicts(mods: &[ModInfo], conflicts: &mut Vec<ConflictReport>) {
    let mut content_pack_map: HashMap<String, Vec<String>> = HashMap::new();

    for mod_info in mods {
        if mod_info.is_content_pack {
            if let Some(ref target) = mod_info.content_pack_for {
                content_pack_map
                    .entry(target.clone())
                    .or_default()
                    .push(mod_info.name.clone());
            }
        }
    }

    for (target_id, pack_names) in &content_pack_map {
        if pack_names.len() > 1 {
            let target_display = resolve_mod_name(target_id);
            conflicts.push(ConflictReport {
                mod_name: target_display.clone(),
                unique_id: target_id.clone(),
                conflict_type: ConflictType::ContentPackTargetConflict,
                description: format!(
                    "多个内容包属于同一框架 '{}'：{}",
                    target_display,
                    pack_names.join("、")
                ),
                severity: Severity::Info,
                solution: "这通常不会造成冲突，但如果出现异常，请检查是否有内容包互相覆盖".to_string(),
                affected_mods: Some(pack_names.clone()),
            });
        }
    }
}

fn check_version_conflicts(
    mods: &[ModInfo],
    installed_ids: &HashSet<String>,
    conflicts: &mut Vec<ConflictReport>,
) {
    let mod_versions: HashMap<String, (String, String)> = mods
        .iter()
        .map(|m| (m.unique_id.to_lowercase(), (m.name.clone(), m.version.clone())))
        .collect();

    for mod_info in mods {
        for dep in &mod_info.dependencies {
            if !dep.is_required {
                continue;
            }
            let dep_id_lower = dep.unique_id.to_lowercase();
            if !installed_ids.contains(&dep_id_lower) {
                continue;
            }
            if let Some(ref min_version) = dep.minimum_version {
                if let Some((dep_name, dep_version)) = mod_versions.get(&dep_id_lower) {
                    if crate::update_checker::compare_versions(dep_version, min_version) < 0 {
                        conflicts.push(ConflictReport {
                            mod_name: mod_info.name.clone(),
                            unique_id: dep.unique_id.clone(),
                            conflict_type: ConflictType::VersionConflict,
                            description: format!(
                                "'{}' 需要 '{}' 版本 ≥ {}，但当前安装版本为 {}",
                                mod_info.name, dep_name, min_version, dep_version
                            ),
                            severity: Severity::Warning,
                            solution: format!("请更新 '{}' 到版本 {} 或更高", dep_name, min_version),
                            affected_mods: None,
                        });
                    }
                }
            }
        }
    }
}

fn check_asset_conflicts(mods: &[ModInfo], conflicts: &mut Vec<ConflictReport>) {
    let mut asset_to_mods: HashMap<String, Vec<(String, String)>> = HashMap::new();

    for mod_info in mods {
        if !mod_info.enabled {
            continue;
        }

        let folder_path = PathBuf::from(&mod_info.folder_path);
        scan_mod_assets(&folder_path, &mut asset_to_mods, &mod_info.unique_id, &mod_info.name);
    }

    for (asset, mod_list) in &asset_to_mods {
        if mod_list.len() > 1 {
            let mod_names: Vec<String> = mod_list.iter().map(|(_, name)| name.clone()).collect();
            let is_patch = asset.ends_with(".json") && asset.contains("patches");

            conflicts.push(ConflictReport {
                mod_name: mod_names.first().unwrap_or(&"".to_string()).clone(),
                unique_id: mod_list.first().map(|(id, _)| id.clone()).unwrap_or_default(),
                conflict_type: ConflictType::AssetConflict,
                description: if is_patch {
                    format!("多个模组修改了同一补丁文件 '{}'：{}", asset, mod_names.join("、"))
                } else {
                    format!("多个模组包含相同资产 '{}'：{}", asset, mod_names.join("、"))
                },
                severity: if is_patch { Severity::Warning } else { Severity::Info },
                solution: if is_patch {
                    "补丁文件冲突可能导致游戏异常，请检查这些模组是否兼容".to_string()
                } else {
                    "资产文件重复通常不会造成问题，后加载的模组会覆盖前者".to_string()
                },
                affected_mods: Some(mod_names),
            });
        }
    }
}

fn scan_mod_assets(
    folder_path: &PathBuf,
    asset_to_mods: &mut HashMap<String, Vec<(String, String)>>,
    unique_id: &str,
    mod_name: &str,
) {
    let assets_dir = folder_path.join("assets");
    if assets_dir.exists() {
        collect_asset_paths(&assets_dir, &assets_dir, asset_to_mods, unique_id, mod_name);
    }
}

fn collect_asset_paths(
    dir: &PathBuf,
    base_dir: &PathBuf,
    asset_to_mods: &mut HashMap<String, Vec<(String, String)>>,
    unique_id: &str,
    mod_name: &str,
) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_asset_paths(&path, base_dir, asset_to_mods, unique_id, mod_name);
            } else if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if matches!(ext_str.as_str(), "json" | "png" | "xnb" | "tbin" | "tmx" | "yaml" | "yml") {
                        if let Ok(relative) = path.strip_prefix(base_dir) {
                            let asset_key = relative.to_string_lossy().to_string().replace('\\', "/");
                            asset_to_mods
                                .entry(asset_key)
                                .or_default()
                                .push((unique_id.to_string(), mod_name.to_string()));
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mod_parser::ModDependencyInfo;

    fn make_mod(name: &str, unique_id: &str, deps: Vec<ModDependencyInfo>) -> ModInfo {
        ModInfo {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            author: "TestAuthor".to_string(),
            description: String::new(),
            unique_id: unique_id.to_string(),
            enabled: true,
            is_required: false,
            has_dependencies: !deps.is_empty(),
            dependency_count: deps.len(),
            is_content_pack: false,
            content_pack_for: None,
            folder_path: format!("C:/Mods/{}", name),
            has_conflict: false,
            conflict_warning: None,
            url: None,
            category: "other".to_string(),
            screenshot_path: None,
            thumbnail_path: None,
            has_update: false,
            latest_version: None,
            update_url: None,
            dependencies: deps,
            manifest_content: None,
            sub_mods: vec![],
            is_group: false,
            internal_component_ids: vec![],
            nexus_mod_id: None,
        }
    }

    #[test]
    fn test_check_conflicts_case_insensitive_dependency_match() {
        let dep = ModDependencyInfo {
            unique_id: "test.dependency".to_string(),
            minimum_version: None,
            is_required: true,
        };
        let mod_a = make_mod("Mod A", "Test.ModA", vec![dep]);
        let mod_b = make_mod("Mod B", "Test.Dependency", vec![]);

        let conflicts = check_conflicts(vec![mod_a, mod_b]).unwrap();
        let missing: Vec<&ConflictReport> = conflicts.iter()
            .filter(|c| matches!(c.conflict_type, ConflictType::MissingDependency))
            .collect();
        assert!(missing.is_empty(), "Should not report missing dependency when case differs: {:?}", missing);
    }

    #[test]
    fn test_check_conflicts_reports_missing_dependency() {
        let dep = ModDependencyInfo {
            unique_id: "Test.MissingMod".to_string(),
            minimum_version: None,
            is_required: true,
        };
        let mod_a = make_mod("Mod A", "Test.ModA", vec![dep]);

        let conflicts = check_conflicts(vec![mod_a]).unwrap();
        let missing: Vec<&ConflictReport> = conflicts.iter()
            .filter(|c| matches!(c.conflict_type, ConflictType::MissingDependency))
            .collect();
        assert_eq!(missing.len(), 1, "Should report missing dependency");
    }

    #[test]
    fn test_check_conflicts_optional_dependency_not_reported_as_error() {
        let dep = ModDependencyInfo {
            unique_id: "Test.OptionalMod".to_string(),
            minimum_version: None,
            is_required: false,
        };
        let mod_a = make_mod("Mod A", "Test.ModA", vec![dep]);

        let conflicts = check_conflicts(vec![mod_a]).unwrap();
        let errors: Vec<&ConflictReport> = conflicts.iter()
            .filter(|c| matches!(c.conflict_type, ConflictType::MissingDependency))
            .collect();
        assert!(errors.is_empty(), "Optional dependency should not be reported as MissingDependency error");
    }
}
