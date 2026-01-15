# 模板需求分析报告

**项目：** Claude Code Session Monitor
**分析日期：** 2025-12-31
**版本：** v1.0
**基于：** main_plan.json v1.4 及各 Wave 任务文件

---

## 📊 执行摘要

### 现有模板覆盖度

| 模板 | 覆盖率 | 评估 |
|------|--------|------|
| tauri-command.rs.template | 80% | ✅ 良好 - 基础命令框架完善 |
| react-component.tsx.template | 70% | ✅ 可用 - 缺少复杂组件示例 |
| zustand-store.ts.template | 90% | ✅ 优秀 - 状态管理完整 |
| database-repository.rs.template | 85% | ✅ 良好 - CRUD 操作完善 |

**总体覆盖率：** 基础功能 81%，专用功能 0%

### 缺失模板统计

- **P0 必需模板：** 4 个（阻塞核心开发）
- **P1 重要模板：** 5 个（高频使用场景）
- **P2 辅助模板：** 4 个（功能增强）
- **P3 可选模板：** 5 个（锦上添花）

**总计：** 18 个专用模板待补充

---

## 🔴 P0 必需模板（阻塞核心任务）

### 1. 会话扫描器模板

**文件名：** `scanner.rs.template`
**对应任务：** T1_2 (Wave 2)
**优先级：** 🔴 CRITICAL
**阻塞任务：** T2B_1, T4_1, 所有前端展示任务

#### 功能需求
- 扫描 `~/.claude/projects/` 目录查找所有 JSONL 文件
- 提取会话元数据（session_id, project_path, created_at, updated_at）
- 判断会话活跃状态（Windows 文件锁定检测 / 时间判断降级）
- 支持可配置的活跃阈值（默认 24 小时）
- 跨平台支持（Windows/macOS/Linux）

#### 技术栈
```toml
[dependencies]
dirs = "5.0"           # 用户目录路径
glob = "0.3"           # 文件模式匹配
walkdir = "2.5"        # 目录遍历（可选）
```

#### 数据结构
```rust
/// 会话元数据
pub struct SessionMetadata {
    pub session_id: String,        // UUID from filename
    pub project_path: String,      // Relative to projects/
    pub file_path: PathBuf,        // Full path
    pub is_active: bool,           // Active status
    pub created_at: SystemTime,    // File creation time
    pub updated_at: SystemTime,    // File modification time
    pub file_size: u64,            // File size in bytes
}
```

#### AI 提示词（用于生成代码）

```
请为 Claude Code Session Monitor 项目创建一个会话扫描器模块，要求：

## 功能需求
1. 扫描 ~/.claude/projects/ 目录下所有 .jsonl 文件
2. 从文件名提取 UUID 作为 session_id
3. 从文件路径提取项目相对路径
4. 判断会话是否活跃：
   - Windows: 使用文件锁定检测（CreateFileW share_mode=0）
   - macOS/Linux: 降级到时间判断（默认 24 小时内）
5. 活跃阈值可从 settings 表读取（active_threshold 字段）

## 平台特定实现
- Windows: 使用 std::os::windows::fs::OpenOptionsExt 的 share_mode(0) 检测文件锁定
- macOS: 直接使用时间判断
- Linux: 直接使用时间判断

## 使用条件编译
使用 #[cfg(target_os = "windows")] 等属性实现平台差异

## 代码风格
- 遵循 Rust 命名规范（snake_case）
- 函数返回 Result<T, CommandError>
- 使用 ? 操作符传播错误
- 添加中文注释说明关键逻辑
- 错误消息使用中文

## 输出文件
src-tauri/src/monitor/scanner.rs
src-tauri/src/monitor/mod.rs

## Tauri 命令
暴露 scan_sessions() 命令供前端调用

请生成完整的实现代码，包括：
1. SessionMetadata 结构体定义
2. get_claude_projects_dir() 函数
3. scan_session_files() 函数
4. extract_session_metadata() 函数
5. is_session_active() 平台特定函数
6. scan_sessions Tauri 命令
7. 完整的错误处理和中文注释
```

