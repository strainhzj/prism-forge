//! Module Initializer Implementation
//!
//! 按正确顺序初始化所有依赖模块

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::command_registry::errors::{ModuleError, ModuleErrorType};

/// Module initializer that manages module dependencies and initialization order
pub struct ModuleInitializer {
    modules: HashMap<String, Box<dyn Module>>,
    dependency_graph: DependencyGraph,
    initialization_state: HashMap<String, InitState>,
}

/// Trait that all modules must implement
pub trait Module: Send + Sync {
    /// Get the module name
    fn name(&self) -> &str;
    
    /// Get the list of module dependencies
    fn dependencies(&self) -> Vec<String>;
    
    /// Initialize the module
    fn initialize(&mut self) -> Result<(), ModuleError>;
    
    /// Perform health check on the module
    fn health_check(&self) -> Result<(), ModuleError>;
    
    /// Shutdown the module
    fn shutdown(&mut self) -> Result<(), ModuleError>;
}

/// State of module initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InitState {
    Pending,
    Initializing,
    Ready,
    Failed(String),
}

/// Dependency graph for modules
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    nodes: HashMap<String, ModuleNode>,
    edges: Vec<DependencyEdge>,
}

/// Node in the dependency graph
#[derive(Debug, Clone)]
pub struct ModuleNode {
    name: String,
    module_type: ModuleType,
    priority: u32,
    required: bool,
}

/// Edge in the dependency graph
#[derive(Debug, Clone)]
pub struct DependencyEdge {
    from: String,
    to: String,
    dependency_type: DependencyType,
}

/// Type of module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModuleType {
    Core,
    Database,
    Monitor,
    LLM,
    Parser,
    Optimizer,
}

/// Type of dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Hard,      // 必须依赖
    Soft,      // 可选依赖
    Circular,  // 循环依赖（需要特殊处理）
}

/// Recovery strategy for failed modules
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    Retry,      // Retry initialization
    Fallback,   // Use fallback configuration
    Skip,       // Skip this module (non-critical)
    Abort,      // Abort initialization (critical failure)
}

/// Health status of a module
#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Critical(String),
    Failed(String),
}

/// Comprehensive health check report
#[derive(Debug, Clone)]
pub struct HealthCheckReport {
    pub timestamp: std::time::SystemTime,
    pub module_statuses: HashMap<String, HealthStatus>,
    pub response_times: HashMap<String, std::time::Duration>,
    pub dependency_issues: Vec<DependencyIssue>,
    pub overall_status: SystemHealthStatus,
}

/// Dependency issue in health check
#[derive(Debug, Clone)]
pub struct DependencyIssue {
    pub module: String,
    pub dependency: String,
    pub issue: String,
}

/// Overall system health status
#[derive(Debug, Clone)]
pub enum SystemHealthStatus {
    Healthy,
    Degraded,
    Critical,
    Failed,
}

impl HealthCheckReport {
    pub fn new() -> Self {
        Self {
            timestamp: std::time::SystemTime::now(),
            module_statuses: HashMap::new(),
            response_times: HashMap::new(),
            dependency_issues: Vec::new(),
            overall_status: SystemHealthStatus::Healthy,
        }
    }

    pub fn add_module_status(&mut self, name: String, status: HealthStatus, response_time: std::time::Duration) {
        // Update overall status based on module status
        match &status {
            HealthStatus::Critical(_) | HealthStatus::Failed(_) => {
                self.overall_status = SystemHealthStatus::Critical;
            }
            HealthStatus::Degraded(_) => {
                if matches!(self.overall_status, SystemHealthStatus::Healthy) {
                    self.overall_status = SystemHealthStatus::Degraded;
                }
            }
            HealthStatus::Healthy => {
                // Keep current status
            }
        }

        self.module_statuses.insert(name.clone(), status);
        self.response_times.insert(name, response_time);
    }

    pub fn add_dependency_issue(&mut self, module: String, dependency: String, issue: String) {
        self.dependency_issues.push(DependencyIssue {
            module,
            dependency,
            issue,
        });
        
        // Dependency issues indicate degraded system health
        if matches!(self.overall_status, SystemHealthStatus::Healthy) {
            self.overall_status = SystemHealthStatus::Degraded;
        }
    }

    pub fn get_module_status(&self, name: &str) -> Option<&HealthStatus> {
        self.module_statuses.get(name)
    }

    pub fn is_system_healthy(&self) -> bool {
        matches!(self.overall_status, SystemHealthStatus::Healthy)
    }

    pub fn get_critical_modules(&self) -> Vec<String> {
        self.module_statuses
            .iter()
            .filter_map(|(name, status)| {
                match status {
                    HealthStatus::Critical(_) | HealthStatus::Failed(_) => Some(name.clone()),
                    _ => None,
                }
            })
            .collect()
    }
}

