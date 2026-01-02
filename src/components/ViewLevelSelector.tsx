/**
 * ViewLevelSelector 组件
 *
 * 日志等级视图选择器（L1/L2/L3）
 */

import { useMemo } from 'react';
import { Eye, Zap, Code } from 'lucide-react';
import { cn } from '@/lib/utils';

/**
 * 提取等级枚举
 */
export enum ExtractionLevel {
  /**
   * L1: 完整追踪
   */
  L1FullTrace = 'l1_full_trace',

  /**
   * L2: 清理流程
   */
  L2CleanFlow = 'l2_clean_flow',

  /**
   * L3: 仅提示词
   */
  L3PromptOnly = 'l3_prompt_only',
}

/**
 * 提取等级元数据
 */
export interface LevelMetadata {
  value: ExtractionLevel;
  label: string;
  description: string;
  icon: React.ReactNode;
  color: string;
}

/**
 * 提取等级配置
 */
const LEVEL_CONFIG: Record<ExtractionLevel, LevelMetadata> = {
  [ExtractionLevel.L1FullTrace]: {
    value: ExtractionLevel.L1FullTrace,
    label: '完整追踪',
    description: '显示所有消息、工具调用和思考过程',
    icon: <Eye className="h-4 w-4" />,
    color: 'text-blue-500',
  },
  [ExtractionLevel.L2CleanFlow]: {
    value: ExtractionLevel.L2CleanFlow,
    label: '清理流程',
    description: '只保留核心消息和工具结果',
    icon: <Zap className="h-4 w-4" />,
    color: 'text-green-500',
  },
  [ExtractionLevel.L3PromptOnly]: {
    value: ExtractionLevel.L3PromptOnly,
    label: '仅提示词',
    description: '只显示用户消息和助手回复',
    icon: <Code className="h-4 w-4" />,
    color: 'text-purple-500',
  },
};

export interface ViewLevelSelectorProps {
  /**
   * 当前选中的等级
   */
  value: ExtractionLevel;
  /**
   * 等级变更回调
   */
  onChange: (level: ExtractionLevel) => void;
  /**
   * 自定义类名
   */
  className?: string;
}

/**
 * ViewLevelSelector 组件
 *
 * @example
 * <ViewLevelSelector
 *   value={ExtractionLevel.L2CleanFlow}
 *   onChange={(level) => setLevel(level)}
 * />
 */
export function ViewLevelSelector({
  value,
  onChange,
  className,
}: ViewLevelSelectorProps) {
  const levels = useMemo(
    () => Object.values(ExtractionLevel).map((level) => LEVEL_CONFIG[level]),
    []
  );

  return (
    <div className={cn('flex flex-col gap-2', className)}>
      <div className="text-sm font-medium">日志等级</div>
      <div className="flex flex-col gap-2">
        {levels.map((level) => {
          const isSelected = value === level.value;

          return (
            <button
              key={level.value}
              onClick={() => onChange(level.value)}
              className={cn(
                'flex items-start gap-3 p-3 rounded-lg border transition-all text-left',
                'hover:bg-accent hover:border-accent',
                isSelected && 'bg-accent border-accent ring-1 ring-ring'
              )}
            >
              <div className={cn('shrink-0 mt-0.5', level.color)}>
                {level.icon}
              </div>
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1">
                  <span className="font-medium text-sm">{level.label}</span>
                  {isSelected && (
                    <span className={cn('text-xs px-1.5 py-0.5 rounded bg-primary text-primary-foreground')}>
                      当前
                    </span>
                  )}
                </div>
                <p className="text-xs text-muted-foreground">
                  {level.description}
                </p>
              </div>
            </button>
          );
        })}
      </div>
    </div>
  );
}

/**
 * 视图等级快捷选择器（横向按钮组）
 */
export interface ViewLevelTabsProps {
  value: ExtractionLevel;
  onChange: (level: ExtractionLevel) => void;
  className?: string;
}

export function ViewLevelTabs({
  value,
  onChange,
  className,
}: ViewLevelTabsProps) {
  const levels = useMemo(
    () => Object.values(ExtractionLevel).map((level) => LEVEL_CONFIG[level]),
    []
  );

  return (
    <div className={cn('flex items-center gap-1 p-1 bg-muted rounded-lg', className)}>
      {levels.map((level) => {
        const isSelected = value === level.value;

        return (
          <button
            key={level.value}
            onClick={() => onChange(level.value)}
            className={cn(
              'flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium transition-all',
              'hover:bg-background',
              isSelected && 'bg-background shadow-sm'
            )}
          >
            <span className={cn('h-4 w-4', level.color)}>{level.icon}</span>
            <span>{level.label}</span>
          </button>
        );
      })}
    </div>
  );
}
