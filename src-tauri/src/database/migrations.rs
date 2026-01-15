//! 数据库迁移和表初始化
//!
//! 管理 SQLite 数据库的版本和表结构

use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::PathBuf;

/// 数据库文件名
const DB_NAME: &str = "prism_forge.db";

/// 获取数据库文件路径
///
/// 存储位置：
/// - Windows: %USERPROFILE%/.prism-forge/prism_forge.db
/// - macOS: ~/.prism-forge/prism_forge.db
/// - Linux: ~/.prism-forge/prism_forge.db
pub fn get_db_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取用户目录"))?;
    let app_dir = home_dir.join(".prism-forge");

    // 确保目录存在
    std::fs::create_dir_all(&app_dir)?;

    Ok(app_dir.join(DB_NAME))
}

/// 数据库版本号
///
/// 每次修改表结构时递增此版本号
const CURRENT_DB_VERSION: i32 = 13;

/// 初始化数据库
///
/// 创建所有必要的表，并设置版本号
pub fn initialize_database() -> Result<Connection> {
    let db_path = get_db_path()?;
    let mut conn = Connection::open(&db_path)?;

    // 启用外键约束
    conn.execute("PRAGMA foreign_keys = ON;", [])?;

    // 执行迁移
    run_migrations(&mut conn)?;

    Ok(conn)
}

/// 运行数据库迁移
///
/// 根据版本号执行相应的迁移脚本
pub fn run_migrations(conn: &mut Connection) -> Result<()> {
    // 创建版本管理表（如果不存在）
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );",
        [],
    )?;

    // 获取当前数据库版本
    let current_version: i32 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;

    // 执行需要的迁移
    for version in (current_version + 1)..=CURRENT_DB_VERSION {
        match version {
            1 => migrate_v1(conn)?,
            2 => migrate_v2(conn)?,
            3 => migrate_v3(conn)?,
            4 => migrate_v4(conn)?,
            5 => migrate_v5(conn)?,
            6 => migrate_v6(conn)?,
            7 => migrate_v7(conn)?,
            8 => migrate_v8(conn)?,
            9 => migrate_v9(conn)?,
            10 => migrate_v10(conn)?,
            11 => migrate_v11(conn)?,
            12 => migrate_v12(conn)?,
            13 => migrate_v13(conn)?,
            _ => anyhow::bail!("未知的数据库版本: {}", version),
        }

        // 记录迁移
        let applied_at = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
            params![version, applied_at],
        )?;
    }

    Ok(())
}

/// 迁移到版本 1: 创建 api_providers 表
#[cfg(test)]
pub fn migrate_v1(conn: &mut Connection) -> Result<()> {
    migrate_v1_impl(conn)
}

#[cfg(not(test))]
fn migrate_v1(conn: &mut Connection) -> Result<()> {
    migrate_v1_impl(conn)
}

fn migrate_v1_impl(conn: &mut Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS api_providers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            provider_type TEXT NOT NULL,
            name TEXT NOT NULL,
            base_url TEXT NOT NULL,
            api_key_ref TEXT,
            config_json TEXT,
            is_active INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );",
        [],
    )?;

    // 创建索引: 查找活跃提供商
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_api_providers_is_active
         ON api_providers(is_active);",
        [],
    )?;

    // 创建索引: 按类型查找
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_api_providers_provider_type
         ON api_providers(provider_type);",
        [],
    )?;

    // 触发器: 确保 is_active 唯一性
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS ensure_single_active_provider
         BEFORE UPDATE OF is_active ON api_providers
         WHEN NEW.is_active = 1
         BEGIN
             UPDATE api_providers SET is_active = 0 WHERE is_active = 1 AND id != NEW.id;
         END;",
        [],
    )?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS ensure_single_active_provider_insert
         BEFORE INSERT ON api_providers
         WHEN NEW.is_active = 1
         BEGIN
             UPDATE api_providers SET is_active = 0 WHERE is_active = 1;
         END;",
        [],
    )?;

    Ok(())
}

