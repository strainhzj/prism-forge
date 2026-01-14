//! 模型 ID 解析器
//!
//! 支持两种模型 ID 格式：
//! - 命名空间格式：`provider:model` (如 `openai:gpt-4o`)
//! - 传统格式：`model` (使用 fallback provider)
//!
//! 参考文档: Cherry Studio MODEL_SERVICE_ARCHITECTURE.md

use anyhow::{Context, Result};
use crate::database::ApiProviderType;

/// 模型分隔符
const MODEL_SEPARATOR: &str = ":";

/// 模型解析结果
#[derive(Debug, Clone)]
pub struct ResolvedModel {
    /// 提供商类型（如果明确指定）
    pub provider_type: Option<ApiProviderType>,
    /// 实际的模型 ID（不含提供商前缀）
    pub model_id: String,
}

/// 模型 ID 解析器
pub struct ModelResolver;

impl ModelResolver {
    /// 解析模型 ID
    ///
    /// # 参数
    /// - `model_id`: 模型 ID，支持两种格式
    ///   - 命名空间格式：`provider:model` (如 `openai:gpt-4o`)
    ///   - 传统格式：`model` (如 `gpt-4o`)
    /// - `fallback_provider`: 当 model_id 为传统格式时使用的提供商
    ///
    /// # 返回
    /// 返回解析结果，包含提供商类型和模型 ID
    ///
    /// # 示例
    /// ```ignore
    /// // 命名空间格式
    /// let resolved = ModelResolver::resolve("openai:gpt-4o", None)?;
    /// assert_eq!(resolved.provider_type, Some(ApiProviderType::OpenAI));
    /// assert_eq!(resolved.model_id, "gpt-4o");
    ///
    /// // 传统格式
    /// let resolved = ModelResolver::resolve("gpt-4o", Some(ApiProviderType::OpenAI))?;
    /// assert_eq!(resolved.provider_type, Some(ApiProviderType::OpenAI));
    /// assert_eq!(resolved.model_id, "gpt-4o");
    /// ```
    pub fn resolve(
        model_id: &str,
        fallback_provider: Option<ApiProviderType>,
    ) -> Result<ResolvedModel> {
        if model_id.contains(MODEL_SEPARATOR) {
            // 命名空间格式：provider:model
            Self::resolve_namespaced(model_id)
        } else {
            // 传统格式：使用 fallback provider
            Self::resolve_traditional(model_id, fallback_provider)
        }
    }

    /// 解析命名空间格式的模型 ID
    ///
    /// 格式: `provider:model`
    /// - `openai:gpt-4o` → OpenAI provider, model `gpt-4o`
    /// - `anthropic:claude-3-5-sonnet` → Anthropic provider, model `claude-3-5-sonnet`
    fn resolve_namespaced(model_id: &str) -> Result<ResolvedModel> {
        let parts: Vec<&str> = model_id.splitn(2, MODEL_SEPARATOR).collect();

        if parts.len() != 2 {
            anyhow::bail!(
                "无效的模型 ID 格式: '{}', 期望格式: 'provider:model'",
                model_id
            );
        }

        let provider_str = parts[0];
        let actual_model_id = parts[1];

        // 解析提供商类型
        let provider_type = Self::parse_provider_type(provider_str)
            .with_context(|| format!("无法解析提供商类型: '{}'", provider_str))?;

        Ok(ResolvedModel {
            provider_type: Some(provider_type),
            model_id: actual_model_id.to_string(),
        })
    }

    /// 解析传统格式的模型 ID
    ///
    /// 格式: `model` (使用 fallback provider)
    /// - `gpt-4o` → 使用 fallback provider
    fn resolve_traditional(
        model_id: &str,
        fallback_provider: Option<ApiProviderType>,
    ) -> Result<ResolvedModel> {
        if fallback_provider.is_none() {
            anyhow::bail!(
                "传统格式的模型 ID '{}' 需要指定 fallback provider",
                model_id
            );
        }

        Ok(ResolvedModel {
            provider_type: fallback_provider,
            model_id: model_id.to_string(),
        })
    }

