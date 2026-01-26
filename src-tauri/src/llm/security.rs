//! API Key 安全存储模块
//!
//! 使用系统密钥库安全存储 LLM API Keys
//!
//! ## 平台支持
//! - **Windows**: Credential Manager
//! - **macOS**: Keychain Services
//! - **Linux**: Secret Service API (gnome-keyring, kwallet)

use anyhow::{Context, Result};
use keyring::{Entry, Error as KeyringError};
use secrecy::{ExposeSecret, SecretString};

/// 应用名称，用于在密钥库中标识我们的应用
pub const APP_NAME: &str = "PrismForge";

/// 密钥库服务名称前缀
const KEYRING_SERVICE_PREFIX: &str = "prism-forge-llm-";

/// API Key 存储接口
///
/// 提供跨平台的 API Key 安全存储功能
pub struct ApiKeyStorage {
    _private: (), // 防止直接实例化
}

impl ApiKeyStorage {
    /// 保存 API Key 到系统密钥库
    ///
    /// # 参数
    /// - `provider_id`: 提供商 ID（例如：1, 2, 3）
    /// - `api_key`: API Key（使用 SecretString 保护内存中的数据）
    ///
    /// # 返回
    /// 成功时返回 Ok(())，失败时返回错误信息
    ///
    /// # 示例
    /// ```no_run
    /// use secrecy::SecretString;
    ///
    /// let api_key = SecretString::new("sk-1234567890".to_string());
    /// ApiKeyStorage::save_api_key(1, api_key)?;
    /// ```
    ///
    /// # 平台行为
    /// - **Windows**: 存储到 Credential Manager，条目名称为 `prism-forge-llm-provider-1`
    /// - **macOS**: 存储到 Keychain，需要用户授权
    /// - **Linux**: 存储到 Secret Service，依赖正在运行的密钥服务
    pub fn save_api_key(provider_id: i64, api_key: SecretString) -> Result<()> {
        let entry = Self::get_entry(provider_id)?;
        
        #[cfg(debug_assertions)]
        eprintln!("[ApiKeyStorage] Saving API Key for provider_id={}", provider_id);

        entry
            .set_password(api_key.expose_secret())
            .with_context(|| format!("保存 API Key 失败 (provider_id={})", provider_id))?;

        #[cfg(debug_assertions)]
        eprintln!("[ApiKeyStorage] API Key saved successfully for provider_id={}", provider_id);

        Ok(())
    }

    /// 从系统密钥库获取 API Key
    ///
    /// # 参数
    /// - `provider_id`: 提供商 ID
    ///
    /// # 返回
    /// 成功时返回包含 API Key 的 SecretString，失败时返回错误
    ///
    /// # 错误处理
    /// - 如果密钥不存在，返回 `KeyringError::NoEntry`
    /// - 如果密钥库访问被拒绝，返回 `KeyringError::PlatformFailure`
    ///
    /// # 示例
    /// ```no_run
    /// use secrecy::ExposeSecret;
    ///
    /// match ApiKeyStorage::get_api_key(1) {
    ///     Ok(key) => println!("API Key: {}", key.expose_secret().chars().take(4).collect::<String>() + "..."),
    ///     Err(e) => eprintln!("获取 API Key 失败: {}", e),
    /// }
    /// ```
    pub fn get_api_key(provider_id: i64) -> Result<SecretString> {
        let entry = Self::get_entry(provider_id)?;

        #[cfg(debug_assertions)]
        eprintln!("[ApiKeyStorage] Getting API Key for provider_id={}", provider_id);

        let password = entry
            .get_password()
            .with_context(|| format!("获取 API Key 失败 (provider_id={})", provider_id))?;

        #[cfg(debug_assertions)]
        eprintln!("[ApiKeyStorage] API Key retrieved successfully for provider_id={}", provider_id);

        Ok(SecretString::new(password.into()))
    }

