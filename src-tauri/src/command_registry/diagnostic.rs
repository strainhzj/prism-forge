//! Diagnostic Tool Implementation
//!
//! 提供命令注册问题的诊断和分析

use std::sync::Arc;
use std::time::SystemTime;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::command_registry::{CommandRegistry, ModuleInitializer, InitState};
use crate::command_registry::errors::{CommandError, DiagnosticError, DiagnosticErrorType};

/// Diagnostic tool for command registration issues
pub struct DiagnosticTool {
    registry: Arc<CommandRegistry>,
    initializer: Arc<ModuleInitializer>,
}

/// Comprehensive diagnostic report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub timestamp: SystemTime,
    pub registered_commands: Vec<String>,
    pub failed_commands: Vec<CommandError>,
    pub module_states: HashMap<String, InitState>,
    pub recommendations: Vec<String>,
    pub summary: DiagnosticSummary,
}

/// Summary of diagnostic results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticSummary {
    pub total_commands: usize,
    pub active_commands: usize,
    pub failed_commands: usize,
    pub total_modules: usize,
    pub ready_modules: usize,
    pub failed_modules: usize,
    pub overall_health: HealthStatus,
}

/// Diagnostic result for a specific command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDiagnostic {
    pub command_name: String,
    pub status: String,
    pub dependencies: Vec<String>,
    pub missing_dependencies: Vec<String>,
    pub last_verified: SystemTime,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Overall health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

/// Report export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportFormat {
    Json,
    Markdown,
    Html,
}

/// Diagnostic issue detected by automated analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticIssue {
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub title: String,
    pub description: String,
    pub recommendations: Vec<String>,
}

/// Severity level of a diagnostic issue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueSeverity {
    Critical,
    Warning,
    Info,
}

/// Category of diagnostic issue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueCategory {
    Configuration,
    Dependencies,
    Performance,
    Security,
    Validation,
}

/// Automated analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomatedAnalysisReport {
    pub timestamp: SystemTime,
    pub detected_issues: Vec<DiagnosticIssue>,
    pub intelligent_suggestions: Vec<String>,
    pub trend_analysis: TrendAnalysis,
    pub risk_assessment: RiskAssessment,
    pub overall_score: u8, // 0-100 health score
}

/// Trend analysis data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub command_usage_trend: String,
    pub error_rate_trend: String,
    pub performance_trend: String,
    pub recommendations: Vec<String>,
}

/// Risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub risk_level: RiskLevel,
    pub risk_factors: Vec<String>,
    pub mitigation_strategies: Vec<String>,
    pub estimated_impact: String,
}

/// Risk level classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Minimal,
    Low,
    Medium,
    High,
}

impl DiagnosticTool {
    /// Create a new diagnostic tool
    pub fn new(registry: Arc<CommandRegistry>, initializer: Arc<ModuleInitializer>) -> Self {
        Self {
            registry,
            initializer,
        }
    }

    /// Run a full diagnostic check
    pub fn run_full_diagnostic(&self) -> DiagnosticReport {
        let timestamp = SystemTime::now();
        
        // Collect registered commands
        let registered_commands = self.registry.list_available_commands();
        
        // Collect failed commands
        let failed_commands = self.registry.get_failed_commands().clone();
        
        // Collect module states
        let module_states = self.initializer.get_all_states().clone();
        
        // Perform comprehensive validation
        let validation_errors = self.registry.verify_all_commands();
        let mut all_failed_commands = failed_commands;
        all_failed_commands.extend(validation_errors);
        
        // Run health checks on all modules
        let health_check_report = self.initializer.comprehensive_health_check();
        
        // Generate recommendations based on all findings
        let recommendations = self.generate_comprehensive_recommendations(
            &all_failed_commands, 
            &module_states, 
            &health_check_report
        );
        
        // Create detailed summary
        let summary = self.create_detailed_summary(
            &registered_commands, 
            &all_failed_commands, 
            &module_states,
            &health_check_report
        );

        DiagnosticReport {
            timestamp,
            registered_commands,
            failed_commands: all_failed_commands,
            module_states,
            recommendations,
            summary,
        }
    }

    /// Check a specific command with comprehensive analysis
    pub fn check_command(&self, name: &str) -> CommandDiagnostic {
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();
        let mut missing_dependencies = Vec::new();

        // Get command info
        let command_info = self.registry.get_command_info(name);
        
        let (status, dependencies, last_verified) = if let Some(info) = command_info {
            // Check dependencies
            for dep in &info.dependencies {
                if !self.registry.has_command(dep) {
                    missing_dependencies.push(dep.clone());
                    issues.push(format!("Missing dependency: {}", dep));
                } else {
                    // Check dependency health
                    if let Some(dep_status) = self.registry.get_command_status(dep) {
                        if !matches!(dep_status, crate::command_registry::registry::CommandStatus::Registered) {
                            issues.push(format!("Dependency '{}' is not in healthy state: {:?}", dep, dep_status));
                        }
                    }
                }
            }

            // Check command age (if not verified recently)
            if let Ok(duration) = SystemTime::now().duration_since(info.last_verified) {
                if duration.as_secs() > 3600 { // More than 1 hour
                    issues.push("Command has not been verified recently".to_string());
                    recommendations.push("Run command validation to ensure it's still functional".to_string());
                }
            }

            // Check call history
            if info.call_count == 0 {
                issues.push("Command has never been called".to_string());
                recommendations.push("Consider testing the command to ensure it works correctly".to_string());
            }

            (
                format!("{:?}", info.status),
                info.dependencies.clone(),
                info.last_verified,
            )
        } else {
            issues.push("Command not found in registry".to_string());
            recommendations.push("Register the command with the registry".to_string());
            recommendations.push("Check if the command is properly defined with #[tauri::command] attribute".to_string());
            (
                "NotFound".to_string(),
                Vec::new(),
                SystemTime::now(),
            )
        };

        // Generate specific recommendations based on issues
        if !missing_dependencies.is_empty() {
            recommendations.push("Ensure all dependencies are registered before this command".to_string());
            recommendations.push(format!("Missing dependencies: {}", missing_dependencies.join(", ")));
        }

        // Check if command is in invoke_handler
        if command_info.is_some() && issues.is_empty() {
            recommendations.push("Command appears to be properly configured".to_string());
        }

        CommandDiagnostic {
            command_name: name.to_string(),
            status,
            dependencies,
            missing_dependencies,
            last_verified,
            issues,
            recommendations,
        }
    }

    /// Generate intelligent fix suggestions based on errors and system state
    pub fn suggest_fixes(&self, errors: &[CommandError]) -> Vec<String> {
        let mut suggestions = Vec::new();
        let mut error_patterns = HashMap::new();

        // Categorize errors by type
        for error in errors {
            let entry = error_patterns.entry(error.error_type.clone()).or_insert_with(Vec::new);
            entry.push(error);
        }

        // Generate specific suggestions for each error type
        for (error_type, error_list) in error_patterns {
            match error_type {
                crate::command_registry::errors::ErrorType::CommandNotFound => {
                    suggestions.extend(self.suggest_command_not_found_fixes(error_list));
                }
                crate::command_registry::errors::ErrorType::DependencyMissing => {
                    suggestions.extend(self.suggest_dependency_missing_fixes(error_list));
                }
                crate::command_registry::errors::ErrorType::RegistrationFailed => {
                    suggestions.extend(self.suggest_registration_failed_fixes(error_list));
                }
                crate::command_registry::errors::ErrorType::ValidationFailed => {
                    suggestions.extend(self.suggest_validation_failed_fixes(error_list));
                }
                crate::command_registry::errors::ErrorType::RuntimeError => {
                    suggestions.extend(self.suggest_runtime_error_fixes(error_list));
                }
            }
        }

        // Add general system health suggestions
        suggestions.extend(self.suggest_system_health_improvements());

        // Remove duplicates and sort by priority
        suggestions.sort();
        suggestions.dedup();

        suggestions
    }

