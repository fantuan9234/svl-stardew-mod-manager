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
    pub xbox_path: Option<String>,
    pub detected_path: Option<String>,
    pub detection_method: Option<String>,
}

#[cfg(target_os = "windows")]
const STEAM_DEFAULT_PATHS: &[&str] = &[
    r"C:\Program Files (x86)\Steam\steamapps\common\Stardew Valley",
    r"C:\Program Files\Steam\steamapps\common\Stardew Valley",
    r"D:\Steam\steamapps\common\Stardew Valley",
    r"D:\steam\steamapps\common\Stardew Valley",
    r"E:\Steam\steamapps\common\Stardew Valley",
    r"E:\steam\steamapps\common\Stardew Valley",
    r"F:\Steam\steamapps\common\Stardew Valley",
    r"F:\steam\steamapps\common\Stardew Valley",
];

#[cfg(target_os = "macos")]
const STEAM_DEFAULT_PATHS: &[&str] = &[
    "/Applications/Stardew Valley.app/Contents/MacOS",
    "~/Library/Application Support/Steam/steamapps/common/Stardew Valley.app/Contents/MacOS",
    "~/Library/Application Support/Steam/steamapps/common/Stardew Valley",
];

#[cfg(target_os = "linux")]
// Linux support disabled (see .github/workflows/build.yml). Keep code for future re-enable.
const STEAM_DEFAULT_PATHS: &[&str] = &[
    "~/.steam/steam/steamapps/common/Stardew Valley",
    "~/.local/share/Steam/steamapps/common/Stardew Valley",
    "/usr/share/steam/steamapps/common/Stardew Valley",
];

#[cfg(target_os = "windows")]
const GOG_DEFAULT_PATHS: &[&str] = &[
    r"C:\GOG Games\Stardew Valley",
    r"C:\Program Files (x86)\GOG Galaxy\Games\Stardew Valley",
    r"D:\GOG Games\Stardew Valley",
    r"E:\GOG Games\Stardew Valley",
];

#[cfg(not(target_os = "windows"))]
const GOG_DEFAULT_PATHS: &[&str] = &[];

#[cfg(target_os = "windows")]
const XBOX_DEFAULT_PATHS: &[&str] = &[
    r"C:\XboxGames\Stardew Valley\Content",
    r"C:\Program Files\WindowsApps\Stardew Valley\Content",
    r"D:\XboxGames\Stardew Valley\Content",
];

#[cfg(not(target_os = "windows"))]
const XBOX_DEFAULT_PATHS: &[&str] = &[];

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
        let p = expand_tilde(PathBuf::from(path));
        println!("[smapi] Checking default path: {} (exists: {})", path, p.exists());
        if p.exists() && is_valid_game_path(&p) {
            println!("[smapi] Found via Steam Default: {}", path);
            return Some((p, "Steam Default".to_string()));
        }
    }

    for path in GOG_DEFAULT_PATHS {
        let p = expand_tilde(PathBuf::from(path));
        if p.exists() && is_valid_game_path(&p) {
            println!("[smapi] Found via GOG Default: {}", path);
            return Some((p, "GOG Default".to_string()));
        }
    }

    for path in XBOX_DEFAULT_PATHS {
        let p = expand_tilde(PathBuf::from(path));
        if p.exists() && is_valid_game_path(&p) {
            println!("[smapi] Found via Xbox Default: {}", path);
            return Some((p, "Xbox Game Pass".to_string()));
        }
    }

    if let Some(path) = find_via_disk_scan() {
        println!("[smapi] Found via Disk Scan: {}", path.display());
        return Some((path, "Disk Scan".to_string()));
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
    use std::os::windows::process::CommandExt;
    let output = Command::new("reg")
        .args(["query", reg_path, "/v", value_name])
        .creation_flags(0x08000000)
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
    let steam_install_dir = find_steam_install_dir()?;
    println!("[smapi] Steam install dir: {}", steam_install_dir.display());

    let library_folders_path = steam_install_dir
        .join("steamapps")
        .join("libraryfolders.vdf");

    println!("[smapi] Checking libraryfolders.vdf: {} (exists: {})", library_folders_path.display(), library_folders_path.exists());

    if library_folders_path.exists() {
        if let Some(path) = parse_library_folders(&library_folders_path) {
            println!("[smapi] Found library path: {}", path.display());
            if let Some(game_path) = resolve_steam_game_path(&path) {
                return Some(game_path);
            }
        }

        if let Some(paths) = parse_all_library_folders(&library_folders_path) {
            for lib_path in paths {
                if let Some(game_path) = resolve_steam_game_path(&lib_path) {
                    return Some(game_path);
                }
            }
        }
    }

    None
}

