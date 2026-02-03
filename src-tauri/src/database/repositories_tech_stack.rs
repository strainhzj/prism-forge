//! 项目技术栈数据库操作
//!
//! 用于管理项目技术栈配置的数据库仓库

use rusqlite::{Connection, params};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 项目技术栈数据结构
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(rename_all = "camelCase")]
pub struct ProjectTechStack {
    /// 数据库 ID
    pub id: i64,
    /// 项目路径
    pub project_path: String,
    /// 技术栈列表
    pub tech_stack: Vec<String>,
    /// 检测方法（auto | manual）
    pub detection_method: String,
    /// 检测来源（CLAUDE.md | README.md | manual）
    pub detection_source: Option<String>,
    /// 是否已确认
    pub is_confirmed: bool,
    /// 最后验证时间
    pub last_verified_at: Option<String>,
}

/// 项目技术栈数据库仓库
pub struct ProjectTechStackRepository {
    db_path: String,
}

impl ProjectTechStackRepository {
    /// 创建新的仓库实例
    pub fn new(db_path: String) -> Self {
        Self { db_path }
    }

    /// 保存或更新项目技术栈
    ///
    /// 使用 ON CONFLICT 实现 upsert 语义
    pub fn upsert(&self, project: &ProjectTechStack) -> anyhow::Result<i64> {
        let conn = Connection::open(&self.db_path)
            .context("无法打开数据库")?;

        conn.execute(
            "INSERT INTO project_tech_stack (project_path, tech_stack, detection_method, detection_source, is_confirmed)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(project_path) DO UPDATE SET
             tech_stack = ?2, detection_method = ?3, detection_source = ?4, is_confirmed = ?5, updated_at = datetime('now', 'localtime')",
            params![
                &project.project_path,
                &serde_json::to_string(&project.tech_stack).unwrap(),
                &project.detection_method,
                &project.detection_source,
                if project.is_confirmed { 1i64 } else { 0i64 },
            ],
        ).context("插入或更新项目技术栈失败")?;

        Ok(conn.last_insert_rowid())
    }

    /// 根据项目路径获取技术栈
    pub fn get_by_path(&self, project_path: &str) -> anyhow::Result<Option<ProjectTechStack>> {
        let conn = Connection::open(&self.db_path)
            .context("无法打开数据库")?;

        let mut stmt = conn.prepare(
            "SELECT id, project_path, tech_stack, detection_method, detection_source, is_confirmed, last_verified_at
             FROM project_tech_stack
             WHERE project_path = ?1"
        ).context("准备查询失败")?;

        let mut rows = stmt.query(params![project_path])
            .context("执行查询失败")?;

        if let Some(row) = rows.next()? {
            let tech_stack_json: String = row.get(2).context("获取 tech_stack 失败")?;
            let tech_stack = serde_json::from_str(&tech_stack_json).unwrap_or_default();

            Ok(Some(ProjectTechStack {
                id: row.get(0).context("获取 id 失败")?,
                project_path: row.get(1).context("获取 project_path 失败")?,
                tech_stack,
                detection_method: row.get(3).context("获取 detection_method 失败")?,
                detection_source: row.get(4).context("获取 detection_source 失败")?,
                is_confirmed: row.get::<_, i64>(5).context("获取 is_confirmed 失败")? == 1,
                last_verified_at: row.get(6).context("获取 last_verified_at 失败")?,
            }))
        } else {
            Ok(None)
        }
    }

    /// 获取已确认的项目技术栈
    pub fn get_confirmed(&self, project_path: &str) -> anyhow::Result<Option<ProjectTechStack>> {
        let conn = Connection::open(&self.db_path)
            .context("无法打开数据库")?;

        let mut stmt = conn.prepare(
            "SELECT id, project_path, tech_stack, detection_method, detection_source, is_confirmed, last_verified_at
             FROM project_tech_stack
             WHERE project_path = ?1 AND is_confirmed = 1"
        ).context("准备查询失败")?;

        let mut rows = stmt.query(params![project_path])
            .context("执行查询失败")?;

        if let Some(row) = rows.next()? {
            let tech_stack_json: String = row.get(2).context("获取 tech_stack 失败")?;
            let tech_stack = serde_json::from_str(&tech_stack_json).unwrap_or_default();

            Ok(Some(ProjectTechStack {
                id: row.get(0).context("获取 id 失败")?,
                project_path: row.get(1).context("获取 project_path 失败")?,
                tech_stack,
                detection_method: row.get(3).context("获取 detection_method 失败")?,
                detection_source: row.get(4).context("获取 detection_source 失败")?,
                is_confirmed: row.get::<_, i64>(5).context("获取 is_confirmed 失败")? == 1,
                last_verified_at: row.get(6).context("获取 last_verified_at 失败")?,
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upsert_and_get() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_tech_stack.db").to_str().unwrap().to_string();

        // 创建表（简化版，仅用于测试）
        let conn = Connection::open(&db_path).unwrap();

        // 先删除旧表，确保测试从干净状态开始
        conn.execute("DROP TABLE IF EXISTS project_tech_stack", [])
            .expect("删除旧表失败");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS project_tech_stack (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_path TEXT NOT NULL UNIQUE,
                tech_stack TEXT,
                detection_method TEXT,
                detection_source TEXT,
                is_confirmed INTEGER NOT NULL DEFAULT 0,
                last_verified_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
            )",
            [],
        ).unwrap();

        let repo = ProjectTechStackRepository::new(db_path);

        let project = ProjectTechStack {
            id: 0,
            project_path: "/test/path".to_string(),
            tech_stack: vec!["Rust".to_string(), "Tauri".to_string()],
            detection_method: "auto".to_string(),
            detection_source: Some("CLAUDE.md".to_string()),
            is_confirmed: false,
            last_verified_at: None,
        };

        // 测试 upsert
        let id = repo.upsert(&project).unwrap();
        assert!(id > 0);

        // 测试 get_by_path
        let retrieved = repo.get_by_path("/test/path").unwrap().unwrap();
        assert_eq!(retrieved.tech_stack, vec!["Rust", "Tauri"]);
        assert_eq!(retrieved.detection_method, "auto");
        assert!(!retrieved.is_confirmed);

        // 测试更新
        let updated = ProjectTechStack {
            id: 0,
            project_path: "/test/path".to_string(),
            tech_stack: vec!["Rust".to_string(), "Tauri".to_string(), "React".to_string()],
            detection_method: "auto".to_string(),
            detection_source: Some("CLAUDE.md".to_string()),
            is_confirmed: true,
            last_verified_at: None,
        };
        repo.upsert(&updated).unwrap();

        let retrieved = repo.get_by_path("/test/path").unwrap().unwrap();
        assert_eq!(retrieved.tech_stack.len(), 3);
        assert!(retrieved.is_confirmed);
    }
}
