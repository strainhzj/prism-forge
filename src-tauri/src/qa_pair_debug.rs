//! 问答对解析调试模块
//!
//! 用于调试和验证问答模式解析的正确性

use crate::database::models::Message;
use crate::parser::view_level::{ViewLevel, MessageFilter, QAPair};
use chrono::Utc;

/// 创建测试消息
fn create_test_message(
    msg_type: &str,
    uuid: &str,
    summary: Option<&str>,
) -> Message {
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
        summary: summary.map(|s| s.to_string()),
        content: Some("test content".to_string()),
        parent_idx: None,
        created_at: Utc::now().to_rfc3339(),
    }
}

/// 调试测试：连续 assistant 消息的配对
#[cfg(test)]
mod tests {
    use super::*;

    /// 测试场景 1：连续 assistant 消息
    ///
    /// 原始序列：
    /// 1. user (question 1)
    /// 2. assistant (intermediate response)
    /// 3. assistant (final response)
    /// 4. user (question 2)
    ///
    /// 期望结果：
    /// - question 1 应该配对到 assistant 3（最后一个 assistant）
    /// - question 2 没有答案
    #[test]
    fn test_consecutive_assistants() {
        let filter = MessageFilter::new(ViewLevel::QAPairs);

        let messages = vec![
            create_test_message("user", "uuid-1", Some("Question 1")),
            create_test_message("assistant", "uuid-2", Some("Intermediate response")),
            create_test_message("assistant", "uuid-3", Some("Final response")),
            create_test_message("user", "uuid-4", Some("Question 2")),
        ];

        let qa_pairs = filter.extract_qa_pairs(messages);

        println!("\n=== 测试场景 1：连续 assistant 消息 ===");
        println!("问答对数量: {}", qa_pairs.len());
        for (i, pair) in qa_pairs.iter().enumerate() {
            println!("\n问答对 {}:", i + 1);
            println!("  问题: {} - {}",
                &pair.question.uuid[..8],
                pair.question.summary.as_ref().unwrap_or(&"[无内容]".to_string())
            );
            if let Some(ref answer) = pair.answer {
                println!("  答案: {} - {}",
                    &answer.uuid[..8],
                    answer.summary.as_ref().unwrap_or(&"[无内容]".to_string())
                );
            } else {
                println!("  答案: None");
            }
        }

        // 当前逻辑的行为验证
        assert_eq!(qa_pairs.len(), 2);
        // question 1 配对到最后一个 assistant (uuid-3)
        assert_eq!(qa_pairs[0].question.uuid, "uuid-1");
        assert!(qa_pairs[0].answer.is_some());
        assert_eq!(qa_pairs[0].answer.as_ref().unwrap().uuid, "uuid-3");
        // question 2 没有答案
        assert_eq!(qa_pairs[1].question.uuid, "uuid-4");
        assert!(qa_pairs[1].answer.is_none());
    }

    /// 测试场景 2：user - thinking - assistant 模式
    ///
    /// 原始序列：
    /// 1. user (question 1)
    /// 2. thinking (思考过程)
    /// 3. assistant (response)
    /// 4. user (question 2)
    ///
    /// 期望结果：
    /// - question 1 应该配对到 assistant 3（跳过 thinking）
    /// - question 2 没有答案
    #[test]
    fn test_user_thinking_assistant_pattern() {
        let filter = MessageFilter::new(ViewLevel::QAPairs);

        let messages = vec![
            create_test_message("user", "uuid-1", Some("Question 1")),
            create_test_message("thinking", "uuid-2", Some("Thinking process")),
            create_test_message("assistant", "uuid-3", Some("Response")),
            create_test_message("user", "uuid-4", Some("Question 2")),
        ];

        let qa_pairs = filter.extract_qa_pairs(messages);

        println!("\n=== 测试场景 2：user - thinking - assistant 模式 ===");
        println!("问答对数量: {}", qa_pairs.len());
        for (i, pair) in qa_pairs.iter().enumerate() {
            println!("\n问答对 {}:", i + 1);
            println!("  问题: {} - {}",
                &pair.question.uuid[..8],
                pair.question.summary.as_ref().unwrap_or(&"[无内容]".to_string())
            );
            if let Some(ref answer) = pair.answer {
                println!("  答案: {} - {}",
                    &answer.uuid[..8],
                    answer.summary.as_ref().unwrap_or(&"[无内容]".to_string())
                );
            } else {
                println!("  答案: None");
            }
        }

        // 验证结果
        assert_eq!(qa_pairs.len(), 2);
        assert_eq!(qa_pairs[0].question.uuid, "uuid-1");
        // 当前逻辑：thinking 被预过滤，所以应该能配对成功
        assert!(qa_pairs[0].answer.is_some());
        assert_eq!(qa_pairs[0].answer.as_ref().unwrap().uuid, "uuid-3");
        assert_eq!(qa_pairs[1].question.uuid, "uuid-4");
        assert!(qa_pairs[1].answer.is_none());
    }

