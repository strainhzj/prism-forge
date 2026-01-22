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
    pub async fn generate_enhanced_prompt(
        &self,
        request: EnhancedPromptRequest,
        llm_manager: &LLMClientManager,
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
                return self.generate_conversation_starter_with_llm(&request.goal, session_file_path, session_id, llm_manager).await;
            }

            // 5. 将问答对转换为对话流格式（时间正序）
            let (original_tokens, conversation_context) = self.format_qa_pairs_to_conversation(&qa_pairs)?;

            // 6. 构建完整提示词
            let full_prompt = self.build_prompt_with_conversation(
                &request.goal,
                &conversation_context,
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
                    self.generate_conversation_template_prompt(&request.goal)
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
            let referenced_sessions = vec![ReferencedSession {
                session_id: session_id.to_string(),
                project_name: "当前会话".to_string(),
                summary: format!("包含 {} 个问答对", qa_pairs.len()),
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
            Err(anyhow::anyhow!("请先在首页选择一个会话"))
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
    ) -> String {
        // 从配置获取模板
        let meta_prompt = self.config_manager.get_meta_prompt();
        let prompt_structure = self.config_manager.get_prompt_structure();

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
    ) -> Result<EnhancedPrompt> {
        // 1. 构建对话开始的完整提示词
        let full_prompt = self.build_conversation_starter_prompt(goal);

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
                self.generate_conversation_fallback_template(goal)
            }
        };

        // 3. 计算 Token 统计
        let compressed_tokens = self.token_counter.count_tokens(&enhanced_prompt)?;
        // 对话开始没有原始上下文，所以 original_tokens 设为 0
        let original_tokens = 0;
        let savings_percentage = 0.0;

        // 4. 构建引用会话信息
        let referenced_sessions = vec![ReferencedSession {
            session_id: session_id.to_string(),
            project_name: "当前会话".to_string(),
            summary: "对话开始（AI 生成）".to_string(),
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
    fn create_conversation_starter_prompt(&self, goal: &str, session_file_path: &str) -> EnhancedPrompt {
        // 从配置获取对话开始模板
        let template = self.config_manager.get_conversation_starter_template();

        let enhanced_prompt = template.replace("{{goal}}", goal);

        // 提取会话信息
        let path_buf = PathBuf::from(session_file_path);
        let session_id = path_buf
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        EnhancedPrompt {
            original_goal: goal.to_string(),
            referenced_sessions: vec![ReferencedSession {
                session_id: session_id.to_string(),
                project_name: "当前会话".to_string(),
                summary: "新对话，无历史记录".to_string(),
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
    fn generate_conversation_template_prompt(&self, goal: &str) -> String {
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

    /// 构建对话开始的完整提示词（使用配置的 conversation_starter_template）
    fn build_conversation_starter_prompt(&self, goal: &str) -> String {
        // 从配置获取对话开始模板
        let template = self.config_manager.get_conversation_starter_template();

        // 替换变量
        template.replace("{{goal}}", goal)
    }

    /// 生成对话回退模板（LLM 调用失败时使用）
    fn generate_conversation_fallback_template(&self, goal: &str) -> String {
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
    ) -> String {
        // 直接调用私有方法
        self.build_prompt_with_conversation(goal, sessions)
    }
}

// ==================== 测试模块 ====================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // 使用全局互斥锁确保测试顺序执行，避免数据库初始化冲突
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_build_prompt_from_config() {
        // 获取锁，确保测试顺序执行
        let _lock = TEST_LOCK.lock().unwrap();

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

        // 4. 调用测试辅助方法生成提示词
        let result = generator.test_build_prompt(goal, sessions);

        // 5. 打印生成的提示词（便于人工检查）
        println!("\n========== 生成的提示词 ==========\n");
        println!("{}", result);
        println!("\n========== 提示词结束 ==========\n");

        // 6. 验证结果包含预期的结构
        let assertions = vec![
            // Meta-Prompt 的内容
            ("Meta-Prompt 标题", "专业的编程助手提示词优化器"),
            ("分析方法步骤", "分析目标与上下文"),
            ("限制条件", "提示词应简洁明了"),

            // Prompt Structure 的内容
            ("输入信息标题", "## 输入信息"),
            ("用户目标标签", "**用户目标**"),
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
        // 获取锁，确保测试顺序执行
        let _lock = TEST_LOCK.lock().unwrap();

        // 测试配置文件是否能正确加载
        let config_path = std::path::PathBuf::from("optimizer_config.toml");

        if !config_path.exists() {
            println!("警告: 配置文件不存在，跳过测试");
            return;
        }

        let generator = PromptGenerator::with_config_path(config_path)
            .expect("创建 PromptGenerator 失败");

        // 验证配置管理器能正确读取配置
        let meta_prompt = generator.config_manager.get_meta_prompt();
        let prompt_structure = generator.config_manager.get_prompt_structure();

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
}
