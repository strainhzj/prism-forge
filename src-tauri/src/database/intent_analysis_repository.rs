//! 意图分析历史数据仓库
//!
//! 提供 intent_analysis_history 表的 CRUD 操作

use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};
use serde_json;
use ts_rs::TS;

use crate::intent_analyzer::{DecisionQAPair, OpeningIntent};

/// 意图分析历史记录
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct IntentAnalysisHistory {
    /// 数据库 ID
    pub id: i64,
    /// 会话文件路径（唯一键）
    pub session_file_path: String,
    /// 问答对列表
    pub qa_pairs: Vec<DecisionQAPair>,
    /// 开场白意图
    pub opening_intent: OpeningIntent,
    /// 语言标识
    pub language: String,
    /// 分析时间
    pub analyzed_at: String,
    /// 创建时间
    pub created_at: String,
}

/// 意图分析历史数据仓库
pub struct IntentAnalysisRepository {
    conn: Arc<Mutex<Connection>>,
}

impl IntentAnalysisRepository {
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

    /// 保存意图分析结果
    ///
    /// 如果该会话文件路径已存在记录，则覆盖更新
    pub fn save_analysis(
        &self,
        session_file_path: &str,
        qa_pairs: &[DecisionQAPair],
        opening_intent: &OpeningIntent,
        language: &str,
    ) -> Result<i64> {
        self.with_conn_inner(|conn| {
            let now = Utc::now().to_rfc3339();
            let qa_pairs_json = serde_json::to_string(qa_pairs)?;
            let opening_intent_json = serde_json::to_string(opening_intent)?;

            // 使用 INSERT OR REPLACE 实现覆盖更新
            conn.execute(
                "INSERT OR REPLACE INTO intent_analysis_history
                 (session_file_path, qa_pairs_json, opening_intent_json, language, analyzed_at, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    session_file_path,
                    qa_pairs_json,
                    opening_intent_json,
                    language,
                    now,
                    now,
                ],
            )?;

            let id = conn.last_insert_rowid();
            Ok(id)
        })
    }

    /// 获取指定会话文件的分析历史
    pub fn get_analysis_by_session(&self, session_file_path: &str) -> Result<Option<IntentAnalysisHistory>> {
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, session_file_path, qa_pairs_json, opening_intent_json,
                        language, analyzed_at, created_at
                 FROM intent_analysis_history
                 WHERE session_file_path = ?1"
            )?;

            let result = stmt.query_row(params![session_file_path], |row| {
                let qa_pairs_json: String = row.get(2)?;
                let opening_intent_json: String = row.get(3)?;

                // 🔍 调试日志：输出原始 JSON
                #[cfg(debug_assertions)]
                {
                    eprintln!("[get_analysis_by_session] opening_intent_json: {}", &opening_intent_json);
                    eprintln!("[get_analysis_by_session] qa_pairs_json: {}", &qa_pairs_json);
                }

                let qa_pairs: Vec<DecisionQAPair> = serde_json::from_str(&qa_pairs_json)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                let opening_intent: OpeningIntent = serde_json::from_str(&opening_intent_json)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                // 🔍 调试日志：输出反序列化后的结构
                #[cfg(debug_assertions)]
                {
                    eprintln!("[get_analysis_by_session] opening_intent.intent_type: {}", opening_intent.intent_type);
                    eprintln!("[get_analysis_by_session] opening_intent.confidence: {}", opening_intent.confidence);
                    eprintln!("[get_analysis_by_session] qa_pairs length: {}", qa_pairs.len());
                }

                Ok(IntentAnalysisHistory {
                    id: row.get(0)?,
                    session_file_path: row.get(1)?,
                    qa_pairs,
                    opening_intent,
                    language: row.get(4)?,
                    analyzed_at: row.get(5)?,
                    created_at: row.get(6)?,
                })
            });