/// 迁移到版本 2: 添加 model 列到 api_providers 表
#[cfg(test)]
pub fn migrate_v2(conn: &mut Connection) -> Result<()> {
    migrate_v2_impl(conn)
}

#[cfg(not(test))]
fn migrate_v2(conn: &mut Connection) -> Result<()> {
    migrate_v2_impl(conn)
}

fn migrate_v2_impl(conn: &mut Connection) -> Result<()> {
    conn.execute(
        "ALTER TABLE api_providers ADD COLUMN model TEXT;",
        [],
    )?;
    Ok(())
}

/// 迁移到版本 3: 添加 temperature 和 max_tokens 列
#[cfg(test)]
pub fn migrate_v3(conn: &mut Connection) -> Result<()> {
    migrate_v3_impl(conn)
}

#[cfg(not(test))]
fn migrate_v3(conn: &mut Connection) -> Result<()> {
    migrate_v3_impl(conn)
}

fn migrate_v3_impl(conn: &mut Connection) -> Result<()> {
    // 添加 temperature 列（默认值 0.7）
    conn.execute(
        "ALTER TABLE api_providers ADD COLUMN temperature REAL DEFAULT 0.7;",
        [],
    )?;

    // 添加 max_tokens 列（默认值 2000）
    conn.execute(
        "ALTER TABLE api_providers ADD COLUMN max_tokens INTEGER DEFAULT 2000;",
        [],
    )?;

    Ok(())
}

/// 迁移到版本 4: 创建 sessions 表
#[cfg(test)]
pub fn migrate_v4(conn: &mut Connection) -> Result<()> {
    migrate_v4_impl(conn)
}

#[cfg(not(test))]
fn migrate_v4(conn: &mut Connection) -> Result<()> {
    migrate_v4_impl(conn)
}

fn migrate_v4_impl(conn: &mut Connection) -> Result<()> {
    // 创建 sessions 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT NOT NULL UNIQUE,
            project_path TEXT NOT NULL,
            project_name TEXT NOT NULL,
            file_path TEXT NOT NULL,
            rating INTEGER CHECK (rating IN (1, 2, 3, 4, 5) OR rating IS NULL),
            tags TEXT DEFAULT '[]',
            is_archived INTEGER NOT NULL DEFAULT 0,
            is_active INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );",
        [],
    )?;

    // 索引: 按 session_id 快速查找
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sessions_session_id
            ON sessions(session_id);",
        [],
    )?;

    // 索引: 按项目路径分组查询
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sessions_project_path
            ON sessions(project_path);",
        [],
    )?;

    // 索引: 活跃会话查询
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sessions_is_active
            ON sessions(is_active);",
        [],
    )?;

    // 索引: 归档状态查询
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sessions_is_archived
            ON sessions(is_archived);",
        [],
    )?;

    // 索引: 评分排序查询
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sessions_rating
            ON sessions(rating) WHERE rating IS NOT NULL;",
        [],
    )?;

    // 触发器: 确保 is_active 唯一性 (UPDATE)
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS ensure_single_active_session
            BEFORE UPDATE OF is_active ON sessions
            WHEN NEW.is_active = 1
            BEGIN
                UPDATE sessions SET is_active = 0 WHERE is_active = 1 AND id != NEW.id;
            END;",
        [],
    )?;

    // 触发器: 确保 is_active 唯一性 (INSERT)
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS ensure_single_active_session_insert
            BEFORE INSERT ON sessions
            WHEN NEW.is_active = 1
            BEGIN
                UPDATE sessions SET is_active = 0 WHERE is_active = 1;
            END;",
        [],
    )?;

    Ok(())
}

/// 迁移到版本 5: 创建 messages 表
#[cfg(test)]
pub fn migrate_v5(conn: &mut Connection) -> Result<()> {
    migrate_v5_impl(conn)
}

#[cfg(not(test))]
fn migrate_v5(conn: &mut Connection) -> Result<()> {
    migrate_v5_impl(conn)
}

