//! 提示词生成器模块
//!
//! 整合上下文压缩和 LLM 生成，创建增强的提示词优化功能
//! 注意：向量检索功能已暂时移除，改用简单的最近会话检索

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::llm::{LLMClientManager, interface::{Message, ModelParams}};
use crate::database::repository::SessionRepository;
use crate::tokenizer::TokenCounter;
use super::compressor::ContextCompressor;
use super::config::ConfigManager;

// ==================== 数据结构 ====================

/// 增强提示词请求
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnhancedPromptRequest {
    /// 用户目标
    pub goal: String,
    /// 可选：会话文件路径列表（从项目目录获取）
    #[serde(rename = "sessionFilePaths")]
    pub session_file_paths: Option<Vec<String>>,
    /// 可选：手动指定会话 ID 列表（已弃用，保留兼容性）
    #[serde(rename = "sessionIds")]
    pub session_ids: Option<Vec<String>>,
    /// 检索限制
    pub limit: Option<usize>,
    /// 是否使用加权检索
    #[serde(rename = "useWeighted")]
    pub use_weighted: Option<bool>,
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

        eprintln!("[PromptGenerator] 可执行文件路径: {:?}", exe_path);
        eprintln!("[PromptGenerator] 可执行文件目录: {:?}", exe_dir);

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
            eprintln!("[PromptGenerator] 使用生产环境配置路径: {:?}", prod_path);
            return Ok(prod_path);
        }

        // 策略 3: 回退 - 尝试使用当前工作目录 + src-tauri（仅限开发环境）
        let cwd_path = env::current_dir()
            .map(|d| d.join("src-tauri").join("optimizer_config.toml"))
            .ok();

        if let Some(ref path) = cwd_path {
            if path.exists() {
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
    /// # 简化流程（不使用向量检索）
    /// 1. 获取最近的会话（或手动指定的会话）
    /// 2. 压缩上下文
    /// 3. 构建 Meta-Prompt
    /// 4. 调用 LLM 生成
    pub async fn generate_enhanced_prompt(
        &self,
        request: EnhancedPromptRequest,
        llm_manager: &LLMClientManager,
    ) -> Result<EnhancedPrompt> {
        // 1. 检索相关会话
        // 优先使用 session_file_paths，否则使用 session_ids（兼容旧版）
        let limit = request.limit.unwrap_or(5);

        // 合并两种会话来源
        let combined_ids: Option<Vec<String>> = request.session_file_paths.as_ref()
            .or(request.session_ids.as_ref())
            .cloned();

        let sessions = self.get_recent_sessions(limit, &combined_ids)?;

        if sessions.is_empty() {
            // 没有相关会话，返回基础提示词
            return Ok(self.create_fallback_prompt(&request.goal));
        }

        // 2. 提取会话上下文并压缩
        let (original_tokens, compressed_context) = self.compress_sessions_context(&sessions)?;

        // 3. 获取 Meta-Prompt（从配置）
        let meta_prompt = self.config_manager.get_meta_prompt();

        // 4. 构建完整提示词（使用配置的结构模板）
        let full_prompt = self.build_prompt_with_meta(
            &request.goal,
            &sessions,
            &compressed_context,
            &meta_prompt,
        );

        // 5. 调用 LLM 生成增强提示词
        let enhanced_prompt = match self.call_llm_generate(&full_prompt, llm_manager).await {
            Ok(prompt) => {
                eprintln!("[PromptGenerator] LLM 生成成功，长度: {}", prompt.len());
                prompt
            },
            Err(e) => {
                // LLM 调用失败时，回退到模板生成
                eprintln!("[PromptGenerator] LLM 调用失败，使用模板: {}", e);
                self.generate_template_prompt(&request.goal, &sessions)
            }
        };

        // 6. 计算 Token 统计
        let compressed_tokens = self.token_counter.count_tokens(&enhanced_prompt)?;
        let savings_percentage = if original_tokens > 0 && compressed_tokens <= original_tokens {
            ((original_tokens - compressed_tokens) as f64 / original_tokens as f64) * 100.0
        } else if original_tokens > 0 {
            // 压缩后更多，表示负节省
            -(((compressed_tokens - original_tokens) as f64 / original_tokens as f64) * 100.0)
        } else {
            0.0
        };

        // 7. 计算置信度（基于评分和更新时间）
        let confidence = self.calculate_confidence(&sessions);

        // 8. 构建引用会话信息
        let referenced_sessions: Vec<ReferencedSession> = sessions
            .into_iter()
            .map(|s| ReferencedSession {
                session_id: s.session_id.clone(),
                project_name: s.project_name.clone(),
                summary: s.summary.clone().unwrap_or_else(|| "无摘要".to_string()),
                similarity_score: s.relevance_score,
            })
            .collect();

        Ok(EnhancedPrompt {
            original_goal: request.goal,
            referenced_sessions,
            enhanced_prompt,
            token_stats: TokenStats {
                original_tokens,
                compressed_tokens,
                savings_percentage,
            },
            confidence,
        })
    }

    /// 获取最近的相关会话
    fn get_recent_sessions(
        &self,
        limit: usize,
        session_ids: &Option<Vec<String>>,
    ) -> Result<Vec<SessionWithScore>> {
        // 首先检查是否有会话文件路径（优先使用）
        if let Some(ref file_paths) = session_ids {
            // 这里实际是 session_file_paths，从文件路径读取会话
            let mut results = Vec::new();
            for file_path in file_paths {
                // 从文件路径提取 session_id
                let session_id = std::path::Path::new(file_path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");

                // 提取项目名称
                let project_name = std::path::Path::new(file_path)
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                results.push(SessionWithScore {
                    session_id: session_id.to_string(),
                    project_name,
                    summary: None,
                    rating: None, // 从文件无法直接获取评分
                    relevance_score: 0.5, // 默认相关性
                });
            }
            eprintln!("[PromptGenerator] 从文件加载会话: {} 个", results.len());
            return Ok(results);
        }

        // 回退到数据库查询（旧逻辑，保留兼容性）
        let all_sessions = self.repository.get_all_sessions()
            .unwrap_or_default();

        eprintln!("[PromptGenerator] 数据库中共有 {} 个会话", all_sessions.len());

        let mut scored_sessions: Vec<SessionWithScore> = all_sessions
            .into_iter()
            .filter(|s| !s.is_archived)
            .map(|s| {
                let rating_score = s.rating.unwrap_or(0) as f64 / 5.0;
                let relevance_score = rating_score * 0.7 + 0.3;
                SessionWithScore {
                    session_id: s.session_id,
                    project_name: s.project_name,
                    summary: None,
                    rating: s.rating,
                    relevance_score,
                }
            })
            .collect();

        scored_sessions.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        scored_sessions.truncate(limit);

        eprintln!("[PromptGenerator] 返回 {} 个相关会话", scored_sessions.len());
        Ok(scored_sessions)
    }

    /// 压缩会话上下文
    fn compress_sessions_context(
        &self,
        sessions: &[SessionWithScore],
    ) -> Result<(usize, String)> {
        // 从配置获取最大摘要长度
        let max_summary_length = self.config_manager.get_max_summary_length();

        // 构建原始上下文
        let original_context = sessions.iter()
            .map(|s| {
                format!(
                    "会话: {}\n项目: {}\n摘要: {}\n",
                    s.session_id,
                    s.project_name,
                    s.summary.as_ref().unwrap_or(&"无摘要".to_string())
                )
            })
            .collect::<Vec<_>>()
            .join("\n---\n");

        // 计算原始 Token 数
        let original_tokens = self.token_counter.count_tokens(&original_context)?;

        // 简化压缩：保留结构但去除冗余，使用配置的长度限制
        let compressed = sessions.iter()
            .map(|s| {
                format!(
                    "会话 {} (评分: {}): {}",
                    s.session_id,
                    s.rating.unwrap_or(0),
                    s.summary.as_ref()
                        .unwrap_or(&"无摘要".to_string())
                        .chars()
                        .take(max_summary_length)
                        .collect::<String>()
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok((original_tokens, compressed))
    }

    /// 使用 Meta-Prompt 构建完整提示词
    fn build_prompt_with_meta(
        &self,
        goal: &str,
        sessions: &[SessionWithScore],
        compressed_context: &str,
        meta_prompt: &str,
    ) -> String {
        // 从配置获取结构模板
        let structure = self.config_manager.get_prompt_structure();

        // 格式化会话信息
        let max_summary_length = self.config_manager.get_max_summary_length();
        let include_rating = self.config_manager.include_rating();
        let include_project = self.config_manager.include_project();
        let session_format = self.config_manager.get_session_format();

        let sessions_info = sessions.iter()
            .map(|s| {
                let summary = s.summary.as_ref()
                    .unwrap_or(&"无摘要".to_string())
                    .chars()
                    .take(max_summary_length)
                    .collect::<String>();

                // 简单的模板变量替换
                let mut formatted = session_format.clone();
                formatted = formatted.replace("{{session_id}}", &s.session_id);
                formatted = formatted.replace("{{project_name}}", &s.project_name);
                formatted = formatted.replace("{{summary}}", &summary);

                if include_rating {
                    let rating_str = format!("{}", s.rating.unwrap_or(0));
                    formatted = formatted.replace("{{rating}}", &rating_str);
                }

                formatted
            })
            .collect::<Vec<_>>()
            .join("\n");

        // 使用配置的结构模板
        structure
            .replace("{{meta_prompt}}", meta_prompt)
            .replace("{{goal}}", goal)
            .replace("{{sessions}}", &sessions_info)
            .replace("{{context}}", compressed_context)
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

    /// 生成模板提示词（LLM 调用失败时回退）
    fn generate_template_prompt(&self, goal: &str, sessions: &[SessionWithScore]) -> String {
        let best_session = sessions.first();
        let reference = if let Some(ref s) = best_session {
            let summary_str = s.summary.as_ref().map(|s| s.as_str()).unwrap_or("无摘要");
            format!(
                "\n## 参考\n相关会话: {} (评分: {})\n{}",
                s.session_id,
                s.rating.unwrap_or(0),
                summary_str.chars().take(300).collect::<String>()
            )
        } else {
            String::new()
        };

        format!(
            "请帮我完成以下编程任务：\n\n{}\n\n{}\n\n请提供详细的实现方案和代码示例。",
            goal, reference
        )
    }

    /// 创建回退提示词（无相关会话时）
    fn create_fallback_prompt(&self, goal: &str) -> EnhancedPrompt {
        // 从配置获取无会话回退模板
        let template = self.config_manager.get_no_sessions_template();

        let enhanced_prompt = template.replace("{{goal}}", goal);

        EnhancedPrompt {
            original_goal: goal.to_string(),
            referenced_sessions: Vec::new(),
            enhanced_prompt,
            token_stats: TokenStats {
                original_tokens: 0,
                compressed_tokens: goal.len(),
                savings_percentage: 0.0,
            },
            confidence: 0.3, // 低置信度
        }
    }

    /// 计算置信度
    fn calculate_confidence(&self, sessions: &[SessionWithScore]) -> f64 {
        if sessions.is_empty() {
            return 0.0;
        }

        // 基于评分计算
        let avg_rating: f64 = sessions.iter()
            .filter_map(|s| s.rating)
            .map(|r| r as f64).sum::<f64>() / sessions.len() as f64;

        // 转换为 0-1 范围
        (avg_rating / 5.0).min(1.0).max(0.0)
    }
}

// ========== 辅助结构体 ==========

/// 带相关性分数的会话
struct SessionWithScore {
    /// 会话 ID
    session_id: String,
    /// 项目名称
    project_name: String,
    /// 摘要（Session 没有摘要字段，总是 None）
    summary: Option<String>,
    /// 评分
    rating: Option<i32>,
    /// 相关性分数（0-1）
    relevance_score: f64,
}
