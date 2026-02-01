import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

// 导入翻译文件
import zhCommon from './locales/zh/common.json';
import zhIndex from './locales/zh/index.json';
import zhNavigation from './locales/zh/navigation.json';
import zhSettings from './locales/zh/settings.json';
import zhSessions from './locales/zh/sessions.json';
import zhPromptLab from './locales/zh/promptLab.json';
import zhPrompts from './locales/zh/prompts.json';
import zhPromptVersions from './locales/zh/promptVersions.json';
import enCommon from './locales/en/common.json';
import enIndex from './locales/en/index.json';
import enNavigation from './locales/en/navigation.json';
import enSettings from './locales/en/settings.json';
import enSessions from './locales/en/sessions.json';
import enPromptLab from './locales/en/promptLab.json';
import enPrompts from './locales/en/prompts.json';
import enPromptVersions from './locales/en/promptVersions.json';

// ==================== i18n 配置 ====================

const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[i18n] ${action}`, ...args);
  }
}

i18n
  .use(initReactI18next) // 绑定 react-i18next
  .init({
    resources: {
      zh: {
        common: zhCommon,
        index: zhIndex,
        navigation: zhNavigation,
        settings: zhSettings,
        sessions: zhSessions,
        promptLab: zhPromptLab,
        prompts: zhPrompts,
        promptVersions: zhPromptVersions,
      },
      en: {
        common: enCommon,
        index: enIndex,
        navigation: enNavigation,
        settings: enSettings,
        sessions: enSessions,
        promptLab: enPromptLab,
        prompts: enPrompts,
        promptVersions: enPromptVersions,
      },
    },
    lng: 'zh', // 默认语言（中文）
    fallbackLng: 'zh', // 回退语言
    defaultNS: 'common', // 默认命名空间
    ns: ['common', 'index', 'navigation', 'settings', 'sessions', 'promptLab', 'prompts', 'promptVersions'], // 可用命名空间
    debug: DEBUG, // 开发模式显示调试信息
    interpolation: {
      escapeValue: false, // React 已经做了 XSS 防护
    },
    react: {
      useSuspense: false, // 禁用 Suspense（避免加载延迟）
    },
  });

// 监听语言变化（调试用）
if (DEBUG) {
  i18n.on('languageChanged', (lng) => {
    debugLog('languageChanged', `Language switched to: ${lng}`);
  });
}

export default i18n;
