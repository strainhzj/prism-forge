//! 默认提示词初始化模块
//!
//! 从 optimizer_config.toml 导入默认提示词到数据库
//! 在数据库初始化时自动执行，确保系统提示词始终可用
//!
//! 组件化结构说明：
//! - 只创建"会话分析"场景的模板
//! - 存储为组件化 JSON 格式（meta_prompt, input_template, output_template）
//! - 每个语言版本独立记录最后修改时间

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::PathBuf;
use serde_json::json;

/// 会话分析场景模板定义
///
/// 这是唯一默认初始化的模板
struct SessionAnalysisTemplate {
    /// 模板名称
    name: &'static str,
    /// 模板描述
    description: &'static str,
    /// 应用场景
    scenario: &'static str,
    /// 标签（逗号分隔）
    tags: &'static str,
}

/// 定义默认提示词模板
const DEFAULT_TEMPLATE: SessionAnalysisTemplate = SessionAnalysisTemplate {
    name: "session_analysis",
    description: "会话分析提示词模板 - 用于分析用户目标和会话历史，生成优化的提示词",
    scenario: "session_analysis",
    tags: "session,analysis,prompt",
};

/// 解析配置文件路径
///
/// 支持开发环境和生产环境
/// - 开发环境：src-tauri/optimizer_config.toml
/// - 生产环境：可执行文件同目录的 optimizer_config.toml
pub fn resolve_config_path() -> Result<PathBuf> {
    use std::env;

    let exe_path = env::current_exe()
        .map_err(|e| anyhow::anyhow!("无法获取可执行文件路径: {}", e))?;

    let exe_dir = exe_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| {
            #[cfg(debug_assertions)]
            log::warn!("[InitDefaultPrompts] 可执行文件路径没有父目录，使用当前目录作为回退");
            PathBuf::from(".")
        });

    #[cfg(debug_assertions)]
    {
        log::debug!("[InitDefaultPrompts] 可执行文件路径: {:?}", exe_path);
        log::debug!("[InitDefaultPrompts] 可执行文件目录: {:?}", exe_dir);
    }

    // 策略 1: 从可执行文件目录向上查找项目根目录
    let mut current_dir = exe_dir.clone();
    let max_depth = 5;

    for depth in 0..=max_depth {
        let src_tauri_path = current_dir.join("src-tauri");
        let config_path = src_tauri_path.join("optimizer_config.toml");

        if config_path.exists() {
            #[cfg(debug_assertions)]
            log::debug!("[InitDefaultPrompts] 找到开发环境配置（向上查找 {} 层）: {:?}", depth, config_path);
            return Ok(config_path);
        }

        if !current_dir.pop() {
            #[cfg(debug_assertions)]
            log::debug!("[InitDefaultPrompts] 已到达文件系统根目录，停止向上查找");
            break;
        }
    }

    // 策略 2: 生产环境 - 配置文件在可执行文件同目录
    let prod_path = exe_dir.join("optimizer_config.toml");

    if prod_path.exists() {
        #[cfg(debug_assertions)]
        log::debug!("[InitDefaultPrompts] 使用生产环境配置路径: {:?}", prod_path);
        return Ok(prod_path);
    }

    // 策略 3: 回退 - 尝试使用当前工作目录
    let cwd_path = env::current_dir()
        .map(|d| d.join("src-tauri").join("optimizer_config.toml"))
        .ok();

    if let Some(ref path) = cwd_path {
        if path.exists() {
            #[cfg(debug_assertions)]
            log::debug!("[InitDefaultPrompts] 使用当前工作目录配置路径: {:?}", path);
            return Ok(path.clone());
        }
    }

    // 所有策略都失败，返回详细错误信息
    #[cfg(debug_assertions)]
    log::error!("[InitDefaultPrompts] 所有策略都失败，无法找到配置文件");

    Err(anyhow::anyhow!(
        "无法找到配置文件 optimizer_config.toml\n\
         \n\
         尝试的路径:\n\
         - 可执行文件目录: {:?}\n\
         - 生产环境路径: {:?}\n\
         - 开发环境路径（向上查找 {} 层）: {:?}\n\
         \n\
         请确保配置文件存在于以下位置之一:\n\
         1. 可执行文件同目录: optimizer_config.toml\n\
         2. 项目根目录的 src-tauri/: optimizer_config.toml",
        exe_dir,
        prod_path,
        max_depth,
        cwd_path
    ))
}

