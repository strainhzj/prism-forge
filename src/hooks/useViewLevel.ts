/**
 * 多级日志读取 React Query Hooks
 *
 * 使用 React Query 管理视图等级的状态和缓存
 */

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { ViewLevel } from '@/types/viewLevel';
import {
  getMessagesByLevel,
  getQAPairsByLevel,
  saveViewLevelPreference,
  getViewLevelPreference,
  exportSessionByLevel,
  type ExportFormat,
} from '@/lib/view-level-api';

// ==================== Query Keys ====================

/**
 * Query Keys 工厂
 */
export const viewLevelQueryKeys = {
  all: ['viewLevel'] as const,
  preference: (sessionId: string) =>
    [...viewLevelQueryKeys.all, 'preference', sessionId] as const,
  messages: (sessionId: string, level: ViewLevel) =>
    [...viewLevelQueryKeys.all, 'messages', sessionId, level] as const,
  qaPairs: (sessionId: string, level: ViewLevel) =>
    [...viewLevelQueryKeys.all, 'qaPairs', sessionId, level] as const,
  export: (sessionId: string, level: ViewLevel, format: ExportFormat) =>
    [...viewLevelQueryKeys.all, 'export', sessionId, level, format] as const,
};

// ==================== Hooks ====================

/**
 * 获取视图等级偏好设置
 *
 * @param sessionId - 会话 ID
 * @returns 视图等级的查询结果
 */
export function useViewLevelPreference(sessionId: string) {
  return useQuery({
    queryKey: viewLevelQueryKeys.preference(sessionId),
    queryFn: () => getViewLevelPreference(sessionId),
    staleTime: 10 * 60 * 1000, // 10 分钟内数据视为新鲜
    gcTime: 30 * 60 * 1000, // 30 分钟后垃圾回收
    retry: 1, // 仅重试一次
  });
}

/**
 * 保存视图等级偏好设置
 *
 * @returns 保存偏好的 mutation
 */
export function useSaveViewLevelPreference() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ sessionId, viewLevel }: { sessionId: string; viewLevel: ViewLevel }) =>
      saveViewLevelPreference(sessionId, viewLevel),
    onSuccess: (_, variables) => {
      // 成功后刷新偏好缓存
      queryClient.invalidateQueries({
        queryKey: viewLevelQueryKeys.preference(variables.sessionId),
      });
    },
  });
}

/**
 * 获取过滤后的消息列表
 *
 * @param sessionId - 会话 ID
 * @param viewLevel - 视图等级
 * @param enabled - 是否启用查询（默认 true）
 * @param filePath - 可选的文件路径
 * @returns 消息列表的查询结果
 */
export function useMessagesByLevel(
  sessionId: string,
  viewLevel: ViewLevel,
  enabled: boolean = true,
  filePath?: string
) {
  return useQuery({
    queryKey: viewLevelQueryKeys.messages(sessionId, viewLevel),
    queryFn: () => getMessagesByLevel(sessionId, viewLevel, filePath),
    enabled: enabled && !!sessionId,
    staleTime: 5 * 60 * 1000, // 5 分钟内数据视为新鲜
    gcTime: 10 * 60 * 1000, // 10 分钟后垃圾回收
    retry: 2, // 重试两次
  });
}

/**
 * 获取问答对列表
 *
 * @param sessionId - 会话 ID
 * @param viewLevel - 视图等级（必须是 QAPairs）
 * @param filePath - 可选的文件路径
 * @param enabled - 是否启用查询（默认 true）
 * @returns 问答对列表的查询结果
 */
export function useQAPairsByLevel(
  sessionId: string,
  viewLevel: ViewLevel,
  filePath?: string,
  enabled: boolean = true
) {
  return useQuery({
    queryKey: viewLevelQueryKeys.qaPairs(sessionId, viewLevel),
    queryFn: () => getQAPairsByLevel(sessionId, viewLevel, filePath),
    enabled: enabled && !!sessionId && viewLevel === ViewLevel.QAPairs,
    staleTime: 5 * 60 * 1000, // 5 分钟内数据视为新鲜
    gcTime: 10 * 60 * 1000, // 10 分钟后垃圾回收
    retry: 2, // 重试两次
  });
}

/**
 * 导出会话
 *
 * @returns 导出的 mutation
 */
export function useExportSessionByLevel() {
  return useMutation({
    mutationFn: ({
      sessionId,
      viewLevel,
      format,
      filePath,
    }: {
      sessionId: string;
      viewLevel: ViewLevel;
      format: ExportFormat;
      filePath?: string;
    }) => exportSessionByLevel(sessionId, viewLevel, format, filePath),
  });
}

// ==================== 组合 Hooks ====================

/**
 * 视图等级管理 Hook（包含偏好设置和消息加载）
 *
 * @param sessionId - 会话 ID
 * @returns 视图等级管理的状态和操作
 */
export function useViewLevelManager(sessionId: string) {
  const queryClient = useQueryClient();

  // 偏好设置查询
  const preferenceQuery = useViewLevelPreference(sessionId);

  // 保存偏好 mutation
  const savePreferenceMutation = useSaveViewLevelPreference();

  // 当前视图等级（优先使用偏好设置，否则使用默认值）
  const currentViewLevel = preferenceQuery.data ?? ViewLevel.Full;

  /**
   * 切换视图等级
   */
  const changeViewLevel = async (newLevel: ViewLevel) => {
    await savePreferenceMutation.mutateAsync({
      sessionId,
      viewLevel: newLevel,
    });

    // 切换视图等级后，刷新消息列表
    queryClient.invalidateQueries({
      queryKey: viewLevelQueryKeys.messages(sessionId, currentViewLevel),
    });
    queryClient.invalidateQueries({
      queryKey: viewLevelQueryKeys.qaPairs(sessionId, currentViewLevel),
    });
  };

  return {
    // 状态
    currentViewLevel,
    isLoading: preferenceQuery.isLoading,
    error: preferenceQuery.error,

    // 操作
    changeViewLevel,
    isSaving: savePreferenceMutation.isPending,

    // 工具方法
    refresh: () => preferenceQuery.refetch(),
  };
}

/**
 * 会话内容加载 Hook（根据视图等级自动选择加载消息或问答对）
 *
 * @param sessionId - 会话 ID
 * @param viewLevel - 视图等级
 * @param filePath - 可选的文件路径
 * @returns 会话内容的查询结果
 */
export function useSessionContent(
  sessionId: string,
  viewLevel: ViewLevel,
  filePath?: string
) {
  // 加载消息列表
  const messagesQuery = useMessagesByLevel(
    sessionId,
    viewLevel,
    viewLevel !== ViewLevel.QAPairs,
    filePath
  );

  // 加载问答对（仅当视图等级为 QAPairs 时）
  const qaPairsQuery = useQAPairsByLevel(sessionId, viewLevel, filePath);

  return {
    // 数据
    messages: messagesQuery.data,
    qaPairs: qaPairsQuery.data,

    // 状态
    isLoading: messagesQuery.isLoading || qaPairsQuery.isLoading,
    error: messagesQuery.error || qaPairsQuery.error,

    // 判断当前模式
    isQAPairsMode: viewLevel === ViewLevel.QAPairs,

    // 工具方法
    refresh: () => {
      messagesQuery.refetch();
      qaPairsQuery.refetch();
    },
  };
}
