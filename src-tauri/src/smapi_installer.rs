use serde::{Deserialize, Serialize};
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