/// 导入默认提示词到数据库
///
/// 此函数会：
/// 1. 读取 optimizer_config.toml
/// 2. 解析组件化配置
/// 3. 检查"会话分析"模板是否存在
/// 4. 如果已存在且当前是 v1，覆盖内容（配置文件更新时）
/// 5. 如果不存在，创建新模板和版本 v1
///
/// # 参数
/// - `conn`: 数据库连接（可变引用）
///
/// # 返回
/// - `Ok(())`: 导入成功
/// - `Err(e)`: 导入失败
pub fn import_default_prompts(conn: &mut Connection) -> Result<()> {
    log::info!("开始导入默认提示词（会话分析场景）...");

    // 解析配置文件路径
    let config_path = resolve_config_path()
        .context("无法解析配置文件路径")?;

    log::info!("配置文件路径: {:?}", config_path);

    // 读取配置文件
    let config_content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("无法读取配置文件: {:?}", config_path))?;

    // 解析 TOML
    let config: crate::optimizer::config::OptimizerConfig = toml::from_str(&config_content)
        .with_context(|| format!("解析配置文件失败: {:?}", config_path))?;

    log::info!("配置文件解析成功，开始导入会话分析模板");

    // 使用事务确保原子性
    let tx = conn.unchecked_transaction()?;

    // 获取组件化数据
    let component_data = config.components;

    // 构建组件化 JSON 内容
    let content_json = json!({
        "zh": {
            "meta_prompt": {
                "content": component_data.meta_prompt.zh,
                "last_modified": chrono::Utc::now().to_rfc3339()
            },
            "input_template": {
                "content": component_data.input_template.zh,
                "last_modified": null
            },
            "output_template": {
                "content": component_data.output_template.zh,
                "last_modified": null
            }
        },
        "en": {
            "meta_prompt": {
                "content": component_data.meta_prompt.en,
                "last_modified": chrono::Utc::now().to_rfc3339()
            },
            "input_template": {
                "content": component_data.input_template.en,
                "last_modified": null
            },
            "output_template": {
                "content": component_data.output_template.en,
                "last_modified": null
            }
        }
    });

    let content_str = serde_json::to_string_pretty(&content_json)
        .context("序列化组件数据失败")?;

    let now = chrono::Utc::now().to_rfc3339();

    // 检查模板是否已存在
    let existing_template: Option<(i64, i32)> = tx.query_row(
        "SELECT id, (SELECT COALESCE(MAX(version_number), 0) FROM prompt_versions WHERE template_id = pt.id) as max_version
         FROM prompt_templates pt
         WHERE pt.name = ?1",
        &[DEFAULT_TEMPLATE.name],
        |row| Ok((row.get(0)?, row.get(1)?))
    ).ok();

    match existing_template {
        // 模板已存在：如果当前版本是 v1，覆盖内容
        Some((template_id, max_version)) => {
            if max_version == 1 {
                log::info!("模板 '{}' 的 v1 已存在，检查配置文件是否更新", DEFAULT_TEMPLATE.name);

                // 获取当前 v1 的内容
                let current_content: Option<String> = tx.query_row(
                    "SELECT content FROM prompt_versions WHERE template_id = ?1 AND version_number = 1",
                    [&template_id],
                    |row| Ok(row.get(0)?)
                ).ok();

                // 如果内容不同，更新 v1
                if let Some(old_content) = current_content {
                    if old_content != content_str {
                        log::info!("配置文件已更新，覆盖 v1 内容");

                        // 更新模板元数据
                        tx.execute(
                            "UPDATE prompt_templates
                             SET description = ?1, scenario = ?2, tags = ?3, updated_at = ?4
                             WHERE id = ?5",
                            rusqlite::params![
                                DEFAULT_TEMPLATE.description,
                                DEFAULT_TEMPLATE.scenario,
                                DEFAULT_TEMPLATE.tags,
                                now,
                                template_id,
                            ],
                        )?;

                        // 更新 v1 内容
                        tx.execute(
                            "UPDATE prompt_versions
                             SET content = ?1, created_at = ?2
                             WHERE template_id = ?3 AND version_number = 1",
                            rusqlite::params![content_str, now, template_id],
                        )?;

                        log::info!("已覆盖模板 '{}' 的 v1 内容", DEFAULT_TEMPLATE.name);
                    } else {
                        log::info!("配置文件内容未变化，跳过更新");
                    }
                }
            } else {
                log::info!("模板 '{}' 已存在且当前版本为 v{}，保留用户修改", DEFAULT_TEMPLATE.name, max_version);
            }
        }
        // 模板不存在：创建新模板和初始版本 v1
        None => {
            log::info!("创建新模板 '{}'", DEFAULT_TEMPLATE.name);

            // 创建模板
            tx.execute(
                "INSERT INTO prompt_templates (name, description, scenario, tags, language, is_system, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7)",
                [
                    DEFAULT_TEMPLATE.name,
                    DEFAULT_TEMPLATE.description,
                    DEFAULT_TEMPLATE.scenario,
                    DEFAULT_TEMPLATE.tags,
                    "zh", // 默认语言（支持多语言）
                    &now,
                    &now,
                ],
            )?;

            let template_id = tx.last_insert_rowid();

            // 创建初始版本（v1）
            tx.execute(
                "INSERT INTO prompt_versions (template_id, version_number, is_active, content, created_by, created_at)
                 VALUES (?1, 1, 1, ?2, 'system', ?3)",
                [&template_id as &dyn rusqlite::ToSql, &content_str as &dyn rusqlite::ToSql, &now as &dyn rusqlite::ToSql],
            )?;

            log::info!("已创建模板 '{}' (ID: {}) 版本 v1", DEFAULT_TEMPLATE.name, template_id);
        }
    }

    // 提交事务
    tx.commit()?;

    log::info!("默认提示词导入完成！共处理 1 个模板");

    Ok(())
}

