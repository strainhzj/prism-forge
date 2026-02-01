# 组件库文档

本文档记录 PrismForge 项目中所有可复用的业务组件。

## 目录结构
- [设置相关组件](#设置相关组件)
- [项目相关组件](#项目相关组件)
- [会话相关组件](#会话相关组件)
- [提示词相关组件](#提示词相关组件)
- [通用业务组件](#通用业务组件)

---

## 设置相关组件

### ProviderForm
- **路径**：`src/components/settings/ProviderForm.tsx`
- **用途**：API 提供商配置表单，用于创建或编辑 API 提供商
- **主要功能**：
  - 动态表单字段（根据 provider_type 显示不同字段）
  - 表单验证（使用 react-hook-form）
  - 国际化支持（useTranslation）
- **Props**：
  - `provider?: ProviderResponse | null` - 编辑模式的提供商数据
  - `onSubmit: (data: SaveProviderRequest) => Promise<void>` - 表单提交回调
- **正在使用的场景**：
  - `src/pages/SettingsPage.tsx` - 设置页面中的提供商创建/编辑对话框

### ProviderSettings
- **路径**：`src/components/settings/ProviderSettings.tsx`
- **用途**：提供商列表和管理界面
- **主要功能**：
  - 显示所有已配置的 API 提供商
  - 支持编辑、删除、切换活跃状态
  - 支持测试 API 连接
- **Props**：
  - `providers: ProviderResponse[]` - 提供商列表
  - `onEdit: (provider: ProviderResponse) => void` - 编辑回调
  - `onDelete: (id: number) => void` - 删除回调
  - `onToggleActive: (id: number) => void` - 切换活跃状态回调
- **正在使用的场景**：
  - `src/pages/SettingsPage.tsx` - 设置页面的提供商管理标签页

### VectorSettings
- **路径**：`src/components/settings/VectorSettings.tsx`
- **用途**：向量数据库配置管理
- **主要功能**：
  - 向量数据库连接配置
  - 集合管理
- **正在使用的场景**：
  - `src/pages/SettingsPage.tsx` - 设置页面的向量配置标签页

### FilterConfigSettings
- **路径**：`src/components/FilterConfigSettings.tsx`
- **用途**：日志过滤配置管理
- **主要功能**：
  - 过滤规则的查看和管理
  - 支持启用/禁用过滤规则
  - 支持重新加载配置
- **主要类型**：
  - `FilterRule` - 过滤规则（包含名称、启用状态、匹配类型、模式）
  - `MatchType` - 匹配类型（Contains/Regex/Exact）
- **正在使用的场景**：
  - `src/pages/SettingsPage.tsx` - 设置页面的过滤配置标签页

### OptimizerSettings
- **路径**：`src/components/OptimizerSettings.tsx`
- **用途**：优化器配置管理
- **主要功能**：
  - 优化器参数配置
  - 提示词模板管理
- **正在使用的场景**：
  - 设置页面的优化器配置标签页

---

## 项目相关组件

### ProjectCard
- **路径**：`src/components/project/ProjectCard.tsx`
- **用途**：项目卡片组件，显示当前选中项目或引导用户选择项目
- **主要功能**：
  - 显示项目路径和名称
  - 支持切换项目
  - 支持深浅色模式自适应
- **Props**：
  - `onConfirm?: (project: any, sessionFile: string | null) => void` - 确认选择回调
  - `onAlert?: (type: AlertType, message: string) => void` - 显示全局 Alert
  - `className?: string` - 自定义类名
- **正在使用的场景**：
  - `src/App.tsx` - 主页面显示当前项目卡片

### ProjectSwitcher
- **路径**：`src/components/project/ProjectSwitcher.tsx`
- **用途**：项目切换器组件
- **主要功能**：
  - 显示项目列表
  - 支持选择和切换项目
- **正在使用的场景**：
  - `src/components/project/ProjectCard.tsx` - 项目卡片内的项目切换功能

### ProjectSwitcherDialog
- **路径**：`src/components/project/ProjectSwitcherDialog.tsx`
- **用途**：项目切换对话框
- **主要功能**：
  - 弹窗形式显示项目列表
  - 支持创建新项目
- **正在使用的场景**：
  - `src/components/project/ProjectCard.tsx` - 项目卡片内的项目切换对话框

---

## 会话相关组件

### ProjectSidebar
- **路径**：`src/components/ProjectSidebar.tsx`
- **用途**：显示项目分组列表，支持折叠/展开，支持手动管理监控目录
- **主要功能**：
  - 显示监控目录列表
  - 支持添加/删除监控目录
  - 支持折叠/展开目录
- **Props**：
  - `onDirectorySelect?: (directoryPath: string, directoryName: string) => void` - 目录选择回调
  - `selectedDirectory?: string` - 当前选中的目录路径
  - `className?: string` - 自定义类名
- **正在使用的场景**：
  - `src/pages/SessionsPage.tsx` - 会话管理页面的侧边栏

### SessionFileList
- **路径**：`src/components/SessionFileList.tsx`
- **用途**：显示指定监控目录下的会话文件列表（按修改时间倒序）
- **主要功能**：
  - 懒加载会话文件
  - 支持点击查看详情
  - 支持多选批量操作
  - 支持智能提取会话显示名称
- **主要类型**：
  - `SessionFileInfo` - 会话文件信息（包含 session_id、file_path、file_size、modified_time、projectPath）
  - `SessionFileType` - 会话文件类型（main/agent/unknown）
- **Props**：
  - `directoryPath: string` - 监控目录路径
  - `onSessionSelect: (sessionInfo: SessionFileInfo) => void` - 会话选择回调
  - `onSelectionChange?: (selectedSessions: SessionFileInfo[]) => void` - 多选变化回调
- **正在使用的场景**：
  - `src/pages/SessionsPage.tsx` - 会话管理页面的会话文件列表

### SessionContentView
- **路径**：`src/components/SessionContentView.tsx`
- **主要功能**：
  - 按照首页 Session Log 的形式显示会话内容
  - 集成多级日志读取功能
  - 支持导出会话内容
  - 支持刷新和返回列表
- **Props**：
  - `sessionInfo: SessionFileInfo` - 会话文件信息
  - `onBack: () => void` - 返回列表回调
  - `className?: string` - 自定义类名
- **正在使用的场景**：
  - `src/pages/SessionsPage.tsx` - 会话管理页面的会话内容视图

### MessageCard
- **路径**：`src/components/session/MessageCard.tsx`
- **用途**：显示单条消息的卡片，包含头像、角色标签、内容和时间戳
- **主要功能**：
  - 支持深浅色主题
  - 支持内容长度限制
  - 显示消息角色（user/assistant/system）
- **Props**：
  - `role: string` - 消息角色
  - `content: string` - 消息内容
  - `timestamp?: string` - 时间戳（ISO 8601 格式）
  - `showAvatar?: boolean` - 是否显示头像
  - `maxContentLength?: number` - 最大内容长度（超过则截断）
  - `className?: string` - 自定义类名
- **正在使用的场景**：
  - `src/components/session/TimelineMessageList.tsx` - 时间线消息列表中的消息卡片

### TimelineMessageList
- **路径**：`src/components/session/TimelineMessageList.tsx`
- **用途**：时间线形式展示消息列表
- **主要功能**：
  - 支持多级视图切换
  - 支持导出消息列表
  - 支持刷新消息
- **正在使用的场景**：
  - `src/components/SessionContentView.tsx` - 会话内容视图的消息列表

### SessionStatsSidebar
- **路径**：`src/components/session/SessionStatsSidebar.tsx`
- **用途**：会话统计信息侧边栏
- **主要功能**：
  - 显示会话统计数据
  - 显示消息数量、字数等
- **正在使用的场景**：
  - 会话内容页面的统计信息展示

---

## 提示词相关组件

### PromptCard (根目录)
- **路径**：`src/components/PromptCard.tsx`
- **用途**：提示词卡片组件
- **主要功能**：
  - 显示提示词内容
  - 支持复制、编辑、删除操作
- **正在使用的场景**：
  - 提示词管理页面

### PromptBuilder
- **路径**：`src/components/PromptBuilder.tsx`
- **用途**：提示词构建器，用于生成增强的 AI 提示词
- **主要功能**：
  - 目标输入
  - 会话选择
  - 生成增强提示词
  - 预览和统计
  - 复制/保存功能
- **Props**：
  - `initialGoal?: string` - 初始目标
  - `onGenerated?: (result: EnhancedPrompt) => void` - 生成完成回调
  - `className?: string` - 自定义类名
- **正在使用的场景**：
  - 提示词实验室页面

### PromptLibrary
- **路径**：`src/components/PromptLibrary.tsx`
- **用途**：提示词库管理组件
- **主要功能**：
  - 三栏布局：Next Goals / AI Analysis / Meta Templates
  - 搜索、筛选、排序功能
  - 编辑、删除功能
- **主要类型**：
  - `PromptLibraryItem` - 提示词库项
  - `PromptCategory` - 提示词分类
  - `PromptLibraryFilters` - 过滤器
- **Props**：
  - `defaultCategory?: PromptCategory` - 初始选中的分类
  - `onSelectPrompt?: (prompt: Prompt) => void` - 选择提示词回调
  - `className?: string` - 自定义类名
- **正在使用的场景**：
  - `src/pages/PromptsPage.tsx` - 提示词管理页面

### PromptCard (prompts 目录)
- **路径**：`src/components/prompts/PromptCard.tsx`
- **用途**：提示词卡片组件（PromptsPage 专用版本）
- **主要功能**：
  - 显示提示词信息
  - 支持编辑和删除操作
  - 支持选择操作
- **正在使用的场景**：
  - `src/pages/PromptsPage.tsx` - 提示词管理页面的提示词卡片

### PromptForm
- **路径**：`src/components/prompts/PromptForm.tsx`
- **用途**：提示词表单组件
- **主要功能**：
  - 创建/编辑提示词
  - 表单验证
  - 国际化支持
- **正在使用的场景**：
  - `src/pages/PromptsPage.tsx` - 提示词管理页面的创建/编辑对话框

### PromptHistory
- **路径**：`src/components/prompt/PromptHistory.tsx`
- **用途**：提示词历史记录组件
- **主要功能**：
  - 显示历史提示词列表
  - 支持查看详情
- **正在使用的场景**：
  - 提示词相关页面的历史记录

### PromptHistoryDetail
- **路径**：`src/components/prompt/PromptHistoryDetail.tsx`
- **用途**：提示词历史详情组件
- **主要功能**：
  - 显示历史提示词的详细信息
  - 支持对比和恢复
- **正在使用的场景**：
  - 提示词历史记录的详情查看

---

## 通用业务组件

### ThemeToggle
- **路径**：`src/components/ThemeToggle.tsx`
- **用途**：主题切换按钮
- **主要功能**：
  - 切换深色/浅色模式
  - 显示当前主题图标
- **正在使用的场景**：
  - `src/pages/SessionsPage.tsx` - 会话管理页面的主题切换
  - `src/pages/SettingsPage.tsx` - 设置页面的主题切换
  - `src/App.tsx` - 主页面的主题切换

### LanguageSwitcher
- **路径**：`src/components/LanguageSwitcher.tsx`
- **用途**：语言切换组件
- **主要功能**：
  - 切换中英文语言
  - 显示当前语言
- **正在使用的场景**：
  - 导航栏或页面顶部的语言切换

### MultiLevelViewSelector
- **路径**：`src/components/MultiLevelViewSelector.tsx`
- **用途**：多级日志读取选择器（Full/Conversation/QAPairs/AssistantOnly/UserOnly）
- **主要功能**：
  - 切换视图等级
  - 支持导出功能
  - 支持禁用和加载状态
- **主要类型**：
  - `ViewLevel` - 视图等级类型
- **Props**：
  - `value: ViewLevel` - 当前选中的视图等级
  - `onChange: (level: ViewLevel) => void` - 视图等级变更回调
  - `showExport?: boolean` - 是否显示导出按钮
  - `onExport?: (format: 'markdown' | 'json') => void` - 导出按钮点击回调
  - `disabled?: boolean` - 是否禁用
  - `loading?: boolean` - 加载状态
  - `className?: string` - 自定义类名
- **正在使用的场景**：
  - `src/components/SessionContentView.tsx` - 会话内容视图的视图等级选择

### CodeBlock
- **路径**：`src/components/CodeBlock.tsx`
- **用途**：代码块展示组件
- **主要功能**：
  - 语法高亮
  - 支持多种编程语言
  - 支持复制代码
- **正在使用的场景**：
  - 消息内容中的代码块展示

### CodeViewer
- **路径**：`src/components/CodeViewer.tsx`
- **用途**：代码查看器组件
- **主要功能**：
  - 显示代码内容
  - 支持语法高亮
  - 支持行号显示
- **正在使用的场景**：
  - 需要查看代码内容的场景

### DiffViewer
- **路径**：`src/components/DiffViewer.tsx`
- **用途**：差异对比查看器
- **主要功能**：
  - 显示文本差异
  - 支持并排和统一视图
- **正在使用的场景**：
  - 提示词对比、版本对比等场景

### MonacoEditor
- **路径**：`src/components/MonacoEditor.tsx`
- **用途**：Monaco 编辑器组件（VS Code 编辑器核心）
- **主要功能**：
  - 代码编辑
  - 语法高亮
  - 自动补全
- **正在使用的场景**：
  - 需要编辑代码或文本的场景

### MessageTree
- **路径**：`src/components/MessageTree.tsx`
- **用途**：消息树组件
- **主要功能**：
  - 树形展示消息结构
  - 支持折叠/展开
- **正在使用的场景**：
  - 复杂会话结构的展示

### MessageNode
- **路径**：`src/components/MessageNode.tsx`
- **用途**：消息节点组件
- **主要功能**：
  - 显示单个消息节点
  - 支持嵌套消息
- **正在使用的场景**：
  - `src/components/MessageTree.tsx` - 消息树中的节点

### RefreshIndicator
- **路径**：`src/components/RefreshIndicator.tsx`
- **用途**：刷新指示器组件
- **主要功能**：
  - 显示刷新状态
  - 支持自动刷新
- **正在使用的场景**：
  - 需要显示刷新状态的页面

### CodeBlockRenderer
- **路径**：`src/components/CodeBlockRenderer.tsx`
- **用途**：代码块渲染器
- **主要功能**：
  - 渲染 Markdown 中的代码块
  - 语法高亮
- **正在使用的场景**：
  - 消息内容中的代码块渲染

---

## 导航相关组件

### SideNav
- **路径**：`src/components/navigation/SideNav.tsx`
- **用途**：侧边导航栏
- **主要功能**：
  - 显示导航菜单
  - 支持路由跳转
  - 支持图标和文本
- **正在使用的场景**：
  - 应用的主导航栏

### NavItem
- **路径**：`src/components/navigation/NavItem.tsx`
- **用途**：导航项组件
- **主要功能**：
  - 显示单个导航项
  - 支持激活状态
  - 支持图标和徽章
- **正在使用的场景**：
  - `src/components/navigation/SideNav.tsx` - 侧边导航栏中的导航项

### TimelineSidebar
- **路径**：`src/components/timeline/TimelineSidebar.tsx`
- **用途**：时间线侧边栏
- **主要功能**：
  - 显示时间线
  - 支持快速跳转
- **正在使用的场景**：
  - 会话内容视图的时间线导航

---

## 使用建议

1. **实现新功能前**：先查阅本文档，确认是否已有可复用的组件
2. **组件复用**：如果功能 >50% 相似，优先考虑扩展现有组件
3. **新建组件**：如果需要新组件，先与项目负责人确认
4. **更新文档**：新增或修改组件后，及时更新本文档

---

## 维护说明

- 本文档应与实际代码保持同步
- 每次新增业务组件时，应更新本文档
- 每次修改组件接口时，应更新本文档中的 Props 说明
- 每次添加新的使用场景时，应更新"正在使用的场景"部分
