/**
 * MessageCard 组件
 *
 * 显示单条消息的卡片，包含头像、角色标签、内容和时间戳
 * 支持深浅色主题
 */

import { memo } from 'react';
import { User, Bot } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Badge } from '@/components/ui/badge';

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[MessageCard] ${action}`, ...args);
  }
}

// ==================== 类型定义 ====================

export interface MessageCardProps {
  /**
   * 消息角色（user/assistant/system）
   */
  role: string;
  /**
   * 消息内容
   */
  content: string;
  /**
   * 时间戳（ISO 8601 格式）
   */
  timestamp?: string;
  /**
   * 是否显示头像
   */
  showAvatar?: boolean;
  /**
   * 最大内容长度（超过则截断）
   */
  maxContentLength?: number;
  /**
   * 自定义类名
   */
  className?: string;
}

/**
 * 格式化时间戳
 */
function formatTimestamp(timestamp?: string): string {
  if (!timestamp) return '';

  try {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);

    if (diffMins < 1) return '刚刚';
    if (diffMins < 60) return `${diffMins} 分钟前`;
    if (diffHours < 24) return `${diffHours} 小时前`;

    // 返回具体时间
    return date.toLocaleString('zh-CN', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  } catch {
    return timestamp;
  }
}

/**
 * 截断内容
 */
function truncateContent(content: string, maxLength?: number): string {
  if (!maxLength || content.length <= maxLength) return content;
  return content.substring(0, maxLength) + '...';
}

/**
 * MessageCard 组件
 *
 * 使用 React.memo 优化性能
 *
 * @example
 * <MessageCard
 *   role="user"
 *   content="这是一条消息"
 *   timestamp="2025-01-09T12:34:56Z"
 * />
 */
export const MessageCard = memo(function MessageCard({
  role,
  content,
  timestamp,
  showAvatar = true,
  maxContentLength,
  className,
}: MessageCardProps) {
  const isUser = role.toLowerCase() === 'user';
  const formattedTime = formatTimestamp(timestamp);
  const displayContent = truncateContent(content, maxContentLength);

  debugLog('render', { role, contentLength: content.length, timestamp });

  return (
    <div
      className={cn(
        'group relative flex gap-3 p-4 rounded-lg border transition-all',
        'hover:shadow-md',
        // 根据设计提示词的配色方案
        isUser
          ? 'bg-[#FF6B6B]/10 border-[#FF6B6B]/30 shadow-[0_0_20px_rgba(255,107,107,0.2)]' // 珊瑚橙色用户消息
          : 'bg-[#1E1E1E] border-[#4A9EFF]/20', // 深炭灰色AI消息 + 天空蓝边框
        className
      )}
    >
      {/* 头像 */}
      {showAvatar && (
        <div
          className={cn(
            'flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center',
            isUser
              ? 'bg-[#FF6B6B] text-white shadow-[0_0_15px_rgba(255,107,107,0.5)]' // 珊瑚橙色用户头像 + 发光
              : 'bg-[#4A9EFF] text-white shadow-[0_0_15px_rgba(74,158,255,0.5)]' // 天空蓝AI头像 + 发光
          )}
        >
          {isUser ? (
            <User className="w-4 h-4" />
          ) : (
            <Bot className="w-4 h-4" />
          )}
        </div>
      )}

      {/* 内容区域 */}
      <div className="flex-1 min-w-0 space-y-2">
        {/* 角色标签和时间戳 */}
        <div className="flex items-center gap-2">
          <Badge
            className={cn(
              'text-xs font-semibold',
              isUser
                ? 'bg-[#FF6B6B] text-white shadow-[0_0_10px_rgba(255,107,107,0.4)]' // 珊瑚橙色标签
                : 'bg-[#4A9EFF] text-white shadow-[0_0_10px_rgba(74,158,255,0.4)]' // 天空蓝标签
            )}
          >
            {isUser ? 'USER' : 'ASSISTANT'}
          </Badge>

          {formattedTime && (
            <span className="text-xs text-gray-400">
              {formattedTime}
            </span>
          )}
        </div>

        {/* 消息内容 */}
        <div className="text-sm whitespace-pre-wrap break-words text-white">
          {displayContent}
        </div>
      </div>

      {/* 悬停效果边框 */}
      <div className={cn(
        'absolute inset-0 rounded-lg pointer-events-none border-2 transition-opacity',
        'opacity-0 group-hover:opacity-100',
        isUser
          ? 'border-[#FF6B6B]/40 shadow-[0_0_20px_rgba(255,107,107,0.3)]' // 珊瑚橙色发光
          : 'border-[#4A9EFF]/40 shadow-[0_0_20px_rgba(74,158,255,0.3)]' // 天空蓝发光
      )} />
    </div>
  );
}, (prevProps, nextProps) => {
  // 自定义比较函数：仅当关键属性变化时重新渲染
  return (
    prevProps.role === nextProps.role &&
    prevProps.content === nextProps.content &&
    prevProps.timestamp === nextProps.timestamp &&
    prevProps.showAvatar === nextProps.showAvatar &&
    prevProps.className === nextProps.className
  );
});
