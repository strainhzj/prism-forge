//! 数据库初始化和全局连接管理
//!
//! 提供单例模式的数据库连接访问

use anyhow::Result;
use rusqlite::Connection;
use std::sync::{Arc, Mutex, OnceLock};

/// 全局数据库连接单例
static GLOBAL_CONNECTION: OnceLock<Arc<Mutex<Connection>>> = OnceLock::new();

/// 初始化标记（确保只初始化一次）
static INIT_LOCK: OnceLock<Mutex<bool>> = OnceLock::new();

/// 获取数据库文件路径
///
/// 重新导出自 migrations 模块
pub fn get_db_path() -> Result<std::path::PathBuf> {
    crate::database::migrations::get_db_path()
}

/// 加载 sqlite-vec 扩展
///
/// 使用 vec_init() 函数初始化向量扩展
fn load_sqlite_vec_extension(conn: &mut Connection) -> Result<()> {
    // 注意: sqlite-vec 0.1.7-alpha.2 使用不同的初始化方式
    // 使用 sqlite-vec 的 bundle 版本会自动加载扩展
    // 这里调用 vec_init() 来确保扩展已正确初始化
    conn.query_row("SELECT vec_init()", [], |_| Ok(()))?;
    log::info!("sqlite-vec 扩展加载成功");
    Ok(())
}

/// 初始化数据库连接
///
/// 只执行一次，创建连接并执行迁移
fn initialize_connection() -> Result<Arc<Mutex<Connection>>> {
    // 获取数据库路径
    let db_path = get_db_path()?;
    log::info!("初始化数据库连接: {:?}", db_path);

    // 打开数据库连接
    let mut conn = Connection::open(&db_path)?;

    // 设置 PRAGMA: 启用外键约束
    conn.execute("PRAGMA foreign_keys = ON;", [])?;
    log::debug!("外键约束已启用");

    // 设置 PRAGMA: 忙等待超时 30 秒
    let _timeout: i32 = conn.query_row("PRAGMA busy_timeout = 30000;", [], |row| row.get(0))?;
    log::debug!("忙等待超时设置为 30 秒");

    // 设置 PRAGMA: 启用 WAL 模式（提升并发性能）
    let _journal_mode: String = conn.query_row("PRAGMA journal_mode = WAL;", [], |row| row.get(0))?;
    log::debug!("WAL 模式已启用");

    // 加载 sqlite-vec 扩展
    if let Err(e) = load_sqlite_vec_extension(&mut conn) {
        log::warn!("sqlite-vec 扩展加载失败: {}", e);
        // 不中断初始化，向量功能将在需要时报错
    }

    // 执行数据库迁移
    crate::database::migrations::run_migrations(&mut conn)?;
    log::info!("数据库迁移完成");

    // 包装在 Arc<Mutex<>> 中返回
    Ok(Arc::new(Mutex::new(conn)))
}

/// 获取全局共享的数据库连接
///
/// 使用单例模式，确保整个应用只使用一个连接
/// 首次调用时会自动执行数据库迁移
pub fn get_connection_shared() -> Result<Arc<Mutex<Connection>>> {
    // 尝试获取已初始化的连接
    if let Some(conn) = GLOBAL_CONNECTION.get() {
        return Ok(conn.clone());
    }

    // 获取初始化锁
    let init_lock = INIT_LOCK.get_or_init(|| Mutex::new(false));
    let mut initializing = init_lock.lock().map_err(|e| {
        anyhow::anyhow!("获取初始化锁失败（Mutex 已被毒化）: {}", e)
    })?;

    // 双重检查：可能在等待锁期间已被其他线程初始化
    if let Some(conn) = GLOBAL_CONNECTION.get() {
        return Ok(conn.clone());
    }

    // 检查是否正在初始化
    if *initializing {
        return Err(anyhow::anyhow!("数据库正在初始化中"));
    }

    *initializing = true;
    drop(initializing);

    // 执行初始化
    let conn = initialize_connection().map_err(|e| {
        // 重置初始化标记
        if let Ok(mut lock) = init_lock.lock() {
            *lock = false;
        }
        e
    })?;

    // 设置初始化完成标记
    if let Ok(mut lock) = init_lock.lock() {
        *lock = true;
    }

    // 存储到全局单例
    GLOBAL_CONNECTION
        .set(conn.clone())
        .map_err(|_| anyhow::anyhow!("数据库连接已初始化"))?;

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
    fn test_get_connection_shared_singleton() {
        // 重置全局状态（仅用于测试）
        // 注意: 这在实际应用中不应该这样做
        let conn1 = get_connection_shared().unwrap();
        let conn2 = get_connection_shared().unwrap();

        // 验证是同一个实例（通过 Arc 指针比较）
        let ptr1 = Arc::as_ptr(&conn1) as usize;
        let ptr2 = Arc::as_ptr(&conn2) as usize;
        assert_eq!(ptr1, ptr2, "应该返回相同的单例实例");
    }

    #[test]
    fn test_connection_is_accessible() {
        let conn = get_connection_shared().unwrap();
        let guard = conn.lock().unwrap();

        // 验证可以执行简单查询
        let result: String = guard
            .query_row("SELECT sqlite_version()", [], |row| row.get(0))
            .unwrap();
        log::info!("SQLite 版本: {}", result);
    }
}
