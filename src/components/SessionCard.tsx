/**
 * SessionCard 组件
 *
 * 显示单个会话的卡片信息
 */

import { useCallback, useMemo, memo } from 'react';
import { Star, Archive, Tag, Clock } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import type { Session } from '@/stores/useSessionStore';

export interface SessionCardProps {
  /**
   * 会话数据
   */
  session: Session;
  /**
   * 点击事件
   */
  onClick?: (session: Session) => void;
  /**
   * 评分变更事件
   */
  onRatingChange?: (sessionId: string, rating: number | null) => void;
  /**
   * 归档事件
   */
  onArchive?: (sessionId: string) => void;
  /**
   * 取消归档事件
   */
  onUnarchive?: (sessionId: string) => void;
  /**
   * 自定义类名
   */
  className?: string;
}

/**
 * 解析标签 JSON 字符串
 */
function parseTags(tagsJson: string): string[] {
  if (!tagsJson || tagsJson === '[]') {
    return [];
  }
  try {
    return JSON.parse(tagsJson) as string[];
  } catch {
    return [];
  }
}

/**
 * 格式化日期时间
 */
function formatDateTime(dateString: string): string {
  try {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return '刚刚';
    if (diffMins < 60) return `${diffMins} 分钟前`;
    if (diffHours < 24) return `${diffHours} 小时前`;
    if (diffDays < 7) return `${diffDays} 天前`;

    return date.toLocaleDateString('zh-CN');
  } catch {
    return dateString;
  }
}

/**
 * SessionCard 组件
 *
 * 使用 React.memo 优化：仅当关键 props 变化时重新渲染
 *
 * @example
 * <SessionCard
 *   session={sessionData}
 *   onClick={(session) => console.log('Clicked', session)}
 *   onRatingChange={(id, rating) => console.log('Rated', id, rating)}
 * />
 */
export const SessionCard = memo(function SessionCard({
  session,
  onClick,
  onRatingChange,
  onArchive,
  onUnarchive,
  className,
}: SessionCardProps) {
  const tags = useMemo(() => parseTags(session.tags), [session.tags]);

  // 处理点击
  const handleClick = useCallback(() => {
    onClick?.(session);
  }, [session, onClick]);

  // 处理评分
  const handleRatingClick = useCallback(
    (rating: number | null) => {
      onRatingChange?.(session.sessionId, rating);
    },
    [session.sessionId, onRatingChange]
  );

  // 处理归档
  const handleArchive = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      onArchive?.(session.sessionId);
    },
    [session.sessionId, onArchive]
  );

  // 处理取消归档
  const handleUnarchive = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      onUnarchive?.(session.sessionId);
    },
    [session.sessionId, onUnarchive]
  );

  return (
    <Card
      className={cn(
        'cursor-pointer transition-all hover:shadow-md',
        session.isArchived && 'opacity-60',
        className
      )}
      onClick={handleClick}
    >
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between">
          <div className="flex-1 min-w-0">
            <CardTitle className="text-base truncate">
              {session.projectName}
            </CardTitle>
            <CardDescription className="text-xs mt-1 truncate">
              {session.sessionId.slice(0, 8)}...
            </CardDescription>
          </div>

          {/* 评分 */}
          {session.rating !== null && session.rating !== undefined && (
            <div className="flex items-center gap-0.5 shrink-0 ml-2">
              {Array.from({ length: 5 }).map((_, i) => (
                <Star
                  key={i}
                  className={cn(
                    'h-4 w-4',
                    i < (session.rating ?? 0)
                      ? 'fill-yellow-400 text-yellow-400'
                      : 'text-gray-300'
                  )}
                />
              ))}
            </div>
          )}
        </div>
      </CardHeader>

      <CardContent className="pb-3">
        {/* 标签 */}
        {tags.length > 0 && (
          <div className="flex items-center gap-1.5 flex-wrap mb-2">
            <Tag className="h-3.5 w-3.5 text-muted-foreground" />
            <div className="flex items-center gap-1 flex-wrap">
              {tags.map((tag) => (
                <Badge key={tag} variant="secondary" className="text-xs">
                  {tag}
                </Badge>
              ))}
            </div>
          </div>
        )}

        {/* 元信息 */}
        <div className="flex items-center gap-3 text-xs text-muted-foreground">
          <div className="flex items-center gap-1">
            <Clock className="h-3.5 w-3.5" />
            <span>{formatDateTime(session.updatedAt)}</span>
          </div>
          {session.isArchived && (
            <Badge variant="outline" className="text-xs">
              已归档
            </Badge>
          )}
        </div>
      </CardContent>

      <CardFooter className="pt-0 border-t">
        <div className="flex items-center justify-between w-full">
          {/* 评分按钮 */}
          <div className="flex items-center gap-1">
            <span className="text-xs text-muted-foreground mr-1">评分:</span>
            {[1, 2, 3, 4, 5].map((rating) => (
              <button
                key={rating}
                onClick={(e) => {
                  e.stopPropagation();
                  // 如果当前评分等于点击的评分，则取消评分
                  const newRating: number | null =
                    (session.rating ?? 0) === rating ? null : rating;
                  handleRatingClick(newRating);
                }}
                className="hover:scale-110 transition-transform"
              >
                <Star
                  className={cn(
                    'h-4 w-4',
                    session.rating && session.rating >= rating
                      ? 'fill-yellow-400 text-yellow-400'
                      : 'text-gray-300 hover:text-yellow-400'
                  )}
                />
              </button>
            ))}
          </div>

          {/* 归档按钮 */}
          {session.isArchived ? (
            <Button
              variant="ghost"
              size="sm"
              onClick={handleUnarchive}
              className="h-7 px-2 text-xs"
            >
              取消归档
            </Button>
          ) : (
            <Button
              variant="ghost"
              size="sm"
              onClick={handleArchive}
              className="h-7 px-2 text-xs"
            >
              <Archive className="h-3.5 w-3.5 mr-1" />
              归档
            </Button>
          )}
        </div>
      </CardFooter>
    </Card>
  );
}, (prevProps, nextProps) => {
  // 自定义比较函数：仅当关键属性变化时重新渲染
  return (
    prevProps.session.sessionId === nextProps.session.sessionId &&
    prevProps.session.rating === nextProps.session.rating &&
    prevProps.session.isArchived === nextProps.session.isArchived &&
    prevProps.session.updatedAt === nextProps.session.updatedAt &&
    prevProps.className === nextProps.className
  );
});
