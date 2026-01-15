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
            ViewLevel::UserOnly => message.msg_type == "user",
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
    /// 从消息列表中提取问答配对。通过 parentUuid 追踪消息链，
    /// 为每个用户消息找到最终的 assistant 回复。
    ///
    /// # 参数
    ///
    /// - `messages`: 原始消息列表
    ///
    /// # 返回
    ///
    /// 问答对列表
    ///
    /// # 算法
    ///
    /// 1. 建立 uuid -> message 的映射
    /// 2. 遍历所有用户消息
    /// 3. 对每个用户消息，沿着消息链向下查找
    /// 4. 选择链中最后一个 assistant 消息作为答案
    /// 5. 如果未找到 assistant 回复，answer 字段为 None
    ///
    /// # 复杂度
    ///
    /// - 时间复杂度: O(n * d)，n 为消息数量，d 为消息链的平均深度
    /// - 空间复杂度: O(n)，用于存储 uuid 映射
    pub fn extract_qa_pairs(&self, messages: Vec<Message>) -> Vec<QAPair> {
        // 构建 uuid -> message 的映射
        let uuid_map: HashMap<String, Message> = messages
            .iter()
            .map(|msg| (msg.uuid.clone(), msg.clone()))
            .collect();

        // 构建 uuid -> children 的映射
        let mut children_map: HashMap<String, Vec<Message>> = HashMap::new();
        for msg in &messages {
            if let Some(parent_uuid) = &msg.parent_uuid {
                children_map
                    .entry(parent_uuid.clone())
                    .or_insert_with(Vec::new)
                    .push(msg.clone());
            }
        }

        let mut qa_pairs = Vec::new();

        // 遍历所有用户消息
        for user_msg in messages.iter().filter(|msg| msg.msg_type == "user") {
            // 查找该用户消息对应的最终 assistant 回复
            let answer = self.find_final_answer(&user_msg.uuid, &children_map, &uuid_map);

            qa_pairs.push(QAPair {
                question: user_msg.clone(),
                answer,
                timestamp: user_msg.timestamp.clone(),
            });
        }

        qa_pairs
    }

    /// 查找用户消息的最终答案
    ///
    /// 沿着消息链向下查找，返回最后一个 assistant 类型的消息。
    ///
    /// # 参数
    ///
    /// - `uuid`: 当前消息的 UUID
    /// - `children_map`: 子消息映射
    /// - `uuid_map`: UUID 到消息的映射
    ///
    /// # 返回
    ///
    /// 最终的 assistant 消息，如果未找到则返回 None
    fn find_final_answer(
        &self,
        uuid: &str,
        children_map: &HashMap<String, Vec<Message>>,
        uuid_map: &HashMap<String, Message>,
    ) -> Option<Message> {
        let mut current_uuid = uuid.to_string();
        let mut last_assistant: Option<Message> = None;

        // 沿着消息链向下查找
        loop {
            if let Some(children) = children_map.get(&current_uuid) {
                // 查找子消息中的 assistant 类型
                for child in children {
                    if child.msg_type == "assistant" {
                        last_assistant = Some(child.clone());
                        current_uuid = child.uuid.clone();
                        break; // 找到 assistant 后继续沿该分支向下
                    }
                }

                // 如果没有找到 assistant，停止查找
                if last_assistant.is_none() {
                    break;
                }
            } else {
                // 没有子消息，停止查找
                break;
            }
        }

        last_assistant
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
        let user_msg = create_test_message("user", "uuid1", None);
        let assistant_msg = create_test_message("assistant", "uuid2", Some("uuid1"));

        let messages = vec![user_msg.clone(), assistant_msg.clone()];
        let qa_pairs = filter.extract_qa_pairs(messages);

        assert_eq!(qa_pairs.len(), 1);
        assert_eq!(qa_pairs[0].question.uuid, user_msg.uuid);
        assert!(qa_pairs[0].answer.is_some());
        assert_eq!(qa_pairs[0].answer.as_ref().unwrap().uuid, assistant_msg.uuid);
    }

    #[test]
    fn test_extract_qa_pairs_unmatched() {
        let filter = MessageFilter::new(ViewLevel::QAPairs);
        let user_msg = create_test_message("user", "uuid1", None);

        let messages = vec![user_msg.clone()];
        let qa_pairs = filter.extract_qa_pairs(messages);

        assert_eq!(qa_pairs.len(), 1);
        assert_eq!(qa_pairs[0].question.uuid, user_msg.uuid);
        assert!(qa_pairs[0].answer.is_none()); // 没有找到回复
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
