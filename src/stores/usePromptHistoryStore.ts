/**
 * 提示词生成历史记录 Store
 *
 * 使用 Zustand + Immer 管理提示词历史状态
 */

import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';
import { invoke } from '@tauri-apps/api/core';
import type { PromptGenerationHistory } from '@/types/generated';

const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[PromptHistoryStore] ${action}`, ...args);
  }
}

// 类型定义
export type { PromptGenerationHistory };

/**
 * 状态接口
 */
interface PromptHistoryState {
  // 数据状态
  histories: PromptGenerationHistory[];
  totalCount: number;
  loading: boolean;
  error: string | null;

  // 分页状态
  currentPage: number;
  pageSize: number;
  hasMore: boolean;

  // Actions
  fetchHistories: (offset?: number, limit?: number) => Promise<void>;
  fetchMoreHistories: () => Promise<void>;
  deleteHistory: (id: bigint) => Promise<void>;
  toggleFavorite: (id: bigint) => Promise<void>;
  getHistoryById: (id: bigint) => Promise<PromptGenerationHistory | null>;
  refresh: () => Promise<void>;
  clearError: () => void;
}

/**
 * 提示词历史记录 Store
 */
export const usePromptHistoryStore = create<PromptHistoryState>()(
  immer((set, get) => ({
    // 初始状态
    histories: [],
    totalCount: 0,
    loading: false,
    error: null,
    currentPage: 0,
    pageSize: 20,
    hasMore: true,

    /**
     * 获取历史记录列表
     */
    fetchHistories: async (offset = 0, limit = 20) => {
      debugLog('fetchHistories called with offset:', offset, 'limit:', limit);
      set((state) => {
        state.loading = true;
        state.error = null;
      });

      try {
        debugLog('Invoking Tauri commands...');
        const [histories, count] = await Promise.all([
          invoke<PromptGenerationHistory[]>('cmd_get_prompt_history_paginated', {
            offset,
            limit,
          }),
          invoke<number>('cmd_count_prompt_history'),
        ]);

        debugLog('Received histories:', histories.length, 'total count:', count);

        set((state) => {
          state.histories = offset === 0 ? histories : [...state.histories, ...histories];
          state.totalCount = count;
          state.currentPage = Math.floor(offset / limit);
          state.hasMore = offset + histories.length < count;
          state.loading = false;
        });

        debugLog('State updated successfully');
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        debugLog('Error fetching histories:', errorMessage);
        set((state) => {
          state.error = errorMessage;
          state.loading = false;
        });
      }
    },

    /**
     * 加载更多历史记录
     */
    fetchMoreHistories: async () => {
      const { currentPage, pageSize, loading } = get();
      if (loading) return;

      const offset = (currentPage + 1) * pageSize;
      await get().fetchHistories(offset, pageSize);
    },

    /**
     * 删除历史记录
     */
    deleteHistory: async (id: bigint) => {
      set((state) => {
        state.loading = true;
        state.error = null;
      });

      try {
        await invoke('cmd_delete_prompt_history', { id: Number(id) });

        set((state) => {
          state.histories = state.histories.filter((h) => h.id !== id);
          state.totalCount = state.totalCount - 1;
          state.loading = false;
        });
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        set((state) => {
          state.error = errorMessage;
          state.loading = false;
        });
      }
    },

    /**
     * 切换收藏状态
     */
    toggleFavorite: async (id: bigint) => {
      try {
        const isFavorite = await invoke<boolean>('cmd_toggle_prompt_history_favorite', {
          id: Number(id),
        });

        set((state) => {
          const history = state.histories.find((h) => h.id === id);
          if (history) {
            history.isFavorite = isFavorite;
          }
        });
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        set((state) => {
          state.error = errorMessage;
        });
      }
    },

    /**
     * 根据 ID 获取历史记录
     */
    getHistoryById: async (id: bigint) => {
      try {
        const history = await invoke<PromptGenerationHistory | null>(
          'cmd_get_prompt_history_by_id',
          { id: Number(id) },
        );
        return history;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        set((state) => {
          state.error = errorMessage;
        });
        return null;
      }
    },

    /**
     * 刷新列表
     */
    refresh: async () => {
      const { pageSize } = get();
      await get().fetchHistories(0, pageSize);
    },

    /**
     * 清除错误
     */
    clearError: () => {
      set((state) => {
        state.error = null;
      });
    },
  }))
);

/**
 * 便捷 Hooks
 */

/**
 * 获取所有历史记录
 */
export const useHistories = () => usePromptHistoryStore((state) => state.histories);

/**
 * 获取加载状态
 */
export const useHistoryLoading = () => usePromptHistoryStore((state) => state.loading);

/**
 * 获取错误信息
 */
export const useHistoryError = () => usePromptHistoryStore((state) => state.error);

/**
 * 获取历史记录 Actions
 */
export const useHistoryActions = () =>
  usePromptHistoryStore((state) => ({
    fetchHistories: state.fetchHistories,
    fetchMoreHistories: state.fetchMoreHistories,
    deleteHistory: state.deleteHistory,
    toggleFavorite: state.toggleFavorite,
    getHistoryById: state.getHistoryById,
    refresh: state.refresh,
    clearError: state.clearError,
  }));

/**
 * 获取分页状态
 */
export const useHistoryPagination = () =>
  usePromptHistoryStore((state) => ({
    currentPage: state.currentPage,
    pageSize: state.pageSize,
    hasMore: state.hasMore,
    totalCount: state.totalCount,
  }));
