//! 提示词生成器模块
//!
//! 整合上下文压缩和 LLM 生成，创建增强的提示词优化功能
//! 注意：向量检索功能已暂时移除，改用简单的最近会话检索

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

use crate::llm::{LLMClientManager, interface::{Message, ModelParams}};
use crate::database::repository::SessionRepository;
use crate::tokenizer::TokenCounter;
use crate::parser::view_level::{ViewLevel, MessageFilter, QAPair};
use crate::session_parser::{SessionParserService, SessionParserConfig};
use super::compressor::ContextCompressor;
use super::config::ConfigManager;

// ==================== 数据结构 ====================

/// 会话消息（用于 JSON 序列化）
///
/// 将 QAPair 转换为统一的 JSON 格式，注入到提示词模板中
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMessage {
    /// 消息文本内容
    pub text: String,
    /// 消息角色 (user/assistant)
    pub role: String,
    /// 消息时间戳
    pub timestamp: String,
}

/// 增强提示词请求
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnhancedPromptRequest {
    /// 用户目标
    pub goal: String,
    /// 可选：当前跟踪的会话文件路径（首页展示的会话）
    #[serde(rename = "currentSessionFilePath")]
    pub current_session_file_path: Option<String>,
}

/// Token 统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenStats {
    /// 原始 Token 数
    pub original_tokens: usize,
    /// 压缩后 Token 数
    pub compressed_tokens: usize,
    /// 节省百分比
    pub savings_percentage: f64,
}

/// 引用的会话信息（简化版本，不包含相似度）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferencedSession {
    /// 会话 ID
    #[serde(rename = "sessionId")]
    pub session_id: String,
    /// 项目名称
    #[serde(rename = "projectName")]
    pub project_name: String,
    /// 摘要
    pub summary: String,
    /// 相关性分数（基于评分和最近更新时间）
    #[serde(rename = "similarityScore")]
    pub similarity_score: f64,
}

/// 增强提示词结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnhancedPrompt {
    /// 原始目标
    #[serde(rename = "originalGoal")]
    pub original_goal: String,
    /// 引用的会话
    #[serde(rename = "referencedSessions")]
    pub referenced_sessions: Vec<ReferencedSession>,
    /// 增强的提示词
    #[serde(rename = "enhancedPrompt")]
    pub enhanced_prompt: String,
    /// Token 统计
    #[serde(rename = "tokenStats")]
    pub token_stats: TokenStats,
    /// 置信度 (0.0 - 1.0)
    pub confidence: f64,
}

// ==================== 提示词生成器 ====================

/// 提示词生成器
///
/// 整合上下文压缩和 LLM 生成（不使用向量检索）
pub struct PromptGenerator {
    /// 数据库仓库
    repository: SessionRepository,
    /// 上下文压缩器
    compressor: ContextCompressor,
    /// Token 计数器
    token_counter: TokenCounter,
    /// 配置管理器
    config_manager: Arc<ConfigManager>,
}

impl PromptGenerator {
    /// 创建新的提示词生成器
    pub fn new() -> Result<Self> {
        // 初始化配置管理器
        // 优先级：开发环境使用项目根目录，生产环境使用可执行文件目录
        let config_path = Self::resolve_config_path()?;

        #[cfg(debug_assertions)]
        eprintln!("[PromptGenerator] 配置文件路径: {:?}", config_path);

        let config_manager = Arc::new(ConfigManager::new(config_path)?);

        Ok(Self {
            repository: SessionRepository::from_default_db()?,
            compressor: ContextCompressor::new()?,
            token_counter: TokenCounter::new()?,
            config_manager,
        })
    }

    /// 解析配置文件路径
    ///
    /// 优先级：
    /// 1. 开发模式：从可执行文件位置向上查找项目根目录，然后定位 src-tauri/optimizer_config.toml
    /// 2. 生产模式：使用可执行文件同目录的 optimizer_config.toml
    fn resolve_config_path() -> Result<PathBuf> {
        use std::env;

        let exe_path = env::current_exe()
            .map_err(|e| anyhow::anyhow!("无法获取可执行文件路径: {}", e))?;

        let exe_dir = exe_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        #[cfg(debug_assertions)]
        {
            eprintln!("[PromptGenerator] 可执行文件路径: {:?}", exe_path);
            eprintln!("[PromptGenerator] 可执行文件目录: {:?}", exe_dir);
        }

        // 策略 1: 从可执行文件目录向上查找项目根目录
        // 开发环境结构: project/src-tauri/target/debug/prism-forge.exe
        // 需要向上查找直到找到包含 src-tauri 的目录（项目根目录）
        let mut current_dir = exe_dir.clone();
        let max_depth = 5; // 最多向上查找 5 层

        for depth in 0..=max_depth {
            // 检查当前目录下是否有 src-tauri 子目录
            let src_tauri_path = current_dir.join("src-tauri");
            let config_path = src_tauri_path.join("optimizer_config.toml");

            if config_path.exists() {
                #[cfg(debug_assertions)]
                eprintln!("[PromptGenerator] 找到开发环境配置（向上查找 {} 层）: {:?}", depth, config_path);
                return Ok(config_path);
            }

            // 向上一级目录
            if !current_dir.pop() {
                break; // 已经到达根目录
            }
        }

        // 策略 2: 生产环境 - 配置文件在可执行文件同目录
        let prod_path = exe_dir.join("optimizer_config.toml");

        if prod_path.exists() {
            #[cfg(debug_assertions)]
            eprintln!("[PromptGenerator] 使用生产环境配置路径: {:?}", prod_path);
            return Ok(prod_path);
        }

        // 策略 3: 回退 - 尝试使用当前工作目录 + src-tauri（仅限开发环境）
        let cwd_path = env::current_dir()
            .map(|d| d.join("src-tauri").join("optimizer_config.toml"))
            .ok();

        if let Some(ref path) = cwd_path {
            if path.exists() {
                #[cfg(debug_assertions)]
                eprintln!("[PromptGenerator] 使用当前工作目录配置路径: {:?}", path);
                return Ok(path.clone());
            }
        }

        // 所有策略都失败，返回错误信息
        Err(anyhow::anyhow!(
            "无法找到配置文件 optimizer_config.toml\n\
             可执行文件目录: {:?}\n\
             尝试的生产路径: {:?}\n\
             尝试的开发路径: {:?}",
            exe_dir, prod_path, cwd_path
        ))
    }

