/**
 * SessionContentView 组件
 *
 * 按照首页 Session Log 的形式显示会话内容
 * 支持懒加载和自动滚动
 */

import { useState, useCallback, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { ChevronLeft, RefreshCw } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[SessionContentView] ${action}`, ...args);
  }
}

// ==================== 类型定义 ====================

/**
 * 解析的事件（与首页 ParsedEvent 一致）
 */
export interface ParsedEvent {
  time: string;
  role: string;
  content: string;
  event_type: string;
}

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
  // 状态管理
  const [events, setEvents] = useState<ParsedEvent[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 加载会话内容
  const loadContent = useCallback(async () => {
    debugLog('loadContent', '开始加载会话内容', sessionInfo.file_path);
    setLoading(true);
    setError(null);

    try {
      // 复用现有的 parse_session_file 命令
      const result = await invoke<ParsedEvent[]>('parse_session_file', {
        filePath: sessionInfo.file_path,
      });

      debugLog('loadContent', '加载成功', result.length, '个事件');
      // 倒序显示（最新的在最上面）
      setEvents(result.slice().reverse());
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      debugLog('loadContent', '加载失败', errorMsg);
      setError(errorMsg);
    } finally {
      setLoading(false);
    }
  }, [sessionInfo.file_path]);

  // 初始加载
  useEffect(() => {
    loadContent();
  }, [loadContent]);

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
        <Button
          variant="ghost"
          size="icon"
          onClick={loadContent}
          disabled={loading}
          className="shrink-0 hover:bg-[var(--color-app-secondary)]"
          title={t('detailView.refresh')}
        >
          <RefreshCw className={cn('h-4 w-4', loading && 'animate-spin')} style={{ color: 'var(--color-text-primary)' }} />
        </Button>
      </div>

      {/* 内容区域 */}
      <div className="flex-1 overflow-y-auto" style={{ backgroundColor: 'var(--color-app-result-bg)' }}>
        {loading ? (
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
        ) : error ? (
          // 错误状态
          <div className="flex flex-col items-center justify-center h-full text-center p-4">
            <p className="font-medium" style={{ color: 'var(--color-app-error-accent)' }}>{t('detailView.loadFailed')}</p>
            <p className="text-sm mt-2" style={{ color: 'var(--color-text-secondary)' }}>{error}</p>
            <Button variant="outline" size="sm" onClick={loadContent} className="mt-4">
              {t('buttons.retry')}
            </Button>
          </div>
        ) : events.length === 0 ? (
          // 空状态
          <div className="flex flex-col items-center justify-center h-full text-center p-4">
            <p className="font-medium" style={{ color: 'var(--color-text-primary)' }}>{t('detailView.noContent')}</p>
            <p className="text-sm mt-2" style={{ color: 'var(--color-text-secondary)' }}>
              {t('detailView.noContentHint')}
            </p>
          </div>
        ) : (
          // 事件列表（倒序显示）
          <div className="p-4 space-y-3">
            {events.map((event, index) => {
              const isUser = event.role === 'user' || event.event_type === 'user_message';
              return (
                <div
                  key={index}
                  className={cn(
                    'border rounded-lg p-4 transition-all'
                  )}
                  style={{
                    backgroundColor: isUser ? 'rgba(245, 158, 11, 0.1)' : 'var(--color-bg-card)',
                    borderColor: isUser ? 'rgba(245, 158, 11, 0.3)' : 'rgba(37, 99, 235, 0.2)',
                    boxShadow: isUser ? '0 0 20px rgba(245, 158, 11, 0.2)' : 'none'
                  }}
                >
                  {/* 元数据 */}
                  <div className="flex items-center gap-2 mb-2">
                    <span
                      className={cn(
                        'text-xs font-semibold px-2 py-0.5 rounded text-white'
                      )}
                      style={{
                        backgroundColor: isUser ? 'var(--color-accent-warm)' : 'var(--color-accent-blue)',
                        boxShadow: isUser ? '0 0 10px rgba(245, 158, 11, 0.4)' : '0 0 10px rgba(37, 99, 235, 0.4)'
                      }}
                    >
                      {event.role.toUpperCase()}
                    </span>
                    <span className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
                      {event.time.split('T')[1]?.substring(0, 8) || event.time}
                    </span>
                    {event.event_type && (
                      <span className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
                        · {event.event_type}
                      </span>
                    )}
                  </div>

                  {/* 内容 */}
                  <div className="text-sm whitespace-pre-wrap break-words" style={{ color: 'var(--color-text-primary)' }}>
                    {event.content.length > 500
                      ? event.content.substring(0, 500) + '...'
                      : event.content}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* 底部统计信息 */}
      {!loading && !error && events.length > 0 && (
        <div className="px-6 py-3 border-t text-xs" style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)', color: 'var(--color-text-secondary)' }}>
          {t('detailView.messageCount', { count: events.length })}
        </div>
      )}
    </div>
  );
}
