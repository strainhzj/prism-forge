//! 向量数据库集成测试
//!
//! 测试向量生成、存储和搜索的完整流程

use std::sync::Arc;
use std::sync::Mutex;

// 注意：这些测试需要实际的 API Key
// 在 CI/CD 环境中应该跳过或使用 mock

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// 测试向量仓库基本操作
    #[test]
    fn test_vector_repository_crud() {
        // 创建内存数据库
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();

        // 执行迁移
        crate::database::migrations::migrate_v6_impl(&mut conn).unwrap();

        let shared_conn = Arc::new(Mutex::new(conn));
        let repo = crate::database::vector_repository::VectorRepository::with_conn(shared_conn);

        // 测试：保存向量
        let embedding = crate::database::models::SessionEmbedding::new(
            "test-session-1".to_string(),
            vec![0.1, 0.2, 0.3, 0.4, 0.5],
            "测试会话摘要".to_string(),
        );

        let result = repo.upsert_session_embedding(embedding);
        assert!(result.is_ok(), "保存向量失败: {:?}", result.err());

        // 测试：查询向量
        let retrieved = repo.get_session_embedding("test-session-1");
        assert!(retrieved.is_ok(), "查询向量失败: {:?}", retrieved.err());
        let retrieved = retrieved.unwrap();
        assert!(retrieved.is_some(), "未找到保存的向量");
        assert_eq!(retrieved.unwrap().session_id, "test-session-1");

        // 测试：删除向量
        let deleted = repo.delete_session_embedding("test-session-1");
        assert!(deleted.is_ok(), "删除向量失败: {:?}", deleted.err());

        let after_delete = repo.get_session_embedding("test-session-1").unwrap();
        assert!(after_delete.is_none(), "删除后仍能查到向量");

        println!("✅ 向量仓库 CRUD 测试通过");
    }

    /// 测试余弦相似度计算
    #[test]
    fn test_cosine_similarity() {
        // 相同向量：相似度 = 1.0
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        let sim = crate::database::vector_repository::cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 0.001, "相同向量相似度应该为 1.0，实际: {}", sim);

        // 正交向量：相似度 = 0.0
        let c = vec![1.0, 0.0, 0.0];
        let d = vec![0.0, 1.0, 0.0];
        let sim2 = crate::database::vector_repository::cosine_similarity(&c, &d);
        assert!((sim2 - 0.0).abs() < 0.001, "正交向量相似度应该为 0.0，实际: {}", sim2);

        // 相反向量：相似度 = -1.0
        let e = vec![1.0, 2.0, 3.0];
        let f = vec![-1.0, -2.0, -3.0];
        let sim3 = crate::database::vector_repository::cosine_similarity(&e, &f);
        assert!((sim3 - (-1.0)).abs() < 0.001, "相反向量相似度应该为 -1.0，实际: {}", sim3);

        println!("✅ 余弦相似度计算测试通过");
    }

    /// 测试向量搜索
    #[test]
    fn test_vector_search() {
        // 创建内存数据库
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();
        crate::database::migrations::migrate_v6_impl(&mut conn).unwrap();

        let shared_conn = Arc::new(Mutex::new(conn));
        let repo = crate::database::vector_repository::VectorRepository::with_conn(shared_conn);

        // 插入测试向量
        let embeddings = vec![
            crate::database::models::SessionEmbedding::new(
                "session-1".to_string(),
                vec![1.0, 0.0, 0.0], // 方向 [1,0,0]
                "数据库优化".to_string(),
            ),
            crate::database::models::SessionEmbedding::new(
                "session-2".to_string(),
                vec![0.0, 1.0, 0.0], // 方向 [0,1,0]
                "前端开发".to_string(),
            ),
            crate::database::models::SessionEmbedding::new(
                "session-3".to_string(),
                vec![0.9, 0.1, 0.0], // 接近 [1,0,0]
                "SQL 性能调优".to_string(),
            ),
        ];

        for emb in embeddings {
            repo.upsert_session_embedding(emb).unwrap();
        }

        // 测试：搜索与 [1,0,0] 相似的向量
        let query = vec![1.0, 0.0, 0.0];
        let results = repo.vector_search_sessions(&query, 10, 0.0).unwrap();

        assert_eq!(results.len(), 3, "应该返回 3 个结果");

        // 第一个结果应该是 session-1（完全相同，相似度 1.0）
        assert_eq!(results[0].session.session_id, "session-1");
        assert!((results[0].similarity_score - 1.0).abs() < 0.001);

        // 第二个结果应该是 session-3（相似，相似度 ~0.995）
        assert_eq!(results[1].session.session_id, "session-3");
        assert!(results[1].similarity_score > 0.9);

        // 第三个结果应该是 session-2（正交，相似度 0.0）
        assert_eq!(results[2].session.session_id, "session-2");
        assert!((results[2].similarity_score - 0.0).abs() < 0.001);

        println!("✅ 向量搜索测试通过");
        println!("   - session-1 (相同): {:.4}", results[0].similarity_score);
        println!("   - session-3 (相似): {:.4}", results[1].similarity_score);
        println!("   - session-2 (正交): {:.4}", results[2].similarity_score);
    }

    /// 测试 Settings 模型
    #[test]
    fn test_settings_model() {
        use crate::database::models::Settings;

        // 测试默认设置
        let settings = Settings::default();
        assert!(!settings.vector_search_enabled);
        assert_eq!(settings.embedding_provider, "openai");
        assert_eq!(settings.embedding_model, "text-embedding-3-small");
        assert_eq!(settings.embedding_batch_size, 10);

        // 测试向量维度获取
        let dim = settings.get_embedding_dimension();
        assert_eq!(dim, 1536, "text-embedding-3-small 应该是 1536 维");

        // 测试验证
        assert!(settings.validate().is_ok(), "默认设置验证失败");

        // 测试无效配置
        let mut invalid = Settings::default();
        invalid.embedding_batch_size = 0;
        assert!(invalid.validate().is_err(), "batch_size=0 应该验证失败");

        println!("✅ Settings 模型测试通过");
    }

    /// 测试 EmbeddingProvider 枚举
    #[test]
    fn test_embedding_provider() {
        use crate::embedding::EmbeddingProvider;

        // 测试解析
        let openai = EmbeddingProvider::from_str("openai").unwrap();
        assert_eq!(openai, EmbeddingProvider::OpenAI);
        assert_eq!(openai.as_str(), "openai");

        let fastembed = EmbeddingProvider::from_str("fastembed").unwrap();
        assert_eq!(fastembed, EmbeddingProvider::FastEmbed);
        assert_eq!(fastembed.as_str(), "fastembed");

        // 测试无效提供商
        let invalid = EmbeddingProvider::from_str("invalid");
        assert!(invalid.is_err(), "应该拒绝无效的提供商");

        println!("✅ EmbeddingProvider 枚举测试通过");
    }

    /// 测试 SessionEmbedding 模型
    #[test]
    fn test_session_embedding_model() {
        use crate::database::models::SessionEmbedding;

        // 测试创建
        let embedding = SessionEmbedding::new(
            "test-session".to_string(),
            vec![0.1, 0.2, 0.3],
            "测试摘要".to_string(),
        );

        assert_eq!(embedding.session_id, "test-session");
        assert_eq!(embedding.dimension, 3);
        assert_eq!(embedding.summary, "测试摘要");

        // 测试向量序列化/反序列化
        let vec = embedding.get_embedding();
        assert!(vec.is_ok(), "向量解析失败: {:?}", vec.err());
        assert_eq!(vec.unwrap(), vec![0.1, 0.2, 0.3]);

        println!("✅ SessionEmbedding 模型测试通过");
    }
}

