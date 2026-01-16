# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

PrismForge 是一个基于 Tauri 2 + React 18 的桌面应用程序，核心功能是 Claude 会话监控和 LLM 提示词优化。应用支持多厂商 LLM API（OpenAI、Anthropic、Ollama、xAI），并提供安全的 API Key 管理和统一的调用接口。

## 工作约束

 **在使用 Claude Code 开发本项目时，必须遵守以下约束：**

### 1. 交互模式（必读）

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

### 2. 代码复用优先

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

### 3. 问题澄清机制

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

### 4. 国际化与主题约束

🌍 **所有用户可见文本必须支持中英文切换**

- **强制要求**：

  1. 所有面向用户的文本必须使用 `useTranslation` hook
  2. 翻译键必须同时提供中文和英文版本
  3. 禁止硬编码中文或英文文本到组件中
  4. 动态内容（如供应商类型）必须通过翻译键映射实现

- **翻译文件结构**：

  ```
  src/i18n/locales/
  ├── zh/
  │   ├── common.json    # 通用文本（项目切换器、按钮等）
  │   ├── index.json     # 首页（timeline、项目操作）
  │   ├── navigation.json # 导航菜单
  │   ├── sessions.json   # 会话管理页面
  │   └── settings.json   # API设置（表单、验证、供应商类型）
  └── en/
      └── (相同结构)
  ```

- **使用示例**：

  ```typescript
  // ✅ 正确：使用翻译
  import { useTranslation } from 'react-i18next';
  
  const { t } = useTranslation('settings');
  <span>{t('form.providerType')}</span>
  <button>{t('buttons.save')}</button>
  
  // ❌ 错误：硬编码文本
  <span>提供商类型</span>
  <button>保存</button>
  ```

- **动态翻译处理**：

  - 供应商类型通过 `PROVIDER_TYPE_KEYS` 映射到翻译键
  - 第三方提供商通过 `THIRD_PARTY_PROVIDER_KEYS` 映射
  - 使用 `useMemo` 缓存动态生成的翻译内容

  ```typescript
  // 示例：供应商类型动态翻译
  const PROVIDER_TYPE_OPTIONS = useMemo(() => {
    return Object.entries(PROVIDER_DISPLAY_INFO).map(([key]) => {
      const providerTypeKey = PROVIDER_TYPE_KEYS[key as ApiProviderType];
      return {
        value: key as ApiProviderType,
        label: t(`providerTypes.${providerTypeKey}.label`),
        description: t(`providerTypes.${providerTypeKey}.description`),
      };
    });
  }, [t]);
  ```

- **翻译键命名规范**：

  - 使用点分路径：`namespace.category.key`
  - 命名空间：`common`, `index`, `navigation`, `sessions`, `settings`
  - 类别：`form`, `buttons`, `placeholders`, `validation`, `helpText`, `errors`
  - 键名：camelCase（如 `providerType`, `save`, `connectionFailed`）

🎨 **组件必须适配暗色/亮色主题**

- **主题系统**：应用支持暗色和亮色两种主题模式，通过 `useThemeStore` 管理

- **CSS 变量规范**：使用 Tailwind 的 `dark:` 前缀适配主题

  ```tsx
  // ✅ 正确：适配两种主题
  <div className="bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100">
  <button className="bg-primary dark:bg-primary-dark text-white">
  
  // ❌ 错误：仅适配单一主题
  <div className="bg-white text-gray-900">
  <button className="bg-blue-500 text-white">
  ```

