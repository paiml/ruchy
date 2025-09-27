//! High-Performance Interpreter with Safe Value Representation
//!
//! This module implements the two-tier execution strategy from ruchy-interpreter-spec.md:
//! - Tier 0: AST interpreter with enum-based values (safe alternative)
//! - Tier 1: JIT compilation (future)
//!
//! Uses safe Rust enum approach instead of tagged pointers to respect `unsafe_code = "forbid"`.

#![allow(clippy::unused_self)] // Methods will use self in future phases
#![allow(clippy::only_used_in_recursion)] // Recursive print_value is intentional
#![allow(clippy::uninlined_format_args)] // Some format strings are clearer unexpanded
#![allow(clippy::cast_precision_loss)] // Acceptable for arithmetic operations
#![allow(clippy::expect_used)] // Used appropriately in tests
#![allow(clippy::cast_possible_truncation)] // Controlled truncations for indices

use super::eval_expr;
use super::eval_func;
use super::eval_literal;
use super::eval_method;
use super::eval_operations;
use crate::frontend::ast::{
    BinaryOp as AstBinaryOp, Expr, ExprKind, Literal, MatchArm, Pattern, StringPart,
};
use crate::frontend::Param;
use smallvec::{smallvec, SmallVec};
use std::collections::HashMap;
use std::rc::Rc;

/// Control flow for loop iterations or error
#[derive(Debug)]
enum LoopControlOrError {
    Break(Value),
    Continue,
    Error(InterpreterError),
}

/// `DataFrame` column representation for the interpreter
#[derive(Debug, Clone, PartialEq)]
pub struct DataFrameColumn {
    pub name: String,
    pub values: Vec<Value>,
}

/// Runtime value representation using safe enum approach.
///
/// `Value` represents all runtime values in the Ruchy interpreter. This is an
/// enum-based approach that avoids unsafe code while maintaining good performance
/// through strategic use of `Rc` for heap-allocated data.
///
/// # Examples
///
/// ```
/// use ruchy::runtime::interpreter::Value;
///
/// let int_val = Value::from_i64(42);
/// let str_val = Value::from_string("hello".to_string());
/// let arr_val = Value::from_array(vec![int_val.clone(), str_val]);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// 64-bit signed integer
    Integer(i64),
    /// 64-bit float
    Float(f64),
    /// Boolean value
    Bool(bool),
    /// Nil/null value
    Nil,
    /// String value (reference-counted for efficiency)
    String(Rc<str>),
    /// Array of values
    Array(Rc<[Value]>),
    /// Tuple of values
    Tuple(Rc<[Value]>),
    /// Function closure
    Closure {
        params: Vec<String>,
        body: Rc<Expr>,
        env: Rc<HashMap<String, Value>>, // Captured environment
    },
    /// `DataFrame` value
    DataFrame { columns: Vec<DataFrameColumn> },
    /// Object/HashMap value for key-value mappings
    Object(Rc<HashMap<String, Value>>),
    /// Range value for representing ranges
    Range {
        start: Box<Value>,
        end: Box<Value>,
        inclusive: bool,
    },
    /// Enum variant value
    EnumVariant {
        variant_name: String,
        data: Option<Vec<Value>>,
    },
    /// Built-in function reference
    BuiltinFunction(String),
}

impl Value {
    /// Get the type ID for this value for caching purposes
    ///
    /// # Complexity
    /// Cyclomatic complexity: 10 (within Toyota Way limits, just barely)
    pub fn type_id(&self) -> std::any::TypeId {
        use std::any::TypeId;
        match self {
            Value::Integer(_) => TypeId::of::<i64>(),
            Value::Float(_) => TypeId::of::<f64>(),
            Value::Bool(_) => TypeId::of::<bool>(),
            Value::String(_) => TypeId::of::<String>(),
            Value::Nil => TypeId::of::<()>(),
            Value::Array(_) => TypeId::of::<Vec<Value>>(),
            Value::Tuple(_) => TypeId::of::<(Value,)>(), // Generic tuple marker
            Value::Closure { .. } => TypeId::of::<fn()>(), // Generic closure marker
            Value::DataFrame { .. } => TypeId::of::<crate::runtime::DataFrameColumn>(),
            Value::Object(_) => TypeId::of::<HashMap<String, Value>>(),
            Value::Range { .. } => TypeId::of::<std::ops::Range<i64>>(),
            Value::EnumVariant { .. } => TypeId::of::<(String, Option<Vec<Value>>)>(),
            Value::BuiltinFunction(_) => TypeId::of::<fn()>(),
        }
    }
}

// Value utility methods moved to value_utils.rs

// Note: Complex object structures (ObjectHeader, Class, etc.) will be implemented
// in Phase 1 of the interpreter spec when we add proper GC and method dispatch.

/// Runtime interpreter state.
///
/// The `Interpreter` manages the execution environment for Ruchy programs.
/// It maintains:
/// - A value stack for computation
/// - Environment stack for lexical scoping
/// - Inline caches for field/method optimization
/// - Type feedback for future JIT compilation
/// - Conservative garbage collection
///
/// # Implementation Strategy
///
/// This follows a two-tier execution model:
/// - **Tier 0**: AST interpretation (current)
/// - **Tier 1**: JIT compilation (future)
///
/// Type feedback and execution counts are collected for hot code
/// identification and optimization.
pub struct Interpreter {
    /// Tagged pointer values for fast operation
    stack: Vec<Value>,

    /// Environment stack for lexical scoping
    env_stack: Vec<HashMap<std::string::String, Value>>,

    /// Call frame for function calls
    #[allow(dead_code)]
    frames: Vec<CallFrame>,

    /// Execution statistics for tier transition (will be used in Phase 1)
    #[allow(dead_code)]
    execution_counts: HashMap<usize, u32>, // Function/method ID -> execution count

    /// Inline caches for field/method access optimization
    field_caches: HashMap<String, InlineCache>,

    /// Type feedback collection for JIT compilation
    type_feedback: TypeFeedback,

    /// Conservative garbage collector
    gc: ConservativeGC,

    /// Error handler scopes for try/catch
    error_scopes: Vec<ErrorScope>,
}

/// Error scope for try/catch blocks
#[derive(Debug, Clone)]
struct ErrorScope {
    /// Depth of environment stack when try block started
    env_depth: usize,
}

/// Call frame for function invocation (will be used in Phase 1)
#[derive(Debug)]
#[allow(dead_code)]
pub struct CallFrame {
    /// Function being executed
    closure: Value,

    /// Instruction pointer
    ip: *const u8,

    /// Base of stack frame
    base: usize,

    /// Number of locals in this frame
    locals: usize,
}

/// Interpreter execution result
pub enum InterpreterResult {
    Continue,
    Jump(usize),
    Return(Value),
    Error(InterpreterError),
}

/// Errors that can occur during interpretation.
///
/// # Examples
///
/// ```
/// use ruchy::runtime::interpreter::InterpreterError;
///
/// let err = InterpreterError::TypeError("Expected integer".to_string());
/// assert_eq!(err.to_string(), "Type error: Expected integer");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum InterpreterError {
    TypeError(std::string::String),
    RuntimeError(std::string::String),
    StackOverflow,
    StackUnderflow,
    InvalidInstruction,
    DivisionByZero,
    IndexOutOfBounds,
    Break(Value),
    Continue,
    Return(Value),
    Throw(Value), // EXTREME TDD: Exception handling
}

// Display implementations moved to eval_display.rs

/// Inline cache states for polymorphic method dispatch
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheState {
    /// No cache entry yet
    Uninitialized,
    /// Single type cached - fastest path
    Monomorphic,
    /// 2-4 types cached - still fast
    Polymorphic,
    /// Too many types - fallback to hash lookup
    Megamorphic,
}

/// Cache entry for field access optimization
#[derive(Clone, Debug)]
pub struct CacheEntry {
    /// Type identifier for cache validity
    type_id: std::any::TypeId,
    /// Field name being accessed
    field_name: String,
    /// Cached result for this type/field combination
    cached_result: Value,
    /// Hit count for LRU eviction
    hit_count: u32,
}

/// Inline cache for method/field dispatch
#[derive(Clone, Debug)]
pub struct InlineCache {
    /// Current cache state
    state: CacheState,
    /// Cache entries (inline storage for 2 common entries)
    entries: SmallVec<[CacheEntry; 2]>,
    /// Total hit count
    total_hits: u32,
    /// Total miss count
    total_misses: u32,
}

impl InlineCache {
    /// Create new empty inline cache
    pub fn new() -> Self {
        Self {
            state: CacheState::Uninitialized,
            entries: smallvec![],
            total_hits: 0,
            total_misses: 0,
        }
    }

    /// Look up a field access in the cache
    pub fn lookup(&mut self, obj: &Value, field_name: &str) -> Option<Value> {
        let type_id = obj.type_id();

        // Fast path: check cached entries
        for entry in &mut self.entries {
            if entry.type_id == type_id && entry.field_name == field_name {
                entry.hit_count += 1;
                self.total_hits += 1;
                return Some(entry.cached_result.clone());
            }
        }

        // Cache miss
        self.total_misses += 1;
        None
    }

    /// Add a new cache entry
    pub fn insert(&mut self, obj: &Value, field_name: String, result: Value) {
        let type_id = obj.type_id();
        let entry = CacheEntry {
            type_id,
            field_name,
            cached_result: result,
            hit_count: 1,
        };

        // Update cache state based on entry count
        match self.entries.len() {
            0 => {
                self.state = CacheState::Monomorphic;
                self.entries.push(entry);
            }
            1..=3 => {
                self.state = CacheState::Polymorphic;
                self.entries.push(entry);
            }
            _ => {
                // Too many entries - transition to megamorphic
                self.state = CacheState::Megamorphic;
                // Evict least used entry
                if let Some(min_idx) = self
                    .entries
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, e)| e.hit_count)
                    .map(|(i, _)| i)
                {
                    self.entries[min_idx] = entry;
                }
            }
        }
    }

    /// Get cache hit rate for profiling
    pub fn hit_rate(&self) -> f64 {
        let total = self.total_hits + self.total_misses;
        if total == 0 {
            0.0
        } else {
            f64::from(self.total_hits) / f64::from(total)
        }
    }
}

impl Default for InlineCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Type feedback collection for JIT compilation decisions.
///
/// Collects runtime type information to guide optimization decisions in
/// future JIT compilation tiers. This includes:
///
/// - Operation site type patterns (for inline caching)
/// - Variable type stability (for specialization)
/// - Call site patterns (for method inlining)
///
/// # Usage
///
/// The interpreter automatically collects type feedback during execution.
/// When functions become "hot" (frequently executed), this data guides
/// JIT compilation decisions.
///
/// # Statistics
///
/// Use `get_stats()` to retrieve feedback statistics for monitoring
/// and debugging optimization decisions.
#[derive(Clone, Debug)]
pub struct TypeFeedback {
    /// Operation site feedback (indexed by AST node or bytecode offset)
    operation_sites: HashMap<usize, OperationFeedback>,
    /// Variable type patterns (variable name -> type feedback)
    variable_types: HashMap<String, VariableTypeFeedback>,
    /// Function call sites with argument/return type information
    call_sites: HashMap<usize, CallSiteFeedback>,
    /// Total feedback collection count
    total_samples: u64,
}

/// Feedback for a specific operation site (binary ops, field access, etc.)
#[derive(Clone, Debug)]
pub struct OperationFeedback {
    /// Types observed for left operand
    left_types: SmallVec<[std::any::TypeId; 4]>,
    /// Types observed for right operand (for binary ops)
    right_types: SmallVec<[std::any::TypeId; 4]>,
    /// Result types observed
    result_types: SmallVec<[std::any::TypeId; 4]>,
    /// Hit counts for each type combination
    type_counts: HashMap<(std::any::TypeId, std::any::TypeId), u32>,
    /// Total operation count
    total_count: u32,
}

