/**
 * 日志过滤配置设置组件
 *
 * 提供过滤规则的查看和管理功能
 */

import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { Settings, FolderOpen, RefreshCw, Check, X, AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { cn } from '@/lib/utils';

// ==================== 类型定义 ====================

export enum MatchType {
  Contains = 'contains',
  Regex = 'regex',
  Exact = 'exact',
}

export interface FilterRule {
  name: string;
  enabled: boolean;
  matchType: MatchType;
  pattern: string;
  description?: string;
}

export interface FilterConfig {
  version: string;
  enabled: boolean;
  rules: FilterRule[];
}

// ==================== 组件 ====================

interface FilterConfigSettingsProps {
  className?: string;
}

export function FilterConfigSettings({ className }: FilterConfigSettingsProps) {
  const { t } = useTranslation('index');
  const [config, setConfig] = useState<FilterConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [configPath, setConfigPath] = useState<string>('');

  // 加载配置
  const loadConfig = async () => {
    try {
      setLoading(true);
      setError(null);
      const [configData, path] = await Promise.all([
        invoke<FilterConfig>('get_filter_config'),
        invoke<string>('get_filter_config_path'),
      ]);
      setConfig(configData);
      setConfigPath(path);
    } catch (err) {
      setError(t('filter.error.loadFailed'));
      console.error('加载过滤配置失败:', err);
    } finally {
      setLoading(false);
    }
  };

  // 切换规则启用状态
  const toggleRule = async (ruleName: string) => {
    if (!config) return;

    const updatedConfig = {
      ...config,
      rules: config.rules.map(rule =>
        rule.name === ruleName ? { ...rule, enabled: !rule.enabled } : rule
      ),
    };

    try {
      await invoke('update_filter_config', { config: updatedConfig });
      setConfig(updatedConfig);
      setSuccessMessage(t('filter.success.saved'));
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (err) {
      setError(t('filter.error.saveFailed'));
      console.error('保存配置失败:', err);
    }
  };

  // 切换全局过滤开关
  const toggleGlobalFilter = async () => {
    if (!config) return;

    const updatedConfig = {
      ...config,
      enabled: !config.enabled,
    };

    try {
      await invoke('update_filter_config', { config: updatedConfig });
      setConfig(updatedConfig);
      setSuccessMessage(t('filter.success.saved'));
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (err) {
      setError(t('filter.error.saveFailed'));
      console.error('保存配置失败:', err);
    }
  };

  // 打开配置文件所在目录
  const openConfigFolder = async () => {
    try {
      await invoke('open_filter_config_folder');
    } catch (err) {
      setError(t('filter.error.loadFailed'));
      console.error('打开配置目录失败:', err);
    }
  };

  // 重新加载配置
  const reloadConfig = async () => {
    try {
      await invoke('reload_filter_config');
      await loadConfig();
      setSuccessMessage(t('filter.success.reloaded'));
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (err) {
      setError(t('filter.error.reloadFailed'));
      console.error('重新加载配置失败:', err);
    }
  };

  // 初始加载
  useEffect(() => {
    loadConfig();
  }, []);

  if (loading) {
    return (
      <div className={cn('flex items-center justify-center p-8', className)}>
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4" />
          <p className="text-sm text-muted-foreground">加载配置中...</p>
        </div>
      </div>
    );
  }

  if (error && !config) {
    return (
      <div className={cn('p-4', className)}>
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      </div>
    );
  }

  if (!config) {
    return null;
  }

  return (
    <div className={cn('space-y-4', className)}>
      {/* 成功提示 */}
      {successMessage && (
        <Alert className="bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800">
          <Check className="h-4 w-4 text-green-600 dark:text-green-400" />
          <AlertDescription className="text-green-800 dark:text-green-200">
            {successMessage}
          </AlertDescription>
        </Alert>
      )}

      {/* 错误提示 */}
      {error && (
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      {/* 配置信息卡片 */}
      <div className="rounded-lg border bg-card p-4">
        <div className="flex items-center gap-3 mb-4">
          <Settings className="h-5 w-5 text-muted-foreground" />
          <div>
            <h3 className="font-semibold">{t('filter.title')}</h3>
            <p className="text-sm text-muted-foreground">{t('filter.description')}</p>
          </div>
        </div>

        {/* 全局开关 */}
        <div className="flex items-center justify-between py-3 border-b">
          <div>
            <div className="font-medium">{t('filter.enabled')}</div>
            <div className="text-sm text-muted-foreground">
              {config.enabled ? t('filter.enabled') : '已禁用'}
            </div>
          </div>
          <Button
            variant={config.enabled ? 'primary' : 'outline'}
            size="sm"
            onClick={toggleGlobalFilter}
          >
            {config.enabled ? <Check className="h-4 w-4 mr-2" /> : <X className="h-4 w-4 mr-2" />}
            {config.enabled ? '已启用' : '已禁用'}
          </Button>
        </div>

        {/* 配置文件路径 */}
        <div className="py-3 border-b">
          <div className="text-sm font-medium mb-1">配置文件路径</div>
          <div className="text-xs text-muted-foreground font-mono break-all">
            {configPath}
          </div>
        </div>

        {/* 操作按钮 */}
        <div className="flex gap-2 pt-3">
          <Button variant="outline" size="sm" onClick={openConfigFolder}>
            <FolderOpen className="h-4 w-4 mr-2" />
            {t('filter.openConfigFile')}
          </Button>
          <Button variant="outline" size="sm" onClick={reloadConfig}>
            <RefreshCw className="h-4 w-4 mr-2" />
            {t('filter.reload')}
          </Button>
        </div>
      </div>

      {/* 过滤规则列表 */}
      <div className="rounded-lg border bg-card p-4">
        <h4 className="font-semibold mb-4">{t('filter.rules')}</h4>

        <div className="space-y-2">
          {config.rules.map((rule) => (
            <div
              key={rule.name}
              className={cn(
                'flex items-start gap-3 p-3 rounded-lg border transition-colors',
                rule.enabled ? 'bg-background' : 'bg-muted/50 opacity-60'
              )}
            >
              {/* 启用开关 */}
              <button
                onClick={() => toggleRule(rule.name)}
                className={cn(
                  'mt-0.5 flex-shrink-0 w-5 h-5 rounded border-2 flex items-center justify-center transition-colors',
                  rule.enabled
                    ? 'bg-primary border-primary text-primary-foreground'
                    : 'border-muted-foreground bg-background'
                )}
              >
                {rule.enabled && <Check className="h-3 w-3" />}
              </button>

              {/* 规则信息 */}
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1">
                  <span className="font-medium text-sm">{rule.name}</span>
                  <span className="text-xs px-2 py-0.5 rounded-full bg-muted">
                    {t(`filter.matchTypes.${rule.matchType}`)}
                  </span>
                </div>

                {rule.description && (
                  <p className="text-xs text-muted-foreground mb-2">{rule.description}</p>
                )}

                <div className="text-xs font-mono bg-muted p-2 rounded overflow-x-auto">
                  {rule.pattern}
                </div>
              </div>
            </div>
          ))}
        </div>

        {config.rules.length === 0 && (
          <div className="text-center py-8 text-muted-foreground text-sm">
            暂无过滤规则，请编辑配置文件添加规则
          </div>
        )}
      </div>

      {/* 帮助信息 */}
      <Alert>
        <AlertCircle className="h-4 w-4" />
        <AlertDescription>
          点击"打开配置文件"按钮可以直接编辑 JSON 配置文件来添加或修改过滤规则。
          修改后点击"重新加载"按钮即可生效。
        </AlertDescription>
      </Alert>
    </div>
  );
}
