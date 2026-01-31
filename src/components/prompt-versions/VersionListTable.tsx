import { useTranslation } from 'react-i18next';
import { Check } from 'lucide-react';
import type { PromptVersion } from '@/types/generated';
import './styles.css';

interface VersionListTableProps {
  versions: PromptVersion[];
  activeVersion?: PromptVersion | null;
  selectedVersion: number | null;
  selectedForCompare: Set<number>;
  onSelectVersion: (versionNumber: number) => void;
  onToggleCompareSelection: (versionNumber: number) => void;
}

/**
 * 版本列表表格组件
 *
 * 显示所有版本，支持选择和查看激活状态
 * 支持勾选版本用于对比（最多选择 2 个）
 */
export function VersionListTable({
  versions,
  activeVersion,
  selectedVersion,
  selectedForCompare,
  onSelectVersion,
  onToggleCompareSelection,
}: VersionListTableProps) {
  const { t } = useTranslation('promptVersions');

  // 格式化日期
  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleDateString('zh-CN');
  };

  return (
    <div className="rounded-lg border overflow-hidden" style={{
      backgroundColor: 'var(--color-bg-card)',
      borderColor: 'var(--color-border-light)',
    }}>
      <div className="flex items-center justify-between p-4 border-b" style={{ borderColor: 'var(--color-border-light)' }}>
        <div className="flex items-center gap-3">
          <h3 className="font-semibold" style={{ color: 'var(--color-text-primary)' }}>
            {t('versionList')}
          </h3>
          {selectedForCompare.size > 0 && (
            <span className="text-xs px-2 py-1 rounded-full" style={{
              backgroundColor: 'var(--color-accent-blue)',
              color: '#FFFFFF',
            }}>
              {t('selectedCount', { count: selectedForCompare.size })}
            </span>
          )}
        </div>
        <span className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
          {versions.length} {t('versions')}
        </span>
      </div>

      {/* Table */}
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="table-header">
              <th className="text-center py-2 px-2 font-medium" style={{ color: 'var(--color-text-secondary)', width: '40px' }}>
                {t('selectToCompare')}
              </th>
              <th className="text-left py-2 px-3 font-medium" style={{ color: 'var(--color-text-secondary)' }}>
                版本
              </th>
              <th className="text-left py-2 px-3 font-medium" style={{ color: 'var(--color-text-secondary)' }}>
                状态
              </th>
              <th className="text-left py-2 px-3 font-medium" style={{ color: 'var(--color-text-secondary)' }}>
                时间
              </th>
              <th className="text-right py-2 px-3 font-medium" style={{ color: 'var(--color-text-secondary)' }}>
                操作
              </th>
            </tr>
          </thead>
          <tbody>
            {versions.map((version) => {
              const isSelected = selectedForCompare.has(version.versionNumber);
              const canSelect = selectedForCompare.size < 2 || isSelected;

              return (
                <tr
                  key={version.id}
                  onClick={() => onSelectVersion(version.versionNumber)}
                  className={`cursor-pointer table-row ${
                    selectedVersion === version.versionNumber ? 'table-row-active' : ''
                  }`}
                >
                  <td className="py-2 px-2 text-center">
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        onToggleCompareSelection(version.versionNumber);
                      }}
                      disabled={!canSelect}
                      className={`w-5 h-5 rounded border-2 flex items-center justify-center transition-all ${
                        isSelected
                          ? 'bg-blue-500 border-blue-500'
                          : canSelect
                          ? 'border-gray-300 dark:border-gray-600 hover:border-blue-400'
                          : 'border-gray-200 dark:border-gray-700 opacity-40 cursor-not-allowed'
                      }`}
                      style={isSelected ? {
                        backgroundColor: 'var(--color-accent-blue)',
                        borderColor: 'var(--color-accent-blue)',
                      } : {}}
                      title={canSelect ? (isSelected ? '取消选择' : '选择对比') : '最多选择 2 个版本'}
                    >
                      {isSelected && <Check className="w-3 h-3 text-white" />}
                    </button>
                  </td>
                  <td className="py-2 px-3">
                    <span className="font-medium" style={{ color: 'var(--color-text-primary)' }}>
                      v{version.versionNumber}
                    </span>
                  </td>
                  <td className="py-2 px-3">
                    {version.isActive ? (
                      <span className="active-badge px-2 py-0.5 rounded-full text-xs font-medium">
                        {t('active')}
                      </span>
                    ) : (
                      <span className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
                        -
                      </span>
                    )}
                  </td>
                  <td className="py-2 px-3" style={{ color: 'var(--color-text-secondary)', fontSize: '12px' }}>
                    {formatDate(version.createdAt)}
                  </td>
                  <td className="py-2 px-3 text-right">
                    <button
                      className="text-xs transition-colors hover:opacity-80 btn-ghost-enhanced"
                      style={{ color: 'var(--color-text-secondary)', padding: '0.25rem 0.5rem', borderRadius: '0.375rem' }}
                      onClick={(e) => {
                        e.stopPropagation();
                        onSelectVersion(version.versionNumber);
                      }}
                    >
                      {t('view')}
                    </button>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      {/* Summary */}
      <div className="p-4 border-t" style={{ borderColor: 'var(--color-border-light)' }}>
        <div className="flex items-center justify-between text-sm">
          <span style={{ color: 'var(--color-text-secondary)' }}>
            {t('activeVersion')}
          </span>
          <span className="font-medium" style={{ color: 'var(--color-text-primary)' }}>
            {activeVersion ? `v${activeVersion.versionNumber}` : '-'}
          </span>
        </div>
      </div>
    </div>
  );
}
