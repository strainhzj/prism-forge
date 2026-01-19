//! 多等级日志读取模块
//!
//! 实现不同等级的消息过滤逻辑，支持 Full、Conversation、QAPairs、AssistantOnly、UserOnly 五种等级。
//!
//! ## 功能目标
//!
//! 允许用户根据不同场景选择不同的日志过滤等级：
//! - **Full**: 完整模式，包含所有消息类型
//! - **Conversation**: 对话模式，包含 user、assistant 和 thinking
//! - **QAPairs**: 问答对模式，提取问答配对
//! - **AssistantOnly**: 仅助手输出
//! - **UserOnly**: 仅用户输入
//!
//! ## 设计原则
//!
//! - **后端过滤优先**: 在 Rust 后端的 JSONL 解析阶段直接应用过滤器
//! - **流式解析支持**: 利用现有的 JsonlParser，在解析时应用过滤逻辑
//! - **状态持久化**: 新增 view_level_preferences 表存储每个会话的等级选择

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use anyhow::Result;

use crate::database::models::Message;

/// 日志读取等级
///
/// 定义五种不同的日志读取等级，按信息完整度排序。
/// 默认值为 Full，包含所有消息类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewLevel {
    /// 完整模式：包含所有消息类型（user、assistant、tool_use、thinking）
    Full,
    /// 对话模式：包含 user、assistant 和 thinking 类型的消息
    Conversation,
    /// 问答对模式：提取用户问题和助手最终回复的配对
    #[serde(rename = "qa_pairs")]
    QAPairs,
    /// 仅助手输出：只包含 assistant 类型的消息
    AssistantOnly,
    /// 仅用户输入：只包含 user 类型的消息
    UserOnly,
}

impl Default for ViewLevel {
    fn default() -> Self {
        ViewLevel::Full
    }
}

impl fmt::Display for ViewLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ViewLevel::Full => write!(f, "full"),
            ViewLevel::Conversation => write!(f, "conversation"),
            ViewLevel::QAPairs => write!(f, "qa_pairs"),
            ViewLevel::AssistantOnly => write!(f, "assistant_only"),
            ViewLevel::UserOnly => write!(f, "user_only"),
        }
    }
}

impl ViewLevel {
    /// 获取等级的显示名称（中文）
    pub fn display_name(&self) -> &'static str {
        match self {
            ViewLevel::Full => "完整模式",
            ViewLevel::Conversation => "对话模式",
            ViewLevel::QAPairs => "问答对模式",
            ViewLevel::AssistantOnly => "仅助手",
            ViewLevel::UserOnly => "仅用户",
        }
    }

    /// 获取等级的描述说明
    pub fn description(&self) -> &'static str {
        match self {
            ViewLevel::Full => "包含所有消息类型，包括工具调用和思考过程",
            ViewLevel::Conversation => "包含用户、助手和思考过程，隐藏工具调用细节",
            ViewLevel::QAPairs => "提取用户问题和助手最终回复的配对",
            ViewLevel::AssistantOnly => "仅显示助手的输出内容",
            ViewLevel::UserOnly => "仅显示用户的输入内容",
        }
    }

    /// 从字符串解析 ViewLevel
    ///
    /// 支持格式："full", "conversation", "qa_pairs", "assistant_only", "user_only"
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "full" => Ok(ViewLevel::Full),
            "conversation" => Ok(ViewLevel::Conversation),
            "qa_pairs" => Ok(ViewLevel::QAPairs),
            "assistant_only" => Ok(ViewLevel::AssistantOnly),
            "user_only" => Ok(ViewLevel::UserOnly),
            _ => Err(format!("无效的等级值: {}", s)),
        }
    }
}

/// 问答对
///
/// 表示一个用户问题和对应的助手最终回复。
/// 如果用户消息没有找到回复，answer 字段为 None。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QAPair {
    /// 用户问题
    pub question: Message,
    /// 助手最终回复（可能为 None，如果未找到配对）
    pub answer: Option<Message>,
    /// 问答对的时间戳（使用问题的时间戳）
    pub timestamp: String,
}

