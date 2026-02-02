# 任务 1.1 背景：数据库迁移 v19

**任务 ID**: 1.1
**任务名称**: 数据库迁移 v19
**状态**: 待开始
**创建时间**: 2026-02-02
**预计耗时**: 4 小时

---

## 📋 任务概述

创建数据库迁移到版本 19，为会话意图分析功能添加 6 张新表。

---

## 🎯 实现目标

### 核心目标
1. 更新数据库版本号从 18 → 19
2. 创建 6 张新表支持意图分析功能
3. 添加索引优化查询性能
4. 设置外键约束保证数据完整性
5. 添加单元测试验证迁移成功

### 验收标准
- [ ] 数据库迁移成功，无错误
- [ ] 所有 6 张表创建成功
- [ ] 外键约束正确
- [ ] 测试：删除旧数据库，重启应用自动创建新表

---

## 📂 代码上下文

### 当前代码状态

**文件**: `src-tauri/src/database/migrations.rs`

**当前数据库版本**:
```rust
const CURRENT_DB_VERSION: i32 = 18;  // 第 31 行
```

**外键约束状态**: ✅ 已启用
```rust
// 第 41 行
conn.execute("PRAGMA foreign_keys = ON;", [])?;
```

**迁移函数调用位置**: `run_migrations()` 函数 (第 52-102 行)
```rust
for version in (current_version + 1)..=CURRENT_DB_VERSION {
    match version {
        // ...
        18 => migrate_v18(conn)?,
        _ => anyhow::bail!("未知的数据库版本: {}", version),
    }
}
```

### 代码风格模式

**迁移函数标准模式**:
```rust
/// 迁移到版本 19: 创建意图分析表
#[cfg(test)]
pub fn migrate_v19(conn: &mut Connection) -> Result<()> {
    migrate_v19_impl(conn)
}

#[cfg(not(test))]
fn migrate_v19(conn: &mut Connection) -> Result<()> {
    migrate_v19_impl(conn)
}

fn migrate_v19_impl(conn: &mut Connection) -> Result<()> {
    // 1. 创建表
    // 2. 创建索引
    // 3. 创建触发器（如需要）
    // 4. 记录日志

    log::info!("✅ 已完成 v19 迁移");
    Ok(())
}
```

**日志模式**:
```rust
log::info!("✅ 已创建 [表名] 表");
log::warn!("创建 [表名] 失败（可能原因）: {}", e);
```

**索引创建模式**:
```rust
conn.execute(
    "CREATE INDEX IF NOT EXISTS idx_[table_name]_[column_name]
     ON [table_name]([column_name]);",
    [],
)?;
```

---

## 🗄️ 数据库表设计

### 表 1: project_tech_stack (项目技术栈)

**用途**: 存储项目技术栈配置，支持项目级复用

**表结构**:
```sql
CREATE TABLE IF NOT EXISTS project_tech_stack (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_path TEXT NOT NULL UNIQUE,
    tech_stack TEXT,                    -- JSON 数组: ["Rust", "Tauri", "React"]
    detection_method TEXT,              -- "auto" | "manual"
    detection_source TEXT,              -- "package.json" | "Cargo.toml" | "user_input"
    is_confirmed INTEGER NOT NULL DEFAULT 0,  -- 布尔值: 0=未确认, 1=已确认
    last_verified_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);
```

**索引**:
```sql
CREATE INDEX IF NOT EXISTS idx_project_tech_stack_path
    ON project_tech_stack(project_path);

CREATE INDEX IF NOT EXISTS idx_project_tech_stack_confirmed
    ON project_tech_stack(is_confirmed);
```

**外键**: 无（根表）

---

### 表 2: session_intents (会话意图)

**用途**: 存储会话意图分析结果（开场白分析）

