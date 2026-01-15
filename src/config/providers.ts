/**
 * LLM Provider 配置
 *
 * 参考 Cherry Studio 的提供商配置结构
 * 每个提供商包含：id、name、apiHost（Base URL）、默认模型、描述等
 */

// ==================== 类型定义 ====================

/**
 * 提供商配置
 */
export interface ProviderConfig {
  /** 提供商唯一标识 */
  id: string;
  /** 提供商显示名称 */
  name: string;
  /** 提供商类型 */
  type: ProviderType;
  /** API 基础 URL（Base URL） */
  apiHost: string;
  /** Anthropic API 基础 URL（某些提供商需要） */
  anthropicApiHost?: string;
  /** 默认模型名称 */
  defaultModel: string;
  /** 是否需要 API Key */
  requiresApiKey: boolean;
  /** 提供商描述 */
  description: string;
  /** 官方网站 URL */
  websiteUrl?: string;
  /** API Key 获取页面 URL */
  apiKeyUrl?: string;
  /** 文档 URL */
  docsUrl?: string;
  /** 模型列表页面 URL */
  modelsUrl?: string;
}

/**
 * 提供商类型（用于内部适配器选择）
 */
export type ProviderType =
  | 'openai'           // OpenAI API 格式
  | 'anthropic'        // Anthropic Claude API
  | 'ollama'           // Ollama 本地服务
  | 'gemini'           // Google Gemini (API Key 认证)
  | 'vertexai'         // Google Vertex AI
  | 'azure-openai'     // Azure OpenAI
  | 'openai-compatible'; // OpenAI 兼容接口

// ==================== 提供商配置 ====================

/**
 * 系统内置提供商配置
 */
