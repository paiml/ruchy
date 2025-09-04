//! Debug and introspection module for REPL
//! Provides debugging utilities and runtime introspection

use std::collections::HashMap;
use std::fmt::Write;
use std::time::Instant;

use crate::runtime::repl::Value;
use crate::frontend::ast::{Expr, ExprKind};

/// Debug event types
#[derive(Debug, Clone)]
pub enum DebugEvent {
    /// Expression evaluation started
    EvalStart { expr: String, depth: usize },
    /// Expression evaluation completed
    EvalEnd { result: String, duration_us: u64 },
    /// Variable binding
    VarBind { name: String, value: String },
    /// Function call
    FuncCall { name: String, args: Vec<String> },
    /// Error occurred
    Error { message: String, context: String },
    /// Breakpoint hit
    Breakpoint { id: usize, location: String },
}

/// Debug mode configuration
#[derive(Debug, Clone)]
pub struct DebugConfig {
    /// Enable trace output
    pub trace_enabled: bool,
    /// Enable profiling
    pub profile_enabled: bool,
    /// Enable breakpoints
    pub breakpoints_enabled: bool,
    /// Maximum trace depth
    pub max_trace_depth: usize,
    /// Collect memory stats
    pub memory_tracking: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            trace_enabled: false,
            profile_enabled: false,
            breakpoints_enabled: false,
            max_trace_depth: 10,
            memory_tracking: false,
        }
    }
}

/// Debug manager for REPL
pub struct DebugManager {
    /// Configuration
    config: DebugConfig,
    /// Debug event log
    events: Vec<DebugEvent>,
    /// Breakpoints
    breakpoints: HashMap<usize, Breakpoint>,
    /// Performance profiler
    profiler: Profiler,
    /// Memory tracker
    memory_tracker: MemoryTracker,
    /// Next breakpoint ID
    next_breakpoint_id: usize,
}

impl DebugManager {
    /// Create new debug manager (complexity: 2)
    pub fn new(config: DebugConfig) -> Self {
        Self {
            config,
            events: Vec::new(),
            breakpoints: HashMap::new(),
            profiler: Profiler::new(),
            memory_tracker: MemoryTracker::new(),
            next_breakpoint_id: 1,
        }
    }

    /// Record debug event (complexity: 4)
    pub fn record_event(&mut self, event: DebugEvent) {
        if !self.is_enabled() {
            return;
        }
        
        // Limit event log size
        if self.events.len() > 10000 {
            self.events.drain(0..5000);
        }
        
        self.events.push(event);
    }

    /// Start evaluation trace (complexity: 5)
    pub fn trace_eval_start(&mut self, expr: &Expr, depth: usize) -> Option<TracingContext> {
        if !self.config.trace_enabled || depth > self.config.max_trace_depth {
            return None;
        }
        
        let expr_str = self.format_expr_short(expr);
        self.record_event(DebugEvent::EvalStart { 
            expr: expr_str.clone(), 
            depth 
        });
        
        Some(TracingContext {
            start_time: Instant::now(),
            expr_str,
            depth,
        })
    }

    /// End evaluation trace (complexity: 4)
    pub fn trace_eval_end(&mut self, ctx: TracingContext, result: &Value) {
        if !self.config.trace_enabled {
            return;
        }
        
        let duration_us = ctx.start_time.elapsed().as_micros() as u64;
        self.record_event(DebugEvent::EvalEnd {
            result: self.format_value_short(result),
            duration_us,
        });
        
        if self.config.profile_enabled {
            self.profiler.record_call(&ctx.expr_str, duration_us);
        }
    }

    /// Trace variable binding (complexity: 3)
    pub fn trace_binding(&mut self, name: &str, value: &Value) {
        if !self.config.trace_enabled {
            return;
        }
        
        self.record_event(DebugEvent::VarBind {
            name: name.to_string(),
            value: self.format_value_short(value),
        });
    }

