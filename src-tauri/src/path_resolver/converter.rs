//! Windows 路径转换器
//!
//! 将 Windows 路径转换为 Claude Code projects 文件夹名称
//!
//! 示例：
//! - C:\software\Java\project → C--software-Java-project
//! - D:\Projects\My App\v2.0 → D--Projects-My App-v2.0

use std::path::Path;
use super::{PathConverter, PathResolveError};

/// Windows 路径转换器
pub struct WindowsPathConverter;

impl WindowsPathConverter {
    pub fn new() -> Self {
        Self
    }

    /// 处理驱动器字母和冒号
    /// C: → C-
    fn normalize_drive_letter(path: &str) -> String {
        // 匹配 "C:" 模式并替换为 "C-"
        if let Some(idx) = path.find(':') {
            if idx == 1 && path.chars().next().map_or(false, |c| c.is_ascii_uppercase()) {
                // 这是驱动器字母格式 (C:, D:, etc.)
                let mut result = String::from(path);
                result.remove(idx); // 移除冒号
                result.insert(idx, '-'); // 插入连字符
                return result;
            }
        }
        path.to_string()
    }

    fn replace_backslashes(path: &str) -> String {
        let with_dashes = path.replace('\\', "-").replace('/', "-");
        // 再替换下划线为连字符（与 Claude Code 的转换逻辑保持一致）
        with_dashes.replace('_', "-")
    }

    /// 移除其他位置的冒号（除了驱动器字母后的冒号）
    fn remove_other_colons(path: &str) -> String {
        // 已经在 normalize_drive_letter 中处理了驱动器字母后的冒号
        // 这里移除其他位置的冒号
        path.replace(':', "")
    }
}

impl PathConverter for WindowsPathConverter {
    fn path_to_folder_name(&self, path: &Path) -> Result<String, PathResolveError> {
        let path_str = path.to_str()
            .ok_or_else(|| PathResolveError::InvalidPath("路径包含无效字符".to_string()))?;

        #[cfg(debug_assertions)]
        eprintln!("[PathConverter] 原始路径: {}", path_str);

        // 步骤 1: 处理驱动器字母 (C: → C-)
        let normalized = Self::normalize_drive_letter(path_str);
        #[cfg(debug_assertions)]
        eprintln!("[PathConverter] 驱动器处理后: {}", normalized);

        // 步骤 2: 替换反斜杠 (\ → -)
        let replaced = Self::replace_backslashes(&normalized);
        #[cfg(debug_assertions)]
        eprintln!("[PathConverter] 反斜杠替换后: {}", replaced);

        // 步骤 3: 移除其他位置的冒号
        let cleaned = Self::remove_other_colons(&replaced);
        #[cfg(debug_assertions)]
        eprintln!("[PathConverter] 最终结果: {}", cleaned);

        Ok(cleaned)
    }

    fn folder_name_to_path(&self, folder_name: &str) -> Result<std::path::PathBuf, PathResolveError> {
        #[cfg(debug_assertions)]
        eprintln!("[PathConverter] 还原文件夹名: {}", folder_name);

        // 验证格式: 必须以大写字母加连字符开头 (C-, D-, etc.)
        let first_char = folder_name.chars().next();
        if !first_char.map_or(false, |c| c.is_ascii_uppercase()) {
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
        } else if let Some(rest) = folder_name.strip_prefix("F-") {
            format!("F:{}", rest)
        } else {
            // 其他格式，可能是 UNC 路径或其他
            folder_name.to_string()
        };

        // 还原连字符为反斜杠
        let restored = path.replace('-', "\\");
        #[cfg(debug_assertions)]
        eprintln!("[PathConverter] 还原路径: {}", restored);

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
    fn test_path_with_spaces() {
        let converter = WindowsPathConverter::new();
        let path = Path::new(r"C:\Projects\My App\v2.0");
        let result = converter.path_to_folder_name(path).unwrap();
        assert_eq!(result, "C--Projects-My App-v2.0");
    }

    #[test]
    fn test_path_with_underscores() {
        // Claude Code 会将下划线也转换为连字符
        let converter = WindowsPathConverter::new();
        let path = Path::new(r"C:\software\full_stack\prism-forge");
        let result = converter.path_to_folder_name(path).unwrap();
        assert_eq!(result, "C--software-full-stack-prism-forge");
    }

    #[test]
    fn test_restore_path() {
        let converter = WindowsPathConverter::new();
        let folder_name = "C--software-Java-project";
        let result = converter.folder_name_to_path(folder_name).unwrap();
        assert_eq!(result, Path::new(r"C:\software\Java\project"));
    }

    #[test]
    fn test_chinese_path() {
        let converter = WindowsPathConverter::new();
        let path = Path::new(r"C:\软件\项目\测试");
        let result = converter.path_to_folder_name(path).unwrap();
        assert_eq!(result, "C--软件-项目-测试");
    }
}
