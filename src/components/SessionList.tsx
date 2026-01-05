/**
 * SessionList 组件
 *
 * 显示会话列表，支持搜索、过滤、排序
 */

import { useState, useEffect, useCallback, useMemo } from 'react';
import { useNavigate } from 'react-router-dom';
import { Search, Filter, Archive } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { SessionCard } from '@/components/SessionCard';
import { Loading } from '@/components/ui/loading';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { InlineRefreshIndicator } from '@/components/RefreshIndicator';
import {
  useFilteredSessions,
  useSessionActions,
  useSessionsLoading,
  useSessionsError,
  type Session,
} from '@/stores/useSessionStore';
import { useSessionMonitor } from '@/hooks/useSessionMonitor';

export interface SessionListProps {
  /**
   * 会话点击事件
   */
  onSessionClick?: (session: Session) => void;
  /**
   * 自定义类名
   */
  className?: string;
}

/**
 * SessionList 组件
 *
 * @example
 * <SessionList
 *   onSessionClick={(session) => console.log('Selected', session)}
 * />
 */
export function SessionList({
  onSessionClick,
  className,
}: SessionListProps) {
  const navigate = useNavigate();
  const sessions = useFilteredSessions();
  const loading = useSessionsLoading();
  const error = useSessionsError();
  const {
    setActiveSessions,
    setSessionRating,
    archiveSession,
    unarchiveSession,
    updateFilters,
    resetFilters,
    clearError,
  } = useSessionActions();

  // 实时监控会话文件变更
  const { isRefreshing: isMonitoringRefresh, pendingChanges } = useSessionMonitor({
    enabled: true,
    debounceMs: 2000,
    onRefresh: async () => {
      await setActiveSessions();
    },
  });

  // 初始化加载
  useEffect(() => {
    setActiveSessions();
  }, [setActiveSessions]);

  // 过滤条件状态（简化版本，只支持搜索）
  const [searchQuery, setSearchQueryState] = useState('');

  // 归档标签页状态
  const [archiveTab, setArchiveTab] = useState<'all' | 'archived'>('all');

  // 处理搜索输入
  const handleSearchChange = useCallback(
    (value: string) => {
      setSearchQueryState(value);
      updateFilters({ searchQuery: value });
    },
    [updateFilters]
  );

  // 处理会话点击
  const handleSessionClick = useCallback(
    (session: Session) => {
      // 如果提供了回调，使用回调；否则跳转到详情页
      if (onSessionClick) {
        onSessionClick(session);
      } else {
        navigate(`/sessions/${session.sessionId}`);
      }
    },
    [navigate, onSessionClick]
  );

  // 处理评分变更
  const handleRatingChange = useCallback(
    async (sessionId: string, rating: number | null) => {
      try {
        await setSessionRating({ sessionId, rating });
      } catch (error) {
        console.error('设置评分失败:', error);
      }
    },
    [setSessionRating]
  );

  // 处理归档
  const handleArchive = useCallback(
    async (sessionId: string) => {
      try {
        await archiveSession(sessionId);
      } catch (error) {
        console.error('归档失败:', error);
      }
    },
    [archiveSession]
  );

  // 处理取消归档
  const handleUnarchive = useCallback(
    async (sessionId: string) => {
      try {
        await unarchiveSession(sessionId);
      } catch (error) {
        console.error('取消归档失败:', error);
      }
    },
    [unarchiveSession]
  );

  // 清除错误
  const handleClearError = useCallback(() => {
    clearError();
  }, [clearError]);

  // 统计信息
  const stats = useMemo(() => {
    return {
      total: sessions.length,
      rated: sessions.filter((s) => s.rating !== null && s.rating !== undefined).length,
      archived: sessions.filter((s) => s.isArchived).length,
    };
  }, [sessions]);

  // 根据标签页过滤会话
  const filteredSessions = useMemo(() => {
    if (archiveTab === 'archived') {
      return sessions.filter((s) => s.isArchived);
    }
    return sessions.filter((s) => !s.isArchived);
  }, [sessions, archiveTab]);

  return (
    <div className={cn('flex flex-col h-full bg-background', className)}>
      {/* 头部：搜索和过滤 */}
      <div className="flex flex-col gap-3 p-4 border-b bg-card">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <h2 className="text-lg font-semibold text-foreground">会话列表</h2>
            {/* 实时刷新指示器 */}
            <InlineRefreshIndicator
              isRefreshing={loading || isMonitoringRefresh}
              pendingChanges={pendingChanges}
            />
          </div>
          <div className="flex items-center gap-2">
            {stats.total > 0 && (
              <>
                <Badge variant="secondary" className="text-xs">
                  总计: {stats.total}
                </Badge>
                {stats.rated > 0 && (
                  <Badge variant="default" className="text-xs">
                    已评分: {stats.rated}
                  </Badge>
                )}
                {stats.archived > 0 && (
                  <Badge variant="outline" className="text-xs">
                    已归档: {stats.archived}
                  </Badge>
                )}
              </>
            )}
          </div>
        </div>

        {/* 搜索框 */}
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            type="text"
            placeholder="搜索会话 ID、项目名称..."
            value={searchQuery}
            onChange={(e) => handleSearchChange(e.target.value)}
            className="pl-9"
          />
        </div>

        {/* 过滤器按钮（占位） */}
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" className="flex-1">
            <Filter className="h-4 w-4 mr-2" />
            高级过滤
          </Button>
          <Button variant="outline" size="sm" onClick={resetFilters}>
            重置
          </Button>
        </div>
      </div>

      {/* 错误提示 */}
      {error && (
        <div className="p-4">
          <Alert variant="destructive">
            <AlertDescription>{error}</AlertDescription>
          </Alert>
          <Button
            variant="outline"
            size="sm"
            onClick={handleClearError}
            className="mt-2"
          >
            关闭
          </Button>
        </div>
      )}

      {/* 标签页 */}
      <Tabs className="flex-1 flex flex-col">
        <div className="px-4 pt-2">
          <TabsList className="w-full justify-start">
            <TabsTrigger
              value="all"
              data-state={archiveTab === 'all' ? 'active' : 'inactive'}
              onClick={() => setArchiveTab('all')}
              className="flex items-center gap-2"
            >
              全部会话
              <Badge variant="secondary" className="text-xs">
                {sessions.filter((s) => !s.isArchived).length}
              </Badge>
            </TabsTrigger>
            <TabsTrigger
              value="archived"
              data-state={archiveTab === 'archived' ? 'active' : 'inactive'}
              onClick={() => setArchiveTab('archived')}
              className="flex items-center gap-2"
            >
              <Archive className="h-4 w-4" />
              已归档
              <Badge variant="secondary" className="text-xs">
                {stats.archived}
              </Badge>
            </TabsTrigger>
          </TabsList>
        </div>

        {/* 加载状态 */}
        {loading && (
          <div className="flex-1 flex items-center justify-center">
            <Loading text="加载会话列表..." />
          </div>
        )}

        {/* 会话列表 */}
        {!loading && (
          <div className="flex-1 overflow-y-auto p-4">
            {filteredSessions.length === 0 ? (
              <div className="flex flex-col items-center justify-center h-full text-center">
                <Archive className="h-12 w-12 mx-auto mb-4 opacity-50 text-muted-foreground" />
                <p className="text-lg text-muted-foreground">
                  {archiveTab === 'archived' ? '暂无已归档的会话' : '暂无会话'}
                </p>
                <p className="text-sm text-muted-foreground mt-2">
                  {archiveTab === 'all' && '点击刷新按钮扫描 Claude 会话'}
                </p>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setActiveSessions()}
                  className="mt-4"
                >
                  刷新
                </Button>
              </div>
            ) : (
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {filteredSessions.map((session) => (
                  <SessionCard
                    key={session.sessionId}
                    session={session}
                    onClick={handleSessionClick}
                    onRatingChange={handleRatingChange}
                    onArchive={handleArchive}
                    onUnarchive={handleUnarchive}
                  />
                ))}
              </div>
            )}
          </div>
        )}
      </Tabs>
    </div>
  );
}
