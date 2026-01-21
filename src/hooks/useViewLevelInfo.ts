/**
 * useViewLevelInfo Hook
 *
 * 提供国际化的视图等级信息
 */

import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { ViewLevel, ViewLevelInfo, VIEW_LEVEL_BASE_INFO, AVAILABLE_VIEW_LEVELS } from '@/types/viewLevel';

/**
 * 获取单个视图等级的国际化信息
 * @param viewLevel - 视图等级
 * @returns 国际化的视图等级信息
 */
export function useViewLevelInfo(viewLevel: ViewLevel): ViewLevelInfo {
  const { t } = useTranslation('sessions');

  return useMemo(() => {
    const baseInfo = VIEW_LEVEL_BASE_INFO[viewLevel];
    return {
      value: baseInfo.value,
      displayName: t(`viewLevel.levels.${baseInfo.labelKey}.label`),
      description: t(`viewLevel.levels.${baseInfo.descriptionKey}.description`),
      icon: baseInfo.icon,
    };
  }, [viewLevel, t]);
}

/**
 * 获取所有视图等级的国际化信息
 * @returns 国际化的视图等级信息数组
 */
export function useViewLevelOptions(): ViewLevelInfo[] {
  const { t } = useTranslation('sessions');

  return useMemo(() => {
    return AVAILABLE_VIEW_LEVELS.map((level) => {
      const baseInfo = VIEW_LEVEL_BASE_INFO[level];
      return {
        value: baseInfo.value,
        displayName: t(`viewLevel.levels.${baseInfo.labelKey}.label`),
        description: t(`viewLevel.levels.${baseInfo.descriptionKey}.description`),
        icon: baseInfo.icon,
      };
    });
  }, [t]);
}
