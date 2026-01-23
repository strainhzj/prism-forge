//! Prompt Optimizer - 提示词优化模块
//!
//! 使用 LLMClientManager 调用当前活跃的 LLM 提供商来分析和优化提示词

pub mod compressor;
pub mod prompt_generator;
pub mod config;

pub use config::{ConfigManager, OptimizerConfig};

use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use crate::llm::LLMClientManager;
use crate::llm::interface::{Message, ModelParams};
use crate::database::ApiProviderType;
use crate::filter_config::FilterConfigManager;

// ==================== 数据结构 ====================

/// 解析后的事件
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParsedEvent {
    pub time: String,
    pub role: String,
    pub content: String,
    #[serde(rename = "fullContent")]
    pub full_content: String,  // 完整内容用于详情弹窗
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

// ==================== 辅助函数 ====================

/// 根据提供商类型创建优化的 ModelParams
///
/// 不同提供商有不同的参数特性，此函数根据提供商类型返回优化的默认参数
fn create_optimizer_params(
    provider_type: ApiProviderType,
    model: &str,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
) -> ModelParams {
    // 使用配置的值，如果没有配置则使用提供商特定的默认值
    let temp = temperature.unwrap_or_else(|| match provider_type {
        ApiProviderType::Google | ApiProviderType::GoogleVertex => 0.5,
        ApiProviderType::Ollama => 0.6,
        _ => 0.7,
    });

    let tokens = max_tokens.unwrap_or_else(|| match provider_type {
        ApiProviderType::Anthropic => 4000,
        ApiProviderType::Ollama => 1500,
        _ => 2000,
    });

    ModelParams::new(model)
        .with_temperature(temp)
        .with_max_tokens(tokens)
}

/// 从配置创建优化的 ModelParams（支持 config_json 覆盖）
///
/// 优先级：数据库字段 > config_json > 提供商默认值
fn create_optimizer_params_with_config(
    provider_type: ApiProviderType,
    model: &str,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    config_json: Option<&String>,
) -> Result<ModelParams> {
    // 先使用数据库配置的值
    let mut temp = temperature;
    let mut tokens = max_tokens;

    // 如果有扩展配置，尝试解析并覆盖
    if let Some(config_str) = config_json {
        if let Ok(config) = serde_json::from_str::<serde_json::Value>(config_str) {
            if let Some(v) = config.get("optimizer_temperature").and_then(|v| v.as_f64()) {
                temp = Some(v as f32);
            }
            if let Some(v) = config.get("optimizer_max_tokens").and_then(|v| v.as_u64()) {
                tokens = Some(v as u32);
            }
        }
    }

    Ok(create_optimizer_params(provider_type, model, temp, tokens))
}

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

        // 4. 获取活跃提供商配置
        let provider = llm_manager.get_active_provider_config()
            .context("无法获取活跃提供商配置")?;
        let model = provider.effective_model();

        #[cfg(debug_assertions)]
        eprintln!("[PromptOptimizer] Using provider: {:?}, model: {}",
                  provider.provider_type, model);

        // 5. 创建提供商特定的参数
        let params = create_optimizer_params_with_config(
            provider.provider_type,
            model,
            provider.temperature,
            provider.max_tokens,
            provider.config_json.as_ref(),
        ).context("创建优化器参数失败")?;

        // 打印参数详情
        #[cfg(debug_assertions)]
        {
            eprintln!("[PromptOptimizer] Params:");
            eprintln!("  - model: {}", params.model);
            eprintln!("  - temperature: {}", params.temperature);
            eprintln!("  - max_tokens: {:?}", params.max_tokens);
        }

        // 6. 调用 LLM
        let client = llm_manager.get_active_client()
            .context("无法获取 LLM 客户端")?;

        let messages = vec![
            Message::user(full_text)
        ];

        let response = client.chat_completion(messages, params).await?;

        // 5. 解析 LLM 响应 - ChatCompletionResponse.content 是 String
        let result = Self::parse_llm_response(&response.content);

        Ok(result)
    }

    /// 解析会话文件（公共方法，可供 Tauri 命令调用）
    pub fn parse_session_file(file_path: &str) -> Result<Vec<ParsedEvent>> {
        // 加载过滤配置
        let filter_manager = FilterConfigManager::with_default_path()
            .context("加载过滤配置失败")?;

        #[cfg(debug_assertions)]
        eprintln!("[parse_session_file] 过滤配置已加载，规则数量: {}",
                  filter_manager.get_config().rules.len());

        let file = File::open(file_path)
            .context(format!("无法打开会话文件: {}", file_path))?;
        let reader = BufReader::new(file);
        let mut events = Vec::new();
        let mut filtered_count = 0;

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
                    // 检查是否需要过滤
                    if filter_manager.should_filter(&final_text) {
                        filtered_count += 1;
                        continue;
                    }

                    events.push(ParsedEvent {
                        time: timestamp.clone(),
                        role,
                        content: final_text.clone(),
                        full_content: final_text,  // 普通消息的完整内容和截断内容相同
                        event_type: "message".to_string(),
                    });
                }
            }

            // 2. 提取 ToolResult (上下文关键)
            if let Some(tool_res) = json.get("toolUseResult") {
                let mut res_str = String::new();
                let mut full_res_str = String::new();

                if let Some(filenames) = tool_res.get("filenames").and_then(|v| v.as_array()) {
                    // 截断版本：只显示前3个文件
                    let preview: Vec<String> = filenames.iter()
                        .take(3)
                        .map(|v| v.as_str().unwrap_or("").to_string())
                        .collect();
                    res_str = format!("[工具结果] 找到 {} 个文件: {:?}...", filenames.len(), preview);
                    // 完整版本：显示所有文件
                    let all_files: Vec<String> = filenames.iter()
                        .map(|v| v.as_str().unwrap_or("").to_string())
                        .collect();
                    full_res_str = format!("[工具结果] 找到 {} 个文件:\n{}", filenames.len(),
                        all_files.join("\n"));
                } else if let Some(file_info) = tool_res.get("file") {
                    let path = file_info["filePath"].as_str().unwrap_or("unknown");
                    let content = file_info["content"].as_str().unwrap_or("");
                    // 截断版本：只显示前300字符
                    let snippet: String = content.chars().take(300).collect();
                    res_str = format!("[工具结果] 已读取文件 {}。\n文件片段: {}...", path, snippet.replace('\n', " "));
                    // 完整版本：显示完整内容
                    full_res_str = format!("[工具结果] 已读取文件: {}\n完整内容:\n{}", path, content);
                } else if let Some(result) = tool_res.get("result") {
                    let raw = result.as_str().unwrap_or("");
                    // 截断版本
                    let limit = if raw.to_lowercase().contains("error") { 800 } else { 300 };
                    let snippet: String = raw.chars().take(limit).collect();
                    res_str = format!("[工具结果] 输出: {}...", snippet);
                    // 完整版本
                    full_res_str = format!("[工具结果] 完整输出:\n{}", raw);
                } else {
                    // 其他情况
                    let raw = tool_res.to_string();
                    let snippet: String = raw.chars().take(200).collect();
                    res_str = format!("[工具结果] {}...", snippet);
                    full_res_str = format!("[工具结果] 完整数据:\n{}", raw);
                }

                if !res_str.is_empty() {
                    // 检查是否需要过滤（同时检查截断和完整内容）
                    if filter_manager.should_filter(&res_str) || filter_manager.should_filter(&full_res_str) {
                        filtered_count += 1;
                        continue;
                    }

                    events.push(ParsedEvent {
                        time: timestamp,
                        role: "system".to_string(),
                        content: res_str,
                        full_content: full_res_str,  // 保存完整内容
                        event_type: "tool_result".to_string(),
                    });
                }
            }
        }

        // 按时间排序
        events.sort_by(|a, b| a.time.cmp(&b.time));

        // 输出过滤统计
        #[cfg(debug_assertions)]
        if filtered_count > 0 {
            eprintln!("[parse_session_file] 已过滤 {} 条事件", filtered_count);
        }

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