    /// 删除 API Key
    ///
    /// # 参数
    /// - `provider_id`: 提供商 ID
    ///
    /// # 返回
    /// 成功时返回 Ok(())，如果密钥不存在也返回 Ok(())
    pub fn delete_api_key(provider_id: i64) -> Result<()> {
        let entry = Self::get_entry(provider_id)?;

        // 尝试删除，如果密钥不存在也不报错
        match entry.delete_credential() {
            Ok(_) => Ok(()),
            Err(KeyringError::NoEntry) => Ok(()), // 密钥不存在，视为成功
            Err(e) => Err(e).with_context(|| format!("删除 API Key 失败 (provider_id={})", provider_id)),
        }
    }

    /// 检查 API Key 是否存在
    ///
    /// # 参数
    /// - `provider_id`: 提供商 ID
    pub fn has_api_key(provider_id: i64) -> bool {
        match Self::get_entry(provider_id) {
            Ok(entry) => entry.get_password().is_ok(),
            Err(_) => false,
        }
    }

    /// 更新 API Key（如果已存在则覆盖）
    ///
    /// # 参数
    /// - `provider_id`: 提供商 ID
    /// - `api_key`: 新的 API Key
    pub fn update_api_key(provider_id: i64, api_key: SecretString) -> Result<()> {
        // keyring 的 set_password 会自动覆盖已存在的密钥
        Self::save_api_key(provider_id, api_key)
    }

    /// 获取所有已存储的提供商 ID 列表
    ///
    /// # 注意
    /// 此功能在不同平台的实现受限：
    /// - Windows: 可以枚举所有条目
    /// - macOS/Linux: 可能无法直接枚举，需要尝试已知 ID
    ///
    /// # 返回
    /// 返回已知存在的提供商 ID 列表
    pub fn list_provider_ids(known_ids: &[i64]) -> Vec<i64> {
        known_ids
            .iter()
            .filter(|&&id| Self::has_api_key(id))
            .copied()
            .collect()
    }

    /// 为指定提供商 ID 创建 keyring Entry
    ///
    /// # 格式
    /// - Service: `prism-forge-llm-provider-{id}`
    /// - Username: `provider-{id}`
    fn get_entry(provider_id: i64) -> Result<Entry> {
        let service = format!("{}provider-{}", KEYRING_SERVICE_PREFIX, provider_id);
        let username = format!("provider-{}", provider_id);

        Entry::new(&service, &username)
            .with_context(|| format!("创建 keyring Entry 失败 (provider_id={})", provider_id))
    }

    /// 获取密钥库类型信息（用于调试）
    ///
    /// # 返回
    /// 返回当前平台的密钥库类型描述
    #[allow(dead_code)]
    pub fn get_keyring_info() -> String {
        format!(
            "平台: {}, 密钥库可用: {}",
            std::env::consts::OS,
            if Self::has_api_key(-1) { "是" } else { "未知状态" }
        )
    }
}

/// API Key 验证工具
///
/// 用于验证和清理 API Key
pub struct ApiKeyValidator;

