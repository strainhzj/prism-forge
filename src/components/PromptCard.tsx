/**
 * PromptCard 组件
 *
 * 提示词卡片 - 用于显示保存的提示词或 Meta 模板
 * 支持评分、编辑、删除、复制等操作
 */

import { useState, useCallback } from 'react';
import {
  Star,
  Copy,
  Edit,
  Trash2,
  Clock,
  TrendingUp,
  Check,
  X
} from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card } from '@/components/ui/card';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle
} from '@/components/ui/alert-dialog';
import type { PromptLibraryItem } from '@/types/promptLibrary';
import { isSavedPrompt } from '@/types/promptLibrary';

export interface PromptCardProps {
  /**
   * 提示词数据
   */
  prompt: PromptLibraryItem;
  /**
   * 是否显示评分选择器
   */
  showRatingSelector?: boolean;
  /**
   * 评分变更回调
   */
  onRatingChange?: (id: number, rating: number) => void;
  /**
   * 编辑回调
   */
  onEdit?: (prompt: PromptLibraryItem) => void;
  /**
   * 删除回调
   */
  onDelete?: (id: number) => void;
  /**
   * 使用提示词回调
   */
  onUse?: (prompt: PromptLibraryItem) => void;
  /**
   * 自定义类名
   */
  className?: string;
}

/**
 * PromptCard 组件
 */
