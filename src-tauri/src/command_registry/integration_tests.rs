//! Integration Tests for Command Registration System
//!
//! 前端到后端的完整调用链测试和自动化回归测试
//! **Validates: Requirements 5.3**

#[cfg(test)]
mod tests {
    use crate::command_registry::{
        CommandRegistry, CommandInfo, CommandStatus, CommandValidator, 
        ModuleInitializer, Module, InitState, DiagnosticTool,
        EnhancedErrorHandler, ErrorCategory,
    };
    use crate::command_registry::errors::{CommandError, ModuleError, ModuleErrorType};
    use std::sync::Arc;
    use std::time::Duration;

    /// Integration test: Complete command registration and validation flow
    /// Tests the full lifecycle from command registration to validation
    #[test]
    fn test_complete_command_registration_flow() {
        // Step 1: Create and configure the command registry
        let mut registry = CommandRegistry::new();
        
        // Step 2: Register multiple commands with dependencies
        let commands = vec![
            ("cmd_get_providers", vec![]),
            ("cmd_save_provider", vec!["cmd_get_providers"]),
            ("cmd_delete_provider", vec!["cmd_get_providers"]),
            ("scan_sessions", vec![]),
            ("parse_session_tree", vec!["scan_sessions"]),
        ];
        
        for (name, deps) in commands {
            let deps_vec: Vec<String> = deps.iter().map(|s: &&str| s.to_string()).collect();
            let command = CommandInfo::with_dependencies(name.to_string(), deps_vec)
                .mark_registered();
            let result = registry.register_command(command);
            assert!(result.is_ok(), "Failed to register command: {}", name);
        }
        
        // Step 3: Verify all commands are registered
        assert_eq!(registry.command_count(), 5);
        assert!(registry.has_command("cmd_get_providers"));
        assert!(registry.has_command("scan_sessions"));
        
        // Step 4: Verify command statuses
        for cmd in ["cmd_get_providers", "cmd_save_provider", "cmd_delete_provider", "scan_sessions", "parse_session_tree"] {
            let status = registry.get_command_status(cmd);
            assert!(matches!(status, Some(CommandStatus::Registered)), 
                "Command {} should be registered", cmd);
        }
        
        // Step 5: Verify dependency chain
        let errors = registry.verify_all_commands();
        assert!(errors.is_empty(), "Should have no verification errors: {:?}", errors);
        
        // Step 6: Test command availability list
        let available = registry.list_available_commands();
        assert_eq!(available.len(), 5);
        assert!(available.contains(&"cmd_get_providers".to_string()));
    }

    /// Integration test: Module initialization with command registration
    /// Tests that modules initialize correctly before commands are registered
    #[test]
    fn test_module_initialization_with_command_registration() {
        use std::sync::Mutex;
        
        // Create mock modules
        struct MockDatabaseModule {
            initialized: Arc<Mutex<bool>>,
        }
        
        impl Module for MockDatabaseModule {
            fn name(&self) -> &str { "database" }
            fn dependencies(&self) -> Vec<String> { vec![] }
            fn initialize(&mut self) -> Result<(), ModuleError> {
                *self.initialized.lock().unwrap() = true;
                Ok(())
            }
            fn health_check(&self) -> Result<(), ModuleError> {
                if *self.initialized.lock().unwrap() {
                    Ok(())
                } else {
                    Err(ModuleError::new("database".to_string(), "Not initialized".to_string(), ModuleErrorType::HealthCheckFailed))
                }
            }
            fn shutdown(&mut self) -> Result<(), ModuleError> { Ok(()) }
        }
        
        struct MockMonitorModule {
            initialized: Arc<Mutex<bool>>,
        }
        
        impl Module for MockMonitorModule {
            fn name(&self) -> &str { "monitor" }
            fn dependencies(&self) -> Vec<String> { vec!["database".to_string()] }
            fn initialize(&mut self) -> Result<(), ModuleError> {
                *self.initialized.lock().unwrap() = true;
                Ok(())
            }
            fn health_check(&self) -> Result<(), ModuleError> {
                if *self.initialized.lock().unwrap() {
                    Ok(())
                } else {
                    Err(ModuleError::new("monitor".to_string(), "Not initialized".to_string(), ModuleErrorType::HealthCheckFailed))
                }
            }
            fn shutdown(&mut self) -> Result<(), ModuleError> { Ok(()) }
        }
        
        // Initialize modules
        let db_initialized = Arc::new(Mutex::new(false));
        let monitor_initialized = Arc::new(Mutex::new(false));
        
        let mut initializer = ModuleInitializer::new();
        initializer.register_module(Box::new(MockDatabaseModule { 
            initialized: db_initialized.clone() 
        })).unwrap();
        initializer.register_module(Box::new(MockMonitorModule { 
            initialized: monitor_initialized.clone() 
        })).unwrap();
        
        // Get initialization order
        let order = initializer.get_initialization_order().unwrap();
        
        // Database should come before monitor
        let db_pos = order.iter().position(|x| x == "database").unwrap();
        let monitor_pos = order.iter().position(|x| x == "monitor").unwrap();
        assert!(db_pos < monitor_pos, "Database should initialize before monitor");
        
        // Initialize all modules
        let result = initializer.initialize_all();
        assert!(result.is_ok(), "Module initialization should succeed");
        
        // Verify both modules are initialized
        assert!(*db_initialized.lock().unwrap(), "Database should be initialized");
        assert!(*monitor_initialized.lock().unwrap(), "Monitor should be initialized");
        
        // Verify module states
        assert!(matches!(initializer.get_module_state("database"), Some(InitState::Ready)));
        assert!(matches!(initializer.get_module_state("monitor"), Some(InitState::Ready)));
    }

