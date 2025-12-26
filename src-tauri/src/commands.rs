//! Tauri Commands - LLM API Provider 管理
//!
//! 暴露给前端调用的命令接口

use tauri::State;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};

use crate::llm::LLMClientManager;
use crate::database::{ApiProvider, ApiProviderType, ApiProviderRepository};
use crate::llm::security::ApiKeyStorage;

/// 创建/更新提供商的请求参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveProviderRequest {
    /// 提供商 ID（更新时需要，创建时为 null）
    pub id: Option<i64>,

    /// 提供商类型
    pub provider_type: ApiProviderType,

    /// 用户自定义名称
    pub name: String,

    /// API 基础 URL
    pub base_url: String,

    /// API Key（明文，仅用于传输）
    pub api_key: Option<String>,

    /// 额外配置 JSON
    pub config_json: Option<String>,

    /// 是否设置为活跃提供商
    pub is_active: bool,
}

/// 提供商响应（包含敏感信息掩码）
/// 
/// 注意：手动展开 ApiProvider 字段以确保 camelCase 序列化一致性
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderResponse {
    /// 主键 ID
    pub id: Option<i64>,

    /// 提供商类型
    pub provider_type: ApiProviderType,

    /// 用户自定义名称
    pub name: String,

    /// API 基础 URL
    pub base_url: String,

    /// API Key 引用标识
    pub api_key_ref: Option<String>,

    /// 额外配置 JSON
    pub config_json: Option<String>,

    /// 是否为当前活跃的提供商
    pub is_active: bool,

    /// API Key 是否已配置（掩码显示）
    pub has_api_key: bool,

    /// API Key 掩码显示（前 8 个字符）
    pub api_key_mask: Option<String>,
}

impl From<ApiProvider> for ProviderResponse {
    fn from(provider: ApiProvider) -> Self {
        Self {
            id: provider.id,
            provider_type: provider.provider_type,
            name: provider.name,
            base_url: provider.base_url,
            api_key_ref: provider.api_key_ref,
            config_json: provider.config_json,
            is_active: provider.is_active,
            has_api_key: false,
            api_key_mask: None,
        }
    }
}

/// 命令错误响应
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub message: String,
}

