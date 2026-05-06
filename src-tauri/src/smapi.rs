use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmapiInfo {
    pub installed: bool,
    pub version: Option<String>,
    pub game_path: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePathInfo {
    pub steam_path: Option<String>,
    pub gog_path: Option<String>,
    pub detected_path: Option<String>,
    pub detection_method: Option<String>,
}

const STEAM_DEFAULT_PATHS: &[&str] = &[
    r"C:\Program Files (x86)\Steam\steamapps\common\Stardew Valley",
    r"C:\Program Files\Steam\steamapps\common\Stardew Valley",
    r"D:\steam\steamapps\common\Stardew Valley",
];

const GOG_DEFAULT_PATHS: &[&str] = &[
    r"C:\GOG Games\Stardew Valley",
    r"C:\Program Files (x86)\GOG Galaxy\Games\Stardew Valley",
];

pub(crate) fn find_game_path() -> Option<(PathBuf, String)> {
    println!("[smapi] Detecting game path...");
    
    if let Some(path) = find_via_steam_registry() {
        println!("[smapi] Found via Steam Registry: {}", path.display());
        return Some((path, "Steam Registry".to_string()));
    }

    if let Some(path) = find_via_steam_library_folders() {
        println!("[smapi] Found via Steam Library: {}", path.display());
        return Some((path, "Steam Library".to_string()));
    }

    for path in STEAM_DEFAULT_PATHS {
        let p = PathBuf::from(path);
        println!("[smapi] Checking default path: {} (exists: {})", path, p.exists());
        if p.exists() && is_valid_game_path(&p) {
            println!("[smapi] Found via Steam Default: {}", path);
            return Some((p, "Steam Default".to_string()));
        }
    }

    for path in GOG_DEFAULT_PATHS {
        let p = PathBuf::from(path);
        if p.exists() && is_valid_game_path(&p) {
            println!("[smapi] Found via GOG Default: {}", path);
            return Some((p, "GOG Default".to_string()));
        }
    }

    println!("[smapi] Game path not found");
    None
}

#[cfg(target_os = "windows")]
fn find_via_steam_registry() -> Option<PathBuf> {
    let reg_paths = [
        r"HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Valve\Steam",
        r"HKEY_LOCAL_MACHINE\SOFTWARE\Valve\Steam",
        r"HKEY_CURRENT_USER\SOFTWARE\Valve\Steam",
    ];

    for reg_path in &reg_paths {
        if let Some(install_path) = query_registry(reg_path, "InstallPath") {
            let steam_path = PathBuf::from(&install_path);
            let library_path = steam_path
                .join("steamapps")
                .join("common")
                .join("Stardew Valley");

            if library_path.exists() && is_valid_game_path(&library_path) {
                return Some(library_path);
            }
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn query_registry(reg_path: &str, value_name: &str) -> Option<String> {
    let output = Command::new("reg")
        .args(["query", reg_path, "/v", value_name])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("REG_SZ") || trimmed.contains("REG_SZ") {
            if let Some(idx) = trimmed.find("REG_SZ") {
                let value = trimmed[idx + 6..].trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }

    None
}

#[cfg(not(target_os = "windows"))]
fn find_via_steam_registry() -> Option<PathBuf> {
    None
}

fn find_via_steam_library_folders() -> Option<PathBuf> {
    // Try to get Steam install dir from registry first
    let steam_install_dir = find_via_steam_registry()
        .or_else(|| {
            // Try all possible Steam install paths (including D: drive)
            STEAM_DEFAULT_PATHS
                .iter()
                .map(PathBuf::from)
                .find(|p| p.exists())
                .and_then(|game_path| {
                    game_path.parent()?.parent()?.parent().map(|p| p.to_path_buf())
                })
        })?;

    println!("[smapi] Steam install dir: {}", steam_install_dir.display());

    let library_folders_path = steam_install_dir
        .join("steamapps")
        .join("libraryfolders.vdf");

    println!("[smapi] Checking libraryfolders.vdf: {} (exists: {})", library_folders_path.display(), library_folders_path.exists());

    if library_folders_path.exists() {
        if let Some(path) = parse_library_folders(&library_folders_path) {
            println!("[smapi] Found library path: {}", path.display());
            let game_path = path
                .join("steamapps")
                .join("common")
                .join("Stardew Valley");

            if game_path.exists() && is_valid_game_path(&game_path) {
                return Some(game_path);
            }
        }
    }

    None
}

fn parse_library_folders(vdf_path: &PathBuf) -> Option<PathBuf> {
    let content = std::fs::read_to_string(vdf_path).ok()?;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.contains("\"path\"") {
            if let Some(path_str) = trimmed.split('"').nth(3) {
                let path = PathBuf::from(path_str);
                if path.exists() {
                    return Some(path);
                }
            }
        }
    }

    None
}

fn is_valid_game_path(path: &PathBuf) -> bool {
    path.join("Stardew Valley.exe").exists()
        || path.join("StardewModdingAPI.exe").exists()
        || path.join("StardewModdingAPI.dll").exists()
}

fn detect_smapi_version(game_path: &PathBuf) -> Option<String> {
    let manifest_path = game_path.join(".smapi").join("manifest.json");

    if manifest_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&manifest_path) {
            if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(version) = manifest.get("Version").and_then(|v| v.as_str()) {
                    return Some(version.to_string());
                }
            }
        }
    }

    let api_dll = game_path.join("StardewModdingAPI.dll");
    if api_dll.exists() {
        return Some("Installed".to_string());
    }

    let api_exe = game_path.join("StardewModdingAPI.exe");
    if api_exe.exists() {
        return Some("Installed".to_string());
    }

    None
}

#[tauri::command]
pub fn detect_game_path() -> Result<GamePathInfo, String> {
    let (detected_path, method) = find_game_path()
        .map(|(p, m)| (p.to_string_lossy().to_string(), m))
        .unzip();

    let steam_path = STEAM_DEFAULT_PATHS
        .iter()
        .map(PathBuf::from)
        .find(|p| p.exists() && is_valid_game_path(p))
        .map(|p| p.to_string_lossy().to_string());

    let gog_path = GOG_DEFAULT_PATHS
        .iter()
        .map(PathBuf::from)
        .find(|p| p.exists() && is_valid_game_path(p))
        .map(|p| p.to_string_lossy().to_string());

    Ok(GamePathInfo {
        steam_path,
        gog_path,
        detected_path,
        detection_method: method,
    })
}

#[tauri::command]
pub fn check_smapi_status(custom_path: Option<String>) -> Result<SmapiInfo, String> {
    let game_path = match custom_path {
        Some(path) => PathBuf::from(path),
        None => find_game_path()
            .map(|(p, _)| p)
            .ok_or("未找到星露谷物语游戏目录，请手动指定路径")?,
    };

    if !game_path.exists() {
        return Err(format!("游戏路径不存在: {}", game_path.display()));
    }

    if !is_valid_game_path(&game_path) {
        return Err("未找到有效的星露谷物语游戏目录".to_string());
    }

    let smapi_version = detect_smapi_version(&game_path);
    let installed = smapi_version.is_some();

    Ok(SmapiInfo {
        installed,
        version: smapi_version,
        game_path: Some(game_path.to_string_lossy().to_string()),
        error: if !installed {
            Some("未检测到 SMAPI，请先安装 SMAPI".to_string())
        } else {
            None
        },
    })
}

#[tauri::command]
pub fn set_custom_game_path(path: &str) -> Result<GamePathInfo, String> {
    let game_path = PathBuf::from(path);

    if !game_path.exists() {
        return Err(format!("路径不存在: {}", game_path.display()));
    }

    if !is_valid_game_path(&game_path) {
        return Err("未找到有效的星露谷物语游戏目录".to_string());
    }

    Ok(GamePathInfo {
        steam_path: None,
        gog_path: None,
        detected_path: Some(game_path.to_string_lossy().to_string()),
        detection_method: Some("Manual".to_string()),
    })
}

#[tauri::command]
pub fn restore_svl_window(app: tauri::AppHandle) -> Result<bool, String> {
    let window = app.get_webview_window("main");
    if let Some(win) = window {
        win.show().ok();
        win.set_focus().ok();
    }
    Ok(true)
}

#[tauri::command]
pub fn open_smapi_installer() -> Result<bool, String> {
    tauri_plugin_opener::open_url("https://smapi.io", Option::<&str>::None)
        .map_err(|e| format!("Cannot open SMAPI installer page: {}", e))?;
    Ok(true)
}
