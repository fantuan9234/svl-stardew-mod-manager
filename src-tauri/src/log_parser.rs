use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::LazyLock;

use crate::mod_name_resolver::resolve_mod_name;

const TAIL_SIZE: u64 = 64 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedLogError {
    pub mod_name: String,
    pub error_type: String,
    pub raw_line: String,
    pub solution: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseSmapiLogResult {
    pub errors: Vec<ParsedLogError>,
    pub log_path: String,
    pub has_errors: bool,
    pub log_not_found: bool,
    pub smapi_not_installed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogError {
    pub raw_message: String,
    pub translated_message: String,
    pub severity: String,
    pub solution: String,
    pub solution_button_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogWarning {
    pub raw_message: String,
    pub translated_message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAnalysis {
    pub errors: Vec<LogError>,
    pub warnings: Vec<LogWarning>,
    pub error_count: usize,
    pub warning_count: usize,
    pub summary: String,
}

struct Rule {
    error_type: String,
    pattern: Regex,
    severity: String,
    extract: fn(&regex::Captures) -> (String, String),
}

static RULES: LazyLock<Vec<Rule>> = LazyLock::new(|| {
    vec![
        Rule {
            error_type: "MissingDependency".into(),
            pattern: Regex::new(r"(?i)(因为|because)\s*(it\s*)?(需要|needs|requires).*?mod\s*'([^']+)'").expect("Invalid regex: MissingDependency"),
            severity: "Error".into(),
            extract: |caps| {
                let missing_mod = caps.get(4).map(|m| m.as_str()).unwrap_or("未知MOD");
                (missing_mod.into(), format!("这个MOD需要 '{}' 才能运行，但你没有安装它。请去Nexus Mods搜索并下载 '{}'，然后放到 Mods 文件夹里。", missing_mod, missing_mod))
            },
        },
        Rule {
            error_type: "MissingDependency".into(),
            pattern: Regex::new(r"(?i)because\s+it\s+needs\s+the\s+'([^']+)'\s+mod").expect("Invalid regex: MissingDependency2"),
            severity: "Error".into(),
            extract: |caps| {
                let missing_mod = caps.get(1).map(|m| m.as_str()).unwrap_or("未知MOD");
                (missing_mod.into(), format!("这个MOD需要 '{}' 才能运行，但你没有安装它。请去Nexus Mods搜索并下载 '{}'，然后放到 Mods 文件夹里。", missing_mod, missing_mod))
            },
        },
        Rule {
            error_type: "MissingDependency".into(),
            pattern: Regex::new(r"(?i)because\s+it\s+needs\s+'([^']+)'").expect("Invalid regex: MissingDependency3"),
            severity: "Error".into(),
            extract: |caps| {
                let missing_mod = caps.get(1).map(|m| m.as_str()).unwrap_or("未知MOD");
                (missing_mod.into(), format!("这个MOD需要 '{}' 才能运行，但你没有安装它。请去Nexus Mods搜索并下载 '{}'，然后放到 Mods 文件夹里。", missing_mod, missing_mod))
            },
        },
        Rule {
            error_type: "MissingDll".into(),
            pattern: Regex::new(r"(?i)(because|由于)\s*(its\s+)?DLL\s*'([^']+)'").expect("Invalid regex: MissingDll"),
            severity: "Error".into(),
            extract: |caps| {
                let dll = caps.get(3).map(|m| m.as_str()).unwrap_or("未知.dll");
                let mod_name = extract_mod_from_dll(dll);
                (mod_name.clone(), format!("{} 的 .dll 文件 '{}' 不存在。这通常意味着MOD文件损坏或安装不完整。请重新下载并安装这个MOD。", mod_name, dll))
            },
        },
        Rule {
            error_type: "MissingDll".into(),
            pattern: Regex::new(r"(?i)because\s+its\s+DLL\s+'([^']+)'").expect("Invalid regex: MissingDll2"),
            severity: "Error".into(),
            extract: |caps| {
                let dll = caps.get(1).map(|m| m.as_str()).unwrap_or("未知.dll");
                let mod_name = extract_mod_from_dll(dll);
                (mod_name.clone(), format!("{} 的 .dll 文件 '{}' 不存在。这通常意味着MOD文件损坏或安装不完整。请重新下载并安装这个MOD。", mod_name, dll))
            },
        },
        Rule {
            error_type: "FailedLoading".into(),
            pattern: Regex::new(r"(?i)这些mod无法被添加到您的游戏中|could\s*not\s*be\s*added\s*to\s*your\s*game").expect("Invalid regex: FailedLoading"),
            severity: "Error".into(),
            extract: |_caps| {
                ("Unknown".into(), "有MOD无法加载。可能是缺少前置依赖、MOD文件损坏或版本不兼容。请查看日志中此行之上的详细错误信息。".into())
            },
        },
        Rule {
            error_type: "FailedLoading".into(),
            pattern: Regex::new(r"(?i)these\s+mods\s+could\s+not\s+be\s+added\s+to\s+your\s+game").expect("Invalid regex: FailedLoading2"),
            severity: "Error".into(),
            extract: |_caps| {
                ("Unknown".into(), "有MOD无法加载。可能是缺少前置依赖、MOD文件损坏或版本不兼容。请查看日志中此行之上的详细错误信息。".into())
            },
        },
        Rule {
            error_type: "UpdateAvailable".into(),
            pattern: Regex::new(r"(?i)(可以更新|can\s*update).*?(SMAPI|mod)\s*(to\s*)?([^:]+)").expect("Invalid regex: UpdateAvailable"),
            severity: "Warning".into(),
            extract: |caps| {
                let new_ver = caps.get(4).map(|m| m.as_str().trim()).unwrap_or("未知版本");
                let target = caps.get(2).map(|m| m.as_str()).unwrap_or("MOD");
                (target.into(), format!("{} 有可用更新（新版本: {}）。建议更新以获得最新功能和修复，避免兼容性问题。", target, new_ver))
            },
        },
        Rule {
            error_type: "UpdateAvailable".into(),
            pattern: Regex::new(r"(?i)you\s+can\s+update\s+(SMAPI|[^ ]+)").expect("Invalid regex: UpdateAvailable2"),
            severity: "Warning".into(),
            extract: |caps| {
                let target = caps.get(1).map(|m| m.as_str()).unwrap_or("MOD");
                (target.into(), format!("{} 有可用更新。建议前往官网或Nexus Mods下载最新版本，避免兼容性问题。", target))
            },
        },
        Rule {
            error_type: "ModuleError".into(),
            pattern: Regex::new(r"(?i)\[ERROR\].*?\|.*?\|(.+)").expect("Invalid regex: ModuleError"),
            severity: "Error".into(),
            extract: |caps| {
                let error_detail = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                let mod_name = extract_mod_from_error_line(error_detail);
                (mod_name.clone(), format!("{} 报告了一个错误。请查看详情：{}", mod_name, error_detail))
            },
        },
    ]
});

static ERROR_INDICATORS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(ERROR|FATAL|missing\s+dependen|failed\s+to\s+load|could\s+not|doesn't\s+exist|incompatible|exception|无法|缺少|不兼容|DLL)").expect("Invalid regex: error_indicators")
});

static WARN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\[WARN\]").expect("Invalid regex: warn_re")
});

static UPDATE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(可以更新|can\s+update|update\s+available).*?(\S+)").expect("Invalid regex: update_re")
});

