/**
 * SessionContentView 组件
 *
 * 按照首页 Session Log 的形式显示会话内容
 * 集成多级日志读取功能
 */

import { useEffect, useState, useMemo, useRef, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import { ChevronLeft, RefreshCw, Download, ArrowUpDown, Code, RefreshCwOff } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';
import { MultiLevelViewDropdown } from '@/components/MultiLevelViewSelector';
import { TimelineMessageList } from '@/components/session/TimelineMessageList';
import { useViewLevelManager, useSessionContent, useExportSessionByLevel } from '@/hooks/useViewLevel';
import { useSessionMonitor } from '@/hooks/useSessionMonitor';
import type { MessageNode } from '@/types/message';
import { ViewLevel } from '@/types/viewLevel';

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

  // ===== 内容显示模式管理 =====
  const [contentDisplayMode, setContentDisplayMode] = useState<'raw' | 'extracted'>('extracted'); // 默认显示提取内容

  // ===== 导出下拉框状态管理 =====
  const [isExportDropdownOpen, setIsExportDropdownOpen] = useState(false);
  const [showForceReParseConfirm, setShowForceReParseConfirm] = useState(false);
  const exportDropdownRef = useRef<HTMLDivElement>(null);

  // 点击外部关闭下拉框
  const handleClickOutside = useCallback((event: MouseEvent) => {
    if (exportDropdownRef.current && !exportDropdownRef.current.contains(event.target as Node)) {
      setIsExportDropdownOpen(false);
    }
  }, []);

  useEffect(() => {
    if (isExportDropdownOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    } else {
      document.removeEventListener('mousedown', handleClickOutside);
    }
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [isExportDropdownOpen, handleClickOutside]);

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
  } = useSessionContent(sessionInfo.session_id, currentViewLevel, sessionInfo.file_path, false);

  const handleAutoRefresh = useCallback(async () => {
    try {
      await forceRefresh();
    } catch (error) {
      // 自动刷新失败，静默处理
    }
  }, [forceRefresh]);

  useSessionMonitor({
    debounceMs: 2000,
    onRefresh: handleAutoRefresh,
  });

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

      // 生成默认文件名
      const defaultFileName = `${sessionInfo.session_id.slice(0, 8)}-${currentViewLevel}.${format === 'markdown' ? 'md' : 'json'}`;

      // 使用 Tauri 原生文件保存对话框
      const filePath = await save({
        defaultPath: defaultFileName,
        filters: [
          {
            name: format === 'markdown' ? 'Markdown Files' : 'JSON Files',
            extensions: [format === 'markdown' ? 'md' : 'json'],
          },
          {
            name: 'All Files',
            extensions: ['*'],
          },
        ],
      });

      // 用户取消选择
      if (!filePath) {
        return;
      }

      // 写入文件（使用 Tauri 的 fs API）
      const { writeTextFile } = await import('@tauri-apps/plugin-fs');
      await writeTextFile(filePath, content);

      // 显示成功提示
      const formatLabel = t(`viewLevel.export.formats.${format}`);
      alert(`${t('viewLevel.export.success')}\n\n${formatLabel}: ${filePath}`);
    } catch (err) {
      const error = err instanceof Error ? err.message : String(err);
      alert(`${t('viewLevel.export.failed')}: ${error}`);
    }
  };

  // 强制重新解析处理函数
  const handleForceReParse = async () => {
    setShowForceReParseConfirm(false);
    try {
      await forceRefresh();
    } catch (error) {
      // 强制重新解析失败，静默处理
    }
  };

  return (
    <>
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

          {/* 强制重新解析按钮 */}
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setShowForceReParseConfirm(true)}
            className="shrink-0 hover:bg-[var(--color-app-secondary)]"
            title={t('detailView.forceReParse.tooltip')}
          >
            <RefreshCwOff className="h-4 w-4" style={{ color: 'var(--color-text-primary)' }} />
          </Button>

          {/* 内容显示模式切换按钮 */}
          <Button
            variant="ghost"
            size="icon"
            onClick={() => {
              setContentDisplayMode(prev => prev === 'raw' ? 'extracted' : 'raw');
            }}
            disabled={contentLoading}
            className={cn('shrink-0 hover:bg-[var(--color-app-secondary)]', contentDisplayMode === 'raw' && 'bg-[var(--color-app-secondary)]')}
            title={contentDisplayMode === 'extracted' ? t('detailView.showRaw') : t('detailView.showExtracted')}
          >
            <Code className="h-4 w-4" style={{ color: 'var(--color-text-primary)' }} />
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

          {/* 导出按钮（下拉菜单） */}
          <div className="relative" ref={exportDropdownRef}>
            <Button
              variant="ghost"
              size="icon"
              onClick={() => setIsExportDropdownOpen(prev => !prev)}
              className="shrink-0 hover:bg-[var(--color-app-secondary)]"
              title={t('viewLevel.export.title')}
            >
              <Download className="h-4 w-4" style={{ color: 'var(--color-text-primary)' }} />
            </Button>
            {/* 下拉菜单 */}
            {isExportDropdownOpen && (
              <div className="absolute right-0 top-full bg-card border rounded-md shadow-lg z-50" style={{ minWidth: '120px', backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}>
                <button
                  onClick={() => {
                    handleExport('markdown');
                    setIsExportDropdownOpen(false);
                  }}
                  disabled={exportMutation.isPending}
                  className="block w-full text-left px-4 py-2 text-sm hover:bg-accent"
                  style={{ color: 'var(--color-text-primary)' }}
                >
                  {t('viewLevel.export.formats.markdown')}
                </button>
                <button
                  onClick={() => {
                    handleExport('json');
                    setIsExportDropdownOpen(false);
                  }}
                  disabled={exportMutation.isPending}
                  className="block w-full text-left px-4 py-2 text-sm hover:bg-accent"
                  style={{ color: 'var(--color-text-primary)' }}
                >
                  {t('viewLevel.export.formats.json')}
                </button>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* 视图等级选择器栏 */}
      <div className="px-6 py-3 border-b" style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}>
        <MultiLevelViewDropdown
          value={currentViewLevel}
          onChange={changeViewLevel}
          disabled={viewLevelSaving}
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
                contentDisplayMode={contentDisplayMode}
                messages={sortedMessages.map((msg): MessageNode => ({
                  id: msg.uuid,
                  parent_id: msg.parentUuid || null,
                  depth: 0,
                  role: msg.msgType || 'unknown',
                  type: msg.msgType || 'unknown',
                  content: msg.summary || '无内容',
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

    {/* 强制重新解析确认对话框 */}
    {showForceReParseConfirm && (
      <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
        <div className="bg-card rounded-lg shadow-lg p-6 max-w-md" style={{ backgroundColor: 'var(--color-bg-card)' }}>
          <h3 className="text-lg font-semibold mb-2" style={{ color: 'var(--color-text-primary)' }}>
            {t('detailView.forceReParse.confirmTitle')}
          </h3>
          <p className="text-sm mb-4" style={{ color: 'var(--color-text-secondary)' }}>
            {t('detailView.forceReParse.confirmMessage')}
          </p>
          <div className="flex justify-end gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => setShowForceReParseConfirm(false)}
            >
              {t('detailView.forceReParse.cancel')}
            </Button>
            <Button
              size="sm"
              onClick={handleForceReParse}
            >
              {t('detailView.forceReParse.confirm')}
            </Button>
          </div>
        </div>
      </div>
    )}
    </>
  );
}
