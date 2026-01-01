//! Google Gemini API 提供商
//!
//! 支持 Gemini 2.5 Flash Lite, Gemini 2.0 Flash 等模型
//! 支持两种 API 类型：
//! - ML Dev API (API Key 认证)
//! - Vertex AI (OAuth2/ADC 认证)

use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

use crate::llm::interface::{
    LLMService, Message, MessageRole, ModelParams, ChatCompletionResponse,
    StreamChunk,
};

/// Google API 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoogleApiType {
    /// ML Dev API (API Key 认证)
    MlDev,
    /// Vertex AI (OAuth2/Service Account 认证)
    VertexAi,
}

/// Google 提供商配置
#[derive(Debug, Clone)]
pub struct GoogleConfig {
    /// API 类型
    pub api_type: GoogleApiType,

    /// ML Dev API Key
    pub api_key: Option<SecretString>,

    /// Vertex AI 项目 ID
    pub project: Option<String>,

    /// Vertex AI 位置（默认 us-central1）
    pub location: String,

    /// 基础 URL（可选，用于自定义端点）
    pub base_url: String,

    /// Vertex AI Access Token（Bearer token）
    pub access_token: Option<String>,
}

impl Default for GoogleConfig {
    fn default() -> Self {
        Self {
            api_type: GoogleApiType::MlDev,
            api_key: None,
            project: None,
            location: "us-central1".to_string(),
            base_url: "https://generativelanguage.googleapis.com".to_string(),
            access_token: None,
        }
    }
}

/// Google 提供商客户端
pub struct GoogleProvider {
    /// HTTP 客户端
    client: Client,

    /// 提供商配置
    config: GoogleConfig,

    /// API Key 引用（仅用于标识）
    _api_key_ref: Option<String>,
}

impl GoogleProvider {
    /// 创建新的 Google 提供商（ML Dev API）
    ///
    /// # 参数
    /// - `api_key`: Google ML Dev API Key
    /// - `base_url`: API 基础 URL
    pub fn new(api_key: SecretString, _base_url: String) -> Self {
        let client = Client::builder()
            .build()
            .expect("创建 HTTP 客户端失败");

        Self {
            client,
            config: GoogleConfig {
                api_type: GoogleApiType::MlDev,
                api_key: Some(api_key),
                base_url: _base_url,
                ..Default::default()
            },
            _api_key_ref: None,
        }
    }

    /// 使用 API Key 引用创建提供商
    pub fn with_ref(api_key: SecretString, base_url: String, api_key_ref: String) -> Self {
        let client = Client::builder()
            .build()
            .expect("创建 HTTP 客户端失败");

        Self {
            client,
            config: GoogleConfig {
                api_type: GoogleApiType::MlDev,
                api_key: Some(api_key),
                base_url,
                ..Default::default()
            },
            _api_key_ref: Some(api_key_ref),
        }
    }

    /// 创建 Vertex AI 提供商
    ///
    /// # 参数
    /// - `project`: GCP 项目 ID
    /// - `location`: Vertex AI 位置（如 us-central1）
    /// - `base_url`: 自定义端点（可选）
    /// - `access_token`: OAuth2 Bearer token（可选）
    pub fn new_vertexai(
        project: String,
        location: String,
        base_url: Option<String>,
        access_token: Option<String>,
    ) -> Self {
        let client = Client::builder()
            .build()
            .expect("创建 HTTP 客户端失败");

        Self {
            client,
            config: GoogleConfig {
                api_type: GoogleApiType::VertexAi,
                api_key: None,
                project: Some(project),
                location,
                base_url: base_url.unwrap_or_else(|| "https://us-central1-aiplatform.googleapis.com".to_string()),
                access_token,
            },
            _api_key_ref: None,
        }
    }