/// 性能测试
#[cfg(test)]
mod performance_tests {
    use super::*;

    /// 测试大规模向量搜索性能
    #[test]
    #[ignore] // 默认跳过，手动运行：cargo test test_vector_search_performance -- --ignored
    fn test_vector_search_performance() {
        use std::time::Instant;

        // 创建内存数据库
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();
        crate::database::migrations::migrate_v6_impl(&mut conn).unwrap();

        let shared_conn = Arc::new(Mutex::new(conn));
        let repo = crate::database::vector_repository::VectorRepository::with_conn(shared_conn);

        // 生成 100 个随机向量
        let count = 100;
        println!("正在生成 {} 个测试向量...", count);

        for i in 0..count {
            let vector: Vec<f32> = (0..1536)
                .map(|_| rand::random::<f32>())
                .collect();

            let embedding = crate::database::models::SessionEmbedding::new(
                format!("session-{}", i),
                vector,
                format!("测试会话 {}", i),
            );

            repo.upsert_session_embedding(embedding).unwrap();
        }

        // 测试搜索性能
        let query: Vec<f32> = (0..1536).map(|_| rand::random()).collect();

        let start = Instant::now();
        let results = repo.vector_search_sessions(&query, 10, 0.0).unwrap();
        let duration = start.elapsed();

        println!("✅ 向量搜索性能测试通过");
        println!("   - 会话数: {}", count);
        println!("   - 返回结果: {}", results.len());
        println!("   - 耗时: {:?}", duration);

        // 断言性能要求
        assert!(
            duration.as_millis() < 500,
            "搜索耗时超过 500ms: {:?}",
            duration
        );
    }
}
