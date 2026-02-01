# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

PrismForge 是一个基于 Tauri 2 + React 18 的桌面应用程序，核心功能是 Claude 会话监控和 LLM 提示词优化。应用支持多厂商 LLM API（OpenAI、Anthropic、Ollama、xAI），并提供安全的 API Key 管理和统一的调用接口。

## 工作约束

 **在使用 Claude Code 开发本项目时，必须遵守以下约束：**

### 1. 代码简洁性原则

✅ **追求代码简洁、模块化、可复用，避免过度复杂**

**核心原则**：代码应该足够简洁、模块化，可以直接复用，无须关心内部过程，并非越多越好。

**实现标准**：

1. **简洁性优先**：用最少的代码实现功能，避免不必要的复杂性
2. **模块化设计**：每个模块只做一件事，职责单一明确
3. **封装内部实现**：使用者只需知道"做什么"，无需了解"怎么做"
4. **可复用性第一**：通用功能提取为公共模块，避免重复代码
5. **拒绝过度工程**：不预支未来需求，不添加当前不需要的功能

**判断标准**：

- ✅ 代码行数减少 = 更好（在不损失可读性的前提下）
- ✅ 一个函数能完成 = 绝不拆成两个
- ✅ 现有组件能满足 = 绝不创建新组件
- ❌ 为了"可能将来需要"而添加功能 = 错误
- ❌ 代码"看起来很专业"但实际没用 = 错误

**示例**：

**✅ 正确示例：**
```typescript
// 简洁：直接使用
const isValid = validateEmail(email);

// 复用：现有组件
<Button onClick={handleSubmit}>提交</Button>
```

```rust
// 简洁：直接返回
pub fn get_config(&self) -> Result<Config> {
    self.config.load()
}
```

**❌ 错误示例：**
```typescript
// 过度封装：不必要的抽象层
class EmailValidatorAdapter {
  constructor(private validator: EmailValidator) {}
  validate(email: string) { return this.validator.validate(email); }
}

// 过度工程：为"将来可能需要"添加功能
interface Props {
  onSubmit: () => void;
  onCancel?: () => void;  // ❌ 当前未使用
}
```

```rust
// 过度拆分：简单操作多层包装
pub fn get_type(&self) -> Result<ProviderType> {
    let provider = self.load_provider()?;
    Ok(provider.get_type())
}
```

**遵循原则**：KISS（保持简单）、DRY（不重复）、YAGNI（不做不需要的事）

### 2. 交互模式（必读）

🔴 **开始任务前，必须先提出实现假设并获得确认**

- **步骤 1**：分析需求，提出你的实现假设
  - 使用的框架和类库
  - 架构设计方案
  - 涉及的关键文件和模块
- **步骤 2**：检查假设之间的矛盾关系
  - 技术栈兼容性
  - 架构设计一致性
  - 与现有代码的冲突
- **步骤 3**：等待用户确认后再开始编码
  - 不要假设用户会接受你的方案
  - 重大变更必须获得明确批准

**示例：**
```
❌ 错误：直接开始编码
✅ 正确："我计划使用 Zustand 创建新的 store 来管理会话状态，
       会修改 src/stores/useSessionStore.ts，这样设计符合吗？"
```

### 3. 代码复用优先

✅ **优先复用现有代码和类，仅在必要时创建新的**

- **检查清单**：
  1. 搜索项目中是否已有相似功能
  2. 检查是否可以扩展现有组件/函数
  3. 评估复用 vs 新增的成本
- **创建新代码的条件**：
  - 现有代码无法满足需求
  - 扩展现有代码会导致复杂度显著增加
  - 新代码有明确的复用价值

**示例：**
```
✅ 优先：使用现有的 useSettingsStore 状态管理模式
✅ 优先：复用 ProviderForm 组件的表单验证逻辑
❌ 避免：创建功能重复的工具函数
```

### 4. 问题澄清机制

❓ **遇到不清楚的细节时，主动提问获取补充信息**

