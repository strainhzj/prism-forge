//! 提示词管理 Tauri 命令
//!
//! 提供前端调用的提示词 CRUD 接口

use crate::database::prompts::PromptRepository;
use crate::database::get_connection_shared;
use crate::database::models::Prompt;

/// 获取提示词列表
#[tauri::command]
pub async fn cmd_get_prompts(
    scenario: Option<String>,
    language: Option<String>,
    search: Option<String>,
) -> Result<Vec<Prompt>, String> {
    let conn = get_connection_shared()
        .map_err(|e| format!("获取数据库连接失败: {}", e))?;

    let conn_guard = conn.lock()
        .map_err(|e| format!("获取数据库锁失败: {}", e))?;

    PromptRepository::list(
        &conn_guard,
        scenario.as_deref(),
        language.as_deref(),
        search.as_deref(),
    )
    .map_err(|e| e.to_string())
}

/// 获取单个提示词
#[tauri::command]
pub async fn cmd_get_prompt(id: i64) -> Result<Option<Prompt>, String> {
    let conn = get_connection_shared()
        .map_err(|e| format!("获取数据库连接失败: {}", e))?;

    let conn_guard = conn.lock()
        .map_err(|e| format!("获取数据库锁失败: {}", e))?;

    PromptRepository::get(&conn_guard, id)
        .map_err(|e| e.to_string())
}

/// 保存提示词（创建或更新）
#[tauri::command]
pub async fn cmd_save_prompt(
    prompt: Prompt,
) -> Result<i64, String> {
    let conn = get_connection_shared()
        .map_err(|e| format!("获取数据库连接失败: {}", e))?;

    let conn_guard = conn.lock()
        .map_err(|e| format!("获取数据库锁失败: {}", e))?;

    if let Some(id) = prompt.id {
        // 更新现有提示词
        PromptRepository::update(&conn_guard, &prompt)
            .map_err(|e| e.to_string())?;
        Ok(id)
    } else {
        // 创建新提示词
        PromptRepository::create(&conn_guard, &prompt)
            .map_err(|e| e.to_string())
    }
}

/// 删除提示词
#[tauri::command]
pub async fn cmd_delete_prompt(id: i64) -> Result<(), String> {
    let conn = get_connection_shared()
        .map_err(|e| format!("获取数据库连接失败: {}", e))?;

    let conn_guard = conn.lock()
        .map_err(|e| format!("获取数据库锁失败: {}", e))?;

    PromptRepository::delete(&conn_guard, id)
        .map_err(|e| e.to_string())
}

/// 重置为默认提示词
#[tauri::command]
pub async fn cmd_reset_default_prompt(name: String) -> Result<(), String> {
    let conn = get_connection_shared()
        .map_err(|e| format!("获取数据库连接失败: {}", e))?;

    let conn_guard = conn.lock()
        .map_err(|e| format!("获取数据库锁失败: {}", e))?;

    PromptRepository::reset_default(&conn_guard, &name)
        .map_err(|e| e.to_string())
}
