/**
 * ProjectSwitcher 组件
 *
 * 项目切换器按钮，显示当前项目名称，点击打开切换弹窗
 */

import { useState, useCallback } from 'react';
import { ChevronDown, FolderOpen } from 'lucide-react';
import { useCurrentProject, useCurrentSessionFile } from '@/stores/useProjectStore';
import { ProjectSwitcherDialog } from './ProjectSwitcherDialog';
import { cn } from '@/lib/utils';

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[ProjectSwitcher] ${action}`, ...args);
  }
}

export interface ProjectSwitcherProps {
  /**
   * 确认选择回调
   */
  onConfirm?: (project: any, sessionFile: string | null) => void;
  /**
   * 自定义类名
   */
  className?: string;
}

/**
 * ProjectSwitcher 组件
 *
 * @example
 * <ProjectSwitcher />
 */
export function ProjectSwitcher({ onConfirm, className }: ProjectSwitcherProps) {
  const currentProject = useCurrentProject();
  const currentSessionFile = useCurrentSessionFile();
  const [dialogOpen, setDialogOpen] = useState(false);

  // 提取会话文件名
  const sessionFileName = currentSessionFile
    ? currentSessionFile.split(/[/\\]/).pop()
    : null;

  return (
    <>
      <button
        onClick={() => {
          debugLog('open dialog');
          setDialogOpen(true);
        }}
        className={cn(
          'flex items-center gap-2 px-4 py-2 rounded-lg border transition-all',
          'hover:shadow-md hover:scale-[1.01] active:scale-[0.99]',
          className
        )}
        style={{
          backgroundColor: 'var(--color-bg-card)',
          borderColor: 'var(--color-border-light)',
        }}
        title={currentProject?.path || '未选择项目'}
      >
        <FolderOpen className="h-4 w-4 shrink-0" style={{ color: 'var(--color-accent-warm)' }} />
        <span className="font-medium" style={{ color: 'var(--color-text-primary)' }}>
          {currentProject?.name || '未选择项目'}
        </span>
        <ChevronDown className="h-4 w-4 shrink-0" style={{ color: 'var(--color-text-secondary)' }} />
      </button>

      <ProjectSwitcherDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        onConfirm={onConfirm}
      />

      {/* 显示当前跟踪的文件路径（小字） */}
      {sessionFileName && (
        <div className="mt-1 text-xs" style={{ color: 'var(--color-text-secondary)' }}>
          跟踪: {sessionFileName}
        </div>
      )}
    </>
  );
}
