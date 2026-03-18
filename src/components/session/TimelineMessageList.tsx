/**
 * TimelineMessageList 组件
 *
 * 时间线式消息列表，支持 user/assistant 视觉区分
 * 参照时间线日志 (TimelineSidebar) 的实现
 * 支持展开/折叠显示完整内容
 */

import { useState, useCallback, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
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
   * 内容显示模式：raw = 显示原始JSON，extracted = 提取content字段
   */
  contentDisplayMode?: 'raw' | 'extracted';
  /**
   * 自定义类名
   */
  className?: string;
  /**
   * 分析意图回调（可选）
   */
  onAnalyzeIntent?: (message: MessageNode) => void;
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
 * 根据显示模式和角色类型使用不同的提取方式：
 * - raw 模式：返回格式化的 JSON 字符串
 * - extracted 模式：
 *   - 用户消息：提取 JSON 中 content 字段的内容
 *     - 如果 content 是字符串，直接返回
 *     - 如果 content 是数组，提取每个元素的 text 字段并用双换行拼接
 *   - 助手消息：提取 JSON 中 content 字段的内容
 *     - 如果 content 是字符串，直接返回
 *     - 如果 content 是数组，提取每个元素的 text 字段并用双换行拼接
 *     - 如果有顶级 text 字段，返回 text
 *
 * @param content - 原始内容
 * @param isUser - 是否是用户消息
 * @param displayMode - 显示模式
 * @returns 显示的文本内容
 */
function extractTextFromContent(content: string, isUser: boolean, displayMode: 'raw' | 'extracted'): string {
  if (!content) return '';

  try {
    const parsed = JSON.parse(content);

    // raw 模式：返回格式化的 JSON
    if (displayMode === 'raw') {
      return JSON.stringify(parsed, null, 2);
    }

    // extracted 模式：从 JSON 中提取内容
    if (typeof parsed === 'object' && parsed !== null) {
      // 用户消息或助手消息：提取 content 字段
      if ('content' in parsed) {
        const msgContent = parsed.content;

        // 如果 content 是数组，提取所有 text 字段
        if (Array.isArray(msgContent)) {
          const texts = msgContent
            .map((item: unknown) => {
              if (typeof item === 'object' && item !== null && 'text' in item) {
                return String((item as { text: unknown }).text);
              }
              return null;
            })
            .filter((text): text is string => text !== null);
          return texts.join('\n\n');
        }

        // 如果 content 是字符串，直接返回
        if (typeof msgContent === 'string') {
          return msgContent;
        }

        // 如果 content 是其他类型，尝试转字符串
        return String(msgContent);
      }

      // 兼容：如果有顶级 text 字段，返回 text（主要针对助手消息）
      if (!isUser && 'text' in parsed) {
        return String(parsed.text);
      }
    }

    // 如果找不到对应字段，返回格式化的原始内容
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
  displayMode: 'raw' | 'extracted';
  isFirstUserMessage: boolean; // 是否为第一条 user 消息（开场白）
  onAnalyzeIntent?: (message: MessageNode) => void; // 分析意图回调
}

function TimelineMessageItem({ message, isExpanded, onToggleExpand, displayMode, isFirstUserMessage, onAnalyzeIntent }: TimelineMessageItemProps) {
  const { t } = useTranslation('sessions');
  const { t: tIntent } = useTranslation('intentAnalysis');
  const isUser = message.role?.toLowerCase() === 'user';

  // 从原始内容中提取并格式化文本（最终展示给用户的文本）
  const rawContent = message.content || '';
  const extractedText = extractTextFromContent(rawContent, isUser, displayMode);
  const formattedText = displayMode === 'extracted' ? formatTextContent(extractedText) : extractedText;

  // 判断最终展示文本是否超过200字
  const isTextLong = formattedText.length > 200;

  // 计算当前显示的内容（折叠时显示前200字+省略号，展开时显示完整内容）
  const collapsedText = isTextLong ? formattedText.substring(0, 200) + '...' : formattedText;
  const displayContent = isExpanded ? formattedText : collapsedText;
  const hasMoreContent = isTextLong;

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
          {isUser ? t('detailView.user') : t('detailView.assistant')}
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
            title={isExpanded ? t('detailView.collapse') : t('detailView.expand')}
          >
            {isExpanded ? (
              <ChevronDown className="w-4 h-4" />
            ) : (
              <ChevronRight className="w-4 h-4" />
            )}
          </button>
        )}

        {/* 分析意图按钮 - 仅在非第一条 user 消息显示 */}
        {!isFirstUserMessage && isUser && onAnalyzeIntent && (
          <button
            onClick={() => onAnalyzeIntent(message)}
            className="ml-2 px-2 py-1 text-xs rounded transition-colors hover:bg-[var(--color-accent-blue)] hover:text-white"
            style={{
              color: 'var(--color-accent-blue)',
              border: '1px solid var(--color-accent-blue)'
            }}
            title={tIntent('actions.analyzeIntent')}
          >
            {tIntent('actions.analyzeIntent')}
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
          {t('detailView.expand')}
        </div>
      )}
      {isExpanded && hasMoreContent && (
        <div
          className="mt-2 text-xs cursor-pointer hover:underline"
          style={{ color: 'var(--color-text-secondary)' }}
          onClick={onToggleExpand}
        >
          {t('detailView.collapse')}
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
  contentDisplayMode = 'raw',
  className,
  onAnalyzeIntent,
}: TimelineMessageListProps) {
  const { t } = useTranslation('sessions');

  // 🔴 调试：组件渲染时立即输出
  console.log('🎨 [TimelineMessageList] 组件渲染！！！', { messageCount: messages.length, contentDisplayMode });

  // 找出第一条 user 消息的 ID（用于判断是否为开场白）
  const firstUserMessageId = useMemo(() => {
    const firstUserMsg = messages.find(m => m.role?.toLowerCase() === 'user');
    return firstUserMsg?.id || null;
  }, [messages]);

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
          {t('detailView.noMessages')}
        </p>
        <p className="text-sm mt-2">{t('detailView.noMessagesHint')}</p>
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
          displayMode={contentDisplayMode}
          isFirstUserMessage={message.id === firstUserMessageId}
          onAnalyzeIntent={onAnalyzeIntent}
        />
      ))}
    </div>
  );
}
