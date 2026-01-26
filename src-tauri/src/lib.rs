// LLM 客户端模块
// 注意：模块声明顺序很重要，被依赖的模块需要先声明
pub mod perf;
mod llm;
pub mod database;
mod commands;
mod commands_prompt;
mod tokenizer;
mod monitor;
mod parser;
pub mod embedding;
pub use embedding::EmbeddingGenerator;
pub mod optimizer;
pub mod command_registry;
pub mod startup;
pub mod command_wrapper;
pub mod logging;
pub mod path_resolver;
pub mod session_reader;
pub mod session_type_detector;
mod filter_config;
pub mod session_parser;

// 导入 Tauri 插件

use llm::LLMClientManager;
use database::migrations;
use commands::*;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// 获取指定项目的最新会话文件路径
///
/// # 参数
/// * `project_path` - 项目路径（监控目录路径）
///
/// # 返回
/// 返回该项目 sessions 目录下修改时间最新的 .jsonl 文件路径
#[tauri::command]
fn get_latest_session_path(project_path: String) -> Result<String, String> {
    optimizer::find_latest_session_file_in_project(&project_path)
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| "当前项目没有会话文件".to_string())
}

/// 解析会话文件（用于前端预览展示）
#[tauri::command]
fn parse_session_file(file_path: String) -> Result<Vec<optimizer::ParsedEvent>, String> {
    optimizer::PromptOptimizer::parse_session_file(&file_path)
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 执行启动验证
    eprintln!("[INFO] Starting application startup validation...");
    let validation_result = startup::perform_startup_validation();
    
    if !validation_result.success {
        eprintln!("[ERROR] Startup validation failed!");
        for error in &validation_result.errors {
            eprintln!("[ERROR]   {}", error);
        }
        // Continue with startup but log warnings
        // In production, you might want to show a dialog or exit
    } else {
        eprintln!(
            "[INFO] Startup validation successful: {} commands registered",
            validation_result.registered_commands.len()
        );
    }

    // Log any warnings
    for warning in &validation_result.warnings {
        eprintln!("[WARN]   {}", warning);
    }

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

    // 创建启动管理器用于运行时诊断
    let startup_manager = startup::create_startup_manager();

    tauri::Builder::default()
        .manage(llm_manager)
        .manage(startup_manager)
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_latest_session_path,
            parse_session_file,
            cmd_get_providers,
            cmd_save_provider,
            cmd_delete_provider,
            cmd_set_active_provider,
            cmd_test_provider_connection,
            count_prompt_tokens,
            scan_sessions,
            scan_directory,
            run_benchmarks,
            parse_session_tree,
            set_session_rating,
            set_session_tags,
            get_session_rating,
            get_session_tags,
            archive_session,
            unarchive_session,
            get_archived_sessions,
            start_file_watcher,
            extract_session_log,
            export_session_log,
            vector_search,
            compress_context,
            optimize_prompt,
            get_meta_template,
            update_meta_template,
            // 优化器配置管理命令
            reload_optimizer_config,
            get_optimizer_config,
            // 监控目录管理命令
            get_monitored_directories,
            add_monitored_directory,
            remove_monitored_directory,
            toggle_monitored_directory,
            update_monitored_directory,
            get_sessions_by_monitored_directory,
            // 向量搜索命令
            semantic_search,
            find_similar_sessions,
            get_vector_settings,
            update_vector_settings,
            sync_embeddings_now,
            // 多级日志读取命令
            cmd_get_messages_by_level,
            cmd_get_qa_pairs_by_level,
            cmd_save_view_level_preference,
            cmd_get_view_level_preference,
            cmd_export_session_by_level,
            // 日志过滤配置管理命令
            get_filter_config,
            update_filter_config,
            reload_filter_config,
            get_filter_config_path,
            open_filter_config_folder,
            // 提示词生成历史管理命令
            cmd_save_prompt_history,
            cmd_get_prompt_history,
            cmd_get_prompt_history_paginated,
            cmd_get_prompt_history_by_id,
            cmd_delete_prompt_history,
            cmd_toggle_prompt_history_favorite,
            cmd_get_favorite_prompt_history,
            cmd_count_prompt_history,
            // 提示词管理命令
            commands_prompt::cmd_get_prompts,
            commands_prompt::cmd_get_prompt,
            commands_prompt::cmd_save_prompt,
            commands_prompt::cmd_delete_prompt,
            commands_prompt::cmd_reset_default_prompt,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