    /// Integration test: Error handling flow from command to frontend response
    /// Tests that errors are properly categorized and formatted for frontend
    #[test]
    fn test_error_handling_flow() {
        let handler = EnhancedErrorHandler::new();
        
        // Test command not found error
        let cmd_error = CommandError::command_not_found("unknown_command");
        let response = handler.handle_command_error(&cmd_error);
        
        assert!(!response.message.is_empty());
        assert!(!response.error_code.is_empty());
        assert!(!response.recovery_suggestions.is_empty());
        
        // Verify error categorization on the original error message
        let category = handler.categorize_error(&cmd_error.message);
        assert!(matches!(category, ErrorCategory::CommandNotFound), 
            "Expected CommandNotFound, got {:?}", category);
        
        // Test module initialization error
        let module_error = ModuleError::initialization_failed("database", "Connection failed");
        let response = handler.handle_module_error(&module_error);
        
        assert!(!response.message.is_empty());
        assert!(response.details.is_some());
        
        // Verify error categorization on the original error message
        let category = handler.categorize_error(&module_error.message);
        assert!(matches!(category, ErrorCategory::ModuleInitializationFailed),
            "Expected ModuleInitializationFailed, got {:?}", category);
    }

    /// Integration test: Diagnostic tool with full system state
    /// Tests that diagnostic tool can analyze complete system state
    #[test]
    fn test_diagnostic_tool_integration() {
        // Setup registry with some commands
        let mut registry = CommandRegistry::new();
        
        // Register some valid commands
        let valid_cmd = CommandInfo::new("valid_command".to_string()).mark_registered();
        registry.register_command(valid_cmd).unwrap();
        
        // Register a command with missing dependency
        let cmd_with_dep = CommandInfo::with_dependencies(
            "dependent_command".to_string(),
            vec!["missing_dependency".to_string()]
        ).mark_registered();
        registry.register_command(cmd_with_dep).unwrap();
        
        // Setup module initializer
        let initializer = ModuleInitializer::new();
        
        // Create diagnostic tool
        let diagnostic = DiagnosticTool::new(
            Arc::new(registry),
            Arc::new(initializer)
        );
        
        // Run full diagnostic
        let report = diagnostic.run_full_diagnostic();
        
        // Verify report contains expected information
        assert!(!report.registered_commands.is_empty());
        assert!(report.registered_commands.contains(&"valid_command".to_string()));
        assert!(report.registered_commands.contains(&"dependent_command".to_string()));
        
        // Should have recommendations for missing dependency
        assert!(!report.recommendations.is_empty() || !report.failed_commands.is_empty());
    }

    /// Integration test: Command validator with auto-generated tests
    /// Tests the complete validation flow with auto-generated test cases
    #[test]
    fn test_command_validator_integration() {
        use crate::command_registry::validator::AutoTestConfig;
        
        // Setup registry
        let mut registry = CommandRegistry::new();
        let commands = vec![
            "cmd_get_providers",
            "cmd_save_provider", 
            "scan_sessions",
            "parse_session_tree",
        ];
        
        for name in &commands {
            let cmd = CommandInfo::new(name.to_string()).mark_registered();
            registry.register_command(cmd).unwrap();
        }
        
        // Create validator with registry
        let mut validator = CommandValidator::with_registry(Arc::new(registry));
        
        // Auto-generate test cases
        let config = AutoTestConfig {
            enable_availability_tests: true,
            enable_parameter_validation_tests: true,
            enable_dependency_tests: true,
            test_timeout: Duration::from_secs(5),
            max_test_cases_per_command: 5,
        };
        
        let generated_count = validator.auto_generate_test_cases(&config).unwrap();
        assert!(generated_count > 0, "Should generate test cases");
        
        // Run integration tests
        let result = validator.run_integration_tests();
        
        // Verify test results
        assert!(result.total_tests > 0);
        assert_eq!(result.passed_tests + result.failed_tests, result.total_tests);
        
        // Verify coverage
        assert!(result.coverage_report.coverage_percentage > 0.0);
        assert_eq!(result.coverage_report.total_commands, commands.len());
    }

