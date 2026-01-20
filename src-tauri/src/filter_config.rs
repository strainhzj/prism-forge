//! 日志过滤配置管理模块
//!
//! 负责加载、保存和验证日志过滤规则配置

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ==================== 数据结构 ====================

/// 过滤规则配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterRule {
    /// 规则名称（唯一标识）
    pub name: String,

    /// 是否启用
    pub enabled: bool,

    /// 匹配类型
    #[serde(rename = "matchType")]
    pub match_type: MatchType,

    /// 匹配模式
    pub pattern: String,

    /// 规则描述
    pub description: Option<String>,
}

/// 匹配类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MatchType {
    /// 包含匹配（简单字符串包含）
    #[serde(rename = "contains")]
    Contains,

    /// 正则表达式匹配（预留）
    #[serde(rename = "regex")]
    Regex,

    /// 精确匹配（预留）
    #[serde(rename = "exact")]
    Exact,
}

/// 过滤配置文件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterConfig {
    /// 配置版本
    pub version: String,

    /// 是否启用全局过滤
    pub enabled: bool,

    /// 过滤规则列表
    pub rules: Vec<FilterRule>,
}

// ==================== 默认配置 ====================

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            enabled: true,
            rules: vec![
                FilterRule {
                    name: "clear_command".to_string(),
                    enabled: true,
                    match_type: MatchType::Contains,
                    pattern: "[操作] 调用工具: clear".to_string(),
                    description: Some("过滤 /clear 命令".to_string()),
                },
                FilterRule {
                    name: "empty_tool_result".to_string(),
                    enabled: true,
                    match_type: MatchType::Contains,
                    pattern: "[工具结果] 输出: ...".to_string(),
                    description: Some("过滤空的工具输出".to_string()),
                },
            ],
        }
    }
}

// ==================== 配置管理器 ====================

/// 过滤配置管理器
pub struct FilterConfigManager {
    /// 配置文件路径
    config_path: PathBuf,

    /// 当前配置
    config: FilterConfig,
}

impl FilterConfigManager {
    /// 创建配置管理器
    ///
    /// # 参数
    /// * `config_path` - 配置文件路径
    pub fn new(config_path: PathBuf) -> Result<Self> {
        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .context("创建配置目录失败")?;
        }

        // 加载配置
        let config = Self::load_config(&config_path)?;

