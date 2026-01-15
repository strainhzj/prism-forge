/**
 * SessionContentView 组件
 *
 * 按照首页 Session Log 的形式显示会话内容
 * 集成多级日志读取功能
 */

import { useTranslation } from 'react-i18next';
import { ChevronLeft, RefreshCw, Filter } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';
import { MultiLevelViewTabs } from '@/components/MultiLevelViewSelector';
import { useViewLevelManager, useSessionContent, useExportSessionByLevel } from '@/hooks/useViewLevel';

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[SessionContentView] ${action}`, ...args);
  }
}

// ==================== 类型定义 ====================

/**
 * 会话文件信息
 */
export interface SessionFileInfo {
  session_id: string;
  file_path: string;
  file_size: number;
  modified_time: string;
}

export interface SessionContentViewProps {
  /**
   * 会话文件信息
   */
  sessionInfo: SessionFileInfo;
  /**
   * 返回列表回调
   */
  onBack: () => void;
  /**
   * 自定义类名
   */
  className?: string;
}

/**
 * SessionContentView 组件
 *
 * @example
 * <SessionContentView
 *   sessionInfo={{
 *     session_id: 'xxx',
 *     file_path: '/path/to/file.jsonl',
 *     file_size: 12345,
 *     modified_time: '2025-01-09T12:34:56Z'
 *   }}
 *   onBack={() => console.log('back')}
 * />
 */
export function SessionContentView({
  sessionInfo,
  onBack,
  className,
}: SessionContentViewProps) {
  const { t } = useTranslation('sessions');

  // ===== 多级日志读取功能 =====
  // 使用视图等级管理 hook
  const {
    currentViewLevel,
    changeViewLevel,
    isSaving: viewLevelSaving
  } = useViewLevelManager(sessionInfo.session_id);

  // 加载会话内容（根据视图等级过滤）
  const {
    messages,
    qaPairs,
    isLoading: contentLoading,
    error: contentError,
    isQAPairsMode,
    refresh: refreshContent
  } = useSessionContent(sessionInfo.session_id, currentViewLevel, sessionInfo.file_path);

  // 导出功能
  const exportMutation = useExportSessionByLevel();

  const handleExport = async (format: 'markdown' | 'json') => {
    try {
      const content = await exportMutation.mutateAsync({
        sessionId: sessionInfo.session_id,
        viewLevel: currentViewLevel,
        format,
        filePath: sessionInfo.file_path,
      });

      // 创建下载链接
      const blob = new Blob([content], { type: 'text/plain;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${sessionInfo.session_id.slice(0, 8)}-${currentViewLevel}.${format === 'markdown' ? 'md' : 'json'}`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);

      debugLog('handleExport', '导出成功', format);
    } catch (err) {
      const error = err instanceof Error ? err.message : String(err);
      debugLog('handleExport', '导出失败', error);
      alert(`导出失败: ${error}`);
    }
  };

  return (
    <div className={cn('flex flex-col h-full', className)} style={{ backgroundColor: 'var(--color-bg-primary)' }}>
      {/* 头部 */}
      <div className="flex items-center gap-3 px-6 py-4 border-b" style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}>
        <Button
          variant="ghost"
          size="icon"
          onClick={onBack}
          className="shrink-0 hover:bg-[var(--color-app-secondary)]"
        >
          <ChevronLeft className="h-5 w-5" style={{ color: 'var(--color-text-primary)' }} />
        </Button>
        <div className="flex-1 min-w-0">
          <h2 className="text-lg font-semibold truncate" style={{ color: 'var(--color-text-primary)' }}>
            {t('detailView.title')}
          </h2>
          <p className="text-xs truncate" style={{ color: 'var(--color-text-secondary)' }}>
            {sessionInfo.session_id.slice(0, 8)}...
          </p>
        </div>
        <div className="flex items-center gap-2">
          {/* 刷新按钮 */}
          <Button
            variant="ghost"
            size="icon"
            onClick={() => {
              refreshContent();
            }}
            disabled={contentLoading}
            className="shrink-0 hover:bg-[var(--color-app-secondary)]"
            title={t('detailView.refresh')}
          >
            <RefreshCw className={cn('h-4 w-4', contentLoading && 'animate-spin')} style={{ color: 'var(--color-text-primary)' }} />
          </Button>

          {/* 导出按钮（下拉菜单） */}
          <div className="relative group">
            <Button
              variant="ghost"
              size="icon"
              className="shrink-0 hover:bg-[var(--color-app-secondary)]"
              title="导出"
            >
              <Filter className="h-4 w-4" style={{ color: 'var(--color-text-primary)' }} />
            </Button>
            {/* 下拉菜单 */}
            <div className="absolute right-0 top-full mt-1 hidden group-hover:block bg-card border rounded-md shadow-lg z-50" style={{ minWidth: '120px', backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}>
              <button
                onClick={() => handleExport('markdown')}
                disabled={exportMutation.isPending}
                className="block w-full text-left px-4 py-2 text-sm hover:bg-accent"
                style={{ color: 'var(--color-text-primary)' }}
              >
                Markdown
              </button>
              <button
                onClick={() => handleExport('json')}
                disabled={exportMutation.isPending}
                className="block w-full text-left px-4 py-2 text-sm hover:bg-accent"
                style={{ color: 'var(--color-text-primary)' }}
              >
                JSON
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* 视图等级选择器栏 */}
      <div className="px-6 py-3 border-b" style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}>
        <MultiLevelViewTabs
          value={currentViewLevel}
          onChange={changeViewLevel}
          disabled={viewLevelSaving || contentLoading}
        />
      </div>

      {/* 内容区域 */}
      <div className="flex-1 overflow-y-auto" style={{ backgroundColor: 'var(--color-app-result-bg)' }}>
        {contentLoading ? (
          // 加载骨架屏
          <div className="p-4 space-y-4">
            {[...Array(3)].map((_, i) => (
              <div key={i} className="border rounded-md p-4 space-y-2" style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}>
                <div className="flex items-center gap-2">
                  <Skeleton className="h-5 w-16" />
                  <Skeleton className="h-4 w-24" />
                </div>
                <Skeleton className="h-16 w-full" />
              </div>
            ))}
          </div>
        ) : contentError ? (
          // 错误状态
          <div className="flex flex-col items-center justify-center h-full text-center p-4">
            <p className="font-medium" style={{ color: 'var(--color-app-error-accent)' }}>{t('detailView.loadFailed')}</p>
            <p className="text-sm mt-2" style={{ color: 'var(--color-text-secondary)' }}>{String(contentError)}</p>
            <Button variant="outline" size="sm" onClick={() => { refreshContent(); }} className="mt-4">
              {t('buttons.retry')}
            </Button>
          </div>
        ) : isQAPairsMode ? (
          // QA Pairs 视图
          <div className="p-4 space-y-4">
            {qaPairs && qaPairs.length > 0 ? (
              qaPairs.map((pair, index) => (
                <div
                  key={index}
                  className="border rounded-lg p-4 space-y-4"
                  style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}
                >
                  {/* 问题 */}
                  <div>
                    <div className="flex items-center gap-2 mb-2">
                      <span className="text-lg">👤</span>
                      <span
                        className="text-xs font-semibold px-2 py-0.5 rounded text-white"
                        style={{ backgroundColor: 'var(--color-accent-warm)', boxShadow: '0 0 10px rgba(245, 158, 11, 0.4)' }}
                      >
                        {t('detailView.question')} #{index + 1}
                      </span>
                      <span className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
                        {pair.question.timestamp.split('T')[1]?.substring(0, 8) || pair.question.timestamp}
                      </span>
                    </div>
                    <div className="text-sm whitespace-pre-wrap break-words pl-8" style={{ color: 'var(--color-text-primary)' }}>
                      {pair.question.summary && pair.question.summary.length > 500
                        ? pair.question.summary.substring(0, 500) + '...'
                        : pair.question.summary || '无内容'}
                    </div>
                  </div>

                  {/* 答案 */}
                  {pair.answer && (
                    <div className="ml-4 border-l-2 pl-4" style={{ borderColor: 'rgba(37, 99, 235, 0.3)' }}>
                      <div className="flex items-center gap-2 mb-2">
                        <span className="text-lg">🤖</span>
                        <span
                          className="text-xs font-semibold px-2 py-0.5 rounded text-white"
                          style={{ backgroundColor: 'var(--color-accent-blue)', boxShadow: '0 0 10px rgba(37, 99, 235, 0.4)' }}
                        >
                          {t('detailView.answer')}
                        </span>
                        <span className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
                          {pair.answer.timestamp.split('T')[1]?.substring(0, 8) || pair.answer.timestamp}
                        </span>
                      </div>
                      <div className="text-sm whitespace-pre-wrap break-words pl-8" style={{ color: 'var(--color-text-primary)' }}>
                        {pair.answer.summary && pair.answer.summary.length > 500
                          ? pair.answer.summary.substring(0, 500) + '...'
                          : pair.answer.summary || '无内容'}
                      </div>
                    </div>
                  )}
                </div>
              ))
            ) : (
              // 空状态
              <div className="flex flex-col items-center justify-center h-full text-center p-4">
                <p className="font-medium" style={{ color: 'var(--color-text-primary)' }}>{t('detailView.noContent')}</p>
                <p className="text-sm mt-2" style={{ color: 'var(--color-text-secondary)' }}>
                  {t('detailView.noContentHint')}
                </p>
              </div>
            )}
          </div>
        ) : (
          // 消息列表视图 - 参考时间线日志的展示方式
          <div className="p-4 space-y-3">
            {messages && messages.length > 0 ? (
              messages.map((msg) => {
                const msgType = msg.msg_type || 'unknown';
                const isUser = msgType === 'user';
                const isAssistant = msgType === 'assistant';
                const displayContent = msg.summary || '无内容';
                const fullContent = msg.summary || '无内容';

                return (
                  <div
                    key={msg.uuid}
                    className="p-3 rounded-lg border transition-all hover:shadow-lg"
                    style={{
                      backgroundColor: 'var(--color-bg-primary)',
                      borderColor: 'var(--color-border-light)',
                    }}
                    onMouseEnter={(e) => {
                      const color = isUser ? '245, 158, 11' : '37, 99, 235';
                      e.currentTarget.style.boxShadow = `0 0 20px rgba(${color}, 0.2)`;
                      e.currentTarget.style.borderColor = `rgba(${color}, 0.3)`;
                    }}
                    onMouseLeave={(e) => {
                      e.currentTarget.style.boxShadow = 'none';
                      e.currentTarget.style.borderColor = 'var(--color-border-light)';
                    }}
                  >
                    {/* 顶部：类型图标 + 时间 */}
                    <div className="flex items-center gap-2 mb-2">
                      {/* 暖橙色/蓝色小点 */}
                      <div
                        className="w-2 h-2 rounded-full"
                        style={{
                          backgroundColor: isUser ? 'var(--color-accent-warm)' : 'var(--color-accent-blue)',
                          boxShadow: isUser
                            ? '0 0 8px rgba(245, 158, 11, 0.5)'
                            : '0 0 8px rgba(37, 99, 235, 0.5)',
                        }}
                      />
                      <span className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
                        {new Date(msg.timestamp).toLocaleTimeString('zh-CN', {
                          hour: '2-digit',
                          minute: '2-digit',
                          second: '2-digit',
                        })}
                      </span>
                    </div>

                    {/* 内容摘要 */}
                    <p
                      className="text-xs line-clamp-3"
                      style={{
                        color: 'var(--color-text-primary)',
                        display: '-webkit-box',
                        WebkitLineClamp: 3,
                        WebkitBoxOrient: 'vertical',
                        overflow: 'hidden',
                        lineHeight: '1.5',
                      }}
                    >
                      {displayContent}
                    </p>
                  </div>
                );
              })
            ) : (
              // 空状态
              <div className="flex flex-col items-center justify-center h-full text-center p-4">
                <p className="font-medium" style={{ color: 'var(--color-text-primary)' }}>{t('detailView.noContent')}</p>
                <p className="text-sm mt-2" style={{ color: 'var(--color-text-secondary)' }}>
                  {t('detailView.noContentHint')}
                </p>
              </div>
            )}
          </div>
        )}
      </div>

      {/* 底部统计信息 */}
      {!contentLoading && !contentError && (
        <div className="px-6 py-3 border-t text-xs" style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)', color: 'var(--color-text-secondary)' }}>
          {isQAPairsMode ? (
            t('detailView.qaPairsCount', { count: qaPairs?.length || 0 })
          ) : (
            t('detailView.messageCount', { count: messages?.length || 0 })
          )}
        </div>
      )}
    </div>
  );
}
