/**
 * TokenCounter 组件
 *
 * 显示 Token 数量、费用估算和相关信息
 */

import { useMemo } from 'react';
import { Coins, TrendingUp, TrendingDown, AlertTriangle } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Badge } from '@/components/ui/badge';
import { useRealTimeTokenCount, MODEL_PRICING, type ModelPricing } from '@/hooks/useTokenCounter';

export interface TokenCounterProps {
  /** 文本内容 */
  text: string;
  /** 模型名称（用于定价） */
  model?: string;
  /** 是否显示详细信息 */
  showDetails?: boolean;
  /** 是否显示费用估算 */
  showCost?: boolean;
  /** 是否显示节省百分比（与基准模型对比） */
  showSavings?: boolean;
  /** 基准模型（用于计算节省百分比） */
  baselineModel?: string;
  /** 自定义类名 */
  className?: string;
}

/**
 * 获取模型定价
 */
function getModelPricing(model?: string): ModelPricing | null {
  if (!model) return null;
  return MODEL_PRICING[model] || MODEL_PRICING['default'];
}

/**
 * TokenCounter 组件
 *
 * @example
 * <TokenCounter
 *   text="Hello, world!"
 *   model="gpt-4"
 *   showCost={true}
 * />
 */
export function TokenCounter({
  text,
  model,
  showDetails = true,
  showCost = true,
  showSavings = false,
  baselineModel = 'gpt-4',
  className,
}: TokenCounterProps) {
  const stats = useRealTimeTokenCount(text, model);

  // 计算节省百分比
  const savings = useMemo(() => {
    if (!showSavings || !model || model === baselineModel) return null;

    const currentPricing = getModelPricing(model);
    const baselinePricing = getModelPricing(baselineModel);

    if (!currentPricing || !baselinePricing) return null;

    // 假设 50% 输入，50% 输出
    const currentCost = (currentPricing.inputPrice * 0.5 + currentPricing.outputPrice * 0.5) / 1_000_000 * stats.tokenCount;
    const baselineCost = (baselinePricing.inputPrice * 0.5 + baselinePricing.outputPrice * 0.5) / 1_000_000 * stats.tokenCount;

    if (baselineCost === 0) return null;

    const savingsPercent = ((baselineCost - currentCost) / baselineCost) * 100;
    return {
      percent: savingsPercent,
      currentCost,
      baselineCost,
    };
  }, [showSavings, model, baselineModel, stats.tokenCount]);

  // 获取定价信息
  const pricing = useMemo(() => getModelPricing(model), [model]);

  return (
    <div className={cn('flex items-center gap-3', className)}>
      {/* Token 数量 */}
      <div className="flex items-center gap-2">
        <Coins className="h-4 w-4 text-primary" />
        <span className="text-sm font-medium">{stats.formatted}</span>
        <span className="text-xs text-muted-foreground">tokens</span>
      </div>

      {/* 费用估算 */}
      {showCost && pricing && (
        <div className="flex items-center gap-2">
          <Badge variant="secondary" className="text-xs">
            {stats.formattedCost}
          </Badge>
          {model && (
            <span className="text-xs text-muted-foreground">
              ({model})
            </span>
          )}
        </div>
      )}

      {/* 详细信息 */}
      {showDetails && pricing && (
        <div className="flex items-center gap-3 text-xs text-muted-foreground">
          <span>
            输入: ${pricing.inputPrice.toFixed(2)}/M
          </span>
          <span>
            输出: ${pricing.outputPrice.toFixed(2)}/M
          </span>
        </div>
      )}

      {/* 节省百分比 */}
      {showSavings && savings && savings.percent !== 0 && (
        <div
          className={cn(
            'flex items-center gap-1 text-xs font-medium',
            savings.percent > 0 ? 'text-green-600' : 'text-red-600'
          )}
        >
          {savings.percent > 0 ? (
            <TrendingDown className="h-3 w-3" />
          ) : (
            <TrendingUp className="h-3 w-3" />
          )}
          <span>
            {savings.percent > 0 ? '节省' : '增加'} {Math.abs(savings.percent).toFixed(1)}%
          </span>
        </div>
      )}
    </div>
  );
}

/**
 * 紧凑型 Token 计数器（仅显示 Token 数）
 */
