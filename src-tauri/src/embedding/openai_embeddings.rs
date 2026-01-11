//! OpenAI Embeddings 客户端
//!
//! 使用 OpenAI API 生成文本向量（text-embedding-3-small）
//! 向量维度: 1536
//! 费用: $0.00002 / 1K tokens

use anyhow::{Context, Result};
use async_openai::{
    config::{OpenAIConfig, Config},
    types::{
        CreateEmbeddingRequestArgs, EmbeddingInput,
    },
    Client,
};
use std::sync::Arc;

/// OpenAI Embeddings 生成器
///
/// 使用 OpenAI API 生成文本向量表示
pub struct OpenAIEmbeddings {
    /// OpenAI 客户端
    client: Arc<Client<OpenAIConfig>>,

    /// Embedding 模型名称
    model: String,

    /// API Key (用于创建客户端)
    api_key: Arc<str>,
}

impl OpenAIEmbeddings {
    /// 创建新的 OpenAI Embeddings 生成器
    ///
    /// # 参数
    /// - `api_key`: OpenAI API Key
    /// - `model`: Embedding 模型名称（默认 text-embedding-3-small）
    ///
    /// # 返回
    /// 返回生成器实例或错误
    ///
    /// # 示例
    /// ```no_run
    /// use prism_forge::embedding::OpenAIEmbeddings;
    ///
    /// let generator = OpenAIEmbeddings::new("sk-...", None).unwrap();
    /// ```
    pub fn new(api_key: &str, model: Option<String>) -> Result<Self> {
        if api_key.is_empty() {
            return Err(anyhow::anyhow!("API Key 不能为空"));
        }

        // 创建 OpenAI 配置
        let config = OpenAIConfig::default()
            .with_api_key(api_key);

        // 创建 OpenAI 客户端
        let client = Client::with_config(config);

        Ok(Self {
            client: Arc::new(client),
            model: model.unwrap_or_else(|| "text-embedding-3-small".to_string()),
            api_key: api_key.into(),
        })
    }

    /// 为单条文本生成向量
    ///
    /// # 参数
    /// - `text`: 输入文本
    ///
    /// # 返回
    /// 返回 1536 维向量或错误
    ///
    /// # 示例
    /// ```no_run
    /// # use prism_forge::embedding::OpenAIEmbeddings;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let generator = OpenAIEmbeddings::new("sk-...", None)?;
    /// let vector = generator.generate_embedding("读取文件并显示内容").await?;
    /// assert_eq!(vector.len(), 1536);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let text = text.trim();

        if text.is_empty() {
            // 返回全零向量
            return Ok(vec![0.0; self.dimension()]);
        }

