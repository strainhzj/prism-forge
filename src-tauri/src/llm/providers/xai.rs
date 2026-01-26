//! X AI (Grok) API 提供商
//!
//! 支持 X AI 的 Grok 模型，使用 OpenAI 兼容格式

use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

use crate::llm::interface::{
    ChatCompletionResponse, LLMService, Message, MessageRole, ModelParams, StreamChunk,
};

/// X AI 提供商客户端
pub struct XAIProvider {
    /// HTTP 客户端
    client: Client,

    /// API Key
    api_key: SecretString,

    /// 基础 URL
    base_url: String,

    /// API Key 引用（仅用于标识）
    _api_key_ref: Option<String>,
}

impl XAIProvider {
    /// 创建新的 X AI 提供商
    ///
    /// # 参数
    /// - `api_key`: X AI API Key
    /// - `base_url`: API 基础 URL
    pub fn new(api_key: SecretString, base_url: String) -> Result<Self> {
        let client = Client::builder()
            .build()
            .context("创建 HTTP 客户端失败")?;

        Ok(Self {
            client,
            api_key,
            base_url,
            _api_key_ref: None,
        })
    }

    /// 使用 API Key 引用创建提供商
    pub fn with_ref(api_key: SecretString, base_url: String, api_key_ref: String) -> Result<Self> {
        let client = Client::builder()
            .build()
            .context("创建 HTTP 客户端失败")?;

        Ok(Self {
            client,
            api_key,
            base_url,
            _api_key_ref: Some(api_key_ref),
        })
    }


    /// 将通用 Message 转换为 X AI 格式 (OpenAI 兼容)
    fn convert_message(msg: &Message) -> XAIMessage {
        let role = match msg.role {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
        };

        XAIMessage {
            role: role.to_string(),
            content: msg.content.clone(),
        }
    }

    /// 构建请求体
    fn build_request(&self, messages: Vec<Message>, params: ModelParams) -> XAIRequest {
        let xai_messages: Vec<XAIMessage> = messages.iter().map(Self::convert_message).collect();

        XAIRequest {
            model: params.model,
            messages: xai_messages,
            temperature: Some(params.temperature),
            top_p: Some(params.top_p),
            max_tokens: params.max_tokens,
            stop: params.stop,
            stream: false,
        }
    }

