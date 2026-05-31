use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use std::process::Command;
use std::sync::LazyLock;
use tauri::Emitter;

use crate::mod_name_resolver::resolve_mod_name;
use crate::smapi::find_game_path;
use crate::nexus_api::search_nexus_mods;

/// 系统环境检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCheck {
    pub dotnet_version: Option<String>,
    pub dotnet_installed: bool,
}

/// 自动修复结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixResult {
    pub total: usize,
    pub fixed: usize,
    pub failed: usize,
    pub details: Vec<FixDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixDetail {
    pub mod_name: String,
    pub error_type: String,
    pub action: String,
    pub success: bool,
    pub message: String,
}

const TAIL_SIZE: u64 = 64 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedLogError {
    pub mod_name: String,
    pub error_type: String,
    pub raw_line: String,
    pub solution: String,
    pub severity: String,
    #[serde(default)]
    pub missing_dep_id: Option<String>,
    #[serde(default)]
    pub missing_dep_name: Option<String>,
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
            error_type: "MissingDll".into(),
            pattern: Regex::new(r"(?i)因为.*?DLL\s*'([^']+)'").expect("Invalid regex: MissingDll3"),
            severity: "Error".into(),
            extract: |caps| {
                let dll = caps.get(1).map(|m| m.as_str()).unwrap_or("未知.dll");
                let mod_name = extract_mod_from_dll(dll);
                (mod_name.clone(), format!("{} 的 .dll 文件 '{}' 不存在。这通常意味着MOD文件损坏或安装不完整。请重新下载并安装这个MOD。", mod_name, dll))
            },
        },
        Rule {
            error_type: "DllLoadFailed".into(),
            pattern: Regex::new(r"(?i)(failed|无法|失败).*?加载.*?\.dll").expect("Invalid regex: DllLoadFailed"),
            severity: "Error".into(),
            extract: |caps| {
                let _ = caps;
                ("Unknown".into(), "MOD 的 DLL 文件加载失败。可能是文件损坏、版本不兼容或缺少运行库。请重新下载该 MOD，或安装 .NET Desktop Runtime。".into())
            },
        },
        Rule {
            error_type: "ManifestError".into(),
            pattern: Regex::new(r"(?i)(failed|无法|失败).*?读取.*?manifest").expect("Invalid regex: ManifestError"),
            severity: "Error".into(),
            extract: |_caps| {
                ("Unknown".into(), "无法读取 MOD 的 manifest.json 文件。该文件可能已损坏、格式错误或权限不足。请重新下载该 MOD。".into())
            },
        },
        Rule {
            error_type: "ManifestError".into(),
            pattern: Regex::new(r"(?i)manifest.*?(failed|无法|失败).*?读取").expect("Invalid regex: ManifestError2"),
            severity: "Error".into(),
            extract: |_caps| {
                ("Unknown".into(), "无法读取 MOD 的 manifest.json 文件。该文件可能已损坏、格式错误或权限不足。请重新下载该 MOD。".into())
            },
        },
        Rule {
            error_type: "CompatibilityError".into(),
            pattern: Regex::new(r"(?i)(不兼容|incompatible).*?game|game.*?(不兼容|incompatible)").expect("Invalid regex: CompatibilityError"),
            severity: "Error".into(),
            extract: |caps| {
                let _ = caps;
                ("Unknown".into(), "该 MOD 与当前游戏版本不兼容。请检查 MOD 页面是否支持你当前的星露谷物语版本，或等待 MOD 作者更新。".into())
            },
        },
        Rule {
            error_type: "VersionMismatch".into(),
            pattern: Regex::new(r"(?i)(version|版本).*?(不匹配|mismatch|不一致|不支持).*?(SMAPI|game|游戏)").expect("Invalid regex: VersionMismatch"),
            severity: "Error".into(),
            extract: |_caps| {
                ("Unknown".into(), "MOD 版本与 SMAPI 或游戏版本不匹配。请更新 SMAPI 到最新版本，或下载兼容你当前版本的 MOD。".into())
            },
        },
        Rule {
            error_type: "VersionMismatch".into(),
            pattern: Regex::new(r"(?i)(.+?)\s+\d+\.\d+.*?no\s+longer\s+compatible").expect("Invalid regex: VersionMismatch2"),
            severity: "Error".into(),
            extract: |caps| {
                let mod_name = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("Unknown").to_string();
                let solution = format!(
                    "{} 已不再兼容当前版本的 SMAPI。请按以下步骤操作：\n\
                    1. 前往 https://smapi.io/mods 搜索该 MOD，下载最新版本\n\
                    2. 如果找不到更新版本，尝试联系 MOD 作者\n\
                    3. 暂时从 Mods 文件夹移除该 MOD，等待更新",
                    mod_name
                );
                (mod_name.clone(), solution)
            },
        },
        Rule {
            error_type: "VersionMismatch".into(),
            pattern: Regex::new(r"(?i)(.+?)\s+\d+\.\d+.*?it's\s+no\s+longer\s+compatible").expect("Invalid regex: VersionMismatch3"),
            severity: "Error".into(),
            extract: |caps| {
                let mod_name = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("Unknown").to_string();
                let solution = format!(
                    "{} 已不再兼容当前版本的 SMAPI。请按以下步骤操作：\n\
                    1. 前往 https://smapi.io/mods 搜索该 MOD，下载最新版本\n\
                    2. 如果找不到更新版本，尝试联系 MOD 作者\n\
                    3. 暂时从 Mods 文件夹移除该 MOD，等待更新",
                    mod_name
                );
                (mod_name.clone(), solution)
            },
        },
        Rule {
            error_type: "DllLoadFailed".into(),
            pattern: Regex::new(r"(?i)^-\s+(.+?)\s+(\d+\.\d+[\d.]*)\s+because\s+its\s+DLL\s+(couldn't|could not)\s+be\s+loaded").expect("Invalid regex: SkippedModsDllFailed"),
            severity: "Error".into(),
            extract: |caps| {
                let mod_name = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("Unknown").to_string();
                let version = caps.get(2).map(|m| m.as_str()).unwrap_or("未知版本");
                let solution = format!(
                    "{} {} 的 DLL 文件无法被加载，已被 SMAPI 跳过。请按以下步骤排查：\n\
                    1. 前往 Nexus Mods 或 smapi.io/mods 搜索该 MOD，下载最新版本\n\
                    2. 确认 MOD 支持你当前的星露谷物语和 SMAPI 版本\n\
                    3. 安装最新的 .NET Desktop Runtime（下载：https://dotnet.microsoft.com/download/dotnet）\n\
                    4. 检查 MOD 文件夹中 .dll 文件是否存在且完整",
                    mod_name, version
                );
                (mod_name, solution)
            },
        },
        Rule {
            error_type: "DllLoadFailed".into(),
            pattern: Regex::new(r"(?i)(.+?)\s+\d+\.\d+.*?because\s+its\s+DLL\s+(couldn't|could not)\s+be\s+loaded").expect("Invalid regex: DllLoadFailed3"),
            severity: "Error".into(),
            extract: |caps| {
                let mod_name = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("Unknown").to_string();
                let solution = format!(
                    "{} 的 DLL 文件无法被加载。请按以下步骤排查：\n\
                    1. 重新下载该 MOD 的最新版本\n\
                    2. 确认 MOD 支持你当前的星露谷物语和 SMAPI 版本\n\
                    3. 安装最新的 .NET Desktop Runtime（下载：https://dotnet.microsoft.com/download/dotnet）\n\
                    4. 检查 MOD 文件夹中 .dll 文件是否存在且完整",
                    mod_name
                );
                (mod_name.clone(), solution)
            },
        },
        Rule {
            error_type: "DllLoadFailed".into(),
            pattern: Regex::new(r"(?i)Rewriting\s+(.+?)\.dll\s+failed").expect("Invalid regex: DllLoadFailed4"),
            severity: "Error".into(),
            extract: |caps| {
                let dll_name = caps.get(1).map(|m| m.as_str()).unwrap_or("Unknown").to_string();
                let solution = format!(
                    "SMAPI 尝试重写 {}.dll 文件失败。请按以下步骤操作：\n\
                    1. 前往 smapi.io/mods 搜索该 MOD，检查是否有新版本可用\n\
                    2. 如果没有更新，尝试联系 MOD 作者反馈兼容性问题\n\
                    3. 暂时禁用该 MOD，等待作者更新",
                    dll_name
                );
                (dll_name.clone(), solution)
            },
        },
        Rule {
            error_type: "SystemError".into(),
            pattern: Regex::new(r"(?i)System\.Exception").expect("Invalid regex: SystemError"),
            severity: "Error".into(),
            extract: |_caps| {
                ("Unknown".into(), "SMAPI 运行过程中发生系统级异常。请查看完整日志了解详细信息。".into())
            },
        },
        Rule {
            error_type: "AssemblyError".into(),
            pattern: Regex::new(r"(?i)Failed to resolve assembly.*?'([^']+)'").expect("Invalid regex: AssemblyError"),
            severity: "Error".into(),
            extract: |caps| {
                let assembly = caps.get(1).map(|m| m.as_str()).unwrap_or("未知程序集").to_string();
                let solution = format!(
                    "缺少 .NET 程序集依赖: {}。\n\
                    请按以下步骤解决：\n\
                    1. 下载并安装 .NET Desktop Runtime（推荐 .NET 8.0）：https://dotnet.microsoft.com/download/dotnet\n\
                    2. 安装后重启电脑\n\
                    3. 如果问题仍然存在，尝试更新该 MOD 到最新版本",
                    assembly
                );
                (assembly.clone(), solution)
            },
        },
        Rule {
            error_type: "NamespaceError".into(),
            pattern: Regex::new(r"(?i)无法识别.*?命名空间|namespace.*?(not found|不存在|未找到)").expect("Invalid regex: NamespaceError"),
            severity: "Error".into(),
            extract: |_caps| {
                ("Unknown".into(), "MOD 引用的命名空间不存在。可能是缺少前置 MOD 或 MOD 版本过旧。请检查 MOD 是否需要更新或安装额外的前置依赖。".into())
            },
        },
        Rule {
            error_type: "JsonParseError".into(),
            pattern: Regex::new(r"(?i)(JSON|json).*?(格式错误|parse error|解析失败|invalid)").expect("Invalid regex: JsonParseError"),
            severity: "Error".into(),
            extract: |_caps| {
                ("Unknown".into(), "MOD 的 JSON 配置文件格式有误。可能是文件损坏或编辑时出现了语法错误。请重新下载该 MOD 或检查 JSON 文件语法。".into())
            },
        },
        Rule {
            error_type: "FileNotFound".into(),
            pattern: Regex::new(r"(?i)(文件|file).*?(不存在|not found|找不到|缺失).*?\.json").expect("Invalid regex: FileNotFound"),
            severity: "Error".into(),
            extract: |_caps| {
                ("Unknown".into(), "MOD 所需的 JSON 文件不存在。可能是安装不完整或解压出错。请重新下载并安装该 MOD。".into())
            },
        },
        Rule {
            error_type: "ContentPatcherError".into(),
            pattern: Regex::new(r"(?i)\[Content Patcher\].*?(ERROR|error|错误)").expect("Invalid regex: ContentPatcherError"),
            severity: "Error".into(),
            extract: |_caps| {
                ("Unknown".into(), "Content Patcher 遇到错误。可能是 content.json 格式错误或路径配置有误。请检查使用该 MOD 的 Content Patcher 配置文件。".into())
            },
        },
        Rule {
            error_type: "GenericError".into(),
            pattern: Regex::new(r"(?i)\[ERROR\].*?mod.*?(failed|失败|错误)").expect("Invalid regex: GenericError"),
            severity: "Error".into(),
            extract: |_caps| {
                ("Unknown".into(), "MOD 运行过程中出现错误。请查看原始日志了解详细信息，或尝试重新安装该 MOD。".into())
            },
        },
        Rule {
            error_type: "GenericError".into(),
            pattern: Regex::new(r"(?i)\[ERROR\].*?mod.*?(error|错误|异常)").expect("Invalid regex: GenericError2"),
            severity: "Error".into(),
            extract: |_caps| {
                ("Unknown".into(), "MOD 运行过程中出现错误。请查看原始日志了解详细信息，或尝试重新安装该 MOD。".into())
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
    Regex::new(r"(?i)(\[FATAL|missing\s+dependen|failed\s+to\s+load|doesn't\s+exist|incompatible|no\s+longer\s+compatible|couldn't\s+be\s+loaded|could\s+not\s+be\s+loaded|these\s+mods\s+could\s+not\s+be\s+added|无法|缺少|不兼容|rewriting\s+.+?\.dll\s+failed|重写.+?\.dll\s+failed|failed\s+to\s+resolve\s+assembly|because\s+its\s+DLL|skipped\s+mods|dll\s+couldn't|patches\s+which\s+aren't\s+expected|because\s+(it\s+)?(needs|requires)|manifest.*?(failed|无法|错误|invalid)|json.*?(parse\s*error|格式错误|invalid|解析失败)|file.*?(not\s+found|不存在|找不到)|content\s+patcher.*?error|\[ERROR\].*?mod|\.dll\s+failed|no\s+longer\s+compatible|version.*?mismatch|系统异常|System\.Exception)").expect("Invalid regex: error_indicators")
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

#[allow(dead_code)]
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
                missing_dep_id: None,
                missing_dep_name: None,
            });
        }
    }
    None
}

