//! OpenAI 兼容 API 提供商
//!
//! 支持 OpenAI 官方 API 及兼容接口（OneAPI、中转服务等）

use anyhow::{Context, Result};
use async_trait::async_trait;
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestUserMessage, ChatCompletionRequestAssistantMessage,
        ChatCompletionRequestSystemMessageContent,
        ChatCompletionRequestUserMessageContent,
        ChatCompletionRequestAssistantMessageContent,
        CreateChatCompletionRequest, CreateChatCompletionRequestArgs,
        FinishReason,
    },
    Client,
};
use secrecy::{ExposeSecret, SecretString};
use futures::StreamExt;

use crate::llm::interface::{
    LLMService, Message, MessageRole, ModelParams, ChatCompletionResponse,
    StreamChunk,
};

/// OpenAI 提供商客户端
pub struct OpenAIProvider {
    /// OpenAI 客户端
    client: Client<OpenAIConfig>,

    /// 基础 URL（用于自定义 endpoint）
    base_url: String,

    /// API Key 引用（仅用于标识）
    _api_key_ref: Option<String>,
}

impl OpenAIProvider {
    /// 创建新的 OpenAI 提供商
    ///
    /// # 参数
    /// - `api_key`: OpenAI API Key
    /// - `base_url`: API 基础 URL
    ///
    /// # 示例
    /// ```no_run
    /// use secrecy::SecretString;
    ///
    /// let api_key = SecretString::new("sk-...".to_string());
    /// let provider = OpenAIProvider::new(api_key, "https://api.openai.com/v1".to_string());
    /// ```
    pub fn new(api_key: SecretString, base_url: String) -> Self {
        let config = OpenAIConfig::default()
            .with_api_key(api_key.expose_secret())
            .with_api_base(&base_url);

        let client = Client::with_config(config);

        Self {
            client,
            base_url,
            _api_key_ref: None,
        }
    }

    /// 使用 API Key 引用创建提供商
    ///
    /// # 参数
    /// - `api_key`: API Key
    /// - `base_url`: API 基础 URL
    /// - `api_key_ref`: API Key 引用标识
    pub fn with_ref(api_key: SecretString, base_url: String, api_key_ref: String) -> Self {
        let config = OpenAIConfig::default()
            .with_api_key(api_key.expose_secret())
            .with_api_base(&base_url);

        let client = Client::with_config(config);

        Self {
            client,
            base_url,
            _api_key_ref: Some(api_key_ref),
        }
    }

    /// 将通用 Message 转换为 OpenAI 格式
    fn convert_messages(messages: Vec<Message>) -> Vec<ChatCompletionRequestMessage> {
        messages
            .into_iter()
            .map(|msg| match msg.role {
                MessageRole::System => {
                    ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                        content: ChatCompletionRequestSystemMessageContent::Text(msg.content),
                        name: None,
                    })
                }
                MessageRole::User => {
                    ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                        content: ChatCompletionRequestUserMessageContent::Text(msg.content),
                        name: None,
                    })
                }
                MessageRole::Assistant => {
                    ChatCompletionRequestMessage::Assistant(ChatCompletionRequestAssistantMessage {
                        content: Some(ChatCompletionRequestAssistantMessageContent::Text(msg.content)),
                        name: None,
                        tool_calls: None,
                        refusal: None,
                        function_call: None,
                    })
                }
            })
            .collect()
    }

    /// 将通用参数转换为 OpenAI 请求
    fn build_request(
        &self,
        messages: Vec<Message>,
        params: ModelParams,
    ) -> Result<CreateChatCompletionRequest> {
        let openai_messages = Self::convert_messages(messages);

        let mut builder = CreateChatCompletionRequestArgs::default();
        builder
            .model(&params.model)
            .messages(openai_messages)
            .temperature(params.temperature)
            .top_p(params.top_p);

        // 可选参数
        if let Some(max_tokens) = params.max_tokens {
            builder.max_tokens(max_tokens);
        }

        if let Some(stop) = params.stop {
            builder.stop(stop);
        }

        // 处理额外参数（例如 presence_penalty, frequency_penalty）
        if let Some(extra) = params.extra {
            if let Some(presence_penalty) = extra.get("presence_penalty").and_then(|v| v.as_f64()) {
                builder.presence_penalty(presence_penalty as f32);
            }
            if let Some(frequency_penalty) = extra.get("frequency_penalty").and_then(|v| v.as_f64()) {
                builder.frequency_penalty(frequency_penalty as f32);
            }
        }

        builder
            .build()
            .context("创建 OpenAI 请求失败")
    }

    /// 转换 finish reason
    fn convert_finish_reason(reason: &Option<FinishReason>) -> Option<String> {
        reason.as_ref().map(|r| match r {
            FinishReason::Stop => "stop".to_string(),
            FinishReason::Length => "length".to_string(),
            FinishReason::ContentFilter => "content_filter".to_string(),
            FinishReason::ToolCalls => "tool_calls".to_string(),
            _ => "unknown".to_string(),
        })
    }
}

