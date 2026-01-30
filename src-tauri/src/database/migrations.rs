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
const CURRENT_DB_VERSION: i32 = 18;

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
            14 => migrate_v14(conn)?,
            15 => migrate_v15(conn)?,
            16 => migrate_v16(conn)?,
            17 => migrate_v17(conn)?,
            18 => migrate_v18(conn)?,
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

    // 创建 message_embeddings 虚拟表（使用 sqlite-vec 扩展）
    // 用于存储消息级别的向量嵌入
    // 注意：如果 sqlite-vec 扩展未加载，这里会失败，但不影响其他功能
    let vec_table_created = conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS message_embeddings USING vec0(
            embedding float[384],
            summary text
        );",
        [],
    );

    if let Err(e) = vec_table_created {
        log::warn!("创建 message_embeddings 虚拟表失败（可能 sqlite-vec 扩展未加载）: {}", e);
        // 不中断迁移，继续创建其他表
    } else {
        log::info!("message_embeddings 虚拟表创建成功");
    }

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

/// 迁移到版本 14: 创建 message_embeddings 虚拟表
///
/// # 功能
/// - 创建用于向量搜索的 message_embeddings 虚拟表（使用 sqlite-vec）
/// - 如果 sqlite-vec 扩展未加载，会记录警告但不中断迁移
#[cfg(test)]
pub fn migrate_v14(conn: &mut Connection) -> Result<()> {
    migrate_v14_impl(conn)
}

#[cfg(not(test))]
pub fn migrate_v14(conn: &mut Connection) -> Result<()> {
    migrate_v14_impl(conn)
}

pub fn migrate_v14_impl(conn: &mut Connection) -> Result<()> {
    // 创建 message_embeddings 虚拟表（使用 sqlite-vec 扩展）
    // 用于存储消息级别的向量嵌入
    let vec_table_created = conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS message_embeddings USING vec0(
            embedding float[384],
            summary text
        );",
        [],
    );

    if let Err(e) = vec_table_created {
        log::warn!("创建 message_embeddings 虚拟表失败（可能 sqlite-vec 扩展未加载）: {}", e);
    } else {
        log::info!("message_embeddings 虚拟表创建成功");
    }

    Ok(())
}

/// 迁移到版本 15: 添加 content_type 列到 messages 表
///
/// # 功能
/// - 为 messages 表添加 content_type 列
/// - 用于存储 message.content[0].type 的值（text/tool_use/tool_result/thinking）
/// - 支持更准确的问答对匹配
#[cfg(test)]
pub fn migrate_v15(conn: &mut Connection) -> Result<()> {
    migrate_v15_impl(conn)
}

#[cfg(not(test))]
fn migrate_v15(conn: &mut Connection) -> Result<()> {
    migrate_v15_impl(conn)
}

pub fn migrate_v15_impl(conn: &mut Connection) -> Result<()> {
    // 检查列是否已存在
    let column_exists: i32 = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('messages') WHERE name='content_type'",
        [],
        |row| row.get(0),
    )?;

    if column_exists == 0 {
        // 列不存在，添加列
        conn.execute(
            "ALTER TABLE messages ADD COLUMN content_type TEXT",
            [],
        )?;
        log::info!("✅ 已添加 content_type 列到 messages 表");
    } else {
        log::info!("ℹ️  content_type 列已存在，跳过迁移");
    }

    Ok(())
}

/// 迁移到版本 16: 创建 prompt_generation_history 表
///
/// # 功能
/// - 创建提示词生成历史记录表
/// - 用于保存每次"分析并生成提示词"的生成记录
#[cfg(test)]
pub fn migrate_v16(conn: &mut Connection) -> Result<()> {
    migrate_v16_impl(conn)
}

#[cfg(not(test))]
fn migrate_v16(conn: &mut Connection) -> Result<()> {
    migrate_v16_impl(conn)
}

fn migrate_v16_impl(conn: &mut Connection) -> Result<()> {
    // 创建提示词生成历史表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS prompt_generation_history (
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
        );",
        [],
    )?;

    // 索引: 按创建时间倒序查询
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_history_created_at
         ON prompt_generation_history(created_at DESC);",
        [],
    )?;

    // 索引: 按会话 ID 查询
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_history_session_id
         ON prompt_generation_history(session_id);",
        [],
    )?;

    // 索引: 按收藏状态查询
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_history_favorite
         ON prompt_generation_history(is_favorite);",
        [],
    )?;

    log::info!("✅ 已创建 prompt_generation_history 表");

    Ok(())
}

