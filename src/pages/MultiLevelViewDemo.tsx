/**
 * 多级日志读取功能演示页面
 *
 * 展示如何使用 MultiLevelViewSelector 组件和相关 Hooks
 */

import { MultiLevelViewSelector, MultiLevelViewTabs } from '@/components/MultiLevelViewSelector';
import { useViewLevelManager, useSessionContent, useExportSessionByLevel } from '@/hooks/useViewLevel';
import { cn } from '@/lib/utils';

// 模拟会话 ID（实际使用时从路由或 props 获取）
const DEMO_SESSION_ID = 'demo-session-123';

export function MultiLevelViewDemo() {
  // 使用视图等级管理 hook
  const { currentViewLevel, changeViewLevel, isSaving } = useViewLevelManager(DEMO_SESSION_ID);

  // 加载会话内容
  const { messages, qaPairs, isLoading: contentLoading, isQAPairsMode } = useSessionContent(
    DEMO_SESSION_ID,
    currentViewLevel
  );

  // 导出功能
  const exportMutation = useExportSessionByLevel();

  const handleExport = async (format: 'markdown' | 'json') => {
    try {
      const content = await exportMutation.mutateAsync({
        sessionId: DEMO_SESSION_ID,
        viewLevel: currentViewLevel,
        format,
      });

      // 创建下载链接
      const blob = new Blob([content], { type: 'text/plain;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `session-${currentViewLevel}.${format === 'markdown' ? 'md' : 'json'}`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);

      alert(`导出成功！格式: ${format}`);
    } catch (err) {
      const error = err instanceof Error ? err.message : String(err);
      console.error('导出失败:', error);
      alert(`导出失败: ${error}`);
    }
  };

  return (
    <div className="container mx-auto p-6 max-w-6xl">
      {/* 页面标题 */}
      <div className="mb-6">
        <h1 className="text-3xl font-bold mb-2">多级日志读取功能演示</h1>
        <p className="text-muted-foreground">
          展示如何使用 ViewLevel 组件来过滤和显示会话消息
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* 左侧：视图等级选择器 */}
        <div className="lg:col-span-1">
          <div className="space-y-4">
            {/* 完整选择器 */}
            <div className="bg-card p-4 rounded-lg border">
              <h2 className="text-lg font-semibold mb-4">视图等级选择器</h2>
              <MultiLevelViewSelector
                value={currentViewLevel}
                onChange={changeViewLevel}
                loading={isSaving}
                showExport
                onExport={handleExport}
              />
            </div>

            {/* 快捷按钮组 */}
            <div className="bg-card p-4 rounded-lg border">
              <h2 className="text-lg font-semibold mb-4">快捷切换</h2>
              <MultiLevelViewTabs
                value={currentViewLevel}
                onChange={changeViewLevel}
              />
            </div>

            {/* 当前状态 */}
            <div className="bg-card p-4 rounded-lg border">
              <h2 className="text-lg font-semibold mb-4">当前状态</h2>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">当前视图:</span>
                  <span className="font-medium">{currentViewLevel}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">消息数量:</span>
                  <span className="font-medium">
                    {isQAPairsMode ? qaPairs?.length || 0 : messages?.length || 0}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">加载状态:</span>
                  <span className={cn(
                    "font-medium",
                    contentLoading ? "text-yellow-600" : "text-green-600"
                  )}>
                    {contentLoading ? '加载中...' : '已完成'}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* 右侧：内容展示区域 */}
        <div className="lg:col-span-2">
          <div className="bg-card p-4 rounded-lg border min-h-[500px]">
            <h2 className="text-lg font-semibold mb-4">
              {isQAPairsMode ? '问答对列表' : '消息列表'}
            </h2>

            {contentLoading ? (
              <div className="flex items-center justify-center h-96">
                <div className="text-center">
                  <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary mx-auto mb-4"></div>
                  <p className="text-muted-foreground">加载中...</p>
                </div>
              </div>
            ) : isQAPairsMode ? (
              // 问答对视图
              <div className="space-y-4">
                {qaPairs && qaPairs.length > 0 ? (
                  qaPairs.map((pair, index) => (
                    <div key={index} className="p-4 border rounded-lg bg-background">
                      {/* 问题 */}
                      <div className="mb-3">
                        <div className="flex items-center gap-2 mb-2">
                          <span className="text-lg">👤</span>
                          <span className="font-medium">用户问题 #{index + 1}</span>
                        </div>
                        <div className="ml-8 p-3 bg-muted rounded">
                          <p className="text-sm">{pair.question.summary || '无内容'}</p>
                          <p className="text-xs text-muted-foreground mt-2">{pair.question.timestamp}</p>
                        </div>
                      </div>

                      {/* 答案 */}
                      {pair.answer && (
                        <div className="ml-4 border-l-2 pl-4">
                          <div className="flex items-center gap-2 mb-2">
                            <span className="text-lg">🤖</span>
                            <span className="font-medium">助手回复</span>
                          </div>
                          <div className="ml-8 p-3 bg-muted rounded">
                            <p className="text-sm">{pair.answer.summary || '无内容'}</p>
                            <p className="text-xs text-muted-foreground mt-2">{pair.answer.timestamp}</p>
                          </div>
                        </div>
                      )}
                    </div>
                  ))
                ) : (
                  <div className="text-center py-12 text-muted-foreground">
                    <p>暂无问答对数据</p>
                    <p className="text-sm mt-2">请确保会话文件存在并包含有效数据</p>
                  </div>
                )}
              </div>
            ) : (
              // 消息列表视图
              <div className="space-y-3">
                {messages && messages.length > 0 ? (
                  messages.map((msg) => (
                    <div key={msg.uuid} className="p-3 border rounded-lg bg-background hover:bg-accent transition-colors">
                      <div className="flex items-start gap-3">
                        {/* 角色图标 */}
                        <span className="text-xl shrink-0">
                          {msg.msgType === 'user' && '👤'}
                          {msg.msgType === 'assistant' && '🤖'}
                          {msg.msgType === 'thinking' && '💭'}
                          {msg.msgType !== 'user' && msg.msgType !== 'assistant' && msg.msgType !== 'thinking' && '📝'}
                        </span>

                        {/* 消息内容 */}
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2 mb-1">
                            <span className="font-medium text-sm capitalize">{msg.msgType}</span>
                            <span className="text-xs text-muted-foreground">{msg.timestamp}</span>
                          </div>
                          <p className="text-sm">{msg.summary || '无内容'}</p>
                          {msg.parentUuid && (
                            <p className="text-xs text-muted-foreground mt-1">
                              父消息: {msg.parentUuid.slice(0, 8)}...
                            </p>
                          )}
                        </div>
                      </div>
                    </div>
                  ))
                ) : (
                  <div className="text-center py-12 text-muted-foreground">
                    <p>暂无消息数据</p>
                    <p className="text-sm mt-2">请确保会话文件存在并包含有效数据</p>
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* 使用说明 */}
      <div className="mt-8 p-4 bg-muted rounded-lg">
        <h2 className="text-lg font-semibold mb-2">使用说明</h2>
        <ul className="list-disc list-inside space-y-1 text-sm text-muted-foreground">
          <li>点击左侧的视图等级选项可以切换不同的消息过滤级别</li>
          <li>视图等级偏好会自动保存到数据库，下次访问时会自动加载</li>
          <li>点击"Markdown"或"JSON"按钮可以导出当前视图的会话内容</li>
          <li>QA Pairs 模式会自动提取用户问题和最终答案，忽略中间的思考过程</li>
          <li>所有操作都支持中英文国际化</li>
        </ul>
      </div>
    </div>
  );
}
