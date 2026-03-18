//! Tauri Commands - LLM API Provider 管理
//!
//! 暴露给前端调用的命令接口

use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use tauri::State;

use crate::database::vector_repository::VectorRepository;
use crate::database::{ApiProvider, ApiProviderRepository, ApiProviderType};
use crate::database::{DecisionKeyword, DecisionKeywordRepository};
use crate::database::DecisionAnalysisRepository;
use crate::embedding::{EmbeddingSyncManager, OpenAIEmbeddings};
use crate::intent_analyzer::{DecisionDetector, DecisionPoint as DetectedDecisionPoint};
use crate::llm::interface::TestConnectionResult;
use crate::llm::security::ApiKeyStorage;
use crate::llm::LLMClientManager;
use crate::optimizer::compressor::CompressionResult;
use crate::optimizer::prompt_generator::{EnhancedPrompt, EnhancedPromptRequest};
use crate::parser::{
    extractor::{ExportFormat, ExtractionEngine, ExtractionLevel},
    jsonl::JsonlParser,
    tree::{ConversationTree, MessageTreeBuilder},
};
use crate::session_type_detector::SessionFileType;
use crate::tokenizer::{TokenCounter, TokenEncodingType};

// ==================== 性能基准测试模块（内联） ====================

/// 性能测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// 测试名称
    pub name: String,
    /// 耗时（毫秒）
    pub duration_ms: f64,
    /// 是否通过阈值
    pub passed: bool,
    /// 阈值（毫秒）
    pub threshold_ms: f64,
    /// 详细信息
    pub details: String,
}

/// 性能测试报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    /// 测试时间戳
    pub timestamp: String,
    /// 测试结果列表
    pub results: Vec<BenchmarkResult>,
    /// 总体是否通过
    pub overall_passed: bool,
}

impl BenchmarkReport {
    /// 生成 Markdown 格式的报告
    pub fn to_markdown(&self) -> String {
        let mut md = format!("# 性能基准测试报告\n\n");
        md.push_str(&format!("**测试时间**: {}\n\n", self.timestamp));
        md.push_str(&format!(
            "**总体结果**: {}\n\n",
            if self.overall_passed {
                "✅ 通过"
            } else {
                "❌ 失败"
            }
        ));

        md.push_str("## 测试结果详情\n\n");
        md.push_str("| 测试名称 | 耗时 (ms) | 阈值 (ms) | 结果 | 详情 |\n");
        md.push_str("|---------|----------|----------|------|------|\n");

        for result in &self.results {
            let status = if result.passed {
                "✅ 通过"
            } else {
                "❌ 失败"
            };
            md.push_str(&format!(
                "| {} | {:.2} | {:.2} | {} | {} |\n",
                result.name, result.duration_ms, result.threshold_ms, status, result.details
            ));
        }

        // 添加总结
        let total_time: f64 = self.results.iter().map(|r| r.duration_ms).sum();
        md.push_str(&format!("\n**总耗时**: {:.2} ms\n", total_time));

        // 添加建议
        md.push_str("\n## 性能优化建议\n\n");
        for result in &self.results {
            if !result.passed {
                md.push_str(&format!("### {} 未达标\n", result.name));
                md.push_str(&format!("- 当前耗时: {:.2} ms\n", result.duration_ms));
                md.push_str(&format!("- 目标阈值: {:.2} ms\n", result.threshold_ms));
                md.push_str(&format!(
                    "- 差距: {:.2} ms\n",
                    result.duration_ms - result.threshold_ms
                ));
                md.push_str(&get_optimization_suggestion(&result.name));
                md.push_str("\n");
            }
        }

        md
    }

    /// 生成 JSON 格式的报告
    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}

/// 获取优化建议
fn get_optimization_suggestion(test_name: &str) -> String {
    match test_name {
        "应用启动时间" => String::from(
            "**优化建议**:\n\
                - 检查数据库连接池配置\n\
                - 考虑延迟加载非关键模块\n\
                - 使用异步初始化避免阻塞主线程\n\
                - 检查是否有冗余的文件 I/O 操作\n",
        ),
        "会话扫描时间" => String::from(
            "**优化建议**:\n\
                - 使用并行扫描处理多个项目目录\n\
                - 增加文件扫描缓存\n\
                - 优化 glob 模式匹配\n\
                - 考虑增量扫描策略（仅扫描变更文件）\n",
        ),
        "数据库查询性能" => String::from(
            "**优化建议**:\n\
                - 添加适当的索引\n\
                - 使用查询预编译语句\n\
                - 考虑使用连接池\n\
                - 优化复杂查询的 SQL 结构\n",
        ),
        _ => String::from("**暂无具体建议**\n"),
    }
}

/// 测试应用启动时间
fn benchmark_startup_time() -> BenchmarkResult {
    let name = String::from("应用启动时间");
    let threshold_ms = 3000.0;

    let start = Instant::now();

    // 1. 测试数据库初始化时间
    let db_start = Instant::now();
    let db_result = crate::database::init::get_connection_shared();
    let db_duration = db_start.elapsed();

    let details = if let Err(e) = db_result {
        format!("数据库初始化失败: {}", e)
    } else {
        format!("数据库初始化耗时: {:.2} ms", db_duration.as_millis())
    };

    let duration = start.elapsed();
    let duration_ms = duration.as_secs_f64() * 1000.0;
    let passed = duration_ms < threshold_ms;

    BenchmarkResult {
        name,
        duration_ms,
        passed,
        threshold_ms,
        details,
    }
}

/// 测试会话扫描时间
fn benchmark_scan_sessions() -> BenchmarkResult {
    let name = String::from("会话扫描时间");
    let threshold_ms = 2000.0;

    let start = Instant::now();

    // 执行会话扫描
    let scan_result = crate::monitor::scanner::scan_session_files();
    let duration = start.elapsed();

    let (details, passed) = match scan_result {
        Ok(sessions) => {
            let count = sessions.len();
            let duration_ms = duration.as_secs_f64() * 1000.0;

            // 根据会话数量调整阈值
            let expected_ms = (count as f64 / 100.0) * threshold_ms;
            let passed = duration_ms < expected_ms;

            let details = format!(
                "扫描 {} 个会话，耗时 {:.2} ms（目标阈值: {:.2} ms）",
                count, duration_ms, expected_ms
            );

            (details, passed)
        }
        Err(e) => {
            let details = format!("扫描失败: {}", e);
            (details, false)
        }
    };

    let duration_ms = duration.as_secs_f64() * 1000.0;

    BenchmarkResult {
        name,
        duration_ms,
        passed,
        threshold_ms,
        details,
    }
}

/// 测试数据库查询性能
fn benchmark_database_queries() -> BenchmarkResult {
    let name = String::from("数据库查询性能");
    let threshold_ms = 100.0;

    let start = Instant::now();

    let query_result = (|| -> anyhow::Result<String> {
        let conn = crate::database::init::get_connection_shared()?;
        let guard = conn
            .lock()
            .map_err(|e| anyhow::anyhow!("获取锁失败: {}", e))?;

        // 测试查询性能
        let query_start = Instant::now();
        let _version: String = guard.query_row("SELECT sqlite_version()", [], |row| row.get(0))?;
        let query_duration = query_start.elapsed();

        Ok(format!(
            "SQLite 版本查询耗时: {:.2} ms",
            query_duration.as_millis()
        ))
    })();

    let duration = start.elapsed();
    let duration_ms = duration.as_secs_f64() * 1000.0;
    let passed = duration_ms < threshold_ms;

    let details = match query_result {
        Ok(msg) => msg,
        Err(e) => format!("查询失败: {}", e),
    };

    BenchmarkResult {
        name,
        duration_ms,
        passed,
        threshold_ms,
        details,
    }
}

/// 运行所有性能测试
fn run_all_benchmarks_internal() -> BenchmarkReport {
    let timestamp = chrono::Utc::now().to_rfc3339();

    let mut results = Vec::new();

    // 测试 1: 应用启动时间
    println!("🚀 测试 1/3: 应用启动时间...");
    results.push(benchmark_startup_time());

    // 测试 2: 会话扫描时间
    println!("🔍 测试 2/3: 会话扫描时间...");
    results.push(benchmark_scan_sessions());

    // 测试 3: 数据库查询性能
    println!("💾 测试 3/3: 数据库查询性能...");
    results.push(benchmark_database_queries());

    // 计算总体结果
    let overall_passed = results.iter().all(|r| r.passed);

    BenchmarkReport {
        timestamp,
        results,
        overall_passed,
    }
}

/// 保存性能测试报告到文件
fn save_benchmark_report_internal(
    report: &BenchmarkReport,
    output_path: &PathBuf,
) -> anyhow::Result<()> {
    // 创建输出目录
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // 保存 Markdown 报告
    let md_path = output_path.with_extension("md");
    fs::write(&md_path, report.to_markdown())?;
    println!("✅ Markdown 报告已保存到: {:?}", md_path);

    // 保存 JSON 报告
    let json_path = output_path.with_extension("json");
    fs::write(&json_path, report.to_json()?)?;
    println!("✅ JSON 报告已保存到: {:?}", json_path);

    Ok(())
}

/// 创建/更新提供商的请求参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveProviderRequest {
    /// 提供商 ID（更新时需要，创建时为 null）
    pub id: Option<i64>,

    /// 提供商类型
    pub provider_type: ApiProviderType,

    /// 用户自定义名称
    pub name: String,

    /// API 基础 URL
    pub base_url: String,

    /// API Key（明文，仅用于传输）
    pub api_key: Option<String>,

    /// 配置的模型名称
    pub model: Option<String>,

    /// 额外配置 JSON
    pub config_json: Option<String>,

    /// Temperature 参数
    pub temperature: Option<f32>,

    /// Max Tokens 参数
    pub max_tokens: Option<u32>,

    /// 是否设置为活跃提供商
    pub is_active: bool,
}

/// 提供商响应（包含敏感信息掩码）
///
/// 注意：手动展开 ApiProvider 字段以确保 camelCase 序列化一致性
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderResponse {
    /// 主键 ID
    pub id: Option<i64>,

    /// 提供商类型
    pub provider_type: ApiProviderType,

    /// 用户自定义名称
    pub name: String,

    /// API 基础 URL
    pub base_url: String,

    /// API Key 引用标识
    pub api_key_ref: Option<String>,

    /// 配置的模型名称
    pub model: Option<String>,

    /// 额外配置 JSON
    pub config_json: Option<String>,

    /// Temperature 参数
    pub temperature: Option<f32>,

    /// Max Tokens 参数
    pub max_tokens: Option<u32>,

    /// 是否为当前活跃的提供商
    pub is_active: bool,

    /// API Key 是否已配置（掩码显示）
    pub has_api_key: bool,

    /// API Key 掩码显示（前 8 个字符）
    pub api_key_mask: Option<String>,
}

impl From<ApiProvider> for ProviderResponse {
    fn from(provider: ApiProvider) -> Self {
        Self {
            id: provider.id,
            provider_type: provider.provider_type,
            name: provider.name,
            base_url: provider.base_url,
            api_key_ref: provider.api_key_ref,
            model: provider.model,
            config_json: provider.config_json,
            temperature: provider.temperature,
            max_tokens: provider.max_tokens,
            is_active: provider.is_active,
            has_api_key: false,
            api_key_mask: None,
        }
    }
}

/// 命令错误响应
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub message: String,
}

