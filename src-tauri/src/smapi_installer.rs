use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use tauri::Emitter;
use tauri_plugin_dialog::{DialogExt, FilePath};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstallProgress {
    pub step: String,
    pub message: String,
    pub percent: Option<f64>,
}

fn emit_progress(app: &tauri::AppHandle, step: &str, message: &str, percent: Option<f64>) {
    let _ = app.emit(
        "smapi-install-progress",
        InstallProgress {
            step: step.to_string(),
            message: message.to_string(),
            percent,
        },
    );
}

#[tauri::command]
pub async fn install_smapi_local(
    app: tauri::AppHandle,
    zip_path: String,
    game_path: String,
) -> Result<InstallResult, String> {
    println!("=== SMAPI 安装后端调试信息 ===");
    println!("接收到的 zip_path: {}", zip_path);
    println!("接收到的 game_path: {}", game_path);

    let zip_path_buf = PathBuf::from(&zip_path);
    let game_path_buf = PathBuf::from(&game_path);

    if let Some(ext) = zip_path_buf.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        if ext_str != "zip" {
            println!("错误: 文件扩展名不是 .zip，而是 .{}", ext_str);
            return Err("文件类型错误，请提供 .zip 格式的安装包".to_string());
        }
        println!("文件扩展名验证通过: .{}", ext_str);
    } else {
        println!("错误: 文件没有扩展名");
        return Err("文件类型错误，请提供 .zip 格式的安装包".to_string());
    }

    if !zip_path_buf.exists() {
        println!("错误: ZIP 文件不存在: {}", zip_path);
        return Err(format!("ZIP 文件不存在: {}", zip_path));
    }
    println!("ZIP 文件存在: {}", zip_path);

    if !game_path_buf.exists() {
        println!("错误: 游戏路径不存在: {}", game_path);
        return Err(format!("游戏路径不存在: {}", game_path));
    }
    println!("游戏路径存在: {}", game_path);

    emit_progress(&app, "extracting", "正在解压 SMAPI...", Some(0.0));

    let zip_data = std::fs::read(&zip_path_buf)
        .map_err(|e| {
            println!("错误: 读取 ZIP 文件失败: {}", e);
            format!("读取压缩包失败: {}", e)
        })?;
    println!("ZIP 文件读取成功，大小: {} 字节", zip_data.len());

    let cursor = std::io::Cursor::new(zip_data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| {
            println!("错误: ZIP 文件解析失败: {}", e);
            format!("ZIP 文件解析失败: {}", e)
        })?;

    let total_files = archive.len();
    println!("压缩包内共有 {} 个文件", total_files);

    for i in 0..total_files {
        let mut file = archive.by_index(i)
            .map_err(|e| {
                println!("错误: 文件 #{} 解压失败: {}", i, e);
                format!("文件 #{} 解压失败: {}", i, e)
            })?;
        
        let outpath = game_path_buf.join(file.name());

        if file.name().ends_with('/') || file.name().ends_with('\\') {
            std::fs::create_dir_all(&outpath)
                .map_err(|e| format!("创建目录失败: {}", e))?;
        } else {
            if let Some(p) = outpath.parent() {
                std::fs::create_dir_all(p)
                    .map_err(|e| format!("创建目录失败: {}", e))?;
            }

            let mut outfile = std::fs::File::create(&outpath)
                .map_err(|e| format!("创建文件失败: {}", e))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("写入文件失败: {}", e))?;
        }

        if (i + 1) % 10 == 0 || i == total_files - 1 {
            let percent = ((i + 1) as f64 / total_files as f64) * 100.0;
            println!("已解压 {}/{} 个文件 ({:.1}%)", i + 1, total_files, percent);
        }
    }

    println!("所有文件已解压到: {}", game_path);

    emit_progress(&app, "extracted", "SMAPI 文件已成功解压至游戏目录", Some(100.0));

    Ok(InstallResult {
        success: true,
        message: format!("SMAPI 文件已成功解压至: {}", game_path),
    })
}

#[tauri::command]
pub fn open_smapi_zip_dialog(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let result = app.dialog().file()
        .set_title("选择 SMAPI 安装包")
        .add_filter("ZIP 文件", &["zip"])
        .blocking_pick_file();

    if let Some(path) = result {
        let path_str = match path {
            FilePath::Path(p) => p.to_string_lossy().to_string(),
            FilePath::Url(u) => u.to_string(),
        };
        Ok(Some(path_str))
    } else {
        Ok(None)
    }
}