    /// 使用自定义配置路径创建提示词生成器
    pub fn with_config_path(config_path: PathBuf) -> Result<Self> {
        let config_manager = Arc::new(ConfigManager::new(config_path)?);

        Ok(Self {
            repository: SessionRepository::from_default_db()?,
            compressor: ContextCompressor::new()?,
            token_counter: TokenCounter::new()?,
            config_manager,
        })
    }

    /// 重新加载配置
    pub fn reload_config(&self) -> Result<()> {
        self.config_manager.reload()
    }

    /// 生成增强提示词（主流程）
    ///
    /// # 新流程（使用当前会话的 QAPairs）
    /// 1. 检查是否有当前会话文件路径
    /// 2. 如果有，解析会话并提取 QAPairs（问答对）
    /// 3. 如果问答对为空，使用对话开始模板
    /// 4. 将问答对转换为对话流格式
    /// 5. 构建 Meta-Prompt 并调用 LLM 生成
    ///
    /// # 参数
    /// - `request`: 增强提示词请求
    /// - `llm_manager`: LLM 客户端管理器
    /// - `language`: 语言标识（"zh" 或 "en"）
    pub async fn generate_enhanced_prompt(
        &self,
        request: EnhancedPromptRequest,
        llm_manager: &LLMClientManager,
        language: &str,
    ) -> Result<EnhancedPrompt> {
        // 1. 检查是否有当前会话文件路径
        if let Some(ref session_file_path) = request.current_session_file_path {
            // 检查文件是否存在
            let path_buf = PathBuf::from(session_file_path);
            if !path_buf.exists() {
                return Err(anyhow::anyhow!("会话文件不存在: {}", session_file_path));
            }

            // 提取 session_id（从文件名）
            let session_id = path_buf
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            // 2. 解析会话并提取问答对
            let config = SessionParserConfig {
                enable_content_filter: false,
                view_level: ViewLevel::Full,
                debug: cfg!(debug_assertions),
            };

            let parser = SessionParserService::new(config);
            let parse_result = parser.parse_session(session_file_path, session_id)
                .map_err(|e| anyhow::anyhow!("解析会话失败: {}", e))?;

            // 3. 提取问答对
            let filter = MessageFilter::new(ViewLevel::QAPairs);
            let qa_pairs = filter.extract_qa_pairs(parse_result.messages);

            #[cfg(debug_assertions)]
            eprintln!("[PromptGenerator] 提取到 {} 个问答对", qa_pairs.len());

            // 4. 判断问答对是否为空
            if qa_pairs.is_empty() {
                // 方案 B: 调用 LLM 生成对话开始提示词
                return self.generate_conversation_starter_with_llm(&request.goal, session_file_path, session_id, llm_manager, language).await;
            }

            // 5. 将问答对转换为对话流格式（时间正序）
            let (original_tokens, conversation_context) = self.format_qa_pairs_to_conversation(&qa_pairs)?;

            // 6. 构建完整提示词
            let full_prompt = self.build_prompt_with_conversation(
                &request.goal,
                &conversation_context,
                language,
            );

            // 7. 调用 LLM 生成增强提示词
            let enhanced_prompt = match self.call_llm_generate(&full_prompt, llm_manager).await {
                Ok(prompt) => {
                    #[cfg(debug_assertions)]
                    eprintln!("[PromptGenerator] LLM 生成成功，长度: {}", prompt.len());
                    prompt
                },
                Err(e) => {
                    // LLM 调用失败时，回退到模板生成
                    #[cfg(debug_assertions)]
                    eprintln!("[PromptGenerator] LLM 调用失败，使用模板: {}", e);
                    self.generate_conversation_template_prompt(&request.goal, language)
                }
            };

            // 8. 计算 Token 统计
            let compressed_tokens = self.token_counter.count_tokens(&enhanced_prompt)?;
            let savings_percentage = if original_tokens > 0 && compressed_tokens <= original_tokens {
                ((original_tokens - compressed_tokens) as f64 / original_tokens as f64) * 100.0
            } else if original_tokens > 0 {
                -(((compressed_tokens - original_tokens) as f64 / original_tokens as f64) * 100.0)
            } else {
                0.0
            };

            // 9. 构建引用会话信息
            let (project_name, summary) = if language == "en" {
                ("Current Session".to_string(), format!("Contains {} Q&A pairs", qa_pairs.len()))
            } else {
                ("当前会话".to_string(), format!("包含 {} 个问答对", qa_pairs.len()))
            };

            let referenced_sessions = vec![ReferencedSession {
                session_id: session_id.to_string(),
                project_name,
                summary,
                similarity_score: 1.0,
            }];

            Ok(EnhancedPrompt {
                original_goal: request.goal,
                referenced_sessions,
                enhanced_prompt,
                token_stats: TokenStats {
                    original_tokens,
                    compressed_tokens,
                    savings_percentage,
                },
                confidence: 1.0, // 当前会话置信度最高
            })
        } else {
            // 没有当前会话，返回提示要求用户先选择会话
            let error_msg = if language == "en" {
                "Please select a session on the home page first"
            } else {
                "请先在首页选择一个会话"
            };
            Err(anyhow::anyhow!(error_msg))
        }
    }

