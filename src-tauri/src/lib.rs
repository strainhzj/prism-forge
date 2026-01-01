// LLM 客户端模块
// 注意：模块声明顺序很重要，被依赖的模块需要先声明
pub mod perf;
mod llm;
mod database;
mod commands;
mod tokenizer;
mod monitor;
mod parser;
mod embedding;
mod optimizer;

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
            count_prompt_tokens,
            scan_sessions,
            run_benchmarks,
            parse_session_tree,
            set_session_rating,
            set_session_tags,
            get_session_rating,
            get_session_tags,
            archive_session,
            unarchive_session,
            get_archived_sessions,
            get_active_sessions,
            start_file_watcher,
            extract_session_log,
            export_session_log,
            vector_search,
            compress_context,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
