use serde::{Deserialize, Serialize};
use std::io::Write;
use tauri::Emitter;

const UPDATE_SERVER_BASE_URL: &str = "http://211.101.247.248:8866";
const GITHUB_REPO: &str = "fantuan9234/svl-stardew-mod-manager";
const GITHUB_API_BASE: &str = "https://api.github.com";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUpdateInfo {
    pub has_update: bool,
    pub current_version: String,
    pub latest_version: String,
    pub download_url: String,
    pub release_notes: Option<String>,
    pub release_date: Option<String>,
    pub file_size: Option<u64>,
    pub sha256: Option<String>,
    pub force_update: bool,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUpdateProgress {
    pub downloaded: u64,
    pub total: u64,
    pub percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUpdateResult {
    pub success: bool,
    pub message: String,
    pub needs_restart: bool,
    pub file_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    body: Option<String>,
    published_at: Option<String>,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

fn version_is_newer(latest: &str, current: &str) -> bool {
    let latest_parts: Vec<u32> = latest
        .trim_start_matches('v')
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    let current_parts: Vec<u32> = current
        .trim_start_matches('v')
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    for i in 0..std::cmp::max(latest_parts.len(), current_parts.len()) {
        let l = latest_parts.get(i).unwrap_or(&0);
        let c = current_parts.get(i).unwrap_or(&0);
        if l > c {
            return true;
        }
        if l < c {
            return false;
        }
    }
    false
}

async fn check_update_from_github(current_version: &str) -> Result<AppUpdateInfo, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("SVL-StardewModManager")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let url = format!("{}/repos/{}/releases/latest", GITHUB_API_BASE, GITHUB_REPO);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("连接 GitHub 更新源失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "GitHub 更新源返回错误: {}",
            response.status()
        ));
    }

    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| format!("解析 GitHub 发布信息失败: {}", e))?;

    let latest_version = release.tag_name.trim_start_matches('v').to_string();

    if !version_is_newer(&latest_version, current_version) {
        return Ok(AppUpdateInfo {
            has_update: false,
            current_version: current_version.to_string(),
            latest_version,
            download_url: String::new(),
            release_notes: None,
            release_date: None,
            file_size: None,
            sha256: None,
            force_update: false,
            source: Some("github".to_string()),
        });
    }

    #[cfg(target_os = "windows")]
    let setup_asset = release
        .assets
        .iter()
        .find(|a| a.name.to_lowercase().ends_with(".exe"))
        .or_else(|| release.assets.first());

    #[cfg(target_os = "linux")]
    let setup_asset = release
        .assets
        .iter()
        .find(|a| {
            let name = a.name.to_lowercase();
            name.ends_with(".appimage") || name.ends_with(".deb") || name.ends_with(".tar.gz")
        })
        .or_else(|| release.assets.first());

    #[cfg(target_os = "macos")]
    let setup_asset = release
        .assets
        .iter()
        .find(|a| {
            let name = a.name.to_lowercase();
            name.ends_with(".dmg") || name.ends_with(".app.tar.gz")
        })
        .or_else(|| release.assets.first());

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    let setup_asset = release.assets.first();

    let (download_url, file_size) = match setup_asset {
        Some(asset) => (asset.browser_download_url.clone(), Some(asset.size)),
        None => (
            format!(
                "https://github.com/{}/releases/latest",
                GITHUB_REPO
            ),
            None,
        ),
    };

    Ok(AppUpdateInfo {
        has_update: true,
        current_version: current_version.to_string(),
        latest_version,
        download_url,
        release_notes: release.body,
        release_date: release.published_at,
        file_size,
        sha256: None,
        force_update: false,
        source: Some("github".to_string()),
    })
}

async fn check_update_from_primary(current_version: &str) -> Result<AppUpdateInfo, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let os_name = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "unknown"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "unknown"
    };

    let url = format!(
        "{}/api/update/check?version={}&os={}&arch={}",
        UPDATE_SERVER_BASE_URL, current_version, os_name, arch
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("连接更新服务器失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("更新服务器返回错误: {}", response.status()));
    }

    let mut update_info: AppUpdateInfo = response
        .json()
        .await
        .map_err(|e| format!("解析更新信息失败: {}", e))?;

    update_info.source = Some("primary".to_string());
    Ok(update_info)
}

#[tauri::command]
pub async fn check_app_update_github() -> Result<AppUpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    check_update_from_github(&current_version).await
}