- **必须提问的场景**：
  - 需求描述模糊或存在歧义
  - 多种实现方案，需要用户决策
  - 涉及架构变更或影响现有功能
  - 不确定业务逻辑或数据流向
- **提问方式**：
  - 描述当前理解
  - 列出可选方案及优劣
  - 推荐方案并说明理由
  - 等待用户决策

**示例：**
```
❌ 错误：自行猜测需求并实现
✅ 正确："你希望提供商列表支持搜索功能吗？
       我建议在前端实现过滤，无需后端修改，性能也更好。
       是否需要我实现这个方案？"
```

### 5. 国际化与主题约束

🌍 **所有用户可见文本必须使用 `useTranslation` hook**

```typescript
// ✅ 正确
const { t } = useTranslation('settings');
t('form.providerType')

// ❌ 错误：硬编码
"提供商类型"
```

- 翻译文件：`src/i18n/locales/{zh,en}/` （common.json, index.json, navigation.json, sessions.json, settings.json）
- 翻译键规范：`namespace.category.key` （如 `settings.form.providerType`）
- 动态内容：通过 `PROVIDER_TYPE_KEYS` 映射 + `useMemo` 缓存

🎨 **组件必须适配暗色/亮色主题（使用 CSS 变量）**

```tsx
// ✅ 正确：使用 CSS 变量
<div style={{ backgroundColor: 'var(--color-bg-card)', color: 'var(--color-text-primary)' }}>

// ❌ 错误：硬编码 Tailwind 颜色类
<div className="bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100">

// ❌ 错误：硬编码内联样式
<div style={{ backgroundColor: '#FFFFFF' }}>
```

**主题色 CSS 变量定义**（`src/index.css`）：

| 用途           | 浅色模式变量值                        | 暗色模式变量值                        |
| -------------- | ------------------------------------- | ------------------------------------- |
| **背景**       | `--color-bg-primary: #F8F9FA`         | `--color-bg-primary: #121212`         |
| **卡片背景**   | `--color-bg-card: #FFFFFF`            | `--color-bg-card: #1E1E1E`            |
| **主强调色**   | `--color-accent-warm: #F59E0B`        | `--color-accent-warm: #FF6B6B`        |
| **次强调色**   | `--color-accent-blue: #2563EB`        | `--color-accent-blue: #4A9EFF`        |
| **绿色主题**   | `--color-accent-green: #4CAF50`       | `--color-accent-green: #66BB6A`       |
| **文本主色**   | `--color-text-primary: #1F2937`       | `--color-text-primary: #E0E0E0`       |
| **文本次要**   | `--color-text-secondary: #6B7280`     | `--color-text-secondary: #9CA3AF`     |
| **边框色**     | `--color-border-light: #E5E7EB`       | `--color-border-light: #333333`       |

**使用示例**：
```tsx
// 按钮（组件已内置 CSS 变量）
<Button variant="primary">点击</Button>

// 卡片
<div style={{
  backgroundColor: 'var(--color-bg-card)',
  borderColor: 'var(--color-border-light)'
}}>

// 文本
<p style={{ color: 'var(--color-text-primary)' }}>内容</p>
```

**注意**：
- 所有 UI 组件（Button、Dialog、Input 等）已内置 CSS 变量，直接使用即可
- 自定义组件必须使用 CSS 变量，禁止硬编码颜色值
- 主题切换通过 `<html>` 标签的 `class="dark"` 自动生效

### 6.提交git前需要先与我确认

**总结**：

- 🤔 **思考** → 📋 **提出假设** → ✅ **等待确认** → 🔨 **开始编码**
- 🔍 **搜索** → ♻️ **复用优先** → 🆕 **必要时创建**
- ❓ **发现疑问** → 💬 **主动提问** → 📊 **提供选项** → 👍 **等待决策**

### **7.Rust 全局单例模式**