/// 视图等级错误类型
///
/// 定义等级过滤过程中可能出现的错误。
#[derive(Debug, thiserror::Error)]
pub enum ViewLevelError {
    /// 会话文件不存在
    #[error("会话文件不存在: {0}")]
    SessionNotFound(String),

    /// 消息解析失败
    #[error("消息解析失败: {0}")]
    ParseError(String),

    /// 无效的等级值
    #[error("无效的等级值: {0}")]
    InvalidLevel(String),

    /// 数据库错误
    #[error("数据库错误: {0}")]
    DatabaseError(String),

    /// IO 错误
    #[error("IO 错误: {0}")]
    IoError(String),

    /// QA 配对失败
    #[error("QA 配对失败: {0}")]
    QAPairingError(String),
}

impl From<std::io::Error> for ViewLevelError {
    fn from(err: std::io::Error) -> Self {
        ViewLevelError::IoError(err.to_string())
    }
}

/// 消息过滤器
///
/// 根据选择的 ViewLevel 对消息列表进行过滤。
/// 支持流式处理，在解析阶段直接应用过滤逻辑。
pub struct MessageFilter {
    view_level: ViewLevel,
}

impl MessageFilter {
    /// 创建新的消息过滤器
    ///
    /// # 参数
    ///
    /// - `view_level`: 日志读取等级
    pub fn new(view_level: ViewLevel) -> Self {
        Self { view_level }
    }

    /// 判断消息是否应该被包含
    ///
    /// 根据当前等级判断消息是否符合过滤条件。
    ///
    /// # 参数
    ///
    /// - `message`: 要判断的消息
    ///
    /// # 返回
    ///
    /// - `true`: 消息应该被包含
    /// - `false`: 消息应该被过滤掉
    ///
    /// # 注意
    ///
    /// - QAPairs 等级总是返回 false，因为它需要特殊的配对逻辑
    /// - 其他等级根据消息的 msg_type 字段进行判断
    /// - UserOnly 等级会额外过滤掉 type 为 tool_result 的消息
    pub fn should_include(&self, message: &Message) -> bool {
        match self.view_level {
            ViewLevel::Full => true,
            ViewLevel::Conversation => {
                matches!(
                    message.msg_type.as_str(),
                    "user" | "assistant" | "thinking"
                )
            }
            ViewLevel::QAPairs => {
                // QAPairs 需要特殊处理，在 extract_qa_pairs 中实现
                false
            }
            ViewLevel::AssistantOnly => message.msg_type == "assistant",
            ViewLevel::UserOnly => {
                // UserOnly: 只包含 user 类型，且排除 tool_result 类型
                if message.msg_type != "user" {
                    return false;
                }

                // 额外检查：如果 summary 内容包含 "tool_result" 标记，则过滤掉
                if let Some(ref summary) = message.summary {
                    // 检查是否包含 tool_result 的 JSON 标记
                    if summary.contains("\"type\":\"tool_result\"") ||
                       summary.contains("\"type\": \"tool_result\"") ||
                       summary.contains("tool_result") {
                        return false;
                    }
                }

                true
            }
        }
    }

    /// 过滤消息列表
    ///
    /// 对消息列表应用过滤逻辑，返回符合条件的消息。
    ///
    /// # 参数
    ///
    /// - `messages`: 原始消息列表
    ///
    /// # 返回
    ///
    /// 过滤后的消息列表，保持原始顺序
    ///
    /// # 性能
    ///
    /// - 时间复杂度: O(n)，n 为消息数量
    /// - 空间复杂度: O(m)，m 为符合条件的消息数量
    pub fn filter_messages(&self, messages: Vec<Message>) -> Vec<Message> {
        messages
            .into_iter()
            .filter(|msg| self.should_include(&msg))
            .collect()
    }

