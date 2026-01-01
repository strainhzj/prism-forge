//! LLM 客户端管理器
//!
//! 统一管理多个 LLM 提供商，负责：
//! - 从数据库读取活跃提供商
//! - 从 keyring 读取 API Key
//! - 实例化对应的 Provider 客户端

use anyhow::{Context, Result};
use std::sync::{Arc, Mutex};

use crate::database::{ApiProvider, ApiProviderType, ApiProviderRepository};
use crate::llm::security::ApiKeyStorage;
use crate::llm::interface::{LLMService, TestConnectionResult};
use crate::llm::providers::{OpenAIProvider, AnthropicProvider, OllamaProvider, XAIProvider, GoogleProvider, GoogleVertexProvider};

/// LLM 客户端管理器
pub struct LLMClientManager {
    /// 数据库仓库（使用 Arc<Mutex<>> 确保线程安全）
    repository: Arc<Mutex<ApiProviderRepository>>,
}

// 确保 LLMClientManager 满足 Send + Sync 要求（因为有 Arc<Mutex<>> 内部可变性）
unsafe impl Send for LLMClientManager {}
unsafe impl Sync for LLMClientManager {}

impl LLMClientManager {
    /// 创建新的管理器
    ///
    /// # 参数
    /// - `repository`: 数据库仓库
    pub fn new(repository: ApiProviderRepository) -> Self {
        Self {
            repository: Arc::new(Mutex::new(repository)),
        }
    }

    /// 从默认数据库创建管理器
    pub fn from_default_db() -> Result<Self> {
        let conn = crate::database::init::get_connection_shared()?;
        let repository = ApiProviderRepository::with_conn(conn);
        Ok(Self::new(repository))
    }

    /// 获取当前活跃的 LLM 客户端
    ///
    /// # 流程
    /// 1. 从数据库读取活跃的提供商配置
    /// 2. 从 keyring 读取 API Key
    /// 3. 实例化对应的客户端
    pub fn get_active_client(&self) -> Result<Box<dyn LLMService>> {
        // 从数据库获取活跃提供商
        let provider = {
            let repo = self.repository.lock().unwrap();
            repo.get_active_provider()?
        }.context("未设置活跃的 API 提供商，请先在设置中配置")?;

        self.create_client_from_provider(&provider)
    }

