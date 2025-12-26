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
const CURRENT_DB_VERSION: i32 = 1;

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
fn run_migrations(conn: &mut Connection) -> Result<()> {
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
fn migrate_v1(conn: &mut Connection) -> Result<()> {
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

/// 获取数据库连接（用于运行时）
///
/// 注意: 每个线程应该有自己的连接
pub fn get_connection() -> Result<Connection> {
    let db_path = get_db_path()?;
    Connection::open(&db_path).map_err(Into::into)
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
        migrate_v1(&mut conn).unwrap();

        // 验证表已创建
        let table_exists: i32 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='api_providers'",
            [],
            |row| row.get(0),
        ).unwrap();

        assert_eq!(table_exists, 1);
    }
}
