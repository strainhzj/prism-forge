//! 关键信息提取器
//!
//! 从消息树中提取工具调用、错误消息、代码变更等关键信息，生成摘要。

use anyhow::{Result, Context};
use serde_json::Value;

use super::tree::{MessageNode, MessageMetadata, ToolCall, ErrorMessage, CodeChange, ConversationTree};

/// 关键信息提取器
///
/// 负责从消息树中提取关键信息并生成元数据
pub struct MetadataExtractor;

impl MetadataExtractor {
    /// 处理整个对话树，为所有节点添加元数据
    ///
    /// # 参数
    /// * `tree` - 对话树的可变引用
    ///
    /// # 返回
    /// 返回处理结果或错误
    pub fn extract_tree_metadata(tree: &mut ConversationTree) -> Result<()> {
        for root in &mut tree.roots {
            Self::extract_node_metadata_recursive(root)?;
        }
        Ok(())
    }

    /// 递归提取节点元数据
    ///
    /// 深度优先遍历树结构，为每个节点提取元数据
    fn extract_node_metadata_recursive(node: &mut MessageNode) -> Result<()> {
        // 提取当前节点的元数据
        node.metadata = Some(Self::extract_metadata_from_node(node)?);

        // 递归处理子节点
        for child in &mut node.children {
            Self::extract_node_metadata_recursive(child)?;
        }

        Ok(())
    }

    /// 从单个节点提取元数据
    ///
    /// # 参数
    /// * `node` - 消息节点引用
    ///
    /// # 返回
    /// 返回提取的元数据或错误
    fn extract_metadata_from_node(node: &MessageNode) -> Result<MessageMetadata> {
        // 提取工具调用
        let tool_calls = Self::extract_tool_calls(node);

        // 提取错误消息
        let errors = Self::extract_errors(node);

        // 提取代码变更
        let code_changes = Self::extract_code_changes(node);

        // 生成摘要
        let summary = Self::generate_summary(node, &tool_calls, &errors, &code_changes);

        Ok(MessageMetadata {
            summary,
            tool_calls,
            errors,
            code_changes,
        })
    }

    /// 提取工具调用信息
    ///
    /// 从消息中识别并提取所有工具调用
    fn extract_tool_calls(node: &MessageNode) -> Vec<ToolCall> {
        let mut tool_calls = Vec::new();

        // 检查消息类型
        let msg_type = node.message_type().unwrap_or_default();

        // 方法1: 直接是 tool_use 类型的消息
        if msg_type == "tool_use" {
            if let Some(tool_name) = node.message_data.get("name")
                .and_then(|v| v.as_str())
            {
                let input = node.message_data.get("input")
                    .cloned()
                    .unwrap_or(Value::Object(serde_json::Map::new()));

                // 从内容中提取状态（如果有）
                let status = Self::extract_tool_status_from_content(&node.message_data);

                tool_calls.push(ToolCall {
                    name: tool_name.to_string(),
                    input,
                    status,
                });
            }
        }

        // 方法2: 从 content 数组中提取 tool_use 块
        if let Some(content) = node.message_data.get("content") {
            if let Some(content_array) = content.as_array() {
                for item in content_array {
                    if let Some(item_type) = item.get("type")
                        .and_then(|v| v.as_str())
                    {
                        if item_type == "tool_use" {
                            if let Some(tool_name) = item.get("name")
                                .and_then(|v| v.as_str())
                            {
                                let input = item.get("input")
                                    .cloned()
                                    .unwrap_or(Value::Object(serde_json::Map::new()));

                                // 从 tool_use 内容中提取状态
                                let status = if let Some(content_text) = item.get("content")
                                    .and_then(|v| v.as_str())
                                {
                                    Self::parse_tool_status(content_text)
                                } else {
                                    "success".to_string()
                                };

                                tool_calls.push(ToolCall {
                                    name: tool_name.to_string(),
                                    input,
                                    status,
                                });
                            }
                        }
                    }
                }
            }
        }

        tool_calls
    }

