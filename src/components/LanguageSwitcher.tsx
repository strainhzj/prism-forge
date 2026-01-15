/**
 * LanguageSwitcher 组件
 *
 * 中英文切换按钮，支持中文/英文两种语言
 */

import { Languages } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { useCurrentLanguage, useLanguageStore } from '@/stores/useLanguageStore';

export interface LanguageSwitcherProps {
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
 * LanguageSwitcher 组件
 *
 * @example
 * <LanguageSwitcher />
 * <LanguageSwitcher showLabel />
 * <LanguageSwitcher className="fixed top-4 right-4" />
 */
export function LanguageSwitcher({
  showIcon = true,
  showLabel = false,
  className,
}: LanguageSwitcherProps) {
  const language = useCurrentLanguage();
  const toggleLanguage = useLanguageStore((state) => state.toggleLanguage);

  // 获取当前标签
  const getLabel = () => {
    return language === 'zh' ? '中文' : 'English';
  };

  // 获取提示文本
  const getTitle = () => {
    const currentLang = language === 'zh' ? '中文' : 'English';
    const nextLang = language === 'zh' ? 'English' : '中文';
    return `当前：${currentLang}（点击切换到 ${nextLang}）`;
  };

  return (
    <Button
      variant="ghost"
      size="sm"
      onClick={toggleLanguage}
      className={cn('gap-2', className)}
      title={getTitle()}
    >
      {showIcon && <Languages className="h-4 w-4" />}
      {showLabel && (
        <span className="text-sm">{getLabel()}</span>
      )}
    </Button>
  );
}

/**
 * 简化版语言切换器（显示两种语言）
 */
export function LanguageSelector() {
  const language = useCurrentLanguage();
  const setLanguage = useLanguageStore((state) => state.setLanguage);

  const languages = [
    { value: 'zh', label: '中文', icon: '🇨🇳' },
    { value: 'en', label: 'English', icon: '🇺🇸' },
  ] as const;

  return (
    <div className="flex items-center gap-1 p-1 bg-muted rounded-lg">
      {languages.map(({ value, label, icon }) => (
        <button
          key={value}
          onClick={() => setLanguage(value)}
          className={cn(
            'flex items-center gap-2 px-3 py-1.5 rounded-md text-sm transition-colors',
            'hover:bg-background',
            language === value && 'bg-background shadow-sm'
          )}
          title={label}
        >
          <span>{icon}</span>
          <span>{label}</span>
        </button>
      ))}
    </div>
  );
}