    /// Integration test: End-to-end command call simulation
    /// Simulates the complete flow from frontend call to backend response
    #[test]
    fn test_end_to_end_command_call_simulation() {
        // Setup complete system
        let mut registry = CommandRegistry::new();
        let _initializer = ModuleInitializer::new();
        
        // Register commands that would be called from frontend
        let frontend_commands = vec![
            ("cmd_get_providers", vec![]),
            ("cmd_save_provider", vec![]),
            ("cmd_test_provider_connection", vec![]),
            ("scan_sessions", vec![]),
            ("scan_directory", vec![]),
            ("parse_session_tree", vec![]),
            ("set_session_rating", vec![]),
            ("get_monitored_directories", vec![]),
        ];
        
        for (name, deps) in frontend_commands {
            let cmd = CommandInfo::with_dependencies(
                name.to_string(),
                deps.into_iter().map(|s: &str| s.to_string()).collect()
            ).mark_registered();
            registry.register_command(cmd).unwrap();
        }
        
        // Simulate frontend call flow
        let test_calls = vec![
            "cmd_get_providers",
            "scan_sessions",
            "get_monitored_directories",
        ];
        
        for call in test_calls {
            // Step 1: Check if command exists
            assert!(registry.has_command(call), "Command {} should exist", call);
            
            // Step 2: Check command status
            let status = registry.get_command_status(call);
            assert!(matches!(status, Some(CommandStatus::Registered)), 
                "Command {} should be registered", call);
            
            // Step 3: Record the call
            registry.record_command_call(call);
            
            // Step 4: Verify call was recorded
            let detailed_status = registry.get_command_status_detailed(call).unwrap();
            assert!(detailed_status.call_count > 0, "Call count should be incremented");
            assert!(detailed_status.last_called.is_some(), "Last called should be set");
        }
        
        // Verify command history
        for call in ["cmd_get_providers", "scan_sessions", "get_monitored_directories"] {
            let history = registry.get_command_history(call).unwrap();
            assert!(!history.is_empty(), "Command {} should have history", call);
        }
    }

    /// Integration test: Regression test for command registration
    /// Ensures previously working commands continue to work after changes
    #[test]
    fn test_regression_command_registration() {
        let mut registry = CommandRegistry::new();
        
        // Register all known Tauri commands from lib.rs
        let known_commands = vec![
            "greet",
            "cmd_get_providers",
            "cmd_save_provider",
            "cmd_delete_provider",
            "cmd_set_active_provider",
            "cmd_test_provider_connection",
            "count_prompt_tokens",
            "scan_sessions",
            "scan_directory",
            "run_benchmarks",
            "parse_session_tree",
            "set_session_rating",
            "set_session_tags",
            "get_session_rating",
            "get_session_tags",
            "archive_session",
            "unarchive_session",
            "get_archived_sessions",
            "start_file_watcher",
            "extract_session_log",
            "export_session_log",
            "vector_search",
            "compress_context",
            "optimize_prompt",
            "get_meta_template",
            "update_meta_template",
            "get_monitored_directories",
            "add_monitored_directory",
            "remove_monitored_directory",
            "toggle_monitored_directory",
            "update_monitored_directory",
        ];
        
        // Register all commands
        for name in &known_commands {
            let cmd = CommandInfo::new(name.to_string()).mark_registered();
            let result = registry.register_command(cmd);
            assert!(result.is_ok(), "Failed to register known command: {}", name);
        }
        
        // Verify all commands are registered
        assert_eq!(registry.command_count(), known_commands.len());
        
        // Verify each command is accessible
        for name in &known_commands {
            assert!(registry.has_command(name), "Command {} should be registered", name);
            assert!(matches!(registry.get_command_status(name), Some(CommandStatus::Registered)),
                "Command {} should have Registered status", name);
        }
        
        // Verify available commands list
        let available = registry.list_available_commands();
        assert_eq!(available.len(), known_commands.len());
        
        for name in &known_commands {
            assert!(available.contains(&name.to_string()), 
                "Command {} should be in available list", name);
        }
    }

