//! 统一会话解析服务
//!
//! 本模块提供统一的会话文件解析服务，集成以下功能：
//! - JSONL 文件解析
//! - 消息格式转换
//! - 内容过滤（基于 FilterConfigManager）
//! - 视图等级过滤（基于 MessageFilter）
//!
//! # 架构设计
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                   SessionParserService                   │
//! ├─────────────────────────────────────────────────────────┤
//! │  1. parse_file      → JsonlParser 解析 JSONL 文件        │
//! │  2. convert_messages → 转换为 Message + 内容过滤         │
//! │  3. apply_view_level_filter → 视图等级过滤               │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! # 使用示例
//!
//! ```no_run
//! use crate::session_parser::{SessionParserService, SessionParserConfig};
//! use crate::parser::view_level::ViewLevel;
//!
//! let config = SessionParserConfig {
//!     enable_content_filter: true,
//!     view_level: ViewLevel::Full,
//!     debug: true,
//! };
//!
//! let parser = SessionParserService::new(config);
//! let result = parser.parse_session("/path/to/session.jsonl", "session_123")?;
//!
//! println!("解析完成: {} 条消息", result.messages.len());
//! println!("统计: {:?}", result.stats);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use anyhow::Result;
use serde::{Deserialize, Serialize};

// 导入现有类型
use crate::parser::jsonl::JsonlParser;
use crate::parser::view_level::{ViewLevel, MessageFilter};
use crate::database::models::Message;

// ==================== 配置 ====================

/// 会话解析配置
#[derive(Debug, Clone)]
pub struct SessionParserConfig {
    /// 是否启用内容过滤（FilterConfigManager）
    pub enable_content_filter: bool,

    /// 视图等级
    pub view_level: ViewLevel,

    /// 是否包含调试日志
    pub debug: bool,
}

impl Default for SessionParserConfig {
    fn default() -> Self {
        Self {
            enable_content_filter: true,
            view_level: ViewLevel::Full,
            debug: cfg!(debug_assertions),
        }
    }
}

// ==================== 解析结果 ====================

/// 会话解析结果
#[derive(Debug)]
pub struct SessionParseResult {
    /// 过滤后的消息列表
    pub messages: Vec<Message>,

    /// 统计信息
    pub stats: ParseStats,
}

/// 解析统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseStats {
    /// 原始条目数量
    pub total_entries: usize,

    /// 转换后的消息数量
    pub converted_messages: usize,

    /// 内容过滤掉的数量
    pub content_filtered: usize,

    /// 视图等级过滤掉的数量
    pub view_level_filtered: usize,

    /// 最终消息数量
    pub final_messages: usize,
}

// ==================== 解析服务 ====================

/// 统一会话解析服务
pub struct SessionParserService {
    config: SessionParserConfig,
}

