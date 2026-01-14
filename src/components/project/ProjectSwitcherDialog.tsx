/**
 * ProjectSwitcherDialog 组件
 *
 * 项目切换弹窗，支持：
 * - 项目列表展示和搜索
 * - 新建/删除项目
 * - 选择会话文件
 * - 记住上次选择
 */

import { useState, useCallback, useEffect, useMemo } from 'react';
import { Search, Plus, Trash2, Folder, FileText, Clock } from 'lucide-react';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { useProjectStore, useProjectActions, useCurrentProject } from '@/stores/useProjectStore';
import type { MonitoredDirectory, SessionFileInfo } from '@/stores/useSessionStore';
import { cn } from '@/lib/utils';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Skeleton } from '@/components/ui/skeleton';

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[ProjectSwitcherDialog] ${action}`, ...args);
  }
}

// ==================== 类型定义 ====================

export interface ProjectSwitcherDialogProps {
  /**
   * 是否打开弹窗
   */
  open: boolean;
  /**
   * 关闭弹窗回调
   */
  onOpenChange: (open: boolean) => void;
  /**
   * 确认选择回调
   */
  onConfirm?: (project: MonitoredDirectory, sessionFile: string | null) => void;
}

/**
 * 从路径中提取目录名称
 */
function extractDirectoryName(path: string): string {
  const normalizedPath = path.replace(/\\/g, '/');
  const parts = normalizedPath.split('/').filter(Boolean);
  return parts[parts.length - 1] || path;
}

/**
 * 格式化时间显示
 */
function formatRelativeTime(isoTime: string): string {
  const date = new Date(isoTime);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);

  if (diffMins < 1) return '刚刚';
  if (diffMins < 60) return `${diffMins}分钟前`;
  const diffHours = Math.floor(diffMs / 3600000);
  if (diffHours < 24) return `${diffHours}小时前`;
  const diffDays = Math.floor(diffMs / 86400000);
  return `${diffDays}天前`;
}

/**
 * ProjectSwitcherDialog 组件
 */
export function ProjectSwitcherDialog({
  open,
  onOpenChange,
  onConfirm,
}: ProjectSwitcherDialogProps) {
  const projects = useProjectStore((state) => state.projects);
  const currentProject = useCurrentProject();
  const {
    addProject,
    removeProject,
    getSessionFiles,
    setCurrentProject,
    setCurrentSessionFile,
  } = useProjectActions();

  // 弹窗内状态
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedProject, setSelectedProject] = useState<MonitoredDirectory | null>(currentProject || null);
  const [sessionFiles, setSessionFiles] = useState<SessionFileInfo[]>([]);
  const [selectedSessionFile, setSelectedSessionFile] = useState<string | null>(null);
  const [loadingFiles, setLoadingFiles] = useState(false);

  // 重置状态
  const resetState = useCallback(() => {
    setSearchQuery('');
    setSelectedProject(currentProject || null);
    setSessionFiles([]);
    setSelectedSessionFile(null);
  }, [currentProject]);

  // 加载会话文件并选择第一个（提取为独立函数）
  const loadAndSelectFirstFile = useCallback(async (project: MonitoredDirectory) => {
    debugLog('loadAndSelectFirstFile', project.name);
    setSelectedProject(project);
    setSelectedSessionFile(null);
    setLoadingFiles(true);

    try {
      const files = await getSessionFiles(project.path, false);
      setSessionFiles(files);
      // 自动选择第一个会话文件
      if (files.length > 0) {
        setSelectedSessionFile(files[0].file_path);
        debugLog('loadAndSelectFirstFile', 'auto selected first file', files[0].file_path);
      }
    } catch (error) {
      debugLog('loadAndSelectFirstFile', 'error', error);
      setSessionFiles([]);
    } finally {
      setLoadingFiles(false);
    }
  }, [getSessionFiles]);

  // 弹窗打开时重置状态，并自动加载当前项目的会话文件
  useEffect(() => {
    if (open) {
      resetState();
      // 如果存在当前项目，自动加载会话文件并选择第一个
      if (currentProject) {
        loadAndSelectFirstFile(currentProject);
      }
    }
  }, [open, resetState, currentProject, loadAndSelectFirstFile]);

  // 选择项目时加载会话文件
  const handleProjectSelect = useCallback(async (project: MonitoredDirectory) => {
    debugLog('handleProjectSelect', project.name);
    loadAndSelectFirstFile(project);
  }, [loadAndSelectFirstFile]);

  // 搜索过滤
  const filteredProjects = useMemo(() => {
    if (!searchQuery) return projects;
    const query = searchQuery.toLowerCase();
    return projects.filter(
      (p) =>
        p.name.toLowerCase().includes(query) ||
        p.path.toLowerCase().includes(query)
    );
  }, [projects, searchQuery]);

  // 新建项目
  const handleAddProject = useCallback(async () => {
    try {
      const selected = await openDialog({
        directory: true,
        multiple: false,
        title: '选择要监控的项目目录',
      });

      if (selected) {
        const extractedName = extractDirectoryName(selected);
        await addProject(selected, extractedName);
      }
    } catch (error) {
      debugLog('handleAddProject', 'error', error);
    }
  }, [addProject]);

  // 删除项目
  const handleRemoveProject = useCallback(async (project: MonitoredDirectory) => {
    if (project.id === undefined) return;

    const confirmed = confirm(`确定要删除项目"${project.name}"吗？`);
    if (!confirmed) return;

    try {
      await removeProject(project.id);
      if (selectedProject?.id === project.id) {
        setSelectedProject(null);
        setSessionFiles([]);
      }
    } catch (error) {
      debugLog('handleRemoveProject', 'error', error);
    }
  }, [removeProject, selectedProject]);

  // 确认选择
  const handleConfirm = useCallback(() => {
    if (!selectedProject) return;

    debugLog('handleConfirm', selectedProject.name, selectedSessionFile);
    setCurrentProject(selectedProject);
    setCurrentSessionFile(selectedSessionFile);
    onConfirm?.(selectedProject, selectedSessionFile);
    onOpenChange(false);
  }, [selectedProject, selectedSessionFile, setCurrentProject, setCurrentSessionFile, onConfirm, onOpenChange]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[850px] w-[95vw]" style={{ backgroundColor: 'var(--color-bg-card)' }}>
        <DialogHeader>
          <DialogTitle style={{ color: 'var(--color-text-primary)' }}>切换项目</DialogTitle>
          <DialogDescription style={{ color: 'var(--color-text-secondary)' }}>
            选择一个项目并选择要跟踪的会话文件
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4 overflow-y-auto max-h-[70vh] pr-2">
          {/* 搜索框 */}
          <div className="flex items-center gap-2">
            <div className="relative flex-1 min-w-0">
              <Search
                className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 shrink-0"
                style={{ color: 'var(--color-text-secondary)' }}
              />
              <Input
                placeholder="搜索项目名称..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-9 pr-3 [&::placeholder]:text-[var(--color-text-secondary)]"
                style={{
                  backgroundColor: 'var(--color-bg-primary)',
                  border: '1px solid var(--color-border-light)',
                  color: 'var(--color-text-primary)'
                }}
              />
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={handleAddProject}
              className="shrink-0 whitespace-nowrap"
              style={{
                border: '1px solid var(--color-border-light)',
                backgroundColor: 'var(--color-bg-primary)',
                color: 'var(--color-text-primary)'
              }}
            >
              <Plus className="h-4 w-4 mr-1 shrink-0" />
              <span className="hidden sm:inline">新建项目</span>
              <span className="sm:hidden">新建</span>
            </Button>
          </div>

          {/* 项目列表 */}
          <div
            className="rounded-lg max-h-[300px] overflow-y-auto w-full"
            style={{
              border: '1px solid var(--color-border-light)',
              backgroundColor: 'var(--color-bg-card)'
            }}
          >
            {filteredProjects.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-8 px-4 text-center">
                <p className="text-sm break-words" style={{ color: 'var(--color-text-secondary)' }}>
                  {searchQuery ? '未找到匹配的项目' : '暂无项目，请点击"新建项目"添加'}
                </p>
              </div>
            ) : (
              <ul className="w-full" style={{ borderTop: '1px solid var(--color-border-light)' }}>
                {filteredProjects.map((project) => (
                  <li
                    key={project.id}
                    className={cn(
                      'flex items-center gap-3 px-4 py-3 transition-colors cursor-pointer',
                      selectedProject?.id === project.id ? 'bg-[var(--color-app-secondary)]' : 'hover:bg-[var(--color-app-secondary)]'
                    )}
                    style={{
                      borderBottom: '1px solid var(--color-border-light)'
                    }}
                    onClick={() => handleProjectSelect(project)}
                  >
                    <Folder
                      className={cn(
                        'h-5 w-5 shrink-0',
                        selectedProject?.id === project.id
                          ? 'text-[var(--color-accent-warm)]'
                          : ''
                      )}
                      style={{
                        color: selectedProject?.id === project.id
                          ? 'var(--color-accent-warm)'
                          : 'var(--color-text-secondary)'
                      }}
                    />
                    <div className="flex-1 min-w-0 overflow-hidden">
                      <div className="flex items-center gap-2 min-w-0">
                        <span className="font-medium truncate" style={{ color: 'var(--color-text-primary)' }}>
                          {project.name}
                        </span>
                        {!project.is_active && (
                          <span className="shrink-0" style={{ color: 'var(--color-text-secondary)', fontSize: '12px' }}>(禁用)</span>
                        )}
                      </div>
                      <p className="text-xs truncate" style={{ color: 'var(--color-text-secondary)' }}>
                        {project.path}
                      </p>
                    </div>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="shrink-0 h-8 w-8 hover:bg-[var(--color-app-secondary)]"
                      style={{ color: 'var(--color-destructive)' }}
                      onClick={(e) => {
                        e.stopPropagation();
                        handleRemoveProject(project);
                      }}
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </li>
                ))}
              </ul>
            )}
          </div>

          {/* 会话文件列表（选择项目后显示） */}
          {selectedProject && (
            <div className="space-y-2">
              <p className="text-sm font-medium break-words" style={{ color: 'var(--color-text-primary)' }}>
                选择会话文件 (可选)
              </p>
              <div
                className="rounded-lg max-h-[200px] overflow-y-auto w-full"
                style={{
                  border: '1px solid var(--color-border-light)',
                  backgroundColor: 'var(--color-bg-card)'
                }}
              >
                {loadingFiles ? (
                  <div className="p-4 space-y-2">
                    {[...Array(3)].map((_, i) => (
                      <div key={i} className="flex items-center gap-3">
                        <Skeleton className="h-4 w-4 shrink-0" />
                        <Skeleton className="flex-1 h-4 min-w-0" />
                      </div>
                    ))}
                  </div>
                ) : sessionFiles.length === 0 ? (
                  <div className="flex flex-col items-center justify-center py-6 px-4 text-center">
                    <FileText
                      className="h-8 w-8 mb-2 shrink-0"
                      style={{ color: 'var(--color-text-secondary)' }}
                    />
                    <p className="text-sm break-words" style={{ color: 'var(--color-text-secondary)' }}>
                      该项目下暂无会话文件
                    </p>
                  </div>
                ) : (
                  <ul className="w-full" style={{ borderTop: '1px solid var(--color-border-light)' }}>
                    {sessionFiles.map((file) => (
                      <li
                        key={file.session_id}
                        className={cn(
                          'flex items-center gap-3 px-4 py-2 transition-colors cursor-pointer',
                          selectedSessionFile === file.file_path ? 'bg-[var(--color-app-secondary)]' : 'hover:bg-[var(--color-app-secondary)]'
                        )}
                        style={{
                          borderBottom: '1px solid var(--color-border-light)'
                        }}
                        onClick={() => setSelectedSessionFile(file.file_path)}
                      >
                        <FileText
                          className={cn(
                            'h-4 w-4 shrink-0'
                          )}
                          style={{
                            color: selectedSessionFile === file.file_path
                              ? 'var(--color-accent-warm)'
                              : 'var(--color-text-secondary)'
                          }}
                        />
                        <div className="flex-1 min-w-0 overflow-hidden">
                          <p className="text-sm truncate" style={{ color: 'var(--color-text-primary)' }}>
                            {file.display_name || file.summary || file.session_id}
                          </p>
                          <div className="flex items-center gap-2 text-xs" style={{ color: 'var(--color-text-secondary)' }}>
                            <span className="flex items-center gap-1 shrink-0">
                              <Clock className="h-3 w-3 shrink-0" />
                              <span className="truncate">{formatRelativeTime(file.modified_time)}</span>
                            </span>
                          </div>
                        </div>
                      </li>
                    ))}
                  </ul>
                )}
              </div>
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            取消
          </Button>
          <Button
            onClick={handleConfirm}
            disabled={!selectedProject}
            style={{ backgroundColor: 'var(--color-accent-warm)' }}
          >
            确认选择
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