    /// Integration test: Command dependency chain validation
    /// Tests that dependency chains are properly validated
    #[test]
    fn test_dependency_chain_validation() {
        let mut registry = CommandRegistry::new();
        
        // Create a dependency chain: A -> B -> C
        let cmd_c = CommandInfo::new("command_c".to_string()).mark_registered();
        let cmd_b = CommandInfo::with_dependencies("command_b".to_string(), vec!["command_c".to_string()]).mark_registered();
        let cmd_a = CommandInfo::with_dependencies("command_a".to_string(), vec!["command_b".to_string()]).mark_registered();
        
        // Register in correct order
        registry.register_command(cmd_c).unwrap();
        registry.register_command(cmd_b).unwrap();
        registry.register_command(cmd_a).unwrap();
        
        // Verify no errors
        let errors = registry.verify_all_commands();
        assert!(errors.is_empty(), "Should have no verification errors");
        
        // Now test with missing dependency
        let mut registry2 = CommandRegistry::new();
        let cmd_with_missing = CommandInfo::with_dependencies(
            "orphan_command".to_string(),
            vec!["nonexistent_dependency".to_string()]
        ).mark_registered();
        registry2.register_command(cmd_with_missing).unwrap();
        
        // Should detect missing dependency
        let errors = registry2.verify_all_commands();
        assert!(!errors.is_empty(), "Should detect missing dependency");
        assert!(errors.iter().any(|e| e.message.contains("nonexistent_dependency")),
            "Error should mention missing dependency");
    }

    /// Integration test: Health check integration
    /// Tests that health checks work across the entire system
    #[test]
    fn test_health_check_integration() {
        struct HealthyModule;
        impl Module for HealthyModule {
            fn name(&self) -> &str { "healthy_module" }
            fn dependencies(&self) -> Vec<String> { vec![] }
            fn initialize(&mut self) -> Result<(), ModuleError> { Ok(()) }
            fn health_check(&self) -> Result<(), ModuleError> { Ok(()) }
            fn shutdown(&mut self) -> Result<(), ModuleError> { Ok(()) }
        }
        
        struct UnhealthyModule;
        impl Module for UnhealthyModule {
            fn name(&self) -> &str { "unhealthy_module" }
            fn dependencies(&self) -> Vec<String> { vec![] }
            fn initialize(&mut self) -> Result<(), ModuleError> { Ok(()) }
            fn health_check(&self) -> Result<(), ModuleError> {
                Err(ModuleError::new(
                    "unhealthy_module".to_string(),
                    "Health check failed".to_string(),
                    ModuleErrorType::HealthCheckFailed
                ))
            }
            fn shutdown(&mut self) -> Result<(), ModuleError> { Ok(()) }
        }
        
        let mut initializer = ModuleInitializer::new();
        initializer.register_module(Box::new(HealthyModule)).unwrap();
        initializer.register_module(Box::new(UnhealthyModule)).unwrap();
        
        // Initialize modules
        initializer.initialize_all().unwrap();
        
        // Run health checks
        let health_results = initializer.health_check_all();
        
        // Verify results
        assert!(health_results.get("healthy_module").unwrap().is_ok());
        assert!(health_results.get("unhealthy_module").unwrap().is_err());
        
        // Run comprehensive health check
        let comprehensive = initializer.comprehensive_health_check();
        assert!(comprehensive.module_statuses.contains_key("healthy_module"));
        assert!(comprehensive.module_statuses.contains_key("unhealthy_module"));
    }

    /// Integration test: Alert mechanism for command anomalies
    /// Tests that alerts are triggered for abnormal command states
    #[test]
    fn test_alert_mechanism_integration() {
        let mut registry = CommandRegistry::new();
        
        // Register a command that will fail
        let failed_cmd = CommandInfo::new("failing_command".to_string())
            .with_status(CommandStatus::Failed("Test failure".to_string()));
        registry.register_command(failed_cmd).unwrap();
        
        // Register a disabled command
        let disabled_cmd = CommandInfo::new("disabled_command".to_string())
            .with_status(CommandStatus::Disabled);
        registry.register_command(disabled_cmd).unwrap();
        
        // Register a normal command
        let normal_cmd = CommandInfo::new("normal_command".to_string()).mark_registered();
        registry.register_command(normal_cmd).unwrap();
        
        // Check for anomalies
        let anomalies = registry.get_anomalous_commands();
        
        // Should detect failed and disabled commands as anomalies
        assert!(anomalies.contains(&"failing_command".to_string()));
        assert!(anomalies.contains(&"disabled_command".to_string()));
        assert!(!anomalies.contains(&"normal_command".to_string()));
    }

