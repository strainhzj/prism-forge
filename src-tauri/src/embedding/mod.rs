//! 向量嵌入模块
//!
//! 支持多种 Embedding 提供商：
//! - OpenAI: 使用 OpenAI API 生成向量（需要 API Key）
//! - FastEmbed: 本地模型生成向量（Windows 编译问题待修复）

pub mod generator;
pub mod openai_embeddings;
pub mod sync;

pub use generator::EmbeddingGenerator;
pub use openai_embeddings::OpenAIEmbeddings;
pub use sync::{EmbeddingSyncManager, SyncConfig};

/// Embedding 提供商枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingProvider {
    /// OpenAI API（需要 API Key）
    OpenAI,

    /// FastEmbed 本地模型（Windows 编译问题待修复）
    FastEmbed,
}

impl EmbeddingProvider {
    /// 从字符串解析提供商
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "openai" => Ok(Self::OpenAI),
            "fastembed" => Ok(Self::FastEmbed),
            _ => Err(format!("未知的 embedding 提供商: {}", s)),
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenAI => "openai",
            Self::FastEmbed => "fastembed",
        }
    }
}
