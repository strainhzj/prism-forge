import { useState, useEffect, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { Prompt } from '@/types/generated';
import PromptCard from '@/components/prompts/PromptCard';
import PromptForm from '@/components/prompts/PromptForm';
import { PromptVersionsDrawer } from '@/components/prompt-versions';
import { Button } from '@/components/ui/button';
import { useDebounce } from '@/hooks/useDebounce';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Loading } from '@/components/ui/loading';
import { CheckCircle, AlertCircle, Home, AlertTriangle } from 'lucide-react';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';

/**
 * 将统一接口返回的数据转换为 Prompt 类型
 *
 * 统一接口返回 camelCase 字段（如 isSystem, templateId），
 * 而 Prompt 类型使用 snake_case（如 is_system, template_id）。
 * 需要进行字段映射转换。
 */
function convertToPrompt(data: any): Prompt {
  return {
    id: data.id,
    name: data.name,
    content: data.content,
    description: data.description || null,
    scenario: data.scenario,
    category: 'general', // 默认值
    isDefault: data.isSystem || false, // 从 isSystem 映射
    isSystem: data.isSystem || false,
    language: data.language || 'zh',
    version: data.versionNumber || 1, // 从 versionNumber 映射
    createdAt: data.createdAt || new Date().toISOString(),
    updatedAt: data.createdAt || new Date().toISOString(), // 使用相同的时间
  };
}

/**
 * 对话框类型常量
 */
const DIALOG_TYPE = {
  DELETE: 'delete',
  RESET: 'reset',
} as const;