    /// 将通用 Message 转换为 Google Gemini 格式
    ///
    /// Google 的消息格式：
    /// - role: "user" | "model"
    /// - parts: [{ text: string }]
    ///
    /// 特殊处理：
    /// - 系统消息：转换为用户消息的前缀指令
    /// - 助手消息：role 映射为 "model"
    fn convert_message(msg: Message) -> GeminiContent {
        match msg.role {
            MessageRole::System => GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart::Text {
                    text: format!("System instruction: {}\n\n", msg.content),
                }],
            },
            MessageRole::User => GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart::Text { text: msg.content }],
            },
            MessageRole::Assistant => GeminiContent {
                role: "model".to_string(),
                parts: vec![GeminiPart::Text { text: msg.content }],
            },
        }
    }

    /// 构建请求体
    fn build_request(
        &self,
        messages: Vec<Message>,
        params: ModelParams,
    ) -> Result<GeminiRequest> {
        // 将所有消息转换为 Google 格式
        let contents: Vec<GeminiContent> = messages
            .into_iter()
            .map(Self::convert_message)
            .collect();

        Ok(GeminiRequest {
            contents,
            generation_config: Some(GenerationConfig {
                temperature: Some(params.temperature),
                max_output_tokens: params.max_tokens,
                top_p: Some(params.top_p),
                stop_sequences: params.stop,
            }),
        })
    }

    /// 获取 API 端点 URL
    fn get_endpoint_url(&self, model: &str, stream: bool) -> String {
        match self.config.api_type {
            GoogleApiType::MlDev => {
                let stream_suffix = if stream { "&alt=sse" } else { "" };
                let api_key = self.config.api_key.as_ref()
                    .map(|k| k.expose_secret().to_string())
                    .unwrap_or_default();
                // ML Dev API URL 格式
                format!(
                    "{}/v1beta/models/{}:generateContent?key={}{}",
                    self.config.base_url, model, api_key, stream_suffix
                )
            }
            GoogleApiType::VertexAi => {
                let project = self.config.project.as_ref().expect("Vertex AI 需要 project ID");
                let location = &self.config.location;
                let stream_suffix = if stream { ":streamGenerateContent" } else { ":generateContent" };
                // Vertex AI URL 格式
                format!(
                    "{}/v1/projects/{}/locations/{}/publishers/google/models/{}{}",
                    self.config.base_url, project, location, model, stream_suffix
                )
            }
        }
    }

    /// 发送非流式请求
    async fn send_request(&self, model: &str, request: &GeminiRequest) -> Result<GeminiResponse> {
        let url = self.get_endpoint_url(model, false);

        #[cfg(debug_assertions)]
        eprintln!("[GoogleProvider] Sending request to: {}", url);

        // 构建请求
        let mut req_builder = self
            .client
            .post(&url)
            .header("content-type", "application/json");

        // Vertex AI 需要添加 Authorization 头
        if self.config.api_type == GoogleApiType::VertexAi {
            if let Some(token) = &self.config.access_token {
                req_builder = req_builder.header("Authorization", format!("Bearer {}", token));
            } else {
                return Err(anyhow::anyhow!(
                    "Vertex AI 模式需要配置 access_token。请在 config_json 中添加 access_token 字段，或使用 gcloud CLI 获取 token: gcloud auth print-access-token"
                ));
            }
        }

        let response = req_builder
            .json(request)
            .send()
            .await
            .context("发送 Google API 请求失败")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Google API 错误: {} - {}",
                status,
                error_text
            ));
        }

        response
            .json::<GeminiResponse>()
            .await
            .context("解析 Google API 响应失败")
    }

    /// 发送流式请求
    async fn send_stream_request(
        &self,
        model: &str,
        request: &GeminiRequest,
    ) -> Result<Box<dyn Stream<Item = Result<StreamChunk>> + Send + Unpin>> {
        let url = self.get_endpoint_url(model, true);

        #[cfg(debug_assertions)]
        eprintln!("[GoogleProvider] Sending stream request to: {}", url);

        // 构建请求
        let mut req_builder = self
            .client
            .post(&url)
            .header("content-type", "application/json");

        // Vertex AI 需要添加 Authorization 头
        if self.config.api_type == GoogleApiType::VertexAi {
            if let Some(token) = &self.config.access_token {
                req_builder = req_builder.header("Authorization", format!("Bearer {}", token));
            } else {
                return Err(anyhow::anyhow!(
                    "Vertex AI 模式需要配置 access_token。请在 config_json 中添加 access_token 字段，或使用 gcloud CLI 获取 token: gcloud auth print-access-token"
                ));
            }
        }

        let response = req_builder
            .json(request)
            .send()
            .await
            .context("发送 Google 流式请求失败")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Google API 错误: {} - {}",
                status,
                error_text
            ));
        }

        let stream = response.bytes_stream().map(|chunk_result| match chunk_result {
            Ok(chunk) => {
                let text = String::from_utf8_lossy(&chunk);
                // 解析 SSE 格式
                for line in text.lines() {
                    if line.starts_with("data:") {
                        let json_str = line[5..].trim();
                        if json_str.is_empty() {
                            continue;
                        }

                        if let Ok(event) = serde_json::from_str::<StreamResponse>(json_str) {
                            if let Some(candidates) = event.candidates {
                                if !candidates.is_empty() {
                                    if let Some(content) = &candidates[0].content {
                                        if !content.parts.is_empty() {
                                            // 检查第一个 part 是否是 Text 类型
                                            if let GeminiPart::Text { text } = &content.parts[0] {
                                                return Ok(StreamChunk {
                                                    delta: text.clone(),
                                                    is_finish: false,
                                                    finish_reason: None,
                                                });
                                            }
                                        }
                                    }
                                    // 检查 finish_reason
                                    if let Some(finish_reason) = &candidates[0].finish_reason {
                                        if finish_reason == "STOP" || finish_reason == "MAX_TOKENS" {
                                            return Ok(StreamChunk {
                                                delta: String::new(),
                                                is_finish: true,
                                                finish_reason: Some(finish_reason.clone()),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(StreamChunk {
                    delta: String::new(),
                    is_finish: false,
                    finish_reason: None,
                })
            }
            Err(e) => Err(anyhow::anyhow!("流式响应错误: {}", e)),
        });

        Ok(Box::new(Box::pin(stream)))
    }
}

#[async_trait]
impl LLMService for GoogleProvider {
    async fn chat_completion(
        &self,
        messages: Vec<Message>,
        params: ModelParams,
    ) -> Result<ChatCompletionResponse> {
        let model = params.model.clone();
        let request = self.build_request(messages, params)?;
        let response = self.send_request(&model, &request).await?;

        // 提取文本内容
        let content = response
            .candidates
            .as_ref()
            .iter()
            .flat_map(|candidates| {
                candidates
                    .iter()
                    .filter(|c| c.finish_reason.as_ref().map_or(true, |r| r != "RECITATION"))
                    .filter_map(|c| c.content.as_ref())
            })
            .filter_map(|content| content.parts.first())
            .filter_map(|part| {
                if let GeminiPart::Text { text } = part {
                    Some(text)
                } else {
                    None
                }
            })
            .cloned()
            .collect::<Vec<_>>()
            .join("");

        let finish_reason = response
            .candidates
            .as_ref()
            .and_then(|candidates| candidates.first())
            .and_then(|c| c.finish_reason.clone());

        let usage_metadata = response
            .usage_metadata
            .as_ref();

        Ok(ChatCompletionResponse {
            content,
            model: response.model.unwrap_or_default(),
            finish_reason,
            prompt_tokens: usage_metadata.map(|u| u.prompt_token_count.unwrap_or(0)),
            completion_tokens: usage_metadata.map(|u| u.candidates_token_count.unwrap_or(0)),
            total_tokens: usage_metadata.map(|u| u.total_token_count.unwrap_or(0)),
        })
    }

    async fn stream_completion(
        &self,
        messages: Vec<Message>,
        params: ModelParams,
    ) -> Result<Box<dyn Stream<Item = Result<StreamChunk>> + Send + Unpin>> {
        let model = params.model.clone();
        let request = self.build_request(messages, params)?;
        self.send_stream_request(&model, &request).await
    }

    fn service_type(&self) -> &'static str {
        "Google"
    }
}

// ========== Google API 数据结构 ==========

/// Gemini 内容（消息）
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

/// Gemini 内容部分
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum GeminiPart {
    Text { text: String },
    InlineData {
        inline_data: InlineData,
    },
    FileData {
        file_data: FileData,
    },
}

/// 内联数据（用于图片等）
#[derive(Debug, Clone, Serialize, Deserialize)]
struct InlineData {
    mime_type: String,
    data: String, // base64 编码
}

/// 文件数据（GCS URI）
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileData {
    mime_type: String,
    file_uri: String,
}

/// Gemini 请求
#[derive(Debug, Clone, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
}

/// 生成配置
#[derive(Debug, Clone, Serialize)]
struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
}

/// Gemini 响应
#[derive(Debug, Clone, Deserialize)]
struct GeminiResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    candidates: Option<Vec<GeminiCandidate>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    usage_metadata: Option<UsageMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
}

/// Gemini 候选结果
#[derive(Debug, Clone, Deserialize)]
struct GeminiCandidate {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    finish_reason: Option<String>,
    #[serde(rename = "index")]
    _index: Option<u32>,
}

/// 使用情况元数据
#[derive(Debug, Clone, Deserialize)]
struct UsageMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_token_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    candidates_token_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_token_count: Option<u32>,
}

/// 流式响应
#[derive(Debug, Clone, Deserialize)]
struct StreamResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    candidates: Option<Vec<GeminiCandidate>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_user_message() {
        let user_msg = Message::user("Hello");
        let gemini_msg = GoogleProvider::convert_message(user_msg);
        assert_eq!(gemini_msg.role, "user");
        assert_eq!(gemini_msg.parts.len(), 1);
        match &gemini_msg.parts[0] {
            GeminiPart::Text { text } => assert_eq!(text, "Hello"),
            _ => panic!("Expected Text part"),
        }
    }

    #[test]
    fn test_convert_system_message() {
        let system_msg = Message::system("You are helpful");
        let gemini_msg = GoogleProvider::convert_message(system_msg);
        assert_eq!(gemini_msg.role, "user");
        match &gemini_msg.parts[0] {
            GeminiPart::Text { text } => {
                assert!(text.starts_with("System instruction:"));
            }
            _ => panic!("Expected Text part"),
        }
    }

    #[test]
    fn test_convert_assistant_message() {
        let assistant_msg = Message::assistant("Hi there!");
        let gemini_msg = GoogleProvider::convert_message(assistant_msg);
        assert_eq!(gemini_msg.role, "model");
        match &gemini_msg.parts[0] {
            GeminiPart::Text { text } => assert_eq!(text, "Hi there!"),
            _ => panic!("Expected Text part"),
        }
    }

    #[test]
    fn test_build_request() {
        let provider = GoogleProvider::new(
            SecretString::new("test-key".to_string().into()),
            "https://generativelanguage.googleapis.com".to_string(),
        );

        let messages = vec![
            Message::system("You are helpful"),
            Message::user("Hello"),
        ];
        let params = ModelParams::new("gemini-2.5-flash-lite");

        let request = provider.build_request(messages, params);
        assert!(request.is_ok());

        let request = request.unwrap();
        assert_eq!(request.contents.len(), 2);
        assert!(request.generation_config.is_some());
    }

    #[test]
    fn test_get_endpoint_ml_dev() {
        let provider = GoogleProvider::new(
            SecretString::new("test-key".to_string().into()),
            "https://generativelanguage.googleapis.com".to_string(),
        );

        let url = provider.get_endpoint_url("gemini-2.5-flash-lite", false);
        assert!(url.contains("generativelanguage.googleapis.com"));
        assert!(url.contains("gemini-2.5-flash-lite"));
        assert!(url.contains("key=test-key"));
    }

    #[test]
    fn test_get_endpoint_vertexai() {
        let provider = GoogleProvider::new_vertexai(
            "test-project".to_string(),
            "us-central1".to_string(),
            None,
            None,
        );

        let url = provider.get_endpoint_url("gemini-2.5-flash-lite", false);
        assert!(url.contains("us-central1-aiplatform.googleapis.com"));
        assert!(url.contains("projects/test-project"));
        assert!(url.contains("locations/us-central1"));
        assert!(url.contains("gemini-2.5-flash-lite"));
    }

    #[test]
    fn test_google_config_default() {
        let config = GoogleConfig::default();
        assert_eq!(config.api_type, GoogleApiType::MlDev);
        assert_eq!(config.location, "us-central1");
        assert!(config.api_key.is_none());
        assert!(config.project.is_none());
    }
}
