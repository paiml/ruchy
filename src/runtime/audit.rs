//! REPL Audit Logging Integration
//!
//! Provides tamper-evident audit trails for REPL sessions using entrenar's
//! inference monitoring infrastructure.
//!
//! # Toyota Way: 現地現物 (Genchi Genbutsu)
//!
//! Every REPL command is traced. All decisions are auditable. Go and see.
//!
//! # Architecture
//!
//! - `ReplPath`: Decision path for REPL command execution
//! - `ReplAuditCollector`: Collector that bridges ruchy's session recording with entrenar
//! - `AuditedRepl`: REPL wrapper with automatic audit trail generation
//!
//! # Example
//!
//! ```ignore
//! use ruchy::runtime::audit::{AuditedRepl, ReplPath};
//! use entrenar::monitor::inference::HashChainCollector;
//!
//! let collector = HashChainCollector::<ReplPath>::new();
//! let mut repl = AuditedRepl::with_collector(collector);
//!
//! repl.eval("let x = 42")?;
//! let chain = repl.audit_chain();
//! assert!(chain.verify_chain().valid);
//! ```

pub use entrenar::monitor::inference::StreamFormat;
use entrenar::monitor::inference::{
    path::{DecisionPath, PathError},
    trace::DecisionTrace,
    HashChainCollector, RingCollector, StreamCollector, TraceCollector,
};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::time::Instant;

use super::replay::{EvalResult, InputMode};

// =============================================================================
// ReplPath - Decision path for REPL command execution
// =============================================================================

/// Decision path for REPL command execution.
///
/// Captures the context and reasoning behind each REPL evaluation,
/// enabling audit trails for educational assessment and debugging.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplPath {
    /// The input command text
    pub command: String,
    /// Input mode (interactive, paste, script, etc.)
    pub input_mode: ReplInputMode,
    /// Type of evaluation performed
    pub eval_type: EvalType,
    /// Whether evaluation succeeded
    pub success: bool,
    /// Error message if evaluation failed
    pub error: Option<String>,
    /// Output value as string
    pub output: String,
    /// Variables modified during evaluation
    pub bindings_changed: Vec<String>,
    /// Memory used (bytes)
    pub memory_bytes: usize,
    /// Execution time (nanoseconds)
    pub execution_ns: u64,
    /// AST depth of parsed expression
    pub ast_depth: usize,
    /// Confidence score (1.0 for successful evals, lower for errors)
    confidence_score: f32,
}

/// Input mode for REPL commands
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplInputMode {
    /// Interactive terminal input
    Interactive,
    /// Pasted multi-line input
    Paste,
    /// File-based input
    File,
    /// Script mode
    Script,
    /// Notebook cell
    Notebook,
}

impl From<InputMode> for ReplInputMode {
    fn from(mode: InputMode) -> Self {
        match mode {
            InputMode::Interactive => Self::Interactive,
            InputMode::Paste => Self::Paste,
            InputMode::File => Self::File,
            InputMode::Script => Self::Script,
        }
    }
}

impl From<ReplInputMode> for InputMode {
    fn from(mode: ReplInputMode) -> Self {
        match mode {
            ReplInputMode::Interactive => Self::Interactive,
            ReplInputMode::Paste => Self::Paste,
            ReplInputMode::File => Self::File,
            ReplInputMode::Script => Self::Script,
            ReplInputMode::Notebook => Self::Script, // Map notebook to script
        }
    }
}

/// Type of evaluation performed
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvalType {
    /// Expression evaluation
    Expression,
    /// Statement execution
    Statement,
    /// Variable binding
    Binding,
    /// Function definition
    FunctionDef,
    /// Control flow (if, loop, match)
    ControlFlow,
    /// Magic command (:help, :type, etc.)
    MagicCommand,
    /// Import statement
    Import,
    /// Type definition
    TypeDef,
}

impl ReplPath {
    /// Create a new REPL path for a successful evaluation
    pub fn success(
        command: String,
        input_mode: ReplInputMode,
        eval_type: EvalType,
        output: String,
    ) -> Self {
        Self {
            command,
            input_mode,
            eval_type,
            success: true,
            error: None,
            output,
            bindings_changed: Vec::new(),
            memory_bytes: 0,
            execution_ns: 0,
            ast_depth: 0,
            confidence_score: 1.0,
        }
    }

    /// Create a new REPL path for a failed evaluation
    pub fn failure(
        command: String,
        input_mode: ReplInputMode,
        eval_type: EvalType,
        error: String,
    ) -> Self {
        Self {
            command,
            input_mode,
            eval_type,
            success: false,
            error: Some(error),
            output: String::new(),
            bindings_changed: Vec::new(),
            memory_bytes: 0,
            execution_ns: 0,
            ast_depth: 0,
            confidence_score: 0.0,
        }
    }

