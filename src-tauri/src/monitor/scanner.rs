//! 会话文件扫描器
//!
//! 负责扫描 ~/.claude/projects/ 目录，查找并提取 Claude Code 会话元数据

use std::path::{Path, PathBuf};
use std::time::{SystemTime, Duration};
use anyhow::Result;
use glob::glob;
use dirs::home_dir;

/// 会话元数据结构
#[derive(Debug, Clone)]
pub struct SessionMetadata {
    /// 会话唯一标识 (UUID)
    pub session_id: String,
    /// 项目路径
    pub project_path: String,
    /// 项目名称
    pub project_name: String,
    /// JSONL 文件完整路径
    pub file_path: PathBuf,
    /// 创建时间 (RFC3339)
    pub created_at: String,
    /// 最后更新时间 (RFC3339)
    pub updated_at: String,
    /// 消息数量
    pub message_count: usize,
    /// 是否活跃
    pub is_active: bool,
}

/// 获取 Claude Code 项目目录
///
/// # 返回
/// 返回 ~/.claude/projects/ 目录的完整路径
pub fn get_claude_projects_dir() -> Result<PathBuf> {
    let home = home_dir().ok_or_else(|| anyhow::anyhow!("无法获取用户目录"))?;
    Ok(home.join(".claude").join("projects"))
}

/// 提取会话元数据
///
/// # 参数
/// - `path`: JSONL 文件路径
///
/// # 返回
/// 返回会话元数据
pub fn extract_session_metadata(path: &Path) -> Result<SessionMetadata> {
    // 1. 从文件名提取 session_id (UUID 格式)
    let file_name = path.file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("无效的文件名: {:?}", path))?;

    // 验证是否为有效的 UUID 格式
    if !is_valid_uuid(file_name) {
        return Err(anyhow::anyhow!("无效的 UUID 格式: {}", file_name));
    }

    let session_id = file_name.to_string();

    // 2. 从文件路径提取 project_path 和 project_name
    let full_path = path.canonicalize()?;
    let path_str = full_path.to_string_lossy().to_string();

    // 查找 .claude/projects/ 后面的部分
    let projects_dir = get_claude_projects_dir()?;
    let projects_str = projects_dir.to_string_lossy().to_string();

    let project_path = if path_str.starts_with(&projects_str) {
        // 提取项目相对路径
        let relative = path_str[projects_str.len()..].trim_start_matches('/');
        // 获取项目根目录（包含会话文件的目录）
        if let Some(parent) = Path::new(relative).parent() {
            parent.to_string_lossy().to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // 提取项目名称（路径的最后一段）
    let project_name = Path::new(&project_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    // 3. 读取文件元数据
    let metadata = std::fs::metadata(path)?;
    let created = metadata.created()
        .or_else(|_| metadata.modified())?;
    let modified = metadata.modified()?;

    // 转换为 RFC3339 格式
    let created_at = system_time_to_rfc3339(created)?;
    let updated_at = system_time_to_rfc3339(modified)?;

    // 4. 计算消息数量（简单方法：计算 JSONL 文件的行数）
    let message_count = count_jsonl_lines(path)?;

    // 5. 判断是否活跃
    let is_active = is_session_active(path);

    Ok(SessionMetadata {
        session_id,
        project_path,
        project_name,
        file_path: path.to_path_buf(),
        created_at,
        updated_at,
        message_count,
        is_active,
    })
}

/// 扫描所有会话文件
///
/// # 返回
/// 返回扫描到的所有会话元数据列表
pub fn scan_session_files() -> Result<Vec<SessionMetadata>> {
    let projects_dir = match get_claude_projects_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("警告: 无法获取 Claude 项目目录: {}", e);
            return Ok(Vec::new());
        }
    };

    scan_directory(&projects_dir)
}

