//! 提示词版本管理 Tauri 命令
//!
//! 提供前端调用的提示词版本管理接口

use crate::database::prompt_versions::PromptVersionRepository;
use crate::database::models::{
    PromptTemplate, PromptVersion, PromptComponent, PromptParameter,
    PromptVersionDiff,
};

/// 获取所有提示词模板
#[tauri::command]
pub async fn cmd_get_prompt_templates() -> Result<Vec<PromptTemplate>, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.list_templates()
        .map_err(|e| e.to_string())
}

/// 根据名称获取提示词模板
#[tauri::command]
pub async fn cmd_get_prompt_template_by_name(name: String) -> Result<Option<PromptTemplate>, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.get_template_by_name(&name)
        .map_err(|e| e.to_string())
}

/// 根据场景获取提示词模板
#[tauri::command]
pub async fn cmd_get_prompt_template_by_scenario(scenario: String) -> Result<Option<PromptTemplate>, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.get_template_by_scenario(&scenario)
        .map_err(|e| e.to_string())
}

/// 获取模板的所有版本
#[tauri::command]
pub async fn cmd_get_prompt_versions(template_id: i64) -> Result<Vec<PromptVersion>, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.list_versions(template_id)
        .map_err(|e| e.to_string())
}

/// 获取模板的激活版本
#[tauri::command]
pub async fn cmd_get_active_prompt_version(template_id: i64) -> Result<Option<PromptVersion>, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.get_active_version(template_id)
        .map_err(|e| e.to_string())
}

/// 根据版本号获取版本
#[tauri::command]
pub async fn cmd_get_prompt_version_by_number(
    template_id: i64,
    version_number: i32,
) -> Result<Option<PromptVersion>, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.get_version_by_number(template_id, version_number)
        .map_err(|e| e.to_string())
}

/// 激活指定版本（软回滚）
#[tauri::command]
pub async fn cmd_activate_prompt_version(
    template_id: i64,
    version_number: i32,
) -> Result<PromptVersion, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.activate_version(template_id, version_number)
        .map_err(|e| e.to_string())
}

/// 硬回滚：创建新版本，复制目标版本的内容并激活
#[tauri::command]
pub async fn cmd_rollback_prompt_version_hard(
    template_id: i64,
    version_number: i32,
    comment: Option<String>,
) -> Result<PromptVersion, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.rollback_to_version_hard(template_id, version_number, comment, "user")
        .map_err(|e| e.to_string())
}

/// 保存新版本（创建并激活）
#[tauri::command]
pub async fn cmd_save_prompt_version(
    template_id: i64,
    content: String,
    components: Vec<PromptComponent>,
    parameters: Vec<PromptParameter>,
    created_by: String,
) -> Result<PromptVersion, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.create_and_activate_version(
        template_id,
        content,
        components,
        parameters,
        &created_by,
    )
    .map_err(|e| e.to_string())
}

/// 对比两个版本
#[tauri::command]
pub async fn cmd_compare_prompt_versions(
    template_id: i64,
    from_version: i32,
    to_version: i32,
) -> Result<PromptVersionDiff, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.compare_versions(template_id, from_version, to_version)
        .map_err(|e| e.to_string())
}

/// 获取版本的所有组件（旧版，兼容保留）
#[tauri::command]
pub async fn cmd_get_prompt_components_by_id(version_id: i64) -> Result<Vec<PromptComponent>, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.list_components(version_id)
        .map_err(|e| e.to_string())
}

/// 获取版本的所有参数
#[tauri::command]
pub async fn cmd_get_prompt_parameters(version_id: i64) -> Result<Vec<PromptParameter>, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.list_parameters(version_id)
        .map_err(|e| e.to_string())
}

/// 获取版本之间的变更记录
#[tauri::command]
pub async fn cmd_get_prompt_version_changes(
    template_id: i64,
    from_version: Option<i32>,
    to_version: i32,
) -> Result<Vec<crate::database::models::PromptChange>, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.get_changes_between_versions(template_id, from_version, to_version)
        .map_err(|e| e.to_string())
}

/// 统一的提示词列表查询接口（从版本管理系统读取）
///
/// 这个命令用于替代旧的 cmd_get_prompts，从 prompt_templates 和 prompt_versions
/// 读取数据并扁平化输出，支持多语言版本的展示
///
/// # 参数
/// - `scenario`: 可选的场景过滤（如 "session_analysis"）
/// - `language`: 可选的语言过滤（如 "zh"、"en"）
/// - `search`: 可选的搜索关键词（匹配 name、description、content）
///
/// # 返回
/// 返回一个扁平化的提示词列表，每个元素包含：
/// - id: 版本 ID（兼容旧的前端代码）
/// - template_id: 模板 ID
/// - name: 模板名称
/// - content: 版本内容
/// - description: 模板描述
/// - scenario: 场景
/// - language: 从 metadata 中提取的语言
/// - is_system: 是否系统级
/// - is_active: 版本是否激活
/// - version_number: 版本号
/// - created_at: 创建时间
#[tauri::command]
pub async fn cmd_get_prompts_unified(
    scenario: Option<String>,
    language: Option<String>,
    search: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    repo.list_prompts_unified(
        scenario.as_deref(),
        language.as_deref(),
        search.as_deref(),
    )
    .map_err(|e| e.to_string())
}