    /// Integration test: Full system startup simulation
    /// Simulates the complete application startup flow
    #[test]
    fn test_full_system_startup_simulation() {
        use std::sync::Mutex;
        
        // Track initialization order
        let init_order = Arc::new(Mutex::new(Vec::new()));
        
        struct TrackedModule {
            name: String,
            deps: Vec<String>,
            init_order: Arc<Mutex<Vec<String>>>,
        }
        
        impl Module for TrackedModule {
            fn name(&self) -> &str { &self.name }
            fn dependencies(&self) -> Vec<String> { self.deps.clone() }
            fn initialize(&mut self) -> Result<(), ModuleError> {
                self.init_order.lock().unwrap().push(self.name.clone());
                Ok(())
            }
            fn health_check(&self) -> Result<(), ModuleError> { Ok(()) }
            fn shutdown(&mut self) -> Result<(), ModuleError> { Ok(()) }
        }
        
        // Create module initializer
        let mut initializer = ModuleInitializer::new();
        
        // Register modules in dependency order
        let modules = vec![
            ("database", vec![]),
            ("monitor", vec!["database"]),
            ("llm", vec!["database"]),
            ("parser", vec!["database"]),
            ("optimizer", vec!["llm", "parser"]),
        ];
        
        for (name, deps) in modules {
            let deps_vec: Vec<String> = deps.iter().map(|s: &&str| s.to_string()).collect();
            let module = TrackedModule {
                name: name.to_string(),
                deps: deps_vec,
                init_order: init_order.clone(),
            };
            initializer.register_module(Box::new(module)).unwrap();
        }
        
        // Initialize all modules
        let result = initializer.initialize_all();
        assert!(result.is_ok(), "System startup should succeed");
        
        // Verify initialization order respects dependencies
        let order = init_order.lock().unwrap();
        let db_pos = order.iter().position(|x| x == "database").unwrap();
        let monitor_pos = order.iter().position(|x| x == "monitor").unwrap();
        let llm_pos = order.iter().position(|x| x == "llm").unwrap();
        let parser_pos = order.iter().position(|x| x == "parser").unwrap();
        let optimizer_pos = order.iter().position(|x| x == "optimizer").unwrap();
        
        assert!(db_pos < monitor_pos, "database should init before monitor");
        assert!(db_pos < llm_pos, "database should init before llm");
        assert!(db_pos < parser_pos, "database should init before parser");
        assert!(llm_pos < optimizer_pos, "llm should init before optimizer");
        assert!(parser_pos < optimizer_pos, "parser should init before optimizer");
        
        // Create command registry
        let mut registry = CommandRegistry::new();
        
        // Register commands after modules are initialized
        let commands = vec![
            ("cmd_get_providers", vec!["database"]),
            ("scan_sessions", vec!["database", "monitor"]),
            ("parse_session_tree", vec!["parser"]),
            ("optimize_prompt", vec!["optimizer"]),
        ];
        
        for (name, deps) in commands {
            let deps_vec: Vec<String> = deps.iter().map(|s: &&str| s.to_string()).collect();
            let cmd = CommandInfo::with_dependencies(
                name.to_string(),
                deps_vec
            ).mark_registered();
            registry.register_command(cmd).unwrap();
        }
        
        // Verify all commands are registered
        assert_eq!(registry.command_count(), 4);
        
        // Verify all modules are ready
        for module_name in ["database", "monitor", "llm", "parser", "optimizer"] {
            assert!(matches!(initializer.get_module_state(module_name), Some(InitState::Ready)),
                "Module {} should be ready", module_name);
        }
    }
}

/// Unit tests for integration test functionality
/// Tests end-to-end call chain correctness and regression test coverage
/// **Validates: Requirements 5.3**
#[cfg(test)]
mod integration_unit_tests {
    use crate::command_registry::{
        CommandRegistry, CommandInfo, CommandStatus, CommandValidator,
        ModuleInitializer, Module, InitState,
        EnhancedErrorHandler,
    };
    use crate::command_registry::errors::{CommandError, ModuleError, ModuleErrorType};
    use crate::command_registry::validator::AutoTestConfig;
    use std::sync::Arc;
    use std::time::Duration;

    // ============================================================================
    // Unit Tests for End-to-End Call Chain Correctness
    // ============================================================================

