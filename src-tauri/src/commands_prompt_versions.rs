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

/// 初始化提示词模板（从现有的 Prompts 表创建）
///
/// 这个命令会从 prompts 表中读取提示词内容，并在 prompt_templates 表中创建对应的模板和初始版本
#[tauri::command]
pub async fn cmd_initialize_prompt_template_from_prompt(
    prompt_name: String,
) -> Result<PromptTemplate, String> {
    use crate::database::prompts::PromptRepository;

    // 创建仓库实例（不使用 with_conn_inner，因为它不是公共方法）
    let conn_shared = crate::database::init::get_connection_shared()
        .map_err(|e| format!("获取数据库连接失败: {}", e))?;

    // 使用 try_lock 并带超时机制，防止 Mutex 毒化导致 Panic
    use std::time::{Duration, Instant};

    const LOCK_TIMEOUT: Duration = Duration::from_secs(5);
    let start = Instant::now();

    let conn_guard = loop {
        match conn_shared.try_lock() {
            Ok(guard) => break guard,
            Err(e) => {
                if start.elapsed() > LOCK_TIMEOUT {
                    let is_poisoned = matches!(e, std::sync::TryLockError::Poisoned(_));
                    return Err(format!(
                        "获取数据库连接超时（5秒），可能原因：{}。建议重启应用",
                        if is_poisoned { "Mutex 已被毒化" } else { "锁竞争激烈" }
                    ));
                }
                // 短暂休眠后重试
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    };

    // try_lock 返回的 MutexGuard 已经处理了毒化情况，直接使用
    let conn = &*conn_guard;

    // 执行数据库操作
    (|conn: &rusqlite::Connection| -> Result<PromptTemplate, String> {
        // 从 prompts 表读取提示词
        let prompt = PromptRepository::get_by_name(conn, &prompt_name)
            .map_err(|e| format!("查询提示词失败: {}", e))?
            .ok_or_else(|| format!("提示词不存在: {}", prompt_name))?;

        conn.execute("BEGIN TRANSACTION", [])
            .map_err(|e| format!("开始事务失败: {}", e))?;

        // 检查模板是否已存在
        let existing_template = conn.query_row(
            "SELECT id FROM prompt_templates WHERE name = ?1",
            params![&prompt_name],
            |row| row.get::<_, i64>(0),
        );

        let template_id = if let Ok(id) = existing_template {
            // 模板已存在，跳过创建
            id
        } else {
            // 创建模板（tags 使用空字符串，因为 Prompt 没有 tags 字段）
            let now = chrono::Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO prompt_templates (name, description, scenario, tags, language, is_system, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    &prompt_name,
                    &prompt.description,
                    &prompt.scenario,
                    "", // tags 使用空字符串
                    &prompt.language,
                    prompt.is_system as i32,
                    &now,
                    &now,
                ],
            ).map_err(|e| format!("创建模板失败: {}", e))?;
            conn.last_insert_rowid()
        };

        // 检查是否已有版本
        let version_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM prompt_versions WHERE template_id = ?1",
            params![template_id],
            |row| row.get(0),
        ).map_err(|e| {
            #[cfg(debug_assertions)]
            log::error!("查询版本数量失败: {}", e);
            format!("查询版本数量失败: {}", e)
        })?;

        #[cfg(debug_assertions)]
        log::debug!("模板 {} 的版本数量: {}", template_id, version_count);

        if version_count == 0 {
            // 创建初始版本 (v1)，created_by 使用 "system"
            let now = chrono::Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO prompt_versions (template_id, version_number, is_active, content, created_by, created_at)
                 VALUES (?1, 1, 1, ?2, ?3, ?4)",
                params![
                    template_id,
                    &prompt.content,
                    "system", // created_by 固定使用 "system"
                    &now,
                ],
            ).map_err(|e| format!("创建版本失败: {}", e))?;
        }

        // 提交事务
        conn.execute("COMMIT", [])
            .map_err(|e| format!("提交事务失败: {}", e))?;

        // 读取并返回模板
        let template = conn.query_row(
            "SELECT id, name, description, scenario, tags, language, is_system, created_at, updated_at
             FROM prompt_templates WHERE id = ?1",
            params![template_id],
            |row| Ok(PromptTemplate {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                scenario: row.get(3)?,
                tags: row.get(4)?,
                language: row.get(5)?,
                is_system: row.get::<_, i32>(6)? == 1,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            }),
        ).map_err(|e| format!("读取模板失败: {}", e))?;

        Ok(template)
    })(&*conn)
}

/// 初始化提示词模板（从 TOML 导入版本 1）
#[tauri::command]
pub async fn cmd_initialize_prompt_template_from_toml(
    _template_name: String,
    _toml_path: String,
) -> Result<PromptTemplate, String> {
    // TODO: 实现 TOML 解析和初始化逻辑
    // 这将在下一步实现
    Err("尚未实现 TOML 初始化逻辑".to_string())
}