    /// 提取问答对
    ///
    /// 从消息列表中提取问答配对。
    ///
    /// # 算法
    ///
    /// **从后向前扫描 + 向前查找**：
    ///
    /// 1. 从后向前扫描，找到每个 user
    /// 2. 遇到 user 时，从当前位置**继续向前**查找该 user 后的最后一个 assistant
    /// 3. 找到后记录配对，然后继续扫描
    ///
    /// 这样确保每个 user 与其后的**最后一个** assistant 配对。
    ///
    /// # 示例
    ///
    /// ```text
    /// 输入: [user1, assistant1, assistant2, user2, assistant3, assistant4]
    /// 输出: [(user1, assistant2), (user2, assistant4)]
    ///       说明：每个user与其后的最后一个assistant配对
    ///
    /// 输入: [user1, assistant1, user2, assistant2]
    /// 输出: [(user1, assistant1), (user2, assistant2)]
    ///
    /// 输入: [user1, thinking, assistant1, user2]
    /// 输出: [(user1, assistant1), (user2, null)]
    ///       说明：thinking被跳过，user2没有答案
    /// ```
    ///
    /// # 参数
    ///
    /// - `messages`: 原始消息列表（按文件顺序）
    ///
    /// # 返回
    ///
    /// 问答对列表（按原始对话顺序）
    ///
    /// # 复杂度
    ///
    /// - 时间复杂度: O(n²)，n 为消息数量（最坏情况）
    /// - 空间复杂度: O(m)，m 为问答对数量
    pub fn extract_qa_pairs(&self, messages: Vec<Message>) -> Vec<QAPair> {
        let mut qa_pairs = Vec::new();
        let mut last_user_idx: Option<usize> = None;  // 记录最后一个user的位置

        // 调试日志
        #[cfg(debug_assertions)]
        {
            eprintln!("🔍 [extract_qa_pairs] 输入消息数量: {}", messages.len());
            let mut user_count = 0;
            let mut assistant_count = 0;
            for msg in &messages {
                match msg.msg_type.as_str() {
                    "user" => user_count += 1,
                    "assistant" => assistant_count += 1,
                    _ => {}
                }
            }
            eprintln!("🔍 [extract_qa_pairs] 统计: user={}, assistant={}", user_count, assistant_count);
        }

        // 从后向前扫描
        let mut i = messages.len();
        while i > 0 {
            i -= 1;
            let msg = &messages[i];

            #[cfg(debug_assertions)]
            {
                eprintln!("🔍 [extract_qa_pairs] [{}] msg_type={}", i, msg.msg_type);
            }

            match msg.msg_type.as_str() {
                "user" => {
                    // 找到一个user，向前查找该user后的最后一个assistant
                    let mut answer: Option<Message> = None;

                    // 从当前user之后开始向前找assistant
                    let mut j = i + 1;
                    while j < messages.len() {
                        let next_msg = &messages[j];
                        match next_msg.msg_type.as_str() {
                            "assistant" => {
                                // 检查 assistant 的 summary 是否包含 tool_result
                                let should_skip = if let Some(ref summary) = next_msg.summary {
                                    summary.contains("\"type\":\"tool_result\"") ||
                                    summary.contains("\"type\": \"tool_result\"") ||
                                    summary.contains("tool_result")
                                } else {
                                    false
                                };

                                if should_skip {
                                    #[cfg(debug_assertions)]
                                    {
                                        eprintln!("   → [j={}] 跳过包含 tool_result 的 assistant", j);
                                    }
                                    j += 1;
                                } else {
                                    // 找到assistant，更新答案（继续找，直到遇到非assistant）
                                    answer = Some(next_msg.clone());
                                    #[cfg(debug_assertions)]
                                    {
                                        eprintln!("   → [j={}] 找到assistant", j);
                                    }
                                    j += 1;
                                }
                            }
                            "thinking" => {
                                // 跳过thinking，继续找
                                #[cfg(debug_assertions)]
                                {
                                    eprintln!("   → [j={}] 跳过thinking", j);
                                }
                                j += 1;
                            }
                            _ => {
                                // 遇到其他类型，停止查找
                                #[cfg(debug_assertions)]
                                {
                                    eprintln!("   → [j={}] 遇到其他类型，停止查找", j);
                                }
                                break;
                            }
                        }
                    }

                    #[cfg(debug_assertions)]
                    {
                        eprintln!("   → 创建问答对: user={}, has_answer={}",
                            &msg.uuid[..8.min(msg.uuid.len())],
                            answer.is_some()
                        );
                    }
                    qa_pairs.push(QAPair {
                        question: msg.clone(),
                        answer,
                        timestamp: msg.timestamp.clone(),
                    });
                }
                _ => {
                    // 其他类型，跳过
                }
            }
        }

        // 从后向前扫描得到的结果是倒序的，需要反转回来
        qa_pairs.reverse();

        #[cfg(debug_assertions)]
        {
            eprintln!("🔍 [extract_qa_pairs] 输出问答对数量: {}", qa_pairs.len());
        }

        qa_pairs
    }

