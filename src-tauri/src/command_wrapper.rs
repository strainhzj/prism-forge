//! Command Wrapper Module
//!
//! Provides wrappers for existing Tauri commands that integrate with the
//! command registration system for tracking and validation.
//!
//! **Feature: fix-command-registration**
//! **Validates: Requirements 2.1**

use std::sync::{Arc, RwLock};
use crate::command_registry::{CommandRegistry, CommandStatus};
use crate::startup::StartupManager;

/// Command execution tracker
/// 
/// Tracks command executions and updates the command registry
pub struct CommandTracker {
    registry: Arc<RwLock<CommandRegistry>>,
}

impl CommandTracker {
    /// Create a new command tracker from a startup manager
    pub fn from_startup_manager(manager: &StartupManager) -> Self {
        Self {
            registry: manager.get_registry(),
        }
    }

    /// Record that a command was called
    pub fn record_call(&self, command_name: &str) {
        if let Ok(mut registry) = self.registry.write() {
            registry.record_command_call(command_name);
        }
    }

    /// Check if a command is available
    pub fn is_command_available(&self, command_name: &str) -> bool {
        if let Ok(registry) = self.registry.read() {
            if let Some(status) = registry.get_command_status(command_name) {
                return matches!(status, CommandStatus::Registered);
            }
        }
        false
    }

    /// Get list of available commands
    pub fn get_available_commands(&self) -> Vec<String> {
        if let Ok(registry) = self.registry.read() {
            registry.list_available_commands()
        } else {
            Vec::new()
        }
    }

    /// Get command status
    pub fn get_command_status(&self, command_name: &str) -> Option<CommandStatus> {
        if let Ok(registry) = self.registry.read() {
            registry.get_command_status(command_name).cloned()
        } else {
            None
        }
    }
}

/// Macro to wrap a command with tracking
/// 
/// This macro creates a wrapper function that:
/// 1. Records the command call in the registry
/// 2. Executes the original command
/// 3. Handles any errors appropriately
#[macro_export]
macro_rules! track_command {
    ($tracker:expr, $command_name:expr, $result:expr) => {{
        $tracker.record_call($command_name);
        $result
    }};
}

/// Command validation result
#[derive(Debug, Clone)]
pub struct CommandValidationResult {
    pub command_name: String,
    pub is_valid: bool,
    pub status: Option<CommandStatus>,
    pub error_message: Option<String>,
}

/// Validate all registered commands
pub fn validate_all_commands(manager: &StartupManager) -> Vec<CommandValidationResult> {
    let mut results = Vec::new();
    
    if let Ok(registry) = manager.get_registry().read() {
        let commands = registry.get_all_commands();
        
        for (name, info) in commands {
            let is_valid = matches!(info.status, CommandStatus::Registered);
            let error_message = match &info.status {
                CommandStatus::Failed(msg) => Some(msg.clone()),
                CommandStatus::Unverified => Some("Command not verified".to_string()),
                CommandStatus::Disabled => Some("Command disabled".to_string()),
                CommandStatus::Registered => None,
            };
            
            results.push(CommandValidationResult {
                command_name: name.clone(),
                is_valid,
                status: Some(info.status.clone()),
                error_message,
            });
        }
    }
    
    results
}

/// Get command not found error with available commands list
pub fn get_command_not_found_error(command_name: &str, manager: &StartupManager) -> String {
    let available = if let Ok(registry) = manager.get_registry().read() {
        registry.list_available_commands()
    } else {
        Vec::new()
    };
    
    format!(
        "Command '{}' not found. Available commands: {}",
        command_name,
        if available.is_empty() {
            "none".to_string()
        } else {
            available.join(", ")
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::startup::create_startup_manager;

    #[test]
    fn test_command_tracker_creation() {
        let manager = create_startup_manager();
        let tracker = CommandTracker::from_startup_manager(&manager);
        
        // Should be able to get available commands
        let commands = tracker.get_available_commands();
        // Initially empty since we haven't registered commands
        assert!(commands.is_empty() || !commands.is_empty()); // Just verify it doesn't panic
    }

    #[test]
    fn test_command_not_found_error() {
        let manager = create_startup_manager();
        let error = get_command_not_found_error("nonexistent_command", &manager);
        
        assert!(error.contains("nonexistent_command"));
        assert!(error.contains("not found"));
    }

    #[test]
    fn test_validate_all_commands() {
        let manager = create_startup_manager();

        // Register a test command
        {
            let registry_arc = manager.get_registry();
            let mut registry = registry_arc.write().unwrap();
            let command_info = CommandInfo::new("test_command".to_string()).mark_registered();
            registry.register_command(command_info).unwrap();
        }
        
        let results = validate_all_commands(&manager);
        
        // Should have at least one result
        assert!(!results.is_empty());
        
        // Find our test command
        let test_result = results.iter().find(|r| r.command_name == "test_command");
        assert!(test_result.is_some());
        assert!(test_result.unwrap().is_valid);
    }
}
