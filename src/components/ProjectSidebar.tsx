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
   * 目录选择回调
   */
  onDirectorySelect?: (directoryPath: string, directoryName: string) => void;
  /**
   * 当前选中的目录路径
   */
  selectedDirectory?: string;
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
 *   onDirectorySelect={(path, name) => console.log(path, name)}
 *   selectedDirectory="/path/to/directory"
 * />
 */
export function ProjectSidebar({
  onDirectorySelect,
  selectedDirectory,
  className,
}: ProjectSidebarProps) {
  const monitoredDirectories = useMonitoredDirectories();
  const { setActiveSessions } = useSessionActions();
  const {
    fetchMonitoredDirectories,
    addMonitoredDirectory,
    removeMonitoredDirectory,
    toggleMonitoredDirectory,
  } = useMonitoredDirectoryActions();

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

  // 选择目录
  const handleDirectoryClick = useCallback(
    (directory: { path: string; name: string; is_active: boolean }) => {
      if (directory.is_active) {
        onDirectorySelect?.(directory.path, directory.name);
      }
    },
    [onDirectorySelect]
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
      <div className="flex-1 overflow-y-auto">
        {monitoredDirectories.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-sm text-muted-foreground p-4">
            <p className="text-foreground font-medium">暂无监控目录</p>
            <p className="text-xs mt-2 text-center">
              点击"添加目录"选择要监控的项目目录
            </p>
          </div>
        ) : (
          <ul className="space-y-1 p-2">
            {monitoredDirectories.map((dir) => {
              const isSelected = selectedDirectory === dir.path;

              return (
                <li key={dir.id}>
                  {/* 目录项 */}
                  <button
                    onClick={() => handleDirectoryClick(dir)}
                    className={cn(
                      'w-full flex items-center gap-2 px-3 py-2.5 rounded-md text-sm transition-colors',
                      'hover:bg-accent hover:text-accent-foreground',
                      isSelected && 'bg-accent text-accent-foreground font-medium',
                      !dir.is_active && 'opacity-50 cursor-not-allowed'
                    )}
                    title={dir.path}
                    disabled={!dir.is_active}
                  >
                    <Folder className="h-4 w-4 shrink-0 text-blue-500" />
                    <span className="flex-1 text-left truncate text-foreground">
                      {dir.name}
                    </span>
                    {!dir.is_active && (
                      <span className="text-xs text-muted-foreground">已禁用</span>
                    )}
                  </button>

                  {/* 操作按钮（悬停时显示） */}
                  <div className="flex items-center gap-1 ml-6 mt-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button
                      onClick={() => handleToggleDirectory(dir.id!)}
                      className={cn(
                        'p-1 rounded hover:bg-accent text-xs',
                        dir.is_active ? 'text-green-500' : 'text-muted-foreground'
                      )}
                      title={dir.is_active ? '禁用' : '启用'}
                    >
                      <Power className="h-3 w-3" />
                    </button>
                    <button
                      onClick={() => handleRemoveDirectory(dir.id!)}
                      className="p-1 rounded hover:bg-accent text-red-500 text-xs"
                      title="删除"
                    >
                      <Trash2 className="h-3 w-3" />
                    </button>
                  </div>
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </div>
  );
}
