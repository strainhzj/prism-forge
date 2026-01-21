/**
 * SessionFileList 组件
 *
 * 显示指定监控目录下的会话文件列表（按修改时间倒序）
 * 支持懒加载和点击查看详情
 */

import { useState, useCallback, useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { FileText, Clock } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Skeleton } from '@/components/ui/skeleton';
import { Checkbox } from '@/components/ui/checkbox';
import { Button } from '@/components/ui/button';

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
  /** 项目路径（所属监控目录路径） */
  projectPath: string;
  /** 会话摘要（从 .jsonl 文件读取，向后兼容） */
  summary?: string;
  /** 显示名称（智能提取，优先使用） */
  displayName?: string;
  /** 名称来源 */
  nameSource?: string;
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
  const { t } = useTranslation('sessions');
  // 状态管理
  const [sessions, setSessions] = useState<SessionFileInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false); // 加载更多状态
  const [error, setError] = useState<string | null>(null);
  const [includeAgent, setIncludeAgent] = useState(false);

  // 分批加载状态
  const [hasMore, setHasMore] = useState(true); // 是否有更多数据
  const observerTarget = useRef<HTMLLIElement>(null);
  const sessionsRef = useRef<SessionFileInfo[]>([]); // 使用 ref 来存储最新的 sessions

  // 同步 sessions 到 ref
  useEffect(() => {
    sessionsRef.current = sessions;
  }, [sessions]);

  // 加载会话列表（初始加载或加载更多）
  const loadSessions = useCallback(async (isLoadMore = false) => {
    if (!directoryPath) return;

    // 设置加载状态
    if (isLoadMore) {
      setLoadingMore(true);
    } else {
      setLoading(true);
      setError(null);
      setSessions([]); // 初始加载时清空列表
    }

    try {
      // 使用 ref 来获取最新的 sessions.length，避免依赖 sessions
      const offset = isLoadMore ? sessionsRef.current.length : 0;

      debugLog('loadSessions', isLoadMore ? '加载更多' : '初始加载', 'offset:', offset);

      const result = await invoke<SessionFileInfo[]>(
        'get_sessions_by_monitored_directory',
        {
          monitoredPath: directoryPath,
          includeAgent: includeAgent,
          limit: 20, // 每批加载 20 个
          offset: offset,
        }
      );

      debugLog('loadSessions', '加载成功', result.length, '个会话');

      // 🔍 调试：检查排序状态
      if (DEBUG && result.length > 0) {
        console.log('🔍 [SessionFileList] 会话排序检查:');
        console.log('  前 3 个会话的修改时间:');
        result.slice(0, 3).forEach((s, i) => {
          console.log(`    [${i}] ${s.displayName || s.session_id}`);
          console.log(`        modified_time: ${s.modified_time}`);
        });
        console.log('  后 3 个会话的修改时间:');
        result.slice(-3).forEach((s, i) => {
          const idx = result.length - 3 + i;
          console.log(`    [${idx}] ${s.displayName || s.session_id}`);
          console.log(`        modified_time: ${s.modified_time}`);
        });
      }

      if (isLoadMore) {
        // 追加数据
        setSessions((prev) => [...prev, ...result]);
      } else {
        // 替换数据
        setSessions(result);
      }

      // 判断是否还有更多数据
      setHasMore(result.length === 20);
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      debugLog('loadSessions', '加载失败', errorMsg);

      if (!isLoadMore) {
        setError(errorMsg);
      }
    } finally {
      if (isLoadMore) {
        setLoadingMore(false);
      } else {
        setLoading(false);
      }
    }
  }, [directoryPath, includeAgent]); // ✅ 移除 sessions.length 依赖

  // 初始加载
  useEffect(() => {
    loadSessions();
  }, [loadSessions]);

  // 懒加载：使用 Intersection Observer
  useEffect(() => {
    const target = observerTarget.current;
    if (!target || !hasMore || loading || loadingMore) return;

    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && hasMore) {
          debugLog('懒加载', '触发加载更多');
          loadSessions(true);
        }
      },
      { threshold: 0.1, rootMargin: '100px' } // 距离底部 100px 时触发
    );

    observer.observe(target);

    return () => observer.disconnect();
  }, [hasMore, loading, loadingMore, loadSessions]);

  // 处理会话点击
  const handleSessionClick = useCallback(
    (session: SessionFileInfo) => {
      debugLog('handleSessionClick', '点击会话', session.session_id);
      onSessionClick?.(session);
    },
    [onSessionClick]
  );

  return (
    <div className={cn('flex flex-col h-full', className)} style={{ backgroundColor: 'var(--color-bg-primary)' }}>
      {/* 头部 */}
      <div className="flex items-center gap-3 px-6 py-4 border-b" style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}>
        <div className="flex-1 min-w-0">
          <h2 className="text-lg font-semibold truncate" style={{ color: 'var(--color-text-primary)' }}>{directoryName}</h2>
          <p className="text-xs truncate" style={{ color: 'var(--color-text-secondary)' }}>
            {directoryPath}
          </p>
        </div>

        {/* 类型筛选复选框 */}
        <div className="flex items-center gap-2 px-3 py-1.5 border rounded-md transition-colors"
             style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}
             onMouseEnter={(e) => e.currentTarget.style.backgroundColor = 'var(--color-app-secondary)'}
             onMouseLeave={(e) => e.currentTarget.style.backgroundColor = 'var(--color-bg-card)'}
        >
          <Checkbox
            id="include-agent"
            checked={includeAgent}
            onCheckedChange={(checked) => {
              debugLog('Checkbox', 'includeAgent changed:', checked);
              setIncludeAgent(checked as boolean);
            }}
            className="cursor-pointer"
          />
          <label
            htmlFor="include-agent"
            className="text-sm cursor-pointer select-none user-select-none flex items-center gap-2"
            style={{ color: 'var(--color-text-secondary)' }}
            onClick={(e) => {
              // 点击 label 也触发 Checkbox 切换
              e.preventDefault();
              const newValue = !includeAgent;
              debugLog('Checkbox', 'Label clicked, toggling:', newValue);
              setIncludeAgent(newValue);
            }}
          >
            {t('fileList.showAgentSessions')}
          </label>
        </div>

        <div className="text-sm" style={{ color: 'var(--color-text-secondary)' }}>
          {sessions.length} {t('fileList.sessionCount')}
        </div>
      </div>

      {/* 会话列表 */}
      <div className="flex-1 overflow-y-auto" style={{ backgroundColor: 'var(--color-bg-primary)' }}>
        {loading ? (
          // 初始加载：完整骨架屏
          <div className="p-4 space-y-3">
            {[...Array(5)].map((_, i) => (
              <div key={i} className="flex items-center gap-3 p-4 border rounded-xl" style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}>
                <Skeleton className="h-4 w-4" />
                <div className="flex-1 space-y-2">
                  {/* 完整骨架：模拟文本和元数据 */}
                  <Skeleton className="h-4 w-3/4" />
                  <Skeleton className="h-3 w-1/3" />
                  <Skeleton className="h-3 w-1/4" />
                </div>
              </div>
            ))}
          </div>
        ) : error ? (
          // 错误状态
          <div className="flex flex-col items-center justify-center h-full text-center p-4">
            <p className="font-medium" style={{ color: 'var(--color-app-error-accent)' }}>{t('fileList.loadFailed')}</p>
            <p className="text-sm mt-2" style={{ color: 'var(--color-text-secondary)' }}>{error}</p>
            <Button variant="outline" size="sm" onClick={() => loadSessions(false)} className="mt-4">
              {t('buttons.retry')}
            </Button>
          </div>
        ) : sessions.length === 0 ? (
          // 空状态
          <div className="flex flex-col items-center justify-center h-full text-center p-4">
            <FileText className="h-12 w-12 mb-3" style={{ color: 'var(--color-text-secondary)' }} />
            <p className="font-medium" style={{ color: 'var(--color-text-primary)' }}>{t('fileList.emptyState')}</p>
            <p className="text-sm mt-2" style={{ color: 'var(--color-text-secondary)' }}>
              {t('fileList.emptyStateHint')}
            </p>
          </div>
        ) : (
          // 会话列表 - 深色圆角区块 + 悬停橙色发光效果
          <ul className="p-4 space-y-3">
            {sessions.map((session) => (
              <li key={session.session_id}>
                <button
                  onClick={() => handleSessionClick(session)}
                  className="w-full flex items-center gap-4 px-5 py-4 rounded-xl border transition-all text-left"
                  style={{
                    backgroundColor: 'var(--color-bg-card)',
                    borderColor: 'var(--color-border-light)',
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.boxShadow = '0 0 20px rgba(245, 158, 11, 0.2)';
                    e.currentTarget.style.borderColor = 'rgba(245, 158, 11, 0.3)';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.boxShadow = 'none';
                    e.currentTarget.style.borderColor = 'var(--color-border-light)';
                  }}
                  title={`修改时间: ${formatFullTime(session.modified_time)}`}
                >
                  <FileText className="h-4 w-4 shrink-0" style={{ color: 'var(--color-text-secondary)' }} />
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span
                        className="text-sm font-semibold truncate max-w-[200px]"
                        style={{ color: 'var(--color-text-primary)' }}
                        title={session.displayName || session.summary || session.session_id}
                      >
                        {session.displayName || session.summary || session.session_id}
                      </span>
                    </div>
                    <div className="flex items-center gap-3 mt-1 text-xs" style={{ color: 'var(--color-text-secondary)' }}>
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

            {/* 加载更多：触发器和骨架屏 */}
            {hasMore && (
              <li ref={observerTarget}>
                {loadingMore ? (
                  // 加载更多的骨架屏（完整骨架）
                  <div className="flex items-center gap-3 p-4 border rounded-xl" style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}>
                    <Skeleton className="h-4 w-4" />
                    <div className="flex-1 space-y-2">
                      <Skeleton className="h-4 w-3/4" />
                      <Skeleton className="h-3 w-1/3" />
                      <Skeleton className="h-3 w-1/4" />
                    </div>
                  </div>
                ) : (
                  // 懒加载触发器（不可见，用于 Intersection Observer）
                  <div className="p-4 text-center text-sm" style={{ color: 'var(--color-text-secondary)' }}>
                    {t('fileList.loadingMore')}
                  </div>
                )}
              </li>
            )}
          </ul>
        )}
      </div>

      {/* 底部统计信息 */}
      {!loading && !error && sessions.length > 0 && (
        <div className="px-6 py-3 border-t text-xs flex items-center justify-between"
             style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)', color: 'var(--color-text-secondary)' }}
        >
          <span>{t('fileList.totalSessions', { count: sessions.length })}</span>
          {!hasMore && <span>{t('fileList.allLoaded')}</span>}
        </div>
      )}
    </div>
  );
}
