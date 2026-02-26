/**
 * IntentPanel 组件
 *
 * 开场白意图面板（中间栏）
 * 显示会话开场白意图分析结果
 */

import { memo } from 'react';
import { useTranslation } from 'react-i18next';
import { Target, Briefcase, Lightbulb, Wrench, CheckCircle } from 'lucide-react';
import { cn } from '@/lib/utils';

import type { OpeningIntent } from '@/types/generated/OpeningIntent';

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[IntentPanel] ${action}`, ...args);
  }
}

// ==================== 辅助函数 ====================

/**
 * 安全获取意图类型显示文本
 */
function getIntentTypeLabel(intentType: string | undefined | null, t: (key: string) => string): string {
  if (!intentType) return t('intent.types.other');

  try {
    const key = `intent.types.${String(intentType).toLowerCase()}`;
    const translated = t(key);
    // 如果翻译键不存在，返回原始值
    return translated.includes('intent.types.') ? String(intentType) : translated;
  } catch {
    return String(intentType);
  }
}

// ==================== 类型定义 ====================

export interface IntentPanelProps {
  /**
   * 开场白意图分析结果
   */
  intent: OpeningIntent | null;
}

/**
 * IntentPanel - 开场白意图面板组件
 */
export const IntentPanel = memo(function IntentPanel({ intent }: IntentPanelProps) {
  const { t } = useTranslation('intentAnalysis');

  debugLog('render', { intent });

  if (!intent) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center p-4">
        <Target className="w-8 h-8 mb-2" style={{ color: 'var(--color-text-secondary)' }} />
        <p className="text-sm" style={{ color: 'var(--color-text-secondary)' }}>
          {t('qaPairs.selectToView')}
        </p>
      </div>
    );
  }

  // 置信度颜色
  const getConfidenceColor = (confidence: number | undefined | null) => {
    if (confidence === undefined || confidence === null) return 'text-[var(--color-text-secondary)]';
    if (confidence >= 0.8) return 'text-[var(--color-accent-green)]';
    if (confidence >= 0.5) return 'text-[var(--color-accent-warm)]';
    return 'text-[var(--color-app-error-accent)]';
  };

  // 安全获取置信度值
  const confidence = intent.confidence ?? 0;
  const confidencePercent = Math.round(confidence * 100);

  return (
    <div className="space-y-4">
      <h3 className="text-sm font-semibold flex items-center gap-2" style={{ color: 'var(--color-text-primary)' }}>
        <Target className="w-4 h-4" style={{ color: 'var(--color-accent-blue)' }} />
        {t('intent.title')}
      </h3>

      {/* 核心目标 */}
      <div className="p-3 rounded-lg border" style={{ backgroundColor: 'var(--color-bg-primary)', borderColor: 'var(--color-border-light)' }}>
        <p className="text-xs font-medium mb-2 flex items-center gap-1" style={{ color: 'var(--color-text-secondary)' }}>
          <Lightbulb className="w-3 h-3" />
          {t('intent.openingGoal')}
        </p>
        <p className="text-sm" style={{ color: 'var(--color-text-primary)' }}>
          {intent.description || t('intent.types.other')}
        </p>
      </div>

      {/* 意图类型 */}
      <div className="p-3 rounded-lg border" style={{ backgroundColor: 'var(--color-bg-primary)', borderColor: 'var(--color-border-light)' }}>
        <p className="text-xs font-medium mb-2 flex items-center gap-1" style={{ color: 'var(--color-text-secondary)' }}>
          <Briefcase className="w-3 h-3" />
          {t('intent.intentType')}
        </p>
        <p className="text-sm" style={{ color: 'var(--color-text-primary)' }}>
          {getIntentTypeLabel(intent.intentType, t)}
        </p>
      </div>

      {/* 关键信息 */}
      {intent.keyInfo && intent.keyInfo.length > 0 && (
        <div className="p-3 rounded-lg border" style={{ backgroundColor: 'var(--color-bg-primary)', borderColor: 'var(--color-border-light)' }}>
          <p className="text-xs font-medium mb-2 flex items-center gap-1" style={{ color: 'var(--color-text-secondary)' }}>
            <Wrench className="w-3 h-3" />
            {t('intent.keyInfo')}
          </p>
          <ul className="space-y-1">
            {intent.keyInfo.map((info, index) => (
              <li key={index} className="text-xs flex items-start gap-2" style={{ color: 'var(--color-text-primary)' }}>
                <CheckCircle className="w-3 h-3 mt-0.5 flex-shrink-0" style={{ color: 'var(--color-accent-green)' }} />
                <span>{info || ''}</span>
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* 置信度 */}
      <div className="p-3 rounded-lg border" style={{ backgroundColor: 'var(--color-bg-primary)', borderColor: 'var(--color-border-light)' }}>
        <p className="text-xs font-medium mb-1" style={{ color: 'var(--color-text-secondary)' }}>
          {t('intent.confidence')}
        </p>
        <div className="flex items-center gap-2">
          <div className="flex-1 h-2 rounded-full bg-[var(--color-bg-primary)] overflow-hidden">
            <div
              className="h-full rounded-full bg-[var(--color-accent-blue)]"
              style={{ width: `${confidencePercent}%` }}
            />
          </div>
          <span className={cn('text-sm font-semibold', getConfidenceColor(intent.confidence))}>
            {confidencePercent}%
          </span>
        </div>
      </div>
    </div>
  );
});

export default IntentPanel;