#### 验收标准
- [ ] 能扫描并返回所有会话文件
- [ ] 正确提取 session_id（UUID 格式）
- [ ] Windows 平台正确检测文件锁定
- [ ] 活跃阈值从 settings 表读取
- [ ] is_active 字段正确标识
- [ ] 性能：<2s 扫描 100 个会话

---

### 2. JSONL 流式解析器模板

**文件名：** `jsonl-parser.rs.template`
**对应任务：** T2_1 (Wave 3)
**优先级：** 🔴 CRITICAL
**阻塞任务：** T2_2, T2_3, 所有后续解析任务

#### 功能需求
- 增量读取 JSONL 文件（避免一次性加载大文件）
- 处理未完成的行缓冲区（FileShare 模式支持）
- 记录每条消息的 offset 和 length（用于按需读取）
- 支持流式处理（异步读取）
- 错误恢复机制（跳过损坏的行）

#### 技术栈
```toml
[dependencies]
serde_json = "1.0"       # JSON 解析
tokio = { version = "1", features = ["fs", "io-util"] }  # 异步 IO
```

#### 数据结构
```rust
/// 消息位置信息
pub struct MessageOffset {
    pub offset: u64,         // 字节偏移量
    pub length: u64,         // 消息长度
    pub uuid: String,        // 消息 UUID
    pub role: String,        // 消息角色
}

/// 解析统计
pub struct ParseStats {
    pub total_messages: usize,
    pub parsed_messages: usize,
    pub failed_lines: usize,
    pub buffer_overflow: usize,
}
```

#### AI 提示词
```
请为 Claude Code Session Monitor 创建 JSONL 流式解析器，要求：

## 功能需求
1. 增量读取 JSONL 文件（使用 BufReader 按行读取）
2. 处理文件末尾未完成的行（缓冲到下次读取）
3. 记录每条消息的 offset 和 length
4. 支持 Windows FileShare 模式（使用 FILE_SHARE_READ | FILE_SHARE_WRITE）
5. 异步流式处理（使用 tokio::fs::File）
6. 错误恢复：跳过无效 JSON 行，记录统计

## 数据流
文件 → BufReader → 按行读取 → JSON 解析 → 提取元数据 → 存储 offset

## 关键逻辑
- 使用 BufReader::new().lines() 逐行读取
- 每行记录当前位置 offset
- 解析失败时跳过该行，记录到 failed_lines
- 文件末尾不完整行存入缓冲区

## FileShare 模式（Windows）
使用 std::os::windows::fs::OpenOptionsExt：
```rust
let file = OpenOptions::new()
    .read(true)
    .share_mode(0x03)  // FILE_SHARE_READ | FILE_SHARE_WRITE
    .open(path)?;
```

## 输出
src-tauri/src/parser/jsonl.rs
src-tauri/src/parser/mod.rs

## Tauri 命令
parse_session_file(file_path: String) -> Result<Vec<MessageOffset>>

请生成完整实现，包括：
1. MessageOffset 结构体
2. JsonlParser 结构体
3. parse_file() 异步方法
4. parse_line() 辅助方法
5. 错误处理和统计
6. 中文注释和文档
```

#### 验收标准
- [ ] 正确解析标准的 JSONL 格式
- [ ] 处理大文件（>100MB）不崩溃
- [ ] Windows FileShare 模式正常工作
- [ ] offset 和 length 准确记录
- [ ] 损坏行不中断解析流程

---

### 3. 消息树构建器模板

**文件名：** `message-tree-builder.rs.template`
**对应任务：** T2_2 (Wave 3)
**优先级：** 🔴 CRITICAL
**阻塞任务：** T2_3, T4_2, T5_1c

#### 功能需求
- 基于 parentUuid 重构消息树
- 支持深层嵌套（100+ 层）
- 使用迭代算法防止栈溢出
- 根节点为 User 消息
- 支持消息元数据（tool_calls, errors, code_changes）

