use crate::save_editor::error::Result;
use crate::save_editor::xml_utils;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SaveFile {
    pub folder_path: PathBuf,
    pub main_save_path: PathBuf,
    pub raw_xml: String,
    pub character_name: String,
}

impl SaveFile {
    pub fn load(folder_path: &Path) -> Result<Self> {
        let main_save_path = locate_main_save_file(folder_path)?;
        let raw_xml = xml_utils::read_xml_file(&main_save_path)?;
        xml_utils::validate_save_root(&raw_xml)?;

        let character_name = extract_character_name(folder_path)
            .unwrap_or_else(|| {
                folder_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string()
            });

        Ok(Self {
            folder_path: folder_path.to_path_buf(),
            main_save_path,
            raw_xml,
            character_name,
        })
    }

    pub fn backup(&self) -> Result<PathBuf> {
        use chrono::Local;
        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_root = self.folder_path.join("SVL_Backups");
        let backup_dir = backup_root.join(format!("EditorAuto_{}", timestamp));
        std::fs::create_dir_all(&backup_dir)?;

        let save_name = self
            .folder_path
            .file_name()
            .ok_or_else(|| {
                crate::save_editor::error::SaveEditorError::NotFound("folder name".to_string())
            })?
            .to_string_lossy()
            .to_string();

        let target = backup_dir.join(&save_name);
        fs_extra::dir::copy(
            &self.folder_path,
            &backup_root,
            &fs_extra::dir::CopyOptions::new()
                .overwrite(true)
                .content_only(false),
        )
        .map_err(|e| {
            crate::save_editor::error::SaveEditorError::BackupFailed(e.to_string())
        })?;

        let copied = backup_root.join(&save_name);
        if copied.exists() && copied != target {
            std::fs::rename(&copied, &target)?;
        }
        Ok(target)
    }

    pub fn write(&self) -> Result<()> {
        let tmp = self.main_save_path.with_extension("xml.tmp");
        std::fs::write(&tmp, &self.raw_xml).map_err(|e| {
            crate::save_editor::error::SaveEditorError::WriteFailed(e.to_string())
        })?;
        std::fs::rename(&tmp, &self.main_save_path).map_err(|e| {
            crate::save_editor::error::SaveEditorError::WriteFailed(e.to_string())
        })?;
        Ok(())
    }

    pub fn set_xml(&mut self, xml: String) {
        self.raw_xml = xml;
    }
}

fn locate_main_save_file(folder_path: &Path) -> Result<PathBuf> {
    let folder_name = folder_path
        .file_name()
        .ok_or_else(|| {
            crate::save_editor::error::SaveEditorError::NotFound("folder name".to_string())
        })?
        .to_string_lossy()
        .to_string();

    let entries = std::fs::read_dir(folder_path)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if name != "SaveGameInfo" && name.ends_with(&folder_name) {
                    return Ok(path);
                }
            }
        }
    }

    Err(crate::save_editor::error::SaveEditorError::NotFound(format!(
        "Main save file in {}",
        folder_path.display()
    )))
}

fn extract_character_name(folder_path: &Path) -> Option<String> {
    let info_path = folder_path.join("SaveGameInfo");
    let content = std::fs::read_to_string(info_path).ok()?;
    let open = "<name>";
    let close = "</name>";
    let start = content.find(open)? + open.len();
    let end = content[start..].find(close)? + start;
    Some(content[start..end].to_string())
}
