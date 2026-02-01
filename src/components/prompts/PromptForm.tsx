import { useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { useTranslation } from 'react-i18next';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { Prompt } from '@/types/generated';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog';

interface PromptFormProps {
  prompt?: Prompt | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess?: () => void;
}

interface PromptFormData {
  name: string;
  content: string;
  description: string;
  scenario: string;
  category: string;
  language: string;
}

/**
 * 提示词表单组件
 *
 * 用于创建和编辑提示词
 */
export default function PromptForm({
  prompt,
  open,
  onOpenChange,
  onSuccess,
}: PromptFormProps) {
  const { t } = useTranslation('prompts');
  const queryClient = useQueryClient();

  // 表单初始化
  const {
    register,
    handleSubmit,
    formState: { errors, isDirty },
    reset,
  } = useForm<PromptFormData>({
    defaultValues: {
      name: prompt?.name ?? '',
      content: prompt?.content ?? '',
      description: prompt?.description ?? '',
      scenario: prompt?.scenario ?? 'session_analysis',
      category: prompt?.category ?? 'general',
      language: prompt?.language ?? 'zh',
    },
  });

  // 当 prompt 变化时重置表单
  useEffect(() => {
    if (prompt) {
      reset({
        name: prompt.name,
        content: prompt.content,
        description: prompt.description ?? '',
        scenario: prompt.scenario,
        category: prompt.category ?? 'general',
        language: prompt.language,
      });
    }
  }, [prompt, reset]);

  // 当对话框关闭时重置表单
  useEffect(() => {
    if (!open) {
      reset();
    }
  }, [open, reset]);

  // 保存提示词
  const saveMutation = useMutation({
    mutationFn: (data: Prompt) =>
      invoke<number>('cmd_save_prompt', { prompt: data }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['prompts'] });
      onOpenChange(false);
      onSuccess?.();
    },
    onError: (error) => {
      const errorMsg = prompt
        ? t('error.updateFailed')
        : t('error.createFailed');
      alert(`${errorMsg}: ${error}`);
    },
  });

  // 表单提交
  const onSubmit = handleSubmit((data) => {
    // 对于系统提示词，name 字段可能因为 disabled 而未包含在 data 中
    // 需要从原始 prompt 对象中获取
    const name = data.name || prompt?.name || '';

    if (!name) {
      alert(t('errors.nameRequired'));
      return;
    }

    const promptData: Prompt = {
      id: prompt?.id ?? null,
      name,
      content: data.content,
      description: data.description || null,
      scenario: data.scenario,
      category: data.category || null,
      language: data.language,
      isDefault: prompt?.isDefault ?? false,
      isSystem: prompt?.isSystem ?? false,
      version: prompt?.version ?? 1,
      createdAt: prompt?.createdAt ?? new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };

    saveMutation.mutate(promptData);
  });

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        className="max-w-2xl max-h-[90vh] overflow-y-auto"
        style={{
          backgroundColor: 'var(--color-bg-card)',
          borderColor: 'var(--color-border-light)',
        }}
      >
        <DialogHeader>
          <DialogTitle style={{ color: 'var(--color-text-primary)' }}>
            {prompt ? t('editPrompt') : t('newPrompt')}
          </DialogTitle>
        </DialogHeader>

        <form onSubmit={onSubmit} className="space-y-4">
          {/* 提示词名称 */}
          <div>
            <label
              className="block text-sm font-medium mb-2"
              style={{ color: 'var(--color-text-primary)' }}
            >
              {t('fields.name')} *
            </label>
            <input
              type="text"
              {...register('name', {
                required: t('errors.nameRequired'),
                disabled: prompt?.isSystem, // 系统提示词不可修改名称
              })}
              className="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 disabled:opacity-50 disabled:cursor-not-allowed"
              style={{
                backgroundColor: 'var(--color-bg-primary)',
                borderColor: 'var(--color-border-light)',
                color: 'var(--color-text-primary)',
                // @ts-ignore - CSS custom properties
                '--tw-ring-color': 'var(--color-accent-warm)',
              }}
              placeholder={t('placeholders.name')}
            />
            {errors.name && (
              <p
                className="mt-1 text-sm"
                style={{ color: '#ef4444' }}
              >
                {errors.name.message}
              </p>
            )}
          </div>

          {/* 提示词内容 */}
          <div>
            <label
              className="block text-sm font-medium mb-2"
              style={{ color: 'var(--color-text-primary)' }}
            >
              {t('fields.content')} *
            </label>
            <textarea
              {...register('content', {
                required: t('errors.contentRequired'),
              })}
              rows={10}
              className="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 font-mono text-sm"
              style={{
                backgroundColor: 'var(--color-bg-primary)',
                borderColor: 'var(--color-border-light)',
                color: 'var(--color-text-primary)',
                // @ts-ignore - CSS custom properties
                '--tw-ring-color': 'var(--color-accent-warm)',
              }}
              placeholder={t('placeholders.content')}
            />
            {errors.content && (
              <p
                className="mt-1 text-sm"
                style={{ color: '#ef4444' }}
              >
                {errors.content.message}
              </p>
            )}
          </div>

          {/* 描述 */}
          <div>
            <label
              className="block text-sm font-medium mb-2"
              style={{ color: 'var(--color-text-primary)' }}
            >
              {t('fields.description')}
            </label>
            <textarea
              {...register('description')}
              rows={3}
              className="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2"
              style={{
                backgroundColor: 'var(--color-bg-primary)',
                borderColor: 'var(--color-border-light)',
                color: 'var(--color-text-primary)',
                // @ts-ignore - CSS custom properties
                '--tw-ring-color': 'var(--color-accent-warm)',
              }}
              placeholder={t('placeholders.description')}
            />
          </div>

          {/* 场景和语言（行布局） */}
          <div className="grid grid-cols-2 gap-4">
            {/* 应用场景 */}
            <div>
              <label
                className="block text-sm font-medium mb-2"
                style={{ color: 'var(--color-text-primary)' }}
              >
                {t('fields.scenario')} *
              </label>
              <select
                {...register('scenario', {
                  required: t('errors.scenarioRequired'),
                })}
                className="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2"
                style={{
                  backgroundColor: 'var(--color-bg-primary)',
                  borderColor: 'var(--color-border-light)',
                  color: 'var(--color-text-primary)',
                  // @ts-ignore - CSS custom properties
                  '--tw-ring-color': 'var(--color-accent-warm)',
                }}
              >
                <option value="session_analysis">
                  {t('scenarios.session_analysis')}
                </option>
              </select>
              {errors.scenario && (
                <p
                  className="mt-1 text-sm"
                  style={{ color: '#ef4444' }}
                >
                  {errors.scenario.message}
                </p>
              )}
            </div>

            {/* 语言 */}
            <div>
              <label
                className="block text-sm font-medium mb-2"
                style={{ color: 'var(--color-text-primary)' }}
              >
                {t('fields.language')} *
              </label>
              <select
                {...register('language', {
                  required: t('errors.languageRequired'),
                })}
                className="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2"
                style={{
                  backgroundColor: 'var(--color-bg-primary)',
                  borderColor: 'var(--color-border-light)',
                  color: 'var(--color-text-primary)',
                  // @ts-ignore - CSS custom properties
                  '--tw-ring-color': 'var(--color-accent-warm)',
                }}
              >
                <option value="zh">{t('filters.zh')}</option>
                <option value="en">{t('filters.en')}</option>
              </select>
              {errors.language && (
                <p
                  className="mt-1 text-sm"
                  style={{ color: '#ef4444' }}
                >
                  {errors.language.message}
                </p>
              )}
            </div>
          </div>

          {/* 按钮 */}
          <DialogFooter>
            <Button
              type="button"
              variant="secondary"
              onClick={() => onOpenChange(false)}
              className="transition-all hover:scale-[1.02]"
              style={{
                border: '1px solid var(--color-border-light)',
                backgroundColor: 'var(--color-bg-card)',
                color: 'var(--color-text-primary)',
                boxShadow: '0 0 0 var(--color-accent-warm-shadow)',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.boxShadow = '0 0 8px var(--color-accent-warm-shadow)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.boxShadow = '0 0 0 var(--color-accent-warm-shadow)';
              }}
            >
              {t('cancel')}
            </Button>
            <Button
              type="submit"
              disabled={!isDirty || saveMutation.isPending}
              className="transition-all hover:scale-[1.02]"
              style={{
                backgroundColor: 'var(--color-accent-warm)',
                color: '#FFFFFF',
                border: '1px solid var(--color-accent-warm)',
                boxShadow: '0 0 10px var(--color-accent-warm-shadow)',
                opacity: !isDirty || saveMutation.isPending ? 0.7 : 1,
              }}
              onMouseEnter={(e) => {
                if (!isDirty || saveMutation.isPending) return;
                e.currentTarget.style.boxShadow = '0 0 14px var(--color-accent-warm-shadow)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.boxShadow = '0 0 10px var(--color-accent-warm-shadow)';
              }}
            >
              {saveMutation.isPending ? t('saving') : t('save')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
