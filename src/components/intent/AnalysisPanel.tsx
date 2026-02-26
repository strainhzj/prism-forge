/**
 * AnalysisPanel 组件
 *
 * 会话意图分析结果弹窗（三栏布局）
 * - 左侧：问答对列表
 * - 中间：开场白意图
 * - 右侧：决策点列表
 */

import { useState, useEffect, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { Loader2 } from 'lucide-react';

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';

import type { DecisionQAPair } from '@/types/generated';
import type { OpeningIntent } from '@/types/generated/OpeningIntent';

import { QAPairList } from './QAPairList';
import { IntentPanel } from './IntentPanel';
import { DecisionList } from './DecisionList';

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
  const [error, setError] = useState<string | null>(null);
  const [qaPairs, setQaPairs] = useState<DecisionQAPair[]>([]);
  const [openingIntent, setOpeningIntent] = useState<OpeningIntent | null>(null);
  const [selectedQaPair, setSelectedQaPair] = useState<DecisionQAPair | null>(null);
  const [selectedQaIndex, setSelectedQaIndex] = useState<number | null>(null);

  /**
   * 执行分析流程
   */
  const performAnalysis = useCallback(async () => {
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
      // 直接显示真实错误消息，不再过度过滤
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  }, [sessionFilePath, language, t]);

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
    performAnalysis();
  }, [performAnalysis]);

  /**
   * 弹窗打开时自动执行分析
   */
  useEffect(() => {
    if (isOpen) {
      performAnalysis();
    } else {
      // 弹窗关闭时重置状态
      setQaPairs([]);
      setOpeningIntent(null);
      setSelectedQaPair(null);
      setSelectedQaIndex(null);
      setError(null);
    }
  }, [isOpen, performAnalysis]);

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <DialogContent
        className="max-w-6xl max-h-[80vh] overflow-hidden"
        style={{ backgroundColor: 'var(--color-bg-card)' }}
      >
        <DialogHeader>
          <DialogTitle style={{ color: 'var(--color-text-primary)' }}>
            {t('panel.title')}
          </DialogTitle>
        </DialogHeader>

        {/* 内容区域 */}
        <div className="mt-4">
          {loading ? (
            // 加载状态
            <div className="flex flex-col items-center justify-center py-12">
              <Loader2 className="w-8 h-8 animate-spin mb-4" style={{ color: 'var(--color-accent-blue)' }} />
              <p className="text-sm" style={{ color: 'var(--color-text-secondary)' }}>
                {t('panel.analyzing')}
              </p>
            </div>
          ) : error ? (
            // 错误状态
            <div className="flex flex-col items-center justify-center py-12">
              <p className="font-medium mb-2" style={{ color: 'var(--color-app-error-accent)' }}>
                {error}
              </p>
              <button
                onClick={handleRetry}
                className="px-4 py-2 text-sm rounded transition-colors hover:bg-[var(--color-accent-blue)] hover:text-white"
                style={{
                  color: 'var(--color-accent-blue)',
                  border: '1px solid var(--color-accent-blue)',
                }}
              >
                {t('actions.retry')}
              </button>
            </div>
          ) : qaPairs.length === 0 ? (
            // 无问答对状态
            <div className="flex flex-col items-center justify-center py-12">
              <p className="text-sm" style={{ color: 'var(--color-text-secondary)' }}>
                {t('panel.noQAPairs')}
              </p>
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
