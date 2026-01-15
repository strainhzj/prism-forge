/**
 * Settings 页面 - LLM API 提供商管理
 * Cherry Studio 风格：左右分栏，左侧搜索+列表，右侧表单
 */

import { useEffect, useState, useCallback, Component, type ReactNode } from 'react';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { Home } from 'lucide-react';
import { useProviderActions, useProviders, useProvidersLoading, useProvidersError, type ProviderResponse, type SaveProviderRequest, type TestConnectionResult, ConnectionErrorType } from '../stores/useSettingsStore';
import { ProviderForm } from '../components/settings/ProviderForm';
import './Settings.css';

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[Settings] ${action}`, ...args);
  }
}

/**
 * 格式化连接测试结果消息
 */
function formatTestResultMessage(result: TestConnectionResult, t: (key: string, params?: any) => string): string {
  if (result.success) {
    return t('testResult.connectionSuccess');
  }

  // 根据错误类型返回更友好的消息
  const errorTypeLabels: Record<ConnectionErrorType, string> = {
    [ConnectionErrorType.AUTHENTICATION]: t('testResult.authenticationError'),
    [ConnectionErrorType.NETWORK]: t('testResult.networkError'),
    [ConnectionErrorType.SERVER]: t('testResult.serverError'),
    [ConnectionErrorType.REQUEST]: t('testResult.requestError'),
    [ConnectionErrorType.UNKNOWN]: t('testResult.unknownError'),
  };

  const typeLabel = result.errorType ? errorTypeLabels[result.errorType] : '';
  const message = result.errorMessage || t('testResult.connectionFailed');

  return typeLabel ? `[${typeLabel}] ${message}` : message;
}

/**
 * 获取提供商图标首字母
 */
function getProviderIcon(providerType: string): string {
  const typeMap: Record<string, string> = {
    'openai': 'O',
    'anthropic': 'A',
    'ollama': 'O',
    'xai': 'X',
    'google': 'G',
    'googlevertex': 'V',
    'azureopenai': 'A',
    'openai-compatible': 'C',
  };
  return typeMap[providerType] || providerType.charAt(0).toUpperCase();
}

// ==================== 错误边界 ====================

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

interface ErrorBoundaryProps {
  children: ReactNode;
  translations: {
    errorTitle: string;
    unknownError: string;
    refreshPage: string;
  };
}

class SettingsErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('[Settings] ErrorBoundary caught error:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="settings-page">
          <div className="alert alert-error">
            <h2>{this.props.translations.errorTitle}</h2>
            <p>{this.state.error?.message || this.props.translations.unknownError}</p>
            <pre style={{ fontSize: '12px', overflow: 'auto' }}>
              {this.state.error?.stack}
            </pre>
            <button onClick={() => window.location.reload()}>{this.props.translations.refreshPage}</button>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}

// ==================== 组件状态 ====================

type ViewMode = 'list' | 'create' | 'edit';

interface SettingsState {
  viewMode: ViewMode;
  selectedProviderId: number | null;
  testingProviderId: number | null;
  testResult: { id: number; result: TestConnectionResult } | null;
  searchQuery: string;
}

// ==================== Settings 页面 ====================

const SettingsContent: React.FC = () => {
  debugLog('render', 'SettingsContent mounting');
  const navigate = useNavigate();
  const { t } = useTranslation('settings');

  // Store 状态
  const providers = useProviders();
  const loading = useProvidersLoading();
  const error = useProvidersError();
  const { fetchProviders, saveProvider, deleteProvider, setActiveProvider, testProviderConnection, clearError } = useProviderActions();

  debugLog('state', { providersCount: providers.length, loading, error });

  // 本地状态
  const [{ viewMode, selectedProviderId, testingProviderId, testResult, searchQuery }, setState] = useState<SettingsState>({
    viewMode: 'list',
    selectedProviderId: null,
    testingProviderId: null,
    testResult: null,
    searchQuery: '',
  });

  // 获取选中的提供商
  const selectedProvider = providers.find((p) => p.id === selectedProviderId);

  // 过滤提供商
  const filteredProviders = providers.filter((p) =>
    p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    p.providerType.toLowerCase().includes(searchQuery.toLowerCase())
  );

  // ==================== 初始化 ====================

  useEffect(() => {
    debugLog('useEffect', 'fetchProviders triggered');
    fetchProviders().catch((err) => {
      debugLog('useEffect', 'fetchProviders failed', err);
    });
  }, [fetchProviders]);

  // ==================== 操作处理 ====================

  // 创建新提供商
  const handleCreate = useCallback(() => {
    setState({
      viewMode: 'create',
      selectedProviderId: null,
      testingProviderId: null,
      testResult: null,
      searchQuery,
    });
    clearError();
  }, [clearError, searchQuery]);

  // 编辑提供商
  const handleEdit = useCallback((provider: ProviderResponse) => {
    setState({
      viewMode: 'edit',
      selectedProviderId: provider.id ?? null,
      testingProviderId: null,
      testResult: null,
      searchQuery,
    });
    clearError();
  }, [clearError, searchQuery]);

  // 取消编辑
  const handleCancel = useCallback(() => {
    setState({
      viewMode: 'list',
      selectedProviderId: null,
      testingProviderId: null,
      testResult: null,
      searchQuery,
    });
  }, [searchQuery]);

  // 保存提供商
  const handleSave = useCallback(async (data: SaveProviderRequest) => {
    try {
      await saveProvider(data);
      // 保存成功后返回列表视图
      setState((prev) => ({
        ...prev,
        viewMode: 'list',
        selectedProviderId: null,
      }));
    } catch (error) {
      // 错误已由 store 处理
      console.error('保存失败:', error);
    }
  }, [saveProvider]);

  // 删除提供商
  const handleDelete = useCallback(async (provider: ProviderResponse) => {
    if (!provider.id) return;

    const confirmed = window.confirm(
      t('deleteConfirm', { name: provider.name })
    );

    if (!confirmed) return;

    try {
      await deleteProvider(provider.id);
      setState((prev) => ({
        ...prev,
        selectedProviderId: null,
      }));
    } catch (error) {
      console.error('删除失败:', error);
    }
  }, [deleteProvider, t]);

  // 切换活跃状态
  const handleToggleActive = useCallback(async (provider: ProviderResponse, e: React.MouseEvent) => {
    e.stopPropagation();
    if (!provider.id) return;

    try {
      await setActiveProvider(provider.id);
    } catch (error) {
      console.error('设置活跃提供商失败:', error);
    }
  }, [setActiveProvider]);

  // 测试连接
  const handleTestConnection = useCallback(async (provider: ProviderResponse) => {
    if (!provider.id) return;

    debugLog('handleTestConnection', 'testing provider', provider.id);

    setState((prev) => ({
      ...prev,
      testingProviderId: provider.id ?? null,
      testResult: null,
    }));

    try {
      const result = await testProviderConnection(provider.id);

      debugLog('handleTestConnection', 'result', result);

      setState((prev) => ({
        ...prev,
        testingProviderId: null,
        testResult: {
          id: provider.id!,
          result,
        },
      }));
    } catch (error) {
      // 解析错误信息
      let errorMessage = t('page.unknownError');
      if (typeof error === 'string') {
        errorMessage = error;
      } else if (error instanceof Error) {
        errorMessage = error.message;
      } else if (error && typeof error === 'object' && 'message' in error) {
        errorMessage = String((error as { message: unknown }).message);
      }

      debugLog('handleTestConnection', 'error', errorMessage);

      setState((prev) => ({
        ...prev,
        testingProviderId: null,
        testResult: {
          id: provider.id!,
          result: {
            success: false,
            errorMessage: `${t('testResult.testFailed')}: ${errorMessage}`,
            errorType: ConnectionErrorType.UNKNOWN,
          },
        },
      }));
    }
  }, [testProviderConnection, t]);

  // 搜索处理
  const handleSearchChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    setState((prev) => ({
      ...prev,
      searchQuery: e.target.value,
    }));
  }, []);

  // ==================== 渲染 ====================

  return (
    <div className="settings-page">
      {/* 错误提示 */}
      {error && (
        <div className="alert alert-error" style={{ margin: '16px' }}>
          <span>{error}</span>
          <button className="close-btn" onClick={clearError}>×</button>
        </div>
      )}

      {/* 主内容区域 */}
      <div className="settings-content">
        {/* 左侧：提供商列表 */}
        <div className="providers-panel">
          {/* 返回按钮 */}
          <div className="providers-panel-back">
            <button className="back-btn" onClick={() => navigate('/')}>
              <Home size={16} />
              <span>{t('page.backToHome')}</span>
            </button>
          </div>

          {/* 左侧顶部：搜索和添加 */}
          <div className="providers-panel-header">
            <div className="providers-panel-search">
              <input
                type="text"
                placeholder={t('page.searchPlaceholder')}
                value={searchQuery}
                onChange={handleSearchChange}
              />
            </div>
            <button className="providers-panel-add-btn" onClick={handleCreate}>
              + {t('page.addProvider')}
            </button>
          </div>

          {/* 提供商列表 */}
          {loading && providers.length === 0 ? (
            <div className="loading-state">{t('page.loading')}</div>
          ) : providers.length === 0 ? (
            <div className="empty-state">
              <p>{t('page.empty')}</p>
            </div>
          ) : (
            <ul className="providers-list">
              {filteredProviders.map((provider) => (
                <li
                  key={provider.id}
                  className={`provider-item ${provider.id === selectedProviderId ? 'active' : ''} ${provider.isActive ? 'is-active' : ''}`}
                  onClick={() => handleEdit(provider)}
                  data-provider-type={provider.providerType}
                >
                  {/* 提供商图标 */}
                  <div className="provider-icon">
                    {getProviderIcon(provider.providerType)}
                  </div>

                  {/* 提供商信息 */}
                  <div className="provider-info">
                    <div className="provider-name">{provider.name}</div>
                  </div>

                  {/* 活跃状态开关 */}
                  <div
                    className={`provider-toggle ${provider.isActive ? 'active' : ''}`}
                    onClick={(e) => handleToggleActive(provider, e)}
                    title={provider.isActive ? t('page.enabled') : t('page.disabled')}
                  />
                </li>
              ))}
            </ul>
          )}
        </div>

        {/* 右侧：表单区域 */}
        <div className="form-panel">
          {viewMode === 'list' && !selectedProvider ? (
            <div className="empty-selection">
              <p>{t('page.selectToEdit')}</p>
            </div>
          ) : (
            <div className="form-container">
              <div className="form-header">
                <h2>
                  {selectedProvider && (
                    <div className="provider-icon" style={{ width: '28px', height: '28px', fontSize: '14px' }}>
                      {getProviderIcon(selectedProvider.providerType)}
                    </div>
                  )}
                  {viewMode === 'create' ? t('page.createTitle') : selectedProvider?.name}
                </h2>
                {selectedProvider && (
                  <div className="form-actions-inline">
                    <button
                      className="btn-test"
                      onClick={() => handleTestConnection(selectedProvider)}
                      disabled={testingProviderId === selectedProvider.id}
                    >
                      {testingProviderId === selectedProvider.id ? t('page.testing') : t('page.test')}
                    </button>
                    <button
                      className="btn-copy"
                      onClick={() => handleDelete(selectedProvider)}
                      title={t('page.deleteTitle')}
                    >
                      {t('page.delete')}
                    </button>
                  </div>
                )}
              </div>

              <ProviderForm
                provider={selectedProvider ?? undefined}
                onSubmit={handleSave}
                onCancel={handleCancel}
                submitText={viewMode === 'create' ? t('buttons.create') : t('buttons.save')}
                loading={loading}
              />

              {/* 表单外测试结果展示 */}
              {selectedProvider && testResult && testResult.id === selectedProvider.id && (
                <div className={`test-result-banner ${testResult.result.success ? 'success' : 'error'}`}>
                  {formatTestResultMessage(testResult.result, t)}
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

// 包装错误边界的 Settings 组件
const Settings: React.FC = () => {
  debugLog('render', 'Settings wrapper mounting');
  const { t } = useTranslation('settings');

  const errorTranslations = {
    errorTitle: t('page.errorTitle'),
    unknownError: t('page.unknownError'),
    refreshPage: t('page.refreshPage'),
  };

  return (
    <SettingsErrorBoundary translations={errorTranslations}>
      <SettingsContent />
    </SettingsErrorBoundary>
  );
};

export default Settings;
