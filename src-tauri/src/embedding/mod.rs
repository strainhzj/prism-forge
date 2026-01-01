//! 向量嵌入模块
//!
//! 使用 FastEmbed 生成消息摘要的向量表示，用于语义相似度检索

pub mod generator;

pub use generator::EmbeddingGenerator;
