//! 上下文压缩器模块
//!
//! 去除冗余信息，保留关键决策点，减少 Token 使用量

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use crate::tokenizer::TokenCounter;

/// 压缩结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompressionResult {
    /// 压缩前的 Token 数量
    pub original_tokens: usize,
    /// 压缩后的 Token 数量
    pub compressed_tokens: usize,
    /// Token 减少百分比 (0.0 - 100.0)
    pub reduction_percentage: f64,
    /// 压缩后的消息列表
    pub compressed_messages: Vec<CompressedMessage>,
}

/// 压缩后的消息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompressedMessage {
    /// 消息角色
    pub role: String,
    /// 压缩后的内容
    pub content: String,
    /// 消息类型
    pub message_type: String,
    /// 是否被压缩（true 表示内容被修改过）
    pub is_compressed: bool,
}

/// 原始消息（用于压缩）
#[derive(Debug, Clone)]
struct RawMessage {
    /// 消息角色
    role: String,
    /// 消息内容
    content: String,
    /// 消息类型（message, thinking, tool_use, tool_output）
    message_type: String,
}

/// 上下文压缩器
///
/// 智能去除冗余信息，保留关键决策点
pub struct ContextCompressor {
    /// Token 计数器
    token_counter: TokenCounter,
}

impl ContextCompressor {
    /// 创建新的压缩器
    pub fn new() -> Result<Self> {
        Ok(Self {
            token_counter: TokenCounter::new()?,
        })
    }

    /// 压缩会话上下文
    ///
    /// # 参数
    /// - `messages_json`: 消息的 JSON 数组字符串
    ///
    /// # 返回
    /// 返回压缩结果，包含 Token 统计和压缩后的消息
    pub fn compress_session(&self, messages_json: &str) -> Result<CompressionResult> {
        // 1. 解析消息
        let raw_messages = self.parse_messages(messages_json)?;

        // 2. 计算 Token 数量
        let original_text = raw_messages.iter()
            .map(|m| format!("{}: {}\n", m.role, m.content))
            .collect::<String>();
        let original_tokens = self.token_counter.count_tokens(&original_text)?;

        // 3. 压缩消息
        let compressed_messages = self.compress_messages(&raw_messages)?;

        // 4. 计算压缩后的 Token 数量
        let compressed_text = compressed_messages.iter()
            .map(|m| format!("{}: {}\n", m.role, m.content))
            .collect::<String>();
        let compressed_tokens = self.token_counter.count_tokens(&compressed_text)?;

        // 5. 计算压缩率
        let reduction_percentage = if original_tokens > 0 {
            ((original_tokens - compressed_tokens) as f64 / original_tokens as f64) * 100.0
        } else {
            0.0
        };

        Ok(CompressionResult {
            original_tokens,
            compressed_tokens,
            reduction_percentage,
            compressed_messages,
        })
    }

    /// 解析消息 JSON
    fn parse_messages(&self, messages_json: &str) -> Result<Vec<RawMessage>> {
        let json_value: serde_json::Value = serde_json::from_str(messages_json)
            .context("解析消息 JSON 失败")?;

        let mut messages = Vec::new();

        // 处理数组格式
        if let Some(arr) = json_value.as_array() {
            for item in arr {
                if let Some(msg) = self.parse_single_message(item) {
                    messages.push(msg);
                }
            }
        }

        Ok(messages)
    }

    /// 解析单条消息
    fn parse_single_message(&self, json: &serde_json::Value) -> Option<RawMessage> {
        // 提取 role
        let role = json.get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("user")
            .to_string();

        // 提取 content
        let content = if let Some(content_arr) = json.get("content").and_then(|v| v.as_array()) {
            // 新格式：content 是数组
            self.extract_content_from_array(content_arr).ok().unwrap_or_default()
        } else {
            // 旧格式：content 是字符串
            json.get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        };

        if content.trim().is_empty() {
            return None;
        }

        // 确定消息类型
        let message_type = self.determine_message_type(&content, json);

        Some(RawMessage {
            role,
            content,
            message_type,
        })
    }