    /// 从工具结果消息中提取状态
    ///
    /// 检查 tool_result 类型消息中的错误信息
    fn extract_tool_status_from_content(message_data: &Value) -> String {
        // 检查是否有 error 字段
        if message_data.get("error").is_some() {
            return "error".to_string();
        }

        // 检查 content 中的错误信息
        if let Some(content) = message_data.get("content") {
            // 如果 content 是字符串
            if let Some(text) = content.as_str() {
                if text.contains("Error:") || text.contains("error:") || text.contains("失败") {
                    return "error".to_string();
                }
            }
            // 如果 content 是数组
            if let Some(content_array) = content.as_array() {
                for item in content_array {
                    if let Some(text) = item.get("text")
                        .and_then(|v| v.as_str())
                    {
                        if text.contains("Error:") || text.contains("error:") || text.contains("失败") {
                            return "error".to_string();
                        }
                    }
                }
            }
        }

        "success".to_string()
    }

    /// 从工具结果文本中解析状态
    fn parse_tool_status(content: &str) -> String {
        if content.contains("Error:") || content.contains("error:") || content.contains("失败") {
            "error".to_string()
        } else {
            "success".to_string()
        }
    }

    /// 提取错误消息
    ///
    /// 识别并提取消息中的所有错误信息
    fn extract_errors(node: &MessageNode) -> Vec<ErrorMessage> {
        let mut errors = Vec::new();

        // 检查消息内容中的错误
        if let Some(content) = node.message_data.get("content") {
            Self::extract_errors_from_value(content, &mut errors, None);
        }

        // 检查是否有工具调用产生的错误
        if let Some(tool_calls) = node.message_data.get("tool_calls") {
            if let Some(calls_array) = tool_calls.as_array() {
                for call in calls_array {
                    if let Some(tool_name) = call.get("name")
                        .and_then(|v| v.as_str())
                    {
                        // 检查工具调用结果中的错误
                        if let Some(result) = call.get("result") {
                            Self::extract_errors_from_value(result, &mut errors, Some(tool_name.to_string()));
                        }
                    }
                }
            }
        }

        errors
    }

    /// 从 JSON 值中递归提取错误
    fn extract_errors_from_value(value: &Value, errors: &mut Vec<ErrorMessage>, related_tool: Option<String>) {
        match value {
            Value::String(text) => {
                if text.contains("Error:") || text.contains("error:") || text.contains("失败") {
                    // 提取错误类型和消息
                    let lines: Vec<&str> = text.lines().collect();
                    for line in lines {
                        if line.contains("Error:") || line.contains("error:") {
                            let parts: Vec<&str> = line.splitn(2, ':').collect();
                            let error_type = parts.get(0)
                                .map(|s| s.trim().to_string())
                                .unwrap_or_else(|| "Error".to_string());

                            let message = parts.get(1)
                                .map(|s| s.trim().to_string())
                                .unwrap_or_else(|| text.trim().to_string());

                            errors.push(ErrorMessage {
                                error_type,
                                message,
                                related_tool: related_tool.clone(),
                            });
                        } else if line.contains("失败") || line.contains("错误") {
                            errors.push(ErrorMessage {
                                error_type: "RuntimeError".to_string(),
                                message: line.trim().to_string(),
                                related_tool: related_tool.clone(),
                            });
                        }
                    }
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    Self::extract_errors_from_value(item, errors, related_tool.clone());
                }
            }
            Value::Object(obj) => {
                // 检查 error 字段
                if let Some(error_msg) = obj.get("error")
                    .and_then(|v| v.as_str())
                {
                    errors.push(ErrorMessage {
                        error_type: "ToolError".to_string(),
                        message: error_msg.to_string(),
                        related_tool: related_tool.clone(),
                    });
                }

                // 递归检查其他字段
                for (_key, val) in obj {
                    Self::extract_errors_from_value(val, errors, related_tool.clone());
                }
            }
            _ => {}
        }
    }

