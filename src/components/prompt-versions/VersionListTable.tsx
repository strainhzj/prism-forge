import { useTranslation } from 'react-i18next';
import type { PromptVersion } from '@/types/generated';
import './styles.css';

interface VersionListTableProps {
  versions: PromptVersion[];
  activeVersion?: PromptVersion | null;
  selectedVersion: number | null;
  onSelectVersion: (versionNumber: number) => void;
}

/**
 * 版本列表表格组件
 *
 * 显示所有版本，支持选择和查看激活状态
 */
export function VersionListTable({
  versions,
  activeVersion,
  selectedVersion,
  onSelectVersion,
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
        <h3 className="font-semibold" style={{ color: 'var(--color-text-primary)' }}>
          {t('versionList')}
        </h3>
        <span className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
          {versions.length} {t('versions')}
        </span>
      </div>

      {/* Table */}
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="table-header">
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
            {versions.map((version) => (
              <tr
                key={version.id}
                onClick={() => onSelectVersion(version.versionNumber)}
                className={`cursor-pointer table-row ${
                  selectedVersion === version.versionNumber ? 'table-row-active' : ''
                }`}
              >
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
            ))}
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
