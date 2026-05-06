use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Cursor, Read, Write};
use std::path::PathBuf;
use tauri_plugin_dialog::{DialogExt, FilePath};
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileExportResult {
    pub success: bool,
    pub zip_path: String,
    pub mod_count: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackImportResult {
    pub success: bool,
    pub profile_name: String,
    mod_count: usize,
    message: String,
}

fn collect_mod_paths(mods_dir: &PathBuf, enabled_folders: &std::collections::HashSet<String>) -> io::Result<Vec<(PathBuf, String)>> {
    let mut files = Vec::new();

    if !mods_dir.exists() {
        return Ok(files);
    }

    for entry in fs::read_dir(mods_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let folder_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            if folder_name.starts_with('.') || folder_name == "Mods" {
                continue;
            }

            if !enabled_folders.contains(&folder_name) {
                let has_enabled_child = find_manifest_folders(&path, enabled_folders);
                if !has_enabled_child {
                    continue;
                }
            }

            collect_dir_files(&path, &mods_dir, &mut files)?;
        }
    }

    Ok(files)
}

fn find_manifest_folders(dir: &PathBuf, enabled_folders: &std::collections::HashSet<String>) -> bool {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let folder_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                if folder_name.starts_with('.') {
                    continue;
                }
                if enabled_folders.contains(&folder_name) {
                    return true;
                }
                if find_manifest_folders(&path, enabled_folders) {
                    return true;
                }
            }
        }
    }
    false
}

fn collect_dir_files(dir: &PathBuf, base_dir: &PathBuf, files: &mut Vec<(PathBuf, String)>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let relative = path.strip_prefix(base_dir)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            if relative.starts_with('/') || relative.starts_with('\\') {
                files.push((path, relative[1..].to_string()));
            } else {
                files.push((path, relative));
            }
        } else if path.is_dir() {
            collect_dir_files(&path, base_dir, files)?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn export_profile_to_zip(
    app: tauri::AppHandle,
    profile_name: String,
    game_path: String,
) -> Result<ProfileExportResult, String> {
    let mods_dir = PathBuf::from(&game_path).join("Mods");

    if !mods_dir.exists() {
        return Err("Mods directory not found".to_string());
    }

    let all_mods = crate::profiles::scan_mods_for_profiles(&game_path);
    let profile = crate::profiles::load_profile(&game_path, &profile_name)
        .map_err(|e| format!("Failed to load profile: {}", e))?;

    let enabled_set: std::collections::HashSet<String> = profile.enabled_mod_ids.iter().cloned().collect();

    let mut enabled_folders = std::collections::HashSet::new();
    for mod_info in &all_mods {
        if enabled_set.contains(&mod_info.unique_id) {
            let mod_path = std::path::PathBuf::from(&mod_info.folder_path);
            if let Some(parent) = mod_path.parent() {
                if parent.ends_with("Mods") {
                    if let Some(name) = mod_path.file_name().and_then(|n| n.to_str()) {
                        enabled_folders.insert(name.to_string());
                    }
                } else if let Some(grandparent) = parent.parent() {
                    if grandparent.ends_with("Mods") {
                        if let Some(name) = parent.file_name().and_then(|n| n.to_str()) {
                            enabled_folders.insert(name.to_string());
                        }
                    }
                }
            }
        }
    }

    let mod_files = collect_mod_paths(&mods_dir, &enabled_folders)
        .map_err(|e| format!("Failed to scan MODs: {}", e))?;

    if mod_files.is_empty() {
        return Err("No MODs found to export".to_string());
    }

    let mod_count = mod_files.len();

    let mut zip_buffer = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut zip_buffer);
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);

        for (file_path, relative_path) in &mod_files {
            zip.start_file(relative_path, options)
                .map_err(|e| format!("Failed to add file to zip: {}", e))?;

            let mut file = fs::File::open(file_path)
                .map_err(|e| format!("Failed to open file {}: {}", file_path.display(), e))?;

            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;

            zip.write_all(&buffer)
                .map_err(|e| format!("Failed to write file to zip: {}", e))?;
        }

        zip.finish()
            .map_err(|e| format!("Failed to finalize zip: {}", e))?;
    }

    let zip_data = zip_buffer.into_inner();

    let (tx, rx) = std::sync::mpsc::channel();

    app.dialog().file()
        .set_title("Save MOD Pack")
        .set_file_name(&format!("{}.zip", profile_name))
        .add_filter("ZIP Files", &["zip"])
        .save_file(move |path| {
            let _ = tx.send(path);
        });

    let selected_path = rx.recv()
        .map_err(|_| "Save dialog cancelled")?
        .ok_or("No path selected")?;

    let path_str = match selected_path {
        FilePath::Path(p) => p.to_string_lossy().to_string(),
        FilePath::Url(u) => u.to_string(),
    };

    fs::write(&path_str, zip_data)
        .map_err(|e| format!("Failed to save zip file: {}", e))?;

    eprintln!("[export_profile_to_zip] Saved {} files to {}", mod_count, path_str);

    Ok(ProfileExportResult {
        success: true,
        zip_path: path_str,
        mod_count,
        message: format!("Successfully exported {} MODs", mod_count),
    })
}

