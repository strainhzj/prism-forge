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
use crate::database::models::Message;
use crate::parser::jsonl::JsonlParser;
use crate::parser::view_level::{MessageFilter, ViewLevel};

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
    pub fn parse_session(&self, file_path: &str, session_id: &str) -> Result<SessionParseResult> {
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
        // 🔧 修复：优先使用 type 字段，如果不存在或无效则尝试使用 role 字段
        // Claude Code 会话文件的 type 字段直接是角色名称 (user/assistant/system)
        // 而不是 "message" 类型
        let msg_type = entry
            .message_type()
            .or_else(|| entry.role()) // Fallback: 使用 role 字段
            .unwrap_or_else(|| {
                // 最后的 fallback: 检查 message.type 字段
                entry
                    .data
                    .get("message")
                    .and_then(|v| v.as_object())
                    .and_then(|obj| obj.get("type"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            });

        // 只处理对话消息类型 (user, assistant, system)
        if !matches!(msg_type.as_str(), "user" | "assistant" | "system") {
            if self.config.debug {
                eprintln!(
                    "[SessionParser] 跳过非对话消息类型: msg_type={:?}",
                    msg_type
                );
            }
            return None;
        }

        // 从 JsonlEntry 提取消息数据
        let uuid = entry.data.get("uuid")?.as_str()?.to_string();
        let parent_uuid = entry
            .data
            .get("parentUuid")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // 从 data 中提取 timestamp
        let timestamp = entry
            .data
            .get("timestamp")
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

        // 提取 content（从 message.content 或 message.content[].text）
        let content = self.extract_message_content(&entry.data);

        // 提取 content_type（从 message.content[0].type）
        let content_type = self.extract_content_type(&entry.data);

        // 在非完整模式下过滤 tool_use 和 tool_result
        // 完整模式（Full）需要保留所有消息，包括工具调用
        if self.config.view_level != ViewLevel::Full {
            // 检查 content_type 是否为 tool_use 或 tool_result
            if let Some(ref ct) = content_type {
                if matches!(ct.as_str(), "tool_use" | "tool_result") {
                    if self.config.debug {
                        eprintln!("[SessionParser] 跳过包含 tool_use/tool_result 的消息: uuid={}, msg_type={}, content_type={}",
                            &uuid[..uuid.len().min(8)],
                            msg_type,
                            ct
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
            content_type,
            timestamp: timestamp.clone(),
            offset: entry.offset as i64,
            length: entry.length as i64,
            summary,
            content,
            parent_idx: None,
            created_at: timestamp,
        })
    }

    /// 提取内容类型
    ///
    /// 从 message.content[0].type 提取内容类型（text/tool_use/tool_result/thinking）
    fn extract_content_type(&self, data: &serde_json::Value) -> Option<String> {
        // 获取 message 对象
        let message = data.get("message")?;

        // 尝试解析为对象
        let message_obj = message.as_object()?;

        // 获取 content 字段
        let content = message_obj.get("content")?;

        // 检查 content 是否为数组
        if let Some(content_array) = content.as_array() {
            if !content_array.is_empty() {
                // 提取第一个元素的 type 字段
                let first_element = &content_array[0];
                if let Some(content_type) = first_element.get("type") {
                    if let Some(type_str) = content_type.as_str() {
                        return Some(type_str.to_string());
                    }
                }
            }
        }

        // 回退：如果是 thinking 类型的消息，直接返回 "thinking"
        if let Some(type_val) = data.get("type") {
            if let Some(type_str) = type_val.as_str() {
                if type_str == "thinking" {
                    return Some("thinking".to_string());
                }
            }
        }

        None
    }

    /// 提取消息内容（从 message.content 或 message.content[].text）
    ///
    /// 支持以下格式：
    /// 1. message 是字符串：直接使用
    /// 2. message.content 是字符串：直接使用
    /// 3. message.content 是数组：提取所有 type="text" 的元素的 text 字段并合并
    fn extract_message_content(&self, data: &serde_json::Value) -> Option<String> {
        let message = data.get("message")?;

        // 情况 1: message 是字符串
        if let Some(s) = message.as_str() {
            return Some(s.to_string());
        }

        // message 是对象，获取 content 字段
        let message_obj = message.as_object()?;
        let content = message_obj.get("content")?;

        // 情况 2: content 是字符串
        if let Some(s) = content.as_str() {
            return Some(s.to_string());
        }

        // 情况 3: content 是数组，提取所有 text 类型的内容
        if let Some(content_array) = content.as_array() {
            let text_parts: Vec<String> = content_array
                .iter()
                .filter_map(|item| {
                    // 只处理 type 为 "text" 的元素
                    if let Some(type_val) = item.get("type") {
                        if type_val.as_str() == Some("text") {
                            // 提取 text 字段
                            if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                                return Some(text.to_string());
                            }
                        }
                    }
                    None
                })
                .collect();

            if !text_parts.is_empty() {
                return Some(text_parts.join("\n"));
            }
        }

        // 回退：尝试从 message.text 提取
        if let Some(text) = message_obj.get("text").and_then(|t| t.as_str()) {
            return Some(text.to_string());
        }

        None
    }

    /// 判断是否应该过滤该消息（基于内容）
    ///
    /// 使用 FilterConfigManager 进行内容过滤，如果配置加载失败则回退到不过滤
    fn should_filter_content(&self, msg: &Message) -> bool {
        use crate::filter_config::FilterConfigManager;

        // 如果没有摘要内容，不过滤
        let summary = match &msg.summary {
            Some(s) => s,
            None => return false,
        };

        // 尝试加载 FilterConfigManager
        match FilterConfigManager::with_default_path() {
            Ok(manager) => {
                // 使用配置管理器进行过滤
                let should_filter = manager.should_filter(summary);

                // 调试日志
                if self.config.debug && should_filter {
                    eprintln!(
                        "[SessionParser] 内容被过滤配置规则过滤: uuid={}",
                        &msg.uuid[..msg.uuid.len().min(8)]
                    );
                }

                should_filter
            }
            Err(e) => {
                // 配置加载失败时，回退到简单过滤逻辑
                if self.config.debug {
                    eprintln!(
                        "[SessionParser] FilterConfigManager 加载失败: {}, 回退到简单过滤",
                        e
                    );
                }

                // 简单回退逻辑：仅过滤 /clear 命令
                let trimmed = summary.trim();
                trimmed.starts_with("/clear")
                    || (trimmed.starts_with("/") && !trimmed.contains(" "))
            }
        }
    }

    /// 应用视图等级过滤（步骤 3）
    ///
    /// 根据视图等级过滤消息
    fn apply_view_level_filter(&self, messages: Vec<Message>) -> Result<(Vec<Message>, usize)> {
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
{"timestamp":"2025-01-19T12:00:04Z","type":"user","uuid":"msg-005","message":"Some text with <command-name>/clear</command-name> inside","parentUuid":"msg-004"}
{"timestamp":"2025-01-19T12:00:05Z","type":"system","uuid":"msg-006","message":"Conversation cleared with <local-command-caveat>","parentUuid":"msg-005"}
"#.to_string()
    }

    /// 创建包含可过滤内容的测试数据（用于内容过滤测试）
    fn create_filterable_test_jsonl_content() -> String {
        r#"{"timestamp":"2025-01-19T12:00:00Z","type":"user","uuid":"msg-001","message":"Hello, how are you?","parentUuid":null}
{"timestamp":"2025-01-19T12:00:01Z","type":"assistant","uuid":"msg-002","message":"I'm doing well, thank you!","parentUuid":"msg-001"}
{"timestamp":"2025-01-19T12:00:02Z","type":"user","uuid":"msg-003","message":"What's the weather like?","parentUuid":"msg-002"}
{"timestamp":"2025-01-19T12:00:03Z","type":"assistant","uuid":"msg-004","message":"I don't have access to real-time weather data.","parentUuid":"msg-003"}
{"timestamp":"2025-01-19T12:00:04Z","type":"user","uuid":"msg-005","message":"Execute <command-name>/clear</command-name> now","parentUuid":"msg-004"}
{"timestamp":"2025-01-19T12:00:05Z","type":"system","uuid":"msg-006","message":"System notification message","parentUuid":"msg-005"}
{"timestamp":"2025-01-19T12:00:06Z","type":"user","uuid":"msg-007","message":"Warning: <local-command-caveat> this is a local command","parentUuid":"msg-006"}
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
            writeln!(file, "{}", create_filterable_test_jsonl_content()).unwrap();
        }

        let file_path = test_file_path.to_str().unwrap();

        let config = SessionParserConfig {
            enable_content_filter: true, // 启用内容过滤
            view_level: ViewLevel::Full,
            debug: false,
        };

        let parser = SessionParserService::new(config);
        let result = parser.parse_session(file_path, "test_session");

        let _ = std::fs::remove_file(&test_file_path);

        assert!(result.is_ok());
        let parse_result = result.unwrap();

        // 验证包含 <command-name>/clear</command-name> 和 <local-command-caveat> 的消息被过滤
        assert!(parse_result.stats.content_filtered > 0);
        assert_eq!(parse_result.stats.final_messages, 5); // 7 - 2 (filtered)
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
            view_level: ViewLevel::Conversation, // 对话模式
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
            writeln!(file, "{}", create_filterable_test_jsonl_content()).unwrap();
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

    #[test]
    fn test_message_order_preserved() {
        let temp_dir = std::env::temp_dir();
        let test_file_path = temp_dir.join("test_session_order.jsonl");

        {
            let mut file = std::fs::File::create(&test_file_path).unwrap();
            writeln!(file, "{}", create_test_jsonl_content()).unwrap();
        }

        let file_path = test_file_path.to_str().unwrap();

        let config = SessionParserConfig::default();
        let parser = SessionParserService::new(config);
        let result = parser.parse_session(file_path, "test_session");

        let _ = std::fs::remove_file(&test_file_path);

        assert!(result.is_ok());
        let parse_result = result.unwrap();

        // 验证消息顺序保持不变
        let timestamps: Vec<&str> = parse_result
            .messages
            .iter()
            .map(|msg| msg.timestamp.as_str())
            .collect();

        let mut sorted_timestamps = timestamps.clone();
        sorted_timestamps.sort();

        assert_eq!(timestamps, sorted_timestamps);
    }

    #[test]
    fn test_error_handling_file_not_found() {
        let config = SessionParserConfig::default();
        let parser = SessionParserService::new(config);
        let result = parser.parse_session("/nonexistent/file.jsonl", "test_session");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("会话文件不存在"));
    }
}