impl ModuleInitializer {
    /// Create a new module initializer
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            dependency_graph: DependencyGraph::new(),
            initialization_state: HashMap::new(),
        }
    }

    /// Register a module with the initializer
    pub fn register_module(&mut self, module: Box<dyn Module>) -> Result<(), ModuleError> {
        let name = module.name().to_string();
        
        // Check for duplicate registration
        if self.modules.contains_key(&name) {
            return Err(ModuleError::new(
                name,
                "Module is already registered".to_string(),
                ModuleErrorType::InitializationFailed,
            ));
        }

        // Add to dependency graph
        let dependencies = module.dependencies();
        self.dependency_graph.add_node(ModuleNode {
            name: name.clone(),
            module_type: ModuleType::Core, // Default type
            priority: 0,
            required: true,
        });

        for dep in dependencies {
            self.dependency_graph.add_edge(DependencyEdge {
                from: name.clone(),
                to: dep,
                dependency_type: DependencyType::Hard,
            });
        }

        // Set initial state
        self.initialization_state.insert(name.clone(), InitState::Pending);
        
        // Register the module
        self.modules.insert(name, module);

        Ok(())
    }

    /// Initialize all modules in dependency order
    pub fn initialize_all(&mut self) -> Result<(), Vec<ModuleError>> {
        let initialization_order = self.get_initialization_order()?;
        let mut errors = Vec::new();

        for module_name in initialization_order {
            if let Err(error) = self.initialize_module(&module_name) {
                errors.push(error.clone());
                
                // Mark as failed
                self.initialization_state.insert(module_name.clone(), InitState::Failed(
                    error.message.clone()
                ));

                // Attempt recovery for critical modules
                if let Err(recovery_error) = self.attempt_recovery(&module_name, &error) {
                    errors.push(recovery_error);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Initialize all modules with recovery strategies
    pub fn initialize_all_with_recovery(&mut self) -> Result<(), Vec<ModuleError>> {
        let initialization_order = self.get_initialization_order()?;
        let mut errors = Vec::new();
        let mut retry_queue = Vec::new();

        // First pass: try to initialize all modules
        for module_name in initialization_order {
            match self.initialize_module(&module_name) {
                Ok(()) => {
                    // Success - continue
                }
                Err(error) => {
                    // Check if this is a recoverable error
                    if self.is_recoverable_error(&error) {
                        retry_queue.push((module_name.clone(), error.clone()));
                    } else {
                        errors.push(error.clone());
                        self.initialization_state.insert(module_name.clone(), InitState::Failed(
                            error.message.clone()
                        ));
                    }
                }
            }
        }

        // Second pass: retry failed modules with recovery
        for (module_name, original_error) in retry_queue {
            match self.attempt_recovery(&module_name, &original_error) {
                Ok(()) => {
                    // Recovery successful, try initialization again
                    if let Err(retry_error) = self.initialize_module(&module_name) {
                        let error_message = retry_error.message.clone();
                        errors.push(retry_error);
                        self.initialization_state.insert(module_name, InitState::Failed(
                            format!("Module initialization failed after recovery attempt: {}", error_message)
                        ));
                    }
                }
                Err(recovery_error) => {
                    errors.push(recovery_error);
                    self.initialization_state.insert(module_name, InitState::Failed(
                        "Recovery failed".to_string()
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get the correct initialization order based on dependencies
    pub fn get_initialization_order(&self) -> Result<Vec<String>, Vec<ModuleError>> {
        // Perform topological sort on dependency graph
        self.dependency_graph.topological_sort()
    }

    /// Perform health check on all modules
    pub fn health_check_all(&self) -> HashMap<String, Result<(), ModuleError>> {
        let mut results = HashMap::new();

        for (name, module) in &self.modules {
            let result = module.health_check();
            results.insert(name.clone(), result);
        }

        results
    }

    /// Perform comprehensive health check with detailed reporting
    pub fn comprehensive_health_check(&self) -> HealthCheckReport {
        let mut report = HealthCheckReport::new();
        
        for (name, module) in &self.modules {
            let start_time = std::time::Instant::now();
            let health_result = module.health_check();
            let duration = start_time.elapsed();
            
            let status = match &health_result {
                Ok(()) => HealthStatus::Healthy,
                Err(error) => {
                    if self.is_critical_module(name) {
                        HealthStatus::Critical(error.message.clone())
                    } else {
                        HealthStatus::Degraded(error.message.clone())
                    }
                }
            };

            report.add_module_status(name.clone(), status, duration);
        }

        // Check for dependency health issues
        for (name, _) in &self.modules {
            if let Some(deps) = self.get_module_dependencies(name) {
                for dep in deps {
                    if let Some(dep_status) = report.get_module_status(&dep) {
                        if matches!(dep_status, HealthStatus::Critical(_) | HealthStatus::Failed(_)) {
                            report.add_dependency_issue(name.clone(), dep, "Dependency unhealthy".to_string());
                        }
                    }
                }
            }
        }

        report
    }

    /// Check if a module is critical to system operation
    fn is_critical_module(&self, module_name: &str) -> bool {
        // Database and core modules are typically critical
        matches!(module_name, "database" | "core" | "security")
    }

    /// Get dependencies for a specific module
    fn get_module_dependencies(&self, module_name: &str) -> Option<Vec<String>> {
        self.modules.get(module_name).map(|module| module.dependencies())
    }

    /// Get the state of a specific module
    pub fn get_module_state(&self, name: &str) -> Option<&InitState> {
        self.initialization_state.get(name)
    }

    /// Get all module states
    pub fn get_all_states(&self) -> &HashMap<String, InitState> {
        &self.initialization_state
    }

    /// Initialize a specific module
    fn initialize_module(&mut self, name: &str) -> Result<(), ModuleError> {
        // Check if module exists
        let module = self.modules.get_mut(name)
            .ok_or_else(|| ModuleError::new(
                name.to_string(),
                "Module not found".to_string(),
                ModuleErrorType::InitializationFailed,
            ))?;

        // Set state to initializing
        self.initialization_state.insert(name.to_string(), InitState::Initializing);

        // Initialize the module
        match module.initialize() {
            Ok(()) => {
                self.initialization_state.insert(name.to_string(), InitState::Ready);
                Ok(())
            }
            Err(error) => {
                self.initialization_state.insert(name.to_string(), InitState::Failed(
                    error.message.clone()
                ));
                Err(error)
            }
        }
    }

    /// Attempt to recover from a module initialization failure
    fn attempt_recovery(&mut self, module_name: &str, error: &ModuleError) -> Result<(), ModuleError> {
        // Determine recovery strategy based on error type and module criticality
        let recovery_strategy = self.determine_recovery_strategy(module_name, error);
        
        match recovery_strategy {
            RecoveryStrategy::Retry => {
                // Simple retry - reset state and try again
                self.initialization_state.insert(module_name.to_string(), InitState::Pending);
                Ok(())
            }
            RecoveryStrategy::Fallback => {
                // Use fallback configuration or safe mode
                self.apply_fallback_configuration(module_name)
            }
            RecoveryStrategy::Skip => {
                // Mark as disabled but continue
                self.initialization_state.insert(module_name.to_string(), InitState::Failed(
                    "Skipped due to non-critical failure".to_string()
                ));
                Ok(())
            }
            RecoveryStrategy::Abort => {
                // Critical failure - cannot recover
                Err(ModuleError::new(
                    module_name.to_string(),
                    format!("Critical module failure: {}", error.message),
                    ModuleErrorType::InitializationFailed,
                ))
            }
        }
    }

    /// Determine the appropriate recovery strategy for a failed module
    fn determine_recovery_strategy(&self, module_name: &str, error: &ModuleError) -> RecoveryStrategy {
        // Critical modules require different handling
        if self.is_critical_module(module_name) {
            match error.error_type {
                ModuleErrorType::DependencyMissing => RecoveryStrategy::Abort,
                ModuleErrorType::InitializationFailed => RecoveryStrategy::Fallback,
                ModuleErrorType::HealthCheckFailed => RecoveryStrategy::Retry,
                ModuleErrorType::ShutdownFailed => RecoveryStrategy::Skip, // Shutdown errors are not critical for recovery
            }
        } else {
            // Non-critical modules can be skipped or retried
            match error.error_type {
                ModuleErrorType::DependencyMissing => RecoveryStrategy::Skip,
                ModuleErrorType::InitializationFailed => RecoveryStrategy::Retry,
                ModuleErrorType::HealthCheckFailed => RecoveryStrategy::Skip,
                ModuleErrorType::ShutdownFailed => RecoveryStrategy::Skip,
            }
        }
    }

    /// Apply fallback configuration for a module
    fn apply_fallback_configuration(&mut self, module_name: &str) -> Result<(), ModuleError> {
        // This would typically involve loading safe defaults or minimal configuration
        // For now, we'll mark it as a degraded state
        self.initialization_state.insert(module_name.to_string(), InitState::Failed(
            "Running in fallback mode".to_string()
        ));
        
        // In a real implementation, this might:
        // - Load minimal configuration
        // - Disable non-essential features
        // - Set up monitoring for recovery
        
        Ok(())
    }

    /// Check if an error is recoverable
    fn is_recoverable_error(&self, error: &ModuleError) -> bool {
        match error.error_type {
            ModuleErrorType::InitializationFailed => true,  // Can retry
            ModuleErrorType::HealthCheckFailed => true,     // Can retry
            ModuleErrorType::DependencyMissing => false,    // Cannot recover without dependency
            ModuleErrorType::ShutdownFailed => false,       // Shutdown errors are not recoverable during initialization
        }
    }

    /// Shutdown all modules in reverse dependency order
    pub fn shutdown_all(&mut self) -> Result<(), Vec<ModuleError>> {
        let mut shutdown_order = self.get_initialization_order().unwrap_or_default();
        shutdown_order.reverse(); // Shutdown in reverse order
        
        let mut errors = Vec::new();

        for module_name in shutdown_order {
            if let Some(module) = self.modules.get_mut(&module_name) {
                if let Err(error) = module.shutdown() {
                    errors.push(error);
                }
            }
            // Mark as pending regardless of shutdown result
            self.initialization_state.insert(module_name, InitState::Pending);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl DependencyGraph {
    /// Create a new dependency graph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: ModuleNode) {
        self.nodes.insert(node.name.clone(), node);
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, edge: DependencyEdge) {
        self.edges.push(edge);
    }

    /// Perform topological sort to get initialization order
    pub fn topological_sort(&self) -> Result<Vec<String>, Vec<ModuleError>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();

        // Initialize in-degree and adjacency list
        for node_name in self.nodes.keys() {
            in_degree.insert(node_name.clone(), 0);
            adj_list.insert(node_name.clone(), Vec::new());
        }

        // Build adjacency list and calculate in-degrees
        for edge in &self.edges {
            if let DependencyType::Hard = edge.dependency_type {
                // For hard dependencies, the dependency must be initialized first
                adj_list.get_mut(&edge.to).unwrap().push(edge.from.clone());
                *in_degree.get_mut(&edge.from).unwrap() += 1;
            }
        }

        // Kahn's algorithm for topological sorting
        let mut queue = Vec::new();
        let mut result = Vec::new();

        // Find all nodes with no incoming edges
        for (node, &degree) in &in_degree {
            if degree == 0 {
                queue.push(node.clone());
            }
        }

        while let Some(current) = queue.pop() {
            result.push(current.clone());

            // For each neighbor of current node
            if let Some(neighbors) = adj_list.get(&current) {
                for neighbor in neighbors {
                    let degree = in_degree.get_mut(neighbor).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push(neighbor.clone());
                    }
                }
            }
        }

        // Check for circular dependencies
        if result.len() != self.nodes.len() {
            let mut errors = Vec::new();
            for (node, &degree) in &in_degree {
                if degree > 0 {
                    errors.push(ModuleError::new(
                        node.clone(),
                        "Circular dependency detected".to_string(),
                        ModuleErrorType::DependencyMissing,
                    ));
                }
            }
            return Err(errors);
        }

        Ok(result)
    }
}

impl Default for ModuleInitializer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock module for testing
    struct MockModule {
        name: String,
        dependencies: Vec<String>,
        should_fail: bool,
    }

    impl Module for MockModule {
        fn name(&self) -> &str {
            &self.name
        }

        fn dependencies(&self) -> Vec<String> {
            self.dependencies.clone()
        }

        fn initialize(&mut self) -> Result<(), ModuleError> {
            if self.should_fail {
                Err(ModuleError::initialization_failed(&self.name, "Mock failure"))
            } else {
                Ok(())
            }
        }

        fn health_check(&self) -> Result<(), ModuleError> {
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), ModuleError> {
            Ok(())
        }
    }

    #[test]
    fn test_module_initializer_creation() {
        let initializer = ModuleInitializer::new();
        assert_eq!(initializer.modules.len(), 0);
    }

    #[test]
    fn test_module_registration() {
        let mut initializer = ModuleInitializer::new();
        let module = Box::new(MockModule {
            name: "test_module".to_string(),
            dependencies: vec![],
            should_fail: false,
        });

        let result = initializer.register_module(module);
        assert!(result.is_ok());
        assert_eq!(initializer.modules.len(), 1);
    }

    #[test]
    fn test_dependency_order() {
        let mut initializer = ModuleInitializer::new();
        
        // Module A depends on Module B
        let module_a = Box::new(MockModule {
            name: "module_a".to_string(),
            dependencies: vec!["module_b".to_string()],
            should_fail: false,
        });
        
        let module_b = Box::new(MockModule {
            name: "module_b".to_string(),
            dependencies: vec![],
            should_fail: false,
        });

        initializer.register_module(module_a).unwrap();
        initializer.register_module(module_b).unwrap();

        let order = initializer.get_initialization_order().unwrap();
        
        // Module B should be initialized before Module A
        let b_index = order.iter().position(|x| x == "module_b").unwrap();
        let a_index = order.iter().position(|x| x == "module_a").unwrap();
        assert!(b_index < a_index);
    }
}