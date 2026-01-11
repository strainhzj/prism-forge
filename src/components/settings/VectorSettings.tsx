/**
 * VectorSettings 组件
 *
 * 向量搜索功能设置界面
 */

import { useState, useCallback, useEffect } from 'react';
import { Search, Database, Zap, AlertCircle } from 'lucide-react';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Checkbox } from '@/components/ui/checkbox';
import { Label } from '@/components/ui/label';
import { cn } from '@/lib/utils';
import {
  useVectorSettings,
  useUpdateVectorSettings,
  useSyncEmbeddings,
} from '@/hooks/useVectorSearch';

export interface VectorSettingsProps {
  /** 自定义类名 */
  className?: string;
}

/**
 * 向量设置组件
 */
export function VectorSettings({ className }: VectorSettingsProps) {
  const { data: settings, isLoading } = useVectorSettings();
  const updateSettings = useUpdateVectorSettings();
  const syncMutation = useSyncEmbeddings();

  // 本地状态（用于编辑）
  const [localSettings, setLocalSettings] = useState(settings);

  // 当远程设置更新时同步到本地
  useEffect(() => {
    setLocalSettings(settings);
  }, [settings]);

  const handleToggleVectorSearch = useCallback(
    async (enabled: boolean) => {
      if (!localSettings) return;

      const newSettings = {
        ...localSettings,
        vectorSearchEnabled: enabled,
      };

      setLocalSettings(newSettings);

      try {
        await updateSettings.mutateAsync(newSettings);
      } catch (err) {
        // 失败时回滚
        setLocalSettings(settings);
        console.error('更新设置失败:', err);
      }
    },
    [localSettings, settings, updateSettings]
  );

  const handleSyncEmbeddings = useCallback(async () => {
    try {
      const count = await syncMutation.mutateAsync();
      alert(`成功向量化 ${count} 个会话`);
    } catch (err) {
      console.error('同步失败:', err);
      alert('同步失败: ' + (err as Error).message);
    }
  }, [syncMutation]);

  if (isLoading) {
    return (
      <div className={cn('space-y-4', className)}>
        <Card className="p-6">
          <div className="flex items-center justify-center py-8">
            <div className="text-muted-foreground">加载中...</div>
          </div>
        </Card>
      </div>
    );
  }

  if (!localSettings) {
    return (
      <div className={cn('space-y-4', className)}>
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>
            无法加载向量设置。请确保数据库已正确初始化。
          </AlertDescription>
        </Alert>
      </div>
    );
  }

  const isSyncing = syncMutation.isPending;
  const isUpdating = updateSettings.isPending;

  return (
    <div className={cn('space-y-4', className)}>
      {/* 功能开关卡片 */}
      <Card className="p-6">
        <div className="space-y-6">
          {/* 标题 */}
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
              <Search className="h-5 w-5 text-primary" />
            </div>
            <div>
              <h3 className="text-lg font-semibold">语义搜索</h3>
              <p className="text-sm text-muted-foreground">
                使用 AI 向量嵌入实现智能会话搜索
              </p>
            </div>
          </div>

          {/* 功能开关 */}
          <div className="flex items-center justify-between rounded-lg border p-4">
            <div className="space-y-1">
              <div className="flex items-center gap-2">
                <Database className="h-4 w-4 text-muted-foreground" />
                <Label htmlFor="vector-search-enabled" className="font-medium">
                  启用向量搜索
                </Label>
              </div>
              <p className="text-sm text-muted-foreground">
                {localSettings.vectorSearchEnabled
                  ? '向量搜索已启用，将使用 AI 语义分析会话内容'
                  : '向量搜索已禁用，将使用传统关键词搜索'}
              </p>
            </div>
            <Checkbox
              id="vector-search-enabled"
              checked={localSettings.vectorSearchEnabled}
              onCheckedChange={handleToggleVectorSearch}
              disabled={isUpdating}
            />
          </div>

          {/* 配置信息 */}
          {localSettings.vectorSearchEnabled && (
            <div className="space-y-3 rounded-lg bg-muted/50 p-4">
              <h4 className="text-sm font-medium">当前配置</h4>
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <span className="text-muted-foreground">Embedding 提供商:</span>{' '}
                  <span className="font-medium">{localSettings.embeddingProvider}</span>
                </div>
                <div>
                  <span className="text-muted-foreground">模型:</span>{' '}
                  <span className="font-medium">{localSettings.embeddingModel}</span>
                </div>
                <div>
                  <span className="text-muted-foreground">批量大小:</span>{' '}
                  <span className="font-medium">{localSettings.embeddingBatchSize}</span>
                </div>
              </div>
            </div>
          )}

          {/* 同步按钮 */}
          {localSettings.vectorSearchEnabled && (
            <div className="flex items-center justify-between rounded-lg border p-4">
              <div className="space-y-1">
                <div className="flex items-center gap-2">
                  <Zap className="h-4 w-4 text-muted-foreground" />
                  <span className="font-medium">手动同步向量</span>
                </div>
                <p className="text-sm text-muted-foreground">
                  将未向量化的会话转换为 AI 向量（需要调用 API）
                </p>
              </div>
              <Button
                onClick={handleSyncEmbeddings}
                disabled={isSyncing}
                variant="outline"
                size="sm"
              >
                {isSyncing ? '同步中...' : '立即同步'}
              </Button>
            </div>
          )}

          {/* 提示信息 */}
          <Alert>
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>
              启用向量搜索需要配置 LLM 提供商 API Key。向量化会话将产生 API
              调用费用（约 $0.00002 / 1K tokens）。
            </AlertDescription>
          </Alert>
        </div>
      </Card>
    </div>
  );
}
