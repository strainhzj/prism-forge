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

    /// 获取默认提示词内容（硬编码常量）- 公共方法
    ///
    /// 此方法现在是公共的，可供其他模块（如 optimizer）使用
    pub fn get_default_content(name: &str) -> Result<String> {
        match name {
            "session_analysis_zh" => {
                Ok(r#"你是一个 Claude Code 结对编程助手。请分析下方的会话日志（包含用户指令、Claude 的操作、以及工具返回的文件内容/报错）。

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
"#.to_string())
            }
            "session_analysis_en" => {
                Ok(r#"You are a Claude Code pair programming assistant. Please analyze the conversation log below (including user instructions, Claude's operations, and tool-returned file contents/errors).

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
"#.to_string())
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
