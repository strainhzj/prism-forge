# Rust 后端实现：辨别 Claude Code 会话文件类型

## 📋 文档概述

本文档指导 Rust 后端开发者如何**辨别 Claude Code 会话文件的类型**，区分主会话文件和 Agent 会话文件。

**关键问题**：`/resume` 命令如何知道该显示哪些 `.jsonl` 文件？

---

## 🎯 会话文件类型

### 1. **主会话文件（Main Session）**

**特征**：
- **文件名格式**：`{UUID}.jsonl`
  - 示例：`7149f370-067c-447e-a7dc-dc161d3f8de7.jsonl`
  - UUID 格式：8-4-4-4-12（36个字符，包含连字符）

- **文件大小**：通常较大（几十 KB 到几百 KB）
  - 示例：852K、195K、422K
  - 行数：通常 100+ 行

- **内容结构**：
  ```json
  {"type":"summary","summary":"会话摘要","leafUuid":"uuid-123"}
  {"type":"file-history-snapshot",...}
  {"type":"user","sessionId":"7149f370-...","message":{...}}
  ```

### 2. **Agent 会话文件（Agent Session）**

**特征**：
- **文件名格式**：`agent-{id}.jsonl`
  - 示例：`agent-eb95d9a3.jsonl`
  - Agent ID：8个字符的十六进制字符串

- **文件大小**：通常很小（1-2 KB）
  - 示例：1.4K、1.7K、1.1K
  - 行数：通常 2-10 行

- **内容结构**：
  ```json
  {"isSidechain":true,"agentId":"eb95d9a3","type":"user",...}
  {"isSidechain":true,"agentId":"eb95d9a3","type":"assistant",...}
  ```

---

## 🔍 辨别方法

### 方法 1：文件名模式匹配（推荐）

**原理**：通过文件名格式快速判断文件类型。

```rust
use std::path::Path;

/// 会话文件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionFileType {
    /// 主会话文件
    Main,
    /// Agent 会话文件
    Agent,
    /// 未知类型
    Unknown,
}

/// 根据文件名判断会话类型
pub fn detect_session_type_by_filename(file_path: impl AsRef<Path>) -> SessionFileType {
    let file_name = file_path.as_ref()
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    // 检查是否是 agent 文件
    if file_name.starts_with("agent-") && file_name.ends_with(".jsonl") {
        return SessionFileType::Agent;
    }

    // 检查是否是主会话文件（UUID 格式）
    if is_uuid_filename(file_name) {
        return SessionFileType::Main;
    }

    SessionFileType::Unknown
}

/// 检查文件名是否是 UUID 格式
fn is_uuid_filename(file_name: &str) -> bool {
    // 去掉 .jsonl 扩展名
    let name_without_ext = file_name.strip_suffix(".jsonl").unwrap_or(file_name);

    // UUID 格式：xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    let uuid_pattern = regex::Regex::new(
        r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    ).unwrap();

    uuid_pattern.is_match(name_without_ext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_main_session() {
        let path = "7149f370-067c-447e-a7dc-dc161d3f8de7.jsonl";
        assert_eq!(
            detect_session_type_by_filename(path),
            SessionFileType::Main
        );
    }

    #[test]
    fn test_detect_agent_session() {
        let path = "agent-eb95d9a3.jsonl";
        assert_eq!(
            detect_session_type_by_filename(path),
            SessionFileType::Agent
        );
    }

    #[test]
    fn test_detect_unknown_session() {
        let path = "random-file.jsonl";
        assert_eq!(
            detect_session_type_by_filename(path),
            SessionFileType::Unknown
        );
    }
}
```

### 方法 2：文件大小过滤（辅助）

**原理**：Agent 文件通常很小，主会话文件较大。

```rust
use std::path::Path;

/// 会话文件配置
pub struct SessionFilterConfig {
    /// 最小文件大小（字节）
    pub min_file_size: u64,
    /// 是否使用文件大小过滤
    pub use_size_filter: bool,
}

impl Default for SessionFilterConfig {
    fn default() -> Self {
        Self {
            // Agent 文件通常 < 5KB，主会话通常 > 10KB
            min_file_size: 10 * 1024, // 10KB
            use_size_filter: true,
        }
    }
}

/// 根据文件大小判断是否可能是主会话
pub fn is_main_session_by_size(
    file_path: impl AsRef<Path>,
    config: &SessionFilterConfig,
) -> bool {
    if !config.use_size_filter {
        return true;
    }

    let metadata = match file_path.as_ref().metadata() {
        Ok(m) => m,
        Err(_) => return false,
    };

    metadata.len() >= config.min_file_size
}
```

