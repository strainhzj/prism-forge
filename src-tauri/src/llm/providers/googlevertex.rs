//! Google Vertex AI Public Preview API 提供商
//!
//! 使用 API Key 通过 URL 参数进行认证
//! URL 格式: https://aiplatform.googleapis.com/v1/publishers/google/models/{model}:generateContent?key={api_key}

use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize, Deserializer};

use crate::llm::interface::{
    LLMService, Message, MessageRole, ModelParams, ChatCompletionResponse,
    StreamChunk,
};

/// Google Vertex AI Public Preview 提供商客户端
pub struct GoogleVertexProvider {
    /// HTTP 客户端
    client: Client,

    /// API Key
    api_key: SecretString,

    /// 基础 URL
    base_url: String,

    /// API Key 引用（仅用于标识）
    _api_key_ref: Option<String>,
}

impl GoogleVertexProvider {
    /// 创建新的 Google Vertex 提供商
    ///
    /// # 参数
    /// - `api_key`: Google Cloud API Key
    /// - `base_url`: API 基础 URL
    pub fn new(api_key: SecretString, base_url: String) -> Self {
        let client = Client::builder()
            .build()
            .expect("创建 HTTP 客户端失败");

        Self {
            client,
            api_key,
            base_url,
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
            api_key,
            base_url,
            _api_key_ref: Some(api_key_ref),
        }
    }

    /// 将通用 Message 转换为 Google Vertex 格式
    ///
    /// Google 的消息格式：
    /// - role: "user" | "model"
    /// - parts: [{ text: string }]
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
    ///
    /// 参考 Python 实现：只包含 contents，不包含 generation_config
    /// Google Vertex AI Public Preview API 可能拒绝包含额外参数的请求
    fn build_request(
        &self,
        messages: Vec<Message>,
        _params: ModelParams,
    ) -> Result<VertexRequest> {
        // 将所有消息转换为 Google 格式
        let contents: Vec<GeminiContent> = messages
            .into_iter()
            .map(Self::convert_message)
            .collect();

        // 与 Python 实现保持一致：只包含 contents，不包含 generation_config
        Ok(VertexRequest {
            contents,
            generation_config: None,  // 暂时禁用，与 Python 版本一致
        })
    }

    /// 获取 API 端点 URL
    fn get_endpoint_url(&self, model: &str, _stream: bool) -> String {
        let api_key = self.api_key.expose_secret();
        let method = "streamGenerateContent";
        // Vertex AI Public Preview URL 格式
        // 注意：base_url 应该包含 /v1，或者在这里添加
        let base = if self.base_url.ends_with("/v1") {
            self.base_url.clone()
        } else if self.base_url.ends_with("/v1/") {
            self.base_url.clone()
        } else {
            format!("{}/v1", self.base_url.trim_end_matches('/'))
        };
        format!(
            "{}/publishers/google/models/{}:{}?key={}",
            base, model, method, api_key
        )
    }

