//! Enhanced Error Handler for Command Registration System
//!
//! Provides comprehensive error handling with pattern matching, categorization,
//! and recovery suggestions for command registration and execution errors.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, Duration};
use regex::Regex;
use serde::{Serialize, Deserialize};
use log::{error, warn, info, debug};

use crate::command_registry::errors::{CommandError, ModuleError};
use crate::command_registry::registry::{CommandRegistry, CommandStatus, CommandStatusInfo};

/// Enhanced error handler with pattern matching and recovery suggestions
pub struct EnhancedErrorHandler {
    logger: Arc<dyn Logger + Send + Sync>,
    error_patterns: HashMap<String, ErrorPattern>,
    alert_manager: Arc<Mutex<AlertManager>>,
    anomaly_detector: AnomalyDetector,
}

/// Logger trait for dependency injection
pub trait Logger {
    fn log_error(&self, message: &str, context: &ErrorContext);
    fn log_warning(&self, message: &str);
    fn log_info(&self, message: &str);
    fn log_debug(&self, message: &str);
}

/// Default logger implementation using the log crate
pub struct DefaultLogger;

impl Logger for DefaultLogger {
    fn log_error(&self, message: &str, context: &ErrorContext) {
        error!("[{}] {} - Context: {:?}", context.error_id, message, context);
    }

    fn log_warning(&self, message: &str) {
        warn!("{}", message);
    }

    fn log_info(&self, message: &str) {
        info!("{}", message);
    }

    fn log_debug(&self, message: &str) {
        debug!("{}", message);
    }
}

/// Error pattern for matching and categorizing errors
#[derive(Debug, Clone)]
pub struct ErrorPattern {
    pub pattern: Regex,
    pub category: ErrorCategory,
    pub recovery_suggestions: Vec<String>,
    pub severity: ErrorSeverity,
    pub auto_retry: bool,
}

/// Categories of errors for classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    CommandNotFound,
    ModuleInitializationFailed,
    DependencyMissing,
    RuntimeError,
    ValidationError,
    ConfigurationError,
    NetworkError,
    PermissionError,
    ResourceExhausted,
    Unknown,
}

/// Severity levels for errors
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Context information for errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub error_id: String,
    pub timestamp: SystemTime,
    pub command_name: Option<String>,
    pub module_name: Option<String>,
    pub call_stack: Vec<String>,
    pub user_action: Option<String>,
    pub system_state: HashMap<String, String>,
}

/// Response structure for error handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error_type: ErrorCategory,
    pub message: String,
    pub details: Option<String>,
    pub available_commands: Option<Vec<String>>,
    pub recovery_suggestions: Vec<String>,
    pub error_code: String,
    pub timestamp: SystemTime,
    pub severity: ErrorSeverity,
    pub retry_after: Option<u64>, // seconds
}

impl EnhancedErrorHandler {
    /// Create a new enhanced error handler with default logger
    pub fn new() -> Self {
        Self::with_logger(Arc::new(DefaultLogger))
    }

    /// Create a new enhanced error handler with custom logger
    pub fn with_logger(logger: Arc<dyn Logger + Send + Sync>) -> Self {
        let mut handler = Self {
            logger,
            error_patterns: HashMap::new(),
            alert_manager: Arc::new(Mutex::new(AlertManager::new())),
            anomaly_detector: AnomalyDetector::new(),
        };
        handler.initialize_default_patterns();
        handler.initialize_default_alert_rules();
        handler
    }

    /// Check for command status anomalies and trigger alerts
    pub fn check_command_anomalies(&mut self, registry: &CommandRegistry) -> Vec<Alert> {
        let mut triggered_alerts = Vec::new();

        // Update metrics for all commands
        for (name, info) in registry.get_all_commands() {
            self.anomaly_detector.update_command_metrics(name, info);
        }

        // Check for anomalies based on detection rules
        for rule in &self.anomaly_detector.detection_rules.clone() {
            if let Some(alert) = self.check_anomaly_rule(rule, registry) {
                triggered_alerts.push(alert.clone());
                
                // Send alert through notification handlers
                if let Ok(mut alert_manager) = self.alert_manager.lock() {
                    alert_manager.trigger_alert(alert);
                }
            }
        }

        triggered_alerts
    }