static OBSOLETE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(obsolete|deprecated|过时)").expect("Invalid regex: obsolete_re")
});

fn extract_mod_from_dll(dll: &str) -> String {
    dll.trim_end_matches(".dll")
        .trim_end_matches(".Dll")
        .trim_end_matches(".DLL")
        .to_string()
}

fn extract_mod_from_error_line(detail: &str) -> String {
    if let Some(start) = detail.find('\'') {
        if let Some(end) = detail[start + 1..].find('\'') {
            return detail[start + 1..start + 1 + end].to_string();
        }
    }
    if let Some(start) = detail.find('"') {
        if let Some(end) = detail[start + 1..].find('"') {
            return detail[start + 1..start + 1 + end].to_string();
        }
    }
    let first_word = detail.split_whitespace().next().unwrap_or("Unknown");
    first_word.to_string()
}

fn extract_mod_name_from_line(line: &str) -> String {
    if let Some(start) = line.find('\'') {
        if let Some(end) = line[start + 1..].find('\'') {
            return line[start + 1..start + 1 + end].to_string();
        }
    }
    if let Some(start) = line.find('"') {
        if let Some(end) = line[start + 1..].find('"') {
            return line[start + 1..start + 1 + end].to_string();
        }
    }
    let re = Regex::new(r"(?i)\b(\w+\.\w+)\b").ok();
    if let Some(r) = re {
        if let Some(caps) = r.captures(line) {
            if let Some(m) = caps.get(1) {
                let name = m.as_str();
                if name.contains('.') && name.len() > 3 {
                    return name.to_string();
                }
            }
        }
    }
    "Unknown".to_string()
}

