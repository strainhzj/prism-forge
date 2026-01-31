import { useState, useCallback, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetDescription } from '@/components/ui/sheet';
import { Loading } from '@/components/ui/loading';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { CheckCircle, AlertCircle, Lock } from 'lucide-react';
import type { PromptVersion, PromptVersionDiff, PromptComponent, PromptParameter, PromptChange } from '@/types/generated';
import { VersionListTable } from './VersionListTable';
import { VersionDetailPanel } from './VersionDetailPanel';
import { VersionComparePanel } from './VersionComparePanel';
import { ChangeHistoryPanel } from './ChangeHistoryPanel';
import { RollbackDialog } from './RollbackDialog';
import './styles.css';

type TabType = 'detail' | 'compare' | 'history';

interface PromptVersionsDrawerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  templateName?: string;
}

type AlertType = 'success' | 'error';

/**
 * 提示词版本管理抽屉组件
 *
 * 功能：
 * - 显示版本列表
 * - 查看版本详情
 * - 版本对比（需要选择 2 个版本）
 * - 变更历史
 * - 版本回滚（软回滚/硬回滚）
 */
export function PromptVersionsDrawer({ open, onOpenChange, templateName = 'session_analysis' }: PromptVersionsDrawerProps) {
  const { t } = useTranslation('promptVersions');
  const queryClient = useQueryClient();

  // 状态管理
  const [selectedVersion, setSelectedVersion] = useState<number | null>(null);
  const [activeTab, setActiveTab] = useState<TabType>('detail');
  const [selectedForCompare, setSelectedForCompare] = useState<Set<number>>(new Set());
  const [rollbackDialogOpen, setRollbackDialogOpen] = useState(false);

  // Alert 状态
  const [alert, setAlert] = useState<{
    show: boolean;
    type: AlertType;
    message: string;
  }>({
    show: false,
    type: 'success',
    message: '',
  });

  // 显示 Alert
  const showAlert = useCallback((type: AlertType, message: string) => {
    setAlert({ show: true, type, message });
    setTimeout(() => {
      setAlert(prev => ({ ...prev, show: false }));
    }, 3000);
  }, []);

  // 从勾选的版本中提取 compareFrom 和 compareTo
  const compareFrom = useMemo(() => {
    if (selectedForCompare.size < 2) return null;
    const sorted = Array.from(selectedForCompare).sort((a, b) => a - b);
    return sorted[0];
  }, [selectedForCompare]);

  const compareTo = useMemo(() => {
    if (selectedForCompare.size < 2) return null;
    const sorted = Array.from(selectedForCompare).sort((a, b) => a - b);
    return sorted[1];
  }, [selectedForCompare]);

  // 对比标签是否可点击（需要恰好选择 2 个版本）
  const isCompareTabEnabled = useMemo(() => {
    return selectedForCompare.size === 2;
  }, [selectedForCompare.size]);

  // 获取模板
  const { data: template, isLoading: templateLoading } = useQuery({
    queryKey: ['prompt-template', templateName],
    queryFn: () => invoke<any>('cmd_get_prompt_template_by_name', { name: templateName }),
    enabled: open && !!templateName,
  });

  // 获取版本列表
  const { data: versions = [], isLoading: versionsLoading } = useQuery({
    queryKey: ['prompt-versions', template?.id],
    queryFn: () => invoke<PromptVersion[]>('cmd_get_prompt_versions', { templateId: template?.id || 0 }),
    enabled: open && !!template?.id,
  });

  // 获取激活版本
  const { data: activeVersion } = useQuery({
    queryKey: ['prompt-active-version', template?.id],
    queryFn: () => invoke<PromptVersion>('cmd_get_active_prompt_version', { templateId: template?.id || 0 }),
    enabled: open && !!template?.id,
  });

  // 获取选中版本的详情
  const { data: versionDetail } = useQuery({
    queryKey: ['prompt-version-detail', template?.id, selectedVersion],
    queryFn: () => invoke<PromptVersion>('cmd_get_prompt_version_by_number', {
      templateId: template?.id || 0,
      versionNumber: selectedVersion || 0,
    }),
    enabled: open && !!template?.id && selectedVersion !== null,
  });

  // 获取版本组件
  const { data: components = [] } = useQuery({
    queryKey: ['prompt-components', versionDetail?.id],
    queryFn: () => invoke<PromptComponent[]>('cmd_get_prompt_components', { versionId: versionDetail?.id || 0 }),
    enabled: open && !!versionDetail?.id,
  });

  // 获取版本参数
  const { data: parameters = [] } = useQuery({
    queryKey: ['prompt-parameters', versionDetail?.id],
    queryFn: () => invoke<PromptParameter[]>('cmd_get_prompt_parameters', { versionId: versionDetail?.id || 0 }),
    enabled: open && !!versionDetail?.id,
  });

  // 自动迁移组件数据（修复旧数据）
  // 当版本列表加载完成时，自动为没有组件的版本迁移数据
  const { data: migrateResult } = useQuery({
    queryKey: ['prompt-migrate-components', template?.id],
    queryFn: () => invoke<number>('cmd_migrate_all_template_components', {
      templateId: template?.id || 0,
    }),
    enabled: open && !!template?.id && versions.length > 0,
    staleTime: Infinity, // 只执行一次
  });

  // 版本对比
  const { data: comparison } = useQuery({
    queryKey: ['prompt-compare', template?.id, compareFrom, compareTo, migrateResult],
    queryFn: () => invoke<PromptVersionDiff>('cmd_compare_prompt_versions', {
      templateId: template?.id || 0,
      fromVersion: compareFrom || 0,
      toVersion: compareTo || 0,
    }),
    enabled: open && !!template?.id && compareFrom !== null && compareTo !== null && compareFrom !== compareTo,
  });

  // 变更历史
  const { data: changes = [] } = useQuery({
    queryKey: ['prompt-changes', template?.id, compareTo],
    queryFn: () => invoke<PromptChange[]>('cmd_get_prompt_version_changes', {
      templateId: template?.id || 0,
      fromVersion: null,
      toVersion: compareTo || 0,
    }),
    enabled: open && !!template?.id && compareTo !== null,
  });

  // 软回滚
  const activateMutation = useMutation({
    mutationFn: (versionNumber: number) =>
      invoke('cmd_activate_prompt_version', {
        templateId: template?.id || 0,
        versionNumber,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['prompt-versions'] });
      queryClient.invalidateQueries({ queryKey: ['prompt-active-version'] });
      showAlert('success', t('success.rolledBackSoft', { version: selectedVersion }));
    },
    onError: (error) => {
      showAlert('error', `${t('error.rollbackFailed')}: ${error}`);
    },
  });

  // 硬回滚
  const rollbackHardMutation = useMutation({
    mutationFn: ({ versionNumber, comment }: { versionNumber: number; comment?: string }) =>
      invoke('cmd_rollback_prompt_version_hard', {
        templateId: template?.id || 0,
        versionNumber,
        comment,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['prompt-versions'] });
      queryClient.invalidateQueries({ queryKey: ['prompt-active-version'] });
      showAlert('success', t('success.rolledBackHard', { targetVersion: selectedVersion }));
    },
    onError: (error) => {
      showAlert('error', `${t('error.rollbackFailed')}: ${error}`);
    },
  });

  // 处理版本选择（点击行查看详情）
  const handleSelectVersion = useCallback((versionNumber: number) => {
    setSelectedVersion(versionNumber);
    setActiveTab('detail');
  }, []);

  // 处理勾选/取消勾选版本用于对比
  const handleToggleCompareSelection = useCallback((versionNumber: number) => {
    setSelectedForCompare((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(versionNumber)) {
        newSet.delete(versionNumber);
      } else if (newSet.size < 2) {
        newSet.add(versionNumber);
      }
      return newSet;
    });
  }, []);

  // 处理回滚
  const handleRollback = useCallback((strategy: 'soft' | 'hard', comment?: string) => {
    if (selectedVersion === null) return;

    if (strategy === 'soft') {
      activateMutation.mutate(selectedVersion);
    } else {
      rollbackHardMutation.mutate({ versionNumber: selectedVersion, comment });
    }

    setRollbackDialogOpen(false);
  }, [selectedVersion, activateMutation, rollbackHardMutation]);

  // 处理对比版本变更（从对比面板内下拉框变更）
  const handleCompareChange = useCallback((from: number | null, to: number | null) => {
    setSelectedForCompare(() => {
      const newSet = new Set<number>();
      if (from !== null) newSet.add(from);
      if (to !== null) newSet.add(to);
      return newSet;
    });
  }, []);

  // 处理标签切换
  const handleTabChange = useCallback((tab: TabType) => {
    if (tab === 'compare' && !isCompareTabEnabled) {
      return; // 禁用状态下不切换
    }
    setActiveTab(tab);
  }, [isCompareTabEnabled]);

  // 加载状态
  if (templateLoading || versionsLoading) {
    return (
      <Sheet open={open} onOpenChange={onOpenChange}>
        <SheetContent className="w-full sm:max-w-4xl overflow-hidden">
          <div className="flex items-center justify-center h-full">
            <Loading text={t('versionList')} />
          </div>
        </SheetContent>
      </Sheet>
    );
  }

  // 错误状态 - 模板不存在
  if (!template && !templateLoading) {
    return (
      <Sheet open={open} onOpenChange={onOpenChange}>
        <SheetContent className="w-full sm:max-w-4xl overflow-hidden">
          <SheetHeader>
            <SheetTitle>{t('title')}</SheetTitle>
            <SheetDescription>{t('description')}</SheetDescription>
          </SheetHeader>
          <div className="flex flex-col items-center justify-center h-full gap-6">
            <div className="text-center">
              <AlertCircle className="h-16 w-16 mx-auto mb-4" style={{ color: 'var(--color-text-secondary)' }} />
              <h3 className="text-lg font-medium mb-2" style={{ color: 'var(--color-text-primary)' }}>
                未找到版本管理模板
              </h3>
              <p className="text-sm mb-4" style={{ color: 'var(--color-text-secondary)' }}>
                模板名称: {templateName}
              </p>
              <p className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
                请检查数据库中是否存在该模板
              </p>
            </div>
          </div>
        </SheetContent>
      </Sheet>
    );
  }

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent className="w-full sm:max-w-6xl overflow-hidden flex flex-col">
        {/* Alert */}
        {alert.show && (
          <div className="absolute top-4 left-4 right-4 z-50">
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

        {/* Header */}
        <SheetHeader className="mb-6">
          <SheetTitle>{t('title')}</SheetTitle>
          <SheetDescription>{t('description')}</SheetDescription>
        </SheetHeader>

        {/* Main Content */}
        <div className="flex-1 flex gap-6 overflow-hidden">
          {/* Left Panel: Version List */}
          <div className="w-1/3 overflow-y-auto prompt-versions-scroll">
            <VersionListTable
              versions={versions}
              activeVersion={activeVersion}
              selectedVersion={selectedVersion}
              selectedForCompare={selectedForCompare}
              onSelectVersion={handleSelectVersion}
              onToggleCompareSelection={handleToggleCompareSelection}
            />
          </div>

          {/* Right Panel: Details / Compare / History */}
          <div className="flex-1 overflow-y-auto prompt-versions-scroll">
            {selectedVersion !== null && versionDetail ? (
              <div className="fade-in">
                {/* Tabs */}
                <div className="flex gap-1 mb-4 border-b" style={{ borderColor: 'var(--color-border-light)' }}>
                  <button
                    onClick={() => handleTabChange('detail')}
                    className={`px-4 py-2 text-sm font-medium cursor-pointer transition-colors ${
                      activeTab === 'detail'
                        ? 'tab-active'
                        : 'text-secondary hover:text-primary'
                    }`}
                    style={
                      activeTab === 'detail'
                        ? { borderBottom: '2px solid var(--color-accent-blue)', color: 'var(--color-accent-blue)' }
                        : { color: 'var(--color-text-secondary)' }
                    }
                  >
                    {t('detail')}
                  </button>
                  <div className="relative group">
                    <button
                      onClick={() => handleTabChange('compare')}
                      disabled={!isCompareTabEnabled}
                      className={`px-4 py-2 text-sm font-medium transition-colors flex items-center gap-1.5 ${
                        activeTab === 'compare'
                          ? 'tab-active'
                          : isCompareTabEnabled
                          ? 'text-secondary hover:text-primary cursor-pointer'
                          : 'cursor-not-allowed opacity-50'
                      }`}
                      style={
                        activeTab === 'compare'
                          ? { borderBottom: '2px solid var(--color-accent-blue)', color: 'var(--color-accent-blue)' }
                          : { color: 'var(--color-text-secondary)' }
                      }
                    >
                      {!isCompareTabEnabled && <Lock className="w-3 h-3" />}
                      {t('compare')}
                    </button>
                    {/* Tooltip */}
                    {!isCompareTabEnabled && (
                      <div className="absolute left-0 top-full mt-2 px-3 py-2 rounded-lg shadow-lg text-xs whitespace-nowrap z-10 opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none" style={{
                        backgroundColor: 'var(--color-bg-card)',
                        border: '1px solid var(--color-border-light)',
                        color: 'var(--color-text-secondary)',
                      }}>
                        {t('compareDisabledTooltip')}
                      </div>
                    )}
                  </div>
                  <button
                    onClick={() => handleTabChange('history')}
                    className={`px-4 py-2 text-sm font-medium cursor-pointer transition-colors ${
                      activeTab === 'history'
                        ? 'tab-active'
                        : 'text-secondary hover:text-primary'
                    }`}
                    style={
                      activeTab === 'history'
                        ? { borderBottom: '2px solid var(--color-accent-blue)', color: 'var(--color-accent-blue)' }
                        : { color: 'var(--color-text-secondary)' }
                    }
                  >
                    {t('history')}
                  </button>
                </div>

                {/* Tab Content */}
                {activeTab === 'detail' && (
                  <VersionDetailPanel
                    version={versionDetail}
                    components={components}
                    parameters={parameters}
                    onCompare={() => {
                      // 快捷方式：从详情面板点击"对比"按钮
                      // 自动将当前详情版本加入对比选择
                      if (selectedVersion !== null) {
                        setSelectedForCompare((prev) => {
                          const newSet = new Set(prev);
                          if (!newSet.has(selectedVersion) && newSet.size < 2) {
                            newSet.add(selectedVersion);
                          }
                          // 如果恰好有 2 个版本，自动切换到对比标签
                          if (newSet.size === 2) {
                            setActiveTab('compare');
                          }
                          return newSet;
                        });
                      }
                    }}
                    onRollback={() => setRollbackDialogOpen(true)}
                  />
                )}

                {activeTab === 'compare' && (
                  <>
                    {comparison ? (
                      <VersionComparePanel
                        versions={versions}
                        comparison={comparison}
                        compareFrom={compareFrom}
                        compareTo={compareTo}
                        onCompareChange={handleCompareChange}
                      />
                    ) : (
                      <div className="flex flex-col items-center justify-center h-full py-16" style={{ color: 'var(--color-text-secondary)' }}>
                        <Lock className="w-12 h-12 mb-4 opacity-40" />
                        <p className="text-sm mb-2">{t('selectVersionsHint')}</p>
                        <p className="text-xs opacity-70">{t('compareDisabledTooltip')}</p>
                      </div>
                    )}
                  </>
                )}

                {activeTab === 'history' && (
                  <ChangeHistoryPanel changes={changes} />
                )}
              </div>
            ) : (
              <div className="flex items-center justify-center h-full text-secondary">
                {t('view')}
              </div>
            )}
          </div>
        </div>

        {/* Rollback Dialog */}
        <RollbackDialog
          open={rollbackDialogOpen}
          onOpenChange={setRollbackDialogOpen}
          onConfirm={handleRollback}
          loading={activateMutation.isPending || rollbackHardMutation.isPending}
        />
      </SheetContent>
    </Sheet>
  );
}
