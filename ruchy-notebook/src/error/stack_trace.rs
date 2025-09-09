use serde::{Deserialize, Serialize};
use crate::vm::OpCode;

/// Stack trace for debugging VM execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackTrace {
    pub frames: Vec<StackFrame>,
}

/// A single frame in the stack trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub function_name: Option<String>,
    pub instruction_pointer: usize,
    pub opcode: Option<OpCode>,
    pub source_location: Option<SourceLocation>,
    pub locals: Vec<(String, String)>, // name -> value representation
}

/// Source location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: Option<String>,
    pub line: usize,
    pub column: usize,
    pub source_line: Option<String>,
}

impl StackTrace {
    /// Create a new empty stack trace
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
        }
    }
    
    /// Add a frame to the stack trace
    pub fn push_frame(&mut self, frame: StackFrame) {
        self.frames.push(frame);
    }
    
    /// Remove the top frame
    pub fn pop_frame(&mut self) -> Option<StackFrame> {
        self.frames.pop()
    }
    
    /// Get the current frame (top of stack)
    pub fn current_frame(&self) -> Option<&StackFrame> {
        self.frames.last()
    }
    
    /// Format the stack trace for display
    pub fn format(&self) -> String {
        if self.frames.is_empty() {
            return "No stack trace available".to_string();
        }
        
        let mut result = String::from("Stack trace:\n");
        
        for (i, frame) in self.frames.iter().rev().enumerate() {
            result.push_str(&format!("  {}: ", i));
            
            if let Some(name) = &frame.function_name {
                result.push_str(&format!("in function '{}'", name));
            } else {
                result.push_str("in <anonymous>");
            }
            
            if let Some(loc) = &frame.source_location {
                result.push_str(&format!(" at line {}", loc.line));
                if let Some(file) = &loc.file {
                    result.push_str(&format!(" in {}", file));
                }
                
                if let Some(source_line) = &loc.source_line {
                    result.push_str(&format!("\n    {}", source_line));
                    if loc.column > 0 {
                        result.push_str(&format!("\n    {}^", " ".repeat(loc.column - 1)));
                    }
                }
            }
            
            if let Some(opcode) = &frame.opcode {
                result.push_str(&format!(" [instruction: {:?}]", opcode));
            }
            
            result.push('\n');
        }
        
        result
    }
    
    /// Get the deepest source location
    pub fn deepest_location(&self) -> Option<&SourceLocation> {
        self.frames.iter()
            .filter_map(|f| f.source_location.as_ref())
            .last()
    }
    
    /// Clear the stack trace
    pub fn clear(&mut self) {
        self.frames.clear();
    }
}

impl StackFrame {
    /// Create a new stack frame
    pub fn new(instruction_pointer: usize) -> Self {
        Self {
            function_name: None,
            instruction_pointer,
            opcode: None,
            source_location: None,
            locals: Vec::new(),
        }
    }
    
    /// Set the function name
    pub fn with_function(mut self, name: impl Into<String>) -> Self {
        self.function_name = Some(name.into());
        self
    }
    
    /// Set the opcode
    pub fn with_opcode(mut self, opcode: OpCode) -> Self {
        self.opcode = Some(opcode);
        self
    }
    
    /// Set the source location
    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.source_location = Some(location);
        self
    }
    
    /// Add a local variable
    pub fn add_local(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.locals.push((name.into(), value.into()));
    }
    
    /// Get local variables as formatted string
    pub fn format_locals(&self) -> String {
        if self.locals.is_empty() {
            return "No local variables".to_string();
        }
        
        let mut result = String::from("Local variables:\n");
        for (name, value) in &self.locals {
            result.push_str(&format!("  {}: {}\n", name, value));
        }
        result
    }
}

impl SourceLocation {
    /// Create a new source location
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            file: None,
            line,
            column,
            source_line: None,
        }
    }
    
    /// Set the file name
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }
    
    /// Set the source line content
    pub fn with_source_line(mut self, source_line: impl Into<String>) -> Self {
        self.source_line = Some(source_line.into());
        self
    }
}

impl Default for StackTrace {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait for objects that can capture stack traces
pub trait StackCapture {
    /// Capture the current stack state
    fn capture_stack(&self) -> StackTrace;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stack_trace_creation() {
        let mut trace = StackTrace::new();
        assert!(trace.frames.is_empty());
        
        let frame = StackFrame::new(100)
            .with_function("main")
            .with_opcode(OpCode::Add)
            .with_location(SourceLocation::new(10, 5).with_file("test.ruchy"));
        
        trace.push_frame(frame);
        assert_eq!(trace.frames.len(), 1);
        assert_eq!(trace.current_frame().unwrap().instruction_pointer, 100);
    }
    
    #[test]
    fn test_stack_trace_formatting() {
        let mut trace = StackTrace::new();
        
        let frame1 = StackFrame::new(0)
            .with_function("helper")
            .with_location(
                SourceLocation::new(5, 10)
                    .with_file("helper.ruchy")
                    .with_source_line("let x = a + b")
            );
        
        let frame2 = StackFrame::new(50)
            .with_function("main")
            .with_opcode(OpCode::Call)
            .with_location(SourceLocation::new(20, 1).with_file("main.ruchy"));
        
        trace.push_frame(frame1);
        trace.push_frame(frame2);
        
        let formatted = trace.format();
        assert!(formatted.contains("Stack trace:"));
        assert!(formatted.contains("function 'main'"));
        assert!(formatted.contains("function 'helper'"));
        assert!(formatted.contains("line 20"));
        assert!(formatted.contains("line 5"));
    }
    
    #[test]
    fn test_local_variables() {
        let mut frame = StackFrame::new(10);
        frame.add_local("x", "42");
        frame.add_local("name", "\"Alice\"");
        
        let formatted = frame.format_locals();
        assert!(formatted.contains("x: 42"));
        assert!(formatted.contains("name: \"Alice\""));
    }
    
    #[test]
    fn test_deepest_location() {
        let mut trace = StackTrace::new();
        
        let frame1 = StackFrame::new(0)
            .with_location(SourceLocation::new(1, 1));
        let frame2 = StackFrame::new(10);  // No location
        let frame3 = StackFrame::new(20)
            .with_location(SourceLocation::new(15, 3));
        
        trace.push_frame(frame1);
        trace.push_frame(frame2);
        trace.push_frame(frame3);
        
        let deepest = trace.deepest_location().unwrap();
        assert_eq!(deepest.line, 15);
        assert_eq!(deepest.column, 3);
    }
}