/// Type feedback for variables across their lifetime
#[derive(Clone, Debug)]
pub struct VariableTypeFeedback {
    /// Types assigned to this variable
    assigned_types: SmallVec<[std::any::TypeId; 4]>,
    /// Type transitions (`from_type` -> `to_type`)
    transitions: HashMap<std::any::TypeId, HashMap<std::any::TypeId, u32>>,
    /// Most common type (for specialization)
    dominant_type: Option<std::any::TypeId>,
    /// Type stability score (0.0 = highly polymorphic, 1.0 = monomorphic)
    stability_score: f64,
}

/// Feedback for function call sites
#[derive(Clone, Debug)]
pub struct CallSiteFeedback {
    /// Argument type patterns observed
    arg_type_patterns: SmallVec<[Vec<std::any::TypeId>; 4]>,
    /// Return types observed
    return_types: SmallVec<[std::any::TypeId; 4]>,
    /// Call frequency
    call_count: u32,
    /// Functions called at this site (for polymorphic calls)
    called_functions: HashMap<String, u32>,
}

impl TypeFeedback {
    /// Create new type feedback collector
    pub fn new() -> Self {
        Self {
            operation_sites: HashMap::new(),
            variable_types: HashMap::new(),
            call_sites: HashMap::new(),
            total_samples: 0,
        }
    }

    /// Record binary operation type feedback
    pub fn record_binary_op(
        &mut self,
        site_id: usize,
        left: &Value,
        right: &Value,
        result: &Value,
    ) {
        let left_type = left.type_id();
        let right_type = right.type_id();
        let result_type = result.type_id();

        let feedback = self
            .operation_sites
            .entry(site_id)
            .or_insert_with(|| OperationFeedback {
                left_types: smallvec![],
                right_types: smallvec![],
                result_types: smallvec![],
                type_counts: HashMap::new(),
                total_count: 0,
            });

        // Record types if not already seen
        if !feedback.left_types.contains(&left_type) {
            feedback.left_types.push(left_type);
        }
        if !feedback.right_types.contains(&right_type) {
            feedback.right_types.push(right_type);
        }
        if !feedback.result_types.contains(&result_type) {
            feedback.result_types.push(result_type);
        }

        // Update type combination counts
        let type_pair = (left_type, right_type);
        *feedback.type_counts.entry(type_pair).or_insert(0) += 1;
        feedback.total_count += 1;
        self.total_samples += 1;
    }