/// 扫描指定目录的会话文件
///
/// # 参数
/// - `directory`: 要扫描的目录路径
///
/// # 返回
/// 返回扫描到的所有会话元数据列表
pub fn scan_directory(directory: &Path) -> Result<Vec<SessionMetadata>> {
    // 如果目录不存在，返回空列表
    if !directory.exists() {
        eprintln!("警告: 目录不存在: {:?}", directory);
        return Ok(Vec::new());
    }

    // 使用 glob 查找所有 JSONL 文件
    let pattern = directory.join("**").join("*.jsonl");
    let pattern_str = pattern.to_str()
        .ok_or_else(|| anyhow::anyhow!("无效的路径模式"))?;

    let mut sessions = Vec::new();

    for entry in glob(pattern_str)? {
        match entry {
            Ok(path) => {
                match extract_session_metadata_from_dir(&path, directory) {
                    Ok(metadata) => sessions.push(metadata),
                    Err(e) => {
                        // 跳过损坏的文件，记录到日志
                        eprintln!("跳过损坏的文件 {:?}: {}", path, e);
                    }
                }
            }
            Err(e) => {
                eprintln!(" glob 错误: {}", e);
            }
        }
    }

    Ok(sessions)
}

/// 从指定基础目录提取会话元数据
fn extract_session_metadata_from_dir(path: &Path, base_dir: &Path) -> Result<SessionMetadata> {
    // 1. 从文件名提取 session_id (UUID 格式)
    let file_name = path.file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("无效的文件名: {:?}", path))?;

    // 验证是否为有效的 UUID 格式
    if !is_valid_uuid(file_name) {
        return Err(anyhow::anyhow!("无效的 UUID 格式: {}", file_name));
    }

    let session_id = file_name.to_string();

    // 2. 从文件路径提取 project_path 和 project_name
    let full_path = path.canonicalize()?;
    let base_path = base_dir.canonicalize()?;
    let path_str = full_path.to_string_lossy().to_string();
    let base_str = base_path.to_string_lossy().to_string();

    let project_path = if path_str.starts_with(&base_str) {
        // 提取项目相对路径
        let relative = path_str[base_str.len()..].trim_start_matches('\\').trim_start_matches('/');
        // 获取项目根目录（包含会话文件的目录）
        if let Some(parent) = Path::new(relative).parent() {
            parent.to_string_lossy().to_string()
        } else {
            String::new()
        }
    } else {
        // 使用完整路径的父目录
        path.parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    };

    // 提取项目名称（路径的最后一段）
    let project_name = if project_path.is_empty() {
        base_dir.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string()
    } else {
        Path::new(&project_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string()
    };

    // 3. 读取文件元数据
    let metadata = std::fs::metadata(path)?;
    let created = metadata.created()
        .or_else(|_| metadata.modified())?;
    let modified = metadata.modified()?;

    // 转换为 RFC3339 格式
    let created_at = system_time_to_rfc3339(created)?;
    let updated_at = system_time_to_rfc3339(modified)?;

    // 4. 计算消息数量（简单方法：计算 JSONL 文件的行数）
    let message_count = count_jsonl_lines(path)?;

    // 5. 判断是否活跃
    let is_active = is_session_active(path);

    Ok(SessionMetadata {
        session_id,
        project_path,
        project_name,
        file_path: path.to_path_buf(),
        created_at,
        updated_at,
        message_count,
        is_active,
    })
}

/// 检查是否为有效的 UUID 格式
fn is_valid_uuid(s: &str) -> bool {
    // 简单的 UUID 格式验证 (xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx)
    let uuid_pattern = regex::Regex::new(
        r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    ).unwrap();

    uuid_pattern.is_match(s)
}

/// 将 SystemTime 转换为 RFC3339 格式字符串
fn system_time_to_rfc3339(time: SystemTime) -> Result<String> {
    let duration = time.duration_since(SystemTime::UNIX_EPOCH)?;
    let secs = duration.as_secs();
    let nsecs = duration.subsec_nanos();

    // 使用 chrono 格式化
    use chrono::{DateTime, Utc, NaiveDateTime};
    let naive = NaiveDateTime::from_timestamp_opt(secs as i64, nsecs);
    match naive {
        Some(dt) => Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc).to_rfc3339()),
        None => Err(anyhow::anyhow!("无效的时间戳")),
    }
}

