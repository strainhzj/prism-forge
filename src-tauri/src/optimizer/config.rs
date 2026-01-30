//! 优化器配置管理模块
//!
//! 从 optimizer_config.toml 文件加载配置，提供热重载支持
//!
//! 组件化结构说明：
//! - components.meta_prompt: 可编辑的 Meta-Prompt 组件
//! - components.input_template: 只读的输入信息模板
//! - components.output_template: 只读的输出格式模板

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock, RwLockReadGuard};

/// 优化器配置
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct OptimizerConfig {
    /// 组件化提示词配置
    pub components: ComponentsConfig,
    /// LLM 调用参数
    pub llm_params: LLMParamsConfig,
    /// 会话上下文配置
    pub session_context: SessionContextConfig,
    /// 上下文压缩配置
    pub compression: CompressionConfig,
    /// 高级设置
    pub advanced: AdvancedConfig,
}

/// 组件化提示词配置
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ComponentsConfig {
    /// 可编辑组件：Meta-Prompt
    pub meta_prompt: LanguageComponent,
    /// 只读组件：输入信息模板
    pub input_template: LanguageComponent,
    /// 只读组件：输出格式模板
    pub output_template: LanguageComponent,
}

/// 语言组件（包含中英文版本）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct LanguageComponent {
    /// 中文版本
    pub zh: String,
    /// 英文版本
    pub en: String,
}

/// 单个提示词组件的完整数据（用于数据库存储）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PromptComponentData {
    /// 可编辑组件：Meta-Prompt
    pub meta_prompt: LanguageComponentWithMeta,
    /// 只读组件：输入信息模板
    pub input_template: LanguageComponent,
    /// 只读组件：输出格式模板
    pub output_template: LanguageComponent,
}

/// 带元数据的语言组件（用于可编辑组件）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LanguageComponentWithMeta {
    /// 中文版本
    pub zh: ComponentContent,
    /// 英文版本
    pub en: ComponentContent,
}

/// 组件内容（包含内容和最后修改时间）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ComponentContent {
    /// 组件内容
    pub content: String,
    /// 最后修改时间（RFC3339 格式）
    #[ts(type = "string")]
    pub last_modified: Option<String>,
}

