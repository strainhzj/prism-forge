/**
 * MessageNode 组件
 *
 * 递归渲染消息树的节点
 */

import { useState, useCallback, useMemo } from 'react';
import { ChevronDown, ChevronRight, User, Bot, Wrench, FileText } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import type { MessageNode as MessageType } from '@/types/message';

export interface MessageNodeProps {
  node: MessageType;
  depth?: number;
  lazy?: boolean;
  onLoadContent?: (nodeId: string) => Promise<MessageType>;
  className?: string;
}

function getRoleIcon(role?: string, type?: string) {
  if (type === 'tool_result') {
    return <Wrench className="h-4 w-4 text-orange-500" />;
  }
  if (role === 'user') {
    return <User className="h-4 w-4 text-blue-500" />;
  }
  if (role === 'assistant') {
    return <Bot className="h-4 w-4 text-green-500" />;
  }
  if (role === 'system') {
    return <FileText className="h-4 w-4 text-gray-500" />;
  }
  return null;
}

function getRoleLabel(role?: string, type?: string): string {
  if (type === 'tool_result') return '工具结果';
  if (role === 'user') return '用户';
  if (role === 'assistant') return '助手';
  if (role === 'system') return '系统';
  return role || type || '未知';
}

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
    return '';
  }
}

function truncateContent(content: string, maxLength = 200): string {
  if (content.length <= maxLength) return content;
  return content.slice(0, maxLength) + '...';
}

export function MessageNode({
  node,
  depth = 0,
  lazy = false,
  onLoadContent,
  className,
}: MessageNodeProps) {
  const [isExpanded, setIsExpanded] = useState(true);
  const [isLoading, setIsLoading] = useState(false);
  const [fullContent, setFullContent] = useState<MessageType | null>(null);

  const hasChildren = node.children && node.children.length > 0;
  const displayNode = fullContent || node;

  const toggleExpand = useCallback(() => {
    setIsExpanded((prev) => !prev);
  }, []);

  const handleLoadContent = useCallback(async () => {
    if (!onLoadContent || isLoading || fullContent) return;

    setIsLoading(true);
    try {
      const content = await onLoadContent(node.id);
      setFullContent(content);
    } catch (error) {
      console.error('加载消息内容失败:', error);
    } finally {
      setIsLoading(false);
    }
  }, [node.id, onLoadContent, isLoading, fullContent]);

  const indent = useMemo(() => {
    return depth * 24;
  }, [depth]);

  const messageContent = useMemo(() => {
    const content = displayNode.content || '';
    if (lazy && !fullContent) {
      return truncateContent(content, 200);
    }
    return content;
  }, [displayNode.content, lazy, fullContent]);

  return (
    <div className={cn('w-full', className)}>
      <div
        className={cn(
          'flex items-start gap-2 py-2 px-3 rounded-md transition-colors',
          'hover:bg-accent/50',
          depth === 0 && 'bg-accent/30'
        )}
        style={{ marginLeft: `${indent}px` }}
      >
        {hasChildren ? (
          <button
            onClick={toggleExpand}
            className="shrink-0 mt-0.5 hover:bg-accent rounded p-0.5 transition-colors"
          >
            {isExpanded ? (
              <ChevronDown className="h-4 w-4" />
            ) : (
              <ChevronRight className="h-4 w-4" />
            )}
          </button>
        ) : (
          <div className="w-5 shrink-0" />
        )}

        <div className="shrink-0 mt-0.5">
          {getRoleIcon(displayNode.role, displayNode.type)}
        </div>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <Badge variant="outline" className="text-xs">
              {getRoleLabel(displayNode.role, displayNode.type)}
            </Badge>
            {displayNode.timestamp && (
              <span className="text-xs text-muted-foreground">
                {formatTimestamp(displayNode.timestamp)}
              </span>
            )}
            {displayNode.thread_id && (
              <Badge variant="secondary" className="text-xs">
                {displayNode.thread_id.slice(0, 4)}
              </Badge>
            )}
          </div>

          {messageContent && (
            <div className="text-sm whitespace-pre-wrap break-words font-mono bg-background/50 rounded p-2">
              {messageContent}
            </div>
          )}

          {lazy && !fullContent && displayNode.content && displayNode.content.length > 200 && (
            <Button
              variant="ghost"
              size="sm"
              onClick={handleLoadContent}
              disabled={isLoading}
              className="mt-2 h-7 text-xs"
            >
              {isLoading ? '加载中...' : '加载完整内容'}
            </Button>
          )}
        </div>
      </div>

      {isExpanded && hasChildren && (
        <div className="mt-1">
          {node.children.map((child) => (
            <MessageNode
              key={child.id}
              node={child}
              depth={depth + 1}
              lazy={lazy}
              onLoadContent={onLoadContent}
            />
          ))}
        </div>
      )}
    </div>
  );
}
