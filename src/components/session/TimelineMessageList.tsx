/**
 * TimelineMessageList 组件
 *
 * 时间线式消息列表，支持 user/assistant 视觉区分
 * 参照时间线日志 (TimelineSidebar) 的实现
 * 支持展开/折叠显示完整内容
 */

import { useState, useCallback } from 'react';
import { User, Bot, ChevronDown, ChevronRight } from 'lucide-react';
import { cn } from '@/lib/utils';
import type { MessageNode } from '@/types/message';

// 🔴 调试：组件加载时立即输出
console.log('🚀 [TimelineMessageList] 组件已加载！！！');

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[TimelineMessageList] ${action}`, ...args);
  }
}

// ==================== 类型定义 ====================

export interface TimelineMessageListProps {
  /**
   * 消息节点列表（扁平化后）
   */
  messages: MessageNode[];
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
    return date.toLocaleTimeString('zh-CN', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  } catch {
    return timestamp;
  }
}

/**
 * 从内容中提取文本
 *
 * 根据角色类型使用不同的提取方式：
 * - 用户消息：直接返回 content 的内容
 * - 助手消息：提取 content 中 text 字段的内容
 *
 * @param content - 原始内容
 * @param isUser - 是否是用户消息
 * @returns 提取的文本内容
 */
function extractTextFromContent(content: string, isUser: boolean): string {
  if (!content) return '';

  // 用户消息：直接返回 content 的内容
  if (isUser) {
    return content;
  }

  // 助手消息：提取 text 字段
  try {
    const parsed = JSON.parse(content);

    // 如果是对象且包含 text 字段，返回 text
    if (typeof parsed === 'object' && parsed !== null && 'text' in parsed) {
      return String(parsed.text);
    }

    // 否则返回原始内容
    return content;
  } catch {
    // 解析失败，返回原始内容
    return content;
  }
}

/**
 * 格式化文本内容
 *
 * - 将 `\n` 转换为真正的换行
 * - 保持其他格式化字符
 *
 * @param text - 文本内容
 * @returns 格式化后的文本
 */
function formatTextContent(text: string): string {
  if (!text) return '';

  // 将 \n 转换为真正的换行符
  return text.replace(/\\n/g, '\n');
}

/**
 * TimelineMessageItem 组件 - 单条消息项
 */
interface TimelineMessageItemProps {
  message: MessageNode;
  isExpanded: boolean;
  onToggleExpand: () => void;
}

function TimelineMessageItem({ message, isExpanded, onToggleExpand }: TimelineMessageItemProps) {
  const isUser = message.role?.toLowerCase() === 'user';

  // 提取内容：用户消息直接显示，助手消息提取 text 字段
  const rawContent = isExpanded ? (message.fullContent || message.content || '') : (message.content || '');
  const textContent = extractTextFromContent(rawContent, isUser);

  // 格式化文本（处理 \n）
  const displayContent = formatTextContent(textContent);

  const hasMoreContent = message.fullContent && message.fullContent !== message.content;

  debugLog('render message', {
    id: message.id,
    role: message.role,
    isUser,
    isExpanded,
    hasMoreContent,
    contentLength: displayContent.length,
  });

  return (
    <div
      className={cn(
        'group relative p-3 rounded-lg border transition-all hover:shadow-lg',
        // 根据角色选择颜色
        isUser
          ? 'bg-[var(--color-bg-primary)] border-[var(--color-border-light)]'
          : 'bg-[var(--color-bg-primary)] border-[var(--color-border-light)]'
      )}
      style={{
        backgroundColor: 'var(--color-bg-primary)',
        borderColor: 'var(--color-border-light)',
      }}
      onMouseEnter={(e) => {
        const color = isUser ? '245, 158, 11' : '37, 99, 235'; // warm orange or blue
        e.currentTarget.style.boxShadow = `0 0 20px rgba(${color}, 0.2)`;
        e.currentTarget.style.borderColor = `rgba(${color}, 0.3)`;
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.boxShadow = 'none';
        e.currentTarget.style.borderColor = 'var(--color-border-light)';
      }}
    >
      {/* 顶部：类型图标 + 时间 */}
      <div className="flex items-center gap-2 mb-2">
        {/* 角色图标 */}
        <div
          className={cn(
            'w-6 h-6 rounded-full flex items-center justify-center',
            isUser
              ? 'bg-[var(--color-accent-warm)] text-white'
              : 'bg-[var(--color-accent-blue)] text-white'
          )}
          style={{
            backgroundColor: isUser ? 'var(--color-accent-warm)' : 'var(--color-accent-blue)',
          }}
        >
          {isUser ? (
            <User className="w-3.5 h-3.5" />
          ) : (
            <Bot className="w-3.5 h-3.5" />
          )}
        </div>

        {/* 角色标签 */}
        <span
          className="text-xs font-medium"
          style={{ color: 'var(--color-text-secondary)' }}
        >
          {isUser ? '用户' : '助手'}
        </span>

        {/* 时间戳 */}
        {message.timestamp && (
          <span
            className="text-xs"
            style={{ color: 'var(--color-text-secondary)' }}
          >
            {formatTimestamp(message.timestamp)}
          </span>
        )}

        {/* 展开/折叠按钮 */}
        {hasMoreContent && (
          <button
            onClick={onToggleExpand}
            className="ml-auto p-1 rounded transition-colors hover:bg-[var(--color-app-secondary)]"
            style={{ color: 'var(--color-text-secondary)' }}
            title={isExpanded ? '收起' : '展开'}
          >
            {isExpanded ? (
              <ChevronDown className="w-4 h-4" />
            ) : (
              <ChevronRight className="w-4 h-4" />
            )}
          </button>
        )}
      </div>

      {/* 内容摘要/完整内容 */}
      <p
        className="text-sm whitespace-pre-wrap break-words leading-relaxed"
        style={{
          color: 'var(--color-text-primary)',
          fontFamily: 'Consolas, Monaco, "Courier New", monospace',
          fontSize: '13px',
          lineHeight: '1.6',
        }}
      >
        {displayContent}
      </p>

      {/* 提示有更多内容或收起按钮 */}
      {!isExpanded && hasMoreContent && (
        <div
          className="mt-2 text-xs cursor-pointer hover:underline"
          style={{ color: 'var(--color-text-secondary)' }}
          onClick={onToggleExpand}
        >
          点击查看完整内容...
        </div>
      )}
      {isExpanded && hasMoreContent && (
        <div
          className="mt-2 text-xs cursor-pointer hover:underline"
          style={{ color: 'var(--color-text-secondary)' }}
          onClick={onToggleExpand}
        >
          点击收起
        </div>
      )}
    </div>
  );
}

/**
 * TimelineMessageList 组件
 *
 * @example
 * <TimelineMessageList messages={messageList} />
 */
export function TimelineMessageList({
  messages,
  className,
}: TimelineMessageListProps) {
  // 🔴 调试：组件渲染时立即输出
  console.log('🎨 [TimelineMessageList] 组件渲染！！！', { messageCount: messages.length });

  // 管理每个消息的展开状态
  const [expandedMessages, setExpandedMessages] = useState<Set<string>>(new Set());

  // 切换展开状态
  const toggleExpand = useCallback((messageId: string) => {
    setExpandedMessages((prev) => {
      const next = new Set(prev);
      if (next.has(messageId)) {
        next.delete(messageId);
      } else {
        next.add(messageId);
      }
      return next;
    });
  }, []);

  debugLog('render', { messageCount: messages.length, expandedCount: expandedMessages.size });

  if (messages.length === 0) {
    return (
      <div
        className={cn('flex flex-col items-center justify-center py-12 text-center', className)}
        style={{ color: 'var(--color-text-secondary)' }}
      >
        <p className="font-medium" style={{ color: 'var(--color-text-primary)' }}>
          暂无消息
        </p>
        <p className="text-sm mt-2">该会话文件为空或格式不正确</p>
      </div>
    );
  }

  return (
    <div className={cn('space-y-3', className)}>
      {messages.map((message) => (
        <TimelineMessageItem
          key={message.id}
          message={message}
          isExpanded={expandedMessages.has(message.id)}
          onToggleExpand={() => toggleExpand(message.id)}
        />
      ))}
    </div>
  );
}
