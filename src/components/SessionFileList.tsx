/**
 * SessionFileList 组件
 *
 * 显示指定监控目录下的会话文件列表（按修改时间倒序）
 * 支持懒加载和点击查看详情
 */

import { useState, useCallback, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { FileText, Clock } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Skeleton } from '@/components/ui/skeleton';
import { Checkbox } from '@/components/ui/checkbox';

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[SessionFileList] ${action}`, ...args);
  }
}

// ==================== 类型定义 ====================

/**
 * 会话文件类型
 */
type SessionFileType = 'main' | 'agent' | 'unknown';

/**
 * 会话文件信息（从后端获取）
 */
export interface SessionFileInfo {
  session_id: string;
  file_path: string;
  file_size: number;
  modified_time: string;
  /** 会话摘要（从 .jsonl 文件读取） */
  summary?: string;
  /** 会话文件类型 */
  fileType?: SessionFileType;
}

export interface SessionFileListProps {
  /**
   * 监控目录路径
   */
  directoryPath: string;
  /**
   * 监控目录名称
   */
  directoryName: string;
  /**
   * 会话文件点击回调
   */
  onSessionClick?: (sessionInfo: SessionFileInfo) => void;
  /**
   * 自定义类名
   */
  className?: string;
}

/**
 * 格式化时间显示（友好的相对时间 + 绝对时间）
 */
function formatRelativeTime(isoTime: string): string {
  const date = new Date(isoTime);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  // 相对时间
  let relativeTime: string;
  if (diffMins < 1) {
    relativeTime = '刚刚';
  } else if (diffMins < 60) {
    relativeTime = `${diffMins}分钟前`;
  } else if (diffHours < 24) {
    relativeTime = `${diffHours}小时前`;
  } else if (diffDays < 7) {
    relativeTime = `${diffDays}天前`;
  } else {
    // 超过一周显示具体日期（包含年份）
    relativeTime = date.toLocaleDateString('zh-CN', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  }

  return relativeTime;
}

/**
 * 格式化完整时间（用于 tooltip）
 */
function formatFullTime(isoTime: string): string {
  const date = new Date(isoTime);
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
}

/**
 * 格式化文件大小
 */
function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

/**
 * SessionFileList 组件
 *
 * @example
 * <SessionFileList
 *   directoryPath="C:\software\Java\project"
 *   directoryName="我的项目"
 *   onSessionClick={(session) => console.log(session)}
 * />
 */
export function SessionFileList({
  directoryPath,
  directoryName,
  onSessionClick,
  className,
}: SessionFileListProps) {
  // 状态管理
  const [sessions, setSessions] = useState<SessionFileInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [includeAgent, setIncludeAgent] = useState(false); // 默认不显示 Agent

  // 懒加载状态
  const [displayedCount, setDisplayedCount] = useState(20); // 初始显示20条
  const observerTarget = useRef<HTMLDivElement>(null);

  // 加载会话列表
  const loadSessions = useCallback(async () => {
    if (!directoryPath) return;

    debugLog('loadSessions', '开始加载会话列表', directoryPath, 'includeAgent:', includeAgent);
    setLoading(true);
    setError(null);

    try {
      const result = await invoke<SessionFileInfo[]>(
        'get_sessions_by_monitored_directory',
        {
          monitoredPath: directoryPath,
          includeAgent: includeAgent,
        }
      );

      debugLog('loadSessions', '加载成功', result.length, '个会话');
      setSessions(result);
      setDisplayedCount(20); // 重置显示数量
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      debugLog('loadSessions', '加载失败', errorMsg);
      setError(errorMsg);
    } finally {
      setLoading(false);
    }
  }, [directoryPath, includeAgent]);

  // 初始加载
  useEffect(() => {
    loadSessions();
  }, [loadSessions]);

  // 懒加载：使用 Intersection Observer
  useEffect(() => {
    const target = observerTarget.current;
    if (!target) return;

    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && displayedCount < sessions.length) {
          debugLog('懒加载', `当前显示 ${displayedCount}，总共 ${sessions.length}`);
          setDisplayedCount((prev) => Math.min(prev + 20, sessions.length));
        }
      },
      { threshold: 0.1 }
    );

    observer.observe(target);

    return () => observer.disconnect();
  }, [displayedCount, sessions.length]);

  // 处理会话点击
  const handleSessionClick = useCallback(
    (session: SessionFileInfo) => {
      debugLog('handleSessionClick', '点击会话', session.session_id);
      onSessionClick?.(session);
    },
    [onSessionClick]
  );

  // 显示的会话列表（懒加载切片）
  const displayedSessions = sessions.slice(0, displayedCount);

  return (
    <div className={cn('flex flex-col h-full bg-background', className)}>
      {/* 头部 */}
      <div className="flex items-center gap-3 px-6 py-4 border-b bg-background">
        <div className="flex-1 min-w-0">
          <h2 className="text-lg font-semibold text-foreground truncate">{directoryName}</h2>
          <p className="text-xs text-muted-foreground truncate">
            {directoryPath}
          </p>
        </div>

        {/* 类型筛选复选框 */}
        <div className="flex items-center gap-2 px-3 py-1.5 border rounded-md bg-background">
          <Checkbox
            id="include-agent"
            checked={includeAgent}
            onCheckedChange={(checked) => {
              debugLog('Checkbox', 'includeAgent changed:', checked);
              setIncludeAgent(checked as boolean);
            }}
          />
          <label
            htmlFor="include-agent"
            className="text-sm cursor-pointer select-none"
          >
            显示 Agent
          </label>
        </div>

        <div className="text-sm text-muted-foreground">
          {sessions.length} 个会话
        </div>
      </div>

      {/* 会话列表 */}
      <div className="flex-1 overflow-y-auto">
        {loading ? (
          // 加载骨架屏
          <div className="p-4 space-y-3">
            {[...Array(5)].map((_, i) => (
              <div key={i} className="flex items-center gap-3 p-3 border rounded-md">
                <Skeleton className="h-4 w-4" />
                <div className="flex-1 space-y-1">
                  <Skeleton className="h-4 w-3/4" />
                  <Skeleton className="h-3 w-1/2" />
                </div>
              </div>
            ))}
          </div>
        ) : error ? (
          // 错误状态
          <div className="flex flex-col items-center justify-center h-full text-center p-4">
            <p className="text-destructive font-medium">加载失败</p>
            <p className="text-sm text-muted-foreground mt-2">{error}</p>
            <Button variant="outline" size="sm" onClick={loadSessions} className="mt-4">
              重试
            </Button>
          </div>
        ) : sessions.length === 0 ? (
          // 空状态
          <div className="flex flex-col items-center justify-center h-full text-center p-4">
            <FileText className="h-12 w-12 text-muted-foreground mb-3" />
            <p className="text-foreground font-medium">暂无会话文件</p>
            <p className="text-sm text-muted-foreground mt-2">
              该目录下还没有 Claude Code 会话记录
            </p>
          </div>
        ) : (
          // 会话列表
          <ul className="divide-y">
            {displayedSessions.map((session) => (
              <li key={session.session_id}>
                <button
                  onClick={() => handleSessionClick(session)}
                  className={cn(
                    'w-full flex items-center gap-3 px-6 py-4 transition-colors',
                    'hover:bg-accent hover:text-accent-foreground',
                    'text-left'
                  )}
                  title={`修改时间: ${formatFullTime(session.modified_time)}`}
                >
                  <FileText className="h-4 w-4 text-muted-foreground shrink-0" />
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span
                        className="text-sm font-medium text-foreground truncate"
                        title={session.summary || session.session_id}
                      >
                        {session.summary || `${session.session_id.slice(0, 8)}...`}
                      </span>
                    </div>
                    <div className="flex items-center gap-3 mt-1 text-xs text-muted-foreground">
                      <span className="flex items-center gap-1">
                        <Clock className="h-3 w-3" />
                        {formatRelativeTime(session.modified_time)}
                      </span>
                      <span>{formatFileSize(session.file_size)}</span>
                    </div>
                  </div>
                </button>
              </li>
            ))}
            {/* 懒加载触发器 */}
            {displayedCount < sessions.length && (
              <div ref={observerTarget} className="p-4 text-center text-sm text-muted-foreground">
                加载更多...
              </div>
            )}
          </ul>
        )}
      </div>
    </div>
  );
}