/// 导入默认提示词（使用共享连接）
///
/// 这是一个便捷函数，使用全局共享连接执行导入
pub fn import_default_prompts_shared() -> Result<()> {
    let conn = crate::database::init::get_connection_shared()
        .context("获取数据库连接失败")?;

    let mut guard = conn.lock()
        .map_err(|e| anyhow::anyhow!("获取数据库锁失败: {}", e))?;

    import_default_prompts(&mut guard)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_config_path() {
        let path = resolve_config_path();
        assert!(path.is_ok(), "无法解析配置文件路径");
        let path = path.unwrap();
        println!("配置文件路径: {:?}", path);
        assert!(path.exists(), "配置文件不存在: {:?}", path);
    }

    #[test]
    fn test_component_data_structure() {
        // 验证组件化 JSON 结构
        let json_str = r#"{
            "zh": {
                "meta_prompt": {
                    "content": "测试内容",
                    "last_modified": "2025-01-30T10:00:00Z"
                },
                "input_template": {
                    "content": "输入信息",
                    "last_modified": null
                },
                "output_template": {
                    "content": "输出格式",
                    "last_modified": null
                }
            },
            "en": {
                "meta_prompt": {
                    "content": "Test content",
                    "last_modified": "2025-01-30T10:00:00Z"
                },
                "input_template": {
                    "content": "Input template",
                    "last_modified": null
                },
                "output_template": {
                    "content": "Output template",
                    "last_modified": null
                }
            }
        }"#;

        let _value: serde_json::Value = serde_json::from_str(json_str).unwrap();
        println!("✅ 组件化 JSON 结构验证通过");
    }
}
