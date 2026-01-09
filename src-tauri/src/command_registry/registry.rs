//! Command Registry Implementation
//!
//! 管理所有 Tauri 命令的注册和验证

use std::collections::HashMap;
use std::time::SystemTime;
use serde::{Serialize, Deserialize};
use crate::command_registry::errors::{CommandError, ErrorType};

/// Command registry that manages all Tauri commands
#[derive(Debug, Clone)]
pub struct CommandRegistry {
    registered_commands: HashMap<String, CommandInfo>,
    failed_commands: Vec<CommandError>,
    initialization_order: Vec<String>,
    command_history: HashMap<String, Vec<CommandHistoryEntry>>,
}

/// Information about a registered command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandInfo {
    pub name: String,
    pub dependencies: Vec<String>,
    pub status: CommandStatus,
    pub last_verified: SystemTime,
    pub last_called: Option<SystemTime>,
    pub call_count: u64,
    pub metadata: Option<CommandMetadata>,
}

/// Command history entry for tracking command lifecycle events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHistoryEntry {
    pub timestamp: SystemTime,
    pub event_type: CommandEventType,
    pub details: String,
}

/// Types of command events for history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandEventType {
    Registered,
    Called,
    Failed,
    StatusChanged,
    DependencyResolved,
    ValidationPassed,
    ValidationFailed,
}

/// Status of a command
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CommandStatus {
    Registered,
    Failed(String),
    Unverified,
    Disabled,
}

/// Detailed command status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandStatusInfo {
    pub name: String,
    pub status: CommandStatus,
    pub last_verified: SystemTime,
    pub last_called: Option<SystemTime>,
    pub call_count: u64,
    pub dependencies: Vec<String>,
    pub dependency_status: HashMap<String, CommandStatus>,
}

/// Metadata about a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMetadata {
    pub description: String,
    pub parameters: Vec<String>,
    pub return_type: String,
    pub version: String,
    pub deprecated: bool,
}

impl CommandRegistry {
    /// Create a new command registry
    pub fn new() -> Self {
        Self {
            registered_commands: HashMap::new(),
            failed_commands: Vec::new(),
            initialization_order: Vec::new(),
            command_history: HashMap::new(),
        }
    }

    /// Register a command with the registry
    pub fn register_command(&mut self, info: CommandInfo) -> Result<(), CommandError> {
        // Validate command name - check if it's empty or contains only whitespace
        if info.name.trim().is_empty() {
            return Err(CommandError::new(
                "Command name cannot be empty or contain only whitespace".to_string(),
                ErrorType::ValidationFailed,
            ));
        }

        // Check for duplicate registration
        if self.registered_commands.contains_key(&info.name) {
            return Err(CommandError::new(
                format!("Command '{}' is already registered", info.name),
                ErrorType::RegistrationFailed,
            ));
        }

        // Validate dependencies exist (if any)
        for dep in &info.dependencies {
            if !self.registered_commands.contains_key(dep) {
                // For now, we'll allow registration but mark as unverified
                // The dependency will be checked during verification
                self.add_history_entry(&info.name, CommandEventType::DependencyResolved, 
                    format!("Dependency '{}' not yet available during registration", dep));
            }
        }

        // Add to initialization order
        self.initialization_order.push(info.name.clone());

        // Add history entry for registration
        self.add_history_entry(&info.name, CommandEventType::Registered, 
            format!("Command registered with {} dependencies", info.dependencies.len()));

        // Register the command
        self.registered_commands.insert(info.name.clone(), info);

        Ok(())
    }

    /// Verify all registered commands and their dependencies
    pub fn verify_all_commands(&self) -> Vec<CommandError> {
        let mut errors = Vec::new();

        for (name, info) in &self.registered_commands {
            // Check dependencies
            for dep in &info.dependencies {
                if !self.registered_commands.contains_key(dep) {
                    errors.push(CommandError::dependency_missing(name, dep));
                } else {
                    // Check if dependency is in a valid state
                    if let Some(dep_info) = self.registered_commands.get(dep) {
                        if matches!(dep_info.status, CommandStatus::Failed(_) | CommandStatus::Disabled) {
                            errors.push(CommandError::new(
                                format!("Command '{}' depends on '{}' which is in invalid state: {:?}", 
                                    name, dep, dep_info.status),
                                ErrorType::DependencyMissing,
                            ));
                        }
                    }
                }
            }

            // Check status
            if let CommandStatus::Failed(reason) = &info.status {
                errors.push(CommandError::new(
                    format!("Command '{}' is in failed state: {}", name, reason),
                    ErrorType::ValidationFailed,
                ));
            }

            // Check if command has been verified recently (within last hour)
            if let Ok(duration) = SystemTime::now().duration_since(info.last_verified) {
                if duration.as_secs() > 3600 && matches!(info.status, CommandStatus::Unverified) {
                    errors.push(CommandError::new(
                        format!("Command '{}' has not been verified recently", name),
                        ErrorType::ValidationFailed,
                    ));
                }
            }
        }

        errors
    }

