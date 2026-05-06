use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use crate::mod_patches;
use crate::nexus_linker;
use crate::smapi_data;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub unique_id: String,
    pub enabled: bool,
    pub is_required: bool,
    pub has_dependencies: bool,
    pub dependency_count: usize,
    pub is_content_pack: bool,
    pub content_pack_for: Option<String>,
    pub folder_path: String,
    pub has_conflict: bool,
    pub conflict_warning: Option<String>,
    pub url: Option<String>,
    pub category: String,
    pub screenshot_path: Option<String>,
    pub thumbnail_path: Option<String>,
    pub has_update: bool,
    pub latest_version: Option<String>,
    pub update_url: Option<String>,
    pub dependencies: Vec<ModDependencyInfo>,
    pub manifest_content: Option<String>,
    pub sub_mods: Vec<ModInfo>,
    pub is_group: bool,
    pub internal_component_ids: Vec<String>,
    pub nexus_mod_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDependencyInfo {
    pub unique_id: String,
    pub minimum_version: Option<String>,
    pub is_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModManifest {
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "Version")]
    version: Option<String>,
    #[serde(rename = "Author")]
    author: Option<String>,
    #[serde(rename = "Description")]
    description: Option<String>,
    #[serde(rename = "UniqueID", alias = "UniqueId")]
    unique_id: Option<String>,
    #[serde(rename = "Dependencies", default)]
    dependencies: Vec<ManifestDependency>,
    #[serde(rename = "ContentPackFor")]
    content_pack_for: Option<ContentPackFor>,
    #[serde(rename = "UpdateKeys")]
    update_keys: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ManifestDependency {
    #[serde(rename = "UniqueID", alias = "UniqueId")]
    unique_id: String,
    #[serde(rename = "MinimumVersion")]
    minimum_version: Option<String>,
    #[serde(rename = "IsRequired", default = "default_is_required")]
    is_required: bool,
}

fn default_is_required() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContentPackFor {
    #[serde(rename = "UniqueID", alias = "UniqueId")]
    unique_id: String,
}

fn remove_trailing_commas(json: &str) -> String {
    let mut result = String::with_capacity(json.len());
    let chars: Vec<char> = json.chars().collect();
    let mut in_string = false;
    let mut escape_next = false;

    for i in 0..chars.len() {
        let c = chars[i];

        if escape_next {
            result.push(c);
            escape_next = false;
            continue;
        }

        if c == '\\' && in_string {
            result.push(c);
            escape_next = true;
            continue;
        }

        if c == '"' {
            in_string = !in_string;
            result.push(c);
            continue;
        }

        if !in_string && c == ',' {
            let mut j = i + 1;
            while j < chars.len() && (chars[j] == ' ' || chars[j] == '\t' || chars[j] == '\n' || chars[j] == '\r') {
                j += 1;
            }
            if j < chars.len() && (chars[j] == '}' || chars[j] == ']') {
                continue;
            }
        }

        result.push(c);
    }

    result
}

fn extract_nexus_id_from_raw(raw: &str) -> Option<u64> {
    let re = regex::Regex::new(r"Nexus:[^0-9-]*(?<modId>-?\d+)(?<flag>@.*)?.*").unwrap();
    if let Some(caps) = re.captures(&format!("Nexus:{}", raw)) {
        if let Some(mod_id_match) = caps.name("modId") {
            if let Ok(mod_id) = mod_id_match.as_str().parse::<i64>() {
                if mod_id > 0 {
                    return Some(mod_id as u64);
                }
            }
        }
    }
    None
}