**表结构**:
```sql
CREATE TABLE IF NOT EXISTS session_intents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_file_path TEXT NOT NULL UNIQUE,
    project_path TEXT,                  -- 外键 → project_tech_stack(project_path)
    opening_goal TEXT,                  -- 核心目标（一句话）
    intent_type TEXT,                   -- "new_feature" | "bug_fix" | "refactor" | "learning" | "other"
    project_type TEXT,                  -- "web_app" | "cli_tool" | "library" | "other"
    tech_stack TEXT,                    -- JSON 数组: 涉及技术栈
    constraints TEXT,                   -- JSON 数组: 约束条件
    language TEXT,                      -- "zh" | "en"
    confidence REAL,                    -- 0-1: 置信度
    analysis_status TEXT NOT NULL DEFAULT 'pending',  -- "pending" | "completed" | "failed"
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),

    FOREIGN KEY (project_path) REFERENCES project_tech_stack(project_path) ON DELETE SET NULL
);
```

**索引**:
```sql
CREATE INDEX IF NOT EXISTS idx_session_intents_file_path
    ON session_intents(session_file_path);

CREATE INDEX IF NOT EXISTS idx_session_intents_project
    ON session_intents(project_path);

CREATE INDEX IF NOT EXISTS idx_session_intents_status
    ON session_intents(analysis_status);
```

**外键**:
- `project_path` → `project_tech_stack(project_path)` ON DELETE SET NULL

---

### 表 3: qa_pairs (问答对)

**用途**: 存储问答对（助手回答 + 用户后续决策）

**⚠️ 重要**: 问答对配对新逻辑
```
原逻辑 (v1.0.2): (user1, assistant1), (user2, assistant2), ...
新逻辑 (v1.0.3): (assistant1, user2), (assistant2, user3), ...
```

**表结构**:
```sql
CREATE TABLE IF NOT EXISTS qa_pairs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_intent_id INTEGER NOT NULL,  -- 外键 → session_intents(id)
    session_file_path TEXT NOT NULL,
    qa_index INTEGER NOT NULL,           -- 问答对序号（0, 1, 2, ...）

    -- 新配对逻辑：assistant 回答 + 用户后续决策
    user_question_uuid TEXT NOT NULL,    -- 第 N 个 user 的 UUID
    assistant_answer_uuid TEXT,          -- 第 N 个 assistant 的 UUID
    user_question TEXT NOT NULL,         -- user 的内容
    assistant_answer TEXT,               -- assistant 的内容

    has_decision INTEGER NOT NULL DEFAULT 0,  -- 布尔值: 是否包含决策
    decision_count INTEGER NOT NULL DEFAULT 0,  -- 决策数量
    analysis_status TEXT NOT NULL DEFAULT 'pending',  -- "pending" | "analyzed" | "failed"
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),

    FOREIGN KEY (session_intent_id) REFERENCES session_intents(id) ON DELETE CASCADE,
    UNIQUE(session_intent_id, qa_index)
);
```

**索引**:
```sql
CREATE INDEX IF NOT EXISTS idx_qa_pairs_session_intent
    ON qa_pairs(session_intent_id);

CREATE INDEX IF NOT EXISTS idx_qa_pairs_file_path
    ON qa_pairs(session_file_path);

CREATE INDEX IF NOT EXISTS idx_qa_pairs_has_decision
    ON qa_pairs(has_decision);
```

**外键**:
- `session_intent_id` → `session_intents(id)` ON DELETE CASCADE

**约束**:
- `UNIQUE(session_intent_id, qa_index)`: 同一会话的问答对序号唯一

---

### 表 4: decision_points (决策点)

**用途**: 存储决策点分析结果

**表结构**:
```sql
CREATE TABLE IF NOT EXISTS decision_points (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    qa_pair_id INTEGER NOT NULL,        -- 外键 → qa_pairs(id)
    session_file_path TEXT NOT NULL,

    decision_type TEXT,                 -- "technology_choice" | "architecture_design" | "tool_selection" | "implementation" | "other"
    decision_made TEXT,                 -- 决策内容（一句话）

    rationale TEXT,                     -- JSON 数组: 明确理由（用户提及）
    inferred_reasons TEXT,              -- JSON 数组: 推测理由（LLM 分析）
    alternatives TEXT,                  -- JSON 数组: 备选方案

    decision_shift TEXT,                -- JSON 对象: 决策演变（v1.0.4 使用）

    confidence REAL NOT NULL DEFAULT 0.5,  -- 0-1: 置信度
    analysis_quality TEXT,              -- "high" | "medium" | "low"

    needs_interview INTEGER NOT NULL DEFAULT 0,  -- 布尔值: 是否需要深度采访
    interview_status TEXT NOT NULL DEFAULT 'pending',  -- "pending" | "completed" | "skipped"
    interview_result TEXT,              -- JSON 对象: 采访结果

    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),

    FOREIGN KEY (qa_pair_id) REFERENCES qa_pairs(id) ON DELETE CASCADE
);
```

