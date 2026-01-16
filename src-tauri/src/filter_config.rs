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
                    pattern: "<command-name>/clear</command-name>".to_string(),
                    description: Some("过滤 /clear 命令".to_string()),
                },
                FilterRule {
                    name: "local_command_caveat".to_string(),
                    enabled: true,
                    match_type: MatchType::Contains,
                    pattern: "<local-command-caveat>".to_string(),
                    description: Some("过滤本地命令提示信息".to_string()),
                },
                FilterRule {
                    name: "empty_stdout".to_string(),
                    enabled: true,
                    match_type: MatchType::Contains,
                    pattern: "<local-command-stdout></local-command-stdout>".to_string(),
                    description: Some("过滤空的命令输出".to_string()),
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

    /// 检查内容是否应该被过滤
    ///
    /// # 参数
    /// * `content` - 要检查的内容
    ///
    /// # 返回
    /// 返回 true 表示应该过滤，false 表示不过滤
    pub fn should_filter(&self, content: &str) -> bool {
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
        assert_eq!(config.rules.len(), 3);
    }

    #[test]
    fn test_should_filter() {
        let config = FilterConfig::default();

        // 测试过滤规则
        assert!(config.rules[0].pattern.contains("/clear"));
        assert!(config.rules[1].pattern.contains("local-command-caveat"));
        assert!(config.rules[2].pattern.contains("local-command-stdout"));
    }

    #[test]
    fn test_match_type_serialization() {
        let match_type = MatchType::Contains;
        let json = serde_json::to_string(&match_type).unwrap();
        assert!(json.contains("contains"));

        let deserialized: MatchType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, MatchType::Contains);
    }
}
