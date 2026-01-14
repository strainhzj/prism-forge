# PrismForge - Claude Code Session Manager & Prompt Optimizer

> **Claude Code 会话文件管理与提示词优化工具** | 专为 Claude Code 用户设计的桌面应用

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-FFC131?logo=tauri)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-19-61DAFB?logo=react)](https://react.dev/)

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

## 技术栈

### 前端
- **React 19** + TypeScript - UI 框架
- **Vite 7.0** - 构建工具
- **React Router 6.30** - 路由管理
- **Zustand 5.0** + Immer - 状态管理
- **React Hook Form 7.69** - 表单管理
- **Tailwind CSS** - 样式框架

### 后端
- **Rust** + **Tauri 2.0** - 桌面应用框架
- **reqwest 0.12** - HTTP 客户端（支持流式传输）
- **rusqlite 0.32** - SQLite 数据库
- **keyring 3.0** - 跨平台安全存储（API Key 管理）
- **async-openai 0.25** - OpenAI SDK
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

- **Windows**: `%APPDATA%\prism-forge\prism-forge.db`
- **macOS**: `~/Library/Application Support/prism-forge/prism-forge.db`
- **Linux**: `~/.config/prism-forge/prism-forge.db`

### API Key 安全存储

使用操作系统凭据管理器（keyring）存储 API Key，数据库仅保留密钥引用，确保安全。

---

## 开发说明

### 前端开发命令

```bash
# 启动开发服务器（端口 1420）
npm run dev

# TypeScript 类型检查
npm run build

# 预览生产构建
npm run preview
```

### Rust 后端开发

```bash
cd src-tauri

# 运行测试
cargo test

# 代码检查（不构建，快速验证）
cargo check

# 格式化代码
cargo fmt

# Lint 检查
cargo clippy

# 仅编译单个包（加速开发）
cargo build -p prism-forge
```

---

## 推荐开发工具

- [VS Code](https://code.visualstudio.com/) - 代码编辑器
- [Tauri VSCode Extension](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) - Tauri 开发支持
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) - Rust 语言支持

---

## 项目架构

```
src-tauri/src/
├── main.rs              # Tauri 入口，应用生命周期
├── lib.rs               # 核心模块注册和状态管理
├── commands.rs          # Tauri 命令接口（前端调用入口）
├── database/            # 数据持久化层
│   ├── models.rs        # ApiProvider 数据模型
│   ├── migrations.rs    # SQLite 表结构和初始化
│   └── repository.rs    # CRUD 操作实现
├── llm/                 # LLM 客户端核心
│   ├── interface.rs     # LLMService trait 和通用类型
│   ├── manager.rs       # LLMClientManager（单例管理器）
│   ├── security.rs      # API Key 安全存储（keyring + 验证）
│   └── providers/       # 厂商适配器实现
│       ├── openai.rs    # OpenAI 适配器
│       ├── anthropic.rs # Anthropic Claude 适配器
│       ├── ollama.rs    # Ollama 适配器
│       └── xai.rs       # xAI 适配器
└── optimizer/           # 提示词优化业务逻辑
    └── mod.rs           # 会话分析和提示词生成

src/
├── main.tsx             # React 入口
├── App.tsx              # 主应用组件
├── stores/              # Zustand 全局状态
│   └── useSettingsStore.ts  # 提供商管理状态
├── pages/               # 页面级组件
│   └── Settings.tsx     # 设置页面
└── components/          # 可复用组件
    └── settings/
        └── ProviderForm.tsx  # 提供商表单
```

---

## 与 Claude Code 的关系

PrismForge 是 **Claude Code** 的配套工具，两者协同工作：

- **Claude Code**：AI 编程助手，执行实际的开发任务
- **PrismForge**：管理 **Claude Code** 会话文件，优化提示词生成

PrismForge 不替代 **Claude Code**，而是增强你的 **Claude Code** 使用体验。

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