    /// Monitor command status and trigger alerts for anomalies
    pub fn monitor_command_status(&mut self, command_name: &str, status_info: &CommandStatusInfo) -> Vec<Alert> {
        let mut alerts = Vec::new();

        // Check if command status indicates an anomaly
        match &status_info.status {
            CommandStatus::Failed(reason) => {
                let alert = Alert::new(
                    AlertType::CommandFailure,
                    AlertSeverity::Critical,
                    command_name.to_string(),
                    format!("Command failed: {}", reason),
                );
                alerts.push(alert.clone());
                
                if let Ok(mut alert_manager) = self.alert_manager.lock() {
                    alert_manager.trigger_alert(alert);
                }
            }
            CommandStatus::Disabled => {
                let alert = Alert::new(
                    AlertType::CommandDisabled,
                    AlertSeverity::Warning,
                    command_name.to_string(),
                    "Command has been disabled".to_string(),
                );
                alerts.push(alert.clone());
                
                if let Ok(mut alert_manager) = self.alert_manager.lock() {
                    alert_manager.trigger_alert(alert);
                }
            }
            _ => {}
        }

        // Check for dependency failures
        for (dep_name, dep_status) in &status_info.dependency_status {
            if matches!(dep_status, CommandStatus::Failed(_)) {
                let alert = Alert::new(
                    AlertType::DependencyFailure,
                    AlertSeverity::Critical,
                    command_name.to_string(),
                    format!("Dependency '{}' has failed", dep_name),
                );
                alerts.push(alert.clone());
                
                if let Ok(mut alert_manager) = self.alert_manager.lock() {
                    alert_manager.trigger_alert(alert);
                }
            }
        }

        // Check for commands that haven't been called recently
        if let Some(last_called) = status_info.last_called {
            if let Ok(duration) = SystemTime::now().duration_since(last_called) {
                if duration > Duration::from_secs(3600) { // 1 hour threshold
                    let alert = Alert::new(
                        AlertType::CommandNotResponding,
                        AlertSeverity::Info,
                        command_name.to_string(),
                        format!("Command hasn't been called for {} minutes", duration.as_secs() / 60),
                    );
                    alerts.push(alert.clone());
                    
                    if let Ok(mut alert_manager) = self.alert_manager.lock() {
                        alert_manager.trigger_alert(alert);
                    }
                }
            }
        }

        alerts
    }

    /// Get all active alerts
    pub fn get_active_alerts(&self) -> Vec<Alert> {
        if let Ok(alert_manager) = self.alert_manager.lock() {
            alert_manager.get_active_alerts()
        } else {
            Vec::new()
        }
    }

    /// Resolve an alert by ID
    pub fn resolve_alert(&self, alert_id: &str) -> Result<(), String> {
        if let Ok(mut alert_manager) = self.alert_manager.lock() {
            alert_manager.resolve_alert(alert_id)
        } else {
            Err("Failed to acquire alert manager lock".to_string())
        }
    }

    /// Add a notification handler
    pub fn add_notification_handler(&self, handler: Box<dyn NotificationHandler + Send + Sync>) {
        if let Ok(mut alert_manager) = self.alert_manager.lock() {
            alert_manager.add_notification_handler(handler);
        }
    }

    /// Get alert statistics
    pub fn get_alert_statistics(&self) -> AlertStatistics {
        if let Ok(alert_manager) = self.alert_manager.lock() {
            alert_manager.get_statistics()
        } else {
            AlertStatistics::default()
        }
    }

    /// Initialize default alert rules
    fn initialize_default_alert_rules(&mut self) {
        if let Ok(mut alert_manager) = self.alert_manager.lock() {
            // Rule for command failures
            alert_manager.add_alert_rule(AlertRule {
                name: "Command Failure".to_string(),
                alert_type: AlertType::CommandFailure,
                condition: AlertCondition::CommandFailureCount {
                    threshold: 3,
                    time_window: Duration::from_secs(300), // 5 minutes
                },
                severity: AlertSeverity::Critical,
                cooldown_period: Duration::from_secs(60),
                enabled: true,
            });

            // Rule for high error rate
            alert_manager.add_alert_rule(AlertRule {
                name: "High Error Rate".to_string(),
                alert_type: AlertType::HighErrorRate,
                condition: AlertCondition::ErrorRateExceeded {
                    rate: 0.5, // 50% error rate
                    time_window: Duration::from_secs(600), // 10 minutes
                },
                severity: AlertSeverity::Warning,
                cooldown_period: Duration::from_secs(120),
                enabled: true,
            });

            // Rule for inactive commands
            alert_manager.add_alert_rule(AlertRule {
                name: "Command Inactive".to_string(),
                alert_type: AlertType::CommandNotResponding,
                condition: AlertCondition::CommandNotCalledFor {
                    duration: Duration::from_secs(7200), // 2 hours
                },
                severity: AlertSeverity::Info,
                cooldown_period: Duration::from_secs(3600), // 1 hour
                enabled: true,
            });

            // Rule for consecutive failures
            alert_manager.add_alert_rule(AlertRule {
                name: "Consecutive Failures".to_string(),
                alert_type: AlertType::CommandFailure,
                condition: AlertCondition::ConsecutiveFailures { count: 5 },
                severity: AlertSeverity::Emergency,
                cooldown_period: Duration::from_secs(30),
                enabled: true,
            });
        }

        // Initialize anomaly detection rules
        self.anomaly_detector.add_rule(AnomalyRule {
            name: "High Failure Rate".to_string(),
            condition: AnomalyCondition::HighFailureRate { threshold: 0.3 },
            alert_type: AlertType::HighErrorRate,
            severity: AlertSeverity::Warning,
        });

        self.anomaly_detector.add_rule(AnomalyRule {
            name: "Unusual Inactivity".to_string(),
            condition: AnomalyCondition::UnusualInactivity {
                threshold: Duration::from_secs(1800), // 30 minutes
            },
            alert_type: AlertType::CommandNotResponding,
            severity: AlertSeverity::Info,
        });

        self.anomaly_detector.add_rule(AnomalyRule {
            name: "Consecutive Failures".to_string(),
            condition: AnomalyCondition::ConsecutiveFailures { count: 3 },
            alert_type: AlertType::CommandFailure,
            severity: AlertSeverity::Critical,
        });
    }