    /// 提取代码变更记录
    ///
    /// 识别 Read/Write/Edit 操作并提取相关信息
    fn extract_code_changes(node: &MessageNode) -> Vec<CodeChange> {
        let mut code_changes = Vec::new();

        // 从工具调用中提取代码变更
        let tool_calls = Self::extract_tool_calls(node);

        for tool_call in &tool_calls {
            match tool_call.name.as_str() {
                "read_file" | "Read" => {
                    if let Some(file_path) = tool_call.input.get("file_path")
                        .or_else(|| tool_call.input.get("path"))
                        .and_then(|v| v.as_str())
                    {
                        code_changes.push(CodeChange {
                            operation: "Read".to_string(),
                            file_path: file_path.to_string(),
                            lines_changed: None,
                        });
                    }
                }
                "write_file" | "Write" | "edit_file" | "Edit" => {
                    if let Some(file_path) = tool_call.input.get("file_path")
                        .or_else(|| tool_call.input.get("path"))
                        .and_then(|v| v.as_str())
                    {
                        // 估算变更行数
                        let lines_changed = tool_call.input.get("content")
                            .and_then(|v| v.as_str())
                            .map(|s| s.lines().count())
                            .or_else(|| {
                                tool_call.input.get("diff")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.lines().count())
                            });

                        code_changes.push(CodeChange {
                            operation: if tool_call.name.contains("edit") || tool_call.name.contains("Edit") {
                                "Edit".to_string()
                            } else {
                                "Write".to_string()
                            },
                            file_path: file_path.to_string(),
                            lines_changed,
                        });
                    }
                }
                _ => {}
            }
        }

        // 从 content 中提取 Read/Write/Edit 操作
        if let Some(content) = node.message_data.get("content") {
            if let Some(content_array) = content.as_array() {
                for item in content_array {
                    if let Some(item_type) = item.get("type")
                        .and_then(|v| v.as_str())
                    {
                        if item_type == "text" {
                            if let Some(text) = item.get("text")
                                .and_then(|v| v.as_str())
                            {
                                // 简单匹配：Read file: xxx 或 Write file: xxx
                                if let Some(caps) = Self::extract_file_operation(text) {
                                    code_changes.push(caps);
                                }
                            }
                        }
                    }
                }
            }
        }