impl From<anyhow::Error> for CommandError {
    fn from(err: anyhow::Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

/// 获取所有 API 提供商
///
/// # 返回
/// 返回所有提供商列表，按创建时间倒序排列
#[tauri::command]
pub fn cmd_get_providers(
    manager: State<'_, LLMClientManager>,
) -> std::result::Result<Vec<ProviderResponse>, CommandError> {
    let providers = manager.get_all_providers()?;

    let responses: Vec<ProviderResponse> = providers
        .into_iter()
        .map(|provider| {
            let provider_id = provider.id;
            let has_api_key = provider_id
                .map(|id| ApiKeyStorage::has_api_key(id))
                .unwrap_or(false);

            // 掩码显示 API Key（不回显完整密钥）
            let api_key_mask = if has_api_key {
                // 从 keyring 读取并掩码
                provider_id.and_then(|id| {
                    ApiKeyStorage::get_api_key(id)
                        .ok()
                        .map(|key| crate::llm::security::ApiKeyValidator::mask_api_key(&key, 8))
                })
            } else {
                None
            };

            ProviderResponse {
                id: provider.id,
                provider_type: provider.provider_type,
                name: provider.name,
                base_url: provider.base_url,
                api_key_ref: provider.api_key_ref,
                model: provider.model,
                config_json: provider.config_json,
                temperature: provider.temperature,
                max_tokens: provider.max_tokens,
                is_active: provider.is_active,
                has_api_key,
                api_key_mask,
            }
        })
        .collect();

    Ok(responses)
}

/// 保存 API 提供商（创建或更新）
///
/// # 流程
/// 1. 如果有 id，则更新现有提供商；否则创建新提供商
/// 2. 如果提供了 api_key，则保存到 keyring
/// 3. 更新数据库中的提供商信息
#[tauri::command]
pub async fn cmd_save_provider(
    _manager: State<'_, LLMClientManager>,
    request: SaveProviderRequest,
) -> std::result::Result<ApiProvider, CommandError> {
    let conn = crate::database::init::get_connection_shared()?;
    let repo = ApiProviderRepository::with_conn(conn);

    // 创建或更新提供商
    let mut provider = if let Some(id) = request.id {
        // 更新现有提供商
        #[cfg(debug_assertions)]
        eprintln!("[cmd_save_provider] 编辑模式: provider_id={}", id);
        eprintln!("[cmd_save_provider] request.base_url={}", request.base_url);

        let existing = repo
            .get_provider_by_id(id)?
            .ok_or_else(|| anyhow::anyhow!("提供商不存在 (id={})", id))?;

        #[cfg(debug_assertions)]
        eprintln!("[cmd_save_provider] existing.base_url={}", existing.base_url);

        let updated = ApiProvider {
            id: existing.id,
            provider_type: request.provider_type,
            name: request.name,
            base_url: request.base_url,
            api_key_ref: existing.api_key_ref,
            model: request.model,
            config_json: request.config_json,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            is_active: request.is_active,
            aliases: existing.aliases,
        };

        #[cfg(debug_assertions)]
        eprintln!("[cmd_save_provider] updated.base_url={}", updated.base_url);

        updated
    } else {
        // 创建新提供商
        let mut new_provider = ApiProvider::new(
            request.provider_type,
            request.name.clone(),
            Some(request.base_url.clone()),
        );
        new_provider.model = request.model;
        new_provider.temperature = request.temperature;
        new_provider.max_tokens = request.max_tokens;

        // 先插入数据库获取 ID
        let mut created = repo.create_provider(new_provider)?;

        // 设置 api_key_ref
        if request.provider_type.requires_api_key() {
            created.api_key_ref = Some(format!("provider_{}", created.id.unwrap()));
        }

        created
    };

    // 处理 API Key
    if let Some(api_key_str) = request.api_key {
        if !api_key_str.is_empty() {
            let provider_id = provider
                .id
                .ok_or_else(|| anyhow::anyhow!("提供商 ID 无效"))?;

            #[cfg(debug_assertions)]
            eprintln!(
                "[cmd_save_provider] Saving API Key for provider_id={}",
                provider_id
            );

            // 验证 API Key 格式
            let api_key = SecretString::new(api_key_str.into());
            crate::llm::security::ApiKeyValidator::validate_format(
                &api_key,
                provider.provider_type,
            )?;

            // 保存到 keyring
            ApiKeyStorage::save_api_key(provider_id, api_key)?;

            #[cfg(debug_assertions)]
            eprintln!("[cmd_save_provider] API Key saved successfully");

            // 更新 api_key_ref
            if provider.api_key_ref.is_none() {
                provider.api_key_ref = Some(format!("provider_{}", provider_id));
            }
        }
    }

    // 如果 is_active 为 true，需要先设置其他提供商为非活跃
    if provider.is_active {
        // 获取当前活跃的提供商
        if let Ok(Some(active)) = repo.get_active_provider() {
            if active.id != provider.id {
                // 取消之前的活跃状态
                let mut inactive = active.clone();
                inactive.is_active = false;
                repo.update_provider(&inactive)?;
            }
        }
    }

    // 保存/更新到数据库
    if provider.id.is_some() {
        #[cfg(debug_assertions)]
        eprintln!("[cmd_save_provider] 更新到数据库: base_url={}", provider.base_url);

        let rows_affected = repo.update_provider(&provider)?;

        #[cfg(debug_assertions)]
        eprintln!("[cmd_save_provider] 数据库更新完成，影响行数: {}", rows_affected);
    } else {
        provider = repo.create_provider(provider)?;
    }

    #[cfg(debug_assertions)]
    eprintln!("[cmd_save_provider] 最终返回的提供商: base_url={}", provider.base_url);

    Ok(provider)
}

/// 删除 API 提供商
///
/// # 流程
/// 1. 从 keyring 删除 API Key
/// 2. 从数据库删除提供商记录
#[tauri::command]
pub fn cmd_delete_provider(
    _manager: State<'_, LLMClientManager>,
    id: i64,
) -> std::result::Result<(), CommandError> {
    // 先从 keyring 删除 API Key
    ApiKeyStorage::delete_api_key(id)?;

    // 从数据库删除提供商
    let conn = crate::database::init::get_connection_shared()?;
    let repo = ApiProviderRepository::with_conn(conn);
    repo.delete_provider(id)?;

    Ok(())
}

/// 设置活跃提供商
///
/// # 流程
/// 1. 将指定的提供商设置为活跃
/// 2. 数据库触发器会自动将其他提供商设置为非活跃
#[tauri::command]
pub fn cmd_set_active_provider(
    manager: State<'_, LLMClientManager>,
    id: i64,
) -> std::result::Result<(), CommandError> {
    #[cfg(debug_assertions)]
    eprintln!("[cmd_set_active_provider] 切换活跃提供商，provider_id={}", id);

    manager.switch_provider(id)?;

    #[cfg(debug_assertions)]
    eprintln!("[cmd_set_active_provider] 切换成功");

    Ok(())
}

/// 测试提供商连接
///
/// # 流程
/// 1. 获取提供商配置
/// 2. 创建对应的客户端
/// 3. 发送测试请求
///
/// # 返回
/// 返回 TestConnectionResult 包含详细的成功/失败信息
#[tauri::command]
pub async fn cmd_test_provider_connection(
    manager: State<'_, LLMClientManager>,
    id: i64,
) -> std::result::Result<TestConnectionResult, CommandError> {
    let result = manager.test_provider(id).await?;
    Ok(result)
}

// ==================== Token 计数器命令 ====================

/// Token 计数请求参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CountTokensRequest {
    /// 要计算的文本内容
    pub text: String,

    /// 模型名称（可选，用于自动选择编码类型）
    pub model: Option<String>,

    /// 手动指定编码类型（优先级高于 model）
    pub encoding_type: Option<String>,
}

/// Token 计数响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CountTokensResponse {
    /// Token 数量
    pub token_count: usize,

    /// 使用的编码类型
    pub encoding_type: String,

    /// 模型名称（如果提供）
    pub model: Option<String>,
}

/// 计算文本的 Token 数量
///
/// # 功能
/// 使用 tiktoken-rs 库准确计算文本的 Token 数量，支持多种 LLM 模型的编码方式
///
/// # 参数
/// * `text` - 要计算的文本内容
/// * `model` - 模型名称（可选，用于自动选择编码类型，如 "gpt-4"、"gpt-3.5-turbo"）
/// * `encoding_type` - 手动指定编码类型（可选，优先级高于 model）
///   - "cl100k_base": GPT-4, GPT-3.5-Turbo（最新版本）
///   - "p50k_base": GPT-3.5-Turbo（旧版本）
///   - "r50k_base": GPT-2 系列, GPT-3 davinci 系列
///   - "gpt2": 旧版 GPT-3
///
/// # 返回
/// 返回 Token 数量和使用的编码类型信息
///
/// # 示例
/// ```javascript
/// // 前端调用示例
/// const result = await invoke('count_prompt_tokens', {
///   text: 'Hello, world!',
///   model: 'gpt-4'
/// });
/// console.log(result.tokenCount); // 4
/// ```
#[tauri::command]
pub fn count_prompt_tokens(
    request: CountTokensRequest,
) -> std::result::Result<CountTokensResponse, CommandError> {
    // 创建 Token 计数器
    let counter = if let Some(encoding) = request.encoding_type {
        // 优先使用手动指定的编码类型
        let encoding_type = match encoding.to_lowercase().as_str() {
            "cl100k_base" => TokenEncodingType::Cl100kBase,
            "p50k_base" => TokenEncodingType::P50kBase,
            "r50k_base" => TokenEncodingType::R50kBase,
            "gpt2" => TokenEncodingType::Gpt2,
            _ => {
                return Err(CommandError {
                    message: format!("不支持的编码类型: {}", encoding),
                });
            }
        };
        TokenCounter::with_encoding(encoding_type)?
    } else if let Some(model) = &request.model {
        // 使用模型名称自动选择编码类型
        TokenCounter::from_model(model)?
    } else {
        // 默认使用 cl100k_base（GPT-4 / GPT-3.5-Turbo 最新版本）
        TokenCounter::new()?
    };

    // 计算 Token 数量
    let token_count = counter.count_tokens(&request.text)?;

    // 获取编码类型名称
    let encoding_type_name = counter.encoding_type().encoding_name().to_string();

    Ok(CountTokensResponse {
        token_count,
        encoding_type: encoding_type_name,
        model: request.model,
    })
}

/// 扫描会话响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMeta {
    pub session_id: String,
    pub project_path: String,
    pub project_name: String,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: usize,
    pub is_active: bool,
}

/// 扫描 Claude Code 会话文件
///
/// 扫描用户配置的监控目录列表，查找所有会话文件并提取元数据
#[tauri::command]
pub async fn scan_sessions(
    _manager: State<'_, LLMClientManager>,
) -> std::result::Result<Vec<SessionMeta>, CommandError> {
    use crate::database::repository::{MonitoredDirectoryRepository, SessionRepository};
    use crate::monitor::scanner;

    // 获取用户配置的监控目录列表
    let dir_repo = MonitoredDirectoryRepository::from_default_db().map_err(|e| CommandError {
        message: format!("创建目录仓库失败: {}", e),
    })?;

    let directories = dir_repo
        .get_active_directories()
        .map_err(|e| CommandError {
            message: format!("获取监控目录失败: {}", e),
        })?;

    // 如果没有配置任何监控目录，返回空列表
    if directories.is_empty() {
        return Ok(Vec::new());
    }

    // 扫描所有配置的监控目录
    let mut all_sessions = Vec::new();
    for directory in directories {
        let path = std::path::PathBuf::from(&directory.path);
        match scanner::scan_directory(&path) {
            Ok(mut sessions) => {
                all_sessions.append(&mut sessions);
            }
            Err(e) => {
                eprintln!("警告: 扫描目录 {} 失败: {}", directory.path, e);
                // 继续扫描其他目录
            }
        }
    }

    // 获取数据库连接并创建 SessionRepository
    let conn = crate::database::init::get_connection_shared().map_err(|e| CommandError {
        message: format!("获取数据库连接失败: {}", e),
    })?;
    let session_repo = SessionRepository::with_conn(conn);

    // 将扫描结果存入数据库
    for metadata in &all_sessions {
        let file_path = metadata.file_path.to_string_lossy().to_string();
        let _ = session_repo.upsert_session(
            &metadata.session_id,
            &metadata.project_path,
            &metadata.project_name,
            &file_path,
            metadata.is_active,
        );
    }

    // 转换为返回格式
    let result: Vec<SessionMeta> = all_sessions
        .into_iter()
        .map(|m| SessionMeta {
            session_id: m.session_id,
            project_path: m.project_path,
            project_name: m.project_name,
            created_at: m.created_at,
            updated_at: m.updated_at,
            message_count: m.message_count,
            is_active: m.is_active,
        })
        .collect();

    Ok(result)
}

/// 扫描指定目录的 Claude Code 会话文件
///
/// 扫描用户选择的目录，查找所有 .jsonl 会话文件并提取元数据
///
/// # 参数
/// - `directory`: 要扫描的目录路径
///
/// # 返回
/// 返回会话元数据列表
#[tauri::command]
pub async fn scan_directory(
    directory: String,
) -> std::result::Result<Vec<SessionMeta>, CommandError> {
    use crate::database::repository::SessionRepository;
    use crate::monitor::scanner;

    let path = PathBuf::from(&directory);

    // 验证目录存在
    if !path.exists() {
        return Err(CommandError {
            message: format!("目录不存在: {}", directory),
        });
    }

    if !path.is_dir() {
        return Err(CommandError {
            message: format!("路径不是目录: {}", directory),
        });
    }

    // 扫描指定目录的会话文件
    let sessions_metadata = scanner::scan_directory(&path).map_err(|e| CommandError {
        message: format!("扫描目录失败: {}", e),
    })?;

    // 获取数据库连接并创建 SessionRepository
    let conn = crate::database::init::get_connection_shared().map_err(|e| CommandError {
        message: format!("获取数据库连接失败: {}", e),
    })?;
    let session_repo = SessionRepository::with_conn(conn);

    // 将扫描结果存入数据库
    for metadata in &sessions_metadata {
        let file_path = metadata.file_path.to_string_lossy().to_string();
        let _ = session_repo.upsert_session(
            &metadata.session_id,
            &metadata.project_path,
            &metadata.project_name,
            &file_path,
            metadata.is_active,
        );
    }

    // 转换为返回格式
    let result: Vec<SessionMeta> = sessions_metadata
        .into_iter()
        .map(|m| SessionMeta {
            session_id: m.session_id,
            project_path: m.project_path,
            project_name: m.project_name,
            created_at: m.created_at,
            updated_at: m.updated_at,
            message_count: m.message_count,
            is_active: m.is_active,
        })
        .collect();

    Ok(result)
}

// ==================== 性能基准测试命令 ====================

/// 性能测试结果响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkResultResponse {
    /// 测试名称
    pub name: String,
    /// 耗时（毫秒）
    pub duration_ms: f64,
    /// 是否通过阈值
    pub passed: bool,
    /// 阈值（毫秒）
    pub threshold_ms: f64,
    /// 详细信息
    pub details: String,
}

/// 性能测试报告响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkReportResponse {
    /// 测试时间戳
    pub timestamp: String,
    /// 测试结果列表
    pub results: Vec<BenchmarkResultResponse>,
    /// 总体是否通过
    pub overall_passed: bool,
    /// Markdown 格式的报告
    pub markdown_report: String,
}

impl From<BenchmarkResult> for BenchmarkResultResponse {
    fn from(result: BenchmarkResult) -> Self {
        Self {
            name: result.name,
            duration_ms: result.duration_ms,
            passed: result.passed,
            threshold_ms: result.threshold_ms,
            details: result.details,
        }
    }
}

impl From<BenchmarkReport> for BenchmarkReportResponse {
    fn from(report: BenchmarkReport) -> Self {
        // 先生成 markdown 报告，避免所有权移动
        let markdown = report.to_markdown();
        Self {
            timestamp: report.timestamp,
            results: report.results.into_iter().map(Into::into).collect(),
            overall_passed: report.overall_passed,
            markdown_report: markdown,
        }
    }
}

/// 运行性能基准测试
///
/// 执行以下测试：
/// - 应用启动时间 (< 3000ms)
/// - 会话扫描时间 (< 2000ms for 100 sessions)
/// - 数据库查询性能 (< 100ms)
///
/// # 返回
/// 返回完整的性能测试报告
#[tauri::command]
pub fn run_benchmarks(
    _manager: State<'_, LLMClientManager>,
) -> std::result::Result<BenchmarkReportResponse, CommandError> {
    // 运行所有性能测试
    let report = run_all_benchmarks_internal();

    // 打印报告到控制台
    println!("\n{}", report.to_markdown());

    // 保存报告到文件
    let output_dir = std::path::PathBuf::from("dev_plans/plan1/logs");
    let output_path = output_dir.join(format!(
        "benchmark_report_{}.json",
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    ));

    if let Err(e) = save_benchmark_report_internal(&report, &output_path) {
        eprintln!("警告: 保存性能测试报告失败: {}", e);
    }

    Ok(report.into())
}

// ==================== 消息树解析命令 ====================

/// 解析会话文件响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseSessionResponse {
    /// 会话 ID
    pub session_id: String,

    /// 消息树
    pub tree: ConversationTree,

    /// 解析耗时（毫秒）
    pub parse_duration_ms: f64,

    /// 消息总数
    pub message_count: usize,

    /// 最大深度
    pub max_depth: usize,
}

