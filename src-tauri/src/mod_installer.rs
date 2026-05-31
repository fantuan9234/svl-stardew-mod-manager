use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use tauri::Emitter;

use crate::mod_name_resolver::resolve_mod_name;
use crate::dependency_patches::apply_final_patches;

fn is_safe_to_delete(target: &Path, mods_dir: &Path) -> bool {
    let target_canon = match target.canonicalize() {
        Ok(c) => c,
        Err(_) => return false,
    };
    let mods_canon = match mods_dir.canonicalize() {
        Ok(c) => c,
        Err(_) => return false,
    };

    if target_canon == mods_canon {
        eprintln!("[SAFETY] BLOCKED: attempted to delete mods directory itself: {}", target.display());
        return false;
    }

    if !target_canon.starts_with(&mods_canon) {
        eprintln!("[SAFETY] BLOCKED: target is outside mods directory: {}", target.display());
        return false;
    }

    true
}

pub fn find_existing_mod_folder(mods_dir: &PathBuf, unique_id: &str) -> Option<PathBuf> {
    if let Ok(entries) = fs::read_dir(mods_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let manifest_path = path.join("manifest.json");
            if let Ok(content) = fs::read_to_string(&manifest_path) {
                let normalized = crate::mod_parser::normalize_smart_quotes(&content);
                let cleaned = crate::mod_parser::remove_trailing_commas(&normalized);
                let cleaned = cleaned.strip_prefix('\u{FEFF}').unwrap_or(&cleaned);
                if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(cleaned) {
                    if manifest["UniqueID"].as_str() == Some(unique_id) {
                        return Some(path);
                    }
                }
            }
            let dot_manifest = path.join(".manifest.json");
            if let Ok(content) = fs::read_to_string(&dot_manifest) {
                let normalized = crate::mod_parser::normalize_smart_quotes(&content);
                let cleaned = crate::mod_parser::remove_trailing_commas(&normalized);
                let cleaned = cleaned.strip_prefix('\u{FEFF}').unwrap_or(&cleaned);
                if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(cleaned) {
                    if manifest["UniqueID"].as_str() == Some(unique_id) {
                        return Some(path);
                    }
                }
            }
        }
    }
    None
}

fn remove_old_mod_versions(mods_dir: &PathBuf, unique_id: &str) -> Vec<String> {
    let mut removed = Vec::new();
    if let Ok(entries) = fs::read_dir(mods_dir) {
        let all_entries: Vec<_> = entries.flatten().collect();
        for entry in &all_entries {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let folder_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            if folder_name.starts_with('.') && folder_name.len() <= 1 {
                continue;
            }
            let mut matched = false;
            let manifest_path = path.join("manifest.json");
            if let Ok(content) = fs::read_to_string(&manifest_path) {
                let normalized = crate::mod_parser::normalize_smart_quotes(&content);
                let cleaned = crate::mod_parser::remove_trailing_commas(&normalized);
                let cleaned = cleaned.strip_prefix('\u{FEFF}').unwrap_or(&cleaned);
                if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(cleaned) {
                    if manifest["UniqueID"].as_str() == Some(unique_id) {
                        matched = true;
                    }
                }
            }
            if !matched {
                let folder_name_stripped = folder_name.strip_prefix('.').unwrap_or(&folder_name);
                let dot_manifest = path.join(format!("{}.manifest.json", folder_name_stripped));
                if let Ok(content) = fs::read_to_string(&dot_manifest) {
                    let normalized = crate::mod_parser::normalize_smart_quotes(&content);
                    let cleaned = crate::mod_parser::remove_trailing_commas(&normalized);
                    let cleaned = cleaned.strip_prefix('\u{FEFF}').unwrap_or(&cleaned);
                    if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(cleaned) {
                        if manifest["UniqueID"].as_str() == Some(unique_id) {
                            matched = true;
                        }
                    }
                }
            }
            if matched {
                if !is_safe_to_delete(&path, mods_dir) {
                    eprintln!("[remove_old_mod_versions] SAFETY BLOCKED: {}", path.display());
                    continue;
                }
                eprintln!("[remove_old_mod_versions] Removing old version: {}", path.display());
                if fs::remove_dir_all(&path).is_ok() {
                    removed.push(folder_name);
                } else {
                    eprintln!("[remove_old_mod_versions] Failed to remove: {}", path.display());
                }
            }
        }
    }
    removed
}

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
pub async fn install_mod_from_archive(
    app: tauri::AppHandle,
    archive_path: String,
    mods_path: String,
    old_unique_id: Option<String>,
) -> Result<InstallResult, String> {
    let archive_path = archive_path.clone();
    let mods_path = mods_path.clone();
    tokio::task::spawn_blocking(move || {
        install_mod_from_archive_blocking(app, archive_path, mods_path, old_unique_id)
    })
    .await
    .map_err(|e| format!("安装任务执行失败: {}", e))?
}

