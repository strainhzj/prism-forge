//! JSONL 流式解析器
//!
//! 支持增量读取 Claude Code 的 JSONL 会话文件，记录每条消息的偏移量和长度。
//! 支持 Windows FileShare 模式，允许在文件被占用时读取。

use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::PathBuf;
use anyhow::{Result, Context};
use serde_json::Value;

/// JSONL 条目
///
/// 包含解析后的 JSON 数据及其在文件中的位置信息
#[derive(Debug, Clone)]
pub struct JsonlEntry {
    /// 文件中的字节偏移量
    pub offset: u64,
    /// JSON 内容长度（字节数）
    pub length: usize,
    /// 解析后的 JSON 值
    pub data: Value,
}

impl JsonlEntry {
    /// 创建新的 JSONL 条目
    pub fn new(offset: u64, length: usize, data: Value) -> Self {
        Self {
            offset,
            length,
            data,
        }
    }

    /// 获取消息类型（Claude Code 会话文件中的 type 字段）
    pub fn message_type(&self) -> Option<String> {
        self.data.get("type")?.as_str().map(|s| s.to_string())
    }

    /// 获取消息角色（user/assistant）
    pub fn role(&self) -> Option<String> {
        self.data.get("role")?.as_str().map(|s| s.to_string())
    }
}

/// JSONL 解析器
///
/// 支持流式读取和增量解析 JSONL 文件
#[derive(Debug)]
pub struct JsonlParser {
    /// 文件路径
    file_path: PathBuf,
    /// 未完成行的缓冲区
    buffer: String,
}

impl JsonlParser {
    /// 创建新的 JSONL 解析器
    ///
    /// # 参数
    /// - `path`: JSONL 文件路径
    ///
    /// # 返回
    /// 返回解析器实例或错误
    pub fn new(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            return Err(anyhow::anyhow!("文件不存在: {:?}", path));
        }