/// 解析会话文件并构建消息树
///
/// # 功能
/// 从 Claude Code 的 JSONL 会话文件解析消息内容，并基于 parentUuid 字段构建嵌套的消息树结构。
///
/// # 参数
/// * `file_path` - JSONL 会话文件的完整路径
///
/// # 返回
/// 返回完整的消息树结构，包含所有消息的嵌套关系
///
/// # 算法特点
/// - **迭代算法**：使用迭代而非递归构建树，避免深层嵌套导致栈溢出
/// - **根节点识别**：自动识别 User 消息作为根节点
/// - **深度计算**：自动计算每个节点的树深度
///
/// # 示例
/// ```javascript
/// // 前端调用示例
/// const result = await invoke('parse_session_tree', {
///   filePath: 'C:/Users/xxx/.claude/projects/xxx/sessions/xxx.jsonl'
/// });
/// console.log(result.tree.roots[0].children); // 访问子消息
/// ```
#[tauri::command]
pub async fn parse_session_tree(
    file_path: String,
) -> std::result::Result<ParseSessionResponse, CommandError> {
    let path = PathBuf::from(&file_path);

    // 验证文件存在
    if !path.exists() {
        return Err(CommandError {
            message: format!("文件不存在: {}", file_path),
        });
    }

    let start = std::time::Instant::now();

    // 创建 JSONL 解析器并解析所有条目
    let mut parser = JsonlParser::new(path).map_err(|e| CommandError {
        message: format!("创建 JSONL 解析器失败: {}", e),
    })?;

    let entries = parser.parse_all().map_err(|e| CommandError {
        message: format!("解析 JSONL 文件失败: {}", e),
    })?;

    // 构建消息树
    let tree = MessageTreeBuilder::build_from_entries(&entries).map_err(|e| CommandError {
        message: format!("构建消息树失败: {}", e),
    })?;

    let duration = start.elapsed();

    // 提取会话 ID（从文件路径或第一条消息）
    let session_id = extract_session_id(&file_path);

    let message_count = tree.total_count;
    let max_depth = tree.max_depth;

    Ok(ParseSessionResponse {
        session_id,
        tree,
        parse_duration_ms: duration.as_secs_f64() * 1000.0,
        message_count,
        max_depth,
    })
}

/// 从文件路径提取会话 ID
fn extract_session_id(file_path: &str) -> String {
    // 尝试从文件路径中提取 UUID
    if let Some(filename) = PathBuf::from(file_path).file_stem() {
        if let Some(name) = filename.to_str() {
            // 如果文件名看起来像 UUID，直接使用
            if name.len() == 36 && name.chars().filter(|&c| c == '-').count() == 4 {
                return name.to_string();
            }
        }
    }

    // 否则使用文件名作为 ID
    PathBuf::from(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()
}

// ==================== 会话评分与标签命令 ====================

/// 设置会话评分请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetSessionRatingRequest {
    /// 会话 ID
    pub session_id: String,
    /// 评分 (1-5)，null 表示清除评分
    pub rating: Option<i32>,
}

/// 设置会话标签请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetSessionTagsRequest {
    /// 会话 ID
    pub session_id: String,
    /// 标签数组
    pub tags: Vec<String>,
}

/// 会话评分和标签响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadataResponse {
    /// 是否成功
    pub success: bool,
    /// 更新的行数
    pub rows_affected: usize,
    /// 消息
    pub message: String,
}

/// 设置会话评分
///
/// 为会话设置 1-5 星评分，或清除评分。
///
/// # 参数
/// - `request`: 包含 session_id 和 rating (1-5 或 null) 的请求
///
/// # 返回
/// 返回操作结果
///
/// # 示例
/// ```javascript
/// // 设置 5 星评分
/// await invoke('set_session_rating', {
///   sessionId: 'uuid-xxx',
///   rating: 5
/// });
///
/// // 清除评分
/// await invoke('set_session_rating', {
///   sessionId: 'uuid-xxx',
///   rating: null
/// });
/// ```
#[tauri::command]
pub async fn set_session_rating(
    request: SetSessionRatingRequest,
) -> std::result::Result<SessionMetadataResponse, CommandError> {
    let conn = crate::database::init::get_connection_shared().map_err(|e| CommandError {
        message: format!("获取数据库连接失败: {}", e),
    })?;

    let repo = crate::database::repository::SessionRepository::with_conn(conn);

    let rows_affected = repo
        .set_session_rating(&request.session_id, request.rating)
        .map_err(|e| CommandError {
            message: format!("设置会话评分失败: {}", e),
        })?;

    let message = if rows_affected > 0 {
        format!(
            "会话评分已{}",
            request
                .rating
                .map(|r| format!("更新为 {} 星", r))
                .unwrap_or_else(|| "清除".to_string())
        )
    } else {
        "会话不存在".to_string()
    };

    Ok(SessionMetadataResponse {
        success: rows_affected > 0,
        rows_affected,
        message,
    })
}

/// 设置会话标签
///
/// 为会话设置标签数组，或清空标签。
///
/// # 参数
/// - `request`: 包含 session_id 和 tags (字符串数组) 的请求
///
/// # 返回
/// 返回操作结果
///
/// # 示例
/// ```javascript
/// // 设置标签
/// await invoke('set_session_tags', {
///   sessionId: 'uuid-xxx',
///   tags: ['bugfix', 'ui', 'feature']
/// });
///
/// // 清空标签
/// await invoke('set_session_tags', {
///   sessionId: 'uuid-xxx',
///   tags: []
/// });
/// ```
#[tauri::command]
pub async fn set_session_tags(
    request: SetSessionTagsRequest,
) -> std::result::Result<SessionMetadataResponse, CommandError> {
    let conn = crate::database::init::get_connection_shared().map_err(|e| CommandError {
        message: format!("获取数据库连接失败: {}", e),
    })?;

    let repo = crate::database::repository::SessionRepository::with_conn(conn);

    // 克隆 tags 以便后续使用
    let tags_clone = request.tags.clone();
    let rows_affected = repo
        .set_session_tags(&request.session_id, request.tags)
        .map_err(|e| CommandError {
            message: format!("设置会话标签失败: {}", e),
        })?;

    let message = if rows_affected > 0 {
        format!("会话标签已更新为: {}", tags_clone.join(", "))
    } else {
        "会话不存在".to_string()
    };

    Ok(SessionMetadataResponse {
        success: rows_affected > 0,
        rows_affected,
        message,
    })
}

/// 获取会话评分
///
/// 获取会话的当前评分。
///
/// # 参数
/// - `session_id`: 会话 ID
///
/// # 返回
/// 返回评分值 (1-5)，null 表示未评分
///
/// # 示例
/// ```javascript
/// const rating = await invoke('get_session_rating', {
///   sessionId: 'uuid-xxx'
/// });
/// console.log(rating); // 5 或 null
/// ```
#[tauri::command]
pub async fn get_session_rating(
    session_id: String,
) -> std::result::Result<Option<i32>, CommandError> {
    let conn = crate::database::init::get_connection_shared().map_err(|e| CommandError {
        message: format!("获取数据库连接失败: {}", e),
    })?;

    let repo = crate::database::repository::SessionRepository::with_conn(conn);

    repo.get_session_rating(&session_id)
        .map_err(|e| CommandError {
            message: format!("获取会话评分失败: {}", e),
        })
}

/// 获取会话标签
///
/// 获取会话的当前标签列表。
///
/// # 参数
/// - `session_id`: 会话 ID
///
/// # 返回
/// 返回标签数组
///
/// # 示例
/// ```javascript
/// const tags = await invoke('get_session_tags', {
///   sessionId: 'uuid-xxx'
/// });
/// console.log(tags); // ['bugfix', 'ui', 'feature']
/// ```
#[tauri::command]
pub async fn get_session_tags(
    session_id: String,
) -> std::result::Result<Vec<String>, CommandError> {
    let conn = crate::database::init::get_connection_shared().map_err(|e| CommandError {
        message: format!("获取数据库连接失败: {}", e),
    })?;

    let repo = crate::database::repository::SessionRepository::with_conn(conn);

    repo.get_session_tags(&session_id)
        .map_err(|e| CommandError {
            message: format!("获取会话标签失败: {}", e),
        })
}

// ==================== 会话归档命令 ====================

/// 归档会话
///
/// 将会话标记为已归档，归档后的会话不会在默认列表中显示，但仍可通过搜索找到。
///
/// # 参数
/// - `session_id`: 会话 ID
///
/// # 返回
/// 返回操作结果
///
/// # 示例
/// ```javascript
/// await invoke('archive_session', {
///   sessionId: 'uuid-xxx'
/// });
/// ```
#[tauri::command]
pub async fn archive_session(
    session_id: String,
) -> std::result::Result<SessionMetadataResponse, CommandError> {
    let conn = crate::database::init::get_connection_shared().map_err(|e| CommandError {
        message: format!("获取数据库连接失败: {}", e),
    })?;

    let repo = crate::database::repository::SessionRepository::with_conn(conn);

    let rows_affected = repo
        .archive_session(&session_id)
        .map_err(|e| CommandError {
            message: format!("归档会话失败: {}", e),
        })?;

    let message = if rows_affected > 0 {
        "会话已归档".to_string()
    } else {
        "会话不存在".to_string()
    };

    Ok(SessionMetadataResponse {
        success: rows_affected > 0,
        rows_affected,
        message,
    })
}

/// 取消归档会话
///
/// 将已归档的会话恢复到默认列表。
///
/// # 参数
/// - `session_id`: 会话 ID
///
/// # 返回
/// 返回操作结果
///
/// # 示例
/// ```javascript
/// await invoke('unarchive_session', {
///   sessionId: 'uuid-xxx'
/// });
/// ```
#[tauri::command]
pub async fn unarchive_session(
    session_id: String,
) -> std::result::Result<SessionMetadataResponse, CommandError> {
    let conn = crate::database::init::get_connection_shared().map_err(|e| CommandError {
        message: format!("获取数据库连接失败: {}", e),
    })?;

    let repo = crate::database::repository::SessionRepository::with_conn(conn);

    let rows_affected = repo
        .unarchive_session(&session_id)
        .map_err(|e| CommandError {
            message: format!("取消归档会话失败: {}", e),
        })?;

    let message = if rows_affected > 0 {
        "会话已恢复到活跃列表".to_string()
    } else {
        "会话不存在".to_string()
    };

    Ok(SessionMetadataResponse {
        success: rows_affected > 0,
        rows_affected,
        message,
    })
}

/// 获取已归档的会话列表
///
/// 返回所有已归档的会话，按更新时间倒序排列。
///
/// # 返回
/// 返回已归档的会话列表
///
/// # 示例
/// ```javascript
/// const archivedSessions = await invoke('get_archived_sessions');
/// console.log(archivedSessions); // Session 对象数组
/// ```
#[tauri::command]
pub async fn get_archived_sessions(
) -> std::result::Result<Vec<crate::database::models::Session>, CommandError> {
    let conn = crate::database::init::get_connection_shared().map_err(|e| CommandError {
        message: format!("获取数据库连接失败: {}", e),
    })?;

    let repo = crate::database::repository::SessionRepository::with_conn(conn);

    repo.get_archived_sessions().map_err(|e| CommandError {
        message: format!("获取已归档会话列表失败: {}", e),
    })
}

// ==================== 文件监控命令 ====================

/// 启动文件监控响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartWatcherResponse {
    /// 是否成功启动
    pub success: bool,
    /// 消息
    pub message: String,
}

/// 启动文件监控
///
/// 启动 Claude 会话文件的实时监控，检测文件变更后推送事件到前端。
///
/// # 返回
/// 返回启动结果
///
/// # 前端事件
/// 启动后，会收到 `sessions-changed` 事件：
/// ```javascript
/// import { listen } from '@tauri-apps/api/event';
///
/// listen('sessions-changed', (event) => {
///     console.log('会话文件变更:', event.payload);
///     // { kind: 'created', path: 'xxx.jsonl', isJsonl: true, timestamp: '...' }
///     // 重新加载会话列表
/// });
/// ```
///
/// # 示例
/// ```javascript
/// await invoke('start_file_watcher');
/// ```
#[tauri::command]
pub async fn start_file_watcher(
    app_handle: tauri::AppHandle,
) -> std::result::Result<StartWatcherResponse, CommandError> {
    use crate::monitor::watcher::{get_claude_projects_dir, SessionWatcher};

    let projects_dir = get_claude_projects_dir().map_err(|e| CommandError {
        message: format!("获取 Claude 项目目录失败: {}", e),
    })?;

    // 检查目录是否存在
    if !projects_dir.exists() {
        return Ok(StartWatcherResponse {
            success: false,
            message: format!("Claude 项目目录不存在: {:?}", projects_dir),
        });
    }

    // 创建监控器
    let watcher =
        SessionWatcher::new(projects_dir.clone(), app_handle).map_err(|e| CommandError {
            message: format!("创建文件监控器失败: {}", e),
        })?;

    // 启动监控（在后台线程）
    watcher.start().map_err(|e| CommandError {
        message: format!("启动文件监控器失败: {}", e),
    })?;

    Ok(StartWatcherResponse {
        success: true,
        message: format!("文件监控已启动，监控目录: {:?}", projects_dir),
    })
}

// ==================== 日志提取与导出命令 ====================

/// 提取会话日志请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractSessionRequest {
    /// 文件路径
    pub file_path: String,
    /// 提取等级：l1_full_trace, l2_clean_flow, l3_prompt_only
    pub level: String,
}

/// 提取会话日志响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractSessionResponse {
    /// 提取的内容
    pub content: String,
    /// 消息总数
    pub message_count: usize,
    /// 提取等级
    pub level: String,
}

/// 导出会话日志请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSessionRequest {
    /// 文件路径
    pub file_path: String,
    /// 提取等级
    pub level: String,
    /// 导出格式：markdown 或 json
    pub format: String,
    /// 输出目录（可选，默认与输入文件同目录）
    pub output_dir: Option<String>,
}

/// 导出会话日志响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSessionResponse {
    /// 导出的文件路径
    pub output_path: String,
    /// 文件大小（字节）
    pub file_size: u64,
}

