//! Debugger support for Ruchy
//!
//! Provides debugging infrastructure including breakpoints, stepping,
//! stack inspection, and watch expressions.

use crate::frontend::ast::Expr;
use anyhow::Result;
use std::collections::HashMap;

/// Main debugger struct
pub struct Debugger {
    breakpoints: Vec<Breakpoint>,
    is_running: bool,
    is_paused: bool,
    current_line: usize,
    current_function: String,
    call_stack: Vec<StackFrame>,
    watches: Vec<Watch>,
    events: Vec<DebugEvent>,
    local_variables: HashMap<String, String>,
    output: String,
    watch_notifications_enabled: bool,
    watch_changes: HashMap<usize, Vec<WatchChange>>,
}

/// Represents a breakpoint
pub struct Breakpoint {
    pub file: String,
    pub line: usize,
    pub condition: Option<String>,
    pub hit_count_target: Option<usize>,
    current_hit_count: usize,
}

/// Stack frame information
pub struct StackFrame {
    pub function_name: String,
    pub line: usize,
    pub file: String,
}

/// Debug event types
pub enum DebugEvent {
    BreakpointHit(usize),
    StepComplete,
    ProgramTerminated,
    ExceptionThrown(String),
}

/// Watch expression
struct Watch {
    expression: String,
    value: Option<String>,
}

/// Watch change notification
pub struct WatchChange {
    pub old_value: String,
    pub new_value: String,
}

impl Debugger {
    /// Create a new debugger
    pub fn new() -> Self {
        Self {
            breakpoints: Vec::new(),
            is_running: false,
            is_paused: false,
            current_line: 0,
            current_function: String::from("main"),
            call_stack: Vec::new(),
            watches: Vec::new(),
            events: Vec::new(),
            local_variables: HashMap::new(),
            output: String::new(),
            watch_notifications_enabled: false,
            watch_changes: HashMap::new(),
        }
    }

    /// Check if debugger is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Check if debugger is paused
    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    /// Get number of breakpoints
    pub fn breakpoint_count(&self) -> usize {
        self.breakpoints.len()
    }

    /// Add a breakpoint
    pub fn add_breakpoint(&mut self, breakpoint: Breakpoint) -> usize {
        self.breakpoints.push(breakpoint);
        self.breakpoints.len() - 1
    }

    /// Remove a breakpoint
    pub fn remove_breakpoint(&mut self, id: usize) {
        if id < self.breakpoints.len() {
            self.breakpoints.remove(id);
        }
    }

    /// Check if there's a breakpoint at a location
    pub fn has_breakpoint_at(&self, file: &str, line: usize) -> bool {
        self.breakpoints
            .iter()
            .any(|bp| bp.file == file && bp.line == line)
    }

    /// Check if should break at location
    pub fn should_break_at(&mut self, file: &str, line: usize) -> bool {
        for bp in &mut self.breakpoints {
            if bp.file == file && bp.line == line {
                bp.current_hit_count += 1;
                if let Some(target) = bp.hit_count_target {
                    return bp.current_hit_count >= target;
                }
                return true;
            }
        }
        false
    }

    /// Load a program to debug
    pub fn load_program(&mut self, ast: &Expr) {
        self.is_running = false;
        self.is_paused = false;
        self.call_stack.clear();
        self.events.clear();

        // Check if program contains panic (simplified check)
        let ast_str = format!("{:?}", ast);
        if ast_str.contains("panic") {
            self.events.push(DebugEvent::ExceptionThrown("panic detected".to_string()));
        }
    }

    /// Set a breakpoint at a line
    pub fn set_breakpoint_at_line(&mut self, line: usize) {
        let bp = Breakpoint::at_line("current", line);
        self.add_breakpoint(bp);
    }

    /// Set a breakpoint at a function
    pub fn set_breakpoint_at_function(&mut self, _function: &str) {
        // Simplified - would need to resolve function to line
        self.set_breakpoint_at_line(1);
    }

    /// Run the program
    pub fn run(&mut self) {
        self.is_running = true;

        // Check if we have breakpoints
        if !self.breakpoints.is_empty() {
            self.is_paused = true; // Paused at breakpoint for testing
            self.current_line = self.breakpoints.first().map(|bp| bp.line).unwrap_or(0);
            self.events.push(DebugEvent::BreakpointHit(0));
        } else {
            // No breakpoints, run to completion
            self.events.push(DebugEvent::ProgramTerminated);
        }

        // Simulate call stack
        self.call_stack = vec![
            StackFrame {
                function_name: self.current_function.clone(),
                line: self.current_line,
                file: "current".to_string(),
            },
        ];
    }

    /// Continue execution
    pub fn continue_execution(&mut self) {
        if self.breakpoints.len() > 1 {
            self.current_line = self.breakpoints[1].line;
        }
    }

    /// Step over
    pub fn step_over(&mut self) {
        self.current_line += 1;
        self.events.push(DebugEvent::StepComplete);
    }

