/**
 * 向量搜索 React Query Hooks
 *
 * 使用 React Query 管理向量搜索的状态和缓存
 */

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  semanticSearch,
  findSimilarSessions,
  getVectorSettings,
  updateVectorSettings,
  syncEmbeddingsNow,
  type SemanticSearchRequest,
  type VectorSettings,
} from '@/lib/vector-api';

// ==================== Query Keys ====================

/**
 * Query Keys 工厂
 */
export const vectorQueryKeys = {
  all: ['vector'] as const,
  settings: () => [...vectorQueryKeys.all, 'settings'] as const,
  search: (request: SemanticSearchRequest) =>
    [...vectorQueryKeys.all, 'search', request] as const,
  similar: (sessionId: string | null, topK?: number, minSimilarity?: number) =>
    [...vectorQueryKeys.all, 'similar', sessionId || '', topK, minSimilarity] as const,
};

// ==================== Hooks ====================

/**
 * 获取向量设置
 *
 * @returns 向量设置的查询结果
 */
export function useVectorSettings() {
  return useQuery({
    queryKey: vectorQueryKeys.settings(),
    queryFn: getVectorSettings,
    staleTime: 5 * 60 * 1000, // 5 分钟内数据视为新鲜
  });
}

/**
 * 更新向量设置
 *
 * @returns 更新设置的 mutation
 */
export function useUpdateVectorSettings() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (settings: VectorSettings) => updateVectorSettings(settings),
    onSuccess: () => {
      // 成功后刷新设置缓存
      queryClient.invalidateQueries({
        queryKey: vectorQueryKeys.settings(),
      });
    },
  });
}

/**
 * 语义搜索
 *
 * @param request - 搜索请求参数
 * @param enabled - 是否启用查询（默认 true）
 * @returns 搜索结果的查询结果
 */
export function useSemanticSearch(
  request: SemanticSearchRequest,
  enabled: boolean = true
) {
  return useQuery({
    queryKey: vectorQueryKeys.search(request),
    queryFn: () => semanticSearch(request),
    enabled: enabled && request.query.length > 0, // 查询为空时不执行
    staleTime: 2 * 60 * 1000, // 2 分钟内结果视为新鲜
  });
}

/**
 * 查找相似会话
 *
 * @param sessionId - 目标会话 ID
 * @param topK - 返回结果数量（默认 10）
 * @param minSimilarity - 最小相似度阈值（默认 0.0）
 * @returns 相似会话的查询结果
 */
export function useSimilarSessions(
  sessionId: string | null,
  topK?: number,
  minSimilarity?: number
) {
  return useQuery({
    queryKey: vectorQueryKeys.similar(sessionId, topK, minSimilarity),
    queryFn: () => findSimilarSessions(sessionId || '', topK, minSimilarity),
    enabled: sessionId !== null && sessionId !== '', // sessionId 为 null 或空时不执行
    staleTime: 5 * 60 * 1000, // 5 分钟内结果视为新鲜
  });
}

/**
 * 手动触发向量同步
 *
 * @returns 同步的 mutation
 */
export function useSyncEmbeddings() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => syncEmbeddingsNow(),
    onSuccess: (count) => {
      // 同步成功后刷新相关缓存
      queryClient.invalidateQueries({
        queryKey: vectorQueryKeys.settings(),
      });
      console.log(`成功向量化 ${count} 个会话`);
    },
  });
}

/**
 * 向量搜索操作的组合 Hook
 *
 * 提供常用的向量搜索操作和状态
 */
export function useVectorSearch() {
  const settingsQuery = useVectorSettings();
  const updateSettingsMutation = useUpdateVectorSettings();
  const syncMutation = useSyncEmbeddings();

  const isVectorSearchEnabled = settingsQuery.data?.vectorSearchEnabled ?? false;
  const isLoading = settingsQuery.isLoading || updateSettingsMutation.isPending;
  const error = settingsQuery.error || updateSettingsMutation.error;

  return {
    // 设置相关
    settings: settingsQuery.data,
    isVectorSearchEnabled,
    isLoading,
    error,

    // 操作
    updateSettings: updateSettingsMutation.mutateAsync,
    syncEmbeddings: syncMutation.mutate,

    // 状态
    isSyncing: syncMutation.isPending,
    isUpdating: updateSettingsMutation.isPending,
  };
}
