//! 向量同步模块
//!
//! 负责将会话内容转换为向量并存储到数据库

use anyhow::Result;
use tokio::sync::RwLock;
use std::sync::Arc;
use std::time::Duration;

use crate::database::{
    models::{Session, SessionEmbedding, Settings},
    vector_repository::VectorRepository,
};
use crate::embedding::{EmbeddingProvider, OpenAIEmbeddings};

/// 向量同步管理器
pub struct EmbeddingSyncManager {
    /// 向量仓库
    repo: Arc<VectorRepository>,

    /// API Key（从 LLM provider 获取）
    api_key: Arc<RwLock<Option<String>>>,

    /// 当前配置
    config: Arc<RwLock<SyncConfig>>,

    /// 运行状态
    is_running: Arc<RwLock<bool>>,
}

/// 同步配置
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Embedding 提供商
    pub provider: EmbeddingProvider,

    /// Embedding 模型名称
    pub model: String,

    /// 批量处理大小
    pub batch_size: usize,

    /// 是否启用同步
    pub enabled: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            provider: EmbeddingProvider::OpenAI,
            model: "text-embedding-3-small".to_string(),
            batch_size: 10,
            enabled: false,
        }
    }
}

impl EmbeddingSyncManager {
    /// 创建新的同步管理器
    pub fn new(repo: Arc<VectorRepository>) -> Self {
        Self {
            repo,
            api_key: Arc::new(RwLock::new(None)),
            config: Arc::new(RwLock::new(SyncConfig::default())),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// 设置 API Key
    pub async fn set_api_key(&self, api_key: String) {
        let mut key = self.api_key.write().await;
        *key = Some(api_key);
    }

    /// 更新配置
    pub async fn update_config(&self, settings: &Settings) -> Result<()> {
        let provider = EmbeddingProvider::from_str(&settings.embedding_provider)
            .map_err(|e| anyhow::anyhow!("无效的 embedding provider: {}", e))?;

        let config = SyncConfig {
            provider,
            model: settings.embedding_model.clone(),
            batch_size: settings.embedding_batch_size as usize,
            enabled: settings.vector_search_enabled,
        };

        let mut cfg = self.config.write().await;
        *cfg = config;

        Ok(())
    }

    /// 同步所有未向量化的会话
    ///
    /// # 返回
    /// 返回成功向量化的数量
    pub async fn sync_all_sessions(&self) -> Result<usize> {
        // 检查是否启用
        let config = self.config.read().await;
        if !config.enabled {
            #[cfg(debug_assertions)]
            eprintln!("[EmbeddingSync] 向量同步未启用");

            return Ok(0);
        }

        // 检查 API Key
        let api_key = self.api_key.read().await;
        let api_key = api_key.as_ref().ok_or_else(|| {
            anyhow::anyhow!("API Key 未配置，无法进行向量同步")
        })?;
        let api_key = api_key.clone();
        drop(api_key);

        // 获取未向量化的会话
        let sessions = self.repo.get_non_vectorized_sessions(config.batch_size)?;

        if sessions.is_empty() {
            #[cfg(debug_assertions)]
            eprintln!("[EmbeddingSync] 没有需要向量化的会话");

            return Ok(0);
        }

        #[cfg(debug_assertions)]
        eprintln!("[EmbeddingSync] 开始向量化 {} 个会话", sessions.len());

        // 批量生成向量
        let summaries: Vec<String> = sessions
            .iter()
            .map(|s| self.extract_summary(s))
            .collect();

        let embeddings = self.generate_embeddings(&summaries).await?;

        // 保存向量
        let mut count = 0;
        for (session, embedding) in sessions.iter().zip(embeddings.iter()) {
            let session_embedding = SessionEmbedding::new(
                session.session_id.clone(),
                embedding.clone(),
                summaries.get(count).unwrap_or(&String::new()).clone(),
            );

            self.repo.upsert_session_embedding(session_embedding)?;
            count += 1;

            #[cfg(debug_assertions)]
            eprintln!("[EmbeddingSync] 已向量化会话: {}", session.session_id);
        }

        Ok(count)
    }

    /// 后台同步任务
    ///
    /// 定期检查并同步未向量化的会话
    pub async fn start_background_sync(&self) {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            #[cfg(debug_assertions)]
            eprintln!("[EmbeddingSync] 后台同步已在运行");

            return;
        }
        *is_running = true;
        drop(is_running);

        let is_running = self.is_running.clone();
        let repo = self.repo.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            #[cfg(debug_assertions)]
            eprintln!("[EmbeddingSync] 后台同步任务启动");

            let mut interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                // 检查是否应该停止
                {
                    let running = is_running.read().await;
                    if !*running {
                        #[cfg(debug_assertions)]
                        eprintln!("[EmbeddingSync] 后台同步任务停止");
                        break;
                    }
                }

                // 检查是否启用
                let cfg = config.read().await;
                if !cfg.enabled {
                    interval.tick().await;
                    continue;
                }

                // 获取未向量化的会话数量
                match repo.get_non_vectorized_count() {
                    Ok(count) => {
                        if count > 0 {
                            #[cfg(debug_assertions)]
                            eprintln!("[EmbeddingSync] 发现 {} 个未向量化的会话", count);

                            // 注意：这里不能直接调用 self.sync_all_sessions()，因为我们在 spawn 中
                            // 需要重新实现同步逻辑或重构代码
                        }
                    }
                    Err(e) => {
                        #[cfg(debug_assertions)]
                        eprintln!("[EmbeddingSync] 检查未向量化会话失败: {}", e);
                    }
                }

                interval.tick().await;
            }
        });
    }

    /// 停止后台同步
    pub async fn stop_background_sync(&self) {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
    }

    /// 提取会话摘要（用于向量化）
    ///
    /// 优先级：summary > displayName > session_id
    fn extract_summary(&self, session: &Session) -> String {
        // TODO: 从会话文件中读取 summary 或 displayName
        // 暂时使用 session_id
        session.session_id.clone()
    }

    /// 生成向量（根据配置选择提供商）
    async fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let config = self.config.read().await;
        let api_key = {
            let key = self.api_key.read().await;
            key.as_ref()
                .ok_or_else(|| anyhow::anyhow!("API Key 未配置"))?
                .clone()
        };

        match config.provider {
            EmbeddingProvider::OpenAI => {
                let client = OpenAIEmbeddings::new(&api_key, Some(config.model.clone()))?;
                client.generate_batch(texts).await
            }
            EmbeddingProvider::FastEmbed => {
                // TODO: 实现 FastEmbed
                Err(anyhow::anyhow!("FastEmbed 当前不可用（Windows 编译问题）"))
            }
        }
    }
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_config_default() {
        let config = SyncConfig::default();
        assert_eq!(config.batch_size, 10);
        assert!(!config.enabled);
    }
}