#### 数据结构
```rust
/// 消息节点
pub struct MessageNode {
    pub uuid: String,
    pub role: String,
    pub content: String,
    pub parent_uuid: Option<String>,
    pub children: Vec<MessageNode>,
    pub metadata: MessageMetadata,
    pub offset: u64,
    pub length: u64,
}

/// 消息元数据
pub struct MessageMetadata {
    pub tool_calls: Vec<ToolCall>,
    pub errors: Vec<String>,
    pub code_changes: Vec<CodeChange>,
    pub summary: Option<String>,
}
```

#### AI 提示词
```
请为 Claude Code Session Monitor 创建消息树构建器，要求：

## 功能需求
1. 从扁平的消息列表构建树形结构
2. 基于 parentUuid 字段建立父子关系
3. 使用迭代算法（避免递归栈溢出）
4. 根节点定位为第一个 User 消息
5. 支持 100+ 层深层嵌套
6. 提取元数据：tool_calls, errors, code_changes

## 算法要求
- 第一遍：建立 uuid -> message 的 HashMap
- 第二遍：迭代构建树（从根节点开始）
- 使用 std::collections::HashMap 快速查找
- 使用 while 循环而非递归

## 边界情况处理
- 孤儿节点（parent_uuid 不存在）→ 挂到虚拟根节点
- 循环引用 → 检测并中断
- 根节点识别 → parent_uuid 为 None 的第一个 User 消息

## 元数据提取
工具调用：提取 tool_use 类型字段
错误消息：提取 error 字段
代码变更：提取 Read/Write 操作的 oldText/newText

## 输出
src-tauri/src/parser/tree.rs

## Tauri 命令
build_message_tree(session_id: String) -> Result<MessageNode>

请生成完整实现，包括：
1. MessageNode 和相关结构体
2. MessageTreeBuilder 实现
3. build_tree() 方法
4. extract_metadata() 辅助方法
5. 边界情况处理
6. 中文注释
```

#### 验收标准
- [ ] 输出正确的嵌套 JSON 结构
- [ ] 根节点为 User 消息
- [ ] 深层嵌套（100+ 层）无栈溢出
- [ ] 元数据正确提取
- [ ] 孤儿节点正确处理

---

### 4. 向量检索器模板

**文件名：** `vector-retriever.rs.template`
**对应任务：** T3_1a (Wave 5)
**优先级：** 🔴 CRITICAL
**阻塞任务：** T3_1b, T3_3, T5_2

#### 功能需求
- 使用 sqlite-vec 的 distance() 函数计算余弦相似度
- 返回 Top-K 相似会话
- 支持评分加权检索（Weighted RAG）
- 性能要求：1000+ 会话检索 <100ms
- 支持批量查询

#### SQL 查询
```sql
SELECT
    s.session_id,
    s.project_path,
    m.summary,
    s.rating,
    distance(me.embedding, ?) AS distance,
    (1.0 - distance(me.embedding, ?)) * 0.7 + (s.rating / 5.0) * 0.3 AS weighted_score
FROM message_embeddings me
JOIN messages m ON m.message_uuid = me.message_uuid
JOIN sessions s ON s.session_id = m.session_id
WHERE s.is_archived = 0
ORDER BY weighted_score DESC
LIMIT ?;
```

#### AI 提示词
```
请为 Claude Code Session Monitor 创建向量检索器，要求：

## 功能需求
1. 使用 sqlite-vec 的 distance() 函数计算向量距离
2. 余弦相似度：cosine_sim = 1.0 - distance
3. 评分加权公式：Score = 0.7 * cosine_sim + 0.3 * (rating / 5.0)
4. 返回 Top-K 结果（默认 K=5）
5. 性能：1000+ 会话检索 <100ms

## SQL 优化
- 使用预编译语句（PREPARE/EXECUTE）
- 为 message_embeddings 表创建向量索引
- 过滤归档会话（is_archived = 0）
- 使用 JOIN 关联 sessions 和 messages 表

## 数据结构
```rust
pub struct SessionMatch {
    pub session_id: String,
    pub project_path: String,
    pub summary: String,
    pub rating: i32,
    pub similarity: f32,
    pub weighted_score: f32,
}
```

## 查询优化技巧
1. 使用 ? 参数绑定（避免 SQL 注入）
2. 限制结果数量（LIMIT 子句）
3. 使用索引覆盖查询
4. 考虑使用 MATERIALIZED VIEW 优化

## 输出
src-tauri/src/optimizer/retriever.rs

## Tauri 命令
vector_search(query: String, limit: usize) -> Result<Vec<SessionMatch>>
vector_search_weighted(query: String, limit: usize) -> Result<Vec<SessionMatch>>

请生成完整实现，包括：
1. VectorRetriever 结构体
2. search_similar() 基础方法
3. search_weighted() 加权方法
4. SQL 查询优化
5. 性能测试代码
6. 中文注释
```