### 方法 3：内容解析（最准确）

**原理**：读取文件第一行，检查 `type` 或 `agentId` 字段。

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use serde::Deserialize;

/// 文件第一行的可能结构
#[derive(Deserialize)]
struct SessionFirstLine {
    #[serde(rename = "type")]
    line_type: Option<String>,
    #[serde(rename = "agentId")]
    agent_id: Option<String>,
    #[serde(rename = "isSidechain")]
    is_sidechain: Option<bool>,
}

/// 通过文件内容判断类型（最准确）
pub fn detect_session_type_by_content(
    file_path: impl AsRef<Path>,
) -> Result<SessionFileType, std::io::Error> {
    let file = File::open(file_path.as_ref())?;
    let reader = BufReader::new(file);
    let first_line = reader.lines().next()
        .ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "文件为空"
        ))??;

    // 解析第一行 JSON
    if let Ok(data) = serde_json::from_str::<SessionFirstLine>(&first_line) {
        // 如果有 agentId，则是 agent 文件
        if data.agent_id.is_some() {
            return Ok(SessionFileType::Agent);
        }

        // 如果 type 是 summary，则是主会话
        if data.line_type.as_deref() == Some("summary") {
            return Ok(SessionFileType::Main);
        }
    }

    // 默认返回未知
    Ok(SessionFileType::Unknown)
}
```

---

## 🛠️ 完整实现

### 数据结构

```rust
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 会话文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionFileInfo {
    /// 文件路径
    pub path: PathBuf,
    /// 会话类型
    pub file_type: SessionFileType,
    /// 会话 ID（主会话是 UUID，Agent 是 agent-{id}）
    pub session_id: String,
    /// 文件大小（字节）
    pub file_size: u64,
}

/// 会话类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionFileType {
    /// 主会话文件（UUID.jsonl）
    Main,
    /// Agent 会话文件（agent-{id}.jsonl）
    Agent,
    /// 未知类型
    Unknown,
}

impl SessionFileType {
    /// 是否是主会话
    pub fn is_main(&self) -> bool {
        matches!(self, Self::Main)
    }

    /// 是否是 Agent 会话
    pub fn is_agent(&self) -> bool {
        matches!(self, Self::Agent)
    }
}
```

### 扫描器实现

```rust
use std::fs;
use std::path::{Path, PathBuf};

/// 扫描项目目录，获取所有主会话文件
///
/// # 参数
/// * `project_dir` - 项目目录路径
///
/// # 返回
/// 返回所有主会话文件的信息
pub fn scan_main_sessions(
    project_dir: impl AsRef<Path>,
) -> Result<Vec<SessionFileInfo>, std::io::Error> {
    let project_dir = project_dir.as_ref();
    let mut main_sessions = Vec::new();

    // 读取目录中的所有条目
    for entry in fs::read_dir(project_dir)? {
        let entry = entry?;
        let path = entry.path();

        // 只处理 .jsonl 文件
        if path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
            continue;
        }

        // 获取文件类型
        let file_type = detect_session_type_by_filename(&path);

        // 只要主会话文件
        if file_type.is_main() {
            let metadata = entry.metadata()?;

            // 提取会话 ID
            let session_id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            main_sessions.push(SessionFileInfo {
                path,
                file_type,
                session_id,
                file_size: metadata.len(),
            });
        }
    }

    // 按文件大小排序（可选）
    main_sessions.sort_by(|a, b| b.file_size.cmp(&a.file_size));

    Ok(main_sessions)
}