    /// Test that the call chain correctly tracks command invocations
    #[test]
    fn test_call_chain_tracking_correctness() {
        let mut registry = CommandRegistry::new();
        
        // Register a command
        let cmd = CommandInfo::new("test_command".to_string()).mark_registered();
        registry.register_command(cmd).unwrap();
        
        // Verify initial state
        let initial_status = registry.get_command_status_detailed("test_command").unwrap();
        assert_eq!(initial_status.call_count, 0, "Initial call count should be 0");
        assert!(initial_status.last_called.is_none(), "Initial last_called should be None");
        
        // Record multiple calls
        for i in 1..=5 {
            registry.record_command_call("test_command");
            
            let status = registry.get_command_status_detailed("test_command").unwrap();
            assert_eq!(status.call_count, i, "Call count should be {}", i);
            assert!(status.last_called.is_some(), "last_called should be set after call");
        }
        
        // Verify history contains all calls
        let history = registry.get_command_history("test_command").unwrap();
        // Should have 1 registration + 5 calls = 6 entries
        assert!(history.len() >= 6, "History should contain registration and all calls");
    }

    /// Test that call chain preserves order of operations
    #[test]
    fn test_call_chain_order_preservation() {
        let mut registry = CommandRegistry::new();
        
        // Register multiple commands
        let commands = vec!["cmd_a", "cmd_b", "cmd_c"];
        for name in &commands {
            let cmd = CommandInfo::new(name.to_string()).mark_registered();
            registry.register_command(cmd).unwrap();
        }
        
        // Call commands in specific order
        let call_order = vec!["cmd_a", "cmd_b", "cmd_a", "cmd_c", "cmd_b"];
        for cmd in &call_order {
            registry.record_command_call(cmd);
        }
        
        // Verify each command's call count
        let status_a = registry.get_command_status_detailed("cmd_a").unwrap();
        let status_b = registry.get_command_status_detailed("cmd_b").unwrap();
        let status_c = registry.get_command_status_detailed("cmd_c").unwrap();
        
        assert_eq!(status_a.call_count, 2, "cmd_a should have 2 calls");
        assert_eq!(status_b.call_count, 2, "cmd_b should have 2 calls");
        assert_eq!(status_c.call_count, 1, "cmd_c should have 1 call");
    }

    /// Test that call chain handles non-existent commands gracefully
    #[test]
    fn test_call_chain_nonexistent_command() {
        let mut registry = CommandRegistry::new();
        
        // Register one command
        let cmd = CommandInfo::new("existing_cmd".to_string()).mark_registered();
        registry.register_command(cmd).unwrap();
        
        // Try to record call for non-existent command (should not panic)
        registry.record_command_call("nonexistent_cmd");
        
        // Verify existing command is unaffected
        let status = registry.get_command_status_detailed("existing_cmd").unwrap();
        assert_eq!(status.call_count, 0, "Existing command should be unaffected");
        
        // Verify non-existent command has no status
        assert!(registry.get_command_status_detailed("nonexistent_cmd").is_none());
    }

    /// Test that call chain correctly handles concurrent-like access patterns
    #[test]
    fn test_call_chain_rapid_calls() {
        let mut registry = CommandRegistry::new();
        
        let cmd = CommandInfo::new("rapid_cmd".to_string()).mark_registered();
        registry.register_command(cmd).unwrap();
        
        // Simulate rapid calls
        let call_count = 100;
        for _ in 0..call_count {
            registry.record_command_call("rapid_cmd");
        }
        
        let status = registry.get_command_status_detailed("rapid_cmd").unwrap();
        assert_eq!(status.call_count, call_count, "All rapid calls should be counted");
    }

    // ============================================================================
    // Unit Tests for Regression Test Coverage
    // ============================================================================

    /// Test that regression tests cover all known Tauri commands
    #[test]
    fn test_regression_coverage_all_known_commands() {
        let known_commands = get_known_tauri_commands();
        let mut registry = CommandRegistry::new();
        
        // Register all known commands
        for name in &known_commands {
            let cmd = CommandInfo::new(name.to_string()).mark_registered();
            registry.register_command(cmd).unwrap();
        }
        
        // Verify all commands are registered
        assert_eq!(registry.command_count(), known_commands.len());
        
        // Verify each command is accessible
        for name in &known_commands {
            assert!(registry.has_command(name), "Command {} should be registered", name);
        }
        
        // Verify available commands list matches
        let available = registry.list_available_commands();
        assert_eq!(available.len(), known_commands.len());
    }

    /// Test that regression tests detect missing commands
    #[test]
    fn test_regression_detects_missing_commands() {
        let mut registry = CommandRegistry::new();
        
        // Register only some commands
        let partial_commands = vec!["cmd_get_providers", "scan_sessions"];
        for name in &partial_commands {
            let cmd = CommandInfo::new(name.to_string()).mark_registered();
            registry.register_command(cmd).unwrap();
        }
        
        // Check for missing commands
        let known_commands = get_known_tauri_commands();
        let available = registry.list_available_commands();
        
        let missing: Vec<_> = known_commands.iter()
            .filter(|cmd| !available.contains(&cmd.to_string()))
            .collect();
        
        assert!(!missing.is_empty(), "Should detect missing commands");
        assert!(missing.len() > 0, "Should have missing commands");
    }

