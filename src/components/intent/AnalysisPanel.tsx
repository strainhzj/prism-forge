/**
 * AnalysisPanel 组件
 *
 * 会话意图分析结果弹窗（三栏布局）
 * - 左侧：问答对列表
 * - 中间：开场白意图
 * - 右侧：决策点列表
 *
 * 新增功能：
 * - 支持历史记录缓存
 * - 显示上次分析时间
 * - 提供重新分析按钮
 * - 提供清除历史按钮
 */

import { useState, useEffect, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { Loader2, RotateCcw, Trash2, Target } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';
import { zhCN, enUS } from 'date-fns/locale';

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';

import type { DecisionQAPair } from '@/types/generated';
import type { OpeningIntent } from '@/types/generated/OpeningIntent';
import type { IntentAnalysisHistory } from '@/types/generated/IntentAnalysisHistory';

import { QAPairList } from './QAPairList';
import { IntentPanel } from './IntentPanel';
import { DecisionList } from './DecisionList';
import { mapIntentAnalysisHistory } from '@/lib/intentMapper';

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[AnalysisPanel] ${action}`, ...args);
  }
}

// ==================== 类型定义 ====================

export interface AnalysisPanelProps {
  /**
   * 会话文件路径
   */
  sessionFilePath: string;
  /**
   * 是否显示弹窗
   */
  isOpen: boolean;
  /**
   * 关闭弹窗回调
   */
  onClose: () => void;
  /**
   * 语言标识（"zh" 或 "en"）
   */
  language?: string;
}

// ==================== 组件 ====================

/**
 * AnalysisPanel - 会话意图分析弹窗
 */
export function AnalysisPanel({
  sessionFilePath,
  isOpen,
  onClose,
  language = 'zh',
}: AnalysisPanelProps) {
  const { t } = useTranslation('intentAnalysis');

  // 状态管理
  const [loading, setLoading] = useState(false);
  const [loadingHistory, setLoadingHistory] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [qaPairs, setQaPairs] = useState<DecisionQAPair[]>([]);  // ✅ 确保初始值是空数组
  const [openingIntent, setOpeningIntent] = useState<OpeningIntent | null>(null);
  const [selectedQaPair, setSelectedQaPair] = useState<DecisionQAPair | null>(null);
  const [selectedQaIndex, setSelectedQaIndex] = useState<number | null>(null);
  const [history, setHistory] = useState<IntentAnalysisHistory | null>(null);

  /**
   * 从历史记录加载数据
   * @returns 是否成功加载到历史记录
   */
  const loadFromHistory = useCallback(async (): Promise<boolean> => {
    setLoadingHistory(true);
    try {
      const rawData = await invoke<any>('cmd_get_intent_analysis_history', {
        sessionFilePath,
      });

      if (rawData) {
        // 🔧 使用映射工具修复字段命名问题
        const historyData = mapIntentAnalysisHistory(rawData);

        if (DEBUG) {
          console.log('[AnalysisPanel] 从历史记录加载 - 映射完成', {
            hasOpeningIntent: !!historyData?.openingIntent,
            intentType: historyData?.openingIntent?.intentType,
          });
        }

        setHistory(historyData);
        setQaPairs(historyData.qaPairs ?? []);
        setOpeningIntent(historyData.openingIntent);
        return true;
      }
      return false;
    } catch (err) {
      debugLog('加载历史记录失败', { error: err });
      setHistory(null);
      return false;
    } finally {
      setLoadingHistory(false);
    }
  }, [sessionFilePath]);

  /**
   * 执行分析流程
   */
  const performAnalysis = useCallback(async (saveToHistory = true) => {
    setLoading(true);
    setError(null);

    try {
      debugLog('开始分析会话', { sessionFilePath });

      // 1. 检测问答对
      const pairs = await invoke<DecisionQAPair[]>('cmd_detect_qa_pairs', {
        sessionFilePath,
      });
      setQaPairs(pairs);
      debugLog('检测到问答对', { count: pairs.length });

      if (pairs.length === 0) {
        setError(t('errors.noQAPairs'));
        return;
      }

      // 2. 分析开场白意图
      const intent = await invoke<OpeningIntent>('cmd_analyze_opening_intent', {
        sessionFilePath,
        language,
      });
      setOpeningIntent(intent);
      debugLog('开场白意图分析完成', { intent });

      // 3. 保存到历史记录
      if (saveToHistory) {
        try {
          await invoke('cmd_save_intent_analysis', {
            sessionFilePath,
            qaPairs: pairs,
            openingIntent: intent,
            language,
          });
          // 更新本地历史状态
          const newHistory: IntentAnalysisHistory = {
            id: BigInt(0), // 保存后会被后端设置
            sessionFilePath,
            qaPairs: pairs,
            openingIntent: intent,
            language,
            analyzedAt: new Date().toISOString(),
            createdAt: new Date().toISOString(),
          };
          setHistory(newHistory);
          debugLog('已保存到历史记录');
        } catch (saveErr) {
          debugLog('保存历史记录失败', { error: saveErr });
        }
      }

    } catch (err) {
      // Tauri invoke 错误格式: { error: string | { message: string } }
      let errorMessage = t('errors.analysisFailed');

      if (typeof err === 'string') {
        errorMessage = err;
      } else if (err && typeof err === 'object') {
        if ('error' in err) {
          const errorValue = (err as { error: unknown }).error;
          if (typeof errorValue === 'string') {
            errorMessage = errorValue;
          } else if (errorValue && typeof errorValue === 'object' && 'message' in errorValue) {
            errorMessage = (errorValue as { message: string }).message;
          }
        } else if ('message' in err) {
          errorMessage = (err as { message: string }).message;
        }
      }

      debugLog('分析失败', { error: err, errorMessage });
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  }, [sessionFilePath, language, t]);

  /**
   * 重新分析
   */
  const handleReanalyze = useCallback(() => {
    setHistory(null);
    performAnalysis(true);
  }, [performAnalysis]);

  /**
   * 清除历史记录
   */
  const handleClearHistory = useCallback(async () => {
    if (!confirm(t('actions.clearHistoryConfirm'))) {
      return;
    }

    try {
      await invoke('cmd_clear_intent_analysis_history', {
        sessionFilePath,
      });
      setHistory(null);
      setQaPairs([]);
      setOpeningIntent(null);
      setSelectedQaPair(null);
      setSelectedQaIndex(null);
      debugLog('已清除历史记录');
    } catch (err) {
      debugLog('清除历史记录失败', { error: err });
    }
  }, [sessionFilePath, t]);

  /**
   * 选择问答对
   */
  const handleSelectQAPair = useCallback((qaPair: DecisionQAPair, index: number) => {
    setSelectedQaPair(qaPair);
    setSelectedQaIndex(index);
    debugLog('选择问答对', { index, qaPair });
  }, []);

  /**
   * 重试分析
   */
  const handleRetry = useCallback(() => {
    performAnalysis(true);
  }, [performAnalysis]);

  /**
   * 格式化相对时间
   */
  const formatRelativeTime = useCallback((timestamp: string) => {
    try {
      const date = new Date(timestamp);
      const locale = language === 'zh' ? zhCN : enUS;
      return formatDistanceToNow(date, { addSuffix: true, locale });
    } catch {
      return timestamp;
    }
  }, [language]);

  /**
   * 弹窗打开时自动加载历史或执行分析
   */
  useEffect(() => {
    if (isOpen) {
      // 先尝试加载历史记录
      loadFromHistory().then((hasHistory) => {
        // 如果没有历史记录，则执行分析
        if (!hasHistory) {
          performAnalysis(true);
        }
      });
    } else {
      // 弹窗关闭时重置状态
      setQaPairs([]);
      setOpeningIntent(null);
      setSelectedQaPair(null);
      setSelectedQaIndex(null);
      setError(null);
      setHistory(null);
      setLoadingHistory(true);
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isOpen]);  // ✅ 只依赖 isOpen，避免循环触发

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <DialogContent
        className="max-w-6xl max-h-[80vh] overflow-hidden"
        style={{ backgroundColor: 'var(--color-bg-card)' }}
      >
        <DialogHeader>
          <div className="flex items-center justify-between">
            <DialogTitle style={{ color: 'var(--color-text-primary)' }}>
              {t('panel.title')}
            </DialogTitle>
            <div className="flex gap-2">
              {history && !loading && (
                <>
                  {/* 显示上次分析时间 */}
                  <span className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
                    {t('panel.lastAnalyzedAt', {
                      time: formatRelativeTime(history.analyzedAt),
                    })}
                  </span>
                  {/* 重新分析按钮 */}
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={handleReanalyze}
                    className="h-7 px-2"
                    title={t('actions.reanalyze')}
                  >
                    <RotateCcw className="w-4 h-4" />
                  </Button>
                  {/* 清除历史按钮 */}
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={handleClearHistory}
                    className="h-7 px-2"
                    title={t('actions.clearHistory')}
                  >
                    <Trash2 className="w-4 h-4" />
                  </Button>
                </>
              )}
            </div>
          </div>
        </DialogHeader>

        {/* 内容区域 */}
        <div className="mt-4">
          {loading || loadingHistory ? (
            // 加载状态
            <div className="flex flex-col items-center justify-center py-12">
              <Loader2 className="w-8 h-8 animate-spin mb-4" style={{ color: 'var(--color-accent-blue)' }} />
              <p className="text-sm" style={{ color: 'var(--color-text-secondary)' }}>
                {loading ? t('panel.analyzing') : t('panel.loadingHistory')}
              </p>
            </div>
          ) : error ? (
            // 错误状态
            <div className="flex flex-col items-center justify-center py-12">
              <p className="font-medium mb-2" style={{ color: 'var(--color-app-error-accent)' }}>
                {error}
              </p>
              <Button
                onClick={handleRetry}
                variant="outline"
                size="sm"
              >
                {t('actions.retry')}
              </Button>
            </div>
          ) : !qaPairs || qaPairs.length === 0 ? (  // ✅ 添加 qaPairs 空值检查
            // 无问答对状态 - 区分开场白已分析和完全无数据
            <div className="flex flex-col items-center justify-center py-12">
              {openingIntent && openingIntent.intentType ? (  // ✅ 更严格的条件判断
                // 开场白专用模式：有开场白意图但无问答对
                <>
                  <Target className="w-12 h-12 mb-3" style={{ color: 'var(--color-accent-blue)' }} />
                  <p className="text-base font-medium mb-2" style={{ color: 'var(--color-text-primary)' }}>
                    {t('panel.openingAnalyzed')}
                  </p>
                  <p className="text-sm text-center max-w-md px-4" style={{ color: 'var(--color-text-secondary)' }}>
                    {t('panel.noQAPairsInOpening')}
                  </p>
                </>
              ) : (
                // 完全无数据状态
                <p className="text-sm" style={{ color: 'var(--color-text-secondary)' }}>
                  {t('panel.noQAPairs')}
                </p>
              )}
            </div>
          ) : (
            // 三栏布局
            <div className="grid grid-cols-3 gap-4 overflow-hidden" style={{ height: '600px' }}>
              {/* 左侧：问答对列表 */}
              <div className="overflow-y-auto border-r pr-4" style={{ borderColor: 'var(--color-border-light)' }}>
                <QAPairList
                  qaPairs={qaPairs}
                  selectedIndex={selectedQaIndex ?? undefined}
                  onSelectQaPair={handleSelectQAPair}
                />
              </div>

              {/* 中间：开场白意图 */}
              <div className="overflow-y-auto border-r px-4" style={{ borderColor: 'var(--color-border-light)' }}>
                <IntentPanel intent={openingIntent} />
              </div>

              {/* 右侧：决策点列表 */}
              <div className="overflow-y-auto pl-4">
                <DecisionList
                  selectedQaPair={selectedQaPair}
                  language={language}
                />
              </div>
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}

export default AnalysisPanel;