    /// Check a specific anomaly rule
    fn check_anomaly_rule(&self, rule: &AnomalyRule, _registry: &CommandRegistry) -> Option<Alert> {
        for (command_name, metrics) in &self.anomaly_detector.command_metrics {
            if self.anomaly_detector.evaluate_condition(&rule.condition, metrics) {
                return Some(Alert::new(
                    rule.alert_type.clone(),
                    rule.severity.clone(),
                    command_name.clone(),
                    format!("Anomaly detected: {}", rule.name),
                ));
            }
        }
        None
    }
    fn initialize_default_patterns(&mut self) {
        // Command not found pattern
        self.add_pattern(ErrorPattern {
            pattern: Regex::new(r"Command '([^']+)' not found").unwrap(),
            category: ErrorCategory::CommandNotFound,
            recovery_suggestions: vec![
                "Check if the command name is spelled correctly".to_string(),
                "Verify that the command is properly registered".to_string(),
                "Use the diagnostic tool to list available commands".to_string(),
            ],
            severity: ErrorSeverity::Medium,
            auto_retry: false,
        });

        // Module initialization failure pattern
        self.add_pattern(ErrorPattern {
            pattern: Regex::new(r"Module initialization failed|Failed to initialize module").unwrap(),
            category: ErrorCategory::ModuleInitializationFailed,
            recovery_suggestions: vec![
                "Check module dependencies are available".to_string(),
                "Verify configuration files are correct".to_string(),
                "Restart the application to retry initialization".to_string(),
                "Check system resources and permissions".to_string(),
            ],
            severity: ErrorSeverity::High,
            auto_retry: true,
        });

        // Dependency missing pattern
        self.add_pattern(ErrorPattern {
            pattern: Regex::new(r"Missing dependency|dependency.*not found").unwrap(),
            category: ErrorCategory::DependencyMissing,
            recovery_suggestions: vec![
                "Install the missing dependency".to_string(),
                "Check the dependency configuration".to_string(),
                "Verify the dependency version compatibility".to_string(),
            ],
            severity: ErrorSeverity::High,
            auto_retry: false,
        });

        // Runtime error pattern
        self.add_pattern(ErrorPattern {
            pattern: Regex::new(r"Runtime error|Execution failed|Panic").unwrap(),
            category: ErrorCategory::RuntimeError,
            recovery_suggestions: vec![
                "Check the input parameters for validity".to_string(),
                "Verify system resources are available".to_string(),
                "Review the error logs for more details".to_string(),
                "Contact support if the issue persists".to_string(),
            ],
            severity: ErrorSeverity::Critical,
            auto_retry: false,
        });

        // Validation error pattern
        self.add_pattern(ErrorPattern {
            pattern: Regex::new(r"Validation failed|Invalid.*parameter|Parameter.*invalid").unwrap(),
            category: ErrorCategory::ValidationError,
            recovery_suggestions: vec![
                "Check the parameter format and values".to_string(),
                "Refer to the API documentation for correct usage".to_string(),
                "Validate input data before sending".to_string(),
            ],
            severity: ErrorSeverity::Medium,
            auto_retry: false,
        });

        // Configuration error pattern
        self.add_pattern(ErrorPattern {
            pattern: Regex::new(r"Configuration.*error|Config.*invalid|Settings.*missing").unwrap(),
            category: ErrorCategory::ConfigurationError,
            recovery_suggestions: vec![
                "Check the configuration file syntax".to_string(),
                "Verify all required settings are present".to_string(),
                "Reset to default configuration if needed".to_string(),
            ],
            severity: ErrorSeverity::High,
            auto_retry: false,
        });

        // Permission error pattern
        self.add_pattern(ErrorPattern {
            pattern: Regex::new(r"Permission denied|Access denied|Unauthorized").unwrap(),
            category: ErrorCategory::PermissionError,
            recovery_suggestions: vec![
                "Check file and directory permissions".to_string(),
                "Run with appropriate privileges if needed".to_string(),
                "Verify user has necessary access rights".to_string(),
            ],
            severity: ErrorSeverity::High,
            auto_retry: false,
        });
    }

    /// Add a custom error pattern
    pub fn add_pattern(&mut self, pattern: ErrorPattern) {
        let key = pattern.pattern.as_str().to_string();
        self.error_patterns.insert(key, pattern);
    }

