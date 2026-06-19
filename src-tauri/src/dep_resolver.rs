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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModPrerequisite {
    pub unique_id: String,
    pub display_name: String,
    pub is_required: bool,
    pub minimum_version: Option<String>,
    pub nexus_mod_id: Option<String>,
    pub nexus_url: Option<String>,
    pub is_installed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrerequisiteCheckResult {
    pub mod_name: String,
    pub prerequisites: Vec<ModPrerequisite>,
    pub missing_count: usize,
}

#[tauri::command]
pub async fn check_mod_prerequisites(
    api_key: String,
    mod_id: String,
) -> Result<PrerequisiteCheckResult, String> {
    eprintln!("[prereq] check_mod_prerequisites called for mod_id={}", mod_id);
    let client = crate::nexus_api::build_nexus_async_client();

    let mod_info = match crate::nexus_api::get_nexus_mod_info_async(&client, &api_key, &mod_id).await {
        Ok(info) => info,
        Err(e) => {
            eprintln!("[prereq] get_nexus_mod_info_async failed: {}", e);
            return Err(format!("获取 MOD 信息失败: {}", e));
        }
    };
    eprintln!("[prereq] mod_info: name={}, mod_id={}", mod_info.name, mod_id);

    let file_result = crate::nexus_api::fetch_mod_file_requirements(&api_key, &mod_id).await;
    let file_deps_count = file_result
        .as_ref()
        .map(|r| r.primary_requirements.len())
        .unwrap_or(0);
    eprintln!(
        "[prereq] fetch_mod_file_requirements: ok={}, primary_requirements_count={}",
        file_result.is_ok(),
        file_deps_count
    );

    let mut dependencies = match &file_result {
        Ok(fr) if !fr.primary_requirements.is_empty() => {
            let converted = convert_file_requirements_to_deps(&fr.primary_requirements);
            eprintln!("[prereq] using N网 modRequirements, converted {} deps", converted.len());
            converted
        }
        _ => {
            eprintln!("[prereq] falling back to description parsing");
            let description = crate::nexus_api::fetch_nexus_mod_description(api_key.clone(), mod_id.clone()).await
                .unwrap_or_default();
            eprintln!("[prereq] description length: {} chars", description.len());
            let parsed = extract_dependencies_from_description(&description, &mod_info.name);
            eprintln!("[prereq] description parser returned {} deps", parsed.len());
            parsed
        }
    };

    if dependencies.is_empty() {
        if let Ok(fr) = &file_result {
            if fr.primary_requirements.is_empty() {
                eprintln!("[prereq] primary_requirements was empty, files_listing_count={}", fr.files.len());
                for f in fr.files.iter().take(3) {
                    eprintln!(
                        "[prereq]   file id={} name={} is_primary={} reqs={}",
                        f.file_id, f.name, f.is_primary, f.requirements.len()
                    );
                }
            }
        }
        let description = crate::nexus_api::fetch_nexus_mod_description(api_key.clone(), mod_id.clone()).await
            .unwrap_or_default();
        dependencies = extract_dependencies_from_description(&description, &mod_info.name);
        eprintln!("[prereq] second-pass description parser returned {} deps", dependencies.len());
    }

    if dependencies.is_empty() {
        if let Some(builtin_prereqs) = lookup_popular_mod_prereqs(&mod_id) {
            eprintln!("[prereq] falling back to built-in popular mod table, found {} prereqs for mod_id={}", builtin_prereqs.len(), mod_id);
            for (uid, display) in builtin_prereqs {
                let mut known_entry = lookup_known_mod_by_unique_id(uid);
                let (final_uid, final_display, final_mod_id) = if let Some((u, d, m)) = known_entry {
                    (u.to_string(), d.to_string(), Some(m.to_string()))
                } else {
                    (uid.to_string(), display.to_string(), None)
                };
                dependencies.push(DependencyEntry {
                    unique_id: final_uid,
                    display_name: Some(final_display),
                    nexus_mod_id: final_mod_id,
                    is_required: true,
                    minimum_version: None,
                });
            }
        } else {
            eprintln!("[prereq] no built-in prereqs for mod_id={}", mod_id);
        }
    }

    let mods_path = {
        if let Some((detected_path, _method)) = crate::smapi::find_game_path() {
            detected_path.join("Mods").to_string_lossy().to_string()
        } else {
            String::new()
        }
    };

    let installed_ids: HashSet<String> = if !mods_path.is_empty() {
        let mods_dir = PathBuf::from(&mods_path);
        let manifest_dirs = crate::mod_parser::recursive_find_manifests(&mods_dir);
        let mut ids = HashSet::new();
        for dir in &manifest_dirs {
            let manifest_path = dir.join("manifest.json");
            if let Ok(content) = fs::read_to_string(&manifest_path) {
                let normalized = crate::mod_parser::normalize_smart_quotes(&content);
                let no_comments = crate::mod_parser::strip_json_comments(&normalized);
                let cleaned = crate::mod_parser::remove_trailing_commas(&no_comments);
                let cleaned = cleaned.strip_prefix('\u{FEFF}').unwrap_or(&cleaned);
                if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&cleaned) {
                    if let Some(uid) = manifest.get("UniqueID").or_else(|| manifest.get("UniqueId")).and_then(|v| v.as_str()) {
                        ids.insert(uid.to_lowercase());
                    }
                }
            }
        }
        ids
    } else {
        HashSet::new()
    };

    let builtin_ids: HashSet<String> = [
        "Pathoschild.SMAPI",
        "Pathoschild.SMAPI.ConsoleCommands",
        "Pathoschild.SMAPI.ErrorHandler",
        "Pathoschild.SMAPI.SaveBackup",
        "StardewValley.GameData",
        "StardewValley",
    ].iter().map(|s| s.to_lowercase()).collect();

    let mut prerequisites: Vec<ModPrerequisite> = Vec::new();
    let mut seen_ids: HashSet<String> = HashSet::new();

    for dep in &dependencies {
        let dep_id_lower = dep.unique_id.to_lowercase();

        if builtin_ids.contains(&dep_id_lower) || dep_id_lower.starts_with("pathoschild.smapi.") {
            continue;
        }

        if seen_ids.contains(&dep_id_lower) {
            continue;
        }
        seen_ids.insert(dep_id_lower.clone());

        let is_installed = installed_ids.contains(&dep_id_lower);

        let display_name = if let Some(name) = &dep.display_name {
            name.clone()
        } else {
            resolve_mod_name(&dep.unique_id)
        };

        let mut nexus_mod_id = dep.nexus_mod_id.clone();
        if nexus_mod_id.is_none() {
            if let Some((_, _, mod_id_known)) = lookup_known_mod_by_unique_id(&dep.unique_id) {
                nexus_mod_id = Some(mod_id_known.to_string());
            }
        }
        if nexus_mod_id.is_none() {
            if let Some(known) = lookup_known_mod_by_display_name(&display_name) {
                nexus_mod_id = Some(known.2.to_string());
            }
        }

        let nexus_url = nexus_mod_id.as_ref().map(|id| {
            format!("https://www.nexusmods.com/stardewvalley/mods/{}", id)
        });

        prerequisites.push(ModPrerequisite {
            unique_id: dep.unique_id.clone(),
            display_name,
            is_required: dep.is_required,
            minimum_version: dep.minimum_version.clone(),
            nexus_mod_id,
            nexus_url,
            is_installed,
        });
    }

    let missing_count = prerequisites.iter().filter(|p| !p.is_installed).count();

    Ok(PrerequisiteCheckResult {
        mod_name: mod_info.name,
        prerequisites,
        missing_count,
    })
}