**索引**:
```sql
CREATE INDEX IF NOT EXISTS idx_decision_points_qa_pair
    ON decision_points(qa_pair_id);

CREATE INDEX IF NOT EXISTS idx_decision_points_file_path
    ON decision_points(session_file_path);

CREATE INDEX IF NOT EXISTS idx_decision_points_type
    ON decision_points(decision_type);

CREATE INDEX IF NOT EXISTS idx_decision_points_needs_interview
    ON decision_points(needs_interview);
```

**外键**:
- `qa_pair_id` → `qa_pairs(id)` ON DELETE CASCADE

---

### 表 5: analysis_feedback (分析反馈)

**用途**: 存储用户对分析结果的反馈（Human-in-the-loop）

**表结构**:
```sql
CREATE TABLE IF NOT EXISTS analysis_feedback (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_file_path TEXT,

    -- 关联到任意分析结果
    decision_point_id INTEGER,          -- 外键 → decision_points(id)
    qa_pair_id INTEGER,                 -- 外键 → qa_pairs(id)
    session_intent_id INTEGER,          -- 外键 → session_intents(id)

    feedback_type TEXT NOT NULL,        -- "correction" | "confirmation" | "addition" | "deletion"
    target_field TEXT,                  -- 目标字段名: "decision_type" | "rationale" 等
    original_content TEXT,              -- 原始内容
    corrected_content TEXT,             -- 修正后的内容
    user_notes TEXT,                    -- 用户备注
    feedback_source TEXT,               -- "manual" | "interview" | "inference"

    is_applied INTEGER NOT NULL DEFAULT 0,  -- 布尔值: 是否已应用
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),

    FOREIGN KEY (decision_point_id) REFERENCES decision_points(id) ON DELETE CASCADE,
    FOREIGN KEY (qa_pair_id) REFERENCES qa_pairs(id) ON DELETE CASCADE,
    FOREIGN KEY (session_intent_id) REFERENCES session_intents(id) ON DELETE CASCADE
);
```

**索引**:
```sql
CREATE INDEX IF NOT EXISTS idx_analysis_feedback_decision
    ON analysis_feedback(decision_point_id);

CREATE INDEX IF NOT EXISTS idx_analysis_feedback_qa_pair
    ON analysis_feedback(qa_pair_id);

CREATE INDEX IF NOT EXISTS idx_analysis_feedback_intent
    ON analysis_feedback(session_intent_id);

CREATE INDEX IF NOT EXISTS idx_analysis_feedback_type
    ON analysis_feedback(feedback_type);

CREATE INDEX IF NOT EXISTS idx_analysis_feedback_applied
    ON analysis_feedback(is_applied);
```

**外键**:
- `decision_point_id` → `decision_points(id)` ON DELETE CASCADE
- `qa_pair_id` → `qa_pairs(id)` ON DELETE CASCADE
- `session_intent_id` → `session_intents(id)` ON DELETE CASCADE

---

### 表 6: prompt_combinations (提示词组合)

**用途**: 存储提示词组合模式（v1.0.5 使用）

**⚠️ 注意**: 此表为未来版本预留，v1.0.3 不使用

**表结构**:
```sql
CREATE TABLE IF NOT EXISTS prompt_combinations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    combination_name TEXT NOT NULL,
    combination_hash TEXT NOT NULL UNIQUE,  -- 组合内容的哈希值
    component_ids TEXT NOT NULL,         -- JSON 数组: 组件 ID 列表
    component_order TEXT,                -- JSON 数组: 组件顺序
    combination_type TEXT NOT NULL,      -- "sequential" | "parallel" | "conditional"
    purpose TEXT,                        -- 组合用途
    target_scenario TEXT,                -- 目标场景
    usage_count INTEGER NOT NULL DEFAULT 0,  -- 使用次数
    success_count INTEGER NOT NULL DEFAULT 0,  -- 成功次数
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    last_used_at TEXT
);
```

