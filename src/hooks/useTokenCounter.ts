/**
 * useTokenCounter Hook
 *
 * 调用后端 Token 计数 API，计算文本的 Token 数量和费用估算
 */

import { useState, useCallback, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';

/**
 * Token 计数响应
 */
export interface TokenCountResult {
  /** Token 数量 */
  tokenCount: number;
  /** 使用的编码类型 */
  encodingType: string;
  /** 模型名称（如果提供） */
  model?: string;
}

/**
 * Token 计数请求参数
 */
export interface CountTokensParams {
  /** 要计算的文本内容 */
  text: string;
  /** 模型名称（可选，用于自动选择编码类型） */
  model?: string;
  /** 手动指定编码类型（优先级高于 model） */
  encodingType?: string;
}

/**
 * LLM 模型定价配置（每 1M tokens 价格，单位：美元）
 */
export interface ModelPricing {
  /** 输入价格（每 1M tokens） */
  inputPrice: number;
  /** 输出价格（每 1M tokens） */
  outputPrice: number;
  /** 货币单位 */
  currency: string;
}

/**
 * 预定义的模型定价
 */
export const MODEL_PRICING: Record<string, ModelPricing> = {
  // OpenAI 模型
  'gpt-4o': {
    inputPrice: 2.50,
    outputPrice: 10.00,
    currency: 'USD',
  },
  'gpt-4o-mini': {
    inputPrice: 0.15,
    outputPrice: 0.60,
    currency: 'USD',
  },
  'gpt-4-turbo': {
    inputPrice: 10.00,
    outputPrice: 30.00,
    currency: 'USD',
  },
  'gpt-4': {
    inputPrice: 30.00,
    outputPrice: 60.00,
    currency: 'USD',
  },
  'gpt-3.5-turbo': {
    inputPrice: 0.50,
    outputPrice: 1.50,
    currency: 'USD',
  },

  // Anthropic 模型
  'claude-3-5-sonnet-20241022': {
    inputPrice: 3.00,
    outputPrice: 15.00,
    currency: 'USD',
  },
  'claude-3-5-sonnet-20240620': {
    inputPrice: 3.00,
    outputPrice: 15.00,
    currency: 'USD',
  },
  'claude-3-opus-20240229': {
    inputPrice: 15.00,
    outputPrice: 75.00,
    currency: 'USD',
  },
  'claude-3-sonnet-20240229': {
    inputPrice: 3.00,
    outputPrice: 15.00,
    currency: 'USD',
  },
  'claude-3-haiku-20240307': {
    inputPrice: 0.25,
    outputPrice: 1.25,
    currency: 'USD',
  },

  // xAI 模型
  'grok-2': {
    inputPrice: 2.00,
    outputPrice: 10.00,
    currency: 'USD',
  },

  // 默认定价
  'default': {
    inputPrice: 1.00,
    outputPrice: 2.00,
    currency: 'USD',
  },
};

/**
 * useTokenCounter Hook
 *
 * @example
 * const { countTokens, result, loading, error } = useTokenCounter();
 * await countTokens({ text: 'Hello, world!', model: 'gpt-4' });
 */
export function useTokenCounter() {
  const [result, setResult] = useState<TokenCountResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /**
   * 计算 Token 数量
   */
  const countTokens = useCallback(async (params: CountTokensParams): Promise<TokenCountResult | null> => {
    setLoading(true);
    setError(null);

    try {
      const response = await invoke<{
        token_count: number;
        encoding_type: string;
        model?: string;
      }>('count_prompt_tokens', {
        text: params.text,
        model: params.model,
        encodingType: params.encodingType,
      });

      const result: TokenCountResult = {
        tokenCount: response.token_count,
        encodingType: response.encoding_type,
        model: response.model,
      };

      setResult(result);
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Token 计数失败: ${errorMessage}`);
      return null;
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * 计算费用估算
   */
  const estimateCost = useCallback((
    tokenCount: number,
    model: string,
    isOutput: boolean = false
  ): number => {
    const pricing = MODEL_PRICING[model] || MODEL_PRICING['default'];
    const price = isOutput ? pricing.outputPrice : pricing.inputPrice;
    return (tokenCount / 1_000_000) * price;
  }, []);

  /**
   * 格式化 Token 数量为千分位
   */
  const formatTokenCount = useCallback((count: number): string => {
    return count.toLocaleString('zh-CN');
  }, []);

  /**
   * 格式化费用为美元
   */
  const formatCost = useCallback((cost: number): string => {
    return `$${cost.toFixed(4)}`;
  }, []);

  return {
    /** Token 计数结果 */
    result,
    /** 是否正在加载 */
    loading,
    /** 错误信息 */
    error,
    /** 计算 Token 数量 */
    countTokens,
    /** 估算费用 */
    estimateCost,
    /** 格式化 Token 数量 */
    formatTokenCount,
    /** 格式化费用 */
    formatCost,
  };
}

/**
 * 使用文本内容的实时 Token 计数 Hook
 *
 * @example
 * const { tokenCount, cost } = useRealTimeTokenCount('Hello, world!', 'gpt-4');
 */
export function useRealTimeTokenCount(text: string, model?: string) {
  const { estimateCost, formatTokenCount, formatCost } = useTokenCounter();

  const stats = useMemo(() => {
    if (!text) {
      return { tokenCount: 0, cost: 0, formatted: '0', formattedCost: '$0.0000' };
    }

    // 简单估算（避免每次都调用后端）
    // 中文约 2 token/字，英文约 0.25 token/词
    let estimatedTokens = 0;
    for (const ch of text) {
      if (ch.charCodeAt(0) > 127) {
        // 非 ASCII 字符（中文等）
        estimatedTokens += 2;
      } else if (ch === ' ') {
        estimatedTokens += 0.25;
      } else {
        estimatedTokens += 0.25;
      }
    }

    const tokenCount = Math.round(estimatedTokens);
    const cost = model ? estimateCost(tokenCount, model) : 0;

    return {
      tokenCount,
      cost,
      formatted: formatTokenCount(tokenCount),
      formattedCost: formatCost(cost),
    };
  }, [text, model, estimateCost, formatTokenCount, formatCost]);

  return stats;
}
