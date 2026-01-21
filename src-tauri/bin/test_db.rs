use rusqlite::Connection;

fn main() {
    println!("测试数据库连接...");
    let mut conn = Connection::open("test.db").expect("无法打开数据库");
    println!("✓ 数据库连接成功");

    // 测试 PRAGMA journal_mode
    println!("测试 PRAGMA journal_mode (query_row)...");
    match conn.query_row("PRAGMA journal_mode = WAL;", [], |row| {
        let mode: String = row.get(0)?;
        Ok(mode)
    }) {
        Ok(mode) => println!("✓ PRAGMA journal_mode 成功, mode={}", mode),
        Err(e) => println!("✗ PRAGMA journal_mode 失败: {}", e),
    }
}
