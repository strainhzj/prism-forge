//! 提示词版本管理 Tauri 命令
//!
//! 提供前端调用的提示词版本管理接口

use crate::database::prompt_versions::PromptVersionRepository;
use crate::database::models::{
    PromptTemplate, PromptVersion, PromptComponent, PromptParameter,
    PromptVersionDiff,
};
use rusqlite::params;

/// 获取所有提示词模板
#[tauri::command]
pub async fn cmd_get_prompt_templates() -> Result<Vec<PromptTemplate>, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.list_templates()
        .map_err(|e| e.to_string())
}

/// 根据名称获取提示词模板
#[tauri::command]
pub async fn cmd_get_prompt_template_by_name(name: String) -> Result<Option<PromptTemplate>, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.get_template_by_name(&name)
        .map_err(|e| e.to_string())
}

/// 根据场景获取提示词模板
#[tauri::command]
pub async fn cmd_get_prompt_template_by_scenario(scenario: String) -> Result<Option<PromptTemplate>, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.get_template_by_scenario(&scenario)
        .map_err(|e| e.to_string())
}

/// 获取模板的所有版本
#[tauri::command]
pub async fn cmd_get_prompt_versions(template_id: i64) -> Result<Vec<PromptVersion>, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.list_versions(template_id)
        .map_err(|e| e.to_string())
}

/// 获取模板的激活版本
#[tauri::command]
pub async fn cmd_get_active_prompt_version(template_id: i64) -> Result<Option<PromptVersion>, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.get_active_version(template_id)
        .map_err(|e| e.to_string())
}

/// 根据版本号获取版本
#[tauri::command]
pub async fn cmd_get_prompt_version_by_number(
    template_id: i64,
    version_number: i32,
) -> Result<Option<PromptVersion>, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.get_version_by_number(template_id, version_number)
        .map_err(|e| e.to_string())
}

/// 激活指定版本（软回滚）
#[tauri::command]
pub async fn cmd_activate_prompt_version(
    template_id: i64,
    version_number: i32,
) -> Result<PromptVersion, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.activate_version(template_id, version_number)
        .map_err(|e| e.to_string())
}

/// 硬回滚：创建新版本，复制目标版本的内容并激活
#[tauri::command]
pub async fn cmd_rollback_prompt_version_hard(
    template_id: i64,
    version_number: i32,
    comment: Option<String>,
) -> Result<PromptVersion, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.rollback_to_version_hard(template_id, version_number, comment, "user")
        .map_err(|e| e.to_string())
}

/// 保存新版本（创建并激活）
#[tauri::command]
pub async fn cmd_save_prompt_version(
    template_id: i64,
    content: String,
    components: Vec<PromptComponent>,
    parameters: Vec<PromptParameter>,
    created_by: String,
) -> Result<PromptVersion, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.create_and_activate_version(
        template_id,
        content,
        components,
        parameters,
        &created_by,
    )
    .map_err(|e| e.to_string())
}

/// 对比两个版本
#[tauri::command]
pub async fn cmd_compare_prompt_versions(
    template_id: i64,
    from_version: i32,
    to_version: i32,
) -> Result<PromptVersionDiff, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.compare_versions(template_id, from_version, to_version)
        .map_err(|e| e.to_string())
}

/// 获取版本的所有组件
#[tauri::command]
pub async fn cmd_get_prompt_components(version_id: i64) -> Result<Vec<PromptComponent>, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.list_components(version_id)
        .map_err(|e| e.to_string())
}

/// 获取版本的所有参数
#[tauri::command]
pub async fn cmd_get_prompt_parameters(version_id: i64) -> Result<Vec<PromptParameter>, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.list_parameters(version_id)
        .map_err(|e| e.to_string())
}

/// 获取版本之间的变更记录
#[tauri::command]
pub async fn cmd_get_prompt_version_changes(
    template_id: i64,
    from_version: Option<i32>,
    to_version: i32,
) -> Result<Vec<crate::database::models::PromptChange>, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.get_changes_between_versions(template_id, from_version, to_version)
        .map_err(|e| e.to_string())
}

/// 统一的提示词列表查询接口（从版本管理系统读取）
///
/// 这个命令用于替代旧的 cmd_get_prompts，从 prompt_templates 和 prompt_versions
/// 读取数据并扁平化输出，支持多语言版本的展示
///
/// # 参数
/// - `scenario`: 可选的场景过滤（如 "session_analysis"）
/// - `language`: 可选的语言过滤（如 "zh"、"en"）
/// - `search`: 可选的搜索关键词（匹配 name、description、content）
///
/// # 返回
/// 返回一个扁平化的提示词列表，每个元素包含：
/// - id: 版本 ID（兼容旧的前端代码）
/// - template_id: 模板 ID
/// - name: 模板名称
/// - content: 版本内容
/// - description: 模板描述
/// - scenario: 场景
/// - language: 从 metadata 中提取的语言
/// - is_system: 是否系统级
/// - is_active: 版本是否激活
/// - version_number: 版本号
/// - created_at: 创建时间
#[tauri::command]
pub async fn cmd_get_prompts_unified(
    scenario: Option<String>,
    language: Option<String>,
    search: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.list_prompts_unified(
        scenario.as_deref(),
        language.as_deref(),
        search.as_deref(),
    )
    .map_err(|e| e.to_string())
}
