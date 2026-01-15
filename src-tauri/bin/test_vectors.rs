//! 向量功能手动测试
//!
//! 编译运行：cargo build --bin test_vectors && ./target/debug/test_vectors

use prism_forge::database::{
    models::{SessionEmbedding, Settings},
    vector_repository::VectorRepository,
};
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 开始向量数据库功能测试\n");

    // 创建内存数据库
    let mut conn = rusqlite::Connection::open_in_memory()?;
    conn.execute("PRAGMA foreign_keys = ON;", [])?;

    // 执行迁移
    println!("📦 执行数据库迁移...");
    prism_forge::database::migrations::run_migrations(&mut conn)?;
    println!("✅ 迁移完成\n");

    let shared_conn = Arc::new(Mutex::new(conn));
    let repo = VectorRepository::with_conn(shared_conn);

    // 测试 1: CRUD 操作
    println!("🧪 测试 1: 向量 CRUD 操作");
    test_crud(&repo)?;
    println!();

    // 测试 2: 余弦相似度
    println!("🧪 测试 2: 余弦相似度计算");
    test_cosine_similarity()?;
    println!();

    // 测试 3: 向量搜索
    println!("🧪 测试 3: 向量搜索");
    test_vector_search(&repo)?;
    println!();

    // 测试 4: Settings 模型
    println!("🧪 测试 4: Settings 模型");
    test_settings_model()?;
    println!();

    // 测试 5: EmbeddingProvider
    println!("🧪 测试 5: EmbeddingProvider");
    test_embedding_provider()?;
    println!();

    println!("🎉 所有测试通过！");
    Ok(())
}

fn test_crud(repo: &VectorRepository) -> Result<(), Box<dyn std::error::Error>> {
    println!("   ➕ 创建测试会话...");
    // 先创建 session（满足外键约束）
    use prism_forge::database::repository::SessionRepository;

    let session_repo = SessionRepository::with_conn(repo.conn.clone());
    session_repo.upsert_session(
        "test-session-1",
        "/test/path",
        "test",
        "/test/path/session.jsonl",
        true,
    )?;

    println!("   ➕ 保存向量...");
    let embedding = SessionEmbedding::new(
        "test-session-1".to_string(),
        vec![0.1, 0.2, 0.3, 0.4, 0.5],
        "测试会话摘要".to_string(),
    );
    repo.upsert_session_embedding(embedding)?;

    println!("   🔍 查询向量...");
    let retrieved = repo.get_session_embedding("test-session-1")?;
    assert!(retrieved.is_some(), "未找到保存的向量");
    assert_eq!(retrieved.unwrap().session_id, "test-session-1");
    println!("   ✅ 查询成功");

    println!("   🗑️  删除向量...");
    repo.delete_session_embedding("test-session-1")?;

    let after_delete = repo.get_session_embedding("test-session-1")?;
    assert!(after_delete.is_none(), "删除后仍能查到向量");
    println!("   ✅ 删除成功");

    Ok(())
}

fn test_cosine_similarity() -> Result<(), Box<dyn std::error::Error>> {
    println!("   📏 计算余弦相似度...");

    // 相同向量：相似度 = 1.0
    let a = vec![1.0, 2.0, 3.0];
    let b = vec![1.0, 2.0, 3.0];
    let sim1 = cosine_similarity(&a, &b);
    assert!((sim1 - 1.0).abs() < 0.001);
    println!("   ✅ 相同向量: {:.4} (期望 1.0)", sim1);

    // 正交向量：相似度 = 0.0
    let c = vec![1.0, 0.0, 0.0];
    let d = vec![0.0, 1.0, 0.0];
    let sim2 = cosine_similarity(&c, &d);
    assert!((sim2 - 0.0).abs() < 0.001);
    println!("   ✅ 正交向量: {:.4} (期望 0.0)", sim2);

    // 相反向量：相似度 = -1.0
    let e = vec![1.0, 2.0, 3.0];
    let f = vec![-1.0, -2.0, -3.0];
    let sim3 = cosine_similarity(&e, &f);
    assert!((sim3 - (-1.0)).abs() < 0.001);
    println!("   ✅ 相反向量: {:.4} (期望 -1.0)", sim3);

    Ok(())
}

