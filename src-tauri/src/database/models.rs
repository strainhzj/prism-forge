//! LLM API Provider 数据模型
//!
//! 定义 API 提供商的数据结构，支持多种 LLM 服务

use serde::{Deserialize, Serialize};
use anyhow::Result;

/// API 提供商类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiProviderType {
    /// OpenAI 兼容接口 (包括 OneAPI、中转服务等)
    OpenAI,
    /// Anthropic Claude
    Anthropic,
    /// Ollama 本地服务
    Ollama,
    /// X AI (Grok)
    XAI,
}

impl ApiProviderType {
    /// 获取提供商类型的默认 base_url
    pub fn default_base_url(&self) -> &'static str {
        match self {
            ApiProviderType::OpenAI => "https://api.openai.com/v1",
            ApiProviderType::Anthropic => "https://api.anthropic.com",
            ApiProviderType::Ollama => "http://127.0.0.1:11434",
            ApiProviderType::XAI => "https://api.x.ai/v1",
        }
    }

    /// 获取提供商类型的默认模型
    pub fn default_model(&self) -> &'static str {
        match self {
            ApiProviderType::OpenAI => "gpt-4o-mini",
            ApiProviderType::Anthropic => "claude-3-5-sonnet-20241022",
            ApiProviderType::Ollama => "llama3",
            ApiProviderType::XAI => "grok-4-1-fast-reasoning",
        }
    }

    /// 判断该类型是否需要 API Key
    pub fn requires_api_key(&self) -> bool {
        match self {
            ApiProviderType::Ollama => false,
            _ => true,
        }
    }
}

/// API 提供商配置模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiProvider {
    /// 主键 ID
    pub id: Option<i64>,

    /// 提供商类型
    #[serde(rename = "provider_type")]
    pub provider_type: ApiProviderType,

    /// 用户自定义名称 (例如: "OpenAI 官方", "Ollama 本地")
    pub name: String,

    /// API 基础 URL
    ///
    /// - OpenAI: https://api.openai.com/v1 或 OneAPI/中转地址
    /// - Anthropic: https://api.anthropic.com
    /// - Ollama: http://127.0.0.1:11434
    #[serde(rename = "base_url")]
    pub base_url: String,

    /// API Key 在 keyring 中的引用标识
    ///
    /// 存储格式: "provider_{id}"，实际密钥通过 keyring crate
    /// 安全存储在系统密钥库中 (Windows Credential Manager, macOS Keychain, Linux Secret Service)
    #[serde(rename = "api_key_ref")]
    pub api_key_ref: Option<String>,

    /// 配置的模型名称
    ///
    /// 如果为 None 或空字符串，使用 provider_type.default_model()
    pub model: Option<String>,

    /// 额外配置 JSON
    ///
    /// 用于存储提供商特定的配置，例如：
    /// - OpenAI: model (gpt-4, gpt-3.5-turbo), temperature
    /// - Anthropic: model (claude-3-5-sonnet), max_tokens
    /// - Ollama: model (llama3, qwen2)
    #[serde(rename = "config_json")]
    pub config_json: Option<String>,

    /// 是否为当前活跃的提供商
    ///
    /// 同一时间只能有一个活跃提供商
    #[serde(rename = "is_active")]
    pub is_active: bool,
}

impl ApiProvider {
    /// 创建新的 API 提供商
    pub fn new(
        provider_type: ApiProviderType,
        name: String,
        base_url: Option<String>,
    ) -> Self {
        Self {
            id: None,
            provider_type,
            name,
            base_url: base_url.unwrap_or_else(|| provider_type.default_base_url().to_string()),
            api_key_ref: None,
            model: None,
            config_json: None,
            is_active: false,
        }
    }

    /// 获取有效的模型名称
    ///
    /// 如果未配置模型或模型为空字符串，返回提供商类型的默认模型
    pub fn effective_model(&self) -> &str {
        self.model
            .as_ref()
            .filter(|m| !m.trim().is_empty())
            .map(|m| m.as_str())
            .unwrap_or_else(|| self.provider_type.default_model())
    }

    /// 设置配置 JSON
    pub fn with_config(mut self, config: serde_json::Value) -> Result<Self> {
        self.config_json = Some(serde_json::to_string(&config)?);
        Ok(self)
    }

    /// 获取配置 JSON
    pub fn get_config(&self) -> Result<Option<serde_json::Value>> {
        match &self.config_json {
            Some(json_str) => Ok(Some(serde_json::from_str(json_str)?)),
            None => Ok(None),
        }
    }