**索引**:
```sql
CREATE INDEX IF NOT EXISTS idx_prompt_combinations_hash
    ON prompt_combinations(combination_hash);

CREATE INDEX IF NOT EXISTS idx_prompt_combinations_scenario
    ON prompt_combinations(target_scenario);

CREATE INDEX IF NOT EXISTS idx_prompt_combinations_usage
    ON prompt_combinations(usage_count DESC);
```

**外键**: 无（独立表）

---

## 📝 实现检查清单

### 准备阶段
- [ ] 确认当前数据库版本是 18
- [ ] 确认外键约束已启用
- [ ] 备份现有数据库（开发环境可选）

### 编码阶段
- [ ] 更新 `CURRENT_DB_VERSION` 常量: 18 → 19
- [ ] 在 `run_migrations()` match 语句添加 `19 => migrate_v19(conn)?`
- [ ] 实现 `migrate_v19_impl()` 函数
  - [ ] 创建 6 张表（按顺序：project_tech_stack → session_intents → qa_pairs → decision_points → analysis_feedback → prompt_combinations）
  - [ ] 每张表创建后立即创建索引
  - [ ] 添加外键约束
  - [ ] 添加日志记录

### 测试阶段
- [ ] 编写单元测试：验证表创建成功
- [ ] 编写单元测试：验证外键约束生效
- [ ] 编写单元测试：验证 UNIQUE 约束
- [ ] 编写单元测试：验证 CASCADE 删除
- [ ] 运行 `cargo test` 确保所有测试通过
- [ ] 删除旧数据库，重启应用验证自动迁移

### 质量检查
- [ ] 运行 `cargo clippy` 无警告
- [ ] 运行 `cargo fmt` 格式化代码
- [ ] 检查所有日志信息清晰准确
- [ ] 验证 SQL 语法正确

---

## ⚠️ 关键约束和注意事项

### 1. 外键约束顺序
**依赖关系**:
```
project_tech_stack (根表)
    ↓
session_intents (依赖 project_tech_stack)
    ↓
qa_pairs (依赖 session_intents)
    ↓
decision_points (依赖 qa_pairs)
    ↓
analysis_feedback (依赖 decision_points, qa_pairs, session_intents)

prompt_combinations (独立表，无依赖)
```

**创建顺序**: 必须按依赖顺序创建表，否则外键约束会失败

---

### 2. 级联删除规则
- `session_intents.project_path`: ON DELETE SET NULL（删除项目时保留会话意图）
- `qa_pairs.session_intent_id`: ON DELETE CASCADE（删除会话时删除问答对）
- `decision_points.qa_pair_id`: ON DELETE CASCADE（删除问答对时删除决策点）
- `analysis_feedback`: 全部 ON DELETE CASCADE（删除关联对象时删除反馈）

---

### 3. JSON 字段格式
所有 JSON 字段存储为 TEXT，插入时使用 `serde_json::to_string()`:

```rust
// 示例：插入 tech_stack (JSON 数组)
let tech_stack = vec!["Rust".to_string(), "Tauri".to_string()];
let tech_stack_json = serde_json::to_string(&tech_stack).unwrap();

conn.execute(
    "INSERT INTO project_tech_stack (project_path, tech_stack) VALUES (?1, ?2)",
    params![project_path, tech_stack_json],
)?;
```

读取时使用 `serde_json::from_str()`:

```rust
let tech_stack_json: String = row.get(1)?;
let tech_stack: Vec<String> = serde_json::from_str(&tech_stack_json).unwrap_or_default();
```

---

### 4. 布尔值存储
SQLite 不支持原生布尔类型，使用 INTEGER 存储：
- `0` = false
- `1` = true

插入时转换:
```rust
conn.execute(
    "INSERT INTO qa_pairs (has_decision) VALUES (?1)",
    params![has_decision as i32],  // bool → i32
)?;
```