```rust
// ❌ 错误：创建新实例
let manager = ConfigManager::new(path)?;

// ✅ 正确：使用全局实例
let manager = get_config_manager()
    .ok_or_else(|| CommandError::new("未初始化"))?;
```

### 8.前端敏感日志防护

```typescript
// ❌ 危险：总是输出
console.log('路径:', sessionInfo.projectPath);

// ✅ 安全：仅开发环境
if (import.meta.env.DEV) {
  console.log('路径:', sessionInfo.projectPath);
}
```

### 9. ts-rs 类型生成

```rust
#[derive(TS)]
#[ts(rename_all = "camelCase")]
pub struct MyStruct {
    #[ts(type = "number")]
    pub timeout: u64,
}
```

- 生成命令：`cargo run --bin generate_types`
- 类型位置：`src/types/generated/`（前端从 `@/types/generated/` 导入）
- 入口文件：`src-tauri/src/build_types.rs`
- ⚠️ 禁止手动编辑生成的 `.ts` 文件

### 10.避免敏感信息泄露

**前端防护：**
- 使用 `import.meta.env.DEV` 判断开发环境
- 路径、用户信息等敏感数据仅开发环境输出
- 生产环境禁止输出敏感日志

**后端防护：**
- 使用 `#[cfg(debug_assertions)]` 条件编译
- API Key、密码等敏感数据禁止日志输出
- 敏感配置使用环境变量或安全存储

**数据传输：**
- API Key 仅在创建/更新时传输，使用 HTTPS
- 返回数据使用掩码处理（如 `api_key_mask`）
- 敏感字段使用 `secrecy::SecretString` 包装

### 11. String → Path 转换错误

```rust
// ❌ 错误：String 没有 file_name() 方法
let path = request.current_session_file_path
    .as_ref()
    .and_then(|p| p.file_name());

// ✅ 正确：使用 Path::new 获取引用
let path = if let Some(ref path_str) = request.current_session_file_path {
    std::path::Path::new(path_str).file_name()
        .and_then(|n| n.to_str())
} else {
    None
};
```

### 12. 常见陷阱

#### 12.1 非空断言滥用

```typescript
// ❌ 危险
onClick={(e) => handleToggleFavorite(history.id!, e)}

// ✅ 安全
onClick={(e) => { if (!history.id) return; handleToggleFavorite(history.id, e); }}
```

#### 12.2 正则捕获组索引变化

```typescript
// ✅ 使用命名捕获组
const regex = /###\s*\*\*(?<lang>目标偏离程度\|Goal Divergence)\*\*\s*\n(?<content>[\s\S]*?)/;
const content = match.groups?.content ?? '';
```

#### 12.3 测试文件模块未声明

```rust
// ❌ 错误：独立测试文件
// src-tauri/src/session_parser_tests.rs

// ✅ 正确：合并到主文件
#[cfg(test)]
mod integration_tests { }
```

#### 12.4 数据库竞态条件

```rust
// ❌ 错误：两次调用
self.with_conn_inner(|conn| conn.execute(...))?;
let id = self.with_conn_inner(|conn| Ok(conn.last_insert_rowid()))?;

// ✅ 正确：同一连接
let id = self.with_conn_inner(|conn| {
    conn.execute(...)?;
    Ok(conn.last_insert_rowid())
})?;
```

#### 12.5 国际化逻辑错误

```rust
// ❌ 错误：非英非中显示中文
if language == "en" { } else { "中文" }

// ✅ 正确：只有中文显示中文
if language == "zh" { "中文" } else { "English" }
```

#### 12.6 条件编译误用

```rust
// ❌ 错误：cfg! 是运行时判断
if cfg!(debug_assertions) { eprintln!("..."); }

// ✅ 正确：使用属性
#[cfg(debug_assertions)]
{ eprintln!("..."); }
```

#### 12.7 避免快速交付思维：初期过分追求"功能能跑"，忽视了用户体验一致性

