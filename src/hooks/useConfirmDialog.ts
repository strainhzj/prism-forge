/**
 * useConfirmDialog Hook
 *
 * 提供确认对话框功能，用于删除、重置等需要用户确认的操作
 */

import { useState, useCallback } from 'react';

export type DialogType = 'delete' | 'reset' | 'confirm';

export interface ConfirmDialogState {
  show: boolean;
  type: DialogType;
  title: string;
  message: string;
  onConfirm: () => void | Promise<void>;
}

/**
 * 确认对话框 Hook
 *
 * @example
 * const { confirmDialog, showConfirm } = useConfirmDialog();
 *
 * <AlertDialog open={confirmDialog.show} onOpenChange={(open) => !open && handleCancel()}>
 *   <AlertDialogContent>
 *     <AlertDialogHeader>
 *       <AlertDialogTitle>{confirmDialog.title}</AlertDialogTitle>
 *       <AlertDialogDescription>{confirmDialog.message}</AlertDialogDescription>
 *     </AlertDialogHeader>
 *     <AlertDialogFooter>
 *       <Button onClick={handleCancel}>取消</Button>
 *       <Button onClick={handleConfirm}>确认</Button>
 *     </AlertDialogFooter>
 *   </AlertDialogContent>
 * </AlertDialog>
 */
export function useConfirmDialog() {
  const [confirmDialog, setConfirmDialog] = useState<ConfirmDialogState>({
    show: false,
    type: 'confirm',
    title: '',
    message: '',
    onConfirm: () => {},
  });

  /**
   * 显示确认对话框
   */
  const showConfirm = useCallback((
    config: {
      type?: DialogType;
      title: string;
      message: string;
      onConfirm: () => void | Promise<void>;
    }
  ) => {
    setConfirmDialog({
      show: true,
      type: config.type || 'confirm',
      title: config.title,
      message: config.message,
      onConfirm: config.onConfirm,
    });
  }, []);

  /**
   * 隐藏确认对话框
   */
  const hideConfirm = useCallback(() => {
    setConfirmDialog(prev => ({ ...prev, show: false }));
  }, []);

  /**
   * 处理取消操作
   */
  const handleCancel = useCallback(() => {
    hideConfirm();
  }, [hideConfirm]);

  /**
   * 处理确认操作
   */
  const handleConfirm = useCallback(async () => {
    try {
      await confirmDialog.onConfirm();
    } catch (error) {
      console.error('确认操作失败:', error);
    } finally {
      hideConfirm();
    }
  }, [confirmDialog.onConfirm, hideConfirm]);

  return {
    confirmDialog,
    showConfirm,
    hideConfirm,
    handleConfirm,
    handleCancel,
  };
}
