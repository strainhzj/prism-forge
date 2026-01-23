//! 向量搜索仓库
//!
//! 负责会话向量的存储和相似度搜索

use anyhow::Result;
use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex};

use crate::database::models::{SessionEmbedding, Session, VectorSearchResult};

/// 向量搜索仓库
pub struct VectorRepository {
    pub conn: Arc<Mutex<Connection>>,
}

impl VectorRepository {
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

    /// 保存或更新会话向量
    ///
    /// # 参数
    /// - `embedding`: 会话向量数据
    ///
    /// # 返回
    /// 返回保存后的向量数据（包含生成的 ID）
    pub fn upsert_session_embedding(&self, embedding: SessionEmbedding) -> Result<SessionEmbedding> {
        self.with_conn_inner(|conn| {
            let now = chrono::Utc::now().to_rfc3339();

            // 尝试更新
            let rows_affected = conn.execute(
                "UPDATE session_embeddings
                 SET embedding = ?1, summary = ?2, dimension = ?3, updated_at = ?4
                 WHERE session_id = ?5",
                params![
                    &embedding.embedding,
                    &embedding.summary,
                    &embedding.dimension,
                    now,
                    &embedding.session_id,
                ],
            )?;

            if rows_affected > 0 {
                // 更新成功
                Ok(embedding)
            } else {
                // 插入新记录
                conn.execute(
                    "INSERT INTO session_embeddings (session_id, embedding, summary, dimension, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        &embedding.session_id,
                        &embedding.embedding,
                        &embedding.summary,
                        &embedding.dimension,
                        now,
                        now,
                    ],
                )?;

                Ok(embedding)
            }
        })
    }

    /// 批量保存会话向量
    ///
    /// # 参数
    /// - `embeddings`: 向量列表
    ///
    /// # 返回
    /// 返回保存的数量
    pub fn batch_upsert_embeddings(&self, embeddings: Vec<SessionEmbedding>) -> Result<usize> {
        self.with_conn_inner(|conn| {
            let now = chrono::Utc::now().to_rfc3339();
            let mut count = 0;

            for embedding in embeddings {
                // 尝试更新
                let rows_affected = conn.execute(
                    "UPDATE session_embeddings
                     SET embedding = ?1, summary = ?2, dimension = ?3, updated_at = ?4
                     WHERE session_id = ?5",
                    params![
                        &embedding.embedding,
                        &embedding.summary,
                        &embedding.dimension,
                        now,
                        &embedding.session_id,
                    ],
                )?;

                if rows_affected == 0 {
                    // 插入新记录
                    conn.execute(
                        "INSERT INTO session_embeddings (session_id, embedding, summary, dimension, created_at, updated_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        params![
                            &embedding.session_id,
                            &embedding.embedding,
                            &embedding.summary,
                            &embedding.dimension,
                            now,
                            now,
                        ],
                    )?;
                }

                count += 1;
            }

            Ok(count)
        })
    }

    /// 获取所有会话向量
    ///
    /// # 返回
    /// 返回所有向量数据
    pub fn get_all_embeddings(&self) -> Result<Vec<SessionEmbedding>> {
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, session_id, embedding, summary, dimension, created_at, updated_at
                 FROM session_embeddings
                 ORDER BY created_at DESC",
            )?;

            let embeddings = stmt
                .query_map([], |row| {
                    Ok(SessionEmbedding {
                        id: Some(row.get(0)?),
                        session_id: row.get(1)?,
                        embedding: row.get(2)?,
                        summary: row.get(3)?,
                        dimension: row.get(4)?,
                        created_at: row.get(5)?,
                        updated_at: row.get(6)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| anyhow::anyhow!("查询失败: {}", e))?;

            Ok(embeddings)
        })
    }

    /// 获取会话向量
    ///
    /// # 参数
    /// - `session_id`: 会话 ID
    ///
    /// # 返回
    /// 返回向量数据或错误
    pub fn get_session_embedding(&self, session_id: &str) -> Result<Option<SessionEmbedding>> {
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, session_id, embedding, summary, dimension, created_at, updated_at
                 FROM session_embeddings
                 WHERE session_id = ?1",
            )?;

            let result = stmt.query_row(params![session_id], |row| {
                Ok(SessionEmbedding {
                    id: Some(row.get(0)?),
                    session_id: row.get(1)?,
                    embedding: row.get(2)?,
                    summary: row.get(3)?,
                    dimension: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            });

            match result {
                Ok(embedding) => Ok(Some(embedding)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(anyhow::anyhow!("查询失败: {}", e)),
            }
        })
    }

    /// 删除会话向量
    ///
    /// # 参数
    /// - `session_id`: 会话 ID
    pub fn delete_session_embedding(&self, session_id: &str) -> Result<()> {
        self.with_conn_inner(|conn| {
            conn.execute(
                "DELETE FROM session_embeddings WHERE session_id = ?1",
                params![session_id],
            )?;
            Ok(())
        })
    }

    /// 向量相似度搜索
    ///
    /// # 参数
    /// - `query_vector`: 查询向量
    /// - `top_k`: 返回前 K 个结果（默认 10）
    /// - `min_similarity`: 最小相似度阈值（0.0-1.0，默认 0.0）
    ///
    /// # 返回
    /// 返回按相似度排序的搜索结果
    ///
    /// # 算法
    /// 使用余弦相似度：cosine_sim = (A · B) / (||A|| * ||B||)
    pub fn vector_search_sessions(
        &self,
        query_vector: &[f32],
        top_k: usize,
        min_similarity: f64,
    ) -> Result<Vec<VectorSearchResult>> {
        // 获取所有向量
        let embeddings = self.get_all_embeddings()?;

        // 计算相似度
        let mut results: Vec<VectorSearchResult> = embeddings
            .into_iter()
            .filter_map(|embedding| {
                // 解析向量
                match embedding.get_embedding() {
                    Ok(vec) => {
                        // 计算余弦相似度
                        let similarity = cosine_similarity(query_vector, &vec);

                        // 过滤低于阈值的结果
                        if similarity >= min_similarity {
                            Some((embedding, similarity))
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                }
            })
            .map(|(embedding, similarity)| {
                // 构造结果（暂时只返回向量，不包含 Session 信息）
                VectorSearchResult {
                    // 注意：这里暂时返回一个空的 Session，需要后续完善
                    session: Session {
                        id: None,
                        session_id: embedding.session_id.clone(),
                        project_path: String::new(),
                        project_name: String::new(),
                        file_path: String::new(),
                        rating: None,
                        tags: "[]".to_string(),
                        is_archived: false,
                        is_active: false,
                        created_at: embedding.created_at.clone(),
                        updated_at: embedding.updated_at,
                    },
                    similarity_score: similarity,
                    summary: embedding.summary,
                }
            })
            .collect();

        // 按相似度降序排序（分数越高越相似）
        results.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 返回前 top_k 个结果
        results.truncate(top_k);

        Ok(results)
    }

    /// 获取未向量化的会话数量
    ///
    /// # 返回
    /// 返回没有向量的会话数量
    pub fn get_non_vectorized_count(&self) -> Result<i64> {
        self.with_conn_inner(|conn| {
            let count = conn.query_row(
                "SELECT COUNT(*) FROM sessions s
                 LEFT JOIN session_embeddings e ON s.session_id = e.session_id
                 WHERE e.session_id IS NULL",
                [],
                |row| row.get(0),
            )?;
            Ok(count)
        })
    }

    /// 获取未向量化的会话列表
    ///
    /// # 参数
    /// - `limit`: 限制数量
    ///
    /// # 返回
    /// 返回未向量化的会话列表
    pub fn get_non_vectorized_sessions(&self, limit: usize) -> Result<Vec<Session>> {
        self.with_conn_inner(|conn| {
            let mut stmt = conn.prepare(
                &format!(
                    "SELECT s.id, s.session_id, s.project_path, s.project_name, s.file_path,
                            s.rating, s.tags, s.is_archived, s.is_active, s.created_at, s.updated_at
                     FROM sessions s
                     LEFT JOIN session_embeddings e ON s.session_id = e.session_id
                     WHERE e.session_id IS NULL
                     ORDER BY s.created_at DESC
                     LIMIT {}",
                    limit
                ),
            )?;

            let sessions = stmt
                .query_map([], |row| {
                    Ok(Session {
                        id: Some(row.get(0)?),
                        session_id: row.get(1)?,
                        project_path: row.get(2)?,
                        project_name: row.get(3)?,
                        file_path: row.get(4)?,
                        rating: row.get(5)?,
                        tags: row.get(6)?,
                        is_archived: row.get(7)?,
                        is_active: row.get(8)?,
                        created_at: row.get(9)?,
                        updated_at: row.get(10)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| anyhow::anyhow!("查询失败: {}", e))?;

            Ok(sessions)
        })
    }
}

/// 计算余弦相似度
///
/// # 参数
/// - `a`: 向量 A
/// - `b`: 向量 B
///
/// # 返回
/// 返回余弦相似度（0.0-1.0，1.0 表示完全相同）
///
/// # 公式
/// cosine_sim = (A · B) / (||A|| * ||B||)
fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    (dot_product / (norm_a * norm_b)) as f64
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![-1.0, -2.0, -3.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_different_length() {
        let a = vec![1.0, 2.0];
        let b = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&a, &b);
        assert_eq!(sim, 0.0);
    }
}