pub(crate) fn install_mod_from_archive_blocking(
    app: tauri::AppHandle,
    archive_path: String,
    mods_path: String,
    old_unique_id: Option<String>,
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
        if !is_safe_to_delete(&dest_path, &mods_dir) {
            return Err(format!("安全拦截: 不允许删除路径 {}", dest_path.display()));
        }
        fs::remove_dir_all(&dest_path)
            .map_err(|e| format!("无法删除已存在的 MOD: {}", e))?;
    }

    if let Some(ref uid) = old_unique_id {
        if !uid.is_empty() {
            let removed = remove_old_mod_versions(&mods_dir, uid);
            if !removed.is_empty() {
                eprintln!("[install_mod_from_archive] Removed old versions of {}: {:?}", uid, removed);
            }
        }
    }

    fs_extra::dir::copy(&mod_folder, &mods_dir, &fs_extra::dir::CopyOptions::new())
        .map_err(|e| format!("复制 MOD 失败: {}", e))?;

    cleanup_temp_dir_with_retry(&temp_dir);

    // Force filesystem flush on Windows
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let _ = std::process::Command::new("cmd")
            .args(["/C", "dir", mods_dir.to_str().unwrap_or("")])
            .creation_flags(0x08000000)
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

    let installed_manifest = dest_path.join("manifest.json");
    if installed_manifest.exists() {
        if let Ok(content) = fs::read_to_string(&installed_manifest) {
            let normalized = crate::mod_parser::normalize_smart_quotes(&content);
            let cleaned = crate::mod_parser::remove_trailing_commas(&normalized);
            let cleaned = cleaned.strip_prefix('\u{FEFF}').unwrap_or(&cleaned);
            if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(cleaned) {
                let installed_version = manifest["Version"].as_str().unwrap_or("unknown");
                let installed_uid = manifest["UniqueID"].as_str().unwrap_or("unknown");
                crate::app_logger::log_info("ModInstaller", &format!(
                    "Installed '{}' (UniqueID: {}) version: {} at {}",
                    mod_name, installed_uid, installed_version, dest_path.display()
                ));
            }
        }
    }

    println!("[install_mod_from_archive] done, mod_name={}, mods_path={}", mod_name, mods_path);

    Ok(InstallResult {
        success: true,
        mod_name: Some(mod_name.clone()),
        message: format!("MOD '{}' 安装成功", mod_name),
    })
}

#[tauri::command]
pub async fn install_mod_from_folder(
    app: tauri::AppHandle,
    source_path: String,
    mods_path: String,
) -> Result<InstallResult, String> {
    tokio::task::spawn_blocking(move || {
        install_mod_from_folder_blocking(app, source_path, mods_path)
    })
    .await
    .map_err(|e| format!("安装任务执行失败: {}", e))?
}

