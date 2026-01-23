//! 提示词历史记录独立测试程序
//!
//! 编译运行: cargo run --bin test_prompt_history

use prism_forge::database::models::PromptGenerationHistory;
use prism_forge::database::repository::PromptHistoryRepository;
use rusqlite::{Connection, Result as SqliteResult};
use std::sync::{Arc, Mutex};

fn create_test_db() -> SqliteResult<Arc<Mutex<Connection>>> {
    let conn = Connection::open_in_memory()?;

    // 创建测试表
    conn.execute(
        "CREATE TABLE prompt_generation_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT,
            original_goal TEXT NOT NULL,
            enhanced_prompt TEXT NOT NULL,
            referenced_sessions TEXT,
            token_stats TEXT,
            confidence REAL,
            llm_provider TEXT,
            llm_model TEXT,
            language TEXT NOT NULL,
            created_at TEXT NOT NULL,
            is_favorite INTEGER NOT NULL DEFAULT 0
        )",
        [],
    )?;

    Ok(Arc::new(Mutex::new(conn)))
}

fn create_test_history() -> PromptGenerationHistory {
    PromptGenerationHistory {
        id: None,
        session_id: Some("test-session-uuid-12345".to_string()),
        original_goal: "实现用户登录功能".to_string(),
        enhanced_prompt: "### 任务目标\n实现用户登录功能，包含邮箱验证...".to_string(),
        referenced_sessions: Some(r#"[{"sessionId":"abc123","projectName":"TestProject","summary":"登录相关讨论","similarityScore":0.95}]"#.to_string()),
        token_stats: Some(r#"{"originalTokens":5000,"compressedTokens":1200,"savingsPercentage":76.0}"#.to_string()),
        confidence: Some(0.85),
        llm_provider: Some("OpenAI".to_string()),
        llm_model: Some("gpt-4o-mini".to_string()),
        language: "zh".to_string(),
        created_at: "2025-01-23T12:34:56Z".to_string(),
        is_favorite: false,
    }
}

fn main() {
    println!("===================================");
    println!("提示词历史记录功能测试");
    println!("===================================\n");

    // 测试 1: 创建记录
    println!("🧪 测试 1: 创建历史记录");
    let conn = create_test_db().expect("❌ 创建数据库失败");
    let mut repo = PromptHistoryRepository::with_conn(conn);

    let history = create_test_history();
    match repo.create_history(history) {
        Ok(saved) => {
            println!("✅ 创建成功");
            println!("   - ID: {:?}", saved.id);
            println!("   - 目标: {}", saved.original_goal);
            println!("   - 语言: {}", saved.language);
        }
        Err(e) => {
            println!("❌ 创建失败: {}", e);
            return;
        }
    }
    println!();

    // 测试 2: 获取所有记录
    println!("🧪 测试 2: 获取所有记录");
    match repo.get_all_histories() {
        Ok(histories) => {
            println!("✅ 查询成功，共 {} 条记录", histories.len());
            for (i, h) in histories.iter().enumerate() {
                println!("   - 记录 {}: {} (语言: {})", i + 1, h.original_goal, h.language);
            }
        }
        Err(e) => {
            println!("❌ 查询失败: {}", e);
            return;
        }
    }
    println!();

    // 测试 3: 分页查询
    println!("🧪 测试 3: 分页查询");
    // 添加更多记录
    for i in 1..=5 {
        let mut h = create_test_history();
        h.original_goal = format!("测试目标 {}", i);
        if let Err(e) = repo.create_history(h) {
            println!("❌ 创建额外记录失败: {}", e);
            return;
        }
    }

    match repo.get_histories_paginated(0, 3) {
        Ok(page) => {
            println!("✅ 分页查询成功，第一页 {} 条记录", page.len());
            for (i, h) in page.iter().enumerate() {
                println!("   - 记录 {}: {}", i + 1, h.original_goal);
            }
        }
        Err(e) => {
            println!("❌ 分页查询失败: {}", e);
            return;
        }
    }
    println!();

    // 测试 4: 统计数量
    println!("🧪 测试 4: 统计记录数量");
    match repo.count_histories() {
        Ok(count) => {
            println!("✅ 统计成功，共 {} 条记录", count);
        }
        Err(e) => {
            println!("❌ 统计失败: {}", e);
            return;
        }
    }
    println!();

    // 测试 5: 切换收藏状态
    println!("🧪 测试 5: 切换收藏状态");
    match repo.get_all_histories() {
        Ok(histories) => {
            if let Some(first) = histories.first() {
                if let Some(id) = first.id {
                    match repo.toggle_favorite(id) {
                        Ok(is_favorite) => {
                            println!("✅ 切换收藏成功: {} -> {}", !is_favorite, is_favorite);
                        }
                        Err(e) => {
                            println!("❌ 切换收藏失败: {}", e);
                            return;
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ 获取记录失败: {}", e);
            return;
        }
    }
    println!();

    // 测试 6: 获取收藏的记录
    println!("🧪 测试 6: 获取收藏的记录");
    match repo.get_favorite_histories() {
        Ok(favorites) => {
            println!("✅ 查询成功，共 {} 条收藏记录", favorites.len());
            for (i, h) in favorites.iter().enumerate() {
                println!("   - 收藏 {}: {}", i + 1, h.original_goal);
            }
        }
        Err(e) => {
            println!("❌ 查询收藏失败: {}", e);
            return;
        }
    }
    println!();

    // 测试 7: 删除记录
    println!("🧪 测试 7: 删除记录");
    match repo.get_all_histories() {
        Ok(histories) => {
            if let Some(first) = histories.first() {
                if let Some(id) = first.id {
                    match repo.delete_history(id) {
                        Ok(deleted) => {
                            println!("✅ 删除成功，删除了 {} 行", deleted);
                        }
                        Err(e) => {
                            println!("❌ 删除失败: {}", e);
                            return;
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ 获取记录失败: {}", e);
            return;
        }
    }
    println!();

    println!("===================================");
    println!("✅ 所有测试完成!");
    println!("===================================");
}
