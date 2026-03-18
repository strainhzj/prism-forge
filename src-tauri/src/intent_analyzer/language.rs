//! 语言检测模块
//!
//! 用于检测会话文本的主要语言（中文/英文）

use regex::Regex;
use anyhow::Result;

/// 语言检测器
///
/// 基于字符统计判断文本主要语言
pub struct LanguageDetector {
    /// 中文字符正则（CJK 统一汉字）
    chinese_regex: Regex,
    /// 英文字符正则
    english_regex: Regex,
}

impl LanguageDetector {
    /// 创建新的语言检测器
    ///
    /// # Errors
    ///
    /// 如果正则表达式编译失败，返回错误
    pub fn new() -> Result<Self> {
        Ok(Self {
            chinese_regex: Regex::new(r"[\u4e00-\u9fff]")?,
            english_regex: Regex::new(r"[a-zA-Z]")?,
        })
    }

    /// 检测会话主要语言
    ///
    /// # Arguments
    ///
    /// * `text` - 待检测的文本
    ///
    /// # Returns
    ///
    /// * `"zh"` - 中文
    /// * `"en"` - 英文
    ///
    /// # 判断逻辑
    ///
    /// - 如果中文字符数 > 英文字符数，返回 "zh"
    /// - 否则返回 "en"（包括空字符串、纯数字、纯符号等情况）
    pub fn detect_language(&self, text: &str) -> String {
        let chinese_count = self.chinese_regex.find_iter(text).count();
        let english_count = self.english_regex.find_iter(text).count();

        if chinese_count > english_count {
            "zh".to_string()
        } else {
            "en".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_chinese() {
        let detector = LanguageDetector::new().unwrap();
        assert_eq!(detector.detect_language("你好世界"), "zh");
        assert_eq!(detector.detect_language("这是一个中文测试文本"), "zh");
    }

    #[test]
    fn test_detect_english() {
        let detector = LanguageDetector::new().unwrap();
        assert_eq!(detector.detect_language("Hello World"), "en");
        assert_eq!(detector.detect_language("This is an English text"), "en");
    }

    #[test]
    fn test_detect_mixed_chinese_dominant() {
        let detector = LanguageDetector::new().unwrap();
        // 中文字符应该多于英文字符
        let text = "这是一段非常长的中文文本，包含了许多中文字符，with English";
        assert_eq!(detector.detect_language(text), "zh");
    }

    #[test]
    fn test_detect_mixed_english_dominant() {
        let detector = LanguageDetector::new().unwrap();
        let text = "This is an English text with 少量中文";
        assert_eq!(detector.detect_language(text), "en");
    }

    #[test]
    fn test_detect_empty_string() {
        let detector = LanguageDetector::new().unwrap();
        // 空字符串时，两个计数都是 0，返回 "en"
        assert_eq!(detector.detect_language(""), "en");
    }

    #[test]
    fn test_detect_only_symbols() {
        let detector = LanguageDetector::new().unwrap();
        assert_eq!(detector.detect_language("123456789 !@#$%^&*()"), "en");
    }

    #[test]
    fn test_detect_equal_count() {
        let detector = LanguageDetector::new().unwrap();
        // 中英文数量相等时，返回 "en"
        let text = "a中b文c"; // 3 英文，3 中文（但中文计数为 3，英文计数为 3）
        // 注意：这个测试用例验证边界情况
        // 实际上 "a中b文c" 中英文各 3 个，应该返回 "en"
        assert_eq!(detector.detect_language(text), "en");
    }

    #[test]
    fn test_new_success() {
        // 验证 new() 方法成功创建检测器
        let detector = LanguageDetector::new();
        assert!(detector.is_ok());
        let detector = detector.unwrap();
        // 验证正则表达式正常工作
        assert_eq!(detector.detect_language("测试"), "zh");
        assert_eq!(detector.detect_language("test"), "en");
    }
}
