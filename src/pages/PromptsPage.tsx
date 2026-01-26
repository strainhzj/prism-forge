import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { Prompt } from '@/types/generated';
import PromptCard from '@/components/prompts/PromptCard';
import PromptForm from '@/components/prompts/PromptForm';
import { Button } from '@/components/ui/button';
import { useDebounce } from '@/hooks/useDebounce';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Loading } from '@/components/ui/loading';
import { CheckCircle, AlertCircle } from 'lucide-react';

/**
 * 提示词管理页面
 *
 * 功能：
 * - 展示提示词列表
 * - 提供搜索和过滤功能
 * - 创建、编辑、删除提示词
 */
export default function PromptsPage() {
  const { t } = useTranslation('prompts');
  const queryClient = useQueryClient();

  // 状态管理
  const [searchQuery, setSearchQuery] = useState('');
  const [scenarioFilter, setScenarioFilter] = useState<string>('all');
  const [languageFilter, setLanguageFilter] = useState<string>('all');
  const [editingPrompt, setEditingPrompt] = useState<Prompt | null>(null);
  const [isFormOpen, setIsFormOpen] = useState(false);

  // Alert 状态
  const [alert, setAlert] = useState<{
    show: boolean;
    type: 'success' | 'error';
    message: string;
  }>({
    show: false,
    type: 'success',
    message: '',
  });

  // 显示 Alert
  const showAlert = (type: 'success' | 'error', message: string) => {
    setAlert({ show: true, type, message });
    setTimeout(() => {
      setAlert(prev => ({ ...prev, show: false }));
    }, 3000);
  };

  // 搜索防抖（300ms 延迟）
  const debouncedSearch = useDebounce(searchQuery, 300);

  // 获取提示词列表
  const {
    data: prompts = [],
    isLoading,
    error,
  } = useQuery({
    queryKey: ['prompts', scenarioFilter, languageFilter, debouncedSearch],
    queryFn: () =>
      invoke<Prompt[]>('cmd_get_prompts', {
        scenario: scenarioFilter === 'all' ? null : scenarioFilter,
        language: languageFilter === 'all' ? null : languageFilter,
        search: debouncedSearch.trim() || null,
      }),
  });

  // 删除提示词
  const deleteMutation = useMutation({
    mutationFn: (id: number) => invoke('cmd_delete_prompt', { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['prompts'] });
      showAlert('success', t('success.deleted'));
    },
    onError: (error) => {
      showAlert('error', `${t('error.deleteFailed')}: ${error}`);
    },
  });

  // 重置默认提示词
  const resetMutation = useMutation({
    mutationFn: (name: string) => invoke('cmd_reset_default_prompt', { name }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['prompts'] });
      showAlert('success', t('success.reset'));
    },
    onError: (error) => {
      showAlert('error', `${t('error.resetFailed')}: ${error}`);
    },
  });

  // 处理新建
  const handleNew = () => {
    setEditingPrompt(null);
    setIsFormOpen(true);
  };

  // 处理编辑
  const handleEdit = (prompt: Prompt) => {
    setEditingPrompt(prompt);
    setIsFormOpen(true);
  };

  // 处理删除
  const handleDelete = (id: number | bigint) => {
    if (confirm(t('confirmDelete'))) {
      deleteMutation.mutate(Number(id));
    }
  };

  // 处理重置
  const handleReset = (name: string) => {
    if (confirm(t('confirmReset'))) {
      resetMutation.mutate(name);
    }
  };

  // 加载状态
  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <Loading text={t('loading')} />
      </div>
    );
  }

  // 错误状态
  if (error) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div
          className="text-center"
          style={{ color: '#ef4444' }}
        >
          {t('errorLoading')}: {error.message}
        </div>
      </div>
    );
  }

  return (
    <div className="container mx-auto px-4 py-8">
      {/* 全局 Alert */}
      {alert.show && (
        <div className="mb-4">
          <Alert variant={alert.type === 'success' ? 'success' : 'destructive'}>
            {alert.type === 'success' ? (
              <CheckCircle className="h-4 w-4" />
            ) : (
              <AlertCircle className="h-4 w-4" />
            )}
            <AlertDescription>{alert.message}</AlertDescription>
          </Alert>
        </div>
      )}

      {/* 页面标题 */}
      <div className="mb-6">
        <h1
          className="text-3xl font-bold"
          style={{ color: 'var(--color-text-primary)' }}
        >
          {t('title')}
        </h1>
        <p
          className="mt-2"
          style={{ color: 'var(--color-text-secondary)' }}
        >
          {t('description')}
        </p>
      </div>

      {/* 工具栏 */}
      <div
        className="mb-6 flex flex-wrap gap-4 items-center justify-between p-4 rounded-lg border"
        style={{
          backgroundColor: 'var(--color-bg-card)',
          borderColor: 'var(--color-border-light)',
        }}
      >
        {/* 左侧：搜索框和过滤器 */}
        <div className="flex flex-wrap gap-4 items-center flex-1">
          {/* 搜索框 */}
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder={t('searchPlaceholder')}
            className="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 min-w-[200px] flex-1"
            style={{
              backgroundColor: 'var(--color-bg-primary)',
              borderColor: 'var(--color-border-light)',
              color: 'var(--color-text-primary)',
              // @ts-ignore - CSS custom properties
              '--tw-ring-color': 'var(--color-accent-warm)',
            }}
          />

          {/* 过滤器 */}
          <div className="flex gap-4">
            {/* 场景过滤 */}
            <select
              value={scenarioFilter}
              onChange={(e) => setScenarioFilter(e.target.value)}
              className="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2"
              style={{
                backgroundColor: 'var(--color-bg-primary)',
                borderColor: 'var(--color-border-light)',
                color: 'var(--color-text-primary)',
                // @ts-ignore - CSS custom properties
                '--tw-ring-color': 'var(--color-accent-warm)',
              }}
            >
              <option value="all">{t('filters.allScenarios')}</option>
              <option value="session_analysis">
                {t('scenarios.session_analysis')}
              </option>
            </select>

            {/* 语言过滤 */}
            <select
              value={languageFilter}
              onChange={(e) => setLanguageFilter(e.target.value)}
              className="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2"
              style={{
                backgroundColor: 'var(--color-bg-primary)',
                borderColor: 'var(--color-border-light)',
                color: 'var(--color-text-primary)',
                // @ts-ignore - CSS custom properties
                '--tw-ring-color': 'var(--color-accent-warm)',
              }}
            >
              <option value="all">{t('filters.allLanguages')}</option>
              <option value="zh">{t('filters.zh')}</option>
              <option value="en">{t('filters.en')}</option>
            </select>
          </div>
        </div>

        {/* 右侧：新建按钮 */}
        <Button onClick={handleNew}>
          + {t('newPrompt')}
        </Button>
      </div>

      {/* 提示词列表 */}
      <div className="space-y-4">
        {prompts.length === 0 ? (
          <div
            className="text-center py-12 rounded-lg border"
            style={{
              color: 'var(--color-text-secondary)',
              backgroundColor: 'var(--color-bg-card)',
              borderColor: 'var(--color-border-light)',
            }}
          >
            {t('noPromptsFound')}
          </div>
        ) : (
          prompts.map((prompt) => (
            <PromptCard
              key={prompt.id}
              prompt={prompt}
              onEdit={() => handleEdit(prompt)}
              onDelete={() => {
                const id = prompt.id;
                if (id === null || id === undefined) return;
                handleDelete(id);
              }}
              onReset={() => handleReset(prompt.name)}
            />
          ))
        )}
      </div>

      {/* 表单对话框 */}
      <PromptForm
        prompt={editingPrompt}
        open={isFormOpen}
        onOpenChange={setIsFormOpen}
        onSuccess={() => {
          const successMsg = editingPrompt
            ? t('success.updated')
            : t('success.created');
          showAlert('success', successMsg);
        }}
      />
    </div>
  );
}
