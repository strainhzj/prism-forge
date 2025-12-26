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
use crate::llm::interface::LLMService;
use crate::llm::providers::{OpenAIProvider, AnthropicProvider, OllamaProvider};

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
        let repository = ApiProviderRepository::from_default_db()?;
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
    pub async fn test_provider(&self, provider_id: i64) -> Result<bool> {
        let provider = {
            let repo = self.repository.lock().unwrap();
            repo.get_provider_by_id(provider_id)?
        }.context("提供商不存在")?;

        let client = self.create_client_from_provider(&provider)?;
        client.test_connection().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_manager() {
        // 使用内存数据库测试
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();

        // 执行迁移
        crate::database::migrations::migrate_v1(&mut conn.clone()).unwrap();

        let repo = ApiProviderRepository::new(conn);
        let _manager = LLMClientManager::new(repo);
    }
}
