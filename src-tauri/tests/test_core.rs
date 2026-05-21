// Core functionality tests for SVL backend
// Tests MOD parsing, conflict detection, and profile management

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Helper: create a valid mod folder with manifest.json
fn create_valid_mod(mods_dir: &PathBuf, folder_name: &str, manifest_content: &str) -> PathBuf {
    let mod_path = mods_dir.join(folder_name);
    fs::create_dir_all(&mod_path).expect("Failed to create mod folder");
    fs::write(mod_path.join("manifest.json"), manifest_content).expect("Failed to write manifest");
    mod_path
}

// Helper: create manifest JSON string
fn make_manifest(
    name: Option<&str>,
    version: Option<&str>,
    author: Option<&str>,
    unique_id: Option<&str>,
    dependencies: Option<Vec<(&str, Option<&str>, Option<bool>)>>,
) -> String {
    let mut json = String::from("{");
    let mut parts = Vec::new();

    if let Some(n) = name {
        parts.push(format!(r#""Name": "{}""#, n));
    }
    if let Some(v) = version {
        parts.push(format!(r#""Version": "{}""#, v));
    }
    if let Some(a) = author {
        parts.push(format!(r#""Author": "{}""#, a));
    }
    if let Some(uid) = unique_id {
        parts.push(format!(r#""UniqueID": "{}""#, uid));
    }
    if let Some(deps) = dependencies {
        let deps_json: Vec<String> = deps
            .iter()
            .map(|(uid, min_ver, is_req)| {
                let mut dep = format!(r#"{{"UniqueID": "{}""#, uid);
                if let Some(mv) = min_ver {
                    dep.push_str(&format!(r#", "MinimumVersion": "{}""#, mv));
                }
                if let Some(ir) = is_req {
                    dep.push_str(&format!(r#", "IsRequired": {}"#, ir));
                }
                dep.push('}');
                dep
            })
            .collect();
        parts.push(format!(r#""Dependencies": [{}]"#, deps_json.join(", ")));
    }

    json.push_str(&parts.join(", "));
    json.push('}');
    json
}

#[test]
fn test_scan_mods_empty_folder() {
    // Create temporary game directory with empty Mods folder
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let game_path = temp_dir.path();
    let mods_path = game_path.join("Mods");
    fs::create_dir_all(&mods_path).expect("Failed to create Mods folder");

    // Call scan_mods with the temporary game path
    let result = svl_lib::mod_parser::scan_mods(Some(game_path.to_string_lossy().to_string()));

    assert!(result.is_ok(), "scan_mods should succeed on empty folder");
    let mods = result.unwrap();
    assert_eq!(mods.len(), 0, "Should return empty array for empty Mods folder");
}

#[test]
fn test_scan_mods_with_valid_mod() {
    // Create temporary game directory with a valid mod
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let game_path = temp_dir.path();
    let mods_path = game_path.join("Mods");
    fs::create_dir_all(&mods_path).expect("Failed to create Mods folder");

    let manifest = make_manifest(
        Some("Test Mod"),
        Some("1.2.3"),
        Some("TestAuthor"),
        Some("TestAuthor.TestMod"),
        None,
    );
    create_valid_mod(&mods_path, "TestMod", &manifest);

    let result = svl_lib::mod_parser::scan_mods(Some(game_path.to_string_lossy().to_string()));

    assert!(result.is_ok(), "scan_mods should succeed with valid mod");
    let mods = result.unwrap();
    assert_eq!(mods.len(), 1, "Should find exactly 1 mod");

    let mod_info = &mods[0];
    assert_eq!(mod_info.name, "Test Mod");
    assert_eq!(mod_info.version, "1.2.3");
    assert_eq!(mod_info.author, "TestAuthor");
    assert_eq!(mod_info.unique_id, "TestAuthor.TestMod");
}

#[test]
fn test_scan_mods_missing_name() {
    // Create mod with missing Name field, should fallback to folder name
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let game_path = temp_dir.path();
    let mods_path = game_path.join("Mods");
    fs::create_dir_all(&mods_path).expect("Failed to create Mods folder");

    let manifest = make_manifest(
        None, // No Name field
        Some("1.0.0"),
        Some("Author"),
        Some("Author.SomeMod"),
        None,
    );
    create_valid_mod(&mods_path, "MyCoolMod", &manifest);

    let result = svl_lib::mod_parser::scan_mods(Some(game_path.to_string_lossy().to_string()));

    assert!(result.is_ok(), "scan_mods should succeed");
    let mods = result.unwrap();
    assert_eq!(mods.len(), 1);

    // Should use folder name as fallback
    assert_eq!(mods[0].name, "MyCoolMod");
}

#[test]
fn test_check_conflicts_missing_required_dependency() {
    // Create mod A that depends on mod B (IsRequired: true), but B is not installed
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let game_path = temp_dir.path();
    let mods_path = game_path.join("Mods");
    fs::create_dir_all(&mods_path).expect("Failed to create Mods folder");

    // Mod A with required dependency on B
    let manifest_a = make_manifest(
        Some("Mod A"),
        Some("1.0.0"),
        Some("AuthorA"),
        Some("AuthorA.ModA"),
        Some(vec![("AuthorB.ModB", None, Some(true))]), // IsRequired: true
    );
    create_valid_mod(&mods_path, "ModA", &manifest_a);

    let result = svl_lib::mod_parser::scan_mods(Some(game_path.to_string_lossy().to_string()));
    assert!(result.is_ok());
    let mods = result.unwrap();

    // Check conflicts
    let conflict_result = svl_lib::conflict_checker::check_conflicts(mods);
    assert!(conflict_result.is_ok());
    let conflicts = conflict_result.unwrap();

    // Should have at least one conflict (missing required dependency)
    assert!(!conflicts.is_empty(), "Should detect missing required dependency");

    // Find the missing dependency conflict
    let missing_dep_conflict = conflicts
        .iter()
        .find(|c| matches!(c.conflict_type, svl_lib::conflict_checker::ConflictType::MissingDependency));

    assert!(
        missing_dep_conflict.is_some(),
        "Should have MissingDependency conflict type"
    );

    let conflict = missing_dep_conflict.unwrap();
    assert!(
        matches!(conflict.severity, svl_lib::conflict_checker::Severity::Error),
        "Missing required dependency should be Error severity"
    );
}

#[test]
fn test_check_conflicts_optional_dependency_info_level() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let game_path = temp_dir.path();
    let mods_path = game_path.join("Mods");
    fs::create_dir_all(&mods_path).expect("Failed to create Mods folder");

    let manifest_a = make_manifest(
        Some("Mod A"),
        Some("1.0.0"),
        Some("AuthorA"),
        Some("AuthorA.ModA"),
        Some(vec![("AuthorB.ModB", None, Some(false))]),
    );
    create_valid_mod(&mods_path, "ModA", &manifest_a);

    let result = svl_lib::mod_parser::scan_mods(Some(game_path.to_string_lossy().to_string()));
    assert!(result.is_ok());
    let mods = result.unwrap();

    let conflict_result = svl_lib::conflict_checker::check_conflicts(mods);
    assert!(conflict_result.is_ok());
    let conflicts = conflict_result.unwrap();

    let optional_conflict = conflicts
        .iter()
        .find(|c| matches!(c.conflict_type, svl_lib::conflict_checker::ConflictType::OptionalDependencyMissing));

    assert!(
        optional_conflict.is_some(),
        "Optional dependency should be reported as OptionalDependencyMissing"
    );

    let conflict = optional_conflict.unwrap();
    assert!(
        matches!(conflict.severity, svl_lib::conflict_checker::Severity::Info),
        "Optional dependency missing should be Info severity, not Error"
    );
    assert!(
        conflict.description.contains("AuthorB.ModB"),
        "Description should mention the missing dependency"
    );
}

#[test]
fn test_create_and_delete_profile() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let game_path = temp_dir.path().to_string_lossy().to_string();

    let profiles_dir = svl_lib::profiles::get_profiles_dir(&game_path)
        .expect("Failed to get profiles dir");

    let all_mods = svl_lib::profiles::scan_mods_for_profiles(&game_path);
    let enabled_ids: Vec<String> = all_mods.iter().map(|m| m.unique_id.clone()).collect();

    let now = chrono::Utc::now().to_rfc3339();
    let profile = svl_lib::profiles::Profile {
        name: "TestProfile".to_string(),
        is_protected: false,
        enabled_mod_ids: enabled_ids,
        created_at: now.clone(),
        last_used: now,
    };

    let profile_path = profiles_dir.join("TestProfile.json");
    let json = serde_json::to_string_pretty(&profile).expect("Failed to serialize profile");
    fs::write(&profile_path, &json).expect("Failed to write profile file");

    assert!(profile_path.exists(), "Profile file should exist after creation");

    let loaded = svl_lib::profiles::load_profile(&game_path, "TestProfile");
    assert!(loaded.is_ok(), "Should load profile successfully");
    let loaded_config = loaded.unwrap();
    assert_eq!(loaded_config.name, "TestProfile");

    fs::remove_file(&profile_path).expect("Should delete profile file successfully");
    assert!(!profile_path.exists(), "Profile file should not exist after deletion");
}