    /// Trace function call (complexity: 4)
    pub fn trace_function_call(&mut self, name: &str, args: &[Value]) {
        if !self.config.trace_enabled {
            return;
        }
        
        let arg_strs: Vec<String> = args.iter()
            .map(|v| self.format_value_short(v))
            .collect();
        
        self.record_event(DebugEvent::FuncCall {
            name: name.to_string(),
            args: arg_strs,
        });
    }

    /// Add breakpoint (complexity: 3)
    pub fn add_breakpoint(&mut self, location: String, condition: Option<String>) -> usize {
        let id = self.next_breakpoint_id;
        self.next_breakpoint_id += 1;
        
        self.breakpoints.insert(id, Breakpoint {
            id,
            location,
            condition,
            hit_count: 0,
            enabled: true,
        });
        
        id
    }

    /// Remove breakpoint (complexity: 2)
    pub fn remove_breakpoint(&mut self, id: usize) -> bool {
        self.breakpoints.remove(&id).is_some()
    }

    /// Check breakpoint (complexity: 6)
    pub fn check_breakpoint(&mut self, location: &str) -> Option<BreakpointHit> {
        if !self.config.breakpoints_enabled {
            return None;
        }
        
        for bp in self.breakpoints.values_mut() {
            if bp.enabled && bp.matches_location(location) {
                bp.hit_count += 1;
                
                self.record_event(DebugEvent::Breakpoint {
                    id: bp.id,
                    location: location.to_string(),
                });
                
                return Some(BreakpointHit {
                    id: bp.id,
                    location: location.to_string(),
                    hit_count: bp.hit_count,
                });
            }
        }
        
        None
    }

    /// Get profiling report (complexity: 3)
    pub fn get_profile_report(&self) -> String {
        if !self.config.profile_enabled {
            return "Profiling not enabled".to_string();
        }
        
        self.profiler.generate_report()
    }

    /// Get memory report (complexity: 3)
    pub fn get_memory_report(&self) -> String {
        if !self.config.memory_tracking {
            return "Memory tracking not enabled".to_string();
        }
        
        self.memory_tracker.generate_report()
    }

    /// Get trace output (complexity: 5)
    pub fn get_trace(&self, limit: Option<usize>) -> String {
        let mut output = String::new();
        let events_to_show = limit.unwrap_or(100).min(self.events.len());
        let start = self.events.len().saturating_sub(events_to_show);
        
        for event in &self.events[start..] {
            writeln!(output, "{}", self.format_event(event)).unwrap();
        }
        
        output
    }

    /// Format debug event (complexity: 6)
    fn format_event(&self, event: &DebugEvent) -> String {
        match event {
            DebugEvent::EvalStart { expr, depth } => {
                format!("{}→ eval: {}", "  ".repeat(*depth), expr)
            }
            DebugEvent::EvalEnd { result, duration_us } => {
                format!("  = {} ({} μs)", result, duration_us)
            }
            DebugEvent::VarBind { name, value } => {
                format!("  bind: {} = {}", name, value)
            }
            DebugEvent::FuncCall { name, args } => {
                format!("  call: {}({})", name, args.join(", "))
            }
            DebugEvent::Error { message, context } => {
                format!("  ERROR: {} (context: {})", message, context)
            }
            DebugEvent::Breakpoint { id, location } => {
                format!("  BREAKPOINT #{} at {}", id, location)
            }
        }
    }

    /// Format expression for display (complexity: 8)
    fn format_expr_short(&self, expr: &Expr) -> String {
        match &expr.kind {
            ExprKind::Literal(lit) => format!("{:?}", lit),
            ExprKind::Identifier(name) => name.clone(),
            ExprKind::Binary { op, .. } => format!("Binary({:?})", op),
            ExprKind::Unary { op, .. } => format!("Unary({:?})", op),
            ExprKind::Call { .. } => "Call".to_string(),
            ExprKind::List(_) => "List".to_string(),
            ExprKind::If { .. } => "If".to_string(),
            ExprKind::For { .. } => "For".to_string(),
            ExprKind::Function { .. } => "Function".to_string(),
            _ => "Expr".to_string(),
        }
    }

