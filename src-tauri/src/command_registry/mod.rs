//! Command Registry Module
//!
//! 提供增强的命令注册和验证功能

pub mod diagnostic;
pub mod error_handler;
pub mod errors;
pub mod initializer;
pub mod registry;
pub mod validator;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests;

pub use diagnostic::{DiagnosticReport, DiagnosticTool};
pub use error_handler::{
    DefaultLogger, EnhancedErrorHandler, ErrorCategory, ErrorContext, ErrorPattern, ErrorResponse,
    ErrorSeverity, ErrorStatistics, Logger,
};
pub use errors::{CommandError as RegistryCommandError, DiagnosticError, ModuleError};
pub use initializer::{
    DependencyEdge, DependencyGraph, DependencyIssue, DependencyType, HealthCheckReport,
    HealthStatus, InitState, Module, ModuleInitializer, ModuleNode, ModuleType, RecoveryStrategy,
    SystemHealthStatus,
};
pub use registry::{
    CommandEventType, CommandHistoryEntry, CommandInfo, CommandRegistry, CommandStatus,
    CommandStatusInfo,
};
pub use validator::CommandValidator;