impl SessionParserService {
    /// 创建新的解析服务
    pub fn new(config: SessionParserConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Result<Self> {
        Ok(Self::new(SessionParserConfig::default()))
    }

    /// 解析会话文件
    ///
    /// # 参数
    /// - `file_path`: 会话文件路径
    /// - `session_id`: 会话 ID（用于填充 Message.session_id）
    ///
    /// # 返回
    /// 解析结果，包含过滤后的消息和统计信息
    ///
    /// # 错误
    /// - 文件不存在
    /// - 文件解析失败
    /// - 消息转换失败
    pub fn parse_session(
        &self,
        file_path: &str,
        session_id: &str,
    ) -> Result<SessionParseResult> {
        // 1. 使用 JsonlParser 解析文件
        let entries = self.parse_file(file_path)?;
        let total_entries = entries.len();

        // 2. 转换为 Message 对象
        let (messages, content_filtered) = self.convert_messages(entries, session_id)?;

        // 3. 应用视图等级过滤
        let (messages, view_level_filtered) = self.apply_view_level_filter(messages)?;

        // 4. 构建统计信息
        let stats = ParseStats {
            total_entries,
            converted_messages: messages.len() + content_filtered + view_level_filtered,
            content_filtered,
            view_level_filtered,
            final_messages: messages.len(),
        };

        // 5. 输出调试信息
        if self.config.debug {
            eprintln!("[SessionParser] 解析统计: {:?}", stats);
        }

        Ok(SessionParseResult { messages, stats })
    }

    /// 解析文件（步骤 1）
    ///
    /// 从 JSONL 文件中读取所有条目
    fn parse_file(&self, file_path: &str) -> Result<Vec<crate::parser::jsonl::JsonlEntry>> {
        let path = std::path::PathBuf::from(file_path);
        if !path.exists() {
            anyhow::bail!("会话文件不存在: {}", file_path);
        }

        let mut parser = JsonlParser::new(path)?;
        let entries = parser.parse_all()?;

        Ok(entries)
    }

    /// 转换消息（步骤 2）
    ///
    /// 将 JsonlEntry 转换为 Message 对象，并应用内容过滤
    fn convert_messages(
        &self,
        entries: Vec<crate::parser::jsonl::JsonlEntry>,
        session_id: &str,
    ) -> Result<(Vec<Message>, usize)> {
        let mut messages = Vec::new();
        let mut content_filtered = 0;

        for entry in entries {
            // 转换逻辑（从 cmd_get_messages_by_level 移植）
            if let Some(msg) = self.convert_entry(&entry, session_id) {
                // 应用内容过滤
                if self.config.enable_content_filter {
                    if self.should_filter_content(&msg) {
                        content_filtered += 1;
                        if self.config.debug {
                            eprintln!("[SessionParser] 内容过滤: {:?}", msg.summary);
                        }
                        continue;
                    }
                }
                messages.push(msg);
            }
        }

        Ok((messages, content_filtered))
    }

    /// 转换单个条目为 Message
    fn convert_entry(
        &self,
        entry: &crate::parser::jsonl::JsonlEntry,
        session_id: &str,
    ) -> Option<Message> {
        use crate::parser::jsonl::JsonlEntry;

        // 🔧 修复：优先使用 type 字段，如果不存在或无效则尝试使用 role 字段
        // Claude Code 会话文件的 type 字段直接是角色名称 (user/assistant/system)
        // 而不是 "message" 类型
        let msg_type = entry.message_type()
            .or_else(|| entry.role())  // Fallback: 使用 role 字段
            .unwrap_or_else(|| {
                // 最后的 fallback: 检查 message.type 字段
                entry.data.get("message")
                    .and_then(|v| v.as_object())
                    .and_then(|obj| obj.get("type"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            });

        // 只处理对话消息类型 (user, assistant, system)
        if !matches!(msg_type.as_str(), "user" | "assistant" | "system") {
            if self.config.debug {
                eprintln!("[SessionParser] 跳过非对话消息类型: msg_type={:?}", msg_type);
            }
            return None;
        }

        // 从 JsonlEntry 提取消息数据
        let uuid = entry.data.get("uuid")?.as_str()?.to_string();
        let parent_uuid = entry.data.get("parentUuid")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // 从 data 中提取 timestamp
        let timestamp = entry.data.get("timestamp")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

        // 从 message 字段提取内容 (summary)
        // Claude Code 的 message 字段可能是字符串或 JSON 对象
        let summary = entry.data.get("message").map(|v| {
            // 尝试作为字符串
            if let Some(s) = v.as_str() {
                s.to_string()
            } else if let Some(obj) = v.as_object() {
                // 如果是对象,尝试提取 text 字段或转为 JSON 字符串
                if let Some(text) = obj.get("text").and_then(|t| t.as_str()) {
                    text.to_string()
                } else {
                    // 转为 JSON 字符串
                    serde_json::to_string(v).unwrap_or_else(|_| "[无法解析的消息]".to_string())
                }
            } else {
                // 其他类型,转为字符串
                v.to_string()
            }
        });

        // 在非完整模式下过滤 tool_use 和 tool_result
        // 完整模式（Full）需要保留所有消息，包括工具调用
        if self.config.view_level != ViewLevel::Full {
            if let Some(ref content) = summary {
                if content.contains("\"type\":\"tool_use\"") ||
                   content.contains("\"type\": \"tool_use\"") ||
                   content.contains("\"type\":\"tool_result\"") ||
                   content.contains("\"type\": \"tool_result\"") {
                    if self.config.debug {
                        eprintln!("[SessionParser] 跳过包含 tool_use/tool_result 的消息: uuid={}, msg_type={}",
                            &uuid[..uuid.len().min(8)],
                            msg_type
                        );
                    }
                    return None;
                }
            }
        }

        // 使用 type 字段值作为 msg_type (user/assistant/system)
        Some(Message {
            id: None,
            session_id: session_id.to_string(),
            uuid,
            parent_uuid,
            msg_type,
            timestamp: timestamp.clone(),
            offset: entry.offset as i64,
            length: entry.length as i64,
            summary,
            parent_idx: None,
            created_at: timestamp,
        })
    }

    /// 判断是否应该过滤该消息（基于内容）
    fn should_filter_content(&self, msg: &Message) -> bool {
        // TODO: 集成 FilterConfigManager
        // 当前实现简单过滤逻辑
        if let Some(ref summary) = msg.summary {
            // 过滤 /clear 命令
            if summary.trim().starts_with("/clear") {
                return true;
            }
            // 过滤系统命令
            if summary.trim().starts_with("/") && !summary.contains(" ") {
                return true;
            }
        }
        false
    }

    /// 应用视图等级过滤（步骤 3）
    ///
    /// 根据视图等级过滤消息
    fn apply_view_level_filter(
        &self,
        messages: Vec<Message>,
    ) -> Result<(Vec<Message>, usize)> {
        let filter = MessageFilter::new(self.config.view_level.clone());
        let before_count = messages.len();
        let filtered = filter.filter_messages(messages);
        let after_count = filtered.len();

        Ok((filtered, before_count - after_count))
    }
}

// ==================== 测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> SessionParserConfig {
        SessionParserConfig {
            enable_content_filter: false,
            view_level: ViewLevel::Full,
            debug: false,
        }
    }

    #[test]
    fn test_session_parser_config_default() {
        let config = SessionParserConfig::default();
        assert!(config.enable_content_filter);
        assert_eq!(config.view_level, ViewLevel::Full);
    }

    #[test]
    fn test_session_parser_service_creation() {
        let config = create_test_config();
        let service = SessionParserService::new(config);
        // 验证服务创建成功
        assert_eq!(service.config.view_level, ViewLevel::Full);
    }

    #[test]
    fn test_session_parser_service_with_defaults() {
        let service = SessionParserService::with_defaults();
        assert!(service.is_ok());
        let service = service.unwrap();
        assert!(service.config.enable_content_filter);
    }

    #[test]
    fn test_parse_stats() {
        let stats = ParseStats {
            total_entries: 100,
            converted_messages: 95,
            content_filtered: 5,
            view_level_filtered: 10,
            final_messages: 80,
        };

        assert_eq!(stats.total_entries, 100);
        assert_eq!(stats.final_messages, 80);
    }

    #[test]
    fn test_view_level_serialization() {
        let full = ViewLevel::Full;
        let serialized = serde_json::to_string(&full).unwrap();
        assert_eq!(serialized, "\"full\"");
    }
}

// ==================== 集成测试 ====================
//
// 注意：以下集成测试需要 tempfile 依赖
// 在 Cargo.toml 中添加：tempfile = "3"
//
// 如果不需要集成测试，可以注释掉以下模块

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::parser::view_level::ViewLevel;
    use std::io::Write;