    /// 将问答对转换为 JSON 格式的会话消息列表
    fn format_qa_pairs_to_conversation(&self, qa_pairs: &[QAPair]) -> Result<(usize, String)> {
        use crate::database::models::Message;

        // 构建 SessionMessage 列表（时间正序）
        let session_messages: Vec<SessionMessage> = qa_pairs.iter().flat_map(|pair| {
            let mut messages = Vec::new();

            // 用户问题
            let user_text = pair.question.content.as_ref()
                .or(pair.question.summary.as_ref())
                .unwrap_or(&String::new())
                .clone();

            messages.push(SessionMessage {
                text: user_text,
                role: "user".to_string(),
                timestamp: pair.question.timestamp.clone(),
            });

            // 助手回复（如果有）
            if let Some(ref answer) = pair.answer {
                let assistant_text = answer.content.as_ref()
                    .or(answer.summary.as_ref())
                    .unwrap_or(&String::new())
                    .clone();

                messages.push(SessionMessage {
                    text: assistant_text,
                    role: "assistant".to_string(),
                    timestamp: answer.timestamp.clone(),
                });
            }

            messages
        }).collect();

        // 序列化为 JSON 字符串
        let json_str = serde_json::to_string_pretty(&session_messages)
            .map_err(|e| anyhow::anyhow!("序列化 SessionMessage 失败: {}", e))?;

        // 计算 Token 数
        let original_tokens = self.token_counter.count_tokens(&json_str)?;

        Ok((original_tokens, json_str))
    }

    /// 使用对话上下文构建完整提示词
    fn build_prompt_with_conversation(
        &self,
        goal: &str,
        conversation: &str,
        language: &str,
    ) -> String {
        // 从配置获取模板
        let meta_prompt = self.config_manager.get_meta_prompt(language);
        let prompt_structure = self.config_manager.get_prompt_structure(language);

        // 组装完整提示词
        prompt_structure
            .replace("{{meta_prompt}}", &meta_prompt)
            .replace("{{goal}}", goal)
            .replace("{{sessions}}", conversation)
    }

    /// 生成对话开始提示词（会话为空时，使用 LLM 生成）
    async fn generate_conversation_starter_with_llm(
        &self,
        goal: &str,
        session_file_path: &str,
        session_id: &str,
        llm_manager: &LLMClientManager,
        language: &str,
    ) -> Result<EnhancedPrompt> {
        // 1. 构建对话开始的完整提示词
        let full_prompt = self.build_conversation_starter_prompt(goal, language);

        // 2. 调用 LLM 生成增强提示词
        let enhanced_prompt = match self.call_llm_generate(&full_prompt, llm_manager).await {
            Ok(prompt) => {
                #[cfg(debug_assertions)]
                eprintln!("[PromptGenerator] 对话开始提示词生成成功，长度: {}", prompt.len());
                prompt
            },
            Err(e) => {
                // LLM 调用失败时，使用回退模板
                #[cfg(debug_assertions)]
                eprintln!("[PromptGenerator] LLM 调用失败，使用回退模板: {}", e);
                self.generate_conversation_fallback_template(goal, language)
            }
        };

        // 3. 计算 Token 统计
        let compressed_tokens = self.token_counter.count_tokens(&enhanced_prompt)?;
        // 对话开始没有原始上下文，所以 original_tokens 设为 0
        let original_tokens = 0;
        let savings_percentage = 0.0;

        // 4. 构建引用会话信息
        let (project_name, summary) = if language == "en" {
            ("Current Session".to_string(), "Conversation start (AI-generated)".to_string())
        } else {
            ("当前会话".to_string(), "对话开始（AI 生成）".to_string())
        };

        let referenced_sessions = vec![ReferencedSession {
            session_id: session_id.to_string(),
            project_name,
            summary,
            similarity_score: 1.0,
        }];

        Ok(EnhancedPrompt {
            original_goal: goal.to_string(),
            referenced_sessions,
            enhanced_prompt,
            token_stats: TokenStats {
                original_tokens,
                compressed_tokens,
                savings_percentage,
            },
            confidence: 0.7, // 对话开始的置信度（LLM 生成，置信度中等偏高）
        })
    }

