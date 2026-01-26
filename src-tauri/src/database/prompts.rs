//! 提示词管理 Repository
//!
//! 提供 prompts 表的 CRUD 操作

use anyhow::Result;
use rusqlite::{Connection, params};
use super::models::Prompt;

/// 提示词 Repository
pub struct PromptRepository;

impl PromptRepository {
    /// 创建新提示词
    pub fn create(conn: &Connection, prompt: &Prompt) -> Result<i64> {
        prompt.validate()?;

        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO prompts (
                name, content, description, scenario, category,
                is_default, is_system, language, version,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                &prompt.name,
                &prompt.content,
                &prompt.description,
                &prompt.scenario,
                &prompt.category,
                prompt.is_default as i32,
                prompt.is_system as i32,
                &prompt.language,
                prompt.version,
                now.clone(),
                now
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// 根据 ID 获取提示词
    pub fn get(conn: &Connection, id: i64) -> Result<Option<Prompt>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, content, description, scenario, category,
                    is_default, is_system, language, version, created_at, updated_at
             FROM prompts WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_prompt(row)?))
        } else {
            Ok(None)
        }
    }

    /// 根据名称获取提示词
    pub fn get_by_name(conn: &Connection, name: &str) -> Result<Option<Prompt>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, content, description, scenario, category,
                    is_default, is_system, language, version, created_at, updated_at
             FROM prompts WHERE name = ?1"
        )?;

        let mut rows = stmt.query(params![name])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_prompt(row)?))
        } else {
            Ok(None)
        }
    }

    /// 获取所有提示词
    ///
    /// # 参数
    /// - `conn`: 数据库连接
    /// - `scenario`: 可选的场景过滤条件
    /// - `language`: 可选的语言过滤条件
    /// - `search`: 可选的搜索关键词（匹配名称或描述）
    pub fn list(
        conn: &Connection,
        scenario: Option<&str>,
        language: Option<&str>,
        search: Option<&str>,
    ) -> Result<Vec<Prompt>> {
        let mut query = "SELECT id, name, content, description, scenario, category,
                            is_default, is_system, language, version, created_at, updated_at
                        FROM prompts WHERE 1=1"
                            .to_string();

        let mut params = Vec::new();

        if let Some(scenario) = scenario {
            query.push_str(" AND scenario = ?");
            params.push(scenario.to_string());
        }

        if let Some(language) = language {
            query.push_str(" AND language = ?");
            params.push(language.to_string());
        }

        if let Some(search_term) = search {
            // 使用 LIKE 进行模糊搜索（名称或描述中包含关键词）
            query.push_str(" AND (name LIKE ? OR description LIKE ?)");
            let search_pattern = format!("%{}%", search_term);
            params.push(search_pattern.clone());
            params.push(search_pattern);
        }

        query.push_str(" ORDER BY is_default DESC, created_at DESC");

        let mut stmt = conn.prepare(&query)?;

        let param_refs: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();

        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            Ok(Self::row_to_prompt(row)?)
        })?;

        let mut prompts = Vec::new();
        for row in rows {
            prompts.push(row?);
        }

        Ok(prompts)
    }

    /// 更新提示词
    pub fn update(conn: &Connection, prompt: &Prompt) -> Result<()> {
        if prompt.id.is_none() {
            return Err(anyhow::anyhow!("更新提示词时 ID 不能为空"));
        }

        prompt.validate()?;

        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE prompts SET
                content = ?1,
                description = ?2,
                category = ?3,
                version = version + 1,
                updated_at = ?4
             WHERE id = ?5",
            params![
                &prompt.content,
                &prompt.description,
                &prompt.category,
                now,
                prompt.id,
            ],
        )?;

        Ok(())
    }

    /// 删除提示词
    pub fn delete(conn: &Connection, id: i64) -> Result<()> {
        // 检查是否为系统内置提示词
        let prompt = Self::get(conn, id)?
            .ok_or_else(|| anyhow::anyhow!("提示词不存在"))?;

        if prompt.is_system {
            return Err(anyhow::anyhow!(
                "系统内置提示词不可删除（名称: {}）",
                prompt.name
            ));
        }

        conn.execute("DELETE FROM prompts WHERE id = ?1", params![id])?;

        Ok(())
    }

    /// 重置为默认提示词
    pub fn reset_default(conn: &Connection, name: &str) -> Result<()> {
        // 获取默认提示词内容（从硬编码常量）
        let default_content = Self::get_default_content(name)?;

        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE prompts SET
                content = ?1,
                version = version + 1,
                updated_at = ?2
             WHERE name = ?3",
            params![default_content, now, name],
        )?;

        Ok(())
    }

    /// 获取默认提示词内容（从 optimizer_config.toml 读取）- 公共方法
    ///
    /// 此方法现在是公共的，可供其他模块（如 optimizer）使用
    ///
    /// 优先从 optimizer_config.toml 读取配置，如果读取失败则使用硬编码 fallback
    pub fn get_default_content(name: &str) -> Result<String> {
        // 从提示词名称提取语言标识
        // session_analysis_zh -> zh
        // session_analysis_en -> en
        let language = if name.ends_with("_zh") {
            "zh"
        } else if name.ends_with("_en") {
            "en"
        } else {
            return Err(anyhow::anyhow!("无法从提示词名称提取语言标识: {}", name));
        };

        // 尝试从全局 ConfigManager 读取配置
        if let Some(config_manager) = crate::optimizer::config::get_config_manager() {
            let meta_prompt = config_manager.get_meta_prompt(language);

            // 获取完整的提示词结构模板
            let prompt_structure = config_manager.get_prompt_structure(language);

            // 组装完整的提示词内容
            // 替换 {{meta_prompt}} 变量
            let full_prompt = prompt_structure.replace("{{meta_prompt}}", &meta_prompt);

            return Ok(full_prompt);
        }

        // Fallback: ConfigManager 未初始化，使用硬编码内容
        // 这些内容与 optimizer_config.toml 中的 template 保持一致
        #[cfg(debug_assertions)]
        eprintln!("[PromptRepository] ConfigManager 未初始化，使用硬编码 fallback 提示词");

        Self::get_fallback_default_content(name)
    }

    /// 获取硬编码的默认提示词内容（fallback）
    ///
    /// 此方法提供与 optimizer_config.toml 中相同的硬编码内容
    /// 用于 ConfigManager 未初始化或读取失败时的回退
    fn get_fallback_default_content(name: &str) -> Result<String> {
        match name {
            "session_analysis_zh" => {
                // 与 optimizer_config.toml 中的 meta_prompt.template_zh 一致
                let meta_prompt = r#"你是一位专业的提示词工程师。你的任务是分析用户目标和会话历史，生成一个结构化、高信噪比的提示词，以指导 AI 编程助手高效地完成任务。

## 你的分析与构建步骤

1. **分析目标与上下文**:
   - **确立核心需求**: 仔细阅读用户目标。
   - **全局历史回顾**: 综合浏览整段历史会话（寻找主线任务），避免仅基于最近一轮对话做出片面判断。
   - **判断目标偏离度**: 根据上下文与下一步目标的关联程度，判断下一步目标对于整个会话来说是否噪音。
   - **确定会话阶段**: 明确当前处于任务探索、代码实现、调试修复还是重构优化阶段。

2. **构建结构化提示词**: 根据分析结果，填充以下"输出格式"的各个部分。

## 限制条件

- 提示词应简洁明了，重点突出，避免不必要的寒暄或背景信息
- 必须根据**上下文**与**下一步目标**的关联程度严格判断目标偏离程度，是否同一个项目不在判断范围内
- 严禁仅参考最新对话，必须建立对**整体历史会话**的认知
- 突出关键技术点和注意事项"#;

                // 与 optimizer_config.toml 中的 prompt_structure.structure_zh 一致
                let structure = r#"{{meta_prompt}}

## 输入信息

- **下一步目标**: {{goal}}
- **相关历史会话**:
{{sessions}}

## 输出格式

请严格按照以下结构生成提示词，并用 Markdown 标题标识：

### **目标偏离程度**

判断**下一步目标**与**相关历史会话**的偏离程度，给出你的建议（例如：可以继续会话、建议用户开启新的会话、如偏离程度高则截断回答直接建议用户开启新会话）

### **任务目标**

简要描述要完成的任务和目标。

### **具体步骤**

提供清晰、可执行的步骤列表，指导 AI 如何完成任务。

### **预期输出**

描述期望的最终产出物是什么，并说明其关键要求。

---

请基于上述信息，生成一个优化的提示词。"#;

                // 组装完整提示词
                let full_prompt = structure.replace("{{meta_prompt}}", meta_prompt);
                Ok(full_prompt.to_string())
            }
            "session_analysis_en" => {
                // 与 optimizer_config.toml 中的 meta_prompt.template_en 一致
                let meta_prompt = r#"You are a professional prompt engineer. Your task is to analyze user goals and conversation history to generate a structured, high signal-to-noise ratio prompt that will guide an AI programming assistant to efficiently complete tasks.

## Your Analysis and Construction Steps

1. **Analyze Goals and Context**:
   - **Establish Core Requirements**: Carefully read the user's goal.
   - **Global History Review**: Comprehensively review the entire conversation history (seeking the main task thread), avoiding making one-sided judgments based only on the most recent exchange.
   - **Determine Goal Drift**: Based on the relevance between context and next step, assess whether the next goal represents noise for the overall conversation.
   - **Identify Conversation Stage**: Determine whether you are in task exploration, implementation, debugging/refinement, or refactoring/optimization phase.

2. **Construct Structured Prompt**: Based on your analysis, populate each section of the "Output Format" below.

## Constraints

- The prompt should be concise and focused, avoiding unnecessary pleasantries or background information
- Must rigorously assess goal drift based on the relevance between **context** and **next step**, irrespective of whether it belongs to the same project
- Strictly prohibit referencing only the latest conversation; must establish cognition of the **entire conversation history**
- Highlight key technical points and considerations"#;

                // 与 optimizer_config.toml 中的 prompt_structure.structure_en 一致
                let structure = r#"{{meta_prompt}}

## Input Information

- **Next Goal**: {{goal}}
- **Related Conversation History**:
{{sessions}}

## Output Format

Please strictly follow the structure below to generate a prompt, using Markdown headings:

### **Goal Drift Level**

Assess the degree of drift between the **next goal** and **related conversation history**, and provide your recommendation (e.g., can continue conversation, suggest user start a new conversation, if drift is high then truncate response and directly suggest user start a new conversation)

### **Task Objective**

Briefly describe the task and objectives to be completed.

### **Specific Steps**

Provide clear, actionable step-by-step instructions to guide the AI in completing the task.

### **Expected Output**

Describe what the final deliverable should be and specify its key requirements.

---

Please generate an optimized prompt based on the information above."#;

                // 组装完整提示词
                let full_prompt = structure.replace("{{meta_prompt}}", meta_prompt);
                Ok(full_prompt.to_string())
            }
            _ => Err(anyhow::anyhow!("未知的默认提示词名称: {}", name)),
        }
    }

    /// 获取 Fallback 提示词内容（公共方法）
    ///
    /// 当数据库中没有提示词时，使用此方法获取硬编码的 fallback 内容
    ///
    /// # 参数
    /// - `language`: 语言标识（"zh" 或 "en"）
    ///
    /// # 返回
    /// 返回硬编码的提示词内容
    pub fn get_fallback_prompt_content(language: &str) -> String {
        match language {
            "en" => {
                r#"You are a Claude Code pair programming assistant. Please analyze the conversation log below (including user instructions, Claude's operations, and tool-returned file contents/errors).

Tasks:
1. **Focus Check**: Has Claude fallen into an infinite loop? Is it repeatedly reading irrelevant files? Is it ignoring errors?
2. **Prompt Generation**: Write a **Chinese instruction** that the user can send directly to Claude.
   - If Claude is off track: Write a stern corrective instruction.
   - If Claude is doing well: Write an instruction to advance to the next step, referencing the file context just read (e.g., "Based on main.py just read...").

Output Format:
---
[Status]: [Normal / Lost / Error Loop]
[Analysis]: (Brief analysis of current situation)
[Suggested Prompt]:
(Your prompt content)
---
"#.to_string()
            }
            _ => {
                // 默认中文
                r#"你是一个 Claude Code 结对编程助手。请分析下方的会话日志（包含用户指令、Claude 的操作、以及工具返回的文件内容/报错）。

任务：
1. **判断焦点 (Focus Check)**：Claude 是否陷入了死循环？是否在反复读取无关文件？是否无视了报错？
2. **生成提示词 (Prompt Generation)**：为用户写一段可以直接发送给 Claude 的**中文指令**。
   - 如果 Claude 走偏了：写一段严厉的纠正指令。
   - 如果 Claude 做得对：写一段推进下一步的指令，并引用刚才读取到的文件上下文（例如："基于刚才读取的 main.py..."）。

输出格式：
---
【状态】: [正常 / 迷失 / 报错循环]
【分析】: (简短分析当前情况)
【建议提示词】:
(你的 Prompt 内容）
---
"#.to_string()
            }
        }
    }

    /// 从数据库行转换为 Prompt 对象
    fn row_to_prompt(row: &rusqlite::Row) -> std::result::Result<Prompt, rusqlite::Error> {
        Ok(Prompt {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            content: row.get(2)?,
            description: row.get(3)?,
            scenario: row.get(4)?,
            category: row.get(5)?,
            is_default: row.get::<_, i32>(6)? == 1,
            is_system: row.get::<_, i32>(7)? == 1,
            language: row.get(8)?,
            version: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_get_prompt() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();

        // 执行迁移（使用公开的函数）
        crate::database::migrations::migrate_v17(&mut conn).unwrap();

        // 创建提示词
        let prompt = Prompt::new(
            "test_prompt".to_string(),
            "Test content".to_string(),
            "session_analysis".to_string(),
            "zh".to_string(),
        );

        let id = PromptRepository::create(&conn, &prompt).unwrap();
        assert!(id > 0);

        // 读取提示词
        let retrieved = PromptRepository::get(&conn, id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test_prompt");
    }

    #[test]
    fn test_unique_name_constraint() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();

        crate::database::migrations::migrate_v17(&mut conn).unwrap();

        let prompt = Prompt::new(
            "duplicate_name".to_string(),
            "Content 1".to_string(),
            "session_analysis".to_string(),
            "zh".to_string(),
        );

        // 第一次创建成功
        PromptRepository::create(&conn, &prompt).unwrap();

        // 第二次创建应该失败（名称冲突）
        let result = PromptRepository::create(&conn, &prompt);
        assert!(result.is_err());
    }

    #[test]
    fn test_system_prompt_deletion_protection() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();

        crate::database::migrations::migrate_v17(&mut conn).unwrap();

        // 尝试删除系统提示词
        let system_prompt = PromptRepository::get_by_name(&conn, "session_analysis_zh")
            .unwrap()
            .unwrap();

        let result = PromptRepository::delete(&conn, system_prompt.id.unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("系统内置提示词不可删除"));
    }

    #[test]
    fn test_reset_default_prompt() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();

        crate::database::migrations::migrate_v17(&mut conn).unwrap();

        // 修改默认提示词
        let mut prompt = PromptRepository::get_by_name(&conn, "session_analysis_zh")
            .unwrap()
            .unwrap();
        prompt.content = "Modified content".to_string();

        PromptRepository::update(&conn, &prompt).unwrap();

        // 重置为默认
        PromptRepository::reset_default(&conn, "session_analysis_zh").unwrap();

        // 验证内容恢复
        let reset_prompt = PromptRepository::get_by_name(&conn, "session_analysis_zh")
            .unwrap()
            .unwrap();

        assert!(!reset_prompt.content.contains("Modified content"));
        assert!(reset_prompt.content.contains("Claude Code 结对编程助手"));
    }
}
