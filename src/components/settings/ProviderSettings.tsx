/**
 * ProviderSettings 组件
 *
 * 使用 Shadcn UI 的提供商管理组件
 */

import { useState, useCallback, useMemo } from 'react';
import { Settings, Trash2, Zap, CheckCircle2, XCircle, AlertCircle } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card } from '@/components/ui/card';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Loading } from '@/components/ui/loading';
import {
  useProviderActions,
  useProviders,
  useProvidersLoading,
  useProvidersError,
  type ProviderResponse,
  type TestConnectionResult,
  ConnectionErrorType,
} from '@/stores/useSettingsStore';

export interface ProviderSettingsProps {
  /** 提供商选中回调 */
  onSelectProvider?: (provider: ProviderResponse) => void;
  /** 自定义类名 */
  className?: string;
}

/**
 * ProviderSettings 组件
 *
 * @example
 * <ProviderSettings
 *   onSelectProvider={(provider) => console.log('Selected', provider)}
 * />
 */
export function ProviderSettings({
  onSelectProvider,
  className,
}: ProviderSettingsProps) {
  const providers = useProviders();
  const loading = useProvidersLoading();
  const error = useProvidersError();
  const {
    setActiveProvider,
    deleteProvider,
    testProviderConnection,
    clearError,
  } = useProviderActions();

  // 测试状态
  const [testingIds, setTestingIds] = useState<Set<number>>(new Set());
  const [testResults, setTestResults] = useState<Map<number, TestConnectionResult>>(new Map());

  // 设置活跃提供商
  const handleSetActive = useCallback(
    async (provider: ProviderResponse) => {
      if (!provider.id) return;
      try {
        await setActiveProvider(provider.id);
      } catch (err) {
        console.error('设置活跃提供商失败:', err);
      }
    },
    [setActiveProvider]
  );

  // 删除提供商
  const handleDelete = useCallback(
    async (provider: ProviderResponse) => {
      if (!provider.id) return;

      const confirmed = window.confirm(
        `确定要删除提供商 "${provider.name}" 吗？\n\n` +
        `此操作将同时删除存储的 API Key，且不可恢复。`
      );

      if (!confirmed) return;

      try {
        await deleteProvider(provider.id!);
        // 清除测试结果
        setTestResults((prev) => {
          const next = new Map(prev);
          next.delete(provider.id!);
          return next;
        });
      } catch (err) {
        console.error('删除失败:', err);
      }
    },
    [deleteProvider]
  );

  // 测试连接
  const handleTestConnection = useCallback(
    async (provider: ProviderResponse) => {
      if (!provider.id) return;

      setTestingIds((prev) => new Set(prev).add(provider.id!));
      setTestResults((prev) => {
        const next = new Map(prev);
        next.delete(provider.id!);
        return next;
      });

      try {
        const result = await testProviderConnection(provider.id!);
        setTestResults((prev) => {
          const next = new Map(prev);
          next.set(provider.id!, result);
          return next;
        });
      } catch (err) {
        const errorResult: TestConnectionResult = {
          success: false,
          errorMessage: err instanceof Error ? err.message : String(err),
          errorType: ConnectionErrorType.UNKNOWN,
        };
        setTestResults((prev) => {
          const next = new Map(prev);
          next.set(provider.id!, errorResult);
          return next;
        });
      } finally {
        setTestingIds((prev) => {
          const next = new Set(prev);
          next.delete(provider.id!);
          return next;
        });
      }
    },
    [testProviderConnection]
  );

  // 获取测试结果状态
  const getTestStatus = useCallback(
    (providerId: number) => {
      const isTesting = testingIds.has(providerId);
      const result = testResults.get(providerId);

      if (isTesting) return 'testing';
      if (result?.success) return 'success';
      if (result) return 'error';
      return 'idle';
    },
    [testingIds, testResults]
  );

  // 统计信息
  const stats = useMemo(() => {
    return {
      total: providers.length,
      active: providers.filter((p) => p.isActive).length,
      withKey: providers.filter((p) => p.hasApiKey).length,
    };
  }, [providers]);

  return (
    <div className={cn('space-y-4', className)}>
      {/* 统计信息 */}
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-2">
          <Settings className="h-5 w-5 text-muted-foreground" />
          <h3 className="text-lg font-semibold">API 提供商</h3>
        </div>
        <div className="flex items-center gap-2 ml-auto">
          {stats.total > 0 && (
            <>
              <Badge variant="secondary" className="text-xs">
                总计: {stats.total}
              </Badge>
              {stats.active > 0 && (
                <Badge variant="default" className="text-xs">
                  活跃: {stats.active}
                </Badge>
              )}
              {stats.withKey > 0 && (
                <Badge variant="outline" className="text-xs">
                  已配置: {stats.withKey}
                </Badge>
              )}
            </>
          )}
        </div>
      </div>

      {/* 错误提示 */}
      {error && (
        <Alert variant="destructive">
          <AlertDescription className="flex items-center justify-between">
            <span>{error}</span>
            <Button
              variant="ghost"
              size="sm"
              onClick={clearError}
              className="h-6 px-2"
            >
              关闭
            </Button>
          </AlertDescription>
        </Alert>
      )}

      {/* 加载状态 */}
      {loading && providers.length === 0 && (
        <div className="flex justify-center py-8">
          <Loading text="加载提供商..." />
        </div>
      )}

      {/* 空状态 */}
      {!loading && providers.length === 0 && (
        <Card className="p-8 text-center">
          <p className="text-muted-foreground">暂无 API 提供商配置</p>
          <p className="text-sm text-muted-foreground mt-2">
            点击下方按钮添加第一个提供商
          </p>
        </Card>
      )}

      {/* 提供商列表 */}
      {providers.length > 0 && (
        <div className="space-y-2">
          {providers.filter((p) => p.id).map((provider) => {
            const providerId = provider.id!;
            const testStatus = getTestStatus(providerId);
            const testResult = testResults.get(providerId);

            return (
              <Card
                key={provider.id}
                className={cn(
                  'p-4 transition-all hover:shadow-md',
                  provider.isActive && 'border-primary bg-primary/5'
                )}
              >
                <div className="flex items-start gap-3">
                  {/* 图标/类型标识 */}
                  <div className="shrink-0 mt-0.5">
                    <div
                      className={cn(
                        'w-10 h-10 rounded-lg flex items-center justify-center',
                        provider.isActive
                          ? 'bg-primary text-primary-foreground'
                          : 'bg-muted'
                      )}
                    >
                      <Settings className="h-5 w-5" />
                    </div>
                  </div>

                  {/* 提供商信息 */}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                      <h4 className="font-medium truncate">{provider.name}</h4>
                      {provider.isActive && (
                        <Badge variant="default" className="text-xs">
                          <Zap className="h-3 w-3 mr-1" />
                          活跃
                        </Badge>
                      )}
                      <Badge variant="outline" className="text-xs">
                        {provider.providerType}
                      </Badge>
                    </div>

                    <div className="flex items-center gap-3 text-xs text-muted-foreground">
                      <span className="truncate max-w-[200px]">
                        {provider.baseUrl}
                      </span>
                      {provider.hasApiKey ? (
                        <span className="flex items-center gap-1">
                          <CheckCircle2 className="h-3 w-3 text-green-500" />
                          已配置密钥
                        </span>
                      ) : (
                        <span className="flex items-center gap-1">
                          <AlertCircle className="h-3 w-3 text-orange-500" />
                          未配置密钥
                        </span>
                      )}
                      {provider.model && (
                        <span className="truncate">模型: {provider.model}</span>
                      )}
                    </div>

                    {/* 测试结果 */}
                    {testResult && (
                      <div
                        className={cn(
                          'mt-2 text-xs flex items-center gap-1.5',
                          testResult.success
                            ? 'text-green-600'
                            : 'text-red-600'
                        )}
                      >
                        {testResult.success ? (
                          <CheckCircle2 className="h-3.5 w-3.5" />
                        ) : (
                          <XCircle className="h-3.5 w-3.5" />
                        )}
                        <span>
                          {testResult.success
                            ? '连接成功'
                            : testResult.errorMessage || '连接失败'}
                        </span>
                      </div>
                    )}
                  </div>

                  {/* 操作按钮 */}
                  <div className="flex items-center gap-1 shrink-0">
                    {!provider.isActive && (
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => handleSetActive(provider)}
                        className="h-8"
                        title="设为活跃"
                      >
                        <Zap className="h-4 w-4" />
                      </Button>
                    )}

                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleTestConnection(provider)}
                      disabled={testStatus === 'testing'}
                      className={cn(
                        'h-8',
                        testStatus === 'success' && 'text-green-600',
                        testStatus === 'error' && 'text-red-600'
                      )}
                      title="测试连接"
                    >
                      {testStatus === 'testing' ? (
                        <Settings className="h-4 w-4 animate-spin" />
                      ) : testStatus === 'success' ? (
                        <CheckCircle2 className="h-4 w-4" />
                      ) : testStatus === 'error' ? (
                        <XCircle className="h-4 w-4" />
                      ) : (
                        <Settings className="h-4 w-4" />
                      )}
                    </Button>

                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => onSelectProvider?.(provider)}
                      className="h-8"
                      title="编辑"
                    >
                      ✏️
                    </Button>

                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleDelete(provider)}
                      className="h-8 text-red-500 hover:text-red-600"
                      title="删除"
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </div>
                </div>
              </Card>
            );
          })}
        </div>
      )}
    </div>
  );
}

