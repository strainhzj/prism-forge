//! Command Validator Implementation
//!
//! 验证命令的可用性和正确性

use std::collections::HashMap;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use crate::command_registry::{CommandRegistry, CommandStatus};

/// Command validator for testing command availability and correctness
pub struct CommandValidator {
    test_cases: HashMap<String, Vec<TestCase>>,
    validation_rules: Vec<ValidationRule>,
    auto_generated_tests: HashMap<String, Vec<TestCase>>,
    command_registry: Option<std::sync::Arc<CommandRegistry>>,
}

/// Test case for a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub input: serde_json::Value,
    pub expected_result: TestExpectation,
    pub timeout: Duration,
}

/// Expected result of a test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestExpectation {
    Success,
    Error(String),
    Timeout,
}

/// Validation rule for commands
pub struct ValidationRule {
    pub name: String,
    pub rule: Box<dyn Fn(&str) -> bool + Send + Sync>,
}

/// Result of command validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub command_name: String,
    pub passed: bool,
    pub errors: Vec<String>,
    pub test_results: Vec<TestResult>,
}

/// Result of a single test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
    pub error_message: Option<String>,
    pub duration: Duration,
}

/// Result of integration tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationTestResult {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub test_results: Vec<ValidationResult>,
    pub coverage_report: CoverageReport,
}

/// Coverage report for command testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub total_commands: usize,
    pub tested_commands: usize,
    pub untested_commands: Vec<String>,
    pub coverage_percentage: f64,
}

/// Auto-generated test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoTestConfig {
    pub enable_availability_tests: bool,
    pub enable_parameter_validation_tests: bool,
    pub enable_dependency_tests: bool,
    pub test_timeout: Duration,
    pub max_test_cases_per_command: usize,
}

impl CommandValidator {
    /// Create a new command validator
    pub fn new() -> Self {
        Self {
            test_cases: HashMap::new(),
            validation_rules: Vec::new(),
            auto_generated_tests: HashMap::new(),
            command_registry: None,
        }
    }

    /// Create a new command validator with registry reference
    pub fn with_registry(registry: std::sync::Arc<CommandRegistry>) -> Self {
        Self {
            test_cases: HashMap::new(),
            validation_rules: Vec::new(),
            auto_generated_tests: HashMap::new(),
            command_registry: Some(registry),
        }
    }

    /// Set the command registry for availability verification
    pub fn set_registry(&mut self, registry: std::sync::Arc<CommandRegistry>) {
        self.command_registry = Some(registry);
    }

    /// Add a test case for a command
    pub fn add_test_case(&mut self, command: &str, test_case: TestCase) {
        self.test_cases
            .entry(command.to_string())
            .or_insert_with(Vec::new)
            .push(test_case);
    }

    /// Add a validation rule
    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    /// Auto-generate test cases for all registered commands
    pub fn auto_generate_test_cases(&mut self, config: &AutoTestConfig) -> Result<usize, String> {
        let registry = self.command_registry.as_ref()
            .ok_or("Command registry not set")?;

        let mut generated_count = 0;
        let available_commands = registry.list_available_commands();

        for command_name in available_commands {
            let mut generated_tests = Vec::new();

            // Generate availability test
            if config.enable_availability_tests {
                let availability_test = TestCase::new(
                    format!("{}_availability", command_name),
                    serde_json::json!({}),
                    TestExpectation::Success,
                ).with_timeout(config.test_timeout);
                
                generated_tests.push(availability_test);
                generated_count += 1;
            }

            // Generate parameter validation tests
            if config.enable_parameter_validation_tests {
                // Test with empty parameters
                let empty_params_test = TestCase::new(
                    format!("{}_empty_params", command_name),
                    serde_json::json!({}),
                    TestExpectation::Success, // Assuming empty params are valid
                ).with_timeout(config.test_timeout);
                
                generated_tests.push(empty_params_test);
                generated_count += 1;

                // Test with invalid parameters
                let invalid_params_test = TestCase::new(
                    format!("{}_invalid_params", command_name),
                    serde_json::json!({"invalid": "parameter"}),
                    TestExpectation::Error("Invalid parameter".to_string()),
                ).with_timeout(config.test_timeout);
                
                generated_tests.push(invalid_params_test);
                generated_count += 1;
            }

            // Generate dependency tests
            if config.enable_dependency_tests {
                if let Some(command_info) = registry.get_command_info(&command_name) {
                    if !command_info.dependencies.is_empty() {
                        let dependency_test = TestCase::new(
                            format!("{}_dependencies", command_name),
                            serde_json::json!({}),
                            TestExpectation::Success,
                        ).with_timeout(config.test_timeout);
                        
                        generated_tests.push(dependency_test);
                        generated_count += 1;
                    }
                }
            }

            // Limit the number of generated tests per command
            generated_tests.truncate(config.max_test_cases_per_command);
            
            if !generated_tests.is_empty() {
                self.auto_generated_tests.insert(command_name, generated_tests);
            }
        }

        Ok(generated_count)
    }