fn resolve_steam_game_path(library_path: &PathBuf) -> Option<PathBuf> {
    let game_path = library_path
        .join("steamapps")
        .join("common")
        .join("Stardew Valley");

    if game_path.exists() && is_valid_game_path(&game_path) {
        return Some(game_path);
    }

    #[cfg(target_os = "macos")]
    {
        let app_path = game_path.join("Stardew Valley.app");
        if app_path.exists() {
            let macos_inner = app_path.join("Contents").join("MacOS");
            if macos_inner.exists() && is_valid_game_path(&macos_inner) {
                return Some(macos_inner);
            }
            if is_valid_game_path(&app_path) {
                return Some(app_path);
            }
        }
    }

    None
}

fn find_steam_install_dir() -> Option<PathBuf> {
    if let Some(path) = find_via_steam_registry_raw() {
        return Some(path);
    }

    #[cfg(target_os = "windows")]
    let steam_candidates = [
        r"C:\Program Files (x86)\Steam",
        r"C:\Program Files\Steam",
        r"D:\Steam",
        r"D:\steam",
        r"E:\Steam",
        r"E:\steam",
        r"F:\Steam",
        r"F:\steam",
    ];

    #[cfg(target_os = "linux")]
    let steam_candidates = [
        "~/.steam/steam",
        "~/.local/share/Steam",
        "~/.steam",
        "/usr/share/steam",
    ];

    #[cfg(target_os = "macos")]
    let steam_candidates = [
        "~/Library/Application Support/Steam",
        "~/Library/Application Support/Steam/Contents/MacOS",
    ];

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let steam_candidates: [&str; 0] = [];

    for candidate in &steam_candidates {
        let p = PathBuf::from(candidate);
        let p = expand_tilde(p);
        if p.exists() && p.join("steamapps").exists() {
            return Some(p);
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn find_via_steam_registry_raw() -> Option<PathBuf> {
    let reg_paths = [
        r"HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Valve\Steam",
        r"HKEY_LOCAL_MACHINE\SOFTWARE\Valve\Steam",
        r"HKEY_CURRENT_USER\SOFTWARE\Valve\Steam",
    ];

    for reg_path in &reg_paths {
        if let Some(install_path) = query_registry(reg_path, "InstallPath") {
            let steam_path = PathBuf::from(&install_path);
            if steam_path.exists() {
                return Some(steam_path);
            }
        }
    }

    None
}

#[cfg(not(target_os = "windows"))]
fn find_via_steam_registry_raw() -> Option<PathBuf> {
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

fn parse_all_library_folders(vdf_path: &PathBuf) -> Option<Vec<PathBuf>> {
    let content = std::fs::read_to_string(vdf_path).ok()?;
    let mut paths = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.contains("\"path\"") {
            if let Some(path_str) = trimmed.split('"').nth(3) {
                let path = PathBuf::from(path_str);
                if path.exists() {
                    paths.push(path);
                }
            }
        }
    }

    if paths.is_empty() {
        None
    } else {
        Some(paths)
    }
}

fn find_via_disk_scan() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let drives = ['C', 'D', 'E', 'F', 'G', 'H'];

        for drive in &drives {
            let steam_path = PathBuf::from(format!("{}:\\Steam\\steamapps\\common\\Stardew Valley", drive));
            if steam_path.exists() && is_valid_game_path(&steam_path) {
                return Some(steam_path);
            }

            let steam_path_lower = PathBuf::from(format!("{}:\\steam\\steamapps\\common\\Stardew Valley", drive));
            if steam_path_lower.exists() && is_valid_game_path(&steam_path_lower) {
                return Some(steam_path_lower);
            }

            let gog_path = PathBuf::from(format!("{}:\\GOG Games\\Stardew Valley", drive));
            if gog_path.exists() && is_valid_game_path(&gog_path) {
                return Some(gog_path);
            }

            let xbox_path = PathBuf::from(format!("{}:\\XboxGames\\Stardew Valley\\Content", drive));
            if xbox_path.exists() && is_valid_game_path(&xbox_path) {
                return Some(xbox_path);
            }

            let game_path = PathBuf::from(format!("{}:\\Stardew Valley", drive));
            if game_path.exists() && is_valid_game_path(&game_path) {
                return Some(game_path);
            }

            let games_path = PathBuf::from(format!("{}:\\Games\\Stardew Valley", drive));
            if games_path.exists() && is_valid_game_path(&games_path) {
                return Some(games_path);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let linux_paths = [
            "~/Games/Stardew Valley",
            "~/stardew-valley",
            "/opt/stardew-valley",
            "/usr/share/stardew-valley",
        ];
        for path_str in &linux_paths {
            let p = expand_tilde(PathBuf::from(path_str));
            if p.exists() && is_valid_game_path(&p) {
                return Some(p);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let macos_paths = [
            "~/Games/Stardew Valley",
            "/Applications/Stardew Valley.app",
            "/Applications/Stardew Valley.app/Contents/MacOS",
            "~/Library/Application Support/Steam/steamapps/common/Stardew Valley",
            "~/Library/Application Support/Steam/steamapps/common/Stardew Valley.app",
            "~/Library/Application Support/Steam/steamapps/common/Stardew Valley.app/Contents/MacOS",
        ];
        for path_str in &macos_paths {
            let p = expand_tilde(PathBuf::from(path_str));
            if p.exists() && is_valid_game_path(&p) {
                return Some(p);
            }
        }
    }

    None
}

fn expand_tilde(path: PathBuf) -> PathBuf {
    let home = std::env::var("HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(dirs::home_dir);

    if let Some(home) = home {
        let path_str = path.to_string_lossy().to_string();
        if path_str.starts_with("~/") {
            let rest = &path_str[2..];
            return home.join(rest);
        }
        if path_str.starts_with("~") {
            let rest = &path_str[1..];
            return home.join(rest);
        }
    }
    path
}

fn is_valid_game_path(path: &PathBuf) -> bool {
    #[cfg(target_os = "windows")]
    {
        path.join("Stardew Valley.exe").exists()
            || path.join("StardewModdingAPI.exe").exists()
            || path.join("StardewModdingAPI.dll").exists()
            || path.join("Content").join("Stardew Valley.exe").exists()
            || (path.join("Content").is_dir() && path.join("Content").join("XNA").is_dir())
    }

    #[cfg(target_os = "linux")]
    // Linux support disabled (see .github/workflows/build.yml). Keep code for future re-enable.
    {
        path.join("Stardew Valley").exists()
            || path.join("StardewModdingAPI").exists()
            || path.join("StardewModdingAPI.dll").exists()
            || (path.join("Content").is_dir() && path.join("Content").join("Linux").is_dir())
    }

    #[cfg(target_os = "macos")]
    {
        let inner_exe = path.join("Stardew Valley");
        let inner_smapi = path.join("StardewModdingAPI");
        let inner_smapi_dll = path.join("StardewModdingAPI.dll");
        let app_exe = path.join("Contents").join("MacOS").join("Stardew Valley");
        let app_smapi = path.join("Contents").join("MacOS").join("StardewModdingAPI");
        let nested_app_exe = path.join("Stardew Valley.app").join("Contents").join("MacOS").join("Stardew Valley");
        let nested_app_smapi = path.join("Stardew Valley.app").join("Contents").join("MacOS").join("StardewModdingAPI");
        let has_content_macos = path.join("Content").is_dir() && path.join("Content").join("MacOS").is_dir();
        let has_app_bundle = path.join("Stardew Valley.app").is_dir() && path.join("Stardew Valley.app").join("Contents").is_dir();

        inner_exe.exists()
            || inner_smapi.exists()
            || inner_smapi_dll.exists()
            || app_exe.exists()
            || app_smapi.exists()
            || nested_app_exe.exists()
            || nested_app_smapi.exists()
            || has_content_macos
            || has_app_bundle
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    // Linux support disabled (see .github/workflows/build.yml). Keep code for future re-enable.
    {
        path.join("Stardew Valley.exe").exists()
            || path.join("StardewModdingAPI.dll").exists()
    }
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

    #[cfg(target_os = "windows")]
    {
        let api_exe = game_path.join("StardewModdingAPI.exe");
        if api_exe.exists() {
            return Some("Installed".to_string());
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let api_bin = game_path.join("StardewModdingAPI");
        if api_bin.exists() {
            return Some("Installed".to_string());
        }
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
        .map(|p| expand_tilde(PathBuf::from(p)))
        .find(|p| p.exists() && is_valid_game_path(p))
        .map(|p| p.to_string_lossy().to_string());

    let gog_path = GOG_DEFAULT_PATHS
        .iter()
        .map(|p| expand_tilde(PathBuf::from(p)))
        .find(|p| p.exists() && is_valid_game_path(p))
        .map(|p| p.to_string_lossy().to_string());

    let xbox_path = XBOX_DEFAULT_PATHS
        .iter()
        .map(|p| expand_tilde(PathBuf::from(p)))
        .find(|p| p.exists() && is_valid_game_path(p))
        .map(|p| p.to_string_lossy().to_string());

    Ok(GamePathInfo {
        steam_path,
        gog_path,
        xbox_path,
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
        xbox_path: None,
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
