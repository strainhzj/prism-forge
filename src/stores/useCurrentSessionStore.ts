/**
 * Current Session Store - 当前会话管理
 *
 * 使用 Zustand 管理全局当前会话状态，用于：
 * - 跟踪用户在首页打开的会话
 * - 在 PromptLab 等功能中使用当前会话上下文
 */

import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';
import { useMemo } from 'react';

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[CurrentSessionStore] ${action}`, ...args);
  }
}

// ==================== 类型定义 ====================

/**
 * 当前会话信息
 */
export interface CurrentSession {
  /** 会话 ID */
  sessionId: string;
  /** 会话文件路径 */
  filePath: string;
  /** 项目名称 */
  projectName: string;
  /** 会话显示名称（可选） */
  displayName?: string;
}

// ==================== Store State ====================

interface CurrentSessionState {
  // 数据状态
  currentSession: CurrentSession | null;

  // Actions
  setCurrentSession: (session: CurrentSession | null) => void;
  clearCurrentSession: () => void;
}

// ==================== Store 实现 ====================

export const useCurrentSessionStore = create<CurrentSessionState>()(
  immer((set) => ({
    // 初始状态
    currentSession: null,

    // 设置当前会话
    setCurrentSession: (session) => {
      debugLog('setCurrentSession', session);
      set((state) => {
        state.currentSession = session;
      });
    },

    // 清除当前会话
    clearCurrentSession: () => {
      debugLog('clearCurrentSession');
      set((state) => {
        state.currentSession = null;
      });
    },
  }))
);

// ==================== 便捷 Hooks ====================

/**
 * 获取当前会话
 */
export const useCurrentSession = () => useCurrentSessionStore((state) => state.currentSession);

/**
 * 获取当前会话 ID
 */
export const useCurrentSessionId = () => useCurrentSessionStore((state) => state.currentSession?.sessionId ?? null);

/**
 * 获取当前会话文件路径
 */
export const useCurrentSessionFilePath = () => useCurrentSessionStore((state) => state.currentSession?.filePath ?? null);

/**
 * 获取 store actions（稳定引用）
 * 直接从 store 获取方法，并通过 useMemo 保持引用稳定
 */
export const useCurrentSessionActions = () => {
  return useMemo(() => {
    const store = useCurrentSessionStore.getState();
    return {
      setCurrentSession: store.setCurrentSession,
      clearCurrentSession: store.clearCurrentSession,
    };
  }, []); // 空依赖数组确保只执行一次
};
