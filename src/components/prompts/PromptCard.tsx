import { useTranslation } from 'react-i18next';
import type { Prompt } from '@/types/generated';
import { Button } from '@/components/ui/button';
import { memo } from 'react';
import { History } from 'lucide-react';

interface PromptCardProps {
  prompt: Prompt;
  onEdit: () => void;
  onDelete: () => void;
  onReset: () => void;
  onViewVersions?: () => void;
}

/**
 * 提示词卡片组件
 *
 * 显示单个提示词的摘要信息
 */
const PromptCard = memo(function PromptCard({
  prompt,
  onEdit,
  onDelete,
  onReset,
  onViewVersions,
}: PromptCardProps) {
  const { t } = useTranslation('prompts');

  // 格式化时间
  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleString();
  };

  return (
    <div
      className="rounded-lg shadow-md p-6 border"
      style={{
        backgroundColor: 'var(--color-bg-card)',
        borderColor: 'var(--color-border-light)',
      }}
    >
      {/* 标题行 */}
      <div className="flex items-start justify-between mb-4">
        <div className="flex-1">
          <h3
            className="text-xl font-semibold flex items-center gap-2"
            style={{ color: 'var(--color-text-primary)' }}
          >
            <span
              className="text-lg"
              aria-label={prompt.isSystem ? t('system') : t('default')}
              role="img"
            >
              {prompt.isSystem ? '📝' : '✏️'}
            </span>
            {prompt.name}
            <span
              className="text-sm"
              style={{ color: 'var(--color-text-secondary)' }}
            >
              ({prompt.language === 'zh' ? '中文' : 'English'})
            </span>
          </h3>
          {prompt.description && (
            <p
              className="mt-2"
              style={{ color: 'var(--color-text-secondary)' }}
            >
              {prompt.description.length > 100
                ? `${prompt.description.substring(0, 100)}...`
                : prompt.description}
            </p>
          )}
        </div>

        {/* 标记 */}
        <div className="flex gap-2">
          {prompt.isDefault && (
            <span
              className="px-2 py-1 text-xs font-medium rounded"
              style={{
                backgroundColor: 'var(--color-accent-warm)',
                color: '#FFFFFF',
                opacity: 0.9,
              }}
            >
              {t('default')}
            </span>
          )}
          {prompt.isSystem && (
            <span
              className="px-2 py-1 text-xs font-medium rounded"
              style={{
                backgroundColor: '#3b82f6',
                color: '#FFFFFF',
                opacity: 0.9,
              }}
            >
              {t('system')}
            </span>
          )}
        </div>
      </div>

      {/* 元信息 */}
      <div
        className="text-sm mb-4"
        style={{ color: 'var(--color-text-secondary)' }}
      >
        <div>
          {t('scenario')}: {t(`scenarios.${prompt.scenario}`)}
        </div>
        <div>
          {t('version')}: {prompt.version}
        </div>
        <div>
          {t('lastUpdated')}: {formatDate(prompt.updatedAt)}
        </div>
      </div>

      {/* 操作按钮 */}
      <div className="flex gap-2 flex-wrap">
        <Button
          onClick={onEdit}
          variant="secondary"
          size="sm"
          className="transition-all hover:scale-[1.02]"
          style={{
            boxShadow: '0 0 0 var(--color-accent-warm-shadow)',
            border: '1px solid var(--color-app-primary-blue)',
            backgroundColor: 'var(--color-bg-card)',
            color: 'var(--color-app-primary-blue)',
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.boxShadow = '0 0 10px var(--color-accent-warm-shadow)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.boxShadow = '0 0 0 var(--color-accent-warm-shadow)';
          }}
        >
          {t('edit')}
        </Button>

        {/* 为 optimizer_prompt_template 添加版本历史按钮 */}
        {onViewVersions && (
          <Button
            onClick={onViewVersions}
            variant="secondary"
            size="sm"
            className="transition-all hover:scale-[1.02] flex items-center gap-1"
            style={{
              boxShadow: '0 0 0 var(--color-accent-warm-shadow)',
              border: '1px solid var(--color-accent-blue)',
              backgroundColor: 'var(--color-bg-card)',
              color: 'var(--color-accent-blue)',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.boxShadow = '0 0 10px var(--color-accent-warm-shadow)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.boxShadow = '0 0 0 var(--color-accent-warm-shadow)';
            }}
          >
            <History className="h-4 w-4" />
            版本历史
          </Button>
        )}

        {prompt.isSystem ? (
          <Button
            onClick={onReset}
            variant="outline"
            size="sm"
            className="transition-all hover:scale-[1.02]"
            style={{
              boxShadow: '0 0 0 var(--color-accent-warm-shadow)',
              border: '1px solid var(--color-accent-warm)',
              backgroundColor: 'var(--color-bg-card)',
              color: 'var(--color-accent-warm)',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.boxShadow = '0 0 10px var(--color-accent-warm-shadow)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.boxShadow = '0 0 0 var(--color-accent-warm-shadow)';
            }}
          >
            {t('resetToDefault')}
          </Button>
        ) : (
          <Button onClick={onDelete} variant="destructive" size="sm">
            {t('delete')}
          </Button>
        )}
      </div>
    </div>
  );
});

export default PromptCard;
