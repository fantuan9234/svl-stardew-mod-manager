use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use tauri::Emitter;

use crate::mod_name_resolver::resolve_mod_name;
use crate::dependency_patches::apply_final_patches;
use crate::app_logger::get_svl_data_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModType {
    Smapi,
    ContentPack,
    Unknown,
}

fn detect_mod_type(temp_dir: &Path) -> ModType {
    let (has_manifest, is_content_pack) = check_manifest_in_dir(temp_dir);
    if has_manifest {
        if is_content_pack {
            return ModType::ContentPack;
        }
        return ModType::Smapi;
    }
    ModType::Unknown
}

fn check_manifest_in_dir(dir: &Path) -> (bool, bool) {
    if let Ok(entries) = fs::read_dir(dir) {
        let dirs: Vec<_> = entries.filter_map(|e| e.ok()).filter(|e| e.path().is_dir()).collect();
        if dirs.len() == 1 {
            let single = &dirs[0];
            if let Some(result) = check_manifest_at(&single.path()) {
                return result;
            }
            if let Ok(sub_entries) = fs::read_dir(single.path()) {
                for sub in sub_entries.flatten() {
                    let sub_path = sub.path();
                    if !sub_path.is_dir() {
                        continue;
                    }
                    if let Some(result) = check_manifest_at(&sub_path) {
                        return result;
                    }
                }
            }
        }
        for entry in &dirs {
            if let Some(result) = check_manifest_at(&entry.path()) {
                return result;
            }
        }
    }
    check_manifest_at(dir).unwrap_or((false, false))
}

fn check_manifest_at(dir: &Path) -> Option<(bool, bool)> {
    let manifest_path = dir.join("manifest.json");
    if !manifest_path.exists() {
        return None;
    }
    if let Ok(content) = fs::read_to_string(&manifest_path) {
        let normalized = crate::mod_parser::normalize_smart_quotes(&content);
        let cleaned = crate::mod_parser::remove_trailing_commas(&normalized);
        let cleaned = cleaned.strip_prefix('\u{FEFF}').unwrap_or(&cleaned);
        if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(cleaned) {
            let is_cp = manifest.get("ContentPackFor")
                .and_then(|v| {
                    if v.is_string() {
                        Some(v.as_str().unwrap_or("").to_string())
                    } else {
                        v.get("UniqueID").or_else(|| v.get("UniqueId")).and_then(|u| u.as_str()).map(|s| s.to_string())
                    }
                })
                .map(|uid| uid == "Pathoschild.ContentPatcher")
                .unwrap_or(false);
            return Some((true, is_cp));
        }
    }
    None
}

fn extract_content_paths_from_description(description: &str) -> Vec<String> {
    let re = regex::Regex::new(r"(?i)Content[/\\][\w./\\]+").unwrap();
    let mut paths: Vec<String> = re
        .find_iter(description)
        .map(|m| m.as_str().replace('\\', "/"))
        .collect();
    paths.sort_by(|a, b| b.len().cmp(&a.len()));
    paths.dedup();
    paths
}


fn decode_zip_filename(raw: &[u8]) -> String {
    if let Ok(s) = std::str::from_utf8(raw) {
        if !s.contains('\u{FFFD}') {
            return s.to_string();
        }
    }

    let (decoded, _, had_errors) = encoding_rs::GBK.decode(raw);
    if !had_errors {
        return decoded.into_owned();
    }

    let (decoded, _, had_errors) = encoding_rs::GB18030.decode(raw);
    if !had_errors {
        return decoded.into_owned();
    }

    String::from_utf8_lossy(raw).into_owned()
}

fn get_trash_dir() -> Option<PathBuf> {
    let data_dir = get_svl_data_dir();
    let trash = data_dir.join("trash");
    if !trash.exists() {
        let _ = fs::create_dir_all(&trash);
    }
    Some(trash)
}

fn cleanup_old_trash() {
    let trash_dir = get_svl_data_dir();
    let trash = trash_dir.join("trash");
    if !trash.exists() {
        return;
    }
    if let Ok(entries) = fs::read_dir(&trash) {
        let now = std::time::SystemTime::now();
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(duration) = now.duration_since(modified) {
                        if duration.as_secs() > 7 * 24 * 3600 {
                            let _ = fs::remove_dir_all(entry.path());
                        }
                    }
                }
            }
        }
    }
}

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

fn is_safe_source_for_install(source: &Path, mods_dir: &Path) -> Result<(), String> {
    let source_canon = source
        .canonicalize()
        .map_err(|e| format!("无法解析源文件夹路径 '{}': {}", source.display(), e))?;
    let mods_canon = mods_dir
        .canonicalize()
        .map_err(|e| format!("无法解析 Mods 目录路径 '{}': {}", mods_dir.display(), e))?;

    if source_canon == mods_canon {
        return Err(format!(
            "安全拦截: 源文件夹不能是 Mods 目录本身 ({})。请选择 Mods 目录之外的文件夹",
            source.display()
        ));
    }

    if source_canon.starts_with(&mods_canon) {
        return Err(format!(
            "安全拦截: 源文件夹不能位于 Mods 目录内部 ({})。请选择 Mods 目录之外的文件夹",
            source.display()
        ));
    }

    Ok(())
}

