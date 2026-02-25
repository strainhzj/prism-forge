//! 数据库模块
//!
//! 包含数据模型和数据库操作

pub mod init;
pub mod init_default_prompts;
pub mod decision_keywords;
pub mod migrations;
pub mod models;
pub mod prompt_versions;
pub mod repository;
pub mod repositories_tech_stack;
pub mod vector_repository;

pub use init::{get_connection_shared, get_db_path as get_db_path_init};
pub use migrations::{get_connection, get_db_path, initialize_database};
pub use decision_keywords::{DecisionKeyword, DecisionKeywordRepository};
pub use repositories_tech_stack::{ProjectTechStack, ProjectTechStackRepository};
pub use models::{
    validate_timestamp,
    ApiProvider,
    ApiProviderType,
    ChangeType,
    ComponentDiff,
    LineChangeType,
    LineDiff,
    Message,
    MessageEmbedding,
    MetaTemplate,
    MetadataDiff,
    ParameterDiff,
    Prompt,
    PromptChange,
    PromptComponent,
    PromptComponentType,
    PromptGenerationHistory,
    PromptParameter,
    PromptParameterType,
    // Prompt version management
    PromptTemplate,
    PromptVersion,
    PromptVersionDiff,
    RollbackRecord,
    SavedPrompt,
    Session,
    SessionEmbedding,
    TokenStats,
    VectorSearchResult,
};
pub use prompt_versions::PromptVersionRepository;
pub use repository::{ApiProviderRepository, PromptHistoryRepository};
pub use vector_repository::VectorRepository;
