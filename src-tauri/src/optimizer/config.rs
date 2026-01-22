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
    /// 中文版本模板
    pub template_zh: String,
    /// 英文版本模板
    pub template_en: String,
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
    /// 中文版本结构模板
    pub structure_zh: String,
    /// 英文版本结构模板
    pub structure_en: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct FallbackConfig {
    /// 中文版本：无会话时模板
    pub no_sessions_template_zh: String,
    /// 英文版本：无会话时模板
    pub no_sessions_template_en: String,
    /// 中文版本：LLM 调用失败时模板
    pub llm_error_template_zh: String,
    /// 英文版本：LLM 调用失败时模板
    pub llm_error_template_en: String,
    /// 中文版本：对话开始模板（当会话消息为空时使用）
    pub conversation_starter_template_zh: String,
    /// 英文版本：对话开始模板
    pub conversation_starter_template_en: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct SessionContextConfig {
    pub max_summary_length: usize,
    pub include_rating: bool,
    pub include_project: bool,
    /// 中文版本：会话格式化模板
    pub session_format_zh: String,
    /// 英文版本：会话格式化模板
    pub session_format_en: String,
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
                template_zh: r#"
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
                template_en: r#"
You are a professional programming assistant prompt optimizer. Based on the following information, generate a clear, specific prompt to help users efficiently complete programming tasks.

## Your Task
1. Analyze the user's programming goals
2. Reference solutions from relevant conversation history
3. Generate a directly usable, structured prompt

## Output Format
Please generate a prompt containing the following sections:
1. **Task Background**: Briefly describe the task to be completed
2. **Technical Requirements**: Technology stack and constraints involved
3. **Reference Solutions**: Suggestions based on historical experience
4. **Specific Steps**: Clear implementation steps
5. **Expected Output**: Expected result format

## Constraints
- The prompt should be concise and clear (within 500 words)
- Prioritize high-rating, high-similarity historical solutions
- Highlight key technical points and considerations
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
                structure_zh: r#"
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
                structure_en: r#"
{{meta_prompt}}

## User Goal
{{goal}}

## Related Conversation History
{{sessions}}

## Context Summary
{{context}}

## Request
Based on the above information, generate an optimized prompt.
"#.trim().to_string(),
            },
            fallback: FallbackConfig {
                no_sessions_template_zh: r#"
请帮我完成以下编程任务：

{{goal}}

请提供详细的实现方案和代码示例。
"#.trim().to_string(),
                no_sessions_template_en: r#"
Please help me complete the following programming task:

{{goal}}

Please provide a detailed implementation plan and code examples.
"#.trim().to_string(),
                llm_error_template_zh: r#"
请帮我完成以下编程任务：

{{goal}}

## 参考
相关会话: (评分: {{best_session_rating}})
{{best_session_summary}}

请提供详细的实现方案和代码示例。
"#.trim().to_string(),
                llm_error_template_en: r#"
Please help me complete the following programming task:

{{goal}}

## Reference
Related session: (Rating: {{best_session_rating}})
{{best_session_summary}}

Please provide a detailed implementation plan and code examples.
"#.trim().to_string(),
                conversation_starter_template_zh: r#"
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
                conversation_starter_template_en: r#"
You are a professional programming assistant. The user wants to start a new conversation. Please generate a clear, friendly prompt to help the user begin the conversation.

## User Goal
{{goal}}

## Requirements
1. Understand the user's goal and provide a friendly opening
2. Ask targeted questions to clarify requirements
3. Provide relevant suggestions or reference directions
4. Keep it concise and clear (within 200 words)

Please generate a conversation-starting prompt.
"#.trim().to_string(),
            },
            session_context: SessionContextConfig {
                max_summary_length: 200,
                include_rating: true,
                include_project: true,
                session_format_zh: "- 会话 {{session_id}} (项目: {{project_name}}) (评分: {{rating}}):\n  {{summary}}".to_string(),
                session_format_en: "- Session {{session_id}} (Project: {{project_name}}) (Rating: {{rating}}):\n  {{summary}}".to_string(),
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

    /// 获取 Meta-Prompt 模板（根据语言）
    pub fn get_meta_prompt(&self, language: &str) -> String {
        match language {
            "zh" => self.config.read().unwrap().meta_prompt.template_zh.clone(),
            _ => self.config.read().unwrap().meta_prompt.template_en.clone(),  // 默认英文
        }
    }

    /// 获取提示词结构模板（根据语言）
    pub fn get_prompt_structure(&self, language: &str) -> String {
        match language {
            "zh" => self.config.read().unwrap().prompt_structure.structure_zh.clone(),
            _ => self.config.read().unwrap().prompt_structure.structure_en.clone(),  // 默认英文
        }
    }

    /// 获取无会话回退模板（根据语言）
    pub fn get_no_sessions_template(&self, language: &str) -> String {
        match language {
            "zh" => self.config.read().unwrap().fallback.no_sessions_template_zh.clone(),
            _ => self.config.read().unwrap().fallback.no_sessions_template_en.clone(),  // 默认英文
        }
    }

    /// 获取 LLM 错误回退模板（根据语言）
    pub fn get_llm_error_template(&self, language: &str) -> String {
        match language {
            "zh" => self.config.read().unwrap().fallback.llm_error_template_zh.clone(),
            _ => self.config.read().unwrap().fallback.llm_error_template_en.clone(),  // 默认英文
        }
    }

    /// 获取对话开始模板（根据语言）
    pub fn get_conversation_starter_template(&self, language: &str) -> String {
        match language {
            "zh" => self.config.read().unwrap().fallback.conversation_starter_template_zh.clone(),
            _ => self.config.read().unwrap().fallback.conversation_starter_template_en.clone(),  // 默认英文
        }
    }

    /// 获取会话格式化模板（根据语言）
    pub fn get_session_format(&self, language: &str) -> String {
        match language {
            "zh" => self.config.read().unwrap().session_context.session_format_zh.clone(),
            _ => self.config.read().unwrap().session_context.session_format_en.clone(),  // 默认英文
        }
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