fn extract_smapi_zip(zip_path: &str, game_path: &str) -> Result<InstallResult, String> {
    let zip_path_buf = PathBuf::from(zip_path);
    let game_path_buf = PathBuf::from(game_path);

    if !zip_path_buf.exists() {
        return Err(format!("ZIP 文件不存在: {}", zip_path));
    }
    if !game_path_buf.exists() {
        return Err(format!("游戏路径不存在: {}", game_path));
    }

    let zip_data = std::fs::read(&zip_path_buf)
        .map_err(|e| format!("读取压缩包失败: {}", e))?;

    let cursor = std::io::Cursor::new(zip_data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("ZIP 文件解析失败: {}", e))?;

    let total_files = archive.len();

    for i in 0..total_files {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("文件 #{} 解压失败: {}", i, e))?;

        let outpath = game_path_buf.join(file.name());

        if file.name().ends_with('/') || file.name().ends_with('\\') {
            std::fs::create_dir_all(&outpath)
                .map_err(|e| format!("创建目录失败: {}", e))?;
        } else {
            if let Some(p) = outpath.parent() {
                std::fs::create_dir_all(p)
                    .map_err(|e| format!("创建目录失败: {}", e))?;
            }

            let mut outfile = std::fs::File::create(&outpath)
                .map_err(|e| format!("创建文件失败: {}", e))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("写入文件失败: {}", e))?;
        }
    }

    Ok(InstallResult {
        success: true,
        message: format!("SMAPI 文件已成功解压至: {}", game_path),
    })
}

