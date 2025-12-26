//! LLM 服务统一接口
//!
//! 定义跨厂商的 LLM 服务抽象层，统一 OpenAI、Anthropic、Ollama 等不同 API 的调用方式

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 聊天消息角色
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// 系统提示词（某些提供商支持）
    System,
    /// 用户消息
    User,
    /// 助手回复
    Assistant,
}

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// 消息角色
    pub role: MessageRole,
    /// 消息内容
    pub content: String,
}

impl Message {
    /// 创建用户消息
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
        }
    }

    /// 创建助手消息
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
        }
    }

    /// 创建系统消息
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
        }
    }
}

/// 模型参数
///
/// 统一不同 LLM 提供商的参数配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelParams {
    /// 模型名称（例如：gpt-4, claude-3-5-sonnet, llama3）
    pub model: String,

    /// 采样温度 (0.0 - 2.0)
    ///
    /// - 0.0: 更确定性，适合事实性回答
    /// - 1.0: 平衡
    /// - 2.0: 更随机创造性
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Top-P 采样 (0.0 - 1.0)
    ///
    /// 考虑概率质量最高的 P 个 tokens
    #[serde(default = "default_top_p")]
    pub top_p: f32,

    /// 最大输出 token 数
    ///
    /// None 表示使用模型默认值
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// 停止序列
    ///
    /// 当生成这些字符串时停止输出
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,

    /// 额外参数（用于提供商特定配置）
    ///
    /// 例如：OpenAI 的 `presence_penalty`, `frequency_penalty`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

fn default_temperature() -> f32 {
    0.7
}

fn default_top_p() -> f32 {
    1.0
}

impl ModelParams {
    /// 创建新的模型参数
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            temperature: default_temperature(),
            top_p: default_top_p(),
            max_tokens: None,
            stop: None,
            extra: None,
        }
    }

    /// 设置温度
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// 设置最大 token 数
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// 设置停止序列
    pub fn with_stop(mut self, stop: Vec<String>) -> Self {
        self.stop = Some(stop);
        self
    }

    /// 设置额外参数
    pub fn with_extra(mut self, extra: serde_json::Value) -> Self {
        self.extra = Some(extra);
        self
    }
}

/// 聊天完成响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    /// 生成的消息内容
    pub content: String,

    /// 使用的模型名称
    pub model: String,

    /// 完成的理由（例如：stop, length）
    pub finish_reason: Option<String>,

    /// 使用的输入 token 数
    pub prompt_tokens: Option<u32>,

    /// 生成的输出 token 数
    pub completion_tokens: Option<u32>,

    /// 总 token 数
    pub total_tokens: Option<u32>,
}

/// 流式响应的块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    /// 增量内容
    pub delta: String,

    /// 是否为最后一个块
    pub is_finish: bool,

    /// 完成理由（仅最后一个块包含）
    pub finish_reason: Option<String>,
}

/// LLM 服务 Trait
///
/// 定义所有 LLM 提供商必须实现的统一接口
#[async_trait]
pub trait LLMService: Send + Sync {
    /// 非流式聊天完成
    ///
    /// # 参数
    /// - `messages`: 消息列表
    /// - `params`: 模型参数
    ///
    /// # 返回
    /// 返回完整的聊天完成响应
    ///
    /// # 示例
    /// ```no_run
    /// use crate::llm::interface::{Message, ModelParams, LLMService};
    ///
    /// let messages = vec![
    ///     Message::user("你好，请介绍一下自己"),
    /// ];
    /// let params = ModelParams::new("gpt-3.5-turbo");
    /// let response = service.chat_completion(messages, params).await?;
    /// println!("回复: {}", response.content);
    /// ```
    async fn chat_completion(
        &self,
        messages: Vec<Message>,
        params: ModelParams,
    ) -> Result<ChatCompletionResponse>;

    /// 流式聊天完成
    ///
    /// # 参数
    /// - `messages`: 消息列表
    /// - `params`: 模型参数
    ///
    /// # 返回
    /// 返回流式响应的异步迭代器
    ///
    /// # 示例
    /// ```no_run
    /// use crate::llm::interface::{Message, ModelParams, LLMService};
    /// use futures::stream::StreamExt;
    ///
    /// let messages = vec![
    ///     Message::user("写一首关于 Rust 的诗"),
    /// ];
    /// let params = ModelParams::new("gpt-4");
    ///
    /// let mut stream = service.stream_completion(messages, params).await?;
    /// while let Some(chunk) = stream.next().await {
    ///     print!("{}", chunk?.delta);
    /// }
    /// ```
    async fn stream_completion(
        &self,
        messages: Vec<Message>,
        params: ModelParams,
    ) -> Result<Box<dyn futures::Stream<Item = Result<StreamChunk>> + Send + Unpin>>;

    /// 测试连接是否有效
    ///
    /// 发送一个简单的请求验证配置是否正确
    async fn test_connection(&self) -> Result<bool> {
        let messages = vec![Message::user("test")];
        let params = ModelParams::new("test")
            .with_max_tokens(5)
            .with_temperature(0.0);

        match self.chat_completion(messages, params).await {
            Ok(_) => Ok(true),
            Err(e) => {
                // 某些错误是可接受的（如 test 模型不存在）
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("model")
                    || error_msg.contains("404")
                    || error_msg.contains("not found")
                {
                    Ok(true)
                } else {
                    Err(e)
                }
            }
        }
    }

    /// 获取服务类型（用于调试）
    fn service_type(&self) -> &'static str {
        "LLMService"
    }
}

/// 流式输出帮助器
///
/// 用于将异步流转换为回调函数模式
pub struct StreamHelper;

impl StreamHelper {
    /// 消费流并调用回调函数
    ///
    /// # 参数
    /// - `stream`: 流式响应
    /// - `callback`: 每收到一个块时调用的回调
    pub async fn consume_with_callback<F>(
        mut stream: Box<dyn futures::Stream<Item = Result<StreamChunk>> + Send + Unpin>,
        mut callback: F,
    ) -> Result<ChatCompletionResponse>
    where
        F: FnMut(&StreamChunk) -> anyhow::Result<()>,
    {
        let mut full_content = String::new();
        let mut finish_reason = None;

        use futures::StreamExt;
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            callback(&chunk)?;

            full_content.push_str(&chunk.delta);
            if chunk.is_finish {
                finish_reason = chunk.finish_reason;
            }
        }

        Ok(ChatCompletionResponse {
            content: full_content,
            model: "stream".to_string(),
            finish_reason,
            prompt_tokens: None,
            completion_tokens: None,
            total_tokens: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let user_msg = Message::user("Hello");
        assert_eq!(user_msg.role, MessageRole::User);
        assert_eq!(user_msg.content, "Hello");

        let system_msg = Message::system("You are helpful");
        assert_eq!(system_msg.role, MessageRole::System);
    }

    #[test]
    fn test_model_params_builder() {
        let params = ModelParams::new("gpt-4")
            .with_temperature(0.5)
            .with_max_tokens(1000)
            .with_stop(vec!["END".to_string()]);

        assert_eq!(params.model, "gpt-4");
        assert_eq!(params.temperature, 0.5);
        assert_eq!(params.max_tokens, Some(1000));
        assert_eq!(params.stop, Some(vec!["END".to_string()]));
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::user("test");
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, r#"{"role":"user","content":"test"}"#);
    }

    #[test]
    fn test_model_params_serialization() {
        let params = ModelParams::new("gpt-3.5-turbo")
            .with_temperature(0.8);
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains(r#""model":"gpt-3.5-turbo""#));
        assert!(json.contains(r#""temperature":0.8"#));
    }
}
