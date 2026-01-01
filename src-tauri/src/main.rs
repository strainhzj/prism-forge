// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 数据库模块
mod database;
// LLM 客户端模块
mod llm;
// 向量嵌入模块
mod embedding;
// 命令模块
mod commands;
// Optimizer 模块
mod optimizer;
// Token 计数器模块
mod tokenizer;
// 监控模块
mod monitor;
// 解析器模块
mod parser;

use std::path::PathBuf;
use std::time::SystemTime;

// 导入 commands
use commands::*;
use optimizer::find_latest_session_file;

// ==================== Tauri Commands ====================

/// 获取最新的会话文件路径
#[tauri::command]
fn get_latest_session_path() -> Result<String, String> {
    find_latest_session_file()
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| "未找到会话文件".to_string())
}

/// 解析会话文件（用于前端预览展示）
///
/// 此命令将 JSONL 格式的 Claude 会话文件解析为结构化事件列表，
/// 供前端 UI 展示会话日志内容。
#[tauri::command]
fn parse_session_file(file_path: String) -> Result<Vec<optimizer::ParsedEvent>, String> {
    use optimizer::PromptOptimizer;

    PromptOptimizer::parse_session_file(&file_path)
        .map_err(|e| e.to_string())
}

fn main() {
    // 初始化数据库
    database::migrations::get_db_path()
        .and_then(|_| {
            // 确保数据库目录存在
            Ok(())
        })
        .expect("初始化数据库失败");

    // 创建 LLM 客户端管理器
    let llm_manager = llm::LLMClientManager::from_default_db()
        .expect("创建 LLM 客户端管理器失败");

    tauri::Builder::default()
        .manage(llm_manager)
        .invoke_handler(tauri::generate_handler![
            get_latest_session_path,
            parse_session_file,
            cmd_get_providers,
            cmd_save_provider,
            cmd_delete_provider,
            cmd_set_active_provider,
            cmd_test_provider_connection,
            count_prompt_tokens,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