fn migrate_v5_impl(conn: &mut Connection) -> Result<()> {
    // 创建 messages 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT NOT NULL,
            uuid TEXT NOT NULL,
            parent_uuid TEXT,
            type TEXT NOT NULL CHECK (type IN ('user', 'assistant', 'tool_use', 'thinking')),
            timestamp TEXT NOT NULL,
            offset INTEGER NOT NULL,
            length INTEGER NOT NULL,
            summary TEXT,
            parent_idx INTEGER,
            created_at TEXT NOT NULL,

            FOREIGN KEY (session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
        );",
        [],
    )?;

    // 索引: 按会话查询所有消息
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_messages_session_id
            ON messages(session_id);",
        [],
    )?;

    // 索引: 按 UUID 查找单条消息
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_messages_uuid
            ON messages(uuid);",
        [],
    )?;

    // 索引: 按父 UUID 查找子消息
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_messages_parent_uuid
            ON messages(parent_uuid) WHERE parent_uuid IS NOT NULL;",
        [],
    )?;

    // 索引: 按类型过滤
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_messages_type
            ON messages(type);",
        [],
    )?;

    // 复合索引: 按会话+时间戳排序
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_messages_session_timestamp
            ON messages(session_id, timestamp);",
        [],
    )?;

    Ok(())
}

/// 迁移到版本 6: 创建 message_embeddings 虚拟表
#[cfg(test)]
pub fn migrate_v6(conn: &mut Connection) -> Result<()> {
    migrate_v6_impl(conn)
}

#[cfg(not(test))]
fn migrate_v6(conn: &mut Connection) -> Result<()> {
    migrate_v6_impl(conn)
}

pub fn migrate_v6_impl(conn: &mut Connection) -> Result<()> {
    // 创建 session_embeddings 表（用于存储会话向量）
    // 使用 JSON 序列化存储向量，支持可变维度
    conn.execute(
        "CREATE TABLE IF NOT EXISTS session_embeddings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT NOT NULL,
            embedding JSON NOT NULL,
            summary TEXT NOT NULL,
            dimension INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,

            FOREIGN KEY (session_id) REFERENCES sessions(session_id) ON DELETE CASCADE,
            UNIQUE(session_id)
        );",
        [],
    )?;

    // 索引: 按会话 ID 查找
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_session_embeddings_session_id
            ON session_embeddings(session_id);",
        [],
    )?;

    // 索引: 按创建时间排序
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_session_embeddings_created_at
            ON session_embeddings(created_at);",
        [],
    )?;

    // 注释：保留 message_embedding_map 表（用于未来的消息级别向量）
    // 当前版本先实现会话级别的向量搜索

    // 关联表: 存储 message_id 到 vec0 行 ID 的映射
    conn.execute(
        "CREATE TABLE IF NOT EXISTS message_embedding_map (
            message_id INTEGER PRIMARY KEY,
            vec_row_id INTEGER NOT NULL,
            created_at TEXT NOT NULL,

            FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
        );",
        [],
    )?;

    // 索引: 快速查找消息对应的向量
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_message_embedding_map_vec_row_id
            ON message_embedding_map(vec_row_id);",
        [],
    )?;

    Ok(())
}

/// 迁移到版本 7: 创建 saved_prompts 表
#[cfg(test)]
pub fn migrate_v7(conn: &mut Connection) -> Result<()> {
    migrate_v7_impl(conn)
}

#[cfg(not(test))]
fn migrate_v7(conn: &mut Connection) -> Result<()> {
    migrate_v7_impl(conn)
}

fn migrate_v7_impl(conn: &mut Connection) -> Result<()> {
    // 创建 saved_prompts 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS saved_prompts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT,
            category TEXT NOT NULL CHECK (category IN ('next_goals', 'ai_analysis', 'user_saved')),
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            rating INTEGER CHECK (rating IN (1, 2, 3, 4, 5) OR rating IS NULL),
            usage_count INTEGER NOT NULL DEFAULT 0,
            tokens INTEGER,
            created_at TEXT NOT NULL,

            FOREIGN KEY (session_id) REFERENCES sessions(session_id) ON DELETE SET NULL
        );",
        [],
    )?;

    // 索引: 按分类查询
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_saved_prompts_category
            ON saved_prompts(category);",
        [],
    )?;

    // 索引: 按会话查询
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_saved_prompts_session_id
            ON saved_prompts(session_id) WHERE session_id IS NOT NULL;",
        [],
    )?;

    // 索引: 按评分排序
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_saved_prompts_rating
            ON saved_prompts(rating) WHERE rating IS NOT NULL;",
        [],
    )?;

    // 索引: 按使用次数排序
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_saved_prompts_usage_count
            ON saved_prompts(usage_count);",
        [],
    )?;

    Ok(())
}