/// 过滤掉 Agent 会话，只保留主会话
pub fn filter_main_sessions(
    all_files: Vec<PathBuf>,
) -> Vec<SessionFileInfo> {
    all_files
        .into_iter()
        .filter_map(|path| {
            let file_type = detect_session_type_by_filename(&path);

            if file_type.is_main() {
                let metadata = fs::metadata(&path).ok()?;

                Some(SessionFileInfo {
                    session_id: path.file_stem()?.to_str()?.to_string(),
                    path,
                    file_type,
                    file_size: metadata.len(),
                })
            } else {
                None
            }
        })
        .collect()
}
```

### 高级扫描器（带缓存）

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 会话扫描器（带缓存）
pub struct SessionScanner {
    claude_dir: PathBuf,
    cache: Arc<RwLock<HashMap<String, Vec<SessionFileInfo>>>>,
}

impl SessionScanner {
    /// 创建新的扫描器
    pub fn new(claude_dir: impl AsRef<Path>) -> Self {
        Self {
            claude_dir: claude_dir.as_ref().to_path_buf(),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 扫描指定项目的所有主会话
    pub fn scan_project(
        &self,
        project_path: &str,
    ) -> Result<Vec<SessionFileInfo>, std::io::Error> {
        // 检查缓存
        {
            let cache = self.cache.read().unwrap();
            if let Some(cached) = cache.get(project_path) {
                return Ok(cached.clone());
            }
        }

        // 转换项目路径
        let converted_path = convert_project_path(project_path);
        let project_dir = self.claude_dir
            .join("projects")
            .join(converted_path);

        // 扫描主会话
        let sessions = scan_main_sessions(&project_dir)?;

        // 写入缓存
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(project_path.to_string(), sessions.clone());
        }

        Ok(sessions)
    }

    /// 清除缓存
    pub fn clear_cache(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }

    /// 扫描所有项目的主会话
    pub fn scan_all_projects(&self) -> Result<HashMap<String, Vec<SessionFileInfo>>, std::io::Error> {
        let projects_dir = self.claude_dir.join("projects");
        let mut all_sessions = HashMap::new();

        for entry in fs::read_dir(&projects_dir)? {
            let entry = entry?;
            let project_dir = entry.path();

            if !project_dir.is_dir() {
                continue;
            }

            // 提取项目名称
            if let Some(project_name) = project_dir.file_name() {
                let sessions = scan_main_sessions(&project_dir)?;
                if !sessions.is_empty() {
                    all_sessions.insert(
                        project_name.to_string_lossy().to_string(),
                        sessions,
                    );
                }
            }
        }

        Ok(all_sessions)
    }
}

/// 转换项目路径为 Claude 格式
fn convert_project_path(path: &str) -> String {
    path.replace('\\', "-")
        .replace('/', "-")
        .replace(':', "-")
}
```

---

## 📊 使用示例

### 示例 1：扫描项目的主会话

```rust
use claude_session_scanner::{scan_main_sessions, SessionFileType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = "/home/user/.claude/projects/C--software-github-claude-code-main";

    // 扫描主会话
    let sessions = scan_main_sessions(project_dir)?;

    println!("找到 {} 个主会话：", sessions.len());
    for session in sessions {
        println!(
            "- {} ({} bytes)",
            session.session_id,
            session.file_size
        );
    }

    Ok(())
}
```

**输出**：
```
找到 4 个主会话：
- 7149f370-067c-447e-a7dc-dc161d3f8de7 (512000 bytes)
- 2e5a931e-e3b0-48b1-a324-5c841aed7cce (199680 bytes)
- 0bf43974-daf7-4ff1-957a-de72f79556e2 (872448 bytes)
- dd0af197-6a71-427b-960a-bcb2c3821084 (43008 bytes)
```

### 示例 2：过滤 Agent 文件

```rust
use std::fs;
use claude_session_scanner::{detect_session_type_by_filename, SessionFileType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = "/home/user/.claude/projects/C--software-github-claude-code-main";

    let mut main_count = 0;
    let mut agent_count = 0;

    for entry in fs::read_dir(project_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
            continue;
        }

        match detect_session_type_by_filename(&path) {
            SessionFileType::Main => {
                main_count += 1;
                println!("主会话: {}", path.file_name().unwrap().to_string_lossy());
            }
            SessionFileType::Agent => {
                agent_count += 1;
            }
            _ => {}
        }
    }

    println!("\n统计：");
    println!("主会话: {} 个", main_count);
    println!("Agent 会话: {} 个", agent_count);

    Ok(())
}
```

