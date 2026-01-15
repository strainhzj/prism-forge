use rusqlite::Connection;

fn main() {
    println!("测试数据库连接...");

    // 测试 1: 打开连接
    let mut conn = match Connection::open("test.db") {
        Ok(c) => {
            println!("✓ 数据库连接成功");
            c
        }
        Err(e) => {
            println!("✗ 数据库连接失败: {}", e);
            return;
        }
    };

    // 测试 2: PRAGMA foreign_keys
    println!("测试 PRAGMA foreign_keys...");
    match conn.execute("PRAGMA foreign_keys = ON;", []) {
        Ok(_) => println!("✓ PRAGMA foreign_keys 成功"),
        Err(e) => println!("✗ PRAGMA foreign_keys 失败: {}", e),
    }

    // 测试 3: PRAGMA busy_timeout
    println!("测试 PRAGMA busy_timeout...");
    match conn.execute("PRAGMA busy_timeout = 30000;", []) {
        Ok(_) => println!("✓ PRAGMA busy_timeout 成功"),
        Err(e) => println!("✗ PRAGMA busy_timeout 失败: {}", e),
    }

    // 测试 4: PRAGMA journal_mode (使用 execute)
    println!("测试 PRAGMA journal_mode (execute)...");
    match conn.execute("PRAGMA journal_mode = WAL;", []) {
        Ok(_) => println!("✓ PRAGMA journal_mode (execute) 成功"),
        Err(e) => println!("✗ PRAGMA journal_mode (execute) 失败: {}", e),
    }

    // 测试 5: PRAGMA journal_mode (使用 query_row)
    println!("测试 PRAGMA journal_mode (query_row)...");
    match conn.query_row("PRAGMA journal_mode = WAL;", [], |row| {
        let mode: String = row.get(0)?;
        Ok(mode)
    }) {
        Ok(mode) => println!("✓ PRAGMA journal_mode (query_row) 成功, mode={}", mode),
        Err(e) => println!("✗ PRAGMA journal_mode (query_row) 失败: {}", e),
    }

    // 清理
    std::fs::remove_file("test.db").ok();
    println!("\n测试完成！");
}
