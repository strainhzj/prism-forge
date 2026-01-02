/**
 * MessageTree 组件
 *
 * 消息树的容器组件，管理整个树的渲染和状态
 */

import { useState, useCallback, useMemo, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Network, Minimize2, Maximize2 } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Loading } from '@/components/ui/loading';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { MessageNode } from '@/components/MessageNode';
import type { ConversationTree, MessageNode as MessageType } from '@/types/message';

export interface MessageTreeProps {
  /**
   * 会话文件路径
   */
  filePath: string;
  /**
   * 是否启用懒加载
   */
  lazy?: boolean;
  /**
   * 树加载完成回调
   */
  onTreeLoaded?: (tree: ConversationTree) => void;
  /**
   * 自定义类名
   */
  className?: string;
}

/**
 * MessageTree 组件
 *
 * @example
 * <MessageTree
 *   filePath="/path/to/session.jsonl"
 *   lazy={true}
 * />
 */
export function MessageTree({ filePath, lazy = false, onTreeLoaded, className }: MessageTreeProps) {
  // 状态
  const [tree, setTree] = useState<ConversationTree | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [compactView, setCompactView] = useState(false);

  // 加载消息树
  const loadTree = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const result = await invoke<{
        tree: ConversationTree;
        parse_duration_ms: number;
        message_count: number;
        max_depth: number;
      }>('parse_session_tree', {
        filePath,
      });

      setTree(result.tree);

      // 调用回调通知父组件
      if (onTreeLoaded) {
        onTreeLoaded(result.tree);
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`解析会话文件失败: ${errorMessage}`);
    } finally {
      setLoading(false);
    }
  }, [filePath, onTreeLoaded]);

  // 初始加载
  useEffect(() => {
    loadTree();
  }, [loadTree]);

  // 懒加载：加载节点完整内容
  const handleLoadContent = useCallback(
    async (nodeId: string): Promise<MessageType> => {
      // TODO: 调用后端 API 加载完整内容
      // 目前返回原节点（模拟）
      console.log('懒加载节点:', nodeId);
      // 这里应该调用类似 get_message_at 的命令
      return {} as MessageType;
    },
    []
  );

  // 统计信息
  const stats = useMemo(() => {
    if (!tree) return null;
    return {
      totalMessages: tree.total_count,
      maxDepth: tree.max_depth,
      threadCount: tree.thread_count,
      rootCount: tree.roots.length,
    };
  }, [tree]);

  return (
    <div className={cn('flex flex-col h-full', className)}>
      {/* 头部 */}
      <div className="flex items-center justify-between px-4 py-3 border-b">
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2">
            <Network className="h-5 w-5 text-primary" />
            <h2 className="text-lg font-semibold">消息树</h2>
          </div>

          {/* 统计信息 */}
          {stats && (
            <div className="flex items-center gap-2">
              <Badge variant="secondary" className="text-xs">
                {stats.totalMessages} 条消息
              </Badge>
              <Badge variant="outline" className="text-xs">
                深度: {stats.maxDepth}
              </Badge>
              {stats.threadCount > 1 && (
                <Badge variant="outline" className="text-xs">
                  {stats.threadCount} 线程
                </Badge>
              )}
            </div>
          )}
        </div>

        {/* 操作按钮 */}
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setCompactView(!compactView)}
            className="h-8"
          >
            {compactView ? (
              <>
                <Maximize2 className="h-4 w-4 mr-1" />
                展开
              </>
            ) : (
              <>
                <Minimize2 className="h-4 w-4 mr-1" />
                收起
              </>
            )}
          </Button>
          <Button variant="outline" size="sm" onClick={loadTree} className="h-8">
            刷新
          </Button>
        </div>
      </div>

      {/* 错误提示 */}
      {error && (
        <div className="p-4">
          <Alert variant="destructive">
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        </div>
      )}

      {/* 加载状态 */}
      {loading && (
        <div className="flex-1 flex items-center justify-center">
          <Loading text="解析消息树..." />
        </div>
      )}

      {/* 消息树内容 */}
      {!loading && tree && (
        <div className="flex-1 overflow-y-auto p-4">
          {tree.roots.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-center">
              <p className="text-muted-foreground">此会话没有消息</p>
            </div>
          ) : (
            <div className="space-y-1">
              {tree.roots.map((root) => (
                <MessageNode
                  key={root.id}
                  node={root}
                  lazy={lazy}
                  onLoadContent={handleLoadContent}
                />
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
