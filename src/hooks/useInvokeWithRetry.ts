/**
 * useInvokeWithRetry Hook
 *
 * 提供带重试机制的 Tauri invoke 调用
 */

import { useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface InvokeRetryOptions {
  maxRetries?: number;
  retryDelay?: number;
  onRetry?: (attempt: number, error: Error) => void;
}

/**
 * 带重试机制的 invoke Hook
 *
 * @example
 * const { invokeWithRetry } = useInvokeWithRetry();
 *
 * try {
 *   const result = await invokeWithRetry('cmd_get_prompts', { scenario: 'session_analysis' });
 * } catch (error) {
 *   console.error('最终失败:', error);
 * }
 */
export function useInvokeWithRetry(defaultOptions: InvokeRetryOptions = {}) {
  const {
    maxRetries = 2,
    retryDelay = 1000,
    onRetry,
  } = defaultOptions;

  const invokeWithRetry = useCallback(
    async <T,>(
      command: string,
      args?: Record<string, any>,
      options: InvokeRetryOptions = {}
    ): Promise<T> => {
      const mergedOptions = { ...defaultOptions, ...options };
      const { maxRetries: retries = maxRetries, retryDelay: delay = retryDelay, onRetry: retryCallback } = mergedOptions;

      let lastError: Error | null = null;

      for (let attempt = 0; attempt <= retries!; attempt++) {
        try {
          return await invoke<T>(command, args);
        } catch (error) {
          lastError = error instanceof Error ? error : new Error(String(error));

          // 如果是最后一次尝试，抛出错误
          if (attempt === retries!) {
            throw lastError;
          }

          // 调用重试回调
          if (retryCallback) {
            retryCallback(attempt + 1, lastError);
          }

          // 等待后重试
          await new Promise(resolve => setTimeout(resolve, delay! * (attempt + 1)));
        }
      }

      // TypeScript 类型守卫，确保一定会返回或抛出
      throw lastError || new Error('Unknown error');
    },
    [maxRetries, retryDelay, onRetry]
  );

  return { invokeWithRetry };
}
