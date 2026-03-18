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

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::database::models::Message;

// ==================== 辅助函数 ====================

/// 安全地将字符串截断到指定字符数
///
/// 确保截断位置不会落在 UTF-8 多字节字符的中间
///
/// # 参数
/// * `s` - 要截断的字符串
/// * `max_chars` - 最大字符数
///
/// # 返回
/// 截断后的字符串（不会超过字符边界）
fn truncate_str_to_chars(s: &str, max_chars: usize) -> String {
    // 使用 chars() 迭代器按字符计数
    s.chars().take(max_chars).collect()
}

/// 日志读取等级
///
/// 定义五种不同的日志读取等级，按信息完整度排序。
/// 默认值为 QAPairs，提取用户问题和助手最终回复的配对。
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
        ViewLevel::QAPairs
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
                matches!(message.msg_type.as_str(), "user" | "assistant" | "thinking")
            }
            ViewLevel::QAPairs => {
                // QAPairs 需要特殊处理，在 extract_qa_pairs 中实现
                false
            }
            ViewLevel::AssistantOnly => message.msg_type == "assistant",
            ViewLevel::UserOnly => {
                // UserOnly: 只包含 user 类型，且排除 tool_result 类型
                message.msg_type == "user" && !self.is_tool_result_message(message)
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
    /// **步骤 1: 预过滤**
    ///
    /// 首先移除不适合问答对的消息：
    /// - user 类型的 tool_result 消息
    /// - assistant 类型的 tool_use 消息
    /// - 包含 <tool_use_error> 或 <system-reminder> 的消息
    /// - system 类型的消息
    ///
    /// **步骤 2: 合并用户消息与技能消息**
    ///
    /// 将用户消息与其直系技能子消息合并：
    /// - 识别技能消息：内容包含 "Base directory for this skill"
    /// - 合并策略：将技能内容追加到用户消息，用分隔符间隔
    /// - 合并后从列表中移除技能消息
    ///
    /// **步骤 3: 从后向前扫描配对**
    ///
    /// 1. 从后向前扫描合并后的消息列表
    /// 2. 遇到 user 时，向后查找第一个 assistant 作为答案
    /// 3. 如果遇到另一个 user，停止查找（该 user 没有答案）
    ///
    /// # 示例
    ///
    /// ```text
    /// 原始消息序列:
    /// 1:  user (text)              → 保留
    /// 2:  assistant (text)         → 保留，配对给 1
    /// 3:  assistant (tool_use)     → 过滤
    /// 4:  user (tool_result)       → 过滤
    /// 5:  assistant (text)         → 保留，但没有对应的 question，忽略
    /// 6:  assistant (tool_use)     → 过滤
    /// 7:  user (tool_result)       → 过滤
    /// 8:  assistant (tool_use)     → 过滤
    /// 9:  user (text)              → 保留
    /// 10: user (tool_result)       → 过滤
    /// 11: assistant (text)         → 保留，配对给 9
    /// 12: user (text)              → 保留，无答案
    ///
    /// 过滤后序列: [1(user), 2(assistant), 5(assistant), 9(user), 11(assistant), 12(user)]
    /// 问答对: [(1, 2), (9, 11), (12, None)]
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
        // 步骤 1: 预过滤，移除不适合问答对的消息
        let filtered_messages = self.pre_filter_for_qa(messages);

        // 步骤 2: 合并用户消息与技能消息
        let merged_messages = self.merge_user_with_skill_messages(filtered_messages);

        let mut qa_pairs = Vec::new();

        // 步骤 3: 从后向前扫描，配对 user 和 assistant
        let mut i = merged_messages.len();
        while i > 0 {
            i -= 1;
            let msg = &merged_messages[i];

            match msg.msg_type.as_str() {
                "user" => {
                    // 找到一个 user，向后查找第一个 assistant 作为答案
                    let mut answer: Option<Message> = None;

                    // 从当前 user 之后开始向后找 assistant
                    let mut j = i + 1;
                    while j < merged_messages.len() {
                        let next_msg = &merged_messages[j];
                        match next_msg.msg_type.as_str() {
                            "assistant" => {
                                // 找到 assistant，记录为答案
                                answer = Some(next_msg.clone());
                                // 继续查找，可能有更合适的 assistant
                                j += 1;
                            }
                            "user" => {
                                // 遇到新的 user，停止查找
                                break;
                            }
                            _ => {
                                // 遇到其他类型，停止查找
                                break;
                            }
                        }
                    }

                    // 只有当找到答案时才创建问答对
                    if answer.is_some() {
                        qa_pairs.push(QAPair {
                            question: msg.clone(),
                            answer,
                            timestamp: msg.timestamp.clone(),
                        });
                    }
                }
                _ => {
                    // 其他类型（如 assistant），跳过
                    // 这些消息会在找到对应的 user 时作为答案被处理
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

    // ========== 问答对过滤辅助方法 ==========

    /// 检测消息是否为 tool_use 类型
    ///
    /// 使用 content_type 字段进行准确判断
    fn is_tool_use_message(&self, msg: &Message) -> bool {
        // 使用 content_type 字段
        if let Some(ref content_type) = msg.content_type {
            return content_type == "tool_use";
        }

        // 如果没有 content_type，尝试从 summary 解析
        if let Some(ref summary) = msg.summary {
            // 尝试解析 JSON
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(summary) {
                // 检查 message.content 数组
                if let Some(content) = parsed.get("content").and_then(|v| v.as_array()) {
                    if !content.is_empty() {
                        if let Some(content_type) = content[0].get("type").and_then(|v| v.as_str())
                        {
                            return content_type == "tool_use";
                        }
                    }
                }
            }
        }

        false
    }

    /// 检测消息是否为 tool_result 类型
    ///
    /// 使用 content_type 字段进行准确判断
    fn is_tool_result_message(&self, msg: &Message) -> bool {
        // 使用 content_type 字段
        if let Some(ref content_type) = msg.content_type {
            return content_type == "tool_result";
        }

        // 如果没有 content_type，尝试从 summary 解析
        if let Some(ref summary) = msg.summary {
            // 尝试解析 JSON
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(summary) {
                // 首先检查顶层的 type 字段
                if let Some(type_val) = parsed.get("type").and_then(|v| v.as_str()) {
                    if type_val == "tool_result" {
                        return true;
                    }
                }

                // 检查 message.content 数组
                if let Some(content) = parsed.get("content").and_then(|v| v.as_array()) {
                    if !content.is_empty() {
                        if let Some(content_type) = content[0].get("type").and_then(|v| v.as_str())
                        {
                            return content_type == "tool_result";
                        }
                    }
                }
            }

            // 回退：检查 summary 中是否包含 "tool_result" 字符串
            // 注意：这是一个简单的启发式方法，可能会误判
            if summary.contains("tool_result") {
                return true;
            }
        }

        false
    }

    /// 检测消息是否包含系统标签
    ///
    /// 检查消息内容是否包含 <tool_use_error> 或 <system-reminder> 标签。
    fn contains_system_tags(&self, msg: &Message) -> bool {
        if let Some(ref summary) = msg.summary {
            return summary.contains("<tool_use_error>") || summary.contains("<system-reminder>");
        }
        false
    }

    /// 预过滤消息列表用于问答对提取
    ///
    /// 移除不适合问答对的消息：
    /// - user 类型的 tool_result 消息
    /// - assistant 类型的 tool_use 消息
    /// - 包含系统标签的消息
    /// - system 类型的消息
    /// - 被内容过滤规则标记的消息（通过 FilterConfigManager）
    ///
    /// # 参数
    ///
    /// - `messages`: 原始消息列表
    ///
    /// # 返回
    ///
    /// 过滤后的消息列表，只包含适合问答对的消息
    fn pre_filter_for_qa(&self, messages: Vec<Message>) -> Vec<Message> {
        use crate::filter_config::FilterConfigManager;

        // 在迭代器之前创建一次 FilterConfigManager，避免为每条消息重复创建
        let filter_mgr = FilterConfigManager::with_default_path().ok();

        messages.into_iter()
            .filter(|msg| {
                // ========== 内容过滤检查（集成 FilterConfigManager）==========
                // 如果消息有 summary，先应用内容过滤规则
                if let Some(ref summary) = msg.summary {
                    // 复用已创建的 manager
                    if let Some(ref manager) = filter_mgr {
                        if manager.should_filter(summary) {
                            #[cfg(debug_assertions)]
                            {
                                // 安全地截断字符串到字符边界
                                let uuid_preview = truncate_str_to_chars(&msg.uuid, 8);
                                let summary_preview = truncate_str_to_chars(summary, 50);
                                eprintln!("[pre_filter_for_qa] 消息被内容过滤规则排除: uuid={}, summary={:?}",
                                    uuid_preview,
                                    summary_preview
                                );
                            }
                            return false;
                        }
                    }
                }

                // ========== 原有过滤逻辑 ==========
                // 保留 user 类型的消息，但排除 tool_result 和包含系统标签的
                if msg.msg_type == "user" {
                    return !self.is_tool_result_message(msg) && !self.contains_system_tags(msg);
                }

                // 保留 assistant 类型的消息，但排除 tool_use 和包含系统标签的
                if msg.msg_type == "assistant" {
                    return !self.is_tool_use_message(msg) && !self.contains_system_tags(msg);
                }

                // 过滤掉其他类型（system, tool_use, tool_result 等）
                false
            })
            .collect()
    }

    // ========== 消息合并方法 ==========

    /// 合并用户消息与技能消息
    ///
    /// 将用户消息及其直系技能子消息合并为一条消息。
    ///
    /// # 算法
    ///
    /// 1. 遍历消息列表
    /// 2. 当遇到 user 消息时，查找其直系技能子消息（parentUuid == user.uuid）
    /// 3. 如果找到技能消息，合并其内容到用户消息
    /// 4. 从列表中移除已合并的技能消息
    ///
    /// # 参数
    ///
    /// - `messages`: 预过滤后的消息列表
    ///
    /// # 返回
    ///
    /// 合并后的消息列表（技能消息已被移除）
    fn merge_user_with_skill_messages(&self, messages: Vec<Message>) -> Vec<Message> {
        use std::collections::HashSet;

        #[cfg(debug_assertions)]
        {
            eprintln!("🔗 [merge_user_with_skill_messages] 开始合并，原始消息数: {}", messages.len());
        }

        let mut result = Vec::new();
        let mut merged_uuids = HashSet::new();

        for msg in &messages {
            // 如果这条消息已经被合并过，跳过
            if merged_uuids.contains(&msg.uuid) {
                continue;
            }

            if msg.msg_type == "user" {
                // 查找该用户消息的直系技能子消息
                let skill_children: Vec<&Message> = messages
                    .iter()
                    .filter(|m| {
                        // parentUuid 指向当前用户消息
                        m.parent_uuid.as_ref() == Some(&msg.uuid)
                        // 且是技能消息
                        && self.is_skill_invocation_message(m)
                    })
                    .collect();

                if !skill_children.is_empty() {
                    // 按原始顺序（timestamp）排序技能消息
                    let mut sorted_skills = skill_children;
                    sorted_skills.sort_by_key(|m| &m.timestamp);

                    // 合并内容
                    let merged_content = self.merge_message_contents(msg, &sorted_skills);

                    // 创建合并后的消息
                    let mut merged_msg = msg.clone();
                    merged_msg.summary = Some(merged_content.clone());
                    merged_msg.content = Some(merged_content);

                    #[cfg(debug_assertions)]
                    {
                        let user_uuid_preview = truncate_str_to_chars(&msg.uuid, 8);
                        eprintln!(
                            "  ✅ 合并用户消息 {} 与 {} 个技能消息",
                            user_uuid_preview,
                            sorted_skills.len()
                        );
                    }

                    // 标记技能消息为已合并
                    for skill in &sorted_skills {
                        merged_uuids.insert(skill.uuid.clone());

                        #[cfg(debug_assertions)]
                        {
                            let skill_uuid_preview = truncate_str_to_chars(&skill.uuid, 8);
                            eprintln!("    - 合并技能消息: {}", skill_uuid_preview);
                        }
                    }

                    result.push(merged_msg);
                } else {
                    // 没有技能子消息，直接添加原消息
                    result.push(msg.clone());
                }
            } else if msg.msg_type == "assistant" {
                // assistant 消息直接添加（不会被合并）
                result.push(msg.clone());
            }
            // 技能消息不添加（已被合并）
        }

        #[cfg(debug_assertions)]
        {
            eprintln!(
                "🔗 [merge_user_with_skill_messages] 合并完成，最终消息数: {}",
                result.len()
            );
        }

        result
    }

    /// 合并消息内容
    ///
    /// 将用户消息和技能消息的内容合并为一个格式化的字符串。
    /// 使用分隔符间隔正文内容和技能内容。
    ///
    /// # 参数
    ///
    /// - `user_msg`: 用户消息
    /// - `skill_msgs`: 技能消息列表（已按 timestamp 排序）
    ///
    /// # 返回
    ///
    /// 合并后的内容字符串
    fn merge_message_contents(&self, user_msg: &Message, skill_msgs: &[&Message]) -> String {
        let mut parts = Vec::new();

        // 添加用户消息内容
        if let Some(ref user_content) = user_msg.summary {
            parts.push(user_content.clone());
        }

        // 添加技能消息内容（使用分隔符）
        for skill_msg in skill_msgs {
            if let Some(ref skill_content) = skill_msg.summary {
                // 使用明显的分隔符标记技能内容
                parts.push(format!("\n\n---\n\n{}", skill_content));
            }
        }

        parts.join("")
    }

    /// 判断是否为技能调用消息
    ///
    /// 通过内容特征识别技能消息。
    ///
    /// # 识别特征
    ///
    /// - 内容包含 "Base directory for this skill"
    ///
    /// # 参数
    ///
    /// - `msg`: 要判断的消息
    ///
    /// # 返回
    ///
    /// - `true`: 是技能调用消息
    /// - `false`: 不是技能调用消息
    fn is_skill_invocation_message(&self, msg: &Message) -> bool {
        // 检查 summary 字段
        if let Some(ref summary) = msg.summary {
            if summary.contains("Base directory for this skill") {
                return true;
            }
        }

        // 检查 content 字段
        if let Some(ref content) = msg.content {
            if content.contains("Base directory for this skill") {
                return true;
            }
        }

        false
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
            content_type: None,
            timestamp: Utc::now().to_rfc3339(),
            offset: 0,
            length: 100,
            summary: Some("test summary".to_string()),
            content: Some("test content".to_string()),
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
            content_type: None,
            timestamp: Utc::now().to_rfc3339(),
            offset: 0,
            length: 100,
            summary: Some(summary.to_string()),
            content: Some("test content".to_string()),
            parent_idx: None,
            created_at: Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn test_view_level_default() {
        assert_eq!(ViewLevel::default(), ViewLevel::QAPairs);
    }

    #[test]
    fn test_view_level_from_str() {
        assert_eq!(ViewLevel::from_str("full").unwrap(), ViewLevel::Full);
        assert_eq!(
            ViewLevel::from_str("conversation").unwrap(),
            ViewLevel::Conversation
        );
        assert_eq!(ViewLevel::from_str("qa_pairs").unwrap(), ViewLevel::QAPairs);

        // 新增：测试技能消息识别
        let filter = MessageFilter::new(ViewLevel::QAPairs);

        // 测试：包含 "Base directory for this skill" 的消息是技能消息
        let skill_msg = create_test_message_with_summary(
            "user",
            "skill-uuid",
            "Base directory for this skill: /path/to/skill"
        );
        assert!(filter.is_skill_invocation_message(&skill_msg));

        // 测试：普通用户消息不是技能消息
        let user_msg = create_test_message_with_summary(
            "user",
            "user-uuid",
            "Help me implement feature X"
        );
        assert!(!filter.is_skill_invocation_message(&user_msg));
    }

    #[test]
    fn test_merge_user_with_skill_messages() {
        let filter = MessageFilter::new(ViewLevel::QAPairs);

        // 创建测试消息：
        // 1. 用户消息
        // 2. 技能消息（parentUuid = 用户消息UUID）
        // 3. 助手回复
        let user_msg = create_test_message_with_summary("user", "user-123", "Help me code");
        let skill_msg = Message {
            uuid: "skill-456".to_string(),
            parent_uuid: Some("user-123".to_string()),
            msg_type: "user".to_string(), // 技能消息可能被标记为 user 类型
            summary: Some("Base directory for this skill: /path".to_string()),
            ..create_test_message("user", "skill-456", Some("user-123"))
        };
        let assistant_msg = create_test_message_with_summary("assistant", "assistant-789", "Sure, I'll help");

        let messages = vec![user_msg.clone(), skill_msg, assistant_msg.clone()];

        // 执行合并
        let merged = filter.merge_user_with_skill_messages(messages);

        // 验证：
        // 1. 技能消息被移除
        // 2. 用户消息内容包含原文和技能内容
        // 3. 助手消息保留
        assert_eq!(merged.len(), 2); // 用户消息(合并后) + 助手消息

        // 检查第一条是合并后的用户消息
        assert_eq!(merged[0].uuid, "user-123");
        assert!(merged[0].summary.as_ref().unwrap().contains("Help me code"));
        assert!(merged[0].summary.as_ref().unwrap().contains("Base directory for this skill"));

        // 检查第二条是助手消息
        assert_eq!(merged[1].uuid, "assistant-789");
    }

    #[test]
    fn test_extract_qa_pairs_with_skill_merge() {
        let filter = MessageFilter::new(ViewLevel::QAPairs);

        // 创建完整的测试场景：
        // 用户消息 -> 技能消息 -> 助手回复
        let user_msg = create_test_message_with_summary("user", "user-123", "Help me code");
        let skill_msg = Message {
            uuid: "skill-456".to_string(),
            parent_uuid: Some("user-123".to_string()),
            msg_type: "user".to_string(),
            summary: Some("Base directory for this skill: /path".to_string()),
            ..create_test_message("user", "skill-456", Some("user-123"))
        };
        let assistant_msg = create_test_message_with_summary("assistant", "assistant-789", "Sure, I'll help");

        let messages = vec![user_msg, skill_msg, assistant_msg];

        // 提取问答对
        let qa_pairs = filter.extract_qa_pairs(messages);

        // 验证：应该生成一个问答对
        assert_eq!(qa_pairs.len(), 1);

        // 验证：问题是合并后的用户消息
        assert_eq!(qa_pairs[0].question.uuid, "user-123");
        assert!(qa_pairs[0].question.summary.as_ref().unwrap().contains("Help me code"));
        assert!(qa_pairs[0].question.summary.as_ref().unwrap().contains("Base directory for this skill"));

        // 验证：答案是助手回复
        assert!(qa_pairs[0].answer.is_some());
        assert_eq!(qa_pairs[0].answer.as_ref().unwrap().uuid, "assistant-789");
    }

    #[test]
    fn test_view_level_from_str_other_values() {
        assert_eq!(
            ViewLevel::from_str("assistant_only").unwrap(),
            ViewLevel::AssistantOnly
        );
        assert_eq!(
            ViewLevel::from_str("user_only").unwrap(),
            ViewLevel::UserOnly
        );
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
            r#"{"type":"tool_result","content":"some content"}"#,
        );
        assert!(!filter.should_include(&user_msg_with_tool_result));

        // 包含带空格的 tool_result 标记的用户消息应该被过滤
        let user_msg_with_tool_result_spaced = create_test_message_with_summary(
            "user",
            "uuid3",
            r#"{"type": "tool_result","content":"some content"}"#,
        );
        assert!(!filter.should_include(&user_msg_with_tool_result_spaced));

        // 包含 tool_result 字符串的用户消息应该被过滤
        let user_msg_with_tool_result_text =
            create_test_message_with_summary("user", "uuid4", "some text with tool_result inside");
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
        let messages = vec![
            user_msg1.clone(),
            assistant_msg1.clone(),
            user_msg2.clone(),
            assistant_msg2.clone(),
        ];
        let qa_pairs = filter.extract_qa_pairs(messages);

        // 从后向前：assistant2 -> user2, assistant1 -> user1
        assert_eq!(qa_pairs.len(), 2);
        assert_eq!(qa_pairs[0].question.uuid, user_msg1.uuid);
        assert!(qa_pairs[0].answer.is_some());
        assert_eq!(
            qa_pairs[0].answer.as_ref().unwrap().uuid,
            assistant_msg1.uuid
        );
        assert_eq!(qa_pairs[1].question.uuid, user_msg2.uuid);
        assert!(qa_pairs[1].answer.is_some());
        assert_eq!(
            qa_pairs[1].answer.as_ref().unwrap().uuid,
            assistant_msg2.uuid
        );
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

        // 从后向前：user2 没有答案（不创建问答对），assistant1 -> user1
        // 只有有答案的用户消息才会创建问答对
        assert_eq!(qa_pairs.len(), 1);
        assert_eq!(qa_pairs[0].question.uuid, user_msg1.uuid);
        assert!(qa_pairs[0].answer.is_some());
        assert_eq!(
            qa_pairs[0].answer.as_ref().unwrap().uuid,
            assistant_msg1.uuid
        );
    }

    #[test]
    fn test_extract_qa_pairs_with_intermediate_messages() {
        let filter = MessageFilter::new(ViewLevel::QAPairs);
        let user_msg1 = create_test_message("user", "uuid1", None);
        let thinking_msg = create_test_message("thinking", "uuid2", None);
        let user_msg2 = create_test_message("user", "uuid3", None);
        let assistant_msg = create_test_message("assistant", "uuid4", None);

        // 顺序：user1, thinking, user2, assistant
        let messages = vec![
            user_msg1.clone(),
            thinking_msg,
            user_msg2.clone(),
            assistant_msg.clone(),
        ];
        let qa_pairs = filter.extract_qa_pairs(messages);

        // 新逻辑：thinking 被预过滤，过滤后序列：[user1, user2, assistant]
        // 从后向前：assistant -> user2（配对），user1 -> 无答案（不创建问答对）
        // 只有有答案的用户消息才会创建问答对
        assert_eq!(qa_pairs.len(), 1);
        assert_eq!(qa_pairs[0].question.uuid, user_msg2.uuid);
        assert!(qa_pairs[0].answer.is_some());
        assert_eq!(
            qa_pairs[0].answer.as_ref().unwrap().uuid,
            assistant_msg.uuid
        );
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
        let messages = vec![
            user1.clone(),
            assistant1.clone(),
            user2.clone(),
            thinking,
            assistant2.clone(),
        ];
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
        let messages = vec![
            user1.clone(),
            assistant1.clone(),
            assistant2.clone(),
            user2.clone(),
        ];
        let qa_pairs = filter.extract_qa_pairs(messages);

        // 从后向前：user2 没有答案（不创建问答对），连续的 assistant 只取最后一个（assistant2）-> user1
        // 只有有答案的用户消息才会创建问答对
        assert_eq!(qa_pairs.len(), 1);
        assert_eq!(qa_pairs[0].question.uuid, user1.uuid);
        assert!(qa_pairs[0].answer.is_some());
        assert_eq!(qa_pairs[0].answer.as_ref().unwrap().uuid, assistant2.uuid); // 注意是 assistant2
    }

    #[test]
    fn test_extract_qa_pairs_skip_intermediate_users() {
        let filter = MessageFilter::new(ViewLevel::QAPairs);
        let user1 = create_test_message("user", "uuid1", None);
        let user2 = create_test_message("user", "uuid2", None);
        let user3 = create_test_message("user", "uuid3", None);
        let assistant1 = create_test_message("assistant", "uuid4", None);
        let assistant2 = create_test_message("assistant", "uuid5", None);

        // 多个 user 后跟多个 assistant：user -> user -> user -> assistant -> assistant
        let messages = vec![
            user1.clone(),
            user2.clone(),
            user3.clone(),
            assistant1.clone(),
            assistant2.clone(),
        ];
        let qa_pairs = filter.extract_qa_pairs(messages);

        // 新逻辑：只有最后一个 user3 才能配对到 assistant，user1 和 user2 无答案
        // 过滤后序列：[user1, user2, user3, assistant1, assistant2]
        // 从后向前：assistant2 -> user3（配对），user2 -> 无答案（不创建问答对），user1 -> 无答案（不创建问答对）
        // 只有有答案的用户消息才会创建问答对
        assert_eq!(qa_pairs.len(), 1);
        assert_eq!(qa_pairs[0].question.uuid, user3.uuid);
        assert!(qa_pairs[0].answer.is_some());
    }

    #[test]
    fn test_extract_qa_pairs_mixed_pattern() {
        let filter = MessageFilter::new(ViewLevel::QAPairs);
        let user1 = create_test_message("user", "uuid1", None);
        let assistant1 = create_test_message("assistant", "uuid2", None);
        let user2 = create_test_message("user", "uuid3", None);
        let user3 = create_test_message("user", "uuid4", None);
        let thinking = create_test_message("thinking", "uuid5", None);
        let assistant2 = create_test_message("assistant", "uuid6", None);

        // 混合模式：user -> assistant -> user -> user -> thinking -> assistant
        let messages = vec![
            user1.clone(),
            assistant1.clone(),
            user2.clone(),
            user3.clone(),
            thinking,
            assistant2.clone(),
        ];
        let qa_pairs = filter.extract_qa_pairs(messages);

        // 新逻辑：thinking 被预过滤，过滤后序列：[user1, assistant1, user2, user3, assistant2]
        // 从后向前：assistant2 -> user3（配对），user2 -> 无答案（遇到 user3，不创建问答对）
        // assistant1 -> user1（配对）
        // 只有有答案的用户消息才会创建问答对
        assert_eq!(qa_pairs.len(), 2);
        assert_eq!(qa_pairs[0].question.uuid, user1.uuid);
        assert!(qa_pairs[0].answer.is_some());
        assert_eq!(qa_pairs[0].answer.as_ref().unwrap().uuid, assistant1.uuid);

        assert_eq!(qa_pairs[1].question.uuid, user3.uuid);
        assert!(qa_pairs[1].answer.is_some());
        assert_eq!(qa_pairs[1].answer.as_ref().unwrap().uuid, assistant2.uuid);
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