    /// 获取当前等级
    pub fn view_level(&self) -> ViewLevel {
        self.view_level
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_message(msg_type: &str, uuid: &str, parent_uuid: Option<&str>) -> Message {
        Message {
            id: None,
            session_id: "test-session".to_string(),
            uuid: uuid.to_string(),
            parent_uuid: parent_uuid.map(|s| s.to_string()),
            msg_type: msg_type.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            offset: 0,
            length: 100,
            summary: Some("test summary".to_string()),
            parent_idx: None,
            created_at: Utc::now().to_rfc3339(),
        }
    }

    fn create_test_message_with_summary(msg_type: &str, uuid: &str, summary: &str) -> Message {
        Message {
            id: None,
            session_id: "test-session".to_string(),
            uuid: uuid.to_string(),
            parent_uuid: None,
            msg_type: msg_type.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            offset: 0,
            length: 100,
            summary: Some(summary.to_string()),
            parent_idx: None,
            created_at: Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn test_view_level_default() {
        assert_eq!(ViewLevel::default(), ViewLevel::Full);
    }

    #[test]
    fn test_view_level_from_str() {
        assert_eq!(ViewLevel::from_str("full").unwrap(), ViewLevel::Full);
        assert_eq!(ViewLevel::from_str("conversation").unwrap(), ViewLevel::Conversation);
        assert_eq!(ViewLevel::from_str("qa_pairs").unwrap(), ViewLevel::QAPairs);
        assert_eq!(ViewLevel::from_str("assistant_only").unwrap(), ViewLevel::AssistantOnly);
        assert_eq!(ViewLevel::from_str("user_only").unwrap(), ViewLevel::UserOnly);
        assert!(ViewLevel::from_str("invalid").is_err());
    }

    #[test]
    fn test_filter_full_level() {
        let filter = MessageFilter::new(ViewLevel::Full);
        let user_msg = create_test_message("user", "uuid1", None);
        let assistant_msg = create_test_message("assistant", "uuid2", Some("uuid1"));
        let tool_msg = create_test_message("tool_use", "uuid3", Some("uuid2"));
        let thinking_msg = create_test_message("thinking", "uuid4", Some("uuid2"));

        assert!(filter.should_include(&user_msg));
        assert!(filter.should_include(&assistant_msg));
        assert!(filter.should_include(&tool_msg));
        assert!(filter.should_include(&thinking_msg));
    }

    #[test]
    fn test_filter_conversation_level() {
        let filter = MessageFilter::new(ViewLevel::Conversation);
        let user_msg = create_test_message("user", "uuid1", None);
        let assistant_msg = create_test_message("assistant", "uuid2", Some("uuid1"));
        let tool_msg = create_test_message("tool_use", "uuid3", Some("uuid2"));
        let thinking_msg = create_test_message("thinking", "uuid4", Some("uuid2"));

        assert!(filter.should_include(&user_msg));
        assert!(filter.should_include(&assistant_msg));
        assert!(!filter.should_include(&tool_msg)); // 工具调用被过滤
        assert!(filter.should_include(&thinking_msg));
    }

    #[test]
    fn test_filter_user_only_level() {
        let filter = MessageFilter::new(ViewLevel::UserOnly);
        let user_msg = create_test_message("user", "uuid1", None);
        let assistant_msg = create_test_message("assistant", "uuid2", Some("uuid1"));

        assert!(filter.should_include(&user_msg));
        assert!(!filter.should_include(&assistant_msg));
    }

    #[test]
    fn test_filter_user_only_level_tool_result() {
        let filter = MessageFilter::new(ViewLevel::UserOnly);

        // 普通用户消息应该被包含
        let user_msg = create_test_message("user", "uuid1", None);
        assert!(filter.should_include(&user_msg));

        // 包含 tool_result 标记的用户消息应该被过滤
        let user_msg_with_tool_result = create_test_message_with_summary(
            "user",
            "uuid2",
            r#"{"type":"tool_result","content":"some content"}"#
        );
        assert!(!filter.should_include(&user_msg_with_tool_result));

        // 包含带空格的 tool_result 标记的用户消息应该被过滤
        let user_msg_with_tool_result_spaced = create_test_message_with_summary(
            "user",
            "uuid3",
            r#"{"type": "tool_result","content":"some content"}"#
        );
        assert!(!filter.should_include(&user_msg_with_tool_result_spaced));

        // 包含 tool_result 字符串的用户消息应该被过滤
        let user_msg_with_tool_result_text = create_test_message_with_summary(
            "user",
            "uuid4",
            "some text with tool_result inside"
        );
        assert!(!filter.should_include(&user_msg_with_tool_result_text));
    }

    #[test]
    fn test_filter_assistant_only_level() {
        let filter = MessageFilter::new(ViewLevel::AssistantOnly);
        let user_msg = create_test_message("user", "uuid1", None);
        let assistant_msg = create_test_message("assistant", "uuid2", Some("uuid1"));

        assert!(!filter.should_include(&user_msg));
        assert!(filter.should_include(&assistant_msg));
    }

    #[test]
    fn test_extract_qa_pairs_simple() {
        let filter = MessageFilter::new(ViewLevel::QAPairs);
        let user_msg1 = create_test_message("user", "uuid1", None);
        let assistant_msg1 = create_test_message("assistant", "uuid2", None);
        let user_msg2 = create_test_message("user", "uuid3", None);
        let assistant_msg2 = create_test_message("assistant", "uuid4", None);

        // 顺序：user1, assistant1, user2, assistant2
        let messages = vec![user_msg1.clone(), assistant_msg1.clone(), user_msg2.clone(), assistant_msg2.clone()];
        let qa_pairs = filter.extract_qa_pairs(messages);

        // 从后向前：assistant2 -> user2, assistant1 -> user1
        assert_eq!(qa_pairs.len(), 2);
        assert_eq!(qa_pairs[0].question.uuid, user_msg1.uuid);
        assert!(qa_pairs[0].answer.is_some());
        assert_eq!(qa_pairs[0].answer.as_ref().unwrap().uuid, assistant_msg1.uuid);
        assert_eq!(qa_pairs[1].question.uuid, user_msg2.uuid);
        assert!(qa_pairs[1].answer.is_some());
        assert_eq!(qa_pairs[1].answer.as_ref().unwrap().uuid, assistant_msg2.uuid);
    }

    #[test]
    fn test_extract_qa_pairs_unmatched() {
        let filter = MessageFilter::new(ViewLevel::QAPairs);
        let user_msg1 = create_test_message("user", "uuid1", None);
        let assistant_msg1 = create_test_message("assistant", "uuid2", None);
        let user_msg2 = create_test_message("user", "uuid3", None);

        // 顺序：user1, assistant1, user2（user2 没有对应的 assistant）
        let messages = vec![user_msg1.clone(), assistant_msg1.clone(), user_msg2.clone()];
        let qa_pairs = filter.extract_qa_pairs(messages);

        // 从后向前：user2 没有答案（最后是 user），assistant1 -> user1
        assert_eq!(qa_pairs.len(), 2);
        assert_eq!(qa_pairs[0].question.uuid, user_msg1.uuid);
        assert!(qa_pairs[0].answer.is_some());
        assert_eq!(qa_pairs[0].answer.as_ref().unwrap().uuid, assistant_msg1.uuid);
        assert_eq!(qa_pairs[1].question.uuid, user_msg2.uuid);
        assert!(qa_pairs[1].answer.is_none());
    }

    #[test]
    fn test_extract_qa_pairs_with_intermediate_messages() {
        let filter = MessageFilter::new(ViewLevel::QAPairs);
        let user_msg1 = create_test_message("user", "uuid1", None);
        let thinking_msg = create_test_message("thinking", "uuid2", None);
        let user_msg2 = create_test_message("user", "uuid3", None);
        let assistant_msg = create_test_message("assistant", "uuid4", None);

        // 顺序：user1, thinking, user2, assistant
        let messages = vec![user_msg1.clone(), thinking_msg, user_msg2.clone(), assistant_msg.clone()];
        let qa_pairs = filter.extract_qa_pairs(messages);

        // 从后向前：assistant -> user2（跳过 thinking），user1 没有答案
        assert_eq!(qa_pairs.len(), 2);
        assert_eq!(qa_pairs[0].question.uuid, user_msg1.uuid);
        assert!(qa_pairs[0].answer.is_none());
        assert_eq!(qa_pairs[1].question.uuid, user_msg2.uuid);
        assert!(qa_pairs[1].answer.is_some());
        assert_eq!(qa_pairs[1].answer.as_ref().unwrap().uuid, assistant_msg.uuid);
    }

    #[test]
    fn test_extract_qa_pairs_conversation_pattern() {
        let filter = MessageFilter::new(ViewLevel::QAPairs);
        let user1 = create_test_message("user", "uuid1", None);
        let assistant1 = create_test_message("assistant", "uuid2", None);
        let user2 = create_test_message("user", "uuid3", None);
        let thinking = create_test_message("thinking", "uuid4", None);
        let assistant2 = create_test_message("assistant", "uuid5", None);

        // 典型的对话模式：user -> assistant -> user -> thinking -> assistant
        let messages = vec![user1.clone(), assistant1.clone(), user2.clone(), thinking, assistant2.clone()];
        let qa_pairs = filter.extract_qa_pairs(messages);

        // 从后向前：assistant2 -> user2（跳过 thinking），assistant1 -> user1
        assert_eq!(qa_pairs.len(), 2);
        assert_eq!(qa_pairs[0].question.uuid, user1.uuid);
        assert!(qa_pairs[0].answer.is_some());
        assert_eq!(qa_pairs[0].answer.as_ref().unwrap().uuid, assistant1.uuid);
        assert_eq!(qa_pairs[1].question.uuid, user2.uuid);
        assert!(qa_pairs[1].answer.is_some());
        assert_eq!(qa_pairs[1].answer.as_ref().unwrap().uuid, assistant2.uuid);
    }

    #[test]
    fn test_extract_qa_pairs_consecutive_assistants() {
        let filter = MessageFilter::new(ViewLevel::QAPairs);
        let user1 = create_test_message("user", "uuid1", None);
        let assistant1 = create_test_message("assistant", "uuid2", None);
        let assistant2 = create_test_message("assistant", "uuid3", None);
        let user2 = create_test_message("user", "uuid4", None);

        // 连续的 assistant：user -> assistant -> assistant -> user
        let messages = vec![user1.clone(), assistant1.clone(), assistant2.clone(), user2.clone()];
        let qa_pairs = filter.extract_qa_pairs(messages);

        // 从后向前：user2 没有答案，连续的 assistant 只取最后一个（assistant2）-> user1
        assert_eq!(qa_pairs.len(), 2);
        assert_eq!(qa_pairs[0].question.uuid, user1.uuid);
        assert!(qa_pairs[0].answer.is_some());
        assert_eq!(qa_pairs[0].answer.as_ref().unwrap().uuid, assistant2.uuid); // 注意是 assistant2
        assert_eq!(qa_pairs[1].question.uuid, user2.uuid);
        assert!(qa_pairs[1].answer.is_none());
    }

    #[test]
    fn test_message_order_preservation() {
        let filter = MessageFilter::new(ViewLevel::Conversation);
        let msg1 = create_test_message("user", "uuid1", None);
        let msg2 = create_test_message("assistant", "uuid2", Some("uuid1"));
        let msg3 = create_test_message("tool_use", "uuid3", Some("uuid2"));
        let msg4 = create_test_message("user", "uuid4", Some("uuid2"));

        let messages = vec![msg1.clone(), msg2.clone(), msg3.clone(), msg4.clone()];
        let filtered = filter.filter_messages(messages);

        assert_eq!(filtered.len(), 3);
        assert_eq!(filtered[0].uuid, msg1.uuid);
        assert_eq!(filtered[1].uuid, msg2.uuid);
        assert_eq!(filtered[2].uuid, msg4.uuid); // tool_use 被过滤
    }
}