- **常用颜色值映射**：

  | 元素           | 亮色主题                                  | 暗色主题                                            |
  | -------------- | ----------------------------------------- | --------------------------------------------------- |
  | **背景**       | `bg-white`                                | `dark:bg-gray-900`                                  |
  | **卡片背景**   | `bg-gray-50`                              | `dark:bg-gray-800`                                  |
  | **边框**       | `border-gray-200`                         | `dark:border-gray-700`                              |
  | **文本主色**   | `text-gray-900`                           | `dark:text-gray-100`                                |
  | **文本次要**   | `text-gray-600`                           | `dark:text-gray-400`                                |
  | **文本禁用**   | `text-gray-400`                           | `dark:text-gray-600`                                |
  | **主色调**     | `bg-orange-500`                           | `dark:bg-orange-600`                                |
  | **主色调悬停** | `hover:bg-orange-600`                     | `dark:hover:bg-orange-700`                          |
  | **输入框**     | `bg-white border-gray-300`                | `dark:bg-gray-800 dark:border-gray-600`             |
  | **输入框文本** | `text-gray-900 placeholder:text-gray-400` | `dark:text-gray-100 dark:placeholder:text-gray-500` |
  | **按钮主色**   | `bg-primary`                              | `dark:bg-primary-dark`                              |
  | **按钮次要**   | `bg-gray-200 text-gray-900`               | `dark:bg-gray-700 dark:text-gray-100`               |
  | **危险操作**   | `text-red-600 hover:text-red-700`         | `dark:text-red-400 dark:hover:text-red-300`         |
  | **成功提示**   | `text-green-600 bg-green-50`              | `dark:text-green-400 dark:bg-green-900/20`          |
  | **警告提示**   | `text-yellow-600 bg-yellow-50`            | `dark:text-yellow-400 dark:bg-yellow-900/20`        |
  | **错误提示**   | `text-red-600 bg-red-50`                  | `dark:text-red-400 dark:bg-red-900/20`              |

### 5.提交git前需要先与我确认

**总结**：

- 🤔 **思考** → 📋 **提出假设** → ✅ **等待确认** → 🔨 **开始编码**
- 🔍 **搜索** → ♻️ **复用优先** → 🆕 **必要时创建**
- ❓ **发现疑问** → 💬 **主动提问** → 📊 **提供选项** → 👍 **等待决策**

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

**核心设计原则：**

1. **适配器模式**：`LLMService` trait 抽象多厂商 API 接口（src-tauri/src/llm/interface.rs）
2. **工厂模式**：`LLMClientManager::create_client_from_provider()` 动态创建客户端实例
3. **仓库模式**：`ApiProviderRepository` 封装所有数据库操作
4. **单例模式**：`LLMClientManager` 通过 Tauri State 注入，全局唯一
5. **安全优先**：API Key 存储在 OS 凭据管理器，数据库仅保留引用

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

```typescript
// 前端调试日志示例（src/stores/useSettingsStore.ts）
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[SettingsStore] ${action}`, ...args);
  }
}
```

```rust
// 后端调试模式示例
#[cfg(debug_assertions)]
eprintln!("调试信息: {}", data);
```

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

- **注释语言**：统一使用中文注释（参考现有代码）
- **Rust 命名**：snake_case（函数/变量）、PascalCase（类型/枚举）、SCREAMING_SNAKE_CASE（常量）
- **TypeScript 命名**：camelCase（变量/函数）、PascalCase（类型/接口/枚举）
- **文件命名**：Rust 使用 snake_case.rs，TS/TSX 使用 PascalCase.tsx

### 调试模式使用规范

**前端调试（TypeScript）：**
```typescript
// 在模块顶部定义调试开关
const DEBUG = import.meta.env.DEV;

// 创建带模块前缀的调试日志函数
function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[ModuleName] ${action}`, ...args);
  }
}

// 使用示例
debugLog('fetchProviders', '开始获取提供商列表');
```

**后端调试（Rust）：**
```rust
// 使用条件编译，仅在开发模式输出
#[cfg(debug_assertions)]
eprintln!("调试信息: {:?}", data);

// 或者使用日志 crate（推荐用于生产环境）
use log::debug;
debug!("调试信息: {:?}", data);
```

**注意事项：**
- ⚠️ **生产环境**：前端调试日志会自动关闭（`import.meta.env.DEV` 为 false）
- ⚠️ **敏感信息**：禁止在日志中输出 API Key、密码等敏感数据
- ✅ **最佳实践**：使用结构化日志，包含时间戳、模块名、日志级别

### 序列化命名约定

**Rust ↔ 前端数据交换：**
- Rust 结构体使用 `#[serde(rename_all = "camelCase")]` 确保序列化为驼峰命名
- 前端发送 camelCase，serde 自动转换为 Rust 的 snake_case
- 日期时间使用 ISO 8601 格式字符串（`2025-01-09T12:34:56Z`）

```rust
// Rust 示例
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiProvider {
    pub api_key_ref: Option<String>,  // 序列化为 "apiKeyRef"
    pub is_active: bool,               // 序列化为 "isActive"
}
```

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