// ==================== 组件化提示词管理命令 ====================

/// 获取模板的组件化数据
///
/// 返回指定模板的当前激活版本的组件化数据，包含：
/// - meta_prompt: 可编辑的 Meta-Prompt 组件（中英文）
/// - input_template: 只读的输入信息模板（中英文）
/// - output_template: 只读的输出格式模板（中英文）
///
/// # 参数
/// - `template_name`: 模板名称（如 "session_analysis"）
///
/// # 返回
/// 组件化数据的 JSON 对象
#[tauri::command]
pub async fn cmd_get_prompt_components(
    template_name: String,
) -> Result<serde_json::Value, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    // 获取模板
    let template = repo.get_template_by_name(&template_name)
        .map_err(|e| format!("获取模板失败: {}", e))?
        .ok_or_else(|| format!("模板 '{}' 不存在", template_name))?;

    // 获取激活版本
    let version = repo.get_active_version(template.id.unwrap())
        .map_err(|e| format!("获取激活版本失败: {}", e))?
        .ok_or_else(|| format!("模板 '{}' 没有激活版本", template_name))?;

    // 解析组件化 JSON
    let content_json: serde_json::Value = serde_json::from_str(&version.content)
        .map_err(|e| format!("解析组件数据失败: {}", e))?;

    Ok(content_json)
}

/// 更新模板的组件化数据并创建新版本
///
/// 此命令会：
/// 1. 解析传入的组件化数据
/// 2. 合并未修改的语言（使用上一版本的内容）
/// 3. 创建新版本（版本号 +1）
/// 4. 激活新版本
///
/// # 参数
/// - `template_name`: 模板名称（如 "session_analysis"）
/// - `components_data`: 组件化数据 JSON 字符串
/// - `updated_languages`: 已修改的语言列表（如 ["zh"]）
///
/// # 返回
/// 新创建的版本信息
#[tauri::command]
pub async fn cmd_update_prompt_components(
    template_name: String,
    components_data: String,
    updated_languages: Vec<String>,
) -> Result<PromptVersion, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    // 获取模板
    let template = repo.get_template_by_name(&template_name)
        .map_err(|e| format!("获取模板失败: {}", e))?
        .ok_or_else(|| format!("模板 '{}' 不存在", template_name))?;

    let template_id = template.id.unwrap();

    // 获取当前激活版本
    let current_version = repo.get_active_version(template_id)
        .map_err(|e| format!("获取当前版本失败: {}", e))?
        .ok_or_else(|| format!("模板 '{}' 没有激活版本", template_name))?;

    // 解析新的组件化数据
    let new_data: serde_json::Value = serde_json::from_str(&components_data)
        .map_err(|e| format!("解析组件数据失败: {}", e))?;

    // 解析当前版本的组件化数据
    let current_data: serde_json::Value = serde_json::from_str(&current_version.content)
        .map_err(|e| format!("解析当前组件数据失败: {}", e))?;

    // 合并数据：对于未修改的语言，使用当前版本的内容
    let merged_data = merge_component_data(
        &current_data,
        &new_data,
        &updated_languages,
    );

    // 序列化合并后的数据
    let merged_content = serde_json::to_string_pretty(&merged_data)
        .map_err(|e| format!("序列化组件数据失败: {}", e))?;

    // 获取下一个版本号
    let versions = repo.list_versions(template_id)
        .map_err(|e| format!("获取版本列表失败: {}", e))?;
    let next_version = versions.iter()
        .map(|v| v.version_number)
        .max()
        .unwrap_or(0) + 1;

    // 创建新版本
    let now = chrono::Utc::now().to_rfc3339();
    let new_version = repo.create_version_direct(
        template_id,
        next_version,
        merged_content,
        "user",
        &now,
    ).map_err(|e| format!("创建新版本失败: {}", e))?;

    // 激活新版本
    repo.activate_version(template_id, next_version)
        .map_err(|e| format!("激活新版本失败: {}", e))?;

    Ok(new_version)
}

