/**
 * SettingsPage 组件
 *
 * 使用 Shadcn UI 的设置页面 - LLM API 提供商管理
 */

import { useState, useCallback, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { ArrowLeft, Plus } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { ThemeToggle } from '@/components/ThemeToggle';
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from '@/components/ui/tabs';
import { ProviderSettings } from '@/components/settings/ProviderSettings';
import { ProviderForm } from '@/components/settings/ProviderForm';
import { VectorSettings } from '@/components/settings/VectorSettings';
import { FilterConfigSettings } from '@/components/FilterConfigSettings';
import {
  useProviderActions,
  useProviders,
  useProvidersLoading,
  useProvidersError,
  type ProviderResponse,
  type SaveProviderRequest,
} from '@/stores/useSettingsStore';

type ViewMode = 'list' | 'create' | 'edit';
type SettingsTab = 'providers' | 'vector' | 'filter';

export interface SettingsPageProps {
  /** 自定义类名 */
  className?: string;
}

/**
 * SettingsPage 组件
 *
 * @example
 * <SettingsPage />
 */
export function SettingsPage({ className }: SettingsPageProps) {
  const { t } = useTranslation('settings');
  const navigate = useNavigate();

  // Store
  const providers = useProviders();
  const loading = useProvidersLoading();
  const error = useProvidersError();
  const { fetchProviders, saveProvider, clearError } = useProviderActions();

  // 状态
  const [viewMode, setViewMode] = useState<ViewMode>('list');
  const [selectedProvider, setSelectedProvider] = useState<ProviderResponse | null>(null);
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<SettingsTab>('providers');

  // 初始化加载
  useEffect(() => {
    fetchProviders();
  }, [fetchProviders]);

  // 返回主页
  const handleBack = useCallback(() => {
    navigate('/');
  }, [navigate]);

  // 创建新提供商
  const handleCreate = useCallback(() => {
    setSelectedProvider(null);
    setViewMode('create');
    setIsDialogOpen(true);
  }, []);

  // 编辑提供商
  const handleEdit = useCallback((provider: ProviderResponse) => {
    setSelectedProvider(provider);
    setViewMode('edit');
    setIsDialogOpen(true);
  }, []);

  // 保存提供商
  const handleSave = useCallback(
    async (data: SaveProviderRequest) => {
      try {
        await saveProvider(data);
        setIsDialogOpen(false);
        setSelectedProvider(null);
        setViewMode('list');
      } catch (err) {
        console.error('保存失败:', err);
      }
    },
    [saveProvider]
  );

  // 取消编辑
  const handleCancel = useCallback(() => {
    setIsDialogOpen(false);
    setSelectedProvider(null);
    setViewMode('list');
  }, []);

  return (
    <div className={cn('flex flex-col h-screen', className)} style={{ backgroundColor: 'var(--color-bg-primary)' }}>
      {/* 顶部导航栏 */}
      <div className="flex items-center gap-4 px-6 py-4 border-b" style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}>
        <Button
          variant="ghost"
          size="icon"
          onClick={handleBack}
          className="shrink-0"
        >
          <ArrowLeft className="h-5 w-5" />
        </Button>
        <div className="flex-1">
          <h1 className="text-xl font-bold" style={{ color: 'var(--color-text-primary)' }}>{t('title')}</h1>
        </div>
        {activeTab === 'providers' && (
          <Button onClick={handleCreate} className="shrink-0">
            <Plus className="h-4 w-4 mr-2" />
            {t('buttons.create')}
          </Button>
        )}
        <ThemeToggle />
      </div>

      {/* 主内容区域 */}
      <div className="flex-1 overflow-y-auto p-6" style={{ backgroundColor: 'var(--color-bg-primary)' }}>
        <div className="max-w-4xl mx-auto space-y-6">
          {/* 错误提示 */}
          {error && (
            <Alert variant="destructive">
              <AlertDescription className="flex items-center justify-between">
                <span>{error}</span>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={clearError}
                  className="h-6 px-2"
                >
                  {t('buttons.close')}
                </Button>
              </AlertDescription>
            </Alert>
          )}

          {/* 标签页 */}
          <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as SettingsTab)}>
            <TabsList className="grid w-full grid-cols-3" style={{ backgroundColor: 'var(--color-bg-card)' }}>
              <TabsTrigger value="providers">{t('tabs.providers')}</TabsTrigger>
              <TabsTrigger value="vector">{t('tabs.vector')}</TabsTrigger>
              <TabsTrigger value="filter">日志过滤</TabsTrigger>
            </TabsList>

            {/* API 提供商标签页 */}
            <TabsContent value="providers" className="space-y-6 mt-6">
              {/* 提供商列表 */}
              <ProviderSettings
                onSelectProvider={handleEdit}
              />

              {/* 空状态提示 */}
              {!loading && providers.length === 0 && (
                <Card className="p-12 text-center" style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}>
                  <div className="space-y-4">
                    <div className="text-6xl">⚙️</div>
                    <h3 className="text-lg font-semibold" style={{ color: 'var(--color-text-primary)' }}>{t('emptyState.title')}</h3>
                    <p className="max-w-md mx-auto" style={{ color: 'var(--color-text-secondary)' }}>
                      {t('emptyState.description')}
                    </p>
                    <Button onClick={handleCreate}>
                      <Plus className="h-4 w-4 mr-2" />
                      {t('buttons.addFirst')}
                    </Button>
                  </div>
                </Card>
              )}
            </TabsContent>

            {/* 向量搜索标签页 */}
            <TabsContent value="vector" className="space-y-6 mt-6">
              <VectorSettings />
            </TabsContent>

            {/* 日志过滤标签页 */}
            <TabsContent value="filter" className="space-y-6 mt-6">
              <FilterConfigSettings />
            </TabsContent>
          </Tabs>
        </div>
      </div>

      {/* 创建/编辑对话框 */}
      <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
        <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>
              {viewMode === 'create' ? t('dialog.createTitle') : t('dialog.editTitle')}
            </DialogTitle>
          </DialogHeader>

          <div className="mt-4">
            <ProviderForm
              provider={selectedProvider}
              onSubmit={handleSave}
              onCancel={handleCancel}
              submitText={viewMode === 'create' ? '创建' : '保存'}
              loading={loading}
            />
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