fn detect_category(unique_id: &str, manifest: &ModManifest) -> String {
    let id_lower = unique_id.to_lowercase();
    let name_lower = manifest.name.as_ref().map(|s| s.to_lowercase()).unwrap_or_default();
    let desc_lower = manifest.description.as_ref().map(|s| s.to_lowercase()).unwrap_or_default();
    let all_text = format!("{} {} {}", id_lower, name_lower, desc_lower);

    if all_text.contains("season") || all_text.contains("festival") || all_text.contains("event") {
        return "seasonal".to_string();
    }
    if all_text.contains("farm") || all_text.contains("crop") || all_text.contains("animal") {
        return "gameplay".to_string();
    }
    if all_text.contains("ui") || all_text.contains("menu") || all_text.contains("hud") {
        return "ui".to_string();
    }
    if all_text.contains("texture") || all_text.contains("portrait") || all_text.contains("sprite") || all_text.contains("skin") {
        return "visual".to_string();
    }
    if all_text.contains("map") || all_text.contains("location") || all_text.contains("expand") {
        return "expansion".to_string();
    }
    if all_text.contains("lib") || all_text.contains("api") || all_text.contains("framework") || all_text.contains("content pack") {
        return "framework".to_string();
    }
    if all_text.contains("multiplayer") || all_text.contains("coop") || all_text.contains("split") {
        return "multiplayer".to_string();
    }

    "other".to_string()
}

fn find_screenshot(path: &PathBuf) -> Option<String> {
    let screenshot_names = ["screenshot.png", "preview.png", "thumb.png", "icon.png"];
    for name in &screenshot_names {
        let screenshot_path = path.join(name);
        if screenshot_path.exists() {
            return Some(screenshot_path.to_string_lossy().to_string());
        }
    }
    None
}

fn recursive_find_manifests(dir: &PathBuf) -> Vec<PathBuf> {
    println!("[mod_parser] Scanning directory: {}", dir.display());

    let mut manifests = Vec::new();

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            println!("[mod_parser] Failed to read directory {}: {}", dir.display(), e);
            return manifests;
        }
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        if let Some(folder_name) = path.file_name().and_then(|n| n.to_str()) {
            if folder_name.starts_with('.') && !folder_name.starts_with("..") {
                let manifest_path = path.join("manifest.json");
                if manifest_path.exists() {
                    println!("[mod_parser] Found disabled mod manifest in: {}", path.display());
                    manifests.push(path.clone());
                }
                println!("[mod_parser] Skipping recursion into disabled folder: {}", path.display());
                continue;
            }

            if folder_name.starts_with('_') {
                println!("[mod_parser] Skipping hidden/temp folder: {}", path.display());
                continue;
            }
        }

        let manifest_path = path.join("manifest.json");
        if manifest_path.exists() {
            println!("[mod_parser] Found manifest.json in: {}", path.display());
            manifests.push(path.clone());
        }
        
        let nested = recursive_find_manifests(&path);
        manifests.extend(nested);
    }

    manifests
}

