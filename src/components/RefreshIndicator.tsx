/**
 * RefreshIndicator 组件
 *
 * 显示会话文件变更时的刷新指示器和动画
 */

import { useMemo } from 'react';
import { RefreshCw, Clock, CheckCircle2 } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Badge } from '@/components/ui/badge';

export interface RefreshIndicatorProps {
  /** 是否正在刷新 */
  isRefreshing?: boolean;
  /** 待处理的变更数量 */
  pendingChanges?: number;
  /** 最后一次变更时间 */
  lastChangeTime?: string;
  /** 显示模式 */
  variant?: 'default' | 'compact' | 'minimal';
  /** 自定义类名 */
  className?: string;
}

/**
 * 格式化时间差
 */
function formatTimeDiff(timestamp: string): string {
  if (!timestamp) return '';

  const now = Date.now();
  const time = new Date(timestamp).getTime();
  const diff = now - time;

  if (diff < 1000) return '刚刚';
  if (diff < 60000) return `${Math.floor(diff / 1000)}秒前`;
  if (diff < 3600000) return `${Math.floor(diff / 60000)}分钟前`;
  return `${Math.floor(diff / 3600000)}小时前`;
}

/**
 * RefreshIndicator 组件
 *
 * @example
 * <RefreshIndicator
 *   isRefreshing={true}
 *   pendingChanges={3}
 * />
 */
export function RefreshIndicator({
  isRefreshing = false,
  pendingChanges = 0,
  lastChangeTime,
  variant = 'default',
  className,
}: RefreshIndicatorProps) {
  // 计算显示状态
  const status = useMemo(() => {
    if (isRefreshing) return 'refreshing';
    if (pendingChanges > 0) return 'pending';
    return 'idle';
  }, [isRefreshing, pendingChanges]);

  // 根据模式渲染不同样式
  if (variant === 'minimal') {
    return (
      <div className={cn('flex items-center gap-2', className)}>
        {isRefreshing ? (
          <RefreshCw className="h-4 w-4 animate-spin text-primary" />
        ) : pendingChanges > 0 ? (
          <Badge variant="secondary" className="text-xs">
            {pendingChanges} 个变更
          </Badge>
        ) : null}
      </div>
    );
  }

  if (variant === 'compact') {
    return (
      <div className={cn('flex items-center gap-2 text-xs', className)}>
        {isRefreshing ? (
          <>
            <RefreshCw className="h-3 w-3 animate-spin text-primary" />
            <span className="text-muted-foreground">刷新中...</span>
          </>
        ) : pendingChanges > 0 ? (
          <>
            <Clock className="h-3 w-3 text-orange-500" />
            <span className="text-muted-foreground">
              {pendingChanges} 个变更，即将刷新...
            </span>
          </>
        ) : lastChangeTime ? (
          <>
            <CheckCircle2 className="h-3 w-3 text-green-500" />
            <span className="text-muted-foreground">
              {formatTimeDiff(lastChangeTime)} 更新
            </span>
          </>
        ) : null}
      </div>
    );
  }

  // 默认模式
  return (
    <div
      className={cn(
        'flex items-center gap-3 px-3 py-2 rounded-lg border bg-card transition-colors',
        status === 'refreshing' && 'border-primary/50 bg-primary/5',
        status === 'pending' && 'border-orange-500/50 bg-orange-500/5',
        className
      )}
    >
      {/* 图标 */}
      <div className="shrink-0">
        {isRefreshing ? (
          <RefreshCw className="h-4 w-4 animate-spin text-primary" />
        ) : pendingChanges > 0 ? (
          <Clock className="h-4 w-4 text-orange-500" />
        ) : (
          <CheckCircle2 className="h-4 w-4 text-green-500" />
        )}
      </div>

      {/* 文本 */}
      <div className="flex-1 min-w-0">
        {isRefreshing ? (
          <p className="text-sm font-medium">正在刷新会话列表...</p>
        ) : pendingChanges > 0 ? (
          <p className="text-sm font-medium">
            检测到 {pendingChanges} 个变更
          </p>
        ) : lastChangeTime ? (
          <p className="text-sm text-muted-foreground">
            {formatTimeDiff(lastChangeTime)} 更新
          </p>
        ) : (
          <p className="text-sm text-muted-foreground">等待变更...</p>
        )}
      </div>

      {/* 徽章 */}
      {pendingChanges > 0 && (
        <Badge variant="secondary" className="shrink-0 text-xs">
          {pendingChanges}
        </Badge>
      )}
    </div>
  );
}

/**
 * 刷新按钮（带指示器）
 */
export interface RefreshButtonProps {
  /** 是否正在刷新 */
  isRefreshing?: boolean;
  /** 待处理的变更数量 */
  pendingChanges?: number;
  /** 点击回调 */
  onClick?: () => void;
  /** 自定义类名 */
  className?: string;
}

export function RefreshButton({
  isRefreshing = false,
  pendingChanges = 0,
  onClick,
  className,
}: RefreshButtonProps) {
  return (
    <button
      onClick={onClick}
      disabled={isRefreshing}
      className={cn(
        'relative inline-flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium transition-colors',
        'hover:bg-accent',
        'focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2',
        'disabled:opacity-50 disabled:cursor-not-allowed',
        className
      )}
    >
      <RefreshCw
        className={cn(
          'h-4 w-4',
          isRefreshing && 'animate-spin text-primary'
        )}
      />
      <span>{isRefreshing ? '刷新中...' : '刷新'}</span>

      {/* 待处理变更徽章 */}
      {pendingChanges > 0 && !isRefreshing && (
        <span className="absolute -top-1 -right-1 h-5 w-5 flex items-center justify-center rounded-full bg-primary text-primary-foreground text-xs font-medium">
          {pendingChanges > 9 ? '9+' : pendingChanges}
        </span>
      )}
    </button>
  );
}

/**
 * 内联刷新指示器（用于工具栏）
 */
export interface InlineRefreshIndicatorProps {
  /** 是否正在刷新 */
  isRefreshing?: boolean;
  /** 待处理的变更数量 */
  pendingChanges?: number;
  /** 自定义类名 */
  className?: string;
}

export function InlineRefreshIndicator({
  isRefreshing = false,
  pendingChanges = 0,
  className,
}: InlineRefreshIndicatorProps) {
  return (
    <div className={cn('flex items-center gap-1.5', className)}>
      {isRefreshing ? (
        <>
          <RefreshCw className="h-3.5 w-3.5 animate-spin text-primary" />
          <span className="text-xs text-muted-foreground">刷新中...</span>
        </>
      ) : pendingChanges > 0 ? (
        <>
          <Clock className="h-3.5 w-3.5 text-orange-500" />
          <span className="text-xs text-muted-foreground">
            {pendingChanges} 个变更
          </span>
        </>
      ) : null}
    </div>
  );
}
