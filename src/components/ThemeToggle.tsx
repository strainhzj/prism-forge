/**
 * ThemeToggle 组件
 *
 * 主题切换按钮，支持浅色/深色/跟随系统三种模式
 */

import { useCallback } from 'react';
import { Moon, Sun, Monitor } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { useTheme, type Theme } from '@/contexts/ThemeContext';

export interface ThemeToggleProps {
  /**
   * 是否显示图标
   * @default true
   */
  showIcon?: boolean;
  /**
   * 是否显示标签
   * @default false
   */
  showLabel?: boolean;
  /**
   * 自定义类名
   */
  className?: string;
}

/**
 * ThemeToggle 组件
 *
 * @example
 * <ThemeToggle />
 * <ThemeToggle showLabel />
 * <ThemeToggle className="fixed top-4 right-4" />
 */
export function ThemeToggle({
  showIcon = true,
  showLabel = false,
  className,
}: ThemeToggleProps) {
  const { theme, setTheme, resolvedTheme } = useTheme();

  // 切换到下一个主题
  const handleCycleTheme = useCallback(() => {
    const themes: Theme[] = ['light', 'dark', 'system'];
    const currentIndex = themes.indexOf(theme);
    const nextIndex = (currentIndex + 1) % themes.length;
    setTheme(themes[nextIndex]);
  }, [theme, setTheme]);

  // 获取当前图标
  const getIcon = () => {
    if (theme === 'system') {
      return <Monitor className="h-4 w-4" />;
    }
    return resolvedTheme === 'dark' ? (
      <Moon className="h-4 w-4" />
    ) : (
      <Sun className="h-4 w-4" />
    );
  };

  // 获取当前标签
  const getLabel = () => {
    if (theme === 'system') {
      return '跟随系统';
    }
    return resolvedTheme === 'dark' ? '深色模式' : '浅色模式';
  };

  // 获取提示文本
  const getTitle = () => {
    if (theme === 'system') {
      return '当前：跟随系统（点击切换到浅色）';
    }
    if (resolvedTheme === 'dark') {
      return '当前：深色模式（点击切换到浅色）';
    }
    return '当前：浅色模式（点击切换到深色）';
  };

  return (
    <Button
      variant="ghost"
      size="sm"
      onClick={handleCycleTheme}
      className={cn('gap-2', className)}
      title={getTitle()}
    >
      {showIcon && getIcon()}
      {showLabel && (
        <span className="text-sm">{getLabel()}</span>
      )}
    </Button>
  );
}

/**
 * 简化版主题切换器（下拉菜单）
 */
export function ThemeSelector() {
  const { theme, setTheme, resolvedTheme } = useTheme();

  const themes: { value: Theme; label: string; icon: React.ReactNode }[] = [
    { value: 'light', label: '浅色', icon: <Sun className="h-4 w-4" /> },
    { value: 'dark', label: '深色', icon: <Moon className="h-4 w-4" /> },
    { value: 'system', label: '跟随系统', icon: <Monitor className="h-4 w-4" /> },
  ];

  return (
    <div className="flex items-center gap-1 p-1 bg-muted rounded-lg">
      {themes.map(({ value, label, icon }) => (
        <button
          key={value}
          onClick={() => setTheme(value)}
          className={cn(
            'flex items-center gap-2 px-3 py-1.5 rounded-md text-sm transition-colors',
            'hover:bg-background',
            (theme === value || (theme === 'system' && resolvedTheme === value)) &&
              'bg-background shadow-sm'
          )}
          title={label}
        >
          {icon}
          <span>{label}</span>
        </button>
      ))}
    </div>
  );
}
