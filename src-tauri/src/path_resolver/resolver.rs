//! Claude Code 路径解析器
//!
//! 根据项目路径解析对应的会话目录

use std::path::{Path, PathBuf};
use std::fs;
use dirs::home_dir;
use super::converter::WindowsPathConverter;
use crate::session_type_detector::{SessionFileType, detect_session_type_by_filename, is_snapshot_file};

/// 路径解析错误
#[derive(Debug, thiserror::Error)]
pub enum PathResolveError {
    #[error("无效的 Windows 路径: {0}")]
    InvalidPath(String),

    #[error("无法获取用户主目录")]
    HomeDirNotFound,

    #[error("Projects 目录不存在: {0}")]
    ProjectsDirNotFound(PathBuf),

    #[error("文件夹名称格式错误: {0}")]
    InvalidFolderName(String),

    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
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

    /// 列出会话目录中的所有会话文件（按修改时间倒序）
    fn list_session_files_sorted(&self, project_path: &Path) -> Result<Vec<SessionFileInfo>, PathResolveError>;
}

/// 会话文件信息
#[derive(Debug, Clone)]
pub struct SessionFileInfo {
    /// 文件名（不含扩展名）
    pub file_name: String,
    /// 完整路径
    pub full_path: PathBuf,
    /// 文件大小（字节）
    pub file_size: u64,
    /// 修改时间（RFC3339）
    pub modified_time: String,
    /// 会话摘要（如果可用）
    pub summary: Option<String>,
    /// 会话文件类型
    pub file_type: SessionFileType,
}

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

        #[cfg(debug_assertions)]
        eprintln!("[PathResolver] Projects 基础目录: {:?}", projects_base_dir);

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

    /// 列出目录中的所有 .jsonl 文件并按修改时间排序
    fn list_jsonl_files_sorted(&self, dir: &Path) -> Result<Vec<SessionFileInfo>, PathResolveError> {
        let mut sessions = Vec::new();

        if !dir.exists() {
            #[cfg(debug_assertions)]
            eprintln!("[PathResolver] 目录不存在: {:?}", dir);
            return Ok(sessions);
        }

        #[cfg(debug_assertions)]
        eprintln!("[PathResolver] 扫描目录: {:?}", dir);

        let entries = fs::read_dir(dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // 检查是否为 .jsonl 文件
            if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                // 过滤掉快照文件（file-history-snapshot）
                if is_snapshot_file(&path) {
                    #[cfg(debug_assertions)]
                    eprintln!("[PathResolver] 过滤掉快照文件: {:?}", path.file_name());
                    continue;
                }

                let metadata = entry.metadata()?;

                // 获取修改时间
                let modified = metadata.modified()?;
                let modified_time = system_time_to_rfc3339(modified)?;

                // 获取文件名（不含扩展名）
                let file_name = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();

                // 检测文件类型
                let file_type = detect_session_type_by_filename(&path);

                sessions.push(SessionFileInfo {
                    file_name,
                    full_path: path,
                    file_size: metadata.len(),
                    modified_time,
                    summary: None, // 稍后异步加载
                    file_type,
                });
            }
        }

        // 按修改时间倒序排序（最新的在前）
        sessions.sort_by(|a, b| b.modified_time.cmp(&a.modified_time));

        #[cfg(debug_assertions)]
        eprintln!("[PathResolver] 找到 {} 个会话文件", sessions.len());

        Ok(sessions)
    }
}

impl PathResolver for ClaudePathResolver {
    fn resolve_session_dir(&self, project_path: &Path) -> Result<PathBuf, PathResolveError> {
        #[cfg(debug_assertions)]
        eprintln!("[PathResolver] 解析项目路径: {:?}", project_path);

        // 步骤 1: 转换路径为文件夹名称
        let folder_name = self.converter.path_to_folder_name(project_path)?;

        // 步骤 2: 构建完整路径
        let session_dir = self.build_session_dir_path(&folder_name);

        #[cfg(debug_assertions)]
        eprintln!("[PathResolver] 会话目录: {:?}", session_dir);

        Ok(session_dir)
    }

    fn session_dir_exists(&self, project_path: &Path) -> Result<bool, PathResolveError> {
        let session_dir = self.resolve_session_dir(project_path)?;
        Ok(session_dir.exists())
    }

    fn list_session_files_sorted(&self, project_path: &Path) -> Result<Vec<SessionFileInfo>, PathResolveError> {
        let session_dir = self.resolve_session_dir(project_path)?;
        self.list_jsonl_files_sorted(&session_dir)
    }
}

/// 将 SystemTime 转换为 RFC3339 格式字符串
fn system_time_to_rfc3339(time: std::time::SystemTime) -> Result<String, PathResolveError> {
    use chrono::{DateTime, Utc};

    // 将 SystemTime 转换为 DateTime<Utc>
    let datetime: DateTime<Utc> = time.into();

    // 格式化为 RFC3339 (包含毫秒)
    Ok(datetime.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
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

/// 便捷函数：列出会话文件（按修改时间倒序）
pub fn list_session_files(project_path: &Path) -> Result<Vec<SessionFileInfo>, PathResolveError> {
    let resolver = ClaudePathResolver::new()?;
    resolver.list_session_files_sorted(project_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
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
        fs::create_dir_all(&session_dir).unwrap();

        // 创建测试文件
        File::create(session_dir.join("session-1.jsonl")).unwrap();
        File::create(session_dir.join("session-2.jsonl")).unwrap();
        File::create(session_dir.join("other.txt")).unwrap();

        let resolver = ClaudePathResolver::with_base_dir(temp_dir.path().to_path_buf());
        let project_path = Path::new(r"C:\software\Java\project");
        let sessions = resolver.list_session_files_sorted(project_path).unwrap();

        assert_eq!(sessions.len(), 2);
        assert!(sessions[0].file_name.contains("session"));
    }
}