    /// Handle a command error and return structured response
    pub fn handle_command_error(&self, error: &CommandError) -> ErrorResponse {
        let context = self.create_error_context(Some(error.message.clone()), None, None);
        let category = self.categorize_error(&error.message);
        let suggestions = self.get_recovery_suggestions(&category);
        let severity = self.get_error_severity(&category);

        self.log_error_with_context(error, &context);

        ErrorResponse {
            error_type: category.clone(),
            message: self.create_friendly_message(&error.message, &category),
            details: error.context.clone(),
            available_commands: self.get_available_commands_if_needed(&category),
            recovery_suggestions: suggestions,
            error_code: self.generate_error_code(&category),
            timestamp: SystemTime::now(),
            severity,
            retry_after: self.get_retry_delay(&category),
        }
    }

    /// Handle a module error and return structured response
    pub fn handle_module_error(&self, error: &ModuleError) -> ErrorResponse {
        let context = self.create_error_context(
            Some(error.message.clone()),
            Some(error.module_name.clone()),
            None,
        );
        let category = self.categorize_error(&error.message);
        let suggestions = self.get_recovery_suggestions(&category);
        let severity = self.get_error_severity(&category);

        self.logger.log_error(&error.message, &context);

        ErrorResponse {
            error_type: category.clone(),
            message: self.create_friendly_message(&error.message, &category),
            details: Some(format!("Module: {}", error.module_name)),
            available_commands: None,
            recovery_suggestions: suggestions,
            error_code: self.generate_error_code(&category),
            timestamp: SystemTime::now(),
            severity,
            retry_after: self.get_retry_delay(&category),
        }
    }

    /// Categorize an error based on its message
    pub fn categorize_error(&self, error_message: &str) -> ErrorCategory {
        for pattern in self.error_patterns.values() {
            if pattern.pattern.is_match(error_message) {
                return pattern.category.clone();
            }
        }
        ErrorCategory::Unknown
    }

    /// Get recovery suggestions for an error category
    pub fn get_recovery_suggestions(&self, category: &ErrorCategory) -> Vec<String> {
        for pattern in self.error_patterns.values() {
            if pattern.category == *category {
                return pattern.recovery_suggestions.clone();
            }
        }
        vec!["Contact support for assistance".to_string()]
    }

    /// Get error severity for a category
    fn get_error_severity(&self, category: &ErrorCategory) -> ErrorSeverity {
        for pattern in self.error_patterns.values() {
            if pattern.category == *category {
                return pattern.severity.clone();
            }
        }
        ErrorSeverity::Medium
    }

    /// Log error with full context
    pub fn log_error_with_context(&self, error: &CommandError, context: &ErrorContext) {
        self.logger.log_error(&error.message, context);
        
        // Log additional details based on error type
        match error.error_type {
            crate::command_registry::errors::ErrorType::CommandNotFound => {
                self.logger.log_info("Consider using the diagnostic tool to list available commands");
            }
            crate::command_registry::errors::ErrorType::DependencyMissing => {
                self.logger.log_warning("Check module initialization order and dependencies");
            }
            crate::command_registry::errors::ErrorType::RuntimeError => {
                self.logger.log_error("Runtime error occurred - check system resources", context);
            }
            _ => {}
        }
    }

    /// Create error context for logging
    fn create_error_context(
        &self,
        _message: Option<String>,
        module_name: Option<String>,
        command_name: Option<String>,
    ) -> ErrorContext {
        ErrorContext {
            error_id: self.generate_error_id(),
            timestamp: SystemTime::now(),
            command_name,
            module_name,
            call_stack: self.capture_call_stack(),
            user_action: None,
            system_state: self.capture_system_state(),
        }
    }

    /// Generate unique error ID
    fn generate_error_id(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("ERR_{}", timestamp)
    }

    /// Capture call stack (simplified implementation)
    fn capture_call_stack(&self) -> Vec<String> {
        // In a real implementation, this would capture the actual call stack
        vec!["enhanced_error_handler::handle_error".to_string()]
    }

    /// Capture system state
    fn capture_system_state(&self) -> HashMap<String, String> {
        let mut state = HashMap::new();
        state.insert("timestamp".to_string(), format!("{:?}", SystemTime::now()));
        state.insert("handler_version".to_string(), "1.0.0".to_string());
        state
    }

