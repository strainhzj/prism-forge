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
    /// Google Gemini (ML Dev API - API Key 认证)
    Google,
    /// Google Vertex AI Public Preview (API Key via URL parameter)
    GoogleVertex,
}

impl ApiProviderType {
    /// 获取提供商类型的默认 base_url
    pub fn default_base_url(&self) -> &'static str {
        match self {
            ApiProviderType::OpenAI => "https://api.openai.com/v1",
            ApiProviderType::Anthropic => "https://api.anthropic.com",
            ApiProviderType::Ollama => "http://127.0.0.1:11434",
            ApiProviderType::XAI => "https://api.x.ai/v1",
            ApiProviderType::Google => "https://generativelanguage.googleapis.com",
            ApiProviderType::GoogleVertex => "https://aiplatform.googleapis.com",
        }
    }

    /// 获取提供商类型的默认模型
    pub fn default_model(&self) -> &'static str {
        match self {
            ApiProviderType::OpenAI => "gpt-4o-mini",
            ApiProviderType::Anthropic => "claude-3-5-sonnet-20241022",
            ApiProviderType::Ollama => "llama3",
            ApiProviderType::XAI => "grok-4-1-fast-reasoning",
            ApiProviderType::Google => "gemini-2.5-flash-lite",
            ApiProviderType::GoogleVertex => "gemini-2.5-flash-lite",
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

    /// Temperature 参数（控制随机性）
    ///
    /// 范围: 0.0 - 2.0，默认 0.7
    #[serde(rename = "temperature")]
    pub temperature: Option<f32>,

    /// Max Tokens 参数（最大输出 token 数）
    ///
    /// 默认 2000
    #[serde(rename = "max_tokens")]
    pub max_tokens: Option<u32>,

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
            temperature: Some(0.7),
            max_tokens: Some(2000),
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
        assert_eq!(ApiProviderType::Google.default_base_url(), "https://generativelanguage.googleapis.com");
        assert_eq!(ApiProviderType::GoogleVertex.default_base_url(), "https://aiplatform.googleapis.com");
    }

    #[test]
    fn test_provider_requires_api_key() {
        assert!(ApiProviderType::OpenAI.requires_api_key());
        assert!(ApiProviderType::Anthropic.requires_api_key());
        assert!(!ApiProviderType::Ollama.requires_api_key());
        assert!(ApiProviderType::XAI.requires_api_key());
        assert!(ApiProviderType::Google.requires_api_key());
        assert!(ApiProviderType::GoogleVertex.requires_api_key());
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
        assert_eq!(ApiProviderType::Google.default_model(), "gemini-2.5-flash-lite");
        assert_eq!(ApiProviderType::GoogleVertex.default_model(), "gemini-2.5-flash-lite");
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

// ============================================================================
// Wave 1: 新增数据模型 (T1_1 任务)
// ============================================================================

/// 会话元数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    /// 主键 ID
    pub id: Option<i64>,

    /// Claude Code 会话唯一标识 (UUID格式)
    #[serde(rename = "session_id")]
    pub session_id: String,

    /// 项目路径
    #[serde(rename = "project_path")]
    pub project_path: String,

    /// 项目名称 (从路径提取最后一段)
    #[serde(rename = "project_name")]
    pub project_name: String,

    /// 会话 JSONL 文件完整路径
    #[serde(rename = "file_path")]
    pub file_path: String,

    /// 用户评分 (1-5, NULL 表示未评分)
    pub rating: Option<i32>,

    /// 标签数组 (JSON Array 字符串)
    pub tags: String,

    /// 是否归档
    #[serde(rename = "is_archived")]
    pub is_archived: bool,

    /// 是否活跃 (触发器确保唯一性)
    #[serde(rename = "is_active")]
    pub is_active: bool,

    /// 会话创建时间 (RFC3339)
    #[serde(rename = "created_at")]
    pub created_at: String,

    /// 最后更新时间 (RFC3339)
    #[serde(rename = "updated_at")]
    pub updated_at: String,
}

impl Session {
    /// 获取标签数组
    pub fn get_tags(&self) -> Result<Vec<String>> {
        if self.tags.is_empty() || self.tags == "[]" {
            return Ok(Vec::new());
        }
        serde_json::from_str(&self.tags).map_err(|e| anyhow::anyhow!("解析 tags 失败: {}", e))
    }

    /// 设置标签数组
    pub fn set_tags(&mut self, tags: Vec<String>) -> Result<()> {
        self.tags = serde_json::to_string(&tags)
            .unwrap_or_else(|_| "[]".to_string());
        Ok(())
    }
}

/// 消息元数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    /// 主键 ID
    pub id: Option<i64>,

    /// 所属会话 ID
    #[serde(rename = "session_id")]
    pub session_id: String,

    /// 消息唯一标识 (从 JSONL 提取)
    pub uuid: String,

    /// 父消息 UUID (用于构建消息树)
    #[serde(rename = "parent_uuid")]
    pub parent_uuid: Option<String>,

    /// 消息类型
    #[serde(rename = "type")]
    pub msg_type: String,

    /// 消息时间戳 (RFC3339)
    pub timestamp: String,

    /// 在 JSONL 文件中的字节偏移量
    pub offset: i64,

    /// 消息内容的字节长度
    pub length: i64,

    /// 消息摘要 (用于列表展示)
    pub summary: Option<String>,

    /// 父消息在数组中的索引
    #[serde(rename = "parent_idx")]
    pub parent_idx: Option<i32>,

    /// 记录创建时间 (RFC3339)
    #[serde(rename = "created_at")]
    pub created_at: String,
}

