//! 决策检测器（规则引擎）
//!
//! 基于规则引擎检测问答对中的决策点
//! 支持多语言和决策类型分类

use crate::database::decision_keywords::{DecisionKeyword, DecisionKeywordRepository};
use crate::database::get_db_path;
use crate::intent_analyzer::qa_detector::DecisionQAPair;
use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 决策点数据结构
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(rename_all = "camelCase")]
pub struct DecisionPoint {
    /// 决策类型
    /// - architecture_design: 架构设计
    /// - technology_choice: 技术选型
    /// - tool_selection: 工具选择
    /// - implementation: 代码实现
    /// - other: 其他
    pub decision_type: String,
    /// 决策内容（一句话描述）
    pub decision_made: String,
    /// 明确理由（用户提及的理由）
    pub rationale: Vec<String>,
    /// 推测理由（基于上下文推断）
    pub inferred_reasons: Vec<String>,
    /// 备选方案
    pub alternatives: Vec<Alternative>,
    /// 匹配到的关键词
    pub matched_keywords: Vec<String>,
}

/// 备选方案
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(rename_all = "camelCase")]
pub struct Alternative {
    /// 备选方案名称
    pub name: String,
}

/// 决策检测器（规则引擎）
pub struct DecisionDetector {
    /// 理由提取正则表达式
    rationale_regex: Regex,
}

impl DecisionDetector {
    /// 创建新的决策检测器
    pub fn new() -> Result<Self> {
        Ok(Self {
            // 匹配常见的理由表达模式
            rationale_regex: Regex::new(r"(因为|由于|理由是|reason:?)")?,
        })
    }

    /// 检测问答对中的决策点
    ///
    /// 根据语言和上下文自动匹配决策关键词
    pub fn detect_decisions(&self, qa_pair: &DecisionQAPair, language: &str) -> Result<Vec<DecisionPoint>> {
        let db_path = get_db_path()?;
        let db_path_str = db_path.to_string_lossy().to_string();
        let repo = DecisionKeywordRepository::new(db_path_str);

        // 加载指定语言的激活关键词
        let keywords = repo.get_by_language(language)
            .unwrap_or_default();

        if keywords.is_empty() {
            return Ok(Vec::new());
        }

        let mut decisions = Vec::new();

        // 合并用户决策和助手回答
        let combined_text = format!(
            "{} {}",
            qa_pair.user_decision,
            qa_pair.assistant_answer
        );

        // 按权重排序的关键词（高权重优先）
        let sorted_keywords: Vec<&DecisionKeyword> = keywords.iter()
            .filter(|k| combined_text.contains(&k.keyword))
            .collect();

        if !sorted_keywords.is_empty() {
            // 使用权重最高的关键词进行决策类型分类
            let best_keyword = sorted_keywords
                .iter()
                .max_by_key(|k| (k.weight * 1000.0) as i64)
                .unwrap();

            let matched_keywords: Vec<String> = sorted_keywords.iter()
                .map(|k| k.keyword.clone())
                .collect();

            let decision = self.extract_decision(
                &combined_text,
                best_keyword,
                &matched_keywords,
            )?;
            decisions.push(decision);
        }

        Ok(decisions)
    }

    /// 提取决策信息
    fn extract_decision(
        &self,
        text: &str,
        keyword: &DecisionKeyword,
        all_keywords: &[String],
    ) -> Result<DecisionPoint> {
        // 提取决策内容
        let decision_made = self.extract_decision_content(text, &keyword.keyword);

        // 提取决策理由
        let rationale = self.extract_rationale(text);

        // 推测理由（简化版）
        let inferred_reasons = if rationale.is_empty() {
            vec![
                format!("用户使用了\"{}\"关键词", keyword.keyword),
                "基于上下文推断的决策".to_string(),
            ]
        } else {
            Vec::new()
        };

        // 提取备选方案（简化版）
        let alternatives = self.extract_alternatives(text);

        Ok(DecisionPoint {
            decision_type: keyword.decision_type.clone(),
            decision_made,
            rationale,
            inferred_reasons,
            alternatives,
            matched_keywords: all_keywords.to_owned(),
        })
    }

    /// 提取决策内容
    fn extract_decision_content(&self, text: &str, keyword: &str) -> String {
        // 查找关键词位置
        if let Some(pos) = text.find(keyword) {
            let after_keyword = &text[pos..];

            // 查找句子结束标记
            let end_markers = ['。', '.', '!', '！', '\n', '\r'];
            let mut end_pos = after_keyword.len();

            for marker in end_markers {
                if let Some(found) = after_keyword.find(marker) {
                    if found < end_pos {
                        end_pos = found;
                    }
                }
            }

            // 限制最大长度（100 字符）
            let max_len = 100.min(end_pos);
            let content = after_keyword[..max_len].trim();

            // 清理内容
            content
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            String::new()
        }
    }

    /// 提取决策理由
    fn extract_rationale(&self, text: &str) -> Vec<String> {
        let mut rationales = Vec::new();

        for capture in self.rationale_regex.captures_iter(text) {
            if let Some(matched) = capture.get(0) {
                let rationale = matched.as_str().trim().to_string();
                if !rationale.is_empty() {
                    rationales.push(rationale);
                }
            }
        }

        rationales
    }