#[allow(dead_code)]
fn parse_log_errors(content: &str) -> Vec<ParsedLogError> {
    let mut errors = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    let section_headers = ["skipped mods", "skipped mods:"];
    let log_prefix_re = Regex::new(r"^\[\d{2}:\d{2}:\d{2}\s+\w+\s+\w+\]\s*").expect("Invalid regex: log_prefix");

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if section_headers.iter().any(|h| trimmed.to_lowercase().ends_with(h)) {
            continue;
        }

        if trimmed.starts_with("--- End of") || trimmed.starts_with("--- end of") || (trimmed.starts_with("at ") && trimmed.contains('(')) || trimmed.starts_with("---\t") {
            continue;
        }

        if (trimmed.contains("[INFO") || trimmed.contains("[TRACE")) && !trimmed.contains("[ERROR") && !trimmed.contains("[FATAL") {
            continue;
        }

        if trimmed.contains("[ERROR game]") || trimmed.contains("[error game]") {
            continue;
        }

        let content_after_prefix = log_prefix_re.replace(trimmed, "");
        if content_after_prefix.chars().all(|c| c == '-' || c == '=' || c == ' ' || c == '─' || c == '·' || c == ':') && content_after_prefix.len() > 5 {
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
            missing_dep_id: None,
            missing_dep_name: None,
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
    #[serde(default)]
    pub missing_dep_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckSmapiLogResult {
    pub has_error: bool,
    pub errors: Vec<SmapiLogError>,
    pub error_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfoBasic {
    pub name: String,
    pub unique_id: String,
    pub version: String,
    pub folder_name: String,
    pub enabled: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModHealthIssue {
    pub mod_name: String,
    pub unique_id: String,
    pub issue_type: String,
    pub severity: String,
    pub issue_detail: String,
    pub fixable: bool,
    pub solution: String,
    pub missing_dep_name: Option<String>,
    pub missing_dep_id: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModHealthCheckResult {
    pub has_issues: bool,
    pub issues: Vec<ModHealthIssue>,
    pub issue_count: usize,
    pub total_mods: usize,
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

#[derive(Debug, Clone, Deserialize)]
struct SimpleManifestDep {
    #[serde(rename = "UniqueID", alias = "UniqueId")]
    unique_id: String,
    #[serde(rename = "IsRequired", default = "default_true")]
    is_required: bool,
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Deserialize)]
struct SimpleManifest {
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "UniqueID", alias = "UniqueId")]
    unique_id: Option<String>,
    #[serde(rename = "Version")]
    version: Option<String>,
    #[serde(rename = "Dependencies", default)]
    dependencies: Vec<SimpleManifestDep>,
}

fn scan_mods_basic(mods_path: &PathBuf) -> Vec<ModInfoBasic> {
    let mut mods = Vec::new();
    let mod_dirs = crate::mod_parser::recursive_find_manifests(&mods_path);
    for dir in &mod_dirs {
        let manifest_path = dir.join("manifest.json");
        let content = match fs::read_to_string(&manifest_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let normalized = crate::mod_parser::normalize_smart_quotes(&content);
        let no_comments = crate::mod_parser::strip_json_comments(&normalized);
        let cleaned = crate::mod_parser::remove_trailing_commas(&no_comments);
        let cleaned = cleaned.strip_prefix('\u{FEFF}').unwrap_or(&cleaned);
        let manifest: SimpleManifest = match serde_json::from_str(cleaned) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let folder_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        let enabled = !folder_name.starts_with('.') || folder_name.starts_with("..");
        mods.push(ModInfoBasic {
            name: manifest.name.unwrap_or_else(|| folder_name.clone()),
            unique_id: manifest.unique_id.unwrap_or_else(|| format!("unknown_{}", folder_name)),
            version: manifest.version.unwrap_or_else(|| "unknown".to_string()),
            folder_name,
            enabled,
        });
    }
    mods
}

#[tauri::command]
pub fn check_smapi_log() -> Result<CheckSmapiLogResult, String> {
    let (game_path, _) = crate::smapi::find_game_path()
        .ok_or_else(|| "无法找到星露谷物语安装路径".to_string())?;
    let mods_path = game_path.join("Mods");

    if !mods_path.exists() {
        return Ok(CheckSmapiLogResult {
            has_error: false,
            errors: vec![],
            error_count: 0,
        });
    }

    let installed_mods = scan_mods_basic(&mods_path);
    if installed_mods.is_empty() {
        return Ok(CheckSmapiLogResult {
            has_error: false,
            errors: vec![],
            error_count: 0,
        });
    }

    let enabled_unique_ids: HashSet<String> = installed_mods.iter()
        .filter(|m| m.enabled)
        .map(|m| m.unique_id.to_lowercase())
        .collect();

    let all_unique_ids_lower: HashSet<String> = installed_mods.iter()
        .map(|m| m.unique_id.to_lowercase())
        .collect();

    let mut errors: Vec<SmapiLogError> = Vec::new();

    let mod_dirs = crate::mod_parser::recursive_find_manifests(&mods_path);
    for dir in &mod_dirs {
        let manifest_path = dir.join("manifest.json");
        let content = match fs::read_to_string(&manifest_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let normalized = crate::mod_parser::normalize_smart_quotes(&content);
        let no_comments = crate::mod_parser::strip_json_comments(&normalized);
        let cleaned = crate::mod_parser::remove_trailing_commas(&no_comments);
        let cleaned = cleaned.strip_prefix('\u{FEFF}').unwrap_or(&cleaned);
        let manifest: SimpleManifest = match serde_json::from_str(cleaned) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let mod_name = manifest.name.clone().unwrap_or_else(|| {
            dir.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string()
        });
        let unique_id = manifest.unique_id.clone().unwrap_or_else(|| format!("unknown_{}", mod_name));
        let folder_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let mod_enabled = !folder_name.starts_with('.') || folder_name.starts_with("..");

        if mod_enabled {
            for dep in &manifest.dependencies {
                if !dep.is_required {
                    continue;
                }
                let dep_id_lower = dep.unique_id.to_lowercase();
                if !enabled_unique_ids.contains(&dep_id_lower) {
                    let dep_name = resolve_dep_name(&dep.unique_id, &installed_mods);
                    if all_unique_ids_lower.contains(&dep_id_lower) {
                        errors.push(SmapiLogError {
                            mod_name: mod_name.clone(),
                            error_type: "MissingDependency".into(),
                            original_line: String::new(),
                            solution: format!(
                                "{} 需要前置 MOD '{}'（UniqueID: {}），但它已被禁用。请在 MOD 管理器中启用该 MOD。",
                                mod_name, dep_name, dep.unique_id
                            ),
                            missing_dep_id: Some(dep.unique_id.clone()),
                        });
                    } else {
                        let nexus_hint = crate::smapi_data::get_mod_nexus_id(&dep.unique_id)
                            .map(|id| format!("\nNexus 链接: https://www.nexusmods.com/stardewvalley/mods/{}", id))
                            .unwrap_or_default();
                        errors.push(SmapiLogError {
                            mod_name: mod_name.clone(),
                            error_type: "MissingDependency".into(),
                            original_line: String::new(),
                            solution: format!(
                                "{} 需要前置 MOD '{}'（UniqueID: {}），但未安装。请去 Nexus Mods 搜索并下载此 MOD。{}",
                                mod_name, dep_name, dep.unique_id, nexus_hint
                            ),
                            missing_dep_id: Some(dep.unique_id.clone()),
                        });
                    }
                }
            }
        }

        let compat_meta = crate::compatibility_list::get_mod_metadata(&unique_id);
        if let Some(meta) = compat_meta {
            if let Some(ref status) = meta.status {
                match status.to_lowercase().as_str() {
                    "broken" | "incompatible" => {
                        let summary = meta.summary.as_deref().unwrap_or("该 MOD 与当前 SMAPI 版本不兼容");
                        errors.push(SmapiLogError {
                            mod_name: mod_name.clone(),
                            error_type: "BrokenMod".into(),
                            original_line: String::new(),
                            solution: format!(
                                "{} 已被 SMAPI 官方标记为不兼容。{}\n建议：检查 MOD 的 Nexus 页面是否有更新版本，或暂时禁用此 MOD。",
                                mod_name, summary
                            ),
                            missing_dep_id: None,
                        });
                    }
                    "abandoned" => {
                        errors.push(SmapiLogError {
                            mod_name: mod_name.clone(),
                            error_type: "AbandonedMod".into(),
                            original_line: String::new(),
                            solution: format!(
                                "{} 已被作者放弃维护，可能在未来的 SMAPI 版本中不再工作。建议寻找替代 MOD 或关注社区是否有接手维护的版本。",
                                mod_name
                            ),
                            missing_dep_id: None,
                        });
                    }
                    "obsolete" => {
                        errors.push(SmapiLogError {
                            mod_name: mod_name.clone(),
                            error_type: "ObsoleteMod".into(),
                            original_line: String::new(),
                            solution: format!(
                                "{} 已过时，不再需要。SMAPI 已内置了该 MOD 的功能，建议删除此 MOD。",
                                mod_name
                            ),
                            missing_dep_id: None,
                        });
                    }
                    "workaround" => {
                        errors.push(SmapiLogError {
                            mod_name: mod_name.clone(),
                            error_type: "NeedsWorkaround".into(),
                            original_line: String::new(),
                            solution: format!(
                                "{} 存在兼容性问题，但可以通过非官方补丁或设置解决。请查看 MOD 页面了解具体操作。",
                                mod_name
                            ),
                            missing_dep_id: None,
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    let has_error = !errors.is_empty();
    let error_count = errors.len();

    Ok(CheckSmapiLogResult {
        has_error,
        error_count,
        errors,
    })
}

fn resolve_dep_name(unique_id: &str, installed_mods: &[ModInfoBasic]) -> String {
    let id_lower = unique_id.to_lowercase();
    for m in installed_mods {
        if m.unique_id.to_lowercase() == id_lower {
            return m.name.clone();
        }
    }
    let compat_meta = crate::compatibility_list::get_mod_metadata(unique_id);
    if let Some(meta) = compat_meta {
        if !meta.name.is_empty() {
            return meta.name;
        }
    }
    let resolved = crate::mod_name_resolver::resolve_mod_name(unique_id);
    if resolved != unique_id {
        return resolved;
    }
    if let Some(pos) = unique_id.find('.') {
        let suffix = &unique_id[pos + 1..];
        if !suffix.is_empty() {
            let suffix_resolved = crate::mod_name_resolver::resolve_mod_name(suffix);
            if suffix_resolved != suffix {
                return suffix_resolved;
            }
            return add_spaces_to_camel_case(suffix);
        }
    }
    add_spaces_to_camel_case(unique_id)
}

fn add_spaces_to_camel_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 8);
    let chars: Vec<char> = s.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && c.is_uppercase() {
            let prev = chars[i - 1];
            let next_is_lower = chars.get(i + 1).map_or(true, |n| n.is_lowercase());
            if prev.is_lowercase() || (prev.is_uppercase() && next_is_lower) {
                result.push(' ');
            }
        }
        result.push(*c);
    }
    result
}

#[tauri::command]
pub fn parse_smapi_log(log_path: Option<String>) -> Result<ParseSmapiLogResult, String> {
    let mut all_errors: Vec<ParsedLogError> = Vec::new();
    let mut seen_keys: HashSet<String> = HashSet::new();
    let mut actual_log_path = String::new();
    let mut log_not_found = true;
    let mut smapi_not_installed = false;

    let resolved_log_path = log_path
        .filter(|p| !p.is_empty() && PathBuf::from(p).exists())
        .or_else(|| get_smapi_log_path().map(|p| p.to_string_lossy().to_string()));

    if let Some(ref path) = resolved_log_path {
        let pb = PathBuf::from(path);
        if pb.exists() {
            match read_log_tail(&pb) {
                Ok(content) => {
                    actual_log_path = path.clone();
                    log_not_found = false;

                    let log_errors = parse_errors_v2(&content);

                    for err in log_errors.iter() {
                        let key = format!("{}|{}|{}", err.severity, err.translated_message, err.raw_message);
                        if seen_keys.contains(&key) {
                            continue;
                        }
                        seen_keys.insert(key);

                        let missing_dep_id = if err.translated_message.starts_with("MissingDependency:") {
                            extract_unique_id_from_solution(&err.solution)
                        } else {
                            None
                        };

                        all_errors.push(ParsedLogError {
                            mod_name: extract_mod_name_from_translated(&err.translated_message),
                            error_type: extract_error_type_from_translated(&err.translated_message),
                            raw_line: err.raw_message.clone(),
                            solution: err.solution.clone(),
                            severity: err.severity.clone(),
                            missing_dep_id,
                            missing_dep_name: None,
                        });
                    }
                }
                Err(_) => {}
            }
        }
    }

    if let Some((game_path, _)) = crate::smapi::find_game_path() {
        let mods_path = game_path.join("Mods");
        if mods_path.exists() {
            let (manifest_errors, _) = run_mod_health_check(&mods_path);
            for err in manifest_errors {
                let key = format!("{}|{}|{}", err.error_type, err.mod_name, err.raw_line);
                if seen_keys.contains(&key) {
                    continue;
                }
                seen_keys.insert(key);
                all_errors.push(err);
            }
            if actual_log_path.is_empty() {
                actual_log_path = mods_path.to_string_lossy().to_string();
                log_not_found = false;
            }
        }
    } else {
        let roaming = dirs::data_dir();
        let smapi_dir = roaming.as_ref().map(|r| r.join("StardewValley").join("ErrorLogs"));
        smapi_not_installed = !smapi_dir.as_ref().map_or(false, |d| d.exists());
    }

    if actual_log_path.is_empty() {
        if let Some(lp) = get_smapi_log_path() {
            actual_log_path = lp.to_string_lossy().to_string();
        }
    }

    all_errors.retain(|e| {
        let combined = format!("{}|{}|{}", e.raw_line, e.error_type, e.solution);
        !combined.to_lowercase().contains("no update keys")
            && !combined.contains("这些mod没有更新键")
            && !combined.contains("这些 MOD 没有更新键")
    });

    let has_errors = !all_errors.is_empty();

    Ok(ParseSmapiLogResult {
        has_errors,
        errors: all_errors,
        log_path: actual_log_path,
        log_not_found,
        smapi_not_installed,
    })
}

fn extract_mod_name_from_translated(translated: &str) -> String {
    if let Some(pos) = translated.find(": ") {
        translated[pos + 2..].trim().to_string()
    } else {
        translated.to_string()
    }
}

fn extract_error_type_from_translated(translated: &str) -> String {
    if let Some(pos) = translated.find(": ") {
        translated[..pos].trim().to_string()
    } else {
        "UnknownError".to_string()
    }
}

fn extract_unique_id_from_solution(solution: &str) -> Option<String> {
    let re = Regex::new(r"UniqueID:\s*([^\),，）]+)").ok()?;
    let caps = re.captures(solution)?;
    caps.get(1).map(|m| m.as_str().trim().to_string())
}

fn run_mod_health_check(mods_path: &PathBuf) -> (Vec<ParsedLogError>, usize) {
    let installed_mods = scan_mods_basic(mods_path);
    if installed_mods.is_empty() {
        return (vec![], 0);
    }

    let enabled_unique_ids: HashSet<String> = installed_mods.iter()
        .filter(|m| m.enabled)
        .map(|m| m.unique_id.to_lowercase())
        .collect();

    let all_unique_ids_lower: HashSet<String> = installed_mods.iter()
        .map(|m| m.unique_id.to_lowercase())
        .collect();

    let mut errors: Vec<ParsedLogError> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    let mod_dirs = crate::mod_parser::recursive_find_manifests(mods_path);
    for dir in &mod_dirs {
        let manifest_path = dir.join("manifest.json");
        let content = match fs::read_to_string(&manifest_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let normalized = crate::mod_parser::normalize_smart_quotes(&content);
        let no_comments = crate::mod_parser::strip_json_comments(&normalized);
        let cleaned = crate::mod_parser::remove_trailing_commas(&no_comments);
        let cleaned = cleaned.strip_prefix('\u{FEFF}').unwrap_or(&cleaned);
        let manifest: SimpleManifest = match serde_json::from_str(cleaned) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let mod_name = manifest.name.clone().unwrap_or_else(|| {
            dir.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string()
        });
        let unique_id = manifest.unique_id.clone().unwrap_or_else(|| format!("unknown_{}", mod_name));
        let folder_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let mod_enabled = !folder_name.starts_with('.') || folder_name.starts_with("..");

        let uid_key = unique_id.to_lowercase();

        if mod_enabled {
            for dep in &manifest.dependencies {
                if !dep.is_required {
                    continue;
                }
                let dep_id_lower = dep.unique_id.to_lowercase();
                if !enabled_unique_ids.contains(&dep_id_lower) {
                    let dep_name = resolve_dep_name(&dep.unique_id, &installed_mods);
                    let dedup_key = format!("dep:{}:{}", uid_key, dep_id_lower);
                    if seen.contains(&dedup_key) {
                        continue;
                    }
                    seen.insert(dedup_key);

                    if all_unique_ids_lower.contains(&dep_id_lower) {
                        errors.push(ParsedLogError {
                            mod_name: mod_name.clone(),
                            error_type: "MissingDependency".into(),
                            raw_line: String::new(),
                            solution: format!(
                                "{} 需要前置 MOD '{}'（UniqueID: {}），但它已被禁用。请在 MOD 管理器中启用该 MOD。",
                                mod_name, dep_name, dep.unique_id
                            ),
                            severity: "Error".into(),
                            missing_dep_id: Some(dep.unique_id.clone()),
                            missing_dep_name: Some(dep_name.clone()),
                        });
                    } else {
                        let nexus_hint = crate::smapi_data::get_mod_nexus_id(&dep.unique_id)
                            .map(|id| format!("\nNexus 链接: https://www.nexusmods.com/stardewvalley/mods/{}", id))
                            .unwrap_or_default();
                        errors.push(ParsedLogError {
                            mod_name: mod_name.clone(),
                            error_type: "MissingDependency".into(),
                            raw_line: String::new(),
                            solution: format!(
                                "{} 需要前置 MOD '{}'（UniqueID: {}），但未安装。请去 Nexus Mods 搜索并下载此 MOD。{}",
                                mod_name, dep_name, dep.unique_id, nexus_hint
                            ),
                            severity: "Error".into(),
                            missing_dep_id: Some(dep.unique_id.clone()),
                            missing_dep_name: Some(dep_name.clone()),
                        });
                    }
                }
            }
        }

        let compat_key = format!("compat:{}", uid_key);
        if seen.contains(&compat_key) {
            continue;
        }

        let compat_meta = crate::compatibility_list::get_mod_metadata(&unique_id);
        if let Some(meta) = compat_meta {
            if let Some(ref status) = meta.status {
                match status.to_lowercase().as_str() {
                    "broken" | "incompatible" => {
                        seen.insert(compat_key);
                        let summary = meta.summary.as_deref().unwrap_or("该 MOD 与当前 SMAPI 版本不兼容");
                        errors.push(ParsedLogError {
                            mod_name: mod_name.clone(),
                            error_type: "BrokenMod".into(),
                            raw_line: String::new(),
                            solution: format!(
                                "{} 已被 SMAPI 官方标记为不兼容。{}\n建议：检查 MOD 的 Nexus 页面是否有更新版本，或暂时禁用此 MOD。",
                                mod_name, summary
                            ),
                            severity: "Error".into(),
                            missing_dep_id: None,
                            missing_dep_name: None,
                        });
                    }
                    "abandoned" => {
                        seen.insert(compat_key);
                        errors.push(ParsedLogError {
                            mod_name: mod_name.clone(),
                            error_type: "AbandonedMod".into(),
                            raw_line: String::new(),
                            solution: format!(
                                "{} 已被作者放弃维护，可能在未来的 SMAPI 版本中不再工作。",
                                mod_name
                            ),
                            severity: "Warning".into(),
                            missing_dep_id: None,
                            missing_dep_name: None,
                        });
                    }
                    "obsolete" => {
                        seen.insert(compat_key);
                        errors.push(ParsedLogError {
                            mod_name: mod_name.clone(),
                            error_type: "ObsoleteMod".into(),
                            raw_line: String::new(),
                            solution: format!(
                                "{} 已过时，不再需要。SMAPI 已内置了该 MOD 的功能，建议删除此 MOD。",
                                mod_name
                            ),
                            severity: "Warning".into(),
                            missing_dep_id: None,
                            missing_dep_name: None,
                        });
                    }
                    "workaround" => {
                        seen.insert(compat_key);
                        errors.push(ParsedLogError {
                            mod_name: mod_name.clone(),
                            error_type: "NeedsWorkaround".into(),
                            raw_line: String::new(),
                            solution: format!(
                                "{} 存在兼容性问题，但可以通过非官方补丁或设置解决。",
                                mod_name
                            ),
                            severity: "Warning".into(),
                            missing_dep_id: None,
                            missing_dep_name: None,
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    let count = errors.len();
    (errors, count)
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

    let section_headers = ["skipped mods", "skipped mods:"];
    let log_prefix_re = Regex::new(r"^\[\d{2}:\d{2}:\d{2}\s+\w+\s+\w+\]\s*").expect("Invalid regex: log_prefix3");

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.contains("DEBUG SMAPI") {
            continue;
        }

        if trimmed.contains("[ERROR game]") || trimmed.contains("[error game]") {
            continue;
        }

        if !ERROR_INDICATORS.is_match(trimmed) {
            continue;
        }

        if section_headers.iter().any(|h| trimmed.to_lowercase().ends_with(h)) {
            continue;
        }

        if trimmed.starts_with("--- End of") || trimmed.starts_with("--- end of") || (trimmed.starts_with("at ") && trimmed.contains('(')) {
            continue;
        }

        let content_after_prefix = log_prefix_re.replace(trimmed, "");
        if content_after_prefix.chars().all(|c| c == '-' || c == '=' || c == ' ' || c == '─' || c == '·' || c == ':') && content_after_prefix.len() > 5 {
            continue;
        }

        let body = content_after_prefix.trim();

        if seen.contains(body) {
            continue;
        }
        seen.insert(body.to_string());

        let mut matched = false;
        for rule in RULES.iter() {
            if let Some(caps) = rule.pattern.captures(body) {
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
            let mod_name = extract_mod_name_from_line(body);
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

#[allow(dead_code)]
fn parse_no_update_keys_section(content: &str) -> Vec<LogError> {
    let mut results = Vec::new();
    let log_prefix_re = Regex::new(r"^\[\d{2}:\d{2}:\d{2}\s+\w+\s+\w+\]\s*").expect("Invalid regex: log_prefix_nouk");
    let mut in_section = false;
    let mut mod_names: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if in_section && !mod_names.is_empty() {
                break;
            }
            continue;
        }

        let body: String = log_prefix_re.replace(trimmed, "").trim().to_string();

        if body.eq_ignore_ascii_case("No update keys") || body.eq_ignore_ascii_case("No update keys:") {
            in_section = true;
            continue;
        }

        if in_section {
            if body.starts_with("---") || body.chars().all(|c| c == '-' || c == '=' || c == ' ' || c == '─') {
                continue;
            }
            if body.starts_with("These mods have no update keys") || body.starts_with("mods. Consider") || body.starts_with("这些mod没有更新键") {
                continue;
            }
            if body.contains("DEBUG SMAPI") && !body.contains("No update keys") && !body.starts_with("-") {
                break;
            }
            if body.contains("INFO SMAPI") || body.contains("ERROR SMAPI") || body.contains("WARN SMAPI") || body.contains("TRACE SMAPI") {
                if !body.starts_with("-") {
                    break;
                }
            }
            if let Some(stripped) = body.strip_prefix("- ") {
                let mod_name = stripped.trim().to_string();
                if !mod_name.is_empty() {
                    mod_names.push(mod_name);
                }
            }
        }
    }

    if !mod_names.is_empty() {
        let mod_list = mod_names.join("、");
        for name in &mod_names {
            results.push(LogError {
                raw_message: format!("No update keys: {}", name),
                translated_message: format!("NoUpdateKeys: {}", name),
                severity: "Warning".to_string(),
                solution: format!(
                    "{} 的 manifest 中没有设置更新键（UpdateKeys），SMAPI 将无法自动通知该 MOD 的更新。建议手动关注该 MOD 的更新，或联系 MOD 作者添加 UpdateKeys。",
                    name
                ),
                solution_button_text: "logParser.viewSolution".to_string(),
            });
        }
        let _ = mod_list;
    }

    results
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

/// 检测系统 .NET 运行库状态
#[tauri::command]
pub fn check_dotnet_status() -> SystemCheck {
    #[cfg(target_os = "windows")]
    let output = {
        use std::os::windows::process::CommandExt;
        Command::new("dotnet")
            .arg("--list-runtimes")
            .creation_flags(0x08000000)
            .output()
    };
    #[cfg(not(target_os = "windows"))]
    let output = Command::new("dotnet")
        .arg("--list-runtimes")
        .output();

    match output {
        Ok(out) => {
            let output_str = String::from_utf8_lossy(&out.stdout);
            if output_str.contains("Microsoft.NETCore.App") || output_str.contains("Microsoft.WindowsDesktop.App") {
                let version = output_str.lines()
                    .find(|line| line.contains("Microsoft.NETCore.App") || line.contains("Microsoft.WindowsDesktop.App"))
                    .and_then(|line| line.split_whitespace().nth(1))
                    .map(|s| s.to_string());

                SystemCheck {
                    dotnet_version: version,
                    dotnet_installed: true,
                }
            } else {
                SystemCheck {
                    dotnet_version: None,
                    dotnet_installed: false,
                }
            }
        }
        Err(_) => SystemCheck {
            dotnet_version: None,
            dotnet_installed: false,
        },
    }
}

/// 自动修复所有日志错误
#[tauri::command]
pub async fn fix_all_log_errors(
    app: tauri::AppHandle,
    errors: Vec<ParsedLogError>,
    api_key: String,
) -> Result<FixResult, String> {
    eprintln!("[fix_all_log_errors] 开始自动修复 {} 个错误", errors.len());

    if api_key.is_empty() {
        return Err("请先在设置中绑定 Nexus API Key 以启用自动修复功能".into());
    }

    let (game_path, _) = find_game_path().ok_or("无法找到星露谷物语安装路径")?;
    let mods_path = game_path.join("Mods");
    
    if !mods_path.exists() {
        fs::create_dir_all(&mods_path).map_err(|e| format!("创建 Mods 文件夹失败: {}", e))?;
    }

    let mut details = Vec::new();
    let mut fixed = 0;

    for err in &errors {
        eprintln!("[fix_all_log_errors] 修复: {} ({})", err.mod_name, err.error_type);
        
        let _ = app.emit("fix-progress", serde_json::json!({
            "mod_name": err.mod_name,
            "error_type": err.error_type,
            "status": "processing"
        }));

        let mods_path_str = mods_path.to_string_lossy().into_owned();
        let result = fix_single_error(&app, err, &mods_path_str, &api_key).await;
        
        match result {
            Ok((action, message)) => {
                fixed += 1;
                details.push(FixDetail {
                    mod_name: err.mod_name.clone(),
                    error_type: err.error_type.clone(),
                    action,
                    success: true,
                    message: message.clone(),
                });
                let _ = app.emit("fix-progress", serde_json::json!({
                    "mod_name": err.mod_name,
                    "error_type": err.error_type,
                    "status": "success",
                    "message": message
                }));
            }
            Err(e) => {
                details.push(FixDetail {
                    mod_name: err.mod_name.clone(),
                    error_type: err.error_type.clone(),
                    action: "跳过".into(),
                    success: false,
                    message: e.clone(),
                });
                let _ = app.emit("fix-progress", serde_json::json!({
                    "mod_name": err.mod_name,
                    "error_type": err.error_type,
                    "status": "failed",
                    "message": e
                }));
            }
        }
    }

    eprintln!("[fix_all_log_errors] 修复完成: {}/{}", fixed, errors.len());
    Ok(FixResult {
        total: errors.len(),
        fixed,
        failed: errors.len() - fixed,
        details,
    })
}

/// 单独修复一个错误，返回 FixDetail 供前端展示
#[tauri::command]
pub async fn fix_single_log_error(
    app: tauri::AppHandle,
    error: ParsedLogError,
    api_key: String,
) -> Result<FixDetail, String> {
    if api_key.is_empty() {
        return Err("请先在设置中绑定 Nexus API Key".into());
    }

    let (game_path, _) = find_game_path().ok_or("无法找到星露谷物语安装路径")?;
    let mods_path = game_path.join("Mods");
    if !mods_path.exists() {
        fs::create_dir_all(&mods_path).map_err(|e| format!("创建 Mods 文件夹失败: {}", e))?;
    }

    let mods_path_str = mods_path.to_string_lossy().into_owned();
    match fix_single_error(&app, &error, &mods_path_str, &api_key).await {
        Ok((action, message)) => Ok(FixDetail {
            mod_name: error.mod_name.clone(),
            error_type: error.error_type.clone(),
            action,
            success: true,
            message,
        }),
        Err(e) => Err(e),
    }
}

/// 修复单个错误
async fn fix_single_error(
    app: &tauri::AppHandle,
    err: &ParsedLogError,
    mods_path: &str,
    api_key: &str,
) -> Result<(String, String), String> {
    match err.error_type.as_str() {
        "MissingDependency" | "MissingDll" => {
            let search_term = if let Some(ref dep_name) = err.missing_dep_name {
                dep_name.clone()
            } else if let Some(ref dep_id) = err.missing_dep_id {
                dep_id.clone()
            } else {
                err.mod_name.clone()
            };
            eprintln!("[fix_single_error] 下载缺失依赖: {} -> 搜索: {}", err.mod_name, search_term);
            download_and_install_mod(app, &search_term, mods_path, api_key).await
        }
        "VersionMismatch" | "DllLoadFailed" | "FailedLoading" | "BrokenMod" | "AbandonedMod" | "ObsoleteMod" | "NeedsWorkaround" => {
            let search_term = &err.mod_name;
            eprintln!("[fix_single_error] 搜索更新: {}", search_term);
            download_and_install_mod(app, search_term, mods_path, api_key).await
        }
        "AssemblyError" => {
            Err("此错误需要安装 .NET Desktop Runtime，请手动前往 dotnet.microsoft.com 下载安装".into())
        }
        _ => {
            Err(format!("暂不支持自动修复此类型的错误: {}", err.error_type))
        }
    }
}

async fn download_and_install_mod(
    app: &tauri::AppHandle,
    search_term: &str,
    mods_path: &str,
    api_key: &str,
) -> Result<(String, String), String> {
    eprintln!("[download_and_install_mod] 搜索: {}", search_term);

    let search_results = search_nexus_mods(search_term.to_string(), api_key.to_string(), 1, None).await?;
    if search_results.0.is_empty() {
        return Err(format!("在 Nexus 上未找到 '{}'", search_term));
    }

    let first_result = &search_results.0[0];
    let mod_id = &first_result.mod_id;
    
    eprintln!("[download_and_install_mod] 找到: {} (id={})", first_result.name, mod_id);

    let files = crate::nexus_api::get_nexus_mod_files(api_key.to_string(), mod_id.clone()).await?;
    if files.is_empty() {
        return Err(format!("MOD '{}' 没有可用的下载文件", first_result.name));
    }

    let target_file = files.into_iter()
        .filter(|f| !f.is_premium_only)
        .max_by_key(|f| f.upload_time.clone())
        .ok_or(format!("MOD '{}' 没有可用的免费文件", first_result.name))?;

    eprintln!("[download_and_install_mod] 下载: {} (file_id={})", target_file.name, target_file.file_id);

    let download_result = crate::nexus_api::download_mod_from_nexus(
        app.clone(),
        mod_id.clone(),
        api_key.to_string(),
        Some(mods_path.to_string()),
        Some(target_file.file_id),
        None,
    ).await?;

    if download_result.success {
        Ok(("自动下载并安装".into(), format!("已成功下载并安装 '{}' 的最新版本", first_result.name)))
    } else {
        Err(format!("下载失败: {}", download_result.message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_smapi_log() -> String {
        r#"[16:40:35 INFO  SMAPI] SMAPI 4.5.2 with Stardew Valley 1.6.15 build 24356 on Windows 10
[16:40:41 TRACE SMAPI]    UI Info Suite 2 (from Mods\UIInfoSuite2\UIInfoSuite2.dll, ID: Annosz.UiInfoSuite2, assembly version: 1.8.4)...
[16:40:42 INFO  SMAPI] Loaded 11 mods:
[16:40:42 ERROR SMAPI]    Skipped mods
[16:40:42 ERROR SMAPI]    --------------------------------------------------
[16:40:42 ERROR SMAPI]       These mods could not be added to your game.

[16:40:42 ERROR SMAPI]       - UI Info Suite 2 2.0.0 because its DLL couldn't be loaded.
[16:40:42 TRACE SMAPI]         (Error: System.Exception: Rewriting UIInfoSuite2.dll failed.
 ---> Mono.Cecil.AssemblyResolutionException: Failed to resolve assembly: 'System.Windows.Extensions, Version=0.0.0.0, Culture=neutral, PublicKeyToken=cc7b13ffcd2ddd51'
   at Mono.Cecil.BaseAssemblyResolver.Resolve(AssemblyNameReference name, ReaderParameters parameters)
   --- End of inner exception stack trace ---
   at StardewModdingAPI.Framework.ModLoading.AssemblyLoader.RewriteAssembly(IModMetadata mod, AssemblyDefinition assembly, HashSet`1 loggedMessages, String logPrefix)
   at StardewModdingAPI.Framework.SCore.TryLoadMod(IModMetadata mod, IModMetadata[] mods, AssemblyLoader assemblyLoader, IInterfaceProxyFactory proxyFactory, JsonHelper jsonHelper, ContentCoordinator contentCore, ModDatabase modDatabase, HashSet`1 suppressUpdateChecks, Nullable`1& failReason, String& errorReasonPhrase, String& errorDetails))

[16:40:42 DEBUG SMAPI]    No update keys
[16:40:42 DEBUG SMAPI]    --------------------------------------------------
[16:40:42 DEBUG SMAPI]       These mods have no update keys in their manifest. SMAPI may not notify you about updates for these
[16:40:42 DEBUG SMAPI]       mods. Consider notifying the mod authors about this problem.

[16:40:42 DEBUG SMAPI]       - Blackjack
[16:40:42 DEBUG SMAPI]       - NoThunderSound

[16:40:42 DEBUG SMAPI] Launching mods...
[16:40:42 INFO  SMAPI] Type 'help' for help, or 'help <cmd>' for a command's usage
[16:40:43 ERROR game] Oops! Steam achievements won't work because Steam isn't loaded.
"#.to_string()
    }

    #[test]
    fn test_parse_errors_v2_detects_skipped_mods_dll_failed() {
        let content = sample_smapi_log();
        let errors = parse_errors_v2(&content);

        let dll_failed: Vec<&LogError> = errors.iter()
            .filter(|e| e.translated_message.contains("DllLoadFailed"))
            .collect();
        assert!(!dll_failed.is_empty(), "Should detect DllLoadFailed for UI Info Suite 2");

        let has_ui_info = dll_failed.iter().any(|e| e.translated_message.contains("UI Info Suite 2"));
        assert!(has_ui_info, "Should extract 'UI Info Suite 2' as mod name, got: {:?}", dll_failed);
    }

    #[test]
    fn test_parse_errors_v2_detects_rewriting_dll_failed() {
        let content = sample_smapi_log();
        let errors = parse_errors_v2(&content);

        let rewriting: Vec<&LogError> = errors.iter()
            .filter(|e| e.raw_message.contains("Rewriting"))
            .collect();
        assert!(!rewriting.is_empty(), "Should detect Rewriting .dll failed");
    }

    #[test]
    fn test_parse_errors_v2_detects_assembly_error() {
        let content = sample_smapi_log();
        let errors = parse_errors_v2(&content);

        let assembly: Vec<&LogError> = errors.iter()
            .filter(|e| e.translated_message.contains("AssemblyError"))
            .collect();
        assert!(!assembly.is_empty(), "Should detect AssemblyError for System.Windows.Extensions");
    }

    #[test]
    fn test_parse_errors_v2_skips_game_errors() {
        let content = sample_smapi_log();
        let errors = parse_errors_v2(&content);

        let game_errors: Vec<&LogError> = errors.iter()
            .filter(|e| e.raw_message.contains("[ERROR game]"))
            .collect();
        assert!(game_errors.is_empty(), "Should skip [ERROR game] lines");
    }

    #[test]
    fn test_parse_no_update_keys_section() {
        let content = sample_smapi_log();
        let results = parse_no_update_keys_section(&content);

        assert!(!results.is_empty(), "Should detect No update keys section");

        let names: Vec<&str> = results.iter().map(|e| e.translated_message.as_str()).collect();
        assert!(names.iter().any(|n| n.contains("Blackjack")), "Should detect Blackjack, got: {:?}", names);
        assert!(names.iter().any(|n| n.contains("NoThunderSound")), "Should detect NoThunderSound, got: {:?}", names);
    }

    #[test]
    fn test_parse_errors_v2_strips_log_prefix() {
        let content = sample_smapi_log();
        let errors = parse_errors_v2(&content);

        for err in &errors {
            assert!(
                !err.translated_message.contains("[16:40:42"),
                "translated_message should not contain log prefix, got: {}",
                err.translated_message
            );
            assert!(
                !err.translated_message.contains("ERROR SMAPI"),
                "translated_message should not contain ERROR SMAPI prefix, got: {}",
                err.translated_message
            );
        }
    }

    #[test]
    fn test_parse_errors_v2_skips_debug_no_update_keys() {
        let content = r#"[16:40:42 DEBUG SMAPI]    No update keys
[16:40:42 DEBUG SMAPI]    --------------------------------------------------
[16:40:42 DEBUG SMAPI]       These mods have no update keys in their manifest. SMAPI may not notify you about updates for these
[16:40:42 DEBUG SMAPI]       mods. Consider notifying the mod authors about this problem.
[16:40:42 DEBUG SMAPI]       - Blackjack
[16:40:42 DEBUG SMAPI]       - NoThunderSound
"#;
        let errors = parse_errors_v2(content);
        assert!(errors.is_empty(), "DEBUG SMAPI 'no update keys' logs should not be treated as errors, got: {:?}", errors);
    }

    #[test]
    fn test_skipped_mods_rule_extracts_clean_name() {
        let line = "- UI Info Suite 2 2.0.0 because its DLL couldn't be loaded.";
        let rule = RULES.iter().find(|r| {
            r.error_type == "DllLoadFailed" && r.pattern.is_match(line)
        });
        assert!(rule.is_some(), "A DllLoadFailed rule should match the Skipped mods line");

        if let Some(r) = rule {
            let caps = r.pattern.captures(line);
            assert!(caps.is_some(), "Should match the line");
            if let Some(c) = caps {
                let (mod_name, _solution) = (r.extract)(&c);
                assert_eq!(mod_name, "UI Info Suite 2", "Should extract clean mod name");
            }
        }
    }

    #[test]
    fn test_add_spaces_to_camel_case_basic() {
        assert_eq!(add_spaces_to_camel_case("QuestFramework"), "Quest Framework");
        assert_eq!(add_spaces_to_camel_case("SpaceCore"), "Space Core");
        assert_eq!(add_spaces_to_camel_case("ContentPatcher"), "Content Patcher");
    }

    #[test]
    fn test_add_spaces_to_camel_case_consecutive_uppercase() {
        assert_eq!(add_spaces_to_camel_case("NPCAdventures"), "NPC Adventures");
        assert_eq!(add_spaces_to_camel_case("SVECode"), "SVE Code");
        assert_eq!(add_spaces_to_camel_case("NPCMapLocations"), "NPC Map Locations");
    }

    #[test]
    fn test_add_spaces_to_camel_case_single_word() {
        assert_eq!(add_spaces_to_camel_case("Climbing"), "Climbing");
        assert_eq!(add_spaces_to_camel_case("Automate"), "Automate");
    }

    #[test]
    fn test_resolve_dep_name_strips_author_prefix() {
        let installed: Vec<ModInfoBasic> = vec![];
        assert_eq!(resolve_dep_name("PurplingCat.QuestFramework", &installed), "Quest Framework");
        assert_eq!(resolve_dep_name("spacechase0.SpaceCore", &installed), "Space Core");
        assert_eq!(resolve_dep_name("Pathoschild.ContentPatcher", &installed), "Content Patcher");
    }

    #[test]
    fn test_resolve_dep_name_consecutive_uppercase() {
        let installed: Vec<ModInfoBasic> = vec![];
        assert_eq!(resolve_dep_name("PurplingCat.NPCAdventures", &installed), "NPC Adventures");
    }

    #[test]
    fn test_resolve_dep_name_no_prefix() {
        let installed: Vec<ModInfoBasic> = vec![];
        assert_eq!(resolve_dep_name("SomeMod", &installed), "Some Mod");
    }

    #[test]
    fn test_scan_mods_basic_with_smart_quotes() {
        let tmp = tempfile::tempdir().unwrap();
        let mod_dir = tmp.path().join("TestMod");
        std::fs::create_dir_all(&mod_dir).unwrap();

        let smart_quote_manifest = format!("{{
            \u{201C}Name\u{201D}: \u{201C}Test Mod\u{201D},
            \u{201C}UniqueID\u{201D}: \u{201C}Test.SmartQuotes\u{201D},
            \u{201C}Version\u{201D}: \u{201C}1.0.0\u{201D}
        }}");
        let manifest_path = mod_dir.join("manifest.json");
        std::fs::write(&manifest_path, &smart_quote_manifest).unwrap();

        let mods = scan_mods_basic(&PathBuf::from(tmp.path()));
        assert_eq!(mods.len(), 1, "Should parse manifest with smart quotes");
        assert_eq!(mods[0].unique_id, "Test.SmartQuotes");
        assert_eq!(mods[0].name, "Test Mod");
    }

    #[test]
    fn test_scan_mods_basic_with_bom() {
        let tmp = tempfile::tempdir().unwrap();
        let mod_dir = tmp.path().join("BomMod");
        std::fs::create_dir_all(&mod_dir).unwrap();

        let bom = "\u{FEFF}";
        let manifest_with_bom = format!("{}{{
            \"Name\": \"BOM Mod\",
            \"UniqueID\": \"Test.BOMMod\",
            \"Version\": \"1.0.0\"
        }}", bom);
        let manifest_path = mod_dir.join("manifest.json");
        std::fs::write(&manifest_path, &manifest_with_bom).unwrap();

        let mods = scan_mods_basic(&PathBuf::from(tmp.path()));
        assert_eq!(mods.len(), 1, "Should parse manifest with BOM prefix");
        assert_eq!(mods[0].unique_id, "Test.BOMMod");
    }

    #[test]
    fn test_parse_errors_v2_filters_game_errors() {
        let log = r#"[16:40:43 ERROR game] Oops! Steam achievements won't work because Steam isn't loaded.
[16:40:42 ERROR SMAPI] UI Info Suite 2 2.0.0 because its DLL couldn't be loaded.
"#;
        let errors = parse_errors_v2(log);
        let game_errors: Vec<&LogError> = errors.iter()
            .filter(|e| e.raw_message.contains("[ERROR game]"))
            .collect();
        assert!(game_errors.is_empty(), "Should filter out [ERROR game] lines, but found: {:?}", game_errors);
    }

    #[test]
    fn test_parse_errors_v2_detects_missing_dependency_mod_format() {
        let log = r#"[16:40:42 ERROR SMAPI] - CJB Show Item Code And Category Menu 1.0.0 because it needs the 'CJBCheatsMenu' mod
"#;
        let errors = parse_errors_v2(log);
        assert!(!errors.is_empty(), "Should detect missing dependency");
        let found = errors.iter().any(|e| e.translated_message.contains("MissingDependency"));
        assert!(found, "Should classify as MissingDependency, got: {:?}", errors);
    }

    #[test]
    fn test_parse_errors_v2_detects_no_longer_compatible() {
        let log = r#"[16:40:42 ERROR SMAPI] - CJB Item Spawner 2.0.0 because it's no longer compatible
"#;
        let errors = parse_errors_v2(log);
        assert!(!errors.is_empty(), "Should detect no longer compatible");
        let found = errors.iter().any(|e| e.translated_message.contains("VersionMismatch"));
        assert!(found, "Should classify as VersionMismatch, got: {:?}", errors);
    }

    #[test]
    fn test_parse_errors_v2_detects_skipped_mods_dll_format() {
        let log = r#"[16:40:42 ERROR SMAPI] - UI Info Suite 2 2.0.0 because its DLL couldn't be loaded.
"#;
        let errors = parse_errors_v2(log);
        assert!(!errors.is_empty(), "Should detect DLL load failure");
        let found = errors.iter().any(|e| e.translated_message.contains("DllLoadFailed"));
        assert!(found, "Should classify as DllLoadFailed, got: {:?}", errors);
    }

    #[test]
    fn test_parse_errors_v2_skips_debug_lines() {
        let log = r#"[16:40:42 DEBUG SMAPI]    No update keys
[16:40:42 DEBUG SMAPI]    - Blackjack
"#;
        let errors = parse_errors_v2(log);
        assert!(errors.is_empty(), "Should skip DEBUG SMAPI lines, got: {:?}", errors);
    }

    #[test]
    fn test_parse_errors_v2_detects_assembly_resolve_error() {
        let log = r#"[16:40:42 ERROR SMAPI] Failed to resolve assembly: 'System.Windows.Extensions, Version=0.0.0.0'
"#;
        let errors = parse_errors_v2(log);
        assert!(!errors.is_empty(), "Should detect assembly resolve error");
        let found = errors.iter().any(|e| e.translated_message.contains("AssemblyError"));
        assert!(found, "Should classify as AssemblyError, got: {:?}", errors);
    }

    #[test]
    fn test_parse_errors_v2_rewriting_dll_failed_rule() {
        let log = r#"[16:40:42 ERROR SMAPI] Rewriting UIInfoSuite2.dll failed
"#;
        let errors = parse_errors_v2(log);
        assert!(!errors.is_empty(), "Should detect rewriting dll failed");
        let found = errors.iter().any(|e| e.translated_message.contains("DllLoadFailed"));
        assert!(found, "Should classify as DllLoadFailed, got: {:?}", errors);
    }
}