/// 计算 JSONL 文件的行数（消息数量）
fn count_jsonl_lines(path: &Path) -> Result<usize> {
    use std::io::BufRead;
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);

    let count = reader.lines().count();
    Ok(count)
}

/// 降级方法：基于时间判断（所有平台通用）
fn is_active_by_time(path: &Path, threshold_secs: u64) -> bool {
    let metadata = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return false,
    };

    let modified = match metadata.modified() {
        Ok(time) => time,
        Err(_) => return false,
    };

    let now = SystemTime::now();
    let duration = match now.duration_since(modified) {
        Ok(d) => d,
        Err(_) => return false,
    };

    duration.as_secs() < threshold_secs
}

/// 判断会话是否活跃（Windows 平台）
#[cfg(target_os = "windows")]
fn is_session_active(path: &Path) -> bool {
    // Windows: 优先使用文件锁定检测
    if is_file_locked(path) {
        return true;
    }

    // 降级到时间判断
    let threshold = get_default_active_threshold();
    is_active_by_time(path, threshold)
}

/// 判断会话是否活跃（非 Windows 平台）
#[cfg(not(target_os = "windows"))]
fn is_session_active(path: &Path) -> bool {
    // macOS/Linux: 直接使用时间判断
    let threshold = get_default_active_threshold();
    is_active_by_time(path, threshold)
}

/// 检查文件是否被锁定（仅 Windows）
#[cfg(target_os = "windows")]
fn is_file_locked(path: &Path) -> bool {
    use std::os::windows::fs::OpenOptionsExt;
    use std::fs::OpenOptions;

    // 尝试以独占模式打开文件
    match OpenOptions::new()
        .read(true)
        .share_mode(0)  // 独占模式
        .open(path)
    {
        Ok(_) => false,  // 成功打开 = 未锁定
        Err(_) => true   // 失败 = 被锁定
    }
}

/// 获取默认活跃阈值（秒）
fn get_default_active_threshold() -> u64 {
    // TODO: 从数据库读取配置
    // 目前使用硬编码默认值 86400（24小时）
    86400
}

/// 带超时的会话扫描
///
/// # 参数
/// - `timeout`: 超时时间
///
/// # 返回
/// 返回扫描到的会话列表，或超时错误
pub fn scan_sessions_with_timeout(timeout: Duration) -> Result<Vec<SessionMetadata>> {
    let start = std::time::Instant::now();

    // 创建通道用于线程间通信
    let (tx, rx) = std::sync::mpsc::channel();

    // 在后台线程执行扫描
    let handle = std::thread::spawn(move || {
        let result = scan_session_files();
        // 发送结果，忽略发送错误（接收端可能已超时关闭）
        let _ = tx.send(result);
    });

    // 等待结果或超时
    let result = rx.recv_timeout(timeout);

    match result {
        Ok(scan_result) => {
            let elapsed = start.elapsed();
            eprintln!("扫描完成，耗时: {:?}", elapsed);
            scan_result
        }
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            eprintln!("⚠️  扫描超时: 超过 {:?}", timeout);
            Err(anyhow::anyhow!("扫描超时"))
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            Err(anyhow::anyhow!("扫描线程意外终止"))
        }
    }
}

/// 便捷函数：使用默认超时（10 秒）
pub fn scan_sessions_with_default_timeout() -> Result<Vec<SessionMetadata>> {
    scan_sessions_with_timeout(Duration::from_secs(10))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_claude_projects_dir() {
        let dir = get_claude_projects_dir();
        assert!(dir.is_ok());
        let path = dir.unwrap();
        assert!(path.ends_with(".claude/projects"));
    }

    #[test]
    fn test_is_valid_uuid() {
        assert!(is_valid_uuid("01234567-89ab-cdef-0123-456789abcdef"));
        assert!(is_valid_uuid("00000000-0000-0000-0000-000000000000"));
        assert!(!is_valid_uuid("not-a-uuid"));
        assert!(!is_valid_uuid("12345"));
    }

    #[test]
    fn test_system_time_to_rfc3339() {
        let time = SystemTime::UNIX_EPOCH;
        let result = system_time_to_rfc3339(time);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1970-01-01T00:00:00+00:00");
    }
}
