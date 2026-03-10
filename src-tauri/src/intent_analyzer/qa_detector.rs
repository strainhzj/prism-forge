//! 问答对检测器
//!
//! 用于检测会话中的"决策问答对"（助手回答 + 用户后续决策）
//!
//! ## 问答对提取逻辑（v1.0.3 新设计）
//!
//! **输入**: `[user1, assistant1, user2, assistant2, user3, ...]`
//! **输出**: `[(assistant1, user2), (assistant2, user3), ...]`
//!
//! **目的**: 分析**用户基于助手回答做出的决策**
//!
//! ## 示例
//!
//! ```text
//! user1 (开场白) → assistant1 → user2 (基于助手回答的决策) → assistant2 → user3 (决策)
//!                      ↓                      ↓
//!                 配对起点：              配对1：
//!               (assistant1, user2)      (assistant2, user3)
//! ```
//!
//! ## 与 v1.0.2 的区别
//!
//! - **v1.0.2 逻辑**: `(user1, assistant1), (user2, assistant2), ...`
//! - **v1.0.3 逻辑**: 排除开场白，配对 `(assistant1, user2), (assistant2, user3), ...`

use crate::database::models::Message;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 前序问答对上下文
///
/// 用于存储当前决策之前的历史问答对
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct QAPairContext {
    /// 用户问题内容
    pub user_question: String,
    /// 助手回答内容
    pub assistant_answer: String,
}

/// 决策问答对（助手回答 + 用户后续决策）
///
/// 表示用户基于助手回答做出的决策记录
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct DecisionQAPair {
    /// 问答对索引（从 0 开始）
    pub qa_index: usize,

    /// 助手回答的 UUID
    pub assistant_answer_uuid: String,

    /// 用户决策的 UUID
    pub user_decision_uuid: String,

    /// 助手回答内容
    pub assistant_answer: String,

    /// 用户决策内容
    pub user_decision: String,

    /// 前序问答对上下文（决策之前的历史对话）
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub context_qa_pairs: Option<Vec<QAPairContext>>,
}

/// 问答对检测器
///
/// 用于从会话消息序列中提取"决策问答对"
pub struct QAPairDetector;

impl QAPairDetector {
    /// 创建新的检测器实例
    pub fn new() -> Self {
        Self
    }

    /// 检测决策问答对（排除开场白），并附带前序上下文
    ///
    /// # 参数
    ///
    /// - `messages`: 完整的会话消息序列
    ///
    /// # 返回
    ///
    /// 决策问答对列表（每个问答对包含前序上下文）
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let detector = QAPairDetector::new();
    /// let messages = vec![
    ///     Message { uuid: "u1".into(), msg_type: "user".into(), content: Some("开场白".into()), ..Default::default() },
    ///     Message { uuid: "a1".into(), msg_type: "assistant".into(), content: Some("回答1".into()), ..Default::default() },
    ///     Message { uuid: "u2".into(), msg_type: "user".into(), content: Some("用户决策1".into()), ..Default::default() },
    /// ];
    /// let pairs = detector.detect_decision_qa_pairs(messages);
    /// assert_eq!(pairs.len(), 1);
    /// assert_eq!(pairs[0].assistant_answer_uuid, "a1");
    /// assert_eq!(pairs[0].user_decision_uuid, "u2");
    /// ```
    ///
    /// # 逻辑说明
    ///
    /// - 从第二个消息开始（跳过 user1 开场白）
    /// - 配对模式: `(assistant_i, user_{i+1})`
    /// - 只提取 assistant → user 的配对
    /// - 跳过 content 为 None 的消息
    /// - 为每个问答对添加前序上下文（最多 5 对）
    pub fn detect_decision_qa_pairs(&self, messages: Vec<Message>) -> Vec<DecisionQAPair> {
        let mut pairs = Vec::new();
        let mut i = 1; // 从第二个消息开始（跳过 user1 开场白）

        while i + 1 < messages.len() {
            let assistant = &messages[i];
            let user = &messages[i + 1];

            // 确保: assistant 消息 → user 消息
            if assistant.msg_type == "assistant" && user.msg_type == "user" {
                if let (Some(assistant_content), Some(user_content)) =
                    (&assistant.content, &user.content)
                {
                    // 构建前序上下文（当前问答对之前的历史对话）
                    let context_qa_pairs = self.build_context_qa_pairs(&messages, i);

                    pairs.push(DecisionQAPair {
                        qa_index: pairs.len(),
                        assistant_answer_uuid: assistant.uuid.clone(),
                        user_decision_uuid: user.uuid.clone(),
                        assistant_answer: assistant_content.clone(),
                        user_decision: user_content.clone(),
                        context_qa_pairs: if context_qa_pairs.is_empty() {
                            None
                        } else {
                            Some(context_qa_pairs)
                        },
                    });
                }
            }

            i += 2; // 跳到下一个 assistant-user 对
        }

        pairs
    }