/// 查找指定项目下最新的 Session 文件
///
/// # 参数
/// * `project_path` - 项目路径（监控目录路径，如 C:\software\my-project）
///
/// # 返回
/// 返回该项目对应的 Claude Desktop 会话目录中修改时间最新的 .jsonl 文件
///
/// # 路径解析逻辑
/// Claude Desktop 将项目路径转换后存储在 ~/.claude/projects/ 目录下：
/// - C:\software\my-project → C--software-my-project
/// - 会话文件位于: ~/.claude/projects/C--software-my-project/*.jsonl
pub fn find_latest_session_file_in_project(project_path: &str) -> Option<PathBuf> {
    use crate::path_resolver::list_session_files;
    use std::path::Path;

    let project = Path::new(project_path);

    #[cfg(debug_assertions)]
    eprintln!("[find_latest_session_file_in_project] 项目路径: {:?}", project);

    // 使用 path_resolver 模块解析会话目录并获取文件列表
    // list_session_files 已经按修改时间倒序排序，第一个就是最新的
    match list_session_files(project) {
        Ok(mut files) => {
            if !files.is_empty() {
                let latest = files.remove(0); // 取第一个（最新的）
                #[cfg(debug_assertions)]
                eprintln!("[find_latest_session_file_in_project] 找到最新文件: {:?}", latest.full_path);
                Some(latest.full_path)
            } else {
                #[cfg(debug_assertions)]
                eprintln!("[find_latest_session_file_in_project] 未找到会话文件");
                None
            }
        }
        Err(e) => {
            #[cfg(debug_assertions)]
            eprintln!("[find_latest_session_file_in_project] 解析失败: {}", e);
            None
        }
    }
}
