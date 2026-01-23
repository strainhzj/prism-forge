//! SessionParserService 集成测试
//!
//! 测试完整的会话解析流程，验证各组件之间的集成

#[cfg(test)]
mod integration_tests {
    use super::super::*;
    use crate::session_parser::{SessionParserService, SessionParserConfig};
    use crate::parser::view_level::ViewLevel;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// 创建临时 JSONL 测试文件
    fn create_test_jsonl_file() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();

        // 写入测试数据
        let test_data = r#"{"timestamp":"2025-01-19T12:00:00Z","type":"user","uuid":"msg-001","message":"Hello, how are you?","parentUuid":null}
{"timestamp":"2025-01-19T12:00:01Z","type":"assistant","uuid":"msg-002","message":"I'm doing well, thank you!","parentUuid":"msg-001"}
{"timestamp":"2025-01-19T12:00:02Z","type":"user","uuid":"msg-003","message":"What's the weather like?","parentUuid":"msg-002"}
{"timestamp":"2025-01-19T12:00:03Z","type":"assistant","uuid":"msg-004","message":"I don't have access to real-time weather data.","parentUuid":"msg-003"}
{"timestamp":"2025-01-19T12:00:04Z","type":"user","uuid":"msg-005","message":"/clear","parentUuid":"msg-004"}
{"timestamp":"2025-01-19T12:00:05Z","type":"system","uuid":"msg-006","message":"Conversation cleared","parentUuid":"msg-005"}
"#;

        writeln!(temp_file, "{}", test_data).unwrap();
        temp_file
    }

    #[test]
    fn test_full_parsing_workflow() {
        let temp_file = create_test_jsonl_file();
        let file_path = temp_file.path().to_str().unwrap();

        let config = SessionParserConfig {
            enable_content_filter: false,
            view_level: ViewLevel::Full,
            debug: false,
        };

        let parser = SessionParserService::new(config);
        let result = parser.parse_session(file_path, "test_session");

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
        let temp_file = create_test_jsonl_file();
        let file_path = temp_file.path().to_str().unwrap();

        let config = SessionParserConfig {
            enable_content_filter: true,  // 启用内容过滤
            view_level: ViewLevel::Full,
            debug: false,
        };

        let parser = SessionParserService::new(config);
        let result = parser.parse_session(file_path, "test_session");

        assert!(result.is_ok());
        let parse_result = result.unwrap();

        // 验证 /clear 命令被过滤
        assert!(parse_result.stats.content_filtered > 0);
        assert_eq!(parse_result.stats.final_messages, 5); // 6 - 1 (filtered)
    }

    #[test]
    fn test_view_level_filtering() {
        let temp_file = create_test_jsonl_file();
        let file_path = temp_file.path().to_str().unwrap();

        let config = SessionParserConfig {
            enable_content_filter: false,
            view_level: ViewLevel::Conversation,  // 对话模式
            debug: false,
        };

        let parser = SessionParserService::new(config);
        let result = parser.parse_session(file_path, "test_session");

        assert!(result.is_ok());
        let parse_result = result.unwrap();

        // Conversation 模式应该过滤掉 system 消息
        assert!(parse_result.stats.view_level_filtered > 0);
    }

    #[test]
    fn test_combined_filtering() {
        let temp_file = create_test_jsonl_file();
        let file_path = temp_file.path().to_str().unwrap();

        let config = SessionParserConfig {
            enable_content_filter: true,
            view_level: ViewLevel::Conversation,
            debug: false,
        };

        let parser = SessionParserService::new(config);
        let result = parser.parse_session(file_path, "test_session");

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
    fn test_message_order_preserved() {
        let temp_file = create_test_jsonl_file();
        let file_path = temp_file.path().to_str().unwrap();

        let config = SessionParserConfig::default();
        let parser = SessionParserService::new(config);
        let result = parser.parse_session(file_path, "test_session");

        assert!(result.is_ok());
        let parse_result = result.unwrap();

        // 验证消息顺序保持不变
        let timestamps: Vec<&str> = parse_result.messages
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

    #[test]
    fn test_session_id_assigned_correctly() {
        let temp_file = create_test_jsonl_file();
        let file_path = temp_file.path().to_str().unwrap();

        let config = SessionParserConfig::default();
        let parser = SessionParserService::new(config);
        let result = parser.parse_session(file_path, "my_test_session");

        assert!(result.is_ok());
        let parse_result = result.unwrap();

        // 验证所有消息都有正确的 session_id
        for msg in &parse_result.messages {
            assert_eq!(msg.session_id, "my_test_session");
        }
    }
}
