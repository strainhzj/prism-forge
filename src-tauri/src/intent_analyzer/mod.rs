//! 意图分析模块
//!
//! 用于分析 Claude 会话中的用户意图、决策点和技术栈

pub mod language;
pub mod opening_intent;
pub mod qa_detector;
pub mod tech_stack_detector;

pub use language::LanguageDetector;
pub use opening_intent::{OpeningIntent, OpeningIntentAnalyzer};
pub use qa_detector::{DecisionQAPair, QAPairDetector};
pub use tech_stack_detector::TechStackDetector;
