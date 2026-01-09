/**
 * ProjectSidebar 组件
 *
 * 显示项目分组列表，支持折叠/展开，支持手动管理监控目录
 */

import { useState, useCallback, useEffect } from 'react';
import { ChevronDown, ChevronRight, Folder, Plus, Trash2, Power } from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';
import { cn } from '@/lib/utils';
import {
  useProjectGroups,
  useSessionActions,
  useMonitoredDirectories,
  useMonitoredDirectoryActions,
} from '@/stores/useSessionStore';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';

/**
 * 从路径中提取目录名称
 */
function extractDirectoryName(path: string): string {
  // 处理 Windows 和 Unix 风格的路径
  const normalizedPath = path.replace(/\\/g, '/');
  const parts = normalizedPath.split('/').filter(Boolean);
  return parts[parts.length - 1] || path;
}

export interface ProjectSidebarProps {
  /**
   * 项目选择回调
   */
  onProjectSelect?: (projectPath: string) => void;
  /**
   * 当前选中的项目路径
   */
  selectedProject?: string;
  /**
   * 自定义类名
   */
  className?: string;
}

/**
 * ProjectSidebar 组件
 *
 * @example
 * <ProjectSidebar
 *   onProjectSelect={(path) => console.log(path)}
 *   selectedProject="/path/to/project"
 * />
 */
