/**
 * MultiLevelViewSelector 组件
 *
 * 多级日志读取选择器（Full/Conversation/QAPairs/AssistantOnly/UserOnly）
 */

import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { cn } from '@/lib/utils';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { ViewLevel, VIEW_LEVEL_INFO } from '@/types/viewLevel';

export interface MultiLevelViewSelectorProps {
  /**
   * 当前选中的视图等级
   */
  value: ViewLevel;
  /**
   * 视图等级变更回调
   */
  onChange: (level: ViewLevel) => void;
  /**
   * 自定义类名
   */
  className?: string;
  /**
   * 是否显示导出按钮
   */
  showExport?: boolean;
  /**
   * 导出按钮点击回调
   */
  onExport?: (format: 'markdown' | 'json') => void;
  /**
   * 是否禁用
   */
  disabled?: boolean;
  /**
   * 加载状态
   */
  loading?: boolean;
}

/**
 * MultiLevelViewSelector 组件
 *
 * @example
 * ```tsx
 * <MultiLevelViewSelector
 *   value={viewLevel}
 *   onChange={setViewLevel}
 *   showExport
 *   onExport={(format) => handleExport(format)}
 * />
 * ```
 */
export function MultiLevelViewSelector({
  value,
  onChange,
  className,
  showExport = false,
  onExport,
  disabled = false,
  loading = false,
}: MultiLevelViewSelectorProps) {
  const { t } = useTranslation('sessions');

  // 获取所有视图等级选项
  const viewLevels = useMemo(() => {
    return Object.values(ViewLevel).map((level) => ({
      ...VIEW_LEVEL_INFO[level],
      value: level,
    }));
  }, []);

  return (
    <div className={cn('flex flex-col gap-3', className)}>
      {/* 标题和描述 */}
      <div className="flex items-center justify-between">
        <div>
          <div className="text-sm font-medium">{t('viewLevel.title')}</div>
          <div className="text-xs text-muted-foreground">
            {t('viewLevel.description')}
          </div>
        </div>

        {/* 导出按钮 */}
        {showExport && onExport && (
          <div className="flex gap-2">
            <button
              onClick={() => onExport('markdown')}
              disabled={disabled || loading}
              className={cn(
                'px-3 py-1.5 text-xs font-medium rounded-md border transition-colors',
                'hover:bg-accent hover:border-accent',
                'disabled:opacity-50 disabled:cursor-not-allowed'
              )}
              title={t('viewLevel.export.formats.markdown')}
            >
              Markdown
            </button>
            <button
              onClick={() => onExport('json')}
              disabled={disabled || loading}
              className={cn(
                'px-3 py-1.5 text-xs font-medium rounded-md border transition-colors',
                'hover:bg-accent hover:border-accent',
                'disabled:opacity-50 disabled:cursor-not-allowed'
              )}
              title={t('viewLevel.export.formats.json')}
            >
              JSON
            </button>
          </div>
        )}
      </div>

      {/* 视图等级选项列表 */}
      <div className="flex flex-col gap-2">
        {viewLevels.map((level) => {
          const isSelected = value === level.value;
          const levelKey = level.value.replace('_', ''); // full, conversation, qa_pairs, etc.

          return (
            <button
              key={level.value}
              onClick={() => !disabled && !loading && onChange(level.value)}
              disabled={disabled || loading}
              className={cn(
                'flex items-start gap-3 p-3 rounded-lg border transition-all text-left',
                'hover:bg-accent hover:border-accent',
                'disabled:opacity-50 disabled:cursor-not-allowed',
                isSelected && 'bg-accent border-accent ring-1 ring-ring'
              )}
            >
              {/* 图标 */}
              <div className="shrink-0 mt-0.5 text-lg">
                {level.icon}
              </div>

              {/* 标签和描述 */}
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1">
                  <span className="font-medium text-sm">
                    {t(`viewLevel.levels.${levelKey}.label`)}
                  </span>
                  {isSelected && (
                    <span className="text-xs px-1.5 py-0.5 rounded bg-primary text-primary-foreground">
                      {t('viewLevel.current')}
                    </span>
                  )}
                </div>
                <p className="text-xs text-muted-foreground">
                  {t(`viewLevel.levels.${levelKey}.description`)}
                </p>
              </div>
            </button>
          );
        })}
      </div>

      {/* 加载状态 */}
      {loading && (
        <div className="flex items-center justify-center py-2">
          <div className="text-xs text-muted-foreground">加载中...</div>
        </div>
      )}
    </div>
  );
}