    /// Format value for display (complexity: 7)
    fn format_value_short(&self, value: &Value) -> String {
        match value {
            Value::Int(n) => n.to_string(),
            Value::Float(f) => format!("{:.2}", f),
            Value::String(s) if s.len() <= 20 => format!("\"{}\"", s),
            Value::String(s) => format!("\"{}...\"", &s[..17]),
            Value::Bool(b) => b.to_string(),
            Value::List(items) => format!("[{} items]", items.len()),
            Value::Unit => "()".to_string(),
            Value::Nil => "nil".to_string(),
            _ => format!("<{}>", self.value_type_name(value)),
        }
    }

    /// Get value type name (complexity: 3)
    fn value_type_name(&self, value: &Value) -> &str {
        match value {
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::String(_) => "String",
            Value::Bool(_) => "Bool",
            Value::List(_) => "List",
            Value::Tuple(_) => "Tuple",
            Value::HashMap(_) => "HashMap",
            Value::HashSet(_) => "HashSet",
            Value::Function { .. } => "Function",
            Value::Lambda { .. } => "Lambda",
            Value::Unit => "Unit",
            Value::Nil => "Nil",
            _ => "Unknown",
        }
    }

    /// Check if debugging is enabled (complexity: 2)
    pub fn is_enabled(&self) -> bool {
        self.config.trace_enabled || 
        self.config.profile_enabled || 
        self.config.breakpoints_enabled
    }

    /// Clear all debug data (complexity: 2)
    pub fn clear(&mut self) {
        self.events.clear();
        self.profiler.clear();
        self.memory_tracker.clear();
    }

    /// Get statistics (complexity: 3)
    pub fn get_stats(&self) -> DebugStats {
        DebugStats {
            total_events: self.events.len(),
            breakpoint_count: self.breakpoints.len(),
            profile_entries: self.profiler.entry_count(),
            memory_allocations: self.memory_tracker.allocation_count(),
        }
    }
}

/// Breakpoint definition
#[derive(Debug, Clone)]
struct Breakpoint {
    id: usize,
    location: String,
    condition: Option<String>,
    hit_count: usize,
    enabled: bool,
}

impl Breakpoint {
    /// Check if breakpoint matches location (complexity: 2)
    fn matches_location(&self, location: &str) -> bool {
        self.location == location || location.contains(&self.location)
    }
}

/// Breakpoint hit information
#[derive(Debug, Clone)]
pub struct BreakpointHit {
    pub id: usize,
    pub location: String,
    pub hit_count: usize,
}

/// Tracing context for performance measurement
pub struct TracingContext {
    start_time: Instant,
    expr_str: String,
    depth: usize,
}

/// Performance profiler
struct Profiler {
    /// Call statistics by function
    call_stats: HashMap<String, CallStats>,
}

impl Profiler {
    /// Create new profiler (complexity: 1)
    fn new() -> Self {
        Self {
            call_stats: HashMap::new(),
        }
    }

    /// Record function call (complexity: 3)
    fn record_call(&mut self, name: &str, duration_us: u64) {
        let stats = self.call_stats.entry(name.to_string())
            .or_insert(CallStats::new());
        stats.record(duration_us);
    }

    /// Generate profiling report (complexity: 6)
    fn generate_report(&self) -> String {
        let mut output = String::new();
        writeln!(output, "Performance Profile:").unwrap();
        
        // Sort by total time
        let mut entries: Vec<_> = self.call_stats.iter().collect();
        entries.sort_by(|a, b| b.1.total_time.cmp(&a.1.total_time));
        
        for (name, stats) in entries.iter().take(20) {
            writeln!(
                output,
                "  {}: {} calls, {:.2}ms total, {:.2}μs avg",
                name,
                stats.count,
                stats.total_time as f64 / 1000.0,
                stats.average_time()
            ).unwrap();
        }
        
        output
    }

    /// Get entry count (complexity: 1)
    fn entry_count(&self) -> usize {
        self.call_stats.len()
    }

    /// Clear profiler data (complexity: 1)
    fn clear(&mut self) {
        self.call_stats.clear();
    }
}

/// Call statistics
#[derive(Debug, Clone)]
struct CallStats {
    count: usize,
    total_time: u64,
    min_time: u64,
    max_time: u64,
}

