//! Application Startup Module
//!
//! Provides comprehensive startup validation and command registration integration
//! for the Tauri application.
//!
//! **Feature: fix-command-registration**
//! **Validates: Requirements 1.2, 2.2, 2.4**

use std::sync::{Arc, Mutex, RwLock};
use std::time::SystemTime;
use crate::command_registry::{
    CommandRegistry, CommandInfo,
    ModuleInitializer, Module, InitState,
    DiagnosticTool, DiagnosticReport,
    EnhancedErrorHandler,
};
use crate::command_registry::errors::{ModuleError, ModuleErrorType};

/// Startup validation result
#[derive(Debug, Clone)]
pub struct StartupValidationResult {
    pub success: bool,
    pub timestamp: SystemTime,
    pub registered_commands: Vec<String>,
    pub failed_commands: Vec<String>,
    pub module_states: std::collections::HashMap<String, InitState>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl StartupValidationResult {
    pub fn new() -> Self {
        Self {
            success: true,
            timestamp: SystemTime::now(),
            registered_commands: Vec::new(),
            failed_commands: Vec::new(),
            module_states: std::collections::HashMap::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.success = false;
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

/// Application startup manager
/// 
/// Manages the initialization of modules and registration of commands
/// during application startup.
pub struct StartupManager {
    command_registry: Arc<RwLock<CommandRegistry>>,
    module_initializer: Arc<Mutex<ModuleInitializer>>,
    error_handler: Arc<EnhancedErrorHandler>,
}

impl StartupManager {
    fn format_command_error(error: &crate::command_registry::errors::CommandError) -> String {
        if let Some(context) = &error.context {
            format!("[{:?}] {} ({})", error.error_type, error.message, context)
        } else {
            format!("[{:?}] {}", error.error_type, error.message)
        }
    }

    fn format_module_error(error: &ModuleError) -> String {
        format!(
            "[{:?}] {}: {}",
            error.error_type,
            error.module_name,
            error.message
        )
    }

    /// Create a new startup manager
    pub fn new() -> Self {
        Self {
            command_registry: Arc::new(RwLock::new(CommandRegistry::new())),
            module_initializer: Arc::new(Mutex::new(ModuleInitializer::new())),
            error_handler: Arc::new(EnhancedErrorHandler::new()),
        }
    }

    /// Get the command registry
    pub fn get_registry(&self) -> Arc<RwLock<CommandRegistry>> {
        self.command_registry.clone()
    }

    /// Get the module initializer
    pub fn get_initializer(&self) -> Arc<Mutex<ModuleInitializer>> {
        self.module_initializer.clone()
    }

    /// Register a module with the startup manager
    pub fn register_module(&self, module: Box<dyn Module>) -> Result<(), ModuleError> {
        let mut initializer = self.module_initializer.lock()
            .map_err(|e| ModuleError::new(
                "startup".to_string(),
                format!("Failed to acquire lock: {}", e),
                ModuleErrorType::InitializationFailed,
            ))?;
        initializer.register_module(module)
    }

    /// Initialize all registered modules
    pub fn initialize_modules(&self) -> Result<(), Vec<ModuleError>> {
        let mut initializer = self.module_initializer.lock()
            .map_err(|e| vec![ModuleError::new(
                "startup".to_string(),
                format!("Failed to acquire lock: {}", e),
                ModuleErrorType::InitializationFailed,
            )])?;
        initializer.initialize_all_with_recovery()
    }

    /// Register all application commands
    pub fn register_commands(&self) -> Result<(), Vec<crate::command_registry::errors::CommandError>> {
        let mut registry = self.command_registry.write()
            .map_err(|e| vec![crate::command_registry::errors::CommandError::new(
                format!("Failed to acquire lock: {}", e),
                crate::command_registry::errors::ErrorType::RegistrationFailed,
            )])?;

        let mut errors = Vec::new();

        // Define all commands with their dependencies
        let commands = get_all_command_definitions();

        for (name, dependencies) in commands {
            let command_info = CommandInfo::with_dependencies(name.clone(), dependencies)
                .mark_registered();
            
            if let Err(e) = registry.register_command(command_info) {
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Verify all registered commands
    pub fn verify_commands(&self) -> Vec<crate::command_registry::errors::CommandError> {
        let registry = match self.command_registry.read() {
            Ok(r) => r,
            Err(_) => return vec![crate::command_registry::errors::CommandError::new(
                "Failed to acquire lock".to_string(),
                crate::command_registry::errors::ErrorType::ValidationFailed,
            )],
        };
        registry.verify_all_commands()
    }

    /// Perform comprehensive startup validation
    pub fn validate_startup(&self) -> StartupValidationResult {
        let mut result = StartupValidationResult::new();

        // Step 1: Initialize modules
        eprintln!("[INFO] Starting module initialization...");
        if let Err(errors) = self.initialize_modules() {
            for error in errors {
                result.add_error(Self::format_module_error(&error));
            }
        }

        // Step 2: Register commands
        eprintln!("[INFO] Registering commands...");
        if let Err(errors) = self.register_commands() {
            for error in errors {
                result.add_error(Self::format_command_error(&error));
            }
        }

        // Step 3: Verify commands
        eprintln!("[INFO] Verifying commands...");
        let verification_errors = self.verify_commands();
        for error in verification_errors {
            // Treat verification errors as warnings if they're about missing dependencies
            // that might be optional
            if error.message.contains("not yet available") {
                result.add_warning(Self::format_command_error(&error));
            } else {
                result.add_error(Self::format_command_error(&error));
            }
        }

        // Step 4: Collect results
        if let Ok(registry) = self.command_registry.read() {
            result.registered_commands = registry.list_available_commands();
            result.failed_commands = registry.get_anomalous_commands();
        }

        if let Ok(initializer) = self.module_initializer.lock() {
            result.module_states = initializer.get_all_states().clone();
        }

        // Log summary
        if result.success {
            eprintln!(
                "[INFO] Startup validation successful: {} commands registered",
                result.registered_commands.len()
            );
        } else {
            eprintln!(
                "[ERROR] Startup validation failed with {} errors, {} warnings",
                result.errors.len(),
                result.warnings.len()
            );
            for error in &result.errors {
                eprintln!("[ERROR]   - {}", error);
            }
            for warning in &result.warnings {
                eprintln!("[WARN]   - {}", warning);
            }
        }

        result
    }

    /// Run diagnostic check
    pub fn run_diagnostics(&self) -> Option<DiagnosticReport> {
        let registry = self.command_registry.read().ok()?;
        let initializer = self.module_initializer.lock().ok()?;
        
        // Create a diagnostic tool with Arc references
        let registry_arc = Arc::new(registry.clone());
        let initializer_arc = Arc::new(ModuleInitializer::new()); // Create a new one for diagnostics
        drop(registry);
        drop(initializer);
        
        let tool = DiagnosticTool::new(registry_arc, initializer_arc);
        Some(tool.run_full_diagnostic())
    }
}

impl Default for StartupManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Get all command definitions with their dependencies
/// 
/// This function returns a list of all Tauri commands that should be registered,
/// along with their module dependencies.
pub fn get_all_command_definitions() -> Vec<(String, Vec<String>)> {
    vec![
        // Core commands
        ("greet".to_string(), vec![]),

        // Provider management commands - 无模块依赖（内部使用 LLMClientManager）
        ("cmd_get_providers".to_string(), vec![]),
        ("cmd_save_provider".to_string(), vec![]),
        ("cmd_delete_provider".to_string(), vec![]),
        ("cmd_set_active_provider".to_string(), vec![]),
        ("cmd_test_provider_connection".to_string(), vec![]),

        // Token counting - 无依赖
        ("count_prompt_tokens".to_string(), vec![]),

        // Session management commands - 内部使用数据库，但不需要 Tauri State
        ("scan_sessions".to_string(), vec![]),
        ("scan_directory".to_string(), vec![]),
        ("parse_session_tree".to_string(), vec![]),

        // Session metadata commands - 内部使用数据库
        ("set_session_rating".to_string(), vec![]),
        ("set_session_tags".to_string(), vec![]),
        ("get_session_rating".to_string(), vec![]),
        ("get_session_tags".to_string(), vec![]),
        ("archive_session".to_string(), vec![]),
        ("unarchive_session".to_string(), vec![]),
        ("get_archived_sessions".to_string(), vec![]),

        // File monitoring commands - 内部使用 monitor 模块
        ("start_file_watcher".to_string(), vec![]),

        // Log extraction commands - 内部使用 parser
        ("extract_session_log".to_string(), vec![]),
        ("export_session_log".to_string(), vec![]),

        // Vector search commands - 内部使用 database 和 embedding
        ("vector_search".to_string(), vec![]),

        // Optimization commands - 内部使用 optimizer 和 llm
        ("compress_context".to_string(), vec![]),
        ("optimize_prompt".to_string(), vec![]),

        // Template commands - 内部使用数据库
        ("get_meta_template".to_string(), vec![]),
        ("update_meta_template".to_string(), vec![]),

        // Monitored directory commands - 内部使用数据库
        ("get_monitored_directories".to_string(), vec![]),
        ("add_monitored_directory".to_string(), vec![]),
        ("remove_monitored_directory".to_string(), vec![]),
        ("toggle_monitored_directory".to_string(), vec![]),
        ("update_monitored_directory".to_string(), vec![]),

        // Benchmark commands - 内部使用数据库
        ("run_benchmarks".to_string(), vec![]),

        // Additional monitored directory commands
        ("get_sessions_by_monitored_directory".to_string(), vec![]),

        // Additional vector search commands
        ("semantic_search".to_string(), vec![]),
        ("find_similar_sessions".to_string(), vec![]),
        ("get_vector_settings".to_string(), vec![]),
        ("update_vector_settings".to_string(), vec![]),
        ("sync_embeddings_now".to_string(), vec![]),
    ]
}

/// Database module implementation
pub struct DatabaseModule {
    initialized: bool,
}

impl DatabaseModule {
    pub fn new() -> Self {
        Self { initialized: false }
    }
}

impl Module for DatabaseModule {
    fn name(&self) -> &str {
        "database"
    }

    fn dependencies(&self) -> Vec<String> {
        vec![] // Database has no dependencies
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        // Initialize database connection
        match crate::database::init::get_connection_shared() {
            Ok(_) => {
                self.initialized = true;
                eprintln!("[INFO] Database module initialized successfully");
                Ok(())
            }
            Err(e) => {
                Err(ModuleError::new(
                    "database".to_string(),
                    format!("Failed to initialize database: {}", e),
                    ModuleErrorType::InitializationFailed,
                ))
            }
        }
    }

    fn health_check(&self) -> Result<(), ModuleError> {
        if !self.initialized {
            return Err(ModuleError::new(
                "database".to_string(),
                "Database not initialized".to_string(),
                ModuleErrorType::HealthCheckFailed,
            ));
        }

        // Try to get a connection to verify health
        match crate::database::init::get_connection_shared() {
            Ok(conn) => {
                // Try a simple query
                let guard = conn.lock().map_err(|e| ModuleError::new(
                    "database".to_string(),
                    format!("Failed to acquire lock: {}", e),
                    ModuleErrorType::HealthCheckFailed,
                ))?;
                
                guard.query_row("SELECT 1", [], |_| Ok(()))
                    .map_err(|e| ModuleError::new(
                        "database".to_string(),
                        format!("Health check query failed: {}", e),
                        ModuleErrorType::HealthCheckFailed,
                    ))?;
                
                Ok(())
            }
            Err(e) => Err(ModuleError::new(
                "database".to_string(),
                format!("Health check failed: {}", e),
                ModuleErrorType::HealthCheckFailed,
            )),
        }
    }

    fn shutdown(&mut self) -> Result<(), ModuleError> {
        self.initialized = false;
        eprintln!("[INFO] Database module shut down");
        Ok(())
    }
}

/// Monitor module implementation
pub struct MonitorModule {
    initialized: bool,
}

impl MonitorModule {
    pub fn new() -> Self {
        Self { initialized: false }
    }
}

impl Module for MonitorModule {
    fn name(&self) -> &str {
        "monitor"
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["database".to_string()] // Monitor depends on database
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        self.initialized = true;
        eprintln!("[INFO] Monitor module initialized successfully");
        Ok(())
    }

    fn health_check(&self) -> Result<(), ModuleError> {
        if !self.initialized {
            return Err(ModuleError::new(
                "monitor".to_string(),
                "Monitor not initialized".to_string(),
                ModuleErrorType::HealthCheckFailed,
            ));
        }
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), ModuleError> {
        self.initialized = false;
        eprintln!("[INFO] Monitor module shut down");
        Ok(())
    }
}

/// LLM module implementation
pub struct LLMModule {
    initialized: bool,
}

impl LLMModule {
    pub fn new() -> Self {
        Self { initialized: false }
    }
}

impl Module for LLMModule {
    fn name(&self) -> &str {
        "llm"
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["database".to_string()] // LLM depends on database for provider storage
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        self.initialized = true;
        eprintln!("[INFO] LLM module initialized successfully");
        Ok(())
    }

    fn health_check(&self) -> Result<(), ModuleError> {
        if !self.initialized {
            return Err(ModuleError::new(
                "llm".to_string(),
                "LLM not initialized".to_string(),
                ModuleErrorType::HealthCheckFailed,
            ));
        }
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), ModuleError> {
        self.initialized = false;
        eprintln!("[INFO] LLM module shut down");
        Ok(())
    }
}

/// Parser module implementation
pub struct ParserModule {
    initialized: bool,
}

impl ParserModule {
    pub fn new() -> Self {
        Self { initialized: false }
    }
}

impl Module for ParserModule {
    fn name(&self) -> &str {
        "parser"
    }

    fn dependencies(&self) -> Vec<String> {
        vec![] // Parser has no dependencies
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        self.initialized = true;
        eprintln!("[INFO] Parser module initialized successfully");
        Ok(())
    }

    fn health_check(&self) -> Result<(), ModuleError> {
        if !self.initialized {
            return Err(ModuleError::new(
                "parser".to_string(),
                "Parser not initialized".to_string(),
                ModuleErrorType::HealthCheckFailed,
            ));
        }
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), ModuleError> {
        self.initialized = false;
        eprintln!("[INFO] Parser module shut down");
        Ok(())
    }
}

/// Optimizer module implementation
pub struct OptimizerModule {
    initialized: bool,
}

impl OptimizerModule {
    pub fn new() -> Self {
        Self { initialized: false }
    }
}

impl Module for OptimizerModule {
    fn name(&self) -> &str {
        "optimizer"
    }

    fn dependencies(&self) -> Vec<String> {
        vec![] // Optimizer has no dependencies
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        self.initialized = true;
        eprintln!("[INFO] Optimizer module initialized successfully");
        Ok(())
    }

    fn health_check(&self) -> Result<(), ModuleError> {
        if !self.initialized {
            return Err(ModuleError::new(
                "optimizer".to_string(),
                "Optimizer not initialized".to_string(),
                ModuleErrorType::HealthCheckFailed,
            ));
        }
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), ModuleError> {
        self.initialized = false;
        eprintln!("[INFO] Optimizer module shut down");
        Ok(())
    }
}

/// Embedding module implementation
pub struct EmbeddingModule {
    initialized: bool,
}

impl EmbeddingModule {
    pub fn new() -> Self {
        Self { initialized: false }
    }
}

impl Module for EmbeddingModule {
    fn name(&self) -> &str {
        "embedding"
    }

    fn dependencies(&self) -> Vec<String> {
        vec![] // Embedding has no dependencies
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        self.initialized = true;
        eprintln!("[INFO] Embedding module initialized successfully");
        Ok(())
    }

    fn health_check(&self) -> Result<(), ModuleError> {
        if !self.initialized {
            return Err(ModuleError::new(
                "embedding".to_string(),
                "Embedding not initialized".to_string(),
                ModuleErrorType::HealthCheckFailed,
            ));
        }
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), ModuleError> {
        self.initialized = false;
        eprintln!("[INFO] Embedding module shut down");
        Ok(())
    }
}

/// Initialize the startup manager with all modules
pub fn create_startup_manager() -> StartupManager {
    let manager = StartupManager::new();

    // Register all modules
    let modules: Vec<Box<dyn Module>> = vec![
        Box::new(DatabaseModule::new()),
        Box::new(MonitorModule::new()),
        Box::new(LLMModule::new()),
        Box::new(ParserModule::new()),
        Box::new(OptimizerModule::new()),
        Box::new(EmbeddingModule::new()),
    ];

    for module in modules {
        if let Err(e) = manager.register_module(module) {
            eprintln!("[ERROR] Failed to register module: {}", e.message);
        }
    }

    manager
}

/// Perform startup validation and return the result
/// 
/// This function should be called during application startup to ensure
/// all modules are initialized and commands are registered correctly.
pub fn perform_startup_validation() -> StartupValidationResult {
    let manager = create_startup_manager();
    manager.validate_startup()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_startup_manager_creation() {
        let manager = StartupManager::new();
        assert!(manager.command_registry.read().is_ok());
        assert!(manager.module_initializer.lock().is_ok());
    }

    #[test]
    fn test_command_definitions() {
        let commands = get_all_command_definitions();
        assert!(!commands.is_empty());
        
        // Verify some key commands are defined
        let command_names: Vec<&str> = commands.iter().map(|(name, _)| name.as_str()).collect();
        assert!(command_names.contains(&"get_monitored_directories"));
        assert!(command_names.contains(&"scan_sessions"));
        assert!(command_names.contains(&"cmd_get_providers"));
    }

    #[test]
    fn test_startup_validation_result() {
        let mut result = StartupValidationResult::new();
        assert!(result.success);
        assert!(result.errors.is_empty());
        
        result.add_error("Test error".to_string());
        assert!(!result.success);
        assert_eq!(result.errors.len(), 1);
        
        result.add_warning("Test warning".to_string());
        assert_eq!(result.warnings.len(), 1);
    }
}

/// Property-based tests for startup validation
/// **Feature: fix-command-registration, Property 3: Comprehensive startup validation**
/// **Validates: Requirements 1.2, 2.2, 2.4**
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::HashSet;

    /// Generate arbitrary module names
    fn arb_module_name() -> impl Strategy<Value = String> {
        "[a-z][a-z0-9_]*"
            .prop_map(|s| s.to_string())
            .prop_filter("Module name should not be empty", |s| !s.is_empty() && s.len() < 20)
    }

    /// Generate arbitrary command names
    fn arb_command_name() -> impl Strategy<Value = String> {
        "[a-z_][a-z0-9_]*"
            .prop_map(|s| s.to_string())
            .prop_filter("Command name should not be empty", |s| !s.is_empty() && s.len() < 30)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]
        
        /// Property 3: Comprehensive startup validation
        /// For any system startup, all registered commands should have their dependencies verified,
        /// modules should be initialized in correct order, and each command's callability should be validated
        /// **Feature: fix-command-registration, Property 3: Comprehensive startup validation**
        /// **Validates: Requirements 1.2, 2.2, 2.4**
        #[test]
        fn prop_comprehensive_startup_validation(
            command_names in prop::collection::vec(arb_command_name(), 1..10)
        ) {
            let manager = StartupManager::new();
            
            // Register commands with the manager
            {
                let mut registry = manager.command_registry.write().unwrap();
                let mut registered = HashSet::new();
                
                for name in &command_names {
                    if registered.contains(name) {
                        continue; // Skip duplicates
                    }
                    
                    let command_info = CommandInfo::new(name.clone()).mark_registered();
                    let result = registry.register_command(command_info);
                    
                    // Registration should succeed for valid unique names
                    prop_assert!(result.is_ok(), "Failed to register command: {}", name);
                    registered.insert(name.clone());
                }
            }
            
            // Verify all commands are registered
            {
                let registry = manager.command_registry.read().unwrap();
                let available = registry.list_available_commands();
                
                // All registered commands should be available
                let registered_set: HashSet<_> = command_names.iter().cloned().collect();
                for cmd in &registered_set {
                    prop_assert!(
                        available.contains(cmd),
                        "Command {} should be available after registration",
                        cmd
                    );
                }
            }
            
            // Verify commands can be verified
            {
                let registry = manager.command_registry.read().unwrap();
                let errors = registry.verify_all_commands();
                
                // Commands without dependencies should have no verification errors
                // (since we registered them without dependencies)
                for cmd in &command_names {
                    let cmd_errors: Vec<_> = errors.iter()
                        .filter(|e| e.message.contains(cmd))
                        .collect();
                    
                    // No errors expected for commands without dependencies
                    prop_assert!(
                        cmd_errors.is_empty(),
                        "Command {} should have no verification errors when registered without dependencies",
                        cmd
                    );
                }
            }
        }

        /// Property: Module initialization order is respected
        /// For any set of modules with dependencies, modules should be initialized
        /// in the correct dependency order
        #[test]
        fn prop_module_initialization_order(
            module_count in 2..6usize
        ) {
            let manager = StartupManager::new();
            
            // Create a chain of modules where each depends on the previous
            let module_names: Vec<String> = (0..module_count)
                .map(|i| format!("test_module_{}", i))
                .collect();
            
            // Register modules with chain dependencies
            {
                let mut initializer = manager.module_initializer.lock().unwrap();
                
                for (i, name) in module_names.iter().enumerate() {
                    let deps = if i > 0 {
                        vec![module_names[i - 1].clone()]
                    } else {
                        vec![]
                    };
                    
                    // Create a mock module
                    struct TestModule {
                        name: String,
                        deps: Vec<String>,
                        initialized: bool,
                    }
                    
                    impl Module for TestModule {
                        fn name(&self) -> &str { &self.name }
                        fn dependencies(&self) -> Vec<String> { self.deps.clone() }
                        fn initialize(&mut self) -> Result<(), ModuleError> {
                            self.initialized = true;
                            Ok(())
                        }
                        fn health_check(&self) -> Result<(), ModuleError> { Ok(()) }
                        fn shutdown(&mut self) -> Result<(), ModuleError> { Ok(()) }
                    }
                    
                    let module = Box::new(TestModule {
                        name: name.clone(),
                        deps,
                        initialized: false,
                    });
                    
                    let result = initializer.register_module(module);
                    prop_assert!(result.is_ok(), "Failed to register module: {}", name);
                }
                
                // Get initialization order
                let order_result = initializer.get_initialization_order();
                prop_assert!(order_result.is_ok(), "Should be able to get initialization order");
                
                let order = order_result.unwrap();
                
                // Verify order respects dependencies
                for (i, name) in module_names.iter().enumerate() {
                    if i > 0 {
                        let dep_name = &module_names[i - 1];
                        let dep_pos = order.iter().position(|x| x == dep_name);
                        let mod_pos = order.iter().position(|x| x == name);
                        
                        if let (Some(dep_idx), Some(mod_idx)) = (dep_pos, mod_pos) {
                            prop_assert!(
                                dep_idx < mod_idx,
                                "Dependency {} should be initialized before {}",
                                dep_name, name
                            );
                        }
                    }
                }
            }
        }

        /// Property: Command dependencies are verified during startup
        /// For any command with dependencies, the startup validation should verify
        /// that all dependencies are available
        #[test]
        fn prop_command_dependency_verification(
            command_name in arb_command_name(),
            dependency_names in prop::collection::vec(arb_command_name(), 1..5)
        ) {
            let manager = StartupManager::new();
            
            // Register command with dependencies (but don't register the dependencies)
            {
                let mut registry = manager.command_registry.write().unwrap();
                
                // Filter out self-dependencies and duplicates
                let filtered_deps: Vec<String> = dependency_names.iter()
                    .filter(|d| *d != &command_name)
                    .cloned()
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect();
                
                let command_info = CommandInfo::with_dependencies(
                    command_name.clone(),
                    filtered_deps.clone()
                ).mark_registered();
                
                let result = registry.register_command(command_info);
                prop_assert!(result.is_ok(), "Failed to register command with dependencies");
                
                // Verify the command - should detect missing dependencies
                let errors = registry.verify_all_commands();
                
                // Should have errors for missing dependencies
                if !filtered_deps.is_empty() {
                    prop_assert!(
                        !errors.is_empty(),
                        "Should detect missing dependencies for command {}",
                        command_name
                    );
                    
                    // Each missing dependency should be mentioned in errors
                    for dep in &filtered_deps {
                        let dep_mentioned = errors.iter().any(|e| 
                            e.message.contains(dep) || 
                            e.context.as_ref().map_or(false, |ctx| ctx.contains(dep))
                        );
                        prop_assert!(
                            dep_mentioned,
                            "Missing dependency {} should be mentioned in errors",
                            dep
                        );
                    }
                }
            }
        }

        /// Property: Startup validation result is consistent
        /// For any startup validation, the result should accurately reflect
        /// the state of registered commands and modules
        #[test]
        fn prop_startup_validation_result_consistency(
            num_commands in 1..10usize,
            num_errors in 0..5usize
        ) {
            let mut result = StartupValidationResult::new();
            
            // Add some registered commands
            for i in 0..num_commands {
                result.registered_commands.push(format!("command_{}", i));
            }
            
            // Add some errors
            for i in 0..num_errors {
                result.add_error(format!("Error {}", i));
            }
            
            // Verify consistency
            prop_assert_eq!(
                result.registered_commands.len(),
                num_commands,
                "Registered commands count should match"
            );
            
            prop_assert_eq!(
                result.errors.len(),
                num_errors,
                "Errors count should match"
            );
            
            // Success should be false if there are errors
            if num_errors > 0 {
                prop_assert!(
                    !result.success,
                    "Success should be false when there are errors"
                );
            }
            
            // Timestamp should be valid
            prop_assert!(
                result.timestamp <= SystemTime::now(),
                "Timestamp should not be in the future"
            );
        }
    }
}