/// 提取会话日志
///
/// 根据指定等级提取会话内容，返回纯文本或结构化数据。
///
/// # 参数
/// - `file_path`: JSONL 会话文件路径
/// - `level`: 提取等级
///
/// # 返回
/// 返回提取后的内容
///
/// # 示例
/// ```javascript
/// const result = await invoke('extract_session_log', {
///   filePath: 'C:/Users/xxx/.claude/projects/xxx/sessions/xxx.jsonl',
///   level: 'l2_clean_flow'
/// });
/// console.log(result.content);
/// ```
#[tauri::command]
pub async fn extract_session_log(
    file_path: String,
    level: String,
) -> std::result::Result<ExtractSessionResponse, CommandError> {
    let path = PathBuf::from(&file_path);

    // 验证文件存在
    if !path.exists() {
        return Err(CommandError {
            message: format!("文件不存在: {}", file_path),
        });
    }

    // 解析等级
    let extraction_level = match level.as_str() {
        "l1_full_trace" => ExtractionLevel::L1FullTrace,
        "l2_clean_flow" => ExtractionLevel::L2CleanFlow,
        "l3_prompt_only" => ExtractionLevel::L3PromptOnly,
        _ => {
            return Err(CommandError {
                message: format!(
                    "无效的提取等级: {}，可选值：l1_full_trace, l2_clean_flow, l3_prompt_only",
                    level
                ),
            });
        }
    };

    // 创建 JSONL 解析器并解析
    let mut parser = JsonlParser::new(path).map_err(|e| CommandError {
        message: format!("创建 JSONL 解析器失败: {}", e),
    })?;

    let entries = parser.parse_all().map_err(|e| CommandError {
        message: format!("解析 JSONL 文件失败: {}", e),
    })?;

    // 构建消息树
    let mut tree = MessageTreeBuilder::build_from_entries(&entries).map_err(|e| CommandError {
        message: format!("构建消息树失败: {}", e),
    })?;

    // 提取元数据
    crate::parser::extractor::MetadataExtractor::extract_tree_metadata(&mut tree).map_err(|e| {
        CommandError {
            message: format!("提取元数据失败: {}", e),
        }
    })?;

    // 提取内容
    let content = ExtractionEngine::extract(&tree, extraction_level).map_err(|e| CommandError {
        message: format!("提取会话内容失败: {}", e),
    })?;

    Ok(ExtractSessionResponse {
        content,
        message_count: tree.total_count,
        level: extraction_level.name().to_string(),
    })
}

/// 导出会话日志
///
/// 提取会话内容并导出为文件（Markdown 或 JSON）。
///
/// # 参数
/// - `file_path`: JSONL 会话文件路径
/// - `level`: 提取等级
/// - `format`: 导出格式（markdown 或 json）
/// - `output_dir`: 输出目录（可选）
///
/// # 返回
/// 返回导出文件的路径和大小
///
/// # 示例
/// ```javascript
/// const result = await invoke('export_session_log', {
///   filePath: 'C:/Users/xxx/.claude/projects/xxx/sessions/xxx.jsonl',
///   level: 'l2_clean_flow',
///   format: 'markdown'
/// });
/// console.log(result.output_path);
/// ```
#[tauri::command]
pub async fn export_session_log(
    file_path: String,
    level: String,
    format: String,
    output_dir: Option<String>,
) -> std::result::Result<ExportSessionResponse, CommandError> {
    let path = PathBuf::from(&file_path);

    // 验证文件存在
    if !path.exists() {
        return Err(CommandError {
            message: format!("文件不存在: {}", file_path),
        });
    }

    // 解析等级
    let extraction_level = match level.as_str() {
        "l1_full_trace" => ExtractionLevel::L1FullTrace,
        "l2_clean_flow" => ExtractionLevel::L2CleanFlow,
        "l3_prompt_only" => ExtractionLevel::L3PromptOnly,
        _ => {
            return Err(CommandError {
                message: format!("无效的提取等级: {}", level),
            });
        }
    };

    // 解析导出格式
    let export_format = match format.as_str() {
        "markdown" => ExportFormat::Markdown,
        "json" => ExportFormat::Json,
        _ => {
            return Err(CommandError {
                message: format!("无效的导出格式: {}，可选值：markdown, json", format),
            });
        }
    };

    // 确定输出目录
    let output_dir = if let Some(dir) = output_dir {
        PathBuf::from(dir)
    } else {
        // 默认使用输入文件的父目录
        path.parent()
            .ok_or_else(|| CommandError {
                message: "无法确定输出目录".to_string(),
            })?
            .to_path_buf()
    };

    // 确保输出目录存在
    fs::create_dir_all(&output_dir).map_err(|e| CommandError {
        message: format!("创建输出目录失败: {}", e),
    })?;

    // 创建 JSONL 解析器并解析
    let mut parser = JsonlParser::new(path.clone()).map_err(|e| CommandError {
        message: format!("创建 JSONL 解析器失败: {}", e),
    })?;

    let entries = parser.parse_all().map_err(|e| CommandError {
        message: format!("解析 JSONL 文件失败: {}", e),
    })?;

    // 构建消息树
    let mut tree = MessageTreeBuilder::build_from_entries(&entries).map_err(|e| CommandError {
        message: format!("构建消息树失败: {}", e),
    })?;

    // 提取元数据
    crate::parser::extractor::MetadataExtractor::extract_tree_metadata(&mut tree).map_err(|e| {
        CommandError {
            message: format!("提取元数据失败: {}", e),
        }
    })?;

    // 确定输出文件名
    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("session");

    let ext = match export_format {
        ExportFormat::Markdown => "md",
        ExportFormat::Json => "json",
    };

    let output_path = output_dir.join(format!("{}_{}.{}", file_stem, level, ext));

    // 导出文件
    match export_format {
        ExportFormat::Markdown => {
            let content =
                ExtractionEngine::extract(&tree, extraction_level).map_err(|e| CommandError {
                    message: format!("提取会话内容失败: {}", e),
                })?;

            ExtractionEngine::export_markdown(&content, &output_path).map_err(|e| {
                CommandError {
                    message: format!("导出 Markdown 文件失败: {}", e),
                }
            })?;
        }
        ExportFormat::Json => {
            ExtractionEngine::export_json(&tree, extraction_level, &output_path).map_err(|e| {
                CommandError {
                    message: format!("导出 JSON 文件失败: {}", e),
                }
            })?;
        }
    }

    // 获取文件大小
    let file_size = fs::metadata(&output_path)
        .map_err(|e| CommandError {
            message: format!("获取文件信息失败: {}", e),
        })?
        .len();

    Ok(ExportSessionResponse {
        output_path: output_path.to_string_lossy().to_string(),
        file_size,
    })
}

// ==================== 向量相似度检索命令 ====================

/// 向量搜索请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchRequest {
    /// 查询文本
    pub query: String,
    /// 返回结果数量上限（默认 5）
    #[serde(rename = "limit")]
    pub limit: Option<usize>,
    /// 是否使用评分加权排序（默认 false）
    ///
    /// 当启用时：
    /// - 结合相似度和用户评分的混合排序
    /// - 公式：weighted_score = 0.7 * cosine_similarity + 0.3 * (rating / 5.0)
    /// - 5 星会话在相似度稍低时仍能排在前面
    /// - 排除低分会话（rating < 2）和归档会话
    #[serde(rename = "weighted")]
    pub weighted: Option<bool>,
}

/// 向量相似度搜索
///
/// 根据查询文本检索最相似的历史会话。
///
/// # 参数
/// - `request`: 包含查询文本、结果数量限制和是否使用加权排序
///
/// # 返回
/// 返回按相似度或加权分数排序的会话搜索结果列表
///
/// # 功能
/// - 使用 BGE-small-en-v1.5 生成查询向量
/// - 使用 sqlite-vec 的 distance 函数计算余弦相似度
/// - 自动合并同一会话的多条匹配消息
/// - 支持评分加权排序（提升优质会话优先级）
///
/// # 加权模式
/// 当 `weighted = true` 时：
/// - 结合相似度和用户评分混合排序
/// - 公式：weighted_score = 0.7 * cosine_similarity + 0.3 * (rating / 5.0)
/// - 5 星会话在相似度稍低时仍能排在前面
/// - 自动排除低分会话（rating < 2）和归档会话
///
/// # 示例
/// ```javascript
/// // 纯相似度排序
/// const results = await invoke('vector_search', {
///   query: '如何实现文件上传功能',
///   limit: 5,
///   weighted: false  // 或省略，默认 false
/// });
///
/// // 评分加权排序
/// const weightedResults = await invoke('vector_search', {
///   query: '实现用户登录',
///   limit: 5,
///   weighted: true  // 启用加权，5 星优质会话优先
/// });
///
/// // 结果格式
/// // [
/// //   {
/// //     session: { session_id: '...', project_name: '...', rating: 5, ... },
/// //     similarityScore: 0.23,
/// //     summary: '实现文件上传...'
/// //   },
/// //   ...
/// // ]
/// ```
#[tauri::command]
pub async fn vector_search(
    request: VectorSearchRequest,
) -> std::result::Result<Vec<crate::database::models::VectorSearchResult>, CommandError> {
    use crate::database::init::get_connection_shared;
    use crate::database::repository::SessionRepository;
    use crate::embedding::EmbeddingGenerator;

    // 参数验证
    let query = request.query.trim();
    if query.is_empty() {
        return Err(CommandError {
            message: "查询文本不能为空".to_string(),
        });
    }

    let limit = request.limit.unwrap_or(5).min(20); // 最多返回 20 条
    let use_weighted = request.weighted.unwrap_or(false); // 默认不使用加权

    // 生成查询向量
    let generator = EmbeddingGenerator::new().map_err(|e| CommandError {
        message: format!("初始化向量生成器失败: {}", e),
    })?;

    let query_embedding = generator
        .generate_for_message(query)
        .map_err(|e| CommandError {
            message: format!("生成查询向量失败: {}", e),
        })?;

    // 检查是否使用占位符实现
    if generator.is_placeholder() {
        eprintln!("警告: 当前使用占位符向量实现，搜索结果可能不准确");
    }

    // 执行向量检索
    let conn = get_connection_shared().map_err(|e| CommandError {
        message: format!("获取数据库连接失败: {}", e),
    })?;

    let repo = SessionRepository::with_conn(conn);

    // 根据加权参数选择检索方法
    let results = if use_weighted {
        repo.weighted_vector_search_sessions(&query_embedding, limit)
            .map_err(|e| CommandError {
                message: format!("加权向量检索失败: {}", e),
            })?
    } else {
        repo.vector_search_sessions(&query_embedding, limit)
            .map_err(|e| CommandError {
                message: format!("向量检索失败: {}", e),
            })?
    };

    Ok(results)
}

// ==================== 上下文压缩命令 ====================

/// 上下文压缩请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompressContextRequest {
    /// 消息的 JSON 数组字符串
    pub messages_json: String,
}

/// 压缩上下文
///
/// 压缩会话消息以减少 Token 使用量，去除冗余信息（thinking、工具输出等）
/// 保留关键决策点和代码变更
#[tauri::command]
pub async fn compress_context(
    request: CompressContextRequest,
) -> Result<CompressionResult, CommandError> {
    use crate::optimizer::compressor::ContextCompressor;

    // 创建压缩器
    let compressor = ContextCompressor::new().map_err(|e| CommandError {
        message: format!("创建压缩器失败: {}", e),
    })?;

    // 执行压缩
    let result = compressor
        .compress_session(&request.messages_json)
        .map_err(|e| CommandError {
            message: format!("压缩失败: {}", e),
        })?;

    Ok(result)
}

// ==================== 增强提示词生成命令 ====================

/// 优化提示词
///
/// 整合向量检索、上下文压缩和 LLM 生成，创建增强的提示词
///
/// # 参数
/// - `request`: 增强提示词请求
/// - `language`: 语言标识（"zh" 或 "en"），可选，默认 "en"
/// - `llm_manager`: LLM 客户端管理器
#[tauri::command]
pub async fn optimize_prompt(
    request: EnhancedPromptRequest,
    language: Option<String>,
    llm_manager: State<'_, LLMClientManager>,
) -> Result<EnhancedPrompt, CommandError> {
    use crate::optimizer::prompt_generator::PromptGenerator;

    // 设置默认语言为英文
    let language = language.unwrap_or_else(|| "en".to_string());

    // 调试：输出收到的请求（仅开发环境）
    #[cfg(debug_assertions)]
    {
        eprintln!("[optimize_prompt] 收到请求:");
        eprintln!("  goal: {}", request.goal);
        // 生产环境脱敏路径：仅显示文件名
        let path_hint = if let Some(ref path_str) = request.current_session_file_path {
            let path = std::path::Path::new(path_str);
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("<无文件>")
        } else {
            "<无文件>"
        };
        eprintln!("  current_session_file_path: {}", path_hint);
        eprintln!("  language: {}", language);
    }

    // 创建提示词生成器
    let generator = PromptGenerator::new().map_err(|e| CommandError {
        message: format!("创建提示词生成器失败: {}", e),
    })?;

    // 生成增强提示词
    let result = generator
        .generate_enhanced_prompt(request, &llm_manager, &language)
        .await
        .map_err(|e| CommandError {
            message: format!("生成提示词失败: {}", e),
        })?;

    // 调试：输出返回结果（仅开发环境）
    #[cfg(debug_assertions)]
    {
        eprintln!("[optimize_prompt] 返回结果: original_goal={}, enhanced_prompt长度={}, referenced_sessions数量={}",
            result.original_goal,
            result.enhanced_prompt.len(),
            result.referenced_sessions.len()
        );
    }

    Ok(result)
}

// ==================== Meta-Prompt 管理命令 ====================

/// 获取 Meta-Prompt 模板
///
/// 根据类别获取元提示词模板内容
#[tauri::command]
pub fn get_meta_template(category: String) -> Result<String, CommandError> {
    use crate::database::repository::SessionRepository;

    let repo = SessionRepository::from_default_db().map_err(|e| CommandError {
        message: format!("创建仓库失败: {}", e),
    })?;

    repo.get_meta_template(&category).map_err(|e| CommandError {
        message: format!("获取模板失败: {}", e),
    })
}

/// 更新 Meta-Prompt 模板
///
/// 根据类别更新元提示词模板内容
#[tauri::command]
pub fn update_meta_template(category: String, content: String) -> Result<(), CommandError> {
    use crate::database::repository::SessionRepository;

    let repo = SessionRepository::from_default_db().map_err(|e| CommandError {
        message: format!("创建仓库失败: {}", e),
    })?;

    repo.update_meta_template(&category, &content)
        .map_err(|e| CommandError {
            message: format!("更新模板失败: {}", e),
        })?;

    Ok(())
}

// ==================== 优化器配置管理命令 ====================