type DialogType = typeof DIALOG_TYPE[keyof typeof DIALOG_TYPE];

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
  const navigate = useNavigate();

  // 状态管理
  const [searchQuery, setSearchQuery] = useState('');
  const [scenarioFilter, setScenarioFilter] = useState<string>('all');
  const [languageFilter, setLanguageFilter] = useState<string>('all');
  const [editingPrompt, setEditingPrompt] = useState<Prompt | null>(null);
  const [isFormOpen, setIsFormOpen] = useState(false);

  // 版本管理抽屉状态
  const [versionsDrawerOpen, setVersionsDrawerOpen] = useState(false);

  // 确认对话框状态
  const [confirmDialog, setConfirmDialog] = useState<{
    show: boolean;
    type: DialogType;
    data: number | string | null;
  }>({
    show: false,
    type: DIALOG_TYPE.DELETE,
    data: null,
  });

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

  // 显示 Alert（使用 useCallback 避免不必要的重渲染）
  const showAlert = useCallback((type: 'success' | 'error', message: string) => {
    setAlert({ show: true, type, message });
  }, []);

  // 自动隐藏 Alert（带清理）
  useEffect(() => {
    if (!alert.show) return;

    const timer = setTimeout(() => {
      setAlert(prev => ({ ...prev, show: false }));
    }, 3000);

    return () => clearTimeout(timer);
  }, [alert.show]);

  // 搜索防抖（300ms 延迟）
  const debouncedSearch = useDebounce(searchQuery, 300);

  // 获取提示词列表（使用统一的版本管理接口）
  const {
    data: prompts = [],
    isLoading,
    error,
  } = useQuery({
    queryKey: ['prompts', scenarioFilter, languageFilter, debouncedSearch],
    queryFn: async () => {
      const data = await invoke<any[]>('cmd_get_prompts_unified', {
        scenario: scenarioFilter === 'all' ? null : scenarioFilter,
        language: languageFilter === 'all' ? null : languageFilter,
        search: debouncedSearch.trim() || null,
      });
      // 将统一接口返回的数据转换为 Prompt 类型
      return data.map(convertToPrompt);
    },
  });

  // 注意：删除和重置功能暂时禁用，需要适配版本管理系统
  // TODO: 实现版本管理系统的删除和重置功能

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

  // 处理删除 - 暂时禁用
  const handleDelete = (_id: number | bigint) => {
    showAlert('error', '删除功能暂时禁用，请使用版本管理界面');
  };

  // 处理重置 - 暂时禁用
  const handleReset = (_name: string) => {
    showAlert('error', '重置功能暂时禁用，请使用版本管理界面');
  };

  // 确认操作 - 暂时禁用
  const handleConfirm = async () => {
    showAlert('error', '此功能暂时禁用，请使用版本管理界面');
    setConfirmDialog({ show: false, type: DIALOG_TYPE.DELETE, data: null });
  };

  // 取消操作
  const handleCancelConfirm = () => {
    setConfirmDialog({ show: false, type: DIALOG_TYPE.DELETE, data: null });
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

      {/* 页面标题 + 返回按钮 */}
      <div className="mb-6 flex items-center gap-4">
        <button
          className="back-btn"
          onClick={() => navigate('/')}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
            padding: '8px 16px',
            background: 'var(--color-bg-card)',
            color: 'var(--color-accent-warm)',
            border: '1px solid var(--color-accent-warm)',
            borderRadius: '8px',
            fontSize: '14px',
            fontWeight: '500',
            cursor: 'pointer',
            transition: 'all 0.2s',
            flexShrink: 0,
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.background = 'color-mix(in srgb, var(--color-accent-warm) 15%, var(--color-bg-card))';
            e.currentTarget.style.transform = 'translateY(-1px)';
            e.currentTarget.style.boxShadow = '0 0 10px var(--color-accent-warm-shadow)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.background = 'var(--color-bg-card)';
            e.currentTarget.style.transform = 'translateY(0)';
            e.currentTarget.style.boxShadow = 'none';
          }}
        >
          <Home size={16} />
          <span>{t('backToHome')}</span>
        </button>
        <div className="flex-1">
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
        <Button
          onClick={handleNew}
          className="transition-all hover:scale-[1.02]"
          style={{
            boxShadow: '0 0 0 var(--color-accent-warm-shadow)',
            border: '1px solid var(--color-accent-warm)',
            backgroundColor: 'var(--color-bg-card)',
            color: 'var(--color-accent-warm)',
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.boxShadow = '0 0 12px var(--color-accent-warm-shadow)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.boxShadow = '0 0 0 var(--color-accent-warm-shadow)';
          }}
        >
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
              // 新架构：所有提示词都支持版本历史
              onViewVersions={() => setVersionsDrawerOpen(true)}
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

      {/* 确认对话框 */}
      <AlertDialog open={confirmDialog.show} onOpenChange={(open) => !open && handleCancelConfirm()}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle className="flex items-center gap-2">
              <AlertTriangle className="h-5 w-5" style={{ color: 'var(--color-accent-warm)' }} />
              {confirmDialog.type === DIALOG_TYPE.DELETE ? t('confirmDelete') : t('confirmReset')}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {confirmDialog.type === DIALOG_TYPE.DELETE
                ? t('confirmDeleteDescription')
                : t('confirmResetDescription')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel
              onClick={handleCancelConfirm}
              className="px-4 py-2 rounded-md border transition-colors hover:opacity-80"
              style={{
                backgroundColor: 'var(--color-app-button-default)',
                color: 'var(--color-text-primary)',
                borderColor: 'var(--color-border-light)',
              }}
            >
              {t('cancel')}
            </AlertDialogCancel>
            <AlertDialogAction
              onClick={handleConfirm}
              className="px-4 py-2 rounded-md transition-colors hover:opacity-90"
              style={
                confirmDialog.type === DIALOG_TYPE.DELETE
                  ? {
                      backgroundColor: 'var(--color-destructive)',
                      color: 'var(--color-destructive-foreground)',
                    }
                  : {
                      backgroundColor: 'var(--color-accent-blue)',
                      color: '#FFFFFF',
                    }
              }
              aria-label={
                confirmDialog.type === DIALOG_TYPE.DELETE
                  ? t('delete')
                  : t('resetToDefault')
              }
            >
              {confirmDialog.type === DIALOG_TYPE.DELETE ? t('delete') : t('resetToDefault')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* 版本管理抽屉 */}
      <PromptVersionsDrawer
        open={versionsDrawerOpen}
        onOpenChange={setVersionsDrawerOpen}
      />
    </div>
  );
}