    /// Set the bindings changed during evaluation
    pub fn with_bindings(mut self, bindings: Vec<String>) -> Self {
        self.bindings_changed = bindings;
        self
    }

    /// Set memory usage
    pub fn with_memory(mut self, bytes: usize) -> Self {
        self.memory_bytes = bytes;
        self
    }

    /// Set execution time
    pub fn with_execution_time(mut self, ns: u64) -> Self {
        self.execution_ns = ns;
        self
    }

    /// Set AST depth
    pub fn with_ast_depth(mut self, depth: usize) -> Self {
        self.ast_depth = depth;
        self
    }

    /// Infer eval type from command text
    pub fn infer_eval_type(command: &str) -> EvalType {
        let trimmed = command.trim();
        if trimmed.starts_with(':') || trimmed.starts_with('%') {
            EvalType::MagicCommand
        } else if trimmed.starts_with("let ") || trimmed.starts_with("let mut ") {
            EvalType::Binding
        } else if trimmed.starts_with("fn ") {
            EvalType::FunctionDef
        } else if trimmed.starts_with("if ")
            || trimmed.starts_with("match ")
            || trimmed.starts_with("for ")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("loop ")
        {
            EvalType::ControlFlow
        } else if trimmed.starts_with("import ") || trimmed.starts_with("use ") {
            EvalType::Import
        } else if trimmed.starts_with("struct ")
            || trimmed.starts_with("enum ")
            || trimmed.starts_with("type ")
        {
            EvalType::TypeDef
        } else if trimmed.ends_with(';') {
            EvalType::Statement
        } else {
            EvalType::Expression
        }
    }
}

impl DecisionPath for ReplPath {
    fn explain(&self) -> String {
        let mut explanation = format!(
            "REPL Command: {:?} ({:?})\n",
            self.eval_type, self.input_mode
        );
        explanation.push_str(&format!("Command: {}\n", self.command));

        if self.success {
            explanation.push_str(&format!("Result: {}\n", self.output));
        } else if let Some(err) = &self.error {
            explanation.push_str(&format!("Error: {err}\n"));
        }

        if !self.bindings_changed.is_empty() {
            explanation.push_str(&format!(
                "Bindings changed: {}\n",
                self.bindings_changed.join(", ")
            ));
        }

        explanation.push_str(&format!(
            "Execution: {}ns, Memory: {} bytes, AST depth: {}\n",
            self.execution_ns, self.memory_bytes, self.ast_depth
        ));

        explanation
    }

    fn feature_contributions(&self) -> &[f32] {
        // REPL doesn't have numerical features in the traditional sense
        &[]
    }

    fn confidence(&self) -> f32 {
        self.confidence_score
    }