#[tauri::command]
pub async fn import_modpack_from_folder(
    _app: tauri::AppHandle,
    folder_path: String,
    target_profile_name: String,
    game_path: String,
) -> Result<ModpackImportResult, String> {
    let source_folder = PathBuf::from(&folder_path);

    if !source_folder.exists() {
        return Err(format!("Folder not found: {}", folder_path));
    }

    if !source_folder.is_dir() {
        return Err(format!("Path is not a directory: {}", folder_path));
    }

    let source_folder = if source_folder.join("Mods").is_dir() {
        source_folder.join("Mods")
    } else {
        source_folder
    };

    let mods_dir = PathBuf::from(&game_path).join("Mods");

    if !mods_dir.exists() {
        fs::create_dir_all(&mods_dir)
            .map_err(|e| format!("Failed to create Mods directory: {}", e))?;
    }

    let mut mod_count = 0;

    fn copy_dir_all(src: PathBuf, dst: PathBuf, count: &mut usize) -> Result<(), String> {
        if src.is_dir() {
            fs::create_dir_all(&dst)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
            
            for entry in fs::read_dir(&src)
                .map_err(|e| format!("Failed to read directory: {}", e))? 
            {
                let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                let path = entry.path();
                let file_name = entry.file_name();
                
                if file_name.to_string_lossy().starts_with('.') {
                    continue;
                }
                
                let dst_path = dst.join(&file_name);
                
                if path.is_dir() {
                    copy_dir_all(path, dst_path, count)?;
                } else {
                    fs::copy(&path, &dst_path)
                        .map_err(|e| format!("Failed to copy file: {}", e))?;
                    *count += 1;
                }
            }
        } else {
            fs::copy(&src, &dst)
                .map_err(|e| format!("Failed to copy file: {}", e))?;
            *count += 1;
        }
        Ok(())
    }

    for entry in fs::read_dir(&source_folder)
        .map_err(|e| format!("Failed to read source folder: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        let file_name = entry.file_name();
        
        if file_name.to_string_lossy().starts_with('.') || file_name.to_string_lossy() == "Mods" {
            continue;
        }
        
        let dst_path = mods_dir.join(&file_name);
        
        if path.is_dir() {
            if dst_path.exists() {
                eprintln!("[import_modpack_from_folder] Skipping existing: {}", dst_path.display());
            } else {
                copy_dir_all(path, dst_path, &mut mod_count)?;
            }
        } else {
            if !dst_path.exists() {
                fs::copy(&path, &dst_path)
                    .map_err(|e| format!("Failed to copy file: {}", e))?;
                mod_count += 1;
            }
        }
    }

    let all_mods = crate::profiles::scan_mods_for_profiles(&game_path);
    let enabled_mod_ids: Vec<String> = all_mods.iter().map(|m| m.unique_id.clone()).collect();

    let now = chrono::Utc::now().to_rfc3339();
    let profile = crate::profiles::Profile {
        name: target_profile_name.clone(),
        is_protected: false,
        enabled_mod_ids,
        created_at: now.clone(),
        last_used: now,
    };

    crate::profiles::save_profile(&profile, &game_path)
        .map_err(|e| format!("Failed to save profile: {}", e))?;

    eprintln!("[import_modpack_from_folder] Imported {} MODs to profile '{}'", mod_count, target_profile_name);

    Ok(ModpackImportResult {
        success: true,
        profile_name: target_profile_name,
        mod_count,
        message: format!("Successfully imported {} MODs", mod_count),
    })
}

#[tauri::command]
pub async fn import_modpack_from_zip(
    _app: tauri::AppHandle,
    zip_path: String,
    target_profile_name: String,
    game_path: String,
) -> Result<ModpackImportResult, String> {
    let zip_file_path = PathBuf::from(&zip_path);

    if !zip_file_path.exists() {
        return Err(format!("ZIP file not found: {}", zip_path));
    }

    let zip_data = fs::read(&zip_file_path)
        .map_err(|e| format!("Failed to read ZIP file: {}", e))?;

    let mut archive = ZipArchive::new(Cursor::new(zip_data))
        .map_err(|e| format!("Failed to open ZIP archive: {}", e))?;

    let mods_dir = PathBuf::from(&game_path).join("Mods");

    if !mods_dir.exists() {
        fs::create_dir_all(&mods_dir)
            .map_err(|e| format!("Failed to create Mods directory: {}", e))?;
    }

    let mut mod_count = 0;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to access file in archive: {}", e))?;

        let raw_path = match file.enclosed_name() {
            Some(path) => path,
            None => continue,
        };

        let clean_path = if raw_path.starts_with("Mods/") || raw_path.starts_with("Mods\\") {
            raw_path.strip_prefix("Mods/").unwrap_or_else(|_| raw_path.strip_prefix("Mods\\").unwrap_or(raw_path)).to_path_buf()
        } else {
            raw_path.to_path_buf()
        };

        if clean_path.as_os_str().is_empty() {
            continue;
        }

        let outpath = mods_dir.join(&clean_path);

        if outpath.exists() {
            continue;
        }

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)
                        .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                }
            }

            let mut outfile = fs::File::create(&outpath)
                .map_err(|e| format!("Failed to create file: {}", e))?;

            io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to extract file: {}", e))?;

            mod_count += 1;
        }
    }

    let all_mods = crate::profiles::scan_mods_for_profiles(&game_path);
    let enabled_mod_ids: Vec<String> = all_mods.iter().map(|m| m.unique_id.clone()).collect();

    let now = chrono::Utc::now().to_rfc3339();
    let profile = crate::profiles::Profile {
        name: target_profile_name.clone(),
        is_protected: false,
        enabled_mod_ids,
        created_at: now.clone(),
        last_used: now,
    };

    crate::profiles::save_profile(&profile, &game_path)
        .map_err(|e| format!("Failed to save profile: {}", e))?;

    eprintln!("[import_modpack_from_zip] Imported {} MODs to profile '{}'", mod_count, target_profile_name);

    Ok(ModpackImportResult {
        success: true,
        profile_name: target_profile_name,
        mod_count,
        message: format!("Successfully imported {} MODs", mod_count),
    })
}
