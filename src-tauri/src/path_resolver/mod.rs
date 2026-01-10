//! 路径解析模块
//!
//! 将 Windows 文件系统路径转换为 Claude Code projects 目录下的文件夹名称

pub mod converter;
pub mod resolver;

pub use converter::WindowsPathConverter;
pub use resolver::{
    ClaudePathResolver,
    PathConverter,
    PathResolver,
    PathResolveError,
    SessionFileInfo,
    resolve_session_directory,
    session_directory_exists,
    list_session_files,
};

/// 重新导出常用类型
pub type Result<T> = std::result::Result<T, PathResolveError>;