读取时转换:
```rust
let has_decision: i64 = row.get(1)?;
let has_decision_bool = has_decision == 1;
```

---

### 5. 时间戳格式
使用 `datetime('now', 'localtime')` 生成本地时间戳，格式为 RFC3339 字符串。

手动插入时使用 `chrono`:
```rust
use chrono::Utc;

let now = Utc::now().to_rfc3339();
conn.execute(
    "INSERT INTO project_tech_stack (project_path, created_at) VALUES (?1, ?2)",
    params![project_path, now],
)?;
```

---

### 6. UNIQUE 约束冲突处理
使用 `INSERT OR REPLACE` 或 `ON CONFLICT` 处理唯一键冲突:

```rust
// 方法 1: INSERT OR REPLACE
conn.execute(
    "INSERT OR REPLACE INTO project_tech_stack (project_path, tech_stack) VALUES (?1, ?2)",
    params![project_path, tech_stack_json],
)?;

// 方法 2: ON CONFLICT（推荐）
conn.execute(
    "INSERT INTO project_tech_stack (project_path, tech_stack) VALUES (?1, ?2)
     ON CONFLICT(project_path) DO UPDATE SET
     tech_stack = ?2, updated_at = datetime('now', 'localtime')",
    params![project_path, tech_stack_json],
)?;
```

---

## 🧪 单元测试模板

### 测试 1: 验证所有表创建成功
```rust
#[test]
fn test_migrate_v19_tables_created() {
    let mut conn = Connection::open_in_memory().unwrap();
    conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();

    migrate_v19_impl(&mut conn).unwrap();

    let tables: Vec<String> = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
    ).unwrap()
    .query_map([], |row| row.get(0))
    .unwrap()
    .collect::<Result<Vec<_>, _>>()
    .unwrap();

    assert!(tables.contains(&"project_tech_stack".to_string()));
    assert!(tables.contains(&"session_intents".to_string()));
    assert!(tables.contains(&"qa_pairs".to_string()));
    assert!(tables.contains(&"decision_points".to_string()));
    assert!(tables.contains(&"analysis_feedback".to_string()));
    assert!(tables.contains(&"prompt_combinations".to_string()));
}
```

### 测试 2: 验证外键约束
```rust
#[test]
fn test_migrate_v19_foreign_keys() {
    let mut conn = Connection::open_in_memory().unwrap();
    migrate_v19_impl(&mut conn).unwrap();

    // 验证外键已启用
    let fk_enabled: i64 = conn.query_row("PRAGMA foreign_keys", [], |row| row.get(0)).unwrap();
    assert_eq!(fk_enabled, 1);

    // 验证外键约束生效（尝试插入无效引用）
    let result = conn.execute(
        "INSERT INTO qa_pairs (session_intent_id, session_file_path, qa_index, user_question)
         VALUES (999, '/test', 0, 'test')",
        []
    );
    assert!(result.is_err(), "应该因外键约束失败");
}
```

### 测试 3: 验证 UNIQUE 约束
```rust
#[test]
fn test_migrate_v19_unique_constraints() {
    let mut conn = Connection::open_in_memory().unwrap();
    migrate_v19_impl(&mut conn).unwrap();

    // 插入第一条记录
    conn.execute(
        "INSERT INTO project_tech_stack (project_path, tech_stack) VALUES ('/test/path', '[\"Rust\"]')",
        []
    ).unwrap();

    // 尝试插入重复路径
    let result = conn.execute(
        "INSERT INTO project_tech_stack (project_path, tech_stack) VALUES ('/test/path', '[\"Rust\"]')",
        []
    );
    assert!(result.is_err(), "应该因 UNIQUE 约束失败");
}
```

