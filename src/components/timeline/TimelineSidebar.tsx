/**
 * TimelineSidebar 组件
 *
 * 右侧时间线日志侧边栏，从 App.tsx 提取
 */

import { useState, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ChevronRight, RefreshCw } from 'lucide-react';
import { cn } from '@/lib/utils';

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[TimelineSidebar] ${action}`, ...args);
  }
}

// ==================== 类型定义 ====================

export interface ParsedEvent {
  time: string;
  role: string;
  content: string;
  event_type: string;
}

export interface TimelineLog {
  id: string;
  timestamp: string;
  type: 'user' | 'assistant' | 'system';
  content: string;
}

export interface TimelineSidebarProps {
  /**
   * 当前会话文件路径
   */
  filePath: string;
  /**
   * 自动刷新间隔（毫秒），0 表示不自动刷新
   */
  autoRefreshInterval?: number;
  /**
   * 自定义类名
   */
  className?: string;
}

// 自动刷新间隔（毫秒）
const DEFAULT_AUTO_REFRESH_INTERVAL = 3000;

/**
 * TimelineSidebar 组件
 *
 * @example
 * <TimelineSidebar filePath="/path/to/session.jsonl" />
 */
export function TimelineSidebar({
  filePath,
  autoRefreshInterval = DEFAULT_AUTO_REFRESH_INTERVAL,
  className,
}: TimelineSidebarProps) {
  const [rightCollapsed, setRightCollapsed] = useState(false);
  const [parsedEvents, setParsedEvents] = useState<ParsedEvent[]>([]);
  const [parseError, setParseError] = useState('');
  const [autoRefresh, setAutoRefresh] = useState(false);

  // 加载解析的事件
  const loadParsedEvents = useCallback(async (path: string) => {
    if (!path) return;
    try {
      setParseError('');
      const events = await invoke<ParsedEvent[]>('parse_session_file', { filePath: path });
      setParsedEvents(events);
      debugLog('loadParsedEvents', `获取到 ${events.length} 个事件`);
    } catch (e) {
      const errorMsg = `解析会话文件失败: ${e}`;
      debugLog('loadParsedEvents', errorMsg);
      setParseError(errorMsg);
      setParsedEvents([]);
    }
  }, []);

  // 初始加载
  useEffect(() => {
    loadParsedEvents(filePath);
  }, [filePath, loadParsedEvents]);

  // 自动刷新定时器
  useEffect(() => {
    if (autoRefresh && filePath && autoRefreshInterval > 0) {
      debugLog('auto-refresh', '启动自动刷新，间隔:', autoRefreshInterval);
      const intervalId = setInterval(() => {
        debugLog('auto-refresh', '自动刷新中...');
        loadParsedEvents(filePath);
      }, autoRefreshInterval);

      return () => {
        debugLog('auto-refresh', '清除自动刷新定时器');
        clearInterval(intervalId);
      };
    }
  }, [autoRefresh, filePath, autoRefreshInterval, loadParsedEvents]);

  // 转换为时间线日志格式
  const timelineLogs: TimelineLog[] = parsedEvents.slice().reverse().map((ev, i) => ({
    id: `log-${i}`,
    timestamp: ev.time,
    type: ev.role.toLowerCase() === 'user' ? 'user' : 'assistant',
    content: ev.content.length > 150 ? ev.content.substring(0, 150) + '...' : ev.content,
  }));

  const toggleAutoRefresh = () => {
    setAutoRefresh((prev) => !prev);
  };

  if (rightCollapsed) {
    // 折叠状态
    return (
      <button
        onClick={() => setRightCollapsed(false)}
        className="w-8 border-l transition-colors flex items-center justify-center hover:bg-[var(--color-bg-card)]"
        style={{ borderColor: 'var(--color-border-light)' }}
        title="展开时间线"
      >
        <ChevronRight className="h-4 w-4" style={{ color: 'var(--color-text-secondary)' }} />
      </button>
    );
  }

  return (
    <aside
      className={cn('w-[240px] border-l shrink-0 flex flex-col', className)}
      style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}
    >
      {/* 头部 */}
      <div
        className="flex items-center justify-between px-4 py-3 border-b"
        style={{ borderColor: 'var(--color-border-light)' }}
      >
        <div>
          <h2 className="text-sm font-semibold" style={{ color: 'var(--color-text-primary)' }}>
            时间线日志
          </h2>
          <p className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
            {timelineLogs.length} 条记录
          </p>
        </div>
        <div className="flex items-center gap-2">
          {/* 刷新控制 */}
          <div className="flex gap-1">
            <button
              onClick={() => loadParsedEvents(filePath)}
              className="p-1.5 rounded transition-colors hover:bg-[var(--color-app-secondary)]"
              title="刷新"
              disabled={autoRefresh}
            >
              <RefreshCw
                className={cn('h-3.5 w-3.5', autoRefresh && 'opacity-50')}
                style={{ color: 'var(--color-text-secondary)' }}
              />
            </button>
            <button
              onClick={toggleAutoRefresh}
              className={cn(
                'p-1.5 rounded transition-colors',
                autoRefresh ? 'text-white' : ''
              )}
              style={
                autoRefresh
                  ? { backgroundColor: 'var(--color-accent-warm)' }
                  : { color: 'var(--color-text-secondary)' }
              }
              onMouseEnter={(e) => {
                if (!autoRefresh) {
                  e.currentTarget.style.backgroundColor = 'var(--color-app-secondary)';
                }
              }}
              onMouseLeave={(e) => {
                if (!autoRefresh) {
                  e.currentTarget.style.backgroundColor = 'transparent';
                }
              }}
              title={autoRefresh ? '停止自动刷新' : '开启自动刷新'}
            >
              {autoRefresh ? '⏸' : '▶'}
            </button>
          </div>
          <button
            onClick={() => setRightCollapsed(true)}
            className="p-1 rounded transition-colors hover:bg-[var(--color-app-secondary)]"
            title="折叠侧边栏"
          >
            <ChevronRight className="h-4 w-4" style={{ color: 'var(--color-text-secondary)' }} />
          </button>
        </div>
      </div>

      {/* 时间线日志列表 */}
      <div className="flex-1 overflow-y-auto p-3 space-y-3">
        {parseError && (
          <div
            className="p-2 rounded text-xs"
            style={{
              backgroundColor: 'var(--color-app-error-bg)',
              border: '1px solid var(--color-app-error-border)',
              color: 'var(--color-app-error-text)',
            }}
          >
            {parseError}
          </div>
        )}

        {timelineLogs.length === 0 && !parseError && (
          <div className="text-center py-8">
            <p className="text-sm" style={{ color: 'var(--color-text-secondary)' }}>
              暂无日志记录
            </p>
          </div>
        )}

        {timelineLogs.map((log) => (
          <div
            key={log.id}
            className="p-3 rounded-lg border transition-all hover:shadow-lg"
            style={{
              backgroundColor: 'var(--color-bg-primary)',
              borderColor: 'var(--color-border-light)',
            }}
            onMouseEnter={(e) => {
              const isUser = log.type === 'user';
              const color = isUser ? '245, 158, 11' : '37, 99, 235'; // warm orange or blue
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
                  backgroundColor:
                    log.type === 'user' ? 'var(--color-accent-warm)' : 'var(--color-accent-blue)',
                  boxShadow:
                    log.type === 'user'
                      ? '0 0 8px rgba(245, 158, 11, 0.5)'
                      : '0 0 8px rgba(37, 99, 235, 0.5)',
                }}
              />
              <span className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
                {new Date(log.timestamp).toLocaleTimeString('zh-CN', {
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
              {log.content}
            </p>
          </div>
        ))}
      </div>

      {/* 底部信息 */}
      <div
        className="px-3 py-2 border-t text-xs text-center"
        style={{
          borderColor: 'var(--color-border-light)',
          color: 'var(--color-text-secondary)',
        }}
      >
        {autoRefresh && '自动刷新中...'}
      </div>
    </aside>
  );
}
