/**
 * Settings Store - LLM API Provider 管理
 *
 * 使用 Zustand 管理全局状态，包括：
 * - providers: API 提供商列表
 * - activeProviderId: 当前活跃的提供商 ID
 * - 各种异步操作 (fetch, save, delete, set active, test)
 */

import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';
import { invoke } from '@tauri-apps/api/core';
import { useMemo } from 'react';

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[SettingsStore] ${action}`, ...args);
  }
}

/**
 * 解析 Tauri 命令错误
 * Tauri 返回的错误可能是：
 * - 字符串
 * - { message: string } 对象
 * - Error 对象
 */
function parseError(error: unknown): string {
  if (typeof error === 'string') {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  if (error && typeof error === 'object') {
    // Tauri CommandError 格式: { message: string }
    if ('message' in error && typeof (error as { message: unknown }).message === 'string') {
      return (error as { message: string }).message;
    }
    // 尝试 JSON 序列化
    try {
      return JSON.stringify(error);
    } catch {
      return String(error);
    }
  }
  return String(error);
}

// ==================== 类型定义 ====================

/**
 * API 提供商类型
 * 注意：值必须与 Rust 端 ApiProviderType 的 serde 序列化一致（lowercase）
 */
export enum ApiProviderType {
  OPENAI = 'openai',
  ANTHROPIC = 'anthropic',
  OLLAMA = 'ollama',
}

/**
 * API 提供商配置
 */
export interface ApiProvider {
  id?: number;
  providerType: ApiProviderType;
  name: string;
  baseUrl: string;
  apiKeyRef?: string;
  configJson?: string;
  isActive: boolean;
}

/**
 * 提供商响应（包含敏感信息掩码）
 */
export interface ProviderResponse {
  id?: number;
  providerType: ApiProviderType;
  name: string;
  baseUrl: string;
  apiKeyRef?: string;
  configJson?: string;
  isActive: boolean;
  hasApiKey: boolean;
  apiKeyMask?: string;
}

/**
 * 保存提供商的请求参数
 */
export interface SaveProviderRequest {
  id?: number;
  providerType: ApiProviderType;
  name: string;
  baseUrl: string;
  apiKey?: string;
  configJson?: string;
  isActive: boolean;
}

// ==================== Store State ====================

interface SettingsState {
  // 数据状态
  providers: ProviderResponse[];
  activeProviderId: number | null;
  loading: boolean;
  error: string | null;

  // Actions
  fetchProviders: () => Promise<void>;
  saveProvider: (request: SaveProviderRequest) => Promise<ApiProvider>;
  deleteProvider: (id: number) => Promise<void>;
  setActiveProvider: (id: number) => Promise<void>;
  testProviderConnection: (id: number) => Promise<boolean>;
  clearError: () => void;
}

// ==================== Store 实现 ====================

export const useSettingsStore = create<SettingsState>()(
  immer((set, get) => ({
    // 初始状态
    providers: [],
    activeProviderId: null,
    loading: false,
    error: null,

    // 获取所有提供商
    fetchProviders: async () => {
      debugLog('fetchProviders', 'start');
      set((state) => {
        state.loading = true;
        state.error = null;
      });

      try {
        debugLog('fetchProviders', 'invoking cmd_get_providers');
        const providers = await invoke<ProviderResponse[]>('cmd_get_providers');
        debugLog('fetchProviders', 'success', providers);

        set((state) => {
          state.providers = providers;
          state.activeProviderId = providers.find((p) => p.isActive)?.id ?? null;
          state.loading = false;
        });
      } catch (error) {
        debugLog('fetchProviders', 'error', error);
        set((state) => {
          state.error = `获取提供商列表失败: ${parseError(error)}`;
          state.loading = false;
        });
        throw error;
      }
    },

    // 保存提供商（创建或更新）
    saveProvider: async (request) => {
      set((state) => {
        state.loading = true;
        state.error = null;
      });

      try {
        const provider = await invoke<ApiProvider>('cmd_save_provider', {
          request: {
            id: request.id,
            providerType: request.providerType,
            name: request.name,
            baseUrl: request.baseUrl,
            apiKey: request.apiKey,
            configJson: request.configJson,
            isActive: request.isActive,
          },
        });

        // 如果保存成功，刷新列表
        await get().fetchProviders();

        set((state) => {
          state.loading = false;
        });

        return provider;
      } catch (error) {
        set((state) => {
          state.error = `保存提供商失败: ${parseError(error)}`;
          state.loading = false;
        });
        throw error;
      }
    },

    // 删除提供商
    deleteProvider: async (id) => {
      set((state) => {
        state.loading = true;
        state.error = null;
      });

      try {
        await invoke('cmd_delete_provider', { id });

        // 刷新列表
        await get().fetchProviders();

        set((state) => {
          state.loading = false;
        });
      } catch (error) {
        set((state) => {
          state.error = `删除提供商失败: ${parseError(error)}`;
          state.loading = false;
        });
        throw error;
      }
    },

    // 设置活跃提供商
    setActiveProvider: async (id) => {
      set((state) => {
        state.loading = true;
        state.error = null;
      });

      try {
        await invoke('cmd_set_active_provider', { id });

        // 刷新列表
        await get().fetchProviders();

        set((state) => {
          state.loading = false;
        });
      } catch (error) {
        set((state) => {
          state.error = `设置活跃提供商失败: ${parseError(error)}`;
          state.loading = false;
        });
        throw error;
      }
    },

    // 测试提供商连接
    testProviderConnection: async (id) => {
      try {
        const success = await invoke<boolean>('cmd_test_provider_connection', { id });
        return success;
      } catch (error) {
        set((state) => {
          state.error = `测试连接失败: ${parseError(error)}`;
        });
        return false;
      }
    },

    // 清除错误
    clearError: () => {
      set((state) => {
        state.error = null;
      });
    },
  }))
);

// ==================== 便捷 Hooks ====================

/**
 * 获取所有提供商
 */
export const useProviders = () => useSettingsStore((state) => state.providers);

/**
 * 获取活跃提供商
 */
export const useActiveProvider = () => {
  const providers = useProviders();
  const activeProviderId = useSettingsStore((state) => state.activeProviderId);
  return providers.find((p) => p.id === activeProviderId);
};

/**
 * 获取加载状态
 */
export const useProvidersLoading = () => useSettingsStore((state) => state.loading);

/**
 * 获取错误信息
 */
export const useProvidersError = () => useSettingsStore((state) => state.error);

/**
 * 获取 store actions（稳定引用）
 * 
 * 注意：直接从 store 获取 actions，这些函数引用是稳定的
 * 不要在组件中创建新对象包装这些函数，否则会导致无限循环
 */
export const useProviderActions = () => {
  const store = useSettingsStore;
  
  // 使用 useMemo 确保返回稳定的对象引用
  return useMemo(() => ({
    fetchProviders: store.getState().fetchProviders,
    saveProvider: store.getState().saveProvider,
    deleteProvider: store.getState().deleteProvider,
    setActiveProvider: store.getState().setActiveProvider,
    testProviderConnection: store.getState().testProviderConnection,
    clearError: store.getState().clearError,
  }), []);
};
