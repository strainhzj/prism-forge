//! 提示词生成器模块
//!
//! 整合向量检索、上下文压缩和 LLM 生成，创建增强的提示词优化功能

use anyhow::{Context, Result};
use crate::embedding::EmbeddingGenerator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::llm::{LLMClientManager, interface::{Message, MessageRole, ModelParams}};
use crate::database::{repository::SessionRepository, models::{Session, VectorSearchResult}};
use crate::tokenizer::TokenCounter;
use super::compressor::{ContextCompressor, CompressionResult};

// ==================== 数据结构 ====================

/// 增强提示词请求
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnhancedPromptRequest {
    /// 用户目标
    pub goal: String,
    /// 可选：手动指定会话 ID 列表
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

/// 引用的会话信息
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
    /// 相似度分数
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
/// 整合 RAG 检索、上下文压缩和 LLM 生成
pub struct PromptGenerator {
    /// 数据库仓库
    repository: SessionRepository,
    /// 嵌入生成器
    embedding_generator: EmbeddingGenerator,
    /// 上下文压缩器
    compressor: ContextCompressor,
    /// Token 计数器
    token_counter: TokenCounter,
}

impl PromptGenerator {
    /// 创建新的提示词生成器
    pub fn new() -> Result<Self> {
        Ok(Self {
            repository: SessionRepository::from_default_db()?,
            embedding_generator: EmbeddingGenerator::new()?,
            compressor: ContextCompressor::new()?,
            token_counter: TokenCounter::new()?,
        })
    }

    /// 生成增强提示词（主流程）
    ///
    /// # 流程
    /// 1. 生成查询向量
    /// 2. 检索相关会话
    /// 3. 压缩上下文
    /// 4. 构建 Meta-Prompt
    /// 5. 调用 LLM 生成
    pub async fn generate_enhanced_prompt(
        &self,
        request: EnhancedPromptRequest,
        llm_manager: &LLMClientManager,
    ) -> Result<EnhancedPrompt> {
        // 1. 生成查询向量
        let query_embedding = self.embedding_generator
            .generate_for_message(&request.goal)
            .context("生成查询向量失败")?;

        // 2. 检索相关会话
        let limit = request.limit.unwrap_or(5);
        let use_weighted = request.use_weighted.unwrap_or(true);

        let search_results = if let Some(ref session_ids) = request.session_ids {
            // 手动指定会话 ID
            self.load_sessions_by_ids(session_ids)?
        } else {
            // 向量检索
            self.search_relevant_sessions(&query_embedding, limit, use_weighted)?
        };

        if search_results.is_empty() {
            // 没有相关会话，返回基础提示词
            return Ok(self.create_fallback_prompt(&request.goal));
        }

        // 3. 提取会话上下文并压缩
        let (original_tokens, compressed_context) = self.compress_sessions_context(&search_results)?;

        // 4. 获取 Meta-Prompt
        let meta_prompt = self.get_meta_prompt("optimizer")?;

        // 5. 构建完整提示词
        let full_prompt = self.build_prompt_with_meta(
            &request.goal,
            &search_results,
            &compressed_context,
            &meta_prompt,
        );

        // 6. 调用 LLM 生成增强提示词
        let enhanced_prompt = self.call_llm_generate(&full_prompt, llm_manager).await
            .unwrap_or_else(|e| {
                // LLM 调用失败时，回退到模板生成
                eprintln!("[PromptGenerator] LLM 调用失败，使用模板: {}", e);
                self.generate_template_prompt(&request.goal, &search_results)
            });

        // 7. 计算 Token 统计
        let compressed_tokens = self.token_counter.count_tokens(&enhanced_prompt)?;
        let savings_percentage = if original_tokens > 0 {
            ((original_tokens - compressed_tokens) as f64 / original_tokens as f64) * 100.0
        } else {
            0.0
        };

        // 8. 计算置信度（基于检索结果的相似度）
        let confidence = self.calculate_confidence(&search_results);

        // 9. 构建引用会话信息
        let referenced_sessions: Vec<ReferencedSession> = search_results
            .into_iter()
            .map(|r| ReferencedSession {
                session_id: r.session.session_id,
                project_name: r.session.project_name,
                summary: r.summary,
                similarity_score: r.similarity_score,
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

    /// 检索相关会话
    fn search_relevant_sessions(
        &self,
        query_embedding: &[f32],
        limit: usize,
        use_weighted: bool,
    ) -> Result<Vec<VectorSearchResult>> {
        let results = if use_weighted {
            self.repository.weighted_vector_search_sessions(query_embedding, limit)?
        } else {
            self.repository.vector_search_sessions(query_embedding, limit)?
        };

        Ok(results)
    }

    /// 按 ID 加载会话
    fn load_sessions_by_ids(&self, session_ids: &[String]) -> Result<Vec<VectorSearchResult>> {
        let mut results = Vec::new();

        for session_id in session_ids {
            if let Ok(Some(session)) = self.repository.get_session_by_id(session_id) {
                results.push(VectorSearchResult {
                    session,
                    similarity_score: 1.0, // 手动指定的会话给予最高相似度
                    summary: "手动指定".to_string(),
                });
            }
        }

        Ok(results)
    }

    /// 压缩会话上下文
    fn compress_sessions_context(
        &self,
        sessions: &[VectorSearchResult],
    ) -> Result<(usize, String)> {
        // 构建原始上下文
        let original_context = sessions.iter()
            .map(|s| {
                format!(
                    "会话: {}\n项目: {}\n摘要: {}\n",
                    s.session.session_id,
                    s.session.project_name,
                    s.summary
                )
            })
            .collect::<Vec<_>>()
            .join("\n---\n");

        // 计算原始 Token 数
        let original_tokens = self.token_counter.count_tokens(&original_context)?;

        // 简化压缩：保留结构但去除冗余
        let compressed = sessions.iter()
            .map(|s| {
                format!(
                    "会话 {} (相似度: {:.2}): {}",
                    s.session.session_id,
                    s.similarity_score,
                    s.summary.chars().take(200).collect::<String>()
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok((original_tokens, compressed))
    }

    /// 获取 Meta-Prompt
    fn get_meta_prompt(&self, category: &str) -> Result<String> {
        // 从数据库获取 Meta-Prompt
        if let Ok(template) = self.repository.get_meta_template(category) {
            return Ok(template);
        }

        // 回退到默认模板
        Ok(self.get_default_meta_prompt())
    }

    /// 获取默认 Meta-Prompt
    fn get_default_meta_prompt(&self) -> String {
        r#"
你是一个专业的编程助手提示词优化器。基于以下信息，生成一个清晰、具体的提示词，帮助用户高效完成编程任务。

## 你的任务
1. 分析用户的编程目标
2. 参考相关历史会话中的解决方案
3. 生成一个可直接使用的、结构化的提示词

## 输出格式
请生成一个包含以下部分的提示词：
1. **任务背景**: 简要描述要完成的任务
2. **技术要求**: 涉及的技术栈和约束条件
3. **参考方案**: 基于历史经验的建议
4. **具体步骤**: 明确的实施步骤
5. **预期输出**: 期望的结果格式

## 限制条件
- 提示词要简洁明了（控制在 500 字以内）
- 优先参考高评分、高相似度的历史方案
- 突出关键技术点和注意事项
"#.trim().to_string()
    }

    /// 使用 Meta-Prompt 构建完整提示词
    fn build_prompt_with_meta(
        &self,
        goal: &str,
        sessions: &[VectorSearchResult],
        compressed_context: &str,
        meta_prompt: &str,
    ) -> String {
        let sessions_info = sessions.iter()
            .map(|s| format!(
                "- 会话 {} (相似度: {:.2}, 评分: {}):\n  {}",
                s.session.session_id,
                s.similarity_score,
                s.session.rating.unwrap_or(0),
                s.summary.chars().take(100).collect::<String>()
            ))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "{}\n\n## 用户目标\n{}\n\n## 相关历史会话\n{}\n\n## 上下文摘要\n{}\n\n## 请求\n基于上述信息，生成一个优化的提示词。",
            meta_prompt, goal, sessions_info, compressed_context
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

        // 创建优化的参数（温度较低以获得更确定的结果）
        let params = ModelParams::new(model)
            .with_temperature(0.3)
            .with_max_tokens(1500);

        let client = llm_manager.get_active_client()
            .context("无法获取 LLM 客户端")?;

        let messages = vec![Message::user(prompt)];

        let response = client.chat_completion(messages, params).await?;

        Ok(response.content)
    }

    /// 生成模板提示词（LLM 调用失败时回退）
    fn generate_template_prompt(&self, goal: &str, sessions: &[VectorSearchResult]) -> String {
        let best_session = sessions.first();
        let reference = if let Some(ref s) = best_session {
            format!(
                "\n## 参考\n相关会话: {} (相似度: {:.2})\n{}",
                s.session.session_id,
                s.similarity_score,
                s.summary.chars().take(300).collect::<String>()
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
        EnhancedPrompt {
            original_goal: goal.to_string(),
            referenced_sessions: Vec::new(),
            enhanced_prompt: format!(
                "请帮我完成以下编程任务：\n\n{}\n\n请提供详细的实现方案。",
                goal
            ),
            token_stats: TokenStats {
                original_tokens: 0,
                compressed_tokens: goal.len(),
                savings_percentage: 0.0,
            },
            confidence: 0.3, // 低置信度
        }
    }

    /// 计算置信度
    fn calculate_confidence(&self, sessions: &[VectorSearchResult]) -> f64 {
        if sessions.is_empty() {
            return 0.0;
        }

        // 基于相似度和评分计算
        let avg_similarity: f64 = sessions.iter()
            .map(|s| s.similarity_score)
            .sum::<f64>() / sessions.len() as f64;

        let avg_rating: f64 = sessions.iter()
            .filter_map(|s| s.session.rating)
            .map(|r| r as f64).sum::<f64>() / sessions.len() as f64;

        // 综合相似度和评分
        (avg_similarity * 0.7 + (avg_rating / 5.0) * 0.3).min(1.0).max(0.0)
    }
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_confidence() {
        let generator = PromptGenerator::new().unwrap();

        // 测试空会话
        let confidence = generator.calculate_confidence(&[]);
        assert_eq!(confidence, 0.0);

        // 测试高相似度会话
        // 这里需要构造测试数据
    }
}