/// 迁移到版本 17: 创建 prompts 表
///
/// # 功能
/// - 创建提示词管理表
/// - 支持系统级和用户自定义提示词
/// - 自动插入默认提示词
#[cfg(test)]
pub fn migrate_v17(conn: &mut Connection) -> Result<()> {
    migrate_v17_impl(conn)
}

#[cfg(not(test))]
fn migrate_v17(conn: &mut Connection) -> Result<()> {
    migrate_v17_impl(conn)
}

fn migrate_v17_impl(conn: &mut Connection) -> Result<()> {
    // 1. 创建 prompts 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS prompts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            content TEXT NOT NULL,
            description TEXT,
            scenario TEXT NOT NULL DEFAULT 'session_analysis',
            category TEXT DEFAULT 'general',
            is_default INTEGER NOT NULL DEFAULT 0,
            is_system INTEGER NOT NULL DEFAULT 0,
            language TEXT NOT NULL DEFAULT 'zh',
            version INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );",
        [],
    )?;

    // 2. 创建索引
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_prompts_scenario
         ON prompts(scenario);",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_prompts_language
         ON prompts(language);",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_prompts_is_default
         ON prompts(is_default);",
        [],
    )?;

    // 3. 插入默认提示词
    let now = chrono::Utc::now().to_rfc3339();

    // 中文默认提示词
    let default_prompt_zh = r#"你是一个 Claude Code 结对编程助手。请分析下方的会话日志（包含用户指令、Claude 的操作、以及工具返回的文件内容/报错）。

任务：
1. **判断焦点 (Focus Check)**：Claude 是否陷入了死循环？是否在反复读取无关文件？是否无视了报错？
2. **生成提示词 (Prompt Generation)**：为用户写一段可以直接发送给 Claude 的**中文指令**。
   - 如果 Claude 走偏了：写一段严厉的纠正指令。
   - 如果 Claude 做得对：写一段推进下一步的指令，并引用刚才读取到的文件上下文（例如："基于刚才读取的 main.py..."）。

输出格式：
---
【状态】: [正常 / 迷失 / 报错循环]
【分析】: (简短分析当前情况)
【建议提示词】:
(你的 Prompt 内容)
---
"#;

    conn.execute(
        "INSERT OR IGNORE INTO prompts (
            name, content, description, scenario, is_default, is_system, language,
            version, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            "session_analysis_zh",
            default_prompt_zh,
            "会话分析提示词（中文）",
            "session_analysis",
            1i32,  // is_default
            1i32,  // is_system
            "zh",
            1i32,  // version
            now.clone(),
            now.clone()
        ],
    )?;

    // 英文默认提示词
    let default_prompt_en = r#"You are a Claude Code pair programming assistant. Please analyze the conversation log below (including user instructions, Claude's operations, and tool-returned file contents/errors).

Tasks:
1. **Focus Check**: Has Claude fallen into an infinite loop? Is it repeatedly reading irrelevant files? Is it ignoring errors?
2. **Prompt Generation**: Write a **Chinese instruction** that the user can send directly to Claude.
   - If Claude is off track: Write a stern corrective instruction.
   - If Claude is doing well: Write an instruction to advance to the next step, referencing the file context just read (e.g., "Based on main.py just read...").

Output Format:
---
[Status]: [Normal / Lost / Error Loop]
[Analysis]: (Brief analysis of current situation)
[Suggested Prompt]:
(Your prompt content)
---
"#;

    conn.execute(
        "INSERT OR IGNORE INTO prompts (
            name, content, description, scenario, is_default, is_system, language,
            version, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            "session_analysis_en",
            default_prompt_en,
            "Session analysis prompt (English)",
            "session_analysis",
            1i32,  // is_default
            1i32,  // is_system
            "en",
            1i32,  // version
            now.clone(),
            now.clone()
        ],
    )?;

    log::info!("✅ 已创建 prompts 表并插入默认提示词");

    Ok(())
}