#### 验收标准
- [ ] 返回最相似的 5 条历史会话
- [ ] 按相似度排序
- [ ] 1000+ 会话检索 <100ms
- [ ] 评分加权正确计算
- [ ] 归档会话不出现在结果中

---

## 🟡 P1 重要模板（高频使用）

### 5. 消息树可视化组件模板

**文件名：** `MessageTree.tsx.template`
**对应任务：** T4_2 (Wave 6)
**优先级：** 🟡 HIGH

#### 功能需求
- 树状折叠/展开 UI
- 懒加载消息内容（通过 offset 按需获取）
- 三级视图切换（L1 Full Trace / L2 Clean Flow / L3 Prompt Only）
- Monaco Editor 集成（代码高亮）
- 性能优化（虚拟滚动）

#### AI 提示词
```
请为 Claude Code Session Monitor 创建消息树可视化组件，要求：

## 技术栈
- React 19 + TypeScript
- shadcn/ui 组件（Collapsible, ScrollArea）
- @monaco-editor/react（代码高亮）
- react-virtual（虚拟滚动）

## 功能需求
1. 递归渲染消息树（节点可折叠/展开）
2. 懒加载：节点展开时通过 offset 调用 Tauri 获取完整内容
3. 三级视图切换：
   - L1: 显示所有内容
   - L2: 过滤冗余工具输出
   - L3: 仅显示 User 和 Assistant 消息
4. 代码块使用 Monaco Editor 渲染
5. 大树性能优化（react-virtual）

## Props 接口
```tsx
interface MessageTreeProps {
  sessionId: string;
  rootMessage: MessageNode;
  viewLevel: 'L1' | 'L2' | 'L3';
  onNodeClick?: (node: MessageNode) => void;
}
```

## Tauri 集成
```tsx
async function loadMessageContent(offset: number, length: number) {
  return invoke('get_message_content', { offset, length });
}
```

## 样式要求
- 使用 Tailwind CSS
- 深色模式支持
- 节点缩进显示层级
- 展开/折叠动画

## 输出
src/components/MessageTree.tsx
src/components/MessageTreeNode.tsx
src/components/CodeBlock.tsx

请生成完整实现，包括：
1. MessageTree 主组件
2. MessageTreeNode 递归组件
3. CodeBlock Monaco 集成
4. 视图级别过滤逻辑
5. 懒加载实现
6. 性能优化
7. TypeScript 类型定义
```

---

### 6. 提示词实验室组件模板

**文件名：** `PromptLab.tsx.template`
**对应任务：** T5_2 (Wave 7)
**优先级：** 🟡 HIGH

#### AI 提示词
```
请创建提示词实验室界面组件，要求：

## 布局
- 左侧：目标输入框（TextArea）
- 中间：会话选择器（多选 Checkbox）
- 右侧：生成按钮 + 结果预览
- 底部：Token 统计 + 保存按钮

## 功能
1. 实时 Token 计数（调用 count_prompt_tokens）
2. 调用 optimize_prompt 生成优化提示词
3. 显示节省的 Token 百分比
4. 保存到 saved_prompts 表
5. 复制到剪贴板

## 组件
- GoalInput: 目标输入
- SessionSelector: 会话多选
- OptimizationPreview: 结果预览
- TokenStats: Token 统计
- ActionButtons: 操作按钮组

## 输出
src/pages/PromptLab.tsx
src/components/prompt-lab/*.tsx

请生成完整实现，包括：
1. 主页面布局
2. 各子组件
3. Tauri invoke 调用
4. Zustand 状态管理
5. 表单验证
```