/// 重新加载优化器配置
///
/// 从 optimizer_config.toml 重新加载配置文件，支持运行时热更新
#[tauri::command]
pub fn reload_optimizer_config() -> Result<String, CommandError> {
    // 创建临时生成器来重新加载配置
    let config_path = std::env::current_dir()
        .map_err(|e| CommandError {
            message: format!("获取当前目录失败: {}", e),
        })?
        .join("src-tauri")
        .join("optimizer_config.toml");

    // 验证配置文件可以成功解析
    let content = std::fs::read_to_string(&config_path).map_err(|e| CommandError {
        message: format!("无法读取配置文件: {}", e),
    })?;

    // 尝试解析以验证配置正确性
    toml::from_str::<toml::Value>(&content).map_err(|e| CommandError {
        message: format!("配置文件解析失败: {}", e),
    })?;

    // 配置验证通过
    eprintln!(
        "[reload_optimizer_config] 配置文件验证成功: {:?}",
        config_path
    );

    // 获取全局配置管理器并重新加载配置
    let manager = crate::optimizer::config::get_config_manager().ok_or_else(|| CommandError {
        message: "配置管理器未初始化".to_string(),
    })?;

    manager.reload().map_err(|e| CommandError {
        message: format!("重新加载配置失败: {}", e),
    })?;

    eprintln!("[reload_optimizer_config] 配置已成功应用到运行时");

    Ok("配置已重新加载".to_string())
}

/// 获取优化器配置
///
/// 返回当前优化器配置的 JSON 表示
#[tauri::command]
pub fn get_optimizer_config() -> Result<String, CommandError> {
    // 使用全局配置管理器
    let manager = crate::optimizer::config::get_config_manager().ok_or_else(|| CommandError {
        message: "配置管理器未初始化".to_string(),
    })?;

    let config = manager.get_config();
    let config_json = serde_json::to_string_pretty(&config).map_err(|e| CommandError {
        message: format!("序列化配置失败: {}", e),
    })?;

    Ok(config_json)
}

// ============================================================================
// 监控目录管理命令 (Wave 2: 手动添加监控目录)
// ============================================================================

/// 获取所有监控目录
///
/// 返回用户配置的所有监控目录列表
#[tauri::command]
pub fn get_monitored_directories(
) -> Result<Vec<crate::database::models::MonitoredDirectory>, CommandError> {
    use crate::database::repository::MonitoredDirectoryRepository;

    let repo = MonitoredDirectoryRepository::from_default_db().map_err(|e| CommandError {
        message: format!("创建目录仓库失败: {}", e),
    })?;

    repo.get_all_directories().map_err(|e| CommandError {
        message: format!("获取监控目录失败: {}", e),
    })
}

/// 添加监控目录
///
/// 添加新的监控目录到配置列表
#[tauri::command]
pub fn add_monitored_directory(
    path: String,
    name: String,
) -> Result<crate::database::models::MonitoredDirectory, CommandError> {
    use crate::database::repository::MonitoredDirectoryRepository;

    let mut repo = MonitoredDirectoryRepository::from_default_db().map_err(|e| CommandError {
        message: format!("创建目录仓库失败: {}", e),
    })?;

    let directory = crate::database::models::MonitoredDirectory::new(path, name);
    repo.create_directory(directory).map_err(|e| CommandError {
        message: format!("添加监控目录失败: {}", e),
    })
}

/// 删除监控目录
///
/// 从配置列表中删除指定的监控目录
#[tauri::command]
pub fn remove_monitored_directory(id: i64) -> Result<(), CommandError> {
    use crate::database::repository::MonitoredDirectoryRepository;

    let repo = MonitoredDirectoryRepository::from_default_db().map_err(|e| CommandError {
        message: format!("创建目录仓库失败: {}", e),
    })?;

    repo.delete_directory(id).map_err(|e| CommandError {
        message: format!("删除监控目录失败: {}", e),
    })?;

    Ok(())
}

/// 切换监控目录的启用状态
///
/// 启用或禁用指定的监控目录
#[tauri::command]
pub fn toggle_monitored_directory(id: i64) -> Result<bool, CommandError> {
    use crate::database::repository::MonitoredDirectoryRepository;

    let mut repo = MonitoredDirectoryRepository::from_default_db().map_err(|e| CommandError {
        message: format!("创建目录仓库失败: {}", e),
    })?;

    let is_active = repo.toggle_directory_active(id).map_err(|e| CommandError {
        message: format!("切换目录状态失败: {}", e),
    })?;

    Ok(is_active)
}

/// 更新监控目录
///
/// 更新监控目录的路径和名称
#[tauri::command]
pub fn update_monitored_directory(
    directory: crate::database::models::MonitoredDirectory,
) -> Result<(), CommandError> {
    use crate::database::repository::MonitoredDirectoryRepository;

    let mut repo = MonitoredDirectoryRepository::from_default_db().map_err(|e| CommandError {
        message: format!("创建目录仓库失败: {}", e),
    })?;

    repo.update_directory(&directory)
        .map_err(|e| CommandError {
            message: format!("更新监控目录失败: {}", e),
        })?;

    Ok(())
}

/// 会话文件信息（返回给前端）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionFileInfo {
    /// 会话 ID（文件名）
    pub session_id: String,
    /// 完整文件路径
    pub file_path: String,
    /// 文件大小（字节）
    pub file_size: u64,
    /// 修改时间（RFC3339）
    pub modified_time: String,
    /// 项目路径（所属监控目录路径）
    #[serde(rename = "projectPath")]
    pub project_path: String,
    /// 会话摘要（从 .jsonl 文件读取，向后兼容）
    #[serde(rename = "summary")]
    pub summary: Option<String>,
    /// 显示名称（智能提取，优先使用）
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    /// 名称来源
    #[serde(rename = "nameSource")]
    pub name_source: Option<String>,
    /// 会话文件类型
    #[serde(rename = "fileType")]
    pub file_type: SessionFileType,
}

/// 获取监控目录对应的会话文件列表（异步版本，包含智能命名和类型筛选）
///
/// 根据监控目录的路径，查找 ~/.claude/projects/ 下对应的会话文件
/// 并使用多级 fallback 策略获取每个会话的显示名称
///
/// # 参数
/// * `monitored_path` - 监控目录路径
/// * `include_agent` - 是否包含 Agent 类型的会话（默认只显示 Main 类型）
/// * `limit` - 返回的会话数量限制（用于分批加载，默认 20）
/// * `offset` - 跳过的会话数量（用于分批加载，默认 0）
#[tauri::command]
pub async fn get_sessions_by_monitored_directory(
    monitored_path: String,
    include_agent: Option<bool>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<SessionFileInfo>, CommandError> {
    use crate::path_resolver::list_session_files;
    use crate::session_reader::{load_default_history_cache, SessionDisplayName};
    use std::path::Path;

    // 提供默认值
    let include_agent = include_agent.unwrap_or(false);
    let limit = limit.unwrap_or(20);
    let offset = offset.unwrap_or(0);

    #[cfg(debug_assertions)]
    eprintln!("[get_sessions_by_monitored_directory] 监控路径: {}, include_agent: {}, limit: {}, offset: {}",
        monitored_path, include_agent, limit, offset);

    // 将监控路径转换为项目路径
    let project_path = Path::new(&monitored_path);

    // 使用路径解析器获取会话文件列表（已按修改时间倒序排序）
    let all_session_files = list_session_files(project_path).map_err(|e| CommandError {
        message: format!("获取会话文件失败: {}", e),
    })?;

    #[cfg(debug_assertions)]
    eprintln!(
        "[get_sessions_by_monitored_directory] 总共找到 {} 个会话文件",
        all_session_files.len()
    );

    // 先应用类型筛选，再分页
    let filtered_session_files: Vec<_> = all_session_files
        .into_iter()
        .filter(|info| {
            // 如果不包含 Agent，则过滤掉 Agent 类型的会话
            if !include_agent && info.file_type.is_agent() {
                #[cfg(debug_assertions)]
                eprintln!(
                    "[get_sessions_by_monitored_directory] 过滤掉 Agent 会话: {}",
                    info.file_name
                );
                false
            } else {
                true
            }
        })
        .collect();

    #[cfg(debug_assertions)]
    eprintln!(
        "[get_sessions_by_monitored_directory] 类型筛选后剩余 {} 个会话",
        filtered_session_files.len()
    );

    // 应用分页：跳过 offset，取 limit 个
    let session_files: Vec<_> = filtered_session_files
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    #[cfg(debug_assertions)]
    eprintln!(
        "[get_sessions_by_monitored_directory] 本批处理 {} 个会话文件",
        session_files.len()
    );

    // 预加载 history.jsonl 缓存
    let history_cache = load_default_history_cache().await.unwrap_or_default();
    #[cfg(debug_assertions)]
    eprintln!(
        "[get_sessions_by_monitored_directory] history 缓存加载完成，共 {} 个条目",
        history_cache.len()
    );

    // 并行加载会话显示名称（使用并发控制和超时机制）
    use futures::stream::{self, StreamExt};
    use std::time::Duration;

    let name_stream = stream::iter(session_files)
        .map(|info| {
            let history_cache = &history_cache;
            async move {
                // 添加超时机制：单个会话名称获取最多 100ms
                let timeout_result = tokio::time::timeout(
                    Duration::from_millis(100),
                    SessionDisplayName::get_display_name(&info.full_path, Some(history_cache)),
                )
                .await;

                match timeout_result {
                    Ok(Ok(display)) => (info, Some(display)),
                    Ok(Err(_)) | Err(_) => (info, None), // 超时或错误都返回 None
                }
            }
        })
        .buffer_unordered(10); // 限制并发数为 10

    let display_names: Vec<(
        crate::path_resolver::SessionFileInfo,
        Option<SessionDisplayName>,
    )> = name_stream.collect().await;

    // 转换为前端格式（类型筛选已在前面完成）
    let mut result: Vec<SessionFileInfo> = display_names
        .into_iter()
        .filter_map(|(info, name_result)| {
            // 处理显示名称结果
            let (display_name, name_source, summary) = match name_result {
                Some(display) => (
                    Some(display.name.clone()),
                    Some(format!("{:?}", display.source)),
                    Some(display.name),
                ),
                None => (None, None, None), // 失败时使用完整的会话ID
            };

            Some(SessionFileInfo {
                session_id: info.file_name,
                file_path: info.full_path.to_string_lossy().to_string(),
                file_size: info.file_size,
                modified_time: info.modified_time.clone(),
                project_path: monitored_path.clone(), // 添加项目路径
                summary,                              // 向后兼容
                display_name,
                name_source,
                file_type: info.file_type,
            })
        })
        .collect();

    // 🔥 修复：并行加载后重新按修改时间倒序排序
    result.sort_by(|a, b| b.modified_time.cmp(&a.modified_time));

    #[cfg(debug_assertions)]
    eprintln!(
        "[get_sessions_by_monitored_directory] 排序完成，返回 {} 个会话",
        result.len()
    );

    #[cfg(debug_assertions)]
    eprintln!(
        "[get_sessions_by_monitored_directory] 返回 {} 个会话（筛选后）",
        result.len()
    );

    Ok(result)
}

// ==================== 向量搜索命令 ====================

/// 语义搜索请求参数
#[derive(Debug, Deserialize)]
pub struct SemanticSearchRequest {
    /// 搜索查询文本
    pub query: String,
    /// 返回结果数量（默认 10）
    #[serde(default)]
    pub top_k: Option<usize>,
    /// 最小相似度阈值（0.0-1.0，默认 0.0）
    #[serde(default)]
    pub min_similarity: Option<f64>,
}

/// 语义搜索结果
#[derive(Debug, Serialize)]
pub struct SemanticSearchResult {
    /// 会话信息
    pub session: SessionInfo,
    /// 相似度分数（0.0-1.0）
    pub similarity_score: f64,
    /// 会话摘要
    pub summary: String,
}

/// 会话信息
#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub project_path: String,
    pub project_name: String,
    pub file_path: String,
    pub rating: Option<i32>,
    pub tags: Vec<String>,
}

/// 语义搜索命令
#[tauri::command]
pub async fn semantic_search(
    request: SemanticSearchRequest,
    manager: State<'_, LLMClientManager>,
) -> Result<Vec<SemanticSearchResult>, String> {
    use crate::database::get_connection_shared;

    let top_k = request.top_k.unwrap_or(10);
    let min_similarity = request.min_similarity.unwrap_or(0.0);

    // 检查向量搜索是否启用
    let conn = get_connection_shared().map_err(|e| format!("获取数据库连接失败: {}", e))?;
    let repo = VectorRepository::with_conn(conn);

    // 获取当前设置
    let settings = crate::database::repository::SettingsRepository::new()
        .get_settings()
        .map_err(|e| format!("获取设置失败: {}", e))?;

    if !settings.vector_search_enabled {
        return Err("向量搜索功能未启用。请在设置中启用向量搜索。".to_string());
    }

    // 获取 API Key（从活跃的 LLM provider）
    let active_provider = manager
        .get_active_provider_config()
        .map_err(|e| format!("获取活跃提供商失败: {}", e))?;

    let provider_id = active_provider
        .id
        .ok_or_else(|| "提供商 ID 无效".to_string())?;

    let api_key = crate::llm::security::ApiKeyStorage::get_api_key(provider_id)
        .map_err(|e| format!("获取 API Key 失败: {}", e))?;

    let api_key = api_key.expose_secret().to_string();

    // 生成查询向量
    let embedding_client = OpenAIEmbeddings::new(&api_key, Some(settings.embedding_model.clone()))
        .map_err(|e| format!("创建 Embedding 客户端失败: {}", e))?;

    let query_vector = embedding_client
        .generate_embedding(&request.query)
        .await
        .map_err(|e| format!("生成查询向量失败: {}", e))?;

    // 执行向量搜索
    let search_results = repo
        .vector_search_sessions(&query_vector, top_k, min_similarity)
        .map_err(|e| format!("向量搜索失败: {}", e))?;

    // 转换结果格式
    let results: Vec<SemanticSearchResult> = search_results
        .into_iter()
        .map(|r| {
            let session = r.session;
            SemanticSearchResult {
                session: SessionInfo {
                    session_id: session.session_id,
                    project_path: session.project_path,
                    project_name: session.project_name,
                    file_path: session.file_path,
                    rating: session.rating,
                    tags: serde_json::from_str(&session.tags).unwrap_or_default(),
                },
                similarity_score: r.similarity_score,
                summary: r.summary,
            }
        })
        .collect();

    Ok(results)
}

