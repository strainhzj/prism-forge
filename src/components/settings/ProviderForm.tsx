/**
 * ProviderForm - API 提供商表单组件
 *
 * 用于创建或编辑 API 提供商配置
 * 根据 provider_type 动态显示字段
 */

import { useForm } from 'react-hook-form';
import { useEffect, useState, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import {
  ApiProviderType,
  type SaveProviderRequest,
  type ProviderResponse,
  DEFAULT_MODELS,
  PROVIDER_DISPLAY_INFO,
  THIRD_PARTY_PROVIDERS,
  PROVIDER_TYPE_KEYS,
  THIRD_PARTY_PROVIDER_KEYS,
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
  submitText,
  loading = false,
}) => {
  const { t } = useTranslation('settings');
  const [showThirdPartyPresets, setShowThirdPartyPresets] = useState(false);

  // 提供商类型选项（使用翻译）
  const PROVIDER_TYPE_OPTIONS = useMemo(() => {
    return Object.entries(PROVIDER_DISPLAY_INFO).map(([key]) => {
      const providerTypeKey = PROVIDER_TYPE_KEYS[key as ApiProviderType];
      return {
        value: key as ApiProviderType,
        label: t(`providerTypes.${providerTypeKey}.label`),
        description: t(`providerTypes.${providerTypeKey}.description`),
      };
    });
  }, [t]);

  // 第三方提供商预设（使用翻译）
  const TRANSLATED_THIRD_PARTY_PROVIDERS = useMemo(() => {
    return THIRD_PARTY_PROVIDERS.map((preset) => {
      const providerKey = THIRD_PARTY_PROVIDER_KEYS[preset.id];
      return {
        ...preset,
        name: providerKey ? t(`thirdPartyProviders.${providerKey}.name`) : preset.name,
        description: providerKey ? t(`thirdPartyProviders.${providerKey}.description`) : preset.description,
      };
    });
  }, [t]);

  const {
    register,
    handleSubmit,
    watch,
    setValue,
    reset,
    formState: { errors, isDirty: _isDirty },
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
    // 切换供应商类型时总是自动更新 baseUrl
    setValue('baseUrl', DEFAULT_BASE_URLS[providerType]);
    // 如果选择的是 OpenAI Compatible，显示第三方预设
    setShowThirdPartyPresets(providerType === ApiProviderType.OPENAI_COMPATIBLE);
  }, [providerType, setValue]);

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

  // 获取当前提供商类型的翻译描述
  const currentProviderTypeKey = PROVIDER_TYPE_KEYS[providerType];
  const providerTypeDescription = currentProviderTypeKey ? t(`providerTypes.${currentProviderTypeKey}.description`) : displayInfo?.description;

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
          {t('form.providerType')} <span className="required">*</span>
        </label>
        <select
          id="providerType"
          className="form-control"
          {...register('providerType', { required: t('validation.selectProviderType') })}
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
          {providerTypeDescription}
          {displayInfo?.websiteUrl && (
            <>
              {' '}| <a href={displayInfo.websiteUrl} target="_blank" rel="noopener noreferrer">{t('form.website')}</a>
            </>
          )}
          {displayInfo?.docsUrl && (
            <>
              {' '}| <a href={displayInfo.docsUrl} target="_blank" rel="noopener noreferrer">{t('form.docs')}</a>
            </>
          )}
        </small>
      </div>

      {/* 第三方服务商快速选择（仅 OpenAI Compatible 显示） */}
      {showThirdPartyPresets && !provider && (
        <div className="form-group third-party-presets">
          <label>{t('form.thirdPartyPresets')}</label>
          <div className="presets-grid">
            {TRANSLATED_THIRD_PARTY_PROVIDERS.map((preset) => (
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
            {t('helpText.selectPresetHint')}
          </small>
        </div>
      )}

      {/* 提供商名称 */}
      <div className="form-group">
        <label htmlFor="name">
          {t('form.name')} <span className="required">*</span>
        </label>
        <input
          id="name"
          type="text"
          className="form-control"
          placeholder={t('placeholders.name')}
          {...register('name', { required: t('validation.enterProviderName') })}
        />
        {errors.name && <span className="error-text">{errors.name.message}</span>}
      </div>

      {/* Base URL */}
      <div className="form-group">
        <label htmlFor="baseUrl">
          {t('form.baseUrl')} <span className="required">*</span>
        </label>
        <input
          id="baseUrl"
          type="text"
          className="form-control"
          placeholder={t('placeholders.baseUrl')}
          {...register('baseUrl', {
            required: t('validation.enterBaseUrl'),
            pattern: {
              value: /^https?:\/\/.+/,
              message: t('validation.baseUrlMustStartWithHttp'),
            },
          })}
        />
        {errors.baseUrl && <span className="error-text">{errors.baseUrl.message}</span>}
        <small className="help-text">
          {t('helpText.defaultBaseUrl')}: {displayInfo?.defaultBaseUrl}
          {displayInfo?.apiKeyUrl && providerType === ApiProviderType.OPENAI && (
            <>
              <br />
              <a href={displayInfo.apiKeyUrl} target="_blank" rel="noopener noreferrer">
                {t('form.getApiKey')}
              </a>
            </>
          )}
        </small>
      </div>

      {/* 模型 */}
      <div className="form-group">
        <label htmlFor="model">{t('form.model')}</label>
        <input
          id="model"
          type="text"
          className="form-control"
          placeholder={DEFAULT_MODELS[providerType]}
          {...register('model')}
        />
        <small className="help-text">
          {t('helpText.leaveEmptyForDefault')}: {DEFAULT_MODELS[providerType]}
          <br />
          <strong>{t('helpText.namespaceHint')}</strong>
        </small>
      </div>

      {/* Temperature */}
      <div className="form-group">
        <label htmlFor="temperature">{t('form.temperature')}</label>
        <input
          id="temperature"
          type="number"
          step="0.1"
          min="0"
          max="2"
          className="form-control"
          placeholder={t('placeholders.temperature')}
          {...register('temperature', {
            valueAsNumber: true,  // 自动转换为数字类型，空值转为 null
            min: { value: 0, message: t('validation.temperatureMin') },
            max: { value: 2, message: t('validation.temperatureMax') },
          })}
        />
        {errors.temperature && <span className="error-text">{errors.temperature.message}</span>}
        <small className="help-text">
          {t('helpText.temperatureHint')}
        </small>
      </div>

      {/* Max Tokens */}
      <div className="form-group">
        <label htmlFor="maxTokens">{t('form.maxTokens')}</label>
        <input
          id="maxTokens"
          type="number"
          min="1"
          className="form-control"
          placeholder={t('placeholders.maxTokens')}
          {...register('maxTokens', {
            valueAsNumber: true,  // 自动转换为数字类型，空值转为 null
            min: { value: 1, message: t('validation.maxTokensMin') },
          })}
        />
        {errors.maxTokens && <span className="error-text">{errors.maxTokens.message}</span>}
        <small className="help-text">
          {t('helpText.maxTokensHint')}
        </small>
      </div>

      {/* API Key - Ollama 不需要 */}
      {requiresApiKey && (
        <div className="form-group">
          <label htmlFor="apiKey">
            {t('form.apiKey')} <span className="required">*</span>
            {provider?.hasApiKey && (
              <span className="existing-key-hint">
                ({t('helpText.apiKeyExisting')}: {provider.apiKeyMask || '****'})
              </span>
            )}
          </label>
          <textarea
            id="apiKey"
            className="form-control"
            rows={provider?.hasApiKey ? 1 : 3}
            placeholder={
              provider?.hasApiKey
                ? t('placeholders.apiKeyKeepExisting')
                : t('placeholders.apiKeySingle') + '\n' + t('placeholders.apiKeyMultiple')
            }
            autoComplete="off"
            {...register('apiKey', {
              required: provider?.hasApiKey ? false : t('validation.enterApiKey'),
              validate: (value) => {
                if (!value || value.trim() === '') {
                  return provider?.hasApiKey ? true : t('validation.enterApiKey');
                }
                // 检查最小长度
                const trimmed = value.trim();
                if (trimmed.length < 10) {
                  return t('validation.apiKeyMinLength');
                }
                // 如果包含逗号，验证多密钥格式
                if (trimmed.includes(',')) {
                  const keys = trimmed.split(',').map(k => k.trim()).filter(k => k);
                  if (keys.length < 2) {
                    return t('validation.multiKeyInvalid');
                  }
                  if (keys.some(k => k.length < 10)) {
                    return t('validation.eachKeyMinLength');
                  }
                }
                return true;
              },
            })}
          />
          {errors.apiKey && <span className="error-text">{errors.apiKey.message}</span>}
          <small className="help-text">
            {provider?.hasApiKey
              ? t('helpText.apiKeyKeepExisting')
              : t('helpText.apiKeyRotation')
            }
          </small>
        </div>
      )}

      {/* 额外配置（可选） */}
      <div className="form-group">
        <label htmlFor="configJson">{t('form.configJson')}</label>
        <textarea
          id="configJson"
          className="form-control"
          rows={3}
          placeholder={t('placeholders.configJson')}
          {...register('configJson', {
            validate: (value) => {
              if (!value) return true;
              try {
                JSON.parse(value);
                return true;
              } catch {
                return t('validation.invalidJson');
              }
            },
          })}
        />
        {errors.configJson && <span className="error-text">{errors.configJson.message}</span>}
        <small className="help-text">
          {t('helpText.configJsonHint')}
        </small>
      </div>

      {/* 别名（可选） */}
      <div className="form-group">
        <label htmlFor="aliases">{t('form.aliases')}</label>
        <input
          id="aliases"
          type="text"
          className="form-control"
          placeholder={t('placeholders.aliases')}
          {...register('aliases', {
            validate: (value) => {
              if (!value || value.trim() === '' || value === '[]') {
                return true;
              }
              try {
                const parsed = JSON.parse(value);
                if (!Array.isArray(parsed)) {
                  return t('validation.aliasesMustBeArray');
                }
                if (!parsed.every((item: unknown) => typeof item === 'string')) {
                  return t('validation.aliasesElementsMustBeString');
                }
                return true;
              } catch {
                return t('validation.aliasesJsonExample');
              }
            },
          })}
        />
        {errors.aliases && <span className="error-text">{errors.aliases.message}</span>}
        <small className="help-text">
          {t('helpText.aliasesHint')}
        </small>
      </div>

      {/* 设为活跃提供商 */}
      <div className="form-group checkbox-group">
        <label className="checkbox-label">
          <input
            type="checkbox"
            {...register('isActive')}
          />
          <span>{t('form.setAsActive')}</span>
        </label>
        <small className="help-text">
          {t('helpText.onlyOneActive')}
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
            {t('buttons.cancel')}
          </button>
        )}
        <button
          type="submit"
          className="btn btn-primary"
          disabled={loading}
        >
          {loading ? t('buttons.saving') : (submitText || t('buttons.save'))}
        </button>
      </div>
    </form>
  );
};

export default ProviderForm;