    /// 构建前序问答对上下文
    ///
    /// 提取指定位置之前的所有问答对（最多 5 对）
    ///
    /// # 参数
    ///
    /// - `messages`: 完整的消息序列
    /// - `current_index`: 当前 assistant 消息的索引
    ///
    /// # 返回
    ///
    /// 前序问答对列表（按时间正序）
    fn build_context_qa_pairs(&self, messages: &[Message], current_index: usize) -> Vec<QAPairContext> {
        let mut context_pairs = Vec::new();
        let mut i = 1; // 从第二个消息开始（跳过开场白）

        // 限制上下文数量（最多 5 对）
        let max_context_pairs = 5;
        let mut pair_count = 0;

        while i < current_index && pair_count < max_context_pairs {
            // 检查是否是有效的 assistant-user 配对
            if i + 1 >= messages.len() {
                break;
            }

            let assistant = &messages[i];
            let user = &messages[i + 1];

            if assistant.msg_type == "assistant" && user.msg_type == "user" {
                if let (Some(assistant_content), Some(user_content)) =
                    (&assistant.content, &user.content)
                {
                    // 注意：这里存储的是 "用户问题 → 助手回答" 的顺序
                    // 因为原始消息序列是 user1 → assistant1 → user2 → assistant2
                    // 而我们需要提取的是 assistant1 之前的内容
                    // 但 assistant1 之前只有 user1（开场白），所以这里需要调整逻辑

                    // 实际上，我们需要提取的是当前问答对之前的历史
                    // 当 current_index = 3（assistant2）时，我们需要提取 (assistant1, user2) 这对
                    // 但此时 i 应该从 1 开始遍历到 current_index - 2

                    // 重新理解：i 是 assistant 的索引
                    // 对于 assistant1（索引=1），它之前没有前序问答对（只有 user0 开场白）
                    // 对于 assistant2（索引=3），它之前有 (assistant1, user2) 这对

                    // 所以我们提取的是从索引 1 开始到 current_index - 2 的所有 assistant-user 配对
                    context_pairs.push(QAPairContext {
                        user_question: user_content.clone(),
                        assistant_answer: assistant_content.clone(),
                    });
                    pair_count += 1;
                }
            }

            i += 2;
        }

        context_pairs
    }
}

