//! Logging and Performance Monitoring Configuration
//!
//! This module provides logging utilities and performance metrics collection
//! for the command registration system.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Log levels for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }
}

/// Performance metrics for a single operation
#[derive(Debug, Clone)]
pub struct OperationMetrics {
    pub name: String,
    pub duration: Duration,
    pub success: bool,
    pub timestamp: Instant,
}

/// Performance monitor for tracking operation metrics
#[derive(Debug, Clone)]
pub struct PerformanceMonitor {
    metrics: Arc<Mutex<Vec<OperationMetrics>>>,
    thresholds: Arc<Mutex<HashMap<String, Duration>>>,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        let mut thresholds = HashMap::new();
        // Default thresholds
        thresholds.insert("startup".to_string(), Duration::from_secs(2));
        thresholds.insert("command_execution".to_string(), Duration::from_millis(500));
        thresholds.insert("module_init".to_string(), Duration::from_secs(1));
        thresholds.insert("database_query".to_string(), Duration::from_millis(100));

        Self {
            metrics: Arc::new(Mutex::new(Vec::new())),
            thresholds: Arc::new(Mutex::new(thresholds)),
        }
    }

    /// Record an operation's metrics
    pub fn record(&self, name: &str, duration: Duration, success: bool) {
        let metric = OperationMetrics {
            name: name.to_string(),
            duration,
            success,
            timestamp: Instant::now(),
        };

        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.push(metric.clone());
        }

        // Check threshold and log warning if exceeded
        if let Ok(thresholds) = self.thresholds.lock() {
            if let Some(threshold) = thresholds.get(name) {
                if duration > *threshold {
                    eprintln!(
                        "[PERF] Warning: {} took {:?} (threshold: {:?})",
                        name, duration, threshold
                    );
                }
            }
        }
    }

    /// Get average duration for an operation type
    pub fn get_average_duration(&self, operation_name: &str) -> Option<Duration> {
        let metrics = self.metrics.lock().ok()?;
        let matching: Vec<_> = metrics
            .iter()
            .filter(|m| m.name == operation_name)
            .collect();

        if matching.is_empty() {
            return None;
        }

        let total: Duration = matching.iter().map(|m| m.duration).sum();
        Some(total / matching.len() as u32)
    }

    /// Get success rate for an operation type
    pub fn get_success_rate(&self, operation_name: &str) -> Option<f64> {
        let metrics = self.metrics.lock().ok()?;
        let matching: Vec<_> = metrics
            .iter()
            .filter(|m| m.name == operation_name)
            .collect();

        if matching.is_empty() {
            return None;
        }

        let successes = matching.iter().filter(|m| m.success).count();
        Some(successes as f64 / matching.len() as f64)
    }

    /// Set a custom threshold for an operation
    pub fn set_threshold(&self, operation_name: &str, threshold: Duration) {
        if let Ok(mut thresholds) = self.thresholds.lock() {
            thresholds.insert(operation_name.to_string(), threshold);
        }
    }

    /// Get all recorded metrics
    pub fn get_all_metrics(&self) -> Vec<OperationMetrics> {
        self.metrics.lock().map(|m| m.clone()).unwrap_or_default()
    }

    /// Clear all recorded metrics
    pub fn clear(&self) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.clear();
        }
    }

    /// Generate a performance summary report
    pub fn generate_summary(&self) -> PerformanceSummary {
        let metrics = self.get_all_metrics();
        let mut by_operation: HashMap<String, Vec<&OperationMetrics>> = HashMap::new();

        for metric in &metrics {
            by_operation
                .entry(metric.name.clone())
                .or_default()
                .push(metric);
        }

        let mut operation_stats = Vec::new();
        for (name, ops) in by_operation {
            let total_duration: Duration = ops.iter().map(|m| m.duration).sum();
            let avg_duration = total_duration / ops.len() as u32;
            let success_count = ops.iter().filter(|m| m.success).count();
            let success_rate = success_count as f64 / ops.len() as f64;

            operation_stats.push(OperationStats {
                name,
                count: ops.len(),
                avg_duration,
                success_rate,
            });
        }

        PerformanceSummary {
            total_operations: metrics.len(),
            operation_stats,
        }
    }
}

