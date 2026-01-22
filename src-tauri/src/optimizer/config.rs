//! 优化器配置管理模块
//!
//! 从 optimizer_config.toml 文件加载配置，提供热重载支持

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// 优化器配置
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct OptimizerConfig {
    pub meta_prompt: MetaPromptConfig,
    pub llm_params: LLMParamsConfig,
    pub prompt_structure: PromptStructureConfig,
    pub fallback: FallbackConfig,
    pub session_context: SessionContextConfig,
    pub compression: CompressionConfig,
    pub advanced: AdvancedConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct MetaPromptConfig {
    pub template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct LLMParamsConfig {
    pub temperature: f32,
    pub max_tokens: usize,
    pub top_p: f32,
    pub frequency_penalty: f32,
    pub presence_penalty: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct PromptStructureConfig {
    pub structure: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct FallbackConfig {
    pub no_sessions_template: String,
    pub llm_error_template: String,
    /// 对话开始模板（当会话消息为空时使用）
    pub conversation_starter_template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct SessionContextConfig {
    pub max_summary_length: usize,
    pub include_rating: bool,
    pub include_project: bool,
    pub session_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct CompressionConfig {
    pub level: String,
    pub preserve_formatting: bool,
    pub min_compression_ratio: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct AdvancedConfig {
    pub parallel_processing: usize,
    pub cache_strategy: String,
    pub debug: bool,
    #[ts(type = "number")]
    pub timeout: u64,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            meta_prompt: MetaPromptConfig {
                template: r#"
你是一个专业的编程助手提示词优化器。基于以下信息，生成一个清晰、具体的提示词，帮助用户高效完成编程任务。

## 你的任务
1. 分析用户的编程目标
2. 参考相关历史会话中的解决方案
3. 生成一个可直接使用的、结构化的提示词

## 输出格式
请生成一个包含以下部分的提示词：
1. **任务背景**: 简要描述要完成的任务
2. **技术要求**: 涉及的技术栈和约束条件
3. **参考方案**: 基于历史经验的建议
4. **具体步骤**: 明确的实施步骤
5. **预期输出**: 期望的结果格式

## 限制条件
- 提示词要简洁明了（控制在 500 字以内）
- 优先参考高评分、高相似度的历史方案
- 突出关键技术点和注意事项
"#.trim().to_string(),
            },
            llm_params: LLMParamsConfig {
                temperature: 0.3,
                max_tokens: 1500,
                top_p: 0.9,
                frequency_penalty: 0.0,
                presence_penalty: 0.0,
            },
            prompt_structure: PromptStructureConfig {
                structure: r#"
{{meta_prompt}}

## 用户目标
{{goal}}

## 相关历史会话
{{sessions}}

## 上下文摘要
{{context}}

## 请求
基于上述信息，生成一个优化的提示词。
"#.trim().to_string(),
            },
            fallback: FallbackConfig {
                no_sessions_template: r#"
请帮我完成以下编程任务：

{{goal}}

请提供详细的实现方案和代码示例。
"#.trim().to_string(),
                llm_error_template: r#"
请帮我完成以下编程任务：

{{goal}}

## 参考
相关会话: (评分: {{best_session_rating}})
{{best_session_summary}}

请提供详细的实现方案和代码示例。
"#.trim().to_string(),
                conversation_starter_template: r#"
你是一个专业的编程助手。用户想要开始一个新的对话，请生成一个清晰、友好的提示词来帮助用户开始对话。

## 用户目标
{{goal}}

## 要求
1. 理解用户的目标，提供友好的开场白
2. 提出针对性的问题来明确需求
3. 提供相关的建议或参考方向
4. 保持简洁明了（控制在 200 字以内）

请生成一个对话开始的提示词。
"#.trim().to_string(),
            },
            session_context: SessionContextConfig {
                max_summary_length: 200,
                include_rating: true,
                include_project: true,
                session_format: "- 会话 {{session_id}} (项目: {{project_name}}) (评分: {{rating}}):\n  {{summary}}".to_string(),
            },
            compression: CompressionConfig {
                level: "basic".to_string(),
                preserve_formatting: true,
                min_compression_ratio: 0.0,
            },
            advanced: AdvancedConfig {
                parallel_processing: 5,
                cache_strategy: "memory".to_string(),
                debug: false,
                timeout: 30,
            },
        }
    }
}

/// 配置管理器
pub struct ConfigManager {
    config_path: PathBuf,
    config: Arc<RwLock<OptimizerConfig>>,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new(config_path: PathBuf) -> Result<Self> {
        let manager = Self {
            config_path,
            config: Arc::new(RwLock::new(OptimizerConfig::default())),
        };

        // 首次加载配置
        manager.reload()?;

        Ok(manager)
    }

    /// 重新加载配置文件
    pub fn reload(&self) -> Result<()> {
        let content = std::fs::read_to_string(&self.config_path)
            .with_context(|| format!("无法读取配置文件: {:?}", self.config_path))?;

        let config: OptimizerConfig = toml::from_str(&content)
            .with_context(|| format!("解析配置文件失败: {:?}", self.config_path))?;

        // 手动处理 RwLock 写入
        {
            let mut guard = self.config.write()
                .map_err(|e| anyhow::anyhow!("获取写锁失败: {}", e))?;
            *guard = config;
        }

        eprintln!("[ConfigManager] 配置已从 {:?} 重新加载", self.config_path);

        Ok(())
    }

    /// 获取配置的克隆
    pub fn get_config(&self) -> OptimizerConfig {
        self.config.read().unwrap().clone()
    }

    /// 获取 Meta-Prompt 模板
    pub fn get_meta_prompt(&self) -> String {
        self.config.read().unwrap().meta_prompt.template.clone()
    }

    /// 获取提示词结构模板
    pub fn get_prompt_structure(&self) -> String {
        self.config.read().unwrap().prompt_structure.structure.clone()
    }

    /// 获取无会话回退模板
    pub fn get_no_sessions_template(&self) -> String {
        self.config.read().unwrap().fallback.no_sessions_template.clone()
    }

    /// 获取 LLM 错误回退模板
    pub fn get_llm_error_template(&self) -> String {
        self.config.read().unwrap().fallback.llm_error_template.clone()
    }

    /// 获取对话开始模板
    pub fn get_conversation_starter_template(&self) -> String {
        self.config.read().unwrap().fallback.conversation_starter_template.clone()
    }

    /// 获取会话格式化模板
    pub fn get_session_format(&self) -> String {
        self.config.read().unwrap().session_context.session_format.clone()
    }

    /// 获取会话上下文配置
    pub fn get_session_context_config(&self) -> SessionContextConfig {
        self.config.read().unwrap().session_context.clone()
    }

    /// 获取 LLM 参数
    pub fn get_llm_params(&self) -> LLMParamsConfig {
        self.config.read().unwrap().llm_params.clone()
    }

    /// 获取最大摘要长度
    pub fn get_max_summary_length(&self) -> usize {
        self.config.read().unwrap().session_context.max_summary_length
    }

    /// 是否包含评分
    pub fn include_rating(&self) -> bool {
        self.config.read().unwrap().session_context.include_rating
    }

    /// 是否包含项目名
    pub fn include_project(&self) -> bool {
        self.config.read().unwrap().session_context.include_project
    }

    /// 是否启用调试模式
    pub fn is_debug(&self) -> bool {
        self.config.read().unwrap().advanced.debug
    }
}

/// 懒加载全局配置管理器
use once_cell::sync::Lazy;

static CONFIG_MANAGER: Lazy<Arc<RwLock<Option<Arc<ConfigManager>>>>> =
    Lazy::new(|| Arc::new(RwLock::new(None)));

/// 初始化全局配置管理器
pub fn init_config_manager(config_path: PathBuf) -> Result<()> {
    let manager = Arc::new(ConfigManager::new(config_path)?);
    // 手动处理 RwLock 写入
    {
        let mut guard = CONFIG_MANAGER.write()
            .map_err(|e| anyhow::anyhow!("获取写锁失败: {}", e))?;
        *guard = Some(manager);
    }
    Ok(())
}

/// 获取全局配置管理器
pub fn get_config_manager() -> Option<Arc<ConfigManager>> {
    CONFIG_MANAGER.read().ok()?.as_ref().cloned()
}