fn parse_manifest(path: &PathBuf, content: &str, enabled: bool) -> Option<ModInfo> {
    println!("[mod_parser] Parsing manifest: {}", path.display());

    let cleaned = remove_trailing_commas(content);

    // Strip UTF-8 BOM if present (EF BB BF)
    let cleaned = cleaned.strip_prefix('\u{FEFF}').unwrap_or(&cleaned);

    let manifest: ModManifest = match serde_json::from_str(cleaned) {
        Ok(m) => m,
        Err(e) => {
            println!("[mod_parser] Failed to parse manifest at {}: {}", path.display(), e);
            return None;
        }
    };

    let name = manifest.name.clone().unwrap_or_else(|| {
        path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });
    let version = manifest.version.as_deref().unwrap_or("1.0.0").to_string();
    let author = manifest.author.clone().unwrap_or_default();
    let description = manifest.description.clone().unwrap_or_default();
    let unique_id = manifest
        .unique_id
        .clone()
        .unwrap_or_else(|| format!("unknown_{}", name));

    println!("[mod_parser] Parsed MOD: {} ({})", name, unique_id);

    let is_content_pack = manifest.content_pack_for.is_some();
    let content_pack_for = manifest
        .content_pack_for
        .as_ref()
        .map(|cp| cp.unique_id.clone());

    let mut dependencies: Vec<ModDependencyInfo> = manifest
        .dependencies
        .iter()
        .map(|d| ModDependencyInfo {
            unique_id: d.unique_id.clone(),
            minimum_version: d.minimum_version.clone(),
            is_required: d.is_required,
        })
        .collect();

    if let Some(patch) = mod_patches::get_missing_dependency(&unique_id) {
        if !dependencies.iter().any(|d| d.unique_id.to_lowercase() == patch.missing_id.to_lowercase()) {
            println!("[mod_parser] Applying patch: adding missing dependency '{}' to '{}' (reason: {})", patch.missing_id, unique_id, patch.reason);
            dependencies.push(ModDependencyInfo {
                unique_id: patch.missing_id,
                minimum_version: None,
                is_required: true,
            });
        }
    }

    let (url, nexus_mod_id) = if let Some(keys) = &manifest.update_keys {
        let mut found_url: Option<String> = None;
        let mut found_nexus_id: Option<u64> = None;
        for key in keys {
            if key.starts_with("Nexus:") {
                let raw_id = key.trim_start_matches("Nexus:");
                if let Some(nexus_id) = extract_nexus_id_from_raw(raw_id) {
                    found_nexus_id = Some(nexus_id);
                    found_url =
                        Some(format!("https://www.nexusmods.com/stardewvalley/mods/{}", nexus_id));
                    break;
                }
            } else if key.starts_with("GitHub:") {
                let parts: Vec<&str> = key.trim_start_matches("GitHub:").split('/').collect();
                if parts.len() >= 2 {
                    found_url = Some(format!("https://github.com/{}/{}", parts[0], parts[1]));
                    break;
                }
            }
        }
        (found_url, found_nexus_id)
    } else {
        (None, None)
    };

    let url = url.or_else(|| {
        if let Some(nexus_id) = nexus_mod_id {
            return Some(format!("https://www.nexusmods.com/stardewvalley/mods/{}", nexus_id));
        }
        let link_result = nexus_linker::build_nexus_link(&unique_id, Some(&name), None);
        Some(link_result.url)
    });

    let category = detect_category(&unique_id, &manifest);
    let screenshot_path = find_screenshot(path);
    let thumbnail_path = crate::mod_thumbnail::get_cached_thumbnail_path(&unique_id);

    let is_required = author.to_lowercase().contains("smapi");

    println!("[mod_parser] Successfully parsed: {} ({})", name, unique_id);

    Some(ModInfo {
        name,
        version,
        author,
        description,
        unique_id,
        enabled,
        is_required,
        has_dependencies: !dependencies.is_empty(),
        dependency_count: dependencies.len(),
        is_content_pack,
        content_pack_for,
        folder_path: path.to_string_lossy().to_string(),
        has_conflict: false,
        conflict_warning: None,
        url,
        category,
        screenshot_path,
        thumbnail_path,
        has_update: false,
        latest_version: None,
        update_url: None,
        dependencies,
        manifest_content: Some(content.to_string()),
        sub_mods: Vec::new(),
        is_group: false,
        internal_component_ids: Vec::new(),
        nexus_mod_id,
    })
}

fn force_scan_ftm(
    mods_path: &PathBuf,
    mods: &mut Vec<ModInfo>,
    seen_ids: &mut HashSet<String>,
) {
    println!("[mod_parser] Force scanning FTM...");

    let ftm_names = ["FarmTypeManager", "Farm Type Manager", "farmtypemanager"];

    for ftm_name in &ftm_names {
        let ftm_path = mods_path.join(ftm_name);
        if ftm_path.exists() && ftm_path.is_dir() {
            let manifest_path = ftm_path.join("manifest.json");
            if manifest_path.exists() {
                println!("[mod_parser] Found FTM manifest: {}", manifest_path.display());
                if let Ok(content) = fs::read_to_string(&manifest_path) {
                    if let Some(mod_info) = parse_manifest(&ftm_path, &content, true) {
                        // Check if the REAL FTM folder is already in the list
                        let already_has_real_ftm = mods.iter().any(|m| {
                            m.folder_path == ftm_path.to_string_lossy().to_string()
                        });

                        if already_has_real_ftm {
                            println!("[mod_parser] FTM already in list from correct path, skipping");
                        } else {
                            // Remove any stale FTM entry (e.g. from .temp_dep_check)
                            let ftm_id_lower = mod_info.unique_id.to_lowercase();
                            mods.retain(|m| m.unique_id.to_lowercase() != ftm_id_lower);
                            seen_ids.remove(&ftm_id_lower);

                            println!(
                                "[mod_parser] Force adding FTM: {} ({}) from {}",
                                mod_info.name, mod_info.unique_id, ftm_path.display()
                            );
                            seen_ids.insert(ftm_id_lower);
                            mods.push(mod_info);
                        }
                    }
                }
            }
            break;
        }
    }
}