impl CallStats {
    /// Create new stats (complexity: 1)
    fn new() -> Self {
        Self {
            count: 0,
            total_time: 0,
            min_time: u64::MAX,
            max_time: 0,
        }
    }

    /// Record call (complexity: 3)
    fn record(&mut self, duration_us: u64) {
        self.count += 1;
        self.total_time += duration_us;
        self.min_time = self.min_time.min(duration_us);
        self.max_time = self.max_time.max(duration_us);
    }

    /// Get average time (complexity: 2)
    fn average_time(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.total_time as f64 / self.count as f64
        }
    }
}

/// Memory usage tracker
struct MemoryTracker {
    allocations: usize,
    deallocations: usize,
    peak_usage: usize,
    current_usage: usize,
}

impl MemoryTracker {
    /// Create new tracker (complexity: 1)
    fn new() -> Self {
        Self {
            allocations: 0,
            deallocations: 0,
            peak_usage: 0,
            current_usage: 0,
        }
    }

    /// Record allocation (complexity: 3)
    fn record_allocation(&mut self, size: usize) {
        self.allocations += 1;
        self.current_usage += size;
        self.peak_usage = self.peak_usage.max(self.current_usage);
    }

    /// Record deallocation (complexity: 2)
    fn record_deallocation(&mut self, size: usize) {
        self.deallocations += 1;
        self.current_usage = self.current_usage.saturating_sub(size);
    }

    /// Generate memory report (complexity: 3)
    fn generate_report(&self) -> String {
        format!(
            "Memory Usage:\n  \
             Allocations: {}\n  \
             Deallocations: {}\n  \
             Current: {} bytes\n  \
             Peak: {} bytes",
            self.allocations,
            self.deallocations,
            self.current_usage,
            self.peak_usage
        )
    }

    /// Get allocation count (complexity: 1)
    fn allocation_count(&self) -> usize {
        self.allocations
    }

    /// Clear tracker (complexity: 1)
    fn clear(&mut self) {
        self.allocations = 0;
        self.deallocations = 0;
        self.current_usage = 0;
        self.peak_usage = 0;
    }
}

/// Debug statistics
#[derive(Debug, Clone)]
pub struct DebugStats {
    pub total_events: usize,
    pub breakpoint_count: usize,
    pub profile_entries: usize,
    pub memory_allocations: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::Literal;

    #[test]
    fn test_debug_manager_creation() {
        let mgr = DebugManager::new(DebugConfig::default());
        assert!(!mgr.is_enabled());
    }

    #[test]
    fn test_breakpoint_management() {
        let mut mgr = DebugManager::new(DebugConfig {
            breakpoints_enabled: true,
            ..Default::default()
        });
        
        let id = mgr.add_breakpoint("test.rs:10".to_string(), None);
        assert_eq!(id, 1);
        
        assert!(mgr.remove_breakpoint(id));
        assert!(!mgr.remove_breakpoint(id)); // Already removed
    }

    #[test]
    fn test_event_recording() {
        let mut mgr = DebugManager::new(DebugConfig {
            trace_enabled: true,
            ..Default::default()
        });
        
        mgr.record_event(DebugEvent::VarBind {
            name: "x".to_string(),
            value: "10".to_string(),
        });
        
        let stats = mgr.get_stats();
        assert_eq!(stats.total_events, 1);
    }

    #[test]
    fn test_profiler() {
        let mut profiler = Profiler::new();
        profiler.record_call("test_func", 1000);
        profiler.record_call("test_func", 2000);
        
        let report = profiler.generate_report();
        assert!(report.contains("test_func"));
        assert!(report.contains("2 calls"));
    }

    #[test]
    fn test_memory_tracker() {
        let mut tracker = MemoryTracker::new();
        tracker.record_allocation(1024);
        tracker.record_allocation(512);
        tracker.record_deallocation(512);
        
        assert_eq!(tracker.current_usage, 1024);
        assert_eq!(tracker.peak_usage, 1536);
        assert_eq!(tracker.allocations, 2);
    }
}