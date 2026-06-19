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

        // 每次都创建独立时间戳目录，毫秒级冲突自动加后缀
        let mut backup_dir = backup_root.join(format!("EditorAuto_{}", timestamp));
        let mut suffix = 0u32;
        while backup_dir.exists() {
            suffix += 1;
            backup_dir = backup_root.join(format!("EditorAuto_{}_{}", timestamp, suffix));
        }
        std::fs::create_dir_all(&backup_dir)?;

        let save_name = self
            .main_save_path
            .file_name()
            .ok_or_else(|| {
                crate::save_editor::error::SaveEditorError::NotFound("save name".to_string())
            })?
            .to_string_lossy()
            .to_string();

        let target = backup_dir.join(&save_name);
        // 只复制存档主文件到备份目录，不复制整个文件夹
        std::fs::copy(&self.main_save_path, &target).map_err(|e| {
            crate::save_editor::error::SaveEditorError::BackupFailed(e.to_string())
        })?;

        // 同时复制 SaveGameInfo（如果有）方便完整恢复
        let info_src = self.folder_path.join("SaveGameInfo");
        if info_src.exists() {
            let _ = std::fs::copy(&info_src, backup_dir.join("SaveGameInfo"));
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

    let entries: Vec<_> = std::fs::read_dir(folder_path)?
        .flatten()
        .filter(|e| e.path().is_file())
        .collect();

    // 第1遍：精确匹配文件名（优先），排除 SaveGameInfo
    for entry in &entries {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
            if name == folder_name {
                return Ok(path);
            }
        }
    }

    // 第2遍：降级使用 ends_with 匹配（兼容旧版或备份文件命名）
    for entry in &entries {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
            if name != "SaveGameInfo" && name.ends_with(&folder_name) {
                return Ok(path);
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
