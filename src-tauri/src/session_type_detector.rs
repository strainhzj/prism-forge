//! 会话文件类型检测模块
//!
//! 辨别 Claude Code 会话文件的类型（Main/Agent）
//! 检测会话快照文件（file-history-snapshot）

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader};

// ==================== 类型定义 ====================

/// 会话文件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionFileType {
    /// 主会话文件（UUID.jsonl）
    Main,
    /// Agent 会话文件（agent-{id}.jsonl）
    Agent,
    /// 未知类型
    Unknown,
}

impl SessionFileType {
    /// 是否是主会话
    pub fn is_main(&self) -> bool {
        matches!(self, Self::Main)
    }

    /// 是否是 Agent 会话
    pub fn is_agent(&self) -> bool {
        matches!(self, Self::Agent)
    }
}

// ==================== 类型检测函数 ====================

/// 根据文件名判断会话类型
///
/// # 参数
/// * `file_path` - 文件路径
///
/// # 返回
/// 返回会话文件类型
///
/// # 示例
/// ```no_run
/// use crate::session_type_detector::detect_session_type_by_filename;
///
/// let main_type = detect_session_type_by_filename("7149f370-067c-447e-a7dc-dc161d3f8de7.jsonl");
/// assert_eq!(main_type, SessionFileType::Main);
///
/// let agent_type = detect_session_type_by_filename("agent-eb95d9a3.jsonl");
/// assert_eq!(agent_type, SessionFileType::Agent);
/// ```
pub fn detect_session_type_by_filename(file_path: impl AsRef<Path>) -> SessionFileType {
    let file_name = file_path.as_ref()
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    // 优先检查 agent 前缀（agent-{id}.jsonl）
    if file_name.starts_with("agent-") && file_name.ends_with(".jsonl") {
        return SessionFileType::Agent;
    }

    // 检查是否是主会话文件（UUID 格式）
    if is_uuid_filename(file_name) {
        return SessionFileType::Main;
    }

    SessionFileType::Unknown
}

/// 检查文件名是否是 UUID 格式
///
/// UUID 格式：xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
/// 示例：7149f370-067c-447e-a7dc-dc161d3f8de7
fn is_uuid_filename(file_name: &str) -> bool {
    // 去掉 .jsonl 扩展名
    let name_without_ext = file_name.strip_suffix(".jsonl").unwrap_or(file_name);

    // 使用正则表达式匹配 UUID 格式
    // UUID 格式：8-4-4-4-12 个十六进制字符
    let uuid_pattern = regex::Regex::new(
        r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    ).unwrap();

    uuid_pattern.is_match(name_without_ext)
}

// ==================== 快照文件检测 ====================

/// 会话文件第一行的可能结构
#[derive(Deserialize)]
struct SessionFirstLine {
    #[serde(rename = "type")]
    line_type: Option<String>,
}

/// 检查文件是否是快照文件
///
/// 如果文件第一行的 type 字段为 "file-history-snapshot"，则认为是快照文件
///
/// # 参数
/// * `file_path` - 文件路径
///
/// # 返回
/// 返回 true 如果是快照文件，false 否何
///
/// # 示例
/// ```no_run
/// use crate::session_type_detector::is_snapshot_file;
///
/// // 快照文件
/// assert!(is_snapshot_file("snapshot.jsonl"));
///
/// // 普通会话文件
/// assert!(!is_snapshot_file("7149f370-067c-447e-a7dc-dc161d3f8de7.jsonl"));
/// ```
pub fn is_snapshot_file(file_path: impl AsRef<Path>) -> bool {
    let file_path = file_path.as_ref();

    // 尝试打开并读取文件的第一行
    let file = match File::open(file_path) {
        Ok(f) => f,
        Err(_) => return false, // 无法读取文件，认为不是快照文件
    };

    let reader = BufReader::new(file);
    let first_line = match reader.lines().next() {
        Some(Ok(line)) => line,
        _ => return false, // 文件为空或读取失败
    };

    // 解析第一行 JSON
    if let Ok(data) = serde_json::from_str::<SessionFirstLine>(&first_line) {
        // 检查 type 字段
        if let Some(line_type) = data.line_type {
            return line_type == "file-history-snapshot";
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_main_session() {
        let path = "7149f370-067c-447e-a7dc-dc161d3f8de7.jsonl";
        assert_eq!(
            detect_session_type_by_filename(path),
            SessionFileType::Main
        );
    }

    #[test]
    fn test_detect_agent_session() {
        let path = "agent-eb95d9a3.jsonl";
        assert_eq!(
            detect_session_type_by_filename(path),
            SessionFileType::Agent
        );
    }

    #[test]
    fn test_detect_unknown_session() {
        let path = "random-file.jsonl";
        assert_eq!(
            detect_session_type_by_filename(path),
            SessionFileType::Unknown
        );
    }

    #[test]
    fn test_uuid_validation() {
        // 有效的 UUID
        assert!(is_uuid_filename("7149f370-067c-447e-a7dc-dc161d3f8de7.jsonl"));
        assert!(is_uuid_filename("0bf43974-daf7-4ff1-957a-de72f79556e2.jsonl"));

        // 无效的 UUID
        assert!(!is_uuid_filename("agent-eb95d9a3.jsonl"));
        assert!(!is_uuid_filename("not-a-uuid.jsonl"));
        assert!(!is_uuid_filename("random-file.jsonl"));
    }

    #[test]
    fn test_session_type_methods() {
        let main_type = SessionFileType::Main;
        assert!(main_type.is_main());
        assert!(!main_type.is_agent());

        let agent_type = SessionFileType::Agent;
        assert!(!agent_type.is_main());
        assert!(agent_type.is_agent());

        let unknown_type = SessionFileType::Unknown;
        assert!(!unknown_type.is_main());
        assert!(!unknown_type.is_agent());
    }
}
