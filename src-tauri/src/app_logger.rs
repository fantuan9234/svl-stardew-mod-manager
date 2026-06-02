use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

pub fn get_svl_data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let d_drive = PathBuf::from("D:\\SVL");
        if PathBuf::from("D:\\").exists() {
            let _ = fs::create_dir_all(&d_drive);
            return d_drive;
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            let app_support = PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("SVL");
            if let Some(parent) = app_support.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::create_dir_all(&app_support);
            return app_support;
        }
    }

    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("SVL")
}

lazy_static::lazy_static! {
    static ref APP_LOGGER: Mutex<AppLogger> = Mutex::new(AppLogger::new());
}

struct AppLogger {
    log_dir: PathBuf,
}

impl AppLogger {
    fn new() -> Self {
        let log_dir = get_svl_data_dir().join("logs");
        let _ = fs::create_dir_all(&log_dir);
        Self { log_dir }
    }

    fn log_file_path(&self) -> PathBuf {
        let date = Local::now().format("%Y-%m-%d").to_string();
        self.log_dir.join(format!("svl-{}.log", date))
    }

    fn write_entry(&self, level: &str, category: &str, message: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
        let line = format!("[{}] [{}] [{}] {}\n", timestamp, level, category, message);

        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.log_file_path())
        {
            let _ = file.write_all(line.as_bytes());
        }

        eprintln!("[{}] [{}] {}", level, category, message);
    }

    fn read_logs(&self, max_lines: usize) -> Vec<String> {
        let path = self.log_file_path();
        if !path.exists() {
            return vec![];
        }
        match fs::read_to_string(&path) {
            Ok(content) => {
                let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
                let start = if lines.len() > max_lines {
                    lines.len() - max_lines
                } else {
                    0
                };
                lines[start..].to_vec()
            }
            Err(_) => vec![],
        }
    }

    fn read_all_log_files(&self) -> Vec<LogFileInfo> {
        let mut files = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.log_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("log") {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                    let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                    let modified = fs::metadata(&path)
                        .and_then(|m| m.modified())
                        .and_then(|t| {
                            let dt: chrono::DateTime<Local> = t.into();
                            Ok(dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        })
                        .unwrap_or_default();
                    files.push(LogFileInfo {
                        name,
                        path: path.to_string_lossy().to_string(),
                        size_bytes: size,
                        modified,
                    });
                }
            }
        }
        files.sort_by(|a, b| b.modified.cmp(&a.modified));
        files
    }

    fn export_logs(&self) -> Result<String, String> {
        let mut all_content = String::new();
        let files = self.read_all_log_files();
        for file in &files {
            all_content.push_str(&format!("========== {} ==========\n", file.name));
            if let Ok(content) = fs::read_to_string(&file.path) {
                all_content.push_str(&content);
            }
            all_content.push_str("\n\n");
        }
        let export_path = self.log_dir.join("svl-export-all.log");
        fs::write(&export_path, &all_content)
            .map_err(|e| format!("导出日志失败: {}", e))?;
        Ok(export_path.to_string_lossy().to_string())
    }

    fn clear_old_logs(&self, keep_days: u64) -> Result<usize, String> {
        let mut removed = 0;
        let cutoff = Local::now() - chrono::Duration::days(keep_days as i64);
        let cutoff_str = cutoff.format("%Y-%m-%d").to_string();

        if let Ok(entries) = fs::read_dir(&self.log_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("log") {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with("svl-") && !name.contains("export") {
                            let date_part = name
                                .trim_start_matches("svl-")
                                .trim_end_matches(".log");
                            if date_part < cutoff_str.as_str() {
                                if fs::remove_file(&path).is_ok() {
                                    removed += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(removed)
    }
}

pub fn log_info(category: &str, message: &str) {
    if let Ok(logger) = APP_LOGGER.lock() {
        logger.write_entry("INFO", category, message);
    }
}

pub fn log_warn(category: &str, message: &str) {
    if let Ok(logger) = APP_LOGGER.lock() {
        logger.write_entry("WARN", category, message);
    }
}

pub fn log_error(category: &str, message: &str) {
    if let Ok(logger) = APP_LOGGER.lock() {
        logger.write_entry("ERROR", category, message);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFileInfo {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub modified: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppLogResult {
    pub lines: Vec<String>,
    pub total_lines: usize,
    pub log_dir: String,
    pub files: Vec<LogFileInfo>,
}

#[tauri::command]
pub fn get_app_logs(max_lines: Option<usize>) -> Result<AppLogResult, String> {
    let max = max_lines.unwrap_or(500);
    let logger = APP_LOGGER.lock().map_err(|e| format!("获取日志锁失败: {}", e))?;
    let lines = logger.read_logs(max);
    let total_lines = lines.len();
    let files = logger.read_all_log_files();
    let log_dir = logger.log_dir.to_string_lossy().to_string();
    Ok(AppLogResult {
        lines,
        total_lines,
        log_dir,
        files,
    })
}

#[tauri::command]
pub fn export_app_logs() -> Result<String, String> {
    let logger = APP_LOGGER.lock().map_err(|e| format!("获取日志锁失败: {}", e))?;
    logger.export_logs()
}

#[tauri::command]
pub fn clear_old_app_logs(keep_days: Option<u64>) -> Result<String, String> {
    let days = keep_days.unwrap_or(30);
    let logger = APP_LOGGER.lock().map_err(|e| format!("获取日志锁失败: {}", e))?;
    let removed = logger.clear_old_logs(days)?;
    Ok(format!("已清理 {} 个旧日志文件（保留最近 {} 天）", removed, days))
}

#[tauri::command]
pub fn get_log_dir_path() -> Result<String, String> {
    let logger = APP_LOGGER.lock().map_err(|e| format!("获取日志锁失败: {}", e))?;
    Ok(logger.log_dir.to_string_lossy().to_string())
}
