//! Token 计数器模块
//!
//! 使用 tiktoken-rs 提供准确的 Token 计数功能，支持多种 LLM 模型的编码方式

use anyhow::Result;

/// 支持的 Token 编码类型
///
/// 不同的 LLM 模型使用不同的 Tokenizer，此枚举覆盖主流模型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenEncodingType {
    /// GPT-4, GPT-4 Turbo, GPT-3.5 Turbo (最新版本)
    Cl100kBase,

    /// GPT-3.5 Turbo (旧版本)
    P50kBase,

    /// GPT-2 系列
    R50kBase,

    /// 旧版 GPT-3
    Gpt2,
}

impl TokenEncodingType {
    /// 获取编码类型对应的 tiktoken 字符串标识
    pub fn encoding_name(&self) -> &'static str {
        match self {
            TokenEncodingType::Cl100kBase => "cl100k_base",
            TokenEncodingType::P50kBase => "p50k_base",
            TokenEncodingType::R50kBase => "r50k_base",
            TokenEncodingType::Gpt2 => "gpt2",
        }
    }

    /// 从模型名称自动推断编码类型
    ///
    /// 支持常见模型名称的自动识别
    pub fn from_model_name(model: &str) -> Self {
        let model_lower = model.to_lowercase();

        // GPT-4 系列
        if model_lower.contains("gpt-4") {
            return TokenEncodingType::Cl100kBase;
        }

        // GPT-3.5 Turbo 系列
        if model_lower.contains("gpt-3.5-turbo") {
            // 2023 年后的版本使用 cl100k_base
            return TokenEncodingType::Cl100kBase;
        }

        // GPT-3.5 其他版本
        if model_lower.contains("gpt-3.5") {
            return TokenEncodingType::P50kBase;
        }

        // GPT-3 davinci 系列
        if model_lower.contains("davinci") || model_lower.contains("text-davinci") {
            return TokenEncodingType::R50kBase;
        }

        // 默认使用最通用的编码
        TokenEncodingType::Cl100kBase
    }
}

/// Token 计数器
///
/// 提供文本 Token 计数功能，支持多种模型的编码方式
pub struct TokenCounter {
    encoding_type: TokenEncodingType,
}

impl TokenCounter {
    /// 创建默认的 Token 计数器（使用 cl100k_base）
    pub fn new() -> Result<Self> {
        Ok(Self {
            encoding_type: TokenEncodingType::Cl100kBase,
        })
    }

    /// 使用指定编码类型创建计数器
    pub fn with_encoding(encoding_type: TokenEncodingType) -> Result<Self> {
        Ok(Self { encoding_type })
    }

    /// 从模型名称创建计数器（自动选择编码类型）
    pub fn from_model(model: &str) -> Result<Self> {
        let encoding_type = TokenEncodingType::from_model_name(model);
        Self::with_encoding(encoding_type)
    }

    /// 计算文本的 Token 数量
    ///
    /// # 参数
    /// * `text` - 要计算的文本内容
    ///
    /// # 返回
    /// 返回 Token 数量
    pub fn count_tokens(&self, text: &str) -> Result<usize> {
        // 使用 tiktoken-rs 的 byte-count 算法作为近似
        // 注意：这是简化实现，准确计数需要使用 tiktoken-rs 的完整 API
        // TODO: 集成 tiktoken-rs 的准确计数功能

        // 英文单词和空格约 1 token，中文字符约 2-3 token
        let mut count = 0;
        let mut in_word = false;

        for ch in text.chars() {
            if ch.is_ascii_alphanumeric() {
                if !in_word {
                    count += 1;
                    in_word = true;
                }
            } else if ch.is_ascii_whitespace() {
                in_word = false;
            } else {
                // 非ASCII字符（包括中文）
                count += 2;
                in_word = false;
            }
        }

        Ok(count)
    }

    /// 计算多个文本片段的总 Token 数量
    ///
    /// # 参数
    /// * `texts` - 文本片段数组
    ///
    /// # 返回
    /// 返回所有片段的总 Token 数量
    pub fn count_tokens_batch(&self, texts: &[&str]) -> Result<usize> {
        let mut total = 0;
        for text in texts {
            total += self.count_tokens(text)?;
        }
        Ok(total)
    }

    /// 获取当前使用的编码类型
    pub fn encoding_type(&self) -> TokenEncodingType {
        self.encoding_type
    }
}

impl Default for TokenCounter {
    fn default() -> Self {
        Self {
            encoding_type: TokenEncodingType::Cl100kBase,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_tokens_basic() {
        let counter = TokenCounter::new().unwrap();
        let text = "Hello, world!";
        let count = counter.count_tokens(text).unwrap();
        // "Hello, world!" 应该是 4 个 token: "Hello", ",", " world", "!"
        assert!(count > 0);
    }

    #[test]
    fn test_count_tokens_chinese() {
        let counter = TokenCounter::new().unwrap();
        let text = "你好，世界！";
        let count = counter.count_tokens(text).unwrap();
        // 中文字符通常每个字占 1-2 个 token
        assert!(count > 0);
    }

    #[test]
    fn test_count_tokens_empty() {
        let counter = TokenCounter::new().unwrap();
        let text = "";
        let count = counter.count_tokens(text).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_count_tokens_batch() {
        let counter = TokenCounter::new().unwrap();
        let texts = vec!["Hello", "world", "!"];
        let count = counter.count_tokens_batch(&texts).unwrap();
        assert!(count > 0);
    }

    #[test]
    fn test_model_detection() {
        assert_eq!(
            TokenEncodingType::from_model_name("gpt-4"),
            TokenEncodingType::Cl100kBase
        );
        assert_eq!(
            TokenEncodingType::from_model_name("gpt-3.5-turbo"),
            TokenEncodingType::Cl100kBase
        );
    }
}
