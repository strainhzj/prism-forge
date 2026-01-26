//! Anthropic Claude API 提供商
//!
//! 支持 Claude 3.5, Claude 3 等模型

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

/// Anthropic API 版本
const API_VERSION: &str = "2023-06-01";

/// Anthropic 提供商客户端
pub struct AnthropicProvider {
    /// HTTP 客户端
    client: Client,

    /// API Key
    api_key: SecretString,

    /// 基础 URL
    base_url: String,

    /// API Key 引用（仅用于标识）
    _api_key_ref: Option<String>,
}

impl AnthropicProvider {
    /// 创建新的 Anthropic 提供商
    ///
    /// # 参数
    /// - `api_key`: Anthropic API Key
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

    /// 将通用 Message 转换为 Anthropic 格式
    fn convert_message(msg: Message) -> AnthropicMessage {
        match msg.role {
            MessageRole::User => AnthropicMessage {
                role: "user".to_string(),
                content: vec![ContentBlock::Text { text: msg.content }],
            },
            MessageRole::Assistant => AnthropicMessage {
                role: "assistant".to_string(),
                content: vec![ContentBlock::Text { text: msg.content }],
            },
            // Anthropic 不支持 system 消息在 messages 数组中
            // 需要通过 system 参数传递
            MessageRole::System => AnthropicMessage {
                role: "user".to_string(),
                content: vec![ContentBlock::Text {
                    text: format!("[System] {}", msg.content),
                }],
            },
        }
    }

    /// 构建请求体
    fn build_request(
        &self,
        messages: Vec<Message>,
        params: ModelParams,
    ) -> Result<AnthropicRequest> {
        // 分离系统消息
        let system_messages: Vec<String> = messages
            .iter()
            .filter(|m| m.role == MessageRole::System)
            .map(|m| m.content.clone())
            .collect();

        let system = if system_messages.is_empty() {
            None
        } else {
            Some(system_messages.join("\n\n"))
        };

        // 过滤掉系统消息，转换其他消息
        let anthropic_messages: Vec<AnthropicMessage> = messages
            .into_iter()
            .filter(|m| m.role != MessageRole::System)
            .map(Self::convert_message)
            .collect();

        Ok(AnthropicRequest {
            model: params.model,
            messages: anthropic_messages,
            system,
            max_tokens: params.max_tokens.unwrap_or(4096),
            temperature: Some(params.temperature),
            top_p: Some(params.top_p),
            stop_sequences: params.stop,
            is_stream: false,
        })
    }

    /// 发送非流式请求
    async fn send_request(&self, request: &AnthropicRequest) -> Result<AnthropicResponse> {
        let url = format!("{}/v1/messages", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("x-api-key", self.api_key.expose_secret())
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .json(request)
            .send()
            .await
            .context("发送 Anthropic API 请求失败")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Anthropic API 错误: {} - {}",
                status,
                error_text
            ));
        }

        response
            .json::<AnthropicResponse>()
            .await
            .context("解析 Anthropic API 响应失败")
    }

    /// 发送流式请求
    async fn send_stream_request(
        &self,
        request: &AnthropicRequest,
    ) -> Result<Box<dyn Stream<Item = Result<StreamChunk>> + Send + Unpin>> {
        let url = format!("{}/v1/messages", self.base_url);

        let mut stream_request = request.clone();
        stream_request.is_stream = true;

        let response = self
            .client
            .post(&url)
            .header("x-api-key", self.api_key.expose_secret())
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .json(&stream_request)
            .send()
            .await
            .context("发送 Anthropic 流式请求失败")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Anthropic API 错误: {} - {}",
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

                        if let Ok(event) = serde_json::from_str::<StreamEvent>(json_str) {
                            if event.type_ == "content_block_delta" {
                                if let Some(delta) = event.delta {
                                    return Ok(StreamChunk {
                                        delta: delta.text.unwrap_or_default(),
                                        is_finish: false,
                                        finish_reason: None,
                                    });
                                }
                            } else if event.type_ == "message_stop" {
                                return Ok(StreamChunk {
                                    delta: String::new(),
                                    is_finish: true,
                                    finish_reason: Some("stop".to_string()),
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
impl LLMService for AnthropicProvider {
    async fn chat_completion(
        &self,
        messages: Vec<Message>,
        params: ModelParams,
    ) -> Result<ChatCompletionResponse> {
        let request = self.build_request(messages, params)?;
        let response = self.send_request(&request).await?;

        let content = response
            .content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        Ok(ChatCompletionResponse {
            content,
            model: response.model,
            finish_reason: Some(response.stop_reason),
            prompt_tokens: response.usage.as_ref().map(|u| u.input_tokens),
            completion_tokens: response.usage.as_ref().map(|u| u.output_tokens),
            total_tokens: response.usage.as_ref().map(|u| u.input_tokens + u.output_tokens),
        })
    }

    async fn stream_completion(
        &self,
        messages: Vec<Message>,
        params: ModelParams,
    ) -> Result<Box<dyn Stream<Item = Result<StreamChunk>> + Send + Unpin>> {
        let request = self.build_request(messages, params)?;
        self.send_stream_request(&request).await
    }

    fn service_type(&self) -> &'static str {
        "Anthropic"
    }
}

// ========== Anthropic API 数据结构 ==========

/// Anthropic 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: Vec<ContentBlock>,
}

/// 内容块
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { source: ImageSource },
}

impl ContentBlock {
    /// 提取文本内容
    fn unwrap_text(&self) -> &str {
        match self {
            ContentBlock::Text { text } => text,
            ContentBlock::Image { .. } => "[图片]",
        }
    }
}

/// 图片源
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ImageSource {
    #[serde(rename = "type")]
    type_: String,
    media_type: String,
    data: String,
}

/// Anthropic 请求
#[derive(Debug, Clone, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    #[serde(rename = "stream")]
    is_stream: bool,
}

/// Anthropic 响应
#[derive(Debug, Clone, Deserialize)]
struct AnthropicResponse {
    id: String,
    model: String,
    content: Vec<ContentBlock>,
    stop_reason: String,
    usage: Option<Usage>,
}

/// Token 使用情况
#[derive(Debug, Clone, Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

/// 流式事件
#[derive(Debug, Clone, Deserialize)]
struct StreamEvent {
    #[serde(rename = "type")]
    type_: String,
    delta: Option<StreamDelta>,
}

/// 流式增量
#[derive(Debug, Clone, Deserialize)]
struct StreamDelta {
    text: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_message() {
        let user_msg = Message::user("Hello");
        let anthropic_msg = AnthropicProvider::convert_message(user_msg);
        assert_eq!(anthropic_msg.role, "user");

        let system_msg = Message::system("You are helpful");
        let anthropic_msg = AnthropicProvider::convert_message(system_msg);
        // 系统消息被转换为带前缀的用户消息
        assert!(anthropic_msg.content[0].clone().unwrap_text().starts_with("[System]"));
    }

    #[test]
    fn test_build_request() {
        let provider = AnthropicProvider::new(
            SecretString::new("test-key".to_string().into()),
            "https://api.anthropic.com".to_string(),
        ).unwrap();

        let messages = vec![
            Message::system("You are helpful"),
            Message::user("Hello"),
        ];
        let params = ModelParams::new("claude-3-5-sonnet-20241022");

        let request = provider.build_request(messages, params);
        assert!(request.is_ok());

        let request = request.unwrap();
        assert_eq!(request.model, "claude-3-5-sonnet-20241022");
        assert!(request.system.is_some());
        assert_eq!(request.messages.len(), 1); // 只有用户消息
    }
}
