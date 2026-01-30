use std::path::PathBuf;

fn main() {
    let config_path = PathBuf::from("optimizer_config.toml");
    let content = std::fs::read_to_string(&config_path).expect("无法读取文件");
    println!("文件内容长度: {}", content.len());
    
    // 尝试解析
    match toml::from_str::<toml::Value>(&content) {
        Ok(value) => {
            println!("✓ TOML 解析成功");
            if let Some(fallback) = value.get("fallback") {
                println!("✓ fallback 部分存在");
                if let Some(template) = fallback.get("conversation_starter_template") {
                    if let Some(s) = template.as_str() {
                        println!("✓ conversation_starter_template 存在，长度: {}", s.len());
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("✗ TOML 解析失败: {}", e);
        }
    }
}