    fn to_bytes(&self) -> Vec<u8> {
        // Version 1 format:
        // [0]: version (1)
        // [1..5]: command_len (u32 LE)
        // [5..5+cmd_len]: command bytes
        // [5+cmd_len]: input_mode (u8)
        // [6+cmd_len]: eval_type (u8)
        // [7+cmd_len]: success (u8)
        // [8+cmd_len..12+cmd_len]: error_len (u32 LE)
        // [...]: error bytes if present
        // [...]: output_len (u32 LE) + output bytes
        // [...]: n_bindings (u32 LE) + bindings
        // [...]: memory_bytes (u64 LE)
        // [...]: execution_ns (u64 LE)
        // [...]: ast_depth (u32 LE)
        // [...]: confidence (f32 LE)

        let mut bytes = Vec::new();
        bytes.push(1); // version

        // Command
        let cmd_bytes = self.command.as_bytes();
        bytes.extend_from_slice(&(cmd_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(cmd_bytes);

        // Input mode
        bytes.push(match self.input_mode {
            ReplInputMode::Interactive => 0,
            ReplInputMode::Paste => 1,
            ReplInputMode::File => 2,
            ReplInputMode::Script => 3,
            ReplInputMode::Notebook => 4,
        });

        // Eval type
        bytes.push(match self.eval_type {
            EvalType::Expression => 0,
            EvalType::Statement => 1,
            EvalType::Binding => 2,
            EvalType::FunctionDef => 3,
            EvalType::ControlFlow => 4,
            EvalType::MagicCommand => 5,
            EvalType::Import => 6,
            EvalType::TypeDef => 7,
        });

        // Success
        bytes.push(u8::from(self.success));

        // Error
        if let Some(err) = &self.error {
            let err_bytes = err.as_bytes();
            bytes.extend_from_slice(&(err_bytes.len() as u32).to_le_bytes());
            bytes.extend_from_slice(err_bytes);
        } else {
            bytes.extend_from_slice(&0u32.to_le_bytes());
        }

        // Output
        let out_bytes = self.output.as_bytes();
        bytes.extend_from_slice(&(out_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(out_bytes);

        // Bindings changed
        bytes.extend_from_slice(&(self.bindings_changed.len() as u32).to_le_bytes());
        for binding in &self.bindings_changed {
            let binding_bytes = binding.as_bytes();
            bytes.extend_from_slice(&(binding_bytes.len() as u32).to_le_bytes());
            bytes.extend_from_slice(binding_bytes);
        }

        // Memory, execution time, AST depth, confidence
        bytes.extend_from_slice(&(self.memory_bytes as u64).to_le_bytes());
        bytes.extend_from_slice(&self.execution_ns.to_le_bytes());
        bytes.extend_from_slice(&(self.ast_depth as u32).to_le_bytes());
        bytes.extend_from_slice(&self.confidence_score.to_le_bytes());

        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, PathError> {
        if bytes.is_empty() {
            return Err(PathError::InsufficientData {
                expected: 1,
                actual: 0,
            });
        }

        let version = bytes[0];
        if version != 1 {
            return Err(PathError::VersionMismatch {
                expected: 1,
                actual: version,
            });
        }

        let mut offset = 1;

        // Helper to read u32
        let read_u32 = |bytes: &[u8], offset: &mut usize| -> Result<u32, PathError> {
            if *offset + 4 > bytes.len() {
                return Err(PathError::InsufficientData {
                    expected: *offset + 4,
                    actual: bytes.len(),
                });
            }
            let val = u32::from_le_bytes([
                bytes[*offset],
                bytes[*offset + 1],
                bytes[*offset + 2],
                bytes[*offset + 3],
            ]);
            *offset += 4;
            Ok(val)
        };

        // Helper to read string
        let read_string = |bytes: &[u8], offset: &mut usize| -> Result<String, PathError> {
            let len = read_u32(bytes, offset)? as usize;
            if *offset + len > bytes.len() {
                return Err(PathError::InsufficientData {
                    expected: *offset + len,
                    actual: bytes.len(),
                });
            }
            let s = String::from_utf8_lossy(&bytes[*offset..*offset + len]).to_string();
            *offset += len;
            Ok(s)
        };

        // Command
        let command = read_string(bytes, &mut offset)?;

        // Input mode
        if offset >= bytes.len() {
            return Err(PathError::InsufficientData {
                expected: offset + 1,
                actual: bytes.len(),
            });
        }
        let input_mode = match bytes[offset] {
            0 => ReplInputMode::Interactive,
            1 => ReplInputMode::Paste,
            2 => ReplInputMode::File,
            3 => ReplInputMode::Script,
            4 => ReplInputMode::Notebook,
            _ => {
                return Err(PathError::InvalidFormat(format!(
                    "Unknown input mode: {}",
                    bytes[offset]
                )))
            }
        };
        offset += 1;

        // Eval type
        if offset >= bytes.len() {
            return Err(PathError::InsufficientData {
                expected: offset + 1,
                actual: bytes.len(),
            });
        }
        let eval_type = match bytes[offset] {
            0 => EvalType::Expression,
            1 => EvalType::Statement,
            2 => EvalType::Binding,
            3 => EvalType::FunctionDef,
            4 => EvalType::ControlFlow,
            5 => EvalType::MagicCommand,
            6 => EvalType::Import,
            7 => EvalType::TypeDef,
            _ => {
                return Err(PathError::InvalidFormat(format!(
                    "Unknown eval type: {}",
                    bytes[offset]
                )))
            }
        };
        offset += 1;

        // Success
        if offset >= bytes.len() {
            return Err(PathError::InsufficientData {
                expected: offset + 1,
                actual: bytes.len(),
            });
        }
        let success = bytes[offset] != 0;
        offset += 1;

        // Error
        let error_len = read_u32(bytes, &mut offset)? as usize;
        let error = if error_len > 0 {
            if offset + error_len > bytes.len() {
                return Err(PathError::InsufficientData {
                    expected: offset + error_len,
                    actual: bytes.len(),
                });
            }
            let s = String::from_utf8_lossy(&bytes[offset..offset + error_len]).to_string();
            offset += error_len;
            Some(s)
        } else {
            None
        };

        // Output
        let output = read_string(bytes, &mut offset)?;

        // Bindings changed
        let n_bindings = read_u32(bytes, &mut offset)? as usize;
        let mut bindings_changed = Vec::with_capacity(n_bindings);
        for _ in 0..n_bindings {
            bindings_changed.push(read_string(bytes, &mut offset)?);
        }

        // Memory, execution time, AST depth, confidence
        if offset + 24 > bytes.len() {
            return Err(PathError::InsufficientData {
                expected: offset + 24,
                actual: bytes.len(),
            });
        }

        let memory_bytes = u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]) as usize;
        offset += 8;

        let execution_ns = u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;

        let ast_depth = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;
        offset += 4;

        let confidence_score = f32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);

        Ok(Self {
            command,
            input_mode,
            eval_type,
            success,
            error,
            output,
            bindings_changed,
            memory_bytes,
            execution_ns,
            ast_depth,
            confidence_score,
        })
    }
}

// =============================================================================
// ReplAuditCollector - Bridges ruchy sessions with entrenar
// =============================================================================

/// Collector that converts ruchy replay events to entrenar traces
pub struct ReplAuditCollector<C: TraceCollector<ReplPath>> {
    collector: C,
    sequence: u64,
    start_time: Instant,
}

impl<C: TraceCollector<ReplPath>> ReplAuditCollector<C> {
    /// Create a new audit collector with the given underlying collector
    pub fn new(collector: C) -> Self {
        Self {
            collector,
            sequence: 0,
            start_time: Instant::now(),
        }
    }

    /// Record a REPL event from the replay system
    pub fn record_event(&mut self, input: &str, mode: InputMode, result: &EvalResult) {
        let repl_mode = ReplInputMode::from(mode);
        let eval_type = ReplPath::infer_eval_type(input);

        let path = match result {
            EvalResult::Success { value } => {
                ReplPath::success(input.to_string(), repl_mode, eval_type, value.clone())
            }
            EvalResult::Error { message } => {
                ReplPath::failure(input.to_string(), repl_mode, eval_type, message.clone())
            }
            EvalResult::Unit => {
                ReplPath::success(input.to_string(), repl_mode, eval_type, "()".to_string())
            }
        };

        self.record_path(path);
    }

    /// Record a REPL path directly
    pub fn record_path(&mut self, path: ReplPath) {
        let timestamp_ns = self.start_time.elapsed().as_nanos() as u64;

        // Create input hash from command
        let input_hash = fnv1a_hash(path.command.as_bytes());

        // Output is confidence score (1.0 for success, 0.0 for error)
        let output = path.confidence_score;

        let trace = DecisionTrace::new(timestamp_ns, self.sequence, input_hash, path, output, 0);

        self.sequence += 1;
        self.collector.record(trace);
    }

    /// Get reference to the underlying collector
    pub fn collector(&self) -> &C {
        &self.collector
    }

    /// Get mutable reference to the underlying collector
    pub fn collector_mut(&mut self) -> &mut C {
        &mut self.collector
    }

    /// Flush any buffered traces
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.collector.flush()
    }

    /// Get the number of recorded traces
    pub fn len(&self) -> usize {
        self.collector.len()
    }

    /// Check if no traces have been recorded
    pub fn is_empty(&self) -> bool {
        self.collector.is_empty()
    }
}

// =============================================================================
// Convenience type aliases
// =============================================================================

/// Ring buffer audit collector (for real-time, memory-bounded logging)
pub type RingAuditCollector<const N: usize> = ReplAuditCollector<RingCollector<ReplPath, N>>;

/// Hash chain audit collector (for tamper-evident audit trails)
pub type HashChainAuditCollector = ReplAuditCollector<HashChainCollector<ReplPath>>;

/// Stream audit collector (for persistent file logging)
pub type StreamAuditCollector<W> = ReplAuditCollector<StreamCollector<ReplPath, W>>;

// =============================================================================
// Factory functions
// =============================================================================

/// Create a new ring buffer audit collector
pub fn ring_collector<const N: usize>() -> RingAuditCollector<N> {
    ReplAuditCollector::new(RingCollector::new())
}

/// Create a new hash chain audit collector
pub fn hash_chain_collector() -> HashChainAuditCollector {
    ReplAuditCollector::new(HashChainCollector::new())
}

/// Create a new stream audit collector
pub fn stream_collector<W: Write + Send + Sync>(
    writer: W,
    format: StreamFormat,
) -> StreamAuditCollector<W> {
    ReplAuditCollector::new(StreamCollector::new(writer, format))
}

// =============================================================================
// FNV-1a hash function
// =============================================================================

/// FNV-1a hash for input commands
#[inline]
fn fnv1a_hash(data: &[u8]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0100_0000_01b3;

    let mut hash = FNV_OFFSET;
    for byte in data {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================================================
    // ReplPath tests
    // ==========================================================================

    #[test]
    fn test_repl_path_success() {
        let path = ReplPath::success(
            "let x = 42".to_string(),
            ReplInputMode::Interactive,
            EvalType::Binding,
            "42".to_string(),
        );

        assert!(path.success);
        assert!(path.error.is_none());
        assert_eq!(path.command, "let x = 42");
        assert_eq!(path.output, "42");
        assert_eq!(path.confidence(), 1.0);
    }

    #[test]
    fn test_repl_path_failure() {
        let path = ReplPath::failure(
            "1 / 0".to_string(),
            ReplInputMode::Interactive,
            EvalType::Expression,
            "Division by zero".to_string(),
        );

        assert!(!path.success);
        assert!(path.error.is_some());
        assert_eq!(path.confidence(), 0.0);
    }

    #[test]
    fn test_repl_path_with_metadata() {
        let path = ReplPath::success(
            "x + 1".to_string(),
            ReplInputMode::Notebook,
            EvalType::Expression,
            "43".to_string(),
        )
        .with_bindings(vec!["x".to_string()])
        .with_memory(1024)
        .with_execution_time(5000)
        .with_ast_depth(3);

        assert_eq!(path.bindings_changed.len(), 1);
        assert_eq!(path.memory_bytes, 1024);
        assert_eq!(path.execution_ns, 5000);
        assert_eq!(path.ast_depth, 3);
    }

    #[test]
    fn test_infer_eval_type() {
        assert_eq!(ReplPath::infer_eval_type("let x = 42"), EvalType::Binding);
        assert_eq!(
            ReplPath::infer_eval_type("fn add(a, b) { a + b }"),
            EvalType::FunctionDef
        );
        assert_eq!(
            ReplPath::infer_eval_type("if x > 0 { x } else { -x }"),
            EvalType::ControlFlow
        );
        assert_eq!(ReplPath::infer_eval_type(":help"), EvalType::MagicCommand);
        assert_eq!(
            ReplPath::infer_eval_type("import std::io"),
            EvalType::Import
        );
        assert_eq!(
            ReplPath::infer_eval_type("struct Point { x: f32, y: f32 }"),
            EvalType::TypeDef
        );
        assert_eq!(
            ReplPath::infer_eval_type("println(x);"),
            EvalType::Statement
        );
        assert_eq!(ReplPath::infer_eval_type("2 + 2"), EvalType::Expression);
    }

    #[test]
    fn test_repl_path_explain() {
        let path = ReplPath::success(
            "let x = 42".to_string(),
            ReplInputMode::Interactive,
            EvalType::Binding,
            "42".to_string(),
        )
        .with_bindings(vec!["x".to_string()]);

        let explanation = path.explain();
        assert!(explanation.contains("REPL Command"));
        assert!(explanation.contains("Binding"));
        assert!(explanation.contains("let x = 42"));
        assert!(explanation.contains("Result: 42"));
        assert!(explanation.contains("Bindings changed: x"));
    }

    #[test]
    fn test_repl_path_serialization_roundtrip() {
        let path = ReplPath::success(
            "let x = 42".to_string(),
            ReplInputMode::Interactive,
            EvalType::Binding,
            "42".to_string(),
        )
        .with_bindings(vec!["x".to_string(), "y".to_string()])
        .with_memory(2048)
        .with_execution_time(10000)
        .with_ast_depth(5);

        let bytes = path.to_bytes();
        let restored = ReplPath::from_bytes(&bytes).expect("Deserialization should succeed");

        assert_eq!(path.command, restored.command);
        assert_eq!(path.input_mode, restored.input_mode);
        assert_eq!(path.eval_type, restored.eval_type);
        assert_eq!(path.success, restored.success);
        assert_eq!(path.error, restored.error);
        assert_eq!(path.output, restored.output);
        assert_eq!(path.bindings_changed, restored.bindings_changed);
        assert_eq!(path.memory_bytes, restored.memory_bytes);
        assert_eq!(path.execution_ns, restored.execution_ns);
        assert_eq!(path.ast_depth, restored.ast_depth);
        assert!((path.confidence_score - restored.confidence_score).abs() < 1e-6);
    }

    #[test]
    fn test_repl_path_serialization_with_error() {
        let path = ReplPath::failure(
            "undefined_var".to_string(),
            ReplInputMode::Script,
            EvalType::Expression,
            "Variable 'undefined_var' not found".to_string(),
        );

        let bytes = path.to_bytes();
        let restored = ReplPath::from_bytes(&bytes).expect("Deserialization should succeed");

        assert_eq!(path.command, restored.command);
        assert!(!restored.success);
        assert_eq!(path.error, restored.error);
    }

    #[test]
    fn test_repl_path_invalid_version() {
        let mut bytes = vec![2u8]; // Invalid version
        bytes.extend_from_slice(&0u32.to_le_bytes()); // command length

        let result = ReplPath::from_bytes(&bytes);
        assert!(matches!(result, Err(PathError::VersionMismatch { .. })));
    }

    #[test]
    fn test_repl_path_insufficient_data() {
        let result = ReplPath::from_bytes(&[1u8]); // Only version byte
        assert!(matches!(result, Err(PathError::InsufficientData { .. })));
    }

    // ==========================================================================
    // Input mode conversion tests
    // ==========================================================================

    #[test]
    fn test_input_mode_conversions() {
        assert_eq!(
            ReplInputMode::from(InputMode::Interactive),
            ReplInputMode::Interactive
        );
        assert_eq!(ReplInputMode::from(InputMode::Paste), ReplInputMode::Paste);
        assert_eq!(ReplInputMode::from(InputMode::File), ReplInputMode::File);
        assert_eq!(
            ReplInputMode::from(InputMode::Script),
            ReplInputMode::Script
        );

        // Reverse conversion
        assert_eq!(
            InputMode::from(ReplInputMode::Interactive),
            InputMode::Interactive
        );
        assert_eq!(InputMode::from(ReplInputMode::Notebook), InputMode::Script);
    }

    // ==========================================================================
    // ReplAuditCollector tests
    // ==========================================================================

    #[test]
    fn test_ring_audit_collector() {
        let mut collector: RingAuditCollector<64> = ring_collector();

        let path = ReplPath::success(
            "1 + 1".to_string(),
            ReplInputMode::Interactive,
            EvalType::Expression,
            "2".to_string(),
        );

        collector.record_path(path);
        assert_eq!(collector.len(), 1);
        assert!(!collector.is_empty());
    }

    #[test]
    fn test_hash_chain_audit_collector() {
        let mut collector = hash_chain_collector();

        collector.record_event(
            "let x = 42",
            InputMode::Interactive,
            &EvalResult::Success {
                value: "42".to_string(),
            },
        );

        collector.record_event(
            "x + 1",
            InputMode::Interactive,
            &EvalResult::Success {
                value: "43".to_string(),
            },
        );

        assert_eq!(collector.len(), 2);

        // Verify chain integrity
        let verification = collector.collector().verify_chain();
        assert!(verification.valid);
        assert_eq!(verification.entries_verified, 2);
    }

    #[test]
    fn test_audit_collector_with_errors() {
        let mut collector = hash_chain_collector();

        collector.record_event(
            "1 / 0",
            InputMode::Interactive,
            &EvalResult::Error {
                message: "Division by zero".to_string(),
            },
        );

        assert_eq!(collector.len(), 1);

        let entry = collector.collector().get(0).expect("Should have entry");
        assert!(!entry.trace.path.success);
        assert!(entry.trace.path.error.is_some());
    }

    #[test]
    fn test_stream_audit_collector() {
        let mut buffer = Vec::new();
        {
            let mut collector = stream_collector(&mut buffer, StreamFormat::JsonLines);

            collector.record_event(
                "let x = 42",
                InputMode::Interactive,
                &EvalResult::Success {
                    value: "42".to_string(),
                },
            );

            collector.flush().expect("Flush should succeed");
        }

        // Verify output
        let output = String::from_utf8(buffer).expect("UTF-8 output");
        assert!(output.contains("let x = 42"));
        assert!(output.contains("42"));
    }

    #[test]
    fn test_audit_collector_sequence_numbers() {
        let mut collector = hash_chain_collector();

        for i in 0..5 {
            collector.record_event(
                &format!("let x{i} = {i}"),
                InputMode::Interactive,
                &EvalResult::Success {
                    value: i.to_string(),
                },
            );
        }

        assert_eq!(collector.len(), 5);

        // Check sequence numbers
        for (i, entry) in collector.collector().entries().iter().enumerate() {
            assert_eq!(entry.sequence, i as u64);
        }
    }

    #[test]
    fn test_audit_collector_timestamps() {
        let mut collector = hash_chain_collector();

        collector.record_event(
            "1 + 1",
            InputMode::Interactive,
            &EvalResult::Success {
                value: "2".to_string(),
            },
        );

        std::thread::sleep(std::time::Duration::from_millis(10));

        collector.record_event(
            "2 + 2",
            InputMode::Interactive,
            &EvalResult::Success {
                value: "4".to_string(),
            },
        );

        let entries = collector.collector().entries();
        assert!(entries[1].trace.timestamp_ns > entries[0].trace.timestamp_ns);
    }

    // ==========================================================================
    // FNV hash tests
    // ==========================================================================

    #[test]
    fn test_fnv_hash_determinism() {
        let hash1 = fnv1a_hash(b"let x = 42");
        let hash2 = fnv1a_hash(b"let x = 42");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_fnv_hash_uniqueness() {
        let hash1 = fnv1a_hash(b"let x = 42");
        let hash2 = fnv1a_hash(b"let x = 43");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_fnv_hash_empty() {
        let hash = fnv1a_hash(b"");
        assert_ne!(hash, 0);
    }

    // ==========================================================================
    // Integration tests
    // ==========================================================================

    #[test]
    fn test_full_audit_workflow() {
        let mut collector = hash_chain_collector();

        // Simulate a REPL session
        let commands = vec![
            (
                "let x = 10",
                EvalResult::Success {
                    value: "10".to_string(),
                },
            ),
            (
                "let y = 20",
                EvalResult::Success {
                    value: "20".to_string(),
                },
            ),
            (
                "x + y",
                EvalResult::Success {
                    value: "30".to_string(),
                },
            ),
            (
                "undefined",
                EvalResult::Error {
                    message: "Variable not found".to_string(),
                },
            ),
            (
                "x * 2",
                EvalResult::Success {
                    value: "20".to_string(),
                },
            ),
        ];

        for (cmd, result) in commands {
            collector.record_event(cmd, InputMode::Interactive, &result);
        }

        // Verify audit trail
        assert_eq!(collector.len(), 5);

        let verification = collector.collector().verify_chain();
        assert!(verification.valid);

        // Verify individual entries
        let entries = collector.collector().entries();

        assert!(entries[0].trace.path.success);
        assert!(entries[1].trace.path.success);
        assert!(entries[2].trace.path.success);
        assert!(!entries[3].trace.path.success);
        assert!(entries[4].trace.path.success);
    }

    // ==========================================================================
    // EXTREME TDD Round 89 - Additional Coverage Tests
    // ==========================================================================

    #[test]
    fn test_eval_type_all_variants() {
        let types = vec![
            EvalType::Expression,
            EvalType::Statement,
            EvalType::Binding,
            EvalType::FunctionDef,
            EvalType::TypeDef,
            EvalType::Import,
            EvalType::ControlFlow,
            EvalType::MagicCommand,
        ];
        assert_eq!(types.len(), 8);
        // Verify each can be used
        for eval_type in types {
            let path = ReplPath::success(
                "test".to_string(),
                ReplInputMode::Interactive,
                eval_type,
                "result".to_string(),
            );
            assert!(path.success);
        }
    }

    #[test]
    fn test_repl_input_mode_all_variants() {
        let modes = vec![
            ReplInputMode::Interactive,
            ReplInputMode::Paste,
            ReplInputMode::File,
            ReplInputMode::Script,
            ReplInputMode::Notebook,
        ];
        assert_eq!(modes.len(), 5);
        for mode in modes {
            let path = ReplPath::success(
                "test".to_string(),
                mode,
                EvalType::Expression,
                "result".to_string(),
            );
            assert_eq!(path.input_mode, mode);
        }
    }

    #[test]
    fn test_repl_path_explain_failure() {
        let path = ReplPath::failure(
            "undefined_var".to_string(),
            ReplInputMode::Interactive,
            EvalType::Expression,
            "Variable 'undefined_var' not found".to_string(),
        );

        let explanation = path.explain();
        assert!(explanation.contains("REPL Command"));
        assert!(explanation.contains("Expression"));
        assert!(explanation.contains("undefined_var"));
        assert!(explanation.contains("Error"));
    }

    #[test]
    fn test_repl_path_confidence_scores() {
        let success_path = ReplPath::success(
            "1 + 1".to_string(),
            ReplInputMode::Interactive,
            EvalType::Expression,
            "2".to_string(),
        );
        assert_eq!(success_path.confidence(), 1.0);

        let failure_path = ReplPath::failure(
            "bad".to_string(),
            ReplInputMode::Interactive,
            EvalType::Expression,
            "error".to_string(),
        );
        assert_eq!(failure_path.confidence(), 0.0);
    }

    #[test]
    fn test_infer_eval_type_edge_cases() {
        // Test 'fn' keyword for function definitions
        assert_eq!(ReplPath::infer_eval_type("fn foo() {}"), EvalType::FunctionDef);
        // Test 'match' for control flow
        assert_eq!(ReplPath::infer_eval_type("match x { }"), EvalType::ControlFlow);
        // Test 'while' for control flow
        assert_eq!(ReplPath::infer_eval_type("while true {}"), EvalType::ControlFlow);
        // Test 'for' for control flow
        assert_eq!(ReplPath::infer_eval_type("for i in range {}"), EvalType::ControlFlow);
        // Test 'struct' for type def
        assert_eq!(ReplPath::infer_eval_type("struct Foo {}"), EvalType::TypeDef);
        // Test 'enum' for type def
        assert_eq!(ReplPath::infer_eval_type("enum Color {}"), EvalType::TypeDef);
        // Test 'type' alias for type def
        assert_eq!(ReplPath::infer_eval_type("type MyInt = i32"), EvalType::TypeDef);
    }

    #[test]
    fn test_repl_path_with_empty_bindings() {
        let path = ReplPath::success(
            "1 + 1".to_string(),
            ReplInputMode::Interactive,
            EvalType::Expression,
            "2".to_string(),
        )
        .with_bindings(vec![]);

        assert!(path.bindings_changed.is_empty());
    }

    #[test]
    fn test_repl_path_with_multiple_bindings() {
        let path = ReplPath::success(
            "let (a, b, c) = (1, 2, 3)".to_string(),
            ReplInputMode::Interactive,
            EvalType::Binding,
            "(1, 2, 3)".to_string(),
        )
        .with_bindings(vec!["a".to_string(), "b".to_string(), "c".to_string()]);

        assert_eq!(path.bindings_changed.len(), 3);
        assert!(path.bindings_changed.contains(&"a".to_string()));
        assert!(path.bindings_changed.contains(&"b".to_string()));
        assert!(path.bindings_changed.contains(&"c".to_string()));
    }

    #[test]
    fn test_repl_path_with_zero_values() {
        let path = ReplPath::success(
            "test".to_string(),
            ReplInputMode::Interactive,
            EvalType::Expression,
            "result".to_string(),
        )
        .with_memory(0)
        .with_execution_time(0)
        .with_ast_depth(0);

        assert_eq!(path.memory_bytes, 0);
        assert_eq!(path.execution_ns, 0);
        assert_eq!(path.ast_depth, 0);
    }

    #[test]
    fn test_repl_path_with_large_values() {
        let path = ReplPath::success(
            "test".to_string(),
            ReplInputMode::Interactive,
            EvalType::Expression,
            "result".to_string(),
        )
        .with_memory(1_000_000_000) // 1GB
        .with_execution_time(60_000_000_000) // 60 seconds
        .with_ast_depth(100);

        assert_eq!(path.memory_bytes, 1_000_000_000);
        assert_eq!(path.execution_ns, 60_000_000_000);
        assert_eq!(path.ast_depth, 100);
    }

    #[test]
    fn test_hash_chain_verification_with_single_entry() {
        let mut collector = hash_chain_collector();

        collector.record_event(
            "42",
            InputMode::Interactive,
            &EvalResult::Success {
                value: "42".to_string(),
            },
        );

        let verification = collector.collector().verify_chain();
        assert!(verification.valid);
        assert_eq!(verification.entries_verified, 1);
    }

    #[test]
    fn test_ring_collector_overflow() {
        let mut collector: RingAuditCollector<4> = ring_collector();

        // Add more entries than capacity
        for i in 0..10 {
            let path = ReplPath::success(
                format!("let x{i} = {i}"),
                ReplInputMode::Interactive,
                EvalType::Binding,
                i.to_string(),
            );
            collector.record_path(path);
        }

        // Ring buffer should have at most 4 entries
        assert!(collector.len() <= 4);
    }

    #[test]
    fn test_audit_collector_record_with_notebook_mode() {
        let mut collector = hash_chain_collector();

        let path = ReplPath::success(
            "# Cell 1".to_string(),
            ReplInputMode::Notebook,
            EvalType::Expression,
            "result".to_string(),
        );
        collector.record_path(path);

        assert_eq!(collector.len(), 1);
        let entry = collector.collector().get(0).expect("Should have entry");
        assert_eq!(entry.trace.path.input_mode, ReplInputMode::Notebook);
    }

    #[test]
    fn test_repl_path_success_with_binding() {
        let path = ReplPath::success(
            "let x = 42".to_string(),
            ReplInputMode::Interactive,
            EvalType::Binding,
            "42".to_string(),
        );

        assert!(path.success);
        assert_eq!(path.output, "42");
        assert!(path.error.is_none());
        assert_eq!(path.eval_type, EvalType::Binding);
    }

    #[test]
    fn test_repl_path_failure_with_error() {
        let path = ReplPath::failure(
            "bad syntax".to_string(),
            ReplInputMode::Interactive,
            EvalType::Expression,
            "Syntax error".to_string(),
        );

        assert!(!path.success);
        assert!(path.error.is_some());
        assert_eq!(path.error.unwrap(), "Syntax error");
    }

    #[test]
    fn test_fnv_hash_long_input() {
        let long_input = "x".repeat(10000);
        let hash = fnv1a_hash(long_input.as_bytes());
        assert_ne!(hash, 0);

        // Same input should produce same hash
        let hash2 = fnv1a_hash(long_input.as_bytes());
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_fnv_hash_special_characters() {
        let hash1 = fnv1a_hash("let x = \"hello\\nworld\"".as_bytes());
        let hash2 = fnv1a_hash("let x = \"hello\\nworld\"".as_bytes());
        assert_eq!(hash1, hash2);

        let hash3 = fnv1a_hash("let x = \"hello\\tworld\"".as_bytes());
        assert_ne!(hash1, hash3);
    }
}
