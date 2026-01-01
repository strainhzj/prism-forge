//! API Provider 数据仓库
//!
//! 提供 api_providers 表的 CRUD 操作

use anyhow::Result;
use rusqlite::{Connection, params};
use chrono::Utc;
use std::sync::{Arc, Mutex};

use crate::database::models::{ApiProvider, ApiProviderType};

/// API Provider 数据仓库
pub struct ApiProviderRepository {
    conn: Arc<Mutex<Connection>>,
}

unsafe impl Send for ApiProviderRepository {}
unsafe impl Sync for ApiProviderRepository {}

impl ApiProviderRepository {
    /// 使用共享连接创建仓库实例
    pub fn with_conn(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// 从默认数据库路径创建仓库
    pub fn from_default_db() -> Result<Self> {
        let conn = crate::database::init::get_connection_shared()?;
        Ok(Self::with_conn(conn))
    }

    /// 辅助方法：获取连接锁
    fn with_conn_inner<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&rusqlite::Connection) -> Result<R>,
    {
        let conn = self.conn.lock().map_err(|e| {
            anyhow::anyhow!("获取数据库连接锁失败（Mutex 已被毒化）: {}", e)
        })?;
        f(&conn)
    }

    /// 创建新的 API 提供商
    ///
    /// # 参数
    /// - `provider`: 要创建的提供商对象（不需要 id）
    ///
    /// # 返回
    /// 返回创建后的提供商（包含生成的 id）
    ///
    /// # 示例
    /// ```no_run
    /// use crate::database::{ApiProvider, ApiProviderType};
    ///
    /// let provider = ApiProvider::new(
    ///     ApiProviderType::OpenAI,
    ///     "OpenAI 官方".to_string(),
    ///     Some("https://api.openai.com/v1".to_string()),
    /// );
    /// let repo = ApiProviderRepository::from_default_db()?;
    /// let created = repo.create_provider(provider)?;
    /// ```
    pub fn create_provider(&self, mut provider: ApiProvider) -> Result<ApiProvider> {
        let now = Utc::now().to_rfc3339();
        let provider_type_str = serde_json::to_string(&provider.provider_type)?;

        self.with_conn_inner(|conn| {
            conn.execute(
                "INSERT INTO api_providers (
                    provider_type, name, base_url, api_key_ref,
                    model, config_json, temperature, max_tokens, is_active, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    provider_type_str,
                    provider.name,
                    provider.base_url,
                    provider.api_key_ref,
                    provider.model,
                    provider.config_json,
                    provider.temperature,
                    provider.max_tokens,
                    if provider.is_active { 1 } else { 0 },
                    now,
                    now,
                ],
            )?;
            Ok(())
        })?;

        let id = self.with_conn_inner(|conn| {
            Ok(conn.last_insert_rowid())
        })?;

