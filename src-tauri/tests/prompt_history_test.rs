//! 提示词生成历史记录测试
//!
//! 测试 PromptGenerationHistory 的数据库保存功能

#[cfg(test)]
mod tests {
    use prism_forge::database::models::PromptGenerationHistory;
    use prism_forge::database::repository::PromptHistoryRepository;
    use rusqlite::{Connection, Result as SqliteResult};
    use std::sync::{Arc, Mutex};

    /// 创建测试用的内存数据库
    fn create_test_db() -> SqliteResult<Arc<Mutex<Connection>>> {
        let conn = Connection::open_in_memory()?;

        // 创建测试表 - 与实际表结构一致
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

    /// 创建测试用的历史记录对象
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

    #[test]
    fn test_create_history() {
        let conn = create_test_db().expect("Failed to create test DB");
        let mut repo = PromptHistoryRepository::with_conn(conn);

        let history = create_test_history();
        let original_goal = history.original_goal.clone();

        // 执行创建
        let result = repo.create_history(history);

        assert!(result.is_ok(), "创建历史记录应成功");

        let saved = result.unwrap();
        assert!(saved.id.is_some(), "保存后应自动生成 ID");
        assert_eq!(saved.original_goal, original_goal);
        assert_eq!(saved.session_id, Some("test-session-uuid-12345".to_string()));

        println!("✅ test_create_history 通过: ID = {:?}", saved.id);
    }

    #[test]
    fn test_get_history_by_id() {
        let conn = create_test_db().expect("Failed to create test DB");
        let mut repo = PromptHistoryRepository::with_conn(conn);

        let history = create_test_history();
        let saved = repo.create_history(history).expect("创建失败");
        let id = saved.id.expect("保存后应有 ID");

        // 查询记录
        let found = repo.get_history_by_id(id);

        assert!(found.is_ok(), "查询应成功");
        let result = found.unwrap();
        assert!(result.is_some(), "应找到记录");

        let history = result.unwrap();
        assert_eq!(history.id, Some(id));
        assert_eq!(history.original_goal, "实现用户登录功能");
        assert_eq!(history.language, "zh");

        println!("✅ test_get_history_by_id 通过: 找到记录 ID = {}", id);
    }

    #[test]
    fn test_get_all_histories() {
        let conn = create_test_db().expect("Failed to create test DB");
        let mut repo = PromptHistoryRepository::with_conn(conn);

        // 创建多条记录
        let history1 = create_test_history();
        let mut history2 = create_test_history();
        history2.original_goal = "实现购物车功能".to_string();
        history2.session_id = Some("test-session-uuid-67890".to_string());

        repo.create_history(history1).expect("创建失败");
        repo.create_history(history2).expect("创建失败");

        // 获取所有记录
        let result = repo.get_all_histories();

        assert!(result.is_ok(), "获取所有记录应成功");
        let histories = result.unwrap();
        assert_eq!(histories.len(), 2, "应有 2 条记录");

        println!("✅ test_get_all_histories 通过: 共 {} 条记录", histories.len());
    }

    #[test]
    fn test_count_histories() {
        let conn = create_test_db().expect("Failed to create test DB");
        let mut repo = PromptHistoryRepository::with_conn(conn);

        // 创建前数量为 0
        let count0 = repo.count_histories().expect("统计失败");
        assert_eq!(count0, 0, "初始数量应为 0");

        // 创建 3 条记录
        for i in 0..3 {
            let mut history = create_test_history();
            history.original_goal = format!("目标 {}", i);
            repo.create_history(history).expect("创建失败");
        }

        // 数量应为 3
        let count3 = repo.count_histories().expect("统计失败");
        assert_eq!(count3, 3, "数量应为 3");

        println!("✅ test_count_histories 通过: 共 {} 条记录", count3);
    }

    #[test]
    fn test_delete_history() {
        let conn = create_test_db().expect("Failed to create test DB");
        let mut repo = PromptHistoryRepository::with_conn(conn);

        let history = create_test_history();
        let saved = repo.create_history(history).expect("创建失败");
        let id = saved.id.expect("保存后应有 ID");

        // 删除前记录存在
        let found = repo.get_history_by_id(id).unwrap();
        assert!(found.is_some(), "删除前记录应存在");

        // 执行删除
        let deleted_rows = repo.delete_history(id);

        assert!(deleted_rows.is_ok(), "删除应成功");
        assert_eq!(deleted_rows.unwrap(), 1, "应删除 1 行");

        // 删除后记录不存在
        let found = repo.get_history_by_id(id).unwrap();
        assert!(found.is_none(), "删除后记录不应存在");

        println!("✅ test_delete_history 通过: 删除记录 ID = {}", id);
    }

    #[test]
    fn test_toggle_favorite() {
        let conn = create_test_db().expect("Failed to create test DB");
        let mut repo = PromptHistoryRepository::with_conn(conn);

        let history = create_test_history();
        let saved = repo.create_history(history).expect("创建失败");
        let id = saved.id.expect("保存后应有 ID");

        // 初始状态为 false
        assert_eq!(saved.is_favorite, false);

        // 切换为 true
        let is_favorite = repo.toggle_favorite(id).expect("切换失败");
        assert_eq!(is_favorite, true);

        // 验证状态
        let retrieved = repo.get_history_by_id(id).unwrap().unwrap();
        assert_eq!(retrieved.is_favorite, true);

        // 切换回 false
        let is_favorite = repo.toggle_favorite(id).expect("切换失败");
        assert_eq!(is_favorite, false);

        println!("✅ test_toggle_favorite 通过: 切换收藏状态 ID = {}", id);
    }

    #[test]
    fn test_get_histories_paginated() {
        let conn = create_test_db().expect("Failed to create test DB");
        let mut repo = PromptHistoryRepository::with_conn(conn);

        // 创建 5 条记录
        for i in 0..5 {
            let mut history = create_test_history();
            history.original_goal = format!("目标 {}", i);
            repo.create_history(history).expect("创建失败");
        }

        // 测试第一页
        let page1 = repo.get_histories_paginated(0, 2).expect("分页查询失败");
        assert_eq!(page1.len(), 2, "第一页应有 2 条记录");

        // 测试第二页
        let page2 = repo.get_histories_paginated(2, 2).expect("分页查询失败");
        assert_eq!(page2.len(), 2, "第二页应有 2 条记录");

        // 测试第三页（不足 2 条）
        let page3 = repo.get_histories_paginated(4, 2).expect("分页查询失败");
        assert_eq!(page3.len(), 1, "第三页应有 1 条记录");

        println!("✅ test_get_histories_paginated 通过: 分页查询正常");
    }

    #[test]
    fn test_json_field_serialization() {
        let conn = create_test_db().expect("Failed to create test DB");
        let mut repo = PromptHistoryRepository::with_conn(conn);

        let history = create_test_history();
        let saved = repo.create_history(history).expect("创建失败");
        let id = saved.id.expect("保存后应有 ID");

        // 验证 referenced_sessions 可以正确解析
        let retrieved = repo.get_history_by_id(id).unwrap().unwrap();
        assert!(retrieved.referenced_sessions.is_some());

        let json_str = retrieved.referenced_sessions.unwrap();
        assert!(json_str.contains("sessionId"), "应包含 sessionId 字段");

        // 验证 token_stats 可以正确解析
        assert!(retrieved.token_stats.is_some());
        let token_json = retrieved.token_stats.unwrap();
        assert!(token_json.contains("originalTokens"), "应包含 originalTokens 字段");

        println!("✅ test_json_field_serialization 通过: JSON 字段序列化正常");
    }

    #[test]
    fn test_null_session_id() {
        let conn = create_test_db().expect("Failed to create test DB");
        let mut repo = PromptHistoryRepository::with_conn(conn);

        let mut history = create_test_history();
        history.session_id = None;  // 测试空 session_id

        let result = repo.create_history(history);
        assert!(result.is_ok(), "空 session_id 应允许保存");

        let saved = result.unwrap();
        assert!(saved.session_id.is_none());

        println!("✅ test_null_session_id 通过: 空 session_id 允许保存");
    }

    #[test]
    fn test_null_optional_fields() {
        let conn = create_test_db().expect("Failed to create test DB");
        let mut repo = PromptHistoryRepository::with_conn(conn);

        let mut history = create_test_history();
        history.referenced_sessions = None;
        history.token_stats = None;
        history.confidence = None;
        history.llm_provider = None;
        history.llm_model = None;

        let result = repo.create_history(history);
        assert!(result.is_ok(), "空可选字段应允许保存");

        let saved = result.unwrap();
        assert!(saved.referenced_sessions.is_none());
        assert!(saved.token_stats.is_none());
        assert!(saved.confidence.is_none());

        println!("✅ test_null_optional_fields 通过: 空可选字段允许保存");
    }
}
