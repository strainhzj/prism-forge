//! API Key 轮换管理模块
//!
//! 支持为单个提供商配置多个 API Key，实现轮换使用和负载均衡
//!
//! ## 功能
//! - 支持逗号分隔的多个 API Key
//! - Round-Robin 轮换算法
//! - 密钥使用次数统计
//! - 密钥健康状态追踪

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 密钥轮换配置（存储在 config_json 中）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationConfig {
    /// 当前使用的密钥索引
    #[serde(default = "default_current_index")]
    pub current_index: usize,

    /// 每个密钥的使用次数统计
    /// key: 密钥索引 (0-based)
    /// value: 使用次数
    #[serde(default)]
    pub usage_count: HashMap<usize, u64>,

    /// 每个密钥的最后使用时间（Unix 时间戳）
    #[serde(default)]
    pub last_used: HashMap<usize, u64>,
}

fn default_current_index() -> usize {
    0
}

impl Default for KeyRotationConfig {
    fn default() -> Self {
        Self {
            current_index: 0,
            usage_count: HashMap::new(),
            last_used: HashMap::new(),
        }
    }
}

impl KeyRotationConfig {
    /// 创建新的轮换配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取下一个要使用的密钥索引
    pub fn get_next_index(&self, total_keys: usize) -> usize {
        if total_keys == 0 {
            return 0;
        }
        // Round-Robin: 简单的轮询算法
        (self.current_index + 1) % total_keys
    }

    /// 更新当前索引
    pub fn update_index(&mut self, new_index: usize) {
        self.current_index = new_index;
    }

    /// 记录密钥使用
    pub fn record_usage(&mut self, index: usize) {
        *self.usage_count.entry(index).or_insert(0) += 1;
        *self.last_used.entry(index).or_insert(0) = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// 获取密钥使用次数
    pub fn get_usage_count(&self, index: usize) -> u64 {
        self.usage_count.get(&index).copied().unwrap_or(0)
    }

    /// 获取总使用次数
    pub fn get_total_usage(&self) -> u64 {
        self.usage_count.values().sum()
    }

    /// 获取使用统计信息
    pub fn get_stats(&self, total_keys: usize) -> Vec<KeyStats> {
        (0..total_keys)
            .map(|i| KeyStats {
                index: i,
                usage_count: self.get_usage_count(i),
                last_used: self.last_used.get(&i).copied(),
            })
            .collect()
    }
}

/// 密钥统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStats {
    /// 密钥索引
    pub index: usize,
    /// 使用次数
    pub usage_count: u64,
    /// 最后使用时间（Unix 时间戳）
    pub last_used: Option<u64>,
}

/// API Key 轮换管理器
pub struct ApiKeyRotator;

impl ApiKeyRotator {
    /// 解析多个 API Key（逗号分隔）
    ///
    /// # 参数
    /// - `keys_str`: 逗号分隔的 API Key 字符串
    ///
    /// # 返回
    /// 成功时返回解析后的密钥向量
    ///
    /// # 示例
    /// ```
    /// let keys = ApiKeyRotator::parse_keys("sk-key1,sk-key2,sk-key3")?;
    /// assert_eq!(keys.len(), 3);
    /// ```
    pub fn parse_keys(keys_str: &str) -> Result<Vec<String>> {
        if keys_str.is_empty() {
            return Err(anyhow::anyhow!("API Key 字符串为空"));
        }

        let keys: Vec<String> = keys_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if keys.is_empty() {
            return Err(anyhow::anyhow!("未找到有效的 API Key"));
        }

        Ok(keys)
    }

    /// 格式化多个 API Key 为逗号分隔的字符串
    pub fn format_keys(keys: &[String]) -> String {
        keys.join(",")
    }

