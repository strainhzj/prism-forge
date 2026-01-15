# Git 提交信息建议

## 标题
```
feat: 实现完整的多级日志读取功能 (Full/Conversation/QAPairs/AssistantOnly/UserOnly)
```

## 详细描述
```
实现多级日志读取功能,允许用户以不同的粒度级别查看 Claude Code 会话消息。

## 功能特性

### 后端实现 (Rust)
- 实现了 ViewLevel 枚举和消息过滤逻辑
- 实现了 QA 配对提取算法 (基于 parentUuid 追踪)
- 添加了数据库迁移 v13 (view_level_preferences 表)
- 实现了 5 个 Tauri 命令:
  - cmd_get_messages_by_level
  - cmd_get_qa_pairs_by_level
  - cmd_save_view_level_preference
  - cmd_get_view_level_preference
  - cmd_export_session_by_level

### 前端实现 (TypeScript + React)
- 创建了完整的类型定义 (src/types/viewLevel.ts)
- 实现了 MultiLevelViewSelector 组件 (支持导出按钮和快捷按钮组)
- 创建了 API 封装层 (src/lib/view-level-api.ts)
- 实现了 React Query Hooks (src/hooks/useViewLevel.ts)
- 添加了完整的中英文国际化支持

## 技术亮点

1. **类型安全**: Rust 和 TypeScript 类型完全对应
2. **性能优化**: React Query 自动缓存,避免重复请求
3. **用户体验**: 偏好设置持久化到数据库
4. **国际化**: 完整的中英文支持
5. **错误处理**: 优雅的错误处理和降级策略

## 测试状态

- 后端单元测试: ✅ 10/10 通过
- Rust 编译: ✅ 成功 (75个警告,0错误)
- 前端编译: ✅ 成功 (0错误)
- 数据库迁移: ✅ v13 成功应用

## 文件清单

### 新建文件 (8个)
- src-tauri/src/parser/view_level.rs
- src/types/viewLevel.ts
- src/components/MultiLevelViewSelector.tsx
- src/lib/view-level-api.ts
- src/hooks/useViewLevel.ts
- MULTI_LEVEL_VIEW_USAGE.md
- MULTI_LEVEL_LOG_READING_SUMMARY.md

### 修改文件 (6个)
- src-tauri/src/parser/mod.rs
- src-tauri/src/database/migrations.rs
- src-tauri/src/database/repository.rs
- src-tauri/src/commands.rs
- src-tauri/src/lib.rs
- src/i18n/locales/zh/sessions.json
- src/i18n/locales/en/sessions.json

## 破坏性变更

无破坏性变更,所有新功能都是增量添加。

## 下一步

- 集成到现有的会话详情页面
- 添加端到端测试
- 性能优化 (流式处理大文件)
```

## 简短版本 (用于 Git)

```
feat: 实现完整的多级日志读取功能

实现了 5 级日志过滤视图 (Full/Conversation/QAPairs/AssistantOnly/UserOnly):
- 后端: ViewLevel 枚举、消息过滤器、QA 配对提取、数据库迁移 v13、5个 Tauri 命令
- 前端: UI 组件、API 封装、React Query Hooks、完整国际化支持
- 测试: 后端单元测试 10/10 通过,编译成功
- 文档: 使用示例和完整实现总结

技术亮点:
- 类型安全 (Rust ↔ TypeScript)
- 性能优化 (React Query 缓存)
- 用户体验 (偏好持久化)
- 国际化 (中英文)
```
