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
                    description: Some("过滤本地命令警告".to_string()),
                },
                FilterRule {
                    name: "local_command_stdout".to_string(),
                    enabled: true,
                    match_type: MatchType::Contains,
                    pattern: "<local-command-stdout>".to_string(),
                    description: Some("过滤本地命令输出".to_string()),
                },
                FilterRule {
                    name: "command_name_tag".to_string(),
                    enabled: true,
                    match_type: MatchType::Contains,
                    pattern: "<command-name>".to_string(),
                    description: Some("过滤命令名称标签".to_string()),
                },
                FilterRule {
                    name: "command_message_tag".to_string(),
                    enabled: true,
                    match_type: MatchType::Contains,
                    pattern: "<command-message>".to_string(),
                    description: Some("过滤命令消息标签".to_string()),
                },
                FilterRule {
                    name: "command_args_tag".to_string(),
                    enabled: true,
                    match_type: MatchType::Contains,
                    pattern: "<command-args>".to_string(),
                    description: Some("过滤命令参数标签".to_string()),
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
                    #[cfg(debug_assertions)]
                    log::warn!("未找到 APPDATA 环境变量，使用默认路径");

                    let username = std::env::var("USERNAME")
                        .unwrap_or_else(|_| {
                            #[cfg(debug_assertions)]
                            log::warn!("未找到 USERNAME 环境变量，使用默认值");
                            "Default".to_string()
                        });

                    let mut path = PathBuf::from("C:");
                    path.push("Users");
                    path.push(username);
                    path.push("AppData");
                    path.push("Roaming");
                    path
                })
        } else if cfg!(target_os = "macos") {
            // macOS: ~/Library/Application Support
            let home = std::env::var("HOME").unwrap_or_else(|_| {
                #[cfg(debug_assertions)]
                log::warn!("未找到 HOME 环境变量，使用当前目录");
                ".".to_string()
            });
            let mut path = PathBuf::from(home);
            path.push("Library");
            path.push("Application Support");
            path
        } else {
            // Linux: ~/.config
            let home = std::env::var("HOME").unwrap_or_else(|_| {
                #[cfg(debug_assertions)]
                log::warn!("未找到 HOME 环境变量，使用当前目录");
                ".".to_string()
            });
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

    /// 检测内容是否包含命令系统标签
    ///
    /// 检查内容是否包含 Claude Code 内部命令系统生成的标签。
    /// 这些标签标识了系统自动生成的消息，不应该被"简单 user 消息"保护机制保护。
    ///
    /// # 命令标签列表
    /// - `<local-command-caveat>` - 本地命令警告
    /// - `<local-command-stdout>` - 本地命令输出
    /// - `<command-name>` - 命令名称
    /// - `<command-message>` - 命令消息
    /// - `<command-args>` - 命令参数
    ///
    /// # 参数
    /// * `content` - 要检查的内容字符串
    ///
    /// # 返回
    /// 返回 true 表示包含命令系统标签
    fn contains_command_tags(content: &str) -> bool {
        content.contains("<local-command-caveat>") ||
        content.contains("<local-command-stdout>") ||
        content.contains("<command-name>") ||
        content.contains("<command-message>") ||
        content.contains("<command-args>")
    }

    /// 检测消息是否为纯系统生成的命令消息
    ///
    /// 通过检查去除命令标签后的剩余内容长度，判断是否为系统自动生成的消息。
    /// 系统命令消息通常只包含标签和极少的辅助文本。
    ///
    /// # 判断标准
    /// 1. 硬编码模式：匹配特定的系统生成文本（仅限纯文本格式）
    /// 2. 长度判断：包含命令标签且去除标签后剩余字符数 < 30
    ///
    /// # 重要：硬编码模式的安全保护
    /// 硬编码模式检测**仅应用于纯文本格式**（不包含 JSON 结构标记），
    /// 避免误过滤用户/助手中引用或讨论这段文本的消息。
    ///
    /// # 参数
    /// * `content` - 要检查的内容字符串
    ///
    /// # 返回
    /// 返回 true 表示是系统生成的命令消息（应该被过滤）
    fn is_system_command_message(content: &str) -> bool {
        // ========== 硬编码模式检测（优先级最高）==========

        // 安全检查：如果内容看起来像 JSON 格式，跳过硬编码检测
        // 这样可以避免误过滤包含 caveat 文本的真正用户/助手消息
        let looks_like_json = content.contains("{\"") || content.contains("\"role\":") || content.contains("\"content\":");
        if !looks_like_json {
            // 只有在纯文本格式中才应用硬编码检测
            // 模式 1: local-command-caveat 的完整警告文本
            let caveat_pattern = "Caveat: The messages below were generated by the user while running local commands";
            if content.contains("<local-command-caveat>") && content.contains(caveat_pattern) {
                #[cfg(debug_assertions)]
                eprintln!("[FilterConfigManager] 检测为纯文本 caveat 模式，直接过滤");
                return true;
            }
        }

        // ========== 长度判断逻辑 ==========

        // 首先检查是否包含命令标签
        if !Self::contains_command_tags(content) {
            return false;
        }

        // 去除所有命令标签，检查剩余内容长度
        let cleaned = content
            .replace("<local-command-caveat>", "")
            .replace("</local-command-caveat>", "")
            .replace("<local-command-stdout>", "")
            .replace("</local-command-stdout>", "")
            .replace("<command-name>", "")
            .replace("</command-name>", "")
            .replace("<command-message>", "")
            .replace("</command-message>", "")
            .replace("<command-args>", "")
            .replace("</command-args>", "")
            // 去除空白字符
            .replace(' ', "")
            .replace('\n', "")
            .replace('\t', "")
            .replace('\r', "");

        // 如果去除标签后剩余字符少于 30 个，认为是系统消息
        let is_system = cleaned.len() < 30;

        #[cfg(debug_assertions)]
        {
            if is_system {
                eprintln!("[FilterConfigManager] 检测为系统命令消息: 去除标签后剩余 {} 字符: '{}'",
                    cleaned.len(), cleaned);
            } else {
                eprintln!("[FilterConfigManager] 检测为包含标签的用户消息: 去除标签后剩余 {} 字符",
                    cleaned.len());
            }
        }

        is_system
    }

    /// 检测是否为简单格式的 user 消息
    ///
    /// 简单格式有两种可能的结构：
    /// 1. **顶层结构**（完整 JSONL 条目）：
    ///    ```json
    ///    {"message":{"role":"user","content":"..."}}
    ///    ```
    /// 2. **消息结构**（message 字段的值）：
    ///    ```json
    ///    {"role":"user","content":"..."}
    ///    ```
    ///
    /// 这种格式表示用户直接输入的文本内容，应该被保护不过滤。
    ///
    /// **排除条件**：如果内容是纯系统命令消息（仅包含命令标签和极少文本），
    /// 即使格式简单，也不应该被保护，而应该被过滤规则处理。
    ///
    /// **重要区别**：
    /// - 系统消息：`{"content":"<command-name>/clear</command-name>","role":"user"}` → 不过滤
    /// - 用户输入：`{"content":"请帮我分析 <command-name>/clear</command-name> 的作用","role":"user"}` → 保护
    ///
    /// # 参数
    /// * `content` - 要检查的内容字符串（可能是完整条目或仅 message 字段）
    ///
    /// # 返回
    /// 返回 true 表示是简单格式的 user 消息（应该保护）
    fn is_simple_user_message(content: &str) -> bool {
        // 尝试解析 JSON
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(content) {
            // 提取实际的内容字符串（从两种可能的格式中）
            let actual_content = if let Some(message) = parsed.get("message") {
                // 格式 1: {"message":{"role":"user","content":"..."}}
                message.get("content").and_then(|v| v.as_str())
            } else {
                // 格式 2: {"role":"user","content":"..."}
                parsed.get("content").and_then(|v| v.as_str())
            };

            // 如果找到了 content，检查是否为系统命令消息
            if let Some(content_str) = actual_content {
                // 如果是纯系统命令消息，不保护
                if Self::is_system_command_message(content_str) {
                    #[cfg(debug_assertions)]
                    eprintln!("[FilterConfigManager] 检测为系统命令消息，不保护");
                    return false;
                }
            }

            // ========== 新增：检查 role 字段 ==========
            // 提取 role（从两种可能的格式中）
            let role = if let Some(message) = parsed.get("message") {
                message.get("role").and_then(|v| v.as_str())
            } else {
                parsed.get("role").and_then(|v| v.as_str())
            };

            // 如果 role 是 assistant，不应用保护
            // 命令标签不应该出现在 assistant 消息中，即使出现也应该被过滤规则处理
            // 这样可以避免 assistant 消息中包含命令标签示例时被误保护
            if let Some(role_str) = role {
                if role_str == "assistant" {
                    #[cfg(debug_assertions)]
                    eprintln!("[FilterConfigManager] 检测到 assistant 角色，不应用简单格式保护");
                    return false;
                }
            }
            // ========== 新增结束 ==========

            // 情况 1: 检查是否有顶层 message 字段（完整 JSONL 条目格式）
            if let Some(message) = parsed.get("message") {
                if let Some(obj) = message.as_object() {
                    // 检查是否有 role 字段且值为 "user"
                    if let Some(role) = obj.get("role").and_then(|v| v.as_str()) {
                        if role == "user" {
                            // 检查是否有 content 字段
                            if obj.get("content").is_some() {
                                #[cfg(debug_assertions)]
                                eprintln!("[FilterConfigManager] 检测到顶层结构简单格式 user 消息，保护不过滤");
                                return true;
                            }
                        }
                    }
                }
            }

            // 情况 2: 直接检查是否为 message 字段值（仅 message 对象）
            // 这个结构是 {"role":"user","content":"..."}
            if let Some(obj) = parsed.as_object() {
                if let Some(role) = obj.get("role").and_then(|v| v.as_str()) {
                    if role == "user" {
                        if obj.get("content").is_some() {
                            #[cfg(debug_assertions)]
                            eprintln!("[FilterConfigManager] 检测到消息对象结构简单格式 user 消息，保护不过滤");
                            return true;
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
    /// 支持两种格式：
    /// 1. **顶层结构**（完整 JSONL 条目）：
    ///    ```json
    ///    {"message":{"stop_sequence":"end"}}
    ///    ```
    /// 2. **消息结构**（message 字段的值）：
    ///    ```json
    ///    {"stop_sequence":"end"}
    ///    ```
    ///
    /// # 参数
    /// * `content` - 要检查的内容字符串
    ///
    /// # 返回
    /// 返回 true 表示有明确的停止序列（应该保护）
    fn has_explicit_stop_sequence(content: &str) -> bool {
        // 尝试解析 JSON
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(content) {
            // 情况 1: 检查是否有顶层 message 字段（完整 JSONL 条目格式）
            if let Some(message) = parsed.get("message") {
                if let Some(obj) = message.as_object() {
                    if let Some(stop_sequence) = obj.get("stop_sequence") {
                        if !stop_sequence.is_null() {
                            #[cfg(debug_assertions)]
                            eprintln!("[FilterConfigManager] 检测到顶层结构停止序列标记，保护不过滤: {:?}", stop_sequence);
                            return true;
                        }
                    }
                }
            }

            // 情况 2: 直接检查是否有 stop_sequence 字段（仅 message 对象）
            if let Some(stop_sequence) = parsed.get("stop_sequence") {
                if !stop_sequence.is_null() {
                    #[cfg(debug_assertions)]
                    eprintln!("[FilterConfigManager] 检测到消息对象停止序列标记，保护不过滤: {:?}", stop_sequence);
                    return true;
                }
            }
        }

        false
    }

    /// 检测是否为 assistant 消息
    ///
    /// 检查内容是否为 assistant 类型的消息。
    /// 支持两种格式：
    /// 1. **顶层结构**（完整 JSONL 条目）：
    ///    ```json
    ///    {"message":{"role":"assistant","content":"..."}}
    ///    ```
    /// 2. **消息结构**（message 字段的值）：
    ///    ```json
    ///    {"role":"assistant","content":"..."}
    ///    ```
    ///
    /// # 参数
    /// * `content` - 要检查的内容字符串
    ///
    /// # 返回
    /// 返回 true 表示是 assistant 消息
    fn is_assistant_message(content: &str) -> bool {
        // 尝试解析 JSON
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(content) {
            // 情况 0: 检查顶层 type 字段（JSONL 条目的类型字段）
            if let Some(msg_type) = parsed.get("type").and_then(|v| v.as_str()) {
                if msg_type == "assistant" {
                    #[cfg(debug_assertions)]
                    eprintln!("[FilterConfigManager] 检测到顶层 type 字段为 assistant");
                    return true;
                }
            }

            // 情况 1: 检查是否有顶层 message 字段（完整 JSONL 条目格式）
            if let Some(message) = parsed.get("message") {
                if let Some(obj) = message.as_object() {
                    if let Some(role) = obj.get("role").and_then(|v| v.as_str()) {
                        if role == "assistant" {
                            #[cfg(debug_assertions)]
                            eprintln!("[FilterConfigManager] 检测到顶层结构 assistant 消息");
                            return true;
                        }
                    }
                }
            }

            // 情况 2: 直接检查是否为 message 字段值（仅 message 对象）
            // 这个检查应该放在 message 字段检查之后，因为如果没有 message 字段，
            // 我们才检查当前的 role 字段
            if let Some(obj) = parsed.as_object() {
                // 只有在没有 message 字段时，才检查 role
                if !parsed.get("message").is_some() {
                    if let Some(role) = obj.get("role").and_then(|v| v.as_str()) {
                        if role == "assistant" {
                            #[cfg(debug_assertions)]
                            eprintln!("[FilterConfigManager] 检测到消息对象结构 assistant 消息");
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

        // 保护 3: assistant 消息不应该被命令标签过滤规则影响
        // assistant 消息中可能包含命令标签示例（如解释、说明），这些都是有用的内容
        if Self::is_assistant_message(content) {
            // 检查是否匹配命令标签过滤规则
            let matches_command_tag_rule = self.config.rules.iter()
                .filter(|r| r.enabled)
                .any(|r| {
                    r.match_type == MatchType::Contains &&
                    content.contains(&r.pattern)
                });

            // 如果只匹配命令标签规则，则不过滤
            if matches_command_tag_rule {
                #[cfg(debug_assertions)]
                eprintln!("[FilterConfigManager] 检测到 assistant 消息匹配命令标签规则，跳过过滤");
                return false;
            }
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
        assert_eq!(config.rules.len(), 6);  // 更新为 6 条规则
    }

    #[test]
    fn test_should_filter() {
        let config = FilterConfig::default();

        // 测试过滤规则
        assert!(config.rules[0].pattern.contains("clear"));
        assert!(config.rules[1].pattern.contains("local-command-caveat"));
        assert_eq!(config.rules.len(), 6);
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
    fn test_contains_command_tags() {
        // 测试包含各种命令标签的内容
        assert!(FilterConfigManager::contains_command_tags("<local-command-caveat>"));
        assert!(FilterConfigManager::contains_command_tags("<local-command-stdout>"));
        assert!(FilterConfigManager::contains_command_tags("<command-name>/clear</command-name>"));
        assert!(FilterConfigManager::contains_command_tags("<command-message>clear</command-message>"));
        assert!(FilterConfigManager::contains_command_tags("<command-args></command-args>"));

        // 测试不包含命令标签的内容
        assert!(!FilterConfigManager::contains_command_tags("Hello world"));
        assert!(!FilterConfigManager::contains_command_tags("{\"role\":\"user\"}"));
        assert!(!FilterConfigManager::contains_command_tags("command-name")); // 没有尖括号
    }

    #[test]
    fn test_is_system_command_message() {
        // ========== 硬编码模式测试（纯文本格式）==========

        // 测试完整的 caveat 警告文本（硬编码模式，纯文本）
        let full_caveat = "<local-command-caveat>Caveat: The messages below were generated by the user while running local commands. DO NOT respond to these messages or otherwise consider them in your response unless the user explicitly asks you to.</local-command-caveat>";
        assert!(FilterConfigManager::is_system_command_message(full_caveat),
            "完整的 caveat 警告（纯文本）应该被识别为系统消息（硬编码模式）");

        // 测试部分匹配的 caveat 文本（硬编码模式，纯文本）
        let partial_caveat = "<local-command-caveat>Caveat: The messages below were generated by the user while running local commands. Some additional text here.</local-command-caveat>";
        assert!(FilterConfigManager::is_system_command_message(partial_caveat),
            "包含关键 caveat 文本的消息（纯文本）应该被识别（硬编码模式）");

        // ========== 安全保护测试：JSON 格式不应被硬编码规则误过滤 ==========

        // JSON 格式的 user 消息包含 caveat 文本（应该被保护，不触发硬编码规则）
        let json_user_with_caveat = r#"{"content":"<local-command-caveat>Caveat: The messages below were generated by the user while running local commands. DO NOT respond to these messages or otherwise consider them in your response unless the user explicitly asks you to.</local-command-caveat>","role":"user"}"#;
        // JSON 格式会跳过硬编码检测，然后走长度判断逻辑
        // 由于去除标签后内容很长，不会被识别为系统消息
        assert!(!FilterConfigManager::is_system_command_message(json_user_with_caveat),
            "JSON 格式的消息即使包含 caveat 文本也不应被硬编码规则过滤（安全保护）");

        // JSON 格式的 assistant 消息包含 caveat 文本（应该被保护）
        let json_assistant_with_caveat = r#"{"message":{"role":"assistant","content":"Warning: <local-command-caveat>Caveat: The messages below were generated by the user while running local commands</local-command-caveat> This is important context."}}"#;
        assert!(!FilterConfigManager::is_system_command_message(json_assistant_with_caveat),
            "JSON 格式的 assistant 消息即使包含 caveat 文本也不应被硬编码规则过滤");

        // ========== 长度判断测试 ==========

        // 测试纯系统命令消息（应该被识别为系统消息）
        let pure_caveat = "<local-command-caveat>Caveat: ...</local-command-caveat>";
        assert!(FilterConfigManager::is_system_command_message(pure_caveat));

        let pure_stdout = "<local-command-stdout></local-command-stdout>";
        assert!(FilterConfigManager::is_system_command_message(pure_stdout));

        let pure_command = "<command-name>/clear</command-name>";
        assert!(FilterConfigManager::is_system_command_message(pure_command));

        // 测试包含命令标签但有其他内容的消息（不应该被识别为系统消息）
        let user_discussing_command = "请帮我分析 <command-name>/clear</command-name> 命令的作用是什么？它会在什么情况下被触发？";
        assert!(!FilterConfigManager::is_system_command_message(user_discussing_command),
            "用户讨论命令标签的消息应该被保护");

        let long_content_with_tag = "这是一个很长的用户输入内容，虽然包含 <command-name> 标签，但是明显是用户自己输入的文本，应该被保护而不是被过滤掉";
        assert!(!FilterConfigManager::is_system_command_message(long_content_with_tag),
            "包含标签但内容丰富的消息应该被保护");

        // 测试不包含命令标签的普通内容
        assert!(!FilterConfigManager::is_system_command_message("Hello world"));
    }

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
    fn test_is_simple_user_message_with_command_tags() {
        // 测试纯系统命令消息（不应该被保护）
        let user_with_caveat = r#"{"content":"<local-command-caveat>Caveat: ...</local-command-caveat>","role":"user"}"#;
        assert!(!FilterConfigManager::is_simple_user_message(user_with_caveat),
            "纯系统命令消息不应该被保护");

        let user_with_stdout = r#"{"message":{"role":"user","content":"<local-command-stdout></local-command-stdout>"}}"#;
        assert!(!FilterConfigManager::is_simple_user_message(user_with_stdout),
            "纯系统命令消息不应该被保护");

        let user_with_command_name = r#"{"role":"user","content":"<command-name>/clear</command-name>\n            <command-message>clear</command-message>\n            <command-args></command-args>"}"#;
        assert!(!FilterConfigManager::is_simple_user_message(user_with_command_name),
            "纯系统命令消息不应该被保护");

        // 测试用户讨论命令标签（应该被保护）
        let user_discussing_command = r#"{"message":{"role":"user","content":"请帮我分析 <command-name>/clear</command-name> 命令的作用是什么？它会在什么情况下被触发？这个命令很重要。"}}"#;
        assert!(FilterConfigManager::is_simple_user_message(user_discussing_command),
            "用户讨论命令标签的消息应该被保护");

        // 测试包含少量标签但主要是用户内容的消息（应该被保护）
        let user_with_mixed_content = r#"{"message":{"role":"user","content":"Execute: <command-name>/test</command-name> 后会发生什么？请详细说明。"}}"#;
        assert!(FilterConfigManager::is_simple_user_message(user_with_mixed_content),
            "主要是用户内容的消息应该被保护");
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

        // 测试 1: 纯系统命令消息（应该被过滤）
        let user_with_command_tag = r#"{"content":"<local-command-caveat>...","role":"user"}"#;
        assert!(manager.should_filter(user_with_command_tag),
            "纯系统命令消息应该被过滤");

        // 测试 2: 完整的 caveat 警告消息（应该被过滤，硬编码模式）
        let full_caveat_message = r#"{"content":"<local-command-caveat>Caveat: The messages below were generated by the user while running local commands. DO NOT respond to these messages or otherwise consider them in your response unless the user explicitly asks you to.</local-command-caveat>","role":"user"}"#;
        assert!(manager.should_filter(full_caveat_message),
            "完整的 caveat 警告消息应该被过滤（硬编码模式）");

        // 测试 3: 真正的用户输入应该被保护，即使内容包含命令标签但内容丰富
        let real_user_input_with_tag = r#"{"message":{"role":"user","content":"请帮我分析 <command-name>/clear</command-name> 命令的作用是什么？它很关键。"}}"#;
        assert!(!manager.should_filter(real_user_input_with_tag),
            "用户讨论命令的消息应该被保护");

        // 测试 4: 带停止序列的 assistant 消息即使匹配过滤规则也不应被过滤
        let assistant_with_stop_and_caveat = r#"{"message":{"stop_sequence":"end","text":"Warning: <local-command-caveat> this is important"}}"#;
        assert!(!manager.should_filter(assistant_with_stop_and_caveat),
            "带停止序列的消息应该被保护，即使包含 <local-command-caveat>");

        // 测试 5: 不满足保护条件的消息应该正常过滤
        let normal_message_with_caveat = "This contains <local-command-caveat> inside";
        assert!(manager.should_filter(normal_message_with_caveat),
            "不满足保护条件的消息应该正常被过滤");

        // 测试 6: 不匹配过滤规则的普通消息不过滤
        let normal_message = "This is a normal message";
        assert!(!manager.should_filter(normal_message),
            "不匹配过滤规则的普通消息不应被过滤");

        // 测试 7: 普通用户输入（不包含标签）应该被保护
        let normal_user_input = r#"{"message":{"role":"user","content":"I want to execute a command"}}"#;
        assert!(!manager.should_filter(normal_user_input),
            "普通用户输入应该被保护");

        // 测试 8: assistant 消息包含命令标签示例不应该被过滤
        // assistant 消息不受命令标签过滤规则的影响
        // 因为这些标签可能是有用的示例或说明
        let assistant_with_command_tag_example = r#"{"message":{"role":"assistant","content":"这是一个示例：<local-command-caveat>Caveat: The messages below were generated by the user while running local commands</local-command-caveat>"}}"#;
        assert!(!manager.should_filter(assistant_with_command_tag_example),
            "assistant 消息包含命令标签示例不应该被过滤");

        // 测试 9: assistant 消息包含多种命令标签不应该被过滤
        let assistant_with_multiple_tags = r#"{"message":{"role":"assistant","content":"命令示例：<command-name>/clear</command-name> 和 <local-command-stdout>output</local-command-stdout>"}}"#;
        assert!(!manager.should_filter(assistant_with_multiple_tags),
            "assistant 消息包含多个命令标签不应该被过滤");

        // 测试 10: assistant 消息（消息对象结构）包含命令标签不应该被过滤
        let assistant_message_obj_with_tag = r#"{"role":"assistant","content":"分析：<local-command-caveat>这是命令警告</local-command-caveat>"}"#;
        assert!(!manager.should_filter(assistant_message_obj_with_tag),
            "assistant 消息对象包含命令标签不应该被过滤");

        // 测试 11: user 消息包含命令标签且内容丰富应该被保护（确保原有逻辑不受影响）
        let user_with_tag_and_content = r#"{"message":{"role":"user","content":"请帮我分析 <command-name>/clear</command-name> 命令的作用是什么？它很关键。"}}"#;
        assert!(!manager.should_filter(user_with_tag_and_content),
            "user 消息包含命令标签但内容丰富应该被保护");

        // 测试 12: assistant 消息不包含命令标签应该被过滤（因为不是 user 类型）
        let assistant_normal = r#"{"message":{"role":"assistant","content":"这是一个正常的助手回复"}}"#;
        assert!(!manager.should_filter(assistant_normal),
            "assistant 消息不包含命令标签不应该被过滤");

        // 测试 13: 真实的 assistant 消息 summary 格式包含命令标签示例（实际场景测试）
        // 这个测试模拟真实的 summary 格式（从 message 字段序列化而来）
        // 根据 user 需求，这个消息应该被保留（不过滤），因为命令标签只是作为示例出现在文本中
        // 通过判断标准是 content 中的 text 内容，这是有用的回复
        let real_assistant_summary = r#"{"id":"msg_202601201644385ea67d7444504bd0","type":"message","role":"assistant","model":"glm-4.7","content":[{"type":"text","text":"我已了解问题。分析命令标签：<local-command-caveat>、<local-command-stdout>、<command-name>"}],"stop_reason":null,"stop_sequence":null}"#;

        // assistant 消息即使包含命令标签（作为示例），也不应该被过滤
        // 因为这些标签出现在 assistant 的回复文本中，是有用的说明内容
        assert!(!manager.should_filter(real_assistant_summary),
            "包含命令标签示例的 assistant 消息应该被保留（不过滤）");

        // 测试 14: 验证 is_assistant_message 方法
        let assistant_msg = r#"{"message":{"role":"assistant","content":"回复内容"}}"#;
        assert!(FilterConfigManager::is_assistant_message(assistant_msg),
            "应该正确识别 assistant 消息");

        let user_msg = r#"{"message":{"role":"user","content":"问题内容"}}"#;
        assert!(!FilterConfigManager::is_assistant_message(user_msg),
            "user 消息不应该被识别为 assistant 消息");
    }
}
