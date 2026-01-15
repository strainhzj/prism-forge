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
  XAI = 'xai',
  GOOGLE = 'google',
  GOOGLE_VERTEX = 'googlevertex',
  AZURE_OPENAI = 'azureopenai',
  OPENAI_COMPATIBLE = 'openai-compatible',
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
  temperature?: number;
  maxTokens?: number;
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
  temperature?: number;
  maxTokens?: number;
  isActive: boolean;
  hasApiKey: boolean;
  apiKeyMask?: string;
  model?: string;
  aliases?: string; // JSON 数组格式，如 ["claude", "anthropic-api"]
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
  temperature?: number;
  maxTokens?: number;
  isActive: boolean;
  model?: string;
  aliases?: string; // JSON 数组格式
}

/**
 * 提供商显示信息（用于 UI 展示，不影响后端序列化）
 */
export interface ProviderDisplayInfo {
  /** 显示名称 */
  label: string;
  /** 描述 */
  description: string;
  /** 默认 Base URL */
  defaultBaseUrl: string;
  /** 默认模型 */
  defaultModel: string;
  /** 是否需要 API Key */
  requiresApiKey: boolean;
  /** 官网链接 */
  websiteUrl?: string;
  /** API Key 获取链接 */
  apiKeyUrl?: string;
  /** 文档链接 */
  docsUrl?: string;
}

/**
 * 提供商显示信息映射
 * 参考 Cherry Studio 的提供商配置
 */
export const PROVIDER_DISPLAY_INFO: Record<ApiProviderType, ProviderDisplayInfo> = {
  // ========== 国际主流提供商 ==========

  [ApiProviderType.OPENAI]: {
    label: 'OpenAI',
    description: 'OpenAI 官方 API，支持 GPT-4、GPT-3.5 等模型',
    defaultBaseUrl: 'https://api.openai.com/v1',
    defaultModel: 'gpt-4o-mini',
    requiresApiKey: true,
    websiteUrl: 'https://openai.com/',
    apiKeyUrl: 'https://platform.openai.com/api-keys',
    docsUrl: 'https://platform.openai.com/docs',
  },

  [ApiProviderType.ANTHROPIC]: {
    label: 'Anthropic',
    description: 'Anthropic Claude 系列，包括 Claude 3.5 Sonnet',
    defaultBaseUrl: 'https://api.anthropic.com',
    defaultModel: 'claude-3-5-sonnet-20241022',
    requiresApiKey: true,
    websiteUrl: 'https://anthropic.com/',
    apiKeyUrl: 'https://console.anthropic.com/settings/keys',
    docsUrl: 'https://docs.anthropic.com/en/docs',
  },

  [ApiProviderType.GOOGLE]: {
    label: 'Google Gemini',
    description: 'Google Gemini (ML Dev API)，使用 API Key 认证',
    defaultBaseUrl: 'https://generativelanguage.googleapis.com',
    defaultModel: 'gemini-2.5-flash-lite',
    requiresApiKey: true,
    websiteUrl: 'https://gemini.google.com/',
    apiKeyUrl: 'https://aistudio.google.com/app/apikey',
    docsUrl: 'https://ai.google.dev/gemini-api/docs',
  },

  [ApiProviderType.GOOGLE_VERTEX]: {
    label: 'Google Vertex AI',
    description: 'Google Vertex AI Public Preview (API Key URL 参数)',
    defaultBaseUrl: 'https://aiplatform.googleapis.com',
    defaultModel: 'gemini-2.5-flash-lite',
    requiresApiKey: true,
    websiteUrl: 'https://cloud.google.com/vertex-ai',
    apiKeyUrl: 'https://console.cloud.google.com/apis/credentials',
    docsUrl: 'https://cloud.google.com/vertex-ai/generative-ai/docs',
  },

  [ApiProviderType.AZURE_OPENAI]: {
    label: 'Azure OpenAI',
    description: 'Microsoft Azure OpenAI 服务',
    defaultBaseUrl: 'https://{your-resource-name}.openai.azure.com/openai/deployments/{deployment}?api-version=2024-02-01',
    defaultModel: 'gpt-4o-mini',
    requiresApiKey: true,
    websiteUrl: 'https://azure.microsoft.com/en-us/products/ai-services/openai-service',
    apiKeyUrl: 'https://portal.azure.com/#view/Microsoft_Azure_ProjectOxford/CognitiveServicesHub/~/OpenAI',
    docsUrl: 'https://learn.microsoft.com/en-us/azure/ai-services/openai/',
  },

  [ApiProviderType.OLLAMA]: {
    label: 'Ollama',
    description: '本地 Ollama 服务，无需 API Key',
    defaultBaseUrl: 'http://127.0.0.1:11434',
    defaultModel: 'llama3',
    requiresApiKey: false,
    websiteUrl: 'https://ollama.com/',
    docsUrl: 'https://github.com/ollama/ollama/tree/main/docs',
  },

  [ApiProviderType.XAI]: {
    label: 'X AI (Grok)',
    description: 'xAI Grok 系列，支持 Grok-2 等模型',
    defaultBaseUrl: 'https://api.x.ai/v1',
    defaultModel: 'grok-4-1-fast-reasoning',
    requiresApiKey: true,
    websiteUrl: 'https://x.ai/',
    docsUrl: 'https://docs.x.ai/',
  },

  // ========== OpenAI 兼容类型（包括所有第三方服务）==========

  [ApiProviderType.OPENAI_COMPATIBLE]: {
    label: 'OpenAI Compatible / 第三方服务',
    description: '兼容 OpenAI API 格式的第三方服务（DeepSeek、硅基流动、智谱等）',
    defaultBaseUrl: 'https://api.example.com/v1',
    defaultModel: 'gpt-4o-mini',
    requiresApiKey: true,
  },
};

