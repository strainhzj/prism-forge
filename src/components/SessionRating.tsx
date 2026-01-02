/**
 * SessionRating 组件
 *
 * 会话星级评分组件，支持点击、悬停预览和快捷键
 */

import { useState, useCallback, useMemo, useEffect } from 'react';
import { Star, StarHalf } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';

export interface SessionRatingProps {
  /** 当前评分（0-5，null 表示未评分） */
  rating?: number | null;
  /** 评分变更回调 */
  onRatingChange?: (rating: number | null) => void;
  /** 是否只读模式 */
  readonly?: boolean;
  /** 是否显示半星 */
  allowHalf?: boolean;
  /** 星星大小 */
  size?: 'sm' | 'md' | 'lg';
  /** 自定义类名 */
  className?: string;
}

/**
 * SessionRating 组件
 *
 * @example
 * <SessionRating
 *   rating={4}
 *   onRatingChange={(rating) => console.log('Rated:', rating)}
 * />
 */
export function SessionRating({
  rating = null,
  onRatingChange,
  readonly = false,
  allowHalf = false,
  size = 'md',
  className,
}: SessionRatingProps) {
  // 悬停状态
  const [hoverRating, setHoverRating] = useState<number | null>(null);

  // 计算显示的评分
  const displayRating = useMemo(() => {
    return hoverRating !== null ? hoverRating : (rating ?? 0);
  }, [hoverRating, rating]);

  // 处理星星点击
  const handleStarClick = useCallback(
    (value: number) => {
      if (readonly) return;

      // 如果点击当前评分，则取消评分
      const newRating: number | null = rating === value ? null : value;
      onRatingChange?.(newRating);
    },
    [rating, onRatingChange, readonly]
  );

  // 处理鼠标进入
  const handleMouseEnter = useCallback((value: number) => {
    if (readonly) return;
    setHoverRating(value);
  }, [readonly]);

  // 处理鼠标离开
  const handleMouseLeave = useCallback(() => {
    setHoverRating(null);
  }, []);

  // 键盘快捷键支持（1-5 键）
  useEffect(() => {
    if (readonly) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      const key = e.key;
      if (key >= '1' && key <= '5') {
        e.preventDefault();
        const value = parseInt(key, 10);
        handleStarClick(value);
      } else if (key === '0' || key === 'Escape') {
        e.preventDefault();
        onRatingChange?.(null);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [handleStarClick, onRatingChange, readonly]);

  // 星星尺寸映射
  const sizeClass = useMemo(() => {
    switch (size) {
      case 'sm':
        return 'h-4 w-4';
      case 'lg':
        return 'h-6 w-6';
      default:
        return 'h-5 w-5';
    }
  }, [size]);

  return (
    <div
      className={cn('inline-flex items-center gap-0.5', className)}
      onMouseLeave={handleMouseLeave}
    >
      {Array.from({ length: 5 }).map((_, i) => {
        const value = i + 1;
        const isActive = displayRating >= value;
        const isHalf = allowHalf && displayRating >= value - 0.5 && displayRating < value;

        return (
          <button
            key={value}
            type="button"
            onClick={() => handleStarClick(value)}
            onMouseEnter={() => handleMouseEnter(value)}
            disabled={readonly}
            className={cn(
              'shrink-0 transition-all duration-150',
              !readonly && 'hover:scale-110 cursor-pointer',
              readonly && 'cursor-default'
            )}
            aria-label={`评分 ${value} 星`}
            title={`${value} 星${readonly ? '' : ' (按 ${value} 键)'}`}
          >
            {isHalf ? (
              <StarHalf
                className={cn(
                  sizeClass,
                  isActive
                    ? 'fill-yellow-400 text-yellow-400'
                    : 'text-gray-300'
                )}
              />
            ) : (
              <Star
                className={cn(
                  sizeClass,
                  isActive
                    ? 'fill-yellow-400 text-yellow-400'
                    : 'text-gray-300'
                )}
              />
            )}
          </button>
        );
      })}
    </div>
  );
}

/**
 * 评分统计显示组件
 */
export interface RatingStatsProps {
  /** 评分值 */
  rating: number;
  /** 总评分数 */
  count?: number;
  /** 是否显示数字 */
  showNumber?: boolean;
  /** 星星大小 */
  size?: 'sm' | 'md' | 'lg';
  /** 自定义类名 */
  className?: string;
}

export function RatingStats({
  rating,
  count,
  showNumber = true,
  size = 'md',
  className,
}: RatingStatsProps) {
  return (
    <div className={cn('flex items-center gap-2', className)}>
      <SessionRating rating={rating} readonly size={size} />
      {showNumber && (
        <span className="text-sm font-medium text-muted-foreground">
          {rating.toFixed(1)}
        </span>
      )}
      {count !== undefined && (
        <span className="text-xs text-muted-foreground">
          ({count} 个评分)
        </span>
      )}
    </div>
  );
}

/**
 * 紧凑型评分组件（只显示星星，无交互）
 */
export interface RatingDisplayProps {
  /** 评分值 */
  rating: number;
  /** 星星大小 */
  size?: 'sm' | 'md' | 'lg';
  /** 自定义类名 */
  className?: string;
}

export function RatingDisplay({
  rating,
  size = 'sm',
  className,
}: RatingDisplayProps) {
  // 星星尺寸映射
  const sizeClass = useMemo(() => {
    switch (size) {
      case 'sm':
        return 'h-3.5 w-3.5';
      case 'lg':
        return 'h-6 w-6';
      default:
        return 'h-4 w-4';
    }
  }, [size]);

  return (
    <div className={cn('flex items-center gap-0.5', className)}>
      {Array.from({ length: 5 }).map((_, i) => (
        <Star
          key={i}
          className={cn(
            sizeClass,
            i < Math.round(rating)
              ? 'fill-yellow-400 text-yellow-400'
              : 'text-gray-300'
          )}
        />
      ))}
    </div>
  );
}

/**
 * 快速评分按钮组
 */
export interface QuickRatingProps {
  /** 当前评分 */
  rating?: number | null;
  /** 评分变更回调 */
  onRatingChange?: (rating: number | null) => void;
  /** 自定义类名 */
  className?: string;
}

export function QuickRating({
  rating,
  onRatingChange,
  className,
}: QuickRatingProps) {
  const ratings = [1, 2, 3, 4, 5] as const;

  return (
    <div className={cn('flex items-center gap-1', className)}>
      <span className="text-xs text-muted-foreground mr-1">评分:</span>
      {ratings.map((value) => (
        <Button
          key={value}
          variant="ghost"
          size="sm"
          onClick={() => onRatingChange?.(value === rating ? null : value)}
          className={cn(
            'h-7 px-2 text-xs',
            rating === value && 'bg-primary text-primary-foreground hover:bg-primary/90'
          )}
        >
          {value} 星
        </Button>
      ))}
    </div>
  );
}
