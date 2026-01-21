use std::env;
use std::fs;
use std::path::PathBuf;

use ts_rs::TS;

use prism_forge::optimizer::config::{
    AdvancedConfig,
    CompressionConfig,
    FallbackConfig,
    LLMParamsConfig,
    MetaPromptConfig,
    OptimizerConfig,
    PromptStructureConfig,
    SessionContextConfig,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let output_dir = manifest_dir.join("..").join("src").join("types").join("generated");
    fs::create_dir_all(&output_dir)?;
    OptimizerConfig::export_to(output_dir.join("OptimizerConfig.ts"))?;
    MetaPromptConfig::export_to(output_dir.join("MetaPromptConfig.ts"))?;
    LLMParamsConfig::export_to(output_dir.join("LLMParamsConfig.ts"))?;
    PromptStructureConfig::export_to(output_dir.join("PromptStructureConfig.ts"))?;
    FallbackConfig::export_to(output_dir.join("FallbackConfig.ts"))?;
    SessionContextConfig::export_to(output_dir.join("SessionContextConfig.ts"))?;
    CompressionConfig::export_to(output_dir.join("CompressionConfig.ts"))?;
    AdvancedConfig::export_to(output_dir.join("AdvancedConfig.ts"))?;
    Ok(())
}