---

### 7. 向量嵌入生成器模板

**文件名：** `embedding-generator.rs.template`
**对应任务：** T2_4 (Wave 4)
**优先级：** 🟡 HIGH

#### AI 提示词
```
请创建向量嵌入生成器，要求：

## 功能
1. 集成 fastembed 3.3 crate
2. 使用 BGE-small-en-v1.5 模型（384 维）
3. 异步生成向量（避免阻塞）
4. 存储到 message_embeddings 表
5. 批量处理支持

## 技术栈
```toml
fastembed = "3.3"
```

## 实现
```rust
pub struct EmbeddingGenerator {
    model: EmbeddingModel,
}

impl EmbeddingGenerator {
    pub async fn generate(&self, text: &str) -> Result<Vec<f32>>;
    pub async fn generate_batch(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>>;
}
```

## 输出
src-tauri/src/embeddings/generator.rs
src-tauri/src/embeddings/mod.rs

## Tauri 命令
generate_embedding(text: String) -> Result<Vec<f32>>

请生成完整实现，包括：
1. EmbeddingGenerator 结构体
2. 模型加载和初始化
3. 单条和批量生成方法
4. 数据库存储逻辑
5. 错误处理
```

---

### 8. 关键信息提取器模板

**文件名：** `extractor.rs.template`
**对应任务：** T2_3 (Wave 3)
**优先级：** 🟡 HIGH

#### AI 提示词
```
请创建关键信息提取器，要求：

## 功能
1. 提取工具调用序列（tool_use）
2. 识别错误消息
3. 检测代码变更（Read/Write 操作）
4. 生成 summary 字段

## 数据结构
```rust
pub struct ToolCall {
    pub name: String,
    pub input: serde_json::Value,
    pub output: Option<String>,
}

pub struct CodeChange {
    pub file_path: String,
    pub operation: String,  // "Read" | "Write" | "Edit"
    pub old_text: Option<String>,
    pub new_text: Option<String>,
}

pub struct MessageMetadata {
    pub tool_calls: Vec<ToolCall>,
    pub errors: Vec<String>,
    pub code_changes: Vec<CodeChange>,
    pub summary: Option<String>,
}
```

## 算法
- 工具调用：解析 content 中的 tool_use 字段
- 错误消息：查找 error, Error, ERROR 关键字
- 代码变更：解析 Read/Write/Edit 工具的参数
- summary：提取前 200 字或工具调用描述

## 输出
src-tauri/src/parser/extractor.rs

请生成完整实现，包括：
1. Extractor 结构体
2. extract_tool_calls() 方法
3. extract_errors() 方法
4. extract_code_changes() 方法
5. generate_summary() 方法
```

---

### 9. 项目侧边栏组件模板

**文件名：** `ProjectSidebar.tsx.template`
**对应任务：** T4_1, T2B_1 (Wave 6)
**优先级：** 🟡 HIGH

#### AI 提示词
```
请创建项目侧边栏组件，要求：

## 功能
1. 显示项目文件夹结构（树形菜单）
2. 每个项目下显示会话列表
3. 支持折叠/展开
4. 点击项目智能切换到最近活跃会话
5. 搜索和过滤（按项目、标签、评分）

## 组件结构
- ProjectTree: 项目树
- SessionList: 会话列表
- SearchBar: 搜索框
- FilterPanel: 过滤面板

## 状态管理
使用 Zustand：
```ts
interface ProjectStore {
  projects: Project[];
  activeProject: string | null;
  sessions: Map<string, Session[]>;
  expandedProjects: Set<string>;
}
```

## 输出
src/components/ProjectSidebar.tsx
src/components/ProjectTree.tsx
src/components/SessionList.tsx

请生成完整实现，包括：
1. 主组件布局
2. 树形菜单
3. 搜索/过滤逻辑
4. Zustand store
5. Tauri 集成
```

---

## 🟢 P2 辅助模板（功能增强）

### 10. 代码 Diff 查看器模板

**文件名：** `CodeDiffViewer.tsx.template`
**对应任务：** T5_1c (Wave 7)

#### AI 提示词
```
请创建代码 Diff 查看器组件，要求：

## 功能
1. 使用 react-diff-viewer-continued
2. 支持并排/统一模式切换
3. 大文件性能优化（>1000 行）
4. 语法高亮（Monaco）
5. 支持行内差异

## 技术栈
```bash
npm install react-diff-viewer-continued
```

## Props
```tsx
interface CodeDiffViewerProps {
  oldText: string;
  newText: string;
  language: string;
  mode: 'split' | 'unified';
  showLineNumbers: boolean;
}
```

## 性能优化
- 虚拟滚动（react-window）
- 懒加载大文件
- Web Worker 计算 diff

请生成完整实现。
```

---

### 11. 数据导出器模板

**文件名：** `DataExporter.tsx.template`
**对应任务：** T5_4 (Wave 7)

#### AI 提示词
```
请创建数据导出组件，要求：

## 功能
1. 格式选择器（JSON/CSV/Markdown）
2. 批量导出
3. 导出进度显示
4. 文件下载

## 后端命令
export_session(session_id: String, format: String) -> Result<String>

请生成完整实现。
```

---

### 12. 上下文压缩器模板

**文件名：** `context-compressor.rs.template`
**对应任务：** T3_2 (Wave 5)

#### AI 提示词
```
请创建上下文压缩器，要求：

## 功能
1. 去除冗余工具调用
2. 过滤中间输出
3. 移除 Thinking 过程
4. 保留关键决策点
5. 压缩率 >50%

## 算法
- 检测重复工具调用
- 识别噪音输出（超长字符串）
- 提取关键步骤

请生成完整实现。
```

---

### 13. 文件监控器模板

**文件名：** `file-watcher.rs.template`
**对应任务：** T2_6 (Wave 4)

#### AI 提示词
```
请创建文件监控器，要求：

## 功能
1. 使用 notify crate 监控目录
2. Tauri Events 推送到前端
3. 事件去重（2 秒防抖）
4. 支持递归监控

## 技术栈
```toml
notify = "6.1"
tokio = "1"
```

## 事件类型
- Create: 新会话创建
- Modify: 会话更新
- Delete: 会话删除

请生成完整实现。
```

---

## ⚪ P3 可选模板（锦上添花）

### 14. 实时更新指示器模板

**文件名：** `RealtimeUpdater.tsx.template`
**对应任务：** T4_4 (Wave 6)

#### AI 提示词
```
请创建实时更新指示器，要求：

## 功能
1. 监听 Tauri Events
2. 显示加载动画
3. 2 秒后自动刷新列表
4. 防抖处理

## 实现
```tsx
useEffect(() => {
  const unlisten = listen('session-changed', () => {
    setShowLoading(true);
    setTimeout(() => {
      refetch();
      setShowLoading(false);
    }, 2000);
  });
  return unlisten;
}, []);
```

请生成完整实现。
```

---

### 15. 日志提取引擎模板

**文件名：** `extraction-engine.rs.template`
**对应任务：** T3_4 (Wave 5)

#### AI 提示词
```
请创建日志提取引擎，要求：

## 功能
1. L1 Full Trace：全部内容
2. L2 Clean Flow：过滤冗余工具输出
3. L3 Prompt Only：仅 QA 对话
4. 导出 Markdown/JSON

## 输出格式
- Markdown: 格式化文档
- JSON: 结构化数据

请生成完整实现。
```

---

### 16. 向量数据操作模板

**文件名：** `vector-repository.rs.template`
**对应任务：** T2_4, T3_1a

#### AI 提示词
```
请创建向量数据 Repository，要求：

## 功能
1. message_embeddings 表 CRUD
2. BLOB 数据存储（向量）
3. 向量距离查询
4. 索引优化

请生成完整实现。
```

---

### 17. 会话聚合查询模板

**文件名：** `session-aggregation.rs.template`
**对应任务：** T2B_1

#### AI 提示词
```
请创建会话聚合查询模块，要求：

## 功能
1. 按项目路径分组
2. 活跃会话统计
3. 智能排序
4. 缓存策略

请生成完整实现。
```

---

### 18. Tauri Events 配置模板

**文件名：** `tauri-events.rs.template`
**对应任务：** T2_6, T4_4

#### AI 提示词
```
请创建 Tauri Events 配置模块，要求：

## 功能
1. 事件定义和类型
2. 前后端事件订阅
3. 事件去重机制
4. 错误处理

## 事件类型
- session-changed
- scan-complete
- embedding-progress

请生成完整实现。
```

---

### 19. 性能测试模板

**文件名：** `performance-test.rs.template`
**对应任务：** T1_PERF, T3_PERF, T6_PERF_FRONTEND

#### AI 提示词
```
请创建性能测试模块，要求：

## 功能
1. 基准测试框架
2. 性能指标收集
3. 报告生成
4. 阈值验证

## 测试场景
- 会话扫描性能（<2s for 100 sessions）
- 向量检索性能（<100ms for 1000+ sessions）
- 前端渲染性能（<500ms for 1000+ items）

请生成完整实现。
```

---

## 📋 模板创建检查清单

### 使用本报告创建模板的步骤

#### 1. 选择模板
- 根据优先级选择需要创建的模板
- 推荐从 P0 必需模板开始

#### 2. 复制 AI 提示词
- 找到对应模板的 "AI 提示词" 部分
- 完整复制提示词内容

#### 3. 请求大模型生成代码
```
用户: [粘贴 AI 提示词]
请根据以上要求生成完整的代码实现。
```

#### 4. 验证生成的代码
- 对照功能需求检查
- 运行测试用例
- 调整代码风格

#### 5. 保存为模板文件
- 将生成的代码保存到 `.template` 文件
- 路径：`C:/Users/thoma/.claude/skills/tech-stack-code-generator/assets/templates/`

#### 6. 更新技能
- 重新打包技能（可选）
- 测试新模板是否工作

---

## 🎯 推荐创建顺序

### 第一批（立即创建）
1. ✅ scanner.rs.template
2. ✅ jsonl-parser.rs.template
3. ✅ message-tree-builder.rs.template
4. ✅ vector-retriever.rs.template

### 第二批（高优先级）
5. ✅ MessageTree.tsx.template
6. ✅ PromptLab.tsx.template
7. ✅ embedding-generator.rs.template
8. ✅ extractor.rs.template
9. ✅ ProjectSidebar.tsx.template

### 第三批（功能增强）
10-18. 其余 P2 和 P3 模板

---

## 📚 参考资料

### 项目文档
- `CLAUDE.md` - 项目整体架构
- `dev_plans/plan1/main_plan.json` - 主计划
- `dev_plans/plan1/waves/` - 各 Wave 详细任务

### 技术文档
- [Tauri 2 官方文档](https://tauri.app/v2/guides/)
- [React 19 文档](https://react.dev/)
- [shadcn/ui 文档](https://ui.shadcn.com/)
- [sqlite-vec 文档](https://github.com/asg017/sqlite-vec)

---

## 📝 更新日志

**v1.0 (2025-12-31)**
- 初始版本
- 分析 18 个缺失模板
- 为每个模板提供详细 AI 提示词
- 按优先级分类（P0/P1/P2/P3）

---

**报告生成者：** tech-stack-code-generator skill
**联系方式：** 通过 Claude Code 技能系统反馈

---

**使用建议：**
1. 将此报告保存到项目文档目录
2. 按优先级逐步补充模板
3. 使用提供的 AI 提示词请求大模型生成代码
4. 验证和调整生成的代码
5. 更新到技能库

**预期效果：**
- 补充所有 P0 模板后，核心开发任务不再阻塞
- 补充所有 P1 模板后，常用功能有完整参考
- 补充所有 P2/P3 模板后，功能增强和质量保证完善