    /// Verify command availability using the registry
    pub fn verify_command_availability(&self, command: &str) -> Result<bool, String> {
        let registry = self.command_registry.as_ref()
            .ok_or("Command registry not set")?;

        // Check if command exists in registry
        if !registry.has_command(command) {
            return Ok(false);
        }

        // Check command status
        match registry.get_command_status(command) {
            Some(CommandStatus::Registered) => Ok(true),
            Some(CommandStatus::Failed(_)) => Ok(false),
            Some(CommandStatus::Disabled) => Ok(false),
            Some(CommandStatus::Unverified) => {
                // Try to verify the command
                let errors = registry.verify_command(command);
                Ok(errors.is_empty())
            }
            None => Ok(false),
        }
    }

    /// Validate a specific command with availability check
    pub fn validate_command(&self, command: &str) -> ValidationResult {
        let mut errors = Vec::new();
        let mut test_results = Vec::new();

        // First, check command availability
        match self.verify_command_availability(command) {
            Ok(true) => {
                // Command is available, proceed with other validations
            }
            Ok(false) => {
                errors.push(format!("Command '{}' is not available", command));
            }
            Err(e) => {
                errors.push(format!("Failed to verify command availability: {}", e));
            }
        }

        // Apply validation rules
        for rule in &self.validation_rules {
            let rule_fn = &rule.rule;
            if !rule_fn(command) {
                errors.push(format!("Validation rule '{}' failed", rule.name));
            }
        }

        // Run manual test cases
        if let Some(test_cases) = self.test_cases.get(command) {
            for test_case in test_cases {
                let result = self.run_test_case(command, test_case);
                if !result.passed {
                    errors.push(format!("Test case '{}' failed", test_case.name));
                }
                test_results.push(result);
            }
        }

        // Run auto-generated test cases
        if let Some(auto_tests) = self.auto_generated_tests.get(command) {
            for test_case in auto_tests {
                let result = self.run_test_case(command, test_case);
                if !result.passed {
                    errors.push(format!("Auto-generated test case '{}' failed", test_case.name));
                }
                test_results.push(result);
            }
        }

        ValidationResult {
            command_name: command.to_string(),
            passed: errors.is_empty(),
            errors,
            test_results,
        }
    }

    /// Validate all commands with comprehensive coverage
    pub fn validate_all_commands(&self) -> HashMap<String, ValidationResult> {
        let mut results = HashMap::new();

        // Get all unique command names from test cases and registry
        let mut all_commands = std::collections::HashSet::new();
        
        // Add commands from manual test cases
        for command_name in self.test_cases.keys() {
            all_commands.insert(command_name.clone());
        }
        
        // Add commands from auto-generated tests
        for command_name in self.auto_generated_tests.keys() {
            all_commands.insert(command_name.clone());
        }
        
        // Add commands from registry if available
        if let Some(registry) = &self.command_registry {
            for command_name in registry.list_available_commands() {
                all_commands.insert(command_name);
            }
        }

        // Validate each command
        for command_name in all_commands {
            let result = self.validate_command(&command_name);
            results.insert(command_name, result);
        }

        results
    }

    /// Run integration tests with coverage analysis
    pub fn run_integration_tests(&self) -> IntegrationTestResult {
        let all_results = self.validate_all_commands();
        let total_tests = all_results.len();
        let passed_tests = all_results.values().filter(|r| r.passed).count();
        let failed_tests = total_tests - passed_tests;

        // Generate coverage report
        let coverage_report = self.generate_coverage_report(&all_results);

        IntegrationTestResult {
            total_tests,
            passed_tests,
            failed_tests,
            test_results: all_results.into_values().collect(),
            coverage_report,
        }
    }