    /// 从 content 数组提取文本
    fn extract_content_from_array(&self, content_arr: &[serde_json::Value]) -> Result<String> {
        let mut result = String::new();

        for part in content_arr {
            let part_type = part.get("type").and_then(|v| v.as_str()).unwrap_or("");

            match part_type {
                "text" => {
                    if let Some(text) = part.get("text").and_then(|v| v.as_str()) {
                        result.push_str(text);
                    }
                }
                "tool_use" => {
                    if let Some(name) = part.get("name").and_then(|v| v.as_str()) {
                        result.push_str(&format!("[使用工具: {}]", name));

                        // 添加关键参数
                        if let Some(input) = part.get("input") {
                            if let Some(path) = input.get("path").and_then(|v| v.as_str()) {
                                result.push_str(&format!(" 路径: {}", path));
                            }
                            if let Some(query) = input.get("query").and_then(|v| v.as_str()) {
                                let query_preview = query.chars().take(50).collect::<String>();
                                result.push_str(&format!(" 查询: {}...", query_preview));
                            }
                        }
                    }
                }
                "tool_result" | "thinking" => {
                    // 这些类型会被压缩，所以只记录关键信息
                    if let Some(text) = part.get("text").and_then(|v| v.as_str()) {
                        let preview = self.compress_output(text, part_type);
                        result.push_str(&preview);
                    }
                }
                _ => {
                    // 其他类型，尝试提取文本
                    if let Some(text) = part.get("text").and_then(|v| v.as_str()) {
                        result.push_str(text);
                    }
                }
            }
        }

        Ok(result)
    }

    /// 确定消息类型
    fn determine_message_type(&self, content: &str, _json: &serde_json::Value) -> String {
        // 检查是否包含 thinking 标记
        if content.contains("<thinking>") || content.contains("Thinking:") {
            return "thinking".to_string();
        }

        // 检查是否是工具输出
        if content.contains("[工具结果]") || content.contains("tool_output") {
            return "tool_output".to_string();
        }

        // 检查是否是工具使用
        if content.contains("[使用工具:") || content.contains("tool_use") {
            return "tool_use".to_string();
        }

        // 默认为普通消息
        "message".to_string()
    }

    /// 压缩消息列表
    fn compress_messages(&self, messages: &[RawMessage]) -> Result<Vec<CompressedMessage>> {
        let mut compressed = Vec::new();
        let mut last_tool_use: Option<String> = None;

        for msg in messages {
            match msg.message_type.as_str() {
                "thinking" => {
                    // 完全移除 thinking 内容（保留消息存在但不包含实际内容）
                    // 不添加到压缩结果中
                }
                "tool_output" => {
                    // 压缩工具输出：只保留是否有错误
                    let compressed_content = if msg.content.contains("Error") || msg.content.contains("error") {
                        format!("[工具执行失败]")
                    } else {
                        // 成功的工具输出通常可以省略
                        continue;
                    };

                    if !compressed_content.is_empty() {
                        compressed.push(CompressedMessage {
                            role: msg.role.clone(),
                            content: compressed_content,
                            message_type: "tool_output".to_string(),
                            is_compressed: true,
                        });
                    }
                }
                "tool_use" => {
                    // 保留工具调用信息，但压缩参数
                    let tool_name = self.extract_tool_name(&msg.content);
                    let compressed_content = format!("[调用工具: {}]", tool_name);

                    // 检查是否是重复的工具调用
                    if let Some(ref last_tool) = last_tool_use {
                        if last_tool == &tool_name {
                            // 重复调用，跳过
                            continue;
                        }
                    }

                    compressed.push(CompressedMessage {
                        role: msg.role.clone(),
                        content: compressed_content,
                        message_type: "tool_use".to_string(),
                        is_compressed: true,
                    });

                    last_tool_use = Some(tool_name);
                }
                _ => {
                    // 普通消息，保留关键信息
                    let compressed_content = self.compress_message_content(&msg.content);
                    let is_compressed = compressed_content != msg.content;

                    compressed.push(CompressedMessage {
                        role: msg.role.clone(),
                        content: compressed_content,
                        message_type: msg.message_type.clone(),
                        is_compressed,
                    });
                }
            }
        }

        Ok(compressed)
    }

