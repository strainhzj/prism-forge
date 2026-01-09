# Requirements Document

## Introduction

修复 Tauri 应用中前端调用后端命令时出现 "Command not found" 错误的问题。具体涉及 `get_monitored_directories` 和 `scan_sessions` 命令无法被前端正确调用的问题。

## Glossary

- **Tauri_Command**: 使用 `#[tauri::command]` 宏标注的 Rust 函数，可被前端 JavaScript 调用
- **Command_Registry**: Tauri 应用中注册所有可调用命令的机制
- **Frontend_Client**: React/TypeScript 前端应用
- **Backend_Service**: Rust Tauri 后端服务
- **Module_Dependency**: Rust 模块之间的依赖关系
- **Runtime_Error**: 应用运行时发生的错误

## Requirements

### Requirement 1: 命令可用性诊断

**User Story:** 作为开发者，我想要诊断为什么特定的 Tauri 命令无法被前端调用，以便快速定位问题根源。

#### Acceptance Criteria

1. WHEN 前端调用不存在的命令 THEN Backend_Service SHALL 返回详细的错误信息包含可用命令列表
2. WHEN Backend_Service 启动时 THEN 系统 SHALL 验证所有注册命令的依赖模块是否正确初始化
3. WHEN 命令注册失败时 THEN Backend_Service SHALL 记录详细的失败原因到日志
4. THE Diagnostic_Tool SHALL 提供命令注册状态的实时检查功能

### Requirement 2: 命令注册修复

**User Story:** 作为开发者，我想要确保所有定义的 Tauri 命令都能被正确注册和调用，以便前端功能正常工作。

#### Acceptance Criteria

1. THE Command_Registry SHALL 包含所有在 commands.rs 中定义的命令
2. WHEN Backend_Service 启动时 THEN 所有命令的依赖模块 SHALL 被正确初始化
3. WHEN 模块初始化失败时 THEN Backend_Service SHALL 提供降级方案或明确的错误提示
4. THE Backend_Service SHALL 在启动时验证每个命令的可调用性

### Requirement 3: 依赖模块初始化

**User Story:** 作为系统架构师，我想要确保所有模块依赖关系正确建立，以便避免运行时的模块加载失败。

#### Acceptance Criteria

1. THE Database_Module SHALL 在任何依赖它的命令注册前完成初始化
2. THE Monitor_Module SHALL 在 scan_sessions 命令注册前完成初始化
3. WHEN 关键依赖模块初始化失败时 THEN Backend_Service SHALL 拒绝启动并提供明确错误信息
4. THE Module_Loader SHALL 按正确的依赖顺序初始化所有模块

### Requirement 4: 错误处理和日志

**User Story:** 作为运维人员，我想要获得详细的错误信息和日志，以便快速诊断和解决命令调用问题。

#### Acceptance Criteria

1. WHEN 命令调用失败时 THEN Backend_Service SHALL 记录包含调用栈的详细错误日志
2. THE Logging_System SHALL 区分命令注册错误和命令执行错误
3. WHEN Frontend_Client 调用不存在的命令时 THEN Backend_Service SHALL 返回友好的错误消息
4. THE Error_Handler SHALL 提供命令调用失败的恢复建议

### Requirement 5: 命令可用性测试

**User Story:** 作为质量保证工程师，我想要自动化测试所有 Tauri 命令的可用性，以便在部署前发现命令注册问题。

#### Acceptance Criteria

1. THE Test_Suite SHALL 验证所有注册命令都可以被成功调用
2. WHEN 添加新命令时 THEN Test_Suite SHALL 自动包含该命令的可用性测试
3. THE Integration_Test SHALL 模拟前端调用验证命令的端到端功能
4. WHEN 命令依赖的模块发生变化时 THEN Test_Suite SHALL 验证命令仍然可用

### Requirement 6: 运行时命令验证

**User Story:** 作为开发者，我想要在应用运行时能够验证命令的状态，以便进行实时调试和监控。

#### Acceptance Criteria

1. THE Backend_Service SHALL 提供内部 API 来查询已注册命令的状态
2. WHEN 请求命令状态时 THEN 系统 SHALL 返回命令的注册状态、依赖状态和最后调用时间
3. THE Health_Check SHALL 包含所有关键命令的可用性检查
4. WHEN 命令状态异常时 THEN 系统 SHALL 触发相应的告警机制