/// 查找相似会话
#[tauri::command]
pub async fn find_similar_sessions(
    session_id: String,
    top_k: Option<usize>,
    min_similarity: Option<f64>,
    _manager: State<'_, LLMClientManager>,
) -> Result<Vec<SemanticSearchResult>, String> {
    use crate::database::get_connection_shared;

    let top_k = top_k.unwrap_or(10);
    let min_similarity = min_similarity.unwrap_or(0.0);

    // 检查向量搜索是否启用
    let conn = get_connection_shared().map_err(|e| format!("获取数据库连接失败: {}", e))?;
    let repo = VectorRepository::with_conn(conn);

    let settings = crate::database::repository::SettingsRepository::new()
        .get_settings()
        .map_err(|e| format!("获取设置失败: {}", e))?;

    if !settings.vector_search_enabled {
        return Err("向量搜索功能未启用".to_string());
    }

    // 获取目标会话的向量
    let target_embedding = repo
        .get_session_embedding(&session_id)
        .map_err(|e| format!("查询会话向量失败: {}", e))?
        .ok_or_else(|| format!("未找到会话 {} 的向量", session_id))?;

    let target_vector = target_embedding
        .get_embedding()
        .map_err(|e| format!("解析向量失败: {}", e))?;

    // 执行向量搜索
    let search_results = repo
        .vector_search_sessions(
            &target_vector,
            top_k + 1, // +1 因为结果会包含自己
            min_similarity,
        )
        .map_err(|e| format!("向量搜索失败: {}", e))?;

    // 过滤掉自己并转换结果格式
    let results: Vec<SemanticSearchResult> = search_results
        .into_iter()
        .filter(|r| r.session.session_id != session_id)
        .take(top_k)
        .map(|r| {
            let session = r.session;
            SemanticSearchResult {
                session: SessionInfo {
                    session_id: session.session_id,
                    project_path: session.project_path,
                    project_name: session.project_name,
                    file_path: session.file_path,
                    rating: session.rating,
                    tags: serde_json::from_str(&session.tags).unwrap_or_default(),
                },
                similarity_score: r.similarity_score,
                summary: r.summary,
            }
        })
        .collect();

    Ok(results)
}

/// 向量设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSettings {
    pub vector_search_enabled: bool,
    pub embedding_provider: String,
    pub embedding_model: String,
    pub embedding_batch_size: i32,
}

/// 获取向量设置
#[tauri::command]
pub async fn get_vector_settings() -> Result<VectorSettings, String> {
    let settings = crate::database::repository::SettingsRepository::new()
        .get_settings()
        .map_err(|e| format!("获取设置失败: {}", e))?;

    Ok(VectorSettings {
        vector_search_enabled: settings.vector_search_enabled,
        embedding_provider: settings.embedding_provider,
        embedding_model: settings.embedding_model,
        embedding_batch_size: settings.embedding_batch_size,
    })
}

/// 更新向量设置
#[tauri::command]
pub async fn update_vector_settings(settings: VectorSettings) -> Result<(), String> {
    let mut repo_settings = crate::database::repository::SettingsRepository::new()
        .get_settings()
        .map_err(|e| format!("获取当前设置失败: {}", e))?;

    repo_settings.vector_search_enabled = settings.vector_search_enabled;
    repo_settings.embedding_provider = settings.embedding_provider;
    repo_settings.embedding_model = settings.embedding_model;
    repo_settings.embedding_batch_size = settings.embedding_batch_size;

    repo_settings
        .validate()
        .map_err(|e| format!("设置验证失败: {}", e))?;

    crate::database::repository::SettingsRepository::new()
        .update_settings(&repo_settings)
        .map_err(|e| format!("更新设置失败: {}", e))?;

    Ok(())
}

/// 手动触发向量同步
#[tauri::command]
pub async fn sync_embeddings_now(manager: State<'_, LLMClientManager>) -> Result<usize, String> {
    use crate::database::get_connection_shared;

    let conn = get_connection_shared().map_err(|e| format!("获取数据库连接失败: {}", e))?;
    let repo = VectorRepository::with_conn(conn);

    let settings = crate::database::repository::SettingsRepository::new()
        .get_settings()
        .map_err(|e| format!("获取设置失败: {}", e))?;

    if !settings.vector_search_enabled {
        return Err("向量搜索功能未启用".to_string());
    }

    // 获取 API Key
    let active_provider = manager
        .get_active_provider_config()
        .map_err(|e| format!("获取活跃提供商失败: {}", e))?;

    let provider_id = active_provider
        .id
        .ok_or_else(|| "提供商 ID 无效".to_string())?;

    let api_key = crate::llm::security::ApiKeyStorage::get_api_key(provider_id)
        .map_err(|e| format!("获取 API Key 失败: {}", e))?;

    let api_key = api_key.expose_secret().to_string();

    // 创建同步管理器
    let sync_manager = EmbeddingSyncManager::new(std::sync::Arc::new(repo));
    sync_manager.set_api_key(api_key).await;
    sync_manager
        .update_config(&settings)
        .await
        .map_err(|e| format!("更新配置失败: {}", e))?;

    // 执行同步
    let count = sync_manager
        .sync_all_sessions()
        .await
        .map_err(|e| format!("同步失败: {}", e))?;

    Ok(count)
}

// ============================================================================
// 多等级日志读取 Commands (Multi-Level Log Reading)
// ============================================================================

use crate::parser::view_level::{MessageFilter, QAPair, ViewLevel};

/// 根据等级获取会话消息
///
/// # 参数
/// - `session_id`: 会话 ID
/// - `view_level`: 视图等级
/// - `file_path`: (可选) 会话文件路径。如果提供，直接使用文件路径而不从数据库查询
///
/// # 返回
/// 过滤后的消息列表
#[tauri::command]
pub async fn cmd_get_messages_by_level(
    session_id: String,
    view_level: ViewLevel,
    file_path: Option<String>,
) -> Result<Vec<crate::database::models::Message>, String> {
    use crate::database::repository::SessionRepository;
    use crate::session_parser::{SessionParserConfig, SessionParserService};

    // 确定文件路径
    let final_file_path = if let Some(fp) = file_path {
        // 如果提供了文件路径，直接使用
        fp
    } else {
        // 否则从数据库查询会话信息
        let repo = SessionRepository::from_default_db()
            .map_err(|e| format!("创建 SessionRepository 失败: {}", e))?;
        let session = repo
            .get_session_by_id(&session_id)
            .map_err(|e| format!("获取会话失败: {}", e))?
            .ok_or_else(|| format!("会话不存在: {}", session_id))?;
        session.file_path
    };

    // 检查会话文件是否存在
    let path_buf = std::path::PathBuf::from(&final_file_path);
    if !path_buf.exists() {
        return Err(format!("会话文件不存在: {}", final_file_path));
    }

    // 创建解析配置
    let config = SessionParserConfig {
        enable_content_filter: true, // ✅ 启用内容过滤
        view_level: view_level.clone(),
        debug: cfg!(debug_assertions),
    };

    // 创建解析服务
    let parser = SessionParserService::new(config);

    // 解析会话
    let result = parser
        .parse_session(&final_file_path, &session_id)
        .map_err(|e| format!("解析会话失败: {}", e))?;

    // 输出调试信息
    #[cfg(debug_assertions)]
    {
        eprintln!("[DEBUG] 解析统计: {:?}", result.stats);
        eprintln!(
            "[DEBUG] 返回 {} 个消息 (view_level: {:?})",
            result.messages.len(),
            view_level
        );

        // 显示前 3 条消息的详细信息
        if !result.messages.is_empty() {
            eprintln!("[DEBUG] 前 3 条消息示例:");
            for (i, msg) in result.messages.iter().take(3).enumerate() {
                eprintln!("  [{}]:", i);
                eprintln!("    msg_type: {:?}", msg.msg_type);
                eprintln!("    uuid: {:?}", msg.uuid.get(..8));
                eprintln!(
                    "    summary: {:?}",
                    msg.summary.as_ref().and_then(|s| s.get(..50))
                );
                eprintln!("    timestamp: {:?}", msg.timestamp);
            }

            // 🔍 序列化调试 - 检查实际输出的 JSON
            eprintln!("[DEBUG] 🔍 序列化前第一条消息的 msg_type 字段值:");
            let first_msg = &result.messages[0];
            eprintln!("  msg_type (原始值) = {:?}", first_msg.msg_type);
            eprintln!("  msg_type (字符串) = {}", first_msg.msg_type);

            // 尝试序列化第一条消息
            match serde_json::to_string_pretty(first_msg) {
                Ok(json) => {
                    eprintln!("[DEBUG] 序列化后的 JSON:");
                    for line in json.lines().take(15) {
                        eprintln!("  {}", line);
                    }

                    // 解析回来验证字段名
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json) {
                        eprintln!("[DEBUG] JSON 中的键名:");
                        if let Some(obj) = value.as_object() {
                            for (key, _) in obj.iter() {
                                eprintln!("  - {}", key);
                            }
                            // 特别检查 type/msgType/msg_type 字段
                            eprintln!("[DEBUG] 特定字段值:");
                            eprintln!("  type 字段存在: {:?}", obj.get("type"));
                            eprintln!("  msgType 字段存在: {:?}", obj.get("msgType"));
                            eprintln!("  msg_type 字段存在: {:?}", obj.get("msg_type"));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[DEBUG] 序列化失败: {}", e);
                }
            }
        }
    }

    Ok(result.messages)
}

/// 根据等级提取问答对
///
/// # 参数
/// - `session_id`: 会话 ID
/// - `view_level`: 视图等级（必须是 QAPairs）
/// - `file_path`: (可选) 会话文件路径。如果提供，直接使用文件路径而不从数据库查询
///
/// # 返回
/// 问答对列表
#[tauri::command]
pub async fn cmd_get_qa_pairs_by_level(
    session_id: String,
    view_level: ViewLevel,
    file_path: Option<String>,
) -> Result<Vec<QAPair>, String> {
    use crate::database::repository::SessionRepository;
    use crate::session_parser::{SessionParserConfig, SessionParserService};

    // 验证等级必须是 QAPairs
    if view_level != ViewLevel::QAPairs {
        return Err("问答对提取仅在 QAPairs 等级下可用".to_string());
    }

    // 确定文件路径
    let final_file_path = if let Some(fp) = file_path {
        // 如果提供了文件路径，直接使用
        fp
    } else {
        // 否则从数据库查询会话信息
        let repo = SessionRepository::from_default_db()
            .map_err(|e| format!("创建 SessionRepository 失败: {}", e))?;
        let session = repo
            .get_session_by_id(&session_id)
            .map_err(|e| format!("获取会话失败: {}", e))?
            .ok_or_else(|| format!("会话不存在: {}", session_id))?;
        session.file_path
    };

    // 检查会话文件是否存在
    let path_buf = std::path::PathBuf::from(&final_file_path);
    if !path_buf.exists() {
        return Err(format!("会话文件不存在: {}", final_file_path));
    }

    // 使用 SessionParserService 解析会话（在 Full 视图等级下获取所有消息）
    let config = SessionParserConfig {
        enable_content_filter: false, // 问答对提取不过滤内容
        view_level: ViewLevel::Full,  // 获取所有消息，后续由 extract_qa_pairs 处理
        debug: cfg!(debug_assertions),
    };

    let parser = SessionParserService::new(config);
    let result = parser
        .parse_session(&final_file_path, &session_id)
        .map_err(|e| format!("解析会话失败: {}", e))?;

    #[cfg(debug_assertions)]
    {
        eprintln!("[DEBUG] 解析统计: {:?}", result.stats);
        eprintln!(
            "[DEBUG] 返回 {} 个消息用于问答对提取",
            result.messages.len()
        );
    }

    // 提取问答对
    let filter = MessageFilter::new(view_level);
    let qa_pairs = filter.extract_qa_pairs(result.messages);

    // 调试日志：检查提取的问答对
    #[cfg(debug_assertions)]
    {
        eprintln!("[DEBUG] 提取的问答对数量: {}", qa_pairs.len());
        if !qa_pairs.is_empty() {
            eprintln!("[DEBUG] 前 3 个问答对:");
            for (i, pair) in qa_pairs.iter().take(3).enumerate() {
                eprintln!(
                    "  [{}] question_uuid={}, question_type={}, has_answer={}",
                    i,
                    &pair.question.uuid[..pair.question.uuid.len().min(8)],
                    pair.question.msg_type,
                    pair.answer.is_some()
                );
                if let Some(ref answer) = pair.answer {
                    eprintln!(
                        "       answer_uuid={}, answer_type={}",
                        &answer.uuid[..answer.uuid.len().min(8)],
                        answer.msg_type
                    );
                }
            }
        }
    }

    Ok(qa_pairs)
}

/// 保存视图等级偏好
///
/// # 参数
/// - `session_id`: 会话 ID
/// - `view_level`: 视图等级
///
/// # 返回
/// 成功返回 Ok(())
#[tauri::command]
pub async fn cmd_save_view_level_preference(
    session_id: String,
    view_level: ViewLevel,
) -> Result<(), String> {
    use crate::database::repository::ViewLevelPreferenceRepository;

    let mut repo = ViewLevelPreferenceRepository::new();
    repo.save_preference(&session_id, view_level)
        .map_err(|e| format!("保存偏好失败: {}", e))
}

/// 获取视图等级偏好
///
/// # 参数
/// - `session_id`: 会话 ID
///
/// # 返回
/// 视图等级，如果不存在则返回默认值 QAPairs
#[tauri::command]
pub async fn cmd_get_view_level_preference(session_id: String) -> Result<ViewLevel, String> {
    use crate::database::repository::ViewLevelPreferenceRepository;

    let repo = ViewLevelPreferenceRepository::new();
    let preference = repo
        .get_preference_or_default(&session_id)
        .map_err(|e| format!("获取偏好失败: {}", e))?;

    Ok(preference)
}

/// 导出格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormatType {
    #[serde(rename = "markdown")]
    Markdown,
    #[serde(rename = "json")]
    Json,
}

