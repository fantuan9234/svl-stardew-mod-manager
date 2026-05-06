use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

const THUMBNAIL_CACHE_DIR: &str = "thumbnail_cache";

static THUMBNAIL_CACHE_PATH: OnceLock<PathBuf> = OnceLock::new();

fn get_cache_dir() -> PathBuf {
    let mut path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    path.pop();
    path.push("assets");
    path.push(THUMBNAIL_CACHE_DIR);
    path
}

pub fn get_thumbnail_cache_path() -> &'static PathBuf {
    THUMBNAIL_CACHE_PATH.get_or_init(get_cache_dir)
}

pub fn get_cached_thumbnail_path(mod_unique_id: &str) -> Option<String> {
    let cache_dir = get_thumbnail_cache_path();
    if !cache_dir.exists() {
        return None;
    }

    let sanitized_id = sanitize_filename(mod_unique_id);
    let thumbnail_path = cache_dir.join(format!("{}.png", sanitized_id));

    if thumbnail_path.exists() {
        Some(thumbnail_path.to_string_lossy().to_string())
    } else {
        None
    }
}

pub async fn download_thumbnail(
    mod_unique_id: &str,
    nexus_mod_id: u64,
) -> Result<Option<String>, String> {
    if let Some(cached) = get_cached_thumbnail_path(mod_unique_id) {
        return Ok(Some(cached));
    }

    let cache_dir = get_thumbnail_cache_path();
    if !cache_dir.exists() {
        fs::create_dir_all(cache_dir)
            .map_err(|e| format!("创建缩略图缓存文件夹失败: {}", e))?;
    }

    let thumbnail_url = format!(
        "https://staticdelivery.nexusmods.com/mods/413/images/{}/{}-thumbnail-1280x1280.jpeg",
        nexus_mod_id, nexus_mod_id
    );

    let client = reqwest::Client::builder()
        .user_agent("SVL-Stardew-Valley-Launcher/1.0")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let response = client
        .get(&thumbnail_url)
        .send()
        .await
        .map_err(|e| format!("下载缩略图失败: {}", e))?;

    if !response.status().is_success() {
        let fallback_url = format!(
            "https://staticdelivery.nexusmods.com/mods/413/images/{}/{}-thumbnail-640x640.jpeg",
            nexus_mod_id, nexus_mod_id
        );

        let fallback_response = client
            .get(&fallback_url)
            .send()
            .await
            .map_err(|e| format!("下载备用缩略图失败: {}", e))?;

        if !fallback_response.status().is_success() {
            return Ok(None);
        }

        let bytes = fallback_response
            .bytes()
            .await
            .map_err(|e| format!("读取备用缩略图失败: {}", e))?;

        let sanitized_id = sanitize_filename(mod_unique_id);
        let thumbnail_path = cache_dir.join(format!("{}.jpeg", sanitized_id));

        fs::write(&thumbnail_path, &bytes)
            .map_err(|e| format!("写入缩略图失败: {}", e))?;

        return Ok(Some(thumbnail_path.to_string_lossy().to_string()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("读取缩略图失败: {}", e))?;

    let sanitized_id = sanitize_filename(mod_unique_id);
    let thumbnail_path = cache_dir.join(format!("{}.jpeg", sanitized_id));

    fs::write(&thumbnail_path, &bytes)
        .map_err(|e| format!("写入缩略图失败: {}", e))?;

    Ok(Some(thumbnail_path.to_string_lossy().to_string()))
}

pub fn cleanup_thumbnail_cache() -> Result<usize, String> {
    let cache_dir = get_thumbnail_cache_path();
    if !cache_dir.exists() {
        return Ok(0);
    }

    let mut count = 0;
    for entry in fs::read_dir(&cache_dir)
        .map_err(|e| format!("读取缓存文件夹失败: {}", e))?
    {
        if let Ok(entry) = entry {
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                if let Err(e) = fs::remove_file(entry.path()) {
                    eprintln!("[thumbnail_cache] Failed to remove {}: {}", entry.path().display(), e);
                } else {
                    count += 1;
                }
            }
        }
    }

    Ok(count)
}

pub fn get_thumbnail_cache_size() -> Result<u64, String> {
    let cache_dir = get_thumbnail_cache_path();
    if !cache_dir.exists() {
        return Ok(0);
    }

    let mut total_size = 0;
    for entry in fs::read_dir(&cache_dir)
        .map_err(|e| format!("读取缓存文件夹失败: {}", e))?
    {
        if let Ok(entry) = entry {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total_size += metadata.len();
                }
            }
        }
    }

    Ok(total_size)
}

fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == '.')
        .collect()
}

#[tauri::command]
pub async fn refresh_mod_thumbnail(
    mod_unique_id: String,
    nexus_mod_id: u64,
) -> Result<Option<String>, String> {
    download_thumbnail(&mod_unique_id, nexus_mod_id).await
}

#[tauri::command]
pub fn clear_thumbnail_cache() -> Result<usize, String> {
    cleanup_thumbnail_cache()
}

#[tauri::command]
pub fn get_thumbnail_cache_info() -> Result<serde_json::Value, String> {
    let size = get_thumbnail_cache_size()?;
    let cache_dir = get_thumbnail_cache_path();

    let file_count = if cache_dir.exists() {
        fs::read_dir(cache_dir)
            .map(|entries| entries.filter(|e| e.is_ok()).count())
            .unwrap_or(0)
    } else {
        0
    };

    Ok(serde_json::json!({
        "size": size,
        "sizeFormatted": format_size(size),
        "fileCount": file_count,
        "cachePath": cache_dir.to_string_lossy().to_string()
    }))
}

fn format_size(size: u64) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.2} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.2} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
