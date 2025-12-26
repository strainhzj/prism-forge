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

/// 连接错误类型
///
/// 用于分类连接测试失败的原因
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionErrorType {
    /// 认证错误（API Key 无效）
    Authentication,
    /// 网络错误（无法连接）
    Network,
    /// 服务器错误（500 等）
    Server,
    /// 请求错误（模型不存在、参数错误等）
    Request,
    /// 未知错误
    Unknown,
}

/// 连接测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConnectionResult {
    /// 是否成功
    pub success: bool,
    /// 错误信息（失败时）
    pub error_message: Option<String>,
    /// 错误类型
    pub error_type: Option<ConnectionErrorType>,
}

impl TestConnectionResult {
    /// 创建成功结果
    pub fn success() -> Self {
        Self {
            success: true,
            error_message: None,
            error_type: None,
        }
    }

    /// 创建失败结果
    pub fn failure(error_type: ConnectionErrorType, message: String) -> Self {
        Self {
            success: false,
            error_message: Some(message),
            error_type: Some(error_type),
        }
    }
}

/// 分类错误信息
///
/// 根据错误消息内容判断错误类型并返回用户友好的错误描述
pub fn categorize_error(error: &str) -> (ConnectionErrorType, String) {
    let error_lower = error.to_lowercase();

    // 认证错误
    if error_lower.contains("unauthorized")
        || error_lower.contains("401")
        || error_lower.contains("403")
        || error_lower.contains("invalid api key")
        || error_lower.contains("invalid_api_key")
        || error_lower.contains("authentication")
        || error_lower.contains("api key")
        || error_lower.contains("apikey")
        || error_lower.contains("permission")
        || error_lower.contains("access denied")
    {
        return (
            ConnectionErrorType::Authentication,
            "API Key 无效或已过期".to_string(),
        );
    }

    // 服务器错误 (检查在网络错误之前，因为 504 包含 "timeout")
    if error_lower.contains("500")
        || error_lower.contains("502")
        || error_lower.contains("503")
        || error_lower.contains("504")
        || error_lower.contains("internal server")
    {
        return (
            ConnectionErrorType::Server,
            "服务器内部错误".to_string(),
        );
    }

    // 网络错误
    if error_lower.contains("connection")
        || error_lower.contains("timeout")
        || error_lower.contains("network")
        || error_lower.contains("dns")
        || error_lower.contains("resolve")
        || error_lower.contains("unreachable")
    {
        return (
            ConnectionErrorType::Network,
            "无法连接到服务器，请检查网络和 Base URL".to_string(),
        );
    }

    // 请求错误（包括中文错误信息）
    if error_lower.contains("请求失败")
        || error_lower.contains("model")
        || error_lower.contains("rate limit")
        || error_lower.contains("429")
        || error_lower.contains("bad request")
        || error_lower.contains("400")
    {
        return (
            ConnectionErrorType::Request,
            format!("请求失败: {}", error),
        );
    }

    // 未知错误
    (
        ConnectionErrorType::Unknown,
        format!("连接测试失败: {}", error),
    )
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

    /// 测试连接是否有效（使用指定模型）
    ///
    /// 发送一个简单的请求验证配置是否正确
    /// 返回 TestConnectionResult 包含详细的成功/失败信息
    ///
    /// # 参数
    /// - `model`: 要测试的模型名称
    async fn test_connection_with_model(&self, model: &str) -> Result<TestConnectionResult> {
        // 使用一个简单的测试消息
        let messages = vec![Message::user("Hi")];
        // 使用指定的模型进行测试
        let params = ModelParams::new(model)
            .with_max_tokens(5)
            .with_temperature(0.0);

        #[cfg(debug_assertions)]
        eprintln!("[LLMService] test_connection_with_model: sending test request with model '{}'...", model);

        match self.chat_completion(messages, params).await {
            Ok(response) => {
                #[cfg(debug_assertions)]
                eprintln!("[LLMService] test_connection_with_model: success, response: {:?}", response.content);
                Ok(TestConnectionResult::success())
            }
            Err(e) => {
                let error_msg = e.to_string();
                #[cfg(debug_assertions)]
                eprintln!("[LLMService] test_connection_with_model: error: {}", error_msg);

                // 分析错误类型，所有错误都返回失败
                let (error_type, message) = categorize_error(&error_msg);
                Ok(TestConnectionResult::failure(error_type, message))
            }
        }
    }

    /// 测试连接是否有效（使用默认模型）
    ///
    /// 发送一个简单的请求验证配置是否正确
    /// 返回 TestConnectionResult 包含详细的成功/失败信息
    /// 
    /// 注意：此方法使用 gpt-3.5-turbo 作为默认测试模型
    /// 如需使用配置的模型，请使用 test_connection_with_model
    async fn test_connection(&self) -> Result<TestConnectionResult> {
        // 使用 gpt-3.5-turbo 作为默认测试模型
        self.test_connection_with_model("gpt-3.5-turbo").await
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

    // Unit tests for error classification
    #[test]
    fn test_categorize_auth_errors() {
        let auth_errors = [
            "unauthorized",
            "401 Unauthorized",
            "403 Forbidden",
            "invalid api key",
            "invalid_api_key",
            "authentication failed",
            "api key is invalid",
            "apikey error",
            "permission denied",
            "access denied",
        ];

        for error in auth_errors {
            let (error_type, _) = categorize_error(error);
            assert_eq!(
                error_type,
                ConnectionErrorType::Authentication,
                "Expected Authentication for: {}",
                error
            );
        }
    }

    #[test]
    fn test_categorize_network_errors() {
        let network_errors = [
            "connection refused",
            "timeout",
            "network error",
            "dns resolution failed",
            "could not resolve host",
            "host unreachable",
        ];

        for error in network_errors {
            let (error_type, _) = categorize_error(error);
            assert_eq!(
                error_type,
                ConnectionErrorType::Network,
                "Expected Network for: {}",
                error
            );
        }
    }

    #[test]
    fn test_categorize_server_errors() {
        let server_errors = [
            "500 Internal Server Error",
            "502 Bad Gateway",
            "503 Service Unavailable",
            "504 Gateway Timeout",
            "internal server error",
        ];

        for error in server_errors {
            let (error_type, _) = categorize_error(error);
            assert_eq!(
                error_type,
                ConnectionErrorType::Server,
                "Expected Server for: {}",
                error
            );
        }
    }

    #[test]
    fn test_categorize_request_errors() {
        let request_errors = [
            "请求失败",
            "model not found",
            "rate limit exceeded",
            "429 Too Many Requests",
            "bad request",
            "400 Bad Request",
        ];

        for error in request_errors {
            let (error_type, _) = categorize_error(error);
            assert_eq!(
                error_type,
                ConnectionErrorType::Request,
                "Expected Request for: {}",
                error
            );
        }
    }

    #[test]
    fn test_categorize_unknown_errors() {
        let unknown_errors = [
            "some random error",
            "unexpected issue",
            "未知错误",
        ];

        for error in unknown_errors {
            let (error_type, _) = categorize_error(error);
            assert_eq!(
                error_type,
                ConnectionErrorType::Unknown,
                "Expected Unknown for: {}",
                error
            );
        }
    }

    #[test]
    fn test_connection_result_success() {
        let result = TestConnectionResult::success();
        assert!(result.success);
        assert!(result.error_message.is_none());
        assert!(result.error_type.is_none());
    }

    #[test]
    fn test_connection_result_failure() {
        let result = TestConnectionResult::failure(
            ConnectionErrorType::Authentication,
            "API Key 无效".to_string(),
        );
        assert!(!result.success);
        assert_eq!(result.error_message, Some("API Key 无效".to_string()));
        assert_eq!(result.error_type, Some(ConnectionErrorType::Authentication));
    }
}

/// Property-based tests for error classification
/// Feature: fix-provider-connection-test, Property 1: Error Classification Correctness
/// Validates: Requirements 1.2, 1.3, 1.4, 1.5, 1.6
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Strategy to generate authentication error messages
    fn auth_error_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("unauthorized".to_string()),
            Just("401".to_string()),
            Just("403".to_string()),
            Just("invalid api key".to_string()),
            Just("invalid_api_key".to_string()),
            Just("authentication".to_string()),
            Just("api key".to_string()),
            Just("apikey".to_string()),
            Just("permission".to_string()),
            Just("access denied".to_string()),
            // With random prefix/suffix
            "[a-z]{0,10}".prop_map(|s| format!("{}unauthorized", s)),
            "[a-z]{0,10}".prop_map(|s| format!("401{}", s)),
            "[a-z]{0,10}".prop_map(|s| format!("{}authentication{}", s, s)),
        ]
    }

    // Strategy to generate network error messages
    fn network_error_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("connection".to_string()),
            Just("timeout".to_string()),
            Just("network".to_string()),
            Just("dns".to_string()),
            Just("resolve".to_string()),
            Just("unreachable".to_string()),
            // With random prefix/suffix
            "[a-z]{0,10}".prop_map(|s| format!("{}connection{}", s, s)),
            "[a-z]{0,10}".prop_map(|s| format!("{}timeout", s)),
        ]
    }

    // Strategy to generate server error messages
    fn server_error_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("500".to_string()),
            Just("502".to_string()),
            Just("503".to_string()),
            Just("504".to_string()),
            Just("internal server".to_string()),
            // With random prefix/suffix
            "[a-z]{0,10}".prop_map(|s| format!("{}500{}", s, s)),
            "[a-z]{0,10}".prop_map(|s| format!("internal server{}", s)),
        ]
    }

    // Strategy to generate request error messages
    fn request_error_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("请求失败".to_string()),
            Just("model".to_string()),
            Just("rate limit".to_string()),
            Just("429".to_string()),
            Just("bad request".to_string()),
            Just("400".to_string()),
            // With random prefix/suffix
            "[a-z]{0,10}".prop_map(|s| format!("{}model{}", s, s)),
            "[a-z]{0,10}".prop_map(|s| format!("{}429", s)),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 1: Authentication errors are correctly classified
        /// For any error message containing authentication keywords,
        /// categorize_error SHALL return ConnectionErrorType::Authentication
        #[test]
        fn prop_auth_errors_classified_correctly(error in auth_error_strategy()) {
            let (error_type, message) = categorize_error(&error);
            prop_assert_eq!(
                error_type,
                ConnectionErrorType::Authentication,
                "Auth error '{}' should be classified as Authentication, got message: {}",
                error,
                message
            );
        }

        /// Property 1: Network errors are correctly classified
        /// For any error message containing network keywords,
        /// categorize_error SHALL return ConnectionErrorType::Network
        #[test]
        fn prop_network_errors_classified_correctly(error in network_error_strategy()) {
            let (error_type, message) = categorize_error(&error);
            prop_assert_eq!(
                error_type,
                ConnectionErrorType::Network,
                "Network error '{}' should be classified as Network, got message: {}",
                error,
                message
            );
        }

        /// Property 1: Server errors are correctly classified
        /// For any error message containing server error keywords,
        /// categorize_error SHALL return ConnectionErrorType::Server
        #[test]
        fn prop_server_errors_classified_correctly(error in server_error_strategy()) {
            let (error_type, message) = categorize_error(&error);
            prop_assert_eq!(
                error_type,
                ConnectionErrorType::Server,
                "Server error '{}' should be classified as Server, got message: {}",
                error,
                message
            );
        }

        /// Property 1: Request errors are correctly classified
        /// For any error message containing request error keywords,
        /// categorize_error SHALL return ConnectionErrorType::Request
        #[test]
        fn prop_request_errors_classified_correctly(error in request_error_strategy()) {
            let (error_type, message) = categorize_error(&error);
            prop_assert_eq!(
                error_type,
                ConnectionErrorType::Request,
                "Request error '{}' should be classified as Request, got message: {}",
                error,
                message
            );
        }

        /// Property 1: Error classification is deterministic
        /// For any error message, calling categorize_error twice
        /// SHALL return the same result
        #[test]
        fn prop_error_classification_deterministic(error in ".*") {
            let (type1, msg1) = categorize_error(&error);
            let (type2, msg2) = categorize_error(&error);
            prop_assert_eq!(type1, type2, "Error type should be deterministic");
            prop_assert_eq!(msg1, msg2, "Error message should be deterministic");
        }

        /// Property 1: Error classification always returns a valid type
        /// For any error message, categorize_error SHALL return
        /// one of the five valid ConnectionErrorType variants
        #[test]
        fn prop_error_classification_returns_valid_type(error in ".*") {
            let (error_type, _) = categorize_error(&error);
            let is_valid = matches!(
                error_type,
                ConnectionErrorType::Authentication
                    | ConnectionErrorType::Network
                    | ConnectionErrorType::Server
                    | ConnectionErrorType::Request
                    | ConnectionErrorType::Unknown
            );
            prop_assert!(is_valid, "Error type should be a valid variant");
        }

        /// Property 1: Failure results always have error details
        /// For any TestConnectionResult with success=false,
        /// error_message SHALL be non-empty and error_type SHALL be set
        #[test]
        fn prop_failure_result_has_details(
            error_type in prop_oneof![
                Just(ConnectionErrorType::Authentication),
                Just(ConnectionErrorType::Network),
                Just(ConnectionErrorType::Server),
                Just(ConnectionErrorType::Request),
                Just(ConnectionErrorType::Unknown),
            ],
            message in ".+"
        ) {
            let result = TestConnectionResult::failure(error_type.clone(), message.clone());
            prop_assert!(!result.success, "Failure result should have success=false");
            prop_assert!(result.error_message.is_some(), "Failure result should have error_message");
            prop_assert!(result.error_type.is_some(), "Failure result should have error_type");
            prop_assert_eq!(result.error_type.unwrap(), error_type);
            prop_assert_eq!(result.error_message.unwrap(), message);
        }

        /// Property 1: Success results have no error details
        /// For any TestConnectionResult with success=true,
        /// error_message SHALL be None and error_type SHALL be None
        #[test]
        fn prop_success_result_has_no_error_details(_dummy in 0..100i32) {
            let result = TestConnectionResult::success();
            prop_assert!(result.success, "Success result should have success=true");
            prop_assert!(result.error_message.is_none(), "Success result should have no error_message");
            prop_assert!(result.error_type.is_none(), "Success result should have no error_type");
        }
    }
}
