//! 会话 Summary 读取模块
//!
//! 从 Claude Code 会话文件（.jsonl）中提取 summary 信息

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;

// ==================== 数据结构定义 ====================

/// 会话摘要信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    /// 摘要内容
    pub summary: String,
    /// 最后一条消息的 UUID
    pub leaf_uuid: String,
    /// 会话 ID
    pub session_id: String,
}

/// Summary 行的 JSON 结构
#[derive(Debug, Deserialize)]
struct SummaryRecord {
    #[serde(rename = "type")]
    record_type: String,
    summary: String,
    #[serde(rename = "leafUuid")]
    leaf_uuid: String,
}

/// 错误类型
#[derive(Error, Debug)]
pub enum SessionReaderError {
    #[error("文件不存在: {0}")]
    FileNotFound(PathBuf),

    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON 解析错误: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("文件格式错误: 第一行不是 summary 类型")]
    InvalidFormat,

    #[error("会话文件为空")]
    EmptyFile,
}

// ==================== Summary 读取实现 ====================

/// 从会话文件中读取 summary（异步版本）
///
/// # 参数
/// * `file_path` - 会话文件的完整路径
///
/// # 返回
/// 返回 `SessionSummary` 或错误
pub async fn read_summary_from_file(file_path: &Path) -> Result<SessionSummary, SessionReaderError> {
    // 检查文件是否存在
    if !file_path.exists() {
        return Err(SessionReaderError::FileNotFound(file_path.to_path_buf()));
    }

    // 异步读取文件内容
    let content = tokio::fs::read_to_string(file_path).await?;

    // 读取第一行
    let first_line = content
        .lines()
        .next()
        .ok_or(SessionReaderError::EmptyFile)?;

    // 解析 JSON
    let record: SummaryRecord = serde_json::from_str(first_line)?;

    // 验证类型是否为 summary
    if record.record_type != "summary" {
        return Err(SessionReaderError::InvalidFormat);
    }

    // 从文件路径提取会话 ID
    let session_id = extract_session_id(file_path)?;

    Ok(SessionSummary {
        summary: record.summary,
        leaf_uuid: record.leaf_uuid,
        session_id,
    })
}

/// 从文件路径提取会话 ID
fn extract_session_id(file_path: &Path) -> Result<String, SessionReaderError> {
    file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(String::from)
        .ok_or_else(|| SessionReaderError::InvalidFormat)
}

// ==================== 缓存管理 ====================

/// Summary 缓存管理器
pub struct SummaryCache {
    cache: Arc<RwLock<HashMap<String, SessionSummary>>>,
}

impl SummaryCache {
    /// 创建新的缓存实例
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 获取或加载 summary（带缓存）
    ///
    /// 先从缓存读取，如果不存在则从文件加载并写入缓存
    pub async fn get_or_load(
        &self,
        file_path: &Path,
    ) -> Result<SessionSummary, SessionReaderError> {
        let path_str = file_path.to_string_lossy().to_string();

        // 先查缓存
        {
            let cache = self.cache.read().await;
            if let Some(summary) = cache.get(&path_str) {
                #[cfg(debug_assertions)]
                eprintln!("[SummaryCache] 缓存命中: {}", path_str);

                return Ok(summary.clone());
            }
        }

        #[cfg(debug_assertions)]
        eprintln!("[SummaryCache] 缓存未命中，从文件加载: {}", path_str);

        // 缓存未命中，从文件加载
        let summary = read_summary_from_file(file_path).await?;

        // 写入缓存
        let mut cache = self.cache.write().await;
        cache.insert(path_str, summary.clone());

        Ok(summary)
    }

    /// 批量预加载 summary（用于列表页初始化）
    pub async fn preload(&self, file_paths: &[PathBuf]) {
        #[cfg(debug_assertions)]
        eprintln!("[SummaryCache] 开始预加载 {} 个文件", file_paths.len());

        let mut cache = self.cache.write().await;

        for path in file_paths {
            let path_str = path.to_string_lossy().to_string();

            // 如果已缓存则跳过
            if cache.contains_key(&path_str) {
                continue;
            }

            // 异步加载并缓存（忽略错误）
            if let Ok(summary) = read_summary_from_file(path).await {
                cache.insert(path_str, summary);
            }
        }

        #[cfg(debug_assertions)]
        eprintln!("[SummaryCache] 预加载完成，当前缓存数: {}", cache.len());
    }

    /// 清除缓存
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();

        #[cfg(debug_assertions)]
        eprintln!("[SummaryCache] 缓存已清空");
    }

    /// 获取缓存大小
    pub async fn size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }
}

impl Default for SummaryCache {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== 辅助函数 ====================

/// 批量读取多个会话文件的 summary（并行加载）
pub async fn batch_read_summaries(
    file_paths: &[PathBuf],
) -> Vec<Result<SessionSummary, SessionReaderError>> {
    use futures::future::join_all;

    let futures = file_paths
        .iter()
        .map(|path| read_summary_from_file(path));

    join_all(futures).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs;
    use std::io::Write;

    #[tokio::test]
    async fn test_read_summary_from_file() {
        // 创建临时测试文件
        let temp_dir = std::env::temp_dir();
        let session_file = temp_dir.join("test-session.jsonl");

        let mut file = fs::File::create(&session_file).await.unwrap();
        file.write_all(
            br#"{"type":"summary","summary":"Test Summary","leafUuid":"uuid-123"}}"#
        ).await.unwrap();

        // 读取 summary
        let summary = read_summary_from_file(&session_file).await.unwrap();

        assert_eq!(summary.summary, "Test Summary");
        assert_eq!(summary.leaf_uuid, "uuid-123");
        assert_eq!(summary.session_id, "test-session");

        // 清理
        fs::remove_file(&session_file).await.ok();
    }

    #[tokio::test]
    async fn test_cache_hit() {
        let cache = SummaryCache::new();
        let temp_dir = std::env::temp_dir();
        let session_file = temp_dir.join("cache-test.jsonl");

        // 创建测试文件
        let mut file = fs::File::create(&session_file).await.unwrap();
        file.write_all(
            br#"{"type":"summary","summary":"Cache Test","leafUuid":"uuid-456"}}"#
        ).await.unwrap();

        // 第一次加载（从文件）
        let summary1 = cache.get_or_load(&session_file).await.unwrap();
        assert_eq!(summary1.summary, "Cache Test");

        // 第二次加载（从缓存）
        let summary2 = cache.get_or_load(&session_file).await.unwrap();
        assert_eq!(summary2.summary, "Cache Test");

        // 清理
        fs::remove_file(&session_file).await.ok();
    }
}
