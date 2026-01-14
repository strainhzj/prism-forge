//! LLM 客户端管理器
//!
//! 统一管理多个 LLM 提供商，负责：
//! - 从数据库读取活跃提供商
//! - 从 keyring 读取 API Key
//! - 实例化对应的 Provider 客户端

use anyhow::{Context, Result};
use std::sync::{Arc, Mutex};
use secrecy::ExposeSecret;

use crate::database::{ApiProvider, ApiProviderType, ApiProviderRepository};
use crate::llm::security::ApiKeyStorage;
use crate::llm::interface::{LLMService, TestConnectionResult};
use crate::llm::providers::{OpenAIProvider, AnthropicProvider, OllamaProvider, XAIProvider, GoogleProvider, GoogleVertexProvider};
use crate::llm::key_rotation::{ApiKeyRotator, KeyRotationConfig};
use crate::llm::model_resolver::ModelResolver;

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
            ApiProviderType::OpenAI | ApiProviderType::AzureOpenAI | ApiProviderType::OpenAICompatible => {
                // 从 keyring 获取 API Key
                let api_key_ref = provider
                    .api_key_ref
                    .as_ref()
                    .context(format!("{:?} 提供商未配置 API Key", provider.provider_type))?;

                let stored_key = ApiKeyStorage::get_api_key(provider.id.unwrap_or(0))
                    .with_context(|| format!("无法获取 {:?} API Key (provider_id={})", provider.provider_type, provider.id.unwrap_or(0)))?;

                // 暴露密钥进行轮换处理
                let key_str = stored_key.expose_secret();

                // 处理多密钥轮换
                let (selected_key, _new_config) = self.select_api_key_with_rotation(
                    key_str,
                    provider.config_json.as_deref(),
                )?;

                // 对于 Azure OpenAI，需要额外配置 api_version
                let base_url = if provider.provider_type == ApiProviderType::AzureOpenAI {
                    // 从 config_json 中读取 api_version，默认使用 "2024-02-01"
                    let config = provider.get_config().ok().flatten();
                    let _api_version = config
                        .as_ref()
                        .and_then(|c| c.get("api_version"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("2024-02-01");

                    // Azure OpenAI URL 格式: https://{resource}.openai.azure.com/openai/deployments/{deployment}?api-version={version}
                    // 这里我们保持 base_url 不变，由用户在界面中配置完整 URL
                    provider.base_url.clone()
                } else {
                    provider.base_url.clone()
                };

                let client = OpenAIProvider::with_ref(
                    secrecy::SecretString::new(selected_key.into()),
                    base_url,
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

                let stored_key = ApiKeyStorage::get_api_key(provider.id.unwrap_or(0))
                    .with_context(|| format!("无法获取 Anthropic API Key (provider_id={})", provider.id.unwrap_or(0)))?;

                let key_str = stored_key.expose_secret();

                // 处理多密钥轮换
                let (selected_key, _new_config) = self.select_api_key_with_rotation(
                    key_str,
                    provider.config_json.as_deref(),
                )?;

                let client = AnthropicProvider::with_ref(
                    secrecy::SecretString::new(selected_key.into()),
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

                let stored_key = ApiKeyStorage::get_api_key(provider.id.unwrap_or(0))
                    .with_context(|| format!("无法获取 X AI API Key (provider_id={})", provider.id.unwrap_or(0)))?;

                let key_str = stored_key.expose_secret();

                // 处理多密钥轮换
                let (selected_key, _new_config) = self.select_api_key_with_rotation(
                    key_str,
                    provider.config_json.as_deref(),
                )?;

                let client = XAIProvider::with_ref(
                    secrecy::SecretString::new(selected_key.into()),
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

                    let stored_key = ApiKeyStorage::get_api_key(provider.id.unwrap_or(0))
                        .with_context(|| format!("无法获取 Google API Key (provider_id={})", provider.id.unwrap_or(0)))?;

                    let key_str = stored_key.expose_secret();

                    // 处理多密钥轮换
                    let (selected_key, _new_config) = self.select_api_key_with_rotation(
                        key_str,
                        provider.config_json.as_deref(),
                    )?;

                    let client = GoogleProvider::with_ref(
                        secrecy::SecretString::new(selected_key.into()),
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

                let stored_key = ApiKeyStorage::get_api_key(provider.id.unwrap_or(0))
                    .with_context(|| format!("无法获取 Google Vertex AI API Key (provider_id={})", provider.id.unwrap_or(0)))?;

                let key_str = stored_key.expose_secret();

                // 处理多密钥轮换
                let (selected_key, _new_config) = self.select_api_key_with_rotation(
                    key_str,
                    provider.config_json.as_deref(),
                )?;

                let client = GoogleVertexProvider::with_ref(
                    secrecy::SecretString::new(selected_key.into()),
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

    /// 从密钥字符串中获取下一个要使用的密钥（支持多密钥轮换）
    ///
    /// # 参数
    /// - `keys_str`: 逗号分隔的密钥字符串
    /// - `config_json`: 当前的密钥轮换配置
    ///
    /// # 返回
    /// 返回 (选中的密钥, 更新后的配置 JSON)
    fn select_api_key_with_rotation(
        &self,
        keys_str: &str,
        config_json: Option<&str>,
    ) -> Result<(String, Option<String>)> {
        // 尝试解析为多个密钥
        match ApiKeyRotator::parse_keys(keys_str) {
            Ok(keys) if keys.len() > 1 => {
                // 多密钥模式：使用轮换逻辑
                let (selected_key, new_config) =
                    ApiKeyRotator::select_next_key(keys_str, config_json)?;
                #[cfg(debug_assertions)]
                eprintln!(
                    "[LLMClientManager] Key rotation: selected key index from {} keys",
                    keys.len()
                );
                Ok((selected_key, Some(new_config)))
            }
            _ => {
                // 单密钥模式：直接返回
                Ok((keys_str.to_string(), config_json.map(|s| s.to_string())))
            }
        }
    }

    /// 更新提供商的密钥轮换配置
    ///
    /// # 参数
    /// - `provider_id`: 提供商 ID
    /// - `new_config_json`: 新的配置 JSON
    pub fn update_key_rotation_config(
        &self,
        provider_id: i64,
        new_config_json: &str,
    ) -> Result<()> {
        let mut provider = {
            let repo = self.repository.lock().unwrap();
            repo.get_provider_by_id(provider_id)?
                .ok_or_else(|| anyhow::anyhow!("提供商不存在 (id={})", provider_id))?
        };

        // 更新配置
        provider.config_json = Some(new_config_json.to_string());

        // 保存到数据库
        let repo = self.repository.lock().unwrap();
        repo.update_provider(&provider)?;

        #[cfg(debug_assertions)]
        eprintln!(
            "[LLMClientManager] Key rotation config updated for provider_id={}",
            provider_id
        );

        Ok(())
    }

    /// 获取当前活跃的提供商配置
    ///
    /// 用于需要访问提供商元数据（如模型配置、类型等）的场景
    pub fn get_active_provider_config(&self) -> Result<ApiProvider> {
        let repo = self.repository.lock().unwrap();
        repo.get_active_provider()?.context("未设置活跃的 API 提供商，请先在设置中配置")
    }

    /// 根据提供商类型获取提供商配置
    ///
    /// # 参数
    /// - `provider_type`: 提供商类型
    ///
    /// # 返回
    /// 返回第一个匹配指定类型的提供商配置
    fn get_provider_by_type(&self, provider_type: ApiProviderType) -> Result<ApiProvider> {
        let repo = self.repository.lock().unwrap();
        let providers = repo.get_all_providers()?;

        providers
            .into_iter()
            .find(|p| p.provider_type == provider_type)
            .ok_or_else(|| anyhow::anyhow!("未找到 {:?} 类型的提供商", provider_type))
    }

    /// 根据模型 ID 获取对应的 LLM 客户端
    ///
    /// # 功能
    /// 支持两种模型 ID 格式：
    /// - 命名空间格式：`provider:model` (如 `openai:gpt-4o`)
    /// - 传统格式：`model` (使用活跃提供商)
    ///
    /// # 参数
    /// - `model_id`: 模型 ID
    ///
    /// # 返回
    /// 返回对应的 LLM 客户端
    ///
    /// # 示例
    /// ```ignore
    /// // 使用命名空间格式指定提供商
    /// let client = manager.get_client_for_model("openai:gpt-4o")?;
    ///
    /// // 使用传统格式，自动使用活跃提供商
    /// let client = manager.get_client_for_model("gpt-4o")?;
    /// ```
    pub fn get_client_for_model(&self, model_id: &str) -> Result<Box<dyn LLMService>> {
        // 解析模型 ID
        let resolved = ModelResolver::resolve(model_id, None)?;

        // 根据解析结果获取提供商
        let provider = if let Some(provider_type) = resolved.provider_type {
            // 命名空间格式：使用指定的提供商类型
            self.get_provider_by_type(provider_type)
                .with_context(|| format!("无法获取 {:?} 提供商配置", provider_type))?
        } else {
            // 传统格式：使用活跃提供商
            let repo = self.repository.lock().unwrap();
            repo.get_active_provider()?
                .context("未设置活跃的 API 提供商，请先在设置中配置")?
        };

        // 创建客户端
        self.create_client_from_provider(&provider)
    }

    /// 解析模型 ID 并返回模型信息
    ///
    /// # 参数
    /// - `model_id`: 模型 ID
    ///
    /// # 返回
    /// 返回解析后的模型信息（提供商类型和模型名称）
    ///
    /// # 示例
    /// ```ignore
    /// let resolved = manager.resolve_model("openai:gpt-4o")?;
    /// assert_eq!(resolved.model_id, "gpt-4o");
    /// assert_eq!(resolved.provider_type, Some(ApiProviderType::OpenAI));
    /// ```
    pub fn resolve_model(&self, model_id: &str) -> Result<crate::llm::model_resolver::ResolvedModel> {
        // 获取活跃提供商类型作为 fallback
        let fallback_provider = {
            let repo = self.repository.lock().unwrap();
            match repo.get_active_provider() {
                Ok(Some(provider)) => Some(provider.provider_type),
                Ok(None) => None,
                Err(e) => return Err(e.into()),
            }
        };

        ModelResolver::resolve(model_id, fallback_provider)
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