**输出**：
```
主会话: 7149f370-067c-447e-a7dc-dc161d3f8de7.jsonl
主会话: 2e5a931e-e3b0-48b1-a324-5c841aed7cce.jsonl
主会话: 0bf43974-daf7-4ff1-957a-de72f79556e2.jsonl
主会话: dd0af197-6a71-427b-960a-bcb2c3821084.jsonl

统计：
主会话: 4 个
Agent 会话: 12 个
```

### 示例 3：实现 /resume 功能

```rust
use claude_session_scanner::{scan_main_sessions, SessionSummary};
use std::fs;

/// 实现 /resume 命令的会话列表
pub fn list_resume_sessions(
    claude_dir: &str,
    project_path: &str,
) -> Result<Vec<ResumeSessionItem>, Box<dyn std::error::Error>> {
    let converted_path = convert_project_path(project_path);
    let project_dir = format!("{}/projects/{}", claude_dir, converted_path);

    // 扫描主会话
    let session_files = scan_main_sessions(&project_dir)?;

    // 读取每个会话的 summary
    let mut items = Vec::new();
    for session_file in session_files {
        match SessionSummary::from_file(&session_file.path) {
            Ok(summary) => {
                items.push(ResumeSessionItem {
                    session_id: summary.session_id.clone(),
                    display_name: summary.summary,
                    project_path: project_path.to_string(),
                    file_size: session_file.file_size,
                });
            }
            Err(_) => {
                // 如果没有 summary，使用会话 ID 作为显示名称
                items.push(ResumeSessionItem {
                    session_id: session_file.session_id.clone(),
                    display_name: format!("会话 {}", &session_file.session_id[..8]),
                    project_path: project_path.to_string(),
                    file_size: session_file.file_size,
                });
            }
        }
    }

    // 按文件大小排序（最近修改的通常更大）
    items.sort_by(|a, b| b.file_size.cmp(&a.file_size));

    Ok(items)
}

#[derive(Debug, Clone)]
pub struct ResumeSessionItem {
    pub session_id: String,
    pub display_name: String,
    pub project_path: String,
    pub file_size: u64,
}
```

---

## 🧪 测试用例

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        writeln!(file, "{}", content).unwrap();
        path
    }

    #[test]
    fn test_detect_main_session_by_filename() {
        let temp_dir = TempDir::new().unwrap();
        let session_file = create_test_file(
            temp_dir.path(),
            "7149f370-067c-447e-a7dc-dc161d3f8de7.jsonl",
            r#"{"type":"summary","summary":"Test"}"#
        );

        assert_eq!(
            detect_session_type_by_filename(&session_file),
            SessionFileType::Main
        );
    }

    #[test]
    fn test_detect_agent_session_by_filename() {
        let temp_dir = TempDir::new().unwrap();
        let agent_file = create_test_file(
            temp_dir.path(),
            "agent-eb95d9a3.jsonl",
            r#"{"agentId":"eb95d9a3","type":"user"}"#
        );

        assert_eq!(
            detect_session_type_by_filename(&agent_file),
            SessionFileType::Agent
        );
    }

    #[test]
    fn test_scan_main_sessions_only() {
        let temp_dir = TempDir::new().unwrap();

        // 创建主会话
        create_test_file(
            temp_dir.path(),
            "7149f370-067c-447e-a7dc-dc161d3f8de7.jsonl",
            r#"{"type":"summary"}"#
        );

        // 创建 Agent 会话
        create_test_file(
            temp_dir.path(),
            "agent-eb95d9a3.jsonl",
            r#"{"agentId":"eb95d9a3"}"#
        );

        // 扫描应该只返回主会话
        let sessions = scan_main_sessions(temp_dir.path()).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].file_type, SessionFileType::Main);
    }

    #[test]
    fn test_uuid_validation() {
        // 有效的 UUID
        assert!(is_uuid_filename("7149f370-067c-447e-a7dc-dc161d3f8de7.jsonl"));
        assert!(is_uuid_filename("0bf43974-daf7-4ff1-957a-de72f79556e2.jsonl"));

        // 无效的 UUID
        assert!(!is_uuid_filename("agent-eb95d9a3.jsonl"));
        assert!(!is_uuid_filename("not-a-uuid.jsonl"));
        assert!(!is_uuid_filename("random-file.jsonl"));
    }
}
```

---

## 🎯 最佳实践

### 1. **组合使用多种方法**

```rust
pub fn detect_session_type(
    file_path: impl AsRef<Path>,
) -> SessionFileType {
    // 方法1：文件名（最快）
    let filename_type = detect_session_type_by_filename(&file_path);

    // 如果已经明确，直接返回
    if filename_type != SessionFileType::Unknown {
        return filename_type;
    }

    // 方法2：内容解析（最准确）
    detect_session_type_by_content(&file_path)
        .unwrap_or(SessionFileType::Unknown)
}
```

### 2. **性能优化**

```rust
// 并行扫描多个项目
use rayon::prelude::*;

