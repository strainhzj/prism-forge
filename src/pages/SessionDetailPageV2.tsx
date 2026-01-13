/**
 * SessionDetailPageV2 组件
 *
 * 重构版会话详情页面 - 采用左右分栏布局
 * 左侧：消息列表（使用 MessageCard）
 * 右侧：统计信息边栏（使用 SessionStatsSidebar）
 * 支持深浅色主题
 */

import { useCallback, useEffect, useState, useMemo } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { ArrowLeft, FileText, Download, RefreshCw } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { ThemeToggle } from '@/components/ThemeToggle';
import { MessageCard } from '@/components/session/MessageCard';
import { SessionStatsSidebar } from '@/components/session/SessionStatsSidebar';
import { ExportDialog } from '@/components/ExportDialog';
import { useSessions, useSessionActions } from '@/stores/useSessionStore';
import type { ConversationTree, MessageNode } from '@/types/message';
import type { ExportData } from '@/types/export';

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[SessionDetailPageV2] ${action}`, ...args);
  }
}

/**
 * SessionDetailPageV2 组件
 *
 * 路由: /sessions/:sessionId
 *
 * @example
 * <SessionDetailPageV2 />
 */
export function SessionDetailPageV2({ className }: { className?: string }) {
  // 🔴 调试日志：页面加载时立即输出
  console.log('🚀 [SessionDetailPageV2] 组件已加载！！！新 UI 应该显示');
  console.log('📍 当前 URL:', window.location.pathname);

  const navigate = useNavigate();
  const { sessionId } = useParams<{ sessionId: string }>();
  const sessions = useSessions();
  const { setActiveSessions } = useSessionActions();

  // 状态管理
  const [conversationTree, setConversationTree] = useState<ConversationTree | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showExportDialog, setShowExportDialog] = useState(false);

  // 查找当前会话
  const session = sessions.find((s) => s.sessionId === sessionId);

  // 计算消息列表（扁平化树结构）
  const messageList = useMemo(() => {
    if (!conversationTree) return [];

    const messages: Array<MessageNode & { depth: number }> = [];

    const traverseTree = (nodes: MessageNode[], depth: number = 0) => {
      for (const node of nodes) {
        messages.push({ ...node, depth });
        if (node.children && node.children.length > 0) {
          traverseTree(node.children, depth + 1);
        }
      }
    };

    traverseTree(conversationTree.roots);
    return messages;
  }, [conversationTree]);

  // 计算 Token 统计（简化估算）
  const tokenStats = useMemo(() => {
    if (!conversationTree) return null;

    let totalTextLength = 0;
    let userMessages = 0;
    let assistantMessages = 0;

    messageList.forEach((msg) => {
      if (msg.content) {
        totalTextLength += msg.content.length;
        if (msg.role === 'user') userMessages++;
        if (msg.role === 'assistant') assistantMessages++;
      }
    });

    // 简单估算：中文约 2 token/字，英文约 0.25 token/字符
    const estimatedTokens = Math.round(totalTextLength * 0.5);
    const inputTokens = Math.round(estimatedTokens * 0.6);
    const outputTokens = Math.round(estimatedTokens * 0.4);

    return {
      inputTokens,
      outputTokens,
      totalTokens: estimatedTokens,
      messageCount: conversationTree.total_count,
    };
  }, [conversationTree, messageList]);

  // 初始加载会话列表
  useEffect(() => {
    if (sessions.length === 0) {
      setActiveSessions();
    }
  }, [setActiveSessions, sessions.length]);

  // 加载会话内容
  const loadSessionContent = useCallback(async () => {
    if (!session?.filePath) return;

    debugLog('loadSessionContent', '开始加载会话内容', session.filePath);
    setLoading(true);
    setError(null);

    try {
      // 动态导入 MessageTree 组件以避免循环依赖
      const { invoke } = await import('@tauri-apps/api/core');

      const result = await invoke<{
        tree: ConversationTree;
        parse_duration_ms: number;
        message_count: number;
        max_depth: number;
      }>('parse_session_tree', {
        filePath: session.filePath,
      });

      debugLog('loadSessionContent', '加载成功', result.message_count, '条消息');
      setConversationTree(result.tree);
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      debugLog('loadSessionContent', '加载失败', errorMsg);
      setError(`加载会话内容失败: ${errorMsg}`);
    } finally {
      setLoading(false);
    }
  }, [session?.filePath]);

  // 初始加载会话内容
  useEffect(() => {
    loadSessionContent();
  }, [loadSessionContent]);

  // 返回主页
  const handleBack = useCallback(() => {
    navigate('/sessions');
  }, [navigate]);

  /**
   * 准备导出数据
   */
  const exportData = useMemo<ExportData>(() => {
    if (!conversationTree || !session) {
      return {
        sessionId: session?.sessionId || '',
        title: session?.sessionId || '',
        projectPath: session?.projectName,
        createdAt: session?.createdAt,
        messages: []
      };
    }

    // 递归转换树节点为导出格式
    const convertNodes = (nodes: MessageNode[]): any[] => {
      const result: any[] = [];
      for (const node of nodes) {
        const message: any = {
          role: node.role || 'assistant',
          content: node.content || ''
        };

        if (node.timestamp) {
          message.timestamp = node.timestamp;
        }

        if (node.metadata?.code_changes && node.metadata.code_changes.length > 0) {
          message.codeBlocks = node.metadata.code_changes.map((change: any) => ({
            language: change.file_path?.split('.').pop() || 'text',
            code: change.new_text || change.old_text || ''
          }));
        }

        if (node.metadata) {
          message.metadata = node.metadata;
        }

        result.push(message);

        if (node.children && node.children.length > 0) {
          result.push(...convertNodes(node.children));
        }
      }
      return result;
    };

    return {
      sessionId: session.sessionId,
      title: session.sessionId,
      projectPath: session.projectName,
      createdAt: session.createdAt,
      messages: convertNodes(conversationTree.roots),
      stats: tokenStats
        ? {
            totalMessages: tokenStats.messageCount,
            totalTokens: tokenStats.totalTokens
          }
        : undefined
    };
  }, [conversationTree, session, tokenStats]);

  // 会话不存在
  if (!session) {
    return (
      <div className={cn('flex flex-col h-full items-center justify-center', className)} style={{ backgroundColor: 'var(--color-bg-primary)' }}>
        <Alert variant="destructive" className="max-w-md">
          <AlertDescription>
            会话不存在或已被删除
          </AlertDescription>
        </Alert>
        <Button variant="outline" onClick={handleBack} className="mt-4">
          返回会话列表
        </Button>
      </div>
    );
  }

  return (
    <div className={cn('flex flex-col h-screen', className)} style={{ backgroundColor: 'var(--color-bg-primary)' }}>
      {/* 顶部导航栏 */}
      <div className="flex items-center gap-4 px-6 py-4 border-b" style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}>
        <Button
          variant="ghost"
          size="icon"
          onClick={handleBack}
          className="shrink-0 hover:bg-[var(--color-app-secondary)]"
        >
          <ArrowLeft className="h-5 w-5" style={{ color: 'var(--color-text-primary)' }} />
        </Button>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <FileText className="h-5 w-5 shrink-0" style={{ color: 'var(--color-text-primary)' }} />
            <h1 className="text-xl font-bold truncate text-foreground">
              {session.projectName}
            </h1>
          </div>
          <p className="text-sm text-muted-foreground truncate mt-0.5">
            {session.sessionId.slice(0, 8)}...
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={() => setShowExportDialog(true)}
          className="shrink-0"
        >
          <Download className="h-4 w-4 mr-2" />
          导出
        </Button>
        <Button
          variant="ghost"
          size="icon"
          onClick={loadSessionContent}
          disabled={loading}
          className="shrink-0 hover:bg-[var(--color-app-secondary)]"
          title="刷新"
        >
          <RefreshCw className={cn('h-4 w-4', loading && 'animate-spin')} style={{ color: 'var(--color-text-primary)' }} />
        </Button>
        <ThemeToggle />
      </div>

      {/* 主内容区域 - 左右分栏布局 */}
      <div className="flex-1 min-h-0 overflow-hidden">
        <div className="flex h-full">
          {/* 左侧：消息列表 */}
          <div className="flex-1 min-w-0 overflow-y-auto" style={{ backgroundColor: 'var(--color-app-result-bg)' }}>
            <div className="max-w-4xl mx-auto p-6 space-y-4">
              {/* 消息列表标题 */}
              <div className="flex items-center justify-between sticky top-0 z-10 py-3">
                <h2 className="text-lg font-semibold" style={{ color: 'var(--color-text-primary)' }}>
                  Messages ({messageList.length})
                </h2>
              </div>

              {/* 加载状态 */}
              {loading && (
                <div className="flex items-center justify-center py-12">
                  <div className="text-center space-y-2">
                    <RefreshCw className="h-8 w-8 animate-spin mx-auto" style={{ color: 'var(--color-text-primary)' }} />
                    <p className="text-sm" style={{ color: 'var(--color-text-secondary)' }}>加载中...</p>
                  </div>
                </div>
              )}

              {/* 错误状态 */}
              {error && (
                <Alert variant="destructive">
                  <AlertDescription>{error}</AlertDescription>
                </Alert>
              )}

              {/* 消息卡片列表 */}
              {!loading && !error && messageList.length > 0 && (
                <div className="space-y-4 pb-6">
                  {messageList.map((message) => (
                    <MessageCard
                      key={message.id}
                      role={message.role || 'assistant'}
                      content={message.content || ''}
                      timestamp={message.timestamp}
                      maxContentLength={1000}
                    />
                  ))}
                </div>
              )}

              {/* 空状态 */}
              {!loading && !error && messageList.length === 0 && (
                <div className="flex flex-col items-center justify-center py-12 text-center">
                  <FileText className="h-12 w-12 mb-4" style={{ color: 'var(--color-text-secondary)' }} />
                  <p className="font-medium" style={{ color: 'var(--color-text-primary)' }}>暂无消息</p>
                  <p className="text-sm mt-2" style={{ color: 'var(--color-text-secondary)' }}>
                    该会话文件为空或格式不正确
                  </p>
                </div>
              )}
            </div>
          </div>

          {/* 右侧：统计信息边栏 */}
          <div className="w-[30%] min-w-[280px] max-w-md shrink-0 border-l overflow-y-auto" style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}>
            <div className="sticky top-0 p-4">
              <SessionStatsSidebar
                sessionId={session.sessionId}
                projectName={session.projectName}
                projectPath={session.projectPath}
                rating={session.rating}
                tags={session.tags}
                createdAt={session.createdAt}
                updatedAt={session.updatedAt}
                tokenStats={tokenStats || undefined}
                messageCount={conversationTree?.total_count}
              />
            </div>
          </div>
        </div>
      </div>

      {/* 导出对话框 */}
      <ExportDialog
        open={showExportDialog}
        onOpenChange={setShowExportDialog}
        data={exportData}
        onExportComplete={(filename) => {
          console.log('导出完成:', filename);
        }}
      />
    </div>
  );
}