/**
 * 视图等级快捷选择器（横向按钮组）
 */
export interface MultiLevelViewTabsProps {
  value: ViewLevel;
  onChange: (level: ViewLevel) => void;
  className?: string;
  disabled?: boolean;
}

export function MultiLevelViewTabs({
  value,
  onChange,
  className,
  disabled = false,
}: MultiLevelViewTabsProps) {
  const { t } = useTranslation('sessions');

  const viewLevels = useMemo(() => {
    return Object.values(ViewLevel).map((level) => ({
      ...VIEW_LEVEL_INFO[level],
      value: level,
    }));
  }, []);

  return (
    <div
      className={cn(
        'flex items-center gap-1 p-1 bg-muted rounded-lg',
        className
      )}
    >
      {viewLevels.map((level) => {
        const isSelected = value === level.value;
        const levelKey = level.value.replace('_', '');

        return (
          <button
            key={level.value}
            onClick={() => !disabled && onChange(level.value)}
            disabled={disabled}
            className={cn(
              'flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium transition-all',
              'hover:bg-background',
              'disabled:opacity-50 disabled:cursor-not-allowed',
              isSelected && 'bg-background shadow-sm'
            )}
            title={t(`viewLevel.levels.${levelKey}.description`)}
          >
            <span className="text-base">{level.icon}</span>
            <span className="hidden sm:inline">
              {t(`viewLevel.levels.${levelKey}.label`)}
            </span>
          </button>
        );
      })}
    </div>
  );
}

/**
 * 视图等级下拉选择器
 */
export interface MultiLevelViewDropdownProps {
  value: ViewLevel;
  onChange: (level: ViewLevel) => void;
  className?: string;
  disabled?: boolean;
}

export function MultiLevelViewDropdown({
  value,
  onChange,
  className,
  disabled = false,
}: MultiLevelViewDropdownProps) {
  const { t } = useTranslation('sessions');

  const viewLevels = useMemo(() => {
    return Object.values(ViewLevel).map((level) => ({
      ...VIEW_LEVEL_INFO[level],
      value: level,
    }));
  }, []);

  // 获取当前选中等级的配置
  const currentLevel = useMemo(
    () => viewLevels.find((level) => level.value === value),
    [value, viewLevels]
  );

  return (
    <div className={cn('flex items-center gap-3', className)}>
      <span className="text-sm font-medium whitespace-nowrap" style={{ color: 'var(--color-text-primary)' }}>
        {t('viewLevel.title')}:
      </span>
      <Select value={value} onValueChange={(val) => onChange(val as ViewLevel)} disabled={disabled}>
        <SelectTrigger className="w-[240px]" style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}>
          {currentLevel ? (
            <div className="flex items-center gap-2">
              <span className="text-base">{currentLevel.icon}</span>
              <span>{t(`viewLevel.levels.${currentLevel.value}.label`)}</span>
            </div>
          ) : (
            <SelectValue placeholder={t('viewLevel.title')} />
          )}
        </SelectTrigger>
        <SelectContent className="z-50" style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}>
          {viewLevels.map((level) => {
            return (
              <SelectItem key={level.value} value={level.value}>
                <div className="flex items-start gap-2">
                  <span className="text-base shrink-0 mt-0.5">
                    {level.icon}
                  </span>
                  <div className="flex-1 min-w-0">
                    <div className="font-medium text-sm" style={{ color: 'var(--color-text-primary)' }}>
                      {t(`viewLevel.levels.${level.value}.label`)}
                    </div>
                    <div className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
                      {t(`viewLevel.levels.${level.value}.description`)}
                    </div>
                  </div>
                </div>
              </SelectItem>
            );
          })}
        </SelectContent>
      </Select>
    </div>
  );
}
