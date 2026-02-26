/**
 * QAPairList 组件
 *
 * 问答对列表（左侧栏）
 * 显示所有检测到的问答对，支持选择查看决策
 */

import { memo } from 'react';
import { useTranslation } from 'react-i18next';
import { MessageSquare } from 'lucide-react';
import { cn } from '@/lib/utils';

import type { DecisionQAPair } from '@/types/generated';

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[QAPairList] ${action}`, ...args);
  }
}

// ==================== 类型定义 ====================

export interface QAPairListProps {
  /**
   * 问答对列表
   */
  qaPairs: DecisionQAPair[];
  /**
   * 当前选中的索引
   */
  selectedIndex?: number;
  /**
   * 选择问答对回调
   */
  onSelectQaPair: (qaPair: DecisionQAPair, index: number) => void;
}

/**
 * QAPairList - 问答对列表组件
 */
export const QAPairList = memo(function QAPairList({
  qaPairs,
  selectedIndex,
  onSelectQaPair,
}: QAPairListProps) {
  const { t } = useTranslation('intentAnalysis');

  debugLog('render', { qaPairsCount: qaPairs.length, selectedIndex });

  if (qaPairs.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center p-4">
        <MessageSquare className="w-8 h-8 mb-2" style={{ color: 'var(--color-text-secondary)' }} />
        <p className="text-sm" style={{ color: 'var(--color-text-secondary)' }}>
          {t('panel.noQAPairs')}
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      <h3 className="text-sm font-semibold mb-3" style={{ color: 'var(--color-text-primary)' }}>
        {t('qaPairs.title')} ({qaPairs.length})
      </h3>

      {qaPairs.map((qaPair, index) => (
        <button
          key={qaPair.qaIndex}
          onClick={() => onSelectQaPair(qaPair, index)}
          className={cn(
            'w-full text-left p-3 rounded-lg border transition-all hover:shadow-md',
            selectedIndex === index
              ? 'border-[var(--color-accent-blue)] bg-[var(--color-accent-blue)/10]'
              : 'border-[var(--color-border-light)] hover:border-[var(--color-accent-blue)/50]'
          )}
          style={{
            backgroundColor: selectedIndex === index ? 'rgba(74, 158, 255, 0.1)' : 'var(--color-bg-card)',
          }}
        >
          {/* 序号 */}
          <div className="flex items-center gap-2 mb-2">
            <span
              className={cn(
                'w-5 h-5 rounded-full flex items-center justify-center text-xs font-semibold',
                selectedIndex === index
                  ? 'bg-[var(--color-accent-blue)] text-white'
                  : 'bg-[var(--color-bg-primary)] text-[var(--color-text-secondary)]'
              )}
            >
              {qaPair.qaIndex + 1}
            </span>
            <span className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
              Q&A #{qaPair.qaIndex + 1}
            </span>
          </div>

          {/* 助手回答预览 */}
          <div className="mb-2">
            <p className="text-xs font-medium mb-1" style={{ color: 'var(--color-text-secondary)' }}>
              {t('qaPairs.assistantAnswer')}
            </p>
            <p className="text-xs line-clamp-2" style={{ color: 'var(--color-text-primary)' }}>
              {qaPair.assistantAnswer.slice(0, 100)}
              {qaPair.assistantAnswer.length > 100 ? '...' : ''}
            </p>
          </div>

          {/* 用户决策预览 */}
          <div>
            <p className="text-xs font-medium mb-1" style={{ color: 'var(--color-text-secondary)' }}>
              {t('qaPairs.userDecision')}
            </p>
            <p className="text-xs line-clamp-2" style={{ color: 'var(--color-text-primary)' }}>
              {qaPair.userDecision.slice(0, 100)}
              {qaPair.userDecision.length > 100 ? '...' : ''}
            </p>
          </div>
        </button>
      ))}
    </div>
  );
});

export default QAPairList;
