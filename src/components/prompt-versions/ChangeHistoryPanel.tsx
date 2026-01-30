import { useTranslation } from 'react-i18next';
import type { PromptChange } from '@/types/generated';

interface ChangeHistoryPanelProps {
  changes: PromptChange[];
}

/**
 * 变更历史面板组件
 *
 * 显示版本之间的变更记录列表
 */
export function ChangeHistoryPanel({ changes }: ChangeHistoryPanelProps) {
  const { t } = useTranslation('promptVersions');

  // 格式化日期
  const formatDateTime = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  };

  // 获取变更类型图标
  const getChangeTypeIcon = (changeType: string) => {
    switch (changeType) {
      case 'Created':
        return '+';
      case 'Deleted':
        return '-';
      case 'Updated':
        return '~';
      default:
        return '•';
    }
  };

  // 获取变更类型样式
  const getChangeTypeStyle = (changeType: string) => {
    switch (changeType) {
      case 'Created':
        return {
          backgroundColor: 'rgba(76, 175, 80, 0.1)',
          color: 'var(--color-accent-green)',
        };
      case 'Deleted':
        return {
          backgroundColor: 'rgba(239, 68, 68, 0.1)',
          color: 'var(--color-accent-red)',
        };
      case 'Updated':
        return {
          backgroundColor: 'rgba(245, 158, 11, 0.1)',
          color: 'var(--color-accent-warm)',
        };
      default:
        return {
          backgroundColor: 'var(--color-bg-primary)',
          color: 'var(--color-text-secondary)',
        };
    }
  };

  // 获取变更类型标签
  const getChangeTypeLabel = (changeType: string) => {
    return t(`changeType.${changeType}` as any) || changeType;
  };

  return (
    <div className="rounded-lg p-5" style={{
      backgroundColor: 'var(--color-bg-card)',
      border: '1px solid var(--color-border-light)',
    }}>
      <h4 className="text-sm font-medium mb-4" style={{ color: 'var(--color-text-primary)' }}>
        {t('changeHistory')}
      </h4>
      <div className="space-y-3">
        {changes.length === 0 ? (
          <div className="text-center py-4 text-sm" style={{ color: 'var(--color-text-secondary)' }}>
            {t('noHistory')}
          </div>
        ) : (
          changes.map((change) => (
            <div
              key={change.id}
              className="flex items-start gap-3 p-3 rounded-lg border"
              style={{
                borderColor: 'var(--color-border-light)',
              }}
            >
              <div
                className="w-8 h-8 rounded-full flex items-center justify-center text-xs font-medium flex-shrink-0"
                style={getChangeTypeStyle(change.changeType)}
              >
                {getChangeTypeIcon(change.changeType)}
              </div>
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1">
                  <span className="text-sm font-medium" style={{ color: 'var(--color-text-primary)' }}>
                    {getChangeTypeLabel(change.changeType)}
                  </span>
                  <span className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
                    •
                  </span>
                  <span className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
                    {formatDateTime(change.changedAt)}
                  </span>
                </div>
                <div className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
                  <span className="font-medium" style={{ color: 'var(--color-text-primary)' }}>
                    {change.fieldName}
                  </span>
                  {change.oldValue && (
                    <>
                      {' '}
                      : {change.oldValue.length > 30
                        ? `${change.oldValue.substring(0, 30)}...`
                        : change.oldValue}
                    </>
                  )}
                  {change.newValue && (
                    <>
                      {' → '}
                      <span className="font-medium" style={{ color: 'var(--color-accent-green)' }}>
                        {change.newValue.length > 30
                          ? `${change.newValue.substring(0, 30)}...`
                          : change.newValue}
                      </span>
                    </>
                  )}
                </div>
                {change.changeSummary && (
                  <div className="text-xs mt-1 p-2 rounded" style={{
                    backgroundColor: 'var(--color-bg-primary)',
                    color: 'var(--color-text-secondary)',
                  }}>
                    {change.changeSummary}
                  </div>
                )}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
