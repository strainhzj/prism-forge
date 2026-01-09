# 命令注册系统操作文档

## 概述

本文档描述了 Tauri 应用命令注册系统的操作指南，包括日常维护、故障排查和性能监控。

## 系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                      应用启动流程                            │
├─────────────────────────────────────────────────────────────┤
│  1. StartupManager 执行启动验证                              │
│  2. ModuleInitializer 按依赖顺序初始化模块                    │
│  3. CommandRegistry 注册所有 Tauri 命令                      │
│  4. EnhancedErrorHandler 配置错误处理                        │
│  5. PerformanceMonitor 启动性能监控                          │
└─────────────────────────────────────────────────────────────┘
```

## 日常操作

### 1. 运行测试

```bash
# 进入后端目录
cd src-tauri

# 运行所有测试
cargo test --lib

# 运行特定模块测试
cargo test --lib startup::           # 启动验证测试
cargo test --lib command_wrapper::   # 命令包装器测试
cargo test --lib logging::           # 日志模块测试
cargo test --lib command_registry::  # 命令注册测试

# 运行属性测试（PBT）
cargo test --lib prop_
```

### 2. 启动应用

```bash
# 开发模式
cd src-tauri
cargo run

# 或使用 Tauri CLI
npm run tauri dev
```

### 3. 查看启动日志

应用启动时会输出验证信息：

```
[INFO] Starting application startup validation...
[INFO] Startup validation successful: 30 commands registered
```

## 故障排查

### 问题 1: 命令未找到

**症状**: 前端调用命令时返回 "command not found"

**排查步骤**:
1. 检查命令是否在 `lib.rs` 的 `invoke_handler` 中注册
2. 检查命令名称拼写（区分大小写）
3. 查看启动日志是否有警告

**解决方案**:
```rust
// 在 lib.rs 中添加命令
.invoke_handler(tauri::generate_handler![
    // ... 现有命令
    my_new_command,  // 添加新命令
])
```

### 问题 2: 模块初始化失败

**症状**: 应用启动失败或部分功能不可用

**排查步骤**:
1. 查看启动日志中的 `[ERROR]` 信息
2. 检查模块依赖是否正确配置
3. 验证数据库连接是否正常

**解决方案**:
```bash
# 检查数据库文件
ls -la ~/.prism-forge/

# 重置数据库（如需要）
rm ~/.prism-forge/prism-forge.db
```

### 问题 3: 性能问题

**症状**: 命令执行缓慢

**排查步骤**:
1. 使用 PerformanceMonitor 记录操作时间
2. 检查是否有阻塞操作
3. 查看数据库查询性能

**代码示例**:
```rust
use crate::logging::{PerformanceMonitor, Timer};

let monitor = PerformanceMonitor::new();
let timer = Timer::start("slow_operation");
// ... 执行操作
let duration = timer.stop(true);
eprintln!("Operation took: {:?}", duration);
```

## 添加新命令

### 步骤 1: 定义命令

在 `commands.rs` 中添加：

```rust
#[tauri::command]
pub async fn my_new_command(param: String) -> Result<String, String> {
    // 实现逻辑
    Ok(format!("Result: {}", param))
}
```

### 步骤 2: 注册命令

在 `lib.rs` 中添加到 `invoke_handler`：

```rust
.invoke_handler(tauri::generate_handler![
    // ... 现有命令
    my_new_command,
])
```

### 步骤 3: 更新命令定义

在 `startup.rs` 的 `get_all_command_definitions()` 中添加：

```rust
CommandDefinition {
    name: "my_new_command".to_string(),
    dependencies: vec!["database".to_string()], // 如有依赖
},
```

### 步骤 4: 编写测试

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_my_new_command() {
        // 测试逻辑
    }
}
```

## 监控和告警

### 性能阈值

| 操作类型 | 默认阈值 | 说明 |
|---------|---------|------|
| startup | 2s | 应用启动时间 |
| command_execution | 500ms | 命令执行时间 |
| module_init | 1s | 模块初始化时间 |
| database_query | 100ms | 数据库查询时间 |

### 自定义阈值

```rust
use crate::logging::PerformanceMonitor;

let monitor = PerformanceMonitor::new();
monitor.set_threshold("my_operation", Duration::from_millis(200));
```

## 文件结构

```
src-tauri/
├── src/
│   ├── lib.rs                 # 应用入口，命令注册
│   ├── startup.rs             # 启动验证逻辑
│   ├── command_wrapper.rs     # 命令跟踪工具
│   ├── logging.rs             # 日志和性能监控
│   └── command_registry/      # 命令注册系统
│       ├── mod.rs
│       ├── registry.rs        # 命令注册器
│       ├── initializer.rs     # 模块初始化器
│       ├── diagnostic.rs      # 诊断工具
│       ├── validator.rs       # 命令验证器
│       ├── error_handler.rs   # 错误处理器
│       ├── errors.rs          # 错误类型定义
│       ├── tests.rs           # 单元测试
│       └── integration_tests.rs # 集成测试
└── docs/
    ├── command-registration-guide.md  # 开发指南
    └── OPERATIONS.md                  # 本文档
```

## 联系方式

如有问题，请查阅：
- 开发指南: `src-tauri/docs/command-registration-guide.md`
- 设计文档: `.kiro/specs/fix-command-registration/design.md`
- 需求文档: `.kiro/specs/fix-command-registration/requirements.md`