fn convert_file_requirements_to_deps(
    requirements: &[crate::nexus_api::NexusFileRequirement],
) -> Vec<DependencyEntry> {
    let mut deps: Vec<DependencyEntry> = Vec::new();
    let mut processed: HashSet<String> = HashSet::new();

    for req in requirements {
        let unique_id = if let Some(uid) = &req.unique_id {
            uid.clone()
        } else if let Some(mod_id_str) = &req.mod_id {
            format!("Unknown.Mod{}", mod_id_str)
        } else {
            let guessed = guess_unique_id_from_name(&req.name);
            if guessed == "Unknown.Unknown" {
                continue;
            }
            guessed
        };

        let lower = unique_id.to_lowercase();
        if processed.contains(&lower) {
            continue;
        }
        if is_junk_unique_id(&lower) {
            continue;
        }
        processed.insert(lower);

        deps.push(DependencyEntry {
            unique_id,
            display_name: Some(req.name.clone()),
            nexus_mod_id: req.mod_id.clone(),
            is_required: !req.optional,
            minimum_version: req.version.clone(),
        });
    }

    deps
}

#[derive(Debug, Clone)]
struct DependencyEntry {
    unique_id: String,
    display_name: Option<String>,
    nexus_mod_id: Option<String>,
    is_required: bool,
    minimum_version: Option<String>,
}

fn known_prerequisite_mods() -> &'static [(&'static str, &'static str, &'static str)] {
    &[
        ("Pathoschild.SMAPI", "SMAPI - Stardew Modding API", "2400"),
        ("Pathoschild.ContentPatcher", "Content Patcher", "1915"),
        ("spacechase0.SpaceCore", "SpaceCore", "1348"),
        ("spacechase0.JsonAssets", "Json Assets", "1030"),
        ("spacechase0.DynamicGameAssets", "Dynamic Game Assets", "4590"),
        ("Cherry.ExpandedPreconditionsUtility", "Expanded Preconditions Utility", "6529"),
        ("spacechase0.GenericModConfigMenu", "Generic Mod Config Menu", "5098"),
        ("Digus.FarmTypeManager", "Farm Type Manager", "323"),
        ("Bouhm.NPCMapLocations", "NPC Map Locations", "239"),
        ("PeacefulEnd.CustomCompanions", "Custom Companions", "8622"),
        ("Pathoschild.LookupAnything", "Lookup Anything", "541"),
        ("mushymato.EventLookup", "Event Lookup", "10833"),
        ("DIGUS.MailFrameworkMod", "Mail Framework Mod", "6076"),
        ("spacechase0.CustomNPCFixes", "Custom NPC Fixes", "4049"),
        ("FlashShifter.StardewValleyExpandedCP", "Stardew Valley Expanded", "3753"),
        ("Rafseazz.RidgesideVillage", "Ridgeside Village", "7286"),
        ("LemurKat.EastScarp", "East Scarp", "5787"),
        ("Paritee.BetterFarmAnimalVariety", "Better Farm Animal Variety", "2102"),
        ("Digus.ProducerFrameworkMod", "Producer Framework Mod", "4970"),
        ("furyx639.ObjectTimeLeft", "Object Time Left", "10756"),
        ("Cherry.ShopTileFramework", "Shop Tile Framework", "7931"),
        ("furyx639.TileSheetManager", "Tile Sheet Manager", "5445"),
        ("furyx639.HxD", "Halt x Days", "1240"),
        ("furyx639.UnlimitedStorage", "Unlimited Storage", "3057"),
        ("CJBok.CheatsMenu", "CJB Cheats Menu", "9"),
        ("CJBok.ItemSpawner", "CJB Item Spawner", "93"),
        ("CJBok.ShowItemSellPrice", "CJB Show Item Sell Price", "5"),
        ("Entoarox.EntoaroxFramework", "Entoarox Framework", "2088"),
        ("spacechase0.MoreRings", "More Rings", "813"),
        ("Platonymous.Toolkit", "Platonymous Toolkit", "1723"),
        ("Mizzion.FurnitureFramework", "Furniture Framework", "1496"),
        ("cat.harmony", "Harmony", "10216"),
    ]
}

