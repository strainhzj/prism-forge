# Claude Code Projects 路径解析模块 - Rust 实现指南

> **文档版本**: v1.0
> **创建日期**: 2025-01-09
> **目标语言**: Rust 1.70+
> **适用场景**: Claude Code Session Monitor 后端开发

---

## 📋 目录

1. [功能概述](#1-功能概述)
2. [路径转换算法](#2-路径转换算法)
3. [Rust 实现方案](#3-rust-实现方案)
4. [完整代码示例](#4-完整代码示例)
5. [测试用例](#5-测试用例)
6. [边界情况处理](#6-边界情况处理)
7. [性能优化建议](#7-性能优化建议)
8. [集成指南](#8-集成指南)

---

## 1. 功能概述

### 1.1 核心需求

实现一个 Rust 模块，用于**将 Windows 文件系统路径转换为 Claude Code projects 目录下的文件夹名称**，并支持反向查找。

### 1.2 功能规格

| 功能 | 输入 | 输出 | 用途 |
|------|------|------|------|
| **路径转文件夹名** | `C:\software\Java\project` | `C--software-Java-project` | 定位 projects 目录下的会话文件夹 |
| **文件夹名还原** | `C--software-Java-project` | `C:\software\Java\project` | 从文件夹名还原原始路径（可选） |
| **验证文件夹存在** | `C:\software\Java\project` | `bool` | 检查项目是否有历史会话 |
| **获取完整路径** | `C:\software\Java\project` | `C:\Users\{user}\.claude\projects\C--software-Java-project` | 构建会话文件夹的绝对路径 |

### 1.3 使用场景

```rust
// 场景 1: 查找项目的会话历史
let project_path = r"C:\software\Java\IoTWebApi-Evlink-Automatic-master";
let session_dir = resolve_session_directory(project_path)?;
// 结果: C:\Users\thoma\.claude\projects\C--software-Java-IoTWebApi-Evlink-Automatic-master

// 场景 2: 列出所有会话文件
let sessions = list_session_files(&session_dir)?;

// 场景 3: 从文件夹名还原路径
let folder_name = "C--software-Java-project";
let original_path = restore_original_path(folder_name)?;
// 结果: C:\software\Java\project
```

---

## 2. 路径转换算法

### 2.1 转换规则

#### **Windows 路径 → 文件夹名称**

```
原始路径: C:\software\Java\IoTWebApi-Evlink-Automatic-master\to\java\src
转换后:   C--software-Java-IoTWebApi-Evlink-Automatic-master-to-java-src
```

**步骤:**

1. **驱动器字母处理**: `C:` → `C-`
2. **反斜杠替换**: `\` → `-`
3. **移除其他冒号**: 删除驱动器字母后的 `:`
4. **保留空格和特殊字符**: 空格、点、下划线等保留

#### **伪代码**

```python
def path_to_foldername(path: str) -> str:
    # 1. 处理驱动器字母
    if path matches r"^([A-Z]):":
        path = path.replace(match, "${1}-")

    # 2. 替换反斜杠
    path = path.replace("\\", "-")

    # 3. 移除其他冒号（如果还有）
    path = path.replace(":", "")

    return path
```

### 2.2 转换示例

| 原始路径 | 文件夹名称 | 说明 |
|---------|-----------|------|
| `C:\software\Java\project` | `C--software-Java-project` | 标准路径 |
| `D:\Projects\My App\v2.0` | `D--Projects-My App-v2.0` | 保留空格和点 |
| `C:\Users\张三\Desktop` | `C--Users-张三-Desktop` | 保留中文字符 |
| `C:\path\to\project` | `C--path-to-project` | `\to\` → `-to-` |
| `\\network\share\folder` | `-network-share-folder` | UNC 路径（可选支持） |

### 2.3 特殊字符处理

| 字符 | 处理方式 | 示例 |
|------|---------|------|
| `:` (驱动器后) | 替换为 `-` | `C:` → `C-` |
| `\` | 替换为 `-` | `\path\` → `-path-` |
| `:` (其他位置) | 移除 | `port:8080` → `port8080` |
| 空格 | 保留 | `My Project` → `My Project` |
| 中文 | 保留 | `项目` → `项目` |
| `.` | 保留 | `v1.0.0` → `v1.0.0` |
| `_` | 保留 | `my_project` → `my_project` |

---

## 3. Rust 实现方案

### 3.1 模块结构

```
src/
├── path_resolver/
│   ├── mod.rs              # 模块导出
│   ├── converter.rs        # 路径转换算法
│   ├── resolver.rs         # 路径解析逻辑
│   └── validator.rs        # 路径验证工具
```

### 3.2 依赖项

```toml
[dependencies]
# 路径处理
dirs = "5.0"                    # 获取用户主目录
thiserror = "1.0"               # 错误处理
regex = "1.10"                  # 正则表达式（可选）

# 序列化（可选）
serde = { version = "1.0", features = ["derive"] }
```

### 3.3 核心 Trait 定义

```rust
use std::path::{Path, PathBuf};
use thiserror::Error;

/// 路径解析错误
#[derive(Error, Debug)]
pub enum PathResolveError {
    #[error("无效的 Windows 路径: {0}")]
    InvalidPath(String),

    #[error("无法获取用户主目录")]
    HomeDirNotFound,

    #[error("Projects 目录不存在: {0}")]
    ProjectsDirNotFound(PathBuf),

    #[error("文件夹名称格式错误: {0}")]
    InvalidFolderName(String),
}

/// 路径转换器 Trait
pub trait PathConverter {
    /// 将 Windows 路径转换为文件夹名称
    fn path_to_folder_name(&self, path: &Path) -> Result<String, PathResolveError>;

    /// 从文件夹名称还原原始路径
    fn folder_name_to_path(&self, folder_name: &str) -> Result<PathBuf, PathResolveError>;
}

/// 路径解析器 Trait
pub trait PathResolver {
    /// 解析项目的会话目录路径
    fn resolve_session_dir(&self, project_path: &Path) -> Result<PathBuf, PathResolveError>;

    /// 检查会话目录是否存在
    fn session_dir_exists(&self, project_path: &Path) -> Result<bool, PathResolveError>;

    /// 列出会话目录中的所有会话文件
    fn list_session_files(&self, project_path: &Path) -> Result<Vec<PathBuf>, PathResolveError>;
}
```

---

## 4. 完整代码示例

### 4.1 converter.rs - 路径转换实现

```rust
use std::path::Path;
use super::{PathConverter, PathResolveError};

/// Windows 路径转换器
pub struct WindowsPathConverter;

impl WindowsPathConverter {
    pub fn new() -> Self {
        Self
    }

    /// 处理驱动器字母和冒号
    fn normalize_drive_letter(path: &str) -> String {
        // 匹配 "C:" 模式并替换为 "C-"
        let re = regex::Regex::new(r"^([A-Z]):").unwrap();
        re.replace(path, "$1-").to_string()
    }

    /// 替换反斜杠为连字符
    fn replace_backslashes(path: &str) -> String {
        path.replace('\\', "-")
    }

    /// 移除其他位置的冒号
    fn remove_colons(path: &str) -> String {
        path.replace(':', "")
    }
}

impl PathConverter for WindowsPathConverter {
    fn path_to_folder_name(&self, path: &Path) -> Result<String, PathResolveError> {
        let path_str = path.to_str()
            .ok_or_else(|| PathResolveError::InvalidPath("路径包含无效字符".to_string()))?;

        // 步骤 1: 处理驱动器字母
        let normalized = Self::normalize_drive_letter(path_str);

        // 步骤 2: 替换反斜杠
        let replaced = Self::replace_backslashes(&normalized);

        // 步骤 3: 移除其他冒号
        let cleaned = Self::remove_colons(&replaced);

        Ok(cleaned)
    }

    fn folder_name_to_path(&self, folder_name: &str) -> Result<std::path::PathBuf, PathResolveError> {
        // 验证格式: 必须以字母加双连字符开头
        if !folder_name.chars().next().map_or(false, |c| c.is_ascii_uppercase()) {
            return Err(PathResolveError::InvalidFolderName(
                "文件夹名称必须以大写字母开头".to_string()
            ));
        }

        // 还原驱动器字母: "C-" -> "C:"
        let path = if let Some(rest) = folder_name.strip_prefix("C-") {
            format!("C:{}", rest)
        } else if let Some(rest) = folder_name.strip_prefix("D-") {
            format!("D:{}", rest)
        } else if let Some(rest) = folder_name.strip_prefix("E-") {
            format!("E:{}", rest)
        } else {
            // UNC 路径或其他格式
            folder_name.to_string()
        };

        // 还原连字符为反斜杠
        let restored = path.replace('-', "\\");

        Ok(std::path::PathBuf::from(restored))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_path() {
        let converter = WindowsPathConverter::new();
        let path = Path::new(r"C:\software\Java\project");
        let result = converter.path_to_folder_name(path).unwrap();
        assert_eq!(result, "C--software-Java-project");
    }

    #[test]
    fn test_path_with_to() {
        let converter = WindowsPathConverter::new();
        let path = Path::new(r"C:\path\to\project");
        let result = converter.path_to_folder_name(path).unwrap();
        assert_eq!(result, "C--path-to-project");
    }

    #[test]
    fn test_restore_path() {
        let converter = WindowsPathConverter::new();
        let folder_name = "C--software-Java-project";
        let result = converter.folder_name_to_path(folder_name).unwrap();
        assert_eq!(result, Path::new(r"C:\software\Java\project"));
    }
}
```

### 4.2 resolver.rs - 路径解析实现

```rust
use std::path::{Path, PathBuf};
use std::fs;
use dirs::home_dir;
use super::{PathResolver, PathResolveError};
use super::converter::WindowsPathConverter;

/// Claude Code 路径解析器
pub struct ClaudePathResolver {
    converter: WindowsPathConverter,
    projects_base_dir: PathBuf,
}

impl ClaudePathResolver {
    /// 创建新的解析器
    pub fn new() -> Result<Self, PathResolveError> {
        let home = home_dir()
            .ok_or(PathResolveError::HomeDirNotFound)?;

        let projects_base_dir = home.join(".claude").join("projects");

        Ok(Self {
            converter: WindowsPathConverter::new(),
            projects_base_dir,
        })
    }

    /// 使用自定义基础目录创建解析器（用于测试）
    pub fn with_base_dir(base_dir: PathBuf) -> Self {
        Self {
            converter: WindowsPathConverter::new(),
            projects_base_dir: base_dir,
        }
    }

    /// 获取 projects 基础目录
    pub fn projects_dir(&self) -> &Path {
        &self.projects_base_dir
    }

    /// 构建会话目录的完整路径
    fn build_session_dir_path(&self, folder_name: &str) -> PathBuf {
        self.projects_base_dir.join(folder_name)
    }

    /// 列出目录中的所有 .jsonl 文件
    fn list_jsonl_files(&self, dir: &Path) -> Result<Vec<PathBuf>, PathResolveError> {
        let mut sessions = Vec::new();

        if !dir.exists() {
            return Ok(sessions);
        }

        let entries = fs::read_dir(dir)
            .map_err(|e| PathResolveError::InvalidPath(format!("无法读取目录: {}", e)))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| PathResolveError::InvalidPath(format!("无法读取目录项: {}", e)))?;

            let path = entry.path();

            // 检查是否为 .jsonl 文件
            if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                sessions.push(path);
            }
        }

        sessions.sort(); // 按文件名排序
        Ok(sessions)
    }
}

impl PathResolver for ClaudePathResolver {
    fn resolve_session_dir(&self, project_path: &Path) -> Result<PathBuf, PathResolveError> {
        // 步骤 1: 转换路径为文件夹名称
        let folder_name = self.converter.path_to_folder_name(project_path)?;

        // 步骤 2: 构建完整路径
        let session_dir = self.build_session_dir_path(&folder_name);

        Ok(session_dir)
    }

    fn session_dir_exists(&self, project_path: &Path) -> Result<bool, PathResolveError> {
        let session_dir = self.resolve_session_dir(project_path)?;
        Ok(session_dir.exists())
    }

    fn list_session_files(&self, project_path: &Path) -> Result<Vec<PathBuf>, PathResolveError> {
        let session_dir = self.resolve_session_dir(project_path)?;
        self.list_jsonl_files(&session_dir)
    }
}

/// 便捷函数：解析会话目录
pub fn resolve_session_directory(project_path: &Path) -> Result<PathBuf, PathResolveError> {
    let resolver = ClaudePathResolver::new()?;
    resolver.resolve_session_dir(project_path)
}

/// 便捷函数：检查会话目录是否存在
pub fn session_directory_exists(project_path: &Path) -> Result<bool, PathResolveError> {
    let resolver = ClaudePathResolver::new()?;
    resolver.session_dir_exists(project_path)
}

/// 便捷函数：列出会话文件
pub fn list_session_files(project_path: &Path) -> Result<Vec<PathBuf>, PathResolveError> {
    let resolver = ClaudePathResolver::new()?;
    resolver.list_session_files(project_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::create_dir_all;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_session_dir() {
        let temp_dir = TempDir::new().unwrap();
        let resolver = ClaudePathResolver::with_base_dir(temp_dir.path().to_path_buf());

        let project_path = Path::new(r"C:\software\Java\project");
        let session_dir = resolver.resolve_session_dir(project_path).unwrap();

        let expected = temp_dir.path().join("C--software-Java-project");
        assert_eq!(session_dir, expected);
    }

    #[test]
    fn test_session_dir_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let resolver = ClaudePathResolver::with_base_dir(temp_dir.path().to_path_buf());

        let project_path = Path::new(r"C:\nonexistent\project");
        let exists = resolver.session_dir_exists(project_path).unwrap();
        assert!(!exists);
    }

    #[test]
    fn test_list_session_files() {
        let temp_dir = TempDir::new().unwrap();
        let session_dir = temp_dir.path().join("C--software-Java-project");
        create_dir_all(&session_dir).unwrap();

        // 创建测试文件
        fs::write(session_dir.join("session-1.jsonl"), "test").unwrap();
        fs::write(session_dir.join("session-2.jsonl"), "test").unwrap();
        fs::write(session_dir.join("other.txt"), "test").unwrap();

        let resolver = ClaudePathResolver::with_base_dir(temp_dir.path().to_path_buf());
        let project_path = Path::new(r"C:\software\Java\project");
        let sessions = resolver.list_session_files(project_path).unwrap();

        assert_eq!(sessions.len(), 2);
        assert!(sessions[0].to_string_lossy().contains("session-1.jsonl"));
        assert!(sessions[1].to_string_lossy().contains("session-2.jsonl"));
    }
}
```

### 4.3 mod.rs - 模块导出

```rust
mod converter;
mod resolver;
mod validator;

pub use converter::WindowsPathConverter;
pub use resolver::{
    ClaudePathResolver,
    PathConverter,
    PathResolver,
    PathResolveError,
    resolve_session_directory,
    session_directory_exists,
    list_session_files,
};
pub use validator::PathValidator;

/// 重新导出常用类型
pub type Result<T> = std::result::Result<T, PathResolveError>;
```

---

## 5. 测试用例

### 5.1 单元测试

```rust
#[cfg(test)]
mod comprehensive_tests {
    use super::*;

    /// 测试标准路径转换
    #[test]
    fn test_standard_path_conversion() {
        let test_cases = vec![
            (r"C:\software\Java\project", "C--software-Java-project"),
            (r"D:\Projects\MyApp\v1.0", "D--Projects-MyApp-v1.0"),
            (r"C:\Users\张三\Desktop", "C--Users-张三-Desktop"),
            (r"C:\path\to\project", "C--path-to-project"),
        ];

        let converter = WindowsPathConverter::new();

        for (input, expected) in test_cases {
            let path = Path::new(input);
            let result = converter.path_to_folder_name(path).unwrap();
            assert_eq!(result, expected, "输入: {}", input);
        }
    }

    /// 测试特殊字符处理
    #[test]
    fn test_special_characters() {
        let test_cases = vec![
            (r"C:\project with spaces", "C--project with spaces"),
            (r"C:\project_with_underscores", "C--project_with_underscores"),
            (r"C:\project.with.dots", "C--project.with.dots"),
            (r"C:\project--double--dash", "C--project--double--dash"),
        ];

        let converter = WindowsPathConverter::new();

        for (input, expected) in test_cases {
            let path = Path::new(input);
            let result = converter.path_to_folder_name(path).unwrap();
            assert_eq!(result, expected, "输入: {}", input);
        }
    }

    /// 测试反向转换
    #[test]
    fn test_reverse_conversion() {
        let test_cases = vec![
            ("C--software-Java-project", r"C:\software\Java\project"),
            ("D--Projects-MyApp-v1.0", r"D:\Projects\MyApp\v1.0"),
        ];

        let converter = WindowsPathConverter::new();

        for (folder_name, expected_path) in test_cases {
            let result = converter.folder_name_to_path(folder_name).unwrap();
            assert_eq!(result, Path::new(expected_path));
        }
    }

    /// 测试错误处理
    #[test]
    fn test_error_handling() {
        let converter = WindowsPathConverter::new();

        // 无效的驱动器字母
        let invalid_path = Path::new("X:\\invalid");
        assert!(converter.path_to_folder_name(invalid_path).is_ok());

        // 空路径
        let empty_path = Path::new("");
        // 这里应该处理空路径的情况
    }
}
```

### 5.2 集成测试

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::{self, File};
    use std::io::Write;

    /// 创建完整的测试环境
    fn setup_test_environment() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let projects_dir = temp_dir.path().join(".claude").join("projects");
        fs::create_dir_all(&projects_dir).unwrap();

        // 创建测试会话目录
        let session_dir = projects_dir.join("C--software-Java-test-project");
        fs::create_dir_all(&session_dir).unwrap();

        // 创建测试会话文件
        let mut file = File::create(session_dir.join("session-1.jsonl")).unwrap();
        writeln!(file, r#"{{"type": "user", "content": "test"}}"#).unwrap();

        temp_dir
    }

    #[test]
    fn test_end_to_end_workflow() {
        let temp_dir = setup_test_environment();
        let resolver = ClaudePathResolver::with_base_dir(
            temp_dir.path().join(".claude").join("projects")
        );

        // 1. 解析会话目录
        let project_path = Path::new(r"C:\software\Java\test-project");
        let session_dir = resolver.resolve_session_dir(project_path).unwrap();
        assert!(session_dir.exists());

        // 2. 检查目录存在
        let exists = resolver.session_dir_exists(project_path).unwrap();
        assert!(exists);

        // 3. 列出会话文件
        let sessions = resolver.list_session_files(project_path).unwrap();
        assert_eq!(sessions.len(), 1);
        assert!(sessions[0].ends_with("session-1.jsonl"));
    }

    #[test]
    fn test_nonexistent_project() {
        let temp_dir = TempDir::new().unwrap();
        let resolver = ClaudePathResolver::with_base_dir(temp_dir.path().to_path_buf());

        let project_path = Path::new(r"C:\nonexistent\project");
        let exists = resolver.session_dir_exists(project_path).unwrap();
        assert!(!exists);

        let sessions = resolver.list_session_files(project_path).unwrap();
        assert!(sessions.is_empty());
    }
}
```

---

## 6. 边界情况处理

### 6.1 特殊路径格式

| 情况 | 处理方式 | 示例 |
|------|---------|------|
| **UNC 路径** | 移除前导 `\\` | `\\server\share` → `-server-share` |
| **相对路径** | 拒绝或转换为绝对路径 | `.\project` → 返回错误 |
| **超长路径** | 正常处理（Windows 支持 260+ 字符） | `C:\very\long\path...` |
| **包含 Unicode** | 保留 Unicode 字符 | `C:\项目\路径` → `C--项目-路径` |
| **连续分隔符** | 转换为连续连字符 | `C:\\path\\to` → `C---path--to` |

### 6.2 错误处理策略

```rust
impl PathConverter for WindowsPathConverter {
    fn path_to_folder_name(&self, path: &Path) -> Result<String, PathResolveError> {
        // 验证路径
        if path.as_os_str().is_empty() {
            return Err(PathResolveError::InvalidPath("路径不能为空".to_string()));
        }

        // 转换为字符串
        let path_str = path.to_str()
            .ok_or_else(|| PathResolveError::InvalidPath("路径包含无效 UTF-8 字符".to_string()))?;

        // 验证 Windows 路径格式
        if !path_str.contains(':') && !path_str.starts_with('\\') {
            return Err(PathResolveError::InvalidPath(
                "不是有效的 Windows 路径".to_string()
            ));
        }

        // 执行转换...
        Ok(folder_name)
    }
}
```

### 6.3 并发安全

```rust
use std::sync::Arc;

/// 线程安全的路径解析器
pub struct ThreadSafePathResolver {
    inner: Arc<ClaudePathResolver>,
}

impl Clone for ThreadSafePathResolver {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl ThreadSafePathResolver {
    pub fn new() -> Result<Self, PathResolveError> {
        Ok(Self {
            inner: Arc::new(ClaudePathResolver::new()?),
        })
    }
}

// 可以安全地跨线程共享
```

---

## 7. 性能优化建议

### 7.1 缓存策略

```rust
use std::collections::HashMap;
use lru::LruCache;

/// 带缓存的路径解析器
pub struct CachedPathResolver {
    resolver: ClaudePathResolver,
    cache: LruCache<String, PathBuf>, // folder_name -> session_dir
}

impl CachedPathResolver {
    pub fn new(capacity: usize) -> Result<Self, PathResolveError> {
        Ok(Self {
            resolver: ClaudePathResolver::new()?,
            cache: LruCache::new(std::num::NonZeroUsize::new(capacity).unwrap()),
        })
    }

    pub fn resolve_session_dir_cached(&mut self, project_path: &Path) -> Result<PathBuf, PathResolveError> {
        // 尝试从缓存获取
        let key = project_path.to_string_lossy().to_string();
        if let Some(cached) = self.cache.get(&key) {
            return Ok(cached.clone());
        }

        // 缓存未命中，执行解析
        let result = self.resolver.resolve_session_dir(project_path)?;

        // 存入缓存
        self.cache.put(key, result.clone());

        Ok(result)
    }
}
```

### 7.2 批量处理

```rust
/// 批量解析多个项目的会话目录
pub fn batch_resolve_session_dirs(
    project_paths: &[&Path]
) -> Result<Vec<(PathBuf, Option<PathBuf>)>, PathResolveError> {
    let resolver = ClaudePathResolver::new()?;

    project_paths
        .iter()
        .map(|&path| {
            match resolver.resolve_session_dir(path) {
                Ok(session_dir) => Ok((path.to_path_buf(), Some(session_dir))),
                Err(_) => Ok((path.to_path_buf(), None)),
            }
        })
        .collect()
}

// 使用示例
let projects = vec![
    Path::new(r"C:\project1"),
    Path::new(r"C:\project2"),
    Path::new(r"C:\project3"),
];

let results = batch_resolve_session_dirs(&projects)?;
```

### 7.3 性能基准

```rust
#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::time::Instant;

    #[test]
    fn benchmark_path_conversion() {
        let converter = WindowsPathConverter::new();
        let path = Path::new(r"C:\software\Java\very\long\path\to\project");

        let start = Instant::now();
        for _ in 0..10_000 {
            let _ = converter.path_to_folder_name(path);
        }
        let duration = start.elapsed();

        println!("10,000 次转换耗时: {:?}", duration);
        // 预期: < 100ms
    }
}
```

---

## 8. 集成指南

### 8.1 Tauri 后端集成

```rust
// src-tauri/src/commands/path_commands.rs
use crate::path_resolver::{resolve_session_directory, list_session_files, PathResolveError};

#[tauri::command]
pub async fn resolve_session_dir(project_path: String) -> Result<String, String> {
    let path = std::path::Path::new(&project_path);
    match resolve_session_directory(path) {
        Ok(session_dir) => Ok(session_dir.to_string_lossy().to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn get_session_files(project_path: String) -> Result<Vec<String>, String> {
    let path = std::path::Path::new(&project_path);
    match list_session_files(path) {
        Ok(files) => {
            let file_paths: Vec<String> = files
                .into_iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            Ok(file_paths)
        },
        Err(e) => Err(e.to_string()),
    }
}

// 注册命令
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            resolve_session_dir,
            get_session_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 8.2 前端调用 (React/TypeScript)

```typescript
// src/api/path-resolver.ts
import { invoke } from '@tauri-apps/api/tauri';

export interface SessionFile {
  path: string;
  name: string;
}

/**
 * 解析项目的会话目录
 */
export async function resolveSessionDir(projectPath: string): Promise<string> {
  return invoke<string>('resolve_session_dir', { projectPath });
}

/**
 * 获取项目的所有会话文件
 */
export async function getSessionFiles(projectPath: string): Promise<SessionFile[]> {
  const paths = await invoke<string[]>('get_session_files', { projectPath });
  return paths.map(path => ({
    path,
    name: path.split('\\').pop() || path,
  }));
}

// 使用示例
async function loadProjectSessions(projectPath: string) {
  try {
    const sessionDir = await resolveSessionDir(projectPath);
    console.log('会话目录:', sessionDir);

    const sessionFiles = await getSessionFiles(projectPath);
    console.log('会话文件:', sessionFiles);
  } catch (error) {
    console.error('加载失败:', error);
  }
}
```

### 8.3 完整使用示例

```rust
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建解析器
    let resolver = ClaudePathResolver::new()?;

    // 2. 解析项目路径
    let project_path = Path::new(r"C:\software\Java\IoTWebApi-Evlink-Automatic-master");
    let session_dir = resolver.resolve_session_dir(project_path)?;
    println!("会话目录: {}", session_dir.display());

    // 3. 检查是否存在
    if resolver.session_dir_exists(project_path)? {
        println!("项目有历史会话");

        // 4. 列出会话文件
        let sessions = resolver.list_session_files(project_path)?;
        println!("找到 {} 个会话:", sessions.len());
        for session in sessions {
            println!("  - {}", session.display());
        }
    } else {
        println!("项目无历史会话");
    }

    Ok(())
}
```

---

## 9. 附录

### 9.1 完整依赖配置

```toml
[package]
name = "claude-path-resolver"
version = "0.1.0"
edition = "2021"

[dependencies]
dirs = "5.0"
thiserror = "1.0"
regex = "1.10"
serde = { version = "1.0", features = ["derive"], optional = true }

[dev-dependencies]
tempfile = "3.8"

[features]
default = []
serde = ["dep:serde"]
```

### 9.2 API 快速参考

| 函数 | 输入 | 输出 | 说明 |
|------|------|------|------|
| `resolve_session_directory(path)` | `&Path` | `Result<PathBuf>` | 解析会话目录路径 |
| `session_directory_exists(path)` | `&Path` | `Result<bool>` | 检查会话目录是否存在 |
| `list_session_files(path)` | `&Path` | `Result<Vec<PathBuf>>` | 列出会话文件 |
| `path_to_folder_name(path)` | `&Path` | `Result<String>` | 转换路径为文件夹名 |
| `folder_name_to_path(name)` | `&str` | `Result<PathBuf>` | 还原文件夹名为路径 |

### 9.3 常见问题

**Q: 如何处理非 Windows 平台？**
```rust
#[cfg(unix)]
fn path_to_folder_name(path: &Path) -> Result<String, PathResolveError> {
    // Unix 平台直接使用路径
    Ok(path.to_string_lossy().to_string())
}
```

**Q: 如何支持长路径（>260 字符）？**
```rust
use std::os::windows::fs::OpenOptionsExt;

// 使用 UNC 前缀支持长路径
fn enable_long_paths() {
    // Windows 10 1607+ 自动支持长路径
}
```

**Q: 如何处理中文路径？**
```rust
// Windows 路径转换器已支持 Unicode
let path = Path::new(r"C:\软件\项目");
let result = converter.path_to_folder_name(path)?;
// 结果: "C--软件-项目"
```

---

## 📞 技术支持

如有问题，请参考：
- **源码位置**: `src/path_resolver/`
- **测试文件**: `src/path_resolver/tests/`
- **文档更新**: 请提交 PR 更新本文档

---

**文档结束** | 最后更新: 2025-01-09
