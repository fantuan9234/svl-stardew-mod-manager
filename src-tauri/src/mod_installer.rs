use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tauri::Emitter;

use crate::mod_name_resolver::resolve_mod_name;
use crate::dependency_patches::apply_final_patches;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallProgressEvent {
    pub step: String,
    pub mod_name: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallResult {
    pub success: bool,
    pub mod_name: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDependencyCheck {
    pub mod_name: String,
    pub unique_id: String,
    pub version: String,
    pub missing_dependencies: Vec<MissingDepInfo>,
    pub can_install: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingDepInfo {
    pub unique_id: String,
    pub display_name: String,
    pub minimum_version: Option<String>,
    pub is_required: bool,
}

fn cleanup_temp_dir_with_retry(path: &PathBuf) {
    for _ in 0..3 {
        if !path.exists() {
            return;
        }
        if fs::remove_dir_all(path).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(200));
    }
    if path.exists() {
        log::warn!("[cleanup_temp_dir_with_retry] Failed to clean up temp dir after 3 retries: {}", path.display());
    }
}

#[tauri::command]
pub fn install_mod_from_archive(
    app: tauri::AppHandle,
    archive_path: String,
    mods_path: String,
) -> Result<InstallResult, String> {
    let archive = PathBuf::from(&archive_path);
    let mods_dir = PathBuf::from(&mods_path);

    if !archive.exists() {
        return Err(format!("压缩包不存在: {}", archive_path));
    }

    if !mods_dir.exists() {
        fs::create_dir_all(&mods_dir)
            .map_err(|e| format!("无法创建 Mods 文件夹: {}", e))?;
    }

    let extension = archive
        .extension()
        .ok_or("无法获取文件扩展名")?
        .to_string_lossy()
        .to_lowercase();

    let temp_dir = mods_dir.join(".temp_extract");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)
            .map_err(|e| format!("无法清理临时文件夹: {}", e))?;
    }
    fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("无法创建临时文件夹: {}", e))?;

    let _ = app.emit(
        "mod-install-progress",
        InstallProgressEvent {
            step: "extracting".to_string(),
            mod_name: None,
            message: "正在解压...".to_string(),
        },
    );

    match extension.as_str() {
        "zip" => extract_zip(&archive, &temp_dir)?,
        "7z" => extract_7z(&archive, &temp_dir)?,
        "rar" => return Err("RAR 格式暂不支持，请使用 ZIP 或 7Z 格式".to_string()),
        _ => return Err(format!("不支持的格式: .{}", extension)),
    }

    let mod_folder = find_mod_folder(&temp_dir)?;
    let mod_name = mod_folder
        .file_name()
        .ok_or("无法获取 MOD 文件夹名称")?
        .to_string_lossy()
        .to_string();

    let _ = app.emit(
        "mod-install-progress",
        InstallProgressEvent {
            step: "installing".to_string(),
            mod_name: Some(mod_name.clone()),
            message: format!("正在安装 '{}'...", mod_name),
        },
    );

    let dest_path = mods_dir.join(&mod_name);

    if dest_path.exists() {
        fs::remove_dir_all(&dest_path)
            .map_err(|e| format!("无法删除已存在的 MOD: {}", e))?;
    }

    fs_extra::dir::copy(&mod_folder, &mods_dir, &fs_extra::dir::CopyOptions::new())
        .map_err(|e| format!("复制 MOD 失败: {}", e))?;

    cleanup_temp_dir_with_retry(&temp_dir);

    // Force filesystem flush on Windows
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "dir", mods_dir.to_str().unwrap_or("")])
            .output();
    }

    let _ = app.emit(
        "mod-install-progress",
        InstallProgressEvent {
            step: "done".to_string(),
            mod_name: Some(mod_name.clone()),
            message: format!("MOD '{}' 安装成功", mod_name),
        },
    );

    println!("[install_mod_from_archive] done, mod_name={}, mods_path={}", mod_name, mods_path);

    Ok(InstallResult {
        success: true,
        mod_name: Some(mod_name.clone()),
        message: format!("MOD '{}' 安装成功", mod_name),
    })
}

