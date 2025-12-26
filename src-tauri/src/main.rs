// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 数据库模块
mod database;
// LLM 客户端模块
mod llm;
// 命令模块
mod commands;
// Optimizer 模块
mod optimizer;

use std::path::PathBuf;
use std::time::SystemTime;

// 导入 commands
use commands::*;
use optimizer::find_latest_session_file;
use optimizer::OptimizeRequest;

// ==================== Tauri Commands ====================

/// 获取最新的会话文件路径
#[tauri::command]
fn get_latest_session_path() -> Result<String, String> {
    find_latest_session_file()
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| "未找到会话文件".to_string())
}

/// 优化提示词（使用 LLMClientManager）
///
/// # 流程
/// 1. 从 LLMClientManager 获取当前活跃的客户端
/// 2. 解析会话文件
/// 3. 调用 LLM 分析
/// 4. 返回优化结果
#[tauri::command]
async fn optimize_prompt(
    session_file: String,
    goal: String,
    llm_manager: tauri::State<'_, llm::LLMClientManager>,
) -> Result<String, String> {
    use optimizer::PromptOptimizer;

    // State 可以自动解引用，通过 * 或 &* 获取内部值的引用
    let manager_ref = &*llm_manager;
    let optimizer = PromptOptimizer::new();
    let request = OptimizeRequest {
        session_file,
        goal,
    };

    let result = optimizer.optimize_prompt(request, manager_ref)
        .await
        .map_err(|e| e.to_string())?;

    // 格式化返回结果
    let output = format!(
        "---\n【状态】: {}\n【分析】: {}\n【建议提示词】:\n{}\n---",
        result.status,
        result.analysis,
        result.suggested_prompt
    );

    Ok(output)
}

/// 保留的旧命令（用于兼容）
/// TODO: 在前端迁移完成后移除
#[tauri::command]
async fn analyze_session(
    session_file: String,
    goal: String,
    llm_manager: tauri::State<'_, llm::LLMClientManager>,
) -> Result<String, String> {
    // 直接调用新的 optimize_prompt
    optimize_prompt(session_file, goal, llm_manager).await
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
            optimize_prompt,
            analyze_session, // 保留旧命令以兼容
            cmd_get_providers,
            cmd_save_provider,
            cmd_delete_provider,
            cmd_set_active_provider,
            cmd_test_provider_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
