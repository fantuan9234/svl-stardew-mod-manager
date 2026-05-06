use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    MissingDependency,
    OptionalDependencyMissing,
    ContentPackConflict,
    Incompatibility,
    HardcodedPatch,
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

    let installed_ids: HashSet<String> = mods.iter().map(|m| m.unique_id.clone()).collect();

    for mod_info in &mods {
        for dep in &mod_info.dependencies {
            if !installed_ids.contains(&dep.unique_id) {
                if dep.is_required {
                    if !reported_ids.contains(&dep.unique_id) {
                        let display_name = resolve_mod_name(&dep.unique_id);
                        conflicts.push(ConflictReport {
                            mod_name: mod_info.name.clone(),
                            unique_id: dep.unique_id.clone(),
                            conflict_type: ConflictType::MissingDependency,
                            description: format!("模组 '{}' 需要依赖 '{}'，但未安装", mod_info.name, display_name),
                            severity: Severity::Error,
                            solution: format!("请安装 '{}' 模组", display_name),
                        });
                        reported_ids.insert(dep.unique_id.clone());
                    }
                } else {
                    if !reported_ids.contains(&dep.unique_id) {
                        let display_name = resolve_mod_name(&dep.unique_id);
                        conflicts.push(ConflictReport {
                            mod_name: mod_info.name.clone(),
                            unique_id: dep.unique_id.clone(),
                            conflict_type: ConflictType::OptionalDependencyMissing,
                            description: format!("模组 '{}' 可选依赖 '{}' 未安装（不影响运行）", mod_info.name, display_name),
                            severity: Severity::Info,
                            solution: format!("如需完整功能，请安装 '{}' 模组", display_name),
                        });
                        reported_ids.insert(dep.unique_id.clone());
                    }
                }
            }
        }
    }

    for mod_info in &mods {
        if let Some(patch) = mod_patches::get_missing_dependency(&mod_info.unique_id) {
            if !reported_ids.contains(&patch.missing_id) && !installed_ids.contains(&patch.missing_id) {
                let display_name = resolve_mod_name(&patch.missing_id);
                conflicts.push(ConflictReport {
                    mod_name: mod_info.name.clone(),
                    unique_id: patch.missing_id.clone(),
                    conflict_type: ConflictType::HardcodedPatch,
                    description: format!("[内置规则] '{}' 需要 '{}'（原因：{}）", mod_info.name, display_name, patch.reason),
                    severity: Severity::Error,
                    solution: format!("请安装 '{}' 模组", display_name),
                });
                reported_ids.insert(patch.missing_id.clone());
            }
        }
    }

    Ok(conflicts)
}