#[tauri::command]
pub fn install_mod_from_folder(
    app: tauri::AppHandle,
    source_path: String,
    mods_path: String,
) -> Result<InstallResult, String> {
    let source = PathBuf::from(&source_path);
    let mods_dir = PathBuf::from(&mods_path);

    if !source.exists() {
        return Err(format!("源文件夹不存在: {}", source_path));
    }

    if !mods_dir.exists() {
        fs::create_dir_all(&mods_dir)
            .map_err(|e| format!("无法创建 Mods 文件夹: {}", e))?;
    }

    let folder_name = source
        .file_name()
        .ok_or("无法获取文件夹名称")?
        .to_string_lossy()
        .to_string();

    let _ = app.emit(
        "mod-install-progress",
        InstallProgressEvent {
            step: "installing".to_string(),
            mod_name: Some(folder_name.clone()),
            message: format!("正在安装 '{}'...", folder_name),
        },
    );

    let dest_path = mods_dir.join(&folder_name);

    if dest_path.exists() {
        fs::remove_dir_all(&dest_path)
            .map_err(|e| format!("无法删除已存在的 MOD: {}", e))?;
    }

    fs_extra::dir::copy(&source, &mods_dir, &fs_extra::dir::CopyOptions::new())
        .map_err(|e| format!("复制 MOD 失败: {}", e))?;

    // Force filesystem flush on Windows
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "dir", mods_dir.to_str().unwrap_or("")])
            .output();
    }

    let _ = app.emit(
        "mod-install-progress",
        InstallProgressEvent {
            step: "done".to_string(),
            mod_name: Some(folder_name.clone()),
            message: format!("MOD '{}' 安装成功", folder_name),
        },
    );

    println!("[install_mod_from_folder] done, mod_name={}, mods_path={}", folder_name, mods_path);

    Ok(InstallResult {
        success: true,
        mod_name: Some(folder_name.clone()),
        message: format!("MOD '{}' 安装成功", folder_name),
    })
}

#[tauri::command]
pub fn uninstall_mod(mod_path: String) -> Result<InstallResult, String> {
    let path = PathBuf::from(&mod_path);

    if !path.exists() {
        return Err(format!("MOD 文件夹不存在: {}", mod_path));
    }

    let mod_name = path
        .file_name()
        .ok_or("无法获取 MOD 文件夹名称")?
        .to_string_lossy()
        .to_string();

    // First clean residual files in parent directory
    clean_residual_files(&path);

    // Then remove the mod folder itself
    fs::remove_dir_all(&path).map_err(|e| format!("删除 MOD 失败: {}", e))?;

    // Verify the folder is actually deleted, retry if needed (Windows file locking)
    for attempt in 0..3 {
        if !path.exists() {
            break;
        }
        thread::sleep(Duration::from_millis(200));
        let _ = fs::remove_dir_all(&path);
    }

    if path.exists() {
        return Err(format!("MOD 文件夹删除后仍然存在: {}", mod_path));
    }

    Ok(InstallResult {
        success: true,
        mod_name: Some(mod_name.clone()),
        message: format!("MOD '{}' 已卸载并清理完成", mod_name),
    })
}

fn extract_zip(archive: &PathBuf, dest: &PathBuf) -> Result<(), String> {
    let file = fs::File::open(archive).map_err(|e| format!("打开文件失败: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("读取 ZIP 失败: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("读取文件索引失败: {}", e))?;
        let outpath = dest.join(file.name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(|e| format!("创建文件夹失败: {}", e))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).map_err(|e| format!("创建文件夹失败: {}", e))?;
                }
            }
            let mut outfile =
                fs::File::create(&outpath).map_err(|e| format!("创建文件失败: {}", e))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("解压文件失败: {}", e))?;
        }
    }

    Ok(())
}

