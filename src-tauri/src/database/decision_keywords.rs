//! 决策关键词数据库操作
//!
//! 用于管理决策关键词配置的数据库仓库

use rusqlite::{Connection, params};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 决策关键词数据结构
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(rename_all = "camelCase")]
pub struct DecisionKeyword {
    /// 数据库 ID
    pub id: i64,
    /// 关键词
    pub keyword: String,
    /// 语言（zh | en）
    pub language: String,
    /// 决策类型
    /// - architecture_design: 架构设计
    /// - technology_choice: 技术选型
    /// - tool_selection: 工具选择
    /// - implementation: 代码实现
    /// - other: 其他
    pub decision_type: String,
    /// 是否激活
    pub is_active: bool,
    /// 权重（用于排序）
    pub weight: f64,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

/// 决策关键词数据库仓库
pub struct DecisionKeywordRepository {
    db_path: String,
}

impl DecisionKeywordRepository {
    /// 创建新的仓库实例
    pub fn new(db_path: String) -> Self {
        Self { db_path }
    }

    /// 添加或更新关键词
    ///
    /// 使用 ON CONFLICT 实现 upsert 语义
    pub fn upsert(&self, keyword: &DecisionKeyword) -> anyhow::Result<i64> {
        let conn = Connection::open(&self.db_path)
            .context("无法打开数据库")?;

        conn.execute(
            "INSERT INTO decision_keywords (keyword, language, decision_type, is_active, weight, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(keyword, language, decision_type) DO UPDATE SET
             is_active = ?4, weight = ?5, updated_at = datetime('now', 'localtime')",
            params![
                &keyword.keyword,
                &keyword.language,
                &keyword.decision_type,
                if keyword.is_active { 1i64 } else { 0i64 },
                keyword.weight,
                &keyword.created_at,
                &keyword.updated_at,
            ],
        ).context("插入或更新决策关键词失败")?;

        Ok(conn.last_insert_rowid())
    }

