/**
 * SideNav 组件
 *
 * 左侧导航栏，包含主要页面导航
 */

import { Home, Folder, Settings, FlaskConical } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { NavItem } from './NavItem';
import { cn } from '@/lib/utils';

export interface SideNavProps {
  /**
   * 自定义类名
   */
  className?: string;
}

/**
 * 导航菜单配置（使用翻译键）
 */
const NAV_MENU_ITEMS = [
  { to: '/', icon: <Home className="h-5 w-5" />, labelKey: 'nav.home' },
  { to: '/sessions', icon: <Folder className="h-5 w-5" />, labelKey: 'nav.sessions' },
  { to: '/prompt-lab', icon: <FlaskConical className="h-5 w-5" />, labelKey: 'nav.promptLab' },
  { to: '/settings', icon: <Settings className="h-5 w-5" />, labelKey: 'nav.settings' },
] as const;

/**
 * SideNav 组件
 *
 * @example
 * <SideNav />
 */
export function SideNav({ className }: SideNavProps) {
  const { t } = useTranslation('navigation');

  return (
    <nav
      className={cn('flex flex-col gap-2 px-3 py-4', className)}
      style={{ backgroundColor: 'var(--color-bg-card)' }}
    >
      {/* 导航菜单列表 */}
      <ul className="space-y-1">
        {NAV_MENU_ITEMS.map((item) => (
          <li key={item.to}>
            <NavItem to={item.to} icon={item.icon} label={t(item.labelKey)} />
          </li>
        ))}
      </ul>
    </nav>
  );
}
