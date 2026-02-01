//! 提示词版本管理 Repository
//!
//! 提供提示词模板、版本、组件、参数的 CRUD 操作
//! 以及版本对比和回滚功能

use anyhow::Result;
use rusqlite::{Connection, params};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use similar::{ChangeTag, TextDiff};

use crate::database::models::{
    PromptTemplate, PromptVersion, PromptComponent, PromptComponentType,
    PromptParameter, PromptParameterType, PromptChange, ChangeType,
    PromptVersionDiff, ComponentDiff, LineDiff, LineChangeType,
    ParameterDiff, MetadataDiff,
};

/// 提示词版本管理 Repository
pub struct PromptVersionRepository {
    conn: Arc<Mutex<Connection>>,
}

// Arc<Mutex<Connection>> 已经自动实现了 Send + Sync
// 无需手动添加 unsafe impl

/// 安全地将 i32 转换为 bool
///
/// 防御性设计：验证数据库中的布尔值只能是 0 或 1
/// 其他值（如脏数据 2、-1）会导致错误而非静默降级
fn bool_from_i32(value: i32, field_name: &str) -> std::result::Result<bool, rusqlite::Error> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        other => {
            #[cfg(debug_assertions)]
            log::warn!(
                "字段 {} 的值不是有效的布尔值（0/1）: {}，将返回错误",
                field_name,
                other
            );

            Err(rusqlite::Error::InvalidParameterName(format!(
                "字段 {} 的值不是有效的布尔值（0/1）: {}",
                field_name, other
            )))
        }
    }
}

impl PromptVersionRepository {
    /// 使用共享连接创建仓库实例
    pub fn with_conn(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// 从默认数据库路径创建仓库
    pub fn from_default_db() -> Result<Self> {
        let conn = crate::database::init::get_connection_shared()?;
        Ok(Self::with_conn(conn))
    }

    /// 辅助方法：获取连接锁（带重试和超时机制）
    ///
    /// 防御性设计：
    /// - 最多重试 3 次
    /// - 单次锁定超时 10 秒
    /// - 检测 Mutex 毒化状态
    /// - 提供详细的错误信息
    fn with_conn_inner<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&rusqlite::Connection) -> Result<R>,
    {
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAY: Duration = Duration::from_millis(100);
        const LOCK_TIMEOUT: Duration = Duration::from_secs(10);

        let start = Instant::now();
        let mut attempts = 0;

        loop {
            attempts += 1;

            match self.conn.try_lock() {
                Ok(guard) => {
                    // 成功获取锁，执行数据库操作
                    return f(&guard).map_err(|e| {
                        anyhow::anyhow!("数据库操作失败: {}", e)
                    });
                }
                Err(e) => {
                    let is_poisoned = matches!(e, std::sync::TryLockError::Poisoned(_));
                    let elapsed = start.elapsed();

                    // 判断是否应该继续重试
                    if attempts >= MAX_RETRIES || elapsed > LOCK_TIMEOUT {
                        return Err(anyhow::anyhow!(
                            "获取数据库连接锁失败（尝试 {}/{}，耗时 {:?}）：{}",
                            attempts,
                            MAX_RETRIES,
                            elapsed,
                            if is_poisoned { "Mutex 已被毒化" } else { "锁竞争超时" }
                        ));
                    }

                    #[cfg(debug_assertions)]
                    log::debug!(
                        "获取锁失败（尝试 {}/{}），{:?} 后重试: {}",
                        attempts,
                        MAX_RETRIES,
                        RETRY_DELAY,
                        if is_poisoned { "已毒化" } else { "繁忙" }
                    );

                    // 短暂休眠后重试
                    std::thread::sleep(RETRY_DELAY);
                }
            }
        }
    }

    // ============================================================================
    // 模板管理 (Template Management)
    // ============================================================================