export function PromptCard({
  prompt,
  showRatingSelector = false,
  onRatingChange,
  onEdit,
  onDelete,
  onUse,
  className
}: PromptCardProps) {
  const [copied, setCopied] = useState(false);
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);
  const [hoverRating, setHoverRating] = useState(0);

  /**
   * 复制提示词内容
   */
  const handleCopy = useCallback(async () => {
    const content = isSavedPrompt(prompt) ? prompt.content : prompt.content;
    try {
      await navigator.clipboard.writeText(content);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('复制失败:', err);
    }
  }, [prompt]);

  /**
   * 处理评分变更
   */
  const handleRatingClick = useCallback(
    (rating: number) => {
      if (prompt.id && onRatingChange) {
        onRatingChange(prompt.id, rating);
      }
    },
    [prompt.id, onRatingChange]
  );

  /**
   * 处理删除确认
   */
  const handleDeleteConfirm = useCallback(() => {
    if (prompt.id && onDelete) {
      onDelete(prompt.id);
      setShowDeleteDialog(false);
    }
  }, [prompt.id, onDelete]);

  /**
   * 获取卡片标题
   */
  const getTitle = (): string => {
    if (isSavedPrompt(prompt)) {
      return prompt.title;
    }
    return prompt.name;
  };

  /**
   * 获取卡片内容
   */
  const getContent = (): string => {
    return prompt.content;
  };

  /**
   * 获取预览内容（截断）
   */
  const getPreview = (): string => {
    const content = getContent();
    const maxLength = 150;
    if (content.length <= maxLength) {
      return content;
    }
    return content.substring(0, maxLength) + '...';
  };

  /**
   * 获取评分
   */
  const getRating = (): number | undefined => {
    if (isSavedPrompt(prompt)) {
      return prompt.rating;
    }
    return undefined;
  };

  /**
   * 获取使用次数
   */
  const getUsageCount = (): number | undefined => {
    if (isSavedPrompt(prompt)) {
      return prompt.usage_count;
    }
    return undefined;
  };

  /**
   * 获取创建时间
   */
  const getCreatedAt = (): string | undefined => {
    if (isSavedPrompt(prompt)) {
      return prompt.created_at;
    }
    return prompt.created_at;
  };

  /**
   * 获取分类标签
   */
  const getCategoryLabel = (): string => {
    if (isSavedPrompt(prompt)) {
      const labels: Record<string, string> = {
        next_goals: '用户目标',
        ai_analysis: 'AI 分析',
        meta_template: '元提示词'
      };
      return labels[prompt.category] || prompt.category;
    }
    return 'Meta 模板';
  };

  /**
   * 获取分类颜色
   */
  const getCategoryColor = (): string => {
    if (isSavedPrompt(prompt)) {
      const colors: Record<string, string> = {
        next_goals: 'bg-blue-500/10 text-blue-500',
        ai_analysis: 'bg-purple-500/10 text-purple-500',
        meta_template: 'bg-orange-500/10 text-orange-500'
      };
      return colors[prompt.category] || 'bg-gray-500/10 text-gray-500';
    }
    return 'bg-green-500/10 text-green-500';
  };

  const rating = getRating();
  const usageCount = getUsageCount();
  const createdAt = getCreatedAt();
  const title = getTitle();

  return (
    <>
      <Card
        className={cn(
          'p-4 hover:bg-accent/50 transition-colors cursor-pointer group',
          className
        )}
        onClick={() => onUse?.(prompt)}
      >
        {/* 头部：标题 + 分类标签 */}
        <div className="flex items-start justify-between mb-3">
          <div className="flex-1 min-w-0 pr-2">
            <div className="flex items-center gap-2 mb-1">
              <h3 className="font-semibold text-sm truncate">{title}</h3>
              <Badge className={cn('text-xs', getCategoryColor())}>
                {getCategoryLabel()}
              </Badge>
            </div>
            <p className="text-xs text-muted-foreground line-clamp-2">
              {getPreview()}
            </p>
          </div>

          {/* 操作按钮 */}
          <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
            {/* 复制按钮 */}
            <Button
              variant="ghost"
              size="sm"
              onClick={(e) => {
                e.stopPropagation();
                handleCopy();
              }}
              className="h-7 w-7 p-0"
              title={copied ? '已复制' : '复制'}
            >
              {copied ? (
                <Check className="h-3 w-3 text-green-500" />
              ) : (
                <Copy className="h-3 w-3" />
              )}
            </Button>

            {/* 编辑按钮 */}
            {onEdit && (
              <Button
                variant="ghost"
                size="sm"
                onClick={(e) => {
                  e.stopPropagation();
                  onEdit(prompt);
                }}
                className="h-7 w-7 p-0"
                title="编辑"
              >
                <Edit className="h-3 w-3" />
              </Button>
            )}

            {/* 删除按钮 */}
            {onDelete && (
              <Button
                variant="ghost"
                size="sm"
                onClick={(e) => {
                  e.stopPropagation();
                  setShowDeleteDialog(true);
                }}
                className="h-7 w-7 p-0 text-destructive hover:text-destructive"
                title="删除"
              >
                <Trash2 className="h-3 w-3" />
              </Button>
            )}
          </div>
        </div>

        {/* 底部：评分 + 统计信息 */}
        <div className="flex items-center justify-between">
          {/* 评分 */}
          <div className="flex items-center gap-1">
            {showRatingSelector ? (
              <div className="flex items-center">
                {[1, 2, 3, 4, 5].map((star) => (
                  <button
                    key={star}
                    onClick={(e) => {
                      e.stopPropagation();
                      handleRatingClick(star);
                    }}
                    onMouseEnter={() => setHoverRating(star)}
                    onMouseLeave={() => setHoverRating(0)}
                    className="p-0.5 hover:scale-110 transition-transform"
                  >
                    <Star
                      className={cn(
                        'h-3.5 w-3.5',
                        (hoverRating || rating || 0) >= star
                          ? 'fill-yellow-400 text-yellow-400'
                          : 'text-gray-300'
                      )}
                    />
                  </button>
                ))}
              </div>
            ) : rating ? (
              <div className="flex items-center gap-1">
                <Star className="h-3.5 w-3.5 fill-yellow-400 text-yellow-400" />
                <span className="text-xs text-muted-foreground">{rating}.0</span>
              </div>
            ) : null}
          </div>

          {/* 统计信息 */}
          <div className="flex items-center gap-3 text-xs text-muted-foreground">
            {usageCount !== undefined && usageCount > 0 && (
              <div className="flex items-center gap-1">
                <TrendingUp className="h-3 w-3" />
                <span>{usageCount}</span>
              </div>
            )}
            {createdAt && (
              <div className="flex items-center gap-1">
                <Clock className="h-3 w-3" />
                <span>{new Date(createdAt).toLocaleDateString()}</span>
              </div>
            )}
          </div>
        </div>
      </Card>

      {/* 删除确认对话框 */}
      <AlertDialog open={showDeleteDialog} onOpenChange={setShowDeleteDialog}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认删除</AlertDialogTitle>
            <AlertDialogDescription>
              确定要删除提示词「{title}」吗？此操作无法撤销。
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={() => setShowDeleteDialog(false)}>
              <X className="h-4 w-4 mr-2" />
              取消
            </AlertDialogCancel>
            <AlertDialogAction onClick={handleDeleteConfirm} className="bg-destructive text-destructive-foreground hover:bg-destructive/90">
              <Trash2 className="h-4 w-4 mr-2" />
              删除
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
