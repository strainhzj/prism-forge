/**
 * PromptBuilder 组件
 *
 * 提示词构建器，用于生成增强的 AI 提示词
 * 包含目标输入、会话选择、生成、预览、统计、复制/保存功能
 */

import { useState, useCallback } from 'react';
import { Wand2, Copy, Save, Check, Loader2, AlertCircle, Info } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Badge } from '@/components/ui/badge';
import { Card } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { useCurrentSessionFilePath, useCurrentSession } from '@/stores/useCurrentSessionStore';
import { useCurrentLanguage } from '@/stores/useLanguageStore';
import type { EnhancedPrompt, EnhancedPromptRequest } from '@/types/prompt';

export interface PromptBuilderProps {
  /**
   * 初始目标
   */
  initialGoal?: string;
  /**
   * 生成完成回调
   */
  onGenerated?: (result: EnhancedPrompt) => void;
  /**
   * 自定义类名
   */
  className?: string;
}

/**
 * PromptBuilder 组件
 */
export function PromptBuilder({
  initialGoal = '',
  onGenerated,
  className,
}: PromptBuilderProps) {
  const [goal, setGoal] = useState(initialGoal);
  const [result, setResult] = useState<EnhancedPrompt | null>(null);
  const [isGenerating, setIsGenerating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [saved, setSaved] = useState(false);

  // 获取当前会话文件路径
  const currentSessionFilePath = useCurrentSessionFilePath();
  const currentSession = useCurrentSession();
  // 获取当前语言（用于提示词优化）
  const currentLanguage = useCurrentLanguage();

  // 调试日志
  const DEBUG = import.meta.env.DEV;
  if (DEBUG) {
    console.log('[PromptBuilder] currentSession:', currentSession);
    console.log('[PromptBuilder] currentSessionFilePath:', currentSessionFilePath);
  }

  /**
   * 生成增强提示词
   */
  const handleGenerate = useCallback(async () => {
    if (!goal.trim()) {
      setError('请输入目标描述');
      return;
    }

    // 检查是否有当前会话
    if (!currentSessionFilePath) {
      setError('请先在首页选择一个会话');
      return;
    }

    setIsGenerating(true);
    setError(null);
    setResult(null);

    try {
      const request: EnhancedPromptRequest = {
        goal: goal.trim(),
        currentSessionFilePath,
      };

      const response = await invoke<EnhancedPrompt>('optimize_prompt', {
        request,
        language: currentLanguage,  // 传递当前语言
      });

      setResult(response);
      setSaved(false);
      onGenerated?.(response);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : '生成失败';
      setError(errorMessage);
      console.error('生成提示词失败:', err);
    } finally {
      setIsGenerating(false);
    }
  }, [goal, currentSessionFilePath, onGenerated]);

  /**
   * 复制增强提示词
   */
  const handleCopy = useCallback(async () => {
    if (!result) return;

    try {
      await navigator.clipboard.writeText(result.enhancedPrompt);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('复制失败:', err);
    }
  }, [result]);

  /**
   * 保存提示词
   */
  const handleSave = useCallback(async () => {
    if (!result) return;

    try {
      // TODO: 实现保存到数据库的功能
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      console.error('保存失败:', err);
    }
  }, [result]);

  /**
   * 计算节省百分比的颜色
   */
  const getSavingsColor = useCallback((percentage: number) => {
    if (percentage >= 50) return 'text-green-500';
    if (percentage >= 30) return 'text-blue-500';
    if (percentage >= 10) return 'text-yellow-500';
    return 'text-gray-500';
  }, []);

  /**
   * 计算置信度的颜色
   */
  const getConfidenceColor = useCallback((confidence: number) => {
    if (confidence >= 0.8) return 'bg-green-500/10 text-green-500';
    if (confidence >= 0.5) return 'bg-yellow-500/10 text-yellow-500';
    return 'bg-red-500/10 text-red-500';
  }, []);

  /**
   * 重置状态
   */
  const handleReset = useCallback(() => {
    setResult(null);
    setError(null);
    setSaved(false);
  }, []);

  return (
    <div className={cn('space-y-4', className)}>
      {/* 输入区域 */}
      <Card className="p-4">
        <div className="space-y-3">
          {/* 会话状态提示 */}
          {currentSessionFilePath ? (
            <div className="flex items-center gap-2 p-2 bg-green-500/10 rounded-md border border-green-500/20">
              <Check className="h-4 w-4 text-green-500" />
              <p className="text-xs text-green-500">已选择当前会话</p>
            </div>
          ) : (
            <div className="flex items-center gap-2 p-2 bg-amber-500/10 rounded-md border border-amber-500/20">
              <Info className="h-4 w-4 text-amber-500" />
              <p className="text-xs text-amber-600 dark:text-amber-400">
                请先在首页选择一个会话，然后返回此处生成提示词
              </p>
            </div>
          )}

          <div>
            <Label htmlFor="goal-input" className="text-base font-medium">
              目标描述
            </Label>
            <p className="text-xs text-muted-foreground mb-2">
              描述您想要完成的任务，AI 会基于当前会话的对话记录生成优化的提示词
            </p>
            <Textarea
              id="goal-input"
              placeholder="例如：实现用户登录功能，包含邮箱验证和记住密码..."
              value={goal}
              onChange={(e) => setGoal(e.target.value)}
              rows={4}
              className="resize-none"
              disabled={isGenerating}
            />
          </div>

          {/* 错误提示 */}
          {error && (
            <div className="flex items-center gap-2 p-3 bg-destructive/10 rounded-md border border-destructive/20">
              <AlertCircle className="h-4 w-4 text-destructive" />
              <p className="text-sm text-destructive">{error}</p>
            </div>
          )}

          {/* 操作按钮 */}
          <div className="flex items-center justify-between">
            <p className="text-xs text-muted-foreground">
              {goal.trim().length > 0 && `${goal.trim().length} 字符`}
            </p>
            <Button
              onClick={handleGenerate}
              disabled={isGenerating || !goal.trim() || !currentSessionFilePath}
              className="min-w-[120px]"
            >
              {isGenerating ? (
                <>
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  生成中...
                </>
              ) : (
                <>
                  <Wand2 className="h-4 w-4 mr-2" />
                  生成提示词
                </>
              )}
            </Button>
          </div>
        </div>
      </Card>

      {/* 结果区域 */}
      {result && (
        <Card className="p-4">
          <div className="space-y-4">
            {/* 统计信息 */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              {/* Token 统计 */}
              <div className="p-3 bg-muted/30 rounded-md">
                <p className="text-xs text-muted-foreground mb-1">Token 统计</p>
                <div className="flex items-baseline gap-2">
                  <span className="text-2xl font-bold">
                    {result.tokenStats.compressedTokens.toLocaleString()}
                  </span>
                  <span className="text-sm text-muted-foreground">
                    / {result.tokenStats.originalTokens.toLocaleString()}
                  </span>
                </div>
                {result.tokenStats.savingsPercentage > 0 && (
                  <p className={cn('text-xs font-medium mt-1', getSavingsColor(result.tokenStats.savingsPercentage))}>
                    节省 {result.tokenStats.savingsPercentage.toFixed(1)}%
                  </p>
                )}
              </div>

              {/* 引用会话 */}
              <div className="p-3 bg-muted/30 rounded-md">
                <p className="text-xs text-muted-foreground mb-1">引用会话</p>
                <p className="text-2xl font-bold">
                  {result.referencedSessions.length}
                </p>
                <p className="text-xs text-muted-foreground mt-1">
                  {result.referencedSessions.length > 0 ? '相关历史会话' : '无相关会话'}
                </p>
              </div>

              {/* 置信度 */}
              <div className="p-3 bg-muted/30 rounded-md">
                <p className="text-xs text-muted-foreground mb-1">置信度</p>
                <div className="flex items-center gap-2">
                  <span className="text-2xl font-bold">
                    {(result.confidence * 100).toFixed(0)}%
                  </span>
                  <Badge className={cn('text-xs', getConfidenceColor(result.confidence))}>
                    {result.confidence >= 0.8 ? '高' : result.confidence >= 0.5 ? '中' : '低'}
                  </Badge>
                </div>
              </div>
            </div>

            {/* 引用的会话列表 */}
            {result.referencedSessions.length > 0 && (
              <div className="space-y-2">
                <Label className="text-sm font-medium">引用的会话</Label>
                <div className="space-y-1 max-h-40 overflow-y-auto">
                  {result.referencedSessions.map((session) => (
                    <div
                      key={session.sessionId}
                      className="flex items-start justify-between p-2 bg-muted/30 rounded-md text-xs"
                    >
                      <div className="flex-1 min-w-0">
                        <p className="font-medium truncate">{session.summary}</p>
                        <p className="text-muted-foreground truncate mt-0.5">
                          {session.projectName}
                        </p>
                      </div>
                      <div className="flex items-center gap-2 ml-2">
                        <Badge variant="outline" className="text-xs">
                          {(session.similarityScore * 100).toFixed(0)}%
                        </Badge>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* 增强的提示词 */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label className="text-sm font-medium">增强的提示词</Label>
                <div className="flex items-center gap-2">
                  {/* 保存按钮 */}
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={handleSave}
                    className="h-7 px-2"
                  >
                    {saved ? (
                      <>
                        <Check className="h-4 w-4 mr-1 text-green-500" />
                        <span className="text-xs">已保存</span>
                      </>
                    ) : (
                      <>
                        <Save className="h-4 w-4 mr-1" />
                        <span className="text-xs">保存</span>
                      </>
                    )}
                  </Button>

                  {/* 复制按钮 */}
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={handleCopy}
                    className="h-7 px-2"
                  >
                    {copied ? (
                      <>
                        <Check className="h-4 w-4 mr-1 text-green-500" />
                        <span className="text-xs">已复制</span>
                      </>
                    ) : (
                      <>
                        <Copy className="h-4 w-4 mr-1" />
                        <span className="text-xs">复制</span>
                      </>
                    )}
                  </Button>
                </div>
              </div>

              <div className="relative">
                <Textarea
                  value={result.enhancedPrompt}
                  readOnly
                  rows={12}
                  className="resize-none font-mono text-sm bg-background"
                />
                <div className="absolute bottom-2 right-2 text-xs text-muted-foreground bg-background/80 px-2 py-1 rounded">
                  {result.enhancedPrompt.length} 字符
                </div>
              </div>
            </div>

            {/* 重置按钮 */}
            <div className="flex justify-center pt-2">
              <Button
                variant="outline"
                size="sm"
                onClick={handleReset}
                className="min-w-[120px]"
              >
                重新生成
              </Button>
            </div>
          </div>
        </Card>
      )}
    </div>
  );
}

/**
 * PromptBuilder 组件的简洁版本（用于嵌入其他页面）
 */
export interface PromptBuilderCompactProps {
  onGenerated?: (result: EnhancedPrompt) => void;
  className?: string;
}

export function PromptBuilderCompact({ onGenerated, className }: PromptBuilderCompactProps) {
  return (
    <PromptBuilder
      onGenerated={onGenerated}
      className={className}
    />
  );
}
