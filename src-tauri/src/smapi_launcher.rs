use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use std::time::Instant;
use tauri::Emitter;
use crate::log_parser::check_smapi_log;

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use windows_sys::Win32::Foundation::CloseHandle;
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{OpenProcess, WaitForSingleObject, PROCESS_QUERY_INFORMATION};

#[cfg(windows)]
const SYNCHRONIZE: u32 = 0x00100000;
#[cfg(windows)]
const WAIT_TIMEOUT: u32 = 0x00000102;

static GAME_START_TIME: Mutex<Option<Instant>> = Mutex::new(None);
static GAME_PROCESS_HANDLE: Mutex<Option<u32>> = Mutex::new(None);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSessionInfo {
    pub is_running: bool,
    pub pid: Option<u32>,
    pub start_time: Option<String>,
}

fn find_smapi_exe(game_path: &str) -> Result<PathBuf, String> {
    let game_dir = PathBuf::from(game_path);
    let smapi_paths = vec![
        game_dir.join("StardewModdingAPI.exe"),
        game_dir.join("smapi.exe"),
    ];
    for path in &smapi_paths {
        if path.exists() {
            return Ok(path.clone());
        }
    }
    Err("SMAPI executable not found".to_string())
}

#[tauri::command]
pub fn launch_game(game_path: String, app: tauri::AppHandle) -> Result<LaunchResult, String> {
    let smapi_path = find_smapi_exe(&game_path)?;
    let app_handle = app.clone();

    let mut child = Command::new(&smapi_path)
        .current_dir(&game_path)
        .creation_flags(if cfg!(windows) { 0x08000000 } else { 0 })
        .spawn()
        .map_err(|e| format!("Failed to launch game: {}", e))?;

    let pid = child.id();

    #[cfg(windows)]
    {
        if let Ok(mut handle_store) = GAME_PROCESS_HANDLE.lock() {
            *handle_store = Some(pid);
        }
    }

    std::thread::spawn(move || {
        let _ = child.wait();
        if let Ok(mut start) = GAME_START_TIME.lock() {
            *start = None;
        }
        #[cfg(windows)]
        {
            if let Ok(mut handle_store) = GAME_PROCESS_HANDLE.lock() {
                *handle_store = None;
            }
        }
        if let Ok(result) = check_smapi_log() {
            if result.has_error {
                let _ = app_handle.emit("game-exit-errors", serde_json::json!({
                    "has_errors": true,
                    "error_count": result.errors.len(),
                    "errors": result.errors,
                }));
            }
        }
    });

    if let Ok(mut start) = GAME_START_TIME.lock() {
        *start = Some(Instant::now());
    }

    Ok(LaunchResult {
        success: true,
        message: format!("Game launched successfully (PID: {})", pid),
    })
}

#[cfg(windows)]
fn is_process_running(pid: u32) -> bool {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | SYNCHRONIZE, 0, pid);
        if handle.is_null() {
            return false;
        }
        let result = WaitForSingleObject(handle, 0);
        CloseHandle(handle);
        result == WAIT_TIMEOUT
    }
}

#[cfg(not(windows))]
fn is_process_running(_pid: u32) -> bool {
    false
}

#[tauri::command]
pub fn get_game_session_info() -> GameSessionInfo {
    #[cfg(windows)]
    {
        let pid_to_check = {
            if let Ok(handle_store) = GAME_PROCESS_HANDLE.lock() {
                *handle_store
            } else {
                None
            }
        };

        if let Some(pid) = pid_to_check {
            if is_process_running(pid) {
                if let Ok(start) = GAME_START_TIME.lock() {
                    if let Some(_start_time) = *start {
                        // process is alive, return running
                    }
                }
                return GameSessionInfo {
                    is_running: true,
                    pid: Some(pid),
                    start_time: Some(format!("PID: {}", pid)),
                };
            } else {
                // Process is dead, clean up both states
                if let Ok(mut start) = GAME_START_TIME.lock() {
                    *start = None;
                }
                if let Ok(mut handle_store) = GAME_PROCESS_HANDLE.lock() {
                    *handle_store = None;
                }
            }
        }
    }

    if let Ok(start) = GAME_START_TIME.lock() {
        if let Some(start_time) = *start {
            let elapsed = start_time.elapsed();
            return GameSessionInfo {
                is_running: true,
                pid: None,
                start_time: Some(format!("{:.0}s ago", elapsed.as_secs_f64())),
            };
        }
    }
    GameSessionInfo {
        is_running: false,
        pid: None,
        start_time: None,
    }
}

#[tauri::command]
pub fn stop_game() -> Result<bool, String> {
    let pid = {
        if let Ok(handle) = GAME_PROCESS_HANDLE.lock() {
            *handle
        } else {
            return Err("无法获取进程信息".to_string());
        }
    };

    if let Some(pid) = pid {
        #[cfg(windows)]
        {
            let output = Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/F"])
                .creation_flags(0x08000000)
                .output()
                .map_err(|e| format!("执行 taskkill 失败: {}", e))?;

            if output.status.success() {
                if let Ok(mut start) = GAME_START_TIME.lock() {
                    *start = None;
                }
                if let Ok(mut handle) = GAME_PROCESS_HANDLE.lock() {
                    *handle = None;
                }
                Ok(true)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("not found") || stderr.contains("找不到") {
                    if let Ok(mut start) = GAME_START_TIME.lock() {
                        *start = None;
                    }
                    if let Ok(mut handle) = GAME_PROCESS_HANDLE.lock() {
                        *handle = None;
                    }
                    Ok(true)
                } else {
                    Err(format!("终止游戏进程失败: {}", stderr.trim()))
                }
            }
        }

        #[cfg(not(windows))]
        {
            let _ = pid;
            Err("当前平台不支持终止游戏进程".to_string())
        }
    } else {
        Err("没有正在运行的游戏进程".to_string())
    }
}