1. **UI 组件统一使用**：必须使用 `src/components/ui/` 中的 UI 组件
   - 按钮统一使用 `<Button>` 组件（禁止直接写 `<button className="...">` 或硬编码颜色）
   - 对话框统一使用 `<Dialog>` 或 `<AlertDialog>`（禁止使用原生 confirm/alert）
   - 表单输入统一使用 `<Input>`、`<Textarea>`、`<Select>` 等
3. **交互模式统一**：
   - 删除操作必须使用 `<AlertDialog>` 确认
   - 表单提交按钮必须在提交时显示 loading 状态
   - 所有用户可见文本必须使用 `useTranslation` hook（见条款 4）

**注意**：项目采用 CSS 变量主题系统（`src/index.css`），所有主题色通过 `:root` 和 `:root.dark` 定义，自动适配深浅色模式。

#### 12.8 避免缺乏组件意识：未意识到项目中已有实现功能的组件

1. **实现前必须查阅组件库文档**：`docs/COMPONENTS.md`（记录所有业务组件）
2. **优先使用现有组件**：
   - UI 基础组件：`src/components/ui/`（Button、Dialog、Input、AlertDialog 等）
   - 业务组件：`src/components/{settings,project,session,prompt}/`（见 COMPONENTS.md）
3. **复用判断标准**：
   - 如果功能 >50% 相似，优先考虑扩展现有组件
   - 如果需要新组件，先与项目负责人确认

#### 12.9 避免设计债务累积：临时解决方案未及时重构，演变成技术债务

1. **禁止临时方案**：除非项目负责人明确允许，否则不允许使用临时方案
2. **实现前检查清单**（必须全部满足才能开始编码）：
   - [ ] 检查是否与现有功能集成（见 12.8 组件复用）
   - [ ] 确认 UI/UX 一致性（见 12.7 用户体验规范）
   - [ ] 验证代码风格符合项目规范
   - [ ] 考虑错误处理和边界情况
   - [ ] 确认安全性检查（避免 XSS、注入等漏洞）
   - [ ] 确认国际化支持（`useTranslation`）
   - [ ] 确认主题适配（使用 CSS 变量，不硬编码颜色）

#### 12.10 提高代码复用意识

实现功能前，优先检查是否存在已实现相同功能的方法

### 13. 防御性编程原则

  - **永不信任外部输入**：包括数据库、API、用户输入
  - **提供默认值**：当数据不符合预期时，使用安全的降级策略
  - **日志记录**：在开发环境输出验证失败的警告
  - **永不使用 `unwrap()`**：在生产代码中，所有 `unwrap()` 应替换为 `?` 或 `map_err`
  - **慎用 `expect()`**：仅在确实不可能失败的情况下使用（如硬编码的常量）
  - **移除手动 `unsafe impl`**：让编译器自动推导 Send/Sync
  - **使用事务作用域**：确保数据库操作原子性
  - **类型安全转换**：避免裸类型比较和断言
  - **超时和重试**：对可能阻塞的操作添加超时机制
   - **使用显式类型注解**: rusqlite 数据库行获取安全

```rust
// ❌ 错误：unwrap_or 在类型不匹配时会 panic
let id: i64 = row.get(0).unwrap_or(0);

// ✅ 正确：使用显式类型注解 + unwrap_or_default
let id: i64 = row.get(0).unwrap_or_default();

// ✅ 最佳：使用 ? 传播错误
let id: i64 = row.get(0)?;
```

### 14. 避免联合类型理解偏差

  - 前后端类型映射时，需要考虑联合类型的处理
  - 使用 as const 确保字面量类型推断
  - 处理 bigint 和 number 的类型差异（Rust i64 → TS bigint）



## 技术栈

**后端 (Rust + Tauri 2):**
- `tauri 2.0` - 桌面应用框架
- `reqwest 0.12` - HTTP 客户端（支持流式传输）
- `async-openai 0.25` - OpenAI SDK
- `rusqlite 0.32` - SQLite 数据库（bundled）
- `keyring 3.0` - 跨平台安全存储（API Key）
- `secrecy 0.10` - 敏感数据保护
- `serde/serde_json` - 序列化
- `ts-rs 0.1` - TypeScript 类型生成