impl ApiKeyValidator {
    /// 验证 API Key 格式是否有效
    ///
    /// # 参数
    /// - `key`: API Key（SecretString）
    /// - `provider_type`: 提供商类型
    ///
    /// # 返回
    /// 返回验证结果
    pub fn validate_format(
        key: &SecretString,
        provider_type: crate::database::ApiProviderType,
    ) -> Result<()> {
        let key = key.expose_secret();

        if key.is_empty() {
            return Err(anyhow::anyhow!("API Key 不能为空"));
        }

        if key.len() < 10 {
            return Err(anyhow::anyhow!("API Key 长度无效"));
        }

        // 根据提供商类型验证格式
        match provider_type {
            crate::database::ApiProviderType::OpenAI => {
                // OpenAI Key 通常以 sk- 开头
                if !key.starts_with("sk-") && !key.starts_with("sk-") {
                    // 某些中转服务可能不以 sk- 开头，这里只做警告不报错
                    // return Err(anyhow::anyhow!("OpenAI Key 应该以 'sk-' 开头"));
                }
            }
            crate::database::ApiProviderType::Anthropic => {
                // Anthropic Key 通常以 sk-ant- 开头
                if !key.starts_with("sk-ant-") {
                    // 某些中转服务可能不同
                }
            }
            crate::database::ApiProviderType::Ollama => {
                // Ollama 不需要 API Key
            }
            crate::database::ApiProviderType::XAI => {
                // X AI Key 通常以 xai- 开头
                if !key.starts_with("xai-") {
                    // 某些情况可能不同，这里只做警告不报错
                }
            }
            crate::database::ApiProviderType::Google => {
                // Google ML Dev API Key 通常以 AIza 或 GOO 开头
                // Vertex AI 使用 OAuth2，不需要 API Key 验证
                if !key.starts_with("AIza") && !key.starts_with("GOO") {
                    // Google Cloud Service Account Key 可能不同
                    // 这里只做警告不报错
                }
            }
            crate::database::ApiProviderType::GoogleVertex => {
                // Google Vertex AI Public Preview 使用 Google Cloud API Key
                // 格式与 ML Dev API 类似
                if !key.starts_with("AIza") && !key.starts_with("GOO") {
                    // Google Cloud Service Account Key 可能不同
                    // 这里只做警告不报错
                }
            }
            crate::database::ApiProviderType::AzureOpenAI => {
                // Azure OpenAI Key 通常以 sk- 开头（与 OpenAI 格式相同）
                if !key.starts_with("sk-") {
                    // 某些中转服务可能不以 sk- 开头，这里只做警告不报错
                }
            }
            crate::database::ApiProviderType::OpenAICompatible => {
                // OpenAI 兼容接口的 API Key 格式不确定
                // 不做格式验证，只检查长度
            }
        }

        Ok(())
    }

    /// 掩码显示 API Key（用于日志或 UI）
    ///
    /// # 参数
    /// - `key`: API Key
    /// - `visible_chars`: 显示前几个字符
    ///
    /// # 返回
    /// 返回掩码后的字符串，例如 "sk-12...89ab"
    pub fn mask_api_key(key: &SecretString, visible_chars: usize) -> String {
        let key = key.expose_secret();

        if key.len() <= visible_chars + 4 {
            return format!("{}...", &key[..key.len().min(4)]);
        }

        format!(
            "{}...{}",
            &key[..visible_chars.min(key.len())],
            &key[key.len() - 4..]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_get_api_key() {
        let provider_id = 999999i64; // 使用测试 ID 避免冲突
        let test_key = SecretString::new("test-api-key-12345".to_string().into());

        // 保存
        ApiKeyStorage::save_api_key(provider_id, test_key.clone()).unwrap();

        // 获取
        let retrieved = ApiKeyStorage::get_api_key(provider_id).unwrap();
        assert_eq!(retrieved.expose_secret(), test_key.expose_secret());

        // 清理
        ApiKeyStorage::delete_api_key(provider_id).unwrap();
    }

    #[test]
    fn test_delete_api_key() {
        let provider_id = 999998i64;
        let test_key = SecretString::new("test-api-key-67890".to_string().into());

        // 保存
        ApiKeyStorage::save_api_key(provider_id, test_key).unwrap();
        assert!(ApiKeyStorage::has_api_key(provider_id));

        // 删除
        ApiKeyStorage::delete_api_key(provider_id).unwrap();
        assert!(!ApiKeyStorage::has_api_key(provider_id));
    }

    #[test]
    fn test_mask_api_key() {
        let key = SecretString::new("sk-1234567890abcdef".to_string().into());
        let masked = ApiKeyValidator::mask_api_key(&key, 8);
        assert_eq!(masked, "sk-12345...cdef");
    }

    #[test]
    fn test_validate_openai_key() {
        let valid_key = SecretString::new("sk-1234567890abcdef".to_string().into());
        assert!(ApiKeyValidator::validate_format(
            &valid_key,
            crate::database::ApiProviderType::OpenAI
        )
        .is_ok());

        let empty_key = SecretString::new("".to_string().into());
        assert!(ApiKeyValidator::validate_format(
            &empty_key,
            crate::database::ApiProviderType::OpenAI
        )
        .is_err());
    }
}