#[async_trait]
impl LLMService for OpenAIProvider {
    async fn chat_completion(
        &self,
        messages: Vec<Message>,
        params: ModelParams,
    ) -> Result<ChatCompletionResponse> {
        let request = self.build_request(messages, params)?;

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .context("OpenAI API 请求失败")?;

        let choice = response
            .choices
            .first()
            .context("OpenAI 返回空响应")?;

        let content = choice
            .message
            .content
            .clone()
            .unwrap_or_default();

        let usage = response.usage.as_ref();

        Ok(ChatCompletionResponse {
            content,
            model: response.model,
            finish_reason: Self::convert_finish_reason(&choice.finish_reason),
            prompt_tokens: usage.map(|u| u.prompt_tokens as u32),
            completion_tokens: usage.map(|u| u.completion_tokens as u32),
            total_tokens: usage.map(|u| u.total_tokens as u32),
        })
    }

    async fn stream_completion(
        &self,
        messages: Vec<Message>,
        params: ModelParams,
    ) -> Result<Box<dyn futures::Stream<Item = Result<StreamChunk>> + Send + Unpin>> {
        let request = self.build_request(messages, params)?;

        let stream = self
            .client
            .chat()
            .create_stream(request)
            .await
            .context("OpenAI 流式请求失败")?;

        Ok(Box::new(stream.map(move |chunk_result| {
            match chunk_result {
                Ok(chunk) => {
                    let deltas = chunk.choices.first();

                    let delta = deltas
                        .and_then(|c| c.delta.content.as_ref())
                        .cloned()
                        .unwrap_or_default();

                    let is_finish = deltas.map_or(false, |c| {
                        c.finish_reason.is_some()
                    });

                    let finish_reason = if is_finish {
                        deltas.and_then(|c| Self::convert_finish_reason(&c.finish_reason))
                    } else {
                        None
                    };

                    Ok(StreamChunk {
                        delta,
                        is_finish,
                        finish_reason,
                    })
                }
                Err(e) => Err(anyhow::anyhow!("流式响应错误: {}", e)),
            }
        })))
    }

    fn service_type(&self) -> &'static str {
        "OpenAI"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_messages() {
        let messages = vec![
            Message::system("You are helpful"),
            Message::user("Hello"),
            Message::assistant("Hi there"),
        ];

        let openai_messages = OpenAIProvider::convert_messages(messages);
        assert_eq!(openai_messages.len(), 3);
    }

    #[test]
    fn test_build_request() {
        let provider = OpenAIProvider::new(
            SecretString::new("test-key".to_string().into()),
            "https://api.openai.com/v1".to_string(),
        );

        let messages = vec![Message::user("test")];
        let params = ModelParams::new("gpt-3.5-turbo");

        let request = provider.build_request(messages, params);
        assert!(request.is_ok());
    }
}
