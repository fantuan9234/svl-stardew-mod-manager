use serde::{Deserialize, Serialize};
use tauri::Emitter;

const UPDATE_SERVER_BASE_URL: &str = "http://211.101.247.248:8866";

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

#[tauri::command]
pub async fn check_app_update_from_server() -> Result<AppUpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let url = format!("{}/api/update/check?version={}", UPDATE_SERVER_BASE_URL, current_version);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("连接更新服务器失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("更新服务器返回错误: {}", response.status()));
    }

    let update_info: AppUpdateInfo = response
        .json()
        .await
        .map_err(|e| format!("解析更新信息失败: {}", e))?;

    Ok(update_info)
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
        return Err(format!("下载更新失败，服务器返回: {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    let temp_dir = std::env::temp_dir();
    let file_name = download_url.split('/').last().unwrap_or("svl-update.exe");
    let temp_file_path = temp_dir.join(format!("svl_update_{}", file_name));
    let mut file = std::fs::File::create(&temp_file_path)
        .map_err(|e| format!("创建临时文件失败: {}", e))?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("读取下载数据失败: {}", e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("写入临时文件失败: {}", e))?;
        downloaded += chunk.len() as u64;

        let percent = if total_size > 0 {
            (downloaded as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };

        let _ = app.emit("app-update-progress", AppUpdateProgress {
            downloaded,
            total: total_size,
            percent,
        });
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

fn open_update_installer(exe_path: &std::path::Path) -> Result<(), String> {
    std::process::Command::new(exe_path)
        .spawn()
        .map_err(|e| format!("启动更新程序失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn run_installer(path: String) -> Result<(), String> {
    let exe_path = std::path::Path::new(&path);
    if !exe_path.exists() {
        return Err(format!("安装程序不存在: {}", path));
    }
    open_update_installer(exe_path)?;
    std::process::exit(0);
}

pub async fn auto_check_app_update(app_handle: tauri::AppHandle) {
    let current_version = env!("CARGO_PKG_VERSION").to_string();

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[auto_check_update] 创建 HTTP 客户端失败: {}", e);
            return;
        }
    };

    let url = format!("{}/api/update/check?version={}", UPDATE_SERVER_BASE_URL, current_version);

    let response = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[auto_check_update] 连接更新服务器失败: {}", e);
            return;
        }
    };

    if !response.status().is_success() {
        eprintln!("[auto_check_update] 更新服务器返回错误: {}", response.status());
        return;
    }

    let update_info: AppUpdateInfo = match response.json().await {
        Ok(info) => info,
        Err(e) => {
            eprintln!("[auto_check_update] 解析更新信息失败: {}", e);
            return;
        }
    };

    if update_info.has_update {
        eprintln!(
            "[auto_check_update] 发现新版本: {} -> {}",
            current_version, update_info.latest_version
        );
        let _ = app_handle.emit("app-update-available", update_info);
    }
}
