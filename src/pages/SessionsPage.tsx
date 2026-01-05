/**
 * SessionsPage 组件
 *
 * 会话管理页面 - 集成项目侧边栏和会话列表
 */

import { useState, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { ArrowLeft } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { ThemeToggle } from '@/components/ThemeToggle';
import { ProjectSidebar } from '@/components/ProjectSidebar';
import { SessionList } from '@/components/SessionList';
import type { Session } from '@/stores/useSessionStore';

export interface SessionsPageProps {
  /**
   * 自定义类名
   */
  className?: string;
}

/**
 * SessionsPage 组件
 *
 * @example
 * <SessionsPage />
 */
export function SessionsPage({ className }: SessionsPageProps) {
  const navigate = useNavigate();
  const [selectedProject, setSelectedProject] = useState<string | undefined>();

  // 返回主页
  const handleBack = useCallback(() => {
    navigate('/');
  }, [navigate]);

  // 处理项目选择
  const handleProjectSelect = useCallback((projectPath: string) => {
    setSelectedProject(projectPath);
  }, []);

  // 处理会话点击
  const handleSessionClick = useCallback(
    (session: Session) => {
      // TODO: 跳转到会话详情页或打开会话
      console.log('Session clicked:', session);
    },
    []
  );

  return (
    <div className={cn('flex flex-col h-screen bg-background', className)}>
      {/* 顶部导航栏 */}
      <div className="flex items-center gap-4 px-6 py-4 border-b bg-background">
        <Button
          variant="ghost"
          size="icon"
          onClick={handleBack}
          className="shrink-0"
        >
          <ArrowLeft className="h-5 w-5" />
        </Button>
        <div className="flex-1">
          <h1 className="text-xl font-bold text-foreground">会话管理</h1>
          <p className="text-sm text-muted-foreground">
            浏览和管理 Claude Code 会话历史
          </p>
        </div>
        <ThemeToggle />
      </div>

      {/* 主内容区域 */}
      <div className="flex-1 flex overflow-hidden bg-background">
        {/* 左侧：项目侧边栏 */}
        <div className="w-64 border-r shrink-0 bg-card">
          <ProjectSidebar
            onProjectSelect={handleProjectSelect}
            selectedProject={selectedProject}
          />
        </div>

        {/* 右侧：会话列表 */}
        <div className="flex-1 min-w-0 bg-background">
          <SessionList onSessionClick={handleSessionClick} />
        </div>
      </div>
    </div>
  );
}