fn translate_mod_name(name: &str) -> String {
    if name.contains('.') && name.len() > 3 {
        let resolved = resolve_mod_name(name);
        if resolved != name {
            return resolved;
        }
    }
    name.to_string()
}

fn get_smapi_log_path() -> Option<PathBuf> {
    let roaming = dirs::data_dir()?;
    let log_dir = roaming.join("StardewValley").join("ErrorLogs");
    if !log_dir.exists() {
        return None;
    }

    let latest = log_dir.join("SMAPI-latest.txt");
    if latest.exists() {
        return Some(latest);
    }

    let crash = log_dir.join("SMAPI-crash.txt");
    if crash.exists() {
        return Some(crash);
    }

    let mut newest: Option<(PathBuf, std::time::SystemTime)> = None;
    if let Ok(entries) = fs::read_dir(&log_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("txt") {
                if let Ok(meta) = fs::metadata(&path) {
                    if let Ok(modified) = meta.modified() {
                        if newest.as_ref().map_or(true, |(_, t)| modified > *t) {
                            newest = Some((path, modified));
                        }
                    }
                }
            }
        }
    }

    newest.map(|(p, _)| p)
}

fn read_log_tail(path: &PathBuf) -> Result<String, String> {
    let file = fs::File::open(path).map_err(|e| format!("读取日志文件失败: {}", e))?;
    let metadata = file.metadata().map_err(|e| format!("获取文件信息失败: {}", e))?;
    let file_size = metadata.len();

    let reader = if file_size > TAIL_SIZE {
        let mut file = file;
        file.seek(SeekFrom::End(-(TAIL_SIZE as i64)))
            .map_err(|e| format!("定位文件失败: {}", e))?;
        BufReader::new(file)
    } else {
        BufReader::new(file)
    };

    let mut content = String::new();
    for line in reader.lines() {
        match line {
            Ok(l) => {
                content.push_str(&l);
                content.push('\n');
            }
            Err(_) => break,
        }
    }

    Ok(content)
}

fn match_rules(line: &str, rules: &[Rule]) -> Option<ParsedLogError> {
    for rule in rules {
        if let Some(caps) = rule.pattern.captures(line) {
            let (mod_name, solution) = (rule.extract)(&caps);
            let resolved = translate_mod_name(&mod_name);
            let final_solution = if resolved != mod_name {
                solution.replace(&mod_name, &resolved)
            } else {
                solution
            };
            return Some(ParsedLogError {
                mod_name: resolved,
                error_type: rule.error_type.clone(),
                raw_line: line.to_string(),
                solution: final_solution,
                severity: rule.severity.clone(),
            });
        }
    }
    None
}

fn parse_log_errors(content: &str) -> Vec<ParsedLogError> {
    let mut errors = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if !ERROR_INDICATORS.is_match(trimmed) {
            continue;
        }

        if seen.contains(trimmed) {
            continue;
        }

        if let Some(parsed) = match_rules(trimmed, &RULES) {
            seen.insert(trimmed.to_string());
            errors.push(parsed);
            continue;
        }

        seen.insert(trimmed.to_string());
        let mod_name = extract_mod_name_from_line(trimmed);
        let resolved = translate_mod_name(&mod_name);
        errors.push(ParsedLogError {
            mod_name: resolved,
            error_type: "UnknownError".into(),
            raw_line: trimmed.to_string(),
            solution: "此错误类型暂无法自动识别，建议查看 SMAPI 官方文档或社区寻求帮助。".into(),
            severity: "Warning".into(),
        });
    }

    errors
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmapiLogError {
    pub mod_name: String,
    pub error_type: String,
    pub original_line: String,
    pub solution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckSmapiLogResult {
    pub has_error: bool,
    pub errors: Vec<SmapiLogError>,
    pub error_count: usize,
}

#[tauri::command]
pub fn get_appdata_path() -> Result<String, String> {
    dirs::data_dir()
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| "无法获取 AppData 路径".to_string())
}