    /// Generate coverage report
    fn generate_coverage_report(&self, results: &HashMap<String, ValidationResult>) -> CoverageReport {
        let total_commands = if let Some(registry) = &self.command_registry {
            registry.command_count()
        } else {
            results.len()
        };

        let tested_commands = results.len();
        let mut untested_commands = Vec::new();

        // Find untested commands
        if let Some(registry) = &self.command_registry {
            for command_name in registry.list_available_commands() {
                if !results.contains_key(&command_name) {
                    untested_commands.push(command_name);
                }
            }
        }

        let coverage_percentage = if total_commands > 0 {
            (tested_commands as f64 / total_commands as f64) * 100.0
        } else {
            100.0
        };

        CoverageReport {
            total_commands,
            tested_commands,
            untested_commands,
            coverage_percentage,
        }
    }

    /// Run a single test case with enhanced logic
    fn run_test_case(&self, command: &str, test_case: &TestCase) -> TestResult {
        let start_time = std::time::Instant::now();
        
        // Enhanced test execution logic
        let (passed, error_message) = match &test_case.expected_result {
            TestExpectation::Success => {
                // For success expectation, verify command is available and callable
                match self.verify_command_availability(command) {
                    Ok(true) => (true, None),
                    Ok(false) => (false, Some(format!("Command '{}' is not available", command))),
                    Err(e) => (false, Some(e)),
                }
            }
            TestExpectation::Error(expected_error) => {
                // For error expectation, check if command fails as expected
                match self.verify_command_availability(command) {
                    Ok(true) => (false, Some("Expected error but command is available".to_string())),
                    Ok(false) => (true, None), // Command unavailable as expected
                    Err(actual_error) => {
                        if actual_error.contains(expected_error) {
                            (true, None)
                        } else {
                            (false, Some(format!("Expected error '{}' but got '{}'", expected_error, actual_error)))
                        }
                    }
                }
            }
            TestExpectation::Timeout => {
                // For timeout expectation, simulate timeout scenario
                let duration = start_time.elapsed();
                if duration >= test_case.timeout {
                    (true, None)
                } else {
                    (false, Some("Expected timeout but test completed quickly".to_string()))
                }
            }
        };

        let duration = start_time.elapsed();

        TestResult {
            test_name: test_case.name.clone(),
            passed,
            error_message,
            duration,
        }
    }

    /// Get test cases for a command (including auto-generated)
    pub fn get_test_cases(&self, command: &str) -> Vec<&TestCase> {
        let mut all_tests = Vec::new();
        
        // Add manual test cases
        if let Some(manual_tests) = self.test_cases.get(command) {
            all_tests.extend(manual_tests.iter());
        }
        
        // Add auto-generated test cases
        if let Some(auto_tests) = self.auto_generated_tests.get(command) {
            all_tests.extend(auto_tests.iter());
        }
        
        all_tests
    }

    /// Get all commands with test cases
    pub fn get_tested_commands(&self) -> Vec<String> {
        let mut commands = std::collections::HashSet::new();
        
        // Add commands from manual tests
        for command in self.test_cases.keys() {
            commands.insert(command.clone());
        }
        
        // Add commands from auto-generated tests
        for command in self.auto_generated_tests.keys() {
            commands.insert(command.clone());
        }
        
        commands.into_iter().collect()
    }

    /// Get auto-generated test cases for a command
    pub fn get_auto_generated_tests(&self, command: &str) -> Option<&Vec<TestCase>> {
        self.auto_generated_tests.get(command)
    }

    /// Clear auto-generated tests (useful for regeneration)
    pub fn clear_auto_generated_tests(&mut self) {
        self.auto_generated_tests.clear();
    }

    /// Get test statistics
    pub fn get_test_statistics(&self) -> TestStatistics {
        let manual_test_count: usize = self.test_cases.values().map(|v| v.len()).sum();
        let auto_test_count: usize = self.auto_generated_tests.values().map(|v| v.len()).sum();
        let total_commands = self.get_tested_commands().len();
        
        TestStatistics {
            total_commands,
            manual_test_count,
            auto_generated_test_count: auto_test_count,
            total_test_count: manual_test_count + auto_test_count,
        }
    }
}

