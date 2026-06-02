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
pub struct OutdatedDependency {
    pub unique_id: String,
    pub display_name: String,
    pub installed_version: String,
    pub minimum_version: String,
    pub required_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyScanResult {
    pub total_installed: usize,
    pub total_missing: usize,
    pub missing_dependencies: Vec<MissingDependency>,
    pub outdated_dependencies: Vec<OutdatedDependency>,
}

#[tauri::command]
pub async fn scan_all_missing_dependencies(
    mods_path: String,
) -> Result<DependencyScanResult, String> {
    tokio::task::spawn_blocking(move || {
        crate::compatibility_list::ensure_loaded_sync();
        crate::smapi_data::ensure_loaded_sync();
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
    let mut mod_versions: HashMap<String, String> = HashMap::new();

    let manifest_dirs = crate::mod_parser::recursive_find_manifests(&mods_dir);
    for dir in &manifest_dirs {
        let manifest_path = dir.join("manifest.json");
        let dot_manifest = dir.join(".manifest.json");

        for mf in [&manifest_path, &dot_manifest] {
            if mf.exists() {
                if let Ok(content) = fs::read_to_string(mf) {
                    let normalized = crate::mod_parser::normalize_smart_quotes(&content);
                    let no_comments = crate::mod_parser::strip_json_comments(&normalized);
                    let cleaned = crate::mod_parser::remove_trailing_commas(&no_comments);
                    let cleaned = cleaned.strip_prefix('\u{FEFF}').unwrap_or(&cleaned);
                    if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&cleaned) {
                        let uid_value = manifest.get("UniqueID")
                            .or_else(|| manifest.get("UniqueId"));
                        if let Some(uid) = uid_value.and_then(|v| v.as_str()) {
                            let uid_lower = uid.to_lowercase();
                            installed_ids.insert(uid_lower.clone());
                            let ver = manifest.get("Version").and_then(|v| match v {
                                serde_json::Value::String(s) => Some(s.clone()),
                                serde_json::Value::Number(n) => Some(n.to_string()),
                                _ => None,
                            });
                            if let Some(ver) = ver {
                                mod_versions.insert(uid_lower.clone(), ver);
                            }
                            mod_manifests.insert(uid_lower, manifest);
                        }
                    }
                }
            }
        }
    }

    let mut missing_map: HashMap<String, MissingDependency> = HashMap::new();
    let mut outdated_map: HashMap<String, OutdatedDependency> = HashMap::new();

    for (_uid, manifest) in &mod_manifests {
        let mod_name = manifest["Name"].as_str().unwrap_or("未知").to_string();

        if let Some(deps) = manifest["Dependencies"].as_array() {
            for dep in deps {
                let dep_id = dep.get("UniqueID")
                    .or_else(|| dep.get("UniqueId"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
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

                let is_required = dep["IsRequired"].as_bool().unwrap_or(true);
                if !is_required {
                    continue;
                }
                let min_version = dep.get("MinimumVersion").and_then(|v| match v {
                    serde_json::Value::String(s) => Some(s.clone()),
                    serde_json::Value::Number(n) => Some(n.to_string()),
                    _ => None,
                });

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
                } else if let Some(ref min_ver) = min_version {
                    if !min_ver.is_empty() {
                        if let Some(installed_ver) = mod_versions.get(&dep_id_lower) {
                            if crate::update_checker::compare_versions(installed_ver, min_ver) < 0 {
                                let display_name = resolve_mod_name(&dep_id);
                                outdated_map
                                    .entry(dep_id_lower.clone())
                                    .and_modify(|existing| {
                                        existing.required_by.push(mod_name.clone());
                                    })
                                    .or_insert_with(|| OutdatedDependency {
                                        unique_id: dep_id,
                                        display_name,
                                        installed_version: installed_ver.clone(),
                                        minimum_version: min_ver.clone(),
                                        required_by: vec![mod_name.clone()],
                                    });
                            }
                        }
                    }
                }
            }
        }

        let cpf_value = manifest.get("ContentPackFor")
            .and_then(|cpf| cpf.get("UniqueID").or_else(|| cpf.get("UniqueId")));
        if let Some(cpf) = cpf_value.and_then(|v| v.as_str()) {
            let cpf_lower = cpf.to_lowercase();
            if !builtin_ids.contains(&cpf_lower)
                && !cpf_lower.starts_with("pathoschild.smapi.")
                && !installed_ids.contains(&cpf_lower)
                && !missing_map.contains_key(&cpf_lower)
            {
                let display_name = resolve_mod_name(cpf);
                let link = build_nexus_link(cpf, Some(&display_name), None);

                let cpf_min_ver = manifest.get("ContentPackFor")
                    .and_then(|cpf| cpf.get("MinimumVersion"))
                    .and_then(|v| match v {
                        serde_json::Value::String(s) => Some(s.clone()),
                        serde_json::Value::Number(n) => Some(n.to_string()),
                        _ => None,
                    });

                missing_map.insert(cpf_lower.clone(), MissingDependency {
                    unique_id: cpf.to_string(),
                    display_name,
                    is_required: true,
                    minimum_version: cpf_min_ver,
                    nexus_mod_id: link.mod_id,
                    nexus_url: link.url,
                    required_by: vec![mod_name],
                });
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
        outdated_dependencies: outdated_map.into_values().collect(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_temp_mods_dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    fn write_manifest(dir: &std::path::Path, manifest_json: &str) {
        let manifest_path = dir.join("manifest.json");
        let mut f = std::fs::File::create(&manifest_path).unwrap();
        f.write_all(manifest_json.as_bytes()).unwrap();
    }

    #[test]
    fn test_smart_quotes_in_manifest_dependencies() {
        let tmp = create_temp_mods_dir();
        let mod_a_dir = tmp.path().join("ModA");
        let mod_b_dir = tmp.path().join("ModB");
        std::fs::create_dir_all(&mod_a_dir).unwrap();
        std::fs::create_dir_all(&mod_b_dir).unwrap();

        write_manifest(&mod_a_dir, r#"{
            "Name": "Mod A",
            "UniqueID": "Test.ModA",
            "Version": "1.0.0",
            "Dependencies": [
                {
                    "UniqueID": "Test.ModB",
                    "IsRequired": true
                }
            ]
        }"#);

        let smart_quote_manifest = format!("{{
            \u{201C}Name\u{201D}: \u{201C}Mod B\u{201D},
            \u{201C}UniqueID\u{201D}: \u{201C}Test.ModB\u{201D},
            \u{201C}Version\u{201D}: \u{201C}1.0.0\u{201D}
        }}");
        write_manifest(&mod_b_dir, &smart_quote_manifest);

        let result = scan_all_missing_dependencies_blocking(tmp.path().to_str().unwrap());
        assert!(result.is_ok(), "Should parse manifest with smart quotes");
        let scan = result.unwrap();
        assert_eq!(scan.total_missing, 0, "ModB should be found even with smart quotes in manifest");
    }

    #[test]
    fn test_is_required_defaults_to_true() {
        let tmp = create_temp_mods_dir();
        let mod_a_dir = tmp.path().join("ModA");
        std::fs::create_dir_all(&mod_a_dir).unwrap();

        write_manifest(&mod_a_dir, r#"{
            "Name": "Mod A",
            "UniqueID": "Test.ModA",
            "Version": "1.0.0",
            "Dependencies": [
                {
                    "UniqueID": "Test.MissingMod"
                }
            ]
        }"#);

        let result = scan_all_missing_dependencies_blocking(tmp.path().to_str().unwrap());
        assert!(result.is_ok());
        let scan = result.unwrap();
        assert_eq!(scan.total_missing, 1, "Dependency without IsRequired should default to true (required)");
        assert!(scan.missing_dependencies[0].is_required, "IsRequired should default to true");
    }

    #[test]
    fn test_content_pack_for_missing_parent() {
        let tmp = create_temp_mods_dir();
        let content_pack_dir = tmp.path().join("ContentPackForA");
        std::fs::create_dir_all(&content_pack_dir).unwrap();

        write_manifest(&content_pack_dir, r#"{
            "Name": "Content Pack For A",
            "UniqueID": "Test.ContentPackA",
            "Version": "1.0.0",
            "ContentPackFor": {
                "UniqueID": "Test.ParentMod"
            }
        }"#);

        let result = scan_all_missing_dependencies_blocking(tmp.path().to_str().unwrap());
        assert!(result.is_ok());
        let scan = result.unwrap();
        let found_missing = scan.missing_dependencies.iter()
            .any(|d| d.unique_id == "Test.ParentMod");
        assert!(found_missing, "ContentPackFor parent should be reported as missing dependency");
    }

    #[test]
    fn test_outdated_dependency_detected() {
        let tmp = create_temp_mods_dir();
        let mod_a_dir = tmp.path().join("ModA");
        let mod_b_dir = tmp.path().join("ModB");
        std::fs::create_dir_all(&mod_a_dir).unwrap();
        std::fs::create_dir_all(&mod_b_dir).unwrap();

        write_manifest(&mod_a_dir, r#"{
            "Name": "Mod A",
            "UniqueID": "Test.ModA",
            "Version": "1.0.0",
            "Dependencies": [
                {
                    "UniqueID": "Test.ModB",
                    "IsRequired": true,
                    "MinimumVersion": "2.0.0"
                }
            ]
        }"#);

        write_manifest(&mod_b_dir, r#"{
            "Name": "Mod B",
            "UniqueID": "Test.ModB",
            "Version": "1.5.0"
        }"#);

        let result = scan_all_missing_dependencies_blocking(tmp.path().to_str().unwrap());
        assert!(result.is_ok());
        let scan = result.unwrap();
        assert_eq!(scan.total_missing, 0, "ModB is installed, should not be missing");
        assert_eq!(scan.outdated_dependencies.len(), 1, "ModB version 1.5.0 < 2.0.0, should be outdated");
        let outdated = &scan.outdated_dependencies[0];
        assert_eq!(outdated.unique_id, "Test.ModB");
        assert_eq!(outdated.installed_version, "1.5.0");
        assert_eq!(outdated.minimum_version, "2.0.0");
    }

    #[test]
    fn test_manifest_with_comments_and_numeric_version() {
        let tmp = create_temp_mods_dir();
        let mod_a_dir = tmp.path().join("ModA");
        let mod_b_dir = tmp.path().join("ModB");
        std::fs::create_dir_all(&mod_a_dir).unwrap();
        std::fs::create_dir_all(&mod_b_dir).unwrap();

        write_manifest(&mod_a_dir, r#"{
            // This is a comment
            "Name": "Mod A",
            "UniqueID": "Test.ModA",
            "Version": 2,
            "Dependencies": [
                {
                    "UniqueID": "Test.ModB",
                    "IsRequired": true
                }
            ]
        }"#);

        write_manifest(&mod_b_dir, r#"{
            /* Another comment */
            "Name": "Mod B",
            "UniqueId": "Test.ModB",
            "Version": 1.5
        }"#);

        let result = scan_all_missing_dependencies_blocking(tmp.path().to_str().unwrap());
        assert!(result.is_ok(), "Should parse manifests with comments");
        let scan = result.unwrap();
        assert_eq!(scan.total_installed, 2, "Both mods should be found");
        assert_eq!(scan.total_missing, 0, "No missing dependencies");
    }
}