    /// Test that regression tests verify command status consistency
    #[test]
    fn test_regression_status_consistency() {
        let mut registry = CommandRegistry::new();
        
        // Register commands with different statuses
        let registered_cmd = CommandInfo::new("registered_cmd".to_string()).mark_registered();
        let failed_cmd = CommandInfo::new("failed_cmd".to_string())
            .with_status(CommandStatus::Failed("Test failure".to_string()));
        let disabled_cmd = CommandInfo::new("disabled_cmd".to_string())
            .with_status(CommandStatus::Disabled);
        
        registry.register_command(registered_cmd).unwrap();
        registry.register_command(failed_cmd).unwrap();
        registry.register_command(disabled_cmd).unwrap();
        
        // Verify status consistency
        assert!(matches!(
            registry.get_command_status("registered_cmd"),
            Some(CommandStatus::Registered)
        ));
        assert!(matches!(
            registry.get_command_status("failed_cmd"),
            Some(CommandStatus::Failed(_))
        ));
        assert!(matches!(
            registry.get_command_status("disabled_cmd"),
            Some(CommandStatus::Disabled)
        ));
        
        // Verify available commands only includes registered ones
        let available = registry.list_available_commands();
        assert!(available.contains(&"registered_cmd".to_string()));
        assert!(!available.contains(&"failed_cmd".to_string()));
        assert!(!available.contains(&"disabled_cmd".to_string()));
    }

    /// Test that regression tests verify dependency chain integrity
    #[test]
    fn test_regression_dependency_chain_integrity() {
        let mut registry = CommandRegistry::new();
        
        // Create a valid dependency chain
        let cmd_base = CommandInfo::new("base_cmd".to_string()).mark_registered();
        let cmd_dependent = CommandInfo::with_dependencies(
            "dependent_cmd".to_string(),
            vec!["base_cmd".to_string()]
        ).mark_registered();
        
        registry.register_command(cmd_base).unwrap();
        registry.register_command(cmd_dependent).unwrap();
        
        // Verify no errors for valid chain
        let errors = registry.verify_all_commands();
        assert!(errors.is_empty(), "Valid dependency chain should have no errors");
        
        // Now test with broken chain
        let mut registry2 = CommandRegistry::new();
        let orphan_cmd = CommandInfo::with_dependencies(
            "orphan_cmd".to_string(),
            vec!["missing_dep".to_string()]
        ).mark_registered();
        registry2.register_command(orphan_cmd).unwrap();
        
        let errors = registry2.verify_all_commands();
        assert!(!errors.is_empty(), "Broken dependency chain should have errors");
    }

    /// Test that regression tests can detect command registration changes
    #[test]
    fn test_regression_detects_registration_changes() {
        let mut registry = CommandRegistry::new();
        
        // Initial registration
        let cmd = CommandInfo::new("test_cmd".to_string()).mark_registered();
        registry.register_command(cmd).unwrap();
        
        // Capture initial state
        let initial_count = registry.command_count();
        let initial_available = registry.list_available_commands();
        
        // Add new command
        let new_cmd = CommandInfo::new("new_cmd".to_string()).mark_registered();
        registry.register_command(new_cmd).unwrap();
        
        // Verify change is detected
        assert_eq!(registry.command_count(), initial_count + 1);
        let new_available = registry.list_available_commands();
        assert!(new_available.len() > initial_available.len());
        assert!(new_available.contains(&"new_cmd".to_string()));
    }

    // ============================================================================
    // Unit Tests for Integration Test Infrastructure
    // ============================================================================

    /// Test that integration test setup correctly initializes all components
    #[test]
    fn test_integration_setup_completeness() {
        // Create all required components
        let registry = CommandRegistry::new();
        let initializer = ModuleInitializer::new();
        let handler = EnhancedErrorHandler::new();
        
        // Verify components are properly initialized
        assert_eq!(registry.command_count(), 0);
        assert!(initializer.get_initialization_order().is_ok());
        
        // Test error handler is functional
        let test_error = CommandError::command_not_found("test");
        let response = handler.handle_command_error(&test_error);
        assert!(!response.message.is_empty());
    }