#[tauri::command]
pub fn open_path(path: String) -> Result<bool, String> {
    tauri_plugin_opener::open_path(&path, Option::<&str>::None)
        .map_err(|e| format!("无法打开路径: {}", e))?;
    Ok(true)
}

#[tauri::command]
pub fn check_smapi_log() -> Result<CheckSmapiLogResult, String> {
    let log_path = match get_smapi_log_path() {
        Some(p) => p,
        None => {
            return Ok(CheckSmapiLogResult {
                has_error: false,
                errors: vec![],
                error_count: 0,
            });
        }
    };

    if !log_path.exists() {
        return Ok(CheckSmapiLogResult {
            has_error: false,
            errors: vec![],
            error_count: 0,
        });
    }

    let content = match fs::read_to_string(&log_path) {
        Ok(c) => c,
        Err(_) => {
            return Ok(CheckSmapiLogResult {
                has_error: false,
                errors: vec![],
                error_count: 0,
            });
        }
    };

    if content.is_empty() {
        return Ok(CheckSmapiLogResult {
            has_error: false,
            errors: vec![],
            error_count: 0,
        });
    }

    let mut errors: Vec<SmapiLogError> = Vec::new();
    let mut has_error = false;
    let mut seen: HashSet<String> = HashSet::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if ERROR_INDICATORS.is_match(trimmed) {
            has_error = true;
        }

        if seen.contains(trimmed) {
            continue;
        }

        if let Some(parsed) = match_rules(trimmed, &RULES) {
            seen.insert(trimmed.to_string());
            errors.push(SmapiLogError {
                mod_name: parsed.mod_name,
                error_type: parsed.error_type,
                original_line: parsed.raw_line,
                solution: parsed.solution,
            });
        }
    }

    Ok(CheckSmapiLogResult {
        has_error,
        error_count: errors.len(),
        errors,
    })
}

#[tauri::command]
pub fn parse_smapi_log(log_path: Option<String>) -> Result<ParseSmapiLogResult, String> {
    let log_path = match log_path {
        Some(path) => PathBuf::from(path),
        None => match get_smapi_log_path() {
            Some(p) => p,
            None => {
                let roaming = dirs::data_dir();
                let smapi_dir = roaming.as_ref().map(|r| r.join("StardewValley").join("ErrorLogs"));
                let smapi_exists = smapi_dir.as_ref().map_or(false, |d| d.exists());

                if !smapi_exists {
                    return Ok(ParseSmapiLogResult {
                        errors: vec![],
                        log_path: String::new(),
                        has_errors: false,
                        log_not_found: true,
                        smapi_not_installed: true,
                    });
                }

                return Ok(ParseSmapiLogResult {
                    errors: vec![],
                    log_path: String::new(),
                    has_errors: false,
                    log_not_found: true,
                    smapi_not_installed: false,
                });
            }
        },
    };

    if !log_path.exists() {
        return Ok(ParseSmapiLogResult {
            errors: vec![],
            log_path: log_path.to_string_lossy().to_string(),
            has_errors: false,
            log_not_found: true,
            smapi_not_installed: false,
        });
    }

    let content = read_log_tail(&log_path)?;

    let errors = parse_log_errors(&content);

    Ok(ParseSmapiLogResult {
        has_errors: !errors.is_empty(),
        errors,
        log_path: log_path.to_string_lossy().to_string(),
        log_not_found: false,
        smapi_not_installed: false,
    })
}

#[tauri::command]
pub fn read_log_file(file_path: String) -> Result<String, String> {
    let path = PathBuf::from(&file_path);

    if !path.exists() {
        return Err(format!("日志文件不存在: {}", file_path));
    }

    fs::read_to_string(&path)
        .map_err(|e| format!("读取日志文件失败: {}", e))
}

