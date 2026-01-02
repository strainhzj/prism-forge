/**
 * TagEditor 组件
 *
 * 会话标签编辑器，支持添加、删除标签和推荐标签
 */

import { useState, useCallback, useMemo, useRef, useEffect } from 'react';
import { X, Hash } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';

/**
 * 推荐标签列表
 */
export const RECOMMENDED_TAGS = [
  '重要',
  '待办',
  '已完成',
  '参考',
  '实验性',
  '调试',
  '优化',
  '重构',
  '文档',
  '学习',
  '工作',
  '个人项目',
  '客户项目',
  'Bug修复',
  '新功能',
  '代码审查',
  '架构设计',
  '性能优化',
  '安全相关',
];

export interface TagEditorProps {
  /** 当前标签列表 */
  tags: string[];
  /** 标签变更回调 */
  onTagsChange?: (tags: string[]) => void;
  /** 是否只读模式 */
  readonly?: boolean;
  /** 最大标签数量 */
  maxTags?: number;
  /** 是否显示推荐标签 */
  showRecommended?: boolean;
  /** 自定义推荐标签列表 */
  recommendedTags?: string[];
  /** 自定义类名 */
  className?: string;
}

/**
 * TagEditor 组件
 *
 * @example
 * <TagEditor
 *   tags={['重要', '待办']}
 *   onTagsChange={(tags) => console.log('Tags:', tags)}
 * />
 */
export function TagEditor({
  tags,
  onTagsChange,
  readonly = false,
  maxTags = 10,
  showRecommended = true,
  recommendedTags = RECOMMENDED_TAGS,
  className,
}: TagEditorProps) {
  const [inputValue, setInputValue] = useState('');
  const [showRecommendations, setShowRecommendations] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  // 过滤已使用的标签
  const availableRecommendations = useMemo(() => {
    return recommendedTags.filter((tag) => !tags.includes(tag));
  }, [recommendedTags, tags]);

  // 添加标签
  const addTag = useCallback(
    (tag: string) => {
      const trimmedTag = tag.trim();
      if (!trimmedTag) return false;

      // 检查是否已存在
      if (tags.includes(trimmedTag)) {
        return false;
      }

      // 检查数量限制
      if (tags.length >= maxTags) {
        return false;
      }

      const newTags = [...tags, trimmedTag];
      onTagsChange?.(newTags);
      setInputValue('');
      return true;
    },
    [tags, maxTags, onTagsChange]
  );

  // 删除标签
  const removeTag = useCallback(
    (tagToRemove: string) => {
      if (readonly) return;
      const newTags = tags.filter((tag) => tag !== tagToRemove);
      onTagsChange?.(newTags);
    },
    [tags, onTagsChange, readonly]
  );

  // 处理输入框提交
  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      addTag(inputValue);
    },
    [inputValue, addTag]
  );

  // 处理键盘事件
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === 'Backspace' && !inputValue && tags.length > 0) {
        // 删除最后一个标签
        removeTag(tags[tags.length - 1]);
      } else if (e.key === 'Escape') {
        setInputValue('');
        setShowRecommendations(false);
      }
    },
    [inputValue, tags, removeTag]
  );

  // 点击外部关闭推荐列表
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (
        inputRef.current &&
        !inputRef.current.contains(e.target as Node) &&
        !(e.target as Element).closest('.tag-recommendations')
      ) {
        setShowRecommendations(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, []);

  return (
    <div className={cn('space-y-2', className)}>
      {/* 标签列表 */}
      {tags.length > 0 && (
        <div className="flex flex-wrap gap-1.5">
          {tags.map((tag) => (
            <Badge
              key={tag}
              variant="secondary"
              className={cn(
                'gap-1 pl-2 pr-1.5 py-0.5 text-xs',
                !readonly && 'hover:bg-secondary/80'
              )}
            >
              <span className="max-w-[150px] truncate">{tag}</span>
              {!readonly && (
                <button
                  type="button"
                  onClick={() => removeTag(tag)}
                  className="hover:text-destructive transition-colors"
                  aria-label={`删除标签 ${tag}`}
                >
                  <X className="h-3 w-3" />
                </button>
              )}
            </Badge>
          ))}
        </div>
      )}

      {/* 输入框 */}
      {!readonly && (
        <form onSubmit={handleSubmit} className="relative">
          <div className="relative">
            <Hash className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <Input
              ref={inputRef}
              type="text"
              value={inputValue}
              onChange={(e) => setInputValue(e.target.value)}
              onKeyDown={handleKeyDown}
              onFocus={() => setShowRecommendations(true)}
              placeholder={
                tags.length >= maxTags
                  ? `已达上限 (${maxTags} 个标签)`
                  : '输入标签名...'
              }
              disabled={tags.length >= maxTags}
              className="pl-9 pr-10 h-8"
            />
            {inputValue && (
              <button
                type="button"
                onClick={() => setInputValue('')}
                className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors"
              >
                <X className="h-3.5 w-3.5" />
              </button>
            )}
          </div>

          {/* 推荐标签下拉列表 */}
          {showRecommendations &&
            showRecommended &&
            availableRecommendations.length > 0 && (
              <div className="tag-recommendations absolute z-10 w-full mt-1 bg-popover border rounded-md shadow-lg max-h-60 overflow-auto">
                <div className="p-1.5 space-y-0.5">
                  <div className="text-xs text-muted-foreground px-2 py-1">
                    推荐标签
                  </div>
                  {availableRecommendations.slice(0, 10).map((tag) => (
                    <button
                      key={tag}
                      type="button"
                      onClick={() => {
                        addTag(tag);
                        setShowRecommendations(false);
                      }}
                      className="w-full text-left px-2 py-1.5 text-sm rounded hover:bg-accent transition-colors flex items-center gap-2"
                    >
                      <Hash className="h-3 w-3 text-muted-foreground" />
                      <span>{tag}</span>
                    </button>
                  ))}
                </div>
              </div>
            )}
        </form>
      )}

      {/* 标签计数 */}
      <div className="text-xs text-muted-foreground">
        {tags.length} / {maxTags} 个标签
      </div>
    </div>
  );
}