    /// Test that integration tests properly validate command availability
    #[test]
    fn test_integration_command_availability_validation() {
        let mut registry = CommandRegistry::new();
        
        // Register commands
        let commands = vec!["cmd_a", "cmd_b", "cmd_c"];
        for name in &commands {
            let cmd = CommandInfo::new(name.to_string()).mark_registered();
            registry.register_command(cmd).unwrap();
        }
        
        // Create validator
        let validator = CommandValidator::with_registry(Arc::new(registry));
        
        // Validate each command
        for name in &commands {
            let result = validator.validate_command(name);
            assert!(result.passed, "Command {} should be valid", name);
        }
        
        // Validate non-existent command
        let result = validator.validate_command("nonexistent");
        assert!(!result.passed, "Non-existent command should be invalid");
    }

    /// Test that integration tests properly handle module initialization failures
    #[test]
    fn test_integration_module_failure_handling() {
        struct FailingModule;
        impl Module for FailingModule {
            fn name(&self) -> &str { "failing_module" }
            fn dependencies(&self) -> Vec<String> { vec![] }
            fn initialize(&mut self) -> Result<(), ModuleError> {
                Err(ModuleError::initialization_failed("failing_module", "Intentional failure"))
            }
            fn health_check(&self) -> Result<(), ModuleError> {
                Err(ModuleError::new(
                    "failing_module".to_string(),
                    "Health check failed".to_string(),
                    ModuleErrorType::HealthCheckFailed
                ))
            }
            fn shutdown(&mut self) -> Result<(), ModuleError> { Ok(()) }
        }
        
        let mut initializer = ModuleInitializer::new();
        initializer.register_module(Box::new(FailingModule)).unwrap();
        
        // Initialization should fail
        let result = initializer.initialize_all();
        assert!(result.is_err(), "Initialization should fail for failing module");
        
        // Module state should reflect failure or recovery attempt
        // Note: The initializer may attempt recovery which changes state to Pending
        let state = initializer.get_module_state("failing_module");
        assert!(
            matches!(state, Some(InitState::Failed(_)) | Some(InitState::Pending)),
            "Module state should be Failed or Pending (after recovery attempt), got {:?}",
            state
        );
        
        // Verify the error contains meaningful information
        let errors = result.unwrap_err();
        assert!(!errors.is_empty(), "Should have at least one error");
        assert!(
            errors.iter().any(|e| e.message.contains("Intentional failure") || e.message.contains("failing_module")),
            "Error should contain failure information"
        );
    }

    /// Test that integration tests properly track test coverage
    #[test]
    fn test_integration_test_coverage_tracking() {
        let mut registry = CommandRegistry::new();
        
        // Register commands
        let commands = vec!["cmd_1", "cmd_2", "cmd_3", "cmd_4"];
        for name in &commands {
            let cmd = CommandInfo::new(name.to_string()).mark_registered();
            registry.register_command(cmd).unwrap();
        }
        
        let mut validator = CommandValidator::with_registry(Arc::new(registry));
        
        // Auto-generate test cases
        let config = AutoTestConfig {
            enable_availability_tests: true,
            enable_parameter_validation_tests: true,
            enable_dependency_tests: true,
            test_timeout: Duration::from_secs(5),
            max_test_cases_per_command: 3,
        };
        
        let generated = validator.auto_generate_test_cases(&config).unwrap();
        assert!(generated > 0, "Should generate test cases");
        
        // Run integration tests
        let result = validator.run_integration_tests();
        
        // Verify coverage tracking
        assert!(result.coverage_report.coverage_percentage > 0.0);
        assert_eq!(result.coverage_report.total_commands, commands.len());
        assert!(result.coverage_report.tested_commands > 0);
    }

    // ============================================================================
    // Helper Functions
    // ============================================================================

    /// Returns the list of known Tauri commands from lib.rs
    fn get_known_tauri_commands() -> Vec<&'static str> {
        vec![
            "greet",
            "cmd_get_providers",
            "cmd_save_provider",
            "cmd_delete_provider",
            "cmd_set_active_provider",
            "cmd_test_provider_connection",
            "count_prompt_tokens",
            "scan_sessions",
            "scan_directory",
            "run_benchmarks",
            "parse_session_tree",
            "set_session_rating",
            "set_session_tags",
            "get_session_rating",
            "get_session_tags",
            "archive_session",
            "unarchive_session",
            "get_archived_sessions",
            "start_file_watcher",
            "extract_session_log",
            "export_session_log",
            "vector_search",
            "compress_context",
            "optimize_prompt",
            "get_meta_template",
            "update_meta_template",
            "get_monitored_directories",
            "add_monitored_directory",
            "remove_monitored_directory",
            "toggle_monitored_directory",
            "update_monitored_directory",
        ]
    }
}