    /// 创建新的提示词模板
    pub fn create_template(&self, template: &PromptTemplate) -> Result<i64> {
        let now = chrono::Utc::now().to_rfc3339();

        let id = self.with_conn_inner(|conn| {
            conn.execute(
                "INSERT INTO prompt_templates (
                    name, description, scenario, tags, language, is_system, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    &template.name,
                    &template.description,
                    &template.scenario,
                    &template.tags,
                    &template.language,
                    template.is_system as i32,
                    now.clone(),
                    now,
                ],
            )?;
            Ok(conn.last_insert_rowid())
        })?;

        Ok(id)
    }

    /// 获取所有模板
    pub fn list_templates(&self) -> Result<Vec<PromptTemplate>> {
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, description, scenario, tags, language, is_system, created_at, updated_at
                 FROM prompt_templates
                 ORDER BY created_at DESC"
            )?;

            let templates = stmt.query_map([], |row| {
                Ok(PromptTemplate {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    description: row.get(2)?,
                    scenario: row.get(3)?,
                    tags: row.get(4)?,
                    language: row.get(5)?,
                    is_system: bool_from_i32(row.get(6)?, "is_system")?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })?;

            templates.collect::<Result<Vec<_>, _>>().map_err(Into::into)
        })
    }

    /// 根据名称获取模板
    pub fn get_template_by_name(&self, name: &str) -> Result<Option<PromptTemplate>> {
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, description, scenario, tags, language, is_system, created_at, updated_at
                 FROM prompt_templates WHERE name = ?1"
            )?;

            let mut rows = stmt.query(params![name])?;

            if let Some(row) = rows.next()? {
                Ok(Some(PromptTemplate {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    description: row.get(2)?,
                    scenario: row.get(3)?,
                    tags: row.get(4)?,
                    language: row.get(5)?,
                    is_system: bool_from_i32(row.get(6)?, "is_system")?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                }))
            } else {
                Ok(None)
            }
        })
    }

    /// 根据场景获取模板
    pub fn get_template_by_scenario(&self, scenario: &str) -> Result<Option<PromptTemplate>> {
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, description, scenario, tags, language, is_system, created_at, updated_at
                 FROM prompt_templates WHERE scenario = ?1"
            )?;

            let mut rows = stmt.query(params![scenario])?;

            if let Some(row) = rows.next()? {
                Ok(Some(PromptTemplate {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    description: row.get(2)?,
                    scenario: row.get(3)?,
                    tags: row.get(4)?,
                    language: row.get(5)?,
                    is_system: bool_from_i32(row.get(6)?, "is_system")?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                }))
            } else {
                Ok(None)
            }
        })
    }

    // ============================================================================
    // 版本管理 (Version Management)
    // ============================================================================

    /// 创建新版本
    pub fn create_version(&self, version: &PromptVersion) -> Result<i64> {
        let id = self.with_conn_inner(|conn| {
            conn.execute(
                "INSERT INTO prompt_versions (
                    template_id, version_number, is_active, content, metadata, created_by, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    &version.template_id,
                    &version.version_number,
                    version.is_active as i32,
                    &version.content,
                    &version.metadata,
                    &version.created_by,
                    &version.created_at,
                ],
            )?;
            Ok(conn.last_insert_rowid())
        })?;

        Ok(id)
    }

    /// 获取模板的所有版本
    pub fn list_versions(&self, template_id: i64) -> Result<Vec<PromptVersion>> {
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, template_id, version_number, is_active, content, metadata, created_by, created_at
                 FROM prompt_versions
                 WHERE template_id = ?1
                 ORDER BY version_number DESC"
            )?;

            let versions = stmt.query_map(params![template_id], |row| {
                Ok(PromptVersion {
                    id: Some(row.get(0)?),
                    template_id: row.get(1)?,
                    version_number: row.get(2)?,
                    is_active: bool_from_i32(row.get(3)?, "is_active")?,
                    content: row.get(4)?,
                    metadata: row.get(5)?,
                    created_by: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })?;

            versions.collect::<Result<Vec<_>, _>>().map_err(Into::into)
        })
    }

    /// 获取激活版本
    pub fn get_active_version(&self, template_id: i64) -> Result<Option<PromptVersion>> {
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, template_id, version_number, is_active, content, metadata, created_by, created_at
                 FROM prompt_versions
                 WHERE template_id = ?1 AND is_active = 1"
            )?;

            let mut rows = stmt.query(params![template_id])?;

            if let Some(row) = rows.next()? {
                Ok(Some(PromptVersion {
                    id: Some(row.get(0)?),
                    template_id: row.get(1)?,
                    version_number: row.get(2)?,
                    is_active: bool_from_i32(row.get(3)?, "is_active")?,
                    content: row.get(4)?,
                    metadata: row.get(5)?,
                    created_by: row.get(6)?,
                    created_at: row.get(7)?,
                }))
            } else {
                Ok(None)
            }
        })
    }

    /// 根据版本号获取版本
    pub fn get_version_by_number(&self, template_id: i64, version_number: i32) -> Result<Option<PromptVersion>> {
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, template_id, version_number, is_active, content, metadata, created_by, created_at
                 FROM prompt_versions
                 WHERE template_id = ?1 AND version_number = ?2"
            )?;

            let mut rows = stmt.query(params![template_id, version_number])?;

            if let Some(row) = rows.next()? {
                Ok(Some(PromptVersion {
                    id: Some(row.get(0)?),
                    template_id: row.get(1)?,
                    version_number: row.get(2)?,
                    is_active: bool_from_i32(row.get(3)?, "is_active")?,
                    content: row.get(4)?,
                    metadata: row.get(5)?,
                    created_by: row.get(6)?,
                    created_at: row.get(7)?,
                }))
            } else {
                Ok(None)
            }
        })
    }

    /// 激活指定版本（软回滚）
    ///
    /// 使用 rusqlite 事务 API，自动处理回滚
    pub fn activate_version(&self, template_id: i64, version_number: i32) -> Result<PromptVersion> {
        self.with_conn_inner(|conn| {
            // 使用 unchecked_transaction 自动管理事务
            // Drop 时如果未 commit 会自动回滚
            let tx = conn.unchecked_transaction()?;

            // 获取目标版本
            let target_version: PromptVersion = tx.query_row(
                "SELECT id, template_id, version_number, is_active, content, metadata, created_by, created_at
                 FROM prompt_versions
                 WHERE template_id = ?1 AND version_number = ?2",
                params![template_id, version_number],
                |row| Ok(PromptVersion {
                    id: Some(row.get(0)?),
                    template_id: row.get(1)?,
                    version_number: row.get(2)?,
                    is_active: bool_from_i32(row.get(3)?, "is_active")?,
                    content: row.get(4)?,
                    metadata: row.get(5)?,
                    created_by: row.get(6)?,
                    created_at: row.get(7)?,
                }),
            )?;

            // 取消当前激活版本
            tx.execute(
                "UPDATE prompt_versions SET is_active = 0 WHERE template_id = ?1 AND is_active = 1",
                params![template_id],
            )?;

            // 激活目标版本（安全地提取 ID）
            let target_version_id = target_version.id.ok_or_else(|| {
                anyhow::anyhow!(
                    "目标版本缺少 ID 字段，无法激活: template_id={}, version_number={}",
                    template_id, version_number
                )
            })?;

            tx.execute(
                "UPDATE prompt_versions SET is_active = 1 WHERE id = ?1",
                params![&target_version_id],
            )?;

            // 提交事务（如果这里失败，Drop 时会自动回滚）
            tx.commit()?;

            Ok(target_version)
        })
    }

    /// 硬回滚：创建新版本，复制目标版本的内容并激活
    ///
    /// 这会保留完整的历史记录，创建一个新的版本号
    /// 使用 rusqlite 事务 API，自动处理回滚
    pub fn rollback_to_version_hard(
        &self,
        template_id: i64,
        version_number: i32,
        comment: Option<String>,
        rolled_back_by: &str,
    ) -> Result<PromptVersion> {
        self.with_conn_inner(|conn| {
            // 使用事务自动管理回滚
            let tx = conn.unchecked_transaction()?;

            // 获取目标版本
            let target_version: PromptVersion = tx.query_row(
                "SELECT id, template_id, version_number, is_active, content, metadata, created_by, created_at
                 FROM prompt_versions
                 WHERE template_id = ?1 AND version_number = ?2",
                params![template_id, version_number],
                |row| Ok(PromptVersion {
                    id: Some(row.get(0)?),
                    template_id: row.get(1)?,
                    version_number: row.get(2)?,
                    is_active: bool_from_i32(row.get(3)?, "is_active")?,
                    content: row.get(4)?,
                    metadata: row.get(5)?,
                    created_by: row.get(6)?,
                    created_at: row.get(7)?,
                }),
            )?;

            // 获取下一个版本号
            let next_version: i32 = tx.query_row(
                "SELECT COALESCE(MAX(version_number), 0) + 1 FROM prompt_versions WHERE template_id = ?1",
                params![template_id],
                |row| row.get(0),
            )?;

            let now = chrono::Utc::now().to_rfc3339();

            // 取消当前激活版本
            tx.execute(
                "UPDATE prompt_versions SET is_active = 0 WHERE template_id = ?1 AND is_active = 1",
                params![template_id],
            )?;

            // 创建新版本，复制目标版本的内容
            tx.execute(
                "INSERT INTO prompt_versions (
                    template_id, version_number, is_active, content, metadata, created_by, created_at
                ) VALUES (?1, ?2, 1, ?3, ?4, ?5, ?6)",
                params![
                    template_id,
                    next_version,
                    &target_version.content,
                    &comment, // 使用 comment 作为新版本的 metadata
                    rolled_back_by,
                    &now,
                ],
            )?;
            let new_version_id = tx.last_insert_rowid();

            // 复制目标版本的组件（安全地提取 ID）
            let target_version_id = target_version.id.ok_or_else(|| {
                anyhow::anyhow!(
                    "目标版本缺少 ID，无法复制组件: template_id={}, version_number={}",
                    template_id, version_number
                )
            })?;

            // 使用作用域限制 stmt 的生命周期
            {
                let mut stmt = tx.prepare(
                    "SELECT id, version_id, component_type, name, content, variables, language, sort_order
                     FROM prompt_components
                     WHERE version_id = ?1"
                )?;

                let components = stmt.query_map(params![target_version_id], |row| {
                    Ok((
                        row.get::<_, String>(2)?,  // component_type
                        row.get::<_, String>(3)?,  // name
                        row.get::<_, String>(4)?,  // content
                        row.get::<_, Option<String>>(5)?,  // variables
                        row.get::<_, String>(6)?,  // language
                        row.get::<_, i32>(7)?,     // sort_order
                    ))
                })?;

                for component in components {
                    let (component_type, name, content, variables, language, sort_order) = component?;
                    tx.execute(
                        "INSERT INTO prompt_components (
                            version_id, component_type, name, content, variables, language, sort_order
                        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        params![
                            new_version_id,
                            component_type,
                            name,
                            content,
                            variables,
                            language,
                            sort_order,
                        ],
                    )?;
                }
            } // stmt 在这里被 drop

            // 复制目标版本的参数
            {
                let mut stmt = tx.prepare(
                    "SELECT id, version_id, key, value, parameter_type, description
                     FROM prompt_parameters
                     WHERE version_id = ?1"
                )?;

                let parameters = stmt.query_map(params![target_version_id], |row| {
                    Ok((
                        row.get::<_, String>(2)?,  // key
                        row.get::<_, String>(3)?,  // value
                        row.get::<_, String>(4)?,  // parameter_type
                        row.get::<_, Option<String>>(5)?,  // description
                    ))
                })?;

                for parameter in parameters {
                    let (key, value, parameter_type, description) = parameter?;
                    tx.execute(
                        "INSERT INTO prompt_parameters (
                            version_id, key, value, parameter_type, description
                        ) VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![
                            new_version_id,
                            key,
                            value,
                            parameter_type,
                            description,
                        ],
                    )?;
                }
            } // stmt 在这里被 drop

            // 提交事务（如果失败，Drop 时自动回滚）
            tx.commit()?;

            Ok(PromptVersion {
                id: Some(new_version_id),
                template_id,
                version_number: next_version,
                is_active: true,
                content: target_version.content,
                metadata: comment,
                created_by: rolled_back_by.to_string(),
                created_at: now,
            })
        })
    }

    /// 创建新版本并激活（用于保存修改）
    ///
    /// 使用 rusqlite 事务 API，自动处理回滚
    pub fn create_and_activate_version(
        &self,
        template_id: i64,
        content: String,
        components: Vec<PromptComponent>,
        parameters: Vec<PromptParameter>,
        created_by: &str,
    ) -> Result<PromptVersion> {
        self.with_conn_inner(|conn| {
            // 使用事务自动管理回滚
            let tx = conn.unchecked_transaction()?;

            // 获取下一个版本号
            let next_version: i32 = tx.query_row(
                "SELECT COALESCE(MAX(version_number), 0) + 1 FROM prompt_versions WHERE template_id = ?1",
                params![template_id],
                |row| row.get(0),
            )?;

            let now = chrono::Utc::now().to_rfc3339();

            // 取消当前激活版本
            tx.execute(
                "UPDATE prompt_versions SET is_active = 0 WHERE template_id = ?1 AND is_active = 1",
                params![template_id],
            )?;

            // 创建新版本
            tx.execute(
                "INSERT INTO prompt_versions (
                    template_id, version_number, is_active, content, created_by, created_at
                ) VALUES (?1, ?2, 1, ?3, ?4, ?5)",
                params![template_id, next_version, &content, created_by, &now],
            )?;

            let version_id = tx.last_insert_rowid();

            // 创建组件
            for component in &components {
                tx.execute(
                    "INSERT INTO prompt_components (
                        version_id, component_type, name, content, variables, language, sort_order
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        version_id,
                        format!("{:?}", component.component_type),
                        &component.name,
                        &component.content,
                        &component.variables,
                        &component.language,
                        component.sort_order,
                    ],
                )?;
            }

            // 创建参数
            for parameter in &parameters {
                tx.execute(
                    "INSERT INTO prompt_parameters (
                        version_id, key, value, parameter_type, description
                    ) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        version_id,
                        &parameter.key,
                        &parameter.value,
                        format!("{:?}", parameter.parameter_type),
                        &parameter.description,
                    ],
                )?;
            }

            // 提交事务（如果失败，Drop 时自动回滚）
            tx.commit()?;

            Ok(PromptVersion {
                id: Some(version_id),
                template_id,
                version_number: next_version,
                is_active: true,
                content,
                metadata: None,
                created_by: created_by.to_string(),
                created_at: now,
            })
        })
    }

    // ============================================================================
    // 组件管理 (Component Management)
    // ============================================================================

    /// 获取版本的所有组件
    pub fn list_components(&self, version_id: i64) -> Result<Vec<PromptComponent>> {
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, version_id, component_type, name, content, variables, language, sort_order
                 FROM prompt_components
                 WHERE version_id = ?1
                 ORDER BY sort_order"
            )?;

            let components = stmt.query_map(params![version_id], |row| {
                let component_type_str: String = row.get(2)?;
                // 使用 FromStr trait 进行安全解析
                let component_type: PromptComponentType = component_type_str.parse()
                    .map_err(|e| {
                        rusqlite::Error::InvalidParameterName(format!(
                            "无效的 PromptComponentType '{}': {}",
                            component_type_str, e
                        ))
                    })?;

                Ok(PromptComponent {
                    id: Some(row.get(0)?),
                    version_id: row.get(1)?,
                    component_type,
                    name: row.get(3)?,
                    content: row.get(4)?,
                    variables: row.get(5)?,
                    language: row.get(6)?,
                    sort_order: row.get(7)?,
                })
            })?;

            components.collect::<Result<Vec<_>, _>>().map_err(Into::into)
        })
    }

    // ============================================================================
    // 参数管理 (Parameter Management)
    // ============================================================================

    /// 获取版本的所有参数
    pub fn list_parameters(&self, version_id: i64) -> Result<Vec<PromptParameter>> {
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, version_id, key, value, parameter_type, description
                 FROM prompt_parameters
                 WHERE version_id = ?1"
            )?;

            let parameters = stmt.query_map(params![version_id], |row| {
                let parameter_type_str: String = row.get(4)?;
                // 使用 FromStr trait 进行安全解析
                let parameter_type: PromptParameterType = parameter_type_str.parse()
                    .map_err(|e| {
                        rusqlite::Error::InvalidParameterName(format!(
                            "无效的 PromptParameterType '{}': {}",
                            parameter_type_str, e
                        ))
                    })?;

                Ok(PromptParameter {
                    id: Some(row.get(0)?),
                    version_id: row.get(1)?,
                    key: row.get(2)?,
                    value: row.get(3)?,
                    parameter_type,
                    description: row.get(5)?,
                })
            })?;

            parameters.collect::<Result<Vec<_>, _>>().map_err(Into::into)
        })
    }

    // ============================================================================
    // 版本对比 (Version Comparison)
    // ============================================================================

    /// 对比两个版本
    pub fn compare_versions(
        &self,
        template_id: i64,
        from_version: i32,
        to_version: i32,
    ) -> Result<PromptVersionDiff> {
        // 获取版本数据
        let from_version_data = self.get_version_by_number(template_id, from_version)?
            .ok_or_else(|| anyhow::anyhow!("源版本不存在: v{}", from_version))?;

        let to_version_data = self.get_version_by_number(template_id, to_version)?
            .ok_or_else(|| anyhow::anyhow!("目标版本不存在: v{}", to_version))?;

        let from_components = if let Some(id) = from_version_data.id {
            self.list_components(id)?
        } else {
            Vec::new()
        };

        let to_components = if let Some(id) = to_version_data.id {
            self.list_components(id)?
        } else {
            Vec::new()
        };

        let from_parameters = if let Some(id) = from_version_data.id {
            self.list_parameters(id)?
        } else {
            Vec::new()
        };

        let to_parameters = if let Some(id) = to_version_data.id {
            self.list_parameters(id)?
        } else {
            Vec::new()
        };

        // 执行对比
        let component_changes = Self::compare_components_internal(&from_components, &to_components);
        let parameter_changes = Self::compare_parameters_internal(&from_parameters, &to_parameters);
        let metadata_changes = Self::compare_metadata_internal(&from_version_data.metadata, &to_version_data.metadata);

        Ok(PromptVersionDiff {
            from_version: from_version_data,
            to_version: to_version_data,
            component_changes,
            parameter_changes,
            metadata_changes,
        })
    }

    /// 内部方法：对比组件变更
    fn compare_components_internal(
        from_components: &[PromptComponent],
        to_components: &[PromptComponent],
    ) -> Vec<ComponentDiff> {
        let mut changes = Vec::new();

        let from_map: HashMap<(PromptComponentType, String), &PromptComponent> = from_components
            .iter()
            .map(|c| ((c.component_type.clone(), c.name.clone()), c))
            .collect();

        let to_map: HashMap<(PromptComponentType, String), &PromptComponent> = to_components
            .iter()
            .map(|c| ((c.component_type.clone(), c.name.clone()), c))
            .collect();

        // 检查新增和修改的组件
        for (key, to_component) in &to_map {
            if let Some(from_component) = from_map.get(key) {
                // 组件存在，检查内容变更
                let line_diffs = Self::compute_line_diff(&from_component.content, &to_component.content);

                if !line_diffs.is_empty() {
                    changes.push(ComponentDiff {
                        component_type: to_component.component_type.clone(),
                        component_name: to_component.name.clone(),
                        change_type: ChangeType::Updated,
                        line_diffs,
                    });
                }
            } else {
                // 新增组件
                changes.push(ComponentDiff {
                    component_type: to_component.component_type.clone(),
                    component_name: to_component.name.clone(),
                    change_type: ChangeType::Created,
                    line_diffs: Self::compute_line_diff("", &to_component.content),
                });
            }
        }

        // 检查删除的组件
        for (key, from_component) in &from_map {
            if !to_map.contains_key(key) {
                changes.push(ComponentDiff {
                    component_type: from_component.component_type.clone(),
                    component_name: from_component.name.clone(),
                    change_type: ChangeType::Deleted,
                    line_diffs: Self::compute_line_diff(&from_component.content, ""),
                });
            }
        }

        changes
    }

    /// 计算行级差异
    fn compute_line_diff(old_content: &str, new_content: &str) -> Vec<LineDiff> {
        let mut line_diffs = Vec::new();
        let diff = TextDiff::from_lines(old_content, new_content);

        for change in diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Equal => {
                    // 不记录未变更的行
                }
                ChangeTag::Delete => {
                    line_diffs.push(LineDiff {
                        line_number: 0, // similar 不提供行号信息，使用 0 占位
                        change_type: LineChangeType::Removed,
                        old_content: Some(change.value().to_string()),
                        new_content: None,
                    });
                }
                ChangeTag::Insert => {
                    line_diffs.push(LineDiff {
                        line_number: 0,
                        change_type: LineChangeType::Added,
                        old_content: None,
                        new_content: Some(change.value().to_string()),
                    });
                }
            }
        }

        line_diffs
    }

    /// 内部方法：对比参数变更
    fn compare_parameters_internal(
        from_parameters: &[PromptParameter],
        to_parameters: &[PromptParameter],
    ) -> Vec<ParameterDiff> {
        let mut changes = Vec::new();

        let from_map: HashMap<(String, PromptParameterType), &PromptParameter> = from_parameters
            .iter()
            .map(|p| ((p.key.clone(), p.parameter_type.clone()), p))
            .collect();

        let to_map: HashMap<(String, PromptParameterType), &PromptParameter> = to_parameters
            .iter()
            .map(|p| ((p.key.clone(), p.parameter_type.clone()), p))
            .collect();

        // 检查新增和修改的参数
        for (key, to_param) in &to_map {
            if let Some(from_param) = from_map.get(key) {
                if from_param.value != to_param.value {
                    changes.push(ParameterDiff {
                        key: to_param.key.clone(),
                        parameter_type: to_param.parameter_type.clone(),
                        old_value: Some(from_param.value.clone()),
                        new_value: Some(to_param.value.clone()),
                    });
                }
            } else {
                // 新增参数
                changes.push(ParameterDiff {
                    key: to_param.key.clone(),
                    parameter_type: to_param.parameter_type.clone(),
                    old_value: None,
                    new_value: Some(to_param.value.clone()),
                });
            }
        }

        // 检查删除的参数
        for (key, from_param) in &from_map {
            if !to_map.contains_key(key) {
                changes.push(ParameterDiff {
                    key: from_param.key.clone(),
                    parameter_type: from_param.parameter_type.clone(),
                    old_value: Some(from_param.value.clone()),
                    new_value: None,
                });
            }
        }

        changes
    }

    /// 内部方法：对比元数据变更
    fn compare_metadata_internal(
        from_metadata: &Option<String>,
        to_metadata: &Option<String>,
    ) -> Option<MetadataDiff> {
        match (from_metadata, to_metadata) {
            (None, None) => None,
            (Some(from), Some(to)) => {
                if from != to {
                    // 简化处理：返回整体变更
                    Some(MetadataDiff {
                        field_name: "metadata".to_string(),
                        old_value: Some(from.clone()),
                        new_value: Some(to.clone()),
                    })
                } else {
                    None
                }
            }
            (None, Some(to)) => Some(MetadataDiff {
                field_name: "metadata".to_string(),
                old_value: None,
                new_value: Some(to.clone()),
            }),
            (Some(from), None) => Some(MetadataDiff {
                field_name: "metadata".to_string(),
                old_value: Some(from.clone()),
                new_value: None,
            }),
        }
    }

    // ============================================================================
    // 变更历史 (Change History)
    // ============================================================================

    /// 获取版本之间的变更记录
    pub fn get_changes_between_versions(
        &self,
        template_id: i64,
        from_version: Option<i32>,
        to_version: i32,
    ) -> Result<Vec<PromptChange>> {
        self.with_conn_inner(|conn| {
            let changes = if let Some(from_ver) = from_version {
                // 有源版本的查询
                let mut stmt = conn.prepare(
                    "SELECT pc.id, pc.template_id, pc.from_version_id, pc.to_version_id,
                            pc.component_id, pc.change_type, pc.field_name,
                            pc.old_value, pc.new_value, pc.line_number, pc.change_summary, pc.changed_at
                     FROM prompt_changes pc
                     INNER JOIN prompt_versions pv1 ON pc.from_version_id = pv1.id
                     INNER JOIN prompt_versions pv2 ON pc.to_version_id = pv2.id
                     WHERE pc.template_id = ?1
                     AND pv1.version_number = ?2
                     AND pv2.version_number = ?3
                     ORDER BY pc.changed_at"
                )?;

                let rows = stmt.query_map(params![template_id, from_ver, to_version], |row| {
                    Ok(Self::row_to_change(row))
                })?;

                rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
            } else {
                // 无源版本的查询
                let mut stmt = conn.prepare(
                    "SELECT pc.id, pc.template_id, pc.from_version_id, pc.to_version_id,
                            pc.component_id, pc.change_type, pc.field_name,
                            pc.old_value, pc.new_value, pc.line_number, pc.change_summary, pc.changed_at
                     FROM prompt_changes pc
                     INNER JOIN prompt_versions pv ON pc.to_version_id = pv.id
                     WHERE pc.template_id = ?1
                     AND pc.from_version_id IS NULL
                     AND pv.version_number = ?2
                     ORDER BY pc.changed_at"
                )?;

                let rows = stmt.query_map(params![template_id, to_version], |row| {
                    Ok(Self::row_to_change(row))
                })?;

                rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
            };

            changes
        })
    }

    /// 辅助方法：从行转换为 PromptChange
    fn row_to_change(row: &rusqlite::Row) -> PromptChange {
        let change_type_str: String = row.get(5).unwrap_or_else(|_| "Updated".to_string());
        // 使用 FromStr trait 进行安全解析，默认为 Updated
        let change_type: ChangeType = change_type_str.parse().unwrap_or(ChangeType::Updated);

        PromptChange {
            id: row.get(0).unwrap_or(0),
            template_id: row.get(1).unwrap_or(0),
            from_version_id: row.get(2).ok(),
            to_version_id: row.get(3).unwrap_or(0),
            component_id: row.get(4).ok(),
            change_type,
            field_name: row.get(5).unwrap_or_else(|_| "unknown".to_string()),
            old_value: row.get(6).ok(),
            new_value: row.get(7).ok(),
            line_number: row.get(8).ok(),
            change_summary: row.get(9).ok(),
            changed_at: row.get(10).unwrap_or_else(|_| chrono::Utc::now().to_rfc3339()),
        }
    }
}