        Ok(Self {
            file_path: path,
            buffer: String::new(),
        })
    }

    /// 解析所有条目
    ///
    /// 读取整个文件并解析所有 JSONL 条目
    ///
    /// # 返回
    /// 返回所有条目的列表或错误
    pub fn parse_all(&mut self) -> Result<Vec<JsonlEntry>> {
        let file = self.open_file_shared()?;
        let mut reader = BufReader::new(file);
        let mut entries = Vec::new();

        let result = self.parse_reader(&mut reader, |entry| {
            entries.push(entry.clone());
        });

        // 清空缓冲区，准备下次解析
        self.buffer.clear();

        result?;
        Ok(entries)
    }

    /// 流式解析（大文件友好）
    ///
    /// 逐行解析并通过回调函数处理每条条目，避免一次性加载全部数据到内存
    ///
    /// # 参数
    /// - `callback`: 处理每条条目的回调函数
    ///
    /// # 返回
    /// 返回解析结果或错误
    pub fn parse_stream<F>(&mut self, mut callback: F) -> Result<()>
    where
        F: FnMut(&JsonlEntry),
    {
        let file = self.open_file_shared()?;
        let mut reader = BufReader::new(file);

        let result = self.parse_reader(&mut reader, callback);

        // 清空缓冲区
        self.buffer.clear();

        result
    }

    /// 按偏移量读取单条消息
    ///
    /// 直接跳转到指定偏移量，读取指定长度的数据并解析
    ///
    /// # 参数
    /// - `offset`: 字节偏移量
    /// - `length`: 数据长度
    ///
    /// # 返回
    /// 返回解析后的 JSON 值或错误
    pub fn parse_entry_at_offset(&self, offset: u64, length: usize) -> Result<Value> {
        let mut file = self.open_file_shared()?;

        // 跳转到指定偏移量
        file.seek(SeekFrom::Start(offset))
            .context(format!("无法跳转到偏移量 {}", offset))?;

        // 读取指定长度的数据
        let mut buffer = vec![0u8; length];
        file.read_exact(&mut buffer)
            .context(format!("无法读取偏移量 {} 处的 {} 字节", offset, length))?;

        // 转换为字符串并解析 JSON
        let json_str = String::from_utf8(buffer)
            .context("偏移量处的数据不是有效的 UTF-8 字符串")?;

        serde_json::from_str(&json_str)
            .context(format!("偏移量 {} 处的 JSON 解析失败", offset))
    }

    /// 从文件中提取消息内容
    ///
    /// 返回文件中原始消息的数量（不包括损坏的条目）
    pub fn count_entries(&mut self) -> Result<usize> {
        let file = self.open_file_shared()?;
        let mut reader = BufReader::new(file);
        let mut count = 0;

        self.parse_reader(&mut reader, |_| {
            count += 1;
        })?;

        // 清空缓冲区
        self.buffer.clear();

        Ok(count)
    }

    // ========== 内部辅助方法 ==========

    /// 以共享模式打开文件（支持 FileShare）
    ///
    /// Windows: 使用 FILE_SHARE_READ | FILE_SHARE_WRITE
    /// Unix: 正常打开（Unix 文件锁定是建议性的）
    fn open_file_shared(&self) -> Result<File> {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::fs::OpenOptionsExt;
            std::fs::OpenOptions::new()
                .read(true)
                .share_mode(0x3) // FILE_SHARE_READ | FILE_SHARE_WRITE
                .open(&self.file_path)
                .context(format!("无法打开文件: {:?}", self.file_path))
        }

        #[cfg(not(target_os = "windows"))]
        {
            File::open(&self.file_path)
                .context(format!("无法打开文件: {:?}", self.file_path))
        }
    }

    /// 解析 Reader 内容
    ///
    /// 核心解析逻辑，处理行缓冲和 JSON 解析
    fn parse_reader<F>(&mut self, reader: &mut BufReader<File>, mut callback: F) -> Result<()>
    where
        F: FnMut(&JsonlEntry),
    {
        let mut line_buffer = String::new();
        let mut current_offset = 0u64;

        loop {
            line_buffer.clear();

            // 读取一行
            let bytes_read = reader.read_line(&mut line_buffer)
                .context("读取文件失败")?;

            // 文件结束
            if bytes_read == 0 {
                // 处理缓冲区中剩余的数据
                if !self.buffer.trim().is_empty() {
                    eprintln!("警告: 文件末尾有未完成的数据，已丢弃: {} 字节", self.buffer.len());
                }
                break;
            }

            let line_length = bytes_read;
            let line = if self.buffer.is_empty() {
                // 无缓冲区数据，直接使用当前行
                line_buffer.trim().to_string()
            } else {
                // 有缓冲区数据，拼接后处理
                self.buffer.push_str(&line_buffer);
                let combined = self.buffer.trim().to_string();
                self.buffer.clear();
                combined
            };

            // 跳过空行
            if line.is_empty() {
                current_offset += line_length as u64;
                continue;
            }

            // 尝试解析 JSON
            match serde_json::from_str::<Value>(&line) {
                Ok(json_data) => {
                    // 解析成功，创建条目
                    let entry = JsonlEntry::new(current_offset, line_length, json_data);
                    callback(&entry);
                }
                Err(e) => {
                    // 解析失败，记录日志并跳过
                    eprintln!(
                        "警告: 偏移量 {} 处的 JSON 解析失败: {}，内容: {}",
                        current_offset,
                        e,
                        if line.len() > 100 { &line[..100] } else { &line }
                    );
                }
            }

            current_offset += line_length as u64;
        }

        Ok(())
    }
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// 创建临时测试文件
    fn create_test_file(content: &str) -> PathBuf {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_session.jsonl");

        let mut file = File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();

        file_path
    }

    #[test]
    fn test_parse_valid_jsonl() {
        let content = r#"{"type": "message", "role": "user"}
{"type": "message", "role": "assistant"}
{"type": "tool_use", "name": "read_file"}"#;

        let file_path = create_test_file(content);
        let mut parser = JsonlParser::new(file_path.clone()).unwrap();

        let entries = parser.parse_all().unwrap();
        assert_eq!(entries.len(), 3);

        assert_eq!(entries[0].message_type(), Some("message".to_string()));
        assert_eq!(entries[0].role(), Some("user".to_string()));

        assert_eq!(entries[1].role(), Some("assistant".to_string()));
        assert_eq!(entries[2].message_type(), Some("tool_use".to_string()));

        // 清理
        std::fs::remove_file(file_path).ok();
    }

    #[test]
    fn test_parse_with_empty_lines() {
        let content = r#"{"type": "message"}

{"type": "message", "role": "user"}


{"type": "tool_use"}"#;

        let file_path = create_test_file(content);
        let mut parser = JsonlParser::new(file_path.clone()).unwrap();

        let entries = parser.parse_all().unwrap();
        assert_eq!(entries.len(), 3);

        // 清理
        std::fs::remove_file(file_path).ok();
    }

    #[test]
    fn test_parse_with_invalid_json() {
        let content = r#"{"type": "message", "role": "user"}
{invalid json}
{"type": "message", "role": "assistant"}
not a json at all
{"type": "tool_use"}"#;

        let file_path = create_test_file(content);
        let mut parser = JsonlParser::new(file_path.clone()).unwrap();

        // 跳过无效的 JSON 行
        let entries = parser.parse_all().unwrap();
        assert_eq!(entries.len(), 3);

        // 清理
        std::fs::remove_file(file_path).ok();
    }

    #[test]
    fn test_parse_entry_at_offset() {
        let content = r#"{"type": "message", "role": "user"}
{"type": "message", "role": "assistant"}
{"type": "tool_use"}"#;

        let file_path = create_test_file(content);
        let parser = JsonlParser::new(file_path.clone()).unwrap();

        // 第一行从偏移量 0 开始
        let value = parser.parse_entry_at_offset(0, 35).unwrap();
        assert_eq!(value.get("role").unwrap().as_str(), Some("user"));

        // 清理
        std::fs::remove_file(file_path).ok();
    }

    #[test]
    fn test_count_entries() {
        let content = r#"{"type": "message", "role": "user"}
{"type": "message", "role": "assistant"}
{"type": "tool_use"}"#;

        let file_path = create_test_file(content);
        let mut parser = JsonlParser::new(file_path.clone()).unwrap();

        let count = parser.count_entries().unwrap();
        assert_eq!(count, 3);

        // 清理
        std::fs::remove_file(file_path).ok();
    }

    #[test]
    fn test_parse_stream() {
        let content = r#"{"type": "message", "role": "user"}
{"type": "message", "role": "assistant"}
{"type": "tool_use"}"#;

        let file_path = create_test_file(content);
        let mut parser = JsonlParser::new(file_path.clone()).unwrap();

        let mut count = 0;
        parser.parse_stream(|_| {
            count += 1;
        }).unwrap();

        assert_eq!(count, 3);

        // 清理
        std::fs::remove_file(file_path).ok();
    }
}
