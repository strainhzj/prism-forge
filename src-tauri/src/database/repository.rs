//! API Provider 数据仓库
//!
//! 提供 api_providers 表的 CRUD 操作

use anyhow::Result;
use rusqlite::{Connection, params};
use chrono::Utc;

use crate::database::models::{ApiProvider, ApiProviderType};

/// API Provider 数据仓库
pub struct ApiProviderRepository {
    conn: Connection,
}

// SQLite Connection 是 Send 但不是 Sync，我们需要手动实现
// 注意：每个线程应该有自己的连接
unsafe impl Send for ApiProviderRepository {}

impl ApiProviderRepository {
    /// 创建新的仓库实例
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }

    /// 从默认数据库路径创建仓库
    pub fn from_default_db() -> Result<Self> {
        let conn = crate::database::migrations::get_connection()?;
        Ok(Self::new(conn))
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

        self.conn.execute(
            "INSERT INTO api_providers (
                provider_type, name, base_url, api_key_ref,
                config_json, is_active, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                provider_type_str,
                provider.name,
                provider.base_url,
                provider.api_key_ref,
                provider.config_json,
                if provider.is_active { 1 } else { 0 },
                now,
                now,
            ],
        )?;

        provider.id = Some(self.conn.last_insert_rowid());
        Ok(provider)
    }

    /// 获取所有提供商
    ///
    /// # 返回
    /// 返回所有提供商的列表，按创建时间倒序排列
    pub fn get_all_providers(&self) -> Result<Vec<ApiProvider>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, provider_type, name, base_url, api_key_ref,
                    config_json, is_active, created_at, updated_at
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
                config_json: row.get(5)?,
                is_active: row.get::<_, i32>(6)? == 1,
            })
        })?;

        providers.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// 根据ID获取提供商
    ///
    /// # 参数
    /// - `id`: 提供商 ID
    pub fn get_provider_by_id(&self, id: i64) -> Result<Option<ApiProvider>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, provider_type, name, base_url, api_key_ref,
                    config_json, is_active, created_at, updated_at
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
                config_json: row.get(5)?,
                is_active: row.get::<_, i32>(6)? == 1,
            }))
        } else {
            Ok(None)
        }
    }

    /// 获取当前活跃的提供商
    ///
    /// # 返回
    /// 返回活跃的提供商，如果没有则返回 None
    pub fn get_active_provider(&self) -> Result<Option<ApiProvider>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, provider_type, name, base_url, api_key_ref,
                    config_json, is_active, created_at, updated_at
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
                config_json: row.get(5)?,
                is_active: row.get::<_, i32>(6)? == 1,
            }))
        } else {
            Ok(None)
        }
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

        let rows = self.conn.execute(
            "UPDATE api_providers
             SET provider_type = ?1, name = ?2, base_url = ?3,
                 api_key_ref = ?4, config_json = ?5, is_active = ?6,
                 updated_at = ?7
             WHERE id = ?8",
            params![
                provider_type_str,
                provider.name,
                provider.base_url,
                provider.api_key_ref,
                provider.config_json,
                if provider.is_active { 1 } else { 0 },
                now,
                id,
            ],
        )?;

        Ok(rows)
    }

    /// 删除提供商
    ///
    /// # 参数
    /// - `id`: 要删除的提供商 ID
    ///
    /// # 返回
    /// 返回删除的行数，如果为 0 表示没有找到对应的提供商
    pub fn delete_provider(&self, id: i64) -> Result<usize> {
        let rows = self.conn.execute(
            "DELETE FROM api_providers WHERE id = ?1",
            params![id],
        )?;

        Ok(rows)
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

        let rows = self.conn.execute(
            "UPDATE api_providers
             SET is_active = 1, updated_at = ?1
             WHERE id = ?2",
            params![now, id],
        )?;

        Ok(rows)
    }

    /// 根据 provider_type 获取提供商列表
    ///
    /// # 参数
    /// - `provider_type`: 提供商类型
    pub fn get_providers_by_type(&self, provider_type: ApiProviderType) -> Result<Vec<ApiProvider>> {
        let provider_type_str = serde_json::to_string(&provider_type)?;

        let mut stmt = self.conn.prepare(
            "SELECT id, provider_type, name, base_url, api_key_ref,
                    config_json, is_active, created_at, updated_at
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
                config_json: row.get(5)?,
                is_active: row.get::<_, i32>(6)? == 1,
            })
        })?;

        providers.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// 统计提供商数量
    pub fn count_providers(&self) -> Result<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM api_providers",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::migrations;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();
        // 执行迁移
        migrations::migrate_v1(&mut conn.clone()).unwrap();
        conn
    }

    #[test]
    fn test_create_and_get_provider() {
        let conn = setup_test_db();
        let repo = ApiProviderRepository::new(conn);

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
        let repo = ApiProviderRepository::new(conn);

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
        let repo = ApiProviderRepository::new(conn);

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
        let repo = ApiProviderRepository::new(conn);

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
        let repo = ApiProviderRepository::new(conn);

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