/// 迁移到版本 8: 创建 meta_templates 表
#[cfg(test)]
pub fn migrate_v8(conn: &mut Connection) -> Result<()> {
    migrate_v8_impl(conn)
}

#[cfg(not(test))]
fn migrate_v8(conn: &mut Connection) -> Result<()> {
    migrate_v8_impl(conn)
}

fn migrate_v8_impl(conn: &mut Connection) -> Result<()> {
    // 创建 meta_templates 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS meta_templates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            key TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            content TEXT NOT NULL,
            description TEXT,
            is_active INTEGER NOT NULL DEFAULT 1,
            updated_at TEXT NOT NULL
        );",
        [],
    )?;

    // 索引: 按 key 快速查找
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_meta_templates_key
            ON meta_templates(key);",
        [],
    )?;

    // 索引: 查询启用的模板
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_meta_templates_is_active
            ON meta_templates(is_active);",
        [],
    )?;

    Ok(())
}

/// 迁移到版本 9: 创建 settings 表
///
/// # 功能
/// - 创建全局配置表
/// - active_threshold: 活跃会话判断时间阈值（秒），默认 86400（24小时）
#[cfg(test)]
pub fn migrate_v9(conn: &mut Connection) -> Result<()> {
    migrate_v9_impl(conn)
}

#[cfg(not(test))]
fn migrate_v9(conn: &mut Connection) -> Result<()> {
    migrate_v9_impl(conn)
}

fn migrate_v9_impl(conn: &mut Connection) -> Result<()> {
    // 创建 settings 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            id INTEGER PRIMARY KEY,
            active_threshold INTEGER NOT NULL DEFAULT 86400
        );",
        [],
    )?;

    // 插入默认配置（如果不存在）
    conn.execute(
        "INSERT OR IGNORE INTO settings (id, active_threshold) VALUES (1, 86400);",
        [],
    )?;

    Ok(())
}

/// 迁移到版本 10: 创建 monitored_directories 表
///
/// # 功能
/// - 存储用户手动添加的监控目录
/// - 支持添加、删除和查询监控目录
#[cfg(test)]
pub fn migrate_v10(conn: &mut Connection) -> Result<()> {
    migrate_v10_impl(conn)
}

#[cfg(not(test))]
fn migrate_v10(conn: &mut Connection) -> Result<()> {
    migrate_v10_impl(conn)
}

fn migrate_v10_impl(conn: &mut Connection) -> Result<()> {
    // 创建 monitored_directories 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS monitored_directories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );",
        [],
    )?;

    // 索引: 按路径快速查找
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_monitored_directories_path
            ON monitored_directories(path);",
        [],
    )?;

    // 索引: 查询启用的目录
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_monitored_directories_is_active
            ON monitored_directories(is_active);",
        [],
    )?;

    Ok(())
}

/// 迁移到版本 11: 添加向量功能配置字段
///
/// # 功能
/// - 向 settings 表添加向量数据库相关配置
/// - vector_search_enabled: 是否启用向量搜索功能
/// - embedding_provider: Embedding 提供商（openai/fastembed）
/// - embedding_model: Embedding 模型名称（默认使用对话分析的相同模型）
#[cfg(test)]
pub fn migrate_v11(conn: &mut Connection) -> Result<()> {
    migrate_v11_impl(conn)
}

#[cfg(not(test))]
fn migrate_v11(conn: &mut Connection) -> Result<()> {
    migrate_v11_impl(conn)
}