pub fn scan_multiple_projects_parallel(
    project_paths: Vec<String>,
) -> HashMap<String, Vec<SessionFileInfo>> {
    project_paths
        .into_par_iter() // 并行迭代
        .filter_map(|project_path| {
            scan_main_sessions(format!("{}/projects/{}", claude_dir(), project_path))
                .ok()
                .map(|sessions| (project_path, sessions))
        })
        .collect()
}
```

### 3. **错误处理**

```rust
pub fn safe_scan_project(
    project_path: &str,
) -> Vec<SessionFileInfo> {
    let result = scan_main_sessions(format!("{}/projects/{}", claude_dir(), project_path));

    match result {
        Ok(sessions) => sessions,
        Err(e) => {
            tracing::error!("扫描项目 {} 失败: {}", project_path, e);
            Vec::new()
        }
    }
}
```

---

## 📚 依赖配置

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
regex = "1.10"
thiserror = "1.0"
tracing = "0.1"

# 可选：并行处理
rayon = { version = "1.8", optional = true }

[dev-dependencies]
tempfile = "3.8"
```

---

## 🔍 故障排查

### 问题1：UUID 验证失败

**症状**：主会话文件被识别为 `Unknown`

**原因**：UUID 格式不正确或正则表达式有问题

**解决**：
```rust
// 更宽松的 UUID 匹配
fn is_uuid_filename(file_name: &str) -> bool {
    let name = file_name.strip_suffix(".jsonl").unwrap_or(file_name);
    let parts: Vec<&str> = name.split('-').collect();

    parts.len() == 5
        && parts[0].len() == 8
        && parts[1].len() == 4
        && parts[2].len() == 4
        && parts[3].len() == 4
        && parts[4].len() == 12
        && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_hexdigit()))
}
```

### 问题2：Agent 文件被误识别为主会话

**症状**：`agent-xxx.jsonl` 出现在 /resume 列表中

**原因**：文件名匹配逻辑有问题

**解决**：
```rust
// 确保先检查 agent 前缀
pub fn detect_session_type_by_filename(file_path: impl AsRef<Path>) -> SessionFileType {
    let file_name = file_path.as_ref()
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    // 优先检查 agent 前缀
    if file_name.starts_with("agent-") {
        return SessionFileType::Agent;
    }

    // 然后检查 UUID 格式
    if is_uuid_filename(file_name) {
        return SessionFileType::Main;
    }

    SessionFileType::Unknown
}
```

---

## 📊 总结

### 辨别流程图

```
┌─────────────────────────┐
│   读取 .jsonl 文件      │
└───────────┬─────────────┘
            │
            ▼
┌─────────────────────────┐
│   检查文件名格式        │
└───────────┬─────────────┘
            │
     ┌──────┴──────┐
     │             │
     ▼             ▼
┌─────────┐   ┌─────────┐
│agent-xxx│   │ UUID格式 │
│.jsonl   │   │ .jsonl  │
└────┬────┘   └────┬────┘
     │             │
     ▼             ▼
┌─────────┐   ┌─────────┐
│ Agent   │   │  Main   │
│ Session │   │ Session │
└─────────┘   └─────────┘
```

### 关键点

1. ✅ **文件名是最可靠的辨别方法**
   - `agent-{id}.jsonl` → Agent
   - `{UUID}.jsonl` → Main

2. ✅ **文件大小作为辅助验证**
   - Agent: 通常 < 5KB
   - Main: 通常 > 10KB

3. ✅ **内容解析作为最后手段**
   - 检查第一行的 `type` 或 `agentId` 字段

---

**文档版本**: 1.0.0
**最后更新**: 2025-01-10
**维护者**: Claude Code 开发团队