    /// 解析提供商类型字符串
    ///
    /// 支持以下格式（不区分大小写）：
    /// - "openai" → ApiProviderType::OpenAI
    /// - "anthropic" → ApiProviderType::Anthropic
    /// - "ollama" → ApiProviderType::Ollama
    /// - "xai" → ApiProviderType::XAI
    /// - "google" → ApiProviderType::Google
    /// - "googlevertex" → ApiProviderType::GoogleVertex
    /// - "azure" / "azureopenai" → ApiProviderType::AzureOpenAI
    /// - "openai-compatible" / "openai_compatible" → ApiProviderType::OpenAICompatible
    fn parse_provider_type(provider_str: &str) -> Result<ApiProviderType> {
        match provider_str.to_lowercase().as_str() {
            "openai" | "oai" => Ok(ApiProviderType::OpenAI),
            "anthropic" | "claude" => Ok(ApiProviderType::Anthropic),
            "ollama" => Ok(ApiProviderType::Ollama),
            "xai" | "grok" => Ok(ApiProviderType::XAI),
            "google" | "gemini" => Ok(ApiProviderType::Google),
            "googlevertex" | "vertex" | "vertexai" => Ok(ApiProviderType::GoogleVertex),
            "azure" | "azureopenai" | "azure-openai" => Ok(ApiProviderType::AzureOpenAI),
            "openai-compatible" | "openai_compatible" | "openai compatible" => {
                Ok(ApiProviderType::OpenAICompatible)
            }
            _ => anyhow::bail!("未知的提供商类型: '{}'", provider_str),
        }
    }

    /// 构建完整的模型 ID（带提供商前缀）
    ///
    /// # 参数
    /// - `provider_type`: 提供商类型
    /// - `model_id`: 模型 ID
    ///
    /// # 示例
    /// ```ignore
    /// let full_id = ModelResolver::build_model_id(ApiProviderType::OpenAI, "gpt-4o");
    /// assert_eq!(full_id, "openai:gpt-4o");
    /// ```
    pub fn build_model_id(provider_type: ApiProviderType, model_id: &str) -> String {
        let provider_str = match provider_type {
            ApiProviderType::OpenAI => "openai",
            ApiProviderType::Anthropic => "anthropic",
            ApiProviderType::Ollama => "ollama",
            ApiProviderType::XAI => "xai",
            ApiProviderType::Google => "google",
            ApiProviderType::GoogleVertex => "googlevertex",
            ApiProviderType::AzureOpenAI => "azureopenai",
            ApiProviderType::OpenAICompatible => "openai-compatible",
        };
        format!("{}{}{}", provider_str, MODEL_SEPARATOR, model_id)
    }

    /// 检查模型 ID 是否为命名空间格式
    ///
    /// # 返回
    /// 如果包含分隔符 `:`，则认为是命名空间格式
    pub fn is_namespaced(model_id: &str) -> bool {
        model_id.contains(MODEL_SEPARATOR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_namespaced_openai() {
        let resolved = ModelResolver::resolve("openai:gpt-4o", None).unwrap();
        assert_eq!(resolved.provider_type, Some(ApiProviderType::OpenAI));
        assert_eq!(resolved.model_id, "gpt-4o");
    }

    #[test]
    fn test_resolve_namespaced_anthropic() {
        let resolved = ModelResolver::resolve("anthropic:claude-3-5-sonnet", None).unwrap();
        assert_eq!(resolved.provider_type, Some(ApiProviderType::Anthropic));
        assert_eq!(resolved.model_id, "claude-3-5-sonnet");
    }

    #[test]
    fn test_resolve_traditional_with_fallback() {
        let resolved = ModelResolver::resolve("gpt-4o", Some(ApiProviderType::OpenAI)).unwrap();
        assert_eq!(resolved.provider_type, Some(ApiProviderType::OpenAI));
        assert_eq!(resolved.model_id, "gpt-4o");
    }

    #[test]
    fn test_resolve_traditional_no_fallback() {
        let result = ModelResolver::resolve("gpt-4o", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_invalid_format() {
        let result = ModelResolver::resolve("openai:gpt-4o:extra", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_unknown_provider() {
        let result = ModelResolver::resolve("unknown:model", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_provider_type_case_insensitive() {
        assert_eq!(
            ModelResolver::parse_provider_type("OPENAI").unwrap(),
            ApiProviderType::OpenAI
        );
        assert_eq!(
            ModelResolver::parse_provider_type("Claude").unwrap(),
            ApiProviderType::Anthropic
        );
    }

    #[test]
    fn test_parse_provider_type_aliases() {
        assert_eq!(
            ModelResolver::parse_provider_type("oai").unwrap(),
            ApiProviderType::OpenAI
        );
        assert_eq!(
            ModelResolver::parse_provider_type("claude").unwrap(),
            ApiProviderType::Anthropic
        );
        assert_eq!(
            ModelResolver::parse_provider_type("azure").unwrap(),
            ApiProviderType::AzureOpenAI
        );
    }

    #[test]
    fn test_build_model_id() {
        assert_eq!(
            ModelResolver::build_model_id(ApiProviderType::OpenAI, "gpt-4o"),
            "openai:gpt-4o"
        );
        assert_eq!(
            ModelResolver::build_model_id(ApiProviderType::Anthropic, "claude-3-5-sonnet"),
            "anthropic:claude-3-5-sonnet"
        );
    }

    #[test]
    fn test_is_namespaced() {
        assert!(ModelResolver::is_namespaced("openai:gpt-4o"));
        assert!(!ModelResolver::is_namespaced("gpt-4o"));
    }
}
