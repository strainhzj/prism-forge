# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

PrismForge 是一个基于 Tauri 2 + React 18 的桌面应用程序，核心功能是 Claude 会话监控和 LLM 提示词优化。应用支持多厂商 LLM API（OpenAI、Anthropic、Ollama、xAI），并提供安全的 API Key 管理和统一的调用接口。

## 技术栈

**后端 (Rust + Tauri 2):**
- `tauri 2.0` - 桌面应用框架
- `reqwest 0.12` - HTTP 客户端（支持流式传输）
- `async-openai 0.25` - OpenAI SDK
- `rusqlite 0.32` - SQLite 数据库（bundled）
- `keyring 3.0` - 跨平台安全存储（API Key）
- `secrecy 0.10` - 敏感数据保护
- `serde/serde_json` - 序列化

**前端 (React + TypeScript):**
- `react 18.3` + `react-dom 18.3`
- `react-router-dom 6.30` - 路由
- `zustand 5.0` + `immer` - 状态管理
- `react-hook-form 7.69` - 表单管理
- `vite 7.0` - 构建工具

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

# 检查代码（不构建）
cargo check

# 格式化代码
cargo fmt

# Lint 检查
cargo clippy
```

## 项目架构

### 整体架构模式

项目采用 **Tauri 前后端分离架构**，前端通过 Tauri Invoke API 调用后端命令。后端实现多厂商 LLM 适配器模式，通过统一的 `LLMService` trait 抽象不同厂商 API。

### Rust 后端结构

```
src-tauri/src/
├── main.rs              # Tauri 入口，应用生命周期
├── lib.rs               # 核心模块注册和 Tauri 状态管理
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
│       ├── openai.rs    # OpenAI 适配器（使用 async-openai）
│       ├── anthropic.rs # Anthropic 适配器（手动 HTTP）
│       ├── ollama.rs    # Ollama 适配器（本地服务）
│       └── xai.rs       # xAI 适配器
└── optimizer/           # 提示词优化业务逻辑
    └── mod.rs           # 会话分析和提示词生成
```

**关键设计决策：**

1. **API Key 安全存储**：使用 `keyring` crate 将密钥存储在操作系统凭据管理器中，数据库仅存储引用（`api_key_ref: "provider_{id}"`）
2. **活跃提供商单例**：数据库使用触发器确保同一时间只有一个 `is_active = true` 的提供商
3. **错误处理链**：`anyhow::Error` → `CommandError`，统一序列化为 `{ message: string }` 返回前端
4. **异步支持**：所有 LLM 调用都是异步的（`async fn`），Tauri 命令使用 `async fn` 自动处理

### React 前端结构

```
src/
├── main.tsx             # React 入口，挂载到 #app root
├── App.tsx              # 主应用组件（会话监控界面）
├── stores/              # Zustand 全局状态
│   └── useSettingsStore.ts  # 提供商管理状态（核心状态）
├── pages/               # 页面级组件
│   └── Settings.tsx     # 设置页面（提供商 CRUD）
└── components/          # 可复用组件
    └── settings/
        └── ProviderForm.tsx  # 提供商表单（react-hook-form）
```

**状态管理模式：**

使用 Zustand + Immer 中间件，所有状态更新都是不可变的。Store 分离为：
- 数据状态：`providers`, `activeProviderId`, `loading`, `error`
- 异步 Actions：`fetchProviders()`, `saveProvider()`, `deleteProvider()`, 等等
- 便捷 Hooks：`useProviders()`, `useActiveProvider()`, `useProviderActions()`

### 数据流架构

```
用户操作 → React 组件
         ↓
   Zustand Action
         ↓
   Tauri invoke(cmd_xxx)
         ↓
   Rust Command Handler
         ↓
   LLMClientManager / Repository
         ↓
   Keyring / SQLite / HTTP
         ↓
   返回结果 → 前端更新状态
```

## 关键技术点

### 1. Tauri 命令接口规范

所有暴露给前端的命令都在 `commands.rs` 中定义，遵循以下模式：

```rust
#[tauri::command]
pub async fn cmd_xxx(
    manager: State<'_, LLMClientManager>,  // 注入状态
    param: Type,                            // 请求参数
) -> Result<Response, CommandError> {
    // 业务逻辑
    Ok(result)
}
```

**重要**：命令必须在 `lib.rs` 的 `invoke_handler!` 宏中注册，否则前端无法调用。

### 2. 序列化命名约定

- **Rust → 前端**：使用 `#[serde(rename_all = "camelCase")]` 确保字段名使用驼峰命名
- **前端 → Rust**：同样使用 camelCase，serde 会自动转换为 Rust 的 snake_case