    /// Suggest fixes for command not found errors
    fn suggest_command_not_found_fixes(&self, errors: Vec<&CommandError>) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        for error in &errors {
            if let Some(command_name) = self.extract_command_name_from_error(error) {
                suggestions.push(format!(
                    "Command '{}' not found. Check if it's properly registered in the invoke_handler! macro in lib.rs",
                    command_name
                ));
                suggestions.push(format!(
                    "Ensure '{}' function is marked with #[tauri::command] attribute",
                    command_name
                ));
                suggestions.push(format!(
                    "Verify that '{}' is imported and accessible in the main module",
                    command_name
                ));
                
                // Check if similar commands exist
                let similar_commands = self.find_similar_commands(&command_name);
                if !similar_commands.is_empty() {
                    suggestions.push(format!(
                        "Did you mean one of these similar commands? {}",
                        similar_commands.join(", ")
                    ));
                }
            }
        }
        
        if errors.len() > 1 {
            suggestions.push("Multiple commands not found. Review your invoke_handler! configuration".to_string());
        }
        
        suggestions
    }

    /// Suggest fixes for dependency missing errors
    fn suggest_dependency_missing_fixes(&self, errors: Vec<&CommandError>) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        for error in errors {
            suggestions.push(format!(
                "Dependency missing: {}. Ensure the dependency module is initialized before this command.",
                error.message
            ));
        }
        
        suggestions.push("Check module initialization order in your startup sequence".to_string());
        suggestions.push("Verify all required modules are properly registered with the ModuleInitializer".to_string());
        suggestions.push("Consider adding retry logic for transient dependency issues".to_string());
        
        suggestions
    }

    /// Suggest fixes for registration failed errors
    fn suggest_registration_failed_fixes(&self, errors: Vec<&CommandError>) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        suggestions.push("Registration failures often indicate duplicate command names or invalid configurations".to_string());
        suggestions.push("Check for duplicate command registrations in your codebase".to_string());
        suggestions.push("Verify command function signatures match Tauri requirements".to_string());
        suggestions.push("Ensure all command dependencies are available during registration".to_string());
        
        for error in errors {
            if error.message.contains("already registered") {
                suggestions.push("Remove duplicate command registrations".to_string());
            }
            if error.message.contains("empty") || error.message.contains("whitespace") {
                suggestions.push("Ensure command names are not empty or whitespace-only".to_string());
            }
        }
        
        suggestions
    }

    /// Suggest fixes for validation failed errors
    fn suggest_validation_failed_fixes(&self, errors: Vec<&CommandError>) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        suggestions.push("Validation failures indicate commands that don't meet correctness requirements".to_string());
        suggestions.push("Run individual command tests to identify specific issues".to_string());
        suggestions.push("Check command implementations for proper error handling".to_string());
        suggestions.push("Verify command parameters and return types are correct".to_string());
        
        for error in errors {
            if error.message.contains("not been verified recently") {
                suggestions.push("Run periodic command validation to ensure continued functionality".to_string());
            }
        }
        
        suggestions
    }

    /// Suggest fixes for runtime errors
    fn suggest_runtime_error_fixes(&self, errors: Vec<&CommandError>) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        suggestions.push("Runtime errors indicate issues during command execution".to_string());
        suggestions.push("Check application logs for detailed error information".to_string());
        suggestions.push("Verify system resources and dependencies are available".to_string());
        suggestions.push("Consider implementing graceful error handling and recovery".to_string());
        
        for error in errors {
            if let Some(context) = &error.context {
                suggestions.push(format!("Review context: {}", context));
            }
        }
        
        suggestions
    }

    /// Suggest general system health improvements
    fn suggest_system_health_improvements(&self) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        let health_report = self.initializer.comprehensive_health_check();
        
        if !health_report.is_system_healthy() {
            suggestions.push("System health check indicates issues. Review module status and dependencies".to_string());
        }
        
        // Check for slow modules
        let slow_modules: Vec<_> = health_report.response_times
            .iter()
            .filter(|(_, duration)| duration.as_millis() > 1000)
            .collect();
            
        if !slow_modules.is_empty() {
            suggestions.push(format!(
                "Slow-responding modules detected: {}. Consider performance optimization.",
                slow_modules.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>().join(", ")
            ));
        }
        
        // Check for dependency issues
        if !health_report.dependency_issues.is_empty() {
            suggestions.push("Dependency issues detected. Review module interdependencies and initialization order".to_string());
        }
        
        suggestions
    }

    /// Find commands with similar names (for typo suggestions)
    fn find_similar_commands(&self, target: &str) -> Vec<String> {
        let available_commands = self.registry.list_available_commands();
        let mut similar = Vec::new();
        
        for command in available_commands {
            if self.calculate_similarity(&command, target) > 0.6 {
                similar.push(command);
            }
        }
        
        similar
    }

    /// Calculate string similarity (simple Levenshtein-based approach)
    fn calculate_similarity(&self, s1: &str, s2: &str) -> f64 {
        let len1 = s1.len();
        let len2 = s2.len();
        
        if len1 == 0 || len2 == 0 {
            return 0.0;
        }
        
        // Simple similarity based on common characters and length
        let common_chars = s1.chars()
            .filter(|c| s2.contains(*c))
            .count();
            
        let max_len = len1.max(len2);
        common_chars as f64 / max_len as f64
    }

    /// Perform automated problem detection and analysis
    pub fn detect_problems(&self) -> Vec<DiagnosticIssue> {
        let mut issues = Vec::new();
        
        // Check for common configuration problems
        issues.extend(self.detect_configuration_issues());
        
        // Check for dependency problems
        issues.extend(self.detect_dependency_issues());
        
        // Check for performance problems
        issues.extend(self.detect_performance_issues());
        
        // Check for security concerns
        issues.extend(self.detect_security_issues());
        
        issues
    }

    /// Detect configuration-related issues
    fn detect_configuration_issues(&self) -> Vec<DiagnosticIssue> {
        let mut issues = Vec::new();
        
        // Check if no commands are registered
        if self.registry.command_count() == 0 {
            issues.push(DiagnosticIssue {
                severity: IssueSeverity::Critical,
                category: IssueCategory::Configuration,
                title: "No commands registered".to_string(),
                description: "The command registry is empty. This suggests a configuration problem.".to_string(),
                recommendations: vec![
                    "Check if commands are properly defined with #[tauri::command]".to_string(),
                    "Verify invoke_handler! macro includes all commands".to_string(),
                    "Ensure command registration is called during startup".to_string(),
                ],
            });
        }
        
        // Check for commands that are registered but never called
        let all_commands = self.registry.get_all_commands();
        let unused_commands: Vec<_> = all_commands
            .iter()
            .filter(|(_, info)| info.call_count == 0)
            .collect();
            
        if unused_commands.len() > all_commands.len() / 2 {
            issues.push(DiagnosticIssue {
                severity: IssueSeverity::Warning,
                category: IssueCategory::Configuration,
                title: "Many unused commands detected".to_string(),
                description: format!("{} out of {} commands have never been called", 
                    unused_commands.len(), all_commands.len()),
                recommendations: vec![
                    "Review if all registered commands are actually needed".to_string(),
                    "Consider removing unused commands to reduce complexity".to_string(),
                    "Add integration tests to verify command functionality".to_string(),
                ],
            });
        }
        
        issues
    }

    /// Detect dependency-related issues
    fn detect_dependency_issues(&self) -> Vec<DiagnosticIssue> {
        let mut issues = Vec::new();
        
        let module_states = self.initializer.get_all_states();
        let failed_modules: Vec<_> = module_states
            .iter()
            .filter(|(_, state)| matches!(state, InitState::Failed(_)))
            .collect();
            
        if !failed_modules.is_empty() {
            issues.push(DiagnosticIssue {
                severity: IssueSeverity::Critical,
                category: IssueCategory::Dependencies,
                title: "Failed module initialization".to_string(),
                description: format!("{} modules failed to initialize", failed_modules.len()),
                recommendations: vec![
                    "Check module initialization logs for specific errors".to_string(),
                    "Verify all module dependencies are available".to_string(),
                    "Consider implementing fallback configurations".to_string(),
                ],
            });
        }
        
        // Check for circular dependencies (this would be detected during initialization)
        let initialization_errors = self.initializer.get_initialization_order();
        if let Err(errors) = initialization_errors {
            for error in errors {
                if error.message.contains("Circular dependency") {
                    issues.push(DiagnosticIssue {
                        severity: IssueSeverity::Critical,
                        category: IssueCategory::Dependencies,
                        title: "Circular dependency detected".to_string(),
                        description: format!("Module '{}' has circular dependencies", error.module_name),
                        recommendations: vec![
                            "Review module dependency graph".to_string(),
                            "Refactor modules to eliminate circular dependencies".to_string(),
                            "Consider using dependency injection patterns".to_string(),
                        ],
                    });
                }
            }
        }
        
        issues
    }

    /// Detect performance-related issues
    fn detect_performance_issues(&self) -> Vec<DiagnosticIssue> {
        let mut issues = Vec::new();
        
        let health_report = self.initializer.comprehensive_health_check();
        
        // Check for slow modules
        let slow_modules: Vec<_> = health_report.response_times
            .iter()
            .filter(|(_, duration)| duration.as_millis() > 1000)
            .collect();
            
        if !slow_modules.is_empty() {
            issues.push(DiagnosticIssue {
                severity: IssueSeverity::Warning,
                category: IssueCategory::Performance,
                title: "Slow module response times".to_string(),
                description: format!("{} modules are responding slowly", slow_modules.len()),
                recommendations: vec![
                    "Profile slow modules to identify bottlenecks".to_string(),
                    "Consider caching or optimization strategies".to_string(),
                    "Review module implementation for efficiency".to_string(),
                ],
            });
        }
        
        issues
    }

    /// Detect security-related issues
    fn detect_security_issues(&self) -> Vec<DiagnosticIssue> {
        let mut issues = Vec::new();
        
        // Check for commands that might have security implications
        let all_commands = self.registry.get_all_commands();
        let potentially_unsafe_commands: Vec<_> = all_commands
            .keys()
            .filter(|name| {
                name.contains("exec") || 
                name.contains("file") || 
                name.contains("system") ||
                name.contains("admin")
            })
            .collect();
            
        if !potentially_unsafe_commands.is_empty() {
            issues.push(DiagnosticIssue {
                severity: IssueSeverity::Info,
                category: IssueCategory::Security,
                title: "Commands with potential security implications".to_string(),
                description: format!("Found {} commands that might need security review", 
                    potentially_unsafe_commands.len()),
                recommendations: vec![
                    "Review command implementations for security best practices".to_string(),
                    "Ensure proper input validation and sanitization".to_string(),
                    "Consider implementing permission checks".to_string(),
                ],
            });
        }
        
        issues
    }

    /// Export diagnostic report in specified format
    pub fn export_report(&self, format: ReportFormat) -> Result<String, DiagnosticError> {
        let report = self.run_full_diagnostic();

        match format {
            ReportFormat::Json => {
                serde_json::to_string_pretty(&report)
                    .map_err(|e| DiagnosticError::new(
                        format!("Failed to serialize report to JSON: {}", e),
                        DiagnosticErrorType::ExportFailed,
                    ))
            }
            ReportFormat::Markdown => {
                Ok(self.format_as_markdown(&report))
            }
            ReportFormat::Html => {
                Ok(self.format_as_html(&report))
            }
        }
    }

    /// Run a quick health check (lightweight version)
    pub fn quick_health_check(&self) -> Result<String, DiagnosticError> {
        let active_commands = self.registry.active_command_count();
        let total_commands = self.registry.command_count();
        let failed_commands = self.registry.get_failed_commands().len();
        
        let status = if failed_commands > 0 {
            "🚨 CRITICAL"
        } else if active_commands < total_commands {
            "⚠️ WARNING"
        } else {
            "✅ HEALTHY"
        };

        Ok(format!(
            "{} - Commands: {}/{} active, {} failed",
            status, active_commands, total_commands, failed_commands
        ))
    }

    /// Get system statistics
    pub fn get_system_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        
        stats.insert("total_commands".to_string(), 
            serde_json::Value::Number(serde_json::Number::from(self.registry.command_count())));
        stats.insert("active_commands".to_string(), 
            serde_json::Value::Number(serde_json::Number::from(self.registry.active_command_count())));
        stats.insert("failed_commands".to_string(), 
            serde_json::Value::Number(serde_json::Number::from(self.registry.get_failed_commands().len())));
        
        let module_states = self.initializer.get_all_states();
        stats.insert("total_modules".to_string(), 
            serde_json::Value::Number(serde_json::Number::from(module_states.len())));
        
        let ready_modules = module_states.values()
            .filter(|state| matches!(state, InitState::Ready))
            .count();
        stats.insert("ready_modules".to_string(), 
            serde_json::Value::Number(serde_json::Number::from(ready_modules)));
        
        let failed_modules = module_states.values()
            .filter(|state| matches!(state, InitState::Failed(_)))
            .count();
        stats.insert("failed_modules".to_string(), 
            serde_json::Value::Number(serde_json::Number::from(failed_modules)));

        stats.insert("timestamp".to_string(), 
            serde_json::Value::String(format!("{:?}", SystemTime::now())));

        stats
    }

    /// Check if system is ready for operation
    pub fn is_system_ready(&self) -> bool {
        let failed_commands = self.registry.get_failed_commands().len();
        let module_states = self.initializer.get_all_states();
        let failed_modules = module_states.values()
            .filter(|state| matches!(state, InitState::Failed(_)))
            .count();
        
        failed_commands == 0 && failed_modules == 0
    }

    /// Get detailed command analysis
    pub fn analyze_all_commands(&self) -> HashMap<String, CommandDiagnostic> {
        let mut analysis = HashMap::new();
        
        // Analyze all registered commands
        for command_name in self.registry.list_available_commands() {
            let diagnostic = self.check_command(&command_name);
            analysis.insert(command_name, diagnostic);
        }
        
        // Also analyze failed commands if we can get their names
        for error in self.registry.get_failed_commands() {
            // Try to extract command name from error message
            if let Some(command_name) = self.extract_command_name_from_error(error) {
                if !analysis.contains_key(&command_name) {
                    let diagnostic = self.check_command(&command_name);
                    analysis.insert(command_name, diagnostic);
                }
            }
        }
        
        analysis
    }

    /// Run automated diagnostic analysis
    pub fn run_automated_analysis(&self) -> AutomatedAnalysisReport {
        let timestamp = SystemTime::now();
        
        // Detect problems automatically
        let detected_issues = self.detect_problems();
        
        // Generate intelligent recommendations
        let failed_commands = self.registry.get_failed_commands();
        let intelligent_suggestions = self.suggest_fixes(failed_commands);
        
        // Analyze system trends (if we had historical data)
        let trend_analysis = self.analyze_system_trends();
        
        // Generate risk assessment
        let risk_assessment = self.assess_system_risks(&detected_issues);
        
        AutomatedAnalysisReport {
            timestamp,
            detected_issues: detected_issues.clone(),
            intelligent_suggestions,
            trend_analysis,
            risk_assessment,
            overall_score: self.calculate_system_health_score(&detected_issues),
        }
    }

    /// Analyze system trends (placeholder for future enhancement)
    fn analyze_system_trends(&self) -> TrendAnalysis {
        // This would analyze historical data if available
        // For now, return basic current state analysis
        TrendAnalysis {
            command_usage_trend: "Stable".to_string(),
            error_rate_trend: "Stable".to_string(),
            performance_trend: "Stable".to_string(),
            recommendations: vec![
                "Enable historical data collection for trend analysis".to_string(),
                "Implement periodic health monitoring".to_string(),
            ],
        }
    }

    /// Assess system risks based on detected issues
    fn assess_system_risks(&self, issues: &[DiagnosticIssue]) -> RiskAssessment {
        let critical_count = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Critical)).count();
        let warning_count = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Warning)).count();
        
        let risk_level = if critical_count > 0 {
            RiskLevel::High
        } else if warning_count > 3 {
            RiskLevel::Medium
        } else if warning_count > 0 {
            RiskLevel::Low
        } else {
            RiskLevel::Minimal
        };
        
        let mut risk_factors = Vec::new();
        
        if critical_count > 0 {
            risk_factors.push(format!("{} critical issues requiring immediate attention", critical_count));
        }
        
        if warning_count > 0 {
            risk_factors.push(format!("{} warning-level issues", warning_count));
        }
        
        // Check for specific high-risk patterns
        for issue in issues {
            if issue.category == IssueCategory::Security {
                risk_factors.push("Security-related issues detected".to_string());
            }
            if issue.category == IssueCategory::Dependencies && issue.severity == IssueSeverity::Critical {
                risk_factors.push("Critical dependency failures".to_string());
            }
        }
        
        let mitigation_strategies = self.generate_mitigation_strategies(&risk_level, issues);
        
        RiskAssessment {
            risk_level: risk_level.clone(),
            risk_factors,
            mitigation_strategies,
            estimated_impact: self.estimate_impact(&risk_level),
        }
    }

    /// Generate mitigation strategies based on risk level and issues
    fn generate_mitigation_strategies(&self, risk_level: &RiskLevel, issues: &[DiagnosticIssue]) -> Vec<String> {
        let mut strategies = Vec::new();
        
        match risk_level {
            RiskLevel::High => {
                strategies.push("Immediate action required - address critical issues first".to_string());
                strategies.push("Consider implementing emergency fallback procedures".to_string());
                strategies.push("Increase monitoring and alerting".to_string());
            }
            RiskLevel::Medium => {
                strategies.push("Schedule maintenance window to address issues".to_string());
                strategies.push("Implement additional monitoring for early warning".to_string());
            }
            RiskLevel::Low => {
                strategies.push("Address issues during regular maintenance".to_string());
                strategies.push("Continue regular monitoring".to_string());
            }
            RiskLevel::Minimal => {
                strategies.push("Maintain current monitoring practices".to_string());
                strategies.push("Consider proactive improvements".to_string());
            }
        }
        
        // Add specific strategies based on issue categories
        let has_dependency_issues = issues.iter().any(|i| i.category == IssueCategory::Dependencies);
        if has_dependency_issues {
            strategies.push("Review and strengthen dependency management".to_string());
        }
        
        let has_performance_issues = issues.iter().any(|i| i.category == IssueCategory::Performance);
        if has_performance_issues {
            strategies.push("Implement performance monitoring and optimization".to_string());
        }
        
        strategies
    }

    /// Estimate impact of current risk level
    fn estimate_impact(&self, risk_level: &RiskLevel) -> String {
        match risk_level {
            RiskLevel::High => "High - System functionality may be severely impacted".to_string(),
            RiskLevel::Medium => "Medium - Some features may be affected".to_string(),
            RiskLevel::Low => "Low - Minor impact on system performance".to_string(),
            RiskLevel::Minimal => "Minimal - System operating normally".to_string(),
        }
    }

    /// Calculate overall system health score (0-100)
    fn calculate_system_health_score(&self, issues: &[DiagnosticIssue]) -> u8 {
        let mut score = 100u8;
        
        for issue in issues {
            let deduction = match issue.severity {
                IssueSeverity::Critical => 25,
                IssueSeverity::Warning => 10,
                IssueSeverity::Info => 2,
            };
            score = score.saturating_sub(deduction);
        }
        
        // Additional factors
        let failed_commands = self.registry.get_failed_commands().len();
        if failed_commands > 0 {
            score = score.saturating_sub((failed_commands * 5) as u8);
        }
        
        let module_states = self.initializer.get_all_states();
        let failed_modules = module_states.values()
            .filter(|state| matches!(state, InitState::Failed(_)))
            .count();
        if failed_modules > 0 {
            score = score.saturating_sub((failed_modules * 10) as u8);
        }
        
        score
    }

    /// Extract command name from error message (helper method)
    fn extract_command_name_from_error(&self, error: &CommandError) -> Option<String> {
        // Simple pattern matching to extract command names from error messages
        // This could be enhanced with more sophisticated parsing
        if error.message.contains("Command '") {
            if let Some(start) = error.message.find("Command '") {
                let start = start + 9; // Length of "Command '"
                if let Some(end) = error.message[start..].find("'") {
                    return Some(error.message[start..start + end].to_string());
                }
            }
        }
        None
    }

    /// Generate comprehensive recommendations based on current state
    fn generate_comprehensive_recommendations(
        &self, 
        failed_commands: &[CommandError], 
        module_states: &HashMap<String, InitState>,
        health_report: &crate::command_registry::initializer::HealthCheckReport
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check for failed commands
        if !failed_commands.is_empty() {
            recommendations.push(format!(
                "Found {} failed commands. Review command implementations and dependencies.",
                failed_commands.len()
            ));
            
            // Categorize failures
            let mut registration_failures = 0;
            let mut dependency_failures = 0;
            let mut validation_failures = 0;
            
            for error in failed_commands {
                match &error.error_type {
                    crate::command_registry::errors::ErrorType::RegistrationFailed => registration_failures += 1,
                    crate::command_registry::errors::ErrorType::DependencyMissing => dependency_failures += 1,
                    crate::command_registry::errors::ErrorType::ValidationFailed => validation_failures += 1,
                    _ => {}
                }
            }
            
            if registration_failures > 0 {
                recommendations.push(format!(
                    "Found {} registration failures. Check command definitions and invoke_handler configuration.",
                    registration_failures
                ));
            }
            
            if dependency_failures > 0 {
                recommendations.push(format!(
                    "Found {} dependency failures. Ensure all required modules are initialized first.",
                    dependency_failures
                ));
            }
            
            if validation_failures > 0 {
                recommendations.push(format!(
                    "Found {} validation failures. Review command implementations for correctness.",
                    validation_failures
                ));
            }
        }

        // Check for failed modules
        let failed_modules: Vec<_> = module_states
            .iter()
            .filter(|(_, state)| matches!(state, InitState::Failed(_)))
            .collect();

        if !failed_modules.is_empty() {
            recommendations.push(format!(
                "Found {} failed modules. Check module initialization logic.",
                failed_modules.len()
            ));
            
            for (module_name, state) in &failed_modules {
                if let InitState::Failed(reason) = state {
                    recommendations.push(format!(
                        "Module '{}' failed: {}. Consider checking dependencies and configuration.",
                        module_name, reason
                    ));
                }
            }
        }

        // Check for uninitialized modules
        let pending_modules: Vec<_> = module_states
            .iter()
            .filter(|(_, state)| matches!(state, InitState::Pending))
            .collect();

        if !pending_modules.is_empty() {
            recommendations.push(format!(
                "Found {} pending modules. Ensure initialization is completed.",
                pending_modules.len()
            ));
        }

        // Check health report
        if !health_report.is_system_healthy() {
            recommendations.push("System health check indicates issues. Review module health status.".to_string());
            
            let critical_modules = health_report.get_critical_modules();
            if !critical_modules.is_empty() {
                recommendations.push(format!(
                    "Critical modules with issues: {}. Immediate attention required.",
                    critical_modules.join(", ")
                ));
            }
        }

        // Check for dependency issues
        if !health_report.dependency_issues.is_empty() {
            recommendations.push(format!(
                "Found {} dependency issues. Review module interdependencies.",
                health_report.dependency_issues.len()
            ));
        }

        // Performance recommendations
        let slow_modules: Vec<_> = health_report.response_times
            .iter()
            .filter(|(_, duration)| duration.as_millis() > 1000) // More than 1 second
            .collect();
            
        if !slow_modules.is_empty() {
            recommendations.push(format!(
                "Found {} slow-responding modules. Consider performance optimization.",
                slow_modules.len()
            ));
        }

        if recommendations.is_empty() {
            recommendations.push("System appears to be healthy. No immediate issues detected.".to_string());
            recommendations.push("Consider running periodic health checks to maintain system reliability.".to_string());
        }

        recommendations
    }

    /// Generate recommendations based on current state (legacy method)
    fn generate_recommendations(&self, failed_commands: &[CommandError], module_states: &HashMap<String, InitState>) -> Vec<String> {
        // Use the comprehensive method with a minimal health report
        let minimal_health_report = crate::command_registry::initializer::HealthCheckReport::new();
        self.generate_comprehensive_recommendations(failed_commands, module_states, &minimal_health_report)
    }

    /// Create detailed diagnostic summary
    fn create_detailed_summary(
        &self, 
        registered_commands: &[String], 
        failed_commands: &[CommandError], 
        module_states: &HashMap<String, InitState>,
        health_report: &crate::command_registry::initializer::HealthCheckReport
    ) -> DiagnosticSummary {
        let total_commands = self.registry.command_count();
        let active_commands = registered_commands.len();
        let failed_commands_count = failed_commands.len();
        
        let total_modules = module_states.len();
        let ready_modules = module_states
            .values()
            .filter(|state| matches!(state, InitState::Ready))
            .count();
        let failed_modules = module_states
            .values()
            .filter(|state| matches!(state, InitState::Failed(_)))
            .count();

        // Determine overall health based on multiple factors
        let overall_health = if failed_commands_count > 0 || failed_modules > 0 {
            HealthStatus::Critical
        } else if total_commands != active_commands || total_modules != ready_modules {
            HealthStatus::Warning
        } else if !health_report.is_system_healthy() {
            match health_report.overall_status {
                crate::command_registry::initializer::SystemHealthStatus::Critical => HealthStatus::Critical,
                crate::command_registry::initializer::SystemHealthStatus::Degraded => HealthStatus::Warning,
                crate::command_registry::initializer::SystemHealthStatus::Failed => HealthStatus::Critical,
                crate::command_registry::initializer::SystemHealthStatus::Healthy => HealthStatus::Healthy,
            }
        } else {
            HealthStatus::Healthy
        };

        DiagnosticSummary {
            total_commands,
            active_commands,
            failed_commands: failed_commands_count,
            total_modules,
            ready_modules,
            failed_modules,
            overall_health,
        }
    }

    /// Create diagnostic summary (legacy method)
    fn create_summary(&self, registered_commands: &[String], failed_commands: &[CommandError], module_states: &HashMap<String, InitState>) -> DiagnosticSummary {
        // Use the detailed method with a minimal health report
        let minimal_health_report = crate::command_registry::initializer::HealthCheckReport::new();
        self.create_detailed_summary(registered_commands, failed_commands, module_states, &minimal_health_report)
    }

    /// Format report as Markdown with comprehensive details
    fn format_as_markdown(&self, report: &DiagnosticReport) -> String {
        let mut md = String::new();
        
        md.push_str("# Command Registry Diagnostic Report\n\n");
        md.push_str(&format!("**Generated:** {:?}\n\n", report.timestamp));
        
        // Executive Summary
        md.push_str("## Executive Summary\n\n");
        md.push_str(&format!("- **Overall Health:** {:?}\n", report.summary.overall_health));
        md.push_str(&format!("- **System Status:** {}\n", 
            match report.summary.overall_health {
                HealthStatus::Healthy => "✅ All systems operational",
                HealthStatus::Warning => "⚠️ Some issues detected, system functional",
                HealthStatus::Critical => "🚨 Critical issues require immediate attention",
            }
        ));
        md.push_str(&format!("- **Commands:** {}/{} active ({} failed)\n", 
            report.summary.active_commands, 
            report.summary.total_commands,
            report.summary.failed_commands
        ));
        md.push_str(&format!("- **Modules:** {}/{} ready ({} failed)\n\n", 
            report.summary.ready_modules, 
            report.summary.total_modules,
            report.summary.failed_modules
        ));

        // Detailed Metrics
        md.push_str("## Detailed Metrics\n\n");
        md.push_str("| Metric | Count | Status |\n");
        md.push_str("|--------|-------|--------|\n");
        md.push_str(&format!("| Total Commands | {} | {} |\n", 
            report.summary.total_commands,
            if report.summary.total_commands > 0 { "✅" } else { "⚠️" }
        ));
        md.push_str(&format!("| Active Commands | {} | {} |\n", 
            report.summary.active_commands,
            if report.summary.active_commands == report.summary.total_commands { "✅" } else { "⚠️" }
        ));
        md.push_str(&format!("| Failed Commands | {} | {} |\n", 
            report.summary.failed_commands,
            if report.summary.failed_commands == 0 { "✅" } else { "🚨" }
        ));
        md.push_str(&format!("| Total Modules | {} | {} |\n", 
            report.summary.total_modules,
            if report.summary.total_modules > 0 { "✅" } else { "⚠️" }
        ));
        md.push_str(&format!("| Ready Modules | {} | {} |\n", 
            report.summary.ready_modules,
            if report.summary.ready_modules == report.summary.total_modules { "✅" } else { "⚠️" }
        ));
        md.push_str(&format!("| Failed Modules | {} | {} |\n\n", 
            report.summary.failed_modules,
            if report.summary.failed_modules == 0 { "✅" } else { "🚨" }
        ));

        // Registered Commands
        md.push_str("## Registered Commands\n\n");
        if report.registered_commands.is_empty() {
            md.push_str("⚠️ No commands are currently registered.\n\n");
        } else {
            md.push_str(&format!("Found {} registered commands:\n\n", report.registered_commands.len()));
            for (i, command) in report.registered_commands.iter().enumerate() {
                md.push_str(&format!("{}. `{}`\n", i + 1, command));
            }
            md.push_str("\n");
        }

        // Failed Commands
        if !report.failed_commands.is_empty() {
            md.push_str("## 🚨 Failed Commands\n\n");
            for (i, error) in report.failed_commands.iter().enumerate() {
                md.push_str(&format!("### {}. {}\n\n", i + 1, error.message));
                md.push_str(&format!("- **Error Type:** {:?}\n", error.error_type));
                if let Some(context) = &error.context {
                    md.push_str(&format!("- **Context:** {}\n", context));
                }
                md.push_str(&format!("- **Timestamp:** {:?}\n\n", error.timestamp));
            }
        }

        // Module States
        md.push_str("## Module States\n\n");
        if report.module_states.is_empty() {
            md.push_str("⚠️ No modules are currently tracked.\n\n");
        } else {
            md.push_str("| Module | State | Status |\n");
            md.push_str("|--------|-------|--------|\n");
            for (module, state) in &report.module_states {
                let (state_str, status_icon) = match state {
                    InitState::Ready => ("Ready", "✅"),
                    InitState::Pending => ("Pending", "⏳"),
                    InitState::Initializing => ("Initializing", "🔄"),
                    InitState::Failed(reason) => (reason.as_str(), "🚨"),
                };
                md.push_str(&format!("| {} | {} | {} |\n", module, state_str, status_icon));
            }
            md.push_str("\n");
        }

        // Recommendations
        md.push_str("## 📋 Recommendations\n\n");
        if report.recommendations.is_empty() {
            md.push_str("✅ No specific recommendations at this time.\n\n");
        } else {
            for (i, recommendation) in report.recommendations.iter().enumerate() {
                md.push_str(&format!("{}. {}\n", i + 1, recommendation));
            }
            md.push_str("\n");
        }

        // Footer
        md.push_str("---\n\n");
        md.push_str("*This report was generated automatically by the Command Registry Diagnostic Tool.*\n");
        md.push_str("*For more information, run individual command diagnostics or check system logs.*\n");

        md
    }

    /// Format report as HTML with enhanced styling
    fn format_as_html(&self, report: &DiagnosticReport) -> String {
        let mut html = String::new();
        
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<title>Command Registry Diagnostic Report</title>\n");
        html.push_str("<meta charset=\"UTF-8\">\n");
        html.push_str("<style>\n");
        html.push_str("body { font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 20px; background-color: #f5f5f5; }\n");
        html.push_str(".container { max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }\n");
        html.push_str("h1 { color: #2c3e50; border-bottom: 3px solid #3498db; padding-bottom: 10px; }\n");
        html.push_str("h2 { color: #34495e; margin-top: 30px; }\n");
        html.push_str(".summary { background: #ecf0f1; padding: 20px; border-radius: 5px; margin: 20px 0; }\n");
        html.push_str(".status-healthy { color: #27ae60; font-weight: bold; }\n");
        html.push_str(".status-warning { color: #f39c12; font-weight: bold; }\n");
        html.push_str(".status-critical { color: #e74c3c; font-weight: bold; }\n");
        html.push_str("table { width: 100%; border-collapse: collapse; margin: 20px 0; }\n");
        html.push_str("th, td { padding: 12px; text-align: left; border-bottom: 1px solid #ddd; }\n");
        html.push_str("th { background-color: #3498db; color: white; }\n");
        html.push_str("tr:nth-child(even) { background-color: #f2f2f2; }\n");
        html.push_str(".command-list { background: #f8f9fa; padding: 15px; border-radius: 5px; }\n");
        html.push_str(".error-item { background: #fdf2f2; border-left: 4px solid #e74c3c; padding: 15px; margin: 10px 0; }\n");
        html.push_str(".recommendation { background: #f0f8ff; border-left: 4px solid #3498db; padding: 15px; margin: 10px 0; }\n");
        html.push_str(".timestamp { color: #7f8c8d; font-size: 0.9em; }\n");
        html.push_str("</style>\n");
        html.push_str("</head>\n<body>\n");
        
        html.push_str("<div class=\"container\">\n");
        html.push_str("<h1>🔍 Command Registry Diagnostic Report</h1>\n");
        html.push_str(&format!("<p class=\"timestamp\"><strong>Generated:</strong> {:?}</p>\n", report.timestamp));
        
        // Executive Summary
        html.push_str("<div class=\"summary\">\n");
        html.push_str("<h2>📊 Executive Summary</h2>\n");
        
        let (health_class, health_icon) = match report.summary.overall_health {
            HealthStatus::Healthy => ("status-healthy", "✅"),
            HealthStatus::Warning => ("status-warning", "⚠️"),
            HealthStatus::Critical => ("status-critical", "🚨"),
        };
        
        html.push_str(&format!("<p><strong>Overall Health:</strong> <span class=\"{}\">{} {:?}</span></p>\n", 
            health_class, health_icon, report.summary.overall_health));
        html.push_str(&format!("<p><strong>Commands:</strong> {}/{} active ({} failed)</p>\n", 
            report.summary.active_commands, report.summary.total_commands, report.summary.failed_commands));
        html.push_str(&format!("<p><strong>Modules:</strong> {}/{} ready ({} failed)</p>\n", 
            report.summary.ready_modules, report.summary.total_modules, report.summary.failed_modules));
        html.push_str("</div>\n");

        // Detailed Metrics Table
        html.push_str("<h2>📈 Detailed Metrics</h2>\n");
        html.push_str("<table>\n<thead>\n<tr><th>Metric</th><th>Count</th><th>Status</th></tr>\n</thead>\n<tbody>\n");
        html.push_str(&format!("<tr><td>Total Commands</td><td>{}</td><td>{}</td></tr>\n", 
            report.summary.total_commands,
            if report.summary.total_commands > 0 { "✅" } else { "⚠️" }
        ));
        html.push_str(&format!("<tr><td>Active Commands</td><td>{}</td><td>{}</td></tr>\n", 
            report.summary.active_commands,
            if report.summary.active_commands == report.summary.total_commands { "✅" } else { "⚠️" }
        ));
        html.push_str(&format!("<tr><td>Failed Commands</td><td>{}</td><td>{}</td></tr>\n", 
            report.summary.failed_commands,
            if report.summary.failed_commands == 0 { "✅" } else { "🚨" }
        ));
        html.push_str(&format!("<tr><td>Total Modules</td><td>{}</td><td>{}</td></tr>\n", 
            report.summary.total_modules,
            if report.summary.total_modules > 0 { "✅" } else { "⚠️" }
        ));
        html.push_str(&format!("<tr><td>Ready Modules</td><td>{}</td><td>{}</td></tr>\n", 
            report.summary.ready_modules,
            if report.summary.ready_modules == report.summary.total_modules { "✅" } else { "⚠️" }
        ));
        html.push_str(&format!("<tr><td>Failed Modules</td><td>{}</td><td>{}</td></tr>\n", 
            report.summary.failed_modules,
            if report.summary.failed_modules == 0 { "✅" } else { "🚨" }
        ));
        html.push_str("</tbody>\n</table>\n");

        // Registered Commands
        html.push_str("<h2>📋 Registered Commands</h2>\n");
        if report.registered_commands.is_empty() {
            html.push_str("<p class=\"status-warning\">⚠️ No commands are currently registered.</p>\n");
        } else {
            html.push_str("<div class=\"command-list\">\n");
            html.push_str(&format!("<p><strong>Found {} registered commands:</strong></p>\n", report.registered_commands.len()));
            html.push_str("<ul>\n");
            for command in &report.registered_commands {
                html.push_str(&format!("<li><code>{}</code></li>\n", command));
            }
            html.push_str("</ul>\n</div>\n");
        }

        // Failed Commands
        if !report.failed_commands.is_empty() {
            html.push_str("<h2>🚨 Failed Commands</h2>\n");
            for (i, error) in report.failed_commands.iter().enumerate() {
                html.push_str("<div class=\"error-item\">\n");
                html.push_str(&format!("<h3>{}. {}</h3>\n", i + 1, error.message));
                html.push_str(&format!("<p><strong>Error Type:</strong> {:?}</p>\n", error.error_type));
                if let Some(context) = &error.context {
                    html.push_str(&format!("<p><strong>Context:</strong> {}</p>\n", context));
                }
                html.push_str(&format!("<p class=\"timestamp\"><strong>Timestamp:</strong> {:?}</p>\n", error.timestamp));
                html.push_str("</div>\n");
            }
        }

        // Module States
        html.push_str("<h2>🔧 Module States</h2>\n");
        if report.module_states.is_empty() {
            html.push_str("<p class=\"status-warning\">⚠️ No modules are currently tracked.</p>\n");
        } else {
            html.push_str("<table>\n<thead>\n<tr><th>Module</th><th>State</th><th>Status</th></tr>\n</thead>\n<tbody>\n");
            for (module, state) in &report.module_states {
                let (state_str, status_icon) = match state {
                    InitState::Ready => ("Ready", "✅"),
                    InitState::Pending => ("Pending", "⏳"),
                    InitState::Initializing => ("Initializing", "🔄"),
                    InitState::Failed(reason) => (reason.as_str(), "🚨"),
                };
                html.push_str(&format!("<tr><td>{}</td><td>{}</td><td>{}</td></tr>\n", module, state_str, status_icon));
            }
            html.push_str("</tbody>\n</table>\n");
        }

        // Recommendations
        html.push_str("<h2>💡 Recommendations</h2>\n");
        if report.recommendations.is_empty() {
            html.push_str("<p class=\"status-healthy\">✅ No specific recommendations at this time.</p>\n");
        } else {
            for (i, recommendation) in report.recommendations.iter().enumerate() {
                html.push_str("<div class=\"recommendation\">\n");
                html.push_str(&format!("<p><strong>{}.</strong> {}</p>\n", i + 1, recommendation));
                html.push_str("</div>\n");
            }
        }

        // Footer
        html.push_str("<hr>\n");
        html.push_str("<p class=\"timestamp\"><em>This report was generated automatically by the Command Registry Diagnostic Tool.</em></p>\n");
        html.push_str("<p class=\"timestamp\"><em>For more information, run individual command diagnostics or check system logs.</em></p>\n");
        
        html.push_str("</div>\n</body>\n</html>");
        html
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_registry::{CommandRegistry, ModuleInitializer, CommandInfo, CommandStatus};
    use crate::command_registry::errors::{CommandError, ErrorType};

    #[test]
    fn test_diagnostic_tool_creation() {
        let registry = Arc::new(CommandRegistry::new());
        let initializer = Arc::new(ModuleInitializer::new());
        let diagnostic = DiagnosticTool::new(registry, initializer);
        
        let report = diagnostic.run_full_diagnostic();
        assert_eq!(report.registered_commands.len(), 0);
        assert_eq!(report.failed_commands.len(), 0);
    }

    #[test]
    fn test_command_diagnostic_not_found() {
        let registry = Arc::new(CommandRegistry::new());
        let initializer = Arc::new(ModuleInitializer::new());
        let diagnostic = DiagnosticTool::new(registry, initializer);
        
        let result = diagnostic.check_command("non_existent_command");
        assert_eq!(result.command_name, "non_existent_command");
        assert_eq!(result.status, "NotFound");
        assert!(!result.issues.is_empty());
        assert!(result.issues.iter().any(|issue| issue.contains("Command not found")));
        assert!(!result.recommendations.is_empty());
    }

    #[test]
    fn test_command_diagnostic_with_registered_command() {
        let mut registry = CommandRegistry::new();
        let command_info = CommandInfo::new("test_command".to_string()).mark_registered();
        registry.register_command(command_info).unwrap();
        
        let registry = Arc::new(registry);
        let initializer = Arc::new(ModuleInitializer::new());
        let diagnostic = DiagnosticTool::new(registry, initializer);
        
        let result = diagnostic.check_command("test_command");
        assert_eq!(result.command_name, "test_command");
        assert_eq!(result.status, "Registered");
        assert_eq!(result.dependencies.len(), 0);
        assert_eq!(result.missing_dependencies.len(), 0);
    }

    #[test]
    fn test_command_diagnostic_with_dependencies() {
        let mut registry = CommandRegistry::new();
        
        // Register dependency first
        let dep_command = CommandInfo::new("dependency_command".to_string()).mark_registered();
        registry.register_command(dep_command).unwrap();
        
        // Register main command with dependency
        let main_command = CommandInfo::with_dependencies(
            "main_command".to_string(),
            vec!["dependency_command".to_string()]
        ).mark_registered();
        registry.register_command(main_command).unwrap();
        
        let registry = Arc::new(registry);
        let initializer = Arc::new(ModuleInitializer::new());
        let diagnostic = DiagnosticTool::new(registry, initializer);
        
        let result = diagnostic.check_command("main_command");
        assert_eq!(result.command_name, "main_command");
        assert_eq!(result.dependencies.len(), 1);
        assert_eq!(result.missing_dependencies.len(), 0);
        assert!(result.dependencies.contains(&"dependency_command".to_string()));
    }

    #[test]
    fn test_command_diagnostic_with_missing_dependencies() {
        let mut registry = CommandRegistry::new();
        
        // Register command with missing dependency
        let main_command = CommandInfo::with_dependencies(
            "main_command".to_string(),
            vec!["missing_dependency".to_string()]
        );
        registry.register_command(main_command).unwrap();
        
        let registry = Arc::new(registry);
        let initializer = Arc::new(ModuleInitializer::new());
        let diagnostic = DiagnosticTool::new(registry, initializer);
        
        let result = diagnostic.check_command("main_command");
        assert_eq!(result.command_name, "main_command");
        assert_eq!(result.missing_dependencies.len(), 1);
        assert!(result.missing_dependencies.contains(&"missing_dependency".to_string()));
        assert!(!result.issues.is_empty());
        assert!(result.issues.iter().any(|issue| issue.contains("Missing dependency")));
    }

    #[test]
    fn test_full_diagnostic_report_generation() {
        let mut registry = CommandRegistry::new();
        
        // Add some test commands
        let cmd1 = CommandInfo::new("command1".to_string()).mark_registered();
        let cmd2 = CommandInfo::new("command2".to_string()).mark_registered();
        registry.register_command(cmd1).unwrap();
        registry.register_command(cmd2).unwrap();
        
        let registry = Arc::new(registry);
        let initializer = Arc::new(ModuleInitializer::new());
        let diagnostic = DiagnosticTool::new(registry, initializer);
        
        let report = diagnostic.run_full_diagnostic();
        
        assert_eq!(report.registered_commands.len(), 2);
        assert!(report.registered_commands.contains(&"command1".to_string()));
        assert!(report.registered_commands.contains(&"command2".to_string()));
        assert_eq!(report.summary.total_commands, 2);
        assert_eq!(report.summary.active_commands, 2);
        assert_eq!(report.summary.failed_commands, 0);
        assert!(matches!(report.summary.overall_health, HealthStatus::Healthy));
    }

    #[test]
    fn test_diagnostic_report_with_failures() {
        let mut registry = CommandRegistry::new();
        
        // Add a failed command
        registry.mark_command_failed("failed_command", "Test failure".to_string());
        
        let registry = Arc::new(registry);
        let initializer = Arc::new(ModuleInitializer::new());
        let diagnostic = DiagnosticTool::new(registry, initializer);
        
        let report = diagnostic.run_full_diagnostic();
        
        assert_eq!(report.failed_commands.len(), 1);
        assert_eq!(report.summary.failed_commands, 1);
        assert!(matches!(report.summary.overall_health, HealthStatus::Critical));
        assert!(!report.recommendations.is_empty());
    }

    #[test]
    fn test_suggest_fixes_command_not_found() {
        let registry = Arc::new(CommandRegistry::new());
        let initializer = Arc::new(ModuleInitializer::new());
        let diagnostic = DiagnosticTool::new(registry, initializer);
        
        let errors = vec![
            CommandError::new(
                "Command 'test_command' not found".to_string(),
                ErrorType::CommandNotFound,
            )
        ];
        
        let suggestions = diagnostic.suggest_fixes(&errors);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("invoke_handler")));
        assert!(suggestions.iter().any(|s| s.contains("#[tauri::command]")));
    }

    #[test]
    fn test_suggest_fixes_dependency_missing() {
        let registry = Arc::new(CommandRegistry::new());
        let initializer = Arc::new(ModuleInitializer::new());
        let diagnostic = DiagnosticTool::new(registry, initializer);
        
        let errors = vec![
            CommandError::new(
                "Dependency missing: database_module".to_string(),
                ErrorType::DependencyMissing,
            )
        ];
        
        let suggestions = diagnostic.suggest_fixes(&errors);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("dependency module is initialized")));
        assert!(suggestions.iter().any(|s| s.contains("initialization order")));
    }

    #[test]
    fn test_suggest_fixes_registration_failed() {
        let registry = Arc::new(CommandRegistry::new());
        let initializer = Arc::new(ModuleInitializer::new());
        let diagnostic = DiagnosticTool::new(registry, initializer);
        
        let errors = vec![
            CommandError::new(
                "Command 'test' is already registered".to_string(),
                ErrorType::RegistrationFailed,
            )
        ];
        
        let suggestions = diagnostic.suggest_fixes(&errors);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("duplicate")));
    }

    #[test]
    fn test_export_report_json() {
        let registry = Arc::new(CommandRegistry::new());
        let initializer = Arc::new(ModuleInitializer::new());
        let diagnostic = DiagnosticTool::new(registry, initializer);
        
        let result = diagnostic.export_report(ReportFormat::Json);
        assert!(result.is_ok());
        
        let json_str = result.unwrap();
        assert!(json_str.contains("timestamp"));
        assert!(json_str.contains("registered_commands"));
        assert!(json_str.contains("failed_commands"));
    }

    #[test]
    fn test_export_report_markdown() {
        let registry = Arc::new(CommandRegistry::new());
        let initializer = Arc::new(ModuleInitializer::new());
        let diagnostic = DiagnosticTool::new(registry, initializer);
        
        let result = diagnostic.export_report(ReportFormat::Markdown);
        assert!(result.is_ok());
        
        let markdown = result.unwrap();
        assert!(markdown.contains("# Command Registry Diagnostic Report"));
        assert!(markdown.contains("## Executive Summary"));
        assert!(markdown.contains("## Recommendations"));
    }

    #[test]
    fn test_export_report_html() {
        let registry = Arc::new(CommandRegistry::new());
        let initializer = Arc::new(ModuleInitializer::new());
        let diagnostic = DiagnosticTool::new(registry, initializer);
        
        let result = diagnostic.export_report(ReportFormat::Html);
        assert!(result.is_ok());
        
        let html = result.unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Command Registry Diagnostic Report</title>"));
        assert!(html.contains("Executive Summary"));
    }

    #[test]
    fn test_quick_health_check() {
        let registry = Arc::new(CommandRegistry::new());
        let initializer = Arc::new(ModuleInitializer::new());
        let diagnostic = DiagnosticTool::new(registry, initializer);
        
        let result = diagnostic.quick_health_check();
        assert!(result.is_ok());
        
        let health_status = result.unwrap();
        assert!(health_status.contains("HEALTHY") || health_status.contains("WARNING") || health_status.contains("CRITICAL"));
        assert!(health_status.contains("Commands:"));
    }

    #[test]
    fn test_system_stats() {
        let mut registry = CommandRegistry::new();
        let cmd = CommandInfo::new("test_command".to_string()).mark_registered();
        registry.register_command(cmd).unwrap();
        
        let registry = Arc::new(registry);
        let initializer = Arc::new(ModuleInitializer::new());
        let diagnostic = DiagnosticTool::new(registry, initializer);
        
        let stats = diagnostic.get_system_stats();
        
        assert!(stats.contains_key("total_commands"));
        assert!(stats.contains_key("active_commands"));
        assert!(stats.contains_key("failed_commands"));
        assert!(stats.contains_key("timestamp"));
        
        if let Some(serde_json::Value::Number(total)) = stats.get("total_commands") {
            assert_eq!(total.as_u64().unwrap(), 1);
        } else {
            panic!("total_commands should be a number");
        }
    }

    #[test]
    fn test_is_system_ready() {
        let registry = Arc::new(CommandRegistry::new());
        let initializer = Arc::new(ModuleInitializer::new());
        let diagnostic = DiagnosticTool::new(registry.clone(), initializer.clone());
        
        // Empty system should be ready (no failures)
        assert!(diagnostic.is_system_ready());
        
        // Add a failed command
        let mut registry = CommandRegistry::new();
        registry.mark_command_failed("failed_command", "Test failure".to_string());
        let registry = Arc::new(registry);
        let diagnostic = DiagnosticTool::new(registry, initializer);
        
        // System with failures should not be ready
        assert!(!diagnostic.is_system_ready());
    }

    #[test]
    fn test_analyze_all_commands() {
        let mut registry = CommandRegistry::new();
        let cmd1 = CommandInfo::new("command1".to_string()).mark_registered();
        let cmd2 = CommandInfo::new("command2".to_string()).mark_registered();
        registry.register_command(cmd1).unwrap();
        registry.register_command(cmd2).unwrap();
        
        let registry = Arc::new(registry);
        let initializer = Arc::new(ModuleInitializer::new());
        let diagnostic = DiagnosticTool::new(registry, initializer);
        
        let analysis = diagnostic.analyze_all_commands();
        
        assert_eq!(analysis.len(), 2);
        assert!(analysis.contains_key("command1"));
        assert!(analysis.contains_key("command2"));
        
        for (_, diagnostic_result) in analysis {
            assert_eq!(diagnostic_result.status, "Registered");
        }
    }

    #[test]
    fn test_detect_configuration_issues() {
        let registry = Arc::new(CommandRegistry::new());
        let initializer = Arc::new(ModuleInitializer::new());
        let diagnostic = DiagnosticTool::new(registry, initializer);
        
        let issues = diagnostic.detect_problems();
        
        // Should detect that no commands are registered
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|issue| 
            issue.category == IssueCategory::Configuration && 
            issue.title.contains("No commands registered")
        ));
    }

    #[test]
    fn test_automated_analysis() {
        let registry = Arc::new(CommandRegistry::new());
        let initializer = Arc::new(ModuleInitializer::new());
        let diagnostic = DiagnosticTool::new(registry, initializer);
        
        let analysis = diagnostic.run_automated_analysis();
        
        assert!(!analysis.detected_issues.is_empty());
        assert!(!analysis.intelligent_suggestions.is_empty());
        assert!(analysis.overall_score <= 100);
        
        // Should detect configuration issues for empty registry
        assert!(analysis.detected_issues.iter().any(|issue| 
            issue.category == IssueCategory::Configuration
        ));
    }

    #[test]
    fn test_calculate_system_health_score() {
        let registry = Arc::new(CommandRegistry::new());
        let initializer = Arc::new(ModuleInitializer::new());
        let diagnostic = DiagnosticTool::new(registry, initializer);
        
        // Test with no issues
        let no_issues = vec![];
        let score = diagnostic.calculate_system_health_score(&no_issues);
        assert_eq!(score, 100);
        
        // Test with critical issue
        let critical_issues = vec![
            DiagnosticIssue {
                severity: IssueSeverity::Critical,
                category: IssueCategory::Configuration,
                title: "Critical issue".to_string(),
                description: "Test".to_string(),
                recommendations: vec![],
            }
        ];
        let score = diagnostic.calculate_system_health_score(&critical_issues);
        assert!(score < 100);
        assert!(score >= 75); // Should deduct 25 points
        
        // Test with warning issue
        let warning_issues = vec![
            DiagnosticIssue {
                severity: IssueSeverity::Warning,
                category: IssueCategory::Performance,
                title: "Warning issue".to_string(),
                description: "Test".to_string(),
                recommendations: vec![],
            }
        ];
        let score = diagnostic.calculate_system_health_score(&warning_issues);
        assert!(score < 100);
        assert!(score >= 90); // Should deduct 10 points
    }

    #[test]
    fn test_find_similar_commands() {
        let mut registry = CommandRegistry::new();
        let cmd1 = CommandInfo::new("get_sessions".to_string()).mark_registered();
        let cmd2 = CommandInfo::new("scan_sessions".to_string()).mark_registered();
        let cmd3 = CommandInfo::new("delete_session".to_string()).mark_registered();
        registry.register_command(cmd1).unwrap();
        registry.register_command(cmd2).unwrap();
        registry.register_command(cmd3).unwrap();
        
        let registry = Arc::new(registry);
        let initializer = Arc::new(ModuleInitializer::new());
        let diagnostic = DiagnosticTool::new(registry, initializer);
        
        let similar = diagnostic.find_similar_commands("session");
        assert!(!similar.is_empty());
        // Should find commands containing "session"
        assert!(similar.iter().any(|cmd| cmd.contains("session")));
    }

    #[test]
    fn test_extract_command_name_from_error() {
        let registry = Arc::new(CommandRegistry::new());
        let initializer = Arc::new(ModuleInitializer::new());
        let diagnostic = DiagnosticTool::new(registry, initializer);
        
        let error = CommandError::new(
            "Command 'test_command' not found".to_string(),
            ErrorType::CommandNotFound,
        );
        
        let extracted = diagnostic.extract_command_name_from_error(&error);
        assert_eq!(extracted, Some("test_command".to_string()));
        
        // Test with error that doesn't contain command name
        let error2 = CommandError::new(
            "General error message".to_string(),
            ErrorType::RuntimeError,
        );
        
        let extracted2 = diagnostic.extract_command_name_from_error(&error2);
        assert_eq!(extracted2, None);
    }
}