### 测试 4: 验证 CASCADE 删除
```rust
#[test]
fn test_migrate_v19_cascade_delete() {
    let mut conn = Connection::open_in_memory().unwrap();
    migrate_v19_impl(&mut conn).unwrap();

    // 创建测试数据
    conn.execute(
        "INSERT INTO session_intents (session_file_path, opening_goal) VALUES ('/test.jsonl', 'test')",
        []
    ).unwrap();

    conn.execute(
        "INSERT INTO qa_pairs (session_intent_id, session_file_path, qa_index, user_question)
         VALUES (1, '/test.jsonl', 0, 'test')",
        []
    ).unwrap();

    // 删除 session_intents 记录
    conn.execute("DELETE FROM session_intents WHERE id = 1", []).unwrap();

    // 验证 qa_pairs 被级联删除
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM qa_pairs", [], |row| row.get(0)).unwrap();
    assert_eq!(count, 0, "qa_pairs 应该被级联删除");
}
```

---

## 🚀 执行步骤

### 步骤 1: 更新版本号
**文件**: `src-tauri/src/database/migrations.rs`
**位置**: 第 31 行

```rust
const CURRENT_DB_VERSION: i32 = 19;  // 从 18 改为 19
```

---

### 步骤 2: 注册迁移
**文件**: `src-tauri/src/database/migrations.rs`
**位置**: 第 70-91 行（`run_migrations` 函数的 match 语句）

```rust
match version {
    // ... 其他版本
    18 => migrate_v18(conn)?,
    19 => migrate_v19(conn)?,  // 新增这一行
    _ => anyhow::bail!("未知的数据库版本: {}", version),
}
```

---

### 步骤 3: 实现迁移函数
**文件**: `src-tauri/src/database/migrations.rs`
**位置**: 在 `migrate_v18_impl()` 函数后追加