    /// Create user-friendly error message
    pub fn create_friendly_message(&self, original_message: &str, category: &ErrorCategory) -> String {
        let friendly_message = match category {
            ErrorCategory::CommandNotFound => {
                "The requested command could not be found. Please check the command name and try again.".to_string()
            }
            ErrorCategory::ModuleInitializationFailed => {
                "A required module failed to initialize. The application may not function correctly.".to_string()
            }
            ErrorCategory::DependencyMissing => {
                "A required dependency is missing. Please check your installation.".to_string()
            }
            ErrorCategory::ValidationError => {
                "The provided parameters are invalid. Please check your input and try again.".to_string()
            }
            ErrorCategory::PermissionError => {
                "Access denied. Please check your permissions and try again.".to_string()
            }
            ErrorCategory::ConfigurationError => {
                "There is an issue with the system configuration. Please check your settings and try again.".to_string()
            }
            ErrorCategory::NetworkError => {
                "A network error occurred. Please check your connection and try again.".to_string()
            }
            ErrorCategory::ResourceExhausted => {
                "System resources are exhausted. Please try again later or free up resources.".to_string()
            }
            ErrorCategory::RuntimeError => {
                "An unexpected error occurred during execution. Please try again or contact support.".to_string()
            }
            ErrorCategory::Unknown => {
                // For unknown errors, create a generic but descriptive message
                if original_message.trim().is_empty() || original_message.len() < 5 {
                    "An unexpected error occurred. Please try again or contact support for assistance.".to_string()
                } else {
                    format!("An error occurred: {}. Please try again or contact support if the issue persists.", original_message)
                }
            }
        };

        // Ensure the message is always descriptive (at least 15 characters)
        if friendly_message.len() < 15 {
            "An unexpected error occurred. Please try again or contact support for assistance.".to_string()
        } else {
            friendly_message
        }
    }

    /// Get available commands list if needed for the error category
    fn get_available_commands_if_needed(&self, category: &ErrorCategory) -> Option<Vec<String>> {
        match category {
            ErrorCategory::CommandNotFound => {
                // In a real implementation, this would query the command registry
                Some(vec![
                    "get_monitored_directories".to_string(),
                    "scan_sessions".to_string(),
                    "health_check".to_string(),
                ])
            }
            _ => None,
        }
    }

    /// Generate error code for the category
    fn generate_error_code(&self, category: &ErrorCategory) -> String {
        match category {
            ErrorCategory::CommandNotFound => "CMD_404".to_string(),
            ErrorCategory::ModuleInitializationFailed => "MOD_500".to_string(),
            ErrorCategory::DependencyMissing => "DEP_404".to_string(),
            ErrorCategory::RuntimeError => "RUN_500".to_string(),
            ErrorCategory::ValidationError => "VAL_400".to_string(),
            ErrorCategory::ConfigurationError => "CFG_500".to_string(),
            ErrorCategory::NetworkError => "NET_500".to_string(),
            ErrorCategory::PermissionError => "PRM_403".to_string(),
            ErrorCategory::ResourceExhausted => "RES_503".to_string(),
            ErrorCategory::Unknown => "UNK_500".to_string(),
        }
    }

    /// Get retry delay for error category
    fn get_retry_delay(&self, category: &ErrorCategory) -> Option<u64> {
        for pattern in self.error_patterns.values() {
            if pattern.category == *category && pattern.auto_retry {
                return Some(5); // 5 seconds default retry delay
            }
        }
        None
    }

    /// Check if error should be auto-retried
    pub fn should_auto_retry(&self, category: &ErrorCategory) -> bool {
        for pattern in self.error_patterns.values() {
            if pattern.category == *category {
                return pattern.auto_retry;
            }
        }
        false
    }

    /// Get error statistics
    pub fn get_error_statistics(&self) -> ErrorStatistics {
        ErrorStatistics {
            total_patterns: self.error_patterns.len(),
            categories: self.error_patterns.values()
                .map(|p| p.category.clone())
                .collect::<std::collections::HashSet<_>>()
                .len(),
        }
    }
}

/// Error statistics for monitoring
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorStatistics {
    pub total_patterns: usize,
    pub categories: usize,
}

/// Alert manager for handling command status anomalies
pub struct AlertManager {
    alerts: Vec<Alert>,
    alert_rules: Vec<AlertRule>,
    notification_handlers: Vec<Box<dyn NotificationHandler + Send + Sync>>,
    alert_history: HashMap<String, Vec<Alert>>,
}

/// Alert structure for command anomalies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub command_name: String,
    pub message: String,
    pub details: HashMap<String, String>,
    pub timestamp: SystemTime,
    pub resolved: bool,
    pub resolution_time: Option<SystemTime>,
}

/// Types of alerts that can be triggered
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertType {
    CommandFailure,
    CommandNotResponding,
    DependencyFailure,
    HighErrorRate,
    CommandDisabled,
    ModuleInitializationFailed,
    UnusualCallPattern,
    ResourceExhaustion,
}

/// Severity levels for alerts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Alert rule for defining when to trigger alerts
#[derive(Debug, Clone)]
pub struct AlertRule {
    pub name: String,
    pub alert_type: AlertType,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub cooldown_period: Duration,
    pub enabled: bool,
}

/// Conditions that trigger alerts
#[derive(Debug, Clone)]
pub enum AlertCondition {
    CommandFailureCount { threshold: u32, time_window: Duration },
    CommandNotCalledFor { duration: Duration },
    DependencyUnavailable,
    ErrorRateExceeded { rate: f64, time_window: Duration },
    CommandStatusChanged { from: CommandStatus, to: CommandStatus },
    ConsecutiveFailures { count: u32 },
}