fn lookup_known_mod_by_display_name(name: &str) -> Option<&'static (&'static str, &'static str, &'static str)> {
    let lower = name.to_lowercase();
    known_prerequisite_mods().iter().find(|(uid, display, _)| {
        display.to_lowercase() == lower || uid.to_lowercase() == lower
    })
}

fn lookup_known_mod_by_unique_id(uid: &str) -> Option<&'static (&'static str, &'static str, &'static str)> {
    let lower = uid.to_lowercase();
    known_prerequisite_mods().iter().find(|(uid_, _, _)| uid_.to_lowercase() == lower)
}

fn known_popular_mod_prereqs() -> &'static [(&'static str, &'static [(&'static str, &'static str)])] {
    &[
        ("3753", &[
            ("Pathoschild.SMAPI", "SMAPI - Stardew Modding API"),
            ("Pathoschild.ContentPatcher", "Content Patcher"),
            ("spacechase0.SpaceCore", "SpaceCore"),
            ("spacechase0.GenericModConfigMenu", "Generic Mod Config Menu"),
        ]),
        ("7286", &[
            ("Pathoschild.SMAPI", "SMAPI - Stardew Modding API"),
            ("Pathoschild.ContentPatcher", "Content Patcher"),
            ("spacechase0.SpaceCore", "SpaceCore"),
            ("spacechase0.GenericModConfigMenu", "Generic Mod Config Menu"),
            ("spacechase0.CustomNPCFixes", "Custom NPC Fixes"),
            ("Digus.FarmTypeManager", "Farm Type Manager"),
            ("Bouhm.NPCMapLocations", "NPC Map Locations"),
            ("Digus.ProducerFrameworkMod", "Producer Framework Mod"),
            ("DIGUS.MailFrameworkMod", "Mail Framework Mod"),
            ("Cherry.ShopTileFramework", "Shop Tile Framework"),
        ]),
        ("5787", &[
            ("Pathoschild.SMAPI", "SMAPI - Stardew Modding API"),
            ("Pathoschild.ContentPatcher", "Content Patcher"),
            ("spacechase0.SpaceCore", "SpaceCore"),
            ("spacechase0.GenericModConfigMenu", "Generic Mod Config Menu"),
            ("spacechase0.CustomNPCFixes", "Custom NPC Fixes"),
        ]),
        ("9333", &[
            ("Pathoschild.SMAPI", "SMAPI - Stardew Modding API"),
            ("Pathoschild.ContentPatcher", "Content Patcher"),
            ("spacechase0.SpaceCore", "SpaceCore"),
            ("spacechase0.GenericModConfigMenu", "Generic Mod Config Menu"),
        ]),
        ("1915", &[
            ("Pathoschild.SMAPI", "SMAPI - Stardew Modding API"),
        ]),
        ("1348", &[
            ("Pathoschild.SMAPI", "SMAPI - Stardew Modding API"),
        ]),
        ("1030", &[
            ("Pathoschild.SMAPI", "SMAPI - Stardew Modding API"),
            ("Pathoschild.ContentPatcher", "Content Patcher"),
        ]),
        ("4590", &[
            ("Pathoschild.SMAPI", "SMAPI - Stardew Modding API"),
            ("Pathoschild.ContentPatcher", "Content Patcher"),
            ("spacechase0.SpaceCore", "SpaceCore"),
        ]),
        ("5098", &[
            ("Pathoschild.SMAPI", "SMAPI - Stardew Modding API"),
        ]),
        ("8622", &[
            ("Pathoschild.SMAPI", "SMAPI - Stardew Modding API"),
            ("Pathoschild.ContentPatcher", "Content Patcher"),
            ("spacechase0.SpaceCore", "SpaceCore"),
            ("spacechase0.GenericModConfigMenu", "Generic Mod Config Menu"),
        ]),
        ("323", &[
            ("Pathoschild.SMAPI", "SMAPI - Stardew Modding API"),
            ("Pathoschild.ContentPatcher", "Content Patcher"),
            ("spacechase0.SpaceCore", "SpaceCore"),
        ]),
        ("239", &[
            ("Pathoschild.SMAPI", "SMAPI - Stardew Modding API"),
            ("Pathoschild.ContentPatcher", "Content Patcher"),
        ]),
        ("4970", &[
            ("Pathoschild.SMAPI", "SMAPI - Stardew Modding API"),
            ("Pathoschild.ContentPatcher", "Content Patcher"),
            ("spacechase0.SpaceCore", "SpaceCore"),
        ]),
        ("7931", &[
            ("Pathoschild.SMAPI", "SMAPI - Stardew Modding API"),
            ("Pathoschild.ContentPatcher", "Content Patcher"),
            ("spacechase0.SpaceCore", "SpaceCore"),
        ]),
        ("6076", &[
            ("Pathoschild.SMAPI", "SMAPI - Stardew Modding API"),
            ("Pathoschild.ContentPatcher", "Content Patcher"),
            ("spacechase0.SpaceCore", "SpaceCore"),
        ]),
        ("6529", &[
            ("Pathoschild.SMAPI", "SMAPI - Stardew Modding API"),
        ]),
        ("4049", &[
            ("Pathoschild.SMAPI", "SMAPI - Stardew Modding API"),
            ("Pathoschild.ContentPatcher", "Content Patcher"),
        ]),
        ("813", &[
            ("Pathoschild.SMAPI", "SMAPI - Stardew Modding API"),
        ]),
    ]
}

