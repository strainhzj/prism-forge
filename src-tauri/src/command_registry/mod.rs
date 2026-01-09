//! Command Registry Module
//!
//! 提供增强的命令注册和验证功能

pub mod registry;
pub mod errors;
pub mod initializer;
pub mod validator;
pub mod diagnostic;
pub mod error_handler;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests;

pub use registry::{
    CommandRegistry, CommandInfo, CommandStatus, CommandStatusInfo, 
    CommandHistoryEntry, CommandEventType
};
pub use errors::{CommandError as RegistryCommandError, ModuleError, DiagnosticError};
pub use initializer::{
    ModuleInitializer, Module, InitState, DependencyGraph, ModuleNode, DependencyEdge,
    ModuleType, DependencyType, RecoveryStrategy, HealthStatus, HealthCheckReport,
    DependencyIssue, SystemHealthStatus
};
pub use validator::CommandValidator;
pub use diagnostic::{DiagnosticTool, DiagnosticReport};
pub use error_handler::{
    EnhancedErrorHandler, ErrorCategory, ErrorSeverity, ErrorContext, ErrorResponse,
    ErrorPattern, Logger, DefaultLogger, ErrorStatistics
};