/**
 * ProjectCard 组件
 *
 * 项目卡片组件，显示当前选中项目或引导用户选择项目
 * 支持深浅色模式自适应
 */

import { useState, useCallback } from 'react';
import { FolderOpen, FolderPlus, ChevronRight } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useCurrentProject } from '@/stores/useProjectStore';
import { ProjectSwitcherDialog } from './ProjectSwitcherDialog';
import { cn } from '@/lib/utils';

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[ProjectCard] ${action}`, ...args);
  }
}

// ==================== 类型定义 ====================

export interface ProjectCardProps {
  /**
   * 确认选择回调
   */
  onConfirm?: (project: any, sessionFile: string | null) => void;
  /**
   * 自定义类名
   */
  className?: string;
}

// ==================== 主组件 ====================

/**
 * ProjectCard 组件
 *
 * @example
 * <ProjectCard onConfirm={handleProjectChange} />
 */
export function ProjectCard({ onConfirm, className }: ProjectCardProps) {
  const { t } = useTranslation('index');
  const currentProject = useCurrentProject();
  const [dialogOpen, setDialogOpen] = useState(false);

  // 打开弹窗
  const handleOpenDialog = useCallback(() => {
    debugLog('open dialog');
    setDialogOpen(true);
  }, []);

  // 确认选择回调
  const handleConfirm = useCallback((project: any, sessionFile: string | null) => {
    debugLog('confirm', project.name, sessionFile);
    onConfirm?.(project, sessionFile);
  }, [onConfirm]);

  // ==================== 已选择项目状态 ====================
  if (currentProject) {
    return (
      <div
        className={cn(
          'rounded-lg border p-4 transition-all hover:shadow-md',
          className
        )}
        style={{
          background: 'linear-gradient(135deg, var(--color-project-card-bg) 0%, transparent 100%)',
          borderColor: 'var(--color-project-card-border)',
        }}
      >
        <div className="flex items-center justify-between gap-4">
          {/* 左侧：项目信息 */}
          <div className="flex items-center gap-3 min-w-0 flex-1">
            <FolderOpen
              className="h-6 w-6 shrink-0"
              style={{ color: 'var(--color-accent-warm)' }}
            />
            <div className="min-w-0 flex-1">
              <h3
                className="text-base font-semibold truncate"
                style={{ color: 'var(--color-text-primary)' }}
              >
                {currentProject.name}
              </h3>
              <p
                className="text-xs truncate mt-0.5"
                style={{ color: 'var(--color-text-secondary)' }}
              >
                {currentProject.path}
              </p>
            </div>
          </div>

          {/* 右侧：切换按钮 */}
          <button
            onClick={handleOpenDialog}
            className="flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium transition-colors shrink-0"
            style={{
              backgroundColor: 'var(--color-app-secondary)',
              color: 'var(--color-text-primary)',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--color-app-button-default)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--color-app-secondary)';
            }}
          >
            {t('project.switchProject')}
            <ChevronRight className="h-4 w-4" />
          </button>
        </div>

        {/* 弹窗 */}
        <ProjectSwitcherDialog
          open={dialogOpen}
          onOpenChange={setDialogOpen}
          onConfirm={handleConfirm}
        />
      </div>
    );
  }

  // ==================== 未选择项目状态 ====================
  return (
    <>
      <div
        className={cn(
          'rounded-lg border-2 border-dashed p-6 transition-all',
          'animate-pulse',
          className
        )}
        style={{
          backgroundColor: 'var(--color-project-card-empty-bg)',
          borderColor: 'var(--color-project-card-empty-border)',
          boxShadow: '0 0 20px var(--color-project-card-pulse)',
        }}
      >
        <div className="flex items-center justify-between gap-4">
          {/* 左侧：引导信息 */}
          <div className="flex items-center gap-3 min-w-0 flex-1">
            <div
              className="h-12 w-12 rounded-full flex items-center justify-center shrink-0"
              style={{ backgroundColor: 'var(--color-app-secondary)' }}
            >
              <FolderPlus
                className="h-6 w-6"
                style={{ color: 'var(--color-accent-warm)' }}
              />
            </div>
            <div className="min-w-0 flex-1">
              <h3
                className="text-base font-semibold"
                style={{ color: 'var(--color-text-primary)' }}
              >
                请选择一个项目
              </h3>
              <p
                className="text-sm mt-0.5"
                style={{ color: 'var(--color-text-secondary)' }}
              >
                选择项目以开始使用 PrismForge
              </p>
            </div>
          </div>

          {/* 右侧：选择按钮（CTA 样式） */}
          <button
            onClick={handleOpenDialog}
            className="flex items-center gap-2 px-5 py-2.5 rounded-lg text-sm font-semibold text-white transition-all shrink-0 hover:scale-105 active:scale-95"
            style={{
              backgroundColor: 'var(--color-accent-warm)',
              boxShadow: '0 4px 12px var(--color-project-card-pulse)',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.boxShadow = '0 6px 16px var(--color-project-card-pulse)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.boxShadow = '0 4px 12px var(--color-project-card-pulse)';
            }}
          >
            <FolderPlus className="h-4 w-4" />
            {t('project.selectProject')}
          </button>
        </div>
      </div>

      {/* 弹窗 */}
      <ProjectSwitcherDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        onConfirm={handleConfirm}
      />
    </>
  );
}
