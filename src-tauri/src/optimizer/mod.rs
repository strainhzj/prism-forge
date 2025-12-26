//! Prompt Optimizer - 提示词优化模块
//!
//! 使用 LLMClientManager 调用当前活跃的 LLM 提供商来分析和优化提示词

use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::SystemTime;

use crate::llm::LLMClientManager;
use crate::llm::interface::{Message, MessageRole, ModelParams};

// ==================== 数据结构 ====================

/// 解析后的事件
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParsedEvent {
    pub time: String,
    pub role: String,
    pub content: String,
    pub event_type: String,
}

/// 优化请求
pub struct OptimizeRequest {
    pub session_file: String,
    pub goal: String,
}

/// 优化结果
pub struct OptimizeResult {
    pub status: String,
    pub analysis: String,
    pub suggested_prompt: String,
}

// ==================== 核心功能 ====================

/// Prompt Optimizer - 使用 LLM 分析和优化提示词
pub struct PromptOptimizer;

impl PromptOptimizer {
    /// 创建新的优化器
    pub fn new() -> Self {
        Self
    }

    /// 优化提示词
    ///
    /// # 流程
    /// 1. 解析会话文件
    /// 2. 构建系统提示词
    /// 3. 调用 LLM 分析
    /// 4. 解析结果
    pub async fn optimize_prompt(
        &self,
        request: OptimizeRequest,
        llm_manager: &LLMClientManager,
    ) -> Result<OptimizeResult> {
        // 1. 解析会话文件
        let events = Self::parse_session_file(&request.session_file)?;

        if events.is_empty() {
            return Ok(OptimizeResult {
                status: "错误".to_string(),
                analysis: "解析结果为空".to_string(),
                suggested_prompt: String::new(),
            });
        }

        // 2. 格式化上下文（取最后 20 条）
        let start_idx = if events.len() > 20 { events.len() - 20 } else { 0 };
        let recent_events = &events[start_idx..];

        let context_str = recent_events.iter().map(|e| {
            format!("[{}] {}:\n{}", e.time, e.role.to_uppercase(), e.content)
        }).collect::<Vec<String>>().join("\n--------------------\n");

        // 3. 构建系统提示词
        let system_prompt = r#"
你是一个 Claude Code 结对编程助手。请分析下方的会话日志（包含用户指令、Claude 的操作、以及工具返回的文件内容/报错）。

任务：
1. **判断焦点 (Focus Check)**：Claude 是否陷入了死循环？是否在反复读取无关文件？是否无视了报错？
2. **生成提示词 (Prompt Generation)**：为用户写一段可以直接发送给 Claude 的**中文指令**。
   - 如果 Claude 走偏了：写一段严厉的纠正指令。
   - 如果 Claude 做得对：写一段推进下一步的指令，并引用刚才读取到的文件上下文（例如："基于刚才读取的 main.py..."）。

输出格式：
---
【状态】: [正常 / 迷失 / 报错循环]
【分析】: (简短分析当前情况)
【建议提示词】:
(你的 Prompt 内容)
---
"#;

        let full_text = format!(
            "{}\n\n== 会话日志 ==\n{}\n\n== 用户当前目标 ==\n{}",
            system_prompt, context_str, request.goal
        );

        // 4. 调用 LLM
        let client = llm_manager.get_active_client()?;

        let messages = vec![
            Message::user(full_text)
        ];

        // ModelParams::new 需要模型名称，使用通用模型名
        let params = ModelParams::new("gpt-3.5-turbo")
            .with_max_tokens(2000);

        let response = client.chat_completion(messages, params).await?;

        // 5. 解析 LLM 响应 - ChatCompletionResponse.content 是 String
        let result = Self::parse_llm_response(&response.content);

        Ok(result)
    }

