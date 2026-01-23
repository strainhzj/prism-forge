/**
 * 提示词历史记录组件
 */

import { useEffect, useState, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { Clock, Star, Trash2, Eye, Loader2, Search } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog';
import { usePromptHistoryStore } from '@/stores/usePromptHistoryStore';
import { PromptHistoryDetail } from './PromptHistoryDetail';
import type { PromptGenerationHistory } from '@/types/generated';

const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[PromptHistory] ${action}`, ...args);
  }
}

/**
 * 提示词历史记录列表组件
 */
export function PromptHistory() {
  const { t } = useTranslation('promptLab');
  debugLog('PromptHistory component mounted');

  // 直接使用 store，获取稳定的引用
  const histories = usePromptHistoryStore((state) => state.histories);
  const loading = usePromptHistoryStore((state) => state.loading);
  const error = usePromptHistoryStore((state) => state.error);
  const hasMore = usePromptHistoryStore((state) => state.hasMore);
  const fetchHistories = usePromptHistoryStore((state) => state.fetchHistories);
  const deleteHistory = usePromptHistoryStore((state) => state.deleteHistory);
  const toggleFavorite = usePromptHistoryStore((state) => state.toggleFavorite);
  const refresh = usePromptHistoryStore((state) => state.refresh);
  const clearError = usePromptHistoryStore((state) => state.clearError);

  const [selectedHistory, setSelectedHistory] = useState<PromptGenerationHistory | null>(
    null,
  );
  const [detailOpen, setDetailOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');

  useEffect(() => {
    debugLog('useEffect triggered - fetching histories');
    fetchHistories();
    // 只在组件挂载时执行一次
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  /**
   * 查看详情
   */
  const handleViewDetail = (history: PromptGenerationHistory) => {
    debugLog('Viewing detail for history:', history.id);
    setSelectedHistory(history);
    setDetailOpen(true);
  };

  /**
   * 切换收藏
   */
  const handleToggleFavorite = async (id: bigint, event: React.MouseEvent) => {
    event.stopPropagation();
    await toggleFavorite(id);
  };

  /**
   * 删除历史
   */
  const handleDelete = async (id: bigint, event: React.MouseEvent) => {
    event.stopPropagation();
    await deleteHistory(id);
  };

  /**
   * 加载更多
   */
  const handleLoadMore = () => {
    if (!loading && hasMore) {
      fetchHistories(histories.length, 20);
    }
  };

  /**
   * 格式化时间
   */
  const formatTime = (timeStr: string) => {
    try {
      const date = new Date(timeStr);
      const now = new Date();
      const diffMs = now.getTime() - date.getTime();
      const diffMins = Math.floor(diffMs / 60000);
      const diffHours = Math.floor(diffMs / 3600000);
      const diffDays = Math.floor(diffMs / 86400000);

      if (diffMins < 1) return '刚刚';
      if (diffMins < 60) return `${diffMins} 分钟前`;
      if (diffHours < 24) return `${diffHours} 小时前`;
      if (diffDays < 7) return `${diffDays} 天前`;

      return new Intl.DateTimeFormat('zh-CN', {
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
      }).format(date);
    } catch {
      return timeStr;
    }
  };

  /**
   * 解析引用会话数
   */
  const getReferencedSessionCount = (history: PromptGenerationHistory): number => {
    if (!history.referencedSessions) return 0;
    try {
      const sessions = JSON.parse(history.referencedSessions);
      return Array.isArray(sessions) ? sessions.length : 0;
    } catch {
      return 0;
    }
  };

  /**
   * 解析 Token 统计
   */
  const getTokenStats = (history: PromptGenerationHistory) => {
    if (!history.tokenStats) return null;
    try {
      return JSON.parse(history.tokenStats);
    } catch {
      return null;
    }
  };

  /**
   * 过滤后的历史记录列表
   */
  const filteredHistories = useMemo(() => {
    if (!searchQuery.trim()) {
      return histories;
    }

    const query = searchQuery.toLowerCase().trim();
    return histories.filter((history) => {
      // 搜索范围：原始目标 + 优化后的提示词
      const goalMatch = history.originalGoal.toLowerCase().includes(query);
      const promptMatch = history.enhancedPrompt.toLowerCase().includes(query);
      return goalMatch || promptMatch;
    });
  }, [histories, searchQuery]);

  return (
    <div className="space-y-4">
      {/* 头部 */}
      <div className="flex items-center justify-between px-6 pt-6">
        <div>
          <h2 className="text-xl font-semibold" style={{ color: 'var(--color-text-primary)' }}>{t('history.title')}</h2>
          <p className="text-sm mt-1" style={{ color: 'var(--color-text-secondary)' }}>
            {t('history.description')}
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={refresh}
          className="bg-white dark:bg-gray-800 border-gray-300 dark:border-gray-600 text-gray-900 dark:text-gray-100 hover:bg-gray-100 dark:hover:bg-gray-700"
        >
          {t('history.refresh')}
        </Button>
      </div>

      {/* 错误提示 */}
      {error && (
        <div className="mx-6 p-4 rounded-lg bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 text-sm border border-red-200 dark:border-red-800">
          {error}
          <Button
            variant="link"
            className="ml-2 p-0 h-auto text-red-600 dark:text-red-400 hover:text-red-700 dark:hover:text-red-300"
            onClick={clearError}
          >
            {t('history.cancel')}
          </Button>
        </div>
      )}

      {/* 搜索框 */}
      <div className="px-6">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400 dark:text-gray-500" />
          <Input
            type="text"
            placeholder={t('history.searchPlaceholder')}
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-10 bg-white dark:bg-gray-800 border-gray-300 dark:border-gray-600 text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-500"
          />
        </div>
      </div>

      {/* 加载状态 */}
      {loading && histories.length === 0 && (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-gray-400 dark:text-gray-600" />
        </div>
      )}

      {/* 空状态 */}
      {!loading && histories.length === 0 && (
        <div className="text-center py-12">
          <Clock className="h-12 w-12 mx-auto mb-4" style={{ color: 'var(--color-text-disabled)' }} />
          <p style={{ color: 'var(--color-text-secondary)' }}>{t('history.noHistory')}</p>
          <p className="text-sm mt-2" style={{ color: 'var(--color-text-disabled)' }}>
            {t('history.noHistoryHint')}
          </p>
        </div>
      )}

      {/* 搜索无结果 */}
      {!loading && histories.length > 0 && filteredHistories.length === 0 && (
        <div className="text-center py-12">
          <Search className="h-12 w-12 mx-auto mb-4 text-gray-400 dark:text-gray-600" />
          <p className="text-gray-600 dark:text-gray-400">{t('history.noSearchResults')}</p>
          <p className="text-sm mt-2 text-gray-400 dark:text-gray-600">
            {t('history.noSearchResultsHint')}
          </p>
        </div>
      )}

      {/* 历史记录列表 */}
      <div className="space-y-3 px-6 pb-6">
        {filteredHistories.map((history) => {
          const tokenStats = getTokenStats(history);
          const sessionCount = getReferencedSessionCount(history);

          return (
            <Card
              key={history.id?.toString()}
              className="p-4 hover:-translate-y-0.5 hover:shadow-md transition-all cursor-pointer"
              style={{
                backgroundColor: 'var(--color-bg-card)',
                border: '1px solid var(--color-border-light)',
              }}
              onMouseEnter={(e) => e.currentTarget.style.backgroundColor = 'var(--color-app-secondary)'}
              onMouseLeave={(e) => e.currentTarget.style.backgroundColor = 'var(--color-bg-card)'}
              onClick={() => handleViewDetail(history)}
            >
              <div className="flex items-start justify-between gap-4">
                {/* 主要内容 */}
                <div className="flex-1 min-w-0">
                  {/* 标题行 */}
                  <div className="flex items-center gap-2 mb-2">
                    {history.isFavorite && (
                      <Star className="h-4 w-4 fill-yellow-400 text-yellow-400" />
                    )}
                    <span
                      className="text-sm font-medium truncate"
                      style={{ color: 'var(--color-text-primary)' }}
                    >
                      {history.originalGoal.slice(0, 60)}
                      {history.originalGoal.length > 60 && '...'}
                    </span>
                  </div>

                  {/* 元信息 */}
                  <div
                    className="flex flex-wrap items-center gap-3 text-xs"
                    style={{ color: 'var(--color-text-secondary)' }}
                  >
                    <div className="flex items-center gap-1">
                      <Clock className="h-3 w-3" />
                      {formatTime(history.createdAt)}
                    </div>

                    {tokenStats && typeof tokenStats === 'object' && (
                      <div className="flex items-center gap-1">
                        <span>📊</span>
                        <span>
                          {tokenStats.inputTokens || 0}/{tokenStats.maxTokens || 5000} tokens
                        </span>
                      </div>
                    )}

                    {history.confidence !== null &&
                      history.confidence !== undefined && (
                        <Badge
                          variant="secondary"
                          className="text-xs"
                          style={{
                            backgroundColor: 'var(--color-app-button-default)',
                            color: 'var(--color-text-secondary)',
                          }}
                        >
                          {t('history.confidence')} {Math.round(history.confidence * 100)}%
                        </Badge>
                      )}

                    {sessionCount > 0 && (
                      <div className="flex items-center gap-1">
                        <span>💬</span>
                        <span>{t('history.referencedSessions')} {sessionCount}</span>
                      </div>
                    )}

                    {history.llmProvider && (
                      <div className="flex items-center gap-1">
                        <span>🤖</span>
                        <span style={{ color: 'var(--color-text-secondary)' }}>
                          {history.llmProvider} / {history.llmModel || 'Unknown'}
                        </span>
                      </div>
                    )}
                  </div>
                </div>

                {/* 操作按钮 */}
                <div className="flex items-center gap-1">
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-8 w-8 p-0 hover:bg-gray-100 dark:hover:bg-gray-700"
                    style={{ color: 'var(--color-text-secondary)' }}
                    onMouseEnter={(e) => e.currentTarget.style.color = 'var(--color-text-primary)'}
                    onMouseLeave={(e) => e.currentTarget.style.color = 'var(--color-text-secondary)'}
                    onClick={(e) => handleToggleFavorite(history.id!, e)}
                  >
                    <Star
                      className={`h-4 w-4 ${
                        history.isFavorite
                          ? 'fill-yellow-400 text-yellow-400'
                          : ''
                      }`}
                    />
                  </Button>

                  <AlertDialog>
                    <AlertDialogTrigger asChild>
                      <Button
                        variant="ghost"
                        size="sm"
                        className="h-8 w-8 p-0"
                        style={{ color: 'var(--color-app-error-text)' }}
                        onMouseEnter={(e) => e.currentTarget.style.color = 'var(--color-settings-error)'}
                        onMouseLeave={(e) => e.currentTarget.style.color = 'var(--color-app-error-text)'}
                        onClick={(e) => e.stopPropagation()}
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </AlertDialogTrigger>
                    <AlertDialogContent
                      className="border"
                      style={{
                        backgroundColor: 'var(--color-bg-card)',
                        borderColor: 'var(--color-border-light)',
                      }}
                    >
                      <AlertDialogHeader>
                        <AlertDialogTitle style={{ color: 'var(--color-text-primary)' }}>
                          {t('history.deleteConfirmTitle')}
                        </AlertDialogTitle>
                        <AlertDialogDescription style={{ color: 'var(--color-text-secondary)' }}>
                          {t('history.deleteConfirm')}
                        </AlertDialogDescription>
                      </AlertDialogHeader>
                      <AlertDialogFooter>
                        <AlertDialogCancel
                          className="hover:opacity-80"
                          style={{
                            backgroundColor: 'var(--color-app-button-default)',
                            color: 'var(--color-text-primary)',
                          }}
                        >
                          {t('history.cancel')}
                        </AlertDialogCancel>
                        <AlertDialogAction
                          onClick={(e) => handleDelete(history.id!, e)}
                          className="hover:opacity-90"
                          style={{
                            backgroundColor: 'var(--color-app-error-accent)',
                            color: '#FFFFFF',
                          }}
                        >
                          {t('history.confirmDelete')}
                        </AlertDialogAction>
                      </AlertDialogFooter>
                    </AlertDialogContent>
                  </AlertDialog>

                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-8 w-8 p-0 hover:bg-gray-100 dark:hover:bg-gray-700"
                    style={{ color: 'var(--color-text-secondary)' }}
                    onMouseEnter={(e) => e.currentTarget.style.color = 'var(--color-text-primary)'}
                    onMouseLeave={(e) => e.currentTarget.style.color = 'var(--color-text-secondary)'}
                    onClick={(e) => {
                      e.stopPropagation();
                      handleViewDetail(history);
                    }}
                  >
                    <Eye className="h-4 w-4" />
                  </Button>
                </div>
              </div>
            </Card>
          );
        })}
      </div>

      {/* 加载更多 */}
      {hasMore && histories.length > 0 && !searchQuery && (
        <div className="flex justify-center pt-4 pb-6">
          <button
            onClick={handleLoadMore}
            disabled={loading}
            className="px-6 py-2.5 border rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2 text-sm font-medium hover:opacity-80"
            style={{
              backgroundColor: 'var(--color-bg-card)',
              borderColor: 'var(--color-border-light)',
              color: 'var(--color-text-primary)',
            }}
            onMouseEnter={(e) => e.currentTarget.style.backgroundColor = 'var(--color-app-secondary)'}
            onMouseLeave={(e) => e.currentTarget.style.backgroundColor = 'var(--color-bg-card)'}
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
            </svg>
            {loading && <Loader2 className="h-4 w-4 animate-spin" />}
            {t('history.loadMore')}
          </button>
        </div>
      )}

      {/* 详情弹窗 */}
      <PromptHistoryDetail
        history={selectedHistory}
        open={detailOpen}
        onOpenChange={setDetailOpen}
      />
    </div>
  );
}