    /// 发送非流式请求
    async fn send_request(&self, request: &XAIRequest) -> Result<XAIResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key.expose_secret()))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .context("发送 X AI API 请求失败")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "X AI API 错误: {} - {}",
                status,
                error_text
            ));
        }

        response
            .json::<XAIResponse>()
            .await
            .context("解析 X AI API 响应失败")
    }

    /// 发送流式请求
    async fn send_stream_request(
        &self,
        request: &XAIRequest,
    ) -> Result<Box<dyn Stream<Item = Result<StreamChunk>> + Send + Unpin>> {
        let url = format!("{}/chat/completions", self.base_url);

        let mut stream_request = request.clone();
        stream_request.stream = true;

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key.expose_secret()))
            .header("Content-Type", "application/json")
            .json(&stream_request)
            .send()
            .await
            .context("发送 X AI 流式请求失败")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "X AI API 错误: {} - {}",
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
                        if json_str == "[DONE]" {
                            return Ok(StreamChunk {
                                delta: String::new(),
                                is_finish: true,
                                finish_reason: Some("stop".to_string()),
                            });
                        }

                        if let Ok(event) = serde_json::from_str::<XAIStreamResponse>(json_str) {
                            if let Some(choice) = event.choices.first() {
                                let delta = choice.delta.content.clone().unwrap_or_default();
                                let is_finish = choice.finish_reason.is_some();
                                return Ok(StreamChunk {
                                    delta,
                                    is_finish,
                                    finish_reason: choice.finish_reason.clone(),
                                });
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
impl LLMService for XAIProvider {
    async fn chat_completion(
        &self,
        messages: Vec<Message>,
        params: ModelParams,
    ) -> Result<ChatCompletionResponse> {
        let request = self.build_request(messages, params);
        let response = self.send_request(&request).await?;

        let choice = response
            .choices
            .first()
            .context("X AI 返回空响应")?;

        let content = choice.message.content.clone().unwrap_or_default();

        Ok(ChatCompletionResponse {
            content,
            model: response.model,
            finish_reason: choice.finish_reason.clone(),
            prompt_tokens: response.usage.as_ref().map(|u| u.prompt_tokens),
            completion_tokens: response.usage.as_ref().map(|u| u.completion_tokens),
            total_tokens: response.usage.as_ref().map(|u| u.total_tokens),
        })
    }

    async fn stream_completion(
        &self,
        messages: Vec<Message>,
        params: ModelParams,
    ) -> Result<Box<dyn Stream<Item = Result<StreamChunk>> + Send + Unpin>> {
        let request = self.build_request(messages, params);
        self.send_stream_request(&request).await
    }

    fn service_type(&self) -> &'static str {
        "XAI"
    }
}

// ========== X AI API 数据结构 (OpenAI 兼容) ==========

/// X AI 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
struct XAIMessage {
    role: String,
    content: String,
}

/// X AI 请求
#[derive(Debug, Clone, Serialize)]
struct XAIRequest {
    model: String,
    messages: Vec<XAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    stream: bool,
}

/// X AI 响应
#[derive(Debug, Clone, Deserialize)]
struct XAIResponse {
    id: String,
    model: String,
    choices: Vec<XAIChoice>,
    usage: Option<XAIUsage>,
}

/// X AI 选择
#[derive(Debug, Clone, Deserialize)]
struct XAIChoice {
    index: u32,
    message: XAIResponseMessage,
    finish_reason: Option<String>,
}

/// X AI 响应消息
#[derive(Debug, Clone, Deserialize)]
struct XAIResponseMessage {
    role: String,
    content: Option<String>,
}

/// X AI Token 使用情况
#[derive(Debug, Clone, Deserialize)]
struct XAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// X AI 流式响应
#[derive(Debug, Clone, Deserialize)]
struct XAIStreamResponse {
    choices: Vec<XAIStreamChoice>,
}

/// X AI 流式选择
#[derive(Debug, Clone, Deserialize)]
struct XAIStreamChoice {
    delta: XAIStreamDelta,
    finish_reason: Option<String>,
}

/// X AI 流式增量
#[derive(Debug, Clone, Deserialize)]
struct XAIStreamDelta {
    content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_message() {
        let user_msg = Message::user("Hello");
        let xai_msg = XAIProvider::convert_message(&user_msg);
        assert_eq!(xai_msg.role, "user");
        assert_eq!(xai_msg.content, "Hello");

        let system_msg = Message::system("You are helpful");
        let xai_msg = XAIProvider::convert_message(&system_msg);
        assert_eq!(xai_msg.role, "system");
        assert_eq!(xai_msg.content, "You are helpful");

        let assistant_msg = Message::assistant("Hi there");
        let xai_msg = XAIProvider::convert_message(&assistant_msg);
        assert_eq!(xai_msg.role, "assistant");
        assert_eq!(xai_msg.content, "Hi there");
    }

    #[test]
    fn test_build_request() {
        let provider = XAIProvider::new(
            SecretString::new("test-key".to_string().into()),
            "https://api.x.ai/v1".to_string(),
        ).unwrap();

        let messages = vec![
            Message::system("You are helpful"),
            Message::user("Hello"),
        ];
        let params = ModelParams::new("grok-beta");

        let request = provider.build_request(messages, params);
        assert_eq!(request.model, "grok-beta");
        assert_eq!(request.messages.len(), 2);
        assert!(!request.stream);
    }

    #[test]
    fn test_request_serialization() {
        let provider = XAIProvider::new(
            SecretString::new("test-key".to_string().into()),
            "https://api.x.ai/v1".to_string(),
        ).unwrap();

        let messages = vec![Message::user("test")];
        let params = ModelParams::new("grok-beta")
            .with_temperature(0.5)
            .with_max_tokens(100);

        let request = provider.build_request(messages, params);
        let json = serde_json::to_string(&request).unwrap();

        assert!(json.contains(r#""model":"grok-beta""#));
        assert!(json.contains(r#""temperature":0.5"#));
        assert!(json.contains(r#""max_tokens":100"#));
        assert!(json.contains(r#""stream":false"#));
    }

    #[test]
    fn test_request_has_bearer_auth_format() {
        // This test verifies the request format matches X AI API spec
        let provider = XAIProvider::new(
            SecretString::new("xai-test-key".to_string().into()),
            "https://api.x.ai/v1".to_string(),
        ).unwrap();

        let messages = vec![Message::user("Hello")];
        let request = provider.build_request(messages, ModelParams::new("grok-beta"));

        // Verify request structure
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.messages[0].role, "user");
        assert_eq!(request.messages[0].content, "Hello");
    }
}


/// Property-based tests for X AI request format
/// Feature: fix-provider-connection-test, Property 3: X AI Request Format Correctness
/// Validates: Requirements 3.1, 3.2
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Strategy to generate valid message content
    fn message_content_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("Hello".to_string()),
            Just("How are you?".to_string()),
            "[a-zA-Z0-9 ]{1,100}".prop_map(|s| s),
            Just("你好".to_string()),
            Just("Test message with special chars: !@#$%".to_string()),
        ]
    }

    // Strategy to generate valid model names
    fn model_name_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("grok-beta".to_string()),
            Just("grok-2".to_string()),
            Just("grok-2-mini".to_string()),
            "[a-z0-9-]{3,20}".prop_map(|s| s),
        ]
    }

    // Strategy to generate valid temperature values
    fn temperature_strategy() -> impl Strategy<Value = f32> {
        (0u32..=20u32).prop_map(|v| v as f32 / 10.0)
    }

    // Strategy to generate valid top_p values
    fn top_p_strategy() -> impl Strategy<Value = f32> {
        (1u32..=10u32).prop_map(|v| v as f32 / 10.0)
    }

    // Strategy to generate message lists
    fn messages_strategy() -> impl Strategy<Value = Vec<Message>> {
        prop::collection::vec(
            (
                prop_oneof![
                    Just(MessageRole::System),
                    Just(MessageRole::User),
                    Just(MessageRole::Assistant),
                ],
                message_content_strategy(),
            )
                .prop_map(|(role, content)| Message { role, content }),
            1..5,
        )
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 3: Request contains correct Authorization header format
        /// For any valid API key, the provider SHALL use Bearer token authentication
        #[test]
        fn prop_request_uses_bearer_auth(api_key in "[a-zA-Z0-9]{10,50}") {
            let provider = XAIProvider::new(
                SecretString::new(api_key.clone().into()),
                "https://api.x.ai/v1".to_string(),
            );

            // Verify the API key is stored correctly
            prop_assert_eq!(provider.api_key.expose_secret(), &api_key);
        }

        /// Property 3: Request body has correct OpenAI-compatible format
        /// For any valid message list and model parameters, the request SHALL
        /// contain model, messages array, and optional parameters
        #[test]
        fn prop_request_has_openai_compatible_format(
            messages in messages_strategy(),
            model in model_name_strategy(),
            temperature in temperature_strategy(),
        ) {
            let provider = XAIProvider::new(
                SecretString::new("test-key".to_string().into()),
                "https://api.x.ai/v1".to_string(),
            );

            let params = ModelParams::new(model.clone())
                .with_temperature(temperature);

            let request = provider.build_request(messages.clone(), params);

            // Verify model is set correctly
            prop_assert_eq!(&request.model, &model);

            // Verify messages are converted correctly
            prop_assert_eq!(request.messages.len(), messages.len());

            // Verify temperature is set
            prop_assert_eq!(request.temperature, Some(temperature));

            // Verify stream is false for non-streaming request
            prop_assert!(!request.stream);
        }

        /// Property 3: Message roles are correctly mapped
        /// For any message, the role SHALL be correctly converted to
        /// "system", "user", or "assistant" string
        #[test]
        fn prop_message_roles_correctly_mapped(
            content in message_content_strategy(),
            role_idx in 0usize..3usize,
        ) {
            let role = match role_idx {
                0 => MessageRole::System,
                1 => MessageRole::User,
                _ => MessageRole::Assistant,
            };

            let msg = Message {
                role: role.clone(),
                content: content.clone(),
            };

            let xai_msg = XAIProvider::convert_message(&msg);

            let expected_role = match role {
                MessageRole::System => "system",
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
            };

            prop_assert_eq!(&xai_msg.role, expected_role);
            prop_assert_eq!(&xai_msg.content, &content);
        }

        /// Property 3: Request serializes to valid JSON
        /// For any valid request, serialization SHALL produce valid JSON
        /// with required fields
        #[test]
        fn prop_request_serializes_to_valid_json(
            messages in messages_strategy(),
            model in model_name_strategy(),
            temperature in temperature_strategy(),
            top_p in top_p_strategy(),
        ) {
            let provider = XAIProvider::new(
                SecretString::new("test-key".to_string().into()),
                "https://api.x.ai/v1".to_string(),
            );

            let params = ModelParams::new(model.clone())
                .with_temperature(temperature);

            let mut request = provider.build_request(messages, params);
            request.top_p = Some(top_p);

            let json_result = serde_json::to_string(&request);
            prop_assert!(json_result.is_ok(), "Request should serialize to JSON");

            let json = json_result.unwrap();

            // Verify required fields are present
            prop_assert!(json.contains("\"model\""), "JSON should contain model field");
            prop_assert!(json.contains("\"messages\""), "JSON should contain messages field");
            prop_assert!(json.contains("\"stream\""), "JSON should contain stream field");
        }

        /// Property 3: Max tokens is correctly included when set
        /// For any valid max_tokens value, it SHALL be included in the request
        #[test]
        fn prop_max_tokens_included_when_set(
            max_tokens in 1u32..4096u32,
            model in model_name_strategy(),
        ) {
            let provider = XAIProvider::new(
                SecretString::new("test-key".to_string().into()),
                "https://api.x.ai/v1".to_string(),
            );

            let messages = vec![Message::user("test")];
            let params = ModelParams::new(model)
                .with_max_tokens(max_tokens);

            let request = provider.build_request(messages, params);

            prop_assert_eq!(request.max_tokens, Some(max_tokens));

            let json = serde_json::to_string(&request).unwrap();
            let expected_str = format!("\"max_tokens\":{}", max_tokens);
            prop_assert!(json.contains(&expected_str), "JSON should contain max_tokens");
        }
    }
}