#[tauri::command]
pub async fn auto_install_smapi(
    app: tauri::AppHandle,
    game_path: String,
) -> Result<InstallResult, String> {
    use futures_util::StreamExt;

    let game_path_buf = PathBuf::from(&game_path);
    if !game_path_buf.exists() {
        return Err(format!("游戏路径不存在: {}", game_path));
    }

    emit_progress(&app, "fetching", "正在获取 SMAPI 最新版本信息...", Some(0.0));

    let client = reqwest::Client::builder()
        .user_agent("SVL-StardewValley-ModManager")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let release_info: serde_json::Value = client
        .get("https://api.github.com/repos/Pathoschild/SMAPI/releases/latest")
        .send()
        .await
        .map_err(|e| format!("获取 SMAPI 版本信息失败: {}", e))?
        .json()
        .await
        .map_err(|e| format!("解析版本信息失败: {}", e))?;

    let tag_name = release_info["tag_name"]
        .as_str()
        .ok_or_else(|| "无法获取 SMAPI 版本".to_string())?
        .to_string();
    let version = tag_name.trim_start_matches('v').to_string();

    let assets = release_info["assets"]
        .as_array()
        .ok_or_else(|| "无法找到 SMAPI 安装包".to_string())?;

    // 注意：必须选择 SMAPI-X.X.X-installer.zip，而不是 -installer-double-zipped.zip
    // 前者包含 platform-specific installer 二进制，后者需要先解压再解压
    let download_url = assets
        .iter()
        .find_map(|asset| {
            let name = asset["name"].as_str()?;
            if name.ends_with("-installer.zip") {
                asset["browser_download_url"].as_str().map(String::from)
            } else {
                None
            }
        })
        .ok_or_else(|| "未找到 SMAPI installer.zip 安装包".to_string())?;

    emit_progress(
        &app,
        "downloading",
        &format!("正在下载 SMAPI {}...", version),
        Some(5.0),
    );

    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| format!("下载 SMAPI 失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("下载失败，HTTP 状态: {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    let temp_dir = std::env::temp_dir();
    let zip_path = temp_dir.join(format!("SVL-SMAPI-{}-installer.zip", version));
    let _ = std::fs::remove_file(&zip_path);

    let mut file = std::fs::File::create(&zip_path)
        .map_err(|e| format!("创建临时文件失败: {}", e))?;

    let mut last_emit_percent: f64 = 0.0;
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("下载中断: {}", e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("写入文件失败: {}", e))?;
        downloaded += chunk.len() as u64;
        if total_size > 0 {
            let percent = 5.0 + (downloaded as f64 / total_size as f64) * 55.0;
            if percent - last_emit_percent >= 1.5 {
                last_emit_percent = percent;
                emit_progress(
                    &app,
                    "downloading",
                    &format!(
                        "已下载 {:.1} MB / {:.1} MB",
                        downloaded as f64 / 1024.0 / 1024.0,
                        total_size as f64 / 1024.0 / 1024.0
                    ),
                    Some(percent),
                );
            }
        }
    }
    drop(file);

    // 解压到临时目录
    emit_progress(&app, "extracting", "正在解压安装包...", Some(62.0));

    let extract_dir = temp_dir.join(format!("SVL-SMAPI-{}", version));
    let _ = std::fs::remove_dir_all(&extract_dir);
    std::fs::create_dir_all(&extract_dir)
        .map_err(|e| format!("创建临时目录失败: {}", e))?;

    let zip_data = std::fs::read(&zip_path)
        .map_err(|e| format!("读取 ZIP 失败: {}", e))?;
    let cursor = std::io::Cursor::new(zip_data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("ZIP 解析失败: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("ZIP 文件 #{} 失败: {}", i, e))?;
        let outpath = match file.enclosed_name() {
            Some(p) => extract_dir.join(p),
            None => continue,
        };
        if file.is_dir() {
            std::fs::create_dir_all(&outpath)
                .map_err(|e| format!("创建目录失败: {}", e))?;
        } else {
            if let Some(p) = outpath.parent() {
                std::fs::create_dir_all(p)
                    .map_err(|e| format!("创建父目录失败: {}", e))?;
            }
            let mut outfile = std::fs::File::create(&outpath)
                .map_err(|e| format!("创建文件失败: {}", e))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("写入文件失败: {}", e))?;
        }
    }
    drop(archive);
    let _ = std::fs::remove_file(&zip_path);

    // SMAPI 4.5+ 的 zip 顶层有 "SMAPI X.Y.Z installer" 包装目录
    // 需要先找到包含 internal/ 的真实安装根
    fn find_installer_root(start: &std::path::Path) -> Option<std::path::PathBuf> {
        let internal_path = start.join("internal");
        if internal_path.exists() {
            return Some(start.to_path_buf());
        }
        if let Ok(entries) = std::fs::read_dir(start) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    if let Some(found) = find_installer_root(&p) {
                        return Some(found);
                    }
                }
            }
        }
        None
    }

    let installer_root = find_installer_root(&extract_dir).ok_or_else(|| {
        let _ = std::fs::remove_dir_all(&extract_dir);
        format!(
            "无法在解压目录中找到 SMAPI 安装根目录: {}",
            extract_dir.display()
        )
    })?;

    // 定位 platform-specific 安装器
    emit_progress(&app, "preparing", "正在准备 SMAPI 安装器...", Some(68.0));

    #[cfg(target_os = "windows")]
    let (installer_subdir, installer_name) = ("windows", "SMAPI.Installer.exe");
    #[cfg(target_os = "macos")]
    let (installer_subdir, installer_name) = ("macOS", "SMAPI.Installer");
    #[cfg(target_os = "linux")]
    let (installer_subdir, installer_name) = ("linux", "SMAPI.Installer");

    let installer_path = installer_root
        .join("internal")
        .join(installer_subdir)
        .join(installer_name);

    if !installer_path.exists() {
        let _ = std::fs::remove_dir_all(&extract_dir);
        return Err(format!(
            "SMAPI Installer 未找到: {}（可能平台不匹配）",
            installer_path.display()
        ));
    }

    // 在 Linux/macOS 上需要确保 Installer 可执行（zip 可能没保留 +x）
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&installer_path)
            .map_err(|e| format!("读取 Installer 权限失败: {}", e))?;
        let mut perms = metadata.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&installer_path, perms)
            .map_err(|e| format!("设置 Installer 可执行权限失败: {}", e))?;
    }

    // 调用 SMAPI.Installer --install --game-path <path> --no-prompt
    // 关键：SVL 是 GUI 进程没有控制台，子进程继承不到 console。
    // SMAPI.Installer 内部大量使用 Console.Clear/ReadKey 会抛 IOException。
    // 必须在 Windows 上给子进程分配新控制台（CREATE_NEW_CONSOLE = 0x10），
    // 且不能重定向 stdio（避免 Stdio::piped() 把标准句柄变成 pipe），
    // 使用 spawn() + wait() 而非 output()，保留标准句柄指向新 console。
    emit_progress(&app, "installing", "正在执行 SMAPI 安装器...", Some(72.0));

    let mut cmd = std::process::Command::new(&installer_path);
    cmd.arg("--install")
        .arg("--game-path")
        .arg(&game_path)
        .arg("--no-prompt");

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NEW_CONSOLE: 为子进程分配新的 console 句柄，
        // 让 .NET 内部的 Console.Clear/ReadKey 能找到真实的 console buffer
        const CREATE_NEW_CONSOLE: u32 = 0x00000010;
        cmd.creation_flags(CREATE_NEW_CONSOLE);
    }

    // spawn() + wait() 不会重定向 stdio，标准句柄保留为新 console 句柄
    let mut child = cmd.spawn().map_err(|e| {
        let _ = std::fs::remove_dir_all(&extract_dir);
        format!("启动 SMAPI Installer 失败: {}", e)
    })?;

    let status = child.wait().map_err(|e| {
        let _ = std::fs::remove_dir_all(&extract_dir);
        format!("等待 SMAPI Installer 失败: {}", e)
    })?;

    let _ = std::fs::remove_dir_all(&extract_dir);

    if !status.success() {
        // 安装失败时保留临时目录，方便用户查看下载/解压结果
        let preserved = extract_dir.clone();
        return Err(format!(
            "SMAPI 安装失败（退出码 {:?}）。已下载/解压的安装包保留在: {}",
            status.code(),
            preserved.display()
        ));
    }

    emit_progress(&app, "done", "SMAPI 安装完成", Some(100.0));

    Ok(InstallResult {
        success: true,
        message: format!("SMAPI {} 已成功安装至: {}", version, game_path),
    })
}
