/**
 * ProviderForm - API 提供商表单组件
 *
 * 用于创建或编辑 API 提供商配置
 * 根据 provider_type 动态显示字段
 */

import { useForm } from 'react-hook-form';
import { useEffect } from 'react';
import {
  ApiProviderType,
  type SaveProviderRequest,
  type ProviderResponse,
} from '../../stores/useSettingsStore';

// ==================== 表单数据类型 ====================

interface ProviderFormData {
  providerType: ApiProviderType;
  name: string;
  baseUrl: string;
  apiKey?: string;
  configJson?: string;
  isActive: boolean;
}

// ==================== Props ====================

interface ProviderFormProps {
  /**
   * 编辑模式的提供商数据（为 null 则为创建模式）
   */
  provider?: ProviderResponse | null;

  /**
   * 表单提交回调
   */
  onSubmit: (data: SaveProviderRequest) => Promise<void>;

  /**
   * 表单取消回调
   */
  onCancel?: () => void;

  /**
   * 提交按钮文本
   */
  submitText?: string;

  /**
   * 是否正在提交
   */
  loading?: boolean;
}

// ==================== 常量 ====================

const PROVIDER_TYPE_OPTIONS = [
  { value: ApiProviderType.OPENAI, label: 'OpenAI', description: 'OpenAI 或兼容接口（OneAPI、中转服务等）' },
  { value: ApiProviderType.ANTHROPIC, label: 'Anthropic', description: 'Claude (Anthropic)' },
  { value: ApiProviderType.OLLAMA, label: 'Ollama', description: '本地 Ollama 服务' },
];

const DEFAULT_BASE_URLS: Record<ApiProviderType, string> = {
  [ApiProviderType.OPENAI]: 'https://api.openai.com/v1',
  [ApiProviderType.ANTHROPIC]: 'https://api.anthropic.com',
  [ApiProviderType.OLLAMA]: 'http://127.0.0.1:11434',
};

// ==================== 组件 ====================

