//! 会话 Summary 读取模块（增强版）
//!
//! 从 Claude Code 会话文件（.jsonl）中提取 summary 信息
//! 支持多级 fallback 策略获取会话显示名称

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;
use regex::Regex;

// ==================== 辅助函数 ====================

/// 安全地将字符串截断到指定字符数
///
/// 确保截断位置不会落在 UTF-8 多字节字符的中间
///
/// # 参数
/// * `s` - 要截断的字符串
/// * `max_chars` - 最大字符数
///
/// # 返回
/// 截断后的字符串（不会超过字符边界）
fn truncate_str_to_chars(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}

// ==================== 数据结构定义 ====================

/// 名称来源枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NameSource {
    /// 从会话文件的 summary 字段
    Summary,
    /// 从第一个真正的 user message（local-command-stdout 之后）
    FirstUserMessage,
    /// 从 history.jsonl 的 display 字段
    HistoryDisplay,
    /// 从会话内容智能提取（Markdown 标题）
    ContentExtraction,
    /// 默认 fallback（会话 ID）
    Fallback,
}

/// 会话显示名称（包含来源信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDisplayName {
    /// 显示名称
    pub name: String,
    /// 名称来源
    pub source: NameSource,
    /// 会话 ID
    pub session_id: String,
}

/// 会话摘要信息（保留向后兼容）
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

/// History 记录结构
#[derive(Debug, Deserialize)]
struct HistoryRecord {
    #[serde(rename = "sessionId")]
    session_id: String,
    display: String,
    project: String,
    timestamp: u64,
}

/// 消息结构（用于内容提取）
#[derive(Debug, Deserialize)]
struct Message {
    #[serde(rename = "type")]
    msg_type: String,
    message: Option<MsgContent>,
}

#[derive(Debug, Deserialize)]
struct MsgContent {
    role: String,
    content: Option<serde_json::Value>,
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

    #[error("无法获取用户主目录")]
    HomeDirNotFound,
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

// ==================== 多级 Fallback 策略实现 ====================

impl SessionDisplayName {
    /// 获取会话的显示名称（使用多级 fallback 策略）
    ///
    /// # 命名优先级
    /// 1. **Summary**（优先）：从会话文件第一行的 summary 字段
    /// 2. **First Real User Message**：找到第一个 `<local-command-stdout>` 标签后的 user message
    /// 3. **History Display**：从 history.jsonl 的 display 字段
    /// 4. **Content Extraction**：从会话内容智能提取 Markdown 标题
    /// 5. **Fallback**：使用会话 ID 的前 8 位
    ///
    /// # 参数
    /// * `file_path` - 会话文件的完整路径
    /// * `history_cache` - history.jsonl 的缓存（可选，建议提供以提高性能）
    ///
    /// # 返回
    /// 返回 `SessionDisplayName`，包含名称及其来源
    pub async fn get_display_name(
        file_path: impl AsRef<Path>,
        history_cache: Option<&HashMap<String, String>>,
    ) -> Result<Self, SessionReaderError> {
        let file_path = file_path.as_ref();
        let session_id = extract_session_id(file_path)?;

        // 策略 1: 优先从 summary 读取
        if let Ok(name) = Self::try_read_summary(file_path, &session_id).await {
            #[cfg(debug_assertions)]
            eprintln!("[SessionDisplayName] 使用 summary: {}", name.name);

            return Ok(name);
        }

        // 策略 2: 提取第一个真正的 user message（在 local-command-stdout 之后）
        if let Ok(name) = Self::extract_first_real_user_message(file_path, &session_id).await {
            #[cfg(debug_assertions)]
            eprintln!("[SessionDisplayName] 使用第一个真正的 user message: {}", name.name);

            return Ok(name);
        }

        // 策略 3: 从 history.jsonl 获取
        if let Some(history) = history_cache {
            if let Some(display) = history.get(&session_id) {
                #[cfg(debug_assertions)]
                eprintln!("[SessionDisplayName] 使用 history display: {}", display);

                return Ok(Self {
                    name: display.clone(),
                    source: NameSource::HistoryDisplay,
                    session_id,
                });
            }
        }

        // 策略 4: 从会话内容智能提取
        if let Ok(name) = Self::extract_from_content(file_path, &session_id).await {
            #[cfg(debug_assertions)]
            eprintln!("[SessionDisplayName] 从内容提取: {} (来源: 智能提取)", name.name);

            return Ok(name);
        }

        // 策略 5: 使用会话 ID 作为 fallback
        #[cfg(debug_assertions)]
        eprintln!("[SessionDisplayName] 无法获取显示名称，使用会话 ID 作为 fallback");

        Ok(Self {
            name: format!("会话 {}", truncate_str_to_chars(&session_id, 8)),
            source: NameSource::Fallback,
            session_id,
        })
    }

