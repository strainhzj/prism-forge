use std::env;
use std::fs;
use std::path::PathBuf;

use ts_rs::TS;

use prism_forge::optimizer::config::{
    AdvancedConfig,
    CompressionConfig,
    ComponentsConfig,
    ComponentContent,
    LanguageComponent,
    LanguageComponentWithMeta,
    LLMParamsConfig,
    OptimizerConfig,
    PromptComponentData,
    SessionContextConfig,
};
use prism_forge::optimizer::prompt_generator::{
    EnhancedPrompt,
    EnhancedPromptRequest,
    ReferencedSession,
    SessionMessage,
};
use prism_forge::database::models::{
    Prompt, PromptGenerationHistory, TokenStats,
    PromptTemplate, PromptVersion, PromptComponent, PromptComponentType,
    PromptParameter, PromptParameterType, PromptChange, ChangeType,
    PromptVersionDiff, ComponentDiff, LineDiff, LineChangeType,
    ParameterDiff, MetadataDiff, RollbackRecord,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let output_dir = manifest_dir.join("..").join("src").join("types").join("generated");
    fs::create_dir_all(&output_dir)?;

    // Optimizer config types
    OptimizerConfig::export_to(output_dir.join("OptimizerConfig.ts"))?;
    ComponentsConfig::export_to(output_dir.join("ComponentsConfig.ts"))?;
    LanguageComponent::export_to(output_dir.join("LanguageComponent.ts"))?;
    LanguageComponentWithMeta::export_to(output_dir.join("LanguageComponentWithMeta.ts"))?;
    ComponentContent::export_to(output_dir.join("ComponentContent.ts"))?;
    PromptComponentData::export_to(output_dir.join("PromptComponentData.ts"))?;
    LLMParamsConfig::export_to(output_dir.join("LLMParamsConfig.ts"))?;
    SessionContextConfig::export_to(output_dir.join("SessionContextConfig.ts"))?;
    CompressionConfig::export_to(output_dir.join("CompressionConfig.ts"))?;
    AdvancedConfig::export_to(output_dir.join("AdvancedConfig.ts"))?;

    // Database models
    Prompt::export_to(output_dir.join("Prompt.ts"))?;
    PromptGenerationHistory::export_to(output_dir.join("PromptGenerationHistory.ts"))?;
    TokenStats::export_to(output_dir.join("TokenStats.ts"))?;

    // Prompt version management types
    PromptTemplate::export_to(output_dir.join("PromptTemplate.ts"))?;
    PromptVersion::export_to(output_dir.join("PromptVersion.ts"))?;
    PromptComponent::export_to(output_dir.join("PromptComponent.ts"))?;
    PromptComponentType::export_to(output_dir.join("PromptComponentType.ts"))?;
    PromptParameter::export_to(output_dir.join("PromptParameter.ts"))?;
    PromptParameterType::export_to(output_dir.join("PromptParameterType.ts"))?;
    PromptChange::export_to(output_dir.join("PromptChange.ts"))?;
    ChangeType::export_to(output_dir.join("ChangeType.ts"))?;
    PromptVersionDiff::export_to(output_dir.join("PromptVersionDiff.ts"))?;
    ComponentDiff::export_to(output_dir.join("ComponentDiff.ts"))?;
    LineDiff::export_to(output_dir.join("LineDiff.ts"))?;
    LineChangeType::export_to(output_dir.join("LineChangeType.ts"))?;
    ParameterDiff::export_to(output_dir.join("ParameterDiff.ts"))?;
    MetadataDiff::export_to(output_dir.join("MetadataDiff.ts"))?;
    RollbackRecord::export_to(output_dir.join("RollbackRecord.ts"))?;

    // Prompt generator types
    EnhancedPrompt::export_to(output_dir.join("EnhancedPrompt.ts"))?;
    EnhancedPromptRequest::export_to(output_dir.join("EnhancedPromptRequest.ts"))?;
    ReferencedSession::export_to(output_dir.join("ReferencedSession.ts"))?;
    SessionMessage::export_to(output_dir.join("SessionMessage.ts"))?;

    Ok(())
}