    /// 压缩单条消息内容
    fn compress_message_content(&self, content: &str) -> String {
        let mut result = String::new();

        for line in content.lines() {
            let line = line.trim();

            // 跳过空行
            if line.is_empty() {
                continue;
            }

            // 跳过明显的冗余内容
            if line.starts_with("---") || line.starts_with("...") {
                continue;
            }

            // 保留关键决策点
            if self.is_decision_point(line) {
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(line);
            }
            // 保留代码块（但压缩）
            else if line.starts_with("```") || result.contains("```") {
                // 简化代码块标记
                if !result.ends_with("[代码]") && !line.starts_with("```") {
                    result.push_str(line);
                } else if !result.ends_with("[代码]") {
                    result.push_str("\n[代码块]");
                }
            }
            // 保留其他内容，但限制长度
            else if result.len() < 5000 {
                // 限制每条消息的长度
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(line);
            }
        }

        if result.is_empty() {
            // 如果压缩后为空，返回摘要
            let preview = content.chars().take(100).collect::<String>();
            format!("{}...", preview)
        } else {
            result
        }
    }

    /// 判断是否为决策点
    fn is_decision_point(&self, line: &str) -> bool {
        let decision_keywords = [
            "决定", "选择", "采用", "方案", "策略",
            "decide", "choose", "adopt", "approach", "strategy",
            "问题:", "错误:", "警告:", "注意:",
            "fix:", "修复:", "solved:", "解决:"
        ];

        decision_keywords.iter().any(|&keyword| {
            line.to_lowercase().contains(&keyword.to_lowercase())
        })
    }

    /// 压缩工具输出
    fn compress_output(&self, text: &str, output_type: &str) -> String {
        // 对于工具输出，只保留关键信息
        if text.contains("Error") || text.contains("error") {
            // 保留错误摘要（前 200 字符）
            let preview: String = text.chars().take(200).collect::<String>();
            format!("[错误] {}...", preview)
        } else {
            // 成功输出，省略大部分内容
            format!("[{} 成功]", output_type)
        }
    }

    /// 提取工具名称
    fn extract_tool_name(&self, content: &str) -> String {
        // 从 "[使用工具: xxx]" 格式中提取
        if let Some(start) = content.find("[使用工具:") {
            if let Some(end) = content[start..].find(']') {
                return content[start + 6..start + end].trim().to_string();
            }
        }

        // 从 "tool_use" 格式中提取
        if content.contains("tool_use") {
            return "未知工具".to_string();
        }

        "工具".to_string()
    }

    /// 计算压缩率
    ///
    /// # 参数
    /// - `original`: 原始文本
    /// - `compressed`: 压缩后文本
    ///
    /// # 返回
    /// 压缩率（0.0 - 1.0）
    pub fn compression_ratio(&self, original: &str, compressed: &str) -> f64 {
        let original_len = original.chars().count();
        let compressed_len = compressed.chars().count();

        if original_len == 0 {
            return 0.0;
        }

        (original_len - compressed_len) as f64 / original_len as f64
    }
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_compressor() {
        let compressor = ContextCompressor::new().unwrap();
        assert!(compressor.token_counter.count_tokens("hello").is_ok());
    }

    #[test]
    fn test_compression_ratio() {
        let compressor = ContextCompressor::new().unwrap();
        let ratio = compressor.compression_ratio("hello world", "hello");
        assert_eq!(ratio, 0.5); // "hello world" (11 chars) -> "hello" (5 chars) = 45% ≈ 0.5
    }

    #[test]
    fn test_extract_tool_name() {
        let compressor = ContextCompressor::new().unwrap();
        let content = "[使用工具: read_file]";
        let tool_name = compressor.extract_tool_name(content);
        assert_eq!(tool_name, "read_file");
    }

    #[test]
    fn test_is_decision_point() {
        let compressor = ContextCompressor::new().unwrap();
        assert!(compressor.is_decision_point("决定采用方案A"));
        assert!(compressor.is_decision_point("Error: 文件未找到"));
        assert!(!compressor.is_decision_point("读取配置文件"));
    }
}
