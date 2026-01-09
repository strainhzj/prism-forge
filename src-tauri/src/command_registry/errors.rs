//! Error types for command registration system

use std::fmt;
use serde::{Serialize, Deserialize};

/// Command registration and execution errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandError {
    pub message: String,
    pub error_type: ErrorType,
    pub context: Option<String>,
    pub timestamp: std::time::SystemTime,
}

/// Module initialization errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleError {
    pub module_name: String,
    pub message: String,
    pub error_type: ModuleErrorType,
}

/// Diagnostic tool errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticError {
    pub message: String,
    pub error_type: DiagnosticErrorType,
}

/// Types of command errors
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ErrorType {
    CommandNotFound,
    RegistrationFailed,
    ValidationFailed,
    DependencyMissing,
    RuntimeError,
}

/// Types of module errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModuleErrorType {
    InitializationFailed,
    DependencyMissing,
    HealthCheckFailed,
    ShutdownFailed,
}

/// Types of diagnostic errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagnosticErrorType {
    ReportGenerationFailed,
    ExportFailed,
    AnalysisFailed,
}

impl CommandError {
    pub fn new(message: String, error_type: ErrorType) -> Self {
        Self {
            message,
            error_type,
            context: None,
            timestamp: std::time::SystemTime::now(),
        }
    }

    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }

    pub fn command_not_found(command_name: &str) -> Self {
        Self::new(
            format!("Command '{}' not found", command_name),
            ErrorType::CommandNotFound,
        )
    }

    pub fn registration_failed(command_name: &str, reason: &str) -> Self {
        Self::new(
            format!("Failed to register command '{}': {}", command_name, reason),
            ErrorType::RegistrationFailed,
        )
    }

    pub fn dependency_missing(command_name: &str, dependency: &str) -> Self {
        Self::new(
            format!("Command '{}' missing dependency: {}", command_name, dependency),
            ErrorType::DependencyMissing,
        )
    }
}

impl ModuleError {
    pub fn new(module_name: String, message: String, error_type: ModuleErrorType) -> Self {
        Self {
            module_name,
            message,
            error_type,
        }
    }

    pub fn initialization_failed(module_name: &str, reason: &str) -> Self {
        Self::new(
            module_name.to_string(),
            format!("Module initialization failed: {}", reason),
            ModuleErrorType::InitializationFailed,
        )
    }

    pub fn dependency_missing(module_name: &str, dependency: &str) -> Self {
        Self::new(
            module_name.to_string(),
            format!("Missing dependency: {}", dependency),
            ModuleErrorType::DependencyMissing,
        )
    }
}

impl DiagnosticError {
    pub fn new(message: String, error_type: DiagnosticErrorType) -> Self {
        Self {
            message,
            error_type,
        }
    }

    pub fn report_generation_failed(reason: &str) -> Self {
        Self::new(
            format!("Report generation failed: {}", reason),
            DiagnosticErrorType::ReportGenerationFailed,
        )
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(context) = &self.context {
            write!(f, " (Context: {})", context)?;
        }
        Ok(())
    }
}

impl fmt::Display for ModuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.module_name, self.message)
    }
}

impl fmt::Display for DiagnosticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CommandError {}
impl std::error::Error for ModuleError {}
impl std::error::Error for DiagnosticError {}

impl From<anyhow::Error> for CommandError {
    fn from(err: anyhow::Error) -> Self {
        Self::new(err.to_string(), ErrorType::RuntimeError)
    }
}