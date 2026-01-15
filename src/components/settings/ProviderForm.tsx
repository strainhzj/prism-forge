/**
 * ProviderForm - API 提供商表单组件
 *
 * 用于创建或编辑 API 提供商配置
 * 根据 provider_type 动态显示字段
 */

import { useForm } from 'react-hook-form';
import { useEffect, useState } from 'react';
import {
  ApiProviderType,
  type SaveProviderRequest,
  type ProviderResponse,
  DEFAULT_MODELS,
  PROVIDER_DISPLAY_INFO,
  THIRD_PARTY_PROVIDERS,
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

// 提供商类型选项（从 PROVIDER_DISPLAY_INFO 生成）
const PROVIDER_TYPE_OPTIONS = Object.entries(PROVIDER_DISPLAY_INFO).map(([key, info]) => ({
  value: key as ApiProviderType,
  label: info.label,
  description: info.description,
}));

// 默认 Base URL（从 PROVIDER_DISPLAY_INFO 生成）
const DEFAULT_BASE_URLS: Record<ApiProviderType, string> = Object.entries(PROVIDER_DISPLAY_INFO).reduce(
  (acc, [key, info]) => ({
    ...acc,
    [key]: info.defaultBaseUrl,
  }),
  {} as Record<ApiProviderType, string>
);

// ==================== 组件 ====================

export const ProviderForm: React.FC<ProviderFormProps> = ({
  provider,
  onSubmit,
  onCancel,
  submitText = '保存',
  loading = false,
}) => {
  const [showThirdPartyPresets, setShowThirdPartyPresets] = useState(false);

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
    // 如果选择的是 OpenAI Compatible，显示第三方预设
    setShowThirdPartyPresets(providerType === ApiProviderType.OPENAI_COMPATIBLE);
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
  const displayInfo = PROVIDER_DISPLAY_INFO[providerType];
  const requiresApiKey = displayInfo?.requiresApiKey ?? true;

  // 选择第三方服务商预设
  const handleSelectThirdPartyPreset = (preset: typeof THIRD_PARTY_PROVIDERS[0]) => {
    setValue('name', preset.name);
    setValue('baseUrl', preset.baseUrl);
    setValue('model', preset.defaultModel);
    setShowThirdPartyPresets(false);
  };

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
          {displayInfo?.description}
          {displayInfo?.websiteUrl && (
            <>
              {' '}| <a href={displayInfo.websiteUrl} target="_blank" rel="noopener noreferrer">官网</a>
            </>
          )}
          {displayInfo?.docsUrl && (
            <>
              {' '}| <a href={displayInfo.docsUrl} target="_blank" rel="noopener noreferrer">文档</a>
            </>
          )}
        </small>
      </div>

      {/* 第三方服务商快速选择（仅 OpenAI Compatible 显示） */}
      {showThirdPartyPresets && !provider && (
        <div className="form-group third-party-presets">
          <label>快速选择常用服务商</label>
          <div className="presets-grid">
            {THIRD_PARTY_PROVIDERS.map((preset) => (
              <button
                key={preset.id}
                type="button"
                className="preset-card"
                onClick={() => handleSelectThirdPartyPreset(preset)}
              >
                <div className="preset-name">{preset.name}</div>
                <div className="preset-description">{preset.description}</div>
              </button>
            ))}
          </div>
          <small className="help-text">
            点击上方卡片快速填充配置，或手动填写下方表单
          </small>
        </div>
      )}

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
          默认值: {displayInfo?.defaultBaseUrl}
          {displayInfo?.apiKeyUrl && providerType === ApiProviderType.OPENAI && (
            <>
              <br />
              <a href={displayInfo.apiKeyUrl} target="_blank" rel="noopener noreferrer">
                获取 API Key →
              </a>
            </>
          )}
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