/**
 * 提供商卡片组件（单个）
 */
export interface ProviderCardProps {
  provider: ProviderResponse;
  /** 是否显示操作按钮 */
  showActions?: boolean;
  /** 设为活跃回调 */
  onSetActive?: (provider: ProviderResponse) => void;
  /** 测试连接回调 */
  onTest?: (provider: ProviderResponse) => void;
  /** 编辑回调 */
  onEdit?: (provider: ProviderResponse) => void;
  /** 删除回调 */
  onDelete?: (provider: ProviderResponse) => void;
  /** 自定义类名 */
  className?: string;
}

export function ProviderCard({
  provider,
  showActions = true,
  onSetActive,
  onTest,
  onEdit,
  onDelete,
  className,
}: ProviderCardProps) {
  return (
    <Card
      className={cn(
        'p-4 transition-all hover:shadow-md',
        provider.isActive && 'border-primary bg-primary/5',
        className
      )}
    >
      <div className="flex items-start gap-3">
        {/* 图标 */}
        <div className="shrink-0">
          <div
            className={cn(
              'w-10 h-10 rounded-lg flex items-center justify-center',
              provider.isActive
                ? 'bg-primary text-primary-foreground'
                : 'bg-muted'
            )}
          >
            <Settings className="h-5 w-5" />
          </div>
        </div>

        {/* 信息 */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <h4 className="font-medium truncate">{provider.name}</h4>
            {provider.isActive && (
              <Badge variant="default" className="text-xs">活跃</Badge>
            )}
          </div>
          <p className="text-xs text-muted-foreground truncate">
            {provider.providerType} · {provider.baseUrl}
          </p>
        </div>

        {/* 操作 */}
        {showActions && (
          <div className="flex items-center gap-1 shrink-0">
            {onSetActive && !provider.isActive && (
              <Button
                variant="ghost"
                size="sm"
                onClick={() => onSetActive(provider)}
                className="h-8"
              >
                <Zap className="h-4 w-4" />
              </Button>
            )}
            {onTest && (
              <Button
                variant="ghost"
                size="sm"
                onClick={() => onTest(provider)}
                className="h-8"
              >
                🔗
              </Button>
            )}
            {onEdit && (
              <Button
                variant="ghost"
                size="sm"
                onClick={() => onEdit(provider)}
                className="h-8"
              >
                ✏️
              </Button>
            )}
            {onDelete && (
              <Button
                variant="ghost"
                size="sm"
                onClick={() => onDelete(provider)}
                className="h-8 text-red-500"
              >
                <Trash2 className="h-4 w-4" />
              </Button>
            )}
          </div>
        )}
      </div>
    </Card>
  );
}
