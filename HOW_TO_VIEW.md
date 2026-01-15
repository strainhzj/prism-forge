# 如何查看和测试多级日志读取功能

## 📋 方法概览

有3种方式可以查看和测试新实现的多级日志读取功能：

### 方式1: 使用演示页面（推荐）⭐

我已经创建了一个完整的演示页面，可以直接运行查看效果。

#### 步骤：

1. **启动开发服务器**
```bash
npm run dev
```

2. **添加演示页面路由**

   需要将演示页面添加到路由中。在 `src/App.tsx` 或相应的路由配置文件中添加：

```tsx
import { MultiLevelViewDemo } from '@/pages/MultiLevelViewDemo';

// 在路由配置中添加
<Route path="/demo/multi-level-view" element={<MultiLevelViewDemo />} />
```

3. **访问演示页面**

   在浏览器中打开：
```
http://localhost:1420/demo/multi-level-view
```

4. **测试功能**
   - ✅ 点击不同的视图等级选项（完整模式、会话模式等）
   - ✅ 观察消息列表的变化
   - ✅ 切换到 QA Pairs 模式查看问答对
   - ✅ 点击导出按钮测试导出功能
   - ✅ 刷新页面验证偏好设置是否被记住

### 方式2: 查看组件源码

直接查看实现的组件代码：

#### 核心组件

1. **UI 组件**
```
src/components/MultiLevelViewSelector.tsx
```
包含：
- `MultiLevelViewSelector` - 完整的选择器组件
- `MultiLevelViewTabs` - 横向快捷按钮组

2. **类型定义**
```
src/types/viewLevel.ts
```
包含：
- `ViewLevel` 枚举
- `Message`, `QAPair` 接口
- `VIEW_LEVEL_INFO` 映射表

3. **API 封装**
```
src/lib/view-level-api.ts
```
包含：
- 所有 Tauri 命令的封装函数
- 错误处理工具

4. **React Hooks**
```
src/hooks/useViewLevel.ts
```
包含：
- `useViewLevelManager` - 视图等级管理
- `useSessionContent` - 会话内容加载
- 其他查询和 mutation hooks

### 方式3: 查看使用示例文档

我已经创建了详细的使用示例文档：

```
MULTI_LEVEL_VIEW_USAGE.md
```

这个文档包含：
- ✅ 基础用法示例
- ✅ 高级用法示例
- ✅ 完整代码示例
- ✅ 注意事项和最佳实践

### 方式4: 查看实现总结文档

查看完整的功能实现总结：

```
MULTI_LEVEL_LOG_READING_SUMMARY.md
```

这个文档包含：
- ✅ 所有已完成的阶段
- ✅ 技术亮点
- ✅ 测试状态
- ✅ 文件清单

## 🎨 功能预览

### 视图等级选择器

演示页面会显示一个3列布局：

**左侧列：**
- 视图等级选择器（5个选项）
- 快捷切换按钮组
- 当前状态显示

**右侧列：**
- 消息列表或问答对列表
- 根据选择的视图等级动态更新

### 支持的视图等级

1. **📄 完整模式** - 显示所有消息
2. **💬 会话模式** - 仅用户、助手和思考消息
3. **❓ 问答对模式** - 提取问题和答案
4. **🤖 仅助手** - 仅助手回复
5. **👤 仅用户** - 仅用户输入

### 特色功能

- ✅ **自动保存**: 选择会自动保存到数据库
- ✅ **国际化**: 完整的中英文支持
- ✅ **导出功能**: 支持 Markdown 和 JSON 导出
- ✅ **响应式布局**: 支持桌面和移动设备
- ✅ **加载状态**: 优雅的加载和错误提示

## 🧪 测试建议

### 手动测试清单

1. **基础功能**
   - [ ] 切换不同的视图等级
   - [ ] 观察消息列表正确过滤
   - [ ] 刷新页面验证偏好保存

2. **QA Pairs 模式**
   - [ ] 切换到 QA Pairs 模式
   - [ ] 验证问答对正确提取
   - [ ] 检查问题和答案的配对

3. **导出功能**
   - [ ] 导出 Markdown 格式
   - [ ] 导出 JSON 格式
   - [ ] 验证导出内容正确

4. **国际化**
   - [ ] 切换到中文查看
   - [ ] 切换到英文查看
   - [ ] 验证所有文本正确翻译

5. **错误处理**
   - [ ] 测试无效会话 ID
   - [ ] 测试文件不存在场景
   - [ ] 验证错误提示友好

## 🔍 调试技巧

### 查看浏览器控制台

```javascript
// 1. 查看当前视图等级
console.log('Current ViewLevel:', currentViewLevel);

// 2. 查看加载的消息
console.log('Messages:', messages);

// 3. 查看问答对
console.log('QA Pairs:', qaPairs);
```

### 查看 Tauri 日志

后端日志会显示在终端中，可以查看：
- 数据库查询
- 文件解析
- 错误信息

## 📊 性能检查

使用 React Query DevTools 查看缓存状态：

```bash
# 安装 React Query DevTools（如果还没安装）
npm install @tanstack/react-query-devtools
```

然后在浏览器中查看：
- 查询缓存状态
- 请求时间
- 缓存命中率

## 🚀 下一步

演示页面验证通过后，可以：

1. **集成到现有页面**
   - 将 `MultiLevelViewSelector` 集成到 `SessionDetailPage`
   - 替换或增强现有的消息显示逻辑

2. **添加测试**
   - 编写单元测试
   - 编写集成测试
   - 添加 E2E 测试

3. **性能优化**
   - 实现虚拟滚动
   - 优化大文件处理
   - 添加性能监控

## 💡 常见问题

### Q: 为什么看不到消息数据？
A: 确保你有一个有效的会话 ID，并且该会话的 JSONL 文件存在。演示页面使用的是模拟 ID `demo-session-123`。

### Q: 如何集成到实际的会话详情页面？
A: 参考 `MULTI_LEVEL_VIEW_USAGE.md` 中的集成示例，主要步骤是：
1. 使用 `useViewLevelManager` hook 管理视图等级
2. 使用 `useSessionContent` hook 加载内容
3. 渲染 `MultiLevelViewSelector` 组件
4. 根据视图等级渲染消息或问答对

### Q: 导出功能不工作？
A: 确保：
1. 会话文件存在
2. Tauri 命令已正确注册
3. 浏览器允许下载文件

## 📞 获取帮助

如果遇到问题，请查看：
1. `MULTI_LEVEL_VIEW_USAGE.md` - 使用示例
2. `MULTI_LEVEL_LOG_READING_SUMMARY.md` - 实现总结
3. 浏览器控制台错误信息
4. Tauri 后端日志