fn migrate_legacy_disabled_folders(mods_path: &PathBuf) {
    fn migrate_dir(dir: &PathBuf) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    migrate_dir(&path);

                    let folder_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                    if folder_name.ends_with(".disabled") {
                        let clean_name = folder_name.trim_end_matches(".disabled");
                        let new_name = format!(".{}", clean_name);
                        let new_path = path.parent().map(|p| p.join(&new_name));
                        if let Some(new_path) = new_path {
                            if !new_path.exists() {
                                println!("[migrate] Renaming {} -> {}", path.display(), new_path.display());
                                if let Err(e) = fs::rename(&path, &new_path) {
                                    println!("[migrate] Failed to rename: {}", e);
                                }
                            } else {
                                println!("[migrate] Target already exists, skipping: {}", new_path.display());
                            }
                        }
                    }
                }
            }
        }
    }

    migrate_dir(mods_path);
}

#[tauri::command]
pub fn scan_mods(game_path: Option<String>) -> Result<Vec<ModInfo>, String> {
    println!("[mod_parser] === Starting MOD scan ===");

    let game_path = match game_path {
        Some(path) => PathBuf::from(path),
        None => {
            // Try to detect game path via registry and library folders first
            if let Some((detected_path, _method)) = crate::smapi::find_game_path() {
                detected_path
            } else {
                // Fallback to hardcoded default paths
                let default_paths = [
                    r"C:\Program Files (x86)\Steam\steamapps\common\Stardew Valley",
                    r"C:\Program Files\Steam\steamapps\common\Stardew Valley",
                    r"D:\steam\steamapps\common\Stardew Valley",
                    r"C:\GOG Games\Stardew Valley",
                ];

                let found = default_paths
                    .iter()
                    .map(PathBuf::from)
                    .find(|p| p.exists());

                match found {
                    Some(path) => path,
                    None => {
                        println!("[mod_parser] Game directory not found");
                        return Err("Game directory not found, please specify game path".to_string());
                    }
                }
            }
        }
    };

    let mods_path = game_path.join("Mods");
    println!("[mod_parser] Scanning Mods folder: {}", mods_path.display());

    if !mods_path.exists() {
        println!("[mod_parser] Mods folder does not exist");
        return Ok(vec![]);
    }

    migrate_legacy_disabled_folders(&mods_path);

    let mut mods = Vec::new();
    let mut seen_ids: HashSet<String> = HashSet::new();

    let manifest_dirs = recursive_find_manifests(&mods_path);
    println!(
        "[mod_parser] Found {} directories with manifest.json",
        manifest_dirs.len()
    );

    for dir in &manifest_dirs {
        let manifest_path = dir.join("manifest.json");
        if !manifest_path.exists() {
            println!("[mod_parser] manifest.json missing in {}", dir.display());
            continue;
        }

        let content = match fs::read_to_string(&manifest_path) {
            Ok(c) => c,
            Err(e) => {
                println!("[mod_parser] Failed to read {}: {}", manifest_path.display(), e);
                continue;
            }
        };

        let folder_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let is_disabled = folder_name.starts_with('.') && !folder_name.starts_with("..");

        println!(
            "[mod_parser] Processing folder: {} (disabled: {})",
            folder_name, is_disabled
        );

        if let Some(mod_info) = parse_manifest(dir, &content, !is_disabled) {
            if seen_ids.contains(&mod_info.unique_id.to_lowercase()) {
                println!(
                    "[mod_parser] Skipping duplicate: {} ({})",
                    mod_info.name, mod_info.unique_id
                );
                continue;
            }
            seen_ids.insert(mod_info.unique_id.to_lowercase());
            mods.push(mod_info);
        } else {
            println!("[mod_parser] Failed to parse manifest in: {}", dir.display());
        }
    }

    force_scan_ftm(&mods_path, &mut mods, &mut seen_ids);

    // Group content packs under their parent mods
    let grouped = group_content_packs(mods);

    let official_ids = smapi_data::get_all_mod_ids();
    println!(
        "[mod_parser] SMAPI official data contains {} MOD IDs for validation",
        official_ids.len()
    );

    for mod_info in &grouped {
        if official_ids.contains(&mod_info.unique_id) {
            println!(
                "[mod_parser] MOD '{}' ({}) validated against official data",
                mod_info.name, mod_info.unique_id
            );
        } else {
            println!(
                "[mod_parser] MOD '{}' ({}) not found in official data (may be custom/unofficial)",
                mod_info.name, mod_info.unique_id
            );
        }
    }

    let mut result = grouped;
    result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    println!("[mod_parser] === Scan complete: {} MODs found ===", result.len());
    Ok(result)
}