            match result {
                Ok(history) => Ok(Some(history)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e.into()),
            }
        })
    }

    /// 清除指定会话文件的分析历史
    pub fn delete_analysis(&self, session_file_path: &str) -> Result<bool> {
        self.with_conn_inner(|conn| {
            let rows_affected = conn.execute(
                "DELETE FROM intent_analysis_history WHERE session_file_path = ?1",
                params![session_file_path],
            )?;
            Ok(rows_affected > 0)
        })
    }

    /// 获取所有分析历史记录（按时间倒序）
    pub fn get_all_histories(&self) -> Result<Vec<IntentAnalysisHistory>> {
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, session_file_path, qa_pairs_json, opening_intent_json,
                        language, analyzed_at, created_at
                 FROM intent_analysis_history
                 ORDER BY analyzed_at DESC"
            )?;

            let histories = stmt.query_map([], |row| {
                let qa_pairs_json: String = row.get(2)?;
                let opening_intent_json: String = row.get(3)?;

                let qa_pairs: Vec<DecisionQAPair> = serde_json::from_str(&qa_pairs_json)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                let opening_intent: OpeningIntent = serde_json::from_str(&opening_intent_json)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                Ok(IntentAnalysisHistory {
                    id: row.get(0)?,
                    session_file_path: row.get(1)?,
                    qa_pairs,
                    opening_intent,
                    language: row.get(4)?,
                    analyzed_at: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })?;

            histories
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.into())
        })
    }

    /// 更新分析时间（用于重新分析）
    pub fn update_analyzed_at(&self, session_file_path: &str) -> Result<bool> {
        self.with_conn_inner(|conn| {
            let now = Utc::now().to_rfc3339();
            let rows_affected = conn.execute(
                "UPDATE intent_analysis_history SET analyzed_at = ?1 WHERE session_file_path = ?2",
                params![now, session_file_path],
            )?;
            Ok(rows_affected > 0)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::migrations;
    use crate::intent_analyzer::qa_detector::QAPairDetector;
    use crate::database::models::Message;

    fn create_test_message(
        uuid: &str,
        msg_type: &str,
        content: Option<&str>,
    ) -> Message {
        Message {
            id: None,
            session_id: "test-session".to_string(),
            uuid: uuid.to_string(),
            parent_uuid: None,
            msg_type: msg_type.to_string(),
            content_type: None,
            timestamp: Utc::now().to_rfc3339(),
            offset: 0,
            length: 0,
            summary: None,
            content: content.map(|s| s.to_string()),
            parent_idx: None,
            created_at: Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn test_save_and_get_analysis() {
        // 创建内存数据库
        let mut conn = Connection::open_in_memory().unwrap();
        migrations::migrate_v21_impl(&mut conn).unwrap();

        let repo = IntentAnalysisRepository::with_conn(Arc::new(Mutex::new(conn)));

        // 准备测试数据
        let messages = vec![
            create_test_message("u1", "user", Some("开场白")),
            create_test_message("a1", "assistant", Some("回答1")),
            create_test_message("u2", "user", Some("用户决策1")),
        ];

        let detector = QAPairDetector::new();
        let qa_pairs = detector.detect_decision_qa_pairs(messages);

        let opening_intent = OpeningIntent {
            original_text: "开场白内容".to_string(),
            core_intent: "测试意图".to_string(),
            key_entities: vec![],
            context_clues: vec![],
            confidence_score: 0.85,
            reasoning: Some("测试推理".to_string()),
        };

        // 保存分析
        let id = repo
            .save_analysis("test_session.jsonl", &qa_pairs, &opening_intent, "zh")
            .unwrap();

        assert!(id > 0);

        // 读取分析
        let history = repo.get_analysis_by_session("test_session.jsonl").unwrap();
        assert!(history.is_some());
        let history = history.unwrap();

        assert_eq!(history.session_file_path, "test_session.jsonl");
        assert_eq!(history.language, "zh");
        assert_eq!(history.qa_pairs.len(), 1);
        assert_eq!(history.qa_pairs[0].assistant_answer, "回答1");
        assert_eq!(history.qa_pairs[0].user_decision, "用户决策1");
        assert_eq!(history.opening_intent.core_intent, "测试意图");
    }

    #[test]
    fn test_save_replaces_existing() {
        // 创建内存数据库
        let mut conn = Connection::open_in_memory().unwrap();
        migrations::migrate_v21_impl(&mut conn).unwrap();

        let repo = IntentAnalysisRepository::with_conn(Arc::new(Mutex::new(conn)));

        let opening_intent = OpeningIntent {
            original_text: "原始内容".to_string(),
            core_intent: "原始意图".to_string(),
            key_entities: vec![],
            context_clues: vec![],
            confidence_score: 0.5,
            reasoning: Some("原始推理".to_string()),
        };

        // 第一次保存
        let id1 = repo
            .save_analysis("test.jsonl", &[], &opening_intent, "zh")
            .unwrap();
        assert!(id1 > 0);

        // 第二次保存（应该覆盖）
        let updated_intent = OpeningIntent {
            original_text: "更新内容".to_string(),
            core_intent: "更新意图".to_string(),
            key_entities: vec![],
            context_clues: vec![],
            confidence_score: 0.9,
            reasoning: Some("更新推理".to_string()),
        };

        let id2 = repo
            .save_analysis("test.jsonl", &[], &updated_intent, "zh")
            .unwrap();
        assert!(id2 > 0);

        // 验证只有一条记录且是更新后的内容
        let history = repo.get_analysis_by_session("test.jsonl").unwrap().unwrap();
        assert_eq!(history.opening_intent.core_intent, "更新意图");
        assert_eq!(history.opening_intent.confidence_score, 0.9);
    }

    #[test]
    fn test_delete_analysis() {
        // 创建内存数据库
        let mut conn = Connection::open_in_memory().unwrap();
        migrations::migrate_v21_impl(&mut conn).unwrap();

        let repo = IntentAnalysisRepository::with_conn(Arc::new(Mutex::new(conn)));

        let opening_intent = OpeningIntent {
            original_text: "测试内容".to_string(),
            core_intent: "测试意图".to_string(),
            key_entities: vec![],
            context_clues: vec![],
            confidence_score: 0.8,
            reasoning: None,
        };

        // 保存
        repo.save_analysis("test.jsonl", &[], &opening_intent, "zh")
            .unwrap();

        // 删除
        let deleted = repo.delete_analysis("test.jsonl").unwrap();
        assert!(deleted);

        // 验证已删除
        let history = repo.get_analysis_by_session("test.jsonl").unwrap();
        assert!(history.is_none());
    }

    #[test]
    fn test_get_all_histories() {
        // 创建内存数据库
        let mut conn = Connection::open_in_memory().unwrap();
        migrations::migrate_v21_impl(&mut conn).unwrap();

        let repo = IntentAnalysisRepository::with_conn(Arc::new(Mutex::new(conn)));

        let opening_intent = OpeningIntent {
            original_text: "测试内容".to_string(),
            core_intent: "测试意图".to_string(),
            key_entities: vec![],
            context_clues: vec![],
            confidence_score: 0.8,
            reasoning: None,
        };

        // 保存多条记录
        repo.save_analysis("test1.jsonl", &[], &opening_intent, "zh")
            .unwrap();
        repo.save_analysis("test2.jsonl", &[], &opening_intent, "en")
            .unwrap();

        // 获取所有历史
        let histories = repo.get_all_histories().unwrap();
        assert_eq!(histories.len(), 2);
    }
}
