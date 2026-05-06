use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use std::time::Instant;
use crate::log_parser::check_smapi_log;

static GAME_START_TIME: Mutex<Option<Instant>> = Mutex::new(None);

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
pub fn launch_game(game_path: String) -> Result<LaunchResult, String> {
    let smapi_path = find_smapi_exe(&game_path)?;

    let mut child = Command::new(&smapi_path)
        .current_dir(&game_path)
        .spawn()
        .map_err(|e| format!("Failed to launch game: {}", e))?;

    let _pid = child.id();

    std::thread::spawn(move || {
        let _ = child.wait();
        if let Ok(mut start) = GAME_START_TIME.lock() {
            *start = None;
        }
        let _ = check_smapi_log();
    });

    if let Ok(mut start) = GAME_START_TIME.lock() {
        *start = Some(Instant::now());
    }

    Ok(LaunchResult {
        success: true,
        message: "Game launched successfully".to_string(),
    })
}

#[tauri::command]
pub fn get_game_session_info() -> GameSessionInfo {
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
    if let Ok(start) = GAME_START_TIME.lock() {
        if start.is_some() {
            return Ok(true);
        }
    }
    Err("No game session found".to_string())
}