/// Statistics for a single operation type
#[derive(Debug, Clone)]
pub struct OperationStats {
    pub name: String,
    pub count: usize,
    pub avg_duration: Duration,
    pub success_rate: f64,
}

/// Summary of all performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub total_operations: usize,
    pub operation_stats: Vec<OperationStats>,
}

/// Timer utility for measuring operation duration
pub struct Timer {
    start: Instant,
    operation_name: String,
    monitor: Option<Arc<PerformanceMonitor>>,
}

impl Timer {
    /// Start a new timer
    pub fn start(operation_name: &str) -> Self {
        Self {
            start: Instant::now(),
            operation_name: operation_name.to_string(),
            monitor: None,
        }
    }

    /// Start a timer with automatic recording to a monitor
    pub fn start_with_monitor(operation_name: &str, monitor: Arc<PerformanceMonitor>) -> Self {
        Self {
            start: Instant::now(),
            operation_name: operation_name.to_string(),
            monitor: Some(monitor),
        }
    }

    /// Stop the timer and return elapsed duration
    pub fn stop(self, success: bool) -> Duration {
        let duration = self.start.elapsed();
        if let Some(monitor) = self.monitor {
            monitor.record(&self.operation_name, duration, success);
        }
        duration
    }

    /// Get elapsed time without stopping
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

/// Log a message with the specified level
pub fn log(level: LogLevel, component: &str, message: &str) {
    eprintln!("[{}] [{}] {}", level.as_str(), component, message);
}

/// Log an error message
pub fn log_error(component: &str, message: &str) {
    log(LogLevel::Error, component, message);
}

/// Log a warning message
pub fn log_warn(component: &str, message: &str) {
    log(LogLevel::Warn, component, message);
}

/// Log an info message
pub fn log_info(component: &str, message: &str) {
    log(LogLevel::Info, component, message);
}

/// Log a debug message
pub fn log_debug(component: &str, message: &str) {
    log(LogLevel::Debug, component, message);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new();
        assert!(monitor.get_all_metrics().is_empty());
    }

    #[test]
    fn test_record_metrics() {
        let monitor = PerformanceMonitor::new();
        monitor.record("test_op", Duration::from_millis(100), true);
        
        let metrics = monitor.get_all_metrics();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].name, "test_op");
        assert!(metrics[0].success);
    }

    #[test]
    fn test_average_duration() {
        let monitor = PerformanceMonitor::new();
        monitor.record("test_op", Duration::from_millis(100), true);
        monitor.record("test_op", Duration::from_millis(200), true);
        
        let avg = monitor.get_average_duration("test_op").unwrap();
        assert_eq!(avg, Duration::from_millis(150));
    }

    #[test]
    fn test_success_rate() {
        let monitor = PerformanceMonitor::new();
        monitor.record("test_op", Duration::from_millis(100), true);
        monitor.record("test_op", Duration::from_millis(100), false);
        
        let rate = monitor.get_success_rate("test_op").unwrap();
        assert!((rate - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_timer() {
        let timer = Timer::start("test_op");
        thread::sleep(Duration::from_millis(10));
        let duration = timer.stop(true);
        assert!(duration >= Duration::from_millis(10));
    }

    #[test]
    fn test_timer_with_monitor() {
        let monitor = Arc::new(PerformanceMonitor::new());
        let timer = Timer::start_with_monitor("test_op", monitor.clone());
        thread::sleep(Duration::from_millis(10));
        timer.stop(true);
        
        let metrics = monitor.get_all_metrics();
        assert_eq!(metrics.len(), 1);
    }

    #[test]
    fn test_generate_summary() {
        let monitor = PerformanceMonitor::new();
        monitor.record("op1", Duration::from_millis(100), true);
        monitor.record("op1", Duration::from_millis(200), true);
        monitor.record("op2", Duration::from_millis(50), false);
        
        let summary = monitor.generate_summary();
        assert_eq!(summary.total_operations, 3);
        assert_eq!(summary.operation_stats.len(), 2);
    }

    #[test]
    fn test_log_levels() {
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
        assert_eq!(LogLevel::Warn.as_str(), "WARN");
        assert_eq!(LogLevel::Info.as_str(), "INFO");
        assert_eq!(LogLevel::Debug.as_str(), "DEBUG");
        assert_eq!(LogLevel::Trace.as_str(), "TRACE");
    }
}