export const SYSTEM_PROVIDERS: Record<string, ProviderConfig> = {
  // ========== 国际主流提供商 ==========

  openai: {
    id: 'openai',
    name: 'OpenAI',
    type: 'openai',
    apiHost: 'https://api.openai.com/v1',
    defaultModel: 'gpt-4o-mini',
    requiresApiKey: true,
    description: 'OpenAI 官方 API，支持 GPT-4、GPT-3.5 等模型',
    websiteUrl: 'https://openai.com/',
    apiKeyUrl: 'https://platform.openai.com/api-keys',
    docsUrl: 'https://platform.openai.com/docs',
    modelsUrl: 'https://platform.openai.com/docs/models',
  },

  anthropic: {
    id: 'anthropic',
    name: 'Anthropic',
    type: 'anthropic',
    apiHost: 'https://api.anthropic.com',
    defaultModel: 'claude-3-5-sonnet-20241022',
    requiresApiKey: true,
    description: 'Anthropic Claude 系列，包括 Claude 3.5 Sonnet',
    websiteUrl: 'https://anthropic.com/',
    apiKeyUrl: 'https://console.anthropic.com/settings/keys',
    docsUrl: 'https://docs.anthropic.com/en/docs',
    modelsUrl: 'https://docs.anthropic.com/en/docs/about-claude/models',
  },

  gemini: {
    id: 'gemini',
    name: 'Google Gemini',
    type: 'gemini',
    apiHost: 'https://generativelanguage.googleapis.com',
    defaultModel: 'gemini-2.5-flash-lite',
    requiresApiKey: true,
    description: 'Google Gemini (ML Dev API)，使用 API Key 认证',
    websiteUrl: 'https://gemini.google.com/',
    apiKeyUrl: 'https://aistudio.google.com/app/apikey',
    docsUrl: 'https://ai.google.dev/gemini-api/docs',
    modelsUrl: 'https://ai.google.dev/gemini-api/docs/models/gemini',
  },

  /** 保留 Google Vertex AI 实现 */
  vertexai: {
    id: 'vertexai',
    name: 'Google Vertex AI',
    type: 'vertexai',
    apiHost: 'https://aiplatform.googleapis.com',
    defaultModel: 'gemini-2.5-flash-lite',
    requiresApiKey: true,
    description: 'Google Vertex AI Public Preview (API Key URL 参数)',
    websiteUrl: 'https://cloud.google.com/vertex-ai',
    apiKeyUrl: 'https://console.cloud.google.com/apis/credentials',
    docsUrl: 'https://cloud.google.com/vertex-ai/generative-ai/docs',
    modelsUrl: 'https://cloud.google.com/vertex-ai/generative-ai/docs/learn/models',
  },

  'azure-openai': {
    id: 'azure-openai',
    name: 'Azure OpenAI',
    type: 'azure-openai',
    apiHost: 'https://{your-resource-name}.openai.azure.com/openai/deployments/{deployment}?api-version=2024-02-01',
    defaultModel: 'gpt-4o-mini',
    requiresApiKey: true,
    description: 'Microsoft Azure OpenAI 服务',
    websiteUrl: 'https://azure.microsoft.com/en-us/products/ai-services/openai-service',
    apiKeyUrl: 'https://portal.azure.com/#view/Microsoft_Azure_ProjectOxford/CognitiveServicesHub/~/OpenAI',
    docsUrl: 'https://learn.microsoft.com/en-us/azure/ai-services/openai/',
    modelsUrl: 'https://learn.microsoft.com/en-us/azure/ai-services/openai/concepts/models',
  },

  ollama: {
    id: 'ollama',
    name: 'Ollama',
    type: 'ollama',
    apiHost: 'http://127.0.0.1:11434',
    defaultModel: 'llama3',
    requiresApiKey: false,
    description: '本地 Ollama 服务，无需 API Key',
    websiteUrl: 'https://ollama.com/',
    docsUrl: 'https://github.com/ollama/ollama/tree/main/docs',
    modelsUrl: 'https://ollama.com/library',
  },

  groq: {
    id: 'groq',
    name: 'Groq',
    type: 'openai',
    apiHost: 'https://api.groq.com/openai',
    defaultModel: 'llama-3.3-70b-versatile',
    requiresApiKey: true,
    description: 'Groq 超快推理引擎，提供 Llama、Mixtral 等开源模型',
    websiteUrl: 'https://groq.com/',
    apiKeyUrl: 'https://console.groq.com/keys',
    docsUrl: 'https://console.groq.com/docs/quickstart',
    modelsUrl: 'https://console.groq.com/docs/models',
  },

  openrouter: {
    id: 'openrouter',
    name: 'OpenRouter',
    type: 'openai',
    apiHost: 'https://openrouter.ai/api/v1',
    defaultModel: 'anthropic/claude-3.5-sonnet',
    requiresApiKey: true,
    description: '统一 API 接口访问多种模型，支持 Claude、GPT-4 等',
    websiteUrl: 'https://openrouter.ai/',
    apiKeyUrl: 'https://openrouter.ai/settings/keys',
    docsUrl: 'https://openrouter.ai/docs/quick-start',
    modelsUrl: 'https://openrouter.ai/models',
  },

  // ========== 国内主流提供商 ==========

  deepseek: {
    id: 'deepseek',
    name: 'DeepSeek',
    type: 'openai',
    apiHost: 'https://api.deepseek.com',
    anthropicApiHost: 'https://api.deepseek.com/anthropic',
    defaultModel: 'deepseek-chat',
    requiresApiKey: true,
    description: 'DeepSeek AI，提供 DeepSeek-V3、DeepSeek-Coder 等模型',
    websiteUrl: 'https://deepseek.com/',
    apiKeyUrl: 'https://platform.deepseek.com/api_keys',
    docsUrl: 'https://platform.deepseek.com/api-docs/',
    modelsUrl: 'https://platform.deepseek.com/api-docs/',
  },

  silicon: {
    id: 'silicon',
    name: 'SiliconFlow (硅基流动)',
    type: 'openai',
    apiHost: 'https://api.siliconflow.cn',
    anthropicApiHost: 'https://api.siliconflow.cn',
    defaultModel: 'Qwen/Qwen2.5-72B-Instruct',
    requiresApiKey: true,
    description: '硅基流动，提供 Qwen、DeepSeek、GLM 等多种模型 API',
    websiteUrl: 'https://www.siliconflow.cn',
    apiKeyUrl: 'https://cloud.siliconflow.cn/i/d1nTBKXU',
    docsUrl: 'https://docs.siliconflow.cn/',
    modelsUrl: 'https://cloud.siliconflow.cn/models',
  },

  zhipu: {
    id: 'zhipu',
    name: 'Zhipu AI (智谱)',
    type: 'openai',
    apiHost: 'https://open.bigmodel.cn/api/paas/v4',
    anthropicApiHost: 'https://open.bigmodel.cn/api/anthropic',
    defaultModel: 'glm-4-flash',
    requiresApiKey: true,
    description: '智谱 AI，提供 GLM-4、GLM-3 等系列模型',
    websiteUrl: 'https://open.bigmodel.cn/',
    apiKeyUrl: 'https://open.bigmodel.cn/usercenter/apikeys',
    docsUrl: 'https://docs.bigmodel.cn/',
    modelsUrl: 'https://open.bigmodel.cn/modelcenter/square',
  },

  moonshot: {
    id: 'moonshot',
    name: 'Moonshot AI (月之暗面)',
    type: 'openai',
    apiHost: 'https://api.moonshot.cn',
    anthropicApiHost: 'https://api.moonshot.cn/anthropic',
    defaultModel: 'moonshot-v1-8k',
    requiresApiKey: true,
    description: '月之暗面，提供 Kimi 系列模型',
    websiteUrl: 'https://www.moonshot.cn/',
    apiKeyUrl: 'https://platform.moonshot.cn/console/api-keys',
    docsUrl: 'https://platform.moonshot.cn/docs/',
    modelsUrl: 'https://platform.moonshot.cn/docs/intro#模型列表',
  },

  baichuan: {
    id: 'baichuan',
    name: 'Baichuan AI (百川)',
    type: 'openai',
    apiHost: 'https://api.baichuan-ai.com',
    defaultModel: 'Baichuan4',
    requiresApiKey: true,
    description: '百川智能，提供 Baichuan2、Baichuan4 等系列模型',
    websiteUrl: 'https://www.baichuan-ai.com/',
    apiKeyUrl: 'https://platform.baichuan-ai.com/console/apikey',
    docsUrl: 'https://platform.baichuan-ai.com/docs',
    modelsUrl: 'https://platform.baichuan-ai.com/price',
  },

  minimax: {
    id: 'minimax',
    name: 'MiniMax',
    type: 'openai',
    apiHost: 'https://api.minimaxi.com/v1',
    anthropicApiHost: 'https://api.minimaxi.com/anthropic',
    defaultModel: 'abab6.5s-chat',
    requiresApiKey: true,
    description: 'MiniMax，提供 abab 系列对话模型',
    websiteUrl: 'https://platform.minimaxi.com/',
    apiKeyUrl: 'https://platform.minimaxi.com/user-center/basic-information/interface-key',
    docsUrl: 'https://platform.minimaxi.com/document/Announcement',
    modelsUrl: 'https://platform.minimaxi.com/document/Models',
  },

  dashscope: {
    id: 'dashscope',
    name: 'Bailian (百炼)',
    type: 'openai',
    apiHost: 'https://dashscope.aliyuncs.com/compatible-mode/v1',
    anthropicApiHost: 'https://dashscope.aliyuncs.com/apps/anthropic',
    defaultModel: 'qwen-turbo-plus',
    requiresApiKey: true,
    description: '阿里云百炼平台，提供 Qwen 系列模型',
    websiteUrl: 'https://www.aliyun.com/product/bailian',
    apiKeyUrl: 'https://bailian.console.aliyun.com/?tab=model#/api-key',
    docsUrl: 'https://help.aliyun.com/zh/model-studio/getting-started/',
    modelsUrl: 'https://bailian.console.aliyun.com/?tab=model#/model-market',
  },

  doubao: {
    id: 'doubao',
    name: 'Doubao (豆包)',
    type: 'openai',
    apiHost: 'https://ark.cn-beijing.volces.com/api/v3',
    defaultModel: 'ep-20241205174028-w9ccq',
    requiresApiKey: true,
    description: '字节跳动豆包大模型',
    websiteUrl: 'https://console.volcengine.com/ark/',
    apiKeyUrl: 'https://www.volcengine.com/experience/ark?utm_term=202502dsinvite&ac=DSASUQY5&rc=DB4II4FC',
    docsUrl: 'https://www.volcengine.com/docs/82379/1182403',
    modelsUrl: 'https://console.volcengine.com/ark/region:ark+cn-beijing/endpoint',
  },

  hunyuan: {
    id: 'hunyuan',
    name: 'Hunyuan (混元)',
    type: 'openai',
    apiHost: 'https://api.hunyuan.cloud.tencent.com',
    defaultModel: 'hunyuan-lite',
    requiresApiKey: true,
    description: '腾讯混元大模型',
    websiteUrl: 'https://cloud.tencent.com/product/hunyuan',
    apiKeyUrl: 'https://console.cloud.tencent.com/hunyuan/api-key',
    docsUrl: 'https://cloud.tencent.com/document/product/1729/111007',
    modelsUrl: 'https://cloud.tencent.com/document/product/1729/104753',
  },

  'baidu-cloud': {
    id: 'baidu-cloud',
    name: 'Baidu Cloud (百度千帆)',
    type: 'openai',
    apiHost: 'https://qianfan.baidubce.com/v2',
    defaultModel: 'ERNIE-Speed-128K',
    requiresApiKey: true,
    description: '百度千帆大模型平台',
    websiteUrl: 'https://cloud.baidu.com/',
    apiKeyUrl: 'https://console.bce.baidu.com/iam/#/iam/apikey/list',
    docsUrl: 'https://cloud.baidu.com/doc/index.html',
    modelsUrl: 'https://cloud.baidu.com/doc/WENXINWORKSHOP/s/Fm2vrveyu',
  },

  stepfun: {
    id: 'stepfun',
    name: 'StepFun',
    type: 'openai',
    apiHost: 'https://api.stepfun.com',
    defaultModel: 'step-1-16k',
    requiresApiKey: true,
    description: 'StepFun，提供 step-1v、step-2-16k 等模型',
    websiteUrl: 'https://platform.stepfun.com/',
    apiKeyUrl: 'https://platform.stepfun.com/interface-key',
    docsUrl: 'https://platform.stepfun.com/docs/overview/concept',
    modelsUrl: 'https://platform.stepfun.com/docs/llm/text',
  },

  // ========== 其他常用提供商 ==========

  together: {
    id: 'together',
    name: 'Together',
    type: 'openai',
    apiHost: 'https://api.together.xyz',
    defaultModel: 'meta-llama/Llama-3.3-70B-Instruct-Turbo',
    requiresApiKey: true,
    description: 'Together AI，提供多种开源模型托管服务',
    websiteUrl: 'https://www.together.ai/',
    apiKeyUrl: 'https://api.together.ai/settings/api-keys',
    docsUrl: 'https://docs.together.ai/docs/introduction',
    modelsUrl: 'https://docs.together.ai/docs/chat-models',
  },

  fireworks: {
    id: 'fireworks',
    name: 'Fireworks',
    type: 'openai',
    apiHost: 'https://api.fireworks.ai/inference',
    defaultModel: 'accounts/fireworks/models/llama-v3p3-70b-instruct',
    requiresApiKey: true,
    description: 'Fireworks AI，快速推理的托管模型服务',
    websiteUrl: 'https://fireworks.ai/',
    apiKeyUrl: 'https://fireworks.ai/account/api-keys',
    docsUrl: 'https://docs.fireworks.ai/getting-started/introduction',
    modelsUrl: 'https://fireworks.ai/dashboard/models',
  },

  nvidia: {
    id: 'nvidia',
    name: 'NVIDIA NIM',
    type: 'openai',
    apiHost: 'https://integrate.api.nvidia.com',
    defaultModel: 'meta/llama-3.3-70b-instruct',
    requiresApiKey: true,
    description: 'NVIDIA NIM，提供优化的 AI 模型推理 API',
    websiteUrl: 'https://build.nvidia.com/explore/discover',
    apiKeyUrl: 'https://build.nvidia.com/meta/llama-3_1-405b-instruct',
    docsUrl: 'https://docs.api.nvidia.com/nim/reference/llm-apis',
    modelsUrl: 'https://build.nvidia.com/nim',
  },

  perplexity: {
    id: 'perplexity',
    name: 'Perplexity',
    type: 'openai',
    apiHost: 'https://api.perplexity.ai',
    defaultModel: 'llama-3.1-sonar-small-128k-online',
    requiresApiKey: true,
    description: 'Perplexity AI，提供带网络搜索的模型',
    websiteUrl: 'https://perplexity.ai/',
    apiKeyUrl: 'https://www.perplexity.ai/settings/api',
    docsUrl: 'https://docs.perplexity.ai/home',
    modelsUrl: 'https://docs.perplexity.ai/guides/model-cards',
  },

  mistral: {
    id: 'mistral',
    name: 'Mistral',
    type: 'openai',
    apiHost: 'https://api.mistral.ai',
    defaultModel: 'mistral-large-latest',
    requiresApiKey: true,
    description: 'Mistral AI，提供 Mistral、Mixtral 等开源模型',
    websiteUrl: 'https://mistral.ai',
    apiKeyUrl: 'https://console.mistral.ai/api-keys/',
    docsUrl: 'https://docs.mistral.ai',
    modelsUrl: 'https://docs.mistral.ai/getting-started/models/models_overview',
  },

  jina: {
    id: 'jina',
    name: 'Jina',
    type: 'openai',
    apiHost: 'https://api.jina.ai',
    defaultModel: 'jina-4o',
    requiresApiKey: true,
    description: 'Jina AI，专注于 embedding 和推理模型',
    websiteUrl: 'https://jina.ai',
    apiKeyUrl: 'https://jina.ai/',
    docsUrl: 'https://jina.ai',
    modelsUrl: 'https://jina.ai',
  },

  github: {
    id: 'github',
    name: 'GitHub Models',
    type: 'openai',
    apiHost: 'https://models.github.ai/inference',
    defaultModel: 'gpt-4o',
    requiresApiKey: true,
    description: 'GitHub Models，提供多种模型的统一接口',
    websiteUrl: 'https://github.com/marketplace/models',
    apiKeyUrl: 'https://github.com/settings/tokens',
    docsUrl: 'https://docs.github.com/en/github-models',
    modelsUrl: 'https://github.com/marketplace/models',
  },

  cerebras: {
    id: 'cerebras',
    name: 'Cerebras',
    type: 'openai',
    apiHost: 'https://api.cerebras.ai/v1',
    defaultModel: 'llama3.3-70b',
    requiresApiKey: true,
    description: 'Cerebras，超快的 LLM 推理服务',
    websiteUrl: 'https://www.cerebras.ai',
    apiKeyUrl: 'https://cloud.cerebras.ai',
    docsUrl: 'https://inference-docs.cerebras.ai/introduction',
    modelsUrl: 'https://inference-docs.cerebras.ai/models/overview',
  },

  hyperbolic: {
    id: 'hyperbolic',
    name: 'Hyperbolic',
    type: 'openai',
    apiHost: 'https://api.hyperbolic.xyz',
    defaultModel: 'meta-llama/Llama-3.3-70B-Instruct',
    requiresApiKey: true,
    description: 'Hyperbolic，低成本 GPU 推理服务',
    websiteUrl: 'https://app.hyperbolic.xyz',
    apiKeyUrl: 'https://app.hyperbolic.xyz/settings',
    docsUrl: 'https://docs.hyperbolic.xyz',
    modelsUrl: 'https://app.hyperbolic.xyz/models',
  },

  voyageai: {
    id: 'voyageai',
    name: 'VoyageAI',
    type: 'openai',
    apiHost: 'https://api.voyageai.com',
    defaultModel: 'voyage-3',
    requiresApiKey: true,
    description: 'VoyageAI，专注于 embedding 模型',
    websiteUrl: 'https://www.voyageai.com/',
    apiKeyUrl: 'https://dashboard.voyageai.com/organization/api-keys',
    docsUrl: 'https://docs.voyageai.com/docs',
    modelsUrl: 'https://docs.voyageai.com/docs',
  },

  // ========== 第三方中转服务 ==========

  '302ai': {
    id: '302ai',
    name: '302.AI',
    type: 'openai',
    apiHost: 'https://api.302.ai',
    defaultModel: 'gpt-4o-mini',
    requiresApiKey: true,
    description: '302.AI 中转服务，支持多种模型',
    websiteUrl: 'https://302.ai',
    apiKeyUrl: 'https://dash.302.ai/apis/list',
    docsUrl: 'https://302ai.apifox.cn/api-147522039',
    modelsUrl: 'https://302.ai/pricing/',
  },

  aihubmix: {
    id: 'aihubmix',
    name: 'AiHubMix',
    type: 'openai',
    apiHost: 'https://aihubmix.com',
    anthropicApiHost: 'https://aihubmix.com',
    defaultModel: 'gpt-4o-mini',
    requiresApiKey: true,
    description: 'AiHubMix 中转服务',
    websiteUrl: 'https://aihubmix.com?aff=SJyh',
    apiKeyUrl: 'https://aihubmix.com?aff=SJyh',
    docsUrl: 'https://doc.aihubmix.com/',
    modelsUrl: 'https://aihubmix.com/models',
  },

  'openai-compatible': {
    id: 'openai-compatible',
    name: 'OpenAI Compatible (自定义)',
    type: 'openai-compatible',
    apiHost: 'https://api.example.com/v1',
    defaultModel: 'gpt-4o-mini',
    requiresApiKey: true,
    description: '兼容 OpenAI API 格式的第三方服务（OneAPI、中转服务等）',
  },
};

// ==================== 辅助函数 ====================

/**
 * 获取提供商配置
 */
export function getProviderConfig(providerId: string): ProviderConfig | undefined {
  return SYSTEM_PROVIDERS[providerId];
}

/**
 * 获取所有提供商配置列表
 */
export function getAllProviders(): ProviderConfig[] {
  return Object.values(SYSTEM_PROVIDERS);
}

/**
 * 获取提供商的默认 Base URL
 */
export function getDefaultBaseUrl(providerId: string): string {
  const config = getProviderConfig(providerId);
  return config?.apiHost || '';
}

/**
 * 获取提供商的默认模型
 */
export function getDefaultModel(providerId: string): string {
  const config = getProviderConfig(providerId);
  return config?.defaultModel || '';
}

/**
 * 根据 ProviderType 获取提供商列表
 */
export function getProvidersByType(type: ProviderType): ProviderConfig[] {
  return Object.values(SYSTEM_PROVIDERS).filter(p => p.type === type);
}

/**
 * 判断提供商是否需要 API Key
 */
export function requiresApiKey(providerId: string): boolean {
  const config = getProviderConfig(providerId);
  return config?.requiresApiKey ?? true;
}