export const ProviderForm: React.FC<ProviderFormProps> = ({
  provider,
  onSubmit,
  onCancel,
  submitText = '保存',
  loading = false,
}) => {
  const {
    register,
    handleSubmit,
    watch,
    setValue,
    reset,
    formState: { errors, isDirty },
  } = useForm<ProviderFormData>({
    defaultValues: {
      providerType: ApiProviderType.OPENAI,
      name: '',
      baseUrl: DEFAULT_BASE_URLS[ApiProviderType.OPENAI],
      apiKey: '',
      configJson: '',
      isActive: false,
    },
  });

  // 监听 providerType 变化，自动更新 baseUrl
  const providerType = watch('providerType');

  useEffect(() => {
    if (!isDirty) {
      // 只在表单未修改时自动更新 baseUrl
      setValue('baseUrl', DEFAULT_BASE_URLS[providerType]);
    }
  }, [providerType, setValue, isDirty]);

  // 编辑模式：填充表单数据
  useEffect(() => {
    if (provider) {
      reset({
        providerType: provider.providerType,
        name: provider.name,
        baseUrl: provider.baseUrl,
        apiKey: '', // API Key 不回显，需要重新输入
        configJson: provider.configJson || '',
        isActive: provider.isActive,
      });
    }
  }, [provider, reset]);

  // 判断当前提供商类型是否需要 API Key
  const requiresApiKey = providerType !== ApiProviderType.OLLAMA;

  // 提交处理
  const handleFormSubmit = async (data: ProviderFormData) => {
    await onSubmit({
      id: provider?.id,
      ...data,
    });
  };

  return (
    <form onSubmit={handleSubmit(handleFormSubmit)} className="provider-form">
      {/* 提供商类型选择 */}
      <div className="form-group">
        <label htmlFor="providerType">
          提供商类型 <span className="required">*</span>
        </label>
        <select
          id="providerType"
          className="form-control"
          {...register('providerType', { required: '请选择提供商类型' })}
          disabled={!!provider} // 编辑模式下不允许修改类型
        >
          {PROVIDER_TYPE_OPTIONS.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
        {errors.providerType && (
          <span className="error-text">{errors.providerType.message}</span>
        )}
        <small className="help-text">
          {PROVIDER_TYPE_OPTIONS.find((o) => o.value === providerType)?.description}
        </small>
      </div>

      {/* 提供商名称 */}
      <div className="form-group">
        <label htmlFor="name">
          名称 <span className="required">*</span>
        </label>
        <input
          id="name"
          type="text"
          className="form-control"
          placeholder="例如: OpenAI 官方、Ollama 本地"
          {...register('name', { required: '请输入提供商名称' })}
        />
        {errors.name && <span className="error-text">{errors.name.message}</span>}
      </div>

      {/* Base URL */}
      <div className="form-group">
        <label htmlFor="baseUrl">
          Base URL <span className="required">*</span>
        </label>
        <input
          id="baseUrl"
          type="text"
          className="form-control"
          placeholder="https://api.openai.com/v1"
          {...register('baseUrl', {
            required: '请输入 Base URL',
            pattern: {
              value: /^https?:\/\/.+/,
              message: 'Base URL 必须以 http:// 或 https:// 开头',
            },
          })}
        />
        {errors.baseUrl && <span className="error-text">{errors.baseUrl.message}</span>}
        <small className="help-text">
          默认值: {DEFAULT_BASE_URLS[providerType]}
        </small>
      </div>

      {/* API Key - Ollama 不需要 */}
      {requiresApiKey && (
        <div className="form-group">
          <label htmlFor="apiKey">
            API Key <span className="required">*</span>
            {provider?.hasApiKey && (
              <span className="existing-key-hint">
                (已配置: {provider.apiKeyMask || '****'})
              </span>
            )}
          </label>
          <input
            id="apiKey"
            type="password"
            className="form-control"
            placeholder={provider?.hasApiKey ? '留空以保持现有密钥' : 'sk-... 或 sk-ant-...'}
            autoComplete="off"
            {...register('apiKey', {
              required: provider?.hasApiKey ? false : '请输入 API Key',
              minLength: {
                value: 10,
                message: 'API Key 长度不能少于 10 个字符',
              },
            })}
          />
          {errors.apiKey && <span className="error-text">{errors.apiKey.message}</span>}
          {!provider?.hasApiKey && (
            <small className="help-text">
              API Key 将被安全存储在系统密钥库中
            </small>
          )}
        </div>
      )}

      {/* 额外配置（可选） */}
      <div className="form-group">
        <label htmlFor="configJson">额外配置 (JSON, 可选)</label>
        <textarea
          id="configJson"
          className="form-control"
          rows={3}
          placeholder='{"model": "gpt-4", "temperature": 0.7}'
          {...register('configJson', {
            validate: (value) => {
              if (!value) return true;
              try {
                JSON.parse(value);
                return true;
              } catch {
                return 'JSON 格式无效';
              }
            },
          })}
        />
        {errors.configJson && <span className="error-text">{errors.configJson.message}</span>}
        <small className="help-text">
          高级配置，例如 model、temperature 等（JSON 格式）
        </small>
      </div>

      {/* 设为活跃提供商 */}
      <div className="form-group checkbox-group">
        <label className="checkbox-label">
          <input
            type="checkbox"
            {...register('isActive')}
          />
          <span>设为活跃提供商</span>
        </label>
        <small className="help-text">
          同一时间只能有一个活跃提供商
        </small>
      </div>

      {/* 表单操作按钮 */}
      <div className="form-actions">
        {onCancel && (
          <button
            type="button"
            className="btn btn-secondary"
            onClick={onCancel}
            disabled={loading}
          >
            取消
          </button>
        )}
        <button
          type="submit"
          className="btn btn-primary"
          disabled={loading}
        >
          {loading ? '保存中...' : submitText}
        </button>
      </div>
    </form>
  );
};

export default ProviderForm;