/// Trait for handling alert notifications
pub trait NotificationHandler {
    fn send_alert(&self, alert: &Alert) -> Result<(), String>;
    fn get_handler_name(&self) -> &str;
}

/// Console notification handler for development
#[derive(Debug)]
pub struct ConsoleNotificationHandler;

impl NotificationHandler for ConsoleNotificationHandler {
    fn send_alert(&self, alert: &Alert) -> Result<(), String> {
        match alert.severity {
            AlertSeverity::Emergency | AlertSeverity::Critical => {
                error!("[ALERT] {} - {}: {}", alert.severity_string(), alert.command_name, alert.message);
            }
            AlertSeverity::Warning => {
                warn!("[ALERT] {} - {}: {}", alert.severity_string(), alert.command_name, alert.message);
            }
            AlertSeverity::Info => {
                info!("[ALERT] {} - {}: {}", alert.severity_string(), alert.command_name, alert.message);
            }
        }
        Ok(())
    }

    fn get_handler_name(&self) -> &str {
        "console"
    }
}

/// Anomaly detector for command behavior
#[derive(Debug)]
pub struct AnomalyDetector {
    command_metrics: HashMap<String, CommandMetrics>,
    detection_rules: Vec<AnomalyRule>,
}

/// Metrics tracked for each command
#[derive(Debug, Clone)]
pub struct CommandMetrics {
    pub call_count: u64,
    pub failure_count: u64,
    pub last_call_time: Option<SystemTime>,
    pub average_response_time: Duration,
    pub consecutive_failures: u32,
    pub status_changes: Vec<(SystemTime, CommandStatus)>,
}

/// Rules for detecting anomalies
#[derive(Debug, Clone)]
pub struct AnomalyRule {
    pub name: String,
    pub condition: AnomalyCondition,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
}

/// Conditions that indicate anomalies
#[derive(Debug, Clone)]
pub enum AnomalyCondition {
    HighFailureRate { threshold: f64 },
    UnusualInactivity { threshold: Duration },
    RapidStatusChanges { count: u32, time_window: Duration },
    ConsecutiveFailures { count: u32 },
}

impl Default for EnhancedErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new() -> Self {
        let mut manager = Self {
            alerts: Vec::new(),
            alert_rules: Vec::new(),
            notification_handlers: Vec::new(),
            alert_history: HashMap::new(),
        };
        
        // Add default console notification handler
        manager.add_notification_handler(Box::new(ConsoleNotificationHandler));
        manager
    }

    /// Add an alert rule
    pub fn add_alert_rule(&mut self, rule: AlertRule) {
        self.alert_rules.push(rule);
    }

    /// Trigger an alert
    pub fn trigger_alert(&mut self, alert: Alert) {
        // Check cooldown period for similar alerts
        if self.is_in_cooldown(&alert) {
            return;
        }

        // Add to active alerts
        self.alerts.push(alert.clone());

        // Add to history
        self.alert_history
            .entry(alert.command_name.clone())
            .or_insert_with(Vec::new)
            .push(alert.clone());

        // Send notifications
        for handler in &self.notification_handlers {
            if let Err(e) = handler.send_alert(&alert) {
                error!("Failed to send alert via {}: {}", handler.get_handler_name(), e);
            }
        }
    }

    /// Get active alerts
    pub fn get_active_alerts(&self) -> Vec<Alert> {
        self.alerts.iter().filter(|a| !a.resolved).cloned().collect()
    }

    /// Resolve an alert
    pub fn resolve_alert(&mut self, alert_id: &str) -> Result<(), String> {
        if let Some(alert) = self.alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.resolved = true;
            alert.resolution_time = Some(SystemTime::now());
            Ok(())
        } else {
            Err(format!("Alert with ID '{}' not found", alert_id))
        }
    }

    /// Add notification handler
    pub fn add_notification_handler(&mut self, handler: Box<dyn NotificationHandler + Send + Sync>) {
        self.notification_handlers.push(handler);
    }

    /// Get alert statistics
    pub fn get_statistics(&self) -> AlertStatistics {
        let total_alerts = self.alerts.len();
        let active_alerts = self.get_active_alerts().len();
        let resolved_alerts = total_alerts - active_alerts;
        
        let mut severity_counts = HashMap::new();
        for alert in &self.alerts {
            *severity_counts.entry(alert.severity.clone()).or_insert(0) += 1;
        }

        AlertStatistics {
            total_alerts,
            active_alerts,
            resolved_alerts,
            severity_counts,
        }
    }

    /// Check if alert is in cooldown period
    fn is_in_cooldown(&self, alert: &Alert) -> bool {
        // Find the most recent similar alert
        if let Some(history) = self.alert_history.get(&alert.command_name) {
            for historical_alert in history.iter().rev() {
                if historical_alert.alert_type == alert.alert_type {
                    if let Ok(duration) = SystemTime::now().duration_since(historical_alert.timestamp) {
                        // Use a default cooldown of 5 minutes if no specific rule found
                        let cooldown = Duration::from_secs(300);
                        return duration < cooldown;
                    }
                    break;
                }
            }
        }
        false
    }
}

