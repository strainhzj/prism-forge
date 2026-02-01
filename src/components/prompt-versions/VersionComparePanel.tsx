import { useTranslation } from 'react-i18next';
import type { PromptVersion, PromptVersionDiff, LineDiff } from '@/types/generated';
import './styles.css';

interface VersionComparePanelProps {
  versions: PromptVersion[];
  comparison: PromptVersionDiff;
  compareFrom: number | null;
  compareTo: number | null;
  onCompareChange: (from: number | null, to: number | null) => void;
}

/**
 * 版本对比面板组件
 *
 * 显示两个版本之间的差异，包括组件变更和参数变更
 * 支持行级差异高亮显示
 */
export function VersionComparePanel({
  versions,
  comparison,
  compareFrom,
  compareTo,
  onCompareChange,
}: VersionComparePanelProps) {
  const { t } = useTranslation('promptVersions');

  // 计算变更统计
  const addedCount = comparison.componentChanges.filter(
    (c) => c.changeType === 'created' || (c.lineDiffs?.some((l) => l.changeType === 'added') ?? false)
  ).length;
  const removedCount = comparison.componentChanges.filter(
    (c) => c.changeType === 'deleted' || (c.lineDiffs?.some((l) => l.changeType === 'removed') ?? false)
  ).length;
  const modifiedCount = comparison.componentChanges.filter(
    (c) => c.changeType === 'updated'
  ).length;

  // 获取变更类型标签
  const getChangeTypeLabel = (changeType: string) => {
    return t(`changeType.${changeType}` as any) || changeType;
  };

  // 渲染行差异
  const renderLineDiffs = (lineDiffs: LineDiff[]) => {
    if (lineDiffs.length === 0) return null;

    return (
      <div className="code-block text-xs">
        {lineDiffs.map((diff, idx) => (
          <div key={idx} className={`diff-${diff.changeType} px-3 py-1`}>
            <span className="mr-2" style={{ color: 'var(--color-text-secondary)' }}>
              {diff.lineNumber || '-'}
            </span>
            <span className={diff.changeType === 'removed' ? 'line-through' : ''} style={{ color: 'var(--color-text-primary)' }}>
              {diff.changeType === 'added'
                ? diff.newContent || ''
                : diff.changeType === 'removed'
                ? diff.oldContent || ''
                : diff.oldContent || diff.newContent || ''}
            </span>
          </div>
        ))}
      </div>
    );
  };

  // 渲染组件变更
  const renderComponentChanges = () => {
    if (comparison.componentChanges.length === 0) {
      return (
        <div className="text-center py-4 text-sm" style={{ color: 'var(--color-text-secondary)' }}>
          {t('noChanges')}
        </div>
      );
    }

    return comparison.componentChanges.map((change, idx) => (
      <div key={idx} className="component-change-card">
        <div className="component-change-header">
          <div className="flex items-center gap-2">
            <span className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
              {change.componentType}
            </span>
            <span className="text-sm font-medium" style={{ color: 'var(--color-text-primary)' }}>
              {change.componentName}
            </span>
          </div>
          <span className={`text-xs component-change-badge ${change.changeType}`}>
            {getChangeTypeLabel(change.changeType)}
          </span>
        </div>
        {change.lineDiffs.length > 0 && renderLineDiffs(change.lineDiffs)}
      </div>
    ));
  };

  // 渲染参数变更
  const renderParameterChanges = () => {
    if (comparison.parameterChanges.length === 0) {
      return (
        <div className="text-center py-4 text-sm" style={{ color: 'var(--color-text-secondary)' }}>
          {t('noParameters')}
        </div>
      );
    }

    return comparison.parameterChanges.map((param, idx) => (
      <div
        key={idx}
        className="flex items-center justify-between p-2 rounded-lg border text-sm"
        style={{ borderColor: 'var(--color-border-light)' }}
      >
        <div>
          <span className="text-xs mr-2" style={{ color: 'var(--color-text-secondary)' }}>
            {param.parameterType}
          </span>
          <span className="font-medium" style={{ color: 'var(--color-text-primary)' }}>
            {param.key}
          </span>
        </div>
        <div className="flex items-center gap-2">
          {param.oldValue && (
            <>
              <span
                className="text-xs line-through font-mono"
                style={{ color: 'var(--color-text-secondary)' }}
              >
                {param.oldValue.length > 15
                  ? `${param.oldValue.substring(0, 15)}...`
                  : param.oldValue}
              </span>
              <svg
                className="w-4 h-4"
                style={{ color: 'var(--color-accent-green)' }}
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M13 7l5 5m0 0l-5 5m5-5H6"
                />
              </svg>
            </>
          )}
          <span
            className="text-xs font-medium font-mono"
            style={{ color: 'var(--color-accent-green)' }}
          >
            {param.newValue
              ? (param.newValue.length > 15
                  ? `${param.newValue.substring(0, 15)}...`
                  : param.newValue)
              : '-'}
          </span>
        </div>
      </div>
    ));
  };

  return (
    <div className="rounded-lg p-5 space-y-5" style={{
      backgroundColor: 'var(--color-bg-card)',
      border: '1px solid var(--color-border-light)',
    }}>
      {/* Version Selector */}
      <div className="flex items-center gap-4">
        <div className="flex-1">
          <label className="text-xs mb-1 block font-medium" style={{ color: 'var(--color-accent-blue)' }}>
            {t('fromVersion')}
          </label>
          <select
            value={compareFrom ?? ''}
            onChange={(e) =>
              onCompareChange(
                e.target.value ? Number(e.target.value) : null,
                compareTo
              )
            }
            className="version-selector"
          >
            <option value="">选择版本</option>
            {versions.map((v) => (
              <option key={v.id} value={v.versionNumber}>
                v{v.versionNumber} {v.isActive ? '(激活)' : ''}
              </option>
            ))}
          </select>
        </div>
        <div className="flex items-center justify-center pt-5">
          <svg className="w-5 h-5" style={{ color: 'var(--color-accent-blue)' }} fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M13 7l5 5m0 0l-5 5m5-5H6" />
          </svg>
        </div>
        <div className="flex-1">
          <label className="text-xs mb-1 block font-medium" style={{ color: 'var(--color-accent-green)' }}>
            {t('toVersion')}
          </label>
          <select
            value={compareTo ?? ''}
            onChange={(e) =>
              onCompareChange(
                compareFrom,
                e.target.value ? Number(e.target.value) : null
              )
            }
            className="version-selector"
          >
            <option value="">选择版本</option>
            {versions.map((v) => (
              <option key={v.id} value={v.versionNumber}>
                v{v.versionNumber} {v.isActive ? '(激活)' : ''}
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* Comparison Summary */}
      <div className="flex gap-3">
        <div className="flex-1 p-3 rounded-lg stat-card-added">
          <div className="text-xs font-medium mb-1" style={{ color: 'var(--color-text-primary)' }}>
            {t('added')}
          </div>
          <div className="text-lg font-semibold" style={{ color: 'var(--color-text-primary)' }}>
            {addedCount}
          </div>
        </div>
        <div className="flex-1 p-3 rounded-lg stat-card-removed">
          <div className="text-xs font-medium mb-1" style={{ color: 'var(--color-text-primary)' }}>
            {t('removed')}
          </div>
          <div className="text-lg font-semibold" style={{ color: 'var(--color-text-primary)' }}>
            {removedCount}
          </div>
        </div>
        <div className="flex-1 p-3 rounded-lg stat-card-modified">
          <div className="text-xs font-medium mb-1" style={{ color: 'var(--color-text-primary)' }}>
            {t('modified')}
          </div>
          <div className="text-lg font-semibold" style={{ color: 'var(--color-text-primary)' }}>
            {modifiedCount}
          </div>
        </div>
      </div>

      {/* Component Changes */}
      <div>
        <h4 className="text-sm font-medium mb-3" style={{ color: 'var(--color-text-primary)' }}>
          {t('componentChanges')}
        </h4>
        <div className="space-y-3">{renderComponentChanges()}</div>
      </div>

      {/* Parameter Changes */}
      <div>
        <h4 className="text-sm font-medium mb-3" style={{ color: 'var(--color-text-primary)' }}>
          {t('parameterChanges')}
        </h4>
        <div className="space-y-2">{renderParameterChanges()}</div>
      </div>
    </div>
  );
}