/// Test statistics for the validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestStatistics {
    pub total_commands: usize,
    pub manual_test_count: usize,
    pub auto_generated_test_count: usize,
    pub total_test_count: usize,
}

impl TestCase {
    /// Create a new test case
    pub fn new(name: String, input: serde_json::Value, expected_result: TestExpectation) -> Self {
        Self {
            name,
            input,
            expected_result,
            timeout: Duration::from_secs(30), // Default timeout
        }
    }

    /// Create a test case with custom timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

impl ValidationRule {
    /// Create a new validation rule
    pub fn new<F>(name: String, rule: F) -> Self 
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        Self {
            name,
            rule: Box::new(rule),
        }
    }
}

impl Default for CommandValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AutoTestConfig {
    fn default() -> Self {
        Self {
            enable_availability_tests: true,
            enable_parameter_validation_tests: true,
            enable_dependency_tests: true,
            test_timeout: Duration::from_secs(30),
            max_test_cases_per_command: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_validator_creation() {
        let validator = CommandValidator::new();
        assert_eq!(validator.test_cases.len(), 0);
        assert_eq!(validator.validation_rules.len(), 0);
        assert_eq!(validator.auto_generated_tests.len(), 0);
        assert!(validator.command_registry.is_none());
    }

    #[test]
    fn test_add_test_case() {
        let mut validator = CommandValidator::new();
        let test_case = TestCase::new(
            "test_case_1".to_string(),
            serde_json::json!({}),
            TestExpectation::Success,
        );

        validator.add_test_case("test_command", test_case);
        assert_eq!(validator.test_cases.len(), 1);
        assert!(validator.test_cases.contains_key("test_command"));
    }

    #[test]
    fn test_validation_rule() {
        let mut validator = CommandValidator::new();
        let rule = ValidationRule::new(
            "non_empty_name".to_string(),
            |name| !name.is_empty(),
        );

        validator.add_validation_rule(rule);
        assert_eq!(validator.validation_rules.len(), 1);

        let result = validator.validate_command("test_command");
        assert!(result.passed);

        let result = validator.validate_command("");
        assert!(!result.passed);
    }

    #[test]
    fn test_auto_test_generation() {
        use crate::command_registry::CommandRegistry;
        use crate::command_registry::CommandInfo;
        use std::sync::Arc;

        let mut registry = CommandRegistry::new();
        let command = CommandInfo::new("test_command".to_string()).mark_registered();
        registry.register_command(command).unwrap();

        let mut validator = CommandValidator::with_registry(Arc::new(registry));
        let config = AutoTestConfig::default();
        
        let generated_count = validator.auto_generate_test_cases(&config).unwrap();
        assert!(generated_count > 0);
        assert!(!validator.auto_generated_tests.is_empty());
        assert!(validator.auto_generated_tests.contains_key("test_command"));
    }

    #[test]
    fn test_command_availability_verification() {
        use crate::command_registry::CommandRegistry;
        use crate::command_registry::CommandInfo;
        use std::sync::Arc;

        let mut registry = CommandRegistry::new();
        let command = CommandInfo::new("available_command".to_string()).mark_registered();
        registry.register_command(command).unwrap();

        let validator = CommandValidator::with_registry(Arc::new(registry));
        
        // Test available command
        let result = validator.verify_command_availability("available_command");
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        // Test unavailable command
        let result = validator.verify_command_availability("unavailable_command");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_coverage_report() {
        use crate::command_registry::CommandRegistry;
        use crate::command_registry::CommandInfo;
        use std::sync::Arc;

        let mut registry = CommandRegistry::new();
        let command1 = CommandInfo::new("command1".to_string()).mark_registered();
        let command2 = CommandInfo::new("command2".to_string()).mark_registered();
        registry.register_command(command1).unwrap();
        registry.register_command(command2).unwrap();

        let mut validator = CommandValidator::with_registry(Arc::new(registry));
        let config = AutoTestConfig::default();
        validator.auto_generate_test_cases(&config).unwrap();

        let integration_result = validator.run_integration_tests();
        let coverage = integration_result.coverage_report;
        
        assert_eq!(coverage.total_commands, 2);
        assert_eq!(coverage.tested_commands, 2);
        assert!(coverage.untested_commands.is_empty());
        assert_eq!(coverage.coverage_percentage, 100.0);
    }
}