/**
 * 常用第三方服务商预设配置
 * 用户可以在选择 "OpenAI Compatible" 后快速选择这些预设
 */
export const THIRD_PARTY_PROVIDERS = [
  {
    id: 'deepseek',
    name: 'DeepSeek (深度求索)',
    baseUrl: 'https://api.deepseek.com',
    defaultModel: 'deepseek-chat',
    description: 'DeepSeek-V3、DeepSeek-Coder',
    websiteUrl: 'https://deepseek.com/',
    apiKeyUrl: 'https://platform.deepseek.com/api_keys',
  },
  {
    id: 'silicon',
    name: 'SiliconFlow (硅基流动)',
    baseUrl: 'https://api.siliconflow.cn',
    defaultModel: 'Qwen/Qwen2.5-72B-Instruct',
    description: '提供 Qwen、DeepSeek、GLM 等',
    websiteUrl: 'https://www.siliconflow.cn',
    apiKeyUrl: 'https://cloud.siliconflow.cn',
  },
  {
    id: 'zhipu',
    name: 'Zhipu AI (智谱)',
    baseUrl: 'https://open.bigmodel.cn/api/paas/v4',
    defaultModel: 'glm-4-flash',
    description: 'GLM-4、GLM-3 系列',
    websiteUrl: 'https://open.bigmodel.cn/',
    apiKeyUrl: 'https://open.bigmodel.cn/usercenter/apikeys',
  },
  {
    id: 'moonshot',
    name: 'Moonshot AI (月之暗面)',
    baseUrl: 'https://api.moonshot.cn',
    defaultModel: 'moonshot-v1-8k',
    description: 'Kimi 系列模型',
    websiteUrl: 'https://www.moonshot.cn/',
    apiKeyUrl: 'https://platform.moonshot.cn/console/api-keys',
  },
  {
    id: 'groq',
    name: 'Groq',
    baseUrl: 'https://api.groq.com/openai',
    defaultModel: 'llama-3.3-70b-versatile',
    description: '超快推理引擎，Llama、Mixtral',
    websiteUrl: 'https://groq.com/',
    apiKeyUrl: 'https://console.groq.com/keys',
  },
  {
    id: 'openrouter',
    name: 'OpenRouter',
    baseUrl: 'https://openrouter.ai/api/v1',
    defaultModel: 'anthropic/claude-3.5-sonnet',
    description: '统一 API 访问多种模型',
    websiteUrl: 'https://openrouter.ai/',
    apiKeyUrl: 'https://openrouter.ai/settings/keys',
  },
  {
    id: 'together',
    name: 'Together',
    baseUrl: 'https://api.together.xyz',
    defaultModel: 'meta-llama/Llama-3.3-70B-Instruct-Turbo',
    description: '开源模型托管服务',
    websiteUrl: 'https://www.together.ai/',
    apiKeyUrl: 'https://api.together.ai/settings/api-keys',
  },
];