impl From<LanguageComponent> for HashMap<String, ComponentContent> {
    fn from(component: LanguageComponent) -> Self {
        let mut map = HashMap::new();
        map.insert("zh".to_string(), ComponentContent {
            content: component.zh,
            last_modified: None,
        });
        map.insert("en".to_string(), ComponentContent {
            content: component.en,
            last_modified: None,
        });
        map
    }
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
            components: ComponentsConfig {
                meta_prompt: LanguageComponent {
                    zh: r#"你是一位专业的提示词工程师。你的任务是分析用户目标和会话历史，生成一个结构化、高信噪比的提示词，以指导 AI 编程助手高效地完成任务。

## 你的分析与构建步骤

1. **分析目标与上下文**:
   - **确立核心需求**: 仔细阅读用户目标。
   - **全局历史回顾**: 综合浏览整段历史会话（寻找主线任务），避免仅基于最近一轮对话做出片面判断。
   - **判断目标偏离度**: 根据上下文与下一步目标的关联程度，判断下一步目标对于整个会话来说是否噪音。
   - **确定会话阶段**: 明确当前处于任务探索、代码实现、调试修复还是重构优化阶段。

2. **构建结构化提示词**: 根据分析结果，填充以下"输出格式"的各个部分。

## 限制条件

- 提示词应简洁明了，重点突出，避免不必要的寒暄或背景信息
- 必须根据**上下文**与**下一步目标**的关联程度严格判断目标偏离程度，是否同一个项目不在判断范围内
- 严禁仅参考最新对话，必须建立对**整体历史会话**的认知
- 突出关键技术点和注意事项"#.trim().to_string(),
                    en: r#"You are a professional prompt engineer. Your task is to analyze user goals and conversation history to generate a structured, high signal-to-noise ratio prompt that will guide an AI programming assistant to efficiently complete tasks.

## Your Analysis and Construction Steps

1. **Analyze Goals and Context**:
   - **Establish Core Requirements**: Carefully read the user's goal.
   - **Global History Review**: Comprehensively review the entire conversation history (seeking the main task thread), avoiding making one-sided judgments based only on the most recent exchange.
   - **Determine Goal Drift**: Based on the relevance between context and next step, assess whether the next goal represents noise for the overall conversation.
   - **Identify Conversation Stage**: Determine whether you are in task exploration, implementation, debugging/refinement, or refactoring/optimization phase.

2. **Construct Structured Prompt**: Based on your analysis, populate each section of the "Output Format" below.

## Constraints

- The prompt should be concise and focused, avoiding unnecessary pleasantries or background information
- Must rigorously assess goal drift based on the relevance between **context** and **next step**, irrespective of whether it belongs to the same project
- Strictly prohibit referencing only the latest conversation; must establish cognition of the **entire conversation history**
- Highlight key technical points and considerations"#.trim().to_string(),
                },
                input_template: LanguageComponent {
                    zh: r#"## 输入信息

- **下一步目标**: {{goal}}
- **相关历史会话**:
{{sessions}}"#.trim().to_string(),
                    en: r#"## Input Information

- **Next Goal**: {{goal}}
- **Related Conversation History**:
{{sessions}}"#.trim().to_string(),
                },
                output_template: LanguageComponent {
                    zh: r#"## 输出格式

请严格按照以下结构生成提示词，并用 Markdown 标题标识：

### **目标偏离程度**

判断**下一步目标**与**相关历史会话**的偏离程度，给出你的建议（例如：可以继续会话、建议用户开启新的会话、如偏离程度高则截断回答直接建议用户开启新会话）

### **任务目标**

简要描述要完成的任务和目标。

### **具体步骤**

提供清晰、可执行的步骤列表，指导 AI 如何完成任务。

### **预期输出**

描述期望的最终产出物是什么，并说明其关键要求。

---

请基于上述信息，生成一个优化的提示词。"#.trim().to_string(),
                    en: r#"## Output Format

Please strictly follow the structure below to generate a prompt, using Markdown headings:

### **Goal Drift Level**

Assess the degree of drift between the **next goal** and **related conversation history**, and provide your recommendation (e.g., can continue conversation, suggest user start a new conversation, if drift is high then truncate response and directly suggest user start a new conversation)

### **Task Objective**

Briefly describe the task and objectives to be completed.

### **Specific Steps**

Provide clear, actionable step-by-step instructions to guide the AI in completing the task.

### **Expected Output**

Describe what the final deliverable should be and specify its key requirements.

---

Please generate an optimized prompt based on the information above."#.trim().to_string(),
                },
            },
            llm_params: LLMParamsConfig {
                temperature: 0.1,
                max_tokens: 1500,
                top_p: 0.9,
                frequency_penalty: 0.0,
                presence_penalty: 0.0,
            },
            session_context: SessionContextConfig {
                max_summary_length: 200,
                include_rating: true,
                include_project: true,
                session_format_zh: "- 会话 {{session_id}} {{#if project_name}}(项目: {{project_name}}){{/if}} {{#if rating}}(评分: {{rating}}){{/if}}:\n  {{summary}}".to_string(),
                session_format_en: "- Session {{session_id}} {{#if project_name}}(Project: {{project_name}}){{/if}} {{#if rating}}(Rating: {{rating}}){{/if}}:\n  {{summary}}".to_string(),
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
    fn read_config(&self) -> RwLockReadGuard<'_, OptimizerConfig> {
        self.config.read().unwrap_or_else(|e| {
            #[cfg(debug_assertions)]
            eprintln!("[ConfigManager] 读取配置读锁失败: {}，忽略 poison 状态继续使用", e);
            e.into_inner()
        })
    }

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
        self.read_config().clone()
    }

    /// 获取组件化提示词数据（用于数据库初始化）
    pub fn get_prompt_components_data(&self) -> PromptComponentData {
        let config = self.read_config();
        PromptComponentData {
            meta_prompt: LanguageComponentWithMeta {
                zh: ComponentContent {
                    content: config.components.meta_prompt.zh.clone(),
                    last_modified: Some(chrono::Utc::now().to_rfc3339()),
                },
                en: ComponentContent {
                    content: config.components.meta_prompt.en.clone(),
                    last_modified: Some(chrono::Utc::now().to_rfc3339()),
                },
            },
            input_template: config.components.input_template.clone(),
            output_template: config.components.output_template.clone(),
        }
    }

    /// 获取 Meta-Prompt 组件（根据语言）
    pub fn get_meta_prompt(&self, language: &str) -> String {
        let config = self.read_config();
        match language {
            "zh" => config.components.meta_prompt.zh.clone(),
            _ => config.components.meta_prompt.en.clone(),
        }
    }

    /// 获取输入信息模板（根据语言）
    pub fn get_input_template(&self, language: &str) -> String {
        let config = self.read_config();
        match language {
            "zh" => config.components.input_template.zh.clone(),
            _ => config.components.input_template.en.clone(),
        }
    }

    /// 获取输出格式模板（根据语言）
    pub fn get_output_template(&self, language: &str) -> String {
        let config = self.read_config();
        match language {
            "zh" => config.components.output_template.zh.clone(),
            _ => config.components.output_template.en.clone(),
        }
    }

    /// 获取完整的组装提示词（用于 LLM 生成）
    /// 组合顺序：meta_prompt + input_template + output_template
    pub fn get_assembled_prompt(&self, language: &str, goal: &str, sessions: &str) -> String {
        let meta_prompt = self.get_meta_prompt(language);
        let input_template = self.get_input_template(language);
        let output_template = self.get_output_template(language);

        // 替换占位符
        let input_section = input_template
            .replace("{{goal}}", goal)
            .replace("{{sessions}}", sessions);

        format!("{}\n\n{}\n\n{}", meta_prompt, input_section, output_template)
    }

    /// 获取会话格式化模板（根据语言）
    pub fn get_session_format(&self, language: &str) -> String {
        let config = self.read_config();
        match language {
            "zh" => config.session_context.session_format_zh.clone(),
            _ => config.session_context.session_format_en.clone(),
        }
    }

    /// 获取会话上下文配置
    pub fn get_session_context_config(&self) -> SessionContextConfig {
        self.read_config().session_context.clone()
    }

    /// 获取 LLM 参数
    pub fn get_llm_params(&self) -> LLMParamsConfig {
        self.read_config().llm_params.clone()
    }

    /// 获取最大摘要长度
    pub fn get_max_summary_length(&self) -> usize {
        self.read_config().session_context.max_summary_length
    }

    /// 是否包含评分
    pub fn include_rating(&self) -> bool {
        self.read_config().session_context.include_rating
    }

    /// 是否包含项目名
    pub fn include_project(&self) -> bool {
        self.read_config().session_context.include_project
    }

    /// 是否启用调试模式
    pub fn is_debug(&self) -> bool {
        self.read_config().advanced.debug
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
