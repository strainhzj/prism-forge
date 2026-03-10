/**
 * DecisionList 组件
 *
 * 决策点列表（右侧栏）
 * 显示选中问答对的决策分析结果
 */

import { useState, useEffect, useCallback, memo } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { GitBranch, Scale, Lightbulb, AlertCircle, Loader2 } from 'lucide-react';
import { cn } from '@/lib/utils';

import type { DecisionQAPair } from '@/types/generated';
import type { DecisionAnalysis, DecisionType } from '@/types/generated';

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[DecisionList] ${action}`, ...args);
  }
}

// ==================== 类型定义 ====================

export interface DecisionListProps {
  /**
   * 当前选中的问答对
   */
  selectedQaPair: DecisionQAPair | null;
  /**
   * 语言标识（"zh" 或 "en"）
   */
  language?: string;
}

/**
 * DecisionList - 决策点列表组件
 */
export const DecisionList = memo(function DecisionList({
  selectedQaPair,
  language = 'zh',
}: DecisionListProps) {
  const { t } = useTranslation('intentAnalysis');

  // 状态管理
  const [loading, setLoading] = useState(false);
  const [decision, setDecision] = useState<DecisionAnalysis | null>(null);
  const [error, setError] = useState<string | null>(null);

  /**
   * 分析问答对决策
   */
  const analyzeDecision = useCallback(async () => {
    if (!selectedQaPair) {
      setDecision(null);
      setError(null);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      debugLog('分析问答对决策', {
        qaIndex: selectedQaPair.qaIndex,
        assistantAnswerUuid: selectedQaPair.assistantAnswerUuid,
        userDecisionUuid: selectedQaPair.userDecisionUuid,
      });

      if (DEBUG) {
        console.log('[DecisionList] selectedQaPair:', selectedQaPair);
      }

      // ✅ Rust 端已添加 #[serde(rename_all = "camelCase")]，直接传递 camelCase 对象
      const result = await invoke<DecisionAnalysis>('cmd_analyze_decision', {
        qaPair: selectedQaPair,
        language,
      });

      setDecision(result);
      debugLog('决策分析完成', { result });
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

      debugLog('决策分析失败', { error: err, errorMessage });
      // 直接显示真实错误消息，不再过度过滤
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  }, [selectedQaPair, language, t]);

  /**
   * 问答对变化时重新分析
   */
  useEffect(() => {
    analyzeDecision();
  }, [analyzeDecision]);

  /**
   * 获取决策类型显示文本
   */
  const getDecisionTypeLabel = (type: DecisionType): string => {
    const typeStr = String(type).toLowerCase();
    const key = `decisionTypes.${typeStr}` as const;
    return t(key);
  };

  /**
   * 获取置信度颜色
   */
  const getConfidenceColor = (confidence: number) => {
    if (confidence >= 0.8) return 'text-[var(--color-accent-green)]';
    if (confidence >= 0.5) return 'text-[var(--color-accent-warm)]';
    return 'text-[var(--color-app-error-accent)]';
  };

  debugLog('render', { selectedQaPair, loading, hasDecision: !!decision, error });

  // 未选择问答对状态
  if (!selectedQaPair) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center p-4">
        <GitBranch className="w-8 h-8 mb-2" style={{ color: 'var(--color-text-secondary)' }} />
        <p className="text-sm" style={{ color: 'var(--color-text-secondary)' }}>
          {t('decisions.noDecisionSelected')}
        </p>
      </div>
    );
  }

  // 加载状态
  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center h-full">
        <Loader2 className="w-6 h-6 animate-spin mb-2" style={{ color: 'var(--color-accent-blue)' }} />
        <p className="text-sm" style={{ color: 'var(--color-text-secondary)' }}>
          {t('panel.loading')}
        </p>
      </div>
    );
  }

  // 错误状态
  if (error) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center p-4">
        <AlertCircle className="w-6 h-6 mb-2" style={{ color: 'var(--color-app-error-accent)' }} />
        <p className="text-sm" style={{ color: 'var(--color-app-error-accent)' }}>
          {error}
        </p>
      </div>
    );
  }

  // 无决策检测到
  if (!decision) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center p-4">
        <AlertCircle className="w-6 h-6 mb-2" style={{ color: 'var(--color-text-secondary)' }} />
        <p className="text-sm" style={{ color: 'var(--color-text-secondary)' }}>
          {t('decisions.noDecisionsDetected')}
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <h3 className="text-sm font-semibold flex items-center gap-2" style={{ color: 'var(--color-text-primary)' }}>
        <GitBranch className="w-4 h-4" style={{ color: 'var(--color-accent-warm)' }} />
        {t('decisions.title')} #{selectedQaPair.qaIndex + 1}
      </h3>

      {/* 决策内容 */}
      <div className="p-3 rounded-lg border" style={{ backgroundColor: 'var(--color-bg-primary)', borderColor: 'var(--color-border-light)' }}>
        <p className="text-xs font-medium mb-2" style={{ color: 'var(--color-text-secondary)' }}>
          {t('decisions.made')}
        </p>
        <p className="text-sm" style={{ color: 'var(--color-text-primary)' }}>
          {decision.decisionMade}
        </p>
      </div>

      {/* 决策类型 */}
      <div className="p-3 rounded-lg border" style={{ backgroundColor: 'var(--color-bg-primary)', borderColor: 'var(--color-border-light)' }}>
        <p className="text-xs font-medium mb-1" style={{ color: 'var(--color-text-secondary)' }}>
          {t('decisions.type')}
        </p>
        <p className="text-sm font-medium" style={{ color: 'var(--color-accent-blue)' }}>
          {getDecisionTypeLabel(decision.decisionType)}
        </p>
      </div>

      {/* 明确理由 */}
      {decision.rationale && decision.rationale.length > 0 && (
        <div className="p-3 rounded-lg border" style={{ backgroundColor: 'var(--color-bg-primary)', borderColor: 'var(--color-border-light)' }}>
          <p className="text-xs font-medium mb-2 flex items-center gap-1" style={{ color: 'var(--color-text-secondary)' }}>
            <Scale className="w-3 h-3" />
            {t('decisions.rationale')}
          </p>
          <ul className="space-y-1">
            {decision.rationale.map((reason, index) => (
              <li key={index} className="text-xs flex items-start gap-2" style={{ color: 'var(--color-text-primary)' }}>
                <span className="font-semibold">•</span>
                <span>{reason}</span>
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* 推测理由 */}
      {decision.inferredReasons && decision.inferredReasons.length > 0 && (
        <div className="p-3 rounded-lg border" style={{ backgroundColor: 'var(--color-bg-primary)', borderColor: 'var(--color-border-light)' }}>
          <p className="text-xs font-medium mb-2 flex items-center gap-1" style={{ color: 'var(--color-text-secondary)' }}>
            <Lightbulb className="w-3 h-3" />
            {t('decisions.inferredReasons')}
          </p>
          <ul className="space-y-1">
            {decision.inferredReasons.map((reason, index) => (
              <li key={index} className="text-xs flex items-start gap-2" style={{ color: 'var(--color-text-primary)' }}>
                <span className="text-[var(--color-text-secondary)]">?</span>
                <span>{reason}</span>
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* 备选方案 */}
      {decision.alternatives && decision.alternatives.length > 0 && (
        <div className="p-3 rounded-lg border" style={{ backgroundColor: 'var(--color-bg-primary)', borderColor: 'var(--color-border-light)' }}>
          <p className="text-xs font-medium mb-2" style={{ color: 'var(--color-text-secondary)' }}>
            {t('decisions.alternatives')}
          </p>
          <div className="flex flex-wrap gap-1">
            {decision.alternatives.map((alt, index) => (
              <span
                key={index}
                className="px-2 py-1 text-xs rounded border"
                style={{
                  backgroundColor: 'var(--color-bg-card)',
                  borderColor: 'var(--color-border-light)',
                  color: 'var(--color-text-secondary)',
                }}
              >
                {alt.name}
              </span>
            ))}
          </div>
        </div>
      )}

      {/* 置信度 */}
      <div className="p-3 rounded-lg border" style={{ backgroundColor: 'var(--color-bg-primary)', borderColor: 'var(--color-border-light)' }}>
        <p className="text-xs font-medium mb-1" style={{ color: 'var(--color-text-secondary)' }}>
          {t('decisions.confidence')}
        </p>
        <div className="flex items-center gap-2">
          <div className="flex-1 h-2 rounded-full bg-[var(--color-bg-primary)] overflow-hidden">
            <div
              className={cn('h-full rounded-full', getConfidenceColor(decision.confidence).replace('text-', 'bg-'))}
              style={{ width: `${decision.confidence * 100}%` }}
            />
          </div>
          <span className={cn('text-sm font-semibold', getConfidenceColor(decision.confidence))}>
            {(decision.confidence * 100).toFixed(0)}%
          </span>
        </div>
      </div>
    </div>
  );
});

export default DecisionList;