/**
 * 每种提供商类型的默认模型
 */
export const DEFAULT_MODELS: Record<ApiProviderType, string> = {
  [ApiProviderType.OPENAI]: 'gpt-4o-mini',
  [ApiProviderType.ANTHROPIC]: 'claude-3-5-sonnet-20241022',
  [ApiProviderType.OLLAMA]: 'llama3',
  [ApiProviderType.XAI]: 'grok-4-1-fast-reasoning',
  [ApiProviderType.GOOGLE]: 'gemini-2.5-flash-lite',
  [ApiProviderType.GOOGLE_VERTEX]: 'gemini-2.5-flash-lite',
  [ApiProviderType.AZURE_OPENAI]: 'gpt-4o-mini',
  [ApiProviderType.OPENAI_COMPATIBLE]: 'gpt-4o-mini',
};

/**
 * 获取提供商的默认 Base URL
 */
export function getDefaultBaseUrl(providerType: ApiProviderType): string {
  return PROVIDER_DISPLAY_INFO[providerType]?.defaultBaseUrl || '';
}

/**
 * 获取提供商的默认模型
 */
export function getDefaultModel(providerType: ApiProviderType): string {
  return PROVIDER_DISPLAY_INFO[providerType]?.defaultModel || '';
}

/**
 * 获取提供商显示信息
 */
export function getProviderDisplayInfo(providerType: ApiProviderType): ProviderDisplayInfo {
  return PROVIDER_DISPLAY_INFO[providerType];
}

/**
 * 获取提供商类型的翻译键映射
 * 用于在组件中动态获取翻译
 */
export const PROVIDER_TYPE_KEYS: Record<ApiProviderType, string> = {
  [ApiProviderType.OPENAI]: 'openai',
  [ApiProviderType.ANTHROPIC]: 'anthropic',
  [ApiProviderType.GOOGLE]: 'google',
  [ApiProviderType.GOOGLE_VERTEX]: 'googleVertex',
  [ApiProviderType.AZURE_OPENAI]: 'azureOpenai',
  [ApiProviderType.OLLAMA]: 'ollama',
  [ApiProviderType.XAI]: 'xai',
  [ApiProviderType.OPENAI_COMPATIBLE]: 'openaiCompatible',
};

/**
 * 第三方提供商的翻译键映射
 */
export const THIRD_PARTY_PROVIDER_KEYS: Record<string, string> = {
  'deepseek': 'deepseek',
  'silicon': 'silicon',
  'zhipu': 'zhipu',
  'moonshot': 'moonshot',
  'groq': 'groq',
  'openrouter': 'openrouter',
  'together': 'together',
};

/**
 * 连接错误类型
 */
export enum ConnectionErrorType {
  AUTHENTICATION = 'authentication',
  NETWORK = 'network',
  SERVER = 'server',
  REQUEST = 'request',
  UNKNOWN = 'unknown',
}

/**
 * 连接测试结果
 */
export interface TestConnectionResult {
  success: boolean;
  errorMessage?: string;
  errorType?: ConnectionErrorType;
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
  testProviderConnection: (id: number) => Promise<TestConnectionResult>;
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
            temperature: request.temperature,
            maxTokens: request.maxTokens,
            isActive: request.isActive,
            model: request.model,
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
        const result = await invoke<TestConnectionResult>('cmd_test_provider_connection', { id });
        return result;
      } catch (error) {
        set((state) => {
          state.error = `测试连接失败: ${parseError(error)}`;
        });
        return {
          success: false,
          errorMessage: parseError(error),
          errorType: ConnectionErrorType.UNKNOWN,
        };
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