**前端 (React + TypeScript):**
- `react 18.3` + `react-dom 18.3`
- `react-router-dom 6.30` - 路由
- `zustand 5.0` + `immer` - 状态管理
- `react-hook-form 7.69` - 表单管理
- `vite 7.0` - 构建工具
- `react-i18next` - 国际化（i18n）
- `@tanstack/react-query` - 数据获取和缓存
- `@heroicons/react` - 图标库

## 开发命令

### 前端开发
```bash
# 安装依赖
npm install

# 启动开发服务器（端口 1420）
npm run dev

# TypeScript 类型检查
npm run build

# 预览生产构建
npm run preview
```

### Tauri 开发
```bash
# 完整开发模式（前端 + 后端热重载）
npm run tauri dev

# 构建生产版本
npm run tauri build
```

### Rust 后端开发
```bash
# 进入 Rust 目录
cd src-tauri

# 运行测试
cargo test

# 检查代码（不构建，快速验证编译）
cargo check

# 格式化代码
cargo fmt

# Lint 检查（捕获潜在问题）
cargo clippy

# 仅编译单个包（加速开发）
cargo build -p prism-forge

# 运行特定测试
cargo test test_name
```

## 项目架构

### 整体架构模式

项目采用 **Tauri 前后端分离架构**，前端通过 Tauri Invoke API 调用后端命令。后端实现多厂商 LLM 适配器模式，通过统一的 `LLMService` trait 抽象不同厂商 API。

### Rust 后端结构

```
src-tauri/src/
├── main.rs              # Tauri 入口，应用生命周期
├── lib.rs               # 核心模块注册和 Tauri 状态管理
├── build_types.rs       # ts-rs 类型生成入口
├── commands.rs          # Tauri 命令接口（前端调用入口）
├── session_parser.rs    # 统一会话解析服务
├── config/              # 配置管理模块
│   ├── mod.rs           # 配置模块入口
│   └── app_config.rs    # 应用配置管理
├── database/            # 数据持久化层
│   ├── models.rs        # ApiProvider 数据模型
│   ├── migrations.rs    # SQLite 表结构和初始化
│   └── repository.rs    # CRUD 操作实现
├── llm/                 # LLM 客户端核心
│   ├── interface.rs     # LLMService trait 和通用类型
│   ├── manager.rs       # LLMClientManager（单例管理器）
│   ├── security.rs      # API Key 安全存储（keyring + 验证）
│   └── providers/       # 厂商适配器实现
│       ├── openai.rs    # OpenAI 适配器（使用 async-openai）
│       ├── anthropic.rs # Anthropic 适配器（手动 HTTP）
│       ├── ollama.rs    # Ollama 适配器（本地服务）
│       └── xai.rs       # xAI 适配器
├── parser/              # JSONL 解析和消息树构建
│   ├── jsonl.rs         # JSONL 文件解析器
│   ├── view_level.rs    # 视图等级过滤和问答对提取
│   ├── tree.rs          # 消息树构建
│   └── extractor.rs     # 会话内容提取器
├── filter_config.rs     # 日志过滤配置管理
└── optimizer/           # 提示词优化业务逻辑
    └── mod.rs           # 会话分析和提示词生成
```

### React 前端结构