/// 迁移到版本 18: 创建提示词版本管理表
///
/// # 功能
/// - 创建 prompt_templates 表（模板定义）
/// - 创建 prompt_versions 表（版本管理）
/// - 创建 prompt_components 表（组件存储）
/// - 创建 prompt_parameters 表（参数配置）
/// - 创建 prompt_changes 表（变更追踪）
/// - 触发器：确保 is_active 唯一性
#[cfg(test)]
pub fn migrate_v18(conn: &mut Connection) -> Result<()> {
    migrate_v18_impl(conn)
}

#[cfg(not(test))]
fn migrate_v18(conn: &mut Connection) -> Result<()> {
    migrate_v18_impl(conn)
}

fn migrate_v18_impl(conn: &mut Connection) -> Result<()> {
    // 1. 创建 prompt_templates 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS prompt_templates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            scenario TEXT NOT NULL DEFAULT 'optimizer',
            tags TEXT,
            language TEXT NOT NULL DEFAULT 'zh',
            is_system INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );",
        [],
    )?;

    // 2. 创建 prompt_versions 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS prompt_versions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            template_id INTEGER NOT NULL,
            version_number INTEGER NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 0,
            content TEXT NOT NULL,
            metadata TEXT,
            created_by TEXT NOT NULL DEFAULT 'user',
            created_at TEXT NOT NULL,
            FOREIGN KEY (template_id) REFERENCES prompt_templates(id) ON DELETE CASCADE,
            UNIQUE(template_id, version_number)
        );",
        [],
    )?;

    // 3. 创建 prompt_components 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS prompt_components (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            version_id INTEGER NOT NULL,
            component_type TEXT NOT NULL,
            name TEXT NOT NULL,
            content TEXT NOT NULL,
            variables TEXT,
            language TEXT NOT NULL DEFAULT 'zh',
            sort_order INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (version_id) REFERENCES prompt_versions(id) ON DELETE CASCADE
        );",
        [],
    )?;

    // 4. 创建 prompt_parameters 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS prompt_parameters (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            version_id INTEGER NOT NULL,
            key TEXT NOT NULL,
            value TEXT NOT NULL,
            parameter_type TEXT NOT NULL DEFAULT 'llm',
            description TEXT,
            FOREIGN KEY (version_id) REFERENCES prompt_versions(id) ON DELETE CASCADE,
            UNIQUE(version_id, key, parameter_type)
        );",
        [],
    )?;

    // 5. 创建 prompt_changes 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS prompt_changes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            template_id INTEGER NOT NULL,
            from_version_id INTEGER,
            to_version_id INTEGER NOT NULL,
            component_id INTEGER,
            change_type TEXT NOT NULL,
            field_name TEXT NOT NULL,
            old_value TEXT,
            new_value TEXT,
            line_number INTEGER,
            change_summary TEXT,
            changed_at TEXT NOT NULL,
            FOREIGN KEY (template_id) REFERENCES prompt_templates(id) ON DELETE CASCADE,
            FOREIGN KEY (from_version_id) REFERENCES prompt_versions(id) ON DELETE SET NULL,
            FOREIGN KEY (to_version_id) REFERENCES prompt_versions(id) ON DELETE CASCADE,
            FOREIGN KEY (component_id) REFERENCES prompt_components(id) ON DELETE SET NULL
        );",
        [],
    )?;

    // 6. 创建索引
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_prompt_versions_template_active
         ON prompt_versions(template_id, is_active);",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_prompt_components_version
         ON prompt_components(version_id);",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_prompt_parameters_version
         ON prompt_parameters(version_id);",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_prompt_changes_template
         ON prompt_changes(template_id);",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_prompt_changes_to_version
         ON prompt_changes(to_version_id);",
        [],
    )?;

    // 7. 创建触发器：确保每个模板只有一个激活版本
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS ensure_single_active_prompt_version
         BEFORE UPDATE OF is_active ON prompt_versions
         WHEN NEW.is_active = 1
         BEGIN
             UPDATE prompt_versions SET is_active = 0
             WHERE template_id = NEW.template_id AND is_active = 1 AND id != NEW.id;
         END;",
        [],
    )?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS ensure_single_active_prompt_version_insert
         BEFORE INSERT ON prompt_versions
         WHEN NEW.is_active = 1
         BEGIN
             UPDATE prompt_versions SET is_active = 0
             WHERE template_id = NEW.template_id AND is_active = 1;
         END;",
        [],
    )?;

    log::info!("✅ 已创建提示词版本管理表（v18）");

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