impl Alert {
    /// Create a new alert
    pub fn new(alert_type: AlertType, severity: AlertSeverity, command_name: String, message: String) -> Self {
        Self {
            id: Self::generate_id(),
            alert_type,
            severity,
            command_name,
            message,
            details: HashMap::new(),
            timestamp: SystemTime::now(),
            resolved: false,
            resolution_time: None,
        }
    }

    /// Generate unique alert ID
    fn generate_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("ALERT_{}", timestamp)
    }

    /// Get severity as string
    pub fn severity_string(&self) -> &str {
        match self.severity {
            AlertSeverity::Info => "INFO",
            AlertSeverity::Warning => "WARNING",
            AlertSeverity::Critical => "CRITICAL",
            AlertSeverity::Emergency => "EMERGENCY",
        }
    }

    /// Add detail to alert
    pub fn add_detail(&mut self, key: String, value: String) {
        self.details.insert(key, value);
    }
}

impl AnomalyDetector {
    /// Create a new anomaly detector
    pub fn new() -> Self {
        Self {
            command_metrics: HashMap::new(),
            detection_rules: Vec::new(),
        }
    }

    /// Add an anomaly detection rule
    pub fn add_rule(&mut self, rule: AnomalyRule) {
        self.detection_rules.push(rule);
    }

    /// Update metrics for a command
    pub fn update_command_metrics(&mut self, command_name: &str, info: &crate::command_registry::registry::CommandInfo) {
        let metrics = self.command_metrics
            .entry(command_name.to_string())
            .or_insert_with(|| CommandMetrics {
                call_count: 0,
                failure_count: 0,
                last_call_time: None,
                average_response_time: Duration::from_millis(0),
                consecutive_failures: 0,
                status_changes: Vec::new(),
            });

        // Update metrics based on command info
        metrics.call_count = info.call_count;
        metrics.last_call_time = info.last_called;

        // Track status changes
        if let Some(last_status) = metrics.status_changes.last() {
            if last_status.1 != info.status {
                metrics.status_changes.push((SystemTime::now(), info.status.clone()));
            }
        } else {
            metrics.status_changes.push((SystemTime::now(), info.status.clone()));
        }

        // Update failure count based on status
        if matches!(info.status, CommandStatus::Failed(_)) {
            metrics.consecutive_failures += 1;
        } else {
            metrics.consecutive_failures = 0;
        }
    }

    /// Evaluate an anomaly condition
    pub fn evaluate_condition(&self, condition: &AnomalyCondition, metrics: &CommandMetrics) -> bool {
        match condition {
            AnomalyCondition::HighFailureRate { threshold } => {
                if metrics.call_count > 0 {
                    let failure_rate = metrics.failure_count as f64 / metrics.call_count as f64;
                    failure_rate > *threshold
                } else {
                    false
                }
            }
            AnomalyCondition::UnusualInactivity { threshold } => {
                if let Some(last_call) = metrics.last_call_time {
                    if let Ok(duration) = SystemTime::now().duration_since(last_call) {
                        duration > *threshold
                    } else {
                        false
                    }
                } else {
                    true // Never called could be considered unusual
                }
            }
            AnomalyCondition::RapidStatusChanges { count, time_window } => {
                let recent_changes = metrics.status_changes
                    .iter()
                    .filter(|(timestamp, _)| {
                        if let Ok(duration) = SystemTime::now().duration_since(*timestamp) {
                            duration <= *time_window
                        } else {
                            false
                        }
                    })
                    .count();
                recent_changes >= *count as usize
            }
            AnomalyCondition::ConsecutiveFailures { count } => {
                metrics.consecutive_failures >= *count
            }
        }
    }
}

