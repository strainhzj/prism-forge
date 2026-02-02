//! 意图分析模块
//!
//! 用于分析 Claude 会话中的用户意图、决策点和技术栈

pub mod qa_detector;
pub mod language;

pub use qa_detector::{QAPairDetector, DecisionQAPair};
pub use language::LanguageDetector;
