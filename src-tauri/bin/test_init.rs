use anyhow::Result;
use rusqlite::Connection;
use std::path::PathBuf;

fn main() {
    println!("=== 数据库初始化测试 ===\n");

    // 获取数据库路径
    let mut db_path: PathBuf = dirs::home_dir().unwrap();
    db_path.push(".prism-forge");
    db_path.push("test_prism_forge.db");

    println!("数据库路径: {:?}", db_path);

    // 确保目录存在
    std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();
    println!("✓ 目录已创建");

    // 打开数据库连接
    println!("\n1. 打开数据库连接...");
    let mut conn = match Connection::open(&db_path) {
        Ok(c) => {
            println!("✓ 连接成功");
            c
        }
        Err(e) => {
            println!("✗ 连接失败: {}", e);
            return;
        }
    };

    // PRAGMA foreign_keys
    println!("\n2. 测试 PRAGMA foreign_keys...");
    match conn.execute("PRAGMA foreign_keys = ON;", []) {
        Ok(_) => println!("✓ PRAGMA foreign_keys 成功"),
        Err(e) => println!("✗ PRAGMA foreign_keys 失败: {}", e),
    }

    // PRAGMA busy_timeout
    println!("\n3. 测试 PRAGMA busy_timeout...");
    match conn.execute("PRAGMA busy_timeout = 30000;", []) {
        Ok(_) => println!("✓ PRAGMA busy_timeout 成功"),
        Err(e) => println!("✗ PRAGMA busy_timeout 失败: {}", e),
    }

    // PRAGMA journal_mode (使用 execute)
    println!("\n4. 测试 PRAGMA journal_mode (execute)...");
    match conn.execute("PRAGMA journal_mode = WAL;", []) {
        Ok(_) => println!("✓ PRAGMA journal_mode (execute) 成功"),
        Err(e) => println!("✗ PRAGMA journal_mode (execute) 失败: {}", e),
    }

    // PRAGMA journal_mode (使用 query_row)
    println!("\n5. 测试 PRAGMA journal_mode (query_row)...");
    match conn.query_row("PRAGMA journal_mode = WAL;", [], |row| {
        let mode: String = row.get(0)?;
        Ok(mode)
    }) {
        Ok(mode) => println!("✓ PRAGMA journal_mode (query_row) 成功, mode={}", mode),
        Err(e) => println!("✗ PRAGMA journal_mode (query_row) 失败: {}", e),
    }

    // SELECT vec_init
    println!("\n6. 测试 SELECT vec_init()...");
    match conn.query_row("SELECT vec_init()", [], |_| Ok(())) {
        Ok(_) => println!("✓ SELECT vec_init() 成功"),
        Err(e) => println!("✗ SELECT vec_init() 失败: {}", e),
    }

    // 清理
    std::fs::remove_file(&db_path).ok();
    println!("\n✓ 测试完成！数据库文件已清理");
}