/// 检查配置文件是否已更新（用于显示警告）
///
/// 此命令比较当前激活版本 v1 的内容和配置文件的内容，
/// 如果不同，说明配置文件已更新，需要显示警告。
///
/// # 参数
/// - `template_name`: 模板名称（如 "session_analysis"）
///
/// # 返回
/// - `true`: 配置文件已更新
/// - `false`: 配置文件未更新
#[tauri::command]
pub async fn cmd_check_config_updated(
    template_name: String,
) -> Result<bool, String> {
    use crate::database::init_default_prompts::resolve_config_path;
    use crate::optimizer::config::OptimizerConfig;

    // 解析配置文件路径
    let config_path = resolve_config_path()
        .map_err(|e| format!("解析配置路径失败: {}", e))?;

    // 读取配置文件
    let config_content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    // 解析配置
    let config: OptimizerConfig = toml::from_str(&config_content)
        .map_err(|e| format!("解析配置失败: {}", e))?;

    // 获取组件化数据
    let component_data = config.components;

    // 构建配置文件的组件化 JSON
    let config_json = serde_json::json!({
        "zh": {
            "meta_prompt": {
                "content": component_data.meta_prompt.zh,
                "last_modified": null
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
                "last_modified": null
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

    let config_str = serde_json::to_string_pretty(&config_json)
        .map_err(|e| format!("序列化配置数据失败: {}", e))?;

    // 获取当前 v1 版本的内容
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    let template = repo.get_template_by_name(&template_name)
        .map_err(|e| format!("获取模板失败: {}", e))?
        .ok_or_else(|| format!("模板 '{}' 不存在", template_name))?;

    let v1_version = repo.get_version_by_number(template.id.unwrap(), 1)
        .map_err(|e| format!("获取 v1 版本失败: {}", e))?
        .ok_or_else(|| format!("v1 版本不存在"))?;

    // 比较：如果内容不同，说明配置文件已更新
    // 注意：忽略 last_modified 字段的差异
    let v1_data: serde_json::Value = serde_json::from_str(&v1_version.content)
        .map_err(|e| format!("解析 v1 数据失败: {}", e))?;

    let config_data: serde_json::Value = serde_json::from_str(&config_str)
        .map_err(|e| format!("解析配置数据失败: {}", e))?;

    // 比较时忽略 last_modified 字段
    let has_update = compare_component_data_without_timestamp(&v1_data, &config_data);

    Ok(has_update)
}

/// 合并组件化数据
///
/// 对于未修改的语言，使用当前版本的内容
fn merge_component_data(
    current: &serde_json::Value,
    new: &serde_json::Value,
    updated_languages: &[String],
) -> serde_json::Value {
    let mut result = current.clone();

    // 遍历新数据中的语言
    if let Some(new_obj) = new.as_object() {
        for (lang, new_components) in new_obj {
            // 如果这个语言被更新了，直接使用新数据
            if updated_languages.contains(lang) {
                if let Some(result_obj) = result.as_object_mut() {
                    result_obj.insert(lang.clone(), new_components.clone());
                }
            }
            // 如果语言未被更新，保留当前版本的数据（已在 result 中）
        }
    }

    result
}

/// 比较组件化数据（忽略时间戳）
fn compare_component_data_without_timestamp(
    v1: &serde_json::Value,
    config: &serde_json::Value,
) -> bool {
    // 递归比较两个 JSON，忽略 last_modified 字段
    fn compare_ignore_timestamp(a: &serde_json::Value, b: &serde_json::Value) -> bool {
        match (a, b) {
            (serde_json::Value::Object(a_map), serde_json::Value::Object(b_map)) => {
                // 过滤掉 last_modified 字段
                let a_filtered: std::collections::HashMap<_, _> = a_map.iter()
                    .filter(|(k, _)| *k != "last_modified")
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();

                let b_filtered: std::collections::HashMap<_, _> = b_map.iter()
                    .filter(|(k, _)| *k != "last_modified")
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();

                if a_filtered.len() != b_filtered.len() {
                    return false;
                }

                for (key, a_val) in &a_filtered {
                    if let Some(b_val) = b_filtered.get(key) {
                        if !compare_ignore_timestamp(a_val, b_val) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }

                true
            }
            (serde_json::Value::Array(a_arr), serde_json::Value::Array(b_arr)) => {
                if a_arr.len() != b_arr.len() {
                    return false;
                }
                a_arr.iter().zip(b_arr.iter())
                    .all(|(a, b)| compare_ignore_timestamp(a, b))
            }
            (a_val, b_val) => a_val == b_val,
        }
    }

    !compare_ignore_timestamp(v1, config)
}

/// 清理所有非会话分析的模板和版本
///
/// 注意：此操作会删除所有非"会话分析"场景的模板及其所有版本
/// 仅保留 session_analysis 模板
#[tauri::command]
pub async fn cmd_cleanup_legacy_templates() -> Result<usize, String> {
    let repo = PromptVersionRepository::from_default_db()
        .map_err(|e| format!("创建版本仓库失败: {}", e))?;

    // 获取所有模板
    let templates = repo.list_templates()
        .map_err(|e| format!("获取模板列表失败: {}", e))?;

    let mut deleted_count = 0;

    // 删除所有非 session_analysis 的模板
    for template in templates {
        if template.scenario != "session_analysis" {
            if let Some(template_id) = template.id {
                // 删除模板及其所有版本
                repo.delete_template(template_id)
                    .map_err(|e| format!("删除模板 {} 失败: {}", template.name, e))?;
                deleted_count += 1;
            }
        }
    }

    Ok(deleted_count)
}
