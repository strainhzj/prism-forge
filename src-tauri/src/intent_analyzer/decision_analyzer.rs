//! 问答对决策分析器
//!
//! 基于问答对（助手回答 + 用户后续决策）分析用户的技术决策
//!
//! # 功能
//!
//! - 加载 `decision_analysis` 提示词模板
//! - 调用 LLM API 分析决策
//! - 解析 JSON 结果

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::database::prompt_versions::PromptVersionRepository;
use crate::intent_analyzer::qa_detector::DecisionQAPair;
use crate::llm::interface::{Message as LLMMessage, ModelParams};
use crate::llm::LLMClientManager;

/// 决策类型（固定枚举）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(rename_all = "camelCase")]
pub enum DecisionType {
    /// 技术选型（选择编程语言、框架、库等）
    TechnologyChoice,
    /// 架构设计（系统架构、模块划分、数据流等）
    ArchitectureDesign,
    /// 工具选择（开发工具、部署方案、第三方服务等）
    ToolSelection,
    /// 代码实现（具体实现方式、算法选择、代码模式等）
    Implementation,
    /// 其他类型
    Other,
}

/// 备选方案
///
/// 用户考虑过但未选择的方案
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(rename_all = "camelCase")]
pub struct Alternative {
    /// 备选方案名称
    pub name: String,
    /// 用户提供的未选择理由（可选，由用户补充，非 LLM 生成）
    pub reason: Option<String>,
}

/// 决策分析结果
///
/// 表示用户在问答对中做出的技术决策分析
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(rename_all = "camelCase")]
pub struct DecisionAnalysis {
    /// 决策内容（一句话总结）
    pub decision_made: String,
    /// 决策类型（固定枚举值）
    pub decision_type: DecisionType,
    /// 涉及的技术栈
    pub tech_stack: Vec<String>,
    /// 明确理由（用户提及的理由）
    pub rationale: Vec<String>,
    /// 推测理由（LLM 分析推断）
    pub inferred_reasons: Vec<String>,
    /// 备选方案
    pub alternatives: Vec<Alternative>,
    /// 置信度（0-1）
    pub confidence: f64,
}

/// 决策分析器
///
/// 负责分析问答对中的技术决策
pub struct DecisionAnalyzer {
    /// 提示词版本仓库
    prompt_repo: PromptVersionRepository,
}

impl DecisionAnalyzer {
    /// 创建新的决策分析器
    ///
    /// # 返回
    ///
    /// - `Result<Self>`: 成功返回分析器实例，失败返回错误
    pub fn new() -> Result<Self> {
        let prompt_repo = PromptVersionRepository::from_default_db()?;
        Ok(Self { prompt_repo })
    }