    /// 提取备选方案
    ///
    /// 查找常见的备选方案表达模式
    fn extract_alternatives(&self, text: &str) -> Vec<Alternative> {
        let mut alternatives = Vec::new();

        // 查找 "还是" 或 "or" 连接的表达
        let or_patterns = vec![
            r"(?P<alt1>[^。！.!?]+)[还是|or](?P<alt2>[^。！.!?]+)",
            r"[不是|not][^是]+[而是|but][^是]+(?P<alt>[^。！.!?]+)",
        ];

        for pattern in or_patterns {
            if let Ok(re) = Regex::new(pattern) {
                for capture in re.captures_iter(text) {
                    // 尝试提取所有命名组
                    for name in re.capture_names().flatten() {
                        if let Some(matched) = capture.name(name) {
                            let alt = matched.as_str().trim().to_string();
                            if !alt.is_empty() && alt.len() < 100 {
                                alternatives.push(Alternative { name: alt });
                            }
                        }
                    }
                }
            }
        }

        // 限制备选方案数量（最多 5 个）
        alternatives.truncate(5);
        alternatives
    }

    /// 根据文本自动检测语言
    pub fn detect_language(&self, text: &str) -> String {
        let chinese_regex = Regex::new(r"[\u4e00-\u9fff]").unwrap();
        let chinese_count = chinese_regex.find_iter(text).count();

        // 如果中文字符数量 > 总字符数的 20%，判定为中文
        if chinese_count > 0 && (chinese_count as f64 / text.len() as f64) > 0.2 {
            "zh".to_string()
        } else {
            "en".to_string()
        }
    }
}

impl Default for DecisionDetector {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language_chinese() {
        let detector = DecisionDetector::new().unwrap();
        assert_eq!(detector.detect_language("我们选择使用 Rust 和 React"), "zh");
        assert_eq!(detector.detect_language("我决定采用 Tauri 框架"), "zh");
    }

    #[test]
    fn test_detect_language_english() {
        let detector = DecisionDetector::new().unwrap();
        assert_eq!(detector.detect_language("We choose to use Rust and React"), "en");
        assert_eq!(detector.detect_language("I decided to adopt Tauri framework"), "en");
    }

    #[test]
    fn test_detect_language_mixed() {
        let detector = DecisionDetector::new().unwrap();
        // 中文字符占比 > 20%，判定为中文
        // "我们选择使用Rust" - 6个中文字符，总共9个字符，占比约67%
        assert_eq!(detector.detect_language("我们选择使用Rust"), "zh");
        // 纯英文
        assert_eq!(detector.detect_language("We choose Rust"), "en");
        // 中英文混合但中文占主导 - "这是中文有很多字" vs "English"
        assert_eq!(detector.detect_language("这是中文有很多字English"), "zh");
    }

    #[test]
    fn test_extract_decision_content() {
        let detector = DecisionDetector::new().unwrap();

        let text = "我们选择使用 Rust 来实现后端，因为它性能优秀。";
        let content = detector.extract_decision_content(text, "选择");
        assert!(content.contains("Rust"));

        let text = "We choose to use React for the frontend.";
        let content = detector.extract_decision_content(text, "choose");
        assert!(content.contains("React"));
    }

    #[test]
    fn test_extract_rationale() {
        let detector = DecisionDetector::new().unwrap();

        let text = "我们选择使用 Rust，因为它性能优秀。";
        let rationale = detector.extract_rationale(text);
        assert!(!rationale.is_empty());
        assert!(rationale[0].contains("因为"));
    }

    #[test]
    fn test_extract_alternatives() {
        let detector = DecisionDetector::new().unwrap();

        let text = "我们选择 Rust 而不是 Python。";
        let alternatives = detector.extract_alternatives(text);
        assert!(!alternatives.is_empty());

        let text = "We choose Rust instead of Python.";
        let alternatives = detector.extract_alternatives(text);
        assert!(!alternatives.is_empty());
    }

    #[test]
    fn test_detect_decisions_with_qa_pair() {
        let detector = DecisionDetector::new().unwrap();

        let qa_pair = DecisionQAPair {
            qa_index: 0,
            assistant_answer_uuid: "a1".to_string(),
            user_decision_uuid: "u1".to_string(),
            assistant_answer: "我建议使用 Rust 实现后端。".to_string(),
            user_decision: "好的，我们选择使用 Rust。".to_string(),
            context_qa_pairs: None,
        };

        let decisions = detector.detect_decisions(&qa_pair, "zh").unwrap();
        // 应该检测到决策（假设数据库中有"选择"关键词）
        assert!(!decisions.is_empty());
    }

    #[test]
    fn test_detect_decisions_no_keywords() {
        let detector = DecisionDetector::new().unwrap();

        let qa_pair = DecisionQAPair {
            qa_index: 0,
            assistant_answer_uuid: "a1".to_string(),
            user_decision_uuid: "u1".to_string(),
            assistant_answer: "这是一个普通的问题。".to_string(),
            user_decision: "这是一个普通的回答。".to_string(),
            context_qa_pairs: None,
        };

        let decisions = detector.detect_decisions(&qa_pair, "zh").unwrap();
        // 没有关键词，应该返回空
        assert!(decisions.is_empty());
    }
}