```rust
/// 迁移到版本 19: 创建意图分析表
///
/// # 功能
/// - 创建 6 张表支持会话意图分析功能
/// - project_tech_stack: 项目技术栈配置
/// - session_intents: 会话意图分析结果
/// - qa_pairs: 问答对（助手回答+用户后续决策）
/// - decision_points: 决策点分析结果
/// - analysis_feedback: 用户反馈
/// - prompt_combinations: 提示词组合（v1.0.5 使用）
#[cfg(test)]
pub fn migrate_v19(conn: &mut Connection) -> Result<()> {
    migrate_v19_impl(conn)
}

#[cfg(not(test))]
fn migrate_v19(conn: &mut Connection) -> Result<()> {
    migrate_v19_impl(conn)
}

fn migrate_v19_impl(conn: &mut Connection) -> Result<()> {
    // 1. 创建 project_tech_stack 表
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
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_project_tech_stack_path
         ON project_tech_stack(project_path);",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_project_tech_stack_confirmed
         ON project_tech_stack(is_confirmed);",
        [],
    )?;

    log::info!("✅ 已创建 project_tech_stack 表");

    // 2. 创建 session_intents 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS session_intents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_file_path TEXT NOT NULL UNIQUE,
            project_path TEXT,
            opening_goal TEXT,
            intent_type TEXT,
            project_type TEXT,
            tech_stack TEXT,
            constraints TEXT,
            language TEXT,
            confidence REAL,
            analysis_status TEXT NOT NULL DEFAULT 'pending',
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),

            FOREIGN KEY (project_path) REFERENCES project_tech_stack(project_path) ON DELETE SET NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_session_intents_file_path
         ON session_intents(session_file_path);",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_session_intents_project
         ON session_intents(project_path);",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_session_intents_status
         ON session_intents(analysis_status);",
        [],
    )?;

    log::info!("✅ 已创建 session_intents 表");

    // 3. 创建 qa_pairs 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS qa_pairs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_intent_id INTEGER NOT NULL,
            session_file_path TEXT NOT NULL,
            qa_index INTEGER NOT NULL,
            user_question_uuid TEXT NOT NULL,
            assistant_answer_uuid TEXT,
            user_question TEXT NOT NULL,
            assistant_answer TEXT,
            has_decision INTEGER NOT NULL DEFAULT 0,
            decision_count INTEGER NOT NULL DEFAULT 0,
            analysis_status TEXT NOT NULL DEFAULT 'pending',
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),

            FOREIGN KEY (session_intent_id) REFERENCES session_intents(id) ON DELETE CASCADE,
            UNIQUE(session_intent_id, qa_index)
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_qa_pairs_session_intent
         ON qa_pairs(session_intent_id);",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_qa_pairs_file_path
         ON qa_pairs(session_file_path);",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_qa_pairs_has_decision
         ON qa_pairs(has_decision);",
        [],
    )?;

    log::info!("✅ 已创建 qa_pairs 表");

    // 4. 创建 decision_points 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS decision_points (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            qa_pair_id INTEGER NOT NULL,
            session_file_path TEXT NOT NULL,
            decision_type TEXT,
            decision_made TEXT,
            rationale TEXT,
            inferred_reasons TEXT,
            alternatives TEXT,
            decision_shift TEXT,
            confidence REAL NOT NULL DEFAULT 0.5,
            analysis_quality TEXT,
            needs_interview INTEGER NOT NULL DEFAULT 0,
            interview_status TEXT NOT NULL DEFAULT 'pending',
            interview_result TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),

            FOREIGN KEY (qa_pair_id) REFERENCES qa_pairs(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_decision_points_qa_pair
         ON decision_points(qa_pair_id);",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_decision_points_file_path
         ON decision_points(session_file_path);",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_decision_points_type
         ON decision_points(decision_type);",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_decision_points_needs_interview
         ON decision_points(needs_interview);",
        [],
    )?;

    log::info!("✅ 已创建 decision_points 表");

    // 5. 创建 analysis_feedback 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS analysis_feedback (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_file_path TEXT,
            decision_point_id INTEGER,
            qa_pair_id INTEGER,
            session_intent_id INTEGER,
            feedback_type TEXT NOT NULL,
            target_field TEXT,
            original_content TEXT,
            corrected_content TEXT,
            user_notes TEXT,
            feedback_source TEXT,
            is_applied INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),

            FOREIGN KEY (decision_point_id) REFERENCES decision_points(id) ON DELETE CASCADE,
            FOREIGN KEY (qa_pair_id) REFERENCES qa_pairs(id) ON DELETE CASCADE,
            FOREIGN KEY (session_intent_id) REFERENCES session_intents(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_analysis_feedback_decision
         ON analysis_feedback(decision_point_id);",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_analysis_feedback_qa_pair
         ON analysis_feedback(qa_pair_id);",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_analysis_feedback_intent
         ON analysis_feedback(session_intent_id);",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_analysis_feedback_type
         ON analysis_feedback(feedback_type);",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_analysis_feedback_applied
         ON analysis_feedback(is_applied);",
        [],
    )?;

    log::info!("✅ 已创建 analysis_feedback 表");

    // 6. 创建 prompt_combinations 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS prompt_combinations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            combination_name TEXT NOT NULL,
            combination_hash TEXT NOT NULL UNIQUE,
            component_ids TEXT NOT NULL,
            component_order TEXT,
            combination_type TEXT NOT NULL,
            purpose TEXT,
            target_scenario TEXT,
            usage_count INTEGER NOT NULL DEFAULT 0,
            success_count INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            last_used_at TEXT
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_prompt_combinations_hash
         ON prompt_combinations(combination_hash);",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_prompt_combinations_scenario
         ON prompt_combinations(target_scenario);",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_prompt_combinations_usage
         ON prompt_combinations(usage_count DESC);",
        [],
    )?;

    log::info!("✅ 已创建 prompt_combinations 表");

    log::info!("✅ 已完成 v19 迁移：创建 6 张意图分析表");

    Ok(())
}
```

---

### 步骤 4: 添加单元测试
**文件**: `src-tauri/src/database/migrations.rs`
**位置**: 在 `mod tests` 块中追加（第 1205 行之后）