#[tauri::command]
pub async fn check_app_update_from_server() -> Result<AppUpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();

    match check_update_from_primary(&current_version).await {
        Ok(info) => Ok(info),
        Err(primary_err) => {
            eprintln!(
                "[check_update] 主更新源失败，尝试 GitHub: {}",
                primary_err
            );
            match check_update_from_github(&current_version).await {
                Ok(info) => Ok(info),
                Err(github_err) => Err(format!(
                    "主更新源失败: {}；GitHub 备用源失败: {}",
                    primary_err, github_err
                )),
            }
        }
    }
}

#[tauri::command]
pub async fn download_app_update_from_server(
    app: tauri::AppHandle,
    download_url: String,
) -> Result<AppUpdateResult, String> {
    use futures_util::StreamExt;
    use std::io::Write;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| format!("下载更新失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "下载更新失败，服务器返回: {}",
            response.status()
        ));
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    let temp_dir = std::env::temp_dir();
    let file_name = download_url
        .split('/')
        .last()
        .unwrap_or("svl-update");
    let temp_file_path = temp_dir.join(format!("svl_update_{}", file_name));
    let mut file = std::fs::File::create(&temp_file_path)
        .map_err(|e| format!("创建临时文件失败: {}", e))?;

    while let Some(chunk) = stream.next().await {
        let chunk =
            chunk.map_err(|e| format!("读取下载数据失败: {}", e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("写入临时文件失败: {}", e))?;
        downloaded += chunk.len() as u64;

        let percent = if total_size > 0 {
            (downloaded as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };

        let _ = app.emit(
            "app-update-progress",
            AppUpdateProgress {
                downloaded,
                total: total_size,
                percent,
            },
        );
    }

    drop(file);

    Ok(AppUpdateResult {
        success: true,
        message: format!("更新已下载到: {}", temp_file_path.display()),
        needs_restart: true,
        file_path: Some(temp_file_path.to_string_lossy().to_string()),
    })
}

#[tauri::command]
pub fn get_update_server_url() -> String {
    UPDATE_SERVER_BASE_URL.to_string()
}

#[tauri::command]
pub fn get_current_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn open_update_installer(installer_path: &std::path::Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new(installer_path)
            .spawn()
            .map_err(|e| format!("启动更新程序失败: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        let name = installer_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if name.ends_with(".AppImage") || name.ends_with(".appimage") {
            use std::os::unix::fs::PermissionsExt;
            let current_exe = std::env::current_exe()
                .map_err(|e| format!("获取当前程序路径失败: {}", e))?;
            let _ = std::fs::set_permissions(installer_path, std::fs::Permissions::from_mode(0o755));
            let _ = std::fs::copy(installer_path, &current_exe);
            std::process::Command::new(&current_exe)
                .spawn()
                .map_err(|e| format!("启动更新程序失败: {}", e))?;
        } else if name.ends_with(".deb") {
            std::process::Command::new("sudo")
                .args(["dpkg", "-i"])
                .arg(installer_path)
                .spawn()
                .map_err(|e| format!("启动更新程序失败: {}", e))?;
        } else {
            std::process::Command::new("xdg-open")
                .arg(installer_path)
                .spawn()
                .map_err(|e| format!("启动更新程序失败: {}", e))?;
        }
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(installer_path)
            .spawn()
            .map_err(|e| format!("启动更新程序失败: {}", e))?;
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        std::process::Command::new(installer_path)
            .spawn()
            .map_err(|e| format!("启动更新程序失败: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub fn run_installer(path: String) -> Result<(), String> {
    let exe_path = std::path::Path::new(&path);
    if !exe_path.exists() {
        return Err(format!("安装程序不存在: {}", path));
    }
    open_update_installer(exe_path)?;
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().flush();
    std::process::exit(0);
}

pub async fn auto_check_app_update(app_handle: tauri::AppHandle) {
    let current_version = env!("CARGO_PKG_VERSION").to_string();

    let update_info = match check_update_from_primary(&current_version).await {
        Ok(info) => info,
        Err(primary_err) => {
            eprintln!(
                "[auto_check_update] 主更新源失败，尝试 GitHub: {}",
                primary_err
            );
            match check_update_from_github(&current_version).await {
                Ok(info) => info,
                Err(github_err) => {
                    eprintln!(
                        "[auto_check_update] GitHub 备用源也失败: {}",
                        github_err
                    );
                    return;
                }
            }
        }
    };

    if update_info.has_update {
        eprintln!(
            "[auto_check_update] 发现新版本: {} -> {} (来源: {})",
            current_version,
            update_info.latest_version,
            update_info.source.as_deref().unwrap_or("unknown")
        );
        let _ = app_handle.emit("app-update-available", update_info);
    }
}