export function ProjectSidebar({
  onProjectSelect,
  selectedProject,
  className,
}: ProjectSidebarProps) {
  const projects = useProjectGroups();
  const monitoredDirectories = useMonitoredDirectories();
  const { setActiveSessions } = useSessionActions();
  const {
    fetchMonitoredDirectories,
    addMonitoredDirectory,
    removeMonitoredDirectory,
    toggleMonitoredDirectory,
  } = useMonitoredDirectoryActions();

  // 折叠状态
  const [collapsedProjects, setCollapsedProjects] = useState<Set<string>>(new Set());

  // 目录管理对话框状态
  const [directoryDialogOpen, setDirectoryDialogOpen] = useState(false);
  const [newDirectoryPath, setNewDirectoryPath] = useState('');
  const [newDirectoryName, setNewDirectoryName] = useState('');

  // 初始化时加载监控目录
  useEffect(() => {
    fetchMonitoredDirectories();
  }, [fetchMonitoredDirectories]);

  // 打开目录选择对话框
  const handleSelectDirectory = useCallback(async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: '选择要监控的 Claude 会话目录',
      });

      if (selected) {
        setNewDirectoryPath(selected);
        // 自动从路径提取目录名称
        const extractedName = extractDirectoryName(selected);
        setNewDirectoryName(extractedName);
        setDirectoryDialogOpen(true);
      }
    } catch (error) {
      console.error('选择目录失败:', error);
    }
  }, []);

  // 切换折叠状态
  const toggleCollapse = useCallback((projectPath: string) => {
    setCollapsedProjects((prev) => {
      const next = new Set(prev);
      if (next.has(projectPath)) {
        next.delete(projectPath);
      } else {
        next.add(projectPath);
      }
      return next;
    });
  }, []);

  // 选择项目
  const handleProjectSelect = useCallback(
    (projectPath: string) => {
      onProjectSelect?.(projectPath);
    },
    [onProjectSelect]
  );

  // 刷新会话列表
  const handleRefresh = useCallback(async () => {
    try {
      await setActiveSessions();
    } catch (error) {
      console.error('刷新会话列表失败:', error);
    }
  }, [setActiveSessions]);

  // 添加监控目录
  const handleAddDirectory = useCallback(async () => {
    if (!newDirectoryPath.trim() || !newDirectoryName.trim()) {
      return;
    }

    try {
      await addMonitoredDirectory(newDirectoryPath, newDirectoryName);
      setNewDirectoryPath('');
      setNewDirectoryName('');
      setDirectoryDialogOpen(false);
      // 刷新会话列表
      await handleRefresh();
    } catch (error) {
      console.error('添加目录失败:', error);
    }
  }, [newDirectoryPath, newDirectoryName, addMonitoredDirectory, handleRefresh]);

  // 删除监控目录
  const handleRemoveDirectory = useCallback(
    async (id: number) => {
      try {
        await removeMonitoredDirectory(id);
        // 刷新会话列表
        await handleRefresh();
      } catch (error) {
        console.error('删除目录失败:', error);
      }
    },
    [removeMonitoredDirectory, handleRefresh]
  );

  // 切换监控目录状态
  const handleToggleDirectory = useCallback(
    async (id: number) => {
      try {
        await toggleMonitoredDirectory(id);
        // 刷新会话列表
        await handleRefresh();
      } catch (error) {
        console.error('切换目录状态失败:', error);
      }
    },
    [toggleMonitoredDirectory, handleRefresh]
  );

  return (
    <div className={cn('flex flex-col h-full bg-card', className)}>
      {/* 头部 */}
      <div className="flex items-center justify-between px-4 py-3 border-b bg-card">
        <h2 className="text-sm font-semibold text-foreground">项目</h2>
        <div className="flex items-center gap-2">
          {/* 添加目录按钮 */}
          <Button
            variant="ghost"
            size="sm"
            onClick={handleSelectDirectory}
            className="h-7 px-2"
          >
            <Plus className="h-4 w-4 mr-1" />
            添加目录
          </Button>
          {/* 对话框 */}
          <Dialog open={directoryDialogOpen} onOpenChange={setDirectoryDialogOpen}>
            <DialogContent className="sm:max-w-[425px]">
              <DialogHeader>
                <DialogTitle>添加监控目录</DialogTitle>
                <DialogDescription>
                  确认要添加此目录到监控列表吗？应用将扫描该目录下的所有会话文件。
                </DialogDescription>
              </DialogHeader>
              <div className="grid gap-4 py-4">
                <div className="grid grid-cols-4 items-center gap-4">
                  <Label htmlFor="directory-name" className="text-right">
                    名称
                  </Label>
                  <Input
                    id="directory-name"
                    value={newDirectoryName}
                    onChange={(e) => setNewDirectoryName(e.target.value)}
                    className="col-span-3"
                    placeholder="目录显示名称"
                  />
                </div>
                <div className="grid grid-cols-4 items-center gap-4">
                  <Label htmlFor="directory-path" className="text-right">
                    路径
                  </Label>
                  <Input
                    id="directory-path"
                    value={newDirectoryPath}
                    disabled
                    className="col-span-3"
                  />
                </div>
              </div>
              <DialogFooter>
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => setDirectoryDialogOpen(false)}
                >
                  取消
                </Button>
                <Button type="button" onClick={handleAddDirectory}>
                  添加
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>
          <Button
            variant="ghost"
            size="sm"
            onClick={handleRefresh}
            className="h-7 px-2"
          >
            刷新
          </Button>
        </div>
      </div>

      {/* 监控目录列表 */}
      {monitoredDirectories.length > 0 && (
        <div className="border-b px-4 py-2 bg-muted/50">
          <p className="text-xs text-muted-foreground mb-2">监控目录</p>
          <ul className="space-y-1">
            {monitoredDirectories.map((dir) => (
              <li
                key={dir.id}
                className={cn(
                  'flex items-center justify-between px-2 py-1 rounded text-xs',
                  !dir.is_active && 'opacity-50'
                )}
              >
                <span className="truncate flex-1" title={dir.path}>
                  {dir.name}
                </span>
                <div className="flex items-center gap-1 ml-2">
                  <button
                    onClick={() => handleToggleDirectory(dir.id!)}
                    className={cn(
                      'p-1 rounded hover:bg-accent',
                      dir.is_active ? 'text-green-500' : 'text-muted-foreground'
                    )}
                    title={dir.is_active ? '禁用' : '启用'}
                  >
                    <Power className="h-3 w-3" />
                  </button>
                  <button
                    onClick={() => handleRemoveDirectory(dir.id!)}
                    className="p-1 rounded hover:bg-accent text-red-500"
                    title="删除"
                  >
                    <Trash2 className="h-3 w-3" />
                  </button>
                </div>
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* 项目列表 */}
      <div className="flex-1 overflow-y-auto">
        {projects.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-40 text-sm text-muted-foreground">
            <p>暂无项目</p>
            <p className="text-xs mt-1">
              {monitoredDirectories.length === 0
                ? '点击"添加目录"添加监控目录'
                : '点击刷新扫描会话'}
            </p>
          </div>
        ) : (
          <ul className="space-y-1 p-2">
            {projects.map((project) => {
              const isCollapsed = collapsedProjects.has(project.projectPath);
              const isSelected = selectedProject === project.projectPath;

              return (
                <li key={project.projectPath}>
                  {/* 项目标题 */}
                  <button
                    onClick={() => toggleCollapse(project.projectPath)}
                    className={cn(
                      'w-full flex items-center gap-2 px-3 py-2 rounded-md text-sm transition-colors',
                      'hover:bg-accent hover:text-accent-foreground',
                      isSelected && 'bg-accent text-accent-foreground'
                    )}
                  >
                    {isCollapsed ? (
                      <ChevronRight className="h-4 w-4 shrink-0" />
                    ) : (
                      <ChevronDown className="h-4 w-4 shrink-0" />
                    )}
                    <Folder className="h-4 w-4 shrink-0 text-blue-500" />
                    <span className="flex-1 text-left truncate text-foreground">
                      {project.projectName}
                    </span>
                    <span className="text-xs text-muted-foreground">
                      {project.sessionCount}
                    </span>
                  </button>

                  {/* 会话列表 */}
                  {!isCollapsed && project.sessions.length > 0 && (
                    <ul className="ml-6 mt-1 space-y-0.5">
                      {project.sessions.map((session) => (
                        <li key={session.sessionId}>
                          <button
                            onClick={() => handleProjectSelect(project.projectPath)}
                            className={cn(
                              'w-full flex items-center gap-2 px-3 py-1.5 rounded-md text-xs transition-colors',
                              'hover:bg-accent hover:text-accent-foreground',
                              'truncate text-foreground'
                            )}
                            title={session.filePath}
                          >
                            <span className="truncate">
                              {session.sessionId.slice(0, 8)}...
                            </span>
                            {session.rating && (
                              <span className="text-yellow-500">★</span>
                            )}
                          </button>
                        </li>
                      ))}
                    </ul>
                  )}
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </div>
  );
}