    /// 创建对话开始提示词（会话为空时，已弃用，保留用于兼容）
    #[deprecated(note = "使用 generate_conversation_starter_with_llm 代替")]
    fn create_conversation_starter_prompt(&self, goal: &str, session_file_path: &str, language: &str) -> EnhancedPrompt {
        // 从配置获取对话开始模板（默认使用中文）
        let template = self.config_manager.get_conversation_starter_template(language);

        let enhanced_prompt = template.replace("{{goal}}", goal);

        // 提取会话信息
        let path_buf = PathBuf::from(session_file_path);
        let session_id = path_buf
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let (project_name, summary) = if language == "en" {
            ("Current Session".to_string(), "New conversation, no history".to_string())
        } else {
            ("当前会话".to_string(), "新对话，无历史记录".to_string())
        };

        EnhancedPrompt {
            original_goal: goal.to_string(),
            referenced_sessions: vec![ReferencedSession {
                session_id: session_id.to_string(),
                project_name,
                summary,
                similarity_score: 1.0,
            }],
            enhanced_prompt,
            token_stats: TokenStats {
                original_tokens: 0,
                compressed_tokens: goal.len(),
                savings_percentage: 0.0,
            },
            confidence: 0.5, // 对话开始的置信度中等
        }
    }

    /// 生成对话模板提示词（LLM 调用失败时回退）
    fn generate_conversation_template_prompt(&self, goal: &str, language: &str) -> String {
        if language == "en" {
            format!(
                r#"Generate an optimized prompt based on the following goal:

{goal}

Requirements:
1. Concise and to the point
2. Include necessary context
3. Clear structure
4. Suitable for programming assistant prompts"#
            )
        } else {
            format!(
                r#"请基于以下目标生成一个优化的提示词：

{goal}

要求：
1. 简洁明了，直击要点
2. 包含必要的上下文信息
3. 结构清晰，易于理解
4. 适合作为编程助手的开场提示词"#
            )
        }
    }

    /// 构建对话开始的完整提示词（使用配置的 conversation_starter_template）
    fn build_conversation_starter_prompt(&self, goal: &str, language: &str) -> String {
        // 从配置获取对话开始模板
        let template = self.config_manager.get_conversation_starter_template(language);

        // 替换变量
        template.replace("{{goal}}", goal)
    }

    /// 生成对话回退模板（LLM 调用失败时使用）
    fn generate_conversation_fallback_template(&self, goal: &str, language: &str) -> String {
        if language == "en" {
            format!(
                r#"You are a professional programming assistant. The user wants to start a new conversation.

## User Goal
{goal}

## Suggestions
1. Analyze the user's goal and understand their requirements
2. Ask targeted questions to clarify requirement details
3. Provide relevant technical suggestions or reference directions
4. Maintain a friendly and professional attitude

Please generate a conversation-starting prompt based on the above information."#
            )
        } else {
            format!(
                r#"你是一个专业的编程助手。用户想要开始一个新的对话。

## 用户目标
{goal}

## 建议
1. 分析用户的目标，理解其需求
2. 提出针对性的问题来明确需求细节
3. 提供相关的技术建议或参考方向
4. 保持友好和专业的态度

请基于以上信息生成一个对话开始的提示词。"#
            )
        }
    }

    /// 调用 LLM 生成增强提示词
    async fn call_llm_generate(
        &self,
        prompt: &str,
        llm_manager: &LLMClientManager,
    ) -> Result<String> {
        let provider = llm_manager.get_active_provider_config()
            .context("无法获取活跃提供商配置")?;
        let model = provider.effective_model();

        // 从配置获取 LLM 参数
        let llm_params = self.config_manager.get_llm_params();

        // 使用配置的参数创建 ModelParams
        let params = ModelParams::new(model)
            .with_temperature(llm_params.temperature)
            .with_max_tokens(llm_params.max_tokens as u32);

        let client = llm_manager.get_active_client()
            .context("无法获取 LLM 客户端")?;

        let messages = vec![Message::user(prompt)];

        let response = client.chat_completion(messages, params).await?;

        Ok(response.content)
    }

    /// 测试辅助方法：直接构建提示词（不调用 LLM）
    ///
    /// 此方法仅用于单元测试，验证模板加载和变量替换是否正确
    #[cfg(test)]
    #[doc(hidden)]
    pub fn test_build_prompt(
        &self,
        goal: &str,
        sessions: &str,
        language: &str,
    ) -> String {
        // 直接调用私有方法
        self.build_prompt_with_conversation(goal, sessions, language)
    }
}

