//! Ollama 本地 LLM 服务提供商
//!
//! 支持本地运行的 Ollama 服务，无需 API Key

use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::llm::interface::{
    LLMService, Message, MessageRole, ModelParams, ChatCompletionResponse,
    StreamChunk,
};

/// Ollama 默认地址
pub const OLLAMA_DEFAULT_URL: &str = "http://127.0.0.1:11434";

/// Ollama 提供商客户端
pub struct OllamaProvider {
    /// HTTP 客户端
    client: Client,

    /// 基础 URL
    base_url: String,
}

impl OllamaProvider {
    /// 创建新的 Ollama 提供商
    ///
    /// # 参数
    /// - `base_url`: Ollama 服务地址（默认 http://127.0.0.1:11434）
    pub fn new(base_url: Option<String>) -> Self {
        let client = Client::builder()
            .build()
            .expect("创建 HTTP 客户端失败");

        Self {
            client,
            base_url: base_url.unwrap_or_else(|| OLLAMA_DEFAULT_URL.to_string()),
        }
    }

    /// 获取默认地址的提供商
    pub fn default() -> Self {
        Self::new(None)
    }

    /// 将通用 Message 转换为 Ollama 格式
    fn convert_message(msg: Message) -> OllamaMessage {
        OllamaMessage {
            role: match msg.role {
                MessageRole::System => "system".to_string(),
                MessageRole::User => "user".to_string(),
                MessageRole::Assistant => "assistant".to_string(),
            },
            content: msg.content,
        }
    }

    /// 构建请求体
    fn build_request(
        &self,
        messages: Vec<Message>,
        params: ModelParams,
    ) -> Result<OllamaRequest> {
        let ollama_messages: Vec<OllamaMessage> = messages
            .into_iter()
            .map(Self::convert_message)
            .collect();

        Ok(OllamaRequest {
            model: params.model,
            messages: ollama_messages,
            stream: false,
            options: Some(OllamaOptions {
                temperature: params.temperature,
                top_p: params.top_p,
                num_predict: params.max_tokens,
                stop: params.stop,
            }),
        })
    }

    /// 发送非流式请求
    async fn send_request(&self, request: &OllamaRequest) -> Result<OllamaResponse> {
        let url = format!("{}/v1/chat/completions", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(request)
            .send()
            .await
            .context("发送 Ollama API 请求失败")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Ollama API 错误: {} - {}",
                status,
                error_text
            ));
        }

        response
            .json::<OllamaResponse>()
            .await
            .context("解析 Ollama API 响应失败")
    }

    /// 发送流式请求
    async fn send_stream_request(
        &self,
        request: &OllamaRequest,
    ) -> Result<Box<dyn futures::Stream<Item = Result<StreamChunk>> + Send + Unpin>> {
        let mut stream_request = request.clone();
        stream_request.stream = true;

        let url = format!("{}/v1/chat/completions", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&stream_request)
            .send()
            .await
            .context("发送 Ollama 流式请求失败")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Ollama API 错误: {} - {}",
                status,
                error_text
            ));
        }

        // Ollama 流式响应格式与 OpenAI 兼容
        let stream = response.bytes_stream().map(|chunk_result| match chunk_result {
            Ok(chunk) => {
                let text = String::from_utf8_lossy(&chunk);
                // 解析 SSE 格式（Ollama 使用 OpenAI 兼容格式）
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

                        if let Ok(resp) = serde_json::from_str::<OllamaStreamResponse>(json_str) {
                            if let Some(choice) = resp.choices.first() {
                                let delta = choice.delta.content.clone().unwrap_or_default();
                                let finish_reason = choice.finish_reason.clone();

                                return Ok(StreamChunk {
                                    delta,
                                    is_finish: finish_reason.is_some(),
                                    finish_reason,
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
impl LLMService for OllamaProvider {
    async fn chat_completion(
        &self,
        messages: Vec<Message>,
        params: ModelParams,
    ) -> Result<ChatCompletionResponse> {
        let request = self.build_request(messages, params)?;
        let response = self.send_request(&request).await?;

        let content = response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(ChatCompletionResponse {
            content,
            model: response.model,
            finish_reason: response
                .choices
                .first()
                .and_then(|c| c.finish_reason.clone()),
            prompt_tokens: response.usage.as_ref().map(|u| u.prompt_tokens),
            completion_tokens: response.usage.as_ref().map(|u| u.completion_tokens),
            total_tokens: response.usage.as_ref().map(|u| u.total_tokens),
        })
    }

    async fn stream_completion(
        &self,
        messages: Vec<Message>,
        params: ModelParams,
    ) -> Result<Box<dyn futures::Stream<Item = Result<StreamChunk>> + Send + Unpin>> {
        let request = self.build_request(messages, params)?;
        self.send_stream_request(&request).await
    }

    fn service_type(&self) -> &'static str {
        "Ollama"
    }
}

// ========== Ollama API 数据结构 ==========

/// Ollama 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

/// Ollama 选项
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaOptions {
    temperature: f32,
    top_p: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

/// Ollama 请求
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

/// Ollama 消息内容
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaMessageContent {
    content: String,
}

/// Ollama 选择
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaChoice {
    message: OllamaMessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    finish_reason: Option<String>,
}

/// Ollama 使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// Ollama 响应
#[derive(Debug, Clone, Deserialize)]
struct OllamaResponse {
    id: String,
    model: String,
    choices: Vec<OllamaChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    usage: Option<OllamaUsage>,
}

/// Ollama 流式 Delta
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
}

/// Ollama 流式选择
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaStreamChoice {
    delta: OllamaDelta,
    #[serde(skip_serializing_if = "Option::is_none")]
    finish_reason: Option<String>,
}

/// Ollama 流式响应
#[derive(Debug, Clone, Deserialize)]
struct OllamaStreamResponse {
    id: String,
    model: String,
    choices: Vec<OllamaStreamChoice>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_message() {
        let user_msg = Message::user("Hello");
        let ollama_msg = OllamaProvider::convert_message(user_msg);
        assert_eq!(ollama_msg.role, "user");
        assert_eq!(ollama_msg.content, "Hello");

        let system_msg = Message::system("You are helpful");
        let ollama_msg = OllamaProvider::convert_message(system_msg);
        assert_eq!(ollama_msg.role, "system");
    }

    #[test]
    fn test_build_request() {
        let provider = OllamaProvider::default();

        let messages = vec![
            Message::system("You are helpful"),
            Message::user("Hello"),
        ];
        let params = ModelParams::new("llama3");

        let request = provider.build_request(messages, params);
        assert!(request.is_ok());

        let request = request.unwrap();
        assert_eq!(request.model, "llama3");
        assert_eq!(request.messages.len(), 2);
    }

    #[test]
    fn test_default_url() {
        let provider = OllamaProvider::default();
        assert_eq!(provider.base_url, OLLAMA_DEFAULT_URL);
    }
}