    /// Verify a specific command and its dependencies
    pub fn verify_command(&self, name: &str) -> Vec<CommandError> {
        let mut errors = Vec::new();

        if let Some(info) = self.registered_commands.get(name) {
            // Check dependencies
            for dep in &info.dependencies {
                if !self.registered_commands.contains_key(dep) {
                    errors.push(CommandError::dependency_missing(name, dep));
                } else {
                    // Check if dependency is in a valid state
                    if let Some(dep_info) = self.registered_commands.get(dep) {
                        if matches!(dep_info.status, CommandStatus::Failed(_) | CommandStatus::Disabled) {
                            errors.push(CommandError::new(
                                format!("Command '{}' depends on '{}' which is in invalid state: {:?}", 
                                    name, dep, dep_info.status),
                                ErrorType::DependencyMissing,
                            ));
                        }
                    }
                }
            }

            // Check status
            if let CommandStatus::Failed(reason) = &info.status {
                errors.push(CommandError::new(
                    format!("Command '{}' is in failed state: {}", name, reason),
                    ErrorType::ValidationFailed,
                ));
            }

            // Check if command has been verified recently (within last hour)
            if let Ok(duration) = SystemTime::now().duration_since(info.last_verified) {
                if duration.as_secs() > 3600 && matches!(info.status, CommandStatus::Unverified) {
                    errors.push(CommandError::new(
                        format!("Command '{}' has not been verified recently", name),
                        ErrorType::ValidationFailed,
                    ));
                }
            }
        } else {
            errors.push(CommandError::command_not_found(name));
        }

        errors
    }

    /// Perform comprehensive validation of a specific command
    pub fn validate_command(&mut self, name: &str) -> Result<(), CommandError> {
        let command_info = self.registered_commands.get(name)
            .ok_or_else(|| CommandError::command_not_found(name))?;

        // Check all dependencies are available and valid
        for dep in &command_info.dependencies {
            if !self.registered_commands.contains_key(dep) {
                let error = CommandError::dependency_missing(name, dep);
                self.add_history_entry(name, CommandEventType::ValidationFailed, 
                    format!("Missing dependency: {}", dep));
                return Err(error);
            }

            // Check dependency status
            if let Some(dep_info) = self.registered_commands.get(dep) {
                if matches!(dep_info.status, CommandStatus::Failed(_) | CommandStatus::Disabled) {
                    let error = CommandError::new(
                        format!("Dependency '{}' is not available (status: {:?})", dep, dep_info.status),
                        ErrorType::DependencyMissing,
                    );
                    self.add_history_entry(name, CommandEventType::ValidationFailed, 
                        format!("Dependency '{}' in invalid state", dep));
                    return Err(error);
                }
            }
        }

        // Update verification time and status
        if let Some(info) = self.registered_commands.get_mut(name) {
            info.last_verified = SystemTime::now();
            if matches!(info.status, CommandStatus::Unverified) {
                info.status = CommandStatus::Registered;
            }
        }

        self.add_history_entry(name, CommandEventType::ValidationPassed, 
            "Command validation successful".to_string());

        Ok(())
    }

    /// Get the status of a specific command
    pub fn get_command_status(&self, name: &str) -> Option<&CommandStatus> {
        self.registered_commands.get(name).map(|info| &info.status)
    }

    /// Get detailed command status information
    pub fn get_command_status_detailed(&self, name: &str) -> Option<CommandStatusInfo> {
        self.registered_commands.get(name).map(|info| {
            CommandStatusInfo {
                name: info.name.clone(),
                status: info.status.clone(),
                last_verified: info.last_verified,
                last_called: info.last_called,
                call_count: info.call_count,
                dependencies: info.dependencies.clone(),
                dependency_status: self.get_dependency_status(&info.dependencies),
            }
        })
    }

