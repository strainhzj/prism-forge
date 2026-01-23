/**
 * PromptLab 页面
 *
 * 提示词实验室 - 提示词历史记录管理
 * Cherry Studio 主题风格
 */

import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { Home } from 'lucide-react';
import { SideNav } from '@/components/navigation';
import { PromptHistory } from '@/components/prompt/PromptHistory';
import { ThemeToggle } from '@/components/ThemeToggle';
import './PromptLab.css';

/**
 * PromptLab 页面
 */
export function PromptLab() {
  const navigate = useNavigate();
  const { t } = useTranslation('promptLab');

  /**
   * 返回首页
   */
  const handleBackToHome = () => {
    navigate('/');
  };

  return (
    <div className="prompt-lab-page flex h-screen" style={{ backgroundColor: 'var(--color-bg-primary)' }}>
      {/* 左侧导航栏 */}
      <aside className="w-[256px] shrink-0 border-r" style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}>
        {/* Logo/标题 */}
        <div className="px-4 py-4 border-b" style={{ borderColor: 'var(--color-border-light)' }}>
          <h1 className="text-lg font-bold" style={{ color: 'var(--color-accent-warm)' }}>PrismForge</h1>
        </div>

        {/* 导航菜单 */}
        <SideNav />
      </aside>

      {/* 主内容区域 */}
      <main className="flex-1 flex flex-col min-w-0">
        {/* 顶部导航栏 */}
        <div className="flex items-center gap-4 px-6 py-4 border-b" style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}>
          <div className="prompt-lab-back">
            <button className="back-btn" onClick={handleBackToHome}>
              <Home size={16} />
              <span>{t('buttons.backToHome', { ns: 'sessions' })}</span>
            </button>
          </div>
          <div className="flex-1">
            <h1 className="text-xl font-bold" style={{ color: 'var(--color-text-primary)' }}>{t('title')}</h1>
            <p className="text-sm" style={{ color: 'var(--color-text-secondary)' }}>
              {t('description')}
            </p>
          </div>
          <ThemeToggle />
        </div>

        {/* 历史记录内容 */}
        <div className="flex-1 overflow-auto">
          <PromptHistory />
        </div>
      </main>
    </div>
  );
}
