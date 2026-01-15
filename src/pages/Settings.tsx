/**
 * Settings 页面 - LLM API 提供商管理
 * Cherry Studio 风格：左右分栏，左侧搜索+列表，右侧表单
 */

import { useEffect, useState, useCallback, Component, type ReactNode } from 'react';
import { useNavigate } from 'react-router-dom';
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
function formatTestResultMessage(result: TestConnectionResult): string {
  if (result.success) {
    return '连接成功！';
  }

  // 根据错误类型返回更友好的消息
  const errorTypeLabels: Record<ConnectionErrorType, string> = {
    [ConnectionErrorType.AUTHENTICATION]: '认证错误',
    [ConnectionErrorType.NETWORK]: '网络错误',
    [ConnectionErrorType.SERVER]: '服务器错误',
    [ConnectionErrorType.REQUEST]: '请求错误',
    [ConnectionErrorType.UNKNOWN]: '未知错误',
  };

  const typeLabel = result.errorType ? errorTypeLabels[result.errorType] : '';
  const message = result.errorMessage || '连接失败，请检查配置';

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

class SettingsErrorBoundary extends Component<{ children: ReactNode }, ErrorBoundaryState> {
  constructor(props: { children: ReactNode }) {
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
            <h2>页面加载出错</h2>
            <p>{this.state.error?.message || '未知错误'}</p>
            <pre style={{ fontSize: '12px', overflow: 'auto' }}>
              {this.state.error?.stack}
            </pre>
            <button onClick={() => window.location.reload()}>刷新页面</button>
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
      `确定要删除提供商 "${provider.name}" 吗？\n\n` +
      `此操作将同时删除存储的 API Key，且不可恢复。`
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
  }, [deleteProvider]);

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
      let errorMessage = '未知错误';
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
            errorMessage: `测试失败: ${errorMessage}`,
            errorType: ConnectionErrorType.UNKNOWN,
          },
        },
      }));
    }
  }, [testProviderConnection]);

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
              <span>返回首页</span>
            </button>
          </div>

          {/* 左侧顶部：搜索和添加 */}
          <div className="providers-panel-header">
            <div className="providers-panel-search">
              <input
                type="text"
                placeholder="搜索提供商..."
                value={searchQuery}
                onChange={handleSearchChange}
              />
            </div>
            <button className="providers-panel-add-btn" onClick={handleCreate}>
              + 添加提供商
            </button>
          </div>

          {/* 提供商列表 */}
          {loading && providers.length === 0 ? (
            <div className="loading-state">加载中...</div>
          ) : providers.length === 0 ? (
            <div className="empty-state">
              <p>暂无提供商</p>
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
                    title={provider.isActive ? '已启用' : '已禁用'}
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
              <p>选择左侧提供商进行编辑，或添加新提供商</p>
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
                  {viewMode === 'create' ? '添加提供商' : selectedProvider?.name}
                </h2>
                {selectedProvider && (
                  <div className="form-actions-inline">
                    <button
                      className="btn-test"
                      onClick={() => handleTestConnection(selectedProvider)}
                      disabled={testingProviderId === selectedProvider.id}
                    >
                      {testingProviderId === selectedProvider.id ? '测试中...' : '检测'}
                    </button>
                    <button
                      className="btn-copy"
                      onClick={() => handleDelete(selectedProvider)}
                      title="删除"
                    >
                      删除
                    </button>
                  </div>
                )}
              </div>

              <ProviderForm
                provider={selectedProvider ?? undefined}
                onSubmit={handleSave}
                onCancel={handleCancel}
                submitText={viewMode === 'create' ? '创建' : '保存'}
                loading={loading}
              />

              {/* 表单外测试结果展示 */}
              {selectedProvider && testResult && testResult.id === selectedProvider.id && (
                <div className={`test-result-banner ${testResult.result.success ? 'success' : 'error'}`}>
                  {formatTestResultMessage(testResult.result)}
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
  return (
    <SettingsErrorBoundary>
      <SettingsContent />
    </SettingsErrorBoundary>
  );
};

export default Settings;
