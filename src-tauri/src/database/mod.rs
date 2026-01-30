//! 数据库模块
//!
//! 包含数据模型和数据库操作

pub mod models;
pub mod migrations;
pub mod repository;
pub mod init;
pub mod vector_repository;
pub mod prompt_versions;
pub mod init_default_prompts;

pub use models::{
    ApiProvider, ApiProviderType,
    Session, Message, MessageEmbedding, SessionEmbedding, SavedPrompt, MetaTemplate,
    TokenStats, validate_timestamp, VectorSearchResult,
    PromptGenerationHistory,
    Prompt,
    // Prompt version management
    PromptTemplate, PromptVersion, PromptComponent, PromptComponentType,
    PromptParameter, PromptParameterType, PromptChange, ChangeType,
    PromptVersionDiff, ComponentDiff, LineDiff, LineChangeType,
    ParameterDiff, MetadataDiff, RollbackRecord,
};
pub use migrations::{get_db_path, initialize_database, get_connection};
pub use repository::{ApiProviderRepository, PromptHistoryRepository};
pub use vector_repository::VectorRepository;
pub use prompt_versions::PromptVersionRepository;
pub use init::{get_connection_shared, get_db_path as get_db_path_init};
