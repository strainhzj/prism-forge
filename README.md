# PrismForge - Claude Code Session Manager & Prompt Optimizer

> **Claude Code 会话文件管理与提示词优化工具** | 专为 Claude Code 用户设计的桌面应用

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-FFC131?logo=tauri)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-18.3-61DAFB?logo=react)](https://react.dev/)

## 什么是 PrismForge？

PrismForge 是一款专为 **Claude Code** 用户设计的会话管理桌面应用。当你使用 **Claude Code** 进行开发时，如果对如何实现目标有疑问，PrismForge 可以：

1. 读取完整的 **Claude Code** 会话文件（`.jsonl`）
2. 分析你的项目上下文和会话历史
3. 结合你的目标描述，智能生成优化后的提示词
4. 帮助你更高效地使用 **Claude Code** 完成复杂任务

## 为什么需要 PrismForge？

**Claude Code** 是 Anthropic 提供的强大 AI 编程助手，但实际使用中常遇到这些问题：

- **表达困难**：有明确目标，但不清楚如何向 **Claude Code** 清楚描述需求
- **提示词优化**：不知道如何根据项目上下文优化提示词

PrismForge 专为 **Claude Code** 用户解决这些痛点。

---

## 核心功能

### 1. 提示词智能生成（Prompt Optimizer）

基于 **Claude Code** 会话历史和项目上下文，自动生成优化提示词：

- 读取 **Claude Code** 会话文件的完整历史记录
- 分析项目代码结构和已有对话
- 结合你的目标描述输入
- 生成适合当前场景的优化提示词
- 支持多厂商 LLM API

**使用场景**：当你有开发目标但不知道如何向 **Claude Code** 表达时。

### 2. 会话历史管理（Session Manager）

统一管理本地所有项目的 **Claude Code** 会话：

- 自动发现本地项目的 **Claude Code** 会话文件
- 统一查看所有项目的会话记录
- 按项目、时间筛选会话历史
- 快速浏览会话内容

### 3. 会话文件切换（Session Switcher）

灵活切换不同项目的 **Claude Code** 会话上下文：

- 默认自动跟踪最新项目的 **Claude Code** 会话文件
- 支持手动切换到任意历史会话文件
- 快速在不同项目的会话上下文间切换

---

## 特性亮点

### 🌍 多语言支持

- **双语界面**：支持中文和英文界面切换
- **实时切换**：无需重启应用，即时切换语言
- **扩展性强**：基于 `react-i18next`，易于添加新语言
- **翻译文件**：位于 `src/i18n/locales/{zh,en}/`
  - `common.json` - 通用文本（按钮、标签等）
  - `settings.json` - 设置页面（表单、验证、供应商类型）
  - `sessions.json` - 会话管理页面
  - `navigation.json` - 导航菜单
  - `index.json` - 首页（项目切换器、时间线）

### 🎨 暗色/亮色主题

- **双主题模式**：支持暗色和亮色两种主题
- **自动检测**：根据系统主题偏好自动切换
- **手动切换**：用户可在设置中手动切换主题
- **全面适配**：所有组件均已适配两种主题

**主题实现：**
- 使用 Tailwind `dark:` 前缀适配主题
- 通过 CSS 变量定义主题颜色（`src/index.css`）
- 主题状态通过 `useThemeStore` 管理

### 🔐 安全的 API Key 管理

- **多厂商支持**：支持 OpenAI、Anthropic、Ollama、xAI
- **安全存储**：使用操作系统凭据管理器存储 API Key
  - **Windows**: Credential Manager
  - **macOS**: Keychain
  - **Linux**: Secret Service (libsecret)
- **密钥隔离**：数据库仅保留密钥引用，不存储明文
- **掩码显示**：界面显示掩码后的密钥（如 `sk-xxxx1234`）
- **验证测试**：支持连接测试，验证 API Key 有效性

### 📝 智能会话解析

- **统一解析服务**：`SessionParserService` 提供统一的会话文件解析接口
- **格式支持**：支持 Claude Code 的 JSONL 会话文件格式
- **消息转换**：将 `JsonlEntry` 转换为结构化的 `Message` 对象
- **内容过滤**：应用 `FilterConfigManager` 规则过滤不需要的内容
- **视图等级**：支持多种视图等级过滤（Full、QAPairs、Summary）

### ⚡ TypeScript 类型同步

- **自动生成**：使用 `ts-rs` 从 Rust 结构体自动生成 TypeScript 类型
- **类型安全**：前后端共享类型定义，减少类型错误
- **实时同步**：修改 Rust 结构体后重新运行生成命令即可
- **命名约定**：自动转换为驼峰命名（camelCase）

### 🔄 多厂商 LLM 适配器

- **统一接口**：通过 `LLMService` trait 抽象不同厂商 API
- **工厂模式**：`LLMClientManager::create_client_from_provider()` 动态创建客户端
- **易于扩展**：添加新厂商只需实现 trait 并更新工厂方法
- **流式支持**：支持流式响应，实时显示生成内容

---

## 技术栈

### 前端 (React + TypeScript)
- **React 18.3** + **React DOM 18.3** - UI 框架
- **Vite 7.0** - 构建工具
- **React Router 6.30** - 路由管理
- **Zustand 5.0** + **Immer** - 状态管理
- **React Hook Form 7.69** - 表单管理
- **React i18next** - 国际化（中英文切换）
- **@tanstack/react-query** - 数据获取和缓存
- **@heroicons/react** - 图标库
- **Tailwind CSS** - 样式框架（支持暗色/亮色主题）

### 后端 (Rust + Tauri 2)
- **Tauri 2.0** - 桌面应用框架
- **reqwest 0.12** - HTTP 客户端（支持流式传输）
- **rusqlite 0.32** - SQLite 数据库
- **keyring 3.0** - 跨平台安全存储（API Key 管理）
- **secrecy 0.10** - 敏感数据保护
- **async-openai 0.25** - OpenAI SDK
- **ts-rs 0.1** - TypeScript 类型生成
- **serde / serde_json** - 序列化

---

## 快速开始

### 环境要求

- Node.js 18+
- Rust 工具链（[安装指南](https://www.rust-lang.org/tools/install)）
- npm / pnpm / yarn

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri dev
```

### 生产构建

```bash
npm run tauri build
```

---

## 数据存储

### 会话数据库位置

- **Windows**:
  ```
  %APPDATA%\prism-forge\prism-forge.db
  完整路径示例：C:\Users\用户名\AppData\Roaming\prism-forge\prism-forge.db
  ```

- **macOS**: `~/Library/Application Support/prism-forge/prism-forge.db`

- **Linux**: `~/.config/prism-forge/prism-forge.db`

### 调试技巧

**查看数据库内容：**
- 使用 SQLite 客户端（如 [DB Browser for SQLite](https://sqlitebrowser.org/)）打开数据库文件
- 查看表结构：`SELECT * FROM sqlite_master;`
- 查看数据：`SELECT * FROM api_providers;`

**重置数据库：**
```bash
# 删除数据库文件后重启应用会自动重新创建
# Windows
del %APPDATA%\prism-forge\prism-forge.db

# macOS/Linux
rm ~/.config/prism-forge/prism-forge.db
```

**Schema 修改：**
- 修改 Schema 时需要删除旧数据库或编写迁移逻辑
- 迁移文件位于 `src-tauri/src/database/migrations.rs`

### API Key 安全存储

使用操作系统凭据管理器存储 API Key，确保密钥安全：

| 操作系统 | 凭据管理器 |
|---------|-----------|
| **Windows** | Credential Manager |
| **macOS** | Keychain |
| **Linux** | Secret Service (libsecret) |

**安全措施：**
- 数据库仅保留密钥引用，不存储明文
- 使用 `keyring` crate 跨平台访问系统凭据管理器
- 使用 `secrecy::SecretString` 包装密钥，防止意外日志泄露
- 界面显示掩码后的密钥（如 `sk-xxxx1234`）

---

## 开发说明

### 前端开发命令

```bash
# 安装依赖
npm install

# 启动开发服务器（端口 1420）
npm run dev

# TypeScript 类型检查
npm run build

# 生成 TypeScript 类型（从 Rust 结构体）
cargo run --bin generate_types

# 预览生产构建
npm run preview
```

### Rust 后端开发

```bash
# 进入 Rust 目录
cd src-tauri

# 运行测试
cargo test

# 运行特定测试
cargo test test_name

# 代码检查（不构建，快速验证）
cargo check

# 格式化代码
cargo fmt

# Lint 检查（捕获潜在问题）
cargo clippy

# 仅编译单个包（加速开发）
cargo build -p prism-forge
```

### 类型生成工作流

当你修改了 Rust 结构体并需要同步 TypeScript 类型时：

```bash
# 1. 在 Rust 结构体上添加 #[derive(TS)] 属性
# src-tauri/src/database/models.rs
#[derive(TS)]
#[ts(rename_all = "camelCase")]
pub struct ApiProvider {
    pub id: i32,
    pub name: String,
    pub provider_type: ApiProviderType,
}

# 2. 在 src-tauri/src/build_types.rs 中注册类型
pub fn export_types() -> Result<()> {
    ApiProvider::export()?;
    // ...

# 3. 运行生成命令
cargo run --bin generate_types

# 4. 前端自动获得 TypeScript 类型定义
# src/types/generated/ApiProvider.ts
export interface ApiProvider {
    id: number;
    name: string;
    providerType: ApiProviderType;
}

# 5. 在前端使用
import { ApiProvider } from '@/types/generated';
```

**常用 ts-rs 属性：**
- `#[ts(rename_all = "camelCase")]` - 字段名转驼峰命名
- `#[ts(type = "number")]` - 覆盖默认类型推断
- `#[ts(export)]` - 强制导出类型
- `#[ts(opaque)]` - 将类型视为不透明（不展开内部结构）

---

## 推荐开发工具

- [VS Code](https://code.visualstudio.com/) - 代码编辑器
- [Tauri VSCode Extension](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) - Tauri 开发支持
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) - Rust 语言支持

---

## 项目架构

### 整体架构

项目采用 **Tauri 前后端分离架构**，前端通过 Tauri Invoke API 调用后端命令。后端实现多厂商 LLM 适配器模式，通过统一的 `LLMService` trait 抽象不同厂商 API。

### Rust 后端结构

```
src-tauri/src/
├── main.rs              # Tauri 入口
├── lib.rs               # 核心模块注册和状态管理
├── build_types.rs       # ts-rs 类型生成
├── commands.rs          # Tauri 命令接口
├── session_parser.rs    # 统一会话解析服务
├── config/              # 配置管理
├── database/            # 数据持久化
├── llm/                 # LLM 客户端核心
│   └── providers/       # 厂商适配器（OpenAI、Anthropic、Ollama、xAI）
├── parser/              # JSONL 解析
├── filter_config.rs     # 日志过滤配置
└── optimizer/           # 提示词优化
```

### React 前端结构

```
src/
├── main.tsx             # React 入口
├── App.tsx              # 主应用组件
├── i18n/                # 国际化配置
│   └── locales/         # 翻译文件 {zh,en}/
├── stores/              # Zustand 全局状态
├── lib/                 # 工具函数库
├── hooks/               # 自定义 Hooks
├── types/               # TypeScript 类型
│   └── generated/       # ts-rs 生成的类型
├── pages/               # 页面级组件
└── components/          # 可复用组件
    └── ui/              # UI 组件库
```

### 核心设计模式

- **适配器模式**：`LLMService` trait 抽象多厂商 API
- **工厂模式**：`LLMClientManager::create_client_from_provider()`
- **仓库模式**：`ApiProviderRepository` 封装数据库操作
- **单例模式**：`LLMClientManager` 通过 Tauri State 注入

---

## 与 Claude Code 的关系

PrismForge 是 **Claude Code** 的配套工具，两者协同工作：

- **Claude Code**：AI 编程助手，执行实际的开发任务
- **PrismForge**：管理 **Claude Code** 会话文件，优化提示词生成

PrismForge 不替代 **Claude Code**，而是增强你的 **Claude Code** 使用体验。

---

## 更新日志 (Changelog)

### v1.0.2 - 提示词版本管理系统 (2026-02-01)

#### 🎯 核心功能

**提示词版本管理系统**
- ✅ 完整的版本控制：支持提示词模板的版本化管理
- ✅ 组件化架构：UserMessage、MetaPrompt、OutputFormat 独立管理
- ✅ 变更追踪：字段级、组件级的自动变更追踪
- ✅ 一键回滚：快速回滚到任意历史版本
- ✅ 版本对比：可视化展示版本间差异

#### 📦 新增功能

**数据库层（Rust）**
- 新增 `PromptVersionRepository`、`PromptComponentRepository`、`RollbackRepository`
- 支持版本 CRUD、组件解析、变更计算
- 防御性编程：全面使用 `?` 和显式类型注解避免 panic
- 事务支持：确保版本创建的原子性
- 版本创建时自动从 content 解析并创建组件记录

**前端 UI 层（React）**
- 提示词库页面（`PromptsPage`）：统一管理所有提示词模板
- 版本管理抽屉（`PromptVersionsDrawer`）：版本列表和详情
- 版本编辑器（`PromptEditDrawer`）：上下双窗格布局，实时预览
- 变更历史面板（`ChangeHistoryPanel`）：字段级变更追踪
- 版本对比面板（`VersionComparePanel`）：可视化差异展示
- 回滚确认对话框（`RollbackDialog`）：安全的版本回滚

#### 🔧 技术改进

**代码质量**
- 修复 rusqlite `row.get()` panic 风险（使用显式类型注解）
- 修复 SQL 参数缺失问题（使用 `rusqlite::params!` 宏）
- 修复数据库列索引不对齐问题
- 修复 ESM 模式下 `__dirname` 不可用问题（使用 `import.meta.url`）

**用户体验**
- 实时刷新编辑后版本列表
- 优化按钮分辨度（区分操作类型）
- 完整暗色模式适配
- 中英文双语支持（新增 `prompts.json`、`promptVersions.json`）

#### 📊 统计数据

- 90 个文件更改
- 11,260 行新增代码
- 494 行删除代码
- 28 个提交记录

#### 🐛 Bug 修复

- 修复 `init_default_prompts` 中 UPDATE 语句缺少 template_id 参数
- 修复 `prompt_versions` 中 `row_to_change` 列索引偏移错误
- 修复 vite/vitest 配置在 ESM 模式下的兼容性问题

---

## 相关链接

- [Claude Code 官方文档](https://code.anthropic.com/)
- [Tauri 官方文档](https://tauri.app/v2/guides/)
- [Anthropic Claude API](https://docs.anthropic.com/)

---

## 许可证

本项目采用 [MIT 许可证](LICENSE)。

---

## 关键词

claude code, claude-code, claude ai, anthropic, session manager, prompt optimizer, prompt engineering, ai assistant, code assistant, tauri, rust, react, typescript