fn install_mod_from_folder_blocking(
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
        if !is_safe_to_delete(&dest_path, &mods_dir) {
            return Err(format!("安全拦截: 不允许删除路径 {}", dest_path.display()));
        }
        fs::remove_dir_all(&dest_path)
            .map_err(|e| format!("无法删除已存在的 MOD: {}", e))?;
    }

    fs_extra::dir::copy(&source, &mods_dir, &fs_extra::dir::CopyOptions::new())
        .map_err(|e| format!("复制 MOD 失败: {}", e))?;

    // Force filesystem flush on Windows
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let _ = std::process::Command::new("cmd")
            .args(["/C", "dir", mods_dir.to_str().unwrap_or("")])
            .creation_flags(0x08000000)
            .output();
    }

    let _ = app.emit("mod-install-progress",
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
pub async fn uninstall_mod(mod_path: String) -> Result<InstallResult, String> {
    let mod_path_clone = mod_path.clone();
    tokio::task::spawn_blocking(move || {
        uninstall_mod_blocking(mod_path_clone)
    })
    .await
    .map_err(|e| format!("卸载任务执行失败: {}", e))?
}

fn uninstall_mod_blocking(mod_path: String) -> Result<InstallResult, String> {
    let path = PathBuf::from(&mod_path);

    if !path.exists() {
        return Err(format!("MOD 文件夹不存在: {}", mod_path));
    }

    let mod_name = path
        .file_name()
        .ok_or("无法获取 MOD 文件夹名称")?
        .to_string_lossy()
        .to_string();

    let mods_dir = path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    if !is_safe_to_delete(&path, &mods_dir) {
        return Err(format!("安全拦截: 不允许删除路径 {}", path.display()));
    }

    // First clean residual files in parent directory
    clean_residual_files(&path);

    // Then remove the mod folder itself
    fs::remove_dir_all(&path).map_err(|e| format!("删除 MOD 失败: {}", e))?;

    // Verify the folder is actually deleted, retry if needed (Windows file locking)
    for _attempt in 0..3 {
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

    let dest_canonical = match dest.canonicalize() {
        Ok(c) => c,
        Err(_) => {
            return Err("无法解析目标目录路径，可能存在安全隐患".to_string());
        }
    };

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("读取文件索引失败: {}", e))?;
        let file_name = file.name();

        if file_name.contains("..") {
            continue;
        }

        let outpath = dest.join(file_name);

        if let Ok(canonical) = outpath.canonicalize() {
            if !canonical.starts_with(&dest_canonical) {
                continue;
            }
        } else if !outpath.starts_with(dest) {
            continue;
        }

        if file_name.ends_with('/') {
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
        Err("压缩包中包含多个文件夹，但均未找到 manifest.json，无法确定 MOD 目录".to_string())
    }
}

fn clean_residual_files(mod_path: &PathBuf) {
    if let Some(parent) = mod_path.parent() {
        if let Ok(entries) = fs::read_dir(parent) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.ends_with(".tmp") || name.ends_with(".bak") {
                            let _ = fs::remove_file(&path);
                        }
                    }
                }
            }
        }
    }
}

#[tauri::command]
pub async fn install_mod(
    app: tauri::AppHandle,
    archive_path: String,
    mods_path: String,
    old_unique_id: Option<String>,
) -> Result<InstallResult, String> {
    let archive_path = archive_path.clone();
    let mods_path = mods_path.clone();
    tokio::task::spawn_blocking(move || {
        install_mod_blocking(app, archive_path, mods_path, old_unique_id)
    })
    .await
    .map_err(|e| format!("安装任务执行失败: {}", e))?
}

fn install_mod_blocking(
    app: tauri::AppHandle,
    archive_path: String,
    mods_path: String,
    old_unique_id: Option<String>,
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
        if !is_safe_to_delete(&dest_path, &mods_dir) {
            return Err(format!("安全拦截: 不允许删除路径 {}", dest_path.display()));
        }
        fs::remove_dir_all(&dest_path)
            .map_err(|e| format!("无法删除已存在的 MOD: {}", e))?;
    }

    if let Some(ref uid) = old_unique_id {
        if !uid.is_empty() {
            let removed = remove_old_mod_versions(&mods_dir, uid);
            if !removed.is_empty() {
                eprintln!("[install_mod_blocking] Removed old versions of {}: {:?}", uid, removed);
            }
        }
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

    println!("[install_mod_blocking] done, mod_name={}, mods_path={}", mod_name, mods_path);

    Ok(InstallResult {
        success: true,
        mod_name: Some(mod_name.clone()),
        message: format!("MOD '{}' 安装成功", mod_name),
    })
}

#[tauri::command]
pub async fn check_mod_dependencies(
    archive_path: String,
    mods_path: String,
) -> Result<ModDependencyCheck, String> {
    tokio::task::spawn_blocking(move || {
        check_mod_dependencies_blocking(archive_path, mods_path)
    })
    .await
    .map_err(|e| format!("依赖检查执行失败: {}", e))?
}