```
src/
├── main.tsx             # React 入口，挂载到 #app root
├── App.tsx              # 主应用组件
├── i18n/                # 国际化配置
│   ├── config.ts        # i18next 配置
│   └── locales/         # 翻译文件
│       ├── zh/          # 中文翻译
│       └── en/          # 英文翻译
├── stores/              # Zustand 全局状态
│   ├── useSettingsStore.ts  # 提供商管理状态
│   └── useThemeStore.ts     # 主题管理状态
├── lib/                 # 工具函数库
│   └── utils.ts         # 通用工具函数
├── hooks/               # 自定义 React Hooks
│   └── useTranslation.ts    # 国际化 Hook
├── types/               # TypeScript 类型定义
│   └── generated/       # ts-rs 生成的类型
│       └── index.ts     # 从 Rust 导出的类型
├── pages/               # 页面级组件
│   ├── Settings.tsx     # 设置页面（提供商 CRUD）
│   └── Sessions.tsx     # 会话管理页面
└── components/          # 可复用组件
    ├── settings/
    │   └── ProviderForm.tsx  # 提供商表单
    └── ui/               # UI 组件库
        ├── Button.tsx
        ├── Input.tsx
        └── Modal.tsx
```

**核心设计原则：**
1. 适配器模式：`LLMService` trait 抽象多厂商 API
2. 工厂模式：`LLMClientManager::create_client_from_provider()`
3. 仓库模式：`ApiProviderRepository` 封装数据库操作
4. 单例模式：`LLMClientManager` 通过 Tauri State 注入
5. 安全优先：API Key 存储在 OS 凭据管理器

## 关键技术点

### 1. Tauri 命令接口

```rust
#[tauri::command]
pub async fn cmd_xxx(
    manager: State<'_, LLMClientManager>,
    param: Type,
) -> Result<Response, CommandError> {
    Ok(result)
}
```

⚠️ **必须在 `lib.rs` 的 `invoke_handler!` 宏中注册**

### 2. 序列化命名

- Rust → 前端：`#[serde(rename_all = "camelCase")]`
- 前端 → Rust：自动转换 camelCase → snake_case

### 3. 敏感信息处理

- API Key 仅保存时传输，立即存入 keyring
- 返回掩码：`api_key_mask`（仅前 8 字符）
- 使用 `secrecy::SecretString` 包装

### 4. 多厂商适配器

```rust
#[async_trait]
pub trait LLMService {
    async fn chat_completion(&self, messages: Vec<Message>, params: ModelParams)
        -> Result<ChatCompletionResponse>;
}
```

扩展新厂商：添加枚举值 → 实现 trait → 更新工厂方法 → 前端同步

### 5. 会话解析服务

```rust
let parser = SessionParserService::new(config);
let result = parser.parse_session("/path/to/session.jsonl", "session_id")?;
```

解析流程：JsonlParser → Message 转换 → 内容过滤 → 视图等级过滤

### 6. 调试模式

- 前端：`const DEBUG = import.meta.env.DEV;`
- 后端：`#[cfg(debug_assertions)]`

## 潜在风险和注意事项

### 安全风险

⚠️ **Keyring 清理风险**（P0 优先级）
- **问题**：删除提供商时 keyring 清理可能失败（commands.rs:569），导致密钥残留
- **缓解措施**：
  - 添加删除验证逻辑，确保 keyring 清理成功
  - 实现定期审计机制清理孤立密钥
  - 考虑实现密钥轮换机制

⚠️ **Linux 兼容性**（P1 优先级）
- **问题**：keyring 在某些 Linux 发行版上可能不稳定（依赖 libsecret）
- **影响**：可能导致 API Key 存储失败
- **测试**：在主流 Linux 发行版（Ubuntu、Fedora、Arch）上验证

⚠️ **输入验证不足**（P1 优先级）
- **问题**：缺少速率限制和全面的输入 sanitization
- **风险**：可能被滥用或注入恶意内容
- **建议**：
  - 实现速率限制（Token Bucket 或 Sliding Window）
  - 使用 `validator` crate 添加邮箱、URL 验证
  - 模型名称添加白名单验证

### 性能风险

⚠️ **Mutex 锁竞争**（P1 优先级）
- **问题**：数据库使用 `Arc<Mutex<>>`，无连接池，高并发场景性能差
- **影响位置**：
  - src-tauri/src/database/repository.rs:13-18
  - src-tauri/src/llm/manager.rs:16-24