    /// List all available commands
    pub fn list_available_commands(&self) -> Vec<String> {
        self.registered_commands
            .iter()
            .filter(|(_, info)| matches!(info.status, CommandStatus::Registered))
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Get command information
    pub fn get_command_info(&self, name: &str) -> Option<&CommandInfo> {
        self.registered_commands.get(name)
    }

    /// Get all registered commands
    pub fn get_all_commands(&self) -> &HashMap<String, CommandInfo> {
        &self.registered_commands
    }

    /// Get failed commands
    pub fn get_failed_commands(&self) -> &Vec<CommandError> {
        &self.failed_commands
    }

    /// Get initialization order
    pub fn get_initialization_order(&self) -> &Vec<String> {
        &self.initialization_order
    }

    /// Mark a command as failed
    pub fn mark_command_failed(&mut self, name: &str, reason: String) {
        if let Some(info) = self.registered_commands.get_mut(name) {
            info.status = CommandStatus::Failed(reason.clone());
        }
        let error = CommandError::new(
            format!("Command '{}' failed: {}", name, reason),
            ErrorType::RuntimeError,
        );
        self.failed_commands.push(error);
        self.add_history_entry(name, CommandEventType::Failed, reason);
    }

    /// Update command verification time
    pub fn update_verification_time(&mut self, name: &str) {
        if let Some(info) = self.registered_commands.get_mut(name) {
            info.last_verified = SystemTime::now();
        }
    }

    /// Record a command call
    pub fn record_command_call(&mut self, name: &str) {
        if let Some(info) = self.registered_commands.get_mut(name) {
            info.last_called = Some(SystemTime::now());
            info.call_count += 1;
        }
        self.add_history_entry(name, CommandEventType::Called, 
            "Command executed".to_string());
    }

    /// Check if a command exists
    pub fn has_command(&self, name: &str) -> bool {
        self.registered_commands.contains_key(name)
    }

    /// Get command count
    pub fn command_count(&self) -> usize {
        self.registered_commands.len()
    }

    /// Get active command count (registered status only)
    pub fn active_command_count(&self) -> usize {
        self.registered_commands
            .values()
            .filter(|info| matches!(info.status, CommandStatus::Registered))
            .count()
    }

    /// Get command history
    pub fn get_command_history(&self, name: &str) -> Option<&Vec<CommandHistoryEntry>> {
        self.command_history.get(name)
    }

    /// Add a history entry for a command
    fn add_history_entry(&mut self, command_name: &str, event_type: CommandEventType, details: String) {
        let entry = CommandHistoryEntry {
            timestamp: SystemTime::now(),
            event_type,
            details,
        };
        
        self.command_history
            .entry(command_name.to_string())
            .or_insert_with(Vec::new)
            .push(entry);
    }

    /// Get dependency status for a list of dependencies
    fn get_dependency_status(&self, dependencies: &[String]) -> HashMap<String, CommandStatus> {
        dependencies
            .iter()
            .filter_map(|dep| {
                self.registered_commands
                    .get(dep)
                    .map(|info| (dep.clone(), info.status.clone()))
            })
            .collect()
    }

    /// Update command status
    pub fn update_command_status(&mut self, name: &str, status: CommandStatus) -> Result<(), CommandError> {
        if let Some(info) = self.registered_commands.get_mut(name) {
            let old_status = info.status.clone();
            info.status = status.clone();
            self.add_history_entry(name, CommandEventType::StatusChanged, 
                format!("Status changed from {:?} to {:?}", old_status, status));
            Ok(())
        } else {
            Err(CommandError::command_not_found(name))
        }
    }

    /// Unregister a command from the registry
    pub fn unregister_command(&mut self, name: &str) -> Result<CommandInfo, CommandError> {
        if let Some(info) = self.registered_commands.remove(name) {
            // Remove from initialization order
            self.initialization_order.retain(|cmd| cmd != name);
            
            // Record unregistration in history
            self.add_history_entry(name, CommandEventType::StatusChanged, 
                "Command unregistered".to_string());
            
            Ok(info)
        } else {
            Err(CommandError::command_not_found(name))
        }
    }

    /// Get commands with anomalous status (failed, disabled, or unverified)
    pub fn get_anomalous_commands(&self) -> Vec<String> {
        self.registered_commands
            .iter()
            .filter(|(_, info)| {
                matches!(info.status, 
                    CommandStatus::Failed(_) | 
                    CommandStatus::Disabled | 
                    CommandStatus::Unverified
                )
            })
            .map(|(name, _)| name.clone())
            .collect()
    }
}

impl CommandInfo {
    /// Create new command info
    pub fn new(name: String) -> Self {
        Self {
            name,
            dependencies: Vec::new(),
            status: CommandStatus::Unverified,
            last_verified: SystemTime::now(),
            last_called: None,
            call_count: 0,
            metadata: None,
        }
    }