fn test_vector_search(repo: &VectorRepository) -> Result<(), Box<dyn std::error::Error>> {
    println!("   📊 插入测试向量...");

    use prism_forge::database::repository::SessionRepository;

    let session_repo = SessionRepository::with_conn(repo.conn.clone());

    // 先创建 session 记录
    for i in 1..=3 {
        session_repo.upsert_session(
            &format!("session-{}", i),
            "/test/path",
            "test",
            &format!("/test/path/session-{}.jsonl", i),
            true,
        )?;
    }

    // 插入测试向量
    let embeddings = vec![
        SessionEmbedding::new(
            "session-1".to_string(),
            vec![1.0, 0.0, 0.0], // 方向 [1,0,0]
            "数据库优化".to_string(),
        ),
        SessionEmbedding::new(
            "session-2".to_string(),
            vec![0.0, 1.0, 0.0], // 方向 [0,1,0]
            "前端开发".to_string(),
        ),
        SessionEmbedding::new(
            "session-3".to_string(),
            vec![0.9, 0.1, 0.0], // 接近 [1,0,0]
            "SQL 性能调优".to_string(),
        ),
    ];

    for emb in embeddings {
        repo.upsert_session_embedding(emb)?;
    }
    println!("   ✅ 插入了 3 个向量");

    println!("   🔍 执行向量搜索...");
    let query = vec![1.0, 0.0, 0.0];
    let results = repo.vector_search_sessions(&query, 10, 0.0)?;

    assert_eq!(results.len(), 3, "应该返回 3 个结果");
    println!("   ✅ 找到 {} 个结果", results.len());

    // 验证排序
    assert_eq!(results[0].session.session_id, "session-1");
    println!("   ✅ 第 1 名: {} (相似度: {:.4})",
        results[0].session.session_id,
        results[0].similarity_score
    );

    assert_eq!(results[1].session.session_id, "session-3");
    println!("   ✅ 第 2 名: {} (相似度: {:.4})",
        results[1].session.session_id,
        results[1].similarity_score
    );

    assert_eq!(results[2].session.session_id, "session-2");
    println!("   ✅ 第 3 名: {} (相似度: {:.4})",
        results[2].session.session_id,
        results[2].similarity_score
    );

    Ok(())
}

fn test_settings_model() -> Result<(), Box<dyn std::error::Error>> {
    println!("   ⚙️  测试 Settings 模型...");

    let settings = Settings::default();
    assert!(!settings.vector_search_enabled);
    assert_eq!(settings.embedding_provider, "openai");
    assert_eq!(settings.embedding_model, "text-embedding-3-small");
    println!("   ✅ 默认设置正确");

    let dim = settings.get_embedding_dimension();
    assert_eq!(dim, 1536);
    println!("   ✅ 向量维度: {} (text-embedding-3-small)", dim);

    assert!(settings.validate().is_ok());
    println!("   ✅ 验证通过");

    Ok(())
}

fn test_embedding_provider() -> Result<(), Box<dyn std::error::Error>> {
    use prism_forge::embedding::EmbeddingProvider;

    println!("   🔧 测试 EmbeddingProvider...");

    let openai = EmbeddingProvider::from_str("openai")?;
    assert_eq!(openai, EmbeddingProvider::OpenAI);
    assert_eq!(openai.as_str(), "openai");
    println!("   ✅ OpenAI 提供商解析成功");

    let fastembed = EmbeddingProvider::from_str("fastembed")?;
    assert_eq!(fastembed, EmbeddingProvider::FastEmbed);
    println!("   ✅ FastEmbed 提供商解析成功");

    assert!(EmbeddingProvider::from_str("invalid").is_err());
    println!("   ✅ 正确拒绝无效提供商");

    Ok(())
}

/// 计算余弦相似度
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