    /// 从提供商配置创建客户端
    fn create_client_from_provider(
        &self,
        provider: &ApiProvider,
    ) -> Result<Box<dyn LLMService>> {
        match provider.provider_type {
            ApiProviderType::OpenAI => {
                // 从 keyring 获取 API Key
                let api_key_ref = provider
                    .api_key_ref
                    .as_ref()
                    .context("OpenAI 提供商未配置 API Key")?;

                let api_key = ApiKeyStorage::get_api_key(provider.id.unwrap_or(0))
                    .with_context(|| format!("无法获取 OpenAI API Key (provider_id={})", provider.id.unwrap_or(0)))?;

                let client = OpenAIProvider::with_ref(
                    api_key,
                    provider.base_url.clone(),
                    api_key_ref.clone(),
                );

                Ok(Box::new(client))
            }
            ApiProviderType::Anthropic => {
                // 从 keyring 获取 API Key
                let api_key_ref = provider
                    .api_key_ref
                    .as_ref()
                    .context("Anthropic 提供商未配置 API Key")?;

                let api_key = ApiKeyStorage::get_api_key(provider.id.unwrap_or(0))
                    .with_context(|| format!("无法获取 Anthropic API Key (provider_id={})", provider.id.unwrap_or(0)))?;

                let client = AnthropicProvider::with_ref(
                    api_key,
                    provider.base_url.clone(),
                    api_key_ref.clone(),
                );

                Ok(Box::new(client))
            }
            ApiProviderType::Ollama => {
                // Ollama 不需要 API Key
                let client = OllamaProvider::new(Some(provider.base_url.clone()));
                Ok(Box::new(client))
            }
            ApiProviderType::XAI => {
                // 从 keyring 获取 API Key
                let api_key_ref = provider
                    .api_key_ref
                    .as_ref()
                    .context("X AI 提供商未配置 API Key")?;

                let api_key = ApiKeyStorage::get_api_key(provider.id.unwrap_or(0))
                    .with_context(|| format!("无法获取 X AI API Key (provider_id={})", provider.id.unwrap_or(0)))?;

                let client = XAIProvider::with_ref(
                    api_key,
                    provider.base_url.clone(),
                    api_key_ref.clone(),
                );

                Ok(Box::new(client))
            }
            ApiProviderType::Google => {
                // 先检查配置，判断使用哪种模式
                let config = provider.get_config()?;
                let use_vertexai = config
                    .as_ref()
                    .and_then(|c| c.get("use_vertexai"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                if use_vertexai {
                    // Vertex AI 模式：需要 project 和 access_token
                    let project = config
                        .as_ref()
                        .and_then(|c| c.get("project"))
                        .and_then(|p| p.as_str())
                        .context("Vertex AI 模式需要配置 project")?
                        .to_string();

                    let location = config
                        .as_ref()
                        .and_then(|c| c.get("location"))
                        .and_then(|l| l.as_str())
                        .unwrap_or("us-central1")
                        .to_string();

                    // 读取 access_token（可选，如果没有配置会返回错误）
                    let access_token = config
                        .as_ref()
                        .and_then(|c| c.get("access_token"))
                        .and_then(|t| t.as_str())
                        .map(|t| t.to_string());

                    let client = GoogleProvider::new_vertexai(
                        project,
                        location,
                        Some(provider.base_url.clone()),
                        access_token,
                    );

                    Ok(Box::new(client))
                } else {
                    // ML Dev API 模式：需要从 keyring 获取 API Key
                    let api_key_ref = provider
                        .api_key_ref
                        .as_ref()
                        .context("Google ML Dev API 提供商未配置 API Key")?;

                    let api_key = ApiKeyStorage::get_api_key(provider.id.unwrap_or(0))
                        .with_context(|| format!("无法获取 Google API Key (provider_id={})", provider.id.unwrap_or(0)))?;

                    let client = GoogleProvider::with_ref(
                        api_key,
                        provider.base_url.clone(),
                        api_key_ref.clone(),
                    );

                    Ok(Box::new(client))
                }
            }
            ApiProviderType::GoogleVertex => {
                // Google Vertex AI Public Preview：从 keyring 获取 API Key
                let api_key_ref = provider
                    .api_key_ref
                    .as_ref()
                    .context("Google Vertex AI 提供商未配置 API Key")?;

                let api_key = ApiKeyStorage::get_api_key(provider.id.unwrap_or(0))
                    .with_context(|| format!("无法获取 Google Vertex AI API Key (provider_id={})", provider.id.unwrap_or(0)))?;

                let client = GoogleVertexProvider::with_ref(
                    api_key,
                    provider.base_url.clone(),
                    api_key_ref.clone(),
                );

                Ok(Box::new(client))
            }
        }
    }

    /// 切换活跃提供商
    ///
    /// # 参数
    /// - `provider_id`: 要设置为活跃的提供商 ID
    pub fn switch_provider(&self, provider_id: i64) -> Result<()> {
        let repo = self.repository.lock().unwrap();
        repo.set_active_provider(provider_id)?;
        Ok(())
    }

    /// 获取所有提供商列表
    pub fn get_all_providers(&self) -> Result<Vec<ApiProvider>> {
        let repo = self.repository.lock().unwrap();
        repo.get_all_providers()
    }

    /// 测试提供商连接
    /// 
    /// 使用提供商配置的模型（或默认模型）进行连接测试
    pub async fn test_provider(&self, provider_id: i64) -> Result<TestConnectionResult> {
        #[cfg(debug_assertions)]
        eprintln!("[LLMClientManager] test_provider called for provider_id={}", provider_id);

        let provider = {
            let repo = self.repository.lock().unwrap();
            repo.get_provider_by_id(provider_id)?
        }.context("提供商不存在")?;

        #[cfg(debug_assertions)]
        eprintln!("[LLMClientManager] Found provider: {:?}", provider.name);

        let client = self.create_client_from_provider(&provider)
            .with_context(|| format!("创建客户端失败: provider_id={}", provider_id))?;

        // 获取有效模型（配置的模型或默认模型）
        let model = provider.effective_model();

        #[cfg(debug_assertions)]
        eprintln!("[LLMClientManager] Client created, testing connection with model '{}'...", model);

        let result = client.test_connection_with_model(model).await
            .with_context(|| format!("测试连接失败: provider={}", provider.name))?;

        #[cfg(debug_assertions)]
        eprintln!("[LLMClientManager] Connection test result: {:?}", result);

        Ok(result)
    }

    /// 获取当前活跃的提供商配置
    ///
    /// 用于需要访问提供商元数据（如模型配置、类型等）的场景
    pub fn get_active_provider_config(&self) -> Result<ApiProvider> {
        let repo = self.repository.lock().unwrap();
        repo.get_active_provider()?.context("未设置活跃的 API 提供商，请先在设置中配置")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_manager() {
        // 使用内存数据库测试
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();

        // 执行迁移
        crate::database::migrations::migrate_v1(&mut conn).unwrap();

        let conn = std::sync::Arc::new(std::sync::Mutex::new(conn));
        let repo = ApiProviderRepository::with_conn(conn);
        let _manager = LLMClientManager::new(repo);
    }
}
