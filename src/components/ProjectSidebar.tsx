/**
 * ProjectSidebar 组件
 *
 * 显示项目分组列表，支持折叠/展开
 */

import { useState, useCallback } from 'react';
import { ChevronDown, ChevronRight, Folder } from 'lucide-react';
import { cn } from '@/lib/utils';
import { useProjectGroups, useSessionActions } from '@/stores/useSessionStore';
import { Button } from '@/components/ui/button';

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
  const { setActiveSessions } = useSessionActions();

  // 折叠状态
  const [collapsedProjects, setCollapsedProjects] = useState<Set<string>>(new Set());

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

  return (
    <div className={cn('flex flex-col h-full', className)}>
      {/* 头部 */}
      <div className="flex items-center justify-between px-4 py-3 border-b">
        <h2 className="text-sm font-semibold">项目</h2>
        <Button
          variant="ghost"
          size="sm"
          onClick={handleRefresh}
          className="h-7 px-2"
        >
          刷新
        </Button>
      </div>

      {/* 项目列表 */}
      <div className="flex-1 overflow-y-auto">
        {projects.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-40 text-sm text-muted-foreground">
            <p>暂无项目</p>
            <p className="text-xs mt-1">点击刷新扫描会话</p>
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
                    <span className="flex-1 text-left truncate">
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
                              'truncate'
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