pub fn migrate_v11_impl(conn: &mut Connection) -> Result<()> {
    // 添加向量搜索启用字段（默认禁用）
    conn.execute(
        "ALTER TABLE settings ADD COLUMN vector_search_enabled INTEGER NOT NULL DEFAULT 0;",
        [],
    )?;

    // 添加 embedding 提供商字段（默认 openai）
    conn.execute(
        "ALTER TABLE settings ADD COLUMN embedding_provider TEXT NOT NULL DEFAULT 'openai';",
        [],
    )?;

    // 添加 embedding 模型字段（默认与对话分析一致）
    conn.execute(
        "ALTER TABLE settings ADD COLUMN embedding_model TEXT NOT NULL DEFAULT 'text-embedding-3-small';",
        [],
    )?;

    // 添加向量同步批次大小（默认 10）
    conn.execute(
        "ALTER TABLE settings ADD COLUMN embedding_batch_size INTEGER NOT NULL DEFAULT 10;",
        [],
    )?;

    Ok(())
}

/// 迁移到版本 12: 添加提供商别名支持
///
/// # 功能
/// - 为 api_providers 表添加 aliases 字段
/// - 支持通过别名快速查找提供商
#[cfg(test)]
pub fn migrate_v12(conn: &mut Connection) -> Result<()> {
    migrate_v12_impl(conn)
}

#[cfg(not(test))]
fn migrate_v12(conn: &mut Connection) -> Result<()> {
    migrate_v12_impl(conn)
}

pub fn migrate_v12_impl(conn: &mut Connection) -> Result<()> {
    // 添加别名字段（JSON 数组格式，存储别名列表）
    conn.execute(
        "ALTER TABLE api_providers ADD COLUMN aliases TEXT DEFAULT '[]';",
        [],
    )?;

    Ok(())
}

/// 迁移到版本 13: 创建 view_level_preferences 表
#[cfg(test)]
pub fn migrate_v13(conn: &mut Connection) -> Result<()> {
    migrate_v13_impl(conn)
}

#[cfg(not(test))]
pub fn migrate_v13(conn: &mut Connection) -> Result<()> {
    migrate_v13_impl(conn)
}

pub fn migrate_v13_impl(conn: &mut Connection) -> Result<()> {
    // 创建视图等级偏好表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS view_level_preferences (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT NOT NULL UNIQUE,
            view_level TEXT NOT NULL CHECK (
                view_level IN ('full', 'conversation', 'qa_pairs', 'assistant_only', 'user_only')
            ),
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,

            FOREIGN KEY (session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
        );",
        [],
    )?;

    // 创建索引以加速查询
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_view_level_preferences_session
        ON view_level_preferences(session_id);",
        [],
    )?;

    Ok(())
}

/// 获取数据库连接（用于运行时）
///
/// 注意: 每个线程应该有自己的连接
/// 此函数会自动确保数据库表已创建
pub fn get_connection() -> Result<Connection> {
    let db_path = get_db_path()?;
    let mut conn = Connection::open(&db_path)?;
    
    // 启用外键约束
    conn.execute("PRAGMA foreign_keys = ON;", [])?;
    
    // 确保迁移已执行
    run_migrations(&mut conn)?;
    
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_db_path() {
        let path = get_db_path().unwrap();
        assert!(path.ends_with("prism_forge.db"));
    }

    #[test]
    fn test_initialize_database() {
        // 使用内存数据库进行测试
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();

        // 执行迁移
        migrate_v1_impl(&mut conn).unwrap();

        // 验证表已创建
        let table_exists: i32 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='api_providers'",
            [],
            |row| row.get(0),
        ).unwrap();

        assert_eq!(table_exists, 1);
    }

    #[test]
    fn test_migrate_v2_adds_model_column() {
        // 使用内存数据库进行测试
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();

        // 先执行 V1 迁移
        migrate_v1_impl(&mut conn).unwrap();

        // 执行 V2 迁移
        migrate_v2_impl(&mut conn).unwrap();

        // 验证 model 列已添加
        let column_exists: i32 = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('api_providers') WHERE name='model'",
            [],
            |row| row.get(0),
        ).unwrap();

        assert_eq!(column_exists, 1);
    }
}
