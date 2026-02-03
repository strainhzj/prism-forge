//! 开场白意图分析器
//!
//! 用于分析 Claude 会话开场白的用户意图

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::database::models::Message;
use crate::database::prompt_versions::PromptVersionRepository;
use crate::llm::interface::{Message as LLMMessage, ModelParams};
use crate::llm::LLMClientManager;

/// 开场白意图分析结果
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(rename_all = "camelCase")]
pub struct OpeningIntent {
    /// 意图类型
    pub intent_type: String,
    /// 置信度 (0.0 - 1.0)
    pub confidence: f32,
    /// 意图描述
    pub description: Option<String>,
    /// 提取的关键信息
    pub key_info: Vec<String>,
}

/// 开场白意图分析器
pub struct OpeningIntentAnalyzer {
    /// 提示词版本仓库
    prompt_repo: PromptVersionRepository,
}

impl OpeningIntentAnalyzer {
    /// 创建新的分析器
    pub fn new() -> Result<Self> {
        let prompt_repo = PromptVersionRepository::from_default_db()?;
        Ok(Self { prompt_repo })
    }

    /// 分析开场白意图
    ///
    /// # 参数
    ///
    /// - `opening_message`: 开场白消息（第一个 user 消息）
    /// - `language`: 语言标识（"zh" 或 "en"）
    /// - `llm_manager`: LLM 客户端管理器
    pub async fn analyze(
        &self,
        opening_message: &Message,
        language: &str,
        llm_manager: &LLMClientManager,
    ) -> Result<OpeningIntent> {
        // 1. 加载提示词模板
        let template = self
            .prompt_repo
            .get_template_by_scenario("opening_intent_analysis")?
            .ok_or_else(|| anyhow::anyhow!("未找到开场白意图分析提示词模板"))?;

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

        // 4. 构建用户消息
        let user_message = opening_message
            .content
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("开场白内容为空"))?;

        let full_prompt = format!(
            "{}\n\n{}\n\n{}",
            meta_prompt,
            input_template.replace("{{opening_message}}", user_message),
            output_template
        );

        // 5. 获取 LLM 客户端
        let client = llm_manager.get_active_client()?;

        // 6. 获取提供商配置
        let provider = llm_manager.get_active_provider_config()?;
        let model = provider.effective_model();

        // 7. 创建参数
        let params = ModelParams::new(model)
            .with_temperature(0.1)
            .with_max_tokens(1000);

        // 8. 调用 LLM
        let messages = vec![LLMMessage::user(full_prompt)];
        let response = client.chat_completion(messages, params).await?;

        // 9. 解析 JSON 响应
        let result: OpeningIntent = serde_json::from_str(&response.content)
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
        let analyzer = OpeningIntentAnalyzer::new();
        assert!(analyzer.is_ok());
    }
}
