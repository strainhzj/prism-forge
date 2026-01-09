# Command Registration System Guide

## Overview

The command registration system provides a robust framework for managing Tauri commands, ensuring proper initialization, validation, and error handling.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    StartupManager                           │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │ ModuleInitializer│  │ CommandRegistry │  │ErrorHandler │ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. CommandRegistry

Manages command registration and status tracking.

```rust
use crate::command_registry::CommandRegistry;

// Create registry
let registry = CommandRegistry::new();

// Register a command
registry.register_command("get_providers", vec!["database"]);

// Check command status
let status = registry.get_command_status("get_providers");
```

### 2. ModuleInitializer

Handles module initialization with dependency ordering.

```rust
use crate::command_registry::ModuleInitializer;

let initializer = ModuleInitializer::new();

// Register modules with dependencies
initializer.register_module("database", vec![], true);
initializer.register_module("llm", vec!["database"], true);

// Initialize in correct order
let order = initializer.get_initialization_order();
```

### 3. StartupManager

Coordinates startup validation and module initialization.

```rust
use crate::startup::StartupManager;

let manager = StartupManager::new();

// Perform startup validation
let result = manager.perform_startup_validation();
if result.is_valid {
    println!("Startup successful");
}
```

## Adding New Commands

1. Define the command in `commands.rs`:
```rust
#[tauri::command]
pub async fn my_new_command() -> Result<String, String> {
    Ok("Hello".to_string())
}
```

2. Register in `lib.rs`:
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    my_new_command,
])
```

3. Add to command definitions in `startup.rs`:
```rust
fn get_all_command_definitions() -> Vec<CommandDefinition> {
    vec![
        // ... existing definitions
        CommandDefinition {
            name: "my_new_command".to_string(),
            dependencies: vec![], // Add dependencies if needed
        },
    ]
}
```

## Error Handling

The system provides user-friendly error messages:

```rust
use crate::command_wrapper::get_command_not_found_error;

// Get helpful error message for unknown command
let error = get_command_not_found_error("unknwon_cmd");
// Returns: "Command 'unknwon_cmd' not found. Did you mean: unknown_command?"
```

## Diagnostics

Run diagnostics to check system health:

```rust
use crate::command_registry::DiagnosticTool;

let diagnostic = DiagnosticTool::new(&registry, &initializer);
let report = diagnostic.generate_report();

println!("System Health: {}%", report.health_score);
```


## Troubleshooting Guide

### Common Issues

#### 1. "Command not found" Error

**Symptom:** Frontend receives "command not found" error when calling a Tauri command.

**Causes:**
- Command not registered in `invoke_handler`
- Typo in command name
- Module dependency not initialized

**Solutions:**
1. Verify command is in `lib.rs` invoke_handler
2. Check command name spelling (case-sensitive)
3. Ensure all dependencies are initialized

```rust
// Check if command is registered
let registry = app.state::<CommandRegistry>();
if !registry.is_command_registered("my_command") {
    eprintln!("Command not registered!");
}
```

#### 2. Module Initialization Failure

**Symptom:** Application fails to start or commands return errors.

**Causes:**
- Circular dependency between modules
- Required module failed to initialize
- Missing configuration

**Solutions:**
1. Check module dependencies for cycles
2. Review initialization logs
3. Verify configuration files exist

```rust
// Check module status
let initializer = app.state::<ModuleInitializer>();
for (name, state) in initializer.get_module_states() {
    if state != InitState::Ready {
        eprintln!("Module {} not ready: {:?}", name, state);
    }
}
```

#### 3. Dependency Resolution Errors

**Symptom:** Commands fail because dependencies aren't available.

**Causes:**
- Dependency not declared
- Dependency initialization failed
- Wrong initialization order

**Solutions:**
1. Add missing dependencies to command definition
2. Check dependency module status
3. Use `get_initialization_order()` to verify order

#### 4. Performance Issues

**Symptom:** Slow startup or command execution.

**Causes:**
- Too many synchronous initializations
- Heavy database operations at startup
- Unnecessary validation checks

**Solutions:**
1. Use async initialization where possible
2. Defer non-critical initialization
3. Cache validation results

### Diagnostic Commands

```bash
# Run all tests
cargo test --package prism-forge --lib

# Run startup tests only
cargo test --package prism-forge --lib startup::

# Run with verbose output
RUST_BACKTRACE=1 cargo test --package prism-forge --lib
```

### Logging

Enable detailed logging for debugging:

```rust
// In lib.rs or main.rs
eprintln!("[STARTUP] Initializing module: {}", module_name);
eprintln!("[REGISTRY] Registering command: {}", command_name);
eprintln!("[ERROR] Command failed: {} - {}", command_name, error);
```

### Health Check API

Use the built-in health check:

```rust
let diagnostic = DiagnosticTool::new(&registry, &initializer);
let health = diagnostic.quick_health_check();

if !health.is_healthy {
    for issue in health.issues {
        eprintln!("Issue: {}", issue);
    }
}
```

## Best Practices

1. **Always declare dependencies** - Even if a command works without them, declare all dependencies for proper initialization order.

2. **Use meaningful command names** - Follow the pattern `verb_noun` (e.g., `get_providers`, `add_directory`).

3. **Handle errors gracefully** - Return user-friendly error messages, not raw error strings.

4. **Test commands in isolation** - Write unit tests for each command before integration.

5. **Monitor startup time** - Keep startup validation under 1 second for good UX.

## API Reference

### CommandRegistry

| Method | Description |
|--------|-------------|
| `new()` | Create new registry |
| `register_command(name, deps)` | Register a command |
| `is_command_registered(name)` | Check if registered |
| `get_command_status(name)` | Get command status |
| `get_all_commands()` | List all commands |

### ModuleInitializer

| Method | Description |
|--------|-------------|
| `new()` | Create new initializer |
| `register_module(name, deps, required)` | Register a module |
| `get_initialization_order()` | Get sorted init order |
| `get_module_states()` | Get all module states |

### StartupManager

| Method | Description |
|--------|-------------|
| `new()` | Create new manager |
| `perform_startup_validation()` | Run full validation |
| `get_command_definitions()` | Get all command defs |