impl Default for QAPairDetector {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_message(
        uuid: &str,
        msg_type: &str,
        content: Option<&str>,
    ) -> Message {
        Message {
            id: None,
            session_id: "test-session".to_string(),
            uuid: uuid.to_string(),
            parent_uuid: None,
            msg_type: msg_type.to_string(),
            content_type: None,
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            offset: 0,
            length: 0,
            summary: None,
            content: content.map(|s| s.to_string()),
            parent_idx: None,
            created_at: "2025-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_detect_qa_pairs_normal_case() {
        // 验证正常情况下的问答对检测
        let detector = QAPairDetector::new();
        let messages = vec![
            create_test_message("u1", "user", Some("开场白")),
            create_test_message("a1", "assistant", Some("回答1")),
            create_test_message("u2", "user", Some("用户决策1")),
            create_test_message("a2", "assistant", Some("回答2")),
            create_test_message("u3", "user", Some("用户决策2")),
        ];

        let pairs = detector.detect_decision_qa_pairs(messages);

        assert_eq!(pairs.len(), 2);

        // 第一个问答对（无前序上下文）
        assert_eq!(pairs[0].qa_index, 0);
        assert_eq!(pairs[0].assistant_answer_uuid, "a1");
        assert_eq!(pairs[0].user_decision_uuid, "u2");
        assert_eq!(pairs[0].assistant_answer, "回答1");
        assert_eq!(pairs[0].user_decision, "用户决策1");
        // 第一个问答对无上下文
        assert!(pairs[0].context_qa_pairs.is_none());

        // 第二个问答对（有前序上下文）
        assert_eq!(pairs[1].qa_index, 1);
        assert_eq!(pairs[1].assistant_answer_uuid, "a2");
        assert_eq!(pairs[1].user_decision_uuid, "u3");
        assert_eq!(pairs[1].assistant_answer, "回答2");
        assert_eq!(pairs[1].user_decision, "用户决策2");

        // 验证前序上下文
        let context = pairs[1].context_qa_pairs.as_ref().unwrap();
        assert_eq!(context.len(), 1);
        assert_eq!(context[0].user_question, "用户决策1");
        assert_eq!(context[0].assistant_answer, "回答1");
    }

    #[test]
    fn test_detect_qa_pairs_empty_messages() {
        // 验证空消息数组的处理
        let detector = QAPairDetector::new();
        let messages = vec![];
        let pairs = detector.detect_decision_qa_pairs(messages);

        assert_eq!(pairs.len(), 0);
    }

    #[test]
    fn test_detect_qa_pairs_only_opening() {
        // 验证只有开场白的情况
        let detector = QAPairDetector::new();
        let messages = vec![create_test_message("u1", "user", Some("开场白"))];

        let pairs = detector.detect_decision_qa_pairs(messages);

        assert_eq!(pairs.len(), 0);
    }

    #[test]
    fn test_detect_qa_pairs_with_thinking() {
        // 验证包含 thinking 消息的情况
        let detector = QAPairDetector::new();
        let messages = vec![
            create_test_message("u1", "user", Some("开场白")),
            create_test_message("t1", "thinking", Some("思考中")),
            create_test_message("a1", "assistant", Some("回答1")),
            create_test_message("u2", "user", Some("用户决策1")),
        ];

        let pairs = detector.detect_decision_qa_pairs(messages);

        // thinking 消息不在预期的 assistant 位置，跳过
        // 下一个循环从 index 3 开始（u2），但 i+1 超出范围
        assert_eq!(pairs.len(), 0);
    }

    #[test]
    fn test_detect_qa_pairs_none_content() {
        // 验证消息内容为 None 的处理
        let detector = QAPairDetector::new();
        let messages = vec![
            create_test_message("u1", "user", Some("开场白")),
            create_test_message("a1", "assistant", None), // content 为 None
            create_test_message("u2", "user", Some("用户决策1")),
        ];

        let pairs = detector.detect_decision_qa_pairs(messages);

        // assistant 内容为 None，应该跳过
        assert_eq!(pairs.len(), 0);
    }

    #[test]
    fn test_detect_qa_pairs_user_content_none() {
        // 验证用户消息内容为 None 的处理
        let detector = QAPairDetector::new();
        let messages = vec![
            create_test_message("u1", "user", Some("开场白")),
            create_test_message("a1", "assistant", Some("回答1")),
            create_test_message("u2", "user", None), // content 为 None
        ];

        let pairs = detector.detect_decision_qa_pairs(messages);

        // user 内容为 None，应该跳过
        assert_eq!(pairs.len(), 0);
    }

    #[test]
    fn test_detect_qa_pairs_single_pair() {
        // 验证只有一个问答对的情况
        let detector = QAPairDetector::new();
        let messages = vec![
            create_test_message("u1", "user", Some("开场白")),
            create_test_message("a1", "assistant", Some("回答1")),
            create_test_message("u2", "user", Some("用户决策1")),
        ];

        let pairs = detector.detect_decision_qa_pairs(messages);

        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].qa_index, 0);
        assert_eq!(pairs[0].assistant_answer, "回答1");
        assert_eq!(pairs[0].user_decision, "用户决策1");
        assert!(pairs[0].context_qa_pairs.is_none()); // 第一个问答对无上下文
    }

    #[test]
    fn test_detect_qa_pairs_long_conversation() {
        // 验证长对话的问答对检测
        let detector = QAPairDetector::new();
        let messages = vec![
            create_test_message("u1", "user", Some("开场白")),
            create_test_message("a1", "assistant", Some("回答1")),
            create_test_message("u2", "user", Some("用户决策1")),
            create_test_message("a2", "assistant", Some("回答2")),
            create_test_message("u3", "user", Some("用户决策2")),
            create_test_message("a3", "assistant", Some("回答3")),
            create_test_message("u4", "user", Some("用户决策3")),
            create_test_message("a4", "assistant", Some("回答4")),
            create_test_message("u5", "user", Some("用户决策4")),
        ];

        let pairs = detector.detect_decision_qa_pairs(messages);

        assert_eq!(pairs.len(), 4);
        assert_eq!(pairs[0].qa_index, 0);
        // 第一个无上下文
        assert!(pairs[0].context_qa_pairs.is_none());

        assert_eq!(pairs[1].qa_index, 1);
        assert!(pairs[1].context_qa_pairs.is_some()); // 第二个有上下文
        assert_eq!(pairs[1].context_qa_pairs.as_ref().unwrap().len(), 1);

        assert_eq!(pairs[2].qa_index, 2);
        assert_eq!(pairs[2].context_qa_pairs.as_ref().unwrap().len(), 2);

        assert_eq!(pairs[3].qa_index, 3);
        assert_eq!(pairs[3].context_qa_pairs.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_qa_pair_detector_default() {
        // 验证 Default trait 实现
        let detector = QAPairDetector::default();
        let messages = vec![
            create_test_message("u1", "user", Some("开场白")),
            create_test_message("a1", "assistant", Some("回答1")),
            create_test_message("u2", "user", Some("用户决策1")),
        ];

        let pairs = detector.detect_decision_qa_pairs(messages);

        assert_eq!(pairs.len(), 1);
    }

    #[test]
    fn test_context_qa_pairs_limit() {
        // 验证上下文数量限制（最多 5 对）
        let detector = QAPairDetector::new();
        let messages = vec![
            create_test_message("u1", "user", Some("开场白")),
            create_test_message("a1", "assistant", Some("回答1")),
            create_test_message("u2", "user", Some("用户决策1")),
            create_test_message("a2", "assistant", Some("回答2")),
            create_test_message("u3", "user", Some("用户决策2")),
            create_test_message("a3", "assistant", Some("回答3")),
            create_test_message("u4", "user", Some("用户决策3")),
            create_test_message("a4", "assistant", Some("回答4")),
            create_test_message("u5", "user", Some("用户决策4")),
            create_test_message("a5", "assistant", Some("回答5")),
            create_test_message("u6", "user", Some("用户决策5")),
            create_test_message("a6", "assistant", Some("回答6")),
            create_test_message("u7", "user", Some("用户决策6")),
        ];

        let pairs = detector.detect_decision_qa_pairs(messages);

        // 最后一个问答对应该有前序上下文（最多 5 对）
        let last_pair = &pairs[pairs.len() - 1];
        assert!(last_pair.context_qa_pairs.is_some());

        let context = last_pair.context_qa_pairs.as_ref().unwrap();
        // 应该限制为 5 对
        assert!(context.len() <= 5);

        // 验证上下文顺序（按时间正序）
        if context.len() >= 2 {
            assert_eq!(context[0].user_question, "用户决策1");
            assert_eq!(context[0].assistant_answer, "回答1");
            assert_eq!(context[1].user_question, "用户决策2");
            assert_eq!(context[1].assistant_answer, "回答2");
        }
    }
}