        code_changes
    }

    /// 从文本中提取文件操作
    ///
    /// 匹配 "Read file: xxx", "Write file: xxx" 等模式
    fn extract_file_operation(text: &str) -> Option<CodeChange> {
        let lower = text.to_lowercase();

        // 匹配 "Read file:" 模式
        if lower.contains("read file:") || lower.contains("reading") {
            if let Some(start) = lower.find("read file:") {
                let remaining = &text[start + 10..];
                let file_path = remaining.split_whitespace().next().unwrap_or("");
                if !file_path.is_empty() {
                    return Some(CodeChange {
                        operation: "Read".to_string(),
                        file_path: file_path.trim_matches('"').trim_matches('\'').to_string(),
                        lines_changed: None,
                    });
                }
            }
        }

        // 匹配 "Write file:" 模式
        if lower.contains("write file:") || lower.contains("writing") {
            if let Some(start) = lower.find("write file:") {
                let remaining = &text[start + 11..];
                let file_path = remaining.split_whitespace().next().unwrap_or("");
                if !file_path.is_empty() {
                    return Some(CodeChange {
                        operation: "Write".to_string(),
                        file_path: file_path.trim_matches('"').trim_matches('\'').to_string(),
                        lines_changed: None,
                    });
                }
            }
        }

        None
    }

    /// 生成消息摘要
    ///
    /// 基于提取的信息生成简短的摘要文本
    fn generate_summary(
        node: &MessageNode,
        tool_calls: &[ToolCall],
        errors: &[ErrorMessage],
        code_changes: &[CodeChange],
    ) -> Option<String> {
        let mut summary_parts = Vec::new();

        // 添加角色类型
        let role = node.role().unwrap_or_default();
        if !role.is_empty() {
            summary_parts.push(format!("[{}]", role));
        }

        // 添加工具调用摘要
        if !tool_calls.is_empty() {
            let tool_names: Vec<&str> = tool_calls.iter()
                .map(|tc| tc.name.as_str())
                .collect();
            summary_parts.push(format!("工具: {}", tool_names.join(", ")));
        }

        // 添加代码变更摘要
        if !code_changes.is_empty() {
            let ops: Vec<String> = code_changes.iter()
                .map(|cc| format!("{} {}", cc.operation, Self::truncate_path(&cc.file_path)))
                .collect();
            summary_parts.push(format!("文件: {}", ops.join(", ")));
        }

        // 如果有错误，添加错误摘要
        if !errors.is_empty() {
            summary_parts.push(format!("{} 个错误", errors.len()));
        }

        // 如果没有特殊信息，尝试从 content 中提取
        if summary_parts.is_empty() {
            if let Some(content) = node.message_data.get("content") {
                let text = Self::extract_text_content(content);
                if !text.is_empty() {
                    // 截取前 100 个字符作为摘要
                    let truncated = if text.len() > 100 {
                        format!("{}...", &text[..100])
                    } else {
                        text
                    };
                    return Some(truncated);
                }
            }
        }

        if summary_parts.is_empty() {
            None
        } else {
            Some(summary_parts.join(" | "))
        }
    }

    /// 提取文本内容
    ///
    /// 从 content 中提取纯文本
    fn extract_text_content(content: &Value) -> String {
        match content {
            Value::String(s) => s.clone(),
            Value::Array(arr) => {
                let mut texts = Vec::new();
                for item in arr {
                    if let Some(item_type) = item.get("type")
                        .and_then(|v| v.as_str())
                    {
                        if item_type == "text" {
                            if let Some(text) = item.get("text")
                                .and_then(|v| v.as_str())
                            {
                                texts.push(text);
                            }
                        }
                    }
                }
                texts.join(" ")
            }
            _ => String::new(),
        }
    }

    /// 截断文件路径（保留最后部分）
    fn truncate_path(path: &str) -> String {
        let parts: Vec<&str> = path.rsplitn(2, '/').collect();
        if parts.len() >= 1 {
            parts[0].to_string()
        } else {
            path.to_string()
        }
    }
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tree::MessageNode;
    use serde_json::json;

    #[test]
    fn test_extract_tool_calls() {
        let message_data = json!({
            "type": "tool_use",
            "name": "read_file",
            "input": {"file_path": "/path/to/file.rs"},
            "content": "Successfully read file"
        });

        let node = MessageNode::new(
            "test-id".to_string(),
            None,
            message_data,
        );

        let tool_calls = MetadataExtractor::extract_tool_calls(&node);
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].name, "read_file");
    }

    #[test]
    fn test_extract_errors() {
        let message_data = json!({
            "type": "message",
            "role": "assistant",
            "content": "Error: File not found\nFailed to read file"
        });

        let node = MessageNode::new(
            "test-id".to_string(),
            None,
            message_data,
        );

        let errors = MetadataExtractor::extract_errors(&node);
        assert!(!errors.is_empty());
        assert_eq!(errors[0].error_type, "Error");
    }

    #[test]
    fn test_extract_code_changes() {
        let message_data = json!({
            "type": "tool_use",
            "name": "write_file",
            "input": {
                "file_path": "/path/to/file.rs",
                "content": "line1\nline2\nline3"
            }
        });

        let node = MessageNode::new(
            "test-id".to_string(),
            None,
            message_data,
        );

        let code_changes = MetadataExtractor::extract_code_changes(&node);
        assert_eq!(code_changes.len(), 1);
        assert_eq!(code_changes[0].operation, "Write");
        assert_eq!(code_changes[0].file_path, "/path/to/file.rs");
        assert_eq!(code_changes[0].lines_changed, Some(3));
    }

    #[test]
    fn test_generate_summary() {
        let message_data = json!({
            "type": "message",
            "role": "user",
            "content": "请帮我读取文件"
        });

        let node = MessageNode::new(
            "test-id".to_string(),
            None,
            message_data,
        );

        let tool_calls = vec![];
        let errors = vec![];
        let code_changes = vec![];

        let summary = MetadataExtractor::generate_summary(
            &node,
            &tool_calls,
            &errors,
            &code_changes,
        );

        assert!(summary.is_some());
        assert!(summary.unwrap().contains("请帮我读取文件"));
    }
}