        // 创建请求
        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.model)
            .input(EmbeddingInput::String(text.to_string()))
            .build()?;

        // 调用 OpenAI API
        let response = self
            .client
            .embeddings()
            .create(request)
            .await
            .context("调用 OpenAI Embeddings API 失败")?;

        // 提取向量
        if let Some(embedding) = response.data.first() {
            Ok(embedding.embedding.clone())
        } else {
            Err(anyhow::anyhow!("API 返回空结果"))
        }
    }

    /// 批量生成向量
    ///
    /// # 参数
    /// - `texts`: 输入文本列表
    ///
    /// # 返回
    /// 返回向量列表（每条文本一个 1536 维向量）或错误
    ///
    /// # 性能
    /// 批量生成比逐条生成更高效，推荐使用
    ///
    /// # 示例
    /// ```no_run
    /// # use prism_forge::embedding::OpenAIEmbeddings;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let generator = OpenAIEmbeddings::new("sk-...", None)?;
    /// let texts = vec![
    ///     "读取配置文件".to_string(),
    ///     "写入日志".to_string(),
    ///     "连接数据库".to_string(),
    /// ];
    /// let vectors = generator.generate_batch(&texts).await?;
    /// assert_eq!(vectors.len(), 3);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn generate_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // 过滤空文本
        let valid_texts: Vec<String> = texts
            .iter()
            .filter(|t| !t.trim().is_empty())
            .map(|t| t.trim().to_string())
            .collect();

        if valid_texts.is_empty() {
            // 如果全部为空，返回全零向量
            return Ok(texts.iter().map(|_| vec![0.0; self.dimension()]).collect());
        }

        // 批量调用 API（最多 2048 个文本）
        let chunk_size = 2048.min(valid_texts.len());
        let mut all_vectors = Vec::new();

        for chunk in valid_texts.chunks(chunk_size) {
            let inputs: Vec<String> = chunk.to_vec();

            let request = CreateEmbeddingRequestArgs::default()
                .model(&self.model)
                .input(inputs)
                .build()?;

            let response = self
                .client
                .embeddings()
                .create(request)
                .await
                .context("批量调用 OpenAI Embeddings API 失败")?;

            // 提取向量（按顺序）
            let mut vectors: Vec<Vec<f32>> = response
                .data
                .into_iter()
                .map(|e| e.embedding)
                .collect();

            all_vectors.append(&mut vectors);
        }

        Ok(all_vectors)
    }

    /// 获取向量维度
    ///
    /// # 返回
    /// 返回固定的向量维度（1536 for text-embedding-3-small）
    pub const fn dimension(&self) -> usize {
        1536
    }

    /// 获取模型名称
    pub fn model(&self) -> &str {
        &self.model
    }

    /// 估算 tokens 数量
    ///
    /// 粗略估算：1 token ≈ 4 个英文字符 或 2-3 个中文字符
    pub fn estimate_tokens(&self, text: &str) -> usize {
        // 简单估算：字符数 / 4（英文）或字符数 / 3（中文）
        let char_count = text.chars().count();

        // 检测是否包含中文
        let has_chinese = text.chars().any(|c| {
            let cp = c as u32;
            (0x4E00..=0x9FFF).contains(&cp) // CJK 统一表意文字
        });

        if has_chinese {
            char_count / 3
        } else {
            char_count / 4
        }
    }

    /// 估算费用（美元）
    ///
    /// # 参数
    /// - `texts`: 文本列表
    ///
    /// # 返回
    /// 返回估算费用（USD）
    ///
    /// # 费率
    /// text-embedding-3-small: $0.00002 / 1K tokens
    pub fn estimate_cost(&self, texts: &[String]) -> f64 {
        let total_tokens: usize = texts
            .iter()
            .map(|t| self.estimate_tokens(t))
            .sum();

        let price_per_1k_tokens = 0.00002;
        (total_tokens as f64 / 1000.0) * price_per_1k_tokens
    }
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_generator() {
        let generator = OpenAIEmbeddings::new("test-key", None);
        assert!(generator.is_ok());

        let gen = generator.unwrap();
        assert_eq!(gen.model(), "text-embedding-3-small");
        assert_eq!(gen.dimension(), 1536);
    }

    #[test]
    fn test_empty_api_key() {
        let generator = OpenAIEmbeddings::new("", None);
        assert!(generator.is_err());
    }

    #[test]
    fn test_custom_model() {
        let generator = OpenAIEmbeddings::new("test-key", Some("text-embedding-3-large".to_string()));
        assert!(generator.is_ok());

        let gen = generator.unwrap();
        assert_eq!(gen.model(), "text-embedding-3-large");
    }

    #[test]
    fn test_estimate_tokens_english() {
        let generator = OpenAIEmbeddings::new("test-key", None).unwrap();
        let text = "Hello world, this is a test message";
        let tokens = generator.estimate_tokens(text);

        // 大约 40 个字符 / 4 = 10 tokens
        assert!(tokens >= 8 && tokens <= 12);
    }

    #[test]
    fn test_estimate_tokens_chinese() {
        let generator = OpenAIEmbeddings::new("test-key", None).unwrap();
        let text = "你好世界，这是一条测试消息";
        let tokens = generator.estimate_tokens(text);

        // 14 个字符 / 3 ≈ 5 tokens
        assert!(tokens >= 4 && tokens <= 6);
    }

    #[test]
    fn test_estimate_cost() {
        let generator = OpenAIEmbeddings::new("test-key", None).unwrap();

        // 1000 tokens ≈ $0.00002
        let texts = vec![
            "a".repeat(4000), // ~1000 tokens
        ];

        let cost = generator.estimate_cost(&texts);
        assert!(cost > 0.00001 && cost < 0.00003);
    }

    #[test]
    fn test_estimate_cost_batch() {
        let generator = OpenAIEmbeddings::new("test-key", None).unwrap();

        let texts = vec![
            "test message".to_string(),
            "another test".to_string(),
        ];

        let cost = generator.estimate_cost(&texts);
        assert!(cost > 0.0);
    }
}
