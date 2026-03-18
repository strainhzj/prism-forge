//! 意图分析模块
//!
//! 用于分析 Claude 会话中的用户意图、决策点和技术栈

pub mod decision_analyzer;
pub mod decision_detector;
pub mod language;
pub mod opening_intent;
pub mod qa_detector;
pub mod tech_stack_detector;

pub use decision_analyzer::{Alternative, DecisionAnalysis, DecisionAnalyzer, DecisionType};
pub use decision_detector::{Alternative as DetectorAlternative, DecisionDetector, DecisionPoint};
pub use language::LanguageDetector;
pub use opening_intent::{OpeningIntent, OpeningIntentAnalyzer};
pub use qa_detector::{DecisionQAPair, QAPairDetector};
pub use tech_stack_detector::TechStackDetector;