    /// 创建临时 JSONL 测试文件
    fn create_test_jsonl_content() -> String {
        r#"{"timestamp":"2025-01-19T12:00:00Z","type":"user","uuid":"msg-001","message":"Hello, how are you?","parentUuid":null}
{"timestamp":"2025-01-19T12:00:01Z","type":"assistant","uuid":"msg-002","message":"I'm doing well, thank you!","parentUuid":"msg-001"}
{"timestamp":"2025-01-19T12:00:02Z","type":"user","uuid":"msg-003","message":"What's the weather like?","parentUuid":"msg-002"}
{"timestamp":"2025-01-19T12:00:03Z","type":"assistant","uuid":"msg-004","message":"I don't have access to real-time weather data.","parentUuid":"msg-003"}
{"timestamp":"2025-01-19T12:00:04Z","type":"user","uuid":"msg-005","message":"/clear","parentUuid":"msg-004"}
{"timestamp":"2025-01-19T12:00:05Z","type":"system","uuid":"msg-006","message":"Conversation cleared","parentUuid":"msg-005"}
"#.to_string()
    }

    #[test]
    fn test_full_parsing_workflow() {
        // 使用临时目录创建测试文件
        let temp_dir = std::env::temp_dir();
        let test_file_path = temp_dir.join("test_session.jsonl");

        {
            let mut file = std::fs::File::create(&test_file_path).unwrap();
            writeln!(file, "{}", create_test_jsonl_content()).unwrap();
        }

        let file_path = test_file_path.to_str().unwrap();

        let config = SessionParserConfig {
            enable_content_filter: false,
            view_level: ViewLevel::Full,
            debug: false,
        };

        let parser = SessionParserService::new(config);
        let result = parser.parse_session(file_path, "test_session");

        // 清理测试文件
        let _ = std::fs::remove_file(&test_file_path);

        assert!(result.is_ok());
        let parse_result = result.unwrap();

        // 验证解析统计
        assert_eq!(parse_result.stats.total_entries, 6);
        assert_eq!(parse_result.stats.final_messages, 6);

        // 验证消息内容
        assert_eq!(parse_result.messages.len(), 6);
        assert_eq!(parse_result.messages[0].msg_type, "user");
        assert_eq!(parse_result.messages[1].msg_type, "assistant");
    }