/// Statistics about alerts
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AlertStatistics {
    pub total_alerts: usize,
    pub active_alerts: usize,
    pub resolved_alerts: usize,
    pub severity_counts: HashMap<AlertSeverity, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_categorization() {
        let handler = EnhancedErrorHandler::new();
        
        assert_eq!(
            handler.categorize_error("Command 'test_command' not found"),
            ErrorCategory::CommandNotFound
        );
        
        assert_eq!(
            handler.categorize_error("Module initialization failed"),
            ErrorCategory::ModuleInitializationFailed
        );
        
        assert_eq!(
            handler.categorize_error("Missing dependency: database"),
            ErrorCategory::DependencyMissing
        );
    }

    #[test]
    fn test_recovery_suggestions() {
        let handler = EnhancedErrorHandler::new();
        
        let suggestions = handler.get_recovery_suggestions(&ErrorCategory::CommandNotFound);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("command name")));
    }

    #[test]
    fn test_command_error_handling() {
        let handler = EnhancedErrorHandler::new();
        let error = CommandError::command_not_found("test_command");
        
        let response = handler.handle_command_error(&error);
        
        assert_eq!(response.error_type, ErrorCategory::CommandNotFound);
        assert!(!response.recovery_suggestions.is_empty());
        assert!(response.available_commands.is_some());
        assert_eq!(response.error_code, "CMD_404");
    }

    #[test]
    fn test_module_error_handling() {
        let handler = EnhancedErrorHandler::new();
        let error = ModuleError::initialization_failed("database", "Connection failed");
        
        let response = handler.handle_module_error(&error);
        
        assert_eq!(response.error_type, ErrorCategory::ModuleInitializationFailed);
        assert!(!response.recovery_suggestions.is_empty());
        assert_eq!(response.error_code, "MOD_500");
    }

    #[test]
    fn test_friendly_message_generation() {
        let handler = EnhancedErrorHandler::new();
        
        let message = handler.create_friendly_message(
            "Command 'xyz' not found",
            &ErrorCategory::CommandNotFound
        );
        
        assert!(message.contains("could not be found"));
        assert!(!message.contains("xyz")); // Should be more generic
    }

    #[test]
    fn test_error_severity_assignment() {
        let handler = EnhancedErrorHandler::new();
        
        assert_eq!(
            handler.get_error_severity(&ErrorCategory::CommandNotFound),
            ErrorSeverity::Medium
        );
        
        assert_eq!(
            handler.get_error_severity(&ErrorCategory::ModuleInitializationFailed),
            ErrorSeverity::High
        );
    }

    #[test]
    fn test_alert_creation() {
        let alert = Alert::new(
            AlertType::CommandFailure,
            AlertSeverity::Critical,
            "test_command".to_string(),
            "Test alert message".to_string(),
        );

        assert_eq!(alert.alert_type, AlertType::CommandFailure);
        assert_eq!(alert.severity, AlertSeverity::Critical);
        assert_eq!(alert.command_name, "test_command");
        assert_eq!(alert.message, "Test alert message");
        assert!(!alert.resolved);
        assert!(alert.resolution_time.is_none());
    }

    #[test]
    fn test_alert_manager_creation() {
        let manager = AlertManager::new();
        assert_eq!(manager.get_active_alerts().len(), 0);
        assert_eq!(manager.notification_handlers.len(), 1); // Default console handler
    }

    #[test]
    fn test_alert_triggering() {
        let mut manager = AlertManager::new();
        let alert = Alert::new(
            AlertType::CommandFailure,
            AlertSeverity::Warning,
            "test_command".to_string(),
            "Test failure".to_string(),
        );

        manager.trigger_alert(alert.clone());
        
        let active_alerts = manager.get_active_alerts();
        assert_eq!(active_alerts.len(), 1);
        assert_eq!(active_alerts[0].command_name, "test_command");
    }

    #[test]
    fn test_alert_resolution() {
        let mut manager = AlertManager::new();
        let alert = Alert::new(
            AlertType::CommandFailure,
            AlertSeverity::Warning,
            "test_command".to_string(),
            "Test failure".to_string(),
        );
        let alert_id = alert.id.clone();

        manager.trigger_alert(alert);
        assert_eq!(manager.get_active_alerts().len(), 1);

        let result = manager.resolve_alert(&alert_id);
        assert!(result.is_ok());
        assert_eq!(manager.get_active_alerts().len(), 0);
    }

    #[test]
    fn test_anomaly_detector_creation() {
        let detector = AnomalyDetector::new();
        assert_eq!(detector.command_metrics.len(), 0);
        assert_eq!(detector.detection_rules.len(), 0);
    }

    #[test]
    fn test_anomaly_condition_evaluation() {
        let detector = AnomalyDetector::new();
        let metrics = CommandMetrics {
            call_count: 10,
            failure_count: 5,
            last_call_time: Some(SystemTime::now() - Duration::from_secs(3600)),
            average_response_time: Duration::from_millis(100),
            consecutive_failures: 3,
            status_changes: vec![(SystemTime::now(), CommandStatus::Registered)],
        };

        // Test high failure rate condition
        let condition = AnomalyCondition::HighFailureRate { threshold: 0.3 };
        assert!(detector.evaluate_condition(&condition, &metrics));

        // Test unusual inactivity condition
        let condition = AnomalyCondition::UnusualInactivity { 
            threshold: Duration::from_secs(1800) 
        };
        assert!(detector.evaluate_condition(&condition, &metrics));

        // Test consecutive failures condition
        let condition = AnomalyCondition::ConsecutiveFailures { count: 2 };
        assert!(detector.evaluate_condition(&condition, &metrics));
    }

    #[test]
    fn test_console_notification_handler() {
        let handler = ConsoleNotificationHandler;
        let alert = Alert::new(
            AlertType::CommandFailure,
            AlertSeverity::Critical,
            "test_command".to_string(),
            "Test alert".to_string(),
        );

        let result = handler.send_alert(&alert);
        assert!(result.is_ok());
        assert_eq!(handler.get_handler_name(), "console");
    }
}