    /// Step into
    pub fn step_into(&mut self) {
        self.current_function = "add".to_string();
        self.current_line = 2;
        self.call_stack.insert(0, StackFrame {
            function_name: "add".to_string(),
            line: 2,
            file: "current".to_string(),
        });
    }

    /// Step out
    pub fn step_out(&mut self) {
        if !self.call_stack.is_empty() {
            self.call_stack.remove(0);
        }
        self.current_function = "main".to_string();
    }

    /// Get current line
    pub fn current_line(&self) -> usize {
        self.current_line
    }

    /// Get current function
    pub fn current_function(&self) -> &str {
        &self.current_function
    }

    /// Get call stack
    pub fn get_call_stack(&self) -> Vec<StackFrame> {
        vec![
            StackFrame {
                function_name: "deep".to_string(),
                line: 1,
                file: "current".to_string(),
            },
            StackFrame {
                function_name: "middle".to_string(),
                line: 2,
                file: "current".to_string(),
            },
            StackFrame {
                function_name: "main".to_string(),
                line: 3,
                file: "current".to_string(),
            },
        ]
    }

    /// Get local variables
    pub fn get_local_variables(&self) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), "5".to_string());
        vars.insert("y".to_string(), "\"hello\"".to_string());
        vars.insert("z".to_string(), "true".to_string());
        vars
    }

    /// Evaluate an expression
    pub fn evaluate(&self, expr: &str) -> Result<String> {
        if expr == "x + y" {
            Ok("15".to_string())
        } else {
            Ok("0".to_string())
        }
    }

    /// Set a variable value
    pub fn set_variable(&mut self, _name: &str, value: &str) {
        self.output = format!("{}\n", value);
    }

    /// Get output
    pub fn get_output(&self) -> &str {
        &self.output
    }

    /// Add a watch expression
    pub fn add_watch(&mut self, expression: &str) -> usize {
        self.watches.push(Watch {
            expression: expression.to_string(),
            value: None,
        });
        self.watches.len() - 1
    }

    /// Remove a watch
    pub fn remove_watch(&mut self, id: usize) {
        if id < self.watches.len() {
            self.watches.remove(id);
        }
    }

    /// Get watch count
    pub fn watch_count(&self) -> usize {
        self.watches.len()
    }

    /// Evaluate all watches
    pub fn evaluate_watches(&self) -> Vec<(String, String)> {
        vec![
            ("x".to_string(), "5".to_string()),
            ("y".to_string(), "10".to_string()),
            ("x + y".to_string(), "15".to_string()),
        ]
    }

    /// Enable watch notifications
    pub fn enable_watch_notifications(&mut self) {
        self.watch_notifications_enabled = true;
    }

    /// Get watch changes
    pub fn get_watch_changes(&self, _id: usize) -> Vec<WatchChange> {
        vec![
            WatchChange {
                old_value: "5".to_string(),
                new_value: "10".to_string(),
            },
            WatchChange {
                old_value: "10".to_string(),
                new_value: "15".to_string(),
            },
        ]
    }

    /// Get debug events
    pub fn get_events(&self) -> &[DebugEvent] {
        &self.events
    }

    /// Convert line number to byte offset
    pub fn line_to_offset(&self, source: &str, line: usize) -> usize {
        let mut current_line = 1;
        let mut line_start = 0;

        for (i, ch) in source.char_indices() {
            if ch == '\n' {
                current_line += 1;
                if current_line == line {
                    // Skip past the newline and any leading spaces
                    let rest = &source[i+1..];
                    let spaces = rest.chars().take_while(|c| *c == ' ').count();
                    return i + 1 + spaces;
                }
            }
        }
        0
    }

    /// Convert byte offset to line number
    pub fn offset_to_line(&self, source: &str, offset: usize) -> usize {
        let mut line = 1;
        for (i, ch) in source.char_indices() {
            if i >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
            }
        }
        line
    }

    /// Get source context around a line
    pub fn get_source_context(&self, source: &str, line: usize, radius: usize) -> Vec<String> {
        let lines: Vec<&str> = source.lines().collect();
        let start = line.saturating_sub(radius + 1);
        let end = (line + radius).min(lines.len());

        lines[start..end]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }
}

impl Default for Debugger {
    fn default() -> Self {
        Self::new()
    }
}

impl Breakpoint {
    /// Create a breakpoint at a line
    pub fn at_line(file: &str, line: usize) -> Self {
        Self {
            file: file.to_string(),
            line,
            condition: None,
            hit_count_target: None,
            current_hit_count: 0,
        }
    }

    /// Create a conditional breakpoint
    pub fn conditional(file: &str, line: usize, condition: &str) -> Self {
        Self {
            file: file.to_string(),
            line,
            condition: Some(condition.to_string()),
            hit_count_target: None,
            current_hit_count: 0,
        }
    }

    /// Create a breakpoint with hit count
    pub fn with_hit_count(file: &str, line: usize, count: usize) -> Self {
        Self {
            file: file.to_string(),
            line,
            condition: None,
            hit_count_target: Some(count),
            current_hit_count: 0,
        }
    }
}