/// 将内容包（Content Packs）合并到它们所属的主模组下
fn group_content_packs(mods: Vec<ModInfo>) -> Vec<ModInfo> {
    use std::collections::HashMap;

    // Step 1: Build a map of parent_id -> list of content packs
    let mut parent_map: HashMap<String, Vec<ModInfo>> = HashMap::new();
    let mut standalone_mods: Vec<ModInfo> = Vec::new();

    for mod_info in mods {
        if let Some(ref parent_id) = mod_info.content_pack_for {
            parent_map
                .entry(parent_id.clone())
                .or_insert_with(Vec::new)
                .push(mod_info);
        } else {
            standalone_mods.push(mod_info);
        }
    }

    // Step 2: For each standalone mod that has content packs, create a group entry
    let mut result: Vec<ModInfo> = Vec::new();

    for mut mod_info in standalone_mods {
        if let Some(sub_mods) = parent_map.remove(&mod_info.unique_id) {
            if !sub_mods.is_empty() {
                println!(
                    "[mod_parser] Grouping {} content packs under '{}'",
                    sub_mods.len(),
                    mod_info.name
                );

                // Merge dependencies from all sub-mods
                let mut all_deps: Vec<ModDependencyInfo> = mod_info.dependencies.clone();
                for sub in &sub_mods {
                    for dep in &sub.dependencies {
                        if !all_deps.iter().any(|d| d.unique_id.to_lowercase() == dep.unique_id.to_lowercase()) {
                            all_deps.push(dep.clone());
                        }
                    }
                }

                // Collect all internal component IDs
                let mut component_ids: Vec<String> = vec![mod_info.unique_id.clone()];
                for sub in &sub_mods {
                    component_ids.push(sub.unique_id.clone());
                }

                // Determine group enabled state: enabled if any component is enabled
                let group_enabled = mod_info.enabled || sub_mods.iter().any(|s| s.enabled);

                // Create group entry
                let group_mod = ModInfo {
                    name: mod_info.name.clone(),
                    version: mod_info.version.clone(),
                    author: mod_info.author.clone(),
                    description: mod_info.description.clone(),
                    unique_id: mod_info.unique_id.clone(),
                    enabled: group_enabled,
                    is_required: mod_info.is_required,
                    has_dependencies: !all_deps.is_empty(),
                    dependency_count: all_deps.len(),
                    is_content_pack: false,
                    content_pack_for: None,
                    folder_path: mod_info.folder_path.clone(),
                    has_conflict: mod_info.has_conflict || sub_mods.iter().any(|s| s.has_conflict),
                    conflict_warning: mod_info.conflict_warning.clone(),
                    url: mod_info.url.clone(),
                    category: mod_info.category.clone(),
                    screenshot_path: mod_info.screenshot_path.clone(),
                    thumbnail_path: mod_info.thumbnail_path.clone(),
                    has_update: mod_info.has_update || sub_mods.iter().any(|s| s.has_update),
                    latest_version: mod_info.latest_version.clone(),
                    update_url: mod_info.update_url.clone(),
                    dependencies: all_deps,
                    manifest_content: mod_info.manifest_content.clone(),
                    sub_mods,
                    is_group: true,
                    internal_component_ids: component_ids,
                    nexus_mod_id: mod_info.nexus_mod_id,
                };

                result.push(group_mod);
            } else {
                result.push(mod_info);
            }
        } else {
            result.push(mod_info);
        }
    }

    result
}

