import { useTranslation } from 'react-i18next';
import { Button } from '@/components/ui/button';
import type { PromptVersion, PromptComponent, PromptParameter } from '@/types/generated';

interface VersionDetailPanelProps {
  version: PromptVersion;
  components: PromptComponent[];
  parameters: PromptParameter[];
  onCompare?: () => void;
  onRollback?: () => void;
}

/**
 * 版本详情面板组件
 *
 * 显示选中版本的详细信息，包括内容预览、组件列表、参数列表
 */
export function VersionDetailPanel({
  version,
  components,
  parameters,
  onCompare,
  onRollback,
}: VersionDetailPanelProps) {
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

  // 组件类型翻译
  const getComponentTypeLabel = (type: string) => {
    return t(`componentType.${type}` as any) || type;
  };

  // 参数类型翻译
  const getParameterTypeLabel = (type: string) => {
    return t(`parameterType.${type}` as any) || type;
  };

  return (
    <div className="rounded-lg p-5 space-y-5" style={{
      backgroundColor: 'var(--color-bg-card)',
      border: '1px solid var(--color-border-light)',
    }}>
      {/* Version Header */}
      <div className="flex items-start justify-between">
        <div>
          <div className="flex items-center gap-2 mb-1">
            <h3 className="text-xl font-semibold" style={{ color: 'var(--color-text-primary)' }}>
              v{version.versionNumber}
            </h3>
            {version.isActive && (
              <span
                className="px-2 py-0.5 rounded-full text-xs font-medium"
                style={{
                  backgroundColor: 'var(--color-accent-green)',
                  color: 'white',
                }}
              >
                {t('active')}
              </span>
            )}
          </div>
          <p className="text-sm" style={{ color: 'var(--color-text-secondary)' }}>
            {formatDateTime(version.createdAt)}
          </p>
        </div>
        <div className="flex gap-2">
          {onCompare && (
            <Button
              variant="ghost"
              size="sm"
              onClick={onCompare}
              className="transition-all hover:scale-[1.02]"
              style={{
                border: '1px solid var(--color-border-light)',
                color: 'var(--color-text-primary)',
              }}
            >
              {t('compareWith')}
            </Button>
          )}
          {onRollback && !version.isActive && (
            <Button
              variant="destructive"
              size="sm"
              onClick={onRollback}
              style={{
                backgroundColor: 'var(--color-accent-red)',
                color: 'white',
              }}
            >
              {t('rollback')}
            </Button>
          )}
        </div>
      </div>

      {/* Metadata */}
      <div className="grid grid-cols-2 gap-4">
        <div className="p-3 rounded-lg" style={{ backgroundColor: 'var(--color-bg-primary)' }}>
          <div className="text-xs mb-1" style={{ color: 'var(--color-text-secondary)' }}>
            {t('createdBy')}
          </div>
          <div className="text-sm font-medium" style={{ color: 'var(--color-text-primary)' }}>
            {version.createdBy}
          </div>
        </div>
        <div className="p-3 rounded-lg" style={{ backgroundColor: 'var(--color-bg-primary)' }}>
          <div className="text-xs mb-1" style={{ color: 'var(--color-text-secondary)' }}>
            {t('components')}
          </div>
          <div className="text-sm font-medium" style={{ color: 'var(--color-text-primary)' }}>
            {components.length} {t('componentsUnit')}
          </div>
        </div>
      </div>

      {/* Content Preview */}
      <div>
        <h4 className="text-sm font-medium mb-2" style={{ color: 'var(--color-text-primary)' }}>
          {t('contentPreview')}
        </h4>
        <div
          className="rounded-lg p-4 max-h-64 overflow-y-auto font-mono text-xs leading-relaxed"
          style={{
            backgroundColor: 'var(--color-bg-primary)',
            border: '1px solid var(--color-border-light)',
            color: 'var(--color-text-primary)',
          }}
        >
          <pre className="whitespace-pre-wrap break-words">{version.content}</pre>
        </div>
      </div>

      {/* Components List */}
      <div>
        <h4 className="text-sm font-medium mb-2" style={{ color: 'var(--color-text-primary)' }}>
          {t('componentList')}
        </h4>
        <div className="space-y-2">
          {components.length === 0 ? (
            <div className="text-sm text-center py-4" style={{ color: 'var(--color-text-secondary)' }}>
              {t('noComponents')}
            </div>
          ) : (
            components.map((component) => (
              <div
                key={component.id}
                className="flex items-center justify-between p-2 rounded-lg border text-sm"
                style={{
                  borderColor: 'var(--color-border-light)',
                }}
              >
                <div className="flex items-center gap-2">
                  <span className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
                    {getComponentTypeLabel(component.componentType)}
                  </span>
                  <span className="font-medium" style={{ color: 'var(--color-text-primary)' }}>
                    {component.name}
                  </span>
                </div>
                <span className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
                  {component.language}
                </span>
              </div>
            ))
          )}
        </div>
      </div>

      {/* Parameters List */}
      {parameters.length > 0 && (
        <div>
          <h4 className="text-sm font-medium mb-2" style={{ color: 'var(--color-text-primary)' }}>
            {t('parameterChanges')}
          </h4>
          <div className="space-y-2">
            {parameters.map((parameter) => (
              <div
                key={parameter.id}
                className="flex items-center justify-between p-2 rounded-lg border text-sm"
                style={{
                  borderColor: 'var(--color-border-light)',
                }}
              >
                <div className="flex items-center gap-2">
                  <span className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
                    {getParameterTypeLabel(parameter.parameterType)}
                  </span>
                  <span className="font-medium" style={{ color: 'var(--color-text-primary)' }}>
                    {parameter.key}
                  </span>
                </div>
                <span className="text-xs font-mono" style={{ color: 'var(--color-text-secondary)' }}>
                  {parameter.value.length > 20
                    ? `${parameter.value.substring(0, 20)}...`
                    : parameter.value}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