#[tauri::command]
pub fn analyze_log(log_path: Option<String>) -> Result<LogAnalysis, String> {
    let log_path = match log_path {
        Some(path) => PathBuf::from(path),
        None => {
            let app_data = dirs::data_dir()
                .ok_or_else(|| "logParser.appDataNotFound".to_string())?;
            let smapi_log = app_data
                .join("StardewValley")
                .join("ErrorLogs")
                .join("SMAPI-latest.txt");

            if !smapi_log.exists() {
                return Err("logParser.logFileNotFound".to_string());
            }

            smapi_log
        }
    };

    if !log_path.exists() {
        return Err(format!("logParser.logFileNotExist|{}", log_path.display()));
    }

    let content = read_log_tail(&log_path)?;

    let errors = parse_errors_v2(&content);
    let warnings = parse_warnings_v2(&content);

    let error_count = errors.len();
    let warning_count = warnings.len();

    let summary = if error_count == 0 && warning_count == 0 {
        "logParser.noIssues".to_string()
    } else {
        format!("logParser.foundIssues|{}|{}", error_count, warning_count)
    };

    Ok(LogAnalysis {
        errors,
        warnings,
        error_count,
        warning_count,
        summary,
    })
}

fn parse_errors_v2(content: &str) -> Vec<LogError> {
    let mut errors = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || !ERROR_INDICATORS.is_match(trimmed) {
            continue;
        }

        if seen.contains(trimmed) {
            continue;
        }
        seen.insert(trimmed.to_string());

        let mut matched = false;
        for rule in RULES.iter() {
            if let Some(caps) = rule.pattern.captures(trimmed) {
                let (mod_name, solution) = (rule.extract)(&caps);
                let resolved = translate_mod_name(&mod_name);
                let final_solution = if resolved != mod_name {
                    solution.replace(&mod_name, &resolved)
                } else {
                    solution
                };
                errors.push(LogError {
                    raw_message: trimmed.to_string(),
                    translated_message: format!("{}: {}", rule.error_type, resolved),
                    severity: rule.severity.clone(),
                    solution: final_solution,
                    solution_button_text: "logParser.viewSolution".to_string(),
                });
                matched = true;
                break;
            }
        }

        if !matched {
            let mod_name = extract_mod_name_from_line(trimmed);
            let resolved = translate_mod_name(&mod_name);
            errors.push(LogError {
                raw_message: trimmed.to_string(),
                translated_message: format!("UnknownError: {}", resolved),
                severity: "Warning".to_string(),
                solution: "此错误类型暂无法自动识别，建议查看 SMAPI 官方文档或社区寻求帮助。".to_string(),
                solution_button_text: "logParser.viewRawLog".to_string(),
            });
        }
    }

    errors
}

