//! 性能基准测试模块
//!
//! 测试应用启动和会话扫描的性能指标

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

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
    pub fn to_json(&self) -> Result<String> {
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
        _ => String::from("**暂无具体建议**\n"),
    }
}

/// 测试应用启动时间
///
/// 包括：
/// - 数据库初始化
/// - LLM 管理器创建
///
/// 阈值: < 3000ms
pub fn benchmark_startup_time() -> BenchmarkResult {
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
///
/// 扫描所有会话文件并统计数量
/// 阈值: < 2000ms (100个会话)
pub fn benchmark_scan_sessions() -> BenchmarkResult {
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
            // 目标: 100个会话 < 2000ms
            // 按比例计算: 每100个会话允许 2000ms
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
///
/// 执行典型查询操作
pub fn benchmark_database_queries() -> BenchmarkResult {
    let name = String::from("数据库查询性能");
    let threshold_ms = 100.0; // 单次查询 < 100ms

    let start = Instant::now();

    let query_result = (|| -> Result<String> {
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
///
/// 返回完整的测试报告
pub fn run_all_benchmarks() -> BenchmarkReport {
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
pub fn save_benchmark_report(report: &BenchmarkReport, output_path: &PathBuf) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_startup_time() {
        let result = benchmark_startup_time();
        println!("{:?}", result);
        assert!(result.duration_ms >= 0.0);
    }

    #[test]
    fn test_benchmark_scan_sessions() {
        let result = benchmark_scan_sessions();
        println!("{:?}", result);
        assert!(result.duration_ms >= 0.0);
    }

    #[test]
    fn test_benchmark_database_queries() {
        let result = benchmark_database_queries();
        println!("{:?}", result);
        assert!(result.duration_ms >= 0.0);
    }

    #[test]
    fn test_run_all_benchmarks() {
        let report = run_all_benchmarks();
        println!("\n{}", report.to_markdown());
        assert!(!report.results.is_empty());
    }

    #[test]
    fn test_benchmark_report_serialization() {
        let report = run_all_benchmarks();

        // 测试 Markdown 生成
        let md = report.to_markdown();
        assert!(md.contains("性能基准测试报告"));
        assert!(md.contains("测试结果详情"));

        // 测试 JSON 生成
        let json = report.to_json();
        assert!(json.is_ok());
    }
}