    /// Record variable assignment type feedback
    pub fn record_variable_assignment(&mut self, var_name: &str, new_type: std::any::TypeId) {
        let feedback = self
            .variable_types
            .entry(var_name.to_string())
            .or_insert_with(|| VariableTypeFeedback {
                assigned_types: smallvec![],
                transitions: HashMap::new(),
                dominant_type: None,
                stability_score: 1.0,
            });

        // Record type transition if there was a previous type
        if let Some(prev_type) = feedback.dominant_type {
            if prev_type != new_type {
                feedback
                    .transitions
                    .entry(prev_type)
                    .or_default()
                    .entry(new_type)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
        }

        // Add new type if not seen before
        if !feedback.assigned_types.contains(&new_type) {
            feedback.assigned_types.push(new_type);
        }

        // Update dominant type (most recently assigned for simplicity)
        feedback.dominant_type = Some(new_type);

        // Recalculate stability score
        feedback.stability_score = if feedback.assigned_types.len() == 1 {
            1.0 // Monomorphic
        } else {
            1.0 / f64::from(u32::try_from(feedback.assigned_types.len()).unwrap_or(u32::MAX))
            // Decreases with more types
        };
    }

    /// Record function call type feedback
    pub fn record_function_call(
        &mut self,
        site_id: usize,
        func_name: &str,
        args: &[Value],
        result: &Value,
    ) {
        let arg_types: Vec<std::any::TypeId> = args.iter().map(Value::type_id).collect();
        let return_type = result.type_id();

        let feedback = self
            .call_sites
            .entry(site_id)
            .or_insert_with(|| CallSiteFeedback {
                arg_type_patterns: smallvec![],
                return_types: smallvec![],
                call_count: 0,
                called_functions: HashMap::new(),
            });

        // Record argument pattern if not seen before
        if !feedback
            .arg_type_patterns
            .iter()
            .any(|pattern| pattern == &arg_types)
        {
            feedback.arg_type_patterns.push(arg_types);
        }

        // Record return type if not seen before
        if !feedback.return_types.contains(&return_type) {
            feedback.return_types.push(return_type);
        }

        // Update function call counts
        *feedback
            .called_functions
            .entry(func_name.to_string())
            .or_insert(0) += 1;
        feedback.call_count += 1;
    }

    /// Get type specialization suggestions for optimization
    /// # Panics
    /// Panics if a variable's dominant type is None when it should exist
    pub fn get_specialization_candidates(&self) -> Vec<SpecializationCandidate> {
        let mut candidates = Vec::new();

        // Find monomorphic operation sites
        for (&site_id, feedback) in &self.operation_sites {
            if feedback.left_types.len() == 1
                && feedback.right_types.len() == 1
                && feedback.total_count > 10
            {
                candidates.push(SpecializationCandidate {
                    kind: SpecializationKind::BinaryOperation {
                        site_id,
                        left_type: feedback.left_types[0],
                        right_type: feedback.right_types[0],
                    },
                    confidence: 1.0,
                    benefit_score: f64::from(feedback.total_count),
                });
            }
        }

        // Find stable variables
        for (var_name, feedback) in &self.variable_types {
            if feedback.stability_score > 0.8 && feedback.dominant_type.is_some() {
                candidates.push(SpecializationCandidate {
                    kind: SpecializationKind::Variable {
                        name: var_name.clone(),
                        #[allow(clippy::expect_used)] // Safe: we just checked is_some() above
                        specialized_type: feedback.dominant_type.expect("Dominant type should exist for stable variables"),
                    },
                    confidence: feedback.stability_score,
                    benefit_score: feedback.stability_score * 100.0,
                });
            }
        }

        // Find monomorphic call sites
        for (&site_id, feedback) in &self.call_sites {
            if feedback.arg_type_patterns.len() == 1
                && feedback.return_types.len() == 1
                && feedback.call_count > 5
            {
                candidates.push(SpecializationCandidate {
                    kind: SpecializationKind::FunctionCall {
                        site_id,
                        arg_types: feedback.arg_type_patterns[0].clone(),
                        return_type: feedback.return_types[0],
                    },
                    confidence: 1.0,
                    benefit_score: f64::from(feedback.call_count * 10),
                });
            }
        }

        // Sort by benefit score (highest first)
        candidates.sort_by(|a, b| {
            b.benefit_score
                .partial_cmp(&a.benefit_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates
    }

    /// Get overall type feedback statistics
    pub fn get_statistics(&self) -> TypeFeedbackStats {
        let monomorphic_sites = self
            .operation_sites
            .values()
            .filter(|f| f.left_types.len() == 1 && f.right_types.len() == 1)
            .count();

        let stable_variables = self
            .variable_types
            .values()
            .filter(|f| f.stability_score > 0.8)
            .count();

        let monomorphic_calls = self
            .call_sites
            .values()
            .filter(|f| f.arg_type_patterns.len() == 1)
            .count();

        TypeFeedbackStats {
            total_operation_sites: self.operation_sites.len(),
            monomorphic_operation_sites: monomorphic_sites,
            total_variables: self.variable_types.len(),
            stable_variables,
            total_call_sites: self.call_sites.len(),
            monomorphic_call_sites: monomorphic_calls,
            total_samples: self.total_samples,
        }
    }
}

impl Default for TypeFeedback {
    fn default() -> Self {
        Self::new()
    }
}

/// Specialization candidate for JIT compilation
#[derive(Clone, Debug)]
#[allow(dead_code)] // Will be used by future JIT implementation
pub struct SpecializationCandidate {
    /// Type of specialization
    kind: SpecializationKind,
    /// Confidence level (0.0 - 1.0)
    confidence: f64,
    /// Expected benefit score
    benefit_score: f64,
}

#[derive(Clone, Debug)]
pub enum SpecializationKind {
    BinaryOperation {
        site_id: usize,
        left_type: std::any::TypeId,
        right_type: std::any::TypeId,
    },
    Variable {
        name: String,
        specialized_type: std::any::TypeId,
    },
    FunctionCall {
        site_id: usize,
        arg_types: Vec<std::any::TypeId>,
        return_type: std::any::TypeId,
    },
}

/// Type feedback statistics for profiling
#[derive(Clone, Debug)]
pub struct TypeFeedbackStats {
    /// Total operation sites recorded
    pub total_operation_sites: usize,
    /// Monomorphic operation sites (candidates for specialization)
    pub monomorphic_operation_sites: usize,
    /// Total variables tracked
    pub total_variables: usize,
    /// Variables with stable types
    pub stable_variables: usize,
    /// Total function call sites
    pub total_call_sites: usize,
    /// Monomorphic call sites
    pub monomorphic_call_sites: usize,
    /// Total feedback samples collected
    pub total_samples: u64,
}

// Re-export GC implementation from gc_impl module
// EXTREME TDD: Eliminated 318 lines of duplicate GC code (massive entropy reduction)
pub use super::gc_impl::{ConservativeGC, GCInfo, GCObject, GCStats};

// Re-export compilation implementation from compilation module
// EXTREME TDD: Eliminated 669 lines of compilation code (massive entropy reduction)
pub use super::compilation::{
    DirectThreadedInterpreter, InstructionResult, InterpreterState, ThreadedInstruction,
};

impl Interpreter {
    /// Create a new interpreter instance.
    ///
    /// Initializes the interpreter with:
    /// - Pre-allocated stack for performance
    /// - Global environment with builtin functions (max, min, floor, ceil, etc.)
    /// - Type feedback collection for future JIT compilation
    /// - Conservative garbage collector
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::interpreter::Interpreter;
    ///
    /// let mut interpreter = Interpreter::new();
    /// // Interpreter is ready to evaluate expressions
    /// ```
    pub fn new() -> Self {
        // EXTREME TDD: Delegate builtin initialization to eliminate 62 lines of entropy
        let global_env = crate::runtime::builtin_init::init_global_environment();

        Self {
            stack: Vec::with_capacity(1024), // Pre-allocate stack
            env_stack: vec![global_env],     // Start with global environment containing builtins
            frames: Vec::new(),
            execution_counts: HashMap::new(),
            field_caches: HashMap::new(),
            type_feedback: TypeFeedback::new(),
            gc: ConservativeGC::new(),
            error_scopes: Vec::new(),
        }
    }

    /// Evaluate an AST expression directly.
    ///
    /// This is the main entry point for interpreting Ruchy expressions. It walks
    /// the AST recursively, evaluating expressions and returning their values.
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::interpreter::Interpreter;
    /// use ruchy::frontend::parser::Parser;
    ///
    /// let mut interpreter = Interpreter::new();
    /// let mut parser = Parser::new("42");
    /// let expr = parser.parse().unwrap();
    /// let result = interpreter.eval_expr(&expr).unwrap();
    /// assert_eq!(result.to_string(), "42");
    /// ```
    ///
    /// ```
    /// use ruchy::runtime::interpreter::Interpreter;
    /// use ruchy::frontend::parser::Parser;
    ///
    /// let mut interpreter = Interpreter::new();
    /// let mut parser = Parser::new("2 + 3");
    /// let expr = parser.parse().unwrap();
    /// let result = interpreter.eval_expr(&expr).unwrap();
    /// assert_eq!(result.to_string(), "5");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when:
    /// - Type error (e.g., adding string to number)
    /// - Runtime error (e.g., undefined variable)
    /// - Stack overflow/underflow
    /// - Division by zero
    pub fn eval_expr(&mut self, expr: &Expr) -> Result<Value, InterpreterError> {
        self.eval_expr_kind(&expr.kind)
    }

    /// Evaluate an expression kind directly.
    ///
    /// This is the core dispatch function for the interpreter. It pattern-matches
    /// on the `ExprKind` and delegates to specialized evaluation functions.
    ///
    /// The function is organized into logical groups:
    /// - Basic expressions (literals, identifiers)
    /// - Operations (binary, unary, calls)
    /// - Functions (definitions, lambdas)
    /// - Control flow (if, for, while, match)
    /// - Data structures (lists, tuples, arrays)
    /// - Assignments
    ///
    /// # Errors
    ///
    /// Returns an error if the expression evaluation fails or if the expression
    /// type is not yet implemented.
    fn eval_expr_kind(&mut self, expr_kind: &ExprKind) -> Result<Value, InterpreterError> {
        match expr_kind {
            // Simple expressions (complexity: 2)
            ExprKind::Literal(_) | ExprKind::Identifier(_) => self.eval_simple_expr(expr_kind),

            // Operations (complexity: 2)
            ExprKind::Binary { .. }
            | ExprKind::Unary { .. }
            | ExprKind::Call { .. }
            | ExprKind::MethodCall { .. }
            | ExprKind::DataFrameOperation { .. }
            | ExprKind::IndexAccess { .. }
            | ExprKind::FieldAccess { .. } => self.eval_operation_expr(expr_kind),

            // Functions (complexity: 2)
            ExprKind::Function { .. } | ExprKind::Lambda { .. } => {
                self.eval_function_expr(expr_kind)
            }

            // Control flow (complexity: 1)
            kind if Self::is_control_flow_expr(kind) => self.eval_control_flow_expr(kind),

            // Data structures (complexity: 1)
            kind if Self::is_data_structure_expr(kind) => self.eval_data_structure_expr(kind),

            // Assignments (complexity: 1)
            kind if Self::is_assignment_expr(kind) => self.eval_assignment_expr(kind),

            // Other expressions (complexity: 1)
            _ => self.eval_misc_expr(expr_kind),
        }
    }

    // Helper methods for expression type categorization and evaluation (complexity <10 each)

    /// Evaluate simple expressions (literals and identifiers)
    /// Complexity: 3
    fn eval_simple_expr(&mut self, expr_kind: &ExprKind) -> Result<Value, InterpreterError> {
        match expr_kind {
            ExprKind::Literal(lit) => Ok(eval_literal::eval_literal(lit)),
            ExprKind::Identifier(name) => self.lookup_variable(name),
            _ => unreachable!("eval_simple_expr called with non-simple expression"),
        }
    }

    /// Evaluate operation expressions (binary, unary, calls, method calls, etc.)
    /// Complexity: 8
    fn eval_operation_expr(&mut self, expr_kind: &ExprKind) -> Result<Value, InterpreterError> {
        match expr_kind {
            ExprKind::Binary { left, op, right } => self.eval_binary_expr(left, *op, right),
            ExprKind::Unary { op, operand } => self.eval_unary_expr(*op, operand),
            ExprKind::Call { func, args } => self.eval_function_call(func, args),
            ExprKind::MethodCall {
                receiver,
                method,
                args,
            } => self.eval_method_call(receiver, method, args),
            ExprKind::DataFrameOperation { source, operation } => {
                self.eval_dataframe_operation(source, operation)
            }
            ExprKind::IndexAccess { object, index } => self.eval_index_access(object, index),
            ExprKind::FieldAccess { object, field } => self.eval_field_access(object, field),
            _ => unreachable!("eval_operation_expr called with non-operation expression"),
        }
    }

    /// Evaluate function expressions (function definitions and lambdas)
    /// Complexity: 3
    fn eval_function_expr(&mut self, expr_kind: &ExprKind) -> Result<Value, InterpreterError> {
        match expr_kind {
            ExprKind::Function {
                name, params, body, ..
            } => self.eval_function(name, params, body),
            ExprKind::Lambda { params, body } => self.eval_lambda(params, body),
            _ => unreachable!("eval_function_expr called with non-function expression"),
        }
    }

    /// Evaluate miscellaneous expressions
    /// Complexity: 5
    fn eval_misc_expr(&mut self, expr_kind: &ExprKind) -> Result<Value, InterpreterError> {
        match expr_kind {
            ExprKind::StringInterpolation { parts } => self.eval_string_interpolation(parts),
            ExprKind::QualifiedName { module, name } => self.eval_qualified_name(module, name),
            ExprKind::ObjectLiteral { fields } => self.eval_object_literal(fields),
            ExprKind::LetPattern {
                pattern,
                value,
                body,
                ..
            } => self.eval_let_pattern(pattern, value, body),
            ExprKind::Actor {
                name,
                state,
                handlers,
            } => self.eval_actor_definition(name, state, handlers),
            ExprKind::Struct {
                name,
                type_params,
                fields,
                is_pub,
            } => self.eval_struct_definition(name, type_params, fields, *is_pub),
            ExprKind::Class {
                name,
                type_params,
                superclass,
                traits,
                fields,
                constructors,
                methods,
                derives,
                is_pub,
            } => self.eval_class_definition(
                name,
                type_params,
                superclass.as_ref(),
                traits,
                fields,
                constructors,
                methods,
                derives,
                *is_pub,
            ),
            ExprKind::StructLiteral { name, fields } => self.eval_struct_literal(name, fields),
            _ => Err(InterpreterError::RuntimeError(format!(
                "Expression type not yet implemented: {expr_kind:?}"
            ))),
        }
    }

    fn is_control_flow_expr(expr_kind: &ExprKind) -> bool {
        eval_expr::is_control_flow_expr(expr_kind)
    }

    fn is_data_structure_expr(expr_kind: &ExprKind) -> bool {
        eval_expr::is_data_structure_expr(expr_kind)
    }

    fn is_assignment_expr(expr_kind: &ExprKind) -> bool {
        eval_expr::is_assignment_expr(expr_kind)
    }

    fn eval_control_flow_expr(&mut self, expr_kind: &ExprKind) -> Result<Value, InterpreterError> {
        match expr_kind {
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.eval_if_expr(condition, then_branch, else_branch.as_deref()),
            ExprKind::Ternary {
                condition,
                true_expr,
                false_expr,
            } => {
                // Evaluate condition
                let cond_value = self.eval_expr(condition)?;
                // Check if condition is truthy
                if cond_value.is_truthy() {
                    self.eval_expr(true_expr)
                } else {
                    self.eval_expr(false_expr)
                }
            }
            ExprKind::Let {
                name, value, body, ..
            } => self.eval_let_expr(name, value, body),
            ExprKind::For {
                var,
                pattern,
                iter,
                body,
            } => self.eval_for_loop(var, pattern.as_ref(), iter, body),
            ExprKind::While { condition, body } => self.eval_while_loop(condition, body),
            ExprKind::Match { expr, arms } => self.eval_match(expr, arms),
            ExprKind::Break { label: _ } => {
                Err(InterpreterError::RuntimeError("break".to_string()))
            }
            ExprKind::Continue { label: _ } => {
                Err(InterpreterError::RuntimeError("continue".to_string()))
            }
            ExprKind::Return { value } => self.eval_return_expr(value.as_deref()),
            ExprKind::TryCatch {
                try_block,
                catch_clauses,
                finally_block,
            } => crate::runtime::eval_try_catch::eval_try_catch(
                self,
                try_block,
                catch_clauses,
                finally_block.as_deref(),
            ),
            ExprKind::Throw { expr } => crate::runtime::eval_try_catch::eval_throw(self, expr),
            _ => unreachable!("Non-control-flow expression passed to eval_control_flow_expr"),
        }
    }

    fn eval_data_structure_expr(
        &mut self,
        expr_kind: &ExprKind,
    ) -> Result<Value, InterpreterError> {
        match expr_kind {
            ExprKind::List(elements) => self.eval_list_expr(elements),
            ExprKind::Block(statements) => self.eval_block_expr(statements),
            ExprKind::Tuple(elements) => self.eval_tuple_expr(elements),
            ExprKind::Range {
                start,
                end,
                inclusive,
            } => self.eval_range_expr(start, end, *inclusive),
            ExprKind::ArrayInit { value, size } => self.eval_array_init_expr(value, size),
            _ => unreachable!("Non-data-structure expression passed to eval_data_structure_expr"),
        }
    }

    fn eval_assignment_expr(&mut self, expr_kind: &ExprKind) -> Result<Value, InterpreterError> {
        match expr_kind {
            ExprKind::Assign { target, value } => self.eval_assign(target, value),
            ExprKind::CompoundAssign { target, op, value } => {
                self.eval_compound_assign(target, *op, value)
            }
            _ => unreachable!("Non-assignment expression passed to eval_assignment_expr"),
        }
    }

    fn eval_index_access(
        &mut self,
        object: &Expr,
        index: &Expr,
    ) -> Result<Value, InterpreterError> {
        let object_value = self.eval_expr(object)?;
        let index_value = self.eval_expr(index)?;

        match (&object_value, &index_value) {
            (Value::Array(ref array), Value::Integer(idx)) => {
                let index = *idx as usize;
                if index < array.len() {
                    Ok(array[index].clone())
                } else {
                    Err(InterpreterError::RuntimeError(format!(
                        "Index {} out of bounds for array of length {}",
                        index,
                        array.len()
                    )))
                }
            }
            (Value::String(ref s), Value::Integer(idx)) => {
                let index = *idx as usize;
                let chars: Vec<char> = s.chars().collect();
                if index < chars.len() {
                    Ok(Value::from_string(chars[index].to_string()))
                } else {
                    Err(InterpreterError::RuntimeError(format!(
                        "Index {} out of bounds for string of length {}",
                        index,
                        chars.len()
                    )))
                }
            }
            (Value::Tuple(ref tuple), Value::Integer(idx)) => {
                let index = *idx as usize;
                if index < tuple.len() {
                    Ok(tuple[index].clone())
                } else {
                    Err(InterpreterError::RuntimeError(format!(
                        "Index {} out of bounds for tuple of length {}",
                        index,
                        tuple.len()
                    )))
                }
            }
            _ => Err(InterpreterError::RuntimeError(format!(
                "Cannot index {} with {}",
                object_value.type_name(),
                index_value.type_name()
            ))),
        }
    }

    fn eval_field_access(&mut self, object: &Expr, field: &str) -> Result<Value, InterpreterError> {
        let object_value = self.eval_expr(object)?;

        match object_value {
            Value::Object(ref object_map) => {
                if let Some(value) = object_map.get(field) {
                    Ok(value.clone())
                } else {
                    Err(InterpreterError::RuntimeError(format!(
                        "Object has no field named '{}'",
                        field
                    )))
                }
            }
            _ => Err(InterpreterError::RuntimeError(format!(
                "Cannot access field '{}' on type {}",
                field,
                object_value.type_name()
            ))),
        }
    }

    fn eval_object_literal(
        &mut self,
        fields: &[crate::frontend::ast::ObjectField],
    ) -> Result<Value, InterpreterError> {
        let mut object = HashMap::new();

        for field in fields {
            match field {
                crate::frontend::ast::ObjectField::KeyValue { key, value } => {
                    let eval_value = self.eval_expr(value)?;
                    object.insert(key.clone(), eval_value);
                }
                crate::frontend::ast::ObjectField::Spread { expr: _ } => {
                    return Err(InterpreterError::RuntimeError(
                        "Spread operator in object literals not yet implemented".to_string(),
                    ));
                }
            }
        }

        Ok(Value::Object(Rc::new(object)))
    }

    fn eval_qualified_name(&self, module: &str, name: &str) -> Result<Value, InterpreterError> {
        if module == "HashMap" && name == "new" {
            Ok(Value::from_string("__builtin_hashmap__".to_string()))
        } else if name == "new" {
            // Check if this is a class constructor call
            if let Ok(class_value) = self.lookup_variable(module) {
                if let Value::Object(ref class_info) = class_value {
                    // Check if it's a class definition
                    if let Some(Value::String(ref type_str)) = class_info.get("__type") {
                        if type_str.as_ref() == "Class" {
                            // Return a special marker for class instantiation
                            return Ok(Value::from_string(format!(
                                "__class_constructor__:{}",
                                module
                            )));
                        }
                    }
                }
            }
            // Check if this is a struct constructor call
            if let Ok(struct_value) = self.lookup_variable(module) {
                if let Value::Object(ref struct_info) = struct_value {
                    // Check if it's a struct definition
                    if let Some(Value::String(ref type_str)) = struct_info.get("__type") {
                        if type_str.as_ref() == "Struct" {
                            // Return a special marker for struct instantiation
                            return Ok(Value::from_string(format!(
                                "__struct_constructor__:{}",
                                module
                            )));
                        }
                    }
                }
            }
            Err(InterpreterError::RuntimeError(format!(
                "Unknown qualified name: {}::{}",
                module, name
            )))
        } else {
            Err(InterpreterError::RuntimeError(format!(
                "Unknown qualified name: {}::{}",
                module, name
            )))
        }
    }

    /// Evaluate a literal value
    fn eval_literal(&self, lit: &Literal) -> Value {
        match lit {
            Literal::Integer(i) => Value::from_i64(*i),
            Literal::Float(f) => Value::from_f64(*f),
            Literal::String(s) => Value::from_string(s.clone()),
            Literal::Bool(b) => Value::from_bool(*b),
            Literal::Char(c) => Value::from_string(c.to_string()),
            Literal::Unit => Value::nil(),
            Literal::Null => Value::nil(),
        }
    }

    /// Look up a variable in the environment (searches from innermost to outermost)
    fn lookup_variable(&self, name: &str) -> Result<Value, InterpreterError> {
        // Check if this is a qualified name (e.g., "Point::new")
        if name.contains("::") {
            let parts: Vec<&str> = name.split("::").collect();
            if parts.len() == 2 && parts[1] == "new" {
                // This is a constructor call pattern
                let class_or_struct_name = parts[0];

                // Look up the class or struct
                for env in self.env_stack.iter().rev() {
                    if let Some(value) = env.get(class_or_struct_name) {
                        if let Value::Object(ref info) = value {
                            // Check if it's a class or struct
                            if let Some(Value::String(ref type_str)) = info.get("__type") {
                                if type_str.as_ref() == "Class" {
                                    return Ok(Value::from_string(format!(
                                        "__class_constructor__:{}",
                                        class_or_struct_name
                                    )));
                                } else if type_str.as_ref() == "Struct" {
                                    return Ok(Value::from_string(format!(
                                        "__struct_constructor__:{}",
                                        class_or_struct_name
                                    )));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Normal variable lookup
        for env in self.env_stack.iter().rev() {
            if let Some(value) = env.get(name) {
                return Ok(value.clone());
            }
        }
        Err(InterpreterError::RuntimeError(format!(
            "Undefined variable: {name}"
        )))
    }

    /// Get the current (innermost) environment
    #[allow(clippy::expect_used)] // Environment stack invariant ensures this never panics
    fn current_env(&self) -> &HashMap<String, Value> {
        self.env_stack
            .last()
            .expect("Environment stack should never be empty")
    }

    /// Set a variable in the current environment
    #[allow(clippy::expect_used)] // Environment stack invariant ensures this never panics
    fn env_set(&mut self, name: String, value: Value) {
        // Record type feedback for optimization
        self.record_variable_assignment_feedback(&name, &value);

        let env = self
            .env_stack
            .last_mut()
            .expect("Environment stack should never be empty");
        env.insert(name, value);
    }

    /// Push a new environment onto the stack
    fn env_push(&mut self, env: HashMap<String, Value>) {
        self.env_stack.push(env);
    }

    /// Pop the current environment from the stack
    fn env_pop(&mut self) -> Option<HashMap<String, Value>> {
        if self.env_stack.len() > 1 {
            // Keep at least the global environment
            self.env_stack.pop()
        } else {
            None
        }
    }

    /// Helper method to call a Value function with arguments (for array methods)
    fn eval_function_call_value(
        &mut self,
        func: &Value,
        args: &[Value],
    ) -> Result<Value, InterpreterError> {
        self.call_function(func.clone(), args)
    }

    /// Call a function with given arguments
    fn call_function(&mut self, func: Value, args: &[Value]) -> Result<Value, InterpreterError> {
        match func {
            Value::String(ref s) if s.starts_with("__class_constructor__:") => {
                // Extract class name from the marker
                let class_name = s.strip_prefix("__class_constructor__:").unwrap();
                self.instantiate_class(class_name, args)
            }
            Value::String(ref s) if s.starts_with("__struct_constructor__:") => {
                // Extract struct name from the marker
                let struct_name = s.strip_prefix("__struct_constructor__:").unwrap();
                self.instantiate_struct_with_args(struct_name, args)
            }
            Value::String(s) if s.starts_with("__builtin_") => {
                // Delegate to extracted builtin module
                match crate::runtime::eval_builtin::eval_builtin_function(&s, args)? {
                    Some(result) => Ok(result),
                    None => Err(InterpreterError::RuntimeError(format!(
                        "Unknown builtin function: {}",
                        s
                    ))),
                }
            }
            Value::Closure { params, body, env } => {
                // Check argument count
                if args.len() != params.len() {
                    return Err(InterpreterError::RuntimeError(format!(
                        "Function expects {} arguments, got {}",
                        params.len(),
                        args.len()
                    )));
                }

                // Create new environment with captured environment as base
                let mut new_env = env.as_ref().clone();

                // Bind parameters to arguments
                for (param, arg) in params.iter().zip(args) {
                    new_env.insert(param.clone(), arg.clone());
                }

                // Push new environment
                self.env_push(new_env);

                // Evaluate function body
                let result = self.eval_expr(&body);

                // Pop environment
                self.env_pop();

                result
            }
            _ => Err(InterpreterError::TypeError(format!(
                "Cannot call non-function value: {}",
                func.type_name()
            ))),
        }
    }

    /// Evaluate a binary operation from AST.
    ///
    /// Dispatches to specialized evaluation functions based on operator type:
    /// - Arithmetic: `+`, `-`, `*`, `/`, `%`, `**`
    /// - Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
    /// - Logical: `&&`, `||`
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Type mismatch (e.g., adding string to number)
    /// - Division by zero
    /// - Unsupported operator
    fn eval_binary_op(
        &self,
        op: AstBinaryOp,
        left: &Value,
        right: &Value,
    ) -> Result<Value, InterpreterError> {
        crate::runtime::eval_operations::eval_binary_op(op, left, right)
    }

    fn eval_unary_op(
        &self,
        op: crate::frontend::ast::UnaryOp,
        operand: &Value,
    ) -> Result<Value, InterpreterError> {
        crate::runtime::eval_operations::eval_unary_op(op, operand)
    }

    /// Evaluate binary expression
    fn eval_binary_expr(
        &mut self,
        left: &Expr,
        op: crate::frontend::ast::BinaryOp,
        right: &Expr,
    ) -> Result<Value, InterpreterError> {
        // Handle short-circuit operators
        match op {
            crate::frontend::ast::BinaryOp::NullCoalesce => {
                let left_val = self.eval_expr(left)?;
                if matches!(left_val, Value::Nil) {
                    self.eval_expr(right)
                } else {
                    Ok(left_val)
                }
            }
            crate::frontend::ast::BinaryOp::And => {
                let left_val = self.eval_expr(left)?;
                if left_val.is_truthy() {
                    self.eval_expr(right)
                } else {
                    Ok(left_val)
                }
            }
            crate::frontend::ast::BinaryOp::Or => {
                let left_val = self.eval_expr(left)?;
                if left_val.is_truthy() {
                    Ok(left_val)
                } else {
                    self.eval_expr(right)
                }
            }
            _ => {
                let left_val = self.eval_expr(left)?;
                let right_val = self.eval_expr(right)?;
                let result = self.eval_binary_op(op, &left_val, &right_val)?;

                // Record type feedback for optimization
                let site_id = left.span.start; // Use span start as site ID
                self.record_binary_op_feedback(site_id, &left_val, &right_val, &result);

                Ok(result)
            }
        }
    }

    /// Evaluate unary expression
    fn eval_unary_expr(
        &mut self,
        op: crate::frontend::ast::UnaryOp,
        operand: &Expr,
    ) -> Result<Value, InterpreterError> {
        let operand_val = self.eval_expr(operand)?;
        self.eval_unary_op(op, &operand_val)
    }

    /// Evaluate if expression
    fn eval_if_expr(
        &mut self,
        condition: &Expr,
        then_branch: &Expr,
        else_branch: Option<&Expr>,
    ) -> Result<Value, InterpreterError> {
        eval_expr::eval_if_expr(condition, then_branch, else_branch, |e| self.eval_expr(e))
    }

    /// Evaluate let expression
    fn eval_let_expr(
        &mut self,
        name: &str,
        value: &Expr,
        body: &Expr,
    ) -> Result<Value, InterpreterError> {
        let val = self.eval_expr(value)?;
        self.env_set(name.to_string(), val.clone());

        // If body is unit (empty), return the value like REPL does
        // This makes `let x = 42` return 42 instead of nil
        match &body.kind {
            ExprKind::Literal(Literal::Unit) => Ok(val),
            _ => self.eval_expr(body),
        }
    }

    /// Evaluate return expression
    fn eval_return_expr(&mut self, value: Option<&Expr>) -> Result<Value, InterpreterError> {
        eval_expr::eval_return_expr(value, |e| self.eval_expr(e))
    }

    /// Evaluate list expression
    fn eval_list_expr(&mut self, elements: &[Expr]) -> Result<Value, InterpreterError> {
        eval_expr::eval_list_expr(elements, |e| self.eval_expr(e))
    }

    /// Evaluate array initialization expression [value; size]
    fn eval_array_init_expr(
        &mut self,
        value_expr: &Expr,
        size_expr: &Expr,
    ) -> Result<Value, InterpreterError> {
        eval_expr::eval_array_init_expr(value_expr, size_expr, |e| self.eval_expr(e))
    }

    /// Evaluate block expression
    fn eval_block_expr(&mut self, statements: &[Expr]) -> Result<Value, InterpreterError> {
        eval_expr::eval_block_expr(statements, |e| self.eval_expr(e))
    }

    /// Evaluate tuple expression
    fn eval_tuple_expr(&mut self, elements: &[Expr]) -> Result<Value, InterpreterError> {
        eval_expr::eval_tuple_expr(elements, |e| self.eval_expr(e))
    }

    /// Evaluate range expression
    fn eval_range_expr(
        &mut self,
        start: &Expr,
        end: &Expr,
        inclusive: bool,
    ) -> Result<Value, InterpreterError> {
        eval_expr::eval_range_expr(start, end, inclusive, |e| self.eval_expr(e))
    }

    /// Helper function for testing - evaluate a string expression via parser
    /// # Errors
    /// Returns error if parsing or evaluation fails
    #[cfg(test)]
    /// Evaluate a string of Ruchy code.
    ///
    /// This convenience function parses and evaluates a string in one step.
    /// It's useful for REPL implementations and testing.
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::interpreter::Interpreter;
    ///
    /// let mut interpreter = Interpreter::new();
    /// let result = interpreter.eval_string("2 * 21").unwrap();
    /// assert_eq!(result.to_string(), "42");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails or if evaluation fails.
    pub fn eval_string(&mut self, input: &str) -> Result<Value, Box<dyn std::error::Error>> {
        use crate::frontend::parser::Parser;

        let mut parser = Parser::new(input);
        let expr = parser.parse_expr()?;

        Ok(self.eval_expr(&expr)?)
    }

    /// Push value onto stack
    /// # Errors
    /// Returns error if stack overflow occurs
    pub fn push(&mut self, value: Value) -> Result<(), InterpreterError> {
        if self.stack.len() >= 10_000 {
            // Stack limit from spec
            return Err(InterpreterError::StackOverflow);
        }
        self.stack.push(value);
        Ok(())
    }

    /// Pop value from stack
    /// # Errors
    /// Returns error if stack underflow occurs
    pub fn pop(&mut self) -> Result<Value, InterpreterError> {
        self.stack.pop().ok_or(InterpreterError::StackUnderflow)
    }

    /// Peek at top of stack without popping
    /// # Errors
    /// Returns error if stack underflow occurs
    pub fn peek(&self, depth: usize) -> Result<Value, InterpreterError> {
        let index = self
            .stack
            .len()
            .checked_sub(depth + 1)
            .ok_or(InterpreterError::StackUnderflow)?;
        Ok(self.stack[index].clone())
    }

    /// Binary arithmetic operation with type checking
    /// # Errors
    /// Returns error if stack underflow, type mismatch, or arithmetic error occurs
    pub fn binary_op(&mut self, op: BinaryOp) -> Result<(), InterpreterError> {
        let right = self.pop()?;
        let left = self.pop()?;

        let result = match op {
            BinaryOp::Add => eval_operations::eval_binary_op(AstBinaryOp::Add, &left, &right)?,
            BinaryOp::Sub => eval_operations::eval_binary_op(AstBinaryOp::Subtract, &left, &right)?,
            BinaryOp::Mul => eval_operations::eval_binary_op(AstBinaryOp::Multiply, &left, &right)?,
            BinaryOp::Div => eval_operations::eval_binary_op(AstBinaryOp::Divide, &left, &right)?,
            BinaryOp::Eq => eval_operations::eval_binary_op(AstBinaryOp::Equal, &left, &right)?,
            BinaryOp::Lt => eval_operations::eval_binary_op(AstBinaryOp::Less, &left, &right)?,
            BinaryOp::Gt => eval_operations::eval_binary_op(AstBinaryOp::Greater, &left, &right)?,
        };

        self.push(result)?;
        Ok(())
    }

    /// Set a variable in the current scope (public for try/catch)
    pub fn set_variable_string(&mut self, name: String, value: Value) {
        self.env_set(name, value);
    }

    /// Apply a binary operation to two values
    fn apply_binary_op(
        &self,
        left: &Value,
        op: AstBinaryOp,
        right: &Value,
    ) -> Result<Value, InterpreterError> {
        // Delegate to existing binary operation evaluation
        self.eval_binary_op(op, left, right)
    }

    /// Check if a pattern matches a value
    /// # Errors
    /// Returns error if pattern matching fails
    /// Try to match a pattern against a value, returning bindings if successful
    fn try_pattern_match(
        &self,
        pattern: &Pattern,
        value: &Value,
    ) -> Result<Option<Vec<(String, Value)>>, InterpreterError> {
        crate::runtime::eval_pattern_match::try_pattern_match(pattern, value, &|lit| {
            self.eval_literal(lit)
        })
    }

    /// Legacy method for backwards compatibility
    fn pattern_matches_internal(
        &self,
        pattern: &Pattern,
        value: &Value,
    ) -> Result<bool, InterpreterError> {
        crate::runtime::eval_pattern_match::pattern_matches(pattern, value, &|lit| {
            self.eval_literal(lit)
        })
    }

    /// Scope management for pattern bindings
    pub fn push_scope(&mut self) {
        let new_env = HashMap::new();
        self.env_push(new_env);
    }

    pub fn pop_scope(&mut self) {
        self.env_pop();
    }

    /// New pattern matching methods that return bindings

    // Helper methods for pattern matching (complexity <10 each)

    fn match_tuple_pattern(
        &self,
        patterns: &[Pattern],
        value: &Value,
    ) -> Result<bool, InterpreterError> {
        crate::runtime::eval_pattern_match::match_tuple_pattern(patterns, value, |lit| {
            self.eval_literal(lit)
        })
    }

    fn match_list_pattern(
        &self,
        patterns: &[Pattern],
        value: &Value,
    ) -> Result<bool, InterpreterError> {
        crate::runtime::eval_pattern_match::match_list_pattern(patterns, value, |lit| {
            self.eval_literal(lit)
        })
    }

    fn match_or_pattern(
        &self,
        patterns: &[Pattern],
        value: &Value,
    ) -> Result<bool, InterpreterError> {
        crate::runtime::eval_pattern_match::match_or_pattern(patterns, value, |lit| {
            self.eval_literal(lit)
        })
    }

    /// Access field with inline caching optimization
    /// # Errors
    /// Returns error if field access fails
    pub fn get_field_cached(
        &mut self,
        obj: &Value,
        field_name: &str,
    ) -> Result<Value, InterpreterError> {
        // Create cache key combining object type and field name
        let cache_key = format!("{:?}::{}", obj.type_id(), field_name);

        // Check inline cache first
        if let Some(cache) = self.field_caches.get_mut(&cache_key) {
            if let Some(cached_result) = cache.lookup(obj, field_name) {
                return Ok(cached_result);
            }
        }

        // Cache miss - compute result and update cache
        let result = self.compute_field_access(obj, field_name)?;

        // Update or create cache entry
        let cache = self.field_caches.entry(cache_key).or_default();
        cache.insert(obj, field_name.to_string(), result.clone());

        Ok(result)
    }

    /// Compute field access result (detailed path)
    fn compute_field_access(
        &self,
        obj: &Value,
        field_name: &str,
    ) -> Result<Value, InterpreterError> {
        match (obj, field_name) {
            // String methods
            (Value::String(s), "len") => Ok(Value::Integer(s.len().try_into().unwrap_or(i64::MAX))),
            (Value::String(s), "to_upper") => Ok(Value::from_string(s.to_uppercase())),
            (Value::String(s), "to_lower") => Ok(Value::from_string(s.to_lowercase())),
            (Value::String(s), "trim") => Ok(Value::from_string(s.trim().to_string())),

            // Array methods
            (Value::Array(arr), "len") => {
                Ok(Value::Integer(arr.len().try_into().unwrap_or(i64::MAX)))
            }
            (Value::Array(arr), "first") => arr
                .first()
                .cloned()
                .ok_or_else(|| InterpreterError::RuntimeError("Array is empty".to_string())),
            (Value::Array(arr), "last") => arr
                .last()
                .cloned()
                .ok_or_else(|| InterpreterError::RuntimeError("Array is empty".to_string())),
            (Value::Array(arr), "is_empty") => Ok(Value::from_bool(arr.is_empty())),

            // Type information
            (obj, "type") => Ok(Value::from_string(obj.type_name().to_string())),

            _ => Err(InterpreterError::RuntimeError(format!(
                "Field '{}' not found on type '{}'",
                field_name,
                obj.type_name()
            ))),
        }
    }

    /// Get inline cache statistics for profiling
    pub fn get_cache_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        for (key, cache) in &self.field_caches {
            stats.insert(key.clone(), cache.hit_rate());
        }
        stats
    }

    /// Clear all inline caches (for testing)
    pub fn clear_caches(&mut self) {
        self.field_caches.clear();
    }

    /// Record type feedback for binary operations
    #[allow(dead_code)] // Used by tests and type feedback system
    fn record_binary_op_feedback(
        &mut self,
        site_id: usize,
        left: &Value,
        right: &Value,
        result: &Value,
    ) {
        self.type_feedback
            .record_binary_op(site_id, left, right, result);
    }

    /// Record type feedback for variable assignments
    #[allow(dead_code)] // Used by tests and type feedback system
    fn record_variable_assignment_feedback(&mut self, var_name: &str, value: &Value) {
        let type_id = value.type_id();
        self.type_feedback
            .record_variable_assignment(var_name, type_id);
    }

    /// Record type feedback for function calls
    fn record_function_call_feedback(
        &mut self,
        site_id: usize,
        func_name: &str,
        args: &[Value],
        result: &Value,
    ) {
        self.type_feedback
            .record_function_call(site_id, func_name, args, result);
    }

    /// Get type feedback statistics
    pub fn get_type_feedback_stats(&self) -> TypeFeedbackStats {
        self.type_feedback.get_statistics()
    }

    /// Get specialization candidates for JIT compilation
    pub fn get_specialization_candidates(&self) -> Vec<SpecializationCandidate> {
        self.type_feedback.get_specialization_candidates()
    }

    /// Clear type feedback data (for testing)
    pub fn clear_type_feedback(&mut self) {
        self.type_feedback = TypeFeedback::new();
    }

    /// Track a value in the garbage collector
    pub fn gc_track(&mut self, value: Value) -> usize {
        self.gc.track_object(value)
    }

    /// Force garbage collection
    pub fn gc_collect(&mut self) -> GCStats {
        self.gc.force_collect()
    }

    /// Get garbage collection statistics
    pub fn gc_stats(&self) -> GCStats {
        self.gc.get_stats()
    }

    /// Get detailed garbage collection information
    pub fn gc_info(&self) -> GCInfo {
        self.gc.get_info()
    }

    /// Set garbage collection threshold
    pub fn gc_set_threshold(&mut self, threshold: usize) {
        self.gc.set_collection_threshold(threshold);
    }

    /// Enable or disable automatic garbage collection
    pub fn gc_set_auto_collect(&mut self, enabled: bool) {
        self.gc.set_auto_collect(enabled);
    }

    /// Clear all GC-tracked objects (for testing)
    pub fn gc_clear(&mut self) {
        self.gc.clear();
    }

    /// Allocate a new array and track it in GC
    pub fn gc_alloc_array(&mut self, elements: Vec<Value>) -> Value {
        let array_value = Value::from_array(elements);
        self.gc.track_object(array_value.clone());
        array_value
    }

    /// Allocate a new string and track it in GC
    pub fn gc_alloc_string(&mut self, content: String) -> Value {
        let string_value = Value::from_string(content);
        self.gc.track_object(string_value.clone());
        string_value
    }

    /// Allocate a new closure and track it in GC
    pub fn gc_alloc_closure(
        &mut self,
        params: Vec<String>,
        body: Rc<Expr>,
        env: Rc<HashMap<String, Value>>,
    ) -> Value {
        let closure_value = Value::Closure { params, body, env };
        self.gc.track_object(closure_value.clone());
        closure_value
    }

    // ========================================================================
    // Public methods for SharedSession integration
    // ========================================================================

    /// Get all bindings from the global environment (for `SharedSession` state persistence)
    pub fn get_global_bindings(&self) -> HashMap<String, Value> {
        if let Some(global_env) = self.env_stack.first() {
            global_env.clone()
        } else {
            HashMap::new()
        }
    }

    /// Set a binding in the global environment (for `SharedSession` state restoration)
    pub fn set_global_binding(&mut self, name: String, value: Value) {
        if let Some(global_env) = self.env_stack.first_mut() {
            global_env.insert(name, value);
        }
    }

    /// Clear all user variables from global environment, keeping only builtins
    pub fn clear_user_variables(&mut self) {
        if let Some(global_env) = self.env_stack.first_mut() {
            // Keep only builtin functions (those starting with "__builtin_") and nil
            global_env.retain(|name, _| name.starts_with("__builtin_") || name == "nil");
        }
    }

    /// Get all bindings from the current environment (for `SharedSession` extraction)
    pub fn get_current_bindings(&self) -> HashMap<String, Value> {
        if let Some(current_env) = self.env_stack.last() {
            current_env.clone()
        } else {
            HashMap::new()
        }
    }

    /// Evaluate a for loop
    fn eval_for_loop(
        &mut self,
        var: &str,
        _pattern: Option<&Pattern>,
        iter: &Expr,
        body: &Expr,
    ) -> Result<Value, InterpreterError> {
        let iter_value = self.eval_expr(iter)?;

        match iter_value {
            Value::Array(ref arr) => self.eval_for_array_iteration(var, arr, body),
            Value::Range {
                ref start,
                ref end,
                inclusive,
            } => self.eval_for_range_iteration(var, start, end, inclusive, body),
            _ => Err(InterpreterError::TypeError(
                "For loop requires an iterable".to_string(),
            )),
        }
    }

    /// Evaluate for loop iteration over an array
    /// Complexity: ≤8
    fn eval_for_array_iteration(
        &mut self,
        var: &str,
        arr: &[Value],
        body: &Expr,
    ) -> Result<Value, InterpreterError> {
        let mut last_value = Value::nil();

        for item in arr {
            self.set_variable(var, item.clone());
            match self.eval_loop_body_with_control_flow(body) {
                Ok(value) => last_value = value,
                Err(LoopControlOrError::Break(val)) => return Ok(val),
                Err(LoopControlOrError::Continue) => {}
                Err(LoopControlOrError::Error(e)) => return Err(e),
            }
        }

        Ok(last_value)
    }

    /// Evaluate for loop iteration over a range
    /// Complexity: ≤9
    fn eval_for_range_iteration(
        &mut self,
        var: &str,
        start: &Value,
        end: &Value,
        inclusive: bool,
        body: &Expr,
    ) -> Result<Value, InterpreterError> {
        let (start_val, end_val) = self.extract_range_bounds(start, end)?;
        let mut last_value = Value::nil();

        for i in self.create_range_iterator(start_val, end_val, inclusive) {
            self.set_variable(var, Value::Integer(i));
            match self.eval_loop_body_with_control_flow(body) {
                Ok(value) => last_value = value,
                Err(LoopControlOrError::Break(val)) => return Ok(val),
                Err(LoopControlOrError::Continue) => {}
                Err(LoopControlOrError::Error(e)) => return Err(e),
            }
        }

        Ok(last_value)
    }

    /// Extract integer bounds from range values
    /// Complexity: ≤3
    fn extract_range_bounds(
        &self,
        start: &Value,
        end: &Value,
    ) -> Result<(i64, i64), InterpreterError> {
        match (start, end) {
            (Value::Integer(s), Value::Integer(e)) => Ok((*s, *e)),
            _ => Err(InterpreterError::TypeError(
                "Range bounds must be integers".to_string(),
            )),
        }
    }

    /// Create range iterator based on inclusive flag
    /// Complexity: ≤2
    fn create_range_iterator(
        &self,
        start: i64,
        end: i64,
        inclusive: bool,
    ) -> Box<dyn Iterator<Item = i64>> {
        if inclusive {
            Box::new(start..=end)
        } else {
            Box::new(start..end)
        }
    }

    /// Evaluate loop body with control flow handling
    /// Complexity: ≤5
    fn eval_loop_body_with_control_flow(
        &mut self,
        body: &Expr,
    ) -> Result<Value, LoopControlOrError> {
        match self.eval_expr(body) {
            Ok(value) => Ok(value),
            Err(InterpreterError::Break(val)) => Err(LoopControlOrError::Break(val)),
            Err(InterpreterError::Continue) => Err(LoopControlOrError::Continue),
            Err(e) => Err(LoopControlOrError::Error(e)),
        }
    }

    /// Evaluate a while loop
    fn eval_while_loop(
        &mut self,
        condition: &Expr,
        body: &Expr,
    ) -> Result<Value, InterpreterError> {
        crate::runtime::eval_loops::eval_while_loop(condition, body, |expr| self.eval_expr(expr))
    }

    /// Evaluate a match expression
    fn eval_match(&mut self, expr: &Expr, arms: &[MatchArm]) -> Result<Value, InterpreterError> {
        let value = self.eval_expr(expr)?;

        for arm in arms {
            // First check if pattern matches
            if let Some(bindings) = self.try_pattern_match(&arm.pattern, &value)? {
                // Create new scope for pattern bindings
                self.push_scope();

                // Bind pattern variables
                for (name, val) in bindings {
                    self.env_set(name, val);
                }

                // Check guard condition if present
                let guard_passed = if let Some(guard) = &arm.guard {
                    match self.eval_expr(guard)? {
                        Value::Bool(true) => true,
                        Value::Bool(false) => false,
                        _ => {
                            self.pop_scope();
                            return Err(InterpreterError::RuntimeError(
                                "Guard condition must evaluate to a boolean".to_string(),
                            ));
                        }
                    }
                } else {
                    true // No guard means always pass
                };

                if guard_passed {
                    // Evaluate body with bindings in scope
                    let result = self.eval_expr(&arm.body);
                    self.pop_scope();
                    return result;
                }
                // Guard failed, restore scope and try next arm
                self.pop_scope();
            }
        }

        Err(InterpreterError::RuntimeError(
            "No match arm matched the value".to_string(),
        ))
    }

    /// Evaluate a let pattern expression (array/tuple destructuring)
    fn eval_let_pattern(
        &mut self,
        pattern: &Pattern,
        value: &Expr,
        body: &Expr,
    ) -> Result<Value, InterpreterError> {
        // Evaluate the right-hand side value
        let rhs_value = self.eval_expr(value)?;

        // Try to match the pattern against the value
        if let Some(bindings) = self.try_pattern_match(pattern, &rhs_value)? {
            // Bind pattern variables directly to current scope (like regular let)
            for (name, val) in bindings {
                self.env_set(name, val);
            }

            // If body is unit (empty), return the value like REPL does
            // This makes `let [a, b] = [1, 2]` return [1, 2] instead of nil
            match &body.kind {
                ExprKind::Literal(Literal::Unit) => Ok(rhs_value),
                _ => self.eval_expr(body),
            }
        } else {
            Err(InterpreterError::RuntimeError(
                "Pattern did not match the value".to_string(),
            ))
        }
    }

    /// Evaluate an assignment
    fn eval_assign(&mut self, target: &Expr, value: &Expr) -> Result<Value, InterpreterError> {
        let val = self.eval_expr(value)?;

        // Handle different assignment targets
        match &target.kind {
            ExprKind::Identifier(name) => {
                self.set_variable(name, val.clone());
                Ok(val)
            }
            _ => Err(InterpreterError::RuntimeError(
                "Invalid assignment target".to_string(),
            )),
        }
    }

    /// Evaluate a compound assignment
    fn eval_compound_assign(
        &mut self,
        target: &Expr,
        op: AstBinaryOp,
        value: &Expr,
    ) -> Result<Value, InterpreterError> {
        // Get current value
        let current = match &target.kind {
            ExprKind::Identifier(name) => self.lookup_variable(name)?,
            _ => {
                return Err(InterpreterError::RuntimeError(
                    "Invalid compound assignment target".to_string(),
                ))
            }
        };

        // Compute new value
        let rhs = self.eval_expr(value)?;
        let new_val = self.apply_binary_op(&current, op, &rhs)?;

        // Assign back
        if let ExprKind::Identifier(name) = &target.kind {
            self.set_variable(name, new_val.clone());
        }

        Ok(new_val)
    }

    /// Evaluate string methods
    fn eval_string_method(
        &mut self,
        s: &Rc<str>,
        method: &str,
        args: &[Value],
    ) -> Result<Value, InterpreterError> {
        super::eval_string_methods::eval_string_method(s, method, args)
    }

    /// Evaluate array methods
    #[allow(clippy::rc_buffer)]
    fn eval_array_method(
        &mut self,
        arr: &Rc<[Value]>,
        method: &str,
        args: &[Value],
    ) -> Result<Value, InterpreterError> {
        // Delegate to extracted array module with function call capability
        crate::runtime::eval_array::eval_array_method(arr, method, args, |func, args| {
            self.eval_function_call_value(func, args)
        })
    }

    /// Evaluate a method call
    fn eval_method_call(
        &mut self,
        receiver: &Expr,
        method: &str,
        args: &[Expr],
    ) -> Result<Value, InterpreterError> {
        let receiver_value = self.eval_expr(receiver)?;

        // Special handling for DataFrame filter method - don't pre-evaluate the condition
        if matches!(receiver_value, Value::DataFrame { .. }) && method == "filter" {
            return self.eval_dataframe_filter_method(&receiver_value, args);
        }

        let arg_values: Result<Vec<_>, _> = args.iter().map(|arg| self.eval_expr(arg)).collect();
        let arg_values = arg_values?;

        self.dispatch_method_call(&receiver_value, method, &arg_values, args.is_empty())
    }

    // Helper methods for method dispatch (complexity <10 each)

    fn dispatch_method_call(
        &mut self,
        receiver: &Value,
        method: &str,
        arg_values: &[Value],
        args_empty: bool,
    ) -> Result<Value, InterpreterError> {
        match receiver {
            Value::String(s) => self.eval_string_method(s, method, arg_values),
            Value::Array(arr) => self.eval_array_method(arr, method, arg_values),
            Value::Float(f) => self.eval_float_method(*f, method, args_empty),
            Value::Integer(n) => self.eval_integer_method(*n, method, args_empty),
            Value::DataFrame { columns } => self.eval_dataframe_method(columns, method, arg_values),
            Value::Object(obj) => self.eval_object_method(obj, method, arg_values, args_empty),
            _ => self.eval_generic_method(receiver, method, args_empty),
        }
    }

    fn eval_float_method(
        &self,
        f: f64,
        method: &str,
        args_empty: bool,
    ) -> Result<Value, InterpreterError> {
        eval_method::eval_float_method(f, method, args_empty)
    }

    fn eval_integer_method(
        &self,
        n: i64,
        method: &str,
        args_empty: bool,
    ) -> Result<Value, InterpreterError> {
        eval_method::eval_integer_method(n, method, args_empty)
    }

    fn eval_generic_method(
        &self,
        receiver: &Value,
        method: &str,
        args_empty: bool,
    ) -> Result<Value, InterpreterError> {
        eval_method::eval_generic_method(receiver, method, args_empty)
    }

    fn eval_object_method(
        &self,
        obj: &std::collections::HashMap<String, Value>,
        method: &str,
        arg_values: &[Value],
        args_empty: bool,
    ) -> Result<Value, InterpreterError> {
        use crate::runtime::eval_method_dispatch;
        eval_method_dispatch::eval_method_call(
            &Value::Object(std::rc::Rc::new(obj.clone())),
            method,
            arg_values,
            args_empty,
            |_receiver, _args| {
                Err(InterpreterError::RuntimeError(
                    "Function call not implemented in actor context".to_string(),
                ))
            },
            |_receiver, _args| {
                Err(InterpreterError::RuntimeError(
                    "DataFrame filter not implemented in actor context".to_string(),
                ))
            },
            |_expr, _columns, _index| {
                Err(InterpreterError::RuntimeError(
                    "Column context not implemented in actor context".to_string(),
                ))
            },
        )
    }

    /// Evaluate actor definition
    ///
    /// Actors are first-class values that can be instantiated and receive messages.
    /// For now, we return an actor type object that can be used to create instances.
    fn eval_actor_definition(
        &mut self,
        name: &str,
        state: &[crate::frontend::ast::StructField],
        handlers: &[crate::frontend::ast::ActorHandler],
    ) -> Result<Value, InterpreterError> {
        use std::collections::HashMap;

        // Create an actor type object
        let mut actor_type = HashMap::new();

        // Store actor metadata
        actor_type.insert(
            "__type".to_string(),
            Value::from_string("Actor".to_string()),
        );
        actor_type.insert("__name".to_string(), Value::from_string(name.to_string()));

        // Store state field definitions
        let mut fields = HashMap::new();
        for field in state {
            // For now, just store field names and types as strings
            let type_name = match &field.ty.kind {
                crate::frontend::ast::TypeKind::Named(n) => n.clone(),
                _ => "Any".to_string(),
            };
            fields.insert(field.name.clone(), Value::from_string(type_name));
        }
        actor_type.insert(
            "__fields".to_string(),
            Value::Object(std::rc::Rc::new(fields)),
        );

        // Store message handlers
        let mut handlers_map = HashMap::new();
        for handler in handlers {
            // Store handler information for later use
            handlers_map.insert(
                handler.message_type.clone(),
                Value::from_string("handler".to_string()), // Placeholder for now
            );
        }
        actor_type.insert(
            "__handlers".to_string(),
            Value::Object(std::rc::Rc::new(handlers_map)),
        );

        // Register this actor type in the environment
        let actor_obj = Value::Object(std::rc::Rc::new(actor_type));
        self.set_variable(name, actor_obj.clone());

        Ok(actor_obj)
    }

    /// Evaluate struct definition
    /// Creates a struct type descriptor that can be used for instantiation
    /// Complexity: 7
    fn eval_struct_definition(
        &mut self,
        name: &str,
        _type_params: &[String], // TODO: Generic type parameters
        fields: &[crate::frontend::ast::StructField],
        _is_pub: bool,
    ) -> Result<Value, InterpreterError> {
        use std::collections::HashMap;

        // Create a struct type object
        let mut struct_type = HashMap::new();

        // Store struct metadata
        struct_type.insert(
            "__type".to_string(),
            Value::from_string("Struct".to_string()),
        );
        struct_type.insert("__name".to_string(), Value::from_string(name.to_string()));

        // Store field definitions
        let mut field_defs = HashMap::new();
        for field in fields {
            // Store field type information
            let type_name = match &field.ty.kind {
                crate::frontend::ast::TypeKind::Named(n) => n.clone(),
                crate::frontend::ast::TypeKind::Array { .. } => "Array".to_string(),
                crate::frontend::ast::TypeKind::Optional(_) => "Option".to_string(),
                crate::frontend::ast::TypeKind::List(_) => "List".to_string(),
                crate::frontend::ast::TypeKind::Tuple(_) => "Tuple".to_string(),
                _ => "Any".to_string(),
            };

            let mut field_info = HashMap::new();
            field_info.insert("type".to_string(), Value::from_string(type_name));
            field_info.insert("is_pub".to_string(), Value::from_bool(field.is_pub));
            field_info.insert("is_mut".to_string(), Value::from_bool(field.is_mut));

            field_defs.insert(
                field.name.clone(),
                Value::Object(std::rc::Rc::new(field_info)),
            );
        }

        struct_type.insert(
            "__fields".to_string(),
            Value::Object(std::rc::Rc::new(field_defs)),
        );

        // Register this struct type in the environment
        let struct_obj = Value::Object(std::rc::Rc::new(struct_type));
        self.set_variable(name, struct_obj.clone());

        Ok(struct_obj)
    }

    /// Evaluate struct literal (instantiation)
    /// Creates an instance of a struct with provided field values
    /// Complexity: 8
    fn eval_struct_literal(
        &mut self,
        name: &str,
        fields: &[(String, crate::frontend::ast::Expr)],
    ) -> Result<Value, InterpreterError> {
        use std::collections::HashMap;

        // Look up the struct type definition
        let struct_type = self.lookup_variable(name).map_err(|_| {
            InterpreterError::RuntimeError(format!("Undefined struct type: {name}"))
        })?;

        // Verify it's actually a struct type
        let struct_type_obj = if let Value::Object(obj) = &struct_type {
            obj
        } else {
            return Err(InterpreterError::RuntimeError(format!(
                "{name} is not a struct type"
            )));
        };

        // Verify it's a struct type (not actor or other type)
        let type_name = struct_type_obj
            .get("__type")
            .and_then(|v| {
                if let Value::String(s) = v {
                    Some(s.as_ref())
                } else {
                    None
                }
            })
            .unwrap_or("");

        if type_name != "Struct" {
            return Err(InterpreterError::RuntimeError(format!(
                "{name} is not a struct type (it's a {type_name})"
            )));
        }

        // Get field definitions
        let field_defs = struct_type_obj
            .get("__fields")
            .and_then(|v| {
                if let Value::Object(obj) = v {
                    Some(obj)
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                InterpreterError::RuntimeError(format!("Invalid struct type definition for {name}"))
            })?;

        // Create struct instance
        let mut instance = HashMap::new();

        // Add metadata
        instance.insert(
            "__struct_type".to_string(),
            Value::from_string(name.to_string()),
        );

        // Evaluate and set field values
        for (field_name, field_expr) in fields {
            // Verify field exists in struct definition
            if !field_defs.contains_key(field_name) {
                return Err(InterpreterError::RuntimeError(format!(
                    "Struct {name} does not have field '{field_name}'"
                )));
            }

            // Evaluate field value
            let field_value = self.eval_expr(field_expr)?;
            instance.insert(field_name.clone(), field_value);
        }

        // Check that all required fields are provided
        for field_name in field_defs.keys() {
            if !instance.contains_key(field_name) && field_name != "__struct_type" {
                return Err(InterpreterError::RuntimeError(format!(
                    "Missing required field '{field_name}' for struct {name}"
                )));
            }
        }

        Ok(Value::Object(std::rc::Rc::new(instance)))
    }

    /// Evaluate class definition (placeholder for now)
    /// Complexity: 5
    fn eval_class_definition(
        &mut self,
        name: &str,
        _type_params: &[String],
        superclass: Option<&String>,
        _traits: &[String],
        fields: &[crate::frontend::ast::StructField],
        constructors: &[crate::frontend::ast::Constructor],
        methods: &[crate::frontend::ast::ClassMethod],
        _derives: &[String],
        _is_pub: bool,
    ) -> Result<Value, InterpreterError> {
        use std::collections::HashMap;
        use std::rc::Rc;

        // Create class metadata object
        let mut class_info = HashMap::new();

        // Mark as class type
        class_info.insert(
            "__type".to_string(),
            Value::from_string("Class".to_string()),
        );
        class_info.insert("__name".to_string(), Value::from_string(name.to_string()));

        // Store superclass if present
        if let Some(parent) = superclass {
            class_info.insert(
                "__superclass".to_string(),
                Value::from_string(parent.clone()),
            );
        }

        // Store field definitions (similar to struct)
        let mut field_defs = HashMap::new();
        for field in fields {
            let mut field_info = HashMap::new();

            // Store field type
            let type_str = format!("{:?}", field.ty);
            field_info.insert("type".to_string(), Value::from_string(type_str));

            // Store visibility
            field_info.insert("is_pub".to_string(), Value::Bool(field.is_pub));
            field_info.insert("is_mut".to_string(), Value::Bool(field.is_mut));

            // Store default value if present
            if let Some(ref default) = field.default_value {
                // Evaluate default value
                let default_val = self.eval_expr(default)?;
                field_info.insert("default".to_string(), default_val);
            }

            field_defs.insert(
                field.name.clone(),
                Value::Object(std::rc::Rc::new(field_info)),
            );
        }
        class_info.insert("__fields".to_string(), Value::Object(Rc::new(field_defs)));

        // Store constructors
        let mut constructor_info = HashMap::new();
        for constructor in constructors {
            // Store constructor by name (default name is "new")
            let ctor_name = constructor
                .name
                .as_ref()
                .unwrap_or(&"new".to_string())
                .clone();

            // Create constructor metadata
            let mut ctor_meta = HashMap::new();
            ctor_meta.insert(
                "params".to_string(),
                Value::from_string(format!("{:?}", constructor.params)),
            );
            ctor_meta.insert(
                "body".to_string(),
                Value::from_string(format!("{:?}", constructor.body)),
            );

            constructor_info.insert(ctor_name, Value::Object(Rc::new(ctor_meta)));
        }
        class_info.insert(
            "__constructors".to_string(),
            Value::Object(Rc::new(constructor_info)),
        );

        // Store methods
        let mut method_info = HashMap::new();
        for method in methods {
            let mut method_meta = HashMap::new();
            method_meta.insert("name".to_string(), Value::from_string(method.name.clone()));
            method_meta.insert("is_static".to_string(), Value::Bool(method.is_static));
            method_meta.insert("is_override".to_string(), Value::Bool(method.is_override));
            method_meta.insert(
                "params".to_string(),
                Value::from_string(format!("{:?}", method.params)),
            );
            method_meta.insert(
                "body".to_string(),
                Value::from_string(format!("{:?}", method.body)),
            );

            method_info.insert(method.name.clone(), Value::Object(Rc::new(method_meta)));
        }
        class_info.insert("__methods".to_string(), Value::Object(Rc::new(method_info)));

        // Store the class definition in the environment
        let class_value = Value::Object(Rc::new(class_info));
        self.set_variable(name, class_value.clone());

        Ok(class_value)
    }

    fn instantiate_class(
        &mut self,
        class_name: &str,
        _args: &[Value],
    ) -> Result<Value, InterpreterError> {
        // Look up the class definition
        let class_def = self.lookup_variable(class_name)?;

        if let Value::Object(ref class_info) = class_def {
            // Verify this is a class
            if let Some(Value::String(ref type_str)) = class_info.get("__type") {
                if type_str.as_ref() != "Class" {
                    return Err(InterpreterError::RuntimeError(format!(
                        "{} is not a class",
                        class_name
                    )));
                }
            }

            // Create instance object
            let mut instance = HashMap::new();
            instance.insert(
                "__class".to_string(),
                Value::from_string(class_name.to_string()),
            );

            // Initialize fields with default values or provided arguments
            if let Some(Value::Object(ref fields)) = class_info.get("__fields") {
                for (field_name, field_info) in fields.iter() {
                    if let Value::Object(ref field_meta) = field_info {
                        // Use default value if present
                        if let Some(default) = field_meta.get("default") {
                            instance.insert(field_name.clone(), default.clone());
                        } else {
                            // For now, initialize with nil
                            instance.insert(field_name.clone(), Value::Nil);
                        }
                    }
                }
            }

            // TODO: Execute the constructor (init method) with provided arguments
            // For now, we're just creating the instance with default values

            Ok(Value::Object(Rc::new(instance)))
        } else {
            Err(InterpreterError::RuntimeError(format!(
                "{} is not a class definition",
                class_name
            )))
        }
    }

    fn instantiate_struct_with_args(
        &mut self,
        struct_name: &str,
        args: &[Value],
    ) -> Result<Value, InterpreterError> {
        // Look up the struct definition
        let struct_def = self.lookup_variable(struct_name)?;

        if let Value::Object(ref struct_info) = struct_def {
            // Verify this is a struct
            if let Some(Value::String(ref type_str)) = struct_info.get("__type") {
                if type_str.as_ref() != "Struct" {
                    return Err(InterpreterError::RuntimeError(format!(
                        "{} is not a struct",
                        struct_name
                    )));
                }
            }

            // For structs with positional arguments, we need to map them to fields
            // This is a simplified version - real implementation would need parameter names
            // For now, create an empty struct instance
            let mut instance = HashMap::new();
            instance.insert(
                "__struct".to_string(),
                Value::from_string(struct_name.to_string()),
            );

            // Initialize fields with default values
            if let Some(Value::Object(ref fields)) = struct_info.get("__fields") {
                // Map positional arguments to fields (assuming order matches definition)
                for (i, (field_name, field_info)) in fields.iter().enumerate() {
                    if i < args.len() {
                        instance.insert(field_name.clone(), args[i].clone());
                    } else if let Value::Object(ref field_meta) = field_info {
                        // Use default value if present
                        if let Some(default) = field_meta.get("default") {
                            instance.insert(field_name.clone(), default.clone());
                        } else {
                            // Initialize with default for type
                            instance.insert(field_name.clone(), Value::Nil);
                        }
                    }
                }
            }

            Ok(Value::Object(Rc::new(instance)))
        } else {
            Err(InterpreterError::RuntimeError(format!(
                "{} is not a struct definition",
                struct_name
            )))
        }
    }

    fn eval_dataframe_method(
        &self,
        columns: &[DataFrameColumn],
        method: &str,
        arg_values: &[Value],
    ) -> Result<Value, InterpreterError> {
        crate::runtime::eval_dataframe_ops::eval_dataframe_method(columns, method, arg_values)
    }
    /// Special handler for `DataFrame` filter method
    /// Special handler for `DataFrame` filter method
    fn eval_dataframe_filter_method(
        &mut self,
        receiver: &Value,
        args: &[Expr],
    ) -> Result<Value, InterpreterError> {
        if args.len() != 1 {
            return Err(InterpreterError::RuntimeError(
                "DataFrame.filter() requires exactly 1 argument (condition)".to_string(),
            ));
        }

        if let Value::DataFrame { columns } = receiver {
            let condition = &args[0];
            crate::runtime::eval_dataframe_ops::eval_dataframe_filter(
                columns,
                condition,
                |expr, cols, idx| self.eval_expr_with_column_context(expr, cols, idx),
            )
        } else {
            Err(InterpreterError::RuntimeError(
                "filter method can only be called on DataFrame".to_string(),
            ))
        }
    }

    /// Compare two values using a comparison function
    fn compare_values<F>(
        &self,
        left: &Value,
        right: &Value,
        cmp: F,
    ) -> Result<Value, InterpreterError>
    where
        F: Fn(i64, i64) -> bool,
    {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(cmp(*a, *b))),
            (Value::Float(a), Value::Float(b)) => {
                // Convert float comparison to integer-like for simplicity
                let a_int = *a as i64;
                let b_int = *b as i64;
                Ok(Value::Bool(cmp(a_int, b_int)))
            }
            (Value::Integer(a), Value::Float(b)) => {
                let b_int = *b as i64;
                Ok(Value::Bool(cmp(*a, b_int)))
            }
            (Value::Float(a), Value::Integer(b)) => {
                let a_int = *a as i64;
                Ok(Value::Bool(cmp(a_int, *b)))
            }
            _ => Err(InterpreterError::RuntimeError(format!(
                "Cannot compare {} and {}",
                left.type_name(),
                right.type_name()
            ))),
        }
    }

    /// Check if two values are equal
    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }

    /// Evaluate an expression with column context (for `DataFrame` filtering)
    fn eval_expr_with_column_context(
        &mut self,
        expr: &Expr,
        columns: &[DataFrameColumn],
        row_idx: usize,
    ) -> Result<Value, InterpreterError> {
        match &expr.kind {
            // Special handling for function calls that might be col() references
            ExprKind::Call { func, args } => {
                if let ExprKind::Identifier(name) = &func.kind {
                    if name == "col" && args.len() == 1 {
                        // This is a col("column_name") call - resolve to actual column value
                        let col_name_expr = &args[0];
                        if let ExprKind::Literal(crate::frontend::ast::Literal::String(col_name)) =
                            &col_name_expr.kind
                        {
                            // Find the column and return the value for this row
                            for col in columns {
                                if col.name == *col_name {
                                    if let Some(value) = col.values.get(row_idx) {
                                        return Ok(value.clone());
                                    }
                                    return Err(InterpreterError::RuntimeError(format!(
                                        "Row index {} out of bounds for column '{}'",
                                        row_idx, col_name
                                    )));
                                }
                            }
                            return Err(InterpreterError::RuntimeError(format!(
                                "Column '{}' not found",
                                col_name
                            )));
                        }
                    }
                }
                // Fall back to normal function call evaluation
                self.eval_expr(expr)
            }
            // Handle binary expressions that might need column context
            ExprKind::Binary { left, right, .. } => {
                let left_val = self.eval_expr_with_column_context(left, columns, row_idx)?;
                let right_val = self.eval_expr_with_column_context(right, columns, row_idx)?;

                // Rebuild the binary expression with resolved values and evaluate
                // For simplicity, handle common comparison operations directly
                if let ExprKind::Binary { op, .. } = &expr.kind {
                    match op {
                        crate::frontend::ast::BinaryOp::Greater => {
                            self.compare_values(&left_val, &right_val, |a, b| a > b)
                        }
                        crate::frontend::ast::BinaryOp::Less => {
                            self.compare_values(&left_val, &right_val, |a, b| a < b)
                        }
                        crate::frontend::ast::BinaryOp::Equal => {
                            Ok(Value::Bool(self.values_equal(&left_val, &right_val)))
                        }
                        crate::frontend::ast::BinaryOp::NotEqual => {
                            Ok(Value::Bool(!self.values_equal(&left_val, &right_val)))
                        }
                        _ => self.eval_expr(expr), // Use regular evaluation for other operators
                    }
                } else {
                    unreachable!()
                }
            }
            // For all other expressions, use normal evaluation
            _ => self.eval_expr(expr),
        }
    }

    fn eval_dataframe_operation(
        &mut self,
        source: &Expr,
        operation: &crate::frontend::ast::DataFrameOp,
    ) -> Result<Value, InterpreterError> {
        let source_value = self.eval_expr(source)?;

        if let Value::DataFrame { columns } = source_value {
            crate::runtime::eval_dataframe_ops::eval_dataframe_operation(
                columns,
                operation,
                |expr, cols, idx| self.eval_expr_with_column_context(expr, cols, idx),
            )
        } else {
            Err(InterpreterError::RuntimeError(
                "DataFrameOperation can only be applied to DataFrame values".to_string(),
            ))
        }
    }

    /// Evaluate string interpolation
    fn eval_string_interpolation(
        &mut self,
        parts: &[StringPart],
    ) -> Result<Value, InterpreterError> {
        let mut result = String::new();
        for part in parts {
            match part {
                StringPart::Text(text) => result.push_str(text),
                StringPart::Expr(expr) => {
                    let value = self.eval_expr(expr)?;
                    result.push_str(&value.to_string());
                }
                StringPart::ExprWithFormat { expr, format_spec } => {
                    let value = self.eval_expr(expr)?;
                    // Apply format specifier for interpreter
                    let formatted = Self::format_value_with_spec(&value, format_spec);
                    result.push_str(&formatted);
                }
            }
        }
        Ok(Value::from_string(result))
    }

    /// Format a value with a format specifier like :.2 for floats
    fn format_value_with_spec(value: &Value, spec: &str) -> String {
        // Parse format specifier (e.g., ":.2" -> precision 2)
        if let Some(stripped) = spec.strip_prefix(":.") {
            if let Ok(precision) = stripped.parse::<usize>() {
                match value {
                    Value::Float(f) => return format!("{:.precision$}", f, precision = precision),
                    Value::Integer(i) => {
                        return format!("{:.precision$}", *i as f64, precision = precision)
                    }
                    _ => {}
                }
            }
        }
        // Default formatting if spec doesn't match or isn't supported
        value.to_string()
    }

    /// Evaluate function definition
    fn eval_function(
        &mut self,
        name: &str,
        params: &[Param],
        body: &Expr,
    ) -> Result<Value, InterpreterError> {
        let param_names: Vec<String> = params
            .iter()
            .map(crate::frontend::ast::Param::name)
            .collect();

        let closure = Value::Closure {
            params: param_names,
            body: Rc::new(body.clone()),
            env: Rc::new(self.current_env().clone()),
        };

        // Bind function name in environment for recursion
        self.env_set(name.to_string(), closure.clone());
        Ok(closure)
    }

    /// Evaluate lambda expression
    fn eval_lambda(&mut self, params: &[Param], body: &Expr) -> Result<Value, InterpreterError> {
        eval_func::eval_lambda(params, body, self.current_env())
    }

    /// Evaluate function call
    fn eval_function_call(
        &mut self,
        func: &Expr,
        args: &[Expr],
    ) -> Result<Value, InterpreterError> {
        let func_val = self.eval_expr(func)?;
        let arg_vals: Result<Vec<Value>, InterpreterError> =
            args.iter().map(|arg| self.eval_expr(arg)).collect();
        let arg_vals = arg_vals?;

        let result = self.call_function(func_val, &arg_vals)?;

        // Collect type feedback for function call
        let site_id = func.span.start; // Use func span start as site ID
        let func_name = match &func.kind {
            ExprKind::Identifier(name) => name.clone(),
            _ => "anonymous".to_string(),
        };
        self.record_function_call_feedback(site_id, &func_name, &arg_vals, &result);
        Ok(result)
    }

    /// Push an error handling scope for try/catch blocks
    ///
    /// # Complexity
    /// Cyclomatic complexity: 1
    pub fn push_error_scope(&mut self) {
        self.error_scopes.push(ErrorScope {
            env_depth: self.env_stack.len(),
        });
    }

    /// Pop an error handling scope
    ///
    /// # Complexity
    /// Cyclomatic complexity: 1
    pub fn pop_error_scope(&mut self) {
        self.error_scopes.pop();
    }

    /// Set a variable in the current scope
    ///
    /// # Complexity
    /// Cyclomatic complexity: 1
    pub fn set_variable(&mut self, name: &str, value: Value) {
        self.env_set(name.to_string(), value);
    }

    /// Pattern matching for try/catch
    ///
    /// # Complexity
    /// Cyclomatic complexity: 8 (delegates to existing pattern matcher)
    pub fn pattern_matches(
        &mut self,
        pattern: &Pattern,
        value: &Value,
    ) -> Result<bool, InterpreterError> {
        // Simplified pattern matching for try/catch
        match pattern {
            Pattern::Identifier(_) => Ok(true), // Always matches
            Pattern::Wildcard => Ok(true),
            Pattern::Literal(literal) => Ok(self.literal_matches(literal, value)),
            _ => Ok(false), // Other patterns not yet supported
        }
    }

    fn literal_matches(&self, literal: &Literal, value: &Value) -> bool {
        match (literal, value) {
            (Literal::Integer(a), Value::Integer(b)) => a == b,
            (Literal::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
            (Literal::String(a), Value::String(b)) => a == b.as_ref(),
            (Literal::Bool(a), Value::Bool(b)) => a == b,
            _ => false,
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

/// Binary operations
#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Lt,
    Gt,
}

#[cfg(test)]
#[allow(clippy::expect_used)] // Tests can use expect for clarity
#[allow(clippy::bool_assert_comparison)] // Clear test assertions
#[allow(clippy::approx_constant)] // Test constants are acceptable
#[allow(clippy::panic)] // Tests can panic on assertion failures
mod tests {
    use super::*;

    #[test]
    fn test_value_creation() {
        let int_val = Value::from_i64(42);
        assert_eq!(int_val.as_i64().expect("Should be integer"), 42);
        assert_eq!(int_val.type_name(), "integer");

        let bool_val = Value::from_bool(true);
        assert_eq!(bool_val.as_bool().expect("Should be boolean"), true);
        assert_eq!(bool_val.type_name(), "boolean");

        let nil_val = Value::nil();
        assert!(nil_val.is_nil());
        assert_eq!(nil_val.type_name(), "nil");

        let float_val = Value::from_f64(3.14);
        let f_value = float_val.as_f64().expect("Should be float");
        assert!((f_value - 3.14).abs() < f64::EPSILON);
        assert_eq!(float_val.type_name(), "float");

        let string_val = Value::from_string("hello".to_string());
        assert_eq!(string_val.type_name(), "string");
    }

    #[test]
    fn test_arithmetic() {
        let mut interp = Interpreter::new();

        // Test 2 + 3 = 5
        assert!(interp.push(Value::from_i64(2)).is_ok());
        assert!(interp.push(Value::from_i64(3)).is_ok());
        assert!(interp.binary_op(BinaryOp::Add).is_ok());

        let result = interp.pop().expect("Stack should not be empty");
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_mixed_arithmetic() {
        let mut interp = Interpreter::new();

        // Test 2 + 3.5 = 5.5 (int + float -> float)
        assert!(interp.push(Value::from_i64(2)).is_ok());
        assert!(interp.push(Value::from_f64(3.5)).is_ok());
        assert!(interp.binary_op(BinaryOp::Add).is_ok());

        let result = interp.pop().expect("Stack should not be empty");
        match result {
            Value::Float(f) => assert!((f - 5.5).abs() < f64::EPSILON),
            _ => unreachable!("Expected float, got {result:?}"),
        }
    }
}

// Tests removed - moved to separate test files