        Ok(Self {
            config_path,
            config,
        })
    }

    /// 使用默认路径创建配置管理器
    ///
    /// # 路径规则
    /// - Windows: %APPDATA%\prism-forge\filter-rules.json
    /// - macOS:   ~/Library/Application Support/prism-forge/filter-rules.json
    /// - Linux:   ~/.config/prism-forge/filter-rules.json
    pub fn with_default_path() -> Result<Self> {
        let config_dir = Self::get_config_dir()?;
        let config_path = config_dir.join("filter-rules.json");
        Self::new(config_path)
    }

    /// 获取配置目录
    fn get_config_dir() -> Result<PathBuf> {
        let base_dir = if cfg!(target_os = "windows") {
            // Windows: %APPDATA%
            std::env::var("APPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    let mut path = PathBuf::new();
                    path.push("C:");
                    path.push("Users");
                    path.push(std::env::var("USERNAME").unwrap_or("Default".to_string()));
                    path.push("AppData");
                    path.push("Roaming");
                    path
                })
        } else if cfg!(target_os = "macos") {
            // macOS: ~/Library/Application Support
            let home = std::env::var("HOME").unwrap_or(".".to_string());
            let mut path = PathBuf::from(home);
            path.push("Library");
            path.push("Application Support");
            path
        } else {
            // Linux: ~/.config
            let home = std::env::var("HOME").unwrap_or(".".to_string());
            let mut path = PathBuf::from(home);
            path.push(".config");
            path
        };

        Ok(base_dir.join("prism-forge"))
    }

    /// 加载配置文件
    fn load_config(path: &PathBuf) -> Result<FilterConfig> {
        // 如果配置文件不存在，创建默认配置
        if !path.exists() {
            #[cfg(debug_assertions)]
            eprintln!("[FilterConfigManager] 配置文件不存在，创建默认配置: {:?}", path);

            let default_config = FilterConfig::default();
            Self::save_config_to_file(path, &default_config)?;
            return Ok(default_config);
        }

        // 读取配置文件
        let content = fs::read_to_string(path)
            .context("读取配置文件失败")?;

        // 解析 JSON
        let config: FilterConfig = serde_json::from_str(&content)
            .with_context(|| format!("解析配置文件失败: {}", path.display()))?;

        #[cfg(debug_assertions)]
        eprintln!("[FilterConfigManager] 加载配置成功，规则数量: {}", config.rules.len());

        Ok(config)
    }

    /// 保存配置到文件
    fn save_config_to_file(path: &PathBuf, config: &FilterConfig) -> Result<()> {
        let json = serde_json::to_string_pretty(config)
            .context("序列化配置失败")?;

        fs::write(path, json)
            .context("写入配置文件失败")?;

        #[cfg(debug_assertions)]
        eprintln!("[FilterConfigManager] 配置已保存: {:?}", path);

        Ok(())
    }

    /// 重新加载配置
    pub fn reload(&mut self) -> Result<()> {
        self.config = Self::load_config(&self.config_path)?;
        Ok(())
    }

    /// 获取配置
    pub fn get_config(&self) -> &FilterConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: FilterConfig) -> Result<()> {
        // 验证配置
        Self::validate_config(&config)?;

        // 保存到文件
        Self::save_config_to_file(&self.config_path, &config)?;

        // 更新内存中的配置
        self.config = config;

        Ok(())
    }

    /// 验证配置
    fn validate_config(config: &FilterConfig) -> Result<()> {
        // 检查规则名称唯一性
        let mut names = std::collections::HashSet::new();
        for rule in &config.rules {
            if !names.insert(&rule.name) {
                anyhow::bail!("规则名称重复: {}", rule.name);
            }
        }

        // 验证匹配模式
        for rule in &config.rules {
            if rule.pattern.is_empty() {
                anyhow::bail!("规则 {} 的匹配模式为空", rule.name);
            }

            // TODO: 如果是正则表达式，验证其有效性
            if rule.match_type == MatchType::Regex {
                anyhow::bail!("正则表达式匹配暂未支持");
            }
        }

        Ok(())
    }

    // ==================== 内容保护辅助方法 ====================

    /// 检测是否为简单格式的 user 消息
    ///
    /// 简单格式是指 message 字段直接包含 role 和 content 的 JSON 结构：
    /// ```json
    /// {"role":"user","content":"..."}
    /// ```
    ///
    /// 这种格式表示用户直接输入的文本内容，应该被保护不过滤。
    ///
    /// # 参数
    /// * `content` - 要检查的内容字符串
    ///
    /// # 返回
    /// 返回 true 表示是简单格式的 user 消息（应该保护）
    fn is_simple_user_message(content: &str) -> bool {
        // 尝试解析 JSON
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(content) {
            // 检查是否有 message 字段
            if let Some(message) = parsed.get("message") {
                // 检查 message 是否是对象（不是数组）
                if let Some(obj) = message.as_object() {
                    // 检查是否有 role 字段且值为 "user"
                    if let Some(role) = obj.get("role").and_then(|v| v.as_str()) {
                        if role == "user" {
                            // 检查是否有 content 字段
                            if obj.contains_key("content") {
                                #[cfg(debug_assertions)]
                                eprintln!("[FilterConfigManager] 检测到简单格式 user 消息，保护不过滤");
                                return true;
                            }
                        }
                    }
                }
            }
        }

        false
    }

    /// 检测是否有明确的停止序列标记
    ///
    /// 检查消息中是否包含 stop_sequence 字段且不为 null。
    /// 这种标记表示 assistant 的完整回复，应该被保护不过滤。
    ///
    /// # 参数
    /// * `content` - 要检查的内容字符串
    ///
    /// # 返回
    /// 返回 true 表示有明确的停止序列（应该保护）
    fn has_explicit_stop_sequence(content: &str) -> bool {
        // 尝试解析 JSON
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(content) {
            // 检查是否有 message 字段
            if let Some(message) = parsed.get("message") {
                // 检查 message 是否是对象
                if let Some(obj) = message.as_object() {
                    // 检查是否有 stop_sequence 字段
                    if let Some(stop_sequence) = obj.get("stop_sequence") {
                        // 如果 stop_sequence 不是 null，则表示有明确停止标记
                        if !stop_sequence.is_null() {
                            #[cfg(debug_assertions)]
                            eprintln!("[FilterConfigManager] 检测到明确停止序列标记，保护不过滤: {:?}", stop_sequence);
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// 检查内容是否应该被过滤
    ///
    /// # 参数
    /// * `content` - 要检查的内容
    ///
    /// # 返回
    /// 返回 true 表示应该过滤，false 表示不过滤
    pub fn should_filter(&self, content: &str) -> bool {
        // ========== 内容保护检查（预处理）==========
        // 保护条件优先于所有过滤规则

        // 保护 1: 简单格式的 user 消息
        if Self::is_simple_user_message(content) {
            return false;
        }

        // 保护 2: 带明确停止序列的 assistant 消息
        if Self::has_explicit_stop_sequence(content) {
            return false;
        }

        // ========== 原有过滤逻辑 ==========

        // 如果全局过滤未启用，不过滤任何内容
        if !self.config.enabled {
            return false;
        }

        // 检查每条规则
        for rule in &self.config.rules {
            // 跳过未启用的规则
            if !rule.enabled {
                continue;
            }

            // 根据匹配类型进行匹配
            let matches = match rule.match_type {
                MatchType::Contains => content.contains(&rule.pattern),
                MatchType::Regex => {
                    // 预留：正则表达式匹配
                    #[cfg(debug_assertions)]
                    eprintln!("[FilterConfigManager] 正则表达式匹配暂未支持");
                    false
                }
                MatchType::Exact => content == rule.pattern,
            };

            if matches {
                #[cfg(debug_assertions)]
                eprintln!("[FilterConfigManager] 内容被规则 '{}' 过滤", rule.name);
                return true;
            }
        }

        false
    }

    /// 获取配置文件路径
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FilterConfig::default();
        assert_eq!(config.version, "1.0");
        assert!(config.enabled);
        assert_eq!(config.rules.len(), 2);
    }

    #[test]
    fn test_should_filter() {
        let config = FilterConfig::default();

        // 测试过滤规则
        assert!(config.rules[0].pattern.contains("clear"));
        assert!(config.rules[1].pattern.contains("工具结果"));
    }

    #[test]
    fn test_match_type_serialization() {
        let match_type = MatchType::Contains;
        let json = serde_json::to_string(&match_type).unwrap();
        assert!(json.contains("contains"));

        let deserialized: MatchType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, MatchType::Contains);
    }

    // ==================== 内容保护测试 ====================

    #[test]
    fn test_is_simple_user_message() {
        // 测试简单格式的 user 消息（应该被保护）
        let simple_user = r#"{"message":{"role":"user","content":"Hello world"}}"#;
        assert!(FilterConfigManager::is_simple_user_message(simple_user));

        // 测试包含额外字段的简单 user 消息
        let user_with_extra = r#"{"message":{"role":"user","content":"test","timestamp":"2026-01-20"}}"#;
        assert!(FilterConfigManager::is_simple_user_message(user_with_extra));

        // 测试非 user 角色消息（不应该被保护）
        let assistant_msg = r#"{"message":{"role":"assistant","content":"Hi there"}}"#;
        assert!(!FilterConfigManager::is_simple_user_message(assistant_msg));

        // 测试嵌套的 content 数组（不是简单格式）
        let complex_user = r#"{"message":{"role":"user","content":[{"type":"text","text":"Hello"}]}}"#;
        // 这种格式虽然有 role 和 content，但 content 是数组，仍然有 content 字段
        assert!(FilterConfigManager::is_simple_user_message(complex_user));

        // 测试没有 message 字段的内容
        let no_message = r#"{"type":"user","content":"test"}"#;
        assert!(!FilterConfigManager::is_simple_user_message(no_message));
    }

    #[test]
    fn test_has_explicit_stop_sequence() {
        // 测试有明确停止序列的消息（应该被保护）
        let with_stop = r#"{"message":{"stop_sequence":"end"}}"#;
        assert!(FilterConfigManager::has_explicit_stop_sequence(with_stop));

        // 测试停止序列为空字符串（应该被保护）
        let with_empty_stop = r#"{"message":{"stop_sequence":""}}"#;
        assert!(FilterConfigManager::has_explicit_stop_sequence(with_empty_stop));

        // 测试停止序列为 null（不应该被保护）
        let with_null_stop = r#"{"message":{"stop_sequence":null}}"#;
        assert!(!FilterConfigManager::has_explicit_stop_sequence(with_null_stop));

        // 测试没有 stop_sequence 字段的消息
        let no_stop = r#"{"message":{"role":"assistant","content":"Hello"}}"#;
        assert!(!FilterConfigManager::has_explicit_stop_sequence(no_stop));
    }

    #[test]
    fn test_should_filter_with_protection() {
        // 创建配置管理器
        let manager = FilterConfigManager::with_default_path().unwrap();

        // 测试 1: 简单 user 消息即使匹配过滤规则也不应被过滤
        let simple_user_with_command = r#"{"message":{"role":"user","content":"Execute <command-name>/clear</command-name> now"}}"#;
        assert!(!manager.should_filter(simple_user_with_command),
            "简单 user 消息应该被保护，即使包含 <command-name>/clear</command-name>");

        // 测试 2: 带停止序列的 assistant 消息即使匹配过滤规则也不应被过滤
        let assistant_with_stop_and_caveat = r#"{"message":{"stop_sequence":"end","text":"Warning: <local-command-caveat> this is important"}}"#;
        assert!(!manager.should_filter(assistant_with_stop_and_caveat),
            "带停止序列的消息应该被保护，即使包含 <local-command-caveat>");

        // 测试 3: 不满足保护条件的消息应该正常过滤
        let normal_message_with_caveat = "This contains <local-command-caveat> inside";
        assert!(manager.should_filter(normal_message_with_caveat),
            "不满足保护条件的消息应该正常被过滤");

        // 测试 4: 不匹配过滤规则的普通消息不过滤
        let normal_message = "This is a normal message";
        assert!(!manager.should_filter(normal_message),
            "不匹配过滤规则的普通消息不应被过滤");
    }
}