    /// 分析问答对决策
    ///
    /// # 参数
    ///
    /// - `qa_pair`: 问答对（助手回答 + 用户后续决策）
    /// - `language`: 语言代码（"zh" 或 "en"）
    /// - `llm_manager`: LLM 客户端管理器
    ///
    /// # 返回
    ///
    /// - `Result<DecisionAnalysis>`: 成功返回决策分析结果，失败返回错误
    pub async fn analyze(
        &self,
        qa_pair: &DecisionQAPair,
        language: &str,
        llm_manager: &LLMClientManager,
    ) -> Result<DecisionAnalysis> {
        // 1. 加载提示词模板
        let template = self
            .prompt_repo
            .get_template_by_scenario("decision_analysis")?
            .ok_or_else(|| anyhow::anyhow!("未找到决策分析提示词模板"))?;

        // 2. 获取激活版本
        let template_id = template.id.ok_or_else(|| anyhow::anyhow!("模板 ID 缺失"))?;
        let version = self
            .prompt_repo
            .get_active_version(template_id)?
            .ok_or_else(|| anyhow::anyhow!("未找到激活版本"))?;

        // 3. 解析组件化内容
        let content_json: serde_json::Value =
            serde_json::from_str(&version.content).context("解析提示词内容失败")?;

        // 根据语言选择对应的内容
        let lang_key = if language == "zh" { "zh" } else { "en" };
        let meta_prompt = content_json[lang_key]["meta_prompt"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("meta_prompt 内容缺失"))?;
        let input_template = content_json[lang_key]["input_template"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("input_template 内容缺失"))?;
        let output_template = content_json[lang_key]["output_template"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("output_template 内容缺失"))?;

        // 4. 构建上下文格式化字符串
        let context_section = if let Some(ref context_pairs) = qa_pair.context_qa_pairs {
            if !context_pairs.is_empty() {
                // 按照 PromptGenerator 的格式构建上下文
                let formatted_pairs: Vec<String> = context_pairs
                    .iter()
                    .enumerate()
                    .map(|(idx, pair)| {
                        format!(
                            "{}. 用户: \"{}\"\n   助手: \"{}\"",
                            idx + 1,
                            pair.user_question,
                            pair.assistant_answer
                        )
                    })
                    .collect();

                format!(
                    "### 对话上下文（前序问答对）\n\n{}\n\n",
                    formatted_pairs.join("\n\n")
                )
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // 5. 构建用户消息
        let full_prompt = format!(
            "{}\n\n{}\n\n{}### 当前决策分析\n\n- **用户后续决策**: \"{}\"\n\n{}",
            meta_prompt,
            context_section,
            input_template.replace("{{assistant_answer}}", &qa_pair.assistant_answer),
            qa_pair.user_decision,
            output_template
        );

        // 6. 获取 LLM 客户端
        let client = llm_manager.get_active_client()?;

        // 7. 获取提供商配置
        let provider = llm_manager.get_active_provider_config()?;
        let model = provider.effective_model();

        // 8. 创建参数
        let params = ModelParams::new(model)
            .with_temperature(0.1)
            .with_max_tokens(1500);

        // 9. 调用 LLM
        let messages = vec![LLMMessage::user(full_prompt)];
        let response = client.chat_completion(messages, params).await?;

        // 10. 解析 JSON 响应
        let result: DecisionAnalysis = serde_json::from_str(&response.content)
            .with_context(|| format!("解析 LLM 响应失败: {}", response.content))?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_success() {
        // 验证 new() 方法成功创建分析器
        let analyzer = DecisionAnalyzer::new();
        assert!(analyzer.is_ok());
    }

    #[test]
    fn test_decision_type_serialization() {
        // 验证决策类型序列化
        let tech_choice = DecisionType::TechnologyChoice;
        let json = serde_json::to_string(&tech_choice).unwrap();
        assert!(json.contains("TechnologyChoice"));
    }

    #[test]
    fn test_alternative_serialization() {
        // 验证备选方案序列化
        let alt = Alternative {
            name: "Electron".to_string(),
            reason: Some("性能较差".to_string()),
        };
        let json = serde_json::to_string(&alt).unwrap();
        assert!(json.contains("Electron"));
        assert!(json.contains("性能较差"));
    }

    #[test]
    fn test_decision_analysis_deserialization() {
        // 验证决策分析结果反序列化
        let json_str = r#"{
            "decision_made": "选择使用 Rust 开发",
            "decision_type": "TechnologyChoice",
            "tech_stack": ["Rust", "Tauri"],
            "rationale": ["性能要求高"],
            "inferred_reasons": ["用户熟悉 Rust"],
            "alternatives": [{"name": "Electron", "reason": "性能较差"}],
            "confidence": 0.9
        }"#;

        let result: DecisionAnalysis = serde_json::from_str(json_str).unwrap();
        assert_eq!(result.decision_made, "选择使用 Rust 开发");
        assert!(matches!(result.decision_type, DecisionType::TechnologyChoice));
        assert_eq!(result.tech_stack.len(), 2);
        assert_eq!(result.rationale.len(), 1);
        assert_eq!(result.alternatives.len(), 1);
        assert_eq!(result.confidence, 0.9);
    }
}
