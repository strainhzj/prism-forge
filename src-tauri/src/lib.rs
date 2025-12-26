// LLM 客户端模块
mod llm;
mod database;
mod commands;

use llm::LLMClientManager;
use database::migrations;
use commands::*;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化数据库
    migrations::get_db_path()
        .and_then(|_| {
            // 确保数据库目录存在
            Ok(())
        })
        .expect("初始化数据库失败");

    // 创建 LLM 客户端管理器
    let llm_manager = LLMClientManager::from_default_db()
        .expect("创建 LLM 客户端管理器失败");

    tauri::Builder::default()
        .manage(llm_manager)
        .invoke_handler(tauri::generate_handler![
            greet,
            cmd_get_providers,
            cmd_save_provider,
            cmd_delete_provider,
            cmd_set_active_provider,
            cmd_test_provider_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
