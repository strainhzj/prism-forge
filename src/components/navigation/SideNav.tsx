/**
 * SideNav 组件
 *
 * 左侧导航栏，包含主要页面导航
 */

import { Home, Folder, Settings } from 'lucide-react';
import { NavItem } from './NavItem';
import { cn } from '@/lib/utils';

export interface SideNavProps {
  /**
   * 自定义类名
   */
  className?: string;
}

/**
 * 导航菜单配置
 */
const NAV_MENU_ITEMS = [
  { to: '/', icon: <Home className="h-5 w-5" />, label: '首页' },
  { to: '/sessions', icon: <Folder className="h-5 w-5" />, label: '会话历史' },
  { to: '/settings', icon: <Settings className="h-5 w-5" />, label: 'API 设置' },
] as const;

/**
 * SideNav 组件
 *
 * @example
 * <SideNav />
 */
export function SideNav({ className }: SideNavProps) {
  return (
    <nav
      className={cn('flex flex-col gap-2 px-3 py-4', className)}
      style={{ backgroundColor: 'var(--color-bg-card)' }}
    >
      {/* 导航菜单列表 */}
      <ul className="space-y-1">
        {NAV_MENU_ITEMS.map((item) => (
          <li key={item.to}>
            <NavItem to={item.to} icon={item.icon} label={item.label} />
          </li>
        ))}
      </ul>
    </nav>
  );
}