### 3. 敏感信息处理

- **API Key 传输**：前端仅在保存时发送明文，Rust 立即存入 keyring
- **掩码显示**：`get_providers` 返回的 `api_key_mask` 仅显示前 8 个字符（如 `sk-xxxx1234`）
- **类型安全**：使用 `secrecy::SecretString` 包装密钥，防止意外日志泄露

### 4. 多厂商适配器模式

每个提供商实现 `LLMService` trait：

```rust
#[async_trait]
pub trait LLMService {
    async fn chat_completion(&self, messages: Vec<Message>, params: ModelParams)
        -> Result<ChatCompletionResponse>;
    async fn test_connection(&self) -> Result<TestConnectionResult>;
}
```

扩展新厂商只需：
1. 在 `database/models.rs` 添加 `ApiProviderType` 枚举值
2. 在 `llm/providers/` 创建新文件实现 `LLMService`
3. 在 `llm/manager.rs` 的工厂方法中添加分支
4. 前端 `useSettingsStore.ts` 同步添加枚举值

### 5. 调试模式

前端和后端都支持调试模式开关：

- **前端**：`const DEBUG = import.meta.env.DEV;` 配合 `debugLog()` 函数
- **后端**：`#[cfg(debug_assertions)]` 条件编译，仅在开发模式输出日志

## 数据库 Schema

核心表 `api_providers` 结构：

```sql
CREATE TABLE api_providers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    provider_type TEXT NOT NULL,  -- 'openai' | 'anthropic' | 'ollama' | 'xai'
    name TEXT NOT NULL,
    base_url TEXT NOT NULL,
    api_key_ref TEXT,             -- keyring 引用: "provider_{id}"
    model TEXT,
    config_json TEXT,             -- JSON 扩展配置
    is_active INTEGER DEFAULT 0,  -- 0 或 1，触发器确保唯一性
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER ensure_single_active
AFTER UPDATE OF is_active ON api_providers
WHEN NEW.is_active = 1
BEGIN
    UPDATE api_providers SET is_active = 0
    WHERE id != NEW.id AND is_active = 1;
END;
```

数据库文件位于：
- **Windows**: `%APPDATA%\prism-forge\prism-forge.db`
- **macOS**: `~/Library/Application Support/prism-forge/prism-forge.db`
- **Linux**: `~/.config/prism-forge/prism-forge.db`

## 常见开发任务

### 添加新的 LLM 提供商

1. **Rust 后端**：
   - `database/models.rs`: 添加 `ApiProviderType::XYZ` 变体
   - `llm/providers/xyz.rs`: 实现 `LLMService` trait
   - `llm/manager.rs`: 在 `create_client()` 方法添加匹配分支
   - `commands.rs`: 更新 `SaveProviderRequest` 文档注释

2. **React 前端**：
   - `stores/useSettingsStore.ts`: 添加 `ApiProviderType.XYZ`
   - `components/settings/ProviderForm.tsx`: 在表单中添加选项

### 修改数据库 Schema

1. 在 `database/migrations.rs` 修改 SQL
2. 删除本地数据库文件让系统重建，或编写迁移逻辑
3. 同步更新 `database/models.rs` 中的结构体字段

### 调试 Tauri 命令

- **前端**：打开浏览器控制台，`invoke()` 调用失败会打印错误
- **后端**：在命令中使用 `eprintln!()` 输出到终端（仅在开发模式可见）
- **Rust 错误**：使用 `?` 操作符传播错误，最终转换为 `CommandError`

## 代码风格规范

- **注释语言**：统一使用中文注释（参考现有代码）
- **Rust 命名**：snake_case（函数/变量）、PascalCase（类型/枚举）、SCREAMING_SNAKE_CASE（常量）
- **TypeScript 命名**：camelCase（变量/函数）、PascalCase（类型/接口/枚举）
- **文件命名**：Rust 使用 snake_case.rs，TS/TSX 使用 PascalCase.tsx

## 安全注意事项

- **禁止**：在代码中硬编码 API Key 或其他密钥
- **禁止**：将 API Key 记录到日志或 console
- **必须**：使用 `secrecy::SecretString` 处理所有敏感数据
- **必须**：前端 API Key 输入框使用 `type="password"`

## 相关资源

- [Tauri 官方文档](https://tauri.app/v2/guides/)
- [Tauri Invoke API](https://tauri.app/v2/api/js/core/#functioninvoke)
- [async-openai 文档](https://github.com/64bit/async-openai)
- [keyring crate 文档](https://docs.rs/keyring/)
- [Zustand 文档](https://docs.pmnd.rs/zustand/getting-started/introduction)
