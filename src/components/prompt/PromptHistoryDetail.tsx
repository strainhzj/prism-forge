/**
 * 提示词历史详情弹窗
 */

import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import {
  Clock,
  Star,
  Copy,
  Check,
  Cpu,
  MessageSquare,
  BarChart3,
} from 'lucide-react';
import type { PromptGenerationHistory } from '@/types/generated';

interface PromptHistoryDetailProps {
  history: PromptGenerationHistory | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

/**
 * 提示词历史详情弹窗组件
 */
export function PromptHistoryDetail({
  history,
  open,
  onOpenChange,
}: PromptHistoryDetailProps) {
  const { t } = useTranslation('promptLab');
  const [copied, setCopied] = useState(false);

  if (!history) return null;

  /**
   * 复制提示词到剪贴板
   */
  const handleCopyPrompt = async () => {
    try {
      await navigator.clipboard.writeText(history.enhancedPrompt);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('复制失败:', err);
    }
  };

  /**
   * 格式化时间
   */
  const formatTime = (timeStr: string) => {
    try {
      const date = new Date(timeStr);
      return new Intl.DateTimeFormat('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
      }).format(date);
    } catch {
      return timeStr;
    }
  };

  /**
   * 解析 JSON 字段
   */
  const parseJsonField = (jsonStr: string | null) => {
    if (!jsonStr) return null;
    try {
      return JSON.parse(jsonStr);
    } catch {
      return null;
    }
  };

  const referencedSessions = parseJsonField(history.referencedSessions);
  const tokenStats = parseJsonField(history.tokenStats);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl max-h-[80vh]">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Clock className="h-5 w-5" />
            {formatTime(history.createdAt)}
          </DialogTitle>
        </DialogHeader>

        <div className="max-h-[60vh] overflow-y-auto pr-4">
          <div className="space-y-6">
            {/* 元信息 */}
            <div className="space-y-4">
              {/* 收藏状态 */}
              <div className="flex items-center gap-2">
                {history.isFavorite ? (
                  <Badge variant="secondary" className="gap-1">
                    <Star className="h-3 w-3 fill-yellow-400 text-yellow-400" />
                    {t('history.favorited')}
                  </Badge>
                ) : (
                  <Badge variant="outline">{t('history.notFavorited')}</Badge>
                )}
              </div>

              {/* LLM 信息 */}
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <Cpu className="h-4 w-4" />
                <span>
                  {history.llmProvider || t('history.unknown')} / {history.llmModel || t('history.unknown')}
                </span>
              </div>

              {/* 置信度 */}
              {history.confidence !== null && history.confidence !== undefined && (
                <div className="flex items-center gap-2 text-sm">
                  <BarChart3 className="h-4 w-4 text-muted-foreground" />
                  <span className="text-muted-foreground">{t('history.confidence')}:</span>
                  <Badge variant="secondary">
                    {Math.round(history.confidence * 100)}%
                  </Badge>
                </div>
              )}

              {/* 引用会话数 */}
              {referencedSessions && Array.isArray(referencedSessions) && (
                <div className="flex items-center gap-2 text-sm">
                  <MessageSquare className="h-4 w-4 text-muted-foreground" />
                  <span className="text-muted-foreground">{t('history.referencedSessions')}:</span>
                  <Badge variant="secondary">{referencedSessions.length}</Badge>
                </div>
              )}
            </div>

            <Separator />

            {/* 原始目标 */}
            <div className="space-y-2">
              <h3 className="font-semibold text-sm">{t('history.originalGoal')}</h3>
              <p className="text-sm text-muted-foreground whitespace-pre-wrap">
                {history.originalGoal}
              </p>
            </div>

            <Separator />

            {/* 优化后的提示词 */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <h3 className="font-semibold text-sm">{t('history.enhancedPrompt')}</h3>
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-8 gap-1"
                  onClick={handleCopyPrompt}
                >
                  {copied ? (
                    <>
                      <Check className="h-4 w-4 text-green-500" />
                      {t('history.copied')}
                    </>
                  ) : (
                    <>
                      <Copy className="h-4 w-4" />
                      {t('history.copy')}
                    </>
                  )}
                </Button>
              </div>
              <div className="rounded-lg bg-muted/50 p-4">
                <pre className="text-sm whitespace-pre-wrap font-mono">
                  {history.enhancedPrompt}
                </pre>
              </div>
            </div>

            {/* Token 统计 */}
            {tokenStats && (
              <>
                <Separator />
                <div className="space-y-2">
                  <h3 className="font-semibold text-sm">{t('history.tokenStats')}</h3>
                  <div className="grid grid-cols-2 gap-2 text-sm">
                    {typeof tokenStats === 'object' && Object.entries(tokenStats).map(([key, value]) => (
                      <div key={key} className="flex justify-between">
                        <span className="text-muted-foreground">{key}:</span>
                        <span className="font-mono">{String(value)}</span>
                      </div>
                    ))}
                  </div>
                </div>
              </>
            )}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
