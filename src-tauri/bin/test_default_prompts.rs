//! 测试默认提示词导入功能
//!
//! 运行方式: cargo run --bin test_default_prompts

use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    println!("=== 默认提示词导入测试 ===\n");


    // 1. 解析配置文件路径
    println!("1. 测试配置文件路径解析...");

    // 尝试从可执行文件位置查找
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap();

    // 尝试多个可能的路径
    let possible_paths = [
        PathBuf::from("optimizer_config.toml"),
        exe_dir.join("optimizer_config.toml"),
        exe_dir.join("src-tauri").join("optimizer_config.toml"),
    ];

    let config_path = possible_paths.iter()
        .find(|p| p.exists())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("找不到 optimizer_config.toml"))?;

    println!("   ✅ 配置文件: {:?}", config_path);

    // 2. 读取配置文件
    println!("\n2. 读取配置文件...");
    let config_content = std::fs::read_to_string(&config_path)?;
    println!("   文件大小: {} 字节", config_content.len());

    // 3. 解析 TOML
    println!("\n3. 解析 TOML...");
    let config: prism_forge::optimizer::config::OptimizerConfig = toml::from_str(&config_content)?;
    println!("   ✅ TOML 解析成功");
    println!("   Meta-Prompt (中文) 长度: {} 字符", config.components.meta_prompt.zh.len());

    // 4. 测试导入功能
    println!("\n4. 测试数据库导入...");
    let db_path = std::env::var("APPDATA")
        .map(|p| PathBuf::from(p).join("prism-forge").join("prism-forge-test.db"))
        .unwrap_or_else(|_| PathBuf::from("prism-forge-test.db"));

    println!("   测试数据库路径: {:?}", db_path);

    // 删除旧测试数据库
    if db_path.exists() {
        std::fs::remove_file(&db_path)?;
        println!("   已删除旧测试数据库");
    }

    // 创建测试数据库
    let mut conn = rusqlite::Connection::open(&db_path)?;

    // 执行迁移
    println!("   执行数据库迁移...");
    prism_forge::database::migrations::run_migrations(&mut conn)?;
    println!("   ✅ 迁移完成");

    // 导入默认提示词
    println!("   导入默认提示词...");
    prism_forge::database::init_default_prompts::import_default_prompts(&mut conn)?;
    println!("   ✅ 导入成功");

    // 5. 验证导入结果
    println!("\n5. 验证导入结果...");
    let templates: Vec<(String, String, String, i32)> = conn
        .prepare("SELECT name, scenario, language, is_system FROM prompt_templates ORDER BY name")?
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    println!("   导入的模板数量: {}", templates.len());
    for (name, scenario, language, is_system) in templates {
        println!("   - {} (场景: {}, 语言: {}, 系统: {})",
            name, scenario, language, if is_system == 1 { "是" } else { "否" }
        );
    }

    // 验证版本
    println!("\n6. 验证版本信息...");
    let versions: Vec<(String, i32, bool)> = conn
        .prepare("
            SELECT pt.name, pv.version_number, pv.is_active
            FROM prompt_versions pv
            JOIN prompt_templates pt ON pv.template_id = pt.id
            ORDER BY pt.name, pv.version_number DESC
        ")?
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    println!("   版本信息:");
    for (name, version, is_active) in versions {
        println!("   - {} v{} (激活: {})", name, version, if is_active { "是" } else { "否" });
    }

    println!("\n=== 测试完成 ===");

    // 清理测试数据库
    std::fs::remove_file(&db_path)?;
    println!("已清理测试数据库");

    Ok(())
}
