/**
 * SessionContentView 组件
 *
 * 按照首页 Session Log 的形式显示会话内容
 * 集成多级日志读取功能
 */

import { useEffect, useState, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { ChevronLeft, RefreshCw, Download, ArrowUpDown, Repeat } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';
import { MultiLevelViewDropdown } from '@/components/MultiLevelViewSelector';
import { TimelineMessageList } from '@/components/session/TimelineMessageList';
import { useViewLevelManager, useSessionContent, useExportSessionByLevel } from '@/hooks/useViewLevel';
import type { MessageNode } from '@/types/message';

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

  // ===== 排序状态管理 =====
  const [sortOrder, setSortOrder] = useState<'desc' | 'asc'>('desc'); // 默认倒序

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
    isLoading: contentLoading,
    error: contentError,
    refresh: refreshContent,
    forceRefresh,
  } = useSessionContent(sessionInfo.session_id, currentViewLevel, sessionInfo.file_path);

  // ===== 清除缓存并重新加载 =====
  const [isClearingCache, setIsClearingCache] = useState(false);

  const handleClearCacheAndReload = async () => {
    setIsClearingCache(true);
    try {
      forceRefresh();
      debugLog('handleClearCacheAndReload', '缓存已清除，正在重新加载');
    } catch (error) {
      console.error('[SessionContentView] 清除缓存失败:', error);
    } finally {
      // 延迟重置加载状态，确保用户看到反馈
      setTimeout(() => {
        setIsClearingCache(false);
      }, 500);
    }
  };

  // ===== 排序后的消息列表 =====
  const sortedMessages = useMemo(() => {
    if (!messages || messages.length === 0) return messages;

    const sorted = [...messages].sort((a, b) => {
      const timeA = new Date(a.timestamp || 0).getTime();
      const timeB = new Date(b.timestamp || 0).getTime();
      return sortOrder === 'desc' ? timeB - timeA : timeA - timeB;
    });

    return sorted;
  }, [messages, sortOrder]);

  // 调试日志：检查返回的数据
  useEffect(() => {
    if (!messages || messages.length === 0) return;

    // 统计所有 msgType 的分布
    const typeCounts: Record<string, number> = {};
    messages.forEach(msg => {
      typeCounts[msg.msgType] = (typeCounts[msg.msgType] || 0) + 1;
    });

    // 显示前 5 条消息的详细信息
    const firstFive = messages.slice(0, 5).map(msg => ({
      uuid: msg.uuid.substring(0, 8),
      msgType: msg.msgType,
      summary_preview: msg.summary?.substring(0, 50) || '(empty)',
      timestamp: msg.timestamp?.substring(11, 19) || '(empty)',
    }));

    debugLog('useSessionContent', 'messages analysis:', {
      totalCount: messages.length,
      typeDistribution: typeCounts,
      firstFiveMessages: firstFive,
      viewLevel: currentViewLevel,
      filePath: sessionInfo.file_path,
    });

    // 检查是否有 "unknown" 或其他非标准的 msgType
    const nonStandardTypes = Object.keys(typeCounts).filter(
      t => !['user', 'assistant', 'system'].includes(t)
    );
    if (nonStandardTypes.length > 0) {
      console.warn('[SessionContentView] 发现非标准消息类型:', nonStandardTypes);

      // 🔍 临时调试：直接读取 JSONL 文件的前几行
      invoke<string>('read_file_first_lines', {
        path: sessionInfo.file_path,
        count: 5
      }).then(result => {
        console.log('[SessionContentView] JSONL 前 5 行:');
        const lines = result.split('\n');
        lines.forEach((line, i) => {
          if (line.trim()) {
            try {
              const parsed = JSON.parse(line);
              console.log(`  [${i}]`, parsed);
            } catch {
              console.log(`  [${i}] (解析失败):`, line.substring(0, 200));
            }
          }
        });
      }).catch(() => {
        console.log('[SessionContentView] read_file_first_lines 不可用，跳过');
      });
    }
  }, [messages, currentViewLevel, sessionInfo.file_path]);

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

          {/* 排序切换按钮 */}
          <Button
            variant="ghost"
            size="icon"
            onClick={() => {
              setSortOrder(prev => prev === 'desc' ? 'asc' : 'desc');
            }}
            disabled={contentLoading}
            className="shrink-0 hover:bg-[var(--color-app-secondary)]"
            title={t(`detailView.sortOrder.${sortOrder}`)}
          >
            <ArrowUpDown className="h-4 w-4" style={{ color: 'var(--color-text-primary)' }} />
          </Button>

          {/* 清除缓存并重新加载按钮 */}
          <Button
            variant="ghost"
            size="icon"
            onClick={handleClearCacheAndReload}
            disabled={contentLoading || isClearingCache}
            className="shrink-0 hover:bg-[var(--color-app-secondary)]"
            title={t('detailView.clearCache')}
          >
            <Repeat className={cn('h-4 w-4', isClearingCache && 'animate-spin')} style={{ color: 'var(--color-text-primary)' }} />
          </Button>

          {/* 导出按钮（下拉菜单） */}
          <div className="relative group">
            <Button
              variant="ghost"
              size="icon"
              className="shrink-0 hover:bg-[var(--color-app-secondary)]"
              title={t('viewLevel.export.title')}
            >
              <Download className="h-4 w-4" style={{ color: 'var(--color-text-primary)' }} />
            </Button>
            {/* 下拉菜单 */}
            <div className="absolute right-0 top-full mt-1 hidden group-hover:block bg-card border rounded-md shadow-lg z-50" style={{ minWidth: '120px', backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}>
              <button
                onClick={() => handleExport('markdown')}
                disabled={exportMutation.isPending}
                className="block w-full text-left px-4 py-2 text-sm hover:bg-accent"
                style={{ color: 'var(--color-text-primary)' }}
              >
                {t('viewLevel.export.formats.markdown')}
              </button>
              <button
                onClick={() => handleExport('json')}
                disabled={exportMutation.isPending}
                className="block w-full text-left px-4 py-2 text-sm hover:bg-accent"
                style={{ color: 'var(--color-text-primary)' }}
              >
                {t('viewLevel.export.formats.json')}
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* 视图等级选择器栏 */}
      <div className="px-6 py-3 border-b" style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}>
        <MultiLevelViewDropdown
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
        ) : (
          // 消息列表视图 - 使用 TimelineMessageList 组件
          <div className="p-4">
            {sortedMessages && sortedMessages.length > 0 ? (
              <TimelineMessageList
                messages={sortedMessages.map((msg): MessageNode => ({
                  id: msg.uuid,
                  parent_id: msg.parentUuid || null,
                  depth: 0,
                  // 使用 msgType 字段
                  role: msg.msgType || 'unknown',
                  type: msg.msgType || 'unknown',
                  content: msg.summary && msg.summary.length > 500
                    ? msg.summary.substring(0, 500) + '...'
                    : msg.summary || '无内容',
                  fullContent: msg.summary || undefined,
                  timestamp: msg.timestamp,
                  children: [],
                  thread_id: null,
                }))}
              />
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
          {t('detailView.messageCount', { count: sortedMessages?.length || 0 })}
        </div>
      )}
    </div>
  );
}