fn toggle_single_mod(mod_path: &PathBuf, enabled: bool) -> Result<(), String> {
    if !mod_path.exists() {
        return Err(format!("MOD path does not exist: {}", mod_path.display()));
    }

    let parent = mod_path
        .parent()
        .ok_or("Cannot determine parent directory")?;
    let folder_name = mod_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    if enabled {
        if folder_name.starts_with('.') && !folder_name.starts_with("..") {
            let clean_name = &folder_name[1..];
            let new_path = parent.join(clean_name);
            if new_path.exists() && new_path != *mod_path {
                return Err(format!("Cannot enable MOD, target already exists: {}", new_path.display()));
            }
            fs::rename(mod_path, &new_path)
                .map_err(|e| format!("Failed to enable MOD: {}", e))?;
            println!("[mod_parser] Enabled: {} -> {}", mod_path.display(), new_path.display());
        }
    } else {
        if !folder_name.starts_with('.') {
            let new_name = format!(".{}", folder_name);
            let new_path = parent.join(&new_name);
            if new_path.exists() {
                return Err(format!("Cannot disable MOD, target already exists: {}", new_path.display()));
            }
            fs::rename(mod_path, &new_path)
                .map_err(|e| format!("Failed to disable MOD: {}", e))?;
            println!("[mod_parser] Disabled: {} -> {}", mod_path.display(), new_path.display());
        }
    }

    Ok(())
}

fn find_sub_mod_folders(dir: &PathBuf) -> Vec<PathBuf> {
    let mut sub_mods = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.join("manifest.json").exists() {
                    sub_mods.push(path.clone());
                }
                let nested = find_sub_mod_folders(&path);
                sub_mods.extend(nested);
            }
        }
    }
    sub_mods
}

fn find_sibling_mods(mod_path: &PathBuf) -> Vec<PathBuf> {
    let mut siblings = Vec::new();

    let parent = match mod_path.parent() {
        Some(p) => p,
        None => return siblings,
    };

    let folder_name = parent.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if folder_name.starts_with("_") {
        return siblings;
    }

    if parent.join("manifest.json").exists() {
        return siblings;
    }

    if let Ok(entries) = fs::read_dir(parent) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if path == *mod_path {
                continue;
            }
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with('_') {
                continue;
            }
            if name == "." || name == ".." {
                continue;
            }
            if path.join("manifest.json").exists() || find_sub_mod_folders(&path).iter().any(|p| p.join("manifest.json").exists()) {
                siblings.push(path);
            }
        }
    }

    siblings
}

#[tauri::command]
pub fn toggle_mod_enabled(mod_path: String, enabled: bool, extra_paths: Option<Vec<String>>) -> Result<bool, String> {
    println!("[toggle_mod_enabled] Requested: path={}, enabled={}, extra_paths={:?}", mod_path, enabled, extra_paths);

    let path = PathBuf::from(&mod_path);

    let mut all_paths: Vec<PathBuf> = if let Some(ref extras) = extra_paths {
        let mut paths = vec![path];
        for p in extras {
            paths.push(PathBuf::from(p));
        }
        paths
    } else {
        vec![path]
    };

    let siblings = find_sibling_mods(&all_paths[0]);
    if !siblings.is_empty() {
        println!("[toggle_mod_enabled] Found {} sibling mods to toggle together", siblings.len());
        for s in siblings {
            if !all_paths.contains(&s) {
                all_paths.push(s);
            }
        }
    }

    for current_path in &all_paths {
        if !current_path.exists() {
            println!("[toggle_mod_enabled] Path does not exist, skipping: {}", current_path.display());
            continue;
        }

        let has_manifest = current_path.join("manifest.json").exists();
        let sub_mods = find_sub_mod_folders(current_path);

        println!("[toggle_mod_enabled] Processing: {}, has_manifest={}, sub_mods={}", 
            current_path.display(), has_manifest, sub_mods.len());

        if has_manifest {
            toggle_single_mod(current_path, enabled)?;
        } else if !sub_mods.is_empty() {
            for sub_mod in &sub_mods {
                toggle_single_mod(sub_mod, enabled)?;
            }
        }
    }

    println!("[toggle_mod_enabled] Completed successfully");
    Ok(true)
}