    /// 发送非流式请求
    async fn send_request(&self, model: &str, request: &VertexRequest) -> Result<VertexResponse> {
        let url = self.get_endpoint_url(model, false);

        #[cfg(debug_assertions)]
        eprintln!("[GoogleVertexProvider] Sending request to: {}", url);

        let response = self
            .client
            .post(&url)
            .header("content-type", "application/json")
            .json(request)
            .send()
            .await
            .context("发送 Google Vertex API 请求失败")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Google Vertex API 错误: {} - {}",
                status,
                error_text
            ));
        }

        // 获取原始响应文本用于调试
        let response_text = response.text().await.unwrap_or_default();

        #[cfg(debug_assertions)]
        {
            eprintln!("[GoogleVertexProvider] Raw response (first 2000 chars):");
            eprintln!("{}", &response_text.chars().take(2000).collect::<String>());
            eprintln!("[GoogleVertexProvider] Total response length: {} chars", response_text.len());
            eprintln!("[GoogleVertexProvider] Attempting to parse response...");
        }

        // 尝试解析响应
        let parsed_response = serde_json::from_str::<VertexResponse>(&response_text)
            .context("解析 Google Vertex API 响应失败")?;

        #[cfg(debug_assertions)]
        {
            if let Some(candidates) = &parsed_response.candidates {
                eprintln!("[GoogleVertexProvider] Parsed successfully: {} candidate(s)", candidates.len());
                for (i, candidate) in candidates.iter().enumerate() {
                    if let Some(content) = &candidate.content {
                        if let Some(first_part) = content.parts.first() {
                            if let GeminiPart::Text { text } = first_part {
                                eprintln!("  - Candidate {} text (first 100 chars): {}...", i, &text.chars().take(100).collect::<String>());
                            }
                        }
                    }
                }
            } else {
                eprintln!("[GoogleVertexProvider] Parsed successfully: no candidates");
            }
        }

        Ok(parsed_response)
    }

    /// 发送流式请求
    async fn send_stream_request(
        &self,
        model: &str,
        request: &VertexRequest,
    ) -> Result<Box<dyn Stream<Item = Result<StreamChunk>> + Send + Unpin>> {
        let url = self.get_endpoint_url(model, true);

        #[cfg(debug_assertions)]
        eprintln!("[GoogleVertexProvider] Sending stream request to: {}", url);

        let response = self
            .client
            .post(&url)
            .header("content-type", "application/json")
            .json(request)
            .send()
            .await
            .context("发送 Google Vertex 流式请求失败")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Google Vertex API 错误: {} - {}",
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
impl LLMService for GoogleVertexProvider {
    async fn chat_completion(
        &self,
        messages: Vec<Message>,
        params: ModelParams,
    ) -> Result<ChatCompletionResponse> {
        // 添加调试日志
        #[cfg(debug_assertions)]
        {
            eprintln!("[GoogleVertexProvider] chat_completion called");
            eprintln!("  - model: {}", params.model);
            eprintln!("  - temperature: {}", params.temperature);
            eprintln!("  - max_tokens: {:?}", params.max_tokens);
            eprintln!("  - messages count: {}", messages.len());
        }

        let model = params.model.clone();
        let request = self.build_request(messages, params)?;

        // 打印请求体
        #[cfg(debug_assertions)]
        {
            eprintln!("  - request body: {}", serde_json::to_string_pretty(&request).unwrap_or_default());
        }

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
        "GoogleVertex"
    }
}

// ========== Google Vertex API 数据结构 ==========

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

/// Vertex AI 请求
#[derive(Debug, Clone, Serialize)]
struct VertexRequest {
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

/// Vertex AI 响应
///
/// Google Vertex AI API 可能返回两种格式：
/// 1. 对象格式: `{ "candidates": [...] }`
/// 2. 数组格式: `[{ "candidates": [...] }]`
///
/// 使用自定义反序列化来兼容两种格式
#[derive(Debug, Clone)]
struct VertexResponse {
    candidates: Option<Vec<VertexCandidate>>,
    usage_metadata: Option<UsageMetadata>,
    model: Option<String>,
}

impl<'de> Deserialize<'de> for VertexResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // 尝试直接反序列化为对象格式
        #[derive(Deserialize)]
        struct ObjectFormat {
            #[serde(default)]
            candidates: Option<Vec<VertexCandidate>>,
            #[serde(default)]
            usage_metadata: Option<UsageMetadata>,
            #[serde(default)]
            model: Option<String>,
        }

        // 尝试反序列化为数组格式
        #[derive(Deserialize)]
        struct ArrayFormatItem {
            #[serde(default)]
            candidates: Option<Vec<VertexCandidate>>,
            #[serde(default)]
            usage_metadata: Option<UsageMetadata>,
            #[serde(default)]
            model: Option<String>,
        }

        // 首先尝试作为对象解析
        let obj_result = serde_json::Value::deserialize(deserializer);
        let value = obj_result.map_err(serde::de::Error::custom)?;

        if let Some(array) = value.as_array() {
            // 数组格式：合并所有元素的候选结果
            let mut all_candidates = Vec::new();
            let mut combined_usage = None;
            let mut model_name = None;

            for item in array {
                let parsed: ArrayFormatItem = serde_json::from_value(item.clone())
                    .map_err(serde::de::Error::custom)?;

                // 合并候选结果
                if let Some(candidates) = parsed.candidates {
                    all_candidates.extend(candidates);
                }

                // 使用第一个非空的 usage_metadata
                if combined_usage.is_none() {
                    combined_usage = parsed.usage_metadata;
                }

                // 使用第一个非空的 model
                if model_name.is_none() {
                    model_name = parsed.model;
                }
            }

            Ok(VertexResponse {
                candidates: if all_candidates.is_empty() { None } else { Some(all_candidates) },
                usage_metadata: combined_usage,
                model: model_name,
            })
        } else {
            // 对象格式
            let obj: ObjectFormat = serde_json::from_value(value)
                .map_err(serde::de::Error::custom)?;
            Ok(VertexResponse {
                candidates: obj.candidates,
                usage_metadata: obj.usage_metadata,
                model: obj.model,
            })
        }
    }
}

/// Vertex AI 候选结果
#[derive(Debug, Clone, Deserialize)]
struct VertexCandidate {
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
    candidates: Option<Vec<VertexCandidate>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_user_message() {
        let user_msg = Message::user("Hello");
        let gemini_msg = GoogleVertexProvider::convert_message(user_msg);
        assert_eq!(gemini_msg.role, "user");
        match &gemini_msg.parts[0] {
            GeminiPart::Text { text } => assert_eq!(text, "Hello"),
            _ => panic!("Expected Text part"),
        }
    }

    #[test]
    fn test_convert_system_message() {
        let system_msg = Message::system("You are helpful");
        let gemini_msg = GoogleVertexProvider::convert_message(system_msg);
        assert_eq!(gemini_msg.role, "user");
        match &gemini_msg.parts[0] {
            GeminiPart::Text { text } => {
                assert!(text.starts_with("System instruction:"));
            }
            _ => panic!("Expected Text part"),
        }
    }

    #[test]
    fn test_get_endpoint_url() {
        let provider = GoogleVertexProvider::new(
            SecretString::new("test-key".to_string().into()),
            "https://aiplatform.googleapis.com".to_string(),
        );

        let url = provider.get_endpoint_url("gemini-2.5-flash-lite", false);
        assert!(url.contains("aiplatform.googleapis.com"));
        assert!(url.contains("publishers/google/models"));
        assert!(url.contains("gemini-2.5-flash-lite"));
        assert!(url.contains("generateContent"));
        assert!(url.contains("key=test-key"));
    }

    #[test]
    fn test_get_endpoint_url_stream() {
        let provider = GoogleVertexProvider::new(
            SecretString::new("test-key".to_string().into()),
            "https://aiplatform.googleapis.com".to_string(),
        );

        let url = provider.get_endpoint_url("gemini-2.5-flash-lite", true);
        assert!(url.contains("streamGenerateContent"));
    }
}
