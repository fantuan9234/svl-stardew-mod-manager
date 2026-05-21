use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use crate::mod_name_resolver::resolve_mod_name;
use crate::nexus_linker::build_nexus_link;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingDependency {
    pub unique_id: String,
    pub display_name: String,
    pub is_required: bool,
    pub minimum_version: Option<String>,
    pub nexus_mod_id: Option<String>,
    pub nexus_url: String,
    pub required_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyScanResult {
    pub total_installed: usize,
    pub total_missing: usize,
    pub missing_dependencies: Vec<MissingDependency>,
}

#[tauri::command]
pub async fn scan_all_missing_dependencies(
    mods_path: String,
) -> Result<DependencyScanResult, String> {
    tokio::task::spawn_blocking(move || {
        scan_all_missing_dependencies_blocking(&mods_path)
    })
    .await
    .map_err(|e| format!("依赖扫描执行失败: {}", e))?
}

fn scan_all_missing_dependencies_blocking(mods_path: &str) -> Result<DependencyScanResult, String> {
    let mods_dir = PathBuf::from(mods_path);
    if !mods_dir.exists() {
        return Err("MOD 目录不存在".to_string());
    }

    let builtin_ids: HashSet<String> = [
        "Pathoschild.SMAPI",
        "Pathoschild.SMAPI.ConsoleCommands",
        "Pathoschild.SMAPI.ErrorHandler",
        "Pathoschild.SMAPI.SaveBackup",
        "StardewValley.GameData",
        "StardewValley",
    ].iter().map(|s| s.to_lowercase()).collect();

    let mut installed_ids: HashSet<String> = HashSet::new();
    let mut mod_manifests: HashMap<String, serde_json::Value> = HashMap::new();
    let mut content_pack_parents: HashSet<String> = HashSet::new();

    let manifest_dirs = crate::mod_parser::recursive_find_manifests(&mods_dir);
    for dir in &manifest_dirs {
        let manifest_path = dir.join("manifest.json");
        let dot_manifest = dir.join(".manifest.json");

        for mf in [&manifest_path, &dot_manifest] {
            if mf.exists() {
                if let Ok(content) = fs::read_to_string(mf) {
                    let cleaned = crate::mod_parser::remove_trailing_commas(&content);
                    let cleaned = cleaned.strip_prefix('\u{FEFF}').unwrap_or(&cleaned);
                    if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&cleaned) {
                        if let Some(cpf) = manifest["ContentPackFor"]["UniqueID"].as_str() {
                            content_pack_parents.insert(cpf.to_lowercase());
                        }
                        if let Some(uid) = manifest["UniqueID"].as_str() {
                            let uid_lower = uid.to_lowercase();
                            installed_ids.insert(uid_lower.clone());
                            mod_manifests.insert(uid_lower, manifest);
                        }
                    }
                }
            }
        }
    }

    let mut missing_map: HashMap<String, MissingDependency> = HashMap::new();

    for (_uid, manifest) in &mod_manifests {
        let mod_name = manifest["Name"].as_str().unwrap_or("未知").to_string();

        if let Some(deps) = manifest["Dependencies"].as_array() {
            for dep in deps {
                let dep_id = dep["UniqueID"].as_str().unwrap_or("").to_string();
                if dep_id.is_empty() {
                    continue;
                }

                let dep_id_lower = dep_id.to_lowercase();

                if builtin_ids.contains(&dep_id_lower) {
                    continue;
                }

                if dep_id_lower.starts_with("pathoschild.smapi.") {
                    continue;
                }

                if content_pack_parents.contains(&dep_id_lower) {
                    continue;
                }

                let is_required = dep["IsRequired"].as_bool().unwrap_or(false);
                if !is_required {
                    continue;
                }
                let min_version = dep["MinimumVersion"].as_str().map(|s| s.to_string());

                if !installed_ids.contains(&dep_id_lower) {
                    let display_name = resolve_mod_name(&dep_id);
                    let link = build_nexus_link(&dep_id, Some(&display_name), None);

                    missing_map
                        .entry(dep_id_lower.clone())
                        .and_modify(|existing| {
                            existing.required_by.push(mod_name.clone());
                        })
                        .or_insert_with(|| MissingDependency {
                            unique_id: dep_id,
                            display_name,
                            is_required,
                            minimum_version: min_version,
                            nexus_mod_id: link.mod_id,
                            nexus_url: link.url,
                            required_by: vec![mod_name.clone()],
                        });
                }
            }
        }
    }

    let mut missing: Vec<MissingDependency> = missing_map.into_values().collect();
    missing.sort_by(|a, b| {
        b.is_required.cmp(&a.is_required)
            .then(a.display_name.to_lowercase().cmp(&b.display_name.to_lowercase()))
    });

    let total_installed = installed_ids.len();
    let total_missing = missing.len();

    Ok(DependencyScanResult {
        total_installed,
        total_missing,
        missing_dependencies: missing,
    })
}

#[tauri::command]
pub async fn auto_install_missing_dependency(
    app: tauri::AppHandle,
    unique_id: String,
    nexus_mod_id: Option<String>,
    mods_path: String,
    api_key: String,
) -> Result<DependencyInstallResult, String> {
    let resolved_mod_id = match nexus_mod_id {
        Some(id) if !id.is_empty() => id,
        _ => {
            let display_name = resolve_mod_name(&unique_id);
            let link = build_nexus_link(&unique_id, Some(&display_name), None);
            match link.mod_id {
                Some(id) => id,
                None => {
                    return Err(format!(
                        "无法找到 '{}' 的 Nexus Mods ID，请手动搜索安装",
                        display_name
                    ));
                }
            }
        }
    };

    let result = crate::nexus_api::download_mod_from_nexus(
        app,
        resolved_mod_id.clone(),
        api_key,
        Some(mods_path),
        None,
        None,
    )
    .await
    .map_err(|e| format!("下载依赖失败: {}", e))?;

    let mod_name = result.mod_name.clone();
    Ok(DependencyInstallResult {
        success: result.success,
        mod_name: result.mod_name,
        message: if result.success {
            format!("依赖 '{}' 安装成功", mod_name)
        } else {
            format!("依赖安装失败: {}", result.message)
        },
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInstallResult {
    pub success: bool,
    pub mod_name: String,
    pub message: String,
}