    /// 策略 2: 尝试从 summary 读取
    async fn try_read_summary(
        file_path: &Path,
        session_id: &str,
    ) -> Result<Self, SessionReaderError> {
        let content = tokio::fs::read_to_string(file_path).await?;
        let first_line = content.lines().next().ok_or(SessionReaderError::EmptyFile)?;

        let record: SummaryRecord = serde_json::from_str(first_line)?;

        if record.record_type == "summary" {
            Ok(Self {
                name: record.summary,
                source: NameSource::Summary,
                session_id: session_id.to_string(),
            })
        } else {
            Err(SessionReaderError::InvalidFormat)
        }
    }

    /// 策略 2: 提取第一个真正的 user message（在 local-command-stdout 之后）
    ///
    /// 这个方法专门处理由 `/clear` 命令开始的会话。
    /// 逻辑：找到第一个包含 `<local-command-stdout>` 标签的消息，
    /// 然后取其下一个 role 为 "user" 的消息内容作为会话名称。
    async fn extract_first_real_user_message(
        file_path: &Path,
        session_id: &str,
    ) -> Result<Self, SessionReaderError> {
        let content = tokio::fs::read_to_string(file_path).await?;

        // 读取前 200 行，避免处理大文件
        let lines: Vec<&str> = content.lines().take(200).collect();

        // 查找第一个包含 <local-command-stdout> 的行索引
        let mut found_command_stdout = false;
        let mut command_stdout_index = 0;

        for (i, line) in lines.iter().enumerate() {
            if line.contains("<local-command-stdout>") {
                found_command_stdout = true;
                command_stdout_index = i;
                break;
            }
        }

        if !found_command_stdout {
            return Err(SessionReaderError::InvalidFormat);
        }

        // 从 local-command-stdout 之后开始查找第一个 user message
        for line in lines.iter().skip(command_stdout_index + 1) {
            if let Ok(msg) = serde_json::from_str::<Message>(line) {
                if msg.role() == "user" {
                    let content = msg.content();

                    // 提取前 50 个字符（按字符数，不是字节数）
                    let char_count = content.chars().count();
                    let truncated = if char_count > 50 {
                        // 找到第 50 个字符的字节边界位置
                        let byte_end = content
                            .char_indices()
                            .nth(50)
                            .map(|(i, _)| i)
                            .unwrap_or(content.len());

                        format!("{}...", &content[..byte_end])
                    } else {
                        content.to_string()
                    };

                    // 清理内容：去除 Markdown 符号和多余空白
                    let cleaned = Self::clean_user_message_content(&truncated);

                    // 如果清理后的内容太短，继续查找下一个 user message
                    if cleaned.chars().count() < 5 {
                        continue;
                    }

                    return Ok(Self {
                        name: cleaned,
                        source: NameSource::FirstUserMessage,
                        session_id: session_id.to_string(),
                    });
                }
            }
        }

        Err(SessionReaderError::InvalidFormat)
    }

    /// 清理 user message 内容
    ///
    /// 去除 Markdown 符号、多余空白和特殊字符
    fn clean_user_message_content(content: &str) -> String {
        // 去除常见的 Markdown 符号
        let cleaned = content
            .replace("#", "")
            .replace("*", "")
            .replace("`", "")
            .replace("[", "")
            .replace("]", "")
            .replace("(", "")
            .replace(")", "")
            .replace("<", "")
            .replace(">", "")
            .replace("|", "")
            // 去除多余空白
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");

        cleaned
    }

