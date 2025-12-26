//! LLM 客户端核心模块
//!
//! 实现多厂商 LLM API 集成，包括：
//! - API Key 安全存储
//! - 统一的 LLM 服务接口
//! - 多厂商适配器 (OpenAI, Anthropic, Ollama)
//! - 客户端管理器

pub mod security;
pub mod interface;
pub mod providers;
pub mod manager;

pub use security::{ApiKeyStorage, ApiKeyValidator, APP_NAME};
pub use interface::{
    LLMService, Message, MessageRole, ModelParams,
    ChatCompletionResponse, StreamChunk, StreamHelper
};
pub use providers::{OpenAIProvider, AnthropicProvider, OllamaProvider};
pub use manager::LLMClientManager;
