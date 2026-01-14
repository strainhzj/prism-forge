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
  DEFAULT_MODELS,
} from '../../stores/useSettingsStore';

// ==================== 表单数据类型 ====================

interface ProviderFormData {
  providerType: ApiProviderType;
  name: string;
  baseUrl: string;
  apiKey?: string;
  configJson?: string;
  isActive: boolean;
  model?: string;
  temperature?: number;
  maxTokens?: number;
  aliases?: string; // JSON 数组格式
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
  { value: ApiProviderType.OPENAI, label: 'OpenAI', description: 'OpenAI 官方 API' },
  { value: ApiProviderType.ANTHROPIC, label: 'Anthropic', description: 'Claude (Anthropic)' },
  { value: ApiProviderType.OLLAMA, label: 'Ollama', description: '本地 Ollama 服务' },
  { value: ApiProviderType.XAI, label: 'X AI', description: 'X AI (Grok)' },
  { value: ApiProviderType.GOOGLE, label: 'Google', description: 'Google Gemini ML Dev API (API Key 认证)' },
  { value: ApiProviderType.GOOGLE_VERTEX, label: 'Google Vertex', description: 'Google Vertex AI Public Preview (API Key URL 参数)' },
  { value: ApiProviderType.AZURE_OPENAI, label: 'Azure OpenAI', description: 'Microsoft Azure OpenAI 服务' },
  { value: ApiProviderType.OPENAI_COMPATIBLE, label: 'OpenAI Compatible', description: '第三方兼容接口（OneAPI、中转服务等）' },
];

const DEFAULT_BASE_URLS: Record<ApiProviderType, string> = {
  [ApiProviderType.OPENAI]: 'https://api.openai.com/v1',
  [ApiProviderType.ANTHROPIC]: 'https://api.anthropic.com',
  [ApiProviderType.OLLAMA]: 'http://127.0.0.1:11434',
  [ApiProviderType.XAI]: 'https://api.x.ai/v1',
  [ApiProviderType.GOOGLE]: 'https://generativelanguage.googleapis.com',
  [ApiProviderType.GOOGLE_VERTEX]: 'https://aiplatform.googleapis.com',
  [ApiProviderType.AZURE_OPENAI]: 'https://{your-resource-name}.openai.azure.com/openai/deployments/{deployment}?api-version=2024-02-01',
  [ApiProviderType.OPENAI_COMPATIBLE]: 'https://api.example.com/v1',
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
      model: '',
      temperature: 0.7,
      maxTokens: 2000,
      aliases: '[]',
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
        model: provider.model || '',
        temperature: provider.temperature ?? 0.7,
        maxTokens: provider.maxTokens ?? 2000,
        aliases: provider.aliases || '[]',
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

      {/* 模型 */}
      <div className="form-group">
        <label htmlFor="model">模型</label>
        <input
          id="model"
          type="text"
          className="form-control"
          placeholder={DEFAULT_MODELS[providerType]}
          {...register('model')}
        />
        <small className="help-text">
          留空使用默认模型: {DEFAULT_MODELS[providerType]}
          <br />
          <strong>提示：</strong>支持命名空间格式，如 "openai:gpt-4o"、"anthropic:claude-3-5-sonnet"
        </small>
      </div>

      {/* Temperature */}
      <div className="form-group">
        <label htmlFor="temperature">Temperature</label>
        <input
          id="temperature"
          type="number"
          step="0.1"
          min="0"
          max="2"
          className="form-control"
          placeholder="0.7"
          {...register('temperature', {
            valueAsNumber: true,  // 自动转换为数字类型，空值转为 null
            min: { value: 0, message: 'Temperature 不能小于 0' },
            max: { value: 2, message: 'Temperature 不能大于 2' },
          })}
        />
        {errors.temperature && <span className="error-text">{errors.temperature.message}</span>}
        <small className="help-text">
          控制随机性 (0.0 - 2.0)，默认 0.7
        </small>
      </div>

      {/* Max Tokens */}
      <div className="form-group">
        <label htmlFor="maxTokens">Max Tokens</label>
        <input
          id="maxTokens"
          type="number"
          min="1"
          className="form-control"
          placeholder="2000"
          {...register('maxTokens', {
            valueAsNumber: true,  // 自动转换为数字类型，空值转为 null
            min: { value: 1, message: 'Max Tokens 必须大于 0' },
          })}
        />
        {errors.maxTokens && <span className="error-text">{errors.maxTokens.message}</span>}
        <small className="help-text">
          最大输出 token 数，默认 2000
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
          <textarea
            id="apiKey"
            className="form-control"
            rows={provider?.hasApiKey ? 1 : 3}
            placeholder={
              provider?.hasApiKey
                ? '留空以保持现有密钥'
                : '输入单个密钥：sk-...\n或多个密钥（逗号分隔）：\nsk-key1,sk-key2,sk-key3'
            }
            autoComplete="off"
            {...register('apiKey', {
              required: provider?.hasApiKey ? false : '请输入 API Key',
              validate: (value) => {
                if (!value || value.trim() === '') {
                  return provider?.hasApiKey ? true : '请输入 API Key';
                }
                // 检查最小长度
                const trimmed = value.trim();
                if (trimmed.length < 10) {
                  return 'API Key 长度不能少于 10 个字符';
                }
                // 如果包含逗号，验证多密钥格式
                if (trimmed.includes(',')) {
                  const keys = trimmed.split(',').map(k => k.trim()).filter(k => k);
                  if (keys.length < 2) {
                    return '多密钥格式无效，请使用逗号分隔至少2个密钥';
                  }
                  if (keys.some(k => k.length < 10)) {
                    return '每个密钥长度不能少于 10 个字符';
                  }
                }
                return true;
              },
            })}
          />
          {errors.apiKey && <span className="error-text">{errors.apiKey.message}</span>}
          <small className="help-text">
            {provider?.hasApiKey
              ? '留空以保持现有密钥，或输入新密钥以更新'
              : '支持多密钥轮换（逗号分隔），系统将自动轮换使用以实现负载均衡'
            }
          </small>
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

      {/* 别名（可选） */}
      <div className="form-group">
        <label htmlFor="aliases">别名（可选）</label>
        <input
          id="aliases"
          type="text"
          className="form-control"
          placeholder='["claude", "anthropic-api"]'
          {...register('aliases', {
            validate: (value) => {
              if (!value || value.trim() === '' || value === '[]') {
                return true;
              }
              try {
                const parsed = JSON.parse(value);
                if (!Array.isArray(parsed)) {
                  return '别名必须是数组格式';
                }
                if (!parsed.every((item: unknown) => typeof item === 'string')) {
                  return '别名数组中的每个元素必须是字符串';
                }
                return true;
              } catch {
                return 'JSON 格式无效，例如: ["alias1", "alias2"]';
              }
            },
          })}
        />
        {errors.aliases && <span className="error-text">{errors.aliases.message}</span>}
        <small className="help-text">
          为此提供商设置别名，JSON 数组格式，例如: ["claude", "anthropic"]
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
