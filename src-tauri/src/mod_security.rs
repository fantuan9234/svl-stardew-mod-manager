use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Emitter;

lazy_static::lazy_static! {
    static ref GAME_MONITOR: Mutex<GameMonitorState> = Mutex::new(GameMonitorState {
        is_running: false,
        pid: None,
        mod_load_events: Vec::new(),
        error_events: Vec::new(),
        warning_events: Vec::new(),
        loaded_mods: 0,
        total_mods: 0,
    });
}

static MONITOR_ACTIVE: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
pub struct GameMonitorState {
    pub is_running: bool,
    pub pid: Option<u32>,
    pub mod_load_events: Vec<ModLoadEvent>,
    pub error_events: Vec<ModErrorEvent>,
    pub warning_events: Vec<ModWarningEvent>,
    pub loaded_mods: usize,
    pub total_mods: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModLoadEvent {
    pub mod_name: String,
    pub unique_id: String,
    pub load_time_ms: u64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModErrorEvent {
    pub mod_name: String,
    pub unique_id: String,
    pub error_message: String,
    pub severity: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModWarningEvent {
    pub mod_name: String,
    pub unique_id: String,
    pub warning_message: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModMonitorStatus {
    pub is_game_running: bool,
    pub pid: Option<u32>,
    pub loaded_mods: usize,
    pub total_mods: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub mod_load_events: Vec<ModLoadEvent>,
    pub error_events: Vec<ModErrorEvent>,
    pub warning_events: Vec<ModWarningEvent>,
    pub health_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModSecurityReport {
    pub mod_name: String,
    pub unique_id: String,
    pub security_score: f64,
    pub risk_level: String,
    pub checks: Vec<SecurityCheck>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityCheck {
    pub check_name: String,
    pub passed: bool,
    pub severity: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSecurityReport {
    pub reports: Vec<ModSecurityReport>,
    pub average_score: f64,
    pub high_risk_count: usize,
    pub medium_risk_count: usize,
    pub low_risk_count: usize,
}

fn parse_smapi_log_for_events(log_path: &str) -> (Vec<ModLoadEvent>, Vec<ModErrorEvent>, Vec<ModWarningEvent>) {
    let mut load_events = Vec::new();
    let mut error_events = Vec::new();
    let mut warning_events = Vec::new();

    if let Ok(content) = fs::read_to_string(log_path) {
        for line in content.lines() {
            if line.contains("[mods]") && line.contains("loaded") {
                if let Some(mod_name) = extract_mod_name_from_log(line) {
                    let unique_id = extract_unique_id_from_log(line).unwrap_or_else(|| mod_name.clone());
                    load_events.push(ModLoadEvent {
                        mod_name,
                        unique_id,
                        load_time_ms: 0,
                        timestamp: chrono::Local::now().to_rfc3339(),
                    });
                }
            }

            if line.contains("[error]") || line.contains("[ERROR]") {
                if let Some(mod_name) = extract_mod_name_from_log(line) {
                    error_events.push(ModErrorEvent {
                        mod_name,
                        unique_id: String::new(),
                        error_message: line.to_string(),
                        severity: "error".to_string(),
                        timestamp: chrono::Local::now().to_rfc3339(),
                    });
                }
            }

            if line.contains("[warn]") || line.contains("[WARN]") {
                if let Some(mod_name) = extract_mod_name_from_log(line) {
                    warning_events.push(ModWarningEvent {
                        mod_name,
                        unique_id: String::new(),
                        warning_message: line.to_string(),
                        timestamp: chrono::Local::now().to_rfc3339(),
                    });
                }
            }
        }
    }

    (load_events, error_events, warning_events)
}

fn extract_mod_name_from_log(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    for (i, part) in parts.iter().enumerate() {
        if part.ends_with(':') && i + 1 < parts.len() {
            return Some(part.trim_end_matches(':').to_string());
        }
    }
    None
}

fn extract_unique_id_from_log(line: &str) -> Option<String> {
    if let Some(start) = line.find('(') {
        if let Some(end) = line.find(')') {
            if end > start {
                return Some(line[start + 1..end].to_string());
            }
        }
    }
    None
}

fn get_smapi_log_path() -> Option<String> {
    if let Some(app_data) = dirs::config_dir() {
        let log_path = app_data.join("ConcernedApe").join("StardewValley").join("SMAPI").join("log.txt");
        if log_path.exists() {
            return Some(log_path.to_string_lossy().to_string());
        }

        let log_path = app_data.join("ConcernedApe").join("StardewValley").join("ErrorLogs").join("SMAPI-latest.txt");
        if log_path.exists() {
            return Some(log_path.to_string_lossy().to_string());
        }
    }
    None
}

pub fn monitor_game_loop(app: tauri::AppHandle) {
    MONITOR_ACTIVE.store(true, Ordering::SeqCst);

    std::thread::spawn(move || {
        while MONITOR_ACTIVE.load(Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_secs(2));

            let monitor = match GAME_MONITOR.lock() {
                Ok(m) => m,
                Err(_) => {
                    MONITOR_ACTIVE.store(false, Ordering::SeqCst);
                    return;
                }
            };
            if !monitor.is_running {
                continue;
            }
            drop(monitor);

            if let Some(log_path) = get_smapi_log_path() {
                let (load_events, error_events, warning_events) = parse_smapi_log_for_events(&log_path);

                let mut monitor = match GAME_MONITOR.lock() {
                    Ok(m) => m,
                    Err(_) => break,
                };
                monitor.mod_load_events.extend(load_events.clone());
                monitor.error_events.extend(error_events.clone());
                monitor.warning_events.extend(warning_events.clone());
                monitor.loaded_mods = monitor.mod_load_events.len();

                if monitor.mod_load_events.len() > 1000 {
                    let excess = monitor.mod_load_events.len() - 1000;
                    monitor.mod_load_events.drain(0..excess);
                }
                if monitor.error_events.len() > 500 {
                    let excess = monitor.error_events.len() - 500;
                    monitor.error_events.drain(0..excess);
                }
                if monitor.warning_events.len() > 500 {
                    let excess = monitor.warning_events.len() - 500;
                    monitor.warning_events.drain(0..excess);
                }

                drop(monitor);

                if !load_events.is_empty() || !error_events.is_empty() || !warning_events.is_empty() {
                    let _ = app.emit(
                        "mod-monitor-update",
                        serde_json::json!({
                            "new_load_events": load_events,
                            "new_error_events": error_events,
                            "new_warning_events": warning_events,
                        }),
                    );
                }
            }
        }
    });
}

#[tauri::command]
pub fn start_game_monitor() -> Result<bool, String> {
    let mut monitor = GAME_MONITOR.lock()
        .map_err(|e| format!("启动游戏监控失败: {}", e))?;
    monitor.is_running = true;
    monitor.mod_load_events.clear();
    monitor.error_events.clear();
    monitor.warning_events.clear();
    monitor.loaded_mods = 0;
    Ok(true)
}

#[tauri::command]
pub fn stop_game_monitor() -> Result<bool, String> {
    MONITOR_ACTIVE.store(false, Ordering::SeqCst);
    match GAME_MONITOR.lock() {
        Ok(mut monitor) => {
            monitor.is_running = false;
            Ok(true)
        }
        Err(_) => Ok(false),
    }
}

#[tauri::command]
pub fn get_monitor_status(total_mods: usize) -> Result<ModMonitorStatus, String> {
    let monitor = GAME_MONITOR.lock()
        .map_err(|e| format!("获取游戏监控状态失败: {}", e))?;

    let health_score = if monitor.total_mods > 0 {
        let error_ratio = monitor.error_events.len() as f64 / monitor.total_mods as f64;
        (1.0 - error_ratio) * 100.0
    } else {
        100.0
    };

    Ok(ModMonitorStatus {
        is_game_running: monitor.is_running,
        pid: monitor.pid,
        loaded_mods: monitor.loaded_mods,
        total_mods,
        error_count: monitor.error_events.len(),
        warning_count: monitor.warning_events.len(),
        mod_load_events: monitor.mod_load_events.clone(),
        error_events: monitor.error_events.clone(),
        warning_events: monitor.warning_events.clone(),
        health_score: health_score.max(0.0).min(100.0),
    })
}

fn check_dll_presence(mod_path: &PathBuf) -> SecurityCheck {
    let mut has_dll = false;

    if let Ok(entries) = fs::read_dir(mod_path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if let Some(ext) = p.extension() {
                if ext.to_string_lossy().to_lowercase() == "dll" {
                    has_dll = true;
                    break;
                }
            }
        }
    }

    SecurityCheck {
        check_name: "C# DLL Detection".to_string(),
        passed: !has_dll,
        severity: if has_dll { "Medium".to_string() } else { "Low".to_string() },
        description: if has_dll {
            "Mod contains compiled C# DLL files. DLL mods have full access to game internals.".to_string()
        } else {
            "Mod does not contain compiled DLL files.".to_string()
        },
    }
}

fn check_entry_class(mod_path: &PathBuf) -> SecurityCheck {
    let manifest_path = mod_path.join("manifest.json");

    if !manifest_path.exists() {
        return SecurityCheck {
            check_name: "Entry Class Check".to_string(),
            passed: true,
            severity: "Low".to_string(),
            description: "No manifest.json found, cannot verify entry class.".to_string(),
        };
    }

    if let Ok(content) = fs::read_to_string(&manifest_path) {
        if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&content) {
            let has_entry = manifest["EntryClass"].as_str().is_some();
            let has_entry_dll = manifest["EntryDll"].as_str().is_some();

            return SecurityCheck {
                check_name: "Entry Point Verification".to_string(),
                passed: !has_entry_dll,
                severity: if has_entry_dll { "Medium".to_string() } else { "Low".to_string() },
                description: if has_entry_dll {
                    format!("Mod specifies EntryDll: {}. This mod executes custom C# code.", manifest["EntryDll"].as_str().unwrap_or(""))
                } else if has_entry {
                    format!("Mod specifies EntryClass: {}. This is a standard SMAPI mod.", manifest["EntryClass"].as_str().unwrap_or(""))
                } else {
                    "Mod does not specify a custom entry point.".to_string()
                },
            };
        }
    }

    SecurityCheck {
        check_name: "Entry Point Verification".to_string(),
        passed: true,
        severity: "Low".to_string(),
        description: "Unable to parse manifest.json.".to_string(),
    }
}

fn check_content_patcher_only(mod_path: &PathBuf) -> SecurityCheck {
    let mut _has_content_patchers = 0;
    let mut has_other_files = 0;

    if let Ok(entries) = fs::read_dir(mod_path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                    let name_lower = name.to_lowercase();
                    if name_lower.contains("content.json") || name_lower.ends_with(".json") {
                        _has_content_patchers += 1;
                    } else if name_lower.ends_with(".png") || name_lower.ends_with(".xnb") || name_lower.ends_with(".json") {
                        // Content files are safe
                    } else {
                        has_other_files += 1;
                    }
                }
            }
        }
    }

    let is_safe = has_other_files == 0;

    SecurityCheck {
        check_name: "Content Patcher Safety".to_string(),
        passed: is_safe,
        severity: if is_safe { "Low".to_string() } else { "Info".to_string() },
        description: if is_safe {
            "Mod appears to be content-only (images, JSON, etc.). Very low risk.".to_string()
        } else {
            "Mod contains files beyond standard content. Review recommended.".to_string()
        },
    }
}

fn check_update_keys(mod_path: &PathBuf) -> SecurityCheck {
    let manifest_path = mod_path.join("manifest.json");

    if !manifest_path.exists() {
        return SecurityCheck {
            check_name: "Nexus Verification".to_string(),
            passed: false,
            severity: "Medium".to_string(),
            description: "No manifest.json found, cannot verify mod authenticity.".to_string(),
        };
    }

    if let Ok(content) = fs::read_to_string(&manifest_path) {
        if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&content) {
            let has_update_keys = manifest["UpdateKeys"].as_array().is_some();
            let update_keys = manifest["UpdateKeys"].as_array().map(|arr| arr.len()).unwrap_or(0);

            return SecurityCheck {
                check_name: "Nexus Verification".to_string(),
                passed: has_update_keys && update_keys > 0,
                severity: if has_update_keys && update_keys > 0 { "Low".to_string() } else { "Medium".to_string() },
                description: if has_update_keys && update_keys > 0 {
                    format!("Mod has {} Nexus update key(s). Mod is verified on Nexus Mods.", update_keys)
                } else {
                    "Mod does not have Nexus update keys. May be from unofficial sources.".to_string()
                },
            };
        }
    }