    /// 策略 3: 从会话内容智能提取（Markdown 标题）
    async fn extract_from_content(
        file_path: &Path,
        session_id: &str,
    ) -> Result<Self, SessionReaderError> {
        let content = tokio::fs::read_to_string(file_path).await?;

        // 读取最后 N 条消息（不需要 ?，因为它直接返回 Vec）
        let messages: Vec<Message> = Self::read_last_n_messages(&content, 10);

        // 优先从助手消息中提取 Markdown 标题
        for msg in messages.iter().rev() {
            if msg.role() == "assistant" {
                if let Some(title) = Self::extract_markdown_title(&msg.content()) {
                    let simplified = Self::simplify_title(title);
                    if !simplified.is_empty() {
                        return Ok(Self {
                            name: simplified,
                            source: NameSource::ContentExtraction,
                            session_id: session_id.to_string(),
                        });
                    }
                }
            }
        }

        Err(SessionReaderError::InvalidFormat)
    }

    /// 提取 Markdown 标题
    fn extract_markdown_title(content: &str) -> Option<String> {
        let title_re = Regex::new(r"^#+\s*(.+?)\s*$").unwrap();

        for line in content.lines().take(20) {
            if let Some(caps) = title_re.captures(line) {
                let title = caps.get(1)?.as_str().trim();
                // 过滤掉过短的标题
                if title.len() >= 4 {
                    return Some(title.to_string());
                }
            }
        }
        None
    }

    /// 简化标题（去除符号，限制长度）
    fn simplify_title(title: String) -> String {
        // 移除 Markdown 符号和表情符号
        let simplified = title
            .replace("## ", "")
            .replace("# ", "")
            .replace("✅", "")
            .replace("❌", "")
            .replace("⚠️", "")
            .replace("！", "")
            .replace("。", "")
            .trim()
            .to_string();

        // 限制长度（安全地按字符截断）
        if simplified.chars().count() > 50 {
            let truncated = truncate_str_to_chars(&simplified, 47);
            format!("{}...", truncated)
        } else {
            simplified
        }
    }

    /// 读取最后 N 条消息
    fn read_last_n_messages(content: &str, n: usize) -> Vec<Message> {
        content
            .lines()
            .filter_map(|line| serde_json::from_str::<Message>(line).ok())
            .rev()
            .take(n)
            .collect()
    }
}

impl Message {
    fn role(&self) -> &str {
        self.message
            .as_ref()
            .map(|m| m.role.as_str())
            .unwrap_or("unknown")
    }

    fn content(&self) -> String {
        self.message
            .as_ref()
            .and_then(|m| m.content.as_ref())
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string()
    }
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

// ==================== History.jsonl 读取功能 ====================

/// 从 history.jsonl 读取会话的显示名称映射
///
/// # 参数
/// * `claude_dir` - Claude 配置目录（通常是 `~/.claude`）
///
/// # 返回
/// 返回 HashMap<session_id, display_name>
///
/// # 示例
/// ```no_run
/// use crate::session_reader::load_history_cache;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let history = load_history_cache(Path::new("/home/user/.claude")).await?;
/// println!("Loaded {} history entries", history.len());
/// # Ok(())
/// # }
/// ```
pub async fn load_history_cache(
    claude_dir: impl AsRef<Path>,
) -> Result<HashMap<String, String>, SessionReaderError> {
    let history_file = claude_dir.as_ref().join("history.jsonl");

    if !history_file.exists() {
        #[cfg(debug_assertions)]
        eprintln!("[HistoryCache] history.jsonl 不存在，返回空缓存");

        return Ok(HashMap::new());
    }

    let content = tokio::fs::read_to_string(&history_file).await?;
    let mut display_names = HashMap::new();

    #[cfg(debug_assertions)]
    eprintln!("[HistoryCache] 开始读取 history.jsonl: {:?}", history_file);

    for line in content.lines() {
        if let Ok(record) = serde_json::from_str::<HistoryRecord>(line) {
            // 只保留每个会话的第一次记录（最早的）
            display_names.entry(record.session_id).or_insert(record.display);
        }
    }

    #[cfg(debug_assertions)]
    eprintln!("[HistoryCache] 加载完成，共 {} 个会话", display_names.len());

    Ok(display_names)
}

/// 便捷函数：从默认路径加载 history 缓存
///
/// 自动查找 `~/.claude/history.jsonl`
pub async fn load_default_history_cache() -> Result<HashMap<String, String>, SessionReaderError> {
    let home = dirs::home_dir().ok_or(SessionReaderError::HomeDirNotFound)?;
    let claude_dir = home.join(".claude");
    load_history_cache(claude_dir).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs;
    use tokio::io::AsyncWriteExt;

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