    #[test]
    fn test_content_filtering() {
        let temp_dir = std::env::temp_dir();
        let test_file_path = temp_dir.join("test_session_filter.jsonl");

        {
            let mut file = std::fs::File::create(&test_file_path).unwrap();
            writeln!(file, "{}", create_test_jsonl_content()).unwrap();
        }

        let file_path = test_file_path.to_str().unwrap();

        let config = SessionParserConfig {
            enable_content_filter: true,  // 启用内容过滤
            view_level: ViewLevel::Full,
            debug: false,
        };

        let parser = SessionParserService::new(config);
        let result = parser.parse_session(file_path, "test_session");

        let _ = std::fs::remove_file(&test_file_path);

        assert!(result.is_ok());
        let parse_result = result.unwrap();

        // 验证 /clear 命令被过滤
        assert!(parse_result.stats.content_filtered > 0);
        assert_eq!(parse_result.stats.final_messages, 5); // 6 - 1 (filtered)
    }

    #[test]
    fn test_view_level_filtering() {
        let temp_dir = std::env::temp_dir();
        let test_file_path = temp_dir.join("test_session_viewlevel.jsonl");

        {
            let mut file = std::fs::File::create(&test_file_path).unwrap();
            writeln!(file, "{}", create_test_jsonl_content()).unwrap();
        }

        let file_path = test_file_path.to_str().unwrap();

        let config = SessionParserConfig {
            enable_content_filter: false,
            view_level: ViewLevel::Conversation,  // 对话模式
            debug: false,
        };

        let parser = SessionParserService::new(config);
        let result = parser.parse_session(file_path, "test_session");

        let _ = std::fs::remove_file(&test_file_path);

        assert!(result.is_ok());
        let parse_result = result.unwrap();

        // Conversation 模式应该过滤掉 system 消息
        assert!(parse_result.stats.view_level_filtered > 0);
    }

    #[test]
    fn test_combined_filtering() {
        let temp_dir = std::env::temp_dir();
        let test_file_path = temp_dir.join("test_session_combined.jsonl");

        {
            let mut file = std::fs::File::create(&test_file_path).unwrap();
            writeln!(file, "{}", create_test_jsonl_content()).unwrap();
        }

        let file_path = test_file_path.to_str().unwrap();

        let config = SessionParserConfig {
            enable_content_filter: true,
            view_level: ViewLevel::Conversation,
            debug: false,
        };

        let parser = SessionParserService::new(config);
        let result = parser.parse_session(file_path, "test_session");

        let _ = std::fs::remove_file(&test_file_path);

        assert!(result.is_ok());
        let parse_result = result.unwrap();

        // 验证两种过滤都生效
        assert!(parse_result.stats.content_filtered > 0);
        assert!(parse_result.stats.view_level_filtered > 0);

        // 验证最终消息数量
        assert_eq!(
            parse_result.stats.final_messages,
            parse_result.stats.total_entries
                - parse_result.stats.content_filtered
                - parse_result.stats.view_level_filtered
        );
    }

    #[test]
    fn test_session_id_assigned_correctly() {
        let temp_dir = std::env::temp_dir();
        let test_file_path = temp_dir.join("test_session_id.jsonl");

        {
            let mut file = std::fs::File::create(&test_file_path).unwrap();
            writeln!(file, "{}", create_test_jsonl_content()).unwrap();
        }

        let file_path = test_file_path.to_str().unwrap();

        let config = SessionParserConfig::default();
        let parser = SessionParserService::new(config);
        let result = parser.parse_session(file_path, "my_test_session");

        let _ = std::fs::remove_file(&test_file_path);

        assert!(result.is_ok());
        let parse_result = result.unwrap();

        // 验证所有消息都有正确的 session_id
        for msg in &parse_result.messages {
            assert_eq!(msg.session_id, "my_test_session");
        }
    }
}