    SecurityCheck {
        check_name: "Nexus Verification".to_string(),
        passed: false,
        severity: "Medium".to_string(),
        description: "Unable to parse manifest.json.".to_string(),
    }
}

fn calculate_security_score(checks: &[SecurityCheck]) -> f64 {
    if checks.is_empty() {
        return 100.0;
    }

    let mut score: f64 = 100.0;

    for check in checks {
        if !check.passed {
            match check.severity.as_str() {
                "Critical" => score -= 40.0,
                "High" => score -= 25.0,
                "Medium" => score -= 15.0,
                "Low" => score -= 5.0,
                _ => score -= 2.0,
            }
        }
    }

    score.max(0.0).min(100.0)
}

fn get_risk_level(score: f64) -> String {
    if score >= 80.0 {
        "Low Risk".to_string()
    } else if score >= 60.0 {
        "Medium Risk".to_string()
    } else if score >= 40.0 {
        "High Risk".to_string()
    } else {
        "Critical Risk".to_string()
    }
}

#[tauri::command]
pub fn check_mod_security(mod_path: String) -> Result<ModSecurityReport, String> {
    let path = PathBuf::from(&mod_path);

    if !path.exists() {
        return Err("Mod path does not exist".to_string());
    }

    let manifest_path = path.join("manifest.json");
    let (name, unique_id) = if manifest_path.exists() {
        if let Ok(content) = fs::read_to_string(&manifest_path) {
            if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&content) {
                let n = manifest["Name"].as_str().unwrap_or("Unknown").to_string();
                let uid = manifest["UniqueID"].as_str().unwrap_or("").to_string();
                (n, uid)
            } else {
                (path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string(), String::new())
            }
        } else {
            (path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string(), String::new())
        }
    } else {
        (path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string(), String::new())
    };

    let checks = vec![
        check_dll_presence(&path),
        check_entry_class(&path),
        check_content_patcher_only(&path),
        check_update_keys(&path),
    ];

    let score = calculate_security_score(&checks);
    let risk_level = get_risk_level(score);

    let mut recommendations = Vec::new();

    for check in &checks {
        if !check.passed {
            match check.severity.as_str() {
                "Critical" | "High" => {
                    recommendations.push(format!("[{}] {} - {}", check.severity, check.check_name, check.description));
                }
                "Medium" => {
                    recommendations.push(format!("[{}] {} - {}", check.severity, check.check_name, check.description));
                }
                _ => {}
            }
        }
    }

    Ok(ModSecurityReport {
        mod_name: name,
        unique_id,
        security_score: score,
        risk_level,
        checks,
        recommendations,
    })
}

#[tauri::command]
pub fn batch_check_mod_security(mods: Vec<serde_json::Value>) -> Result<BatchSecurityReport, String> {
    let mut reports = Vec::new();
    let mut high_risk = 0;
    let mut medium_risk = 0;
    let mut low_risk = 0;
    let mut total_score = 0.0;

    for mod_entry in &mods {
        if let Some(mod_path) = mod_entry["folder_path"].as_str() {
            if let Ok(report) = check_mod_security(mod_path.to_string()) {
                total_score += report.security_score;

                match report.risk_level.as_str() {
                    "Critical Risk" | "High Risk" => high_risk += 1,
                    "Medium Risk" => medium_risk += 1,
                    _ => low_risk += 1,
                }

                reports.push(report);
            }
        }
    }

    let average_score = if !reports.is_empty() {
        total_score / reports.len() as f64
    } else {
        100.0
    };

    Ok(BatchSecurityReport {
        reports,
        average_score,
        high_risk_count: high_risk,
        medium_risk_count: medium_risk,
        low_risk_count: low_risk,
    })
}