    /// 选择下一个要使用的 API Key
    ///
    /// # 参数
    /// - `keys_str`: 逗号分隔的 API Key 字符串
    /// - `config_json`: 当前的轮换配置（JSON 字符串）
    ///
    /// # 返回
    /// 返回 (选中的密钥, 更新后的配置 JSON)
    pub fn select_next_key(
        keys_str: &str,
        config_json: Option<&str>,
    ) -> Result<(String, String)> {
        // 解析密钥列表
        let keys = Self::parse_keys(keys_str)?;

        // 解析或创建配置
        let mut config: KeyRotationConfig = if let Some(json) = config_json {
            serde_json::from_str(json).unwrap_or_default()
        } else {
            KeyRotationConfig::new()
        };

        // 获取下一个索引
        let next_index = config.get_next_index(keys.len());

        // 更新配置
        config.update_index(next_index);
        config.record_usage(next_index);

        // 序列化配置
        let new_config_json = serde_json::to_string(&config)?;

        // 返回选中的密钥和新配置
        let selected_key = keys
            .get(next_index)
            .ok_or_else(|| anyhow::anyhow!("密钥索引 {} 超出范围", next_index))?
            .clone();

        Ok((selected_key, new_config_json))
    }

    /// 获取指定索引的 API Key
    ///
    /// # 参数
    /// - `keys_str`: 逗号分隔的 API Key 字符串
    /// - `index`: 密钥索引
    pub fn get_key_at_index(keys_str: &str, index: usize) -> Result<String> {
        let keys = Self::parse_keys(keys_str)?;
        keys.get(index)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("密钥索引 {} 超出范围（共 {} 个密钥）", index, keys.len()))
    }

    /// 获取密钥数量
    pub fn get_key_count(keys_str: &str) -> usize {
        Self::parse_keys(keys_str).map(|keys| keys.len()).unwrap_or(0)
    }

    /// 验证密钥列表格式
    ///
    /// # 参数
    /// - `keys_str`: 逗号分隔的 API Key 字符串
    ///
    /// # 返回
    /// 成功时返回 Ok(())，失败时返回错误信息
    pub fn validate_keys(keys_str: &str) -> Result<()> {
        let keys = Self::parse_keys(keys_str)?;

        if keys.is_empty() {
            return Err(anyhow::anyhow!("密钥列表不能为空"));
        }

        // 检查是否有重复的密钥
        let unique_keys: std::collections::HashSet<_> = keys.iter().collect();
        if unique_keys.len() != keys.len() {
            return Err(anyhow::anyhow!("密钥列表中存在重复的密钥"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_keys() {
        let keys = ApiKeyRotator::parse_keys("sk-key1,sk-key2,sk-key3").unwrap();
        assert_eq!(keys.len(), 3);
        assert_eq!(keys[0], "sk-key1");
        assert_eq!(keys[1], "sk-key2");
        assert_eq!(keys[2], "sk-key3");
    }

    #[test]
    fn test_parse_keys_with_spaces() {
        let keys = ApiKeyRotator::parse_keys("sk-key1 , sk-key2 , sk-key3").unwrap();
        assert_eq!(keys.len(), 3);
    }

    #[test]
    fn test_format_keys() {
        let keys = vec!["sk-key1".to_string(), "sk-key2".to_string()];
        let formatted = ApiKeyRotator::format_keys(&keys);
        assert_eq!(formatted, "sk-key1,sk-key2");
    }

    #[test]
    fn test_select_next_key() {
        let keys_str = "sk-key1,sk-key2,sk-key3";
        let (key, config) = ApiKeyRotator::select_next_key(keys_str, None).unwrap();
        assert_eq!(key, "sk-key2"); // current_index=0, next=1

        let config: KeyRotationConfig = serde_json::from_str(&config).unwrap();
        assert_eq!(config.current_index, 1);
        assert_eq!(config.get_usage_count(1), 1);
    }

    #[test]
    fn test_round_robin() {
        let keys_str = "sk-key1,sk-key2";

        // 第1次: index 0 -> 1
        let (key1, config1) = ApiKeyRotator::select_next_key(keys_str, None).unwrap();
        assert_eq!(key1, "sk-key2");

        // 第2次: index 1 -> 0
        let (key2, config2) = ApiKeyRotator::select_next_key(keys_str, Some(&config1)).unwrap();
        assert_eq!(key2, "sk-key1");

        // 第3次: index 0 -> 1
        let (key3, _) = ApiKeyRotator::select_next_key(keys_str, Some(&config2)).unwrap();
        assert_eq!(key3, "sk-key2");
    }
}