```rust
#[test]
fn test_migrate_v19_tables_created() {
    let mut conn = Connection::open_in_memory().unwrap();
    conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();

    migrate_v19_impl(&mut conn).unwrap();

    let tables: Vec<String> = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
    ).unwrap()
    .query_map([], |row| row.get(0))
    .unwrap()
    .collect::<Result<Vec<_>, _>>()
    .unwrap();

    assert!(tables.contains(&"analysis_feedback".to_string()));
    assert!(tables.contains(&"decision_points".to_string()));
    assert!(tables.contains(&"prompt_combinations".to_string()));
    assert!(tables.contains(&"project_tech_stack".to_string()));
    assert!(tables.contains(&"qa_pairs".to_string()));
    assert!(tables.contains(&"session_intents".to_string()));
}

#[test]
fn test_migrate_v19_foreign_keys() {
    let mut conn = Connection::open_in_memory().unwrap();
    migrate_v19_impl(&mut conn).unwrap();

    let fk_enabled: i64 = conn.query_row("PRAGMA foreign_keys", [], |row| row.get(0)).unwrap();
    assert_eq!(fk_enabled, 1);
}

#[test]
fn test_migrate_v19_unique_constraints() {
    let mut conn = Connection::open_in_memory().unwrap();
    migrate_v19_impl(&mut conn).unwrap();

    conn.execute(
        "INSERT INTO project_tech_stack (project_path, tech_stack) VALUES ('/test/path', '[\"Rust\"]')",
        []
    ).unwrap();

    let result = conn.execute(
        "INSERT INTO project_tech_stack (project_path, tech_stack) VALUES ('/test/path', '[\"Rust\"]')",
        []
    );
    assert!(result.is_err());
}

#[test]
fn test_migrate_v19_cascade_delete() {
    let mut conn = Connection::open_in_memory().unwrap();
    migrate_v19_impl(&mut conn).unwrap();

    // 创建测试数据
    conn.execute(
        "INSERT INTO session_intents (session_file_path, opening_goal) VALUES ('/test.jsonl', 'test')",
        []
    ).unwrap();

    conn.execute(
        "INSERT INTO qa_pairs (session_intent_id, session_file_path, qa_index, user_question)
         VALUES (1, '/test.jsonl', 0, 'test')",
        []
    ).unwrap();

    // 删除 session_intents 记录
    conn.execute("DELETE FROM session_intents WHERE id = 1", []).unwrap();

    // 验证 qa_pairs 被级联删除
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM qa_pairs", [], |row| row.get(0)).unwrap();
    assert_eq!(count, 0);
}
```

---

### 步骤 5: 验证迁移
```bash
# 1. 运行单元测试
cd src-tauri
cargo test migrate_v19

# 2. 检查代码
cargo clippy

# 3. 格式化代码
cargo fmt

# 4. 完整测试
cargo test

# 5. 删除旧数据库（Windows）
del %USERPROFILE%\.prism-forge\prism_forge.db

# 5. 删除旧数据库（macOS/Linux）
rm ~/.prism-forge/prism_forge.db

# 6. 启动应用验证自动迁移
npm run tauri dev
```

---

## ❓ 待确认问题

### 无待确认问题
所有实现细节已在任务计划中明确定义，可以直接开始编码。

---

## 📚 参考资料

### 相关文档
- `docs/tasks/v1.0.3-intent-analysis/REQUIREMENTS.md` - 需求文档
- `docs/tasks/v1.0.3-intent-analysis/DESIGN.md` - 设计文档
- `docs/tasks/v1.0.3-intent-analysis/EXECUTION_PLAN.md` - 执行计划
- `docs/tasks/v1.0.3-intent-analysis/DATABASE_MIGRATION_V19.md` - 数据库迁移详细设计

### 相关代码
- `src-tauri/src/database/migrations.rs` - 数据库迁移实现
- `src-tauri/src/database/models.rs` - 数据模型定义

### 技术栈文档
- SQLite 外键约束: https://www.sqlite.org/foreignkeys.html
- rusqlite 文档: https://docs.rs/rusqlite/

---

## 📊 进度追踪

**当前状态**: 待开始

**已完成**:
- [x] 分析现有代码结构
- [x] 确认当前数据库版本 (v18)
- [x] 确认外键约束已启用
- [x] 分析代码风格模式

**进行中**:
- [ ] 实现迁移函数

**待完成**:
- [ ] 添加单元测试
- [ ] 运行测试验证
- [ ] 更新 progress.json

---

**文档版本**: v1.0
**创建时间**: 2026-02-02
**最后更新**: 2026-02-02
