import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import i18n from '@/i18n';

// ==================== 类型定义 ====================

export type Language = 'zh' | 'en';

interface LanguageState {
  language: Language;
  setLanguage: (language: Language) => void;
  toggleLanguage: () => void;
}

// ==================== 调试模式 ====================

const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[LanguageStore] ${action}`, ...args);
  }
}

// ==================== Store 定义 ====================

export const useLanguageStore = create<LanguageState>()(
  persist(
    (set, get) => ({
      language: 'zh', // 默认中文

      // 设置语言
      setLanguage: (language: Language) => {
        debugLog('setLanguage', `Switching to language: ${language}`);
        set({ language });
        i18n.changeLanguage(language);
      },

      // 切换语言（zh ↔ en）
      toggleLanguage: () => {
        const currentLanguage = get().language;
        const newLanguage: Language = currentLanguage === 'zh' ? 'en' : 'zh';
        debugLog('toggleLanguage', `Toggling from ${currentLanguage} to ${newLanguage}`);
        get().setLanguage(newLanguage);
      },
    }),
    {
      name: 'prism-forge-language', // localStorage key
      version: 1, // 版本号（用于迁移）
    }
  )
);

// ==================== 便捷 Hooks ====================

/**
 * 获取当前语言
 */
export const useCurrentLanguage = () => useLanguageStore((state) => state.language);

/**
 * 获取语言切换 actions
 */
export const useLanguageActions = () => useLanguageStore((state) => ({
  setLanguage: state.setLanguage,
  toggleLanguage: state.toggleLanguage,
}));