export interface TokenCounterCompactProps {
  text: string;
  model?: string;
  className?: string;
}

export function TokenCounterCompact({ text, model, className }: TokenCounterCompactProps) {
  const stats = useRealTimeTokenCount(text, model);

  return (
    <div className={cn('flex items-center gap-1.5 text-xs', className)}>
      <Coins className="h-3 w-3 text-muted-foreground" />
      <span className="font-medium">{stats.formatted}</span>
      <span className="text-muted-foreground">tokens</span>
    </div>
  );
}

/**
 * Token 使用进度条
 */
export interface TokenProgressBarProps {
  /** 当前 Token 数 */
  current: number;
  /** 最大 Token 数（上下文限制） */
  max: number;
  /** 警告阈值（百分比） */
  warningThreshold?: number;
  /** 自定义类名 */
  className?: string;
}

export function TokenProgressBar({
  current,
  max,
  warningThreshold = 80,
  className,
}: TokenProgressBarProps) {
  const percentage = useMemo(() => {
    if (max === 0) return 0;
    return Math.min((current / max) * 100, 100);
  }, [current, max]);

  const isWarning = percentage >= warningThreshold;

  return (
    <div className={cn('flex items-center gap-2', className)}>
      <div className="flex-1 h-2 bg-muted rounded-full overflow-hidden">
        <div
          className={cn(
            'h-full transition-all duration-300',
            isWarning ? 'bg-red-500' : 'bg-primary'
          )}
          style={{ width: `${percentage}%` }}
        />
      </div>
      <div className="flex items-center gap-1 text-xs">
        {isWarning && <AlertTriangle className="h-3 w-3 text-red-500" />}
        <span className={cn('font-medium', isWarning ? 'text-red-500' : '')}>
          {percentage.toFixed(1)}%
        </span>
        <span className="text-muted-foreground">
          ({current.toLocaleString()} / {max.toLocaleString()})
        </span>
      </div>
    </div>
  );
}

/**
 * Token 使用统计卡片
 */
export interface TokenStatsCardProps {
  /** 输入 Token 数 */
  inputTokens: number;
  /** 输出 Token 数 */
  outputTokens: number;
  /** 模型名称 */
  model?: string;
  /** 自定义类名 */
  className?: string;
}

export function TokenStatsCard({
  inputTokens,
  outputTokens,
  model,
  className,
}: TokenStatsCardProps) {
  const pricing = useMemo(() => getModelPricing(model), [model]);

  const totalTokens = inputTokens + outputTokens;
  const inputCost = pricing ? (inputTokens / 1_000_000) * pricing.inputPrice : 0;
  const outputCost = pricing ? (outputTokens / 1_000_000) * pricing.outputPrice : 0;
  const totalCost = inputCost + outputCost;

  return (
    <div className={cn('rounded-lg border bg-card p-4', className)}>
      <div className="space-y-3">
        {/* 标题 */}
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-semibold">Token 使用统计</h3>
          {model && (
            <Badge variant="outline" className="text-xs">
              {model}
            </Badge>
          )}
        </div>

        {/* Token 数量 */}
        <div className="grid grid-cols-3 gap-4">
          <div>
            <div className="text-xs text-muted-foreground">输入</div>
            <div className="text-lg font-semibold">{inputTokens.toLocaleString()}</div>
          </div>
          <div>
            <div className="text-xs text-muted-foreground">输出</div>
            <div className="text-lg font-semibold">{outputTokens.toLocaleString()}</div>
          </div>
          <div>
            <div className="text-xs text-muted-foreground">总计</div>
            <div className="text-lg font-semibold">{totalTokens.toLocaleString()}</div>
          </div>
        </div>

        {/* 费用明细 */}
        {pricing && (
          <div className="pt-3 border-t space-y-1">
            <div className="flex justify-between text-xs">
              <span className="text-muted-foreground">输入费用</span>
              <span className="font-medium">${inputCost.toFixed(4)}</span>
            </div>
            <div className="flex justify-between text-xs">
              <span className="text-muted-foreground">输出费用</span>
              <span className="font-medium">${outputCost.toFixed(4)}</span>
            </div>
            <div className="flex justify-between text-sm font-semibold pt-1 border-t">
              <span>总费用</span>
              <span>${totalCost.toFixed(4)}</span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
