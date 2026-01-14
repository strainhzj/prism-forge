/**
 * NavItem 组件
 *
 * 左侧导航栏的单个导航项
 */

import { ReactNode } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { cn } from '@/lib/utils';

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[NavItem] ${action}`, ...args);
  }
}

export interface NavItemProps {
  /**
   * 导航路径
   */
  to: string;
  /**
   * 图标
   */
  icon: ReactNode;
  /**
   * 标签文本
   */
  label: string;
  /**
   * 自定义类名
   */
  className?: string;
}

/**
 * NavItem 组件
 *
 * @example
 * <NavItem to="/sessions" icon={<Folder />} label="会话历史" />
 */
export function NavItem({ to, icon, label, className }: NavItemProps) {
  const location = useLocation();
  const navigate = useNavigate();

  const isActive = location.pathname === to;

  const handleClick = () => {
    debugLog('navigate', to);
    navigate(to);
  };

  return (
    <button
      onClick={handleClick}
      className={cn(
        'w-full flex items-center gap-3 px-4 py-3 rounded-lg transition-all text-left',
        'hover:scale-[1.02] active:scale-[0.98]',
        isActive
          ? 'bg-[var(--color-app-secondary)] shadow-sm'
          : 'hover:bg-[var(--color-app-secondary)] opacity-80 hover:opacity-100',
        className
      )}
      style={{
        backgroundColor: isActive ? 'rgba(245, 158, 11, 0.1)' : undefined,
        border: isActive ? '1px solid rgba(245, 158, 11, 0.3)' : undefined,
      }}
      title={label}
    >
      <span className="shrink-0" style={{ color: isActive ? 'var(--color-accent-warm)' : 'var(--color-text-secondary)' }}>
        {icon}
      </span>
      <span
        className={cn(
          'text-sm font-medium',
          isActive ? 'text-[var(--color-accent-warm)]' : 'text-[var(--color-text-secondary)]'
        )}
      >
        {label}
      </span>
    </button>
  );
}
