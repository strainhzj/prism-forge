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
  filePath?: string,
  autoRefreshEnabled: boolean = true
) {
  return useQuery({
    queryKey: viewLevelQueryKeys.messages(sessionId, viewLevel),
    queryFn: () => getMessagesByLevel(sessionId, viewLevel, filePath),
    enabled: enabled && !!sessionId,
    staleTime: 5 * 60 * 1000, // 5 分钟内数据视为新鲜
    gcTime: 10 * 60 * 1000, // 10 分钟后垃圾回收
    retry: 2, // 重试两次
    refetchInterval: autoRefreshEnabled ? 5000 : false, // 5 秒自动刷新
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
  enabled: boolean = true,
  autoRefreshEnabled: boolean = true
) {
  return useQuery({
    queryKey: viewLevelQueryKeys.qaPairs(sessionId, viewLevel),
    queryFn: () => getQAPairsByLevel(sessionId, viewLevel, filePath),
    enabled: enabled && !!sessionId && viewLevel === ViewLevel.QAPairs,
    staleTime: 5 * 60 * 1000,
    gcTime: 10 * 60 * 1000,
    retry: 2,
    refetchInterval: autoRefreshEnabled ? 5000 : false, // 5 秒自动刷新
  });
}

/**
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
    // 先失效旧的视图等级的查询缓存
    queryClient.invalidateQueries({
      queryKey: viewLevelQueryKeys.messages(sessionId, currentViewLevel),
    });

    // 保存偏好设置（异步，不阻塞）
    savePreferenceMutation.mutateAsync({
      sessionId,
      viewLevel: newLevel,
    }).catch((error) => {
      console.error('[useViewLevelManager] 保存视图等级偏好失败:', error);
    });

    // 失效新的视图等级的查询缓存，触发重新加载
    queryClient.invalidateQueries({
      queryKey: viewLevelQueryKeys.messages(sessionId, newLevel),
    });

    // 失效偏好查询缓存，确保 currentViewLevel 更新
    queryClient.invalidateQueries({
      queryKey: viewLevelQueryKeys.preference(sessionId),
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
 * 问答模式使用 qaPairs API，其他模式使用 messages API
 * 但都转换为统一的 messages 格式供 TimelineMessageList 展示
 *
 * @param sessionId - 会话 ID
 * @param viewLevel - 视图等级
 * @param filePath - 可选的文件路径
 * @param autoRefreshEnabled - 是否启用自动刷新（默认 true）
 * @returns 会话内容的查询结果
 */
export function useSessionContent(
  sessionId: string,
  viewLevel: ViewLevel,
  filePath?: string,
  autoRefreshEnabled: boolean = true
) {
  const queryClient = useQueryClient();
  const isQAPairsMode = viewLevel === ViewLevel.QAPairs;

  // 加载消息列表（非问答模式）
  const messagesQuery = useMessagesByLevel(
    sessionId,
    viewLevel,
    !isQAPairsMode, // 问答模式禁用此查询
    filePath,
    autoRefreshEnabled
  );

  // 加载问答对（仅问答模式）
  const qaPairsQuery = useQAPairsByLevel(
    sessionId,
    viewLevel,
    filePath,
    true,
    autoRefreshEnabled
  );

  // 将问答对转换为消息列表
  const qaMessages = qaPairsQuery.data ? convertQAPairsToMessages(qaPairsQuery.data) : undefined;

  // 清除缓存并强制刷新
  const forceRefresh = async () => {
    // 使用 refetchQueries 替代 invalidateQueries，等待数据重新加载完成
    await queryClient.refetchQueries({
      queryKey: viewLevelQueryKeys.messages(sessionId, viewLevel),
    });
    await queryClient.refetchQueries({
      queryKey: viewLevelQueryKeys.qaPairs(sessionId, viewLevel),
    });
    await queryClient.refetchQueries({
      queryKey: viewLevelQueryKeys.preference(sessionId),
    });

    // 开发模式下输出日志
    if (import.meta.env.DEV) {
      console.log('[useSessionContent] 强制刷新缓存:', {
        sessionId,
        viewLevel,
        filePath,
      });
    }
  };

  return {
    // 数据 - 问答模式使用转换后的消息，否则使用原始消息
    messages: isQAPairsMode ? qaMessages : messagesQuery.data,

    // 状态
    isLoading: isQAPairsMode ? qaPairsQuery.isLoading : messagesQuery.isLoading,
    error: isQAPairsMode ? qaPairsQuery.error : messagesQuery.error,

    // 工具方法
    refresh: () => {
      messagesQuery.refetch();
      qaPairsQuery.refetch();
    },

    // 强制刷新（清除缓存）
    forceRefresh,
  };
}

/**
 * 将问答对列表转换为消息列表
 *
 * 将 QAPair[] 转换为 Message[]，以便使用 TimelineMessageList 展示
 * 结果按时间戳倒序排列（最新的消息在最前面）
 *
 * @param qaPairs - 问答对列表
 * @returns 消息列表（按时间倒序）
 */
function convertQAPairsToMessages(qaPairs: import('@/types/viewLevel').QAPair[]): import('@/types/viewLevel').Message[] {
  const messages: import('@/types/viewLevel').Message[] = [];

  for (const pair of qaPairs) {
    // 添加问题消息
    messages.push({
      uuid: pair.question.uuid,
      sessionId: pair.question.sessionId,
      parentUuid: pair.question.parentUuid,
      msgType: pair.question.msgType,
      timestamp: pair.question.timestamp,
      summary: pair.question.summary,
      offset: 0,
      length: 0,
      createdAt: pair.question.timestamp,
    });

    // 添加答案消息（如果存在）
    if (pair.answer) {
      messages.push({
        uuid: pair.answer.uuid,
        sessionId: pair.answer.sessionId,
        parentUuid: pair.answer.parentUuid,
        msgType: pair.answer.msgType,
        timestamp: pair.answer.timestamp,
        summary: pair.answer.summary,
        offset: 0,
        length: 0,
        createdAt: pair.answer.timestamp,
      });
    }
  }

  // 按时间戳倒序排序（最新的在最前面）
  messages.sort((a, b) => {
    const timeA = new Date(a.timestamp).getTime();
    const timeB = new Date(b.timestamp).getTime();
    return timeB - timeA; // 倒序：timeB - timeA
  });

  return messages;
}