        provider.id = Some(id);
        Ok(provider)
    }

    /// 获取所有提供商
    ///
    /// # 返回
    /// 返回所有提供商的列表，按创建时间倒序排列
    pub fn get_all_providers(&self) -> Result<Vec<ApiProvider>> {
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, provider_type, name, base_url, api_key_ref,
                        model, config_json, temperature, max_tokens, is_active, created_at, updated_at
                 FROM api_providers
                 ORDER BY created_at DESC"
            )?;

            let providers = stmt.query_map([], |row| {
                let provider_type_str: String = row.get(1)?;
                let provider_type: ApiProviderType = serde_json::from_str(&provider_type_str)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                Ok(ApiProvider {
                    id: Some(row.get(0)?),
                    provider_type,
                    name: row.get(2)?,
                    base_url: row.get(3)?,
                    api_key_ref: row.get(4)?,
                    model: row.get(5)?,
                    config_json: row.get(6)?,
                    temperature: row.get(7)?,
                    max_tokens: row.get(8)?,
                    is_active: row.get::<_, i32>(9)? == 1,
                })
            })?;

            providers.collect::<Result<Vec<_>, _>>().map_err(Into::into)
        })
    }

    /// 根据ID获取提供商
    ///
    /// # 参数
    /// - `id`: 提供商 ID
    pub fn get_provider_by_id(&self, id: i64) -> Result<Option<ApiProvider>> {
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, provider_type, name, base_url, api_key_ref,
                        model, config_json, temperature, max_tokens, is_active, created_at, updated_at
                 FROM api_providers
                 WHERE id = ?1"
            )?;

            let mut rows = stmt.query(params![id])?;

            if let Some(row) = rows.next()? {
                let provider_type_str: String = row.get(1)?;
                let provider_type: ApiProviderType = serde_json::from_str(&provider_type_str)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                Ok(Some(ApiProvider {
                    id: Some(row.get(0)?),
                    provider_type,
                    name: row.get(2)?,
                    base_url: row.get(3)?,
                    api_key_ref: row.get(4)?,
                    model: row.get(5)?,
                    config_json: row.get(6)?,
                    temperature: row.get(7)?,
                    max_tokens: row.get(8)?,
                    is_active: row.get::<_, i32>(9)? == 1,
                }))
            } else {
                Ok(None)
            }
        })
    }

    /// 获取当前活跃的提供商
    ///
    /// # 返回
    /// 返回活跃的提供商，如果没有则返回 None
    pub fn get_active_provider(&self) -> Result<Option<ApiProvider>> {
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, provider_type, name, base_url, api_key_ref,
                        model, config_json, temperature, max_tokens, is_active, created_at, updated_at
                 FROM api_providers
                 WHERE is_active = 1
                 LIMIT 1"
            )?;

            let mut rows = stmt.query([])?;

            if let Some(row) = rows.next()? {
                let provider_type_str: String = row.get(1)?;
                let provider_type: ApiProviderType = serde_json::from_str(&provider_type_str)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                Ok(Some(ApiProvider {
                    id: Some(row.get(0)?),
                    provider_type,
                    name: row.get(2)?,
                    base_url: row.get(3)?,
                    api_key_ref: row.get(4)?,
                    model: row.get(5)?,
                    config_json: row.get(6)?,
                    temperature: row.get(7)?,
                    max_tokens: row.get(8)?,
                    is_active: row.get::<_, i32>(9)? == 1,
                }))
            } else {
                Ok(None)
            }
        })
    }

    /// 更新提供商
    ///
    /// # 参数
    /// - `provider`: 要更新的提供商对象（必须包含 id）
    ///
    /// # 返回
    /// 返回更新后的行数，如果为 0 表示没有找到对应的提供商
    pub fn update_provider(&self, provider: &ApiProvider) -> Result<usize> {
        let id = provider.id.ok_or_else(|| anyhow::anyhow!("提供商必须有 id"))?;
        let now = Utc::now().to_rfc3339();
        let provider_type_str = serde_json::to_string(&provider.provider_type)?;

        self.with_conn_inner(|conn| {
            let rows = conn.execute(
                "UPDATE api_providers
                 SET provider_type = ?1, name = ?2, base_url = ?3,
                     api_key_ref = ?4, model = ?5, config_json = ?6,
                     temperature = ?7, max_tokens = ?8, is_active = ?9,
                     updated_at = ?10
                 WHERE id = ?11",
                params![
                    provider_type_str,
                    provider.name,
                    provider.base_url,
                    provider.api_key_ref,
                    provider.model,
                    provider.config_json,
                    provider.temperature,
                    provider.max_tokens,
                    if provider.is_active { 1 } else { 0 },
                    now,
                    id,
                ],
            )?;

            Ok(rows)
        })
    }

    /// 删除提供商
    ///
    /// # 参数
    /// - `id`: 要删除的提供商 ID
    ///
    /// # 返回
    /// 返回删除的行数，如果为 0 表示没有找到对应的提供商
    pub fn delete_provider(&self, id: i64) -> Result<usize> {
        self.with_conn_inner(|conn| {
            let rows = conn.execute(
                "DELETE FROM api_providers WHERE id = ?1",
                params![id],
            )?;

            Ok(rows)
        })
    }

    /// 设置活跃提供商
    ///
    /// # 参数
    /// - `id`: 要设置为活跃的提供商 ID
    ///
    /// # 说明
    /// 此方法会自动将其他提供商的 is_active 设置为 0
    /// （通过数据库触发器实现）
    pub fn set_active_provider(&self, id: i64) -> Result<usize> {
        let now = Utc::now().to_rfc3339();

        self.with_conn_inner(|conn| {
            let rows = conn.execute(
                "UPDATE api_providers
                 SET is_active = 1, updated_at = ?1
                 WHERE id = ?2",
                params![now, id],
            )?;

            Ok(rows)
        })
    }

    /// 根据 provider_type 获取提供商列表
    ///
    /// # 参数
    /// - `provider_type`: 提供商类型
    pub fn get_providers_by_type(&self, provider_type: ApiProviderType) -> Result<Vec<ApiProvider>> {
        let provider_type_str = serde_json::to_string(&provider_type)?;

        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, provider_type, name, base_url, api_key_ref,
                        model, config_json, temperature, max_tokens, is_active, created_at, updated_at
                 FROM api_providers
                 WHERE provider_type = ?1
                 ORDER BY created_at DESC"
            )?;

            let providers = stmt.query_map(params![provider_type_str], |row| {
                let provider_type_str: String = row.get(1)?;
                let provider_type: ApiProviderType = serde_json::from_str(&provider_type_str)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                Ok(ApiProvider {
                    id: Some(row.get(0)?),
                    provider_type,
                    name: row.get(2)?,
                    base_url: row.get(3)?,
                    api_key_ref: row.get(4)?,
                    model: row.get(5)?,
                    config_json: row.get(6)?,
                    temperature: row.get(7)?,
                    max_tokens: row.get(8)?,
                    is_active: row.get::<_, i32>(9)? == 1,
                })
            })?;

            providers.collect::<Result<Vec<_>, _>>().map_err(Into::into)
        })
    }

    /// 统计提供商数量
    pub fn count_providers(&self) -> Result<i64> {
        self.with_conn_inner(|conn| {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM api_providers",
                [],
                |row| row.get(0),
            )?;
            Ok(count)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::migrations;
    use std::sync::{Arc, Mutex};

    fn setup_test_db() -> Arc<Mutex<Connection>> {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();
        // 执行迁移
        migrations::migrate_v1(&mut conn).unwrap();
        migrations::migrate_v2(&mut conn).unwrap();
        migrations::migrate_v3(&mut conn).unwrap();
        Arc::new(Mutex::new(conn))
    }

    #[test]
    fn test_create_and_get_provider() {
        let conn = setup_test_db();
        let repo = ApiProviderRepository::with_conn(conn);

        let provider = ApiProvider::new(
            ApiProviderType::Ollama,
            "测试提供商".to_string(),
            Some("http://localhost:11434".to_string()),
        );

        let created = repo.create_provider(provider).unwrap();
        assert!(created.id.is_some());

        let retrieved = repo.get_provider_by_id(created.id.unwrap()).unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.name, "测试提供商");
        assert_eq!(retrieved.base_url, "http://localhost:11434");
    }

    #[test]
    fn test_get_all_providers() {
        let conn = setup_test_db();
        let repo = ApiProviderRepository::with_conn(conn);

        // 创建多个提供商
        for i in 1..=3 {
            let provider = ApiProvider::new(
                ApiProviderType::Ollama,
                format!("提供商 {}", i),
                None,
            );
            repo.create_provider(provider).unwrap();
        }

        let all = repo.get_all_providers().unwrap();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_update_provider() {
        let conn = setup_test_db();
        let repo = ApiProviderRepository::with_conn(conn);

        let mut provider = ApiProvider::new(
            ApiProviderType::Ollama,
            "原始名称".to_string(),
            None,
        );
        provider = repo.create_provider(provider).unwrap();

        provider.name = "更新后的名称".to_string();
        let rows = repo.update_provider(&provider).unwrap();
        assert_eq!(rows, 1);

        let updated = repo.get_provider_by_id(provider.id.unwrap()).unwrap().unwrap();
        assert_eq!(updated.name, "更新后的名称");
    }

    #[test]
    fn test_delete_provider() {
        let conn = setup_test_db();
        let repo = ApiProviderRepository::with_conn(conn);

        let provider = ApiProvider::new(
            ApiProviderType::Ollama,
            "待删除".to_string(),
            None,
        );
        let created = repo.create_provider(provider).unwrap();

        let rows = repo.delete_provider(created.id.unwrap()).unwrap();
        assert_eq!(rows, 1);

        let retrieved = repo.get_provider_by_id(created.id.unwrap()).unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_active_provider() {
        let conn = setup_test_db();
        let repo = ApiProviderRepository::with_conn(conn);

        // 创建第一个提供商
        let mut provider1 = ApiProvider::new(
            ApiProviderType::Ollama,
            "提供商1".to_string(),
            None,
        );
        provider1.is_active = true;
        let created1 = repo.create_provider(provider1).unwrap();

        // 验证它是活跃的
        let active = repo.get_active_provider().unwrap();
        assert!(active.is_some());
        assert_eq!(active.as_ref().unwrap().id, created1.id);

        // 创建第二个活跃提供商
        let mut provider2 = ApiProvider::new(
            ApiProviderType::OpenAI,
            "提供商2".to_string(),
            Some("https://api.openai.com/v1".to_string()),
        );
        provider2.api_key_ref = Some("test_key_ref".to_string());
        provider2.is_active = true;
        repo.create_provider(provider2).unwrap();

        // 验证只有第二个是活跃的
        let active = repo.get_active_provider().unwrap().unwrap();
        assert_eq!(active.name, "提供商2");

        // 验证第一个不再是活跃的
        let provider1_updated = repo.get_provider_by_id(created1.id.unwrap()).unwrap().unwrap();
        assert!(!provider1_updated.is_active);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::database::migrations;
    use proptest::prelude::*;
    use std::sync::{Arc, Mutex};

    fn setup_test_db() -> Arc<Mutex<Connection>> {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();
        migrations::migrate_v1(&mut conn).unwrap();
        migrations::migrate_v2(&mut conn).unwrap();
        migrations::migrate_v3(&mut conn).unwrap();
        Arc::new(Mutex::new(conn))
    }

    fn arb_provider_type() -> impl Strategy<Value = ApiProviderType> {
        prop_oneof![
            Just(ApiProviderType::OpenAI),
            Just(ApiProviderType::Anthropic),
            Just(ApiProviderType::Ollama),
            Just(ApiProviderType::XAI),
        ]
    }

    fn arb_model() -> impl Strategy<Value = Option<String>> {
        prop_oneof![
            Just(None),
            "[a-zA-Z0-9_-]{1,50}".prop_map(Some),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Feature: provider-model-config, Property 3: Model Persistence Round Trip
        /// *For any* ApiProvider with a configured model, saving to database and
        /// retrieving SHALL preserve the exact model value.
        /// **Validates: Requirements 2.3**
        #[test]
        fn test_model_persistence_round_trip(
            provider_type in arb_provider_type(),
            model in arb_model(),
        ) {
            let conn = setup_test_db();
            let repo = ApiProviderRepository::with_conn(conn);

            let mut provider = ApiProvider::new(
                provider_type,
                "Test Provider".to_string(),
                None,
            );
            provider.model = model.clone();

            // Save to database
            let created = repo.create_provider(provider).unwrap();
            let id = created.id.unwrap();

            // Retrieve from database
            let retrieved = repo.get_provider_by_id(id).unwrap().unwrap();

            // Verify model is preserved
            prop_assert_eq!(retrieved.model, model);
        }
    }
}

/// 获取活跃会话判断阈值
///
/// # 返回
/// 返回 active_threshold 配置值（秒），默认 86400（24小时）
pub fn get_active_threshold(conn: &Connection) -> Result<u64> {
    conn.query_row(
        "SELECT active_threshold FROM settings WHERE id = 1",
        [],
        |row| row.get(0),
    )
    .map_err(|e| anyhow::anyhow!("获取活跃阈值失败: {}", e))
}

/// 更新活跃会话判断阈值
///
/// # 参数
/// - `conn`: 数据库连接
/// - `value`: 新的阈值（秒）
pub fn update_active_threshold(conn: &Connection, value: u64) -> Result<()> {
    conn.execute(
        "UPDATE settings SET active_threshold = ?1 WHERE id = 1",
        params![value],
    )
    .map_err(|e| anyhow::anyhow!("更新活跃阈值失败: {}", e))?;
    Ok(())
}

/// Session 数据仓库
///
/// 提供 sessions 表的 CRUD 操作
pub struct SessionRepository {
    conn: Arc<Mutex<Connection>>,
}

unsafe impl Send for SessionRepository {}
unsafe impl Sync for SessionRepository {}

impl SessionRepository {
    /// 使用共享连接创建仓库实例
    pub fn with_conn(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// 从默认数据库路径创建仓库
    pub fn from_default_db() -> Result<Self> {
        let conn = crate::database::init::get_connection_shared()?;
        Ok(Self::with_conn(conn))
    }

    /// 辅助方法：获取连接锁
    fn with_conn_inner<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&rusqlite::Connection) -> Result<R>,
    {
        let conn = self.conn.lock().map_err(|e| {
            anyhow::anyhow!("获取数据库连接锁失败（Mutex 已被毒化）: {}", e)
        })?;
        f(&conn)
    }

    /// 插入或更新会话
    ///
    /// # 参数
    /// - `session_id`: 会话唯一标识
    /// - `project_path`: 项目路径
    /// - `project_name`: 项目名称
    /// - `file_path`: 文件路径
    /// - `is_active`: 是否活跃
    ///
    /// # 返回
    /// 返回插入/更新的行数
    pub fn upsert_session(
        &self,
        session_id: &str,
        project_path: &str,
        project_name: &str,
        file_path: &str,
        is_active: bool,
    ) -> Result<usize> {
        let now = Utc::now().to_rfc3339();

        self.with_conn_inner(|conn| {
            // 尝试更新
            let updated = conn.execute(
                "UPDATE sessions SET
                    project_path = ?1,
                    project_name = ?2,
                    file_path = ?3,
                    is_active = ?4,
                    updated_at = ?5
                 WHERE session_id = ?6",
                params![
                    project_path,
                    project_name,
                    file_path,
                    if is_active { 1 } else { 0 },
                    now,
                    session_id,
                ],
            )?;

            if updated > 0 {
                return Ok(updated);
            }

            // 不存在则插入
            let inserted = conn.execute(
                "INSERT INTO sessions (
                    session_id, project_path, project_name, file_path,
                    is_active, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    session_id,
                    project_path,
                    project_name,
                    file_path,
                    if is_active { 1 } else { 0 },
                    now,
                    now,
                ],
            )?;

            Ok(inserted)
        })
    }

    /// 获取所有会话
    pub fn get_all_sessions(&self) -> Result<Vec<crate::database::models::Session>> {
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, session_id, project_path, project_name, file_path,
                        rating, tags, is_archived, is_active, created_at, updated_at
                 FROM sessions
                 ORDER BY updated_at DESC"
            )?;

            let sessions = stmt.query_map([], |row| {
                Ok(crate::database::models::Session {
                    id: Some(row.get(0)?),
                    session_id: row.get(1)?,
                    project_path: row.get(2)?,
                    project_name: row.get(3)?,
                    file_path: row.get(4)?,
                    rating: row.get(5)?,
                    tags: row.get(6)?,
                    is_archived: row.get::<_, i32>(7)? == 1,
                    is_active: row.get::<_, i32>(8)? == 1,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            })?;

            sessions.collect::<Result<Vec<_>, _>>().map_err(Into::into)
        })
    }

    /// 设置会话评分
    ///
    /// # 参数
    /// - `session_id`: 会话唯一标识
    /// - `rating`: 评分值 (1-5)，None 表示清除评分
    ///
    /// # 返回
    /// 返回更新的行数
    pub fn set_session_rating(&self, session_id: &str, rating: Option<i32>) -> Result<usize> {
        // 验证评分范围
        if let Some(r) = rating {
            if r < 1 || r > 5 {
                return Err(anyhow::anyhow!("评分必须在 1-5 之间，当前值: {}", r));
            }
        }

        let now = Utc::now().to_rfc3339();

        self.with_conn_inner(|conn| {
            conn.execute(
                "UPDATE sessions
                 SET rating = ?1, updated_at = ?2
                 WHERE session_id = ?3",
                params![rating, now, session_id],
            )
            .map_err(|e| anyhow::anyhow!("更新会话评分失败: {}", e))
        })
    }

    /// 设置会话标签
    ///
    /// # 参数
    /// - `session_id`: 会话唯一标识
    /// - `tags`: 标签数组，空数组表示清除所有标签
    ///
    /// # 返回
    /// 返回更新的行数
    pub fn set_session_tags(&self, session_id: &str, tags: Vec<String>) -> Result<usize> {
        // 将标签数组序列化为 JSON 字符串
        let tags_json = serde_json::to_string(&tags)
            .unwrap_or_else(|_| "[]".to_string());

        let now = Utc::now().to_rfc3339();

        self.with_conn_inner(|conn| {
            conn.execute(
                "UPDATE sessions
                 SET tags = ?1, updated_at = ?2
                 WHERE session_id = ?3",
                params![tags_json, now, session_id],
            )
            .map_err(|e| anyhow::anyhow!("更新会话标签失败: {}", e))
        })
    }

    /// 获取会话评分
    ///
    /// # 参数
    /// - `session_id`: 会话唯一标识
    ///
    /// # 返回
    /// 返回评分值 (1-5)，None 表示未评分
    pub fn get_session_rating(&self, session_id: &str) -> Result<Option<i32>> {
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT rating FROM sessions WHERE session_id = ?1"
            )?;

            let rating = stmt.query_row(params![session_id], |row| {
                row.get(0)
            })?;

            Ok(rating)
        })
    }

    /// 获取会话标签
    ///
    /// # 参数
    /// - `session_id`: 会话唯一标识
    ///
    /// # 返回
    /// 返回标签数组
    pub fn get_session_tags(&self, session_id: &str) -> Result<Vec<String>> {
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT tags FROM sessions WHERE session_id = ?1"
            )?;

            let tags_json: String = stmt.query_row(params![session_id], |row| {
                row.get(0)
            })?;

            // 解析 JSON 数组
            if tags_json.is_empty() || tags_json == "[]" {
                return Ok(Vec::new());
            }

            serde_json::from_str(&tags_json)
                .map_err(|e| anyhow::anyhow!("解析标签失败: {}", e))
        })
    }

    /// 归档会话
    ///
    /// 将会话标记为已归档，归档后的会话不会在默认列表中显示
    ///
    /// # 参数
    /// - `session_id`: 会话唯一标识
    ///
    /// # 返回
    /// 返回更新的行数
    pub fn archive_session(&self, session_id: &str) -> Result<usize> {
        let now = Utc::now().to_rfc3339();

        self.with_conn_inner(|conn| {
            conn.execute(
                "UPDATE sessions
                 SET is_archived = 1, updated_at = ?1
                 WHERE session_id = ?2",
                params![now, session_id],
            )
            .map_err(|e| anyhow::anyhow!("归档会话失败: {}", e))
        })
    }

    /// 取消归档会话
    ///
    /// 将会话标记为未归档，恢复到默认列表
    ///
    /// # 参数
    /// - `session_id`: 会话唯一标识
    ///
    /// # 返回
    /// 返回更新的行数
    pub fn unarchive_session(&self, session_id: &str) -> Result<usize> {
        let now = Utc::now().to_rfc3339();

        self.with_conn_inner(|conn| {
            conn.execute(
                "UPDATE sessions
                 SET is_archived = 0, updated_at = ?1
                 WHERE session_id = ?2",
                params![now, session_id],
            )
            .map_err(|e| anyhow::anyhow!("取消归档会话失败: {}", e))
        })
    }

    /// 获取已归档的会话列表
    ///
    /// # 返回
    /// 返回所有已归档的会话，按更新时间倒序排列
    pub fn get_archived_sessions(&self) -> Result<Vec<crate::database::models::Session>> {
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, session_id, project_path, project_name, file_path,
                        rating, tags, is_archived, is_active, created_at, updated_at
                 FROM sessions
                 WHERE is_archived = 1
                 ORDER BY updated_at DESC"
            )?;

            let sessions = stmt.query_map([], |row| {
                Ok(crate::database::models::Session {
                    id: Some(row.get(0)?),
                    session_id: row.get(1)?,
                    project_path: row.get(2)?,
                    project_name: row.get(3)?,
                    file_path: row.get(4)?,
                    rating: row.get(5)?,
                    tags: row.get(6)?,
                    is_archived: row.get::<_, i32>(7)? == 1,
                    is_active: row.get::<_, i32>(8)? == 1,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            })?;

            sessions.collect::<Result<Vec<_>, _>>().map_err(Into::into)
        })
    }

    /// 获取未归档的活跃会话列表
    ///
    /// # 返回
    /// 返回所有未归档的会话，按更新时间倒序排列
    pub fn get_active_sessions(&self) -> Result<Vec<crate::database::models::Session>> {
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, session_id, project_path, project_name, file_path,
                        rating, tags, is_archived, is_active, created_at, updated_at
                 FROM sessions
                 WHERE is_archived = 0
                 ORDER BY updated_at DESC"
            )?;

            let sessions = stmt.query_map([], |row| {
                Ok(crate::database::models::Session {
                    id: Some(row.get(0)?),
                    session_id: row.get(1)?,
                    project_path: row.get(2)?,
                    project_name: row.get(3)?,
                    file_path: row.get(4)?,
                    rating: row.get(5)?,
                    tags: row.get(6)?,
                    is_archived: row.get::<_, i32>(7)? == 1,
                    is_active: row.get::<_, i32>(8)? == 1,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            })?;

            sessions.collect::<Result<Vec<_>, _>>().map_err(Into::into)
        })
    }

    /// 保存消息向量嵌入
    ///
    /// # 参数
    /// - `message_id`: 消息 ID
    /// - `embedding`: 384 维向量
    /// - `summary`: 消息摘要文本
    ///
    /// # 返回
    /// 返回插入结果或错误
    ///
    /// # 说明
    /// 此方法会将向量插入到 message_embeddings 虚拟表，并更新关联映射
    pub fn save_message_embedding(
        &self,
        message_id: i64,
        embedding: &[f32],
        summary: &str,
    ) -> Result<()> {
        // 验证向量维度
        if embedding.len() != 384 {
            return Err(anyhow::anyhow!(
                "向量维度错误，期望 384，实际 {}",
                embedding.len()
            ));
        }

        let now = Utc::now().to_rfc3339();

        self.with_conn_inner(|conn| {
            // 将向量转换为 JSON 数组字符串
            let embedding_json = serde_json::to_string(embedding)
                .map_err(|e| anyhow::anyhow!("序列化向量失败: {}", e))?;

            // 检查是否已存在该消息的向量
            let existing: Option<i64> = conn.query_row(
                "SELECT vec_row_id FROM message_embedding_map WHERE message_id = ?1",
                params![message_id],
                |row| row.get(0),
            ).ok();

            if let Some(vec_row_id) = existing {
                // 更新现有向量
                conn.execute(
                    "UPDATE message_embeddings
                     SET embedding = ?1, summary = ?2
                     WHERE rowid = ?3",
                    params![embedding_json, summary, vec_row_id],
                )?;
            } else {
                // 插入新向量到 vec0 虚拟表
                conn.execute(
                    "INSERT INTO message_embeddings (embedding, summary)
                     VALUES (?1, ?2)",
                    params![embedding_json, summary],
                )?;

                // 获取新插入的行 ID
                let vec_row_id = conn.last_insert_rowid();

                // 创建关联映射
                conn.execute(
                    "INSERT INTO message_embedding_map (message_id, vec_row_id, created_at)
                     VALUES (?1, ?2, ?3)",
                    params![message_id, vec_row_id, now],
                )?;
            }

            Ok(())
        })
    }

    /// 批量保存消息向量嵌入
    ///
    /// # 参数
    /// - `embeddings`: 消息 ID、向量、摘要的三元组列表
    ///
    /// # 返回
    /// 返回插入成功的数量或错误
    ///
    /// # 性能
    /// 批量操作比逐条插入更高效
    pub fn save_message_embeddings_batch(
        &self,
        embeddings: &[(i64, Vec<f32>, String)],
    ) -> Result<usize> {
        if embeddings.is_empty() {
            return Ok(0);
        }

        let now = Utc::now().to_rfc3339();

        self.with_conn_inner(|conn| {
            // 开始事务
            let tx = conn.unchecked_transaction()?;

            let mut count = 0;

            for (message_id, embedding, summary) in embeddings {
                // 验证向量维度
                if embedding.len() != 384 {
                    eprintln!("警告: 消息 {} 的向量维度错误，跳过", message_id);
                    continue;
                }

                let embedding_json = serde_json::to_string(embedding)
                    .map_err(|e| anyhow::anyhow!("序列化向量失败: {}", e))?;

                // 检查是否已存在
                let existing: Option<i64> = tx.query_row(
                    "SELECT vec_row_id FROM message_embedding_map WHERE message_id = ?1",
                    params![message_id],
                    |row| row.get(0),
                ).ok();

                if let Some(vec_row_id) = existing {
                    tx.execute(
                        "UPDATE message_embeddings
                         SET embedding = ?1, summary = ?2
                         WHERE rowid = ?3",
                        params![embedding_json, summary, vec_row_id],
                    )?;
                } else {
                    tx.execute(
                        "INSERT INTO message_embeddings (embedding, summary)
                         VALUES (?1, ?2)",
                        params![embedding_json, summary],
                    )?;

                    let vec_row_id = tx.last_insert_rowid();

                    tx.execute(
                        "INSERT INTO message_embedding_map (message_id, vec_row_id, created_at)
                         VALUES (?1, ?2, ?3)",
                        params![message_id, vec_row_id, now],
                    )?;
                }

                count += 1;
            }

            // 提交事务
            tx.commit()?;

            Ok(count)
        })
    }

    /// 获取消息的向量嵌入
    ///
    /// # 参数
    /// - `message_id`: 消息 ID
    ///
    /// # 返回
    /// 返回向量和摘要，如果不存在则返回 None
    pub fn get_message_embedding(
        &self,
        message_id: i64,
    ) -> Result<Option<(Vec<f32>, String)>> {
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT me.embedding, me.summary
                 FROM message_embeddings me
                 INNER JOIN message_embedding_map mem ON me.rowid = mem.vec_row_id
                 WHERE mem.message_id = ?1"
            )?;

            let result = stmt.query_row(params![message_id], |row| {
                let embedding_json: String = row.get(0)?;
                let summary: String = row.get(1)?;

                let embedding: Vec<f32> = serde_json::from_str(&embedding_json)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                Ok((embedding, summary))
            });

            match result {
                Ok(data) => Ok(Some(data)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e.into()),
            }
        })
    }

    /// 向量相似度检索
    ///
    /// # 参数
    /// - `query_embedding`: 查询向量（384 维）
    /// - `limit`: 返回结果数量上限
    ///
    /// # 返回
    /// 返回按相似度排序的消息 ID 和相似度分数列表
    ///
    /// # 说明
    /// 使用 sqlite-vec 的 distance 函数计算余弦相似度
    /// 分数越小表示越相似（距离）
    pub fn vector_search(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<(i64, f64)>> {
        if query_embedding.len() != 384 {
            return Err(anyhow::anyhow!(
                "查询向量维度错误，期望 384，实际 {}",
                query_embedding.len()
            ));
        }

        let embedding_json = serde_json::to_string(query_embedding)
            .map_err(|e| anyhow::anyhow!("序列化查询向量失败: {}", e))?;

        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT mem.message_id, distance(me.embedding, ?1) as dist
                 FROM message_embeddings me
                 INNER JOIN message_embedding_map mem ON me.rowid = mem.vec_row_id
                 ORDER BY dist
                 LIMIT ?2"
            )?;

            let results = stmt.query_map(params![embedding_json, limit], |row| {
                let message_id: i64 = row.get(0)?;
                let distance: f64 = row.get(1)?;
                Ok((message_id, distance))
            })?;

            results.collect::<Result<Vec<_>, _>>().map_err(Into::into)
        })
    }

    /// 向量相似度检索（返回会话详情）
    ///
    /// # 参数
    /// - `query_embedding`: 查询向量（384 维）
    /// - `limit`: 返回结果数量上限
    ///
    /// # 返回
    /// 返回按相似度排序的会话搜索结果列表
    ///
    /// # 说明
    /// 使用 sqlite-vec 的 distance 函数计算余弦相似度
    /// 自动合并同一会话的多条匹配消息，取最相似的一条
    pub fn vector_search_sessions(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<crate::database::models::VectorSearchResult>> {
        // 执行向量检索获取 (message_id, distance) 列表
        let message_results = self.vector_search(query_embedding, limit * 2)?;

        let mut session_results: std::collections::HashMap<String, (f64, String)> = std::collections::HashMap::new();

        // 获取每条消息对应的会话信息
        for (message_id, distance) in message_results {
            self.with_conn_inner(|conn| {
                let mut stmt = conn.prepare(
                    "SELECT m.session_id, m.summary
                     FROM messages m
                     WHERE m.id = ?1"
                )?;

                let result = stmt.query_row(params![message_id], |row| {
                    let session_id: String = row.get(0)?;
                    let summary: Option<String> = row.get(1)?;
                    Ok((session_id, summary.unwrap_or_default()))
                });

                if let Ok((session_id, summary)) = result {
                    session_results
                        .entry(session_id)
                        .and_modify(|(existing_dist, _)| {
                            if distance < *existing_dist {
                                *existing_dist = distance;
                            }
                        })
                        .or_insert((distance, summary));
                }

                Ok::<(), anyhow::Error>(())
            })?;
        }

        // 构建结果列表
        let mut results = Vec::new();
        for (session_id, (distance, summary)) in session_results {
            if let Some(session) = self.get_session_by_id(&session_id)? {
                results.push(crate::database::models::VectorSearchResult {
                    session,
                    similarity_score: distance,
                    summary,
                });
            }
        }

        results.sort_by(|a, b| a.similarity_score.partial_cmp(&b.similarity_score).unwrap());
        results.truncate(limit);

        Ok(results)
    }

    /// 根据 session_id 获取会话详情
    pub fn get_session_by_id(&self, session_id: &str) -> Result<Option<crate::database::models::Session>> {
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, session_id, project_path, project_name, file_path,
                        rating, tags, is_archived, is_active, created_at, updated_at
                 FROM sessions
                 WHERE session_id = ?1"
            )?;

            let session = stmt.query_row(params![session_id], |row| {
                Ok(crate::database::models::Session {
                    id: Some(row.get(0)?),
                    session_id: row.get(1)?,
                    project_path: row.get(2)?,
                    project_name: row.get(3)?,
                    file_path: row.get(4)?,
                    rating: row.get(5)?,
                    tags: row.get(6)?,
                    is_archived: row.get::<_, i32>(7)? == 1,
                    is_active: row.get::<_, i32>(8)? == 1,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            });

            match session {
                Ok(s) => Ok(Some(s)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e.into()),
            }
        })
    }

    /// 评分加权向量相似度检索
    ///
    /// # 参数
    /// - `query_embedding`: 查询向量（384 维）
    /// - `limit`: 返回结果数量上限
    ///
    /// # 返回
    /// 返回按加权分数排序的会话搜索结果列表
    ///
    /// # 说明
    /// 结合相似度和用户评分的混合排序：
    /// - 加权公式：weighted_score = 0.7 * cosine_similarity + 0.3 * (rating / 5.0)
    /// - cosine_similarity = 1.0 - distance
    /// - 未评分会话使用默认 2.5 分
    /// - 自动排除低分会话（rating < 2）和归档会话
    /// - 自动合并同一会话的多条匹配消息，取加权分数最高的一条
    ///
    /// # 加权效果
    /// 5 星会话在相似度稍低时仍能排在前面，提升优质内容的检索优先级
    pub fn weighted_vector_search_sessions(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<crate::database::models::VectorSearchResult>> {
        if query_embedding.len() != 384 {
            return Err(anyhow::anyhow!(
                "查询向量维度错误，期望 384，实际 {}",
                query_embedding.len()
            ));
        }

        let embedding_json = serde_json::to_string(query_embedding)
            .map_err(|e| anyhow::anyhow!("序列化查询向量失败: {}", e))?;

        // 使用单条 SQL 完成加权计算和过滤
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT
                    s.session_id,
                    s.rating,
                    m.summary,
                    distance(me.embedding, ?1) AS vec_dist,
                    ((1.0 - distance(me.embedding, ?1)) * 0.7 +
                     (COALESCE(s.rating, 2.5) / 5.0 * 0.3)) AS weighted_score
                 FROM message_embeddings me
                 INNER JOIN message_embedding_map mem ON me.rowid = mem.vec_row_id
                 INNER JOIN messages m ON m.id = mem.message_id
                 INNER JOIN sessions s ON m.session_id = s.session_id
                 WHERE s.is_archived = 0  -- 排除归档会话
                   AND (s.rating IS NULL OR s.rating >= 2)  -- 排除低分会话
                 ORDER BY weighted_score DESC
                 LIMIT ?2"
            )?;

            let results = stmt.query_map(params![embedding_json, limit * 2], |row| {
                let session_id: String = row.get(0)?;
                let rating: Option<i32> = row.get(1)?;
                let summary: Option<String> = row.get(2)?;
                let distance: f64 = row.get(3)?;
                let weighted_score: f64 = row.get(4)?;

                Ok((
                    session_id,
                    rating.unwrap_or(2),  // 用于后续计算
                    summary.unwrap_or_default(),
                    distance,
                    weighted_score,
                ))
            })?;

            // 收集并去重（同一会话取最高加权分数）
            let mut session_map: std::collections::HashMap<String, (f64, f64, String)> = std::collections::HashMap::new();

            for result in results {
                let (session_id, _rating, summary, distance, weighted_score) = result?;
                session_map
                    .entry(session_id)
                    .and_modify(|(existing_score, existing_dist, _)| {
                        if weighted_score > *existing_score {
                            *existing_score = weighted_score;
                            *existing_dist = distance;
                        }
                    })
                    .or_insert((weighted_score, distance, summary));
            }

            // 构建结果列表
            let mut final_results = Vec::new();
            for (session_id, (_weighted_score, distance, summary)) in session_map {
                if let Some(session) = self.get_session_by_id(&session_id)? {
                    final_results.push(crate::database::models::VectorSearchResult {
                        session,
                        similarity_score: distance,
                        summary,
                    });
                }
            }

            // 按加权分数排序并限制数量
            final_results.sort_by(|a, b| {
                let score_a = calculate_weighted_score(
                    1.0 - a.similarity_score,
                    a.session.rating.unwrap_or(2),
                );
                let score_b = calculate_weighted_score(
                    1.0 - b.similarity_score,
                    b.session.rating.unwrap_or(2),
                );
                score_b.partial_cmp(&score_a).unwrap()
            });
            final_results.truncate(limit);

            Ok(final_results)
        })
    }