fn extract_7z(archive: &PathBuf, dest: &PathBuf) -> Result<(), String> {
    sevenz_rust::decompress_file(archive, dest)
        .map_err(|e| format!("解压 7Z 失败: {}", e))?;
    Ok(())
}

fn find_mod_folder(temp_dir: &PathBuf) -> Result<PathBuf, String> {
    let entries: Vec<_> = fs::read_dir(temp_dir)
        .map_err(|e| format!("读取临时文件夹失败: {}", e))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();

    if entries.len() == 1 {
        Ok(entries[0].path())
    } else if entries.is_empty() {
        let has_manifest = temp_dir.join("manifest.json").exists();
        if has_manifest {
            Ok(temp_dir.clone())
        } else {
            Err("未在压缩包中找到 MOD 文件夹或 manifest.json".to_string())
        }
    } else {
        for entry in &entries {
            if entry.path().join("manifest.json").exists() {
                return Ok(entry.path());
            }
        }
        Ok(entries[0].path())
    }
}

fn clean_residual_files(mod_path: &PathBuf) {
    if let Some(parent) = mod_path.parent() {
        if let Ok(entries) = fs::read_dir(parent) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with('.') || name.ends_with(".tmp") || name.ends_with(".bak") {
                            let _ = fs::remove_file(&path);
                        }
                    }
                }
            }
        }
    }
}

#[tauri::command]
pub fn install_mod(
    app: tauri::AppHandle,
    archive_path: String,
    mods_path: String,
) -> Result<InstallResult, String> {
    let archive = PathBuf::from(&archive_path);
    let mods_dir = PathBuf::from(&mods_path);

    if !archive.exists() {
        return Err(format!("压缩包不存在: {}", archive_path));
    }

    if !mods_dir.exists() {
        fs::create_dir_all(&mods_dir)
            .map_err(|e| format!("无法创建 Mods 文件夹: {}", e))?;
    }

    let extension = archive
        .extension()
        .ok_or("无法获取文件扩展名")?
        .to_string_lossy()
        .to_lowercase();

    let temp_dir = mods_dir.join(".temp_extract");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)
            .map_err(|e| format!("无法清理临时文件夹: {}", e))?;
    }
    fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("无法创建临时文件夹: {}", e))?;

    let _ = app.emit(
        "mod-install-progress",
        InstallProgressEvent {
            step: "extracting".to_string(),
            mod_name: None,
            message: "正在解压...".to_string(),
        },
    );

    match extension.as_str() {
        "zip" => extract_zip(&archive, &temp_dir)?,
        "7z" => extract_7z(&archive, &temp_dir)?,
        "rar" => return Err("RAR 格式暂不支持，请使用 ZIP 或 7Z 格式".to_string()),
        _ => return Err(format!("不支持的格式: .{}", extension)),
    }

    let mod_folder = find_mod_folder(&temp_dir)?;
    let mod_name = mod_folder
        .file_name()
        .ok_or("无法获取 MOD 文件夹名称")?
        .to_string_lossy()
        .to_string();

    let _ = app.emit(
        "mod-install-progress",
        InstallProgressEvent {
            step: "installing".to_string(),
            mod_name: Some(mod_name.clone()),
            message: format!("正在安装 '{}'...", mod_name),
        },
    );

    let dest_path = mods_dir.join(&mod_name);

    if dest_path.exists() {
        fs::remove_dir_all(&dest_path)
            .map_err(|e| format!("无法删除已存在的 MOD: {}", e))?;
    }

    fs_extra::dir::copy(&mod_folder, &mods_dir, &fs_extra::dir::CopyOptions::new())
        .map_err(|e| format!("复制 MOD 失败: {}", e))?;

    cleanup_temp_dir_with_retry(&temp_dir);

    // Force filesystem flush on Windows
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "dir", mods_dir.to_str().unwrap_or("")])
            .output();
    }

    let _ = app.emit(
        "mod-install-progress",
        InstallProgressEvent {
            step: "done".to_string(),
            mod_name: Some(mod_name.clone()),
            message: format!("MOD '{}' 安装成功", mod_name),
        },
    );

    println!("[install_mod] done, mod_name={}, mods_path={}", mod_name, mods_path);

    Ok(InstallResult {
        success: true,
        mod_name: Some(mod_name.clone()),
        message: format!("MOD '{}' 安装成功", mod_name),
    })
}

