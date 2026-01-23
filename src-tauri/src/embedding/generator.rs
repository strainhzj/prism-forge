//! 向量嵌入生成器
//!
//! 使用 FastEmbed (BGE-small-en-v1.5-quantized) 生成 384 维向量
//!
//! **注意**: 由于 fastembed 的依赖库 ort 在 Windows 上存在编译问题，
//! 当前在 Windows 平台使用占位符实现。将来上游修复后会自动启用真实实现。

use anyhow::{Context, Result};

#[cfg(not(target_os = "windows"))]
use fastembed::{EmbeddingModel, InitOptions, ModelType};

/// 向量嵌入生成器
///
/// 负责将文本转换为 384 维向量表示
pub struct EmbeddingGenerator {
    #[cfg(not(target_os = "windows"))]
    /// FastEmbed 模型实例
    model: EmbeddingModel,

    #[cfg(target_os = "windows")]
    /// 占位符：Windows 平台暂不支持
    _placeholder: (),
}

impl EmbeddingGenerator {
    /// 创建新的向量生成器
    ///
    /// 加载 BGE-small-en-v1.5-quantized 模型
    ///
    /// # 返回
    /// 返回生成器实例或错误
    ///
    /// # 示例
    /// ```no_run
    /// use prism_forge::embedding::EmbeddingGenerator;
    ///
    /// let generator = EmbeddingGenerator::new().unwrap();
    /// ```
    pub fn new() -> Result<Self> {
        #[cfg(not(target_os = "windows"))]
        {
            // 配置模型选项
            let options = InitOptions {
                model_name: ModelType::BGESmallENV15Quantized,
                ..Default::default()
            };

            // 初始化模型（会自动下载模型文件）
            let model = EmbeddingModel::try_new(options)
                .context("无法加载 FastEmbed 模型，请检查网络连接或手动下载模型")?;

            Ok(Self { model })
        }

        #[cfg(target_os = "windows")]
        {
            eprintln!("警告: 向量嵌入功能在 Windows 平台暂不可用");
            eprintln!("这是由于 fastembed 依赖的 ort 库在 Windows 上存在编译问题");
            eprintln!("相关 issue: https://github.com/dgrine/fastembed/issues");
            Ok(Self { _placeholder: () })
        }
    }

    /// 为单条消息生成向量
    ///
    /// # 参数
    /// - `content`: 消息摘要文本
    ///
    /// # 返回
    /// 返回 384 维向量或错误
    ///
    /// # 示例
    /// ```no_run
    /// # use prism_forge::embedding::EmbeddingGenerator;
    /// let generator = EmbeddingGenerator::new().unwrap();
    /// let vector = generator.generate_for_message("读取文件并显示内容").unwrap();
    /// assert_eq!(vector.len(), 384);
    /// ```
    pub fn generate_for_message(&self, content: &str) -> Result<Vec<f32>> {
        if content.trim().is_empty() {
            return Ok(vec![0.0; 384]);
        }

        #[cfg(not(target_os = "windows"))]
        {
            // 生成向量
            let vectors = self.model.embed(vec![content], None)
                .context("向量生成失败")?;

            // 返回第一个向量
            if let Some(vector) = vectors.first() {
                Ok(vector.clone())
            } else {
                Err(anyhow::anyhow!("向量生成返回空结果"))
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Windows 平台：返回基于内容的伪随机向量（占位符实现）
            let mut vector = vec![0.0; 384];
            for (i, val) in vector.iter_mut().enumerate() {
                // 使用内容和索引生成确定性的伪随机值
                let hash = content.chars().map(|c| c as usize).sum::<usize>() + i;
                *val = ((hash % 1000) as f32) / 1000.0;
            }
            Ok(vector)
        }
    }

    /// 批量生成向量
    ///
    /// # 参数
    /// - `contents`: 消息摘要文本列表
    ///
    /// # 返回
    /// 返回向量列表（每条消息一个 384 维向量）或错误
    ///
    /// # 性能
    /// 批量生成比逐条生成更高效，推荐使用
    ///
    /// # 示例
    /// ```no_run
    /// # use prism_forge::embedding::EmbeddingGenerator;
    /// let generator = EmbeddingGenerator::new().unwrap();
    /// let summaries = vec![
    ///     "读取配置文件",
    ///     "写入日志",
    ///     "连接数据库",
    /// ];
    /// let vectors = generator.generate_batch(&summaries).unwrap();
    /// assert_eq!(vectors.len(), 3);
    /// ```
    pub fn generate_batch(&self, contents: &[String]) -> Result<Vec<Vec<f32>>> {
        if contents.is_empty() {
            return Ok(Vec::new());
        }

        #[cfg(not(target_os = "windows"))]
        {
            // 过滤空文本
            let valid_contents: Vec<&str> = contents.iter()
                .map(|s| s.as_str())
                .filter(|s| !s.trim().is_empty())
                .collect();

            if valid_contents.is_empty() {
                // 如果全部为空，返回全零向量
                return Ok(contents.iter()
                    .map(|_| vec![0.0; 384])
                    .collect());
            }

            // 批量生成向量
            let vectors = self.model.embed(valid_contents, None)
                .context("批量向量生成失败")?;

            Ok(vectors)
        }

        #[cfg(target_os = "windows")]
        {
            // Windows 平台：逐条生成占位符向量
            contents.iter()
                .map(|content| self.generate_for_message(content))
                .collect()
        }
    }

    /// 获取向量维度
    ///
    /// # 返回
    /// 返回固定的向量维度（384）
    pub const fn dimension(&self) -> usize {
        384
    }

    /// 检查是否使用占位符实现
    ///
    /// # 返回
    /// 如果是 Windows 平台返回 true，否则返回 false
    pub fn is_placeholder(&self) -> bool {
        cfg!(target_os = "windows")
    }
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_generator() {
        let generator = EmbeddingGenerator::new().unwrap();
        assert_eq!(generator.dimension(), 384);
    }

    #[test]
    fn test_generate_single_vector() {
        let generator = EmbeddingGenerator::new().unwrap();
        let vector = generator.generate_for_message("测试消息").unwrap();
        assert_eq!(vector.len(), 384);
    }

    #[test]
    fn test_generate_batch_vectors() {
        let generator = EmbeddingGenerator::new().unwrap();
        let summaries = vec![
            "读取文件".to_string(),
            "写入数据".to_string(),
            "删除记录".to_string(),
        ];
        let vectors = generator.generate_batch(&summaries).unwrap();
        assert_eq!(vectors.len(), 3);
        for vector in vectors {
            assert_eq!(vector.len(), 384);
        }
    }

    #[test]
    fn test_empty_content() {
        let generator = EmbeddingGenerator::new().unwrap();
        let vector = generator.generate_for_message("").unwrap();
        assert_eq!(vector, vec![0.0; 384]);
    }

    #[test]
    fn test_batch_with_empty_strings() {
        let generator = EmbeddingGenerator::new().unwrap();
        let summaries = vec![
            "有效内容".to_string(),
            "".to_string(),
            "   ".to_string(),
        ];
        let vectors = generator.generate_batch(&summaries).unwrap();
        assert_eq!(vectors.len(), 3);
    }

    #[test]
    fn test_placeholder_check() {
        let generator = EmbeddingGenerator::new().unwrap();
        #[cfg(target_os = "windows")]
        assert!(generator.is_placeholder());

        #[cfg(not(target_os = "windows"))]
        assert!(!generator.is_placeholder());
    }
}
