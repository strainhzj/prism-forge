/**
 * SessionsPage 组件
 *
 * 会话管理页面 - 支持两种视图状态：
 * 1. 目录列表视图：显示选中目录的会话文件列表
 * 2. 会话内容视图：显示选中会话的详细内容
 */

import { useState, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { Home } from 'lucide-react';
import { cn } from '@/lib/utils';
import { ThemeToggle } from '@/components/ThemeToggle';
import { ProjectSidebar } from '@/components/ProjectSidebar';
import { SessionFileList, type SessionFileInfo } from '@/components/SessionFileList';
import { SessionContentView } from '@/components/SessionContentView';
import './SessionsPage.css';

// ==================== 类型定义 ====================

/**
 * 视图状态
 */
type ViewState =
  | 'directory_list'  // 显示目录列表
  | 'session_content'; // 显示会话内容

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
  const { t } = useTranslation('sessions');

  // 视图状态管理
  const [viewState, setViewState] = useState<ViewState>('directory_list');
  const [selectedDirectory, setSelectedDirectory] = useState<{
    path: string;
    name: string;
  } | null>(null);
  const [selectedSession, setSelectedSession] = useState<SessionFileInfo | null>(null);

  // 返回主页
  const handleBack = useCallback(() => {
    navigate('/');
  }, [navigate]);

  // 处理目录选择
  const handleDirectorySelect = useCallback((path: string, name: string) => {
    setSelectedDirectory({ path, name });
    setViewState('directory_list');
    setSelectedSession(null); // 清除之前选择的会话
  }, []);

  // 处理会话选择
  const handleSessionClick = useCallback((sessionInfo: SessionFileInfo) => {
    setSelectedSession(sessionInfo);
    setViewState('session_content');
  }, []);

  // 返回到目录列表
  const handleBackToDirectoryList = useCallback(() => {
    setViewState('directory_list');
    setSelectedSession(null);
  }, []);

  return (
    <div className={cn('sessions-page flex flex-col h-screen', className)} style={{ backgroundColor: 'var(--color-bg-primary)' }}>
      {/* 顶部导航栏 */}
      <div className="flex items-center gap-4 px-6 py-4 border-b" style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}>
        <div className="sessions-page-back">
          <button className="back-btn" onClick={handleBack}>
            <Home size={16} />
            <span>{t('buttons.backToHome')}</span>
          </button>
        </div>
        <div className="flex-1">
          <h1 className="text-xl font-bold" style={{ color: 'var(--color-text-primary)' }}>{t('title')}</h1>
          <p className="text-sm" style={{ color: 'var(--color-text-secondary)' }}>
            {t('subtitle')}
          </p>
        </div>
        <ThemeToggle />
      </div>

      {/* 主内容区域 */}
      <div className="flex-1 flex overflow-hidden" style={{ backgroundColor: 'var(--color-bg-primary)' }}>
        {/* 左侧：目录侧边栏 (356px = 256px + 100px) */}
        <div className="w-[356px] border-r shrink-0" style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}>
          <ProjectSidebar
            onDirectorySelect={handleDirectorySelect}
            selectedDirectory={selectedDirectory?.path}
          />
        </div>

        {/* 右侧：内容区域（根据状态切换） */}
        <div className="flex-1 min-w-0" style={{ backgroundColor: 'var(--color-bg-primary)' }}>
          {viewState === 'directory_list' && selectedDirectory ? (
            // 目录列表视图
            <SessionFileList
              directoryPath={selectedDirectory.path}
              directoryName={selectedDirectory.name}
              onSessionClick={handleSessionClick}
            />
          ) : viewState === 'session_content' && selectedSession ? (
            // 会话内容视图
            <SessionContentView
              sessionInfo={selectedSession}
              onBack={handleBackToDirectoryList}
            />
          ) : (
            // 空状态（未选择目录）
            <div className="flex flex-col items-center justify-center h-full text-center p-4">
              <p className="font-medium text-lg" style={{ color: 'var(--color-text-primary)' }}>{t('emptyState.title')}</p>
              <p className="text-sm mt-2" style={{ color: 'var(--color-text-secondary)' }}>
                {t('emptyState.description')}
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