fn lookup_popular_mod_prereqs(nexus_mod_id: &str) -> Option<&'static [(&'static str, &'static str)]> {
    known_popular_mod_prereqs()
        .iter()
        .find(|(mid, _)| *mid == nexus_mod_id)
        .map(|(_, prereqs)| *prereqs)
}

fn extract_dependencies_from_description(description: &str, mod_name: &str) -> Vec<DependencyEntry> {
    let mut deps: Vec<DependencyEntry> = Vec::new();
    let mut processed: std::collections::HashSet<String> = std::collections::HashSet::new();

    let sections = extract_all_requirements_sections(description);

    if sections.is_empty() {
        if let Some(mut fallback_deps) = scan_nexus_links_in_text(description, mod_name) {
            deps.append(&mut fallback_deps);
        }
        return deps;
    }

    for section_text in &sections {
        extract_deps_from_section(section_text, mod_name, &mut deps, &mut processed);
    }

    let total: std::collections::HashSet<String> = deps.iter()
        .map(|d| d.unique_id.to_lowercase())
        .collect();
    if total.len() < 2 {
        if let Some(mut fallback_deps) = scan_nexus_links_in_text(description, mod_name) {
            for dep in fallback_deps.drain(..) {
                if !processed.contains(&dep.unique_id.to_lowercase()) {
                    processed.insert(dep.unique_id.to_lowercase());
                    deps.push(dep);
                }
            }
        }
    }

    deps
}

