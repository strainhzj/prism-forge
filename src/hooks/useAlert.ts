/**
 * useAlert Hook
 *
 * 提供自动隐藏的 Alert 提示功能
 */

import { useState, useCallback, useEffect } from 'react';

export interface AlertState {
  show: boolean;
  type: 'success' | 'error' | 'warning' | 'info';
  message: string;
}

/**
 * Alert Hook
 *
 * @example
 * const { alert, showAlert, hideAlert } = useAlert();
 *
 * // 显示成功提示
 * showAlert('success', '操作成功');
 *
 * // 显示错误提示
 * showAlert('error', '操作失败');
 *
 * // 在组件中使用
 * {alert.show && (
 *   <div className={`alert alert-${alert.type}`}>
 *     {alert.message}
 *   </div>
 * )}
 */
export function useAlert(autoHideDelay: number = 3000) {
  const [alert, setAlert] = useState<AlertState>({
    show: false,
    type: 'success',
    message: '',
  });

  /**
   * 显示 Alert
   */
  const showAlert = useCallback((type: 'success' | 'error' | 'warning' | 'info', message: string) => {
    setAlert({ show: true, type, message });
  }, []);

  /**
   * 隐藏 Alert
   */
  const hideAlert = useCallback(() => {
    setAlert(prev => ({ ...prev, show: false }));
  }, []);

  /**
   * 自动隐藏 Alert（带清理）
   */
  useEffect(() => {
    if (!alert.show) return;

    const timer = setTimeout(() => {
      hideAlert();
    }, autoHideDelay);

    return () => clearTimeout(timer);
  }, [alert.show, autoHideDelay, hideAlert]);

  return {
    alert,
    showAlert,
    hideAlert,
  };
}
