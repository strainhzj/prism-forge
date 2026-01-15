# 多级日志读取功能使用示例

本文档展示如何使用新实现的多级日志读取功能。

## 基础用法

### 1. 在会话详情页面集成视图等级选择器

```tsx
import { useState } from 'react';
import { MultiLevelViewSelector } from '@/components/MultiLevelViewSelector';
import { ViewLevel } from '@/types/viewLevel';
import { useViewLevelManager, useSessionContent } from '@/hooks/useViewLevel';

export function SessionDetailPage({ sessionId }: { sessionId: string }) {
  // 使用视图等级管理 hook
  const { currentViewLevel, changeViewLevel, isLoading } = useViewLevelManager(sessionId);

  // 加载会话内容
  const { messages, qaPairs, isLoading: contentLoading } = useSessionContent(
    sessionId,
    currentViewLevel
  );

  return (
    <div className="container mx-auto p-4">
      {/* 视图等级选择器 */}
      <MultiLevelViewSelector
        value={currentViewLevel}
        onChange={changeViewLevel}
        loading={isLoading}
        showExport
        onExport={(format) => handleExport(format)}
      />

      {/* 显示内容 */}
      {contentLoading ? (
        <div>加载中...</div>
      ) : currentViewLevel === ViewLevel.QAPairs ? (
        <QAPairsList qaPairs={qaPairs} />
      ) : (
        <MessagesList messages={messages} />
      )}
    </div>
  );
}
```

### 2. 使用快捷按钮组 (Tabs)

```tsx
import { MultiLevelViewTabs } from '@/components/MultiLevelViewTabs';

function SessionToolbar({ sessionId }: { sessionId: string }) {
  const { currentViewLevel, changeViewLevel } = useViewLevelManager(sessionId);

  return (
    <div className="flex items-center justify-between">
      <MultiLevelViewTabs
        value={currentViewLevel}
        onChange={changeViewLevel}
      />
    </div>
  );
}
```

### 3. 导出会话

```tsx
import { useExportSessionByLevel } from '@/hooks/useViewLevel';
import { useToast } from '@/hooks/use-toast';

function ExportButton({ sessionId, viewLevel }: { sessionId: string; viewLevel: ViewLevel }) {
  const exportMutation = useExportSessionByLevel();
  const { toast } = useToast();

  const handleExport = async (format: 'markdown' | 'json') => {
    try {
      const content = await exportMutation.mutateAsync({
        sessionId,
        viewLevel,
        format,
      });

      // 下载文件
      const blob = new Blob([content], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `session.${format === 'markdown' ? 'md' : 'json'}`;
      a.click();
      URL.revokeObjectURL(url);

      toast({
        title: '导出成功',
        description: `会话已导出为 ${format} 格式`,
      });
    } catch (error) {
      toast({
        title: '导出失败',
        description: error.message,
        variant: 'destructive',
      });
    }
  };

  return (
    <div className="flex gap-2">
      <button onClick={() => handleExport('markdown')} disabled={exportMutation.isPending}>
        导出 Markdown
      </button>
      <button onClick={() => handleExport('json')} disabled={exportMutation.isPending}>
        导出 JSON
      </button>
    </div>
  );
}
```

## 高级用法

### 1. 自定义消息渲染

```tsx
import { ViewLevel } from '@/types/viewLevel';

function MessageRenderer({ messages, viewLevel }: { messages: Message[]; viewLevel: ViewLevel }) {
  const getRoleIcon = (msgType: string) => {
    switch (msgType) {
      case 'user': return '👤';
      case 'assistant': return '🤖';
      case 'thinking': return '💭';
      default: return '📝';
    }
  };

  return (
    <div className="space-y-4">
      {messages.map((msg) => (
        <div key={msg.uuid} className="p-4 border rounded-lg">
          <div className="flex items-center gap-2 mb-2">
            <span>{getRoleIcon(msg.msg_type)}</span>
            <span className="font-medium">{msg.msg_type}</span>
            <span className="text-sm text-muted-foreground">{msg.timestamp}</span>
          </div>
          <p>{msg.summary || '无内容'}</p>
        </div>
      ))}
    </div>
  );
}
```

### 2. 问答对渲染

```tsx
import { type QAPair } from '@/types/viewLevel';

function QAPairsList({ qaPairs }: { qaPairs: QAPair[] }) {
  return (
    <div className="space-y-6">
      {qaPairs.map((pair, index) => (
        <div key={index} className="p-6 border rounded-lg bg-card">
          {/* 问题 */}
          <div className="mb-4">
            <div className="flex items-center gap-2 mb-2">
              <span className="text-lg">👤</span>
              <span className="font-medium">用户问题</span>
            </div>
            <p className="text-sm">{pair.question.summary}</p>
            <span className="text-xs text-muted-foreground">{pair.question.timestamp}</span>
          </div>

          {/* 答案 */}
          {pair.answer && (
            <div className="ml-4 border-l-2 pl-4">
              <div className="flex items-center gap-2 mb-2">
                <span className="text-lg">🤖</span>
                <span className="font-medium">助手回复</span>
              </div>
              <p className="text-sm">{pair.answer.summary}</p>
              <span className="text-xs text-muted-foreground">{pair.answer.timestamp}</span>
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
```

### 3. 结合状态管理

```tsx
import { useViewLevelManager } from '@/hooks/useViewLevel';
import { ViewLevel } from '@/types/viewLevel';

function SessionView({ sessionId }: { sessionId: string }) {
  const {
    currentViewLevel,
    changeViewLevel,
    isLoading,
    isSaving,
  } = useViewLevelManager(sessionId);

  // 切换视图等级时自动保存偏好
  const handleViewLevelChange = async (newLevel: ViewLevel) => {
    await changeViewLevel(newLevel);
    // 偏好已自动保存到数据库
  };

  return (
    <div>
      <MultiLevelViewSelector
        value={currentViewLevel}
        onChange={handleViewLevelChange}
        loading={isLoading || isSaving}
      />
    </div>
  );
}
```

## API 调用示例

### 直接使用 API 函数

```tsx
import {
  getMessagesByLevel,
  getQAPairsByLevel,
  saveViewLevelPreference,
  exportSessionByLevel,
} from '@/lib/view-level-api';
import { ViewLevel } from '@/types/viewLevel';

async function example() {
  const sessionId = 'session-123';

  // 获取消息
  const messages = await getMessagesByLevel(sessionId, ViewLevel.Conversation);
  console.log('消息列表:', messages);

  // 获取问答对
  const qaPairs = await getQAPairsByLevel(sessionId, ViewLevel.QAPairs);
  console.log('问答对:', qaPairs);

  // 保存偏好
  await saveViewLevelPreference(sessionId, ViewLevel.Full);

  // 导出会话
  const markdown = await exportSessionByLevel(sessionId, ViewLevel.Full, 'markdown');
  console.log('导出内容:', markdown);
}
```

## 完整示例

查看 `src/components/SessionDetailPage.tsx` 了解完整的使用示例（需要集成）。

## 注意事项

1. **类型安全**: 所有 API 都有完整的 TypeScript 类型定义
2. **错误处理**: 使用 try-catch 捕获错误，或使用 React Query 的错误状态
3. **缓存**: React Query 会自动缓存结果，避免重复请求
4. **国际化**: 所有文本都支持中英文切换
5. **主题**: 组件自动适配暗色/亮色主题

## 测试建议

1. 测试不同视图等级的切换
2. 测试问答对的正确提取
3. 测试导出功能（Markdown 和 JSON）
4. 测试偏好设置的持久化
5. 测试错误场景（文件不存在、会话不存在等）