fn scan_nexus_links_in_text(text: &str, mod_name: &str) -> Option<Vec<DependencyEntry>> {
    let re = regex::Regex::new(r#"\[url=([^\]]+)\]([^\[]+)\[/url\]"#).ok()?;
    let mut deps: Vec<DependencyEntry> = Vec::new();
    let mut processed: std::collections::HashSet<String> = std::collections::HashSet::new();

    for cap in re.captures_iter(text) {
        let url = &cap[1];
        let label = cap[2].trim().to_string();
        let nexus_mod_id = extract_mod_id_from_url(url);
        if nexus_mod_id.is_none() {
            continue;
        }
        if label.is_empty() {
            continue;
        }
        if label.to_lowercase().contains("(optional)") || label.contains("（可选）") {
            continue;
        }
        if label.to_lowercase() == mod_name.to_lowercase() {
            continue;
        }
        if processed.contains(&label.to_lowercase()) {
            continue;
        }
        processed.insert(label.to_lowercase());

        let known = lookup_known_mod_by_display_name(&label);
        let (unique_id, display_name) = if let Some((uid, display, _)) = known {
            (uid.to_string(), display.to_string())
        } else {
            (guess_unique_id_from_name(&label), label.clone())
        };

        if unique_id.to_lowercase() == mod_name.to_lowercase() {
            continue;
        }

        deps.push(DependencyEntry {
            unique_id,
            display_name: Some(display_name),
            nexus_mod_id,
            is_required: true,
            minimum_version: None,
        });
    }

    if deps.is_empty() {
        None
    } else {
        Some(deps)
    }
}

fn extract_deps_from_section(
    section_text: &str,
    mod_name: &str,
    deps: &mut Vec<DependencyEntry>,
    processed: &mut std::collections::HashSet<String>,
) {

    let link_re = regex::Regex::new(r#"\[url=([^\]]+)\]([^\[]+)\[/url\](\s*\([^\)]+\))?"#).unwrap();

    for cap in link_re.captures_iter(&section_text) {
        let url = &cap[1];
        let label = cap[2].trim().to_string();
        let trailing = cap.get(3).map(|m| m.as_str()).unwrap_or("");

        if processed.contains(&label.to_lowercase()) {
            continue;
        }
        processed.insert(label.to_lowercase());

        let nexus_mod_id = extract_mod_id_from_url(url);
        let is_optional = label.contains("(optional)")
            || label.contains("(Optional)")
            || label.contains("[optional]")
            || label.contains("（可选）")
            || label.contains("非必须")
            || trailing.contains("(optional)")
            || trailing.contains("(Optional)")
            || trailing.contains("[optional]")
            || trailing.contains("（可选）")
            || trailing.contains("非必须");

        let known = lookup_known_mod_by_display_name(&label);
        let (unique_id, display_name) = if let Some((uid, display, _)) = known {
            (uid.to_string(), display.to_string())
        } else if let Some(mod_id) = &nexus_mod_id {
            let uid_guess = guess_unique_id_from_name(&label);
            (uid_guess, label.clone())
        } else {
            (guess_unique_id_from_name(&label), label.clone())
        };

        if unique_id.to_lowercase() == mod_name.to_lowercase() {
            continue;
        }

        let mut nexus_id_string: Option<String> = None;
        if let Some(known_entry) = lookup_known_mod_by_unique_id(&unique_id) {
            nexus_id_string = Some(known_entry.2.to_string());
        }
        if nexus_id_string.is_none() {
            nexus_id_string = nexus_mod_id;
        }

        deps.push(DependencyEntry {
            unique_id,
            display_name: Some(display_name),
            nexus_mod_id: nexus_id_string,
            is_required: !is_optional,
            minimum_version: None,
        });
    }

    let bullet_re = regex::Regex::new(r"(?m)^\s*[-*•·]\s*([^\n]+)").unwrap();
    for cap in bullet_re.captures_iter(&section_text) {
        let line = cap[1].trim();
        if line.is_empty() || line.contains("[url=") {
            continue;
        }
        if processed.contains(&line.to_lowercase()) {
            continue;
        }

        let is_optional = line.contains("(optional)")
            || line.contains("(Optional)")
            || line.contains("[optional]")
            || line.contains("（可选）")
            || line.contains("非必须");

        let name_only = line
            .replace("(optional)", "")
            .replace("(Optional)", "")
            .replace("[optional]", "")
            .replace("（可选）", "")
            .replace("非必须", "")
            .trim()
            .to_string();

        let known = lookup_known_mod_by_display_name(&name_only);
        if let Some((uid, display, mod_id)) = known {
            if processed.contains(&display.to_lowercase()) {
                continue;
            }
            processed.insert(display.to_lowercase());

            deps.push(DependencyEntry {
                unique_id: uid.to_string(),
                display_name: Some(display.to_string()),
                nexus_mod_id: Some(mod_id.to_string()),
                is_required: !is_optional,
                minimum_version: None,
            });
            continue;
        }

        if name_only.to_lowercase().contains("smapi") {
            if !processed.contains("smapi") {
                processed.insert("smapi".to_string());
                deps.push(DependencyEntry {
                    unique_id: "Pathoschild.SMAPI".to_string(),
                    display_name: Some("SMAPI - Stardew Modding API".to_string()),
                    nexus_mod_id: Some("2400".to_string()),
                    is_required: !is_optional,
                    minimum_version: None,
                });
            }
            continue;
        }
    }

    let uid_re = regex::Regex::new(r"\b([A-Za-z][A-Za-z0-9_]*(?:\.[A-Za-z][A-Za-z0-9_]*){1,4})\b").unwrap();
    for cap in uid_re.captures_iter(&section_text) {
        let potential_uid = cap[1].to_string();
        let lower = potential_uid.to_lowercase();

        if lower.contains("stardew") || lower.contains("valley") || lower.contains("nexus")
            || lower.contains("github") || lower.contains("http") || lower.contains("www")
            || lower.starts_with("pathoschild.smapi")
        {
            continue;
        }
        if lower.contains("contentpatcher") {
            continue;
        }
        if lower.contains("spacecore") {
            continue;
        }

        let known = lookup_known_mod_by_unique_id(&potential_uid);
        if let Some((uid, display, mod_id)) = known {
            if processed.contains(&display.to_lowercase()) {
                continue;
            }
            processed.insert(display.to_lowercase());

            deps.push(DependencyEntry {
                unique_id: uid.to_string(),
                display_name: Some(display.to_string()),
                nexus_mod_id: Some(mod_id.to_string()),
                is_required: true,
                minimum_version: None,
            });
            continue;
        }

        if !processed.contains(&lower) {
            let parts: Vec<&str> = potential_uid.split('.').collect();
            if parts.len() >= 2 && parts.iter().all(|p| !p.is_empty()) {
                if !is_junk_unique_id(&lower) {
                    processed.insert(lower.clone());
                    deps.push(DependencyEntry {
                        unique_id: potential_uid,
                        display_name: None,
                        nexus_mod_id: None,
                        is_required: true,
                        minimum_version: None,
                    });
                }
            }
        }
    }

    let _ = deps;
}

fn is_junk_unique_id(lower: &str) -> bool {
    if lower.contains(".com") || lower.contains(".app") || lower.contains(".net")
        || lower.contains(".gg") || lower.contains(".io") || lower.contains(".org")
        || lower.contains(".php") || lower.contains(".html") || lower.contains(".htm")
        || lower.contains(".js") || lower.contains(".css") || lower.contains(".xml")
        || lower.contains(".json") || lower.contains(".bsky") || lower.contains(".bsky.social")
    {
        return true;
    }
    let junk_substrings = [
        "twitter", "linktr", "bsky", "discord", "patreon", "youtube",
        "facebook", "instagram", "reddit", "tiktok", "twitch",
        "github", "bit.ly", "goo.gl", "tinyurl", "pastebin",
        "settings", "config", "manifest", "index.php", "gmail", "outlook",
        "yahoo", "hotmail", "protonmail", "email", "mods/", "stardewvalley",
    ];
    for s in &junk_substrings {
        if lower.contains(s) {
            return true;
        }
    }
    if lower == "index.php" || lower.ends_with(".php") {
        return true;
    }
    if lower.starts_with("www.") {
        return true;
    }
    if lower.split('.').any(|p| p.parse::<u32>().is_ok()) && lower.contains("nexusmods") {
        return true;
    }
    false
}

fn extract_requirements_section(description: &str) -> String {
    let sections = extract_all_requirements_sections(description);
    sections.into_iter().next().unwrap_or_default()
}

fn extract_all_requirements_sections(description: &str) -> Vec<String> {
    let lower = description.to_lowercase();

    let keywords = [
        "requirement", "requirements", "required", "requires",
        "dependency", "dependencies",
        "prerequisite", "prerequisites",
        "compatibility", "compatibility",
        "you will need", "you'll need", "you need", "you'll want", "you will want",
        "needed mod", "needed mods", "needed:",
        "必备", "依赖", "需要", "前置",
    ];

    let mut candidates: Vec<(usize, usize)> = Vec::new();

    for keyword in &keywords {
        let mut search_from = 0;
        while let Some(pos) = lower[search_from..].find(keyword) {
            let abs_pos = search_from + pos;
            search_from = abs_pos + keyword.len();

            let (is_header, header_end) = is_section_header_at(description, abs_pos, keyword.len());
            if is_header {
                candidates.push((abs_pos, header_end));
            }
        }
    }

    candidates.sort_by_key(|(pos, _)| *pos);
    candidates.dedup_by(|a, b| (a.0 as i64 - b.0 as i64).abs() < 20);

    let mut sections: Vec<String> = Vec::new();

    for (_, header_end) in &candidates {
        let remaining = &description[*header_end..];

        let mut depth: i32 = 0;
        let mut end: usize = remaining.len();
        let mut i: usize = 0;
        let mut prev_char: Option<char> = None;

        while i < remaining.len() {
            let c = remaining[i..].chars().next().unwrap();
            let c_len = c.len_utf8();

            if c == '[' {
                depth += 1;
            } else if c == ']' {
                depth -= 1;
            } else if c == '\n' && depth == 0 && i > 0 {
                if let Some(prev) = prev_char {
                    if prev == '\n' || prev == '\r' {
                        end = i;
                        break;
                    }
                }
                if remaining[i..].starts_with("\n#")
                    || remaining[i..].starts_with("\n[size=")
                    || remaining[i..].starts_with("\n[b][size=")
                    || remaining[i..].starts_with("\n[/b]")
                    || remaining[i..].starts_with("\n[center]")
                    || remaining[i..].starts_with("\n[left]")
                    || remaining[i..].starts_with("\n[right]")
                {
                    end = i;
                    break;
                }
            }

            prev_char = Some(c);
            i += c_len;
        }

        if remaining.len() < 50 {
            end = remaining.len();
        }

        let section_text = remaining[..end].to_string();
        if !section_text.trim().is_empty() {
            sections.push(section_text);
        }
    }

    sections
}

fn is_section_header_at(description: &str, pos: usize, keyword_len: usize) -> (bool, usize) {
    let bytes = description.as_bytes();
    let after = pos + keyword_len;

    let pre = &description[..pos];
    let pre_byte_start = pos.saturating_sub(15);
    let pre_window = &description[pre_byte_start..pos];

    let pre_has_bold_open = pre_window.contains("[b]")
        || pre_window.contains("[b][size=")
        || pre_window.contains("[size=")
        || pre_window.contains("[center]")
        || pre_window.contains("[left]")
        || pre_window.contains("[right]");

    let pre_is_line_start = pre.ends_with('\n')
        || pre.ends_with('\r')
        || pre.is_empty()
        || pre_byte_start == 0
        || (pre_byte_start > 0 && bytes[pre_byte_start - 1] == b'\n');

    if !pre_has_bold_open && !pre_is_line_start {
        return (false, 0);
    }

    let next_part = &description[after..];
    let chars_after: Vec<char> = next_part.chars().collect();

    if let Some(&c) = chars_after.first() {
        if c == ' ' || c == '\n' || c == '\r' || c == '\t' || c == ':' || c == '：' || c == '[' {
            let mut offset_after_skip = after;
            if next_part.starts_with('[') {
                let mut depth: i32 = 0;
                let mut idx: usize = 0;
                let mut closed_one = false;
                while idx < next_part.len() {
                    let ch = next_part[idx..].chars().next().unwrap();
                    if ch == '[' {
                        depth += 1;
                    } else if ch == ']' {
                        depth -= 1;
                        if depth == 0 {
                            idx += ch.len_utf8();
                            offset_after_skip = after + idx;
                            closed_one = true;
                            break;
                        }
                    }
                    idx += ch.len_utf8();
                }
                if !closed_one {
                    offset_after_skip = after;
                }
            }
            return (true, offset_after_skip);
        }
    }

    (false, 0)
}

fn extract_mod_id_from_url(url: &str) -> Option<String> {
    let re = regex::Regex::new(r"nexusmods\.com/[^/]+/mods/(\d+)").unwrap();
    if let Some(caps) = re.captures(url) {
        return Some(caps[1].to_string());
    }
    None
}

fn guess_unique_id_from_name(name: &str) -> String {
    let cleaned: String = name
        .replace("(optional)", "")
        .replace("(Optional)", "")
        .replace("[optional]", "")
        .replace("（可选）", "")
        .replace("非必须", "")
        .trim()
        .to_string();

    if cleaned.is_empty() {
        return "Unknown.Unknown".to_string();
    }

    let known = lookup_known_mod_by_display_name(&cleaned);
    if let Some((uid, _, _)) = known {
        return uid.to_string();
    }

    let words: Vec<&str> = cleaned.split_whitespace().collect();
    if words.is_empty() {
        return "Unknown.Unknown".to_string();
    }

    let first_word_lower = words[0].to_lowercase();
    let author_candidate = match first_word_lower.as_str() {
        "the" | "a" | "an" => {
            if words.len() >= 2 {
                words[1].to_lowercase()
            } else {
                first_word_lower.clone()
            }
        }
        _ => first_word_lower.clone(),
    };

    let mut remaining_words: Vec<String> = words
        .iter()
        .skip(if first_word_lower == "the" || first_word_lower == "a" || first_word_lower == "an" { 2 } else { 1 })
        .map(|w| {
            let mut chars: Vec<char> = w.chars().filter(|c| c.is_alphanumeric()).collect();
            if chars.is_empty() {
                return String::new();
            }
            chars[0] = chars[0].to_ascii_uppercase();
            for c in chars.iter_mut().skip(1) {
                if c.is_ascii_uppercase() {
                    *c = c.to_ascii_lowercase();
                }
            }
            chars.into_iter().collect()
        })
        .filter(|s| !s.is_empty())
        .collect();

    if remaining_words.is_empty() {
        remaining_words.push(cleaned.replace(' ', ""));
    }

    let mod_name = remaining_words.join("");
    let result = format!("{}.{}", author_candidate, mod_name);

    if !result.contains('.') {
        return format!("Unknown.{}", result);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_extract_dependencies_from_bbcode_description() {
        let desc = r#"[center][b][size=5]Test Mod[/size][/b][/center]
This is a test mod description.

[center][b][size=4]Requirements[/size][/b][/center]
[list]
[*] [url=https://www.nexusmods.com/stardewvalley/mods/2400]SMAPI - Stardew Modding API[/url]
[*] [url=https://www.nexusmods.com/stardewvalley/mods/1915]Content Patcher[/url]
[*] [url=https://www.nexusmods.com/stardewvalley/mods/1348]SpaceCore[/url]
[/list]

[center][b][size=4]Installation[/size][/b][/center]
[list]
[*] Drop into Mods folder
[/list]"#;

        let deps = extract_dependencies_from_description(desc, "Test.Mod");
        assert!(deps.iter().any(|d| d.unique_id == "Pathoschild.SMAPI" && d.display_name.as_deref() == Some("SMAPI - Stardew Modding API")));
        assert!(deps.iter().any(|d| d.unique_id == "Pathoschild.ContentPatcher" && d.nexus_mod_id.as_deref() == Some("1915")));
        assert!(deps.iter().any(|d| d.unique_id == "spacechase0.SpaceCore" && d.nexus_mod_id.as_deref() == Some("1348")));
    }

    #[test]
    fn test_extract_dependencies_no_requirements_header_fallback() {
        let desc = r#"Welcome to my mod! You can find it on the [url=https://github.com/test/test]GitHub repo[/url].
This mod was inspired by [url=https://www.nexusmods.com/stardewvalley/mods/9999]Some Other Mod[/url].
Compatibility: [url=https://www.nexusmods.com/stardewvalley/mods/1915]Content Patcher[/url] is required.
[url=https://www.nexusmods.com/stardewvalley/mods/2400]SMAPI[/url] must also be installed."#;

        let deps = extract_dependencies_from_description(desc, "My Mod");
        assert!(deps.iter().any(|d| d.nexus_mod_id.as_deref() == Some("1915")));
        assert!(deps.iter().any(|d| d.nexus_mod_id.as_deref() == Some("2400")));
    }

    #[test]
    fn test_popular_mod_prereqs_sve() {
        let prereqs = lookup_popular_mod_prereqs("3753");
        assert!(prereqs.is_some(), "SVE (3753) should have built-in prereqs");
        let prereqs = prereqs.unwrap();
        assert!(prereqs.iter().any(|(uid, _)| *uid == "Pathoschild.SMAPI"));
        assert!(prereqs.iter().any(|(uid, _)| *uid == "Pathoschild.ContentPatcher"));
        assert!(prereqs.iter().any(|(uid, _)| *uid == "spacechase0.SpaceCore"));
        assert!(prereqs.iter().any(|(uid, _)| *uid == "spacechase0.GenericModConfigMenu"));
    }

    #[test]
    fn test_popular_mod_prereqs_unknown() {
        assert!(lookup_popular_mod_prereqs("9999999").is_none());
    }

    #[test]
    fn test_popular_mod_prereqs_content_patcher() {
        let prereqs = lookup_popular_mod_prereqs("1915");
        assert!(prereqs.is_some());
        assert!(prereqs.unwrap().iter().any(|(uid, _)| *uid == "Pathoschild.SMAPI"));
    }

    #[test]
    fn test_extract_dependencies_from_plain_text() {
        let desc = "Some mod description.

Requirements:
- Content Patcher
- SMAPI - Stardew Modding API
- SpaceCore (optional)

Installation:
Unzip and place in Mods folder.";

        let deps = extract_dependencies_from_description(desc, "Test.Mod");
        assert!(deps.iter().any(|d| d.unique_id == "Pathoschild.ContentPatcher"));
        assert!(deps.iter().any(|d| d.unique_id == "Pathoschild.SMAPI"));
        assert!(deps.iter().any(|d| d.unique_id == "spacechase0.SpaceCore" && !d.is_required));
    }

    #[test]
    fn test_extract_dependencies_bold_header() {
        let desc = r#"[b][size=4]Compatibility[/size][/b]
[list]
[*] [url=https://www.nexusmods.com/stardewvalley/mods/1915]Content Patcher[/url]
[/list]"#;

        let deps = extract_dependencies_from_description(desc, "Test.Mod");
        assert!(deps.iter().any(|d| d.unique_id == "Pathoschild.ContentPatcher" && d.nexus_mod_id.as_deref() == Some("1915")));
    }

    #[test]
    fn test_extract_mod_id_from_url() {
        assert_eq!(extract_mod_id_from_url("https://www.nexusmods.com/stardewvalley/mods/2400"), Some("2400".to_string()));
        assert_eq!(extract_mod_id_from_url("https://www.nexusmods.com/stardewvalley/mods/1915?tab=description"), Some("1915".to_string()));
        assert_eq!(extract_mod_id_from_url("https://example.com/foo"), None);
    }

    #[test]
    fn test_guess_unique_id_from_name() {
        assert_eq!(guess_unique_id_from_name("Content Patcher"), "Pathoschild.ContentPatcher");
        assert_eq!(guess_unique_id_from_name("Content Patcher (optional)"), "Pathoschild.ContentPatcher");
        assert_eq!(guess_unique_id_from_name("Some Random Mod"), "some.RandomMod");
    }

    #[test]
    fn test_known_prerequisite_lookup() {
        assert!(lookup_known_mod_by_display_name("Content Patcher").is_some());
        assert!(lookup_known_mod_by_display_name("SMAPI - Stardew Modding API").is_some());
        assert!(lookup_known_mod_by_unique_id("Pathoschild.ContentPatcher").is_some());
        assert!(lookup_known_mod_by_display_name("Nonexistent Mod").is_none());
    }

    #[test]
    fn test_extract_dependencies_chinese_description() {
        let desc = "本mod的前置依赖：
- SMAPI
- Content Patcher
- SpaceCore（可选）";

        let deps = extract_dependencies_from_description(desc, "Test.Mod");
        assert!(deps.iter().any(|d| d.unique_id == "Pathoschild.SMAPI"));
        assert!(deps.iter().any(|d| d.unique_id == "Pathoschild.ContentPatcher"));
        assert!(deps.iter().any(|d| d.unique_id == "spacechase0.SpaceCore" && !d.is_required));
    }

    #[test]
    fn test_extract_requirements_section() {
        let desc = r#"[b]Description[/b]
Some text.

[b]Requirements[/b]
- Content Patcher
- SMAPI

[b]Installation[/b]
- Drop into Mods"#;
        let section = extract_requirements_section(desc);
        assert!(section.contains("Content Patcher"));
        assert!(section.contains("SMAPI"));
        assert!(!section.contains("Installation"));
    }

    #[test]
    fn test_extract_requirements_section_no_match_returns_empty() {
        let desc = "[b]Configurations[/b]\nAdd leaves to roofs during fall.";
        let section = extract_requirements_section(desc);
        assert_eq!(section, "");
    }

    #[test]
    fn test_real_sve_description() {
        let desc = r#"[center][size=5][b]Stardew Valley Expanded[/b][/size][/center]
SVE is a large mod that adds 28 new NPCs, 58 new locations, 278 new character events, and much more.

[center][b]Compatibility[/b][/center]
SVE is compatible with SMAPI 3.13 and Stardew Valley 1.5.5 or later.

[center][b]Installation[/b][/center]
Extract the downloaded ZIP into your Mods folder.

[center][b]Bug Reports[/b][/center]
If you encounter any issues, please report them on our [url=https://discord.gg/sve]Discord server[/url] or [url=https://www.nexusmods.com/stardewvalley]Nexus Mods[/url].

[center][b]Credits[/b][/center]
[url=https://www.nexusmods.com/stardewvalley]Click Me![/url] [url=https://discord.gg/sve]Stardew Valley Expanded Discord Server[/url]
[url=https://www.paypal.com/donate]PayPal[/url] [url=https://wiki.stardewvalley]Stardew Valley Expanded Wiki[/url]
[url=https://discord.gg]Click Here![/url]

[center][b]Requirements[/b][/center]
[list]
[*] [url=https://www.nexusmods.com/stardewvalley/mods/2400]SMAPI - Stardew Modding API[/url]
[*] [url=https://www.nexusmods.com/stardewvalley/mods/1915]Content Patcher[/url]
[*] [url=https://www.nexusmods.com/stardewvalley/mods/1348]SpaceCore[/url]
[*] [url=https://www.nexusmods.com/stardewvalley/mods/5098]Generic Mod Config Menu[/url] (optional)
[/list]

[center][b]Configuration[/b][/center]
Default = "True"
This config requires Content Patcher 1.0 or later.

<br /> <br /> <br /> <br />"#;

        let section = extract_requirements_section(desc);
        eprintln!("=== EXTRACTED SECTION ===");
        eprintln!("{}", section);
        eprintln!("=== END ===");

        let deps = extract_dependencies_from_description(desc, "FlashShifter.StardewValleyExpandedCP");
        eprintln!("=== DEPS COUNT: {} ===", deps.len());
        for d in &deps {
            eprintln!("  - {} (required={})", d.unique_id, d.is_required);
        }
        assert_eq!(deps.len(), 4);
        assert!(deps.iter().any(|d| d.unique_id == "Pathoschild.SMAPI" && d.is_required));
        assert!(deps.iter().any(|d| d.unique_id == "Pathoschild.ContentPatcher" && d.is_required));
        assert!(deps.iter().any(|d| d.unique_id == "spacechase0.SpaceCore" && d.is_required));
        assert!(deps.iter().any(|d| d.unique_id == "spacechase0.GenericModConfigMenu" && !d.is_required));

        for dep in &deps {
            assert!(!dep.unique_id.contains("linktr"));
            assert!(!dep.unique_id.contains("twitter"));
            assert!(!dep.unique_id.contains("discord"));
            assert!(!dep.unique_id.contains("bsky"));
            assert!(!dep.unique_id.contains("nexusmods"));
        }
    }

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