    /// 解析会话文件
    fn parse_session_file(file_path: &str) -> Result<Vec<ParsedEvent>> {
        let file = File::open(file_path)
            .context(format!("无法打开会话文件: {}", file_path))?;
        let reader = BufReader::new(file);
        let mut events = Vec::new();

        for line in reader.lines() {
            let line_content = line.context("读取行失败")?;
            if line_content.trim().is_empty() { continue; }

            let json: serde_json::Value = serde_json::from_str(&line_content)
                .unwrap_or(serde_json::Value::Null);
            if json.is_null() { continue; }

            let timestamp = json["timestamp"].as_str().unwrap_or("").to_string();

            // 1. 提取 Message (User / Assistant)
            if let Some(msg) = json.get("message") {
                let role = msg["role"].as_str().unwrap_or("unknown").to_string();
                let mut final_text = String::new();

                // 处理 content 是列表的情况 (新版格式)
                if let Some(content_arr) = msg["content"].as_array() {
                    for part in content_arr {
                        let p_type = part["type"].as_str().unwrap_or("");
                        if p_type == "text" {
                            final_text.push_str(part["text"].as_str().unwrap_or(""));
                            final_text.push('\n');
                        } else if p_type == "tool_use" {
                            let name = part["name"].as_str().unwrap_or("unknown");
                            final_text.push_str(&format!("[操作] 调用工具: {}\n", name));
                            if let Some(input) = part.get("input") {
                                if let Some(path) = input.get("path").and_then(|v| v.as_str()) {
                                    final_text.push_str(&format!("  - 路径: {}\n", path));
                                }
                            }
                        }
                    }
                }
                // 处理 content 是字符串的情况 (旧格式)
                else if let Some(content_str) = msg["content"].as_str() {
                    final_text = content_str.to_string();
                }

                if !final_text.trim().is_empty() {
                    events.push(ParsedEvent {
                        time: timestamp.clone(),
                        role,
                        content: final_text,
                        event_type: "message".to_string(),
                    });
                }
            }

            // 2. 提取 ToolResult (上下文关键)
            if let Some(tool_res) = json.get("toolUseResult") {
                let mut res_str = String::new();

                if let Some(filenames) = tool_res.get("filenames").and_then(|v| v.as_array()) {
                    let preview: Vec<String> = filenames.iter()
                        .take(3)
                        .map(|v| v.as_str().unwrap_or("").to_string())
                        .collect();
                    res_str = format!("[工具结果] 找到 {} 个文件: {:?}...", filenames.len(), preview);
                } else if let Some(file_info) = tool_res.get("file") {
                    let path = file_info["filePath"].as_str().unwrap_or("unknown");
                    let content = file_info["content"].as_str().unwrap_or("");
                    let snippet: String = content.chars().take(300).collect();
                    res_str = format!("[工具结果] 已读取文件 {}。\n文件片段: {}...", path, snippet.replace('\n', " "));
                } else if let Some(result) = tool_res.get("result") {
                    let raw = result.as_str().unwrap_or("");
                    let limit = if raw.to_lowercase().contains("error") { 800 } else { 300 };
                    let snippet: String = raw.chars().take(limit).collect();
                    res_str = format!("[工具结果] 输出: {}...", snippet);
                } else {
                    res_str = format!("[工具结果] {}", tool_res.to_string().chars().take(200).collect::<String>());
                }

                if !res_str.is_empty() {
                    events.push(ParsedEvent {
                        time: timestamp,
                        role: "system".to_string(),
                        content: res_str,
                        event_type: "tool_result".to_string(),
                    });
                }
            }
        }

        // 按时间排序
        events.sort_by(|a, b| a.time.cmp(&b.time));

        Ok(events)
    }

    /// 解析 LLM 响应
    fn parse_llm_response(response: &str) -> OptimizeResult {
        // 简单解析：尝试提取状态、分析和建议提示词
        let mut status = "正常".to_string();
        let mut analysis = String::new();
        let mut suggested_prompt = String::new();

        let lines: Vec<&str> = response.lines().collect();
        let mut current_section = String::new();

        for line in lines {
            if line.contains("【状态】") {
                if let Some(s) = line.split("【状态】").nth(1) {
                    status = s.trim().to_string();
                }
                current_section = "status".to_string();
            } else if line.contains("【分析】") {
                current_section = "analysis".to_string();
                if let Some(s) = line.split("【分析】").nth(1) {
                    analysis = s.trim().to_string();
                }
            } else if line.contains("【建议提示词】") {
                current_section = "prompt".to_string();
            } else {
                match current_section.as_str() {
                    "analysis" => {
                        if !analysis.is_empty() {
                            analysis.push('\n');
                        }
                        analysis.push_str(line.trim());
                    }
                    "prompt" => {
                        if !suggested_prompt.is_empty() {
                            suggested_prompt.push('\n');
                        }
                        suggested_prompt.push_str(line.trim());
                    }
                    _ => {}
                }
            }
        }

        // 如果没有解析到结构化内容，将整个响应作为建议提示词
        if suggested_prompt.is_empty() {
            suggested_prompt = response.trim().to_string();
            analysis = "LLM 返回了非结构化响应".to_string();
        }

        OptimizeResult {
            status,
            analysis,
            suggested_prompt,
        }
    }
}

// ==================== 辅助函数 ====================

/// 查找最近的 Session 文件
pub fn find_latest_session_file() -> Option<PathBuf> {
    let home_dir = dirs::home_dir()?;
    let base_pattern = home_dir.join(".claude/projects/**/*.jsonl");
    let pattern_str = base_pattern.to_str()?;

    let mut latest_file: Option<PathBuf> = None;
    let mut latest_time = SystemTime::UNIX_EPOCH;

    if let Ok(paths) = glob::glob(pattern_str) {
        for entry in paths.filter_map(Result::ok) {
            if let Ok(metadata) = std::fs::metadata(&entry) {
                if let Ok(modified) = metadata.modified() {
                    if modified > latest_time {
                        latest_time = modified;
                        latest_file = Some(entry);
                    }
                }
            }
        }
    }
    latest_file
}