#[tauri::command]
pub fn check_mod_dependencies(
    archive_path: String,
    mods_path: String,
) -> Result<ModDependencyCheck, String> {
    let archive = PathBuf::from(&archive_path);
    let mods_dir = PathBuf::from(&mods_path);

    if !archive.exists() {
        return Err(format!("压缩包不存在: {}", archive_path));
    }

    let extension = archive
        .extension()
        .ok_or("无法获取文件扩展名")?
        .to_string_lossy()
        .to_lowercase();

    let temp_dir = mods_dir.join(".temp_dep_check");
    if temp_dir.exists() {
        cleanup_temp_dir_with_retry(&temp_dir);
    }
    fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("无法创建临时文件夹: {}", e))?;

    match extension.as_str() {
        "zip" => extract_zip(&archive, &temp_dir)?,
        "7z" => extract_7z(&archive, &temp_dir)?,
        _ => return Err(format!("不支持的格式: .{}", extension)),
    }

    let mod_folder = find_mod_folder(&temp_dir)?;
    let manifest_path = mod_folder.join("manifest.json");

    if !manifest_path.exists() {
        cleanup_temp_dir_with_retry(&temp_dir);
        return Err("未找到 manifest.json，无法检查依赖".to_string());
    }

    let content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("读取 manifest.json 失败: {}", e))?;

    let manifest: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("解析 manifest.json 失败: {}", e))?;

    let mod_name = manifest["Name"].as_str().unwrap_or("未知 MOD").to_string();
    let unique_id = manifest["UniqueID"].as_str().unwrap_or("").to_string();
    let version = manifest["Version"].as_str().unwrap_or("1.0.0").to_string();

    let mut missing_deps = Vec::new();

    if let Some(deps) = manifest["Dependencies"].as_array() {
        for dep in deps {
            let dep_id = dep["UniqueID"].as_str().unwrap_or("").to_string();
            let is_required = dep["IsRequired"].as_bool().unwrap_or(true);

            let dep_folder = mods_dir.join(&dep_id);
            if !dep_folder.exists() {
                let mut found = false;
                if let Ok(entries) = fs::read_dir(&mods_dir) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_dir() {
                            let mf = p.join("manifest.json");
                            if mf.exists() {
                                if let Ok(mc) = fs::read_to_string(&mf) {
                                    if let Ok(mv) = serde_json::from_str::<serde_json::Value>(&mc) {
                                        if mv["UniqueID"].as_str() == Some(&dep_id) {
                                            found = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if !found {
                    let display_name = resolve_mod_name(&dep_id);
                    missing_deps.push(MissingDepInfo {
                        unique_id: dep_id,
                        display_name,
                        minimum_version: dep["MinimumVersion"].as_str().map(|s| s.to_string()),
                        is_required,
                    });
                }
            }
        }
    }

    cleanup_temp_dir_with_retry(&temp_dir);

    let mut installed_mod_ids = HashSet::new();
    if let Ok(entries) = fs::read_dir(&mods_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                let mf = p.join("manifest.json");
                if mf.exists() {
                    if let Ok(mc) = fs::read_to_string(&mf) {
                        if let Ok(mv) = serde_json::from_str::<serde_json::Value>(&mc) {
                            if let Some(uid) = mv["UniqueID"].as_str() {
                                installed_mod_ids.insert(uid.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    apply_final_patches(&installed_mod_ids, &mut missing_deps);

    let has_required_missing = missing_deps.iter().any(|d| d.is_required);

    Ok(ModDependencyCheck {
        mod_name,
        unique_id,
        version,
        missing_dependencies: missing_deps,
        can_install: !has_required_missing,
    })
}