/**
 * 只读标签显示组件
 */
export interface TagDisplayProps {
  /** 标签列表 */
  tags: string[];
  /** 最大显示数量 */
  maxDisplay?: number;
  /** 星星大小 */
  size?: 'sm' | 'md' | 'lg';
  /** 自定义类名 */
  className?: string;
}

export function TagDisplay({
  tags,
  maxDisplay = 5,
  size = 'sm',
  className,
}: TagDisplayProps) {
  const displayTags = tags.slice(0, maxDisplay);
  const remainingCount = tags.length - maxDisplay;

  const sizeClass = useMemo(() => {
    switch (size) {
      case 'sm':
        return 'text-xs px-1.5 py-0';
      case 'lg':
        return 'text-sm px-3 py-1';
      default:
        return 'text-xs px-2 py-0.5';
    }
  }, [size]);

  return (
    <div className={cn('flex flex-wrap gap-1', className)}>
      {displayTags.map((tag) => (
        <Badge key={tag} variant="outline" className={cn(sizeClass, 'gap-1')}>
          <Hash className="h-2.5 w-2.5" />
          <span className="max-w-[150px] truncate">{tag}</span>
        </Badge>
      ))}
      {remainingCount > 0 && (
        <Badge variant="secondary" className={cn(sizeClass, 'text-muted-foreground')}>
          +{remainingCount}
        </Badge>
      )}
    </div>
  );
}

/**
 * 快速标签选择器
 */
export interface QuickTagSelectorProps {
  /** 可选标签列表 */
  options: string[];
  /** 已选标签 */
  value: string[];
  /** 变更回调 */
  onChange?: (tags: string[]) => void;
  /** 最大选择数量 */
  maxSelections?: number;
  /** 自定义类名 */
  className?: string;
}

export function QuickTagSelector({
  options,
  value,
  onChange,
  maxSelections = 5,
  className,
}: QuickTagSelectorProps) {
  const handleToggle = useCallback(
    (tag: string) => {
      const isSelected = value.includes(tag);

      if (isSelected) {
        // 取消选择
        onChange?.(value.filter((t) => t !== tag));
      } else {
        // 选择标签
        if (value.length >= maxSelections) {
          return;
        }
        onChange?.([...value, tag]);
      }
    },
    [value, maxSelections, onChange]
  );

  return (
    <div className={cn('flex flex-wrap gap-1.5', className)}>
      {options.map((tag) => {
        const isSelected = value.includes(tag);
        return (
          <Button
            key={tag}
            type="button"
            variant={isSelected ? 'primary' : 'outline'}
            size="sm"
            onClick={() => handleToggle(tag)}
            disabled={!isSelected && value.length >= maxSelections}
            className="h-7 px-2.5 text-xs"
          >
            <Hash className="h-3 w-3 mr-1" />
            {tag}
          </Button>
        );
      })}
    </div>
  );
}