- **改进建议**：
  - 使用 `r2d2` 或 `sqlx` 引入连接池
  - 读多写少场景使用 `RwLock` 替代 `Mutex`
  - 使用 `tokio::sync::Semaphore` 限制并发数

⚠️ **前端缺少缓存和防抖**
- **问题**：频繁调用 API，无请求缓存
- **建议**：使用 lodash debounce 或手动实现防抖

### 并发安全风险

🔴 **手动实现 Send/Sync**（P0 优先级）
- **问题**：多处使用 `unsafe impl Send/Sync`，存在数据竞争风险
- **影响位置**：
  - src-tauri/src/commands.rs:23-24
  - src-tauri/src/llm/manager.rs:16-24
  - src-tauri/src/database/repository.rs:13-18
- **修复**：移除手动 `unsafe impl`，让编译器自动推导
- **示例**：
  ```rust
  // ❌ 不安全：手动实现
  unsafe impl Send for LLMClientManager {}
  unsafe impl Sync for LLMClientManager {}
  
  // ✅ 安全：移除 unsafe，使用 Arc<Mutex<T>> 自动推导
  pub struct LLMClientManager {
      repository: Arc<Mutex<ApiProviderRepository>>,
  }
  ```

### 数据一致性风险

⚠️ **Keyring 与数据库不一致**
- **场景**：删除提供商但 keyring 清理失败
- **影响**：密钥泄漏，存储空间浪费
- **建议**：添加清理验证和定期审计

⚠️ **活跃提供商不一致**
- **场景**：数据库触发器失败但代码未检查
- **影响**：多个活跃提供商导致混乱
- **建议**：添加应用层验证逻辑

## 关键限制和注意事项

### Tauri 命令注册限制

🔴 **命令必须注册**（新手常见错误）
- **规则**：所有暴露给前端的命令必须在 `lib.rs` 的 `invoke_handler!` 宏中注册
- **症状**：未注册的命令前端调用时不会报错，但无响应
- **检查**：每次添加新命令后，务必检查 `lib.rs` 中的 `invoke_handler!` 宏

```rust
// lib.rs
invoke_handler![
    cmd_get_providers,        // ✅ 已注册
    cmd_save_provider,        // ✅ 已注册
    // cmd_new_command,       // ❌ 未注册，前端无法调用
]
```

### 错误处理限制

 **引入错误码枚举**
```rust
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub code: ErrorCode,           // ProviderNotFound | AuthenticationFailed | NetworkError
    pub message: String,
    pub details: Option<String>,
}
```

### 测试限制

· **引入依赖注入容器（如 `diagonal` crate）**

· **配置测试工具链，添加 CI 自动运行**

### 架构权衡

**引入 `LLMServiceExt` trait 支持扩展功能**

## 数据库文件位置

**开发环境数据库位置：**

```
Windows: %APPDATA%\prism-forge\prism-forge.db
         完整路径示例：C:\Users\用户名\AppData\Roaming\prism-forge\prism-forge.db

macOS:   ~/Library/Application Support/prism-forge/prism-forge.db

Linux:   ~/.config/prism-forge/prism-forge.db
```

**调试技巧：**
- 使用 SQLite 客户端（如 DB Browser for SQLite）打开数据库文件查看内容
- 删除数据库文件后重启应用会自动重新创建
- 修改 Schema 时需要删除旧数据库或编写迁移逻辑

## 代码风格规范

- **注释**：中文
- **Rust 命名**：snake_case（函数/变量）、PascalCase（类型/枚举）、SCREAMING_SNAKE_CASE（常量）
- **TypeScript 命名**：camelCase（变量/函数）、PascalCase（类型/接口/枚举）
- **文件命名**：Rust 使用 snake_case.rs，TS/TSX 使用 PascalCase.tsx

## 相关资源

- [Tauri 官方文档](https://tauri.app/v2/guides/)
- [Tauri Invoke API](https://tauri.app/v2/api/js/core/#functioninvoke)