    /// 获取所有激活的关键词
    pub fn get_active_keywords(&self) -> anyhow::Result<Vec<DecisionKeyword>> {
        let conn = Connection::open(&self.db_path)
            .context("无法打开数据库")?;

        let mut stmt = conn.prepare(
            "SELECT id, keyword, language, decision_type, is_active, weight, created_at, updated_at
             FROM decision_keywords
             WHERE is_active = 1
             ORDER BY weight DESC"
        ).context("准备查询失败")?;

        let keywords = stmt.query_map([], |row| {
            Ok(DecisionKeyword {
                id: row.get(0)?,
                keyword: row.get(1)?,
                language: row.get(2)?,
                decision_type: row.get(3)?,
                is_active: row.get::<_, i64>(4)? == 1,
                weight: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        }).context("执行查询失败")?
            .collect::<Result<Vec<_>, _>>()
            .context("解析查询结果失败")?;

        Ok(keywords)
    }

    /// 根据语言获取激活的关键词
    pub fn get_by_language(&self, language: &str) -> anyhow::Result<Vec<DecisionKeyword>> {
        let conn = Connection::open(&self.db_path)
            .context("无法打开数据库")?;

        let mut stmt = conn.prepare(
            "SELECT id, keyword, language, decision_type, is_active, weight, created_at, updated_at
             FROM decision_keywords
             WHERE language = ?1 AND is_active = 1
             ORDER BY weight DESC"
        ).context("准备查询失败")?;

        let keywords = stmt.query_map(params![language], |row| {
            Ok(DecisionKeyword {
                id: row.get(0)?,
                keyword: row.get(1)?,
                language: row.get(2)?,
                decision_type: row.get(3)?,
                is_active: row.get::<_, i64>(4)? == 1,
                weight: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        }).context("执行查询失败")?
            .collect::<Result<Vec<_>, _>>()
            .context("解析查询结果失败")?;

        Ok(keywords)
    }

    /// 根据决策类型获取激活的关键词
    pub fn get_by_decision_type(&self, decision_type: &str) -> anyhow::Result<Vec<DecisionKeyword>> {
        let conn = Connection::open(&self.db_path)
            .context("无法打开数据库")?;

        let mut stmt = conn.prepare(
            "SELECT id, keyword, language, decision_type, is_active, weight, created_at, updated_at
             FROM decision_keywords
             WHERE decision_type = ?1 AND is_active = 1
             ORDER BY weight DESC"
        ).context("准备查询失败")?;

        let keywords = stmt.query_map(params![decision_type], |row| {
            Ok(DecisionKeyword {
                id: row.get(0)?,
                keyword: row.get(1)?,
                language: row.get(2)?,
                decision_type: row.get(3)?,
                is_active: row.get::<_, i64>(4)? == 1,
                weight: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        }).context("执行查询失败")?
            .collect::<Result<Vec<_>, _>>()
            .context("解析查询结果失败")?;

        Ok(keywords)
    }

    /// 根据语言和决策类型获取激活的关键词
    pub fn get_by_language_and_type(&self, language: &str, decision_type: &str) -> anyhow::Result<Vec<DecisionKeyword>> {
        let conn = Connection::open(&self.db_path)
            .context("无法打开数据库")?;

        let mut stmt = conn.prepare(
            "SELECT id, keyword, language, decision_type, is_active, weight, created_at, updated_at
             FROM decision_keywords
             WHERE language = ?1 AND decision_type = ?2 AND is_active = 1
             ORDER BY weight DESC"
        ).context("准备查询失败")?;

        let keywords = stmt.query_map(params![language, decision_type], |row| {
            Ok(DecisionKeyword {
                id: row.get(0)?,
                keyword: row.get(1)?,
                language: row.get(2)?,
                decision_type: row.get(3)?,
                is_active: row.get::<_, i64>(4)? == 1,
                weight: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        }).context("执行查询失败")?
            .collect::<Result<Vec<_>, _>>()
            .context("解析查询结果失败")?;

        Ok(keywords)
    }

    /// 删除关键词
    pub fn delete(&self, id: i64) -> anyhow::Result<bool> {
        let conn = Connection::open(&self.db_path)
            .context("无法打开数据库")?;

        let affected = conn.execute(
            "DELETE FROM decision_keywords WHERE id = ?1",
            params![id],
        ).context("删除关键词失败")?;

        Ok(affected > 0)
    }

    /// 批量导入关键词
    pub fn batch_import(&self, keywords: &[DecisionKeyword]) -> anyhow::Result<usize> {
        let conn = Connection::open(&self.db_path)
            .context("无法打开数据库")?;

        let tx = conn.unchecked_transaction().context("开始事务失败")?;

        let mut count = 0;
        for keyword in keywords {
            match tx.execute(
                "INSERT INTO decision_keywords (keyword, language, decision_type, is_active, weight, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(keyword, language, decision_type) DO UPDATE SET
                 is_active = ?4, weight = ?5, updated_at = datetime('now', 'localtime')",
                params![
                    &keyword.keyword,
                    &keyword.language,
                    &keyword.decision_type,
                    if keyword.is_active { 1i64 } else { 0i64 },
                    keyword.weight,
                    &keyword.created_at,
                    &keyword.updated_at,
                ],
            ) {
                Ok(_) => count += 1,
                Err(e) => {
                    log::warn!("插入关键词 {} 失败: {}", keyword.keyword, e);
                }
            }
        }

        tx.commit().context("提交事务失败")?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upsert_and_get() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db").to_str().unwrap().to_string();

        let repo = DecisionKeywordRepository::new(db_path);

        // 创建测试关键词
        let keyword = DecisionKeyword {
            id: 0,
            keyword: "选择".to_string(),
            language: "zh".to_string(),
            decision_type: "technology_choice".to_string(),
            is_active: true,
            weight: 1.0,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        // 插入
        let id = repo.upsert(&keyword).unwrap();
        assert!(id > 0);

        // 查询
        let keywords = repo.get_by_language("zh").unwrap();
        assert_eq!(keywords.len(), 1);
        assert_eq!(keywords[0].keyword, "选择");
    }

    #[test]
    fn test_get_by_decision_type() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db").to_str().unwrap().to_string();

        let repo = DecisionKeywordRepository::new(db_path);

        // 创建测试关键词
        let keyword1 = DecisionKeyword {
            id: 0,
            keyword: "架构".to_string(),
            language: "zh".to_string(),
            decision_type: "architecture_design".to_string(),
            is_active: true,
            weight: 1.0,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        let keyword2 = DecisionKeyword {
            id: 0,
            keyword: "选择".to_string(),
            language: "zh".to_string(),
            decision_type: "technology_choice".to_string(),
            is_active: true,
            weight: 1.0,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        repo.upsert(&keyword1).unwrap();
        repo.upsert(&keyword2).unwrap();

        // 查询架构设计类关键词
        let keywords = repo.get_by_decision_type("architecture_design").unwrap();
        assert_eq!(keywords.len(), 1);
        assert_eq!(keywords[0].keyword, "架构");

        // 查询技术选型类关键词
        let keywords = repo.get_by_decision_type("technology_choice").unwrap();
        assert_eq!(keywords.len(), 1);
        assert_eq!(keywords[0].keyword, "选择");
    }

    #[test]
    fn test_delete() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db").to_str().unwrap().to_string();

        let repo = DecisionKeywordRepository::new(db_path);

        let keyword = DecisionKeyword {
            id: 0,
            keyword: "测试".to_string(),
            language: "zh".to_string(),
            decision_type: "other".to_string(),
            is_active: true,
            weight: 1.0,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        let id = repo.upsert(&keyword).unwrap();
        let deleted = repo.delete(id).unwrap();
        assert!(deleted);

        let keywords = repo.get_by_language("zh").unwrap();
        assert_eq!(keywords.len(), 0);
    }

    #[test]
    fn test_batch_import() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db").to_str().unwrap().to_string();

        let repo = DecisionKeywordRepository::new(db_path);

        let now = chrono::Utc::now().to_rfc3339();
        let keywords = vec![
            DecisionKeyword {
                id: 0,
                keyword: "选择".to_string(),
                language: "zh".to_string(),
                decision_type: "technology_choice".to_string(),
                is_active: true,
                weight: 1.0,
                created_at: now.clone(),
                updated_at: now.clone(),
            },
            DecisionKeyword {
                id: 0,
                keyword: "采用".to_string(),
                language: "zh".to_string(),
                decision_type: "technology_choice".to_string(),
                is_active: true,
                weight: 1.0,
                created_at: now.clone(),
                updated_at: now.clone(),
            },
        ];

        let count = repo.batch_import(&keywords).unwrap();
        assert_eq!(count, 2);

        let all_keywords = repo.get_by_language("zh").unwrap();
        assert_eq!(all_keywords.len(), 2);
    }
}
