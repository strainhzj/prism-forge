/**
 * TimelineSidebar 组件
 *
 * 右侧时间线日志侧边栏，从 App.tsx 提取
 */

import { useState, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ChevronRight, ChevronLeft, RefreshCw } from 'lucide-react';
import { cn } from '@/lib/utils';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';

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
  fullContent: string;  // 完整内容用于详情弹窗
  event_type: string;
}

export interface TimelineLog {
  id: string;
  timestamp: string;
  type: 'user' | 'assistant' | 'system';
  content: string;
  fullContent: string; // 完整内容用于弹窗
  tooltipContent: string; // 截断内容用于 tooltip
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
  /**
   * 是否折叠
   */
  collapsed?: boolean;
  /**
   * 切换折叠状态的回调
   */
  onToggleCollapse?: () => void;
}

// 自动刷新间隔（毫秒）
const DEFAULT_AUTO_REFRESH_INTERVAL = 3000;

// Tooltip 内容最大长度
const TOOLTIP_MAX_LENGTH = 500;

/**
 * 日志详情对话框组件
 * 提取为独立组件以避免代码重复
 */
interface LogDetailDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  log: TimelineLog | null;
}

function LogDetailDialog({ open, onOpenChange, log }: LogDetailDialogProps) {
  if (!log) return null;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[80vh] overflow-hidden flex flex-col">
        <DialogHeader>
          <DialogTitle>日志详情</DialogTitle>
        </DialogHeader>
        <div className="flex-1 overflow-y-auto">
          {/* 元信息 */}
          <div className="flex items-center gap-2 mb-4 pb-3 border-b" style={{ borderColor: 'var(--color-border-light)' }}>
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
              {log.type === 'user' ? '用户' : '助手'}
            </span>
            <span className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
              {new Date(log.timestamp).toLocaleString('zh-CN', {
                year: 'numeric',
                month: '2-digit',
                day: '2-digit',
                hour: '2-digit',
                minute: '2-digit',
                second: '2-digit',
              })}
            </span>
          </div>

          {/* 完整内容 */}
          <pre
            className="text-sm whitespace-pre-wrap break-words leading-relaxed"
            style={{
              color: 'var(--color-text-primary)',
              fontFamily: 'Consolas, Monaco, "Courier New", monospace',
              fontSize: '13px',
              lineHeight: '1.6',
            }}
          >
            {formatJsonContent(log.fullContent)}
          </pre>
        </div>
      </DialogContent>
    </Dialog>
  );
}

/**
 * 尝试格式化 JSON 内容
 * 如果内容是有效的 JSON，返回格式化后的字符串
 * 否则返回原内容
 */
function formatJsonContent(content: string): string {
  const trimmed = content.trim();

  // 尝试检测并提取 JSON 部分
  // 情况1: 整个内容就是 JSON
  if (trimmed.startsWith('{') || trimmed.startsWith('[')) {
    try {
      const parsed = JSON.parse(trimmed);
      return JSON.stringify(parsed, null, 2);
    } catch {
      // 不是有效 JSON，继续检查
    }
  }

  // 情况2: 内容中包含 JSON（例如 "完整输出:\n{...}"）
  // 尝试找到第一个 { 或 [ 的位置
  const jsonStartIndex = trimmed.indexOf('{') !== -1 ? trimmed.indexOf('{')
    : trimmed.indexOf('[') !== -1 ? trimmed.indexOf('[')
    : -1;

  if (jsonStartIndex > 0) {
    const potentialJson = trimmed.slice(jsonStartIndex);
    try {
      const parsed = JSON.parse(potentialJson);
      const formattedJson = JSON.stringify(parsed, null, 2);
      // 保留前缀文本 + 格式化的 JSON
      return trimmed.slice(0, jsonStartIndex) + '\n' + formattedJson;
    } catch {
      // 不是有效 JSON
    }
  }

  // 无法格式化，返回原内容
  return content;
}

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
  collapsed = false,
  onToggleCollapse,
}: TimelineSidebarProps) {
  const [parsedEvents, setParsedEvents] = useState<ParsedEvent[]>([]);
  const [parseError, setParseError] = useState('');
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [selectedLog, setSelectedLog] = useState<TimelineLog | null>(null);

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
    void loadParsedEvents(filePath);
  }, [filePath, loadParsedEvents]);

  // 自动刷新定时器
  useEffect(() => {
    if (autoRefresh && filePath && autoRefreshInterval > 0) {
      debugLog('auto-refresh', '启动自动刷新，间隔:', autoRefreshInterval);
      const intervalId = setInterval(() => {
        debugLog('auto-refresh', '自动刷新中...');
        void loadParsedEvents(filePath);
      }, autoRefreshInterval);

      return () => {
        debugLog('auto-refresh', '清除自动刷新定时器');
        clearInterval(intervalId);
      };
    }
  }, [autoRefresh, filePath, autoRefreshInterval, loadParsedEvents]);

  // 转换为时间线日志格式
  const timelineLogs: TimelineLog[] = parsedEvents.slice().reverse().map((ev, i) => {
    // 使用后端返回的完整内容
    const fullContent = ev.fullContent || ev.content;
    const displayContent = ev.content;  // 后端已经处理过截断
    // Tooltip 内容从完整内容中截取，避免显示时间过长
    const tooltipContent = fullContent.length > TOOLTIP_MAX_LENGTH
      ? fullContent.substring(0, TOOLTIP_MAX_LENGTH) + '...'
      : fullContent;
    return {
      id: `log-${i}`,
      timestamp: ev.time,
      type: ev.role.toLowerCase() === 'user' ? 'user' : 'assistant',
      content: displayContent,
      fullContent,  // 完整内容用于弹窗
      tooltipContent,  // 截断内容用于 tooltip
    };
  });

  const toggleAutoRefresh = () => {
    setAutoRefresh((prev) => !prev);
  };

  if (collapsed) {
    // 折叠状态 - 显示展开按钮
    return (
      <>
        <div className="h-full flex flex-col" style={{ backgroundColor: 'var(--color-bg-card)' }}>
          <button
            onClick={() => onToggleCollapse?.()}
            className="flex-1 w-8 border-l transition-colors flex items-center justify-center hover:bg-[var(--color-bg-card)]"
            style={{ borderColor: 'var(--color-border-light)' }}
            title="展开时间线"
          >
            <ChevronLeft className="h-4 w-4" style={{ color: 'var(--color-text-secondary)' }} />
          </button>
        </div>
        {/* 折叠状态下也保留对话框 */}
        <LogDetailDialog
          open={dialogOpen}
          onOpenChange={setDialogOpen}
          log={selectedLog}
        />
      </>
    );
  }

  return (
    <>
      <aside
        className={cn('border-l shrink-0 flex flex-col', className)}
        style={{
          backgroundColor: 'var(--color-bg-card)',
          borderColor: 'var(--color-border-light)',
        }}
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
              onClick={() => void loadParsedEvents(filePath)}
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
            onClick={() => onToggleCollapse?.()}
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
              title={log.tooltipContent}
              onClick={() => {
                setSelectedLog(log);
                setDialogOpen(true);
              }}
              style={{
                color: 'var(--color-text-primary)',
                display: '-webkit-box',
                WebkitLineClamp: 3,
                WebkitBoxOrient: 'vertical',
                overflow: 'hidden',
                lineHeight: '1.5',
                cursor: 'pointer', // 鼠标悬停时显示指针
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

    {/* 详情对话框 */}
    <LogDetailDialog
      open={dialogOpen}
      onOpenChange={setDialogOpen}
      log={selectedLog}
    />
    </>
  );
}