// ==================== Meta-Prompt 管理方法 ====================

    /// 获取 Meta-Prompt 模板
    ///
    /// 根据类别（key）获取元提示词模板
    pub fn get_meta_template(&self, category: &str) -> Result<String> {
        use rusqlite::params;

        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT content FROM meta_templates WHERE key = ?1 AND is_active = 1"
            )?;

            stmt.query_row(params![category], |row| row.get(0))
                .map_err(|_| anyhow::anyhow!(
                    "未找到类别为 \"{}\" 的 Meta-Prompt 模板", category
                ))
        })
    }

    /// 更新 Meta-Prompt 模板
    ///
    /// 根据类别（key）更新元提示词模板内容
    pub fn update_meta_template(&self, category: &str, content: &str) -> Result<()> {
        use rusqlite::params;

        self.with_conn_inner(|conn| {
            conn.execute(
                "UPDATE meta_templates SET content = ?1, updated_at = datetime('now') WHERE key = ?2",
                params![content, category]
            )?;

            Ok(())
        })
    }
}

/// 计算加权分数
///
/// # 参数
/// - `rating`: 用户评分（1-5）
///
/// # 返回
/// 加权分数（0-1）
fn calculate_weighted_score(cosine_similarity: f64, rating: i32) -> f64 {
    let rating_normalized = rating as f64 / 5.0;
    0.7 * cosine_similarity + 0.3 * rating_normalized
}