    /// 验证配置是否有效
    pub fn validate(&self) -> Result<()> {
        // 检查 base_url 格式
        if self.base_url.is_empty() {
            return Err(anyhow::anyhow!("base_url 不能为空"));
        }

        // 检查需要 API Key 的提供商是否配置了密钥引用
        if self.provider_type.requires_api_key() && self.api_key_ref.is_none() {
            return Err(anyhow::anyhow!(
                "{:?} 提供商需要配置 API Key",
                self.provider_type
            ));
        }

        // 验证 URL 格式
        if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
            return Err(anyhow::anyhow!("base_url 必须以 http:// 或 https:// 开头"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_default_url() {
        assert_eq!(ApiProviderType::OpenAI.default_base_url(), "https://api.openai.com/v1");
        assert_eq!(ApiProviderType::Anthropic.default_base_url(), "https://api.anthropic.com");
        assert_eq!(ApiProviderType::Ollama.default_base_url(), "http://127.0.0.1:11434");
        assert_eq!(ApiProviderType::XAI.default_base_url(), "https://api.x.ai/v1");
    }

    #[test]
    fn test_provider_requires_api_key() {
        assert!(ApiProviderType::OpenAI.requires_api_key());
        assert!(ApiProviderType::Anthropic.requires_api_key());
        assert!(!ApiProviderType::Ollama.requires_api_key());
        assert!(ApiProviderType::XAI.requires_api_key());
    }

    #[test]
    fn test_new_provider() {
        let provider = ApiProvider::new(
            ApiProviderType::Ollama,
            "本地 Ollama".to_string(),
            None,
        );
        assert_eq!(provider.base_url, "http://127.0.0.1:11434");
        assert!(!provider.is_active);
    }

    #[test]
    fn test_validate_openai_without_key() {
        let provider = ApiProvider::new(
            ApiProviderType::OpenAI,
            "OpenAI".to_string(),
            Some("https://api.openai.com/v1".to_string()),
        );
        assert!(provider.validate().is_err());
    }

    #[test]
    fn test_validate_ollama_without_key() {
        let provider = ApiProvider::new(
            ApiProviderType::Ollama,
            "Ollama".to_string(),
            Some("http://127.0.0.1:11434".to_string()),
        );
        assert!(provider.validate().is_ok());
    }

    #[test]
    fn test_provider_type_default_model() {
        // Feature: provider-model-config, Property 1: Default Model Consistency
        // Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5
        assert_eq!(ApiProviderType::OpenAI.default_model(), "gpt-4o-mini");
        assert_eq!(ApiProviderType::Anthropic.default_model(), "claude-3-5-sonnet-20241022");
        assert_eq!(ApiProviderType::Ollama.default_model(), "llama3");
        assert_eq!(ApiProviderType::XAI.default_model(), "grok-4-1-fast-reasoning");
    }

    #[test]
    fn test_effective_model_with_configured_model() {
        // Feature: provider-model-config, Property 2: Model Fallback Behavior
        // Validates: Requirements 2.2
        let mut provider = ApiProvider::new(
            ApiProviderType::OpenAI,
            "OpenAI".to_string(),
            None,
        );
        provider.model = Some("gpt-4".to_string());
        assert_eq!(provider.effective_model(), "gpt-4");
    }

    #[test]
    fn test_effective_model_with_none() {
        // Feature: provider-model-config, Property 2: Model Fallback Behavior
        // Validates: Requirements 2.2
        let provider = ApiProvider::new(
            ApiProviderType::OpenAI,
            "OpenAI".to_string(),
            None,
        );
        assert_eq!(provider.effective_model(), "gpt-4o-mini");
    }

    #[test]
    fn test_effective_model_with_empty_string() {
        // Feature: provider-model-config, Property 2: Model Fallback Behavior
        // Validates: Requirements 2.2
        let mut provider = ApiProvider::new(
            ApiProviderType::Anthropic,
            "Anthropic".to_string(),
            None,
        );
        provider.model = Some("".to_string());
        assert_eq!(provider.effective_model(), "claude-3-5-sonnet-20241022");
    }

    #[test]
    fn test_effective_model_with_whitespace_only() {
        // Feature: provider-model-config, Property 2: Model Fallback Behavior
        // Validates: Requirements 2.2
        let mut provider = ApiProvider::new(
            ApiProviderType::XAI,
            "XAI".to_string(),
            None,
        );
        provider.model = Some("   ".to_string());
        assert_eq!(provider.effective_model(), "grok-4-1-fast-reasoning");
    }

    #[test]
    fn test_new_provider_model_is_none() {
        // Validates: Requirements 2.1
        let provider = ApiProvider::new(
            ApiProviderType::Ollama,
            "Ollama".to_string(),
            None,
        );
        assert!(provider.model.is_none());
    }
}