/// 根据等级导出会话
///
/// # 参数
/// - `session_id`: 会话 ID
/// - `view_level`: 视图等级
/// - `format`: 导出格式（markdown 或 json）
/// - `file_path`: (可选) 会话文件路径。如果提供，直接使用文件路径而不从数据库查询
///
/// # 返回
/// 导出的内容字符串
#[tauri::command]
pub async fn cmd_export_session_by_level(
    session_id: String,
    view_level: ViewLevel,
    format: ExportFormatType,
    file_path: Option<String>,
) -> Result<String, String> {
    // 获取过滤后的消息
    let messages = if view_level == ViewLevel::QAPairs {
        // 对于 QAPairs，先获取问答对
        let qa_pairs =
            cmd_get_qa_pairs_by_level(session_id.clone(), view_level, file_path.clone()).await?;

        // 将问答对转换为可导出的格式
        let export_messages: Vec<crate::database::models::Message> = qa_pairs
            .into_iter()
            .flat_map(|qa| {
                let mut messages = vec![qa.question];
                if let Some(answer) = qa.answer {
                    messages.push(answer);
                }
                messages
            })
            .collect();

        export_messages
    } else {
        // 其他等级直接获取消息
        cmd_get_messages_by_level(session_id.clone(), view_level, file_path.clone()).await?
    };

    // 保存 file_path 的引用供后续使用
    let file_path_ref = file_path.as_deref();

    match format {
        ExportFormatType::Markdown => {
            // 导出为 Markdown 格式
            let mut markdown = format!("# 会话导出\n\n");
            markdown.push_str(&format!("**会话 ID**: {}\n", session_id));
            if let Some(fp) = file_path_ref {
                markdown.push_str(&format!("**文件路径**: {}\n", fp));
            }
            markdown.push_str(&format!("**视图等级**: {}\n\n", view_level.display_name()));
            markdown.push_str("---\n\n");

            for msg in &messages {
                let role_label = match msg.msg_type.as_str() {
                    "user" => "👤 用户",
                    "assistant" => "🤖 助手",
                    "thinking" => "💭 思考",
                    _ => "📝 其他",
                };

                markdown.push_str(&format!("## {}\n\n", role_label));
                markdown.push_str(&format!("**时间**: {}\n\n", msg.timestamp));

                if let Some(summary) = &msg.summary {
                    markdown.push_str(&format!("{}\n\n", summary));
                } else {
                    markdown.push_str("*（无内容）*\n\n");
                }

                markdown.push_str("---\n\n");
            }

            Ok(markdown)
        }
        ExportFormatType::Json => {
            // 导出为 JSON 格式
            let export_data = serde_json::json!({
                "session": {
                    "session_id": session_id,
                    "file_path": file_path,
                },
                "view_level": {
                    "value": view_level.to_string(),
                    "display_name": view_level.display_name(),
                    "description": view_level.description(),
                },
                "messages": messages,
                "exported_at": chrono::Utc::now().to_rfc3339()
            });

            serde_json::to_string_pretty(&export_data)
                .map_err(|e| format!("JSON 序列化失败: {}", e))
        }
    }
}

// ==================== 日志过滤配置管理命令 ====================

/// 获取过滤配置
#[tauri::command]
pub fn get_filter_config() -> Result<crate::filter_config::FilterConfig, CommandError> {
    use crate::filter_config::FilterConfigManager;

    let manager = FilterConfigManager::with_default_path().map_err(|e| CommandError {
        message: format!("加载过滤配置失败: {}", e),
    })?;

    Ok(manager.get_config().clone())
}

/// 更新过滤配置
#[tauri::command]
pub fn update_filter_config(
    config: crate::filter_config::FilterConfig,
) -> Result<(), CommandError> {
    use crate::filter_config::FilterConfigManager;

    let mut manager = FilterConfigManager::with_default_path().map_err(|e| CommandError {
        message: format!("加载过滤配置失败: {}", e),
    })?;

    manager.update_config(config).map_err(|e| CommandError {
        message: format!("更新过滤配置失败: {}", e),
    })?;

    Ok(())
}

/// 重新加载过滤配置
#[tauri::command]
pub fn reload_filter_config() -> Result<(), CommandError> {
    use crate::filter_config::FilterConfigManager;

    let mut manager = FilterConfigManager::with_default_path().map_err(|e| CommandError {
        message: format!("加载过滤配置失败: {}", e),
    })?;

    manager.reload().map_err(|e| CommandError {
        message: format!("重新加载过滤配置失败: {}", e),
    })?;

    Ok(())
}

/// 获取过滤配置文件路径
#[tauri::command]
pub fn get_filter_config_path() -> Result<String, CommandError> {
    use crate::filter_config::FilterConfigManager;

    let manager = FilterConfigManager::with_default_path().map_err(|e| CommandError {
        message: format!("获取配置路径失败: {}", e),
    })?;

    Ok(manager.config_path().to_string_lossy().to_string())
}

/// 在系统默认文件管理器中打开配置文件所在目录
#[tauri::command]
pub fn open_filter_config_folder() -> Result<(), CommandError> {
    use crate::filter_config::FilterConfigManager;

    let manager = FilterConfigManager::with_default_path().map_err(|e| CommandError {
        message: format!("获取配置路径失败: {}", e),
    })?;

    let config_dir = manager.config_path().parent().ok_or_else(|| CommandError {
        message: "无法获取配置目录".to_string(),
    })?;

    // 使用系统默认程序打开目录
    open::that(config_dir).map_err(|e| CommandError {
        message: format!("打开配置目录失败: {}", e),
    })?;

    Ok(())
}

// ============================================================================
// 提示词生成历史管理命令 (Prompt Generation History)
// ============================================================================

use crate::database::{PromptGenerationHistory, PromptHistoryRepository};

/// 保存提示词生成历史
///
/// # 参数
/// - `history`: 要保存的历史记录
#[tauri::command]
pub async fn cmd_save_prompt_history(
    history: PromptGenerationHistory,
) -> Result<PromptGenerationHistory, CommandError> {
    let mut repo = PromptHistoryRepository::from_default_db().map_err(|e| CommandError {
        message: format!("创建仓库失败: {}", e),
    })?;

    let created = repo.create_history(history).map_err(|e| CommandError {
        message: format!("保存历史记录失败: {}", e),
    })?;

    Ok(created)
}

/// 获取所有提示词生成历史
#[tauri::command]
pub async fn cmd_get_prompt_history() -> Result<Vec<PromptGenerationHistory>, CommandError> {
    let repo = PromptHistoryRepository::from_default_db().map_err(|e| CommandError {
        message: format!("创建仓库失败: {}", e),
    })?;

    let histories = repo.get_all_histories().map_err(|e| CommandError {
        message: format!("获取历史记录失败: {}", e),
    })?;

    Ok(histories)
}

/// 分页获取提示词生成历史
///
/// # 参数
/// - `offset`: 偏移量
/// - `limit`: 每页数量
#[tauri::command]
pub async fn cmd_get_prompt_history_paginated(
    offset: i64,
    limit: i64,
) -> Result<Vec<PromptGenerationHistory>, CommandError> {
    let repo = PromptHistoryRepository::from_default_db().map_err(|e| CommandError {
        message: format!("创建仓库失败: {}", e),
    })?;

    let histories = repo
        .get_histories_paginated(offset, limit)
        .map_err(|e| CommandError {
            message: format!("获取历史记录失败: {}", e),
        })?;

    Ok(histories)
}

/// 根据 ID 获取提示词生成历史
///
/// # 参数
/// - `id`: 历史 ID
#[tauri::command]
pub async fn cmd_get_prompt_history_by_id(
    id: i64,
) -> Result<Option<PromptGenerationHistory>, CommandError> {
    let repo = PromptHistoryRepository::from_default_db().map_err(|e| CommandError {
        message: format!("创建仓库失败: {}", e),
    })?;

    let history = repo.get_history_by_id(id).map_err(|e| CommandError {
        message: format!("获取历史记录失败: {}", e),
    })?;

    Ok(history)
}

/// 删除提示词生成历史
///
/// # 参数
/// - `id`: 要删除的历史 ID
#[tauri::command]
pub async fn cmd_delete_prompt_history(id: i64) -> Result<(), CommandError> {
    let repo = PromptHistoryRepository::from_default_db().map_err(|e| CommandError {
        message: format!("创建仓库失败: {}", e),
    })?;

    repo.delete_history(id).map_err(|e| CommandError {
        message: format!("删除历史记录失败: {}", e),
    })?;

    Ok(())
}

/// 切换提示词历史的收藏状态
///
/// # 参数
/// - `id`: 历史 ID
///
/// # 返回
/// 返回更新后的收藏状态
#[tauri::command]
pub async fn cmd_toggle_prompt_history_favorite(id: i64) -> Result<bool, CommandError> {
    let mut repo = PromptHistoryRepository::from_default_db().map_err(|e| CommandError {
        message: format!("创建仓库失败: {}", e),
    })?;

    let is_favorite = repo.toggle_favorite(id).map_err(|e| CommandError {
        message: format!("切换收藏状态失败: {}", e),
    })?;

    Ok(is_favorite)
}

/// 获取收藏的提示词历史
#[tauri::command]
pub async fn cmd_get_favorite_prompt_history() -> Result<Vec<PromptGenerationHistory>, CommandError>
{
    let repo = PromptHistoryRepository::from_default_db().map_err(|e| CommandError {
        message: format!("创建仓库失败: {}", e),
    })?;

    let histories = repo.get_favorite_histories().map_err(|e| CommandError {
        message: format!("获取收藏历史失败: {}", e),
    })?;

    Ok(histories)
}

/// 统计提示词历史数量
#[tauri::command]
pub async fn cmd_count_prompt_history() -> Result<i64, CommandError> {
    let repo = PromptHistoryRepository::from_default_db().map_err(|e| CommandError {
        message: format!("创建仓库失败: {}", e),
    })?;

    let count = repo.count_histories().map_err(|e| CommandError {
        message: format!("统计历史记录失败: {}", e),
    })?;

    Ok(count)
}

// ==================== 项目技术栈管理命令 ====================

use crate::database::repositories_tech_stack::{ProjectTechStack, ProjectTechStackRepository};
use crate::intent_analyzer::decision_analyzer::{DecisionAnalysis, DecisionAnalyzer};
use crate::intent_analyzer::opening_intent::OpeningIntent;
use crate::intent_analyzer::opening_intent::OpeningIntentAnalyzer;
use crate::intent_analyzer::qa_detector::DecisionQAPair;
use crate::intent_analyzer::tech_stack_detector::TechStackDetector;

/// 保存或更新项目技术栈
///
/// # 参数
///
/// - `project_path`: 项目路径
/// - `tech_stack`: 技术栈列表
/// - `detection_method`: 检测方法（auto | manual）
/// - `detection_source`: 检测来源（CLAUDE.md | README.md | manual）
/// - `is_confirmed`: 是否已确认
#[tauri::command]
pub async fn cmd_save_project_tech_stack(
    project_path: String,
    tech_stack: Vec<String>,
    detection_method: String,
    detection_source: Option<String>,
    is_confirmed: bool,
) -> Result<ProjectTechStack, CommandError> {
    let db_path = crate::database::get_db_path().map_err(|e| CommandError {
        message: format!("获取数据库路径失败: {}", e),
    })?;

    let db_path_str = db_path.to_string_lossy().to_string();
    let repo = ProjectTechStackRepository::new(db_path_str);

    let project = ProjectTechStack {
        id: 0,
        project_path,
        tech_stack,
        detection_method,
        detection_source,
        is_confirmed,
        last_verified_at: None,
    };

    let _id = repo.upsert(&project).map_err(|e| CommandError {
        message: format!("保存项目技术栈失败: {}", e),
    })?;

    Ok(project)
}

/// 获取项目技术栈
///
/// # 参数
///
/// - `project_path`: 项目路径
#[tauri::command]
pub async fn cmd_get_project_tech_stack(
    project_path: String,
) -> Result<Option<ProjectTechStack>, CommandError> {
    let db_path = crate::database::get_db_path().map_err(|e| CommandError {
        message: format!("获取数据库路径失败: {}", e),
    })?;

    let db_path_str = db_path.to_string_lossy().to_string();
    let repo = ProjectTechStackRepository::new(db_path_str);

    let project = repo.get_by_path(&project_path).map_err(|e| CommandError {
        message: format!("获取项目技术栈失败: {}", e),
    })?;

    Ok(project)
}

/// 自动检测项目技术栈
///
/// 从 CLAUDE.md 和 README.md 中自动检测技术栈
///
/// # 参数
///
/// - `project_path`: 项目路径
#[tauri::command]
pub async fn cmd_detect_project_tech_stack(
    project_path: String,
) -> Result<Vec<String>, CommandError> {
    let detector = TechStackDetector::new().map_err(|e| CommandError {
        message: format!("创建技术栈检测器失败: {}", e),
    })?;

    let tech_stack = detector.detect_from_project(&project_path);

    Ok(tech_stack)
}

/// 检测并保存项目技术栈
///
/// 自动检测并保存到数据库
///
/// # 参数
///
/// - `project_path`: 项目路径
#[tauri::command]
pub async fn cmd_detect_and_save_project_tech_stack(
    project_path: String,
) -> Result<ProjectTechStack, CommandError> {
    // 1. 检测技术栈
    let detector = TechStackDetector::new().map_err(|e| CommandError {
        message: format!("创建技术栈检测器失败: {}", e),
    })?;

    let tech_stack = detector.detect_from_project(&project_path);

    // 2. 确定检测来源
    let detection_source = if tech_stack.is_empty() {
        None
    } else {
        Some("CLAUDE.md/README.md".to_string())
    };

    // 3. 保存到数据库
    let db_path = crate::database::get_db_path().map_err(|e| CommandError {
        message: format!("获取数据库路径失败: {}", e),
    })?;

    let db_path_str = db_path.to_string_lossy().to_string();
    let repo = ProjectTechStackRepository::new(db_path_str);

    let project = ProjectTechStack {
        id: 0,
        project_path: project_path.clone(),
        tech_stack,
        detection_method: "auto".to_string(),
        detection_source,
        is_confirmed: false,
        last_verified_at: None,
    };

    let _id = repo.upsert(&project).map_err(|e| CommandError {
        message: format!("保存项目技术栈失败: {}", e),
    })?;

    Ok(project)
}

// ==================== 开场白意图分析命令 ====================