fn parse_warnings_v2(content: &str) -> Vec<LogWarning> {
    let mut warnings = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || !WARN_RE.is_match(trimmed) {
            continue;
        }

        if seen.contains(trimmed) {
            continue;
        }
        seen.insert(trimmed.to_string());

        if let Some(caps) = UPDATE_RE.captures(trimmed) {
            let target = caps.get(2).map(|m| m.as_str()).unwrap_or("MOD");
            warnings.push(LogWarning {
                raw_message: trimmed.to_string(),
                translated_message: format!("UpdateAvailable: {}", target),
                suggestion: format!("{} 有可用更新。建议更新以获得最新功能和修复。", target),
            });
            continue;
        }

        if OBSOLETE_RE.is_match(trimmed) {
            let mod_name = extract_mod_name_from_line(trimmed);
            let resolved = translate_mod_name(&mod_name);
            warnings.push(LogWarning {
                raw_message: trimmed.to_string(),
                translated_message: format!("ObsoleteApi: {}", resolved),
                suggestion: format!("{} 使用了已过时的 SMAPI API，建议关注更新。", resolved),
            });
            continue;
        }

        warnings.push(LogWarning {
            raw_message: trimmed.to_string(),
            translated_message: "UnknownWarning".to_string(),
            suggestion: "此警告类型暂无法自动识别，通常不影响游戏运行。".to_string(),
        });
    }

    warnings
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtmLogEntry {
    pub raw_line: String,
    pub line_number: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtmLogAnalysis {
    pub log_path: String,
    pub error_lines: Vec<FtmLogEntry>,
    pub error_count: usize,
    pub core_reason: String,
    pub plain_explanation: String,
    pub suggested_action: String,
    pub has_ftm_errors: bool,
}

#[tauri::command]
pub fn analyze_ftm_errors() -> Result<FtmLogAnalysis, String> {
    let app_data = dirs::data_dir()
        .ok_or_else(|| "无法获取 AppData 路径".to_string())?;
    let smapi_log = app_data
        .join("StardewValley")
        .join("ErrorLogs")
        .join("SMAPI-latest.txt");

    if !smapi_log.exists() {
        return Ok(FtmLogAnalysis {
            log_path: smapi_log.to_string_lossy().to_string(),
            error_lines: vec![],
            error_count: 0,
            core_reason: String::new(),
            plain_explanation: "未找到 SMAPI 日志文件，请确保已安装并运行过 SMAPI。".to_string(),
            suggested_action: "启动一次带 MOD 的游戏，让 SMAPI 生成日志文件。".to_string(),
            has_ftm_errors: false,
        });
    }

    let content = fs::read_to_string(&smapi_log)
        .map_err(|e| format!("读取日志文件失败: {}", e))?;

    let ftm_patterns: Vec<Regex> = vec![
        Regex::new(r"(?i)\[FTM\]").unwrap(),
        Regex::new(r"(?i)\[Farm Type Manager\]").unwrap(),
        Regex::new(r"(?i)Farm Type Manager").unwrap(),
    ];

    let mut error_lines: Vec<FtmLogEntry> = Vec::new();
    let mut has_error = false;

    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let is_ftm_related = ftm_patterns.iter().any(|re| re.is_match(trimmed));
        if !is_ftm_related {
            continue;
        }

        let is_error_line = ERROR_INDICATORS.is_match(trimmed);
        if is_error_line {
            has_error = true;
            error_lines.push(FtmLogEntry {
                raw_line: trimmed.to_string(),
                line_number: idx + 1,
            });
        }
    }

    let (core_reason, plain_explanation, suggested_action) = if !has_error {
        (
            "无错误".to_string(),
            "日志中未发现 FTM (Farm Type Manager) 相关的错误信息。FTM 运行正常！".to_string(),
            "无需操作，FTM 当前没有报错。".to_string(),
        )
    } else {
        let combined: String = error_lines.iter()
            .map(|e| e.raw_line.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        analyze_ftm_error_content(&combined)
    };

    Ok(FtmLogAnalysis {
        log_path: smapi_log.to_string_lossy().to_string(),
        error_count: error_lines.len(),
        error_lines,
        core_reason,
        plain_explanation,
        suggested_action,
        has_ftm_errors: has_error,
    })
}

fn analyze_ftm_error_content(content: &str) -> (String, String, String) {
    let lower = content.to_lowercase();

    if lower.contains("could not load") || lower.contains("failed to load") || lower.contains("load failed") {
        if lower.contains("jsonassets") || lower.contains("ja ") || lower.contains("json assets") {
            return (
                "缺少前置 MOD：JsonAssets".to_string(),
                "FTM 需要 JsonAssets 这个前置 MOD 才能运行，但你的电脑上没有安装它。\n\n简单来说：FTM 就像一个需要电池才能工作的玩具，JsonAssets 就是那个电池。没有电池，玩具就动不了。".to_string(),
                "去 Nexus Mods 下载并安装 JsonAssets（UniqueID: spacechase0.JsonAssets，Nexus ID: 1720），安装后重启游戏即可。".to_string(),
            );
        }

        if lower.contains("content patcher") || lower.contains("contentpatcher") {
            return (
                "缺少前置 MOD：Content Patcher".to_string(),
                "FTM 需要 Content Patcher 这个前置 MOD，但你没装。\n\n就像 FTM 是一本中文书，Content Patcher 是翻译官。没有翻译官，FTM 看不懂游戏内容。".to_string(),
                "去 Nexus Mods 下载并安装 Content Patcher（UniqueID: Pathoschild.ContentPatcher，Nexus ID: 1915），安装后重启游戏。".to_string(),
            );
        }

        if lower.contains("spacecore") {
            return (
                "缺少前置 MOD：SpaceCore".to_string(),
                "FTM 依赖 SpaceCore 框架，但你的 MOD 列表里没有它。\n\nSpaceCore 是很多高级 MOD 的基础设施，就像盖房子需要地基一样。".to_string(),
                "去 Nexus Mods 下载并安装 SpaceCore（UniqueID: spacechase0.SpaceCore，Nexus ID: 1348）。".to_string(),
            );
        }

        return (
            "MOD 加载失败".to_string(),
            format!("FTM 在加载时遇到了问题。可能的原因有：\n1. MOD 文件损坏或不完整\n2. 缺少某个前置 MOD\n3. MOD 版本与当前 SMAPI/游戏版本不兼容\n\n原始报错信息中包含了具体原因。"),
            "1. 检查上方原始报错行，看是否提到了缺失的前置 MOD\n2. 尝试重新下载并安装 FTM\n3. 确认 FTM 版本支持你当前的游戏版本".to_string(),
        );
    }

    if lower.contains("missing") || lower.contains("not found") || lower.contains("does not exist") {
        if lower.contains("manifest") {
            return (
                "MOD 文件缺失：manifest.json".to_string(),
                "FTM 文件夹里缺少 manifest.json 这个关键文件。\n\nmanifest.json 就像 MOD 的身份证，SMAPI 靠它识别 MOD 的名称、版本和作者。没有它，SMAPI 就当这个 MOD 不存在。".to_string(),
                "重新下载 FTM 并确保完整解压。MOD 文件夹根目录下必须有 manifest.json 文件。".to_string(),
            );
        }

        if lower.contains("dll") || lower.contains("assembly") {
            return (
                "MOD 文件缺失：DLL 文件".to_string(),
                "FTM 缺少必要的 DLL 文件（程序运行库）。\n\nDLL 文件是 MOD 的核心代码部分。没有它，MOD 就是个空壳，无法执行任何功能。".to_string(),
                "重新下载 FTM 的完整版本，确保所有文件都正确解压到 MOD 文件夹中。".to_string(),
            );
        }

        return (
            "文件缺失".to_string(),
            format!("FTM 缺少某些必要的文件。原始报错信息中指出了具体缺失的文件。\n\n这通常是因为解压不完整，或者下载过程中文件丢失。"),
            "重新下载 FTM 并完整解压，确保 MOD 文件夹包含所有原始文件。".to_string(),
        );
    }

    if lower.contains("incompatible") || lower.contains("version") || lower.contains("api") {
        return (
            "版本不兼容".to_string(),
            "FTM 的版本与你当前的 SMAPI 或游戏版本不匹配。\n\n就像你拿着一把旧钥匙去开新锁，或者拿新钥匙开旧锁——对不上号。".to_string(),
            "1. 去 FTM 的 Nexus 页面下载最新版本\n2. 确认该版本支持你当前的 Stardew Valley 和 SMAPI 版本".to_string(),
        );
    }

    if lower.contains("exception") || lower.contains("error") || lower.contains("crash") {
        return (
            "运行时错误".to_string(),
            format!("FTM 在运行过程中发生了错误。这可能是：\n1. MOD 本身的 Bug\n2. 与其他 MOD 冲突\n3. 游戏数据异常\n\n具体原因需要看原始报错信息。"),
            "1. 查看上方原始报错行了解具体错误\n2. 尝试禁用其他 MOD，单独运行 FTM 排查冲突\n3. 更新 FTM 到最新版本".to_string(),
        );
    }

    (
        "FTM 报错".to_string(),
        format!("日志中发现了 FTM 相关的错误，但具体类型无法自动识别。\n\n原始报错信息如下：\n{}", content.chars().take(500).collect::<String>()),
        "建议：\n1. 将上方原始报错信息复制到 SMAPI 社区或 Discord 寻求帮助\n2. 检查 FTM 页面是否有已知问题\n3. 尝试重新安装 FTM".to_string(),
    )
}
