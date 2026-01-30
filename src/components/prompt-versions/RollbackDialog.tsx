import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';

interface RollbackDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onConfirm: (strategy: 'soft' | 'hard', comment?: string) => void;
  loading?: boolean;
}

/**
 * 回滚确认对话框组件
 *
 * 支持软回滚和硬回滚两种策略
 */
export function RollbackDialog({ open, onOpenChange, onConfirm, loading }: RollbackDialogProps) {
  const { t } = useTranslation('promptVersions');
  const [strategy, setStrategy] = useState<'soft' | 'hard'>('soft');
  const [comment, setComment] = useState('');

  const handleConfirm = () => {
    onConfirm(strategy, comment || undefined);
    // 重置表单
    setComment('');
  };

  const handleCancel = () => {
    onOpenChange(false);
    // 重置表单
    setComment('');
  };

  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t('confirmRollback')}</AlertDialogTitle>
          <AlertDialogDescription>{t('rollbackWarning')}</AlertDialogDescription>
        </AlertDialogHeader>

        {/* Rollback Strategy Selection */}
        <div className="mb-4">
          <label className="text-sm font-medium mb-2 block" style={{ color: 'var(--color-text-primary)' }}>
            {t('rollbackStrategy')}
          </label>
          <div className="space-y-2">
            <label
              className={`flex items-center gap-3 p-3 rounded-lg border-2 cursor-pointer transition-all hover:shadow-sm ${
                strategy === 'soft' ? 'shadow-sm' : ''
              }`}
              style={{
                backgroundColor: strategy === 'soft'
                  ? 'rgba(76, 175, 80, 0.08)'
                  : 'var(--color-bg-primary)',
                borderColor: strategy === 'soft'
                  ? 'var(--color-accent-green)'
                  : 'var(--color-border-light)',
              }}
              onClick={() => setStrategy('soft')}
            >
              <input
                type="radio"
                name="rollback-strategy"
                value="soft"
                checked={strategy === 'soft'}
                onChange={() => setStrategy('soft')}
                className="cursor-pointer accent-green-600"
                style={{ accentColor: 'var(--color-accent-green)' }}
              />
              <div className="flex-1">
                <div
                  className="text-sm font-medium"
                  style={{
                    color: strategy === 'soft'
                      ? 'var(--color-accent-green)'
                      : 'var(--color-text-primary)'
                  }}
                >
                  {t('softRollback')}
                </div>
                <div className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
                  {t('softRollbackDesc')}
                </div>
              </div>
            </label>
            <label
              className={`flex items-center gap-3 p-3 rounded-lg border-2 cursor-pointer transition-all hover:shadow-sm ${
                strategy === 'hard' ? 'shadow-sm' : ''
              }`}
              style={{
                backgroundColor: strategy === 'hard'
                  ? 'rgba(245, 158, 11, 0.08)'
                  : 'var(--color-bg-primary)',
                borderColor: strategy === 'hard'
                  ? 'var(--color-accent-warm)'
                  : 'var(--color-border-light)',
              }}
              onClick={() => setStrategy('hard')}
            >
              <input
                type="radio"
                name="rollback-strategy"
                value="hard"
                checked={strategy === 'hard'}
                onChange={() => setStrategy('hard')}
                className="cursor-pointer"
                style={{ accentColor: 'var(--color-accent-warm)' }}
              />
              <div className="flex-1">
                <div
                  className="text-sm font-medium"
                  style={{
                    color: strategy === 'hard'
                      ? 'var(--color-accent-warm)'
                      : 'var(--color-text-primary)'
                  }}
                >
                  {t('hardRollback')}
                </div>
                <div className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
                  {t('hardRollbackDesc')}
                </div>
              </div>
            </label>
          </div>
        </div>

        {/* Comment Input */}
        <div className="mb-5">
          <label className="text-sm font-medium mb-2 block" style={{ color: 'var(--color-text-primary)' }}>
            {t('comment')}
          </label>
          <Input
            type="text"
            value={comment}
            onChange={(e) => setComment(e.target.value)}
            placeholder={t('commentPlaceholder')}
            className="w-full"
            style={{
              backgroundColor: 'var(--color-bg-card)',
              borderColor: 'var(--color-border-light)',
              color: 'var(--color-text-primary)',
            }}
          />
        </div>

        {/* Actions */}
        <AlertDialogFooter>
          <Button
            variant="ghost"
            onClick={handleCancel}
            disabled={loading}
            className="border-2 transition-all hover:scale-[1.02]"
            style={{
              borderColor: 'var(--color-border-light)',
              color: 'var(--color-text-secondary)',
              fontWeight: 500,
            }}
          >
            {t('cancel')}
          </Button>
          <Button
            variant="destructive"
            onClick={handleConfirm}
            disabled={loading}
            className="border-2 transition-all hover:scale-[1.02] hover:shadow-md"
            style={{
              backgroundColor: 'var(--color-accent-red)',
              borderColor: 'var(--color-accent-red)',
              color: 'white',
              fontWeight: 600,
            }}
          >
            {loading ? '处理中...' : t('confirmRollbackBtn')}
          </Button>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