/// 分析开场白意图
///
/// # 参数
///
/// - `session_file_path`: 会话文件路径
/// - `language`: 语言标识（"zh" 或 "en"）
///
/// # 返回
///
/// 开场白意图分析结果
#[tauri::command]
pub async fn cmd_analyze_opening_intent(
    llm_manager: State<'_, LLMClientManager>,
    session_file_path: String,
    language: String,
) -> Result<OpeningIntent, CommandError> {
    #[cfg(debug_assertions)]
    eprintln!("[cmd_analyze_opening_intent] 开始分析开场白意图");
    #[cfg(debug_assertions)]
    eprintln!("[cmd_analyze_opening_intent] 会话文件: {}", session_file_path);
    #[cfg(debug_assertions)]
    eprintln!("[cmd_analyze_opening_intent] 语言: {}", language);

    // 1. 解析会话文件
    let events = crate::optimizer::PromptOptimizer::parse_session_file(&session_file_path)
        .map_err(|e| CommandError {
            message: format!("解析会话文件失败: {}", e),
        })?;

    #[cfg(debug_assertions)]
    eprintln!("[cmd_analyze_opening_intent] 解析到 {} 个事件", events.len());

    if events.is_empty() {
        return Err(CommandError {
            message: "会话文件为空".to_string(),
        });
    }

    // 2. 提取第一个 user 消息作为开场白
    let opening_event = events
        .iter()
        .find(|e| e.role == "user")
        .ok_or_else(|| CommandError {
            message: "未找到开场白".to_string(),
        })?;

    // 构建 Message 结构
    let opening_message = crate::database::models::Message {
        id: None,
        session_id: String::new(),
        uuid: String::new(),
        parent_uuid: None,
        msg_type: "user".to_string(),
        content_type: Some("text".to_string()),
        timestamp: opening_event.time.clone(),
        offset: 0,
        length: opening_event.content.len() as i64,
        summary: None,
        content: Some(opening_event.content.clone()),
        parent_idx: None,
        created_at: opening_event.time.clone(),
    };

    // 3. 创建分析器并分析
    #[cfg(debug_assertions)]
    eprintln!("[cmd_analyze_opening_intent] 创建 OpeningIntentAnalyzer");

    let analyzer = OpeningIntentAnalyzer::new().map_err(|e| CommandError {
        message: format!("创建分析器失败: {}", e),
    })?;

    #[cfg(debug_assertions)]
    eprintln!("[cmd_analyze_opening_intent] 开始调用 LLM 分析");

    let result = analyzer
        .analyze(&opening_message, &language, &llm_manager)
        .await
        .map_err(|e| CommandError {
            message: format!("分析失败: {}", e),
        })?;

    #[cfg(debug_assertions)]
    eprintln!("[cmd_analyze_opening_intent] 分析完成: intent_type={}", result.intent_type);

    Ok(result)
}

/// 检测会话中的问答对（助手回答 + 用户后续决策）
///
/// # 参数
///
/// - `session_file_path`: 会话文件路径
///
/// # 返回
///
/// 检测到的问答对列表
#[tauri::command]
pub async fn cmd_detect_qa_pairs(
    session_file_path: String,
) -> Result<Vec<DecisionQAPair>, CommandError> {
    // 1. 解析会话文件
    let events = crate::optimizer::PromptOptimizer::parse_session_file(&session_file_path)
        .map_err(|e| CommandError {
            message: format!("解析会话文件失败: {}", e),
        })?;

    if events.is_empty() {
        return Ok(Vec::new());
    }

    // 2. 将 ParsedEvent 转换为 Message
    let messages: Vec<crate::database::models::Message> = events
        .into_iter()
        .enumerate()
        .map(|(idx, event)| crate::database::models::Message {
            id: None,
            session_id: String::new(),
            uuid: format!("msg-{}", idx),
            parent_uuid: None,
            msg_type: event.role.clone(),
            content_type: Some("text".to_string()),
            timestamp: event.time.clone(),
            offset: 0,
            length: event.content.len() as i64,
            summary: None,
            content: Some(event.content),
            parent_idx: None,
            created_at: event.time,
        })
        .collect();

    // 3. 使用 QAPairDetector 检测问答对
    let detector = crate::intent_analyzer::qa_detector::QAPairDetector::new();
    let qa_pairs = detector.detect_decision_qa_pairs(messages);

    #[cfg(debug_assertions)]
    eprintln!(
        "[cmd_detect_qa_pairs] 检测到 {} 个问答对",
        qa_pairs.len()
    );

    Ok(qa_pairs)
}

/// 保存意图分析结果到数据库
///
/// # 参数
///
/// - `session_file_path`: 会话文件路径
/// - `qa_pairs`: 问答对列表
/// - `opening_intent`: 开场白意图
/// - `language`: 语言标识
///
/// # 返回
///
/// 保存的记录 ID
#[tauri::command]
pub async fn cmd_save_intent_analysis(
    session_file_path: String,
    qa_pairs: Vec<DecisionQAPair>,
    opening_intent: OpeningIntent,
    language: String,
) -> Result<i64, CommandError> {
    let repo = crate::database::IntentAnalysisRepository::from_default_db()
        .map_err(|e| CommandError {
            message: format!("创建数据库仓库失败: {}", e),
        })?;

    let id = repo
        .save_analysis(&session_file_path, &qa_pairs, &opening_intent, &language)
        .map_err(|e| CommandError {
            message: format!("保存分析结果失败: {}", e),
        })?;

    #[cfg(debug_assertions)]
    eprintln!(
        "[cmd_save_intent_analysis] 已保存分析结果，ID: {}",
        id
    );

    Ok(id)
}

/// 获取指定会话文件的意图分析历史
///
/// # 参数
///
/// - `session_file_path`: 会话文件路径
///
/// # 返回
///
/// 分析历史记录（如果存在）
#[tauri::command]
pub async fn cmd_get_intent_analysis_history(
    session_file_path: String,
) -> Result<Option<crate::database::IntentAnalysisHistory>, CommandError> {
    let repo = crate::database::IntentAnalysisRepository::from_default_db()
        .map_err(|e| CommandError {
            message: format!("创建数据库仓库失败: {}", e),
        })?;

    let history = repo
        .get_analysis_by_session(&session_file_path)
        .map_err(|e| CommandError {
            message: format!("获取分析历史失败: {}", e),
        })?;

    #[cfg(debug_assertions)]
    if let Some(ref h) = history {
        eprintln!(
            "[cmd_get_intent_analysis_history] 找到分析历史，分析时间: {}",
            h.analyzed_at
        );
        // 🔍 调试日志：序列化测试
        match serde_json::to_string(h) {
            Ok(json) => {
                eprintln!("[cmd_get_intent_analysis_history] 序列化成功: {}", json);
            }
            Err(e) => {
                eprintln!("[cmd_get_intent_analysis_history] 序列化失败: {}", e);
            }
        }
    } else {
        eprintln!(
            "[cmd_get_intent_analysis_history] 未找到分析历史: {}",
            session_file_path
        );
    }

    Ok(history)
}

/// 清除指定会话文件的意图分析历史
///
/// # 参数
///
/// - `session_file_path`: 会话文件路径
///
/// # 返回
///
/// 是否成功删除（true 表示删除了记录，false 表示没有记录可删除）
#[tauri::command]
pub async fn cmd_clear_intent_analysis_history(
    session_file_path: String,
) -> Result<bool, CommandError> {
    let repo = crate::database::IntentAnalysisRepository::from_default_db()
        .map_err(|e| CommandError {
            message: format!("创建数据库仓库失败: {}", e),
        })?;

    let deleted = repo
        .delete_analysis(&session_file_path)
        .map_err(|e| CommandError {
            message: format!("清除分析历史失败: {}", e),
        })?;

    #[cfg(debug_assertions)]
    eprintln!(
        "[cmd_clear_intent_analysis_history] 清除结果: {}",
        deleted
    );

    Ok(deleted)
}

/// 分析问答对决策
///
/// # 参数
///
/// - `session_file_path`: 会话文件路径（用于历史记录查询）
/// - `qa_pair`: 问答对（助手回答 + 用户后续决策）
/// - `language`: 语言标识（"zh" 或 "en"）
/// - `force_reanalyze`: 是否强制重新分析（默认 false）
///
/// # 返回
///
/// 决策分析结果
#[tauri::command]
pub async fn cmd_analyze_decision(
    llm_manager: State<'_, LLMClientManager>,
    session_file_path: String,
    qa_pair: DecisionQAPair,
    language: String,
    force_reanalyze: Option<bool>,
) -> Result<DecisionAnalysis, CommandError> {
    let force_reanalyze = force_reanalyze.unwrap_or(false);
    let qa_index = qa_pair.qa_index as i64;

    // 1. 尝试从历史记录加载（如果未强制重新分析）
    if !force_reanalyze {
        if let Ok(repo) = DecisionAnalysisRepository::from_default_db() {
            if let Ok(Some(history)) = repo.get_analysis(&session_file_path, qa_index) {
                #[cfg(debug_assertions)]
                {
                    eprintln!(
                        "[cmd_analyze_decision] 从历史记录加载决策分析: session={}, qa_index={}",
                        session_file_path, qa_index
                    );
                }

                return Ok(history.decision_analysis);
            }
        }
    }

    // 2. 执行 LLM 分析
    let analyzer = DecisionAnalyzer::new().map_err(|e| CommandError {
        message: format!("创建分析器失败: {}", e),
    })?;

    let result = analyzer
        .analyze(&qa_pair, &language, &llm_manager)
        .await
        .map_err(|e| CommandError {
            message: format!("分析失败: {}", e),
        })?;

    // 3. 保存到历史记录
    if let Ok(repo) = DecisionAnalysisRepository::from_default_db() {
        if let Err(e) = repo.save_analysis(&session_file_path, qa_index, &result) {
            #[cfg(debug_assertions)]
            {
                eprintln!("[cmd_analyze_decision] 保存决策分析历史失败: {}", e);
            }
            // 保存失败不影响结果返回
        } else {
            #[cfg(debug_assertions)]
            {
                eprintln!(
                    "[cmd_analyze_decision] 已保存决策分析历史: session={}, qa_index={}",
                    session_file_path, qa_index
                );
            }
        }
    }

    Ok(result)
}

// ==================== 决策检测命令 ====================

/// 检测问答对中的决策点（规则引擎）
///
/// # 参数
///
/// - `qa_pair`: 问答对（助手回答 + 用户后续决策）
/// - `language`: 语言标识（"zh" 或 "en"），如果不指定则自动检测
///
/// # 返回
///
/// 检测到的决策点列表
#[tauri::command]
pub async fn cmd_detect_decisions(
    qa_pair: DecisionQAPair,
    language: Option<String>,
) -> Result<Vec<DetectedDecisionPoint>, CommandError> {
    // 创建检测器
    let detector = DecisionDetector::new().map_err(|e| CommandError {
        message: format!("创建检测器失败: {}", e),
    })?;

    // 如果没有指定语言，自动检测
    let detected_language = if let Some(lang) = language {
        lang
    } else {
        let combined_text = format!(
            "{} {}",
            qa_pair.user_decision,
            qa_pair.assistant_answer
        );
        detector.detect_language(&combined_text)
    };

    // 执行检测
    let decisions = detector.detect_decisions(&qa_pair, &detected_language).map_err(|e| CommandError {
        message: format!("检测失败: {}", e),
    })?;

    Ok(decisions)
}

/// 获取所有决策关键词
///
/// # 返回
///
/// 所有激活的决策关键词列表
#[tauri::command]
pub async fn cmd_get_decision_keywords() -> Result<Vec<DecisionKeyword>, CommandError> {
    let db_path = crate::database::get_db_path().map_err(|e| CommandError {
        message: format!("获取数据库路径失败: {}", e),
    })?;

    let db_path_str = db_path.to_string_lossy().to_string();
    let repo = DecisionKeywordRepository::new(db_path_str);
    let keywords = repo.get_active_keywords().map_err(|e| CommandError {
        message: format!("获取关键词失败: {}", e),
    })?;

    Ok(keywords)
}

/// 根据语言获取决策关键词
///
/// # 参数
///
/// - `language`: 语言标识（"zh" 或 "en"）
///
/// # 返回
///
/// 指定语言的激活关键词列表
#[tauri::command]
pub async fn cmd_get_decision_keywords_by_language(
    language: String,
) -> Result<Vec<DecisionKeyword>, CommandError> {
    let db_path = crate::database::get_db_path().map_err(|e| CommandError {
        message: format!("获取数据库路径失败: {}", e),
    })?;

    let db_path_str = db_path.to_string_lossy().to_string();
    let repo = DecisionKeywordRepository::new(db_path_str);
    let keywords = repo.get_by_language(&language).map_err(|e| CommandError {
        message: format!("获取关键词失败: {}", e),
    })?;

    Ok(keywords)
}

/// 根据决策类型获取关键词
///
/// # 参数
///
/// - `decision_type`: 决策类型
///
/// # 返回
///
/// 指定决策类型的激活关键词列表
#[tauri::command]
pub async fn cmd_get_decision_keywords_by_type(
    decision_type: String,
) -> Result<Vec<DecisionKeyword>, CommandError> {
    let db_path = crate::database::get_db_path().map_err(|e| CommandError {
        message: format!("获取数据库路径失败: {}", e),
    })?;

    let db_path_str = db_path.to_string_lossy().to_string();
    let repo = DecisionKeywordRepository::new(db_path_str);
    let keywords = repo.get_by_decision_type(&decision_type).map_err(|e| CommandError {
        message: format!("获取关键词失败: {}", e),
    })?;

    Ok(keywords)
}

/// 添加或更新决策关键词
///
/// # 参数
///
/// - `keyword`: 关键词数据
///
/// # 返回
///
/// 插入或更新的记录 ID
#[tauri::command]
pub async fn cmd_upsert_decision_keyword(
    keyword: DecisionKeyword,
) -> Result<i64, CommandError> {
    let db_path = crate::database::get_db_path().map_err(|e| CommandError {
        message: format!("获取数据库路径失败: {}", e),
    })?;

    let db_path_str = db_path.to_string_lossy().to_string();
    let repo = DecisionKeywordRepository::new(db_path_str);
    let id = repo.upsert(&keyword).map_err(|e| CommandError {
        message: format!("保存关键词失败: {}", e),
    })?;

    Ok(id)
}

/// 删除决策关键词
///
/// # 参数
///
/// - `id`: 关键词 ID
///
/// # 返回
///
/// 是否删除成功
#[tauri::command]
pub async fn cmd_delete_decision_keyword(id: i64) -> Result<bool, CommandError> {
    let db_path = crate::database::get_db_path().map_err(|e| CommandError {
        message: format!("获取数据库路径失败: {}", e),
    })?;

    let db_path_str = db_path.to_string_lossy().to_string();
    let repo = DecisionKeywordRepository::new(db_path_str);
    let deleted = repo.delete(id).map_err(|e| CommandError {
        message: format!("删除关键词失败: {}", e),
    })?;

    Ok(deleted)
}

/// 批量导入决策关键词
///
/// # 参数
///
/// - `keywords`: 关键词列表
///
/// # 返回
///
/// 成功导入的关键词数量
#[tauri::command]
pub async fn cmd_import_decision_keywords(
    keywords: Vec<DecisionKeyword>,
) -> Result<usize, CommandError> {
    let db_path = crate::database::get_db_path().map_err(|e| CommandError {
        message: format!("获取数据库路径失败: {}", e),
    })?;

    let db_path_str = db_path.to_string_lossy().to_string();
    let repo = DecisionKeywordRepository::new(db_path_str);
    let count = repo.batch_import(&keywords).map_err(|e| CommandError {
        message: format!("批量导入失败: {}", e),
    })?;

    Ok(count)
}