    /// 测试场景 3：多个 user 后跟多个 assistant
    ///
    /// 原始序列：
    /// 1. user (question 1)
    /// 2. user (question 2)
    /// 3. assistant (response to question 2)
    /// 4. assistant (response to question 1?)
    ///
    /// 期望结果（当前逻辑）：
    /// - question 1 配对到 assistant 4（最后一个）
    /// - question 2 配对到 assistant 3（第一个）
    #[test]
    fn test_multiple_users_multiple_assistants() {
        let filter = MessageFilter::new(ViewLevel::QAPairs);

        let messages = vec![
            create_test_message("user", "uuid-1", Some("Question 1")),
            create_test_message("user", "uuid-2", Some("Question 2")),
            create_test_message("assistant", "uuid-3", Some("Response to Q2")),
            create_test_message("assistant", "uuid-4", Some("Response to Q1")),
        ];

        let qa_pairs = filter.extract_qa_pairs(messages);

        println!("\n=== 测试场景 3：多个 user 后跟多个 assistant ===");
        println!("问答对数量: {}", qa_pairs.len());
        for (i, pair) in qa_pairs.iter().enumerate() {
            println!("\n问答对 {}:", i + 1);
            println!("  问题: {} - {}",
                &pair.question.uuid[..8],
                pair.question.summary.as_ref().unwrap_or(&"[无内容]".to_string())
            );
            if let Some(ref answer) = pair.answer {
                println!("  答案: {} - {}",
                    &answer.uuid[..8],
                    answer.summary.as_ref().unwrap_or(&"[无内容]".to_string())
                );
            } else {
                println!("  答案: None");
            }
        }

        // 当前逻辑的行为验证
        assert_eq!(qa_pairs.len(), 2);
        // 从后向前扫描：
        // - uuid-2 (user) -> 向后找第一个 assistant (uuid-3) -> 配对 (uuid-2, uuid-3)
        // - uuid-1 (user) -> 向后找第一个 assistant (uuid-4) -> 配对 (uuid-1, uuid-4)
        assert_eq!(qa_pairs[0].question.uuid, "uuid-1");
        assert!(qa_pairs[0].answer.is_some());
        assert_eq!(qa_pairs[0].answer.as_ref().unwrap().uuid, "uuid-4");
        assert_eq!(qa_pairs[1].question.uuid, "uuid-2");
        assert!(qa_pairs[1].answer.is_some());
        assert_eq!(qa_pairs[1].answer.as_ref().unwrap().uuid, "uuid-3");
    }

    /// 测试场景 4：复杂混合场景
    ///
    /// 原始序列：
    /// 1. user (question 1)
    /// 2. assistant (response 1)
    /// 3. thinking (thinking 1)
    /// 4. user (question 2)
    /// 5. assistant (tool_use) - 应该被预过滤
    /// 6. assistant (response 2)
    /// 7. user (question 3)
    ///
    /// 期望结果：
    /// - question 1 配对到 response 1
    /// - question 2 配对到 response 2（跳过 tool_use）
    /// - question 3 没有答案
    #[test]
    fn test_complex_mixed_scenario() {
        let filter = MessageFilter::new(ViewLevel::QAPairs);

        let messages = vec![
            create_test_message("user", "uuid-1", Some("Question 1")),
            create_test_message("assistant", "uuid-2", Some("Response 1")),
            create_test_message("thinking", "uuid-3", Some("Thinking 1")),
            create_test_message("user", "uuid-4", Some("Question 2")),
            // tool_use 消息
            create_test_message(
                "assistant",
                "uuid-5",
                Some(r#"{"content":[{"type":"tool_use","id":"tool-1","name":"test","input":{}}]}"#)
            ),
            create_test_message("assistant", "uuid-6", Some("Response 2")),
            create_test_message("user", "uuid-7", Some("Question 3")),
        ];

        let qa_pairs = filter.extract_qa_pairs(messages);

        println!("\n=== 测试场景 4：复杂混合场景 ===");
        println!("问答对数量: {}", qa_pairs.len());
        for (i, pair) in qa_pairs.iter().enumerate() {
            println!("\n问答对 {}:", i + 1);
            println!("  问题: {} - {}",
                &pair.question.uuid[..8],
                pair.question.summary.as_ref().unwrap_or(&"[无内容]".to_string())
            );
            if let Some(ref answer) = pair.answer {
                println!("  答案: {} - {}",
                    &answer.uuid[..8],
                    answer.summary.as_ref().unwrap_or(&"[无内容]".to_string())
                );
            } else {
                println!("  答案: None");
            }
        }

        // 验证结果（预过滤后 thinking 和 tool_use 被移除）
        assert_eq!(qa_pairs.len(), 3);
        assert_eq!(qa_pairs[0].question.uuid, "uuid-1");
        assert!(qa_pairs[0].answer.is_some());
        assert_eq!(qa_pairs[0].answer.as_ref().unwrap().uuid, "uuid-2");
        assert_eq!(qa_pairs[1].question.uuid, "uuid-4");
        assert!(qa_pairs[1].answer.is_some());
        assert_eq!(qa_pairs[1].answer.as_ref().unwrap().uuid, "uuid-6");
        assert_eq!(qa_pairs[2].question.uuid, "uuid-7");
        assert!(qa_pairs[2].answer.is_none());
    }
}