fn install_via_staging(
    source: &Path,
    dest_path: &Path,
    mods_dir: &Path,
) -> Result<(), String> {
    let folder_name = dest_path
        .file_name()
        .ok_or_else(|| format!("无法获取目标文件夹名称: {}", dest_path.display()))?
        .to_string_lossy()
        .to_string();

    let mut backup_path: Option<PathBuf> = None;
    if dest_path.exists() {
        if !is_safe_to_delete(dest_path, mods_dir) {
            return Err(format!(
                "安全拦截: 不允许替换目标路径 {}",
                dest_path.display()
            ));
        }

        if let Some(trash_dir) = get_trash_dir() {
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let trash_name = format!("{}_{}", folder_name, timestamp);
            let trash_target = trash_dir.join(&trash_name);
            if let Err(_) = fs::rename(dest_path, &trash_target) {
                let backup_name = format!(".{}.svl_backup", folder_name);
                let backup = mods_dir.join(&backup_name);
                if backup.exists() {
                    let _ = fs::remove_dir_all(&backup);
                }
                fs::rename(dest_path, &backup).map_err(|e| {
                    format!(
                        "备份已存在的 MOD 失败 ({} -> {}): {}",
                        dest_path.display(),
                        backup.display(),
                        e
                    )
                })?;
                backup_path = Some(backup);
            } else {
                eprintln!("[install_via_staging] Moved old mod to trash: {}", trash_name);
            }
        } else {
            let backup_name = format!(".{}.svl_backup", folder_name);
            let backup = mods_dir.join(&backup_name);
            if backup.exists() {
                let _ = fs::remove_dir_all(&backup);
            }
            fs::rename(dest_path, &backup).map_err(|e| {
                format!(
                    "备份已存在的 MOD 失败 ({} -> {}): {}",
                    dest_path.display(),
                    backup.display(),
                    e
                )
            })?;
            backup_path = Some(backup);
        }
    }

    if let Err(e) = fs_extra::dir::copy(source, mods_dir, &fs_extra::dir::CopyOptions::new()) {
        if dest_path.exists() {
            let _ = fs::remove_dir_all(dest_path);
        }
        if let Some(backup) = &backup_path {
            let _ = fs::rename(backup, dest_path);
        }
        return Err(format!("复制 MOD 失败: {}", e));
    }

    if !dest_path.exists() {
        if let Some(backup) = &backup_path {
            let _ = fs::rename(backup, dest_path);
        }
        return Err(format!(
            "复制完成后未找到目标目录: {}",
            dest_path.display()
        ));
    }

    if let Some(backup) = backup_path {
        if let Some(trash_dir) = get_trash_dir() {
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let trash_name = format!("{}_{}", folder_name, timestamp);
            let trash_target = trash_dir.join(&trash_name);
            if fs::rename(&backup, &trash_target).is_ok() {
                eprintln!("[install_via_staging] Moved backup to trash: {}", trash_name);
            } else {
                let _ = fs::remove_dir_all(&backup);
            }
        } else {
            let _ = fs::remove_dir_all(&backup);
        }
    }

    Ok(())
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

fn remove_old_mod_versions(mods_dir: &PathBuf, unique_id: &str, new_folder_name: &str) -> Vec<String> {
    let mut removed = Vec::new();
    if let Ok(entries) = fs::read_dir(mods_dir) {
        let all_entries: Vec<_> = entries.flatten().collect();
        for entry in &all_entries {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let folder_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            if folder_name.starts_with('.') || folder_name.starts_with('_') {
                continue;
            }
            if folder_name == new_folder_name {
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
                let dot_manifest = path.join(".manifest.json");
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
                if let Some(trash_dir) = get_trash_dir() {
                    let trash_target = trash_dir.join(&folder_name);
                    if trash_target.exists() {
                        let _ = fs::remove_dir_all(&trash_target);
                    }
                    if fs::rename(&path, &trash_target).is_ok() {
                        eprintln!("[remove_old_mod_versions] Moved old version to trash: {}", folder_name);
                        removed.push(folder_name);
                    } else {
                        eprintln!("[remove_old_mod_versions] Failed to move to trash, removing: {}", path.display());
                        if fs::remove_dir_all(&path).is_ok() {
                            removed.push(folder_name);
                        }
                    }
                } else {
                    eprintln!("[remove_old_mod_versions] Removing old version: {}", path.display());
                    if fs::remove_dir_all(&path).is_ok() {
                        removed.push(folder_name);
                    }
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
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub installed_mods: Option<Vec<String>>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallSourceSafety {
    pub safe: bool,
    pub risk: String,
    pub reason: String,
    pub source_path: String,
    pub mods_path: String,
    pub conflicting_mod_name: Option<String>,
}

#[tauri::command]
pub fn check_install_source_safety(
    source_path: String,
    mods_path: String,
) -> Result<InstallSourceSafety, String> {
    let source = PathBuf::from(&source_path);
    let mods_dir = PathBuf::from(&mods_path);

    if !source.exists() {
        return Ok(InstallSourceSafety {
            safe: false,
            risk: "missing".to_string(),
            reason: format!("源文件夹不存在: {}", source_path),
            source_path,
            mods_path,
            conflicting_mod_name: None,
        });
    }

    if !source.is_dir() {
        return Ok(InstallSourceSafety {
            safe: false,
            risk: "not_dir".to_string(),
            reason: format!("所选路径不是一个文件夹: {}", source_path),
            source_path,
            mods_path,
            conflicting_mod_name: None,
        });
    }

    match is_safe_source_for_install(&source, &mods_dir) {
        Ok(()) => Ok(InstallSourceSafety {
            safe: true,
            risk: "none".to_string(),
            reason: "源路径合法，可安全安装".to_string(),
            source_path,
            mods_path,
            conflicting_mod_name: None,
        }),
        Err(reason) => {
            let folder_name = source
                .file_name()
                .map(|n| n.to_string_lossy().to_string());
            let dest_path = folder_name.as_ref().map(|n| mods_dir.join(n));
            let conflicting_mod_name = dest_path.and_then(|p| {
                if p.exists() {
                    folder_name.clone()
                } else {
                    None
                }
            });
            Ok(InstallSourceSafety {
                safe: false,
                risk: "inside_mods".to_string(),
                reason,
                source_path,
                mods_path,
                conflicting_mod_name,
            })
        }
    }
}

#[tauri::command]
pub async fn install_mod_from_archive(
    app: tauri::AppHandle,
    archive_path: String,
    mods_path: String,
    old_unique_id: Option<String>,
    variant_filter: Option<String>,
    nexus_description: Option<String>,
) -> Result<InstallResult, String> {
    let archive_path = archive_path.clone();
    let mods_path = mods_path.clone();
    tokio::task::spawn_blocking(move || {
        install_mod_from_archive_blocking(app, archive_path, mods_path, old_unique_id, variant_filter, nexus_description)
    })
    .await
    .map_err(|e| format!("安装任务执行失败: {}", e))?
}

pub(crate) fn install_mod_from_archive_blocking(
    app: tauri::AppHandle,
    archive_path: String,
    mods_path: String,
    old_unique_id: Option<String>,
    variant_filter: Option<String>,
    nexus_description: Option<String>,
) -> Result<InstallResult, String> {
    let archive = PathBuf::from(&archive_path);
    let mods_dir = PathBuf::from(&mods_path);

    if !archive.exists() {
        return Err(format!("压缩包不存在: {}", archive_path));
    }

    if let Ok(mut f) = std::fs::File::open(&archive) {
        use std::io::Read;
        let mut head = [0u8; 4];
        let n = f.read(&mut head).unwrap_or(0);
        let valid_zip = n == 4 && &head == b"PK\x03\x04";
        if !valid_zip {
            let size = std::fs::metadata(&archive).map(|m| m.len()).unwrap_or(0);
            let ascii_preview: String = head.iter().map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' }).collect();
            return Err(format!(
                "下载的文件不是有效的 ZIP 压缩包 ({} 字节，前 4 字节预览: '{}' 0x{:02x?})。可能原因：1) 下载未完成；2) Nexus 限流返回了错误页面；3) 需要重新登录。文件路径: {}",
                size, ascii_preview, head, archive_path
            ));
        }
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

    let extract_result = match extension.as_str() {
        "zip" => extract_zip(&archive, &temp_dir),
        "7z" => extract_7z(&archive, &temp_dir),
        "rar" => Err("RAR 格式暂不支持，请使用 ZIP 或 7Z 格式".to_string()),
        _ => Err(format!("不支持的格式: .{}", extension)),
    };
    if let Err(e) = extract_result {
        cleanup_temp_dir_with_retry(&temp_dir);
        return Err(e);
    }

    let mod_type = detect_mod_type(&temp_dir);

    match mod_type {
        ModType::Unknown => {
            cleanup_temp_dir_with_retry(&temp_dir);
            return Err("无法识别的模组类型：未找到 manifest.json。请确认压缩包内容是否正确".to_string());
        }
        _ => {}
    }

    let mod_folders = match find_mod_folders(&temp_dir) {
        Ok(f) => f,
        Err(e) => {
            cleanup_temp_dir_with_retry(&temp_dir);
            return Err(e);
        }
    };

    eprintln!("[install_mod_from_archive] Found {} mod folder(s) in archive", mod_folders.len());
    for (i, folder) in mod_folders.iter().enumerate() {
        eprintln!("[install_mod_from_archive]   [{}] {}", i, folder.display());
    }

    let mut installed_names: Vec<String> = Vec::new();
    let mut last_error: Option<String> = None;

    if mod_folders.len() > 1 {
        let common_parent = mod_folders[0].parent().map(|p| p.to_path_buf());
        if let Some(ref parent) = common_parent {
            let all_same_parent = mod_folders.iter().all(|f| f.parent().map(|p| p.to_path_buf()).as_ref() == Some(parent));
            if all_same_parent && *parent != temp_dir {
                let bundle_name = match parent.file_name() {
                    Some(n) => n.to_string_lossy().to_string(),
                    None => "Unknown".to_string(),
                };

                let _ = app.emit(
                    "mod-install-progress",
                    InstallProgressEvent {
                        step: "installing".to_string(),
                        mod_name: Some(bundle_name.clone()),
                        message: format!("正在安装 '{}' (包含 {} 个子模组)...", bundle_name, mod_folders.len()),
                    },
                );

                let dest_path = mods_dir.join(&bundle_name);

                if let Some(ref uid) = old_unique_id {
                    if !uid.is_empty() {
                        let removed = remove_old_mod_versions(&mods_dir, uid, &bundle_name);
                        if !removed.is_empty() {
                            eprintln!("[install_mod_from_archive] Removed old versions of {}: {:?}", uid, removed);
                        }
                    }
                }

                if let Err(e) = install_via_staging(parent, &dest_path, &mods_dir) {
                    eprintln!("[install_mod_from_archive] Failed to install bundle '{}': {}", bundle_name, e);
                    last_error = Some(format!("安装 '{}' 失败: {}", bundle_name, e));
                } else {
                    installed_names.push(bundle_name.clone());

                    for mf in &mod_folders {
                        if let Some(sub_name) = mf.file_name() {
                            let sub_manifest = dest_path.join(sub_name).join("manifest.json");
                            if sub_manifest.exists() {
                                if let Ok(content) = fs::read_to_string(&sub_manifest) {
                                    let normalized = crate::mod_parser::normalize_smart_quotes(&content);
                                    let cleaned = crate::mod_parser::remove_trailing_commas(&normalized);
                                    let cleaned = cleaned.strip_prefix('\u{FEFF}').unwrap_or(&cleaned);
                                    if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(cleaned) {
                                        let ver = manifest["Version"].as_str().unwrap_or("unknown");
                                        let uid = manifest["UniqueID"].as_str().unwrap_or("unknown");
                                        crate::app_logger::log_info("ModInstaller", &format!(
                                            "Installed '{} {}' (UniqueID: {}) version: {}",
                                            bundle_name, sub_name.to_string_lossy(), uid, ver
                                        ));
                                    }
                                }
                            }
                        }
                    }

                    let _ = app.emit(
                        "mod-install-progress",
                        InstallProgressEvent {
                            step: "sub-done".to_string(),
                            mod_name: Some(bundle_name.clone()),
                            message: format!("MOD '{}' 安装成功 (含 {} 个子模组)", bundle_name, mod_folders.len()),
                        },
                    );
                }

                cleanup_temp_dir_with_retry(&temp_dir);

                if installed_names.is_empty() {
                    return Err(last_error.unwrap_or_else(|| "安装失败".to_string()));
                }

                let primary_name = installed_names[0].clone();
                let _ = app.emit(
                    "mod-install-progress",
                    InstallProgressEvent {
                        step: "done".to_string(),
                        mod_name: Some(primary_name.clone()),
                        message: format!("MOD '{}' 安装成功", primary_name),
                    },
                );

                println!("[install_mod_from_archive] done (bundle mode), installed={:?}, mods_path={}", installed_names, mods_path);

                return Ok(InstallResult {
                    success: true,
                    mod_name: Some(primary_name.clone()),
                    message: format!("MOD '{}' 安装成功 (含 {} 个子模组)", primary_name, mod_folders.len()),
                    installed_mods: Some(installed_names),
                });
            }
        }
    }

    for mod_folder in &mod_folders {
        let mod_name = match mod_folder.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };

        let _ = app.emit(
            "mod-install-progress",
            InstallProgressEvent {
                step: "installing".to_string(),
                mod_name: Some(mod_name.clone()),
                message: format!("正在安装 '{}'...", mod_name),
            },
        );

        let dest_path = mods_dir.join(&mod_name);

        if let Some(ref uid) = old_unique_id {
            if !uid.is_empty() {
                let removed = remove_old_mod_versions(&mods_dir, uid, &mod_name);
                if !removed.is_empty() {
                    eprintln!("[install_mod_from_archive] Removed old versions of {}: {:?}", uid, removed);
                }
            }
        }

        if let Err(e) = install_via_staging(mod_folder, &dest_path, &mods_dir) {
            eprintln!("[install_mod_from_archive] Failed to install sub-mod '{}': {}", mod_name, e);
            last_error = Some(format!("子模组 '{}' 安装失败: {}", mod_name, e));
            continue;
        }

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

        installed_names.push(mod_name.clone());
    }

    cleanup_temp_dir_with_retry(&temp_dir);

    if installed_names.is_empty() {
        return Err(last_error.unwrap_or_else(|| "安装失败：未成功安装任何子模组".to_string()));
    }

    let primary_name = installed_names[0].clone();
    let _ = app.emit(
        "mod-install-progress",
        InstallProgressEvent {
            step: "done".to_string(),
            mod_name: Some(primary_name.clone()),
            message: if installed_names.len() > 1 {
                format!("已安装 {} 个子模组: {}", installed_names.len(), installed_names.join(", "))
            } else {
                format!("MOD '{}' 安装成功", primary_name)
            },
        },
    );

    println!("[install_mod_from_archive] done, installed={:?}, mods_path={}", installed_names, mods_path);

    Ok(InstallResult {
        success: true,
        mod_name: Some(primary_name.clone()),
        message: if installed_names.len() > 1 {
            format!("已安装 {} 个子模组", installed_names.len())
        } else {
            format!("MOD '{}' 安装成功", primary_name)
        },
        installed_mods: if installed_names.len() > 1 {
            Some(installed_names)
        } else {
            None
        },
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

    if !source.is_dir() {
        return Err(format!("源路径不是一个文件夹: {}", source_path));
    }

    if !mods_dir.exists() {
        fs::create_dir_all(&mods_dir)
            .map_err(|e| format!("无法创建 Mods 文件夹: {}", e))?;
    }

    is_safe_source_for_install(&source, &mods_dir)?;

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

    install_via_staging(&source, &dest_path, &mods_dir)?;

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
        installed_mods: None,
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

    clean_residual_files(&path);

    cleanup_old_trash();

    let mut moved_to_trash = false;
    if let Some(trash_dir) = get_trash_dir() {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let trash_name = format!("{}_{}", mod_name, timestamp);
        let trash_target = trash_dir.join(&trash_name);
        if fs::rename(&path, &trash_target).is_ok() {
            eprintln!("[uninstall_mod] Moved to trash: {}", trash_name);
            moved_to_trash = true;
        }
    }

    if !moved_to_trash {
        fs::remove_dir_all(&path).map_err(|e| format!("删除 MOD 失败: {}", e))?;
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
    }

    Ok(InstallResult {
        success: true,
        mod_name: Some(mod_name.clone()),
        message: format!("MOD '{}' 已卸载并清理完成", mod_name),
        installed_mods: None,
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
        let file_name_raw = file.name_raw();
        let file_name = decode_zip_filename(file_name_raw);

        if file_name.contains("..") {
            continue;
        }

        let outpath = dest.join(&file_name);

        if let Ok(canonical) = outpath.canonicalize() {
            if !canonical.starts_with(&dest_canonical) {
                continue;
            }
        } else if !outpath.starts_with(dest) {
            continue;
        }

        if file.is_dir() {
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

fn find_mod_folders(temp_dir: &PathBuf) -> Result<Vec<PathBuf>, String> {
    if temp_dir.join("manifest.json").exists() {
        return Ok(vec![temp_dir.clone()]);
    }

    let entries: Vec<_> = fs::read_dir(temp_dir)
        .map_err(|e| format!("读取临时文件夹失败: {}", e))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();

    if entries.is_empty() {
        return Err("未在压缩包中找到 MOD 文件夹或 manifest.json".to_string());
    }

    if entries.len() == 1 {
        let single = entries[0].path();
        if single.join("manifest.json").exists() {
            return Ok(vec![single]);
        }
        let mut found: Vec<PathBuf> = Vec::new();
        if let Ok(sub_entries) = fs::read_dir(&single) {
            for sub in sub_entries.flatten() {
                let sub_path = sub.path();
                if sub_path.is_dir() && sub_path.join("manifest.json").exists() {
                    found.push(sub_path);
                }
            }
        }
        if !found.is_empty() {
            return Ok(found);
        }
        return Ok(vec![single]);
    }

    let mut found: Vec<PathBuf> = Vec::new();
    for entry in &entries {
        if entry.path().join("manifest.json").exists() {
            found.push(entry.path());
        }
    }
    if found.is_empty() {
        return Err("压缩包中包含多个文件夹，但均未找到 manifest.json，无法确定 MOD 目录".to_string());
    }
    Ok(found)
}

fn find_mod_folder(temp_dir: &PathBuf) -> Result<PathBuf, String> {
    let mut folders = find_mod_folders(temp_dir)?;
    if folders.is_empty() {
        return Err("未在压缩包中找到 MOD 文件夹或 manifest.json".to_string());
    }
    Ok(folders.remove(0))
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
    variant_filter: Option<String>,
    nexus_description: Option<String>,
) -> Result<InstallResult, String> {
    let archive_path = archive_path.clone();
    let mods_path = mods_path.clone();
    tokio::task::spawn_blocking(move || {
        install_mod_blocking(app, archive_path, mods_path, old_unique_id, variant_filter, nexus_description)
    })
    .await
    .map_err(|e| format!("安装任务执行失败: {}", e))?
}

fn install_mod_blocking(
    app: tauri::AppHandle,
    archive_path: String,
    mods_path: String,
    old_unique_id: Option<String>,
    variant_filter: Option<String>,
    nexus_description: Option<String>,
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

    let extract_result = match extension.as_str() {
        "zip" => extract_zip(&archive, &temp_dir),
        "7z" => extract_7z(&archive, &temp_dir),
        "rar" => Err("RAR 格式暂不支持，请使用 ZIP 或 7Z 格式".to_string()),
        _ => Err(format!("不支持的格式: .{}", extension)),
    };
    if let Err(e) = extract_result {
        cleanup_temp_dir_with_retry(&temp_dir);
        return Err(e);
    }

    let mod_type = detect_mod_type(&temp_dir);

    match mod_type {
        ModType::Unknown => {
            cleanup_temp_dir_with_retry(&temp_dir);
            return Err("无法识别的模组类型：未找到 manifest.json。请确认压缩包内容是否正确".to_string());
        }
        _ => {}
    }

    let mod_folders = match find_mod_folders(&temp_dir) {
        Ok(f) => f,
        Err(e) => {
            cleanup_temp_dir_with_retry(&temp_dir);
            return Err(e);
        }
    };

    eprintln!("[install_mod_blocking] Found {} mod folder(s) in archive", mod_folders.len());
    for (i, folder) in mod_folders.iter().enumerate() {
        eprintln!("[install_mod_blocking]   [{}] {}", i, folder.display());
    }

    let mut installed_names: Vec<String> = Vec::new();
    let mut last_error: Option<String> = None;

    if mod_folders.len() > 1 {
        let common_parent = mod_folders[0].parent().map(|p| p.to_path_buf());
        if let Some(ref parent) = common_parent {
            let all_same_parent = mod_folders.iter().all(|f| f.parent().map(|p| p.to_path_buf()).as_ref() == Some(parent));
            if all_same_parent && *parent != temp_dir {
                let bundle_name = match parent.file_name() {
                    Some(n) => n.to_string_lossy().to_string(),
                    None => "Unknown".to_string(),
                };

                let _ = app.emit(
                    "mod-install-progress",
                    InstallProgressEvent {
                        step: "installing".to_string(),
                        mod_name: Some(bundle_name.clone()),
                        message: format!("正在安装 '{}' (包含 {} 个子模组)...", bundle_name, mod_folders.len()),
                    },
                );

                let dest_path = mods_dir.join(&bundle_name);

                if let Some(ref uid) = old_unique_id {
                    if !uid.is_empty() {
                        let removed = remove_old_mod_versions(&mods_dir, uid, &bundle_name);
                        if !removed.is_empty() {
                            eprintln!("[install_mod_blocking] Removed old versions of {}: {:?}", uid, removed);
                        }
                    }
                }

                if let Err(e) = install_via_staging(parent, &dest_path, &mods_dir) {
                    eprintln!("[install_mod_blocking] Failed to install bundle '{}': {}", bundle_name, e);
                    last_error = Some(format!("安装 '{}' 失败: {}", bundle_name, e));
                } else {
                    installed_names.push(bundle_name.clone());

                    for mf in &mod_folders {
                        if let Some(sub_name) = mf.file_name() {
                            let sub_manifest = dest_path.join(sub_name).join("manifest.json");
                            if sub_manifest.exists() {
                                if let Ok(content) = fs::read_to_string(&sub_manifest) {
                                    let normalized = crate::mod_parser::normalize_smart_quotes(&content);
                                    let cleaned = crate::mod_parser::remove_trailing_commas(&normalized);
                                    let cleaned = cleaned.strip_prefix('\u{FEFF}').unwrap_or(&cleaned);
                                    if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(cleaned) {
                                        let ver = manifest["Version"].as_str().unwrap_or("unknown");
                                        let uid = manifest["UniqueID"].as_str().unwrap_or("unknown");
                                        crate::app_logger::log_info("ModInstaller", &format!(
                                            "Installed '{} {}' (UniqueID: {}) version: {}",
                                            bundle_name, sub_name.to_string_lossy(), uid, ver
                                        ));
                                    }
                                }
                            }
                        }
                    }

                    let _ = app.emit(
                        "mod-install-progress",
                        InstallProgressEvent {
                            step: "sub-done".to_string(),
                            mod_name: Some(bundle_name.clone()),
                            message: format!("MOD '{}' 安装成功 (含 {} 个子模组)", bundle_name, mod_folders.len()),
                        },
                    );
                }

                cleanup_temp_dir_with_retry(&temp_dir);

                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("cmd")
                        .args(["/C", "dir", mods_dir.to_str().unwrap_or("")])
                        .output();
                }

                if installed_names.is_empty() {
                    return Err(last_error.unwrap_or_else(|| "安装失败".to_string()));
                }

                let primary_name = installed_names[0].clone();
                let _ = app.emit(
                    "mod-install-progress",
                    InstallProgressEvent {
                        step: "done".to_string(),
                        mod_name: Some(primary_name.clone()),
                        message: format!("MOD '{}' 安装成功", primary_name),
                    },
                );

                println!("[install_mod_blocking] done (bundle mode), installed={:?}, mods_path={}", installed_names, mods_path);

                return Ok(InstallResult {
                    success: true,
                    mod_name: Some(primary_name.clone()),
                    message: format!("MOD '{}' 安装成功 (含 {} 个子模组)", primary_name, mod_folders.len()),
                    installed_mods: Some(installed_names),
                });
            }
        }
    }

    for mod_folder in mod_folders {
        let mod_name = match mod_folder.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };

        let _ = app.emit(
            "mod-install-progress",
            InstallProgressEvent {
                step: "installing".to_string(),
                mod_name: Some(mod_name.clone()),
                message: format!("正在安装 '{}'...", mod_name),
            },
        );

        let dest_path = mods_dir.join(&mod_name);

        if let Some(ref uid) = old_unique_id {
            if !uid.is_empty() {
                let removed = remove_old_mod_versions(&mods_dir, uid, &mod_name);
                if !removed.is_empty() {
                    eprintln!("[install_mod_blocking] Removed old versions of {}: {:?}", uid, removed);
                }
            }
        }

        if let Err(e) = install_via_staging(&mod_folder, &dest_path, &mods_dir) {
            eprintln!("[install_mod_blocking] Failed to install sub-mod '{}': {}", mod_name, e);
            last_error = Some(format!("子模组 '{}' 安装失败: {}", mod_name, e));
            continue;
        }

        installed_names.push(mod_name.clone());

        let _ = app.emit(
            "mod-install-progress",
            InstallProgressEvent {
                step: "sub-done".to_string(),
                mod_name: Some(mod_name.clone()),
                message: format!("MOD '{}' 安装成功", mod_name),
            },
        );
    }

    cleanup_temp_dir_with_retry(&temp_dir);

    // Force filesystem flush on Windows
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "dir", mods_dir.to_str().unwrap_or("")])
            .output();
    }

    if installed_names.is_empty() {
        return Err(last_error.unwrap_or_else(|| "安装失败：未成功安装任何子模组".to_string()));
    }

    let primary_name = installed_names[0].clone();
    let _ = app.emit(
        "mod-install-progress",
        InstallProgressEvent {
            step: "done".to_string(),
            mod_name: Some(primary_name.clone()),
            message: if installed_names.len() > 1 {
                format!("已安装 {} 个子模组: {}", installed_names.len(), installed_names.join(", "))
            } else {
                format!("MOD '{}' 安装成功", primary_name)
            },
        },
    );

    println!("[install_mod_blocking] done, installed={:?}, mods_path={}", installed_names, mods_path);

    Ok(InstallResult {
        success: true,
        mod_name: Some(primary_name.clone()),
        installed_mods: if installed_names.len() > 1 {
            Some(installed_names.clone())
        } else {
            None
        },
        message: if installed_names.len() > 1 {
            format!("已安装 {} 个子模组", installed_names.len())
        } else {
            format!("MOD '{}' 安装成功", primary_name)
        },
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

    let cp_for = manifest.get("ContentPackFor")
        .and_then(|v| {
            if v.is_string() {
                Some(v.as_str().unwrap_or("").to_string())
            } else {
                v.get("UniqueID").or_else(|| v.get("UniqueId")).and_then(|u| u.as_str()).map(|s| s.to_string())
            }
        });

    if let Some(ref cp_uid) = cp_for {
        if !cp_uid.is_empty() && cp_uid != "Pathoschild.SMAPI" {
            let cp_found = mods_dir.join(cp_uid).exists() || {
                let mut found = false;
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
                                        if mv["UniqueID"].as_str() == Some(cp_uid.as_str()) {
                                            found = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                found
            };
            if !cp_found {
                let display_name = resolve_mod_name(cp_uid);
                missing_deps.push(MissingDepInfo {
                    unique_id: cp_uid.clone(),
                    display_name,
                    minimum_version: None,
                    is_required: true,
                });
            }
        }
    }

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
    fn test_find_mod_folder_nested_single_top_dir() {
        let tmp = TempDir::new().unwrap();
        let temp_dir = tmp.path().join("extracted");
        fs::create_dir_all(&temp_dir).unwrap();

        let top = temp_dir.join("Stardew Valley Expanded");
        fs::create_dir_all(&top).unwrap();
        fs::write(top.join("readme.txt"), "wrapper dir, not the real mod").unwrap();

        let real_mod = top.join("[CP] Stardew Valley Expanded");
        fs::create_dir_all(&real_mod).unwrap();
        fs::write(real_mod.join("manifest.json"), r#"{"Name":"SVE","UniqueID":"FlashShifter.StardewValleyExpandedCP","Version":"1.0.0","ContentPackFor":{"UniqueID":"Pathoschild.ContentPatcher"}}"#).unwrap();

        let result = find_mod_folder(&temp_dir);
        assert!(result.is_ok(), "Should drill into single top-level dir to find manifest.json, got {:?}", result);
        assert_eq!(result.unwrap().file_name().unwrap(), "[CP] Stardew Valley Expanded");
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

    #[test]
    fn test_decode_zip_filename_ascii() {
        let result = decode_zip_filename(b"StardewMod/manifest.json");
        assert_eq!(result, "StardewMod/manifest.json");
    }

    #[test]
    fn test_decode_zip_filename_utf8() {
        let result = decode_zip_filename("StardewMod/中文修订/i18n.json".as_bytes());
        assert_eq!(result, "StardewMod/中文修订/i18n.json");
    }

    #[test]
    fn test_decode_zip_filename_gbk() {
        let gbk_bytes: Vec<u8> = encoding_rs::GBK.encode("中文修订").0.into_owned();
        let result = decode_zip_filename(&gbk_bytes);
        assert_eq!(result, "中文修订");
    }

    #[test]
    fn test_decode_zip_filename_gbk_with_path() {
        let gbk_bytes: Vec<u8> = encoding_rs::GBK.encode("StardewMod/中文修订/图片.png").0.into_owned();
        let result = decode_zip_filename(&gbk_bytes);
        assert_eq!(result, "StardewMod/中文修订/图片.png");
    }

    #[test]
    fn test_decode_zip_filename_gb18030() {
        let gb18030_bytes: Vec<u8> = encoding_rs::GB18030.encode("中文修订测试").0.into_owned();
        let result = decode_zip_filename(&gb18030_bytes);
        assert_eq!(result, "中文修订测试");
    }

    #[test]
    fn test_extract_zip_chinese_localization() {
        use std::io::Write;
        let tmp = TempDir::new().unwrap();
        let archive_path = tmp.path().join("test_cn.zip");
        let extract_dir = tmp.path().join("extracted");
        fs::create_dir_all(&extract_dir).unwrap();

        let file = fs::File::create(&archive_path).unwrap();
        let mut zip_writer = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        zip_writer.add_directory("StardewMod中文修订/", options).unwrap();

        zip_writer.start_file("StardewMod中文修订/图片.png", options).unwrap();
        zip_writer.write_all(b"fake image data").unwrap();

        zip_writer.start_file("StardewMod中文修订/manifest.json", options).unwrap();
        let manifest_content = r#"{"Name":"测试","UniqueID":"test.cn","Version":"1.0.0"}"#;
        zip_writer.write_all(manifest_content.as_bytes()).unwrap();

        zip_writer.finish().unwrap();

        extract_zip(&archive_path, &extract_dir).unwrap();

        let mod_dir = extract_dir.join("StardewMod中文修订");
        assert!(mod_dir.exists(), "Mod directory with Chinese name should exist: {:?}", mod_dir);

        let image_file = mod_dir.join("图片.png");
        assert!(image_file.exists(), "Image file with Chinese name should exist: {:?}", image_file);

        let manifest_file = mod_dir.join("manifest.json");
        assert!(manifest_file.exists(), "Manifest should exist");
    }

    #[test]
    fn test_is_safe_source_for_install_outside_mods() {
        let tmp = TempDir::new().unwrap();
        let mods_dir = tmp.path().join("Mods");
        fs::create_dir_all(&mods_dir).unwrap();
        let source = tmp.path().join("ExternalMod");
        fs::create_dir_all(&source).unwrap();

        let result = is_safe_source_for_install(&source, &mods_dir);
        assert!(result.is_ok(), "External source should be allowed, got: {:?}", result);
    }

    #[test]
    fn test_is_safe_source_for_install_blocks_mods_dir_itself() {
        let tmp = TempDir::new().unwrap();
        let mods_dir = tmp.path().join("Mods");
        fs::create_dir_all(&mods_dir).unwrap();

        let result = is_safe_source_for_install(&mods_dir, &mods_dir);
        assert!(result.is_err(), "Mods dir itself should be blocked");
        assert!(result.unwrap_err().contains("不能是 Mods 目录本身"));
    }

    #[test]
    fn test_is_safe_source_for_install_blocks_subfolder_of_mods() {
        let tmp = TempDir::new().unwrap();
        let mods_dir = tmp.path().join("Mods");
        fs::create_dir_all(&mods_dir).unwrap();
        let sub = mods_dir.join("美化合集");
        fs::create_dir_all(&sub).unwrap();

        let result = is_safe_source_for_install(&sub, &mods_dir);
        assert!(result.is_err(), "Subfolder of Mods should be blocked, got: {:?}", result);
        assert!(result.unwrap_err().contains("不能位于 Mods 目录内部"));
    }

    #[test]
    fn test_is_safe_source_for_install_blocks_nested_inside_mods() {
        let tmp = TempDir::new().unwrap();
        let mods_dir = tmp.path().join("Mods");
        fs::create_dir_all(&mods_dir).unwrap();
        let nested = mods_dir.join("category").join("MyMod");
        fs::create_dir_all(&nested).unwrap();

        let result = is_safe_source_for_install(&nested, &mods_dir);
        assert!(result.is_err(), "Nested folder inside Mods should be blocked");
    }

    #[test]
    fn test_install_via_staging_copies_new_mod() {
        let tmp = TempDir::new().unwrap();
        let mods_dir = tmp.path().join("Mods");
        fs::create_dir_all(&mods_dir).unwrap();
        let source = tmp.path().join("NewMod");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("manifest.json"), r#"{"Name":"X","UniqueID":"x.mod","Version":"1.0.0"}"#).unwrap();
        fs::write(source.join("data.txt"), "hello").unwrap();

        let dest_path = mods_dir.join("NewMod");
        install_via_staging(&source, &dest_path, &mods_dir).unwrap();

        assert!(dest_path.exists(), "Dest should exist after install");
        assert!(dest_path.join("manifest.json").exists());
        assert_eq!(
            fs::read_to_string(dest_path.join("data.txt")).unwrap(),
            "hello"
        );
        assert!(!mods_dir.join(".NewMod.svl_backup").exists(), "No backup should be left when there was no existing mod");
    }

    #[test]
    fn test_install_via_staging_replaces_existing_mod() {
        let tmp = TempDir::new().unwrap();
        let mods_dir = tmp.path().join("Mods");
        fs::create_dir_all(&mods_dir).unwrap();

        let source = tmp.path().join("SameName");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("new.txt"), "new content").unwrap();

        let dest_path = mods_dir.join("SameName");
        fs::create_dir_all(&dest_path).unwrap();
        fs::write(dest_path.join("old.txt"), "old content").unwrap();

        install_via_staging(&source, &dest_path, &mods_dir).unwrap();

        assert!(dest_path.exists(), "Dest should still exist after install");
        assert!(dest_path.join("new.txt").exists(), "New file should be present");
        assert!(!dest_path.join("old.txt").exists(), "Old file should be gone after replacement");
        assert!(!mods_dir.join(".SameName.svl_backup").exists(), "Backup should be removed after successful install");
    }

    #[test]
    fn test_install_via_staging_preserves_existing_on_copy_failure() {
        let tmp = TempDir::new().unwrap();
        let mods_dir = tmp.path().join("Mods");
        fs::create_dir_all(&mods_dir).unwrap();

        let dest_path = mods_dir.join("ImportantMod");
        let important_file = dest_path.join("important.txt");
        fs::create_dir_all(&dest_path).unwrap();
        fs::write(&important_file, "I must not be lost").unwrap();

        let source = tmp.path().join("NonexistentSource");
        let result = install_via_staging(&source, &dest_path, &mods_dir);
        assert!(result.is_err(), "Should fail when source doesn't exist");

        assert!(dest_path.exists(), "Original mod dir must still exist after failure");
        assert!(important_file.exists(), "Important file must be preserved on failure");
        assert_eq!(fs::read_to_string(&important_file).unwrap(), "I must not be lost");
        assert!(!mods_dir.join(".ImportantMod.svl_backup").exists(), "Backup must be cleaned up on failure");
    }

    #[test]
    fn test_install_via_staging_replaces_stale_backup_atomically() {
        let tmp = TempDir::new().unwrap();
        let mods_dir = tmp.path().join("Mods");
        fs::create_dir_all(&mods_dir).unwrap();

        let stale_backup = mods_dir.join(".MyMod.svl_backup");
        fs::create_dir_all(&stale_backup).unwrap();
        fs::write(stale_backup.join("stale.txt"), "leftover from previous failed install").unwrap();

        let source = tmp.path().join("MyMod");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("manifest.json"), r#"{"Name":"X","UniqueID":"x","Version":"1"}"#).unwrap();

        let dest_path = mods_dir.join("MyMod");
        fs::create_dir_all(&dest_path).unwrap();
        fs::write(dest_path.join("current.txt"), "current mod content").unwrap();

        install_via_staging(&source, &dest_path, &mods_dir).unwrap();

        assert!(dest_path.exists());
        assert!(dest_path.join("manifest.json").exists(), "New mod should be installed");
        assert!(!stale_backup.exists(), "Stale backup should be removed when replaced by fresh backup");
    }
}
