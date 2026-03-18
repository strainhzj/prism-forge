//! 决策分析历史数据仓库
//!
//! 提供 decision_analysis_history 表的 CRUD 操作

use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};
use serde_json;
use ts_rs::TS;

use crate::intent_analyzer::decision_analyzer::DecisionAnalysis;

/// 决策分析历史记录
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct DecisionAnalysisHistory {
    /// 数据库 ID
    pub id: i64,
    /// 会话文件路径
    pub session_file_path: String,
    /// 问答对索引
    pub qa_index: i64,
    /// 决策分析结果
    pub decision_analysis: DecisionAnalysis,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

/// 决策分析历史数据仓库
pub struct DecisionAnalysisRepository {
    conn: Arc<Mutex<Connection>>,
}

impl DecisionAnalysisRepository {
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
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("获取数据库连接锁失败: {}", e))?;
        f(&conn)
    }

    /// 保存或更新决策分析结果
    ///
    /// 如果该会话文件路径和 QA 索引已存在记录，则覆盖更新
    pub fn save_analysis(
        &self,
        session_file_path: &str,
        qa_index: i64,
        decision_analysis: &DecisionAnalysis,
    ) -> Result<i64> {
        self.with_conn_inner(|conn| {
            let now = Utc::now().to_rfc3339();
            let decision_analysis_json = serde_json::to_string(decision_analysis)?;

            // 使用 INSERT OR REPLACE 实现覆盖更新
            conn.execute(
                "INSERT OR REPLACE INTO decision_analysis_history
                 (session_file_path, qa_index, decision_analysis_json, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    session_file_path,
                    qa_index,
                    decision_analysis_json,
                    now,
                    now,
                ],
            )?;

            let id = conn.last_insert_rowid();
            Ok(id)
        })
    }

    /// 获取指定会话文件和 QA 索引的决策分析历史
    pub fn get_analysis(
        &self,
        session_file_path: &str,
        qa_index: i64,
    ) -> Result<Option<DecisionAnalysisHistory>> {
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, session_file_path, qa_index, decision_analysis_json, created_at, updated_at
                 FROM decision_analysis_history
                 WHERE session_file_path = ?1 AND qa_index = ?2"
            )?;

            let result = stmt.query_row(params![session_file_path, qa_index], |row| {
                let decision_analysis_json: String = row.get(3)?;

                #[cfg(debug_assertions)]
                {
                    eprintln!("[get_analysis] decision_analysis_json: {}", &decision_analysis_json);
                }

                let decision_analysis: DecisionAnalysis = serde_json::from_str(&decision_analysis_json)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                Ok(DecisionAnalysisHistory {
                    id: row.get(0)?,
                    session_file_path: row.get(1)?,
                    qa_index: row.get(2)?,
                    decision_analysis,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            });

            match result {
                Ok(history) => Ok(Some(history)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e.into()),
            }
        })
    }

    /// 获取指定会话文件的所有决策分析历史
    pub fn get_analyses_by_session(&self, session_file_path: &str) -> Result<Vec<DecisionAnalysisHistory>> {
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, session_file_path, qa_index, decision_analysis_json, created_at, updated_at
                 FROM decision_analysis_history
                 WHERE session_file_path = ?1
                 ORDER BY qa_index ASC"
            )?;

            let histories = stmt.query_map(params![session_file_path], |row| {
                let decision_analysis_json: String = row.get(3)?;
                let decision_analysis: DecisionAnalysis = serde_json::from_str(&decision_analysis_json)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                Ok(DecisionAnalysisHistory {
                    id: row.get(0)?,
                    session_file_path: row.get(1)?,
                    qa_index: row.get(2)?,
                    decision_analysis,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })?;

            histories.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
        })
    }

    /// 清除指定会话文件的所有决策分析历史
    pub fn delete_analyses_by_session(&self, session_file_path: &str) -> Result<usize> {
        self.with_conn_inner(|conn| {
            let rows_affected = conn.execute(
                "DELETE FROM decision_analysis_history WHERE session_file_path = ?1",
                params![session_file_path],
            )?;
            Ok(rows_affected)
        })
    }

    /// 清除指定会话文件和 QA 索引的决策分析历史
    pub fn delete_analysis(&self, session_file_path: &str, qa_index: i64) -> Result<bool> {
        self.with_conn_inner(|conn| {
            let rows_affected = conn.execute(
                "DELETE FROM decision_analysis_history WHERE session_file_path = ?1 AND qa_index = ?2",
                params![session_file_path, qa_index],
            )?;
            Ok(rows_affected > 0)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::migrations;
    use crate::intent_analyzer::decision_analyzer::{DecisionAnalysis, DecisionType, Alternative};

    fn create_test_analysis() -> DecisionAnalysis {
        DecisionAnalysis {
            decision_made: "选择使用 Rust 开发".to_string(),
            decision_type: DecisionType::TechnologyChoice,
            tech_stack: vec!["Rust".to_string(), "Tauri".to_string()],
            rationale: vec!["性能要求高".to_string()],
            inferred_reasons: vec!["用户熟悉 Rust".to_string()],
            alternatives: vec![Alternative {
                name: "Electron".to_string(),
                reason: Some("性能较差".to_string()),
            }],
            confidence: 0.9,
        }
    }

    #[test]
    fn test_save_and_get_analysis() {
        // 创建内存数据库
        let mut conn = Connection::open_in_memory().unwrap();
        migrations::migrate_v22(&mut conn).unwrap();

        let repo = DecisionAnalysisRepository::with_conn(Arc::new(Mutex::new(conn)));

        // 保存分析
        let analysis = create_test_analysis();
        let id = repo
            .save_analysis("test_session.jsonl", 0, &analysis)
            .unwrap();

        assert!(id > 0);

        // 读取分析
        let history = repo.get_analysis("test_session.jsonl", 0).unwrap();
        assert!(history.is_some());
        let history = history.unwrap();

        assert_eq!(history.session_file_path, "test_session.jsonl");
        assert_eq!(history.qa_index, 0);
        assert_eq!(history.decision_analysis.decision_made, "选择使用 Rust 开发");
        assert!(matches!(history.decision_analysis.decision_type, DecisionType::TechnologyChoice));
        assert_eq!(history.decision_analysis.confidence, 0.9);
    }

    #[test]
    fn test_save_replaces_existing() {
        // 创建内存数据库
        let mut conn = Connection::open_in_memory().unwrap();
        migrations::migrate_v22(&mut conn).unwrap();

        let repo = DecisionAnalysisRepository::with_conn(Arc::new(Mutex::new(conn)));

        // 第一次保存
        let analysis1 = create_test_analysis();
        let id1 = repo
            .save_analysis("test.jsonl", 0, &analysis1)
            .unwrap();
        assert!(id1 > 0);

        // 第二次保存（应该覆盖）
        let mut analysis2 = create_test_analysis();
        analysis2.decision_made = "选择使用 TypeScript 开发".to_string();
        analysis2.confidence = 0.8;

        let id2 = repo
            .save_analysis("test.jsonl", 0, &analysis2)
            .unwrap();
        assert!(id2 > 0);

        // 验证只有一条记录且是更新后的内容
        let history = repo.get_analysis("test.jsonl", 0).unwrap().unwrap();
        assert_eq!(history.decision_analysis.decision_made, "选择使用 TypeScript 开发");
        assert_eq!(history.decision_analysis.confidence, 0.8);
    }

    #[test]
    fn test_delete_analysis() {
        // 创建内存数据库
        let mut conn = Connection::open_in_memory().unwrap();
        migrations::migrate_v22(&mut conn).unwrap();

        let repo = DecisionAnalysisRepository::with_conn(Arc::new(Mutex::new(conn)));

        // 保存
        let analysis = create_test_analysis();
        repo.save_analysis("test.jsonl", 0, &analysis)
            .unwrap();

        // 删除
        let deleted = repo.delete_analysis("test.jsonl", 0).unwrap();
        assert!(deleted);

        // 验证已删除
        let history = repo.get_analysis("test.jsonl", 0).unwrap();
        assert!(history.is_none());
    }

    #[test]
    fn test_get_analyses_by_session() {
        // 创建内存数据库
        let mut conn = Connection::open_in_memory().unwrap();
        migrations::migrate_v22(&mut conn).unwrap();

        let repo = DecisionAnalysisRepository::with_conn(Arc::new(Mutex::new(conn)));

        // 保存多条记录
        let analysis = create_test_analysis();
        repo.save_analysis("test.jsonl", 0, &analysis).unwrap();
        repo.save_analysis("test.jsonl", 1, &analysis).unwrap();
        repo.save_analysis("test.jsonl", 2, &analysis).unwrap();

        // 获取所有分析
        let histories = repo.get_analyses_by_session("test.jsonl").unwrap();
        assert_eq!(histories.len(), 3);
        assert_eq!(histories[0].qa_index, 0);
        assert_eq!(histories[1].qa_index, 1);
        assert_eq!(histories[2].qa_index, 2);
    }

    #[test]
    fn test_delete_analyses_by_session() {
        // 创建内存数据库
        let mut conn = Connection::open_in_memory().unwrap();
        migrations::migrate_v22(&mut conn).unwrap();

        let repo = DecisionAnalysisRepository::with_conn(Arc::new(Mutex::new(conn)));

        // 保存多条记录
        let analysis = create_test_analysis();
        repo.save_analysis("test.jsonl", 0, &analysis).unwrap();
        repo.save_analysis("test.jsonl", 1, &analysis).unwrap();

        // 删除会话的所有分析
        let deleted_count = repo.delete_analyses_by_session("test.jsonl").unwrap();
        assert_eq!(deleted_count, 2);

        // 验证已全部删除
        let histories = repo.get_analyses_by_session("test.jsonl").unwrap();
        assert_eq!(histories.len(), 0);
    }
}
