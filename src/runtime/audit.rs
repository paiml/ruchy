//! REPL Audit Records
//!
//! Decision-path records for REPL command execution, with a binary
//! serialization format (version 1) for persisted audit trails.
//!
//! # Toyota Way: 現地現物 (Genchi Genbutsu)
//!
//! Every REPL command is traced. All decisions are auditable. Go and see.
//!
//! # Port note (#329)
//!
//! Ported off `entrenar` (aprender-train, removed with the `training` feature).
//! The entrenar-typed parts were dropped: the `DecisionPath` trait impl became
//! inherent methods, the `TraceCollector`-based collectors, factories and
//! FNV input hashing were removed, and `entrenar::monitor::inference::path::PathError`
//! is replaced by the local `PathError` below with the same variants.
//!
//! # Example
//!
//! ```ignore
//! use ruchy::runtime::audit::{ReplInputMode, EvalType, ReplPath};
//!
//! let path = ReplPath::success("let x = 42".into(), ReplInputMode::Interactive,
//!     EvalType::Binding, "42".into());
//! let bytes = path.to_bytes();
//! assert!(ReplPath::from_bytes(&bytes).is_ok());
//! ```

use serde::{Deserialize, Serialize};

use super::replay::InputMode;

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
    pub fn explain(&self) -> String {
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

    pub fn feature_contributions(&self) -> &[f32] {
        // REPL doesn't have numerical features in the traditional sense
        &[]
    }

    pub fn confidence(&self) -> f32 {
        self.confidence_score
    }

    pub fn to_bytes(&self) -> Vec<u8> {
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

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PathError> {
        let mut r = Reader { bytes, offset: 0 };
        let version = r.u8()?;
        if version != 1 {
            return Err(PathError::VersionMismatch {
                expected: 1,
                actual: version,
            });
        }
        let command = r.string()?;
        let input_mode = decode_input_mode(r.u8()?)?;
        let eval_type = decode_eval_type(r.u8()?)?;
        let success = r.u8()? != 0;
        let error_len = r.u32()? as usize;
        let error = if error_len > 0 {
            Some(r.str_of(error_len)?)
        } else {
            None
        };
        let output = r.string()?;
        let n_bindings = r.u32()? as usize;
        let mut bindings_changed = Vec::with_capacity(n_bindings);
        for _ in 0..n_bindings {
            bindings_changed.push(r.string()?);
        }
        let memory_bytes = r.u64()? as usize;
        let execution_ns = r.u64()?;
        let ast_depth = r.u32()? as usize;
        let confidence_score = r.f32()?;
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

/// Bounds-checked cursor over a serialized `ReplPath`.
struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl Reader<'_> {
    fn slice(&mut self, len: usize) -> Result<&[u8], PathError> {
        let end = self.offset + len;
        let chunk = self
            .bytes
            .get(self.offset..end)
            .ok_or(PathError::InsufficientData {
                expected: end,
                actual: self.bytes.len(),
            })?;
        self.offset = end;
        Ok(chunk)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], PathError> {
        let mut out = [0u8; N];
        out.copy_from_slice(self.slice(N)?);
        Ok(out)
    }

    fn u8(&mut self) -> Result<u8, PathError> {
        Ok(self.array::<1>()?[0])
    }

    fn u32(&mut self) -> Result<u32, PathError> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    fn u64(&mut self) -> Result<u64, PathError> {
        Ok(u64::from_le_bytes(self.array()?))
    }

    fn f32(&mut self) -> Result<f32, PathError> {
        Ok(f32::from_le_bytes(self.array()?))
    }

    fn str_of(&mut self, len: usize) -> Result<String, PathError> {
        Ok(String::from_utf8_lossy(self.slice(len)?).into_owned())
    }

    fn string(&mut self) -> Result<String, PathError> {
        let len = self.u32()? as usize;
        self.str_of(len)
    }
}

fn decode_input_mode(tag: u8) -> Result<ReplInputMode, PathError> {
    match tag {
        0 => Ok(ReplInputMode::Interactive),
        1 => Ok(ReplInputMode::Paste),
        2 => Ok(ReplInputMode::File),
        3 => Ok(ReplInputMode::Script),
        4 => Ok(ReplInputMode::Notebook),
        _ => Err(PathError::InvalidFormat(format!(
            "Unknown input mode: {tag}"
        ))),
    }
}

fn decode_eval_type(tag: u8) -> Result<EvalType, PathError> {
    match tag {
        0 => Ok(EvalType::Expression),
        1 => Ok(EvalType::Statement),
        2 => Ok(EvalType::Binding),
        3 => Ok(EvalType::FunctionDef),
        4 => Ok(EvalType::ControlFlow),
        5 => Ok(EvalType::MagicCommand),
        6 => Ok(EvalType::Import),
        7 => Ok(EvalType::TypeDef),
        _ => Err(PathError::InvalidFormat(format!(
            "Unknown eval type: {tag}"
        ))),
    }
}

// =============================================================================
// PathError - local replacement for entrenar's PathError (#329)
// =============================================================================

/// Error decoding a serialized `ReplPath`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathError {
    /// Input ended before the expected number of bytes
    InsufficientData { expected: usize, actual: usize },
    /// Serialization version does not match
    VersionMismatch { expected: u8, actual: u8 },
    /// Malformed payload
    InvalidFormat(String),
}

impl std::fmt::Display for PathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsufficientData { expected, actual } => {
                write!(f, "insufficient data: expected {expected}, got {actual}")
            }
            Self::VersionMismatch { expected, actual } => {
                write!(f, "version mismatch: expected {expected}, got {actual}")
            }
            Self::InvalidFormat(msg) => write!(f, "invalid format: {msg}"),
        }
    }
}

impl std::error::Error for PathError {}

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
        assert_eq!(
            ReplPath::infer_eval_type("fn foo() {}"),
            EvalType::FunctionDef
        );
        // Test 'match' for control flow
        assert_eq!(
            ReplPath::infer_eval_type("match x { }"),
            EvalType::ControlFlow
        );
        // Test 'while' for control flow
        assert_eq!(
            ReplPath::infer_eval_type("while true {}"),
            EvalType::ControlFlow
        );
        // Test 'for' for control flow
        assert_eq!(
            ReplPath::infer_eval_type("for i in range {}"),
            EvalType::ControlFlow
        );
        // Test 'struct' for type def
        assert_eq!(
            ReplPath::infer_eval_type("struct Foo {}"),
            EvalType::TypeDef
        );
        // Test 'enum' for type def
        assert_eq!(
            ReplPath::infer_eval_type("enum Color {}"),
            EvalType::TypeDef
        );
        // Test 'type' alias for type def
        assert_eq!(
            ReplPath::infer_eval_type("type MyInt = i32"),
            EvalType::TypeDef
        );
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
}