// ==================== 测试模块 ====================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizer::config::OptimizerConfig;
    use std::sync::RwLock;

    // 使用全局互斥锁确保需要数据库的测试顺序执行
    static DB_TEST_LOCK: RwLock<()> = RwLock::new(());

    #[test]
    fn test_build_prompt_from_config() {
        let _lock = DB_TEST_LOCK.read().unwrap();
        // 获取锁，确保测试顺序执行

        // 1. 解析配置文件路径（使用开发环境的默认路径）
        let config_path = std::path::PathBuf::from("optimizer_config.toml");

        // 如果配置文件不存在，跳过测试
        if !config_path.exists() {
            println!("警告: 配置文件 optimizer_config.toml 不存在，跳过测试");
            println!("预期路径: {:?}", config_path);
            println!("当前工作目录: {:?}", std::env::current_dir().unwrap());
            return;
        }

        // 2. 创建 PromptGenerator 实例
        let generator = match PromptGenerator::with_config_path(config_path) {
            Ok(gen) => gen,
            Err(e) => {
                panic!("创建 PromptGenerator 失败: {}", e);
            }
        };

        // 3. 准备测试数据（新 JSON 格式）
        let goal = "Write a Hello World program in Python";
        let sessions = r#"[
  {
    "text": "How do I print in Python?",
    "role": "user",
    "timestamp": "2025-01-22T10:00:00Z"
  },
  {
    "text": "You can use the print() function.",
    "role": "assistant",
    "timestamp": "2025-01-22T10:00:01Z"
  },
  {
    "text": "Show me an example",
    "role": "user",
    "timestamp": "2025-01-22T10:00:02Z"
  },
  {
    "text": "print(\"Hello, World!\")",
    "role": "assistant",
    "timestamp": "2025-01-22T10:00:03Z"
  }
]"#;

        // 4. 调用测试辅助方法生成提示词（测试中文版本）
        let result = generator.test_build_prompt(goal, sessions, "zh");

        // 5. 打印生成的提示词（便于人工检查）
        println!("\n========== 生成的提示词 ==========\n");
        println!("{}", result);
        println!("\n========== 提示词结束 ==========\n");

        // 6. 验证结果包含预期的结构
        let assertions = vec![
            // Meta-Prompt 的内容（已更新为新版本）
            ("Meta-Prompt 标题", "专业的提示词工程师"),
            ("分析方法步骤", "分析目标与上下文"),
            ("限制条件", "提示词应简洁明了"),

            // Prompt Structure 的内容
            ("输入信息标题", "## 输入信息"),
            ("用户目标标签", "**下一步目标**"),
            ("会话标签", "**相关历史会话**"),

            // 输出格式说明（更新后的模板）
            ("输出格式标题", "## 输出格式"),
            ("目标偏离程度标题", "### **目标偏离程度**"),
            ("任务目标标题", "### **任务目标**"),
            ("具体步骤标题", "### **具体步骤**"),
            ("预期输出标题", "### **预期输出**"),

            // 注入的变量内容
            ("注入的 goal", "Write a Hello World program in Python"),
            ("注入的 sessions (text)", "How do I print in Python?"),
            ("注入的 sessions (role)", "\"user\""),
            ("注入的 sessions (timestamp)", "\"timestamp\""),
        ];

        for (description, expected) in assertions {
            assert!(
                result.contains(expected),
                "验证失败: {} - 未找到预期内容 '{}'\n生成的提示词:\n{}",
                description,
                expected,
                result
            );
        }

        // 7. 验证变量占位符已被替换
        assert!(
            !result.contains("{{goal}}") && !result.contains("{{sessions}}") && !result.contains("{{meta_prompt}}"),
            "错误: 变量占位符未被完全替换\n发现的占位符: {}",
            if result.contains("{{goal}}") { " {{goal}}" }
            else if result.contains("{{sessions}}") { " {{sessions}}" }
            else if result.contains("{{meta_prompt}}") { " {{meta_prompt}}" }
            else { "" }
        );

        println!("✅ 所有断言通过！");
    }

    #[test]
    fn test_config_loading() {
        let _lock = DB_TEST_LOCK.read().unwrap();
        // 获取锁，确保测试顺序执行

        // 测试配置文件是否能正确加载
        let config_path = std::path::PathBuf::from("optimizer_config.toml");

        if !config_path.exists() {
            println!("警告: 配置文件不存在，跳过测试");
            return;
        }

        let generator = PromptGenerator::with_config_path(config_path)
            .expect("创建 PromptGenerator 失败");

        // 验证配置管理器能正确读取配置
        let meta_prompt = generator.config_manager.get_meta_prompt("zh");
        let prompt_structure = generator.config_manager.get_prompt_structure("zh");

        println!("Meta-Prompt 长度: {} 字符", meta_prompt.len());
        println!("Prompt Structure 长度: {} 字符", prompt_structure.len());

        assert!(
            !meta_prompt.is_empty(),
            "Meta-Prompt 不应为空"
        );
        assert!(
            !prompt_structure.is_empty(),
            "Prompt Structure 不应为空"
        );
        assert!(
            prompt_structure.contains("{{meta_prompt}}"),
            "Prompt Structure 应包含 {{meta_prompt}} 占位符"
        );
        assert!(
            prompt_structure.contains("{{goal}}"),
            "Prompt Structure 应包含 {{goal}} 占位符"
        );
        assert!(
            prompt_structure.contains("{{sessions}}"),
            "Prompt Structure 应包含 {{sessions}} 占位符"
        );

        println!("✅ 配置加载测试通过！");
    }

    // ==================== 语言路由测试 ====================

    /// 测试中文语言路由
    ///
    /// 验证：当传入 `language: "zh"` 时，系统正确加载中文模板
    #[test]
    fn test_zh_routing() {

        // 1. 创建带 Mock 配置的 PromptGenerator
        let config = create_mock_optimizer_config();
        let generator = create_mock_prompt_generator(config);

        // 2. 准备测试数据
        let goal = "创建一个 Rust 函数";
        let sessions = r#"[{"text":"如何定义结构体？","role":"user","timestamp":"2025-01-22T10:00:00Z"}]"#;

        // 3. 调用生成方法（传入中文语言代码）
        let result = generator.test_build_prompt(goal, sessions, "zh");

        // 4. 断言：验证中文标记存在
        assert!(
            result.contains(MOCK_ZH_MARKER),
            "错误: 中文路由失败，未找到中文标记 '{}'\n生成的提示词:\n{}",
            MOCK_ZH_MARKER,
            result
        );

        // 5. 断言：验证英文标记不存在
        assert!(
            !result.contains(MOCK_EN_MARKER),
            "错误: 中文路由中混入了英文标记 '{}'\n生成的提示词:\n{}",
            MOCK_EN_MARKER,
            result
        );

        println!("✅ 中文路由测试通过！");
    }

    /// 测试英文语言路由
    ///
    /// 验证：当传入 `language: "en"` 时，系统正确加载英文模板
    #[test]
    fn test_en_routing() {

        // 1. 创建带 Mock 配置的 PromptGenerator
        let config = create_mock_optimizer_config();
        let generator = create_mock_prompt_generator(config);

        // 2. 准备测试数据
        let goal = "Create a Rust function";
        let sessions = r#"[{"text":"How to define a struct?","role":"user","timestamp":"2025-01-22T10:00:00Z"}]"#;

        // 3. 调用生成方法（传入英文语言代码）
        let result = generator.test_build_prompt(goal, sessions, "en");

        // 4. 断言：验证英文标记存在
        assert!(
            result.contains(MOCK_EN_MARKER),
            "错误: 英文路由失败，未找到英文标记 '{}'\n生成的提示词:\n{}",
            MOCK_EN_MARKER,
            result
        );

        // 5. 断言：验证中文标记不存在
        assert!(
            !result.contains(MOCK_ZH_MARKER),
            "错误: 英文路由中混入了中文标记 '{}'\n生成的提示词:\n{}",
            MOCK_ZH_MARKER,
            result
        );

        println!("✅ 英文路由测试通过！");
    }

    /// 测试默认语言回退逻辑
    ///
    /// 验证：当传入不支持的语言代码时，系统回退到默认语言（英文）
    #[test]
    fn test_default_language_fallback() {

        // 1. 创建带 Mock 配置的 PromptGenerator
        let config = create_mock_optimizer_config();
        let generator = create_mock_prompt_generator(config);

        // 2. 准备测试数据
        let goal = "Test goal";
        let sessions = "[]";

        // 3. 调用生成方法（传入不支持的语言代码，如 "fr"）
        let result = generator.test_build_prompt(goal, sessions, "fr");

        // 4. 断言：应该回退到英文（因为不支持的语言默认使用英文）
        assert!(
            result.contains(MOCK_EN_MARKER),
            "错误: 默认语言回退失败，未找到英文标记 '{}'\n生成的提示词:\n{}",
            MOCK_EN_MARKER,
            result
        );

        assert!(
            !result.contains(MOCK_ZH_MARKER),
            "错误: 默认回退中混入了中文标记 '{}'\n生成的提示词:\n{}",
            MOCK_ZH_MARKER,
            result
        );

        println!("✅ 默认语言回退测试通过！");
    }

    /// 测试会话上下文的多语言支持
    ///
    /// 验证：会话格式化模板也支持语言切换
    #[test]
    fn test_session_format_multilingual() {

        // 1. 创建 Mock 配置
        let config = create_mock_optimizer_config();
        let manager = MockConfigManager::new(config);

        // 2. 测试中文会话格式
        let zh_format = manager.get_session_format("zh");
        assert!(
            zh_format.contains("会话"),
            "错误: 中文会话格式应包含 '会话'，实际: {}",
            zh_format
        );
        assert!(
            zh_format.contains("项目"),
            "错误: 中文会话格式应包含 '项目'，实际: {}",
            zh_format
        );

        // 3. 测试英文会话格式
        let en_format = manager.get_session_format("en");
        assert!(
            en_format.contains("Session"),
            "错误: 英文会话格式应包含 'Session'，实际: {}",
            en_format
        );
        assert!(
            en_format.contains("Project"),
            "错误: 英文会话格式应包含 'Project'，实际: {}",
            en_format
        );

        println!("✅ 会话格式多语言测试通过！");
    }

    /// 测试回退模板的多语言支持
    ///
    /// 验证：所有回退模板（conversation_template, fallback_template）都支持语言切换
    #[test]
    fn test_fallback_templates_multilingual() {

        // 1. 创建 Mock 配置
        let config = create_mock_optimizer_config();
        let manager = MockConfigManager::new(config);

        // 2. 测试对话开始模板
        let zh_starter = manager.get_conversation_starter_template("zh");
        assert!(
            zh_starter.contains(MOCK_ZH_STARTER_MARKER),
            "错误: 中文对话开始模板应包含标记 '{}'",
            MOCK_ZH_STARTER_MARKER
        );

        let en_starter = manager.get_conversation_starter_template("en");
        assert!(
            en_starter.contains(MOCK_EN_STARTER_MARKER),
            "错误: 英文对话开始模板应包含标记 '{}'",
            MOCK_EN_STARTER_MARKER
        );

        // 3. 测试无会话回退模板
        let zh_no_sessions = manager.get_no_sessions_template("zh");
        assert!(
            zh_no_sessions.contains(MOCK_ZH_NO_SESSIONS_MARKER),
            "错误: 中文无会话模板应包含标记 '{}'",
            MOCK_ZH_NO_SESSIONS_MARKER
        );

        let en_no_sessions = manager.get_no_sessions_template("en");
        assert!(
            en_no_sessions.contains(MOCK_EN_NO_SESSIONS_MARKER),
            "错误: 英文无会话模板应包含标记 '{}'",
            MOCK_EN_NO_SESSIONS_MARKER
        );

        println!("✅ 回退模板多语言测试通过！");
    }

    /// 综合测试：完整的 Prompt 生成流程（包含变量替换）
    #[test]
    fn test_full_prompt_generation_with_language() {
        let _lock = DB_TEST_LOCK.read().unwrap();

        // 1. 创建 Mock 配置
        let config = create_mock_optimizer_config();
        let generator = create_mock_prompt_generator(config);

        // 2. 准备测试数据
        let goal = "实现二叉树遍历";
        let sessions = r#"[
            {"text":"什么是二叉树？","role":"user","timestamp":"2025-01-22T10:00:00Z"},
            {"text":"二叉树是一种数据结构...","role":"assistant","timestamp":"2025-01-22T10:00:01Z"}
        ]"#;

        // 3. 生成中文 Prompt
        let zh_result = generator.test_build_prompt(goal, sessions, "zh");
        assert!(
            zh_result.contains(MOCK_ZH_MARKER),
            "中文 Prompt 应包含中文标记"
        );
        assert!(
            zh_result.contains(goal),
            "中文 Prompt 应包含原始目标"
        );
        assert!(
            zh_result.contains("实现二叉树遍历"),
            "中文 Prompt 应包含目标内容"
        );
        assert!(
            !zh_result.contains(MOCK_EN_MARKER),
            "中文 Prompt 不应包含英文标记"
        );

        // 4. 生成英文 Prompt
        let en_goal = "Implement binary tree traversal";
        let en_result = generator.test_build_prompt(en_goal, sessions, "en");
        assert!(
            en_result.contains(MOCK_EN_MARKER),
            "英文 Prompt 应包含英文标记"
        );
        assert!(
            en_result.contains(en_goal),
            "英文 Prompt 应包含原始目标"
        );
        assert!(
            en_result.contains("Implement binary tree traversal"),
            "英文 Prompt 应包含目标内容"
        );
        assert!(
            !en_result.contains(MOCK_ZH_MARKER),
            "英文 Prompt 不应包含中文标记"
        );

        println!("✅ 完整 Prompt 生成测试通过！");
    }

    // ==================== Mock 测试辅助工具 ====================

    /// Mock 配置标记常量
    ///
    /// 这些标记用于区分中英文模板，验证路由逻辑是否正确
    const MOCK_ZH_MARKER: &str = "[[[ZH_LANGUAGE_MARKER]]]";
    const MOCK_EN_MARKER: &str = "[[[EN_LANGUAGE_MARKER]]]";
    const MOCK_ZH_STARTER_MARKER: &str = "ZH_STARTER";
    const MOCK_EN_STARTER_MARKER: &str = "EN_STARTER";
    const MOCK_ZH_NO_SESSIONS_MARKER: &str = "ZH_NO_SESSIONS";
    const MOCK_EN_NO_SESSIONS_MARKER: &str = "EN_NO_SESSIONS";

    /// 创建 Mock 的 OptimizerConfig
    ///
    /// 使用特征明显的标记字符串，便于断言验证
    fn create_mock_optimizer_config() -> OptimizerConfig {
        OptimizerConfig {
            meta_prompt: crate::optimizer::config::MetaPromptConfig {
                template_zh: format!("{}\n# 中文 Meta-Prompt\n你是一位专业的提示词工程师。", MOCK_ZH_MARKER),
                template_en: format!("{}\n# English Meta-Prompt\nYou are a professional prompt engineer.", MOCK_EN_MARKER),
            },
            llm_params: crate::optimizer::config::LLMParamsConfig {
                temperature: 0.1,
                max_tokens: 1500,
                top_p: 0.9,
                frequency_penalty: 0.0,
                presence_penalty: 0.0,
            },
            prompt_structure: crate::optimizer::config::PromptStructureConfig {
                structure_zh: format!(
                    "{{{{meta_prompt}}}}\n## 用户目标\n{{{{goal}}}}\n## 历史会话\n{{{{sessions}}}}\n{}",
                    MOCK_ZH_MARKER
                ),
                structure_en: format!(
                    "{{{{meta_prompt}}}}\n## User Goal\n{{{{goal}}}}\n## Conversation History\n{{{{sessions}}}}\n{}",
                    MOCK_EN_MARKER
                ),
            },
            fallback: crate::optimizer::config::FallbackConfig {
                no_sessions_template_zh: format!("{}\n请帮我完成：{{{{goal}}}}", MOCK_ZH_NO_SESSIONS_MARKER),
                no_sessions_template_en: format!("{}\nPlease help me complete: {{{{goal}}}}", MOCK_EN_NO_SESSIONS_MARKER),
                llm_error_template_zh: format!("{}\nLLM 调用失败：{{{{goal}}}}", MOCK_ZH_MARKER),
                llm_error_template_en: format!("{}\nLLM call failed: {{{{goal}}}}", MOCK_EN_MARKER),
                conversation_starter_template_zh: format!("{}\n## 用户目标\n{{{{goal}}}}", MOCK_ZH_STARTER_MARKER),
                conversation_starter_template_en: format!("{}\n## User Goal\n{{{{goal}}}}", MOCK_EN_STARTER_MARKER),
            },
            session_context: crate::optimizer::config::SessionContextConfig {
                max_summary_length: 200,
                include_rating: true,
                include_project: true,
                session_format_zh: "- 会话 {{{{session_id}}}} {{#if project_name}}(项目: {{{{project_name}}}}){{/if}}".to_string(),
                session_format_en: "- Session {{{{session_id}}}} {{#if project_name}}(Project: {{{{project_name}}}}){{/if}}".to_string(),
            },
            compression: crate::optimizer::config::CompressionConfig {
                level: "basic".to_string(),
                preserve_formatting: true,
                min_compression_ratio: 0.0,
            },
            advanced: crate::optimizer::config::AdvancedConfig {
                parallel_processing: 5,
                cache_strategy: "memory".to_string(),
                debug: false,
                timeout: 30,
            },
        }
    }

    /// Mock 配置管理器
    ///
    /// 简化版的 ConfigManager，用于测试环境
    struct MockConfigManager {
        config: OptimizerConfig,
    }

    impl MockConfigManager {
        fn new(config: OptimizerConfig) -> Self {
            Self { config }
        }

        fn get_meta_prompt(&self, language: &str) -> String {
            match language {
                "zh" => self.config.meta_prompt.template_zh.clone(),
                _ => self.config.meta_prompt.template_en.clone(),  // 默认英文
            }
        }

        fn get_prompt_structure(&self, language: &str) -> String {
            match language {
                "zh" => self.config.prompt_structure.structure_zh.clone(),
                _ => self.config.prompt_structure.structure_en.clone(),  // 默认英文
            }
        }

        fn get_no_sessions_template(&self, language: &str) -> String {
            match language {
                "zh" => self.config.fallback.no_sessions_template_zh.clone(),
                _ => self.config.fallback.no_sessions_template_en.clone(),  // 默认英文
            }
        }

        fn get_llm_error_template(&self, language: &str) -> String {
            match language {
                "zh" => self.config.fallback.llm_error_template_zh.clone(),
                _ => self.config.fallback.llm_error_template_en.clone(),  // 默认英文
            }
        }

        fn get_conversation_starter_template(&self, language: &str) -> String {
            match language {
                "zh" => self.config.fallback.conversation_starter_template_zh.clone(),
                _ => self.config.fallback.conversation_starter_template_en.clone(),  // 默认英文
            }
        }

        fn get_session_format(&self, language: &str) -> String {
            match language {
                "zh" => self.config.session_context.session_format_zh.clone(),
                _ => self.config.session_context.session_format_en.clone(),  // 默认英文
            }
        }
    }

    /// 创建 Mock PromptGenerator
    ///
    /// 使用临时配置文件创建测试用的 PromptGenerator
    /// 每个测试使用唯一的文件名，避免并发测试冲突
    fn create_mock_prompt_generator(config: OptimizerConfig) -> PromptGenerator {
        // 使用时间戳和随机数创建唯一的临时文件名
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_path = std::env::temp_dir().join(format!("test_optimizer_config_{}.toml", timestamp));
        let toml_str = toml::to_string_pretty(&config).unwrap();
        std::fs::write(&temp_path, toml_str).unwrap();

        PromptGenerator::with_config_path(temp_path).unwrap()
    }
}