/// 消息向量嵌入模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageEmbedding {
    /// 关联的消息 ID
    #[serde(rename = "message_id")]
    pub message_id: i64,

    /// 384维向量 (JSON 序列化的 Vec<f32>)
    pub embedding: String,

    /// 消息摘要文本 (用于语义搜索)
    pub summary: String,

    /// 向量生成时间
    #[serde(rename = "created_at")]
    pub created_at: String,
}

impl MessageEmbedding {
    /// 获取向量数组
    pub fn get_embedding(&self) -> Result<Vec<f32>> {
        serde_json::from_str(&self.embedding)
            .map_err(|e| anyhow::anyhow!("解析 embedding 失败: {}", e))
    }

    /// 设置向量数组
    pub fn set_embedding(&mut self, embedding: Vec<f32>) -> Result<()> {
        self.embedding = serde_json::to_string(&embedding)
            .map_err(|e| anyhow::anyhow!("序列化 embedding 失败: {}", e))?;
        Ok(())
    }
}

/// 保存的提示词模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedPrompt {
    /// 主键 ID
    pub id: Option<i64>,

    /// 关联的会话 ID (可为空，表示全局提示词)
    #[serde(rename = "session_id")]
    pub session_id: Option<String>,

    /// 分类
    pub category: String,

    /// 提示词标题
    pub title: String,

    /// 提示词内容
    pub content: String,

    /// 用户评分 (1-5)
    pub rating: Option<i32>,

    /// 使用次数
    #[serde(rename = "usage_count")]
    pub usage_count: i32,

    /// Token 数量
    pub tokens: Option<i32>,

    /// 创建时间
    #[serde(rename = "created_at")]
    pub created_at: String,
}

/// 元提示词模板模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaTemplate {
    /// 主键 ID
    pub id: Option<i64>,

    /// 模板唯一标识
    pub key: String,

    /// 模板名称
    pub name: String,

    /// 模板内容 (支持变量占位符)
    pub content: String,

    /// 模板描述
    pub description: Option<String>,

    /// 是否启用
    #[serde(rename = "is_active")]
    pub is_active: bool,

    /// 最后更新时间
    #[serde(rename = "updated_at")]
    pub updated_at: String,
}

/// Token 统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenStats {
    /// Token 数量
    pub count: usize,

    /// 估算费用 (USD)
    #[serde(rename = "estimated_cost")]
    pub estimated_cost: f64,
}

/// 时间戳验证函数
///
/// 验证时间戳字符串是否为有效的 RFC3339 格式
pub fn validate_timestamp(timestamp: &str) -> Result<()> {
    if timestamp.is_empty() {
        return Err(anyhow::anyhow!("时间戳不能为空"));
    }

    // 尝试解析为 RFC3339 格式
    chrono::DateTime::parse_from_rfc3339(timestamp)
        .map_err(|e| anyhow::anyhow!("无效的 RFC3339 时间戳: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests_wave1 {
    use super::*;

    #[test]
    fn test_session_tags_operations() {
        let mut session = Session {
            id: None,
            session_id: "test-uuid".to_string(),
            project_path: "/path/to/project".to_string(),
            project_name: "project".to_string(),
            file_path: "/path/to/session.jsonl".to_string(),
            rating: None,
            tags: "[]".to_string(),
            is_archived: false,
            is_active: true,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        // 测试空标签
        assert_eq!(session.get_tags().unwrap().len(), 0);

        // 测试设置标签
        session.set_tags(vec!["bugfix".to_string(), "ui".to_string()]).unwrap();
        assert_eq!(session.get_tags().unwrap(), vec!["bugfix", "ui"]);
    }

    #[test]
    fn test_embedding_vector_operations() {
        let mut embedding = MessageEmbedding {
            message_id: 1,
            embedding: "[]".to_string(),
            summary: "test summary".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        // 测试空向量
        assert_eq!(embedding.get_embedding().unwrap().len(), 0);

        // 测试设置向量
        embedding.set_embedding(vec![0.1, 0.2, 0.3]).unwrap();
        assert_eq!(embedding.get_embedding().unwrap(), vec![0.1, 0.2, 0.3]);
    }

    #[test]
    fn test_validate_timestamp_valid() {
        let timestamp = chrono::Utc::now().to_rfc3339();
        assert!(validate_timestamp(&timestamp).is_ok());
    }

    #[test]
    fn test_validate_timestamp_invalid() {
        assert!(validate_timestamp("").is_err());
        assert!(validate_timestamp("invalid-timestamp").is_err());
    }

    #[test]
    fn test_token_stats_serialization() {
        let stats = TokenStats {
            count: 1000,
            estimated_cost: 0.002,
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"count\":1000"));
        assert!(json.contains("\"estimatedCost\":0.002"));
    }
}
