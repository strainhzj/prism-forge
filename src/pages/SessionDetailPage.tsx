/**
 * SessionDetailPage 组件
 *
 * 会话详情页面 - 显示消息树、提取等级选择等
 */

import { useCallback, useEffect, useState, useMemo } from 'react';
import { useNavigate, useSearchParams, useParams } from 'react-router-dom';
import { ArrowLeft, FileText, Edit3, Check } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { ViewLevelTabs, ExtractionLevel } from '@/components/ViewLevelSelector';
import { MessageTree } from '@/components/MessageTree';
import { TokenStatsCard } from '@/components/TokenCounter';
import { SessionRating } from '@/components/SessionRating';
import { TagEditor, TagDisplay } from '@/components/TagEditor';
import { Badge } from '@/components/ui/badge';
import { useSessions, useSessionActions } from '@/stores/useSessionStore';
import type { ConversationTree } from '@/types/message';

/**
 * SessionDetailPage 组件
 *
 * @example
 * 路由: /sessions/:sessionId
 */
export function SessionDetailPage({ className }: { className?: string }) {
  const navigate = useNavigate();
  const { sessionId } = useParams<{ sessionId: string }>();
  const [searchParams, setSearchParams] = useSearchParams();
  const sessions = useSessions();
  const { setActiveSessions, setSessionRating } = useSessionActions();

  // Token 统计状态
  const [conversationTree, setConversationTree] = useState<ConversationTree | null>(null);
  // 默认使用 gpt-4o-mini 模型进行定价估算
  const selectedModel = 'gpt-4o-mini';

  // 标签编辑状态
  const [isEditingTags, setIsEditingTags] = useState(false);
  const [editingTags, setEditingTags] = useState<string[]>([]);

  // 查找当前会话
  const session = sessions.find((s) => s.sessionId === sessionId);

  // 计算 Token 统计（简化估算）
  const tokenStats = useMemo(() => {
    if (!conversationTree) return null;

    // 递归计算树中所有内容的 Token 数
    let totalTextLength = 0;
    let userMessages = 0;
    let assistantMessages = 0;

    const traverseTree = (nodes: any[]) => {
      for (const node of nodes) {
        if (node.content) {
          totalTextLength += node.content.length;
          if (node.role === 'user') userMessages++;
          if (node.role === 'assistant') assistantMessages++;
        }
        if (node.children && node.children.length > 0) {
          traverseTree(node.children);
        }
      }
    };

    traverseTree(conversationTree.roots);

    // 简单估算：中文约 2 token/字，英文约 0.25 token/字符
    const estimatedTokens = Math.round(totalTextLength * 0.5);
    const inputTokens = Math.round(estimatedTokens * 0.6); // 假设 60% 是输入
    const outputTokens = Math.round(estimatedTokens * 0.4); // 假设 40% 是输出

    return {
      inputTokens,
      outputTokens,
      totalTokens: estimatedTokens,
      messageCount: conversationTree.total_count,
    };
  }, [conversationTree]);

  // 从 URL 参数获取视图等级，默认 L2
  const viewLevel = (searchParams.get('view') as ExtractionLevel) || ExtractionLevel.L2CleanFlow;

  // 初始加载会话列表
  useEffect(() => {
    if (sessions.length === 0) {
      setActiveSessions();
    }
  }, [setActiveSessions, sessions.length]);

  // 返回主页
  const handleBack = useCallback(() => {
    navigate('/sessions');
  }, [navigate]);

  // 切换视图等级
  const handleViewLevelChange = useCallback(
    (level: ExtractionLevel) => {
      setSearchParams({ view: level });
    },
    [setSearchParams]
  );

  // 处理评分变更
  const handleRatingChange = useCallback(
    async (rating: number | null) => {
      if (!session) return;
      try {
        await setSessionRating({ sessionId: session.sessionId, rating });
      } catch (error) {
        console.error('设置评分失败:', error);
      }
    },
    [session, setSessionRating]
  );

  // 开始编辑标签
  const handleStartEditTags = useCallback(() => {
    if (!session) return;
    // 解析标签字符串（逗号或空格分隔）
    const tags = session.tags
      ? session.tags.split(/[,，\s]+/).filter(Boolean)
      : [];
    setEditingTags(tags);
    setIsEditingTags(true);
  }, [session]);

  // 保存标签
  const handleSaveTags = useCallback(async () => {
    if (!session) return;
    // TODO: 调用后端 API 保存标签
    // 目前暂时更新本地状态
    setIsEditingTags(false);
    console.log('保存标签:', editingTags);
  }, [session, editingTags]);

  // 取消编辑标签
  const handleCancelEditTags = useCallback(() => {
    setIsEditingTags(false);
    setEditingTags([]);
  }, []);

  // 解析会话标签
  const sessionTags = useMemo(() => {
    if (!session?.tags) return [];
    return session.tags.split(/[,，\s]+/).filter(Boolean);
  }, [session?.tags]);

  // 会话不存在
  if (!session) {
    return (
      <div className={cn('flex flex-col h-full items-center justify-center', className)}>
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
    <div className={cn('flex flex-col h-screen', className)}>
      {/* 顶部导航栏 */}
      <div className="flex items-center gap-4 px-6 py-4 border-b bg-background">
        <Button
          variant="ghost"
          size="icon"
          onClick={handleBack}
          className="shrink-0"
        >
          <ArrowLeft className="h-5 w-5" />
        </Button>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <FileText className="h-5 w-5 text-primary shrink-0" />
            <h1 className="text-xl font-bold truncate">会话详情</h1>
          </div>
          <p className="text-sm text-muted-foreground truncate mt-0.5">
            {session.projectName} · {session.sessionId.slice(0, 8)}...
          </p>
        </div>
      </div>

      {/* 评分和标签区域 */}
      <div className="px-6 py-4 border-b bg-muted/30">
        <div className="max-w-4xl mx-auto space-y-4">
          {/* 评分 */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <span className="text-sm font-medium">评分:</span>
              <SessionRating
                rating={session.rating}
                onRatingChange={handleRatingChange}
                size="md"
              />
              {session.rating && (
                <Badge variant="secondary" className="text-xs">
                  {session.rating} 星
                </Badge>
              )}
            </div>
          </div>

          {/* 标签 */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium">标签:</span>
              {!isEditingTags && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={handleStartEditTags}
                  className="h-7 text-xs"
                >
                  <Edit3 className="h-3 w-3 mr-1" />
                  编辑
                </Button>
              )}
            </div>

            {isEditingTags ? (
              <div className="space-y-2">
                <TagEditor
                  tags={editingTags}
                  onTagsChange={setEditingTags}
                  maxTags={10}
                  showRecommended={true}
                />
                <div className="flex items-center gap-2">
                  <Button
                    size="sm"
                    onClick={handleSaveTags}
                    className="h-7"
                  >
                    <Check className="h-3.5 w-3.5 mr-1" />
                    保存
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleCancelEditTags}
                    className="h-7"
                  >
                    取消
                  </Button>
                </div>
              </div>
            ) : (
              <TagDisplay
                tags={sessionTags}
                maxDisplay={10}
                size="sm"
              />
            )}
          </div>
        </div>
      </div>

      {/* 视图等级选择器 */}
      <div className="px-6 py-4 border-b bg-background/50">
        <ViewLevelTabs value={viewLevel} onChange={handleViewLevelChange} />
      </div>

      {/* Token 统计卡片 */}
      {tokenStats && (
        <div className="px-6 py-3 border-b bg-muted/30">
          <TokenStatsCard
            inputTokens={tokenStats.inputTokens}
            outputTokens={tokenStats.outputTokens}
            model={selectedModel}
            className="max-w-2xl mx-auto"
          />
        </div>
      )}

      {/* 主内容区域 */}
      <div className="flex-1 min-h-0">
        <MessageTree
          filePath={session.filePath}
          lazy={true}
          onTreeLoaded={setConversationTree}
          className="h-full"
        />
      </div>
    </div>
  );
}