fn check_mod_dependencies_blocking(
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

    let normalized = crate::mod_parser::normalize_smart_quotes(&content);
    let cleaned = crate::mod_parser::remove_trailing_commas(&normalized);
    let cleaned = cleaned.strip_prefix('\u{FEFF}').unwrap_or(&cleaned);
    let manifest: serde_json::Value = serde_json::from_str(cleaned)
        .map_err(|e| format!("解析 manifest.json 失败: {}", e))?;

    let mod_name = manifest["Name"].as_str().unwrap_or("未知 MOD").to_string();
    let unique_id = manifest["UniqueID"].as_str().unwrap_or("").to_string();
    let version = manifest["Version"].as_str().unwrap_or("1.0.0").to_string();

    let mut missing_deps = Vec::new();

    if let Some(deps) = manifest["Dependencies"].as_array() {
        for dep in deps {
            let dep_id = dep["UniqueID"].as_str().unwrap_or("").to_string();
            if dep_id.is_empty() {
                continue;
            }

            if dep_id == "Pathoschild.SMAPI" {
                continue;
            }

            let is_required = dep["IsRequired"].as_bool().unwrap_or(true);

            let dep_folder = mods_dir.join(&dep_id);
            let mut found = false;
            if dep_folder.exists() {
                found = true;
            } else {
                if let Ok(entries) = fs::read_dir(&mods_dir) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_dir() {
                            let mf = p.join("manifest.json");
                            if mf.exists() {
                                if let Ok(mc) = fs::read_to_string(&mf) {
                                    let normalized = crate::mod_parser::normalize_smart_quotes(&mc);
                                    let cleaned = crate::mod_parser::remove_trailing_commas(&normalized);
                                    let cleaned = cleaned.strip_prefix('\u{FEFF}').unwrap_or(&cleaned);
                                    if let Ok(mv) = serde_json::from_str::<serde_json::Value>(cleaned) {
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

    cleanup_temp_dir_with_retry(&temp_dir);

    let mut installed_mod_ids = HashSet::new();
    if let Ok(entries) = fs::read_dir(&mods_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                let mf = p.join("manifest.json");
                if mf.exists() {
                    if let Ok(mc) = fs::read_to_string(&mf) {
                        let normalized = crate::mod_parser::normalize_smart_quotes(&mc);
                        let cleaned = crate::mod_parser::remove_trailing_commas(&normalized);
                        let cleaned = cleaned.strip_prefix('\u{FEFF}').unwrap_or(&cleaned);
                        if let Ok(mv) = serde_json::from_str::<serde_json::Value>(cleaned) {
                            if let Some(uid) = mv["UniqueID"].as_str() {
                                installed_mod_ids.insert(uid.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    apply_final_patches(&unique_id, &installed_mod_ids, &mut missing_deps);

    let has_required_missing = missing_deps.iter().any(|d| d.is_required);

    Ok(ModDependencyCheck {
        mod_name,
        unique_id,
        version,
        missing_dependencies: missing_deps,
        can_install: !has_required_missing,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_clean_residual_files_preserves_dot_config_files() {
        let tmp = TempDir::new().unwrap();
        let mods_dir = tmp.path().join("Mods");
        fs::create_dir_all(&mods_dir).unwrap();

        let dot_config = mods_dir.join(".smapi_config");
        fs::write(&dot_config, "config data").unwrap();

        let mod_path = mods_dir.join("SomeMod");
        fs::create_dir_all(&mod_path).unwrap();

        clean_residual_files(&mod_path);

        assert!(dot_config.exists(), "Dot-prefixed config files should NOT be deleted by clean_residual_files");
    }

    #[test]
    fn test_clean_residual_files_deletes_tmp_files() {
        let tmp = TempDir::new().unwrap();
        let mods_dir = tmp.path().join("Mods");
        fs::create_dir_all(&mods_dir).unwrap();

        let tmp_file = mods_dir.join("download.tmp");
        fs::write(&tmp_file, "temp data").unwrap();

        let mod_path = mods_dir.join("SomeMod");
        fs::create_dir_all(&mod_path).unwrap();

        clean_residual_files(&mod_path);

        assert!(!tmp_file.exists(), "Temp files should be deleted by clean_residual_files");
    }

    #[test]
    fn test_clean_residual_files_deletes_bak_files() {
        let tmp = TempDir::new().unwrap();
        let mods_dir = tmp.path().join("Mods");
        fs::create_dir_all(&mods_dir).unwrap();

        let bak_file = mods_dir.join("old_config.bak");
        fs::write(&bak_file, "backup data").unwrap();

        let mod_path = mods_dir.join("SomeMod");
        fs::create_dir_all(&mod_path).unwrap();

        clean_residual_files(&mod_path);

        assert!(!bak_file.exists(), "Backup files should be deleted by clean_residual_files");
    }

    #[test]
    fn test_clean_residual_files_preserves_normal_files() {
        let tmp = TempDir::new().unwrap();
        let mods_dir = tmp.path().join("Mods");
        fs::create_dir_all(&mods_dir).unwrap();

        let normal_file = mods_dir.join("readme.txt");
        fs::write(&normal_file, "readme content").unwrap();

        let mod_path = mods_dir.join("SomeMod");
        fs::create_dir_all(&mod_path).unwrap();

        clean_residual_files(&mod_path);

        assert!(normal_file.exists(), "Normal files should NOT be deleted by clean_residual_files");
    }

    #[test]
    fn test_find_mod_folder_multiple_dirs_none_with_manifest() {
        let tmp = TempDir::new().unwrap();
        let temp_dir = tmp.path().join("extracted");
        fs::create_dir_all(&temp_dir).unwrap();

        let dir1 = temp_dir.join("NotAMod1");
        fs::create_dir_all(&dir1).unwrap();
        fs::write(dir1.join("readme.txt"), "not a mod").unwrap();

        let dir2 = temp_dir.join("NotAMod2");
        fs::create_dir_all(&dir2).unwrap();
        fs::write(dir2.join("config.ini"), "not a mod").unwrap();

        let result = find_mod_folder(&temp_dir);
        assert!(result.is_err(), "Should return error when no directory contains manifest.json, got {:?}", result);
    }

    #[test]
    fn test_find_mod_folder_single_dir_with_manifest() {
        let tmp = TempDir::new().unwrap();
        let temp_dir = tmp.path().join("extracted");
        fs::create_dir_all(&temp_dir).unwrap();

        let mod_dir = temp_dir.join("RealMod");
        fs::create_dir_all(&mod_dir).unwrap();
        fs::write(mod_dir.join("manifest.json"), r#"{"Name":"Test","UniqueID":"test.mod","Version":"1.0.0"}"#).unwrap();

        let result = find_mod_folder(&temp_dir);
        assert!(result.is_ok(), "Should find mod folder with manifest.json");
        assert_eq!(result.unwrap().file_name().unwrap(), "RealMod");
    }

    #[test]
    fn test_find_mod_folder_multiple_dirs_one_with_manifest() {
        let tmp = TempDir::new().unwrap();
        let temp_dir = tmp.path().join("extracted");
        fs::create_dir_all(&temp_dir).unwrap();

        let not_mod = temp_dir.join("Documentation");
        fs::create_dir_all(&not_mod).unwrap();

        let real_mod = temp_dir.join("ActualMod");
        fs::create_dir_all(&real_mod).unwrap();
        fs::write(real_mod.join("manifest.json"), r#"{"Name":"Test","UniqueID":"test.mod","Version":"1.0.0"}"#).unwrap();

        let result = find_mod_folder(&temp_dir);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().file_name().unwrap(), "ActualMod");
    }

    #[test]
    fn test_is_safe_to_delete_direct_child() {
        let tmp = TempDir::new().unwrap();
        let mods_dir = tmp.path().join("Mods");
        fs::create_dir_all(&mods_dir).unwrap();
        let child = mods_dir.join("SomeMod");
        fs::create_dir_all(&child).unwrap();

        assert!(is_safe_to_delete(&child, &mods_dir));
    }

    #[test]
    fn test_is_safe_to_delete_nested_child() {
        let tmp = TempDir::new().unwrap();
        let mods_dir = tmp.path().join("Mods");
        fs::create_dir_all(&mods_dir).unwrap();
        let nested = mods_dir.join("SomeMod").join("subdir");
        fs::create_dir_all(&nested).unwrap();

        assert!(is_safe_to_delete(&nested, &mods_dir));
    }

    #[test]
    fn test_is_safe_to_delete_blocks_mods_dir_itself() {
        let tmp = TempDir::new().unwrap();
        let mods_dir = tmp.path().join("Mods");
        fs::create_dir_all(&mods_dir).unwrap();

        assert!(!is_safe_to_delete(&mods_dir, &mods_dir));
    }

    #[test]
    fn test_is_safe_to_delete_blocks_outside_path() {
        let tmp = TempDir::new().unwrap();
        let mods_dir = tmp.path().join("Mods");
        fs::create_dir_all(&mods_dir).unwrap();
        let outside = tmp.path().join("OtherDir");
        fs::create_dir_all(&outside).unwrap();

        assert!(!is_safe_to_delete(&outside, &mods_dir));
    }
}