impl From<anyhow::Error> for CommandError {
    fn from(err: anyhow::Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

/// 获取所有 API 提供商
///
/// # 返回
/// 返回所有提供商列表，按创建时间倒序排列
#[tauri::command]
pub fn cmd_get_providers(
    manager: State<'_, LLMClientManager>,
) -> std::result::Result<Vec<ProviderResponse>, CommandError> {
    let providers = manager.get_all_providers()?;

    let responses: Vec<ProviderResponse> = providers
        .into_iter()
        .map(|provider| {
            let provider_id = provider.id;
            let has_api_key = provider_id
                .map(|id| ApiKeyStorage::has_api_key(id))
                .unwrap_or(false);

            // 掩码显示 API Key（不回显完整密钥）
            let api_key_mask = if has_api_key {
                // 从 keyring 读取并掩码
                provider_id.and_then(|id| {
                    ApiKeyStorage::get_api_key(id)
                        .ok()
                        .map(|key| crate::llm::security::ApiKeyValidator::mask_api_key(&key, 8))
                })
            } else {
                None
            };

            ProviderResponse {
                id: provider.id,
                provider_type: provider.provider_type,
                name: provider.name,
                base_url: provider.base_url,
                api_key_ref: provider.api_key_ref,
                config_json: provider.config_json,
                is_active: provider.is_active,
                has_api_key,
                api_key_mask,
            }
        })
        .collect();

    Ok(responses)
}

/// 保存 API 提供商（创建或更新）
///
/// # 流程
/// 1. 如果有 id，则更新现有提供商；否则创建新提供商
/// 2. 如果提供了 api_key，则保存到 keyring
/// 3. 更新数据库中的提供商信息
#[tauri::command]
pub async fn cmd_save_provider(
    _manager: State<'_, LLMClientManager>,
    request: SaveProviderRequest,
) -> std::result::Result<ApiProvider, CommandError> {
    let repo = ApiProviderRepository::from_default_db()?;

    // 创建或更新提供商
    let mut provider = if let Some(id) = request.id {
        // 更新现有提供商
        let existing = repo
            .get_provider_by_id(id)?
            .ok_or_else(|| anyhow::anyhow!("提供商不存在 (id={})", id))?;

        ApiProvider {
            id: existing.id,
            provider_type: request.provider_type,
            name: request.name,
            base_url: request.base_url,
            api_key_ref: existing.api_key_ref,
            config_json: request.config_json,
            is_active: request.is_active,
        }
    } else {
        // 创建新提供商
        let new_provider = ApiProvider::new(
            request.provider_type,
            request.name.clone(),
            Some(request.base_url.clone()),
        );

        // 先插入数据库获取 ID
        let mut created = repo.create_provider(new_provider)?;

        // 设置 api_key_ref
        if request.provider_type.requires_api_key() {
            created.api_key_ref = Some(format!("provider_{}", created.id.unwrap()));
        }

        created
    };

    // 处理 API Key
    if let Some(api_key_str) = request.api_key {
        if !api_key_str.is_empty() {
            let provider_id = provider.id.ok_or_else(|| anyhow::anyhow!("提供商 ID 无效"))?;

            #[cfg(debug_assertions)]
            eprintln!("[cmd_save_provider] Saving API Key for provider_id={}", provider_id);

            // 验证 API Key 格式
            let api_key = SecretString::new(api_key_str.into());
            crate::llm::security::ApiKeyValidator::validate_format(
                &api_key,
                provider.provider_type,
            )?;

            // 保存到 keyring
            ApiKeyStorage::save_api_key(provider_id, api_key)?;

            #[cfg(debug_assertions)]
            eprintln!("[cmd_save_provider] API Key saved successfully");

            // 更新 api_key_ref
            if provider.api_key_ref.is_none() {
                provider.api_key_ref = Some(format!("provider_{}", provider_id));
            }
        }
    }

    // 如果 is_active 为 true，需要先设置其他提供商为非活跃
    if provider.is_active {
        // 获取当前活跃的提供商
        if let Ok(Some(active)) = repo.get_active_provider() {
            if active.id != provider.id {
                // 取消之前的活跃状态
                let mut inactive = active.clone();
                inactive.is_active = false;
                repo.update_provider(&inactive)?;
            }
        }
    }

    // 保存/更新到数据库
    if provider.id.is_some() {
        repo.update_provider(&provider)?;
    } else {
        provider = repo.create_provider(provider)?;
    }

    Ok(provider)
}

/// 删除 API 提供商
///
/// # 流程
/// 1. 从 keyring 删除 API Key
/// 2. 从数据库删除提供商记录
#[tauri::command]
pub fn cmd_delete_provider(
    _manager: State<'_, LLMClientManager>,
    id: i64,
) -> std::result::Result<(), CommandError> {
    // 先从 keyring 删除 API Key
    ApiKeyStorage::delete_api_key(id)?;

    // 从数据库删除提供商
    let repo = ApiProviderRepository::from_default_db()?;
    repo.delete_provider(id)?;

    Ok(())
}

/// 设置活跃提供商
///
/// # 流程
/// 1. 将指定的提供商设置为活跃
/// 2. 数据库触发器会自动将其他提供商设置为非活跃
#[tauri::command]
pub fn cmd_set_active_provider(
    manager: State<'_, LLMClientManager>,
    id: i64,
) -> std::result::Result<(), CommandError> {
    manager.switch_provider(id)?;
    Ok(())
}

/// 测试提供商连接
///
/// # 流程
/// 1. 获取提供商配置
/// 2. 创建对应的客户端
/// 3. 发送测试请求
///
/// # 返回
/// 返回测试是否成功
#[tauri::command]
pub async fn cmd_test_provider_connection(
    manager: State<'_, LLMClientManager>,
    id: i64,
) -> std::result::Result<bool, CommandError> {
    let success = manager.test_provider(id).await?;
    Ok(success)
}