    /// Create command info with dependencies
    pub fn with_dependencies(name: String, dependencies: Vec<String>) -> Self {
        Self {
            name,
            dependencies,
            status: CommandStatus::Unverified,
            last_verified: SystemTime::now(),
            last_called: None,
            call_count: 0,
            metadata: None,
        }
    }

    /// Set command status to registered
    pub fn mark_registered(mut self) -> Self {
        self.status = CommandStatus::Registered;
        self
    }

    /// Add metadata to command
    pub fn with_metadata(mut self, metadata: CommandMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Set command status
    pub fn with_status(mut self, status: CommandStatus) -> Self {
        self.status = status;
        self
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_registry_creation() {
        let registry = CommandRegistry::new();
        assert_eq!(registry.command_count(), 0);
        assert_eq!(registry.active_command_count(), 0);
    }

    #[test]
    fn test_command_registration() {
        let mut registry = CommandRegistry::new();
        let command_info = CommandInfo::new("test_command".to_string()).mark_registered();
        
        let result = registry.register_command(command_info);
        assert!(result.is_ok());
        assert_eq!(registry.command_count(), 1);
        assert_eq!(registry.active_command_count(), 1);
        assert!(registry.has_command("test_command"));
    }

    #[test]
    fn test_duplicate_command_registration() {
        let mut registry = CommandRegistry::new();
        let command_info1 = CommandInfo::new("test_command".to_string()).mark_registered();
        let command_info2 = CommandInfo::new("test_command".to_string()).mark_registered();
        
        registry.register_command(command_info1).unwrap();
        let result = registry.register_command(command_info2);
        
        assert!(result.is_err());
        assert_eq!(registry.command_count(), 1);
    }

    #[test]
    fn test_empty_command_name() {
        let mut registry = CommandRegistry::new();
        let command_info = CommandInfo::new("".to_string());
        
        let result = registry.register_command(command_info);
        assert!(result.is_err());
        assert_eq!(registry.command_count(), 0);
        
        // Test whitespace-only names
        let whitespace_command = CommandInfo::new("   ".to_string());
        let result2 = registry.register_command(whitespace_command);
        assert!(result2.is_err());
        assert_eq!(registry.command_count(), 0);
    }

    #[test]
    fn test_command_validation() {
        let mut registry = CommandRegistry::new();
        
        // Register a command with dependencies
        let dep_command = CommandInfo::new("dependency_command".to_string()).mark_registered();
        registry.register_command(dep_command).unwrap();
        
        let main_command = CommandInfo::with_dependencies(
            "main_command".to_string(), 
            vec!["dependency_command".to_string()]
        );
        registry.register_command(main_command).unwrap();
        
        // Validate the main command
        let result = registry.validate_command("main_command");
        assert!(result.is_ok());
        
        // Check that status was updated
        let status = registry.get_command_status("main_command");
        assert!(matches!(status, Some(CommandStatus::Registered)));
    }

    #[test]
    fn test_command_call_tracking() {
        let mut registry = CommandRegistry::new();
        let command_info = CommandInfo::new("test_command".to_string()).mark_registered();
        registry.register_command(command_info).unwrap();
        
        // Record some calls
        registry.record_command_call("test_command");
        registry.record_command_call("test_command");
        
        let info = registry.get_command_info("test_command").unwrap();
        assert_eq!(info.call_count, 2);
        assert!(info.last_called.is_some());
    }

    #[test]
    fn test_command_history() {
        let mut registry = CommandRegistry::new();
        let command_info = CommandInfo::new("test_command".to_string()).mark_registered();
        registry.register_command(command_info).unwrap();
        
        // History should contain registration event
        let history = registry.get_command_history("test_command");
        assert!(history.is_some());
        assert!(!history.unwrap().is_empty());
        
        // Record a call and check history
        registry.record_command_call("test_command");
        let history = registry.get_command_history("test_command").unwrap();
        assert!(history.len() >= 2); // Registration + call
    }

    #[test]
    fn test_detailed_status_info() {
        let mut registry = CommandRegistry::new();
        
        // Register dependency
        let dep_command = CommandInfo::new("dep".to_string()).mark_registered();
        registry.register_command(dep_command).unwrap();
        
        // Register main command with dependency
        let main_command = CommandInfo::with_dependencies(
            "main".to_string(), 
            vec!["dep".to_string()]
        );
        registry.register_command(main_command).unwrap();
        
        let status_info = registry.get_command_status_detailed("main");
        assert!(status_info.is_some());
        
        let info = status_info.unwrap();
        assert_eq!(info.name, "main");
        assert_eq!(info.dependencies.len(), 1);
        assert_eq!(info.dependency_status.len(), 1);
        assert!(info.dependency_status.contains_key("dep"));
    }
}