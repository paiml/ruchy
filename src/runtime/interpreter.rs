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
#![allow(unsafe_code)] // Required for CallFrame Send implementation - see DEFECT-001-B

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
use std::sync::Arc;

/// Control flow for loop iterations or error
#[derive(Debug)]
enum LoopControlOrError {
    Break(Option<String>, Value),
    Continue(Option<String>),
    Return(Value), // Early return from function (exits both loop and function)
    Error(InterpreterError),
}

/// `DataFrame` column representation for the interpreter
#[derive(Debug, Clone)]
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
#[derive(Clone, Debug)]
pub enum Value {
    /// 64-bit signed integer
    Integer(i64),
    /// 64-bit float
    Float(f64),
    /// Boolean value
    Bool(bool),
    /// Byte value (0-255)
    Byte(u8),
    /// Nil/null value
    Nil,
    /// String value (reference-counted for efficiency, thread-safe)
    String(Arc<str>),
    /// Array of values
    Array(Arc<[Value]>),
    /// Tuple of values
    Tuple(Arc<[Value]>),
    /// Function closure
    Closure {
        params: Vec<String>,
        body: Arc<Expr>,
        env: Arc<HashMap<String, Value>>, // Captured environment
    },
    /// `DataFrame` value
    DataFrame { columns: Vec<DataFrameColumn> },
    /// Object/HashMap value for key-value mappings (immutable)
    Object(Arc<HashMap<String, Value>>),
    /// Mutable object with interior mutability (for actors and classes, thread-safe)
    ObjectMut(Arc<std::sync::Mutex<HashMap<String, Value>>>),
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
    /// Struct instance (value type with named fields)
    /// Thread-safe via Arc, value semantics via cloning
    Struct {
        name: String,
        fields: Arc<HashMap<String, Value>>,
    },
    /// Class instance (reference type with methods)
    /// Thread-safe via Arc, reference semantics, mutable via `RwLock`
    /// Identity-based equality (pointer comparison)
    Class {
        class_name: String,
        fields: Arc<std::sync::RwLock<HashMap<String, Value>>>,
        methods: Arc<HashMap<String, Value>>, // method name -> Closure
    },
    /// HTML document (HTTP-002-C)
    #[cfg(not(target_arch = "wasm32"))]
    HtmlDocument(crate::stdlib::html::HtmlDocument),
    /// HTML element (HTTP-002-C)
    #[cfg(not(target_arch = "wasm32"))]
    HtmlElement(crate::stdlib::html::HtmlElement),
}

// Manual PartialEq implementation because Mutex doesn't implement PartialEq
// ObjectMut uses identity-based equality (Arc pointer comparison) since it represents mutable state
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Tuple(a), Value::Tuple(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => Arc::ptr_eq(a, b) || **a == **b,
            (Value::ObjectMut(a), Value::ObjectMut(b)) => Arc::ptr_eq(a, b), // Identity-based
            (Value::Struct { name: n1, fields: f1 }, Value::Struct { name: n2, fields: f2 }) => {
                n1 == n2 && **f1 == **f2 // Value equality (compare fields)
            }
            (Value::Class { fields: f1, .. }, Value::Class { fields: f2, .. }) => {
                Arc::ptr_eq(f1, f2) // Identity-based: same instance only
            }
            (Value::Nil, Value::Nil) => true,
            (Value::Byte(a), Value::Byte(b)) => a == b,
            #[cfg(not(target_arch = "wasm32"))]
            (Value::HtmlDocument(_), Value::HtmlDocument(_)) => false, // Documents compared by identity
            #[cfg(not(target_arch = "wasm32"))]
            (Value::HtmlElement(_), Value::HtmlElement(_)) => false, // Elements compared by identity
            _ => false, // Different variants are not equal
        }
    }
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
            Value::Byte(_) => TypeId::of::<u8>(),
            Value::String(_) => TypeId::of::<String>(),
            Value::Nil => TypeId::of::<()>(),
            Value::Array(_) => TypeId::of::<Vec<Value>>(),
            Value::Tuple(_) => TypeId::of::<(Value,)>(), // Generic tuple marker
            Value::Closure { .. } => TypeId::of::<fn()>(), // Generic closure marker
            Value::DataFrame { .. } => TypeId::of::<crate::runtime::DataFrameColumn>(),
            Value::Object(_) => TypeId::of::<HashMap<String, Value>>(),
            Value::ObjectMut(_) => TypeId::of::<HashMap<String, Value>>(),
            Value::Range { .. } => TypeId::of::<std::ops::Range<i64>>(),
            Value::EnumVariant { .. } => TypeId::of::<(String, Option<Vec<Value>>)>(),
            Value::BuiltinFunction(_) => TypeId::of::<fn()>(),
            Value::Struct { .. } => TypeId::of::<HashMap<String, Value>>(),
            Value::Class { .. } => TypeId::of::<HashMap<String, Value>>(),
            #[cfg(not(target_arch = "wasm32"))]
            Value::HtmlDocument(_) => TypeId::of::<crate::stdlib::html::HtmlDocument>(),
            #[cfg(not(target_arch = "wasm32"))]
            Value::HtmlElement(_) => TypeId::of::<crate::stdlib::html::HtmlElement>(),
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
#[derive(Debug)]
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

    /// Stdout buffer for capturing println output (WASM/REPL)
    /// Complexity: 1 (simple field addition)
    stdout_buffer: Vec<String>,
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

// SAFETY: CallFrame can safely be Send because:
// 1. The `ip` raw pointer points to immutable bytecode that never changes
// 2. CallFrame has exclusive ownership of the data (no sharing)
// 3. The pointer is never dereferenced across thread boundaries
// 4. CallFrame is only used in single-threaded execution contexts within each thread
// 5. When Repl is shared across threads, each thread gets its own CallFrame instance
unsafe impl Send for CallFrame {}

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
#[derive(Debug, Clone)]
pub enum InterpreterError {
    TypeError(std::string::String),
    RuntimeError(std::string::String),
    StackOverflow,
    StackUnderflow,
    InvalidInstruction,
    DivisionByZero,
    IndexOutOfBounds,
    Break(Option<String>, Value),
    Continue(Option<String>),
    Return(Value),
    Throw(Value), // EXTREME TDD: Exception handling
    AssertionFailed(std::string::String), // BUG-037: Test assertions
    /// Recursion depth limit exceeded (`current_depth`, `max_depth`)
    /// Added via [RUNTIME-001] fix for stack overflow crashes
    RecursionLimitExceeded(usize, usize),
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
            stdout_buffer: Vec::new(),       // Initialize empty stdout buffer
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
            | ExprKind::FieldAccess { .. }
            | ExprKind::TypeCast { .. } => self.eval_operation_expr(expr_kind),

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

    /// Evaluate operation expressions (binary, unary, calls, method calls, type casts, etc.)
    /// Complexity: 9
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
            ExprKind::TypeCast { expr, target_type } => self.eval_type_cast(expr, target_type),
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

    /// Helper: Check if expression is a type definition
    /// Complexity: 2
    fn is_type_definition(expr_kind: &ExprKind) -> bool {
        matches!(
            expr_kind,
            ExprKind::Actor { .. }
                | ExprKind::Enum { .. }
                | ExprKind::Struct { .. }
                | ExprKind::TupleStruct { .. }
                | ExprKind::Class { .. }
                | ExprKind::Impl { .. }
        )
    }

    /// Helper: Check if expression is an actor operation
    /// Complexity: 2
    fn is_actor_operation(expr_kind: &ExprKind) -> bool {
        matches!(
            expr_kind,
            ExprKind::Spawn { .. } | ExprKind::ActorSend { .. } | ExprKind::ActorQuery { .. }
        )
    }

    /// Helper: Check if expression is a special form
    /// Complexity: 2
    fn is_special_form(expr_kind: &ExprKind) -> bool {
        matches!(
            expr_kind,
            ExprKind::None
                | ExprKind::Some { .. }
                | ExprKind::Set(_)
                | ExprKind::LetPattern { .. }
                | ExprKind::StringInterpolation { .. }
                | ExprKind::QualifiedName { .. }
                | ExprKind::ObjectLiteral { .. }
                | ExprKind::StructLiteral { .. }
        )
    }

    /// Evaluate type definition expressions (Actor, Struct, Class, Impl)
    /// Complexity: 6
    fn eval_type_definition(&mut self, expr_kind: &ExprKind) -> Result<Value, InterpreterError> {
        match expr_kind {
            ExprKind::Actor {
                name,
                state,
                handlers,
            } => self.eval_actor_definition(name, state, handlers),
            ExprKind::Enum {
                name,
                type_params,
                variants,
                is_pub,
            } => self.eval_enum_definition(name, type_params, variants, *is_pub),
            ExprKind::Struct {
                name,
                type_params,
                fields,
                derives: _,
                is_pub,
            } => self.eval_struct_definition(name, type_params, fields, *is_pub),
            ExprKind::TupleStruct { .. } => {
                // Tuple structs are transpilation feature, return Nil at runtime
                Ok(Value::Nil)
            }
            ExprKind::Class {
                name,
                type_params,
                superclass,
                traits,
                fields,
                constructors,
                methods,
                constants,
                properties: _,
                derives,
                is_pub,
                is_sealed: _,
                is_abstract: _,
                decorators: _,
            } => self.eval_class_definition(
                name,
                type_params,
                superclass.as_ref(),
                traits,
                fields,
                constructors,
                methods,
                constants,
                derives,
                *is_pub,
            ),
            ExprKind::Impl {
                trait_name: _,
                for_type,
                methods,
                ..
            } => self.eval_impl_block(for_type, methods),
            _ => unreachable!("eval_type_definition called with non-type-definition"),
        }
    }

    /// Evaluate actor operation expressions (Spawn, `ActorSend`, `ActorQuery`)
    /// Complexity: 4
    fn eval_actor_operation(&mut self, expr_kind: &ExprKind) -> Result<Value, InterpreterError> {
        match expr_kind {
            ExprKind::Spawn { actor } => self.eval_spawn_actor(actor),
            ExprKind::ActorSend { actor, message } => self.eval_actor_send(actor, message),
            ExprKind::ActorQuery { actor, message } => self.eval_actor_query(actor, message),
            _ => unreachable!("eval_actor_operation called with non-actor-operation"),
        }
    }

    /// Evaluate special form expressions (None, Some, Set, patterns, literals)
    /// Complexity: 9
    fn eval_special_form(&mut self, expr_kind: &ExprKind) -> Result<Value, InterpreterError> {
        match expr_kind {
            ExprKind::None => Ok(Value::EnumVariant {
                variant_name: "None".to_string(),
                data: None,
            }),
            ExprKind::Some { value } => Ok(Value::EnumVariant {
                variant_name: "Some".to_string(),
                data: Some(vec![self.eval_expr(value)?]),
            }),
            ExprKind::Set(statements) => {
                let mut result = Value::Nil;
                for stmt in statements {
                    result = self.eval_expr(stmt)?;
                }
                Ok(result)
            }
            ExprKind::LetPattern {
                pattern,
                value,
                body,
                ..
            } => self.eval_let_pattern(pattern, value, body),
            ExprKind::StringInterpolation { parts } => self.eval_string_interpolation(parts),
            ExprKind::QualifiedName { module, name } => self.eval_qualified_name(module, name),
            ExprKind::ObjectLiteral { fields } => self.eval_object_literal(fields),
            ExprKind::StructLiteral {
                name,
                fields,
                base: _,
            } => self.eval_struct_literal(name, fields),
            _ => unreachable!("eval_special_form called with non-special-form"),
        }
    }

    /// Evaluate miscellaneous expressions
    /// Complexity: 7 (was 5, added import handling)
    fn eval_misc_expr(&mut self, expr_kind: &ExprKind) -> Result<Value, InterpreterError> {
        if Self::is_type_definition(expr_kind) {
            return self.eval_type_definition(expr_kind);
        }
        if Self::is_actor_operation(expr_kind) {
            return self.eval_actor_operation(expr_kind);
        }
        if Self::is_special_form(expr_kind) {
            return self.eval_special_form(expr_kind);
        }

        // Handle import statements (GitHub Issue #59)
        // Currently no-op until full module resolution is implemented
        match expr_kind {
            ExprKind::Import { .. } | ExprKind::ImportAll { .. } | ExprKind::ImportDefault { .. } => {
                // TODO: Implement module resolution and symbol imports
                // For now, import statements are valid but don't load anything
                Ok(Value::Nil)
            }
            // Handle vec! macro (GitHub Issue #62)
            ExprKind::Macro { name, args } => {
                if name == "vec" {
                    // vec![...] expands to an array with evaluated arguments
                    let mut elements = Vec::new();
                    for arg in args {
                        let value = self.eval_expr(arg)?;
                        elements.push(value);
                    }
                    Ok(Value::Array(elements.into()))
                } else if name == "println" {
                    // println!() macro: Evaluate arguments, print with newline
                    // PARSER-085: Minimal implementation for eval mode (complexity: 6)
                    if args.is_empty() {
                        println!();
                    } else if args.len() == 1 {
                        // Single argument: print directly
                        let value = self.eval_expr(&args[0])?;
                        println!("{}", value);
                    } else {
                        // Multiple arguments: ignore first (format string), print remaining
                        // This handles println!("{}", value) pattern
                        for arg in &args[1..] {
                            let value = self.eval_expr(arg)?;
                            println!("{}", value);
                        }
                    }
                    Ok(Value::Nil)
                } else {
                    // Other macros not yet implemented
                    Err(InterpreterError::RuntimeError(format!(
                        "Macro '{}!' not yet implemented", name
                    )))
                }
            }
            // RUNTIME-001: Handle MacroInvocation (FORMATTER-088 changed parser output)
            // MacroInvocation is the correct AST variant for macro CALLS (not definitions)
            // Delegate to same logic as Macro for backward compatibility (GitHub Issue #74)
            ExprKind::MacroInvocation { name, args } => {
                if name == "vec" {
                    let mut elements = Vec::new();
                    for arg in args {
                        let value = self.eval_expr(arg)?;
                        elements.push(value);
                    }
                    Ok(Value::Array(elements.into()))
                } else if name == "println" {
                    if args.is_empty() {
                        println!();
                    } else if args.len() == 1 {
                        let value = self.eval_expr(&args[0])?;
                        println!("{}", value);
                    } else {
                        for arg in &args[1..] {
                            let value = self.eval_expr(arg)?;
                            println!("{}", value);
                        }
                    }
                    Ok(Value::Nil)
                } else {
                    Err(InterpreterError::RuntimeError(format!(
                        "Macro '{}!' not yet implemented", name
                    )))
                }
            }
            _ => {
                // Fallback for unimplemented expressions
                Err(InterpreterError::RuntimeError(format!(
                    "Expression type not yet implemented: {expr_kind:?}"
                )))
            }
        }
    }

    /// Helper: Evaluate spawn actor expression with proper nesting handling
    /// Complexity: 10 (extracted from inline code)
    fn eval_spawn_actor(&mut self, actor: &Expr) -> Result<Value, InterpreterError> {
        // Handle: spawn ActorName (no args)
        if let ExprKind::Identifier(name) = &actor.kind {
            if let Ok(def_value) = self.lookup_variable(name) {
                if let Value::Object(ref obj) = def_value {
                    if let Some(Value::String(type_str)) = obj.get("__type") {
                        if type_str.as_ref() == "Actor" {
                            let constructor_marker =
                                Value::from_string(format!("__actor_constructor__:{}", name));
                            return self.call_function(constructor_marker, &[]);
                        }
                    }
                }
            }
        }

        // Handle: spawn ActorName(args...)
        if let ExprKind::Call { func, args } = &actor.kind {
            if let ExprKind::Identifier(name) = &func.kind {
                if let Ok(def_value) = self.lookup_variable(name) {
                    if let Value::Object(ref obj) = def_value {
                        if let Some(Value::String(type_str)) = obj.get("__type") {
                            if type_str.as_ref() == "Actor" {
                                let constructor_marker =
                                    Value::from_string(format!("__actor_constructor__:{}", name));
                                let arg_vals: Result<Vec<Value>, _> =
                                    args.iter().map(|arg| self.eval_expr(arg)).collect();
                                let arg_vals = arg_vals?;
                                return self.call_function(constructor_marker, &arg_vals);
                            }
                        }
                    }
                }
            }
        }

        // Default: evaluate the actor expression normally
        let actor_value = self.eval_expr(actor)?;
        Ok(actor_value)
    }

    /// Helper: Evaluate actor send expression (fire-and-forget)
    /// Complexity: 4
    fn eval_actor_send(&mut self, actor: &Expr, message: &Expr) -> Result<Value, InterpreterError> {
        let actor_value = self.eval_expr(actor)?;
        let message_value = self.eval_message_expr(message)?;

        if let Value::ObjectMut(cell_rc) = actor_value {
            self.process_actor_message_sync_mut(&cell_rc, &message_value)?;
            Ok(Value::Nil)
        } else {
            Err(InterpreterError::RuntimeError(format!(
                "ActorSend requires an actor instance, got {}",
                actor_value.type_name()
            )))
        }
    }

    /// Helper: Evaluate actor query expression (ask pattern)
    /// Complexity: 4
    fn eval_actor_query(
        &mut self,
        actor: &Expr,
        message: &Expr,
    ) -> Result<Value, InterpreterError> {
        let actor_value = self.eval_expr(actor)?;
        let message_value = self.eval_message_expr(message)?;

        if let Value::ObjectMut(cell_rc) = actor_value {
            self.process_actor_message_sync_mut(&cell_rc, &message_value)
        } else {
            Err(InterpreterError::RuntimeError(format!(
                "ActorQuery requires an actor instance, got {}",
                actor_value.type_name()
            )))
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
                label,
                var,
                pattern,
                iter,
                body,
            } => self.eval_for_loop(label.as_ref(), var, pattern.as_ref(), iter, body),
            ExprKind::While {
                label,
                condition,
                body,
            } => self.eval_while_loop(label.as_ref(), condition, body),
            ExprKind::Loop { label, body } => self.eval_loop(label.as_ref(), body),
            ExprKind::Match { expr, arms } => self.eval_match(expr, arms),
            ExprKind::Break { label, value } => {
                // Evaluate the break value (default to Nil if not provided)
                let break_val = if let Some(expr) = value {
                    self.eval_expr(expr)?
                } else {
                    Value::Nil
                };
                Err(InterpreterError::Break(label.clone(), break_val))
            }
            ExprKind::Continue { label } => Err(InterpreterError::Continue(label.clone())),
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
            ExprKind::DataFrame { columns } => self.eval_dataframe_literal(columns),
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
            (Value::Array(ref array), Value::Integer(idx)) => Self::index_array(array, *idx),
            (Value::String(ref s), Value::Integer(idx)) => Self::index_string(s, *idx),
            (Value::Tuple(ref tuple), Value::Integer(idx)) => Self::index_tuple(tuple, *idx),
            (Value::Object(ref fields), Value::String(ref key)) => Self::index_object(fields, key),
            (Value::ObjectMut(ref cell), Value::String(ref key)) => {
                Self::index_object_mut(cell, key)
            }
            (Value::DataFrame { columns }, Value::Integer(idx)) => {
                Self::index_dataframe_row(columns, *idx)
            }
            (Value::DataFrame { columns }, Value::String(ref col_name)) => {
                Self::index_dataframe_column(columns, col_name)
            }
            _ => Err(InterpreterError::RuntimeError(format!(
                "Cannot index {} with {}",
                object_value.type_name(),
                index_value.type_name()
            ))),
        }
    }

    /// Index into an array (complexity: 5 - added negative indexing support)
    /// FEATURE-042 (GitHub Issue #46): Support Python/Ruby-style negative indexing
    fn index_array(array: &[Value], idx: i64) -> Result<Value, InterpreterError> {
        let len = array.len() as i64;
        let actual_index = if idx < 0 {
            // Negative index: count from the end
            // -1 => len-1 (last), -2 => len-2 (second-to-last), etc.
            len + idx
        } else {
            idx
        };

        // Check bounds (actual_index must be in range [0, len))
        if actual_index < 0 || actual_index >= len {
            return Err(InterpreterError::RuntimeError(format!(
                "Index {idx} out of bounds for array of length {len}"
            )));
        }

        #[allow(clippy::cast_sign_loss)] // Safe: we've verified actual_index >= 0
        Ok(array[actual_index as usize].clone())
    }

    /// Index into a string (complexity: 5 - added negative indexing support)
    /// FEATURE-042 (GitHub Issue #46): Support Python/Ruby-style negative indexing
    fn index_string(s: &str, idx: i64) -> Result<Value, InterpreterError> {
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len() as i64;
        let actual_index = if idx < 0 {
            // Negative index: count from the end
            len + idx
        } else {
            idx
        };

        // Check bounds
        if actual_index < 0 || actual_index >= len {
            return Err(InterpreterError::RuntimeError(format!(
                "Index {idx} out of bounds for string of length {len}"
            )));
        }

        #[allow(clippy::cast_sign_loss)] // Safe: we've verified actual_index >= 0
        Ok(Value::from_string(chars[actual_index as usize].to_string()))
    }

    /// Index into a tuple (complexity: 5 - added negative indexing support)
    /// FEATURE-042 (GitHub Issue #46): Support Python/Ruby-style negative indexing
    fn index_tuple(tuple: &[Value], idx: i64) -> Result<Value, InterpreterError> {
        let len = tuple.len() as i64;
        let actual_index = if idx < 0 {
            // Negative index: count from the end
            len + idx
        } else {
            idx
        };

        // Check bounds
        if actual_index < 0 || actual_index >= len {
            return Err(InterpreterError::RuntimeError(format!(
                "Index {idx} out of bounds for tuple of length {len}"
            )));
        }

        #[allow(clippy::cast_sign_loss)] // Safe: we've verified actual_index >= 0
        Ok(tuple[actual_index as usize].clone())
    }

    /// Index into an object with string key (complexity: 1)
    fn index_object(fields: &HashMap<String, Value>, key: &str) -> Result<Value, InterpreterError> {
        fields.get(key).cloned().ok_or_else(|| {
            InterpreterError::RuntimeError(format!("Key '{key}' not found in object"))
        })
    }

    /// Index into a mutable object with string key (complexity: 1)
    fn index_object_mut(
        cell: &Arc<std::sync::Mutex<HashMap<String, Value>>>,
        key: &str,
    ) -> Result<Value, InterpreterError> {
        cell.lock().unwrap().get(key).cloned().ok_or_else(|| {
            InterpreterError::RuntimeError(format!("Key '{key}' not found in object"))
        })
    }

    /// Index into a `DataFrame` by row index (complexity: 5)
    /// Returns a row as an Object with column names as keys
    fn index_dataframe_row(
        columns: &[DataFrameColumn],
        row_idx: i64,
    ) -> Result<Value, InterpreterError> {
        if columns.is_empty() {
            return Err(InterpreterError::RuntimeError(
                "Cannot index empty DataFrame".to_string(),
            ));
        }

        let index = row_idx as usize;
        let num_rows = columns[0].values.len();

        if index >= num_rows {
            return Err(InterpreterError::RuntimeError(format!(
                "Row index {index} out of bounds for DataFrame with {num_rows} rows"
            )));
        }

        // Build row as Object with column names as keys
        let mut row = HashMap::new();
        for col in columns {
            row.insert(col.name.clone(), col.values[index].clone());
        }

        Ok(Value::Object(Arc::new(row)))
    }

    /// Index into a `DataFrame` by column name (complexity: 3)
    /// Returns a column as an Array
    fn index_dataframe_column(
        columns: &[DataFrameColumn],
        col_name: &str,
    ) -> Result<Value, InterpreterError> {
        columns
            .iter()
            .find(|col| col.name == col_name)
            .map(|col| Value::Array(Arc::from(col.values.clone())))
            .ok_or_else(|| {
                InterpreterError::RuntimeError(format!("Column '{col_name}' not found in DataFrame"))
            })
    }

    /// Check if a field is accessible based on visibility rules
    /// Complexity: 5
    fn check_field_visibility(
        &self,
        struct_name: &str,
        field: &str,
    ) -> Result<(), InterpreterError> {
        // Look up struct type definition
        let struct_type = self.lookup_variable(struct_name).ok();
        if let Some(Value::Object(struct_obj)) = struct_type {
            if let Some(Value::Object(fields)) = struct_obj.get("__fields") {
                if let Some(Value::Object(field_info)) = fields.get(field) {
                    if let Some(Value::String(visibility)) = field_info.get("visibility") {
                        if visibility.as_ref() == "private" {
                            return Err(InterpreterError::RuntimeError(format!(
                                "Field '{}' is private and cannot be accessed outside the struct",
                                field
                            )));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn eval_field_access(&mut self, object: &Expr, field: &str) -> Result<Value, InterpreterError> {
        let object_value = self.eval_expr(object)?;

        match object_value {
            Value::Object(ref object_map) => {
                // Check if this is an enum type trying to construct a variant
                if let Some(Value::String(type_str)) = object_map.get("__type") {
                    if type_str.as_ref() == "Enum" {
                        // This is enum variant construction: EnumName::VariantName
                        return Ok(Value::EnumVariant {
                            variant_name: field.to_string(),
                            data: None, // Unit variant (no data)
                        });
                    }
                }
                self.access_object_field(object_map, field)
            }
            Value::ObjectMut(ref cell) => self.access_object_mut_field(cell, field),
            Value::Struct { ref name, ref fields } => {
                // Struct field access
                fields.get(field).cloned().ok_or_else(|| {
                    InterpreterError::RuntimeError(format!(
                        "Field '{field}' not found in struct {name}"
                    ))
                })
            }
            Value::Class {
                ref class_name,
                ref fields,
                ..
            } => {
                // Class field access
                let fields_read = fields.read().unwrap();
                fields_read.get(field).cloned().ok_or_else(|| {
                    InterpreterError::RuntimeError(format!(
                        "Field '{field}' not found in class {class_name}"
                    ))
                })
            }
            Value::Tuple(ref elements) => {
                // Tuple field access (e.g., tuple.0, tuple.1)
                crate::runtime::eval_data_structures::eval_tuple_field_access(elements, field)
            }
            Value::DataFrame { ref columns } => {
                // DataFrame field access (df.column_name returns column as array)
                Self::index_dataframe_column(columns, field)
            }
            _ => Err(InterpreterError::RuntimeError(format!(
                "Cannot access field '{}' on type {}",
                field,
                object_value.type_name()
            ))),
        }
    }

    /// Access field on immutable object (complexity: 5)
    fn access_object_field(
        &self,
        object_map: &HashMap<String, Value>,
        field: &str,
    ) -> Result<Value, InterpreterError> {
        // Check for constructor access (.new)
        if let Some(constructor) = Self::check_constructor_access(object_map, field) {
            return Ok(constructor);
        }

        // Check for actor field access
        if let Some(actor_field) = Self::check_actor_field_access(object_map, field)? {
            return Ok(actor_field);
        }

        // Check struct visibility
        self.check_struct_visibility(object_map, field)?;

        // Regular field access
        Self::get_object_field(object_map, field)
    }

    /// Access field on mutable object (complexity: 4)
    fn access_object_mut_field(
        &self,
        cell: &Arc<std::sync::Mutex<HashMap<String, Value>>>,
        field: &str,
    ) -> Result<Value, InterpreterError> {
        let object_map = cell.lock().unwrap();

        // Check for actor field access
        if let Some(actor_field) = Self::check_actor_field_access(&object_map, field)? {
            return Ok(actor_field);
        }

        // Check struct visibility
        self.check_struct_visibility(&object_map, field)?;

        // Regular field access
        Self::get_object_field(&object_map, field)
    }

    /// Check for constructor access (.new on type definitions) (complexity: 4)
    fn check_constructor_access(object_map: &HashMap<String, Value>, field: &str) -> Option<Value> {
        if field != "new" {
            return None;
        }

        if let Some(Value::String(ref type_str)) = object_map.get("__type") {
            if let Some(Value::String(ref name)) = object_map.get("__name") {
                return match type_str.as_ref() {
                    "Actor" => Some(Value::from_string(format!("__actor_constructor__:{name}"))),
                    "Struct" => Some(Value::from_string(format!("__struct_constructor__:{name}"))),
                    "Class" => Some(Value::from_string(format!(
                        "__class_constructor__:{name}:new"
                    ))),
                    _ => None,
                };
            }
        }
        None
    }

    /// Check for actor field access (complexity: 2)
    fn check_actor_field_access(
        object_map: &HashMap<String, Value>,
        field: &str,
    ) -> Result<Option<Value>, InterpreterError> {
        if let Some(Value::String(actor_id)) = object_map.get("__actor_id") {
            use crate::runtime::actor_runtime::ACTOR_RUNTIME;
            let field_value = ACTOR_RUNTIME.get_actor_field(actor_id.as_ref(), field)?;
            Ok(Some(field_value.to_value()))
        } else {
            Ok(None)
        }
    }

    /// Check struct field visibility (complexity: 2)
    fn check_struct_visibility(
        &self,
        object_map: &HashMap<String, Value>,
        field: &str,
    ) -> Result<(), InterpreterError> {
        if let Some(Value::String(struct_name)) = object_map.get("__struct_type") {
            self.check_field_visibility(struct_name.as_ref(), field)?;
        }
        Ok(())
    }

    /// Get field from object map (complexity: 2)
    fn get_object_field(
        object_map: &HashMap<String, Value>,
        field: &str,
    ) -> Result<Value, InterpreterError> {
        object_map.get(field).cloned().ok_or_else(|| {
            InterpreterError::RuntimeError(format!("Object has no field named '{field}'"))
        })
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

        Ok(Value::Object(Arc::new(object)))
    }

    fn eval_qualified_name(&self, module: &str, name: &str) -> Result<Value, InterpreterError> {
        if module == "HashMap" && name == "new" {
            Ok(Value::from_string("__builtin_hashmap__".to_string()))
        } else if module == "String" && (name == "new" || name == "from") {
            // REGRESSION-077: Route String::new() and String::from() to builtin handlers
            Ok(Value::from_string(format!("__builtin_String_{}__", name)))
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
            // Check if this is an actor constructor call
            if let Ok(actor_value) = self.lookup_variable(module) {
                if let Value::Object(ref actor_info) = actor_value {
                    // Check if it's an actor definition
                    if let Some(Value::String(ref type_str)) = actor_info.get("__type") {
                        if type_str.as_ref() == "Actor" {
                            // Return a special marker for actor instantiation
                            return Ok(Value::from_string(format!(
                                "__actor_constructor__:{}",
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
            // REGRESSION-077: Check if this is an impl method (stored with qualified name)
            // Example: Logger::new_with_options stored as "Logger::new_with_options"
            let qualified_method_name = format!("{}::{}", module, name);
            if let Ok(method_value) = self.lookup_variable(&qualified_method_name) {
                Ok(method_value)
            } else {
                Err(InterpreterError::RuntimeError(format!(
                    "Unknown qualified name: {}::{}",
                    module, name
                )))
            }
        }
    }

    /// Evaluate a literal value
    fn eval_literal(&self, lit: &Literal) -> Value {
        match lit {
            Literal::Integer(i, _) => Value::from_i64(*i),
            Literal::Float(f) => Value::from_f64(*f),
            Literal::String(s) => Value::from_string(s.clone()),
            Literal::Bool(b) => Value::from_bool(*b),
            Literal::Char(c) => Value::from_string(c.to_string()),
            Literal::Byte(b) => Value::Byte(*b),
            Literal::Unit => Value::nil(),
            Literal::Null => Value::nil(),
        }
    }

    /// Look up a variable in the environment (searches from innermost to outermost)
    fn lookup_variable(&self, name: &str) -> Result<Value, InterpreterError> {
        // REGRESSION-077: Handle Option enum variants (Option::None, Option::Some)
        if name == "Option::None" {
            return Ok(Value::EnumVariant {
                variant_name: "None".to_string(),
                data: None,
            });
        }

        // Check if this is a qualified name (e.g., "Point::new" or "Rectangle::square")
        if name.contains("::") {
            let parts: Vec<&str> = name.split("::").collect();
            if parts.len() == 2 {
                let type_name = parts[0];
                let method_name = parts[1];

                // Look up the class or struct
                for env in self.env_stack.iter().rev() {
                    if let Some(value) = env.get(type_name) {
                        if let Value::Object(ref info) = value {
                            // Check if it's a class or struct
                            if let Some(Value::String(ref type_str)) = info.get("__type") {
                                if type_str.as_ref() == "Class" {
                                    // Check if it's a static method
                                    if let Some(Value::Object(ref methods)) = info.get("__methods")
                                    {
                                        if let Some(Value::Object(ref method_meta)) =
                                            methods.get(method_name)
                                        {
                                            if let Some(Value::Bool(true)) =
                                                method_meta.get("is_static")
                                            {
                                                // Return marker for static method
                                                return Ok(Value::from_string(format!(
                                                    "__class_static_method__:{}:{}",
                                                    type_name, method_name
                                                )));
                                            }
                                        }
                                    }

                                    // Check if it's a constructor
                                    if let Some(Value::Object(ref constructors)) =
                                        info.get("__constructors")
                                    {
                                        if constructors.contains_key(method_name) {
                                            // Return marker for class constructor
                                            return Ok(Value::from_string(format!(
                                                "__class_constructor__:{}:{}",
                                                type_name, method_name
                                            )));
                                        }
                                    }
                                } else if type_str.as_ref() == "Struct" && method_name == "new" {
                                    return Ok(Value::from_string(format!(
                                        "__struct_constructor__:{}",
                                        type_name
                                    )));
                                } else if type_str.as_ref() == "Actor" && method_name == "new" {
                                    return Ok(Value::from_string(format!(
                                        "__actor_constructor__:{}",
                                        type_name
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
    pub fn current_env(&self) -> &HashMap<String, Value> {
        self.env_stack
            .last()
            .expect("Environment stack should never be empty")
    }

    /// Set a variable in the current environment
    #[allow(clippy::expect_used)] // Environment stack invariant ensures this never panics
    /// Create a new variable binding in the current scope (for `let` bindings)
    ///
    /// RUNTIME-038 FIX: `let` bindings create NEW variables in current scope (shadowing),
    /// they do NOT update variables in parent scopes. This prevents variable collision
    /// in nested function calls.
    ///
    /// # Complexity
    /// Cyclomatic complexity: 1 (within Toyota Way limits)
    fn env_set(&mut self, name: String, value: Value) {
        // Record type feedback for optimization
        self.record_variable_assignment_feedback(&name, &value);

        // ALWAYS create in current scope - `let` bindings shadow outer scopes
        // Do NOT search parent scopes (that's for reassignments without `let`)
        let env = self
            .env_stack
            .last_mut()
            .expect("Environment stack should never be empty");
        env.insert(name, value);
    }

    /// Set a mutable variable in the environment
    /// ISSUE-040 FIX: Searches parent scopes for existing variable and mutates it.
    /// Falls back to creating new binding in current scope if variable doesn't exist.
    ///
    /// # Complexity
    /// Cyclomatic complexity: 4 (within Toyota Way limits ≤10)
    fn env_set_mut(&mut self, name: String, value: Value) {
        // Record type feedback for optimization
        self.record_variable_assignment_feedback(&name, &value);

        // Search from innermost to outermost scope for existing variable
        for env in self.env_stack.iter_mut().rev() {
            if let std::collections::hash_map::Entry::Occupied(mut e) = env.entry(name.clone()) {
                // Found existing variable - mutate it in place
                e.insert(value);
                return;
            }
        }

        // Variable doesn't exist in any scope - create new binding in current scope
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
                // Extract class name and constructor name from the marker
                let parts: Vec<&str> = s
                    .strip_prefix("__class_constructor__:")
                    .unwrap()
                    .split(':')
                    .collect();

                if parts.len() == 2 {
                    let class_name = parts[0];
                    let constructor_name = parts[1];
                    self.instantiate_class_with_constructor(class_name, constructor_name, args)
                } else {
                    // Legacy format for backward compatibility
                    self.instantiate_class_with_constructor(parts[0], "new", args)
                }
            }
            Value::String(ref s) if s.starts_with("__class_static_method__:") => {
                // Extract class name and method name from the marker
                let parts: Vec<&str> = s
                    .strip_prefix("__class_static_method__:")
                    .unwrap()
                    .split(':')
                    .collect();

                if parts.len() == 2 {
                    let class_name = parts[0];
                    let method_name = parts[1];
                    self.call_static_method(class_name, method_name, args)
                } else {
                    Err(InterpreterError::RuntimeError(
                        "Invalid static method marker".to_string(),
                    ))
                }
            }
            Value::String(ref s) if s.starts_with("__struct_constructor__:") => {
                // Extract struct name from the marker
                let struct_name = s.strip_prefix("__struct_constructor__:").unwrap();
                self.instantiate_struct_with_args(struct_name, args)
            }
            Value::String(ref s) if s.starts_with("__actor_constructor__:") => {
                // Extract actor name from the marker
                let actor_name = s.strip_prefix("__actor_constructor__:").unwrap();
                self.instantiate_actor_with_args(actor_name, args)
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
                // [RUNTIME-001] CHECK RECURSION DEPTH BEFORE ENTERING
                crate::runtime::eval_function::check_recursion_depth()?;

                // Check argument count
                if args.len() != params.len() {
                    crate::runtime::eval_function::decrement_depth();
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
                // Catch InterpreterError::Return and extract value (early return support)
                let result = match self.eval_expr(&body) {
                    Err(InterpreterError::Return(val)) => Ok(val),
                    other => other,
                };

                // Pop environment
                self.env_pop();

                // [RUNTIME-001] ALWAYS DECREMENT, EVEN ON ERROR
                crate::runtime::eval_function::decrement_depth();

                result
            }
            Value::Object(ref obj) => {
                // Check if this is a struct or actor definition being called as a constructor
                if let Some(Value::String(type_str)) = obj.get("__type") {
                    match type_str.as_ref() {
                        "Struct" => {
                            // Get struct name and instantiate
                            if let Some(Value::String(name)) = obj.get("__name") {
                                self.instantiate_struct_with_args(name.as_ref(), args)
                            } else {
                                Err(InterpreterError::RuntimeError(
                                    "Struct missing __name field".to_string(),
                                ))
                            }
                        }
                        "Actor" => {
                            // Get actor name and instantiate
                            if let Some(Value::String(name)) = obj.get("__name") {
                                self.instantiate_actor_with_args(name.as_ref(), args)
                            } else {
                                Err(InterpreterError::RuntimeError(
                                    "Actor missing __name field".to_string(),
                                ))
                            }
                        }
                        "Class" => {
                            // Get class name and instantiate
                            if let Some(Value::String(name)) = obj.get("__name") {
                                self.instantiate_class_with_args(name.as_ref(), args)
                            } else {
                                Err(InterpreterError::RuntimeError(
                                    "Class missing __name field".to_string(),
                                ))
                            }
                        }
                        _ => Err(InterpreterError::TypeError(format!(
                            "Cannot call object of type: {}",
                            type_str
                        ))),
                    }
                } else {
                    Err(InterpreterError::TypeError(format!(
                        "Cannot call non-function value: {}",
                        func.type_name()
                    )))
                }
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
        // Handle short-circuit operators and special operators
        match op {
            crate::frontend::ast::BinaryOp::Send => {
                // Actor send operator: actor ! message
                let left_val = self.eval_expr(left)?;
                let message_val = self.eval_message_expr(right)?;

                // Extract the ObjectMut from the actor
                if let Value::ObjectMut(cell_rc) = left_val {
                    // Process the message synchronously
                    self.process_actor_message_sync_mut(&cell_rc, &message_val)?;
                    // Fire-and-forget returns Nil
                    Ok(Value::Nil)
                } else {
                    Err(InterpreterError::RuntimeError(format!(
                        "Send operator requires an actor instance, got {}",
                        left_val.type_name()
                    )))
                }
            }
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

    /// Evaluate type cast expression (as operator)
    ///
    /// # Complexity
    /// Cyclomatic complexity: 8 (within Toyota Way limits)
    fn eval_type_cast(
        &mut self,
        expr: &Expr,
        target_type: &str,
    ) -> Result<Value, InterpreterError> {
        // Special case: Enum variant to integer (Issue #79)
        // Must extract enum name from AST BEFORE evaluating expression
        if matches!(target_type, "i32" | "i64" | "isize") {
            if let ExprKind::FieldAccess { object, field } = &expr.kind {
                if let ExprKind::Identifier(enum_name) = &object.kind {
                    // Direct enum literal: LogLevel::Info as i32
                    let variant_name = field;

                    // Lookup enum definition in environment
                    if let Some(Value::Object(enum_def)) = self.get_variable(enum_name) {
                        if let Some(Value::Object(variants)) = enum_def.get("__variants") {
                            if let Some(Value::Object(variant_info)) = variants.get(variant_name) {
                                if let Some(Value::Integer(disc)) =
                                    variant_info.get("discriminant")
                                {
                                    return Ok(Value::Integer(*disc));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Standard case: Evaluate expression first, then cast
        let value = self.eval_expr(expr)?;

        match (value, target_type) {
            // Integer to Float
            (Value::Integer(i), "f64" | "f32") => Ok(Value::Float(i as f64)),

            // Float to Integer (truncation)
            (Value::Float(f), "i32" | "i64" | "isize") => Ok(Value::Integer(f as i64)),

            // Integer to Integer (identity for i32/i64)
            (Value::Integer(i), "i32" | "i64" | "isize") => Ok(Value::Integer(i)),

            // Float to Float (identity)
            (Value::Float(f), "f64" | "f32") => Ok(Value::Float(f)),

            // Enum variant to Integer - variable case (e.g., variant as i32)
            // This will fail because we don't store enum type info at runtime
            (Value::EnumVariant { variant_name, .. }, "i32" | "i64" | "isize") => {
                Err(InterpreterError::TypeError(format!(
                    "Cannot cast enum variant {} to integer: enum type information lost at runtime. \
                     Use direct enum literal casts instead (e.g., LogLevel::Info as i32)",
                    variant_name
                )))
            }

            // Unsupported cast
            (val, target) => Err(InterpreterError::TypeError(format!(
                "Cannot cast {} to {}",
                val.type_name(),
                target
            ))),
        }
    }

    /// Evaluate if expression
    fn eval_if_expr(
        &mut self,
        condition: &Expr,
        then_branch: &Expr,
        else_branch: Option<&Expr>,
    ) -> Result<Value, InterpreterError> {
        crate::runtime::eval_control_flow_new::eval_if_expr(
            condition,
            then_branch,
            else_branch,
            |e| self.eval_expr(e),
        )
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
        crate::runtime::eval_control_flow_new::eval_return_expr(value, |e| self.eval_expr(e))
    }

    /// Evaluate list expression
    fn eval_list_expr(&mut self, elements: &[Expr]) -> Result<Value, InterpreterError> {
        crate::runtime::eval_control_flow_new::eval_list_expr(elements, |e| self.eval_expr(e))
    }

    /// Evaluate array initialization expression [value; size]
    fn eval_array_init_expr(
        &mut self,
        value_expr: &Expr,
        size_expr: &Expr,
    ) -> Result<Value, InterpreterError> {
        crate::runtime::eval_control_flow_new::eval_array_init_expr(value_expr, size_expr, |e| {
            self.eval_expr(e)
        })
    }

    /// Evaluate block expression
    fn eval_block_expr(&mut self, statements: &[Expr]) -> Result<Value, InterpreterError> {
        crate::runtime::eval_control_flow_new::eval_block_expr(statements, |e| self.eval_expr(e))
    }

    /// Evaluate tuple expression
    fn eval_tuple_expr(&mut self, elements: &[Expr]) -> Result<Value, InterpreterError> {
        crate::runtime::eval_control_flow_new::eval_tuple_expr(elements, |e| self.eval_expr(e))
    }

    /// Evaluate `DataFrame` literal expression
    /// Complexity: 5 (within Toyota Way limits)
    fn eval_dataframe_literal(
        &mut self,
        columns: &[crate::frontend::ast::DataFrameColumn],
    ) -> Result<Value, InterpreterError> {
        let mut evaluated_columns = Vec::new();

        for col in columns {
            // Evaluate each value expression in the column
            let mut evaluated_values = Vec::new();
            for value_expr in &col.values {
                evaluated_values.push(self.eval_expr(value_expr)?);
            }

            // Create runtime DataFrameColumn
            evaluated_columns.push(DataFrameColumn {
                name: col.name.clone(),
                values: evaluated_values,
            });
        }

        Ok(Value::DataFrame {
            columns: evaluated_columns,
        })
    }

    /// Evaluate range expression
    fn eval_range_expr(
        &mut self,
        start: &Expr,
        end: &Expr,
        inclusive: bool,
    ) -> Result<Value, InterpreterError> {
        crate::runtime::eval_control_flow_new::eval_range_expr(start, end, inclusive, |e| {
            self.eval_expr(e)
        })
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
        body: Arc<Expr>,
        env: Arc<HashMap<String, Value>>,
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
        label: Option<&String>,
        var: &str,
        _pattern: Option<&Pattern>,
        iter: &Expr,
        body: &Expr,
    ) -> Result<Value, InterpreterError> {
        let iter_value = self.eval_expr(iter)?;

        match iter_value {
            Value::Array(ref arr) => self.eval_for_array_iteration(label, var, arr, body),
            Value::Range {
                ref start,
                ref end,
                inclusive,
            } => self.eval_for_range_iteration(label, var, start, end, inclusive, body),
            _ => Err(InterpreterError::TypeError(
                "For loop requires an iterable".to_string(),
            )),
        }
    }

    /// Evaluate for loop iteration over an array
    /// Complexity: ≤8
    fn eval_for_array_iteration(
        &mut self,
        label: Option<&String>,
        loop_var: &str,
        arr: &[Value],
        body: &Expr,
    ) -> Result<Value, InterpreterError> {
        let mut last_value = Value::nil();

        for item in arr {
            self.set_variable(loop_var, item.clone());
            match self.eval_loop_body_with_control_flow(body) {
                Ok(value) => last_value = value,
                Err(LoopControlOrError::Break(break_label, break_val)) => {
                    // If break has no label or matches this loop's label, break here
                    if break_label.is_none() || break_label.as_deref() == label.map(String::as_str)
                    {
                        return Ok(break_val);
                    }
                    // Otherwise, propagate to outer loop
                    return Err(InterpreterError::Break(break_label, break_val));
                }
                Err(LoopControlOrError::Continue(continue_label)) => {
                    // If continue has no label or matches this loop's label, continue here
                    if continue_label.is_none()
                        || continue_label.as_deref() == label.map(String::as_str)
                    {
                        continue;
                    }
                    // Otherwise, propagate to outer loop
                    return Err(InterpreterError::Continue(continue_label));
                }
                Err(LoopControlOrError::Return(return_val)) => {
                    return Err(InterpreterError::Return(return_val))
                }
                Err(LoopControlOrError::Error(e)) => return Err(e),
            }
        }

        Ok(last_value)
    }

    /// Evaluate for loop iteration over a range
    /// Complexity: ≤9
    fn eval_for_range_iteration(
        &mut self,
        label: Option<&String>,
        loop_var: &str,
        start: &Value,
        end: &Value,
        inclusive: bool,
        body: &Expr,
    ) -> Result<Value, InterpreterError> {
        let (start_val, end_val) = self.extract_range_bounds(start, end)?;
        let mut last_value = Value::nil();

        for i in self.create_range_iterator(start_val, end_val, inclusive) {
            self.set_variable(loop_var, Value::Integer(i));
            match self.eval_loop_body_with_control_flow(body) {
                Ok(value) => last_value = value,
                Err(LoopControlOrError::Break(break_label, break_val)) => {
                    if break_label.is_none() || break_label.as_deref() == label.map(String::as_str)
                    {
                        return Ok(break_val);
                    }
                    return Err(InterpreterError::Break(break_label, break_val));
                }
                Err(LoopControlOrError::Continue(continue_label)) => {
                    if continue_label.is_none()
                        || continue_label.as_deref() == label.map(String::as_str)
                    {
                        continue;
                    }
                    return Err(InterpreterError::Continue(continue_label));
                }
                Err(LoopControlOrError::Return(return_val)) => {
                    return Err(InterpreterError::Return(return_val))
                }
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
            Err(InterpreterError::Break(label, val)) => Err(LoopControlOrError::Break(label, val)),
            Err(InterpreterError::Continue(label)) => Err(LoopControlOrError::Continue(label)),
            Err(InterpreterError::Return(val)) => Err(LoopControlOrError::Return(val)),
            Err(e) => Err(LoopControlOrError::Error(e)),
        }
    }

    /// Evaluate a while loop
    fn eval_while_loop(
        &mut self,
        label: Option<&String>,
        condition: &Expr,
        body: &Expr,
    ) -> Result<Value, InterpreterError> {
        let mut last_value = Value::Nil;
        loop {
            let cond_value = self.eval_expr(condition)?;
            if !matches!(cond_value, Value::Bool(true)) && cond_value != Value::Integer(1) {
                break;
            }

            match self.eval_loop_body_with_control_flow(body) {
                Ok(value) => last_value = value,
                Err(LoopControlOrError::Break(break_label, break_val)) => {
                    if break_label.is_none() || break_label.as_deref() == label.map(String::as_str)
                    {
                        return Ok(break_val);
                    }
                    return Err(InterpreterError::Break(break_label, break_val));
                }
                Err(LoopControlOrError::Continue(continue_label)) => {
                    if continue_label.is_none()
                        || continue_label.as_deref() == label.map(String::as_str)
                    {
                        continue;
                    }
                    return Err(InterpreterError::Continue(continue_label));
                }
                Err(LoopControlOrError::Return(return_val)) => {
                    return Err(InterpreterError::Return(return_val))
                }
                Err(LoopControlOrError::Error(e)) => return Err(e),
            }
        }
        Ok(last_value)
    }

    /// Evaluate an infinite loop (loop { ... })
    fn eval_loop(
        &mut self,
        label: Option<&String>,
        body: &Expr,
    ) -> Result<Value, InterpreterError> {
        loop {
            match self.eval_loop_body_with_control_flow(body) {
                Ok(_) => {}
                Err(LoopControlOrError::Break(break_label, break_val)) => {
                    if break_label.is_none() || break_label.as_deref() == label.map(String::as_str)
                    {
                        return Ok(break_val);
                    }
                    return Err(InterpreterError::Break(break_label, break_val));
                }
                Err(LoopControlOrError::Continue(continue_label)) => {
                    if continue_label.is_none()
                        || continue_label.as_deref() == label.map(String::as_str)
                    {
                        continue;
                    }
                    return Err(InterpreterError::Continue(continue_label));
                }
                Err(LoopControlOrError::Return(return_val)) => {
                    return Err(InterpreterError::Return(return_val))
                }
                Err(LoopControlOrError::Error(e)) => return Err(e),
            }
        }
    }

    /// Evaluate a match expression
    pub fn eval_match(&mut self, expr: &Expr, arms: &[MatchArm]) -> Result<Value, InterpreterError> {
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
    /// Extract names of identifiers marked as mutable in a pattern
    /// Complexity: 4 (within Toyota Way limits)
    fn extract_mut_names(pattern: &Pattern) -> std::collections::HashSet<String> {
        let mut mut_names = std::collections::HashSet::new();

        fn walk_pattern(
            p: &Pattern,
            mut_names: &mut std::collections::HashSet<String>,
            is_mut: bool,
        ) {
            match p {
                Pattern::Mut(inner) => walk_pattern(inner, mut_names, true),
                Pattern::Identifier(name) if is_mut => {
                    mut_names.insert(name.clone());
                }
                Pattern::Tuple(patterns) | Pattern::List(patterns) => {
                    for pat in patterns {
                        walk_pattern(pat, mut_names, is_mut);
                    }
                }
                Pattern::Struct { fields, .. } => {
                    for field in fields {
                        if let Some(ref pat) = field.pattern {
                            walk_pattern(pat, mut_names, is_mut);
                        }
                    }
                }
                Pattern::AtBinding { pattern, .. } => walk_pattern(pattern, mut_names, is_mut),
                _ => {}
            }
        }

        walk_pattern(pattern, &mut mut_names, false);
        mut_names
    }

    /// Evaluate let pattern with support for mut destructuring
    /// Complexity: 6 (within Toyota Way limits)
    fn eval_let_pattern(
        &mut self,
        pattern: &Pattern,
        value: &Expr,
        body: &Expr,
    ) -> Result<Value, InterpreterError> {
        // Evaluate the right-hand side value
        let rhs_value = self.eval_expr(value)?;

        // Extract names marked as mutable in the pattern
        let mut_names = Self::extract_mut_names(pattern);

        // Try to match the pattern against the value
        if let Some(bindings) = self.try_pattern_match(pattern, &rhs_value)? {
            // Bind pattern variables, using mutable binding for names wrapped in Pattern::Mut
            for (name, val) in bindings {
                if mut_names.contains(&name) {
                    self.env_set_mut(name.clone(), val);
                } else {
                    self.env_set(name, val);
                }
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
    /// Evaluates assignment expressions including field assignments.
    ///
    /// This method handles variable assignments (`x = value`) and field assignments (`obj.field = value`).
    /// For field assignments, it creates a new object with the updated field value.
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::frontend::parser::Parser;
    /// use ruchy::runtime::interpreter::{Interpreter, Value};
    ///
    /// let mut interpreter = Interpreter::new();
    /// let code = r#"
    ///     class Point {
    ///         x: i32,
    ///         y: i32
    ///
    ///         new(x: i32, y: i32) {
    ///             self.x = x
    ///             self.y = y
    ///         }
    ///     }
    ///
    ///     fn main() {
    ///         let p = Point::new(10, 20)
    ///         p.x
    ///     }
    /// "#;
    ///
    /// let mut parser = Parser::new(code);
    /// let expr = parser.parse().unwrap();
    /// interpreter.eval_expr(&expr).unwrap();
    /// let main_call = Parser::new("main()").parse().unwrap();
    /// let result = interpreter.eval_expr(&main_call).unwrap();
    /// assert!(matches!(result, Value::Integer(10)));
    /// ```
    fn eval_assign(&mut self, target: &Expr, value: &Expr) -> Result<Value, InterpreterError> {
        let val = self.eval_expr(value)?;

        // Handle different assignment targets
        match &target.kind {
            ExprKind::Identifier(name) => {
                self.set_variable(name, val.clone());
                Ok(val)
            }
            ExprKind::FieldAccess { object, field } => {
                // Handle field assignment like: obj.field = value
                // We need to get the object, update it, and reassign it
                match &object.kind {
                    ExprKind::Identifier(obj_name) => {
                        // Get the object
                        let obj = self.lookup_variable(obj_name)?;

                        // Update the field based on object type
                        match obj {
                            Value::Object(ref map) => {
                                // Immutable object: create new copy with updated field
                                let mut new_map = (**map).clone();
                                new_map.insert(field.clone(), val.clone());
                                let new_obj = Value::Object(Arc::new(new_map));

                                // Update the variable with the modified object
                                self.set_variable(obj_name, new_obj);
                                Ok(val)
                            }
                            Value::ObjectMut(ref cell) => {
                                // Mutable object: update in place via RefCell
                                cell.lock().unwrap().insert(field.clone(), val.clone());
                                Ok(val)
                            }
                            Value::Class { ref fields, .. } => {
                                // Class: update field in place via RwLock
                                let mut fields_write = fields.write().unwrap();
                                fields_write.insert(field.clone(), val.clone());
                                Ok(val)
                            }
                            Value::Struct { ref name, ref fields } => {
                                // Struct: create new copy with updated field (value semantics)
                                let mut new_fields = (**fields).clone();
                                new_fields.insert(field.clone(), val.clone());
                                let new_struct = Value::Struct {
                                    name: name.clone(),
                                    fields: Arc::new(new_fields),
                                };

                                // Update the variable with the modified struct
                                self.set_variable(obj_name, new_struct);
                                Ok(val)
                            }
                            _ => Err(InterpreterError::RuntimeError(format!(
                                "Cannot access field '{}' on non-object",
                                field
                            ))),
                        }
                    }
                    _ => Err(InterpreterError::RuntimeError(
                        "Complex assignment targets not yet supported".to_string(),
                    )),
                }
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
        s: &Arc<str>,
        method: &str,
        args: &[Value],
    ) -> Result<Value, InterpreterError> {
        super::eval_string_methods::eval_string_method(s, method, args)
    }

    /// Evaluate array methods
    #[allow(clippy::rc_buffer)]
    fn eval_array_method(
        &mut self,
        arr: &Arc<[Value]>,
        method: &str,
        args: &[Value],
    ) -> Result<Value, InterpreterError> {
        // Delegate to extracted array module with function call capability
        crate::runtime::eval_array::eval_array_method(arr, method, args, |func, args| {
            self.eval_function_call_value(func, args)
        })
    }

    /// Evaluate a method call
    pub fn eval_method_call(
        &mut self,
        receiver: &Expr,
        method: &str,
        args: &[Expr],
    ) -> Result<Value, InterpreterError> {
        // Special handling for stdlib namespace methods (e.g., Html.parse())
        if let ExprKind::Identifier(namespace) = &receiver.kind {
            // Check if this is a stdlib namespace call before trying to look it up as a variable
            let namespace_method = format!("{namespace}_{method}");

            // Try to evaluate as builtin function first
            let arg_values: Result<Vec<_>, _> = args.iter().map(|arg| self.eval_expr(arg)).collect();
            let arg_values = arg_values?;

            if let Ok(Some(result)) = crate::runtime::eval_builtin::eval_builtin_function(&namespace_method, &arg_values) {
                return Ok(result);
            }
        }

        // Special handling for mutating array methods on simple identifiers
        // e.g., messages.push(item)
        if let ExprKind::Identifier(var_name) = &receiver.kind {
            if method == "push" && args.len() == 1 {
                // Get current array value
                if let Ok(Value::Array(arr)) = self.lookup_variable(var_name) {
                    // Evaluate the argument
                    let arg_value = self.eval_expr(&args[0])?;

                    // Create new array with item added
                    let mut new_arr = arr.to_vec();
                    new_arr.push(arg_value);

                    // Update the variable binding
                    self.env_set(var_name.clone(), Value::Array(Arc::from(new_arr)));

                    return Ok(Value::Nil); // push returns nil
                }
            } else if method == "pop" && args.is_empty() {
                // Get current array value
                if let Ok(Value::Array(arr)) = self.lookup_variable(var_name) {
                    // Create new array with last item removed
                    let mut new_arr = arr.to_vec();
                    let popped_value = new_arr.pop().unwrap_or(Value::Nil);

                    // Update the variable binding
                    self.env_set(var_name.clone(), Value::Array(Arc::from(new_arr)));

                    return Ok(popped_value); // pop returns the removed item
                }
            }
        }

        // Special handling for mutating array methods on ObjectMut fields
        // e.g., self.messages.push(item)
        if let ExprKind::FieldAccess { object, field } = &receiver.kind {
            if let Ok(object_value) = self.eval_expr(object) {
                if let Value::ObjectMut(cell_rc) = object_value {
                    // Check if this is a mutating array method
                    if method == "push" && args.len() == 1 {
                        // Evaluate the argument
                        let arg_value = self.eval_expr(&args[0])?;

                        // Get mutable access to the object
                        let mut obj = cell_rc.lock().unwrap();

                        // Get the field value
                        if let Some(field_value) = obj.get(field) {
                            // If it's an array, push to it
                            if let Value::Array(arr) = field_value {
                                let mut new_arr = arr.to_vec();
                                new_arr.push(arg_value);
                                obj.insert(field.clone(), Value::Array(Arc::from(new_arr)));
                                return Ok(Value::Nil); // push returns nil
                            }
                        }
                    }
                }
            }
        }

        let receiver_value = self.eval_expr(receiver)?;

        // Special handling for DataFrame methods with closures - don't pre-evaluate the closure argument
        if matches!(receiver_value, Value::DataFrame { .. }) {
            match method {
                "filter" => return self.eval_dataframe_filter_method(&receiver_value, args),
                "with_column" => {
                    return self.eval_dataframe_with_column_method(&receiver_value, args)
                }
                "transform" => return self.eval_dataframe_transform_method(&receiver_value, args),
                _ => {}
            }
        }

        // Special handling for actor send/ask methods - convert undefined identifiers to messages
        if (method == "send" || method == "ask") && args.len() == 1 {
            // Check if receiver is an actor instance (immutable or mutable)
            let is_actor = match &receiver_value {
                Value::Object(ref obj) => obj.contains_key("__actor"),
                Value::ObjectMut(ref cell) => cell.lock().unwrap().contains_key("__actor"),
                _ => false,
            };

            if is_actor {
                // Try to evaluate the argument as a message
                let arg_value = match &args[0].kind {
                    ExprKind::Identifier(name) => {
                        // Try to evaluate as variable first
                        if let Ok(val) = self.lookup_variable(name) {
                            val
                        } else {
                            // Treat as a zero-argument message constructor
                            let mut message = HashMap::new();
                            message.insert(
                                "__type".to_string(),
                                Value::from_string("Message".to_string()),
                            );
                            message.insert("type".to_string(), Value::from_string(name.clone()));
                            message.insert("data".to_string(), Value::Array(Arc::from(vec![])));
                            Value::Object(Arc::new(message))
                        }
                    }
                    _ => self.eval_expr(&args[0])?,
                };
                return self.dispatch_method_call(&receiver_value, method, &[arg_value], false);
            }
        }

        let arg_values: Result<Vec<_>, _> = args.iter().map(|arg| self.eval_expr(arg)).collect();
        let arg_values = arg_values?;

        self.dispatch_method_call(&receiver_value, method, &arg_values, args.is_empty())
    }

    // Helper methods for method dispatch (complexity <10 each)

    /// Evaluate a message expression - if it's an undefined identifier, treat as message name
    /// Complexity: ≤5
    fn eval_message_expr(&mut self, message: &Expr) -> Result<Value, InterpreterError> {
        match &message.kind {
            ExprKind::Identifier(name) => {
                // Try to evaluate as variable first
                if let Ok(val) = self.lookup_variable(name) {
                    Ok(val)
                } else {
                    // Treat as a zero-argument message constructor
                    let mut msg_obj = HashMap::new();
                    msg_obj.insert(
                        "__type".to_string(),
                        Value::from_string("Message".to_string()),
                    );
                    msg_obj.insert("type".to_string(), Value::from_string(name.clone()));
                    msg_obj.insert("data".to_string(), Value::Array(Arc::from(vec![])));
                    Ok(Value::Object(Arc::new(msg_obj)))
                }
            }
            _ => self.eval_expr(message),
        }
    }

    fn dispatch_method_call(
        &mut self,
        receiver: &Value,
        method: &str,
        arg_values: &[Value],
        args_empty: bool,
    ) -> Result<Value, InterpreterError> {
        // EVALUATOR-001: Strip turbofish syntax from method names
        // Example: "parse::<i32>" becomes "parse"
        // Turbofish is for type hints only, not used in runtime method lookup
        let base_method = if let Some(pos) = method.find("::") {
            &method[..pos]
        } else {
            method
        };

        match receiver {
            Value::String(s) => self.eval_string_method(s, base_method, arg_values),
            Value::Array(arr) => self.eval_array_method(arr, base_method, arg_values),
            Value::Float(f) => self.eval_float_method(*f, base_method, args_empty),
            Value::Integer(n) => self.eval_integer_method(*n, base_method, arg_values),
            Value::DataFrame { columns } => self.eval_dataframe_method(columns, base_method, arg_values),
            Value::Object(obj) => {
                // Check if this is a type definition with constructor
                if base_method == "new" {
                    if let Some(Value::String(ref type_str)) = obj.get("__type") {
                        if let Some(Value::String(ref name)) = obj.get("__name") {
                            match type_str.as_ref() {
                                "Actor" => {
                                    return self.instantiate_actor_with_args(name, arg_values);
                                }
                                "Struct" => {
                                    return self.instantiate_struct_with_args(name, arg_values);
                                }
                                "Class" => {
                                    return self.instantiate_class_with_constructor(
                                        name, "new", arg_values,
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                }

                // Check if this is an actor instance
                if let Some(Value::String(actor_name)) = obj.get("__actor") {
                    self.eval_actor_instance_method(obj, actor_name.as_ref(), base_method, arg_values)
                }
                // Check if this is a class instance
                else if let Some(Value::String(class_name)) = obj.get("__class") {
                    self.eval_class_instance_method(obj, class_name.as_ref(), base_method, arg_values)
                }
                // Check if this is a struct instance with impl methods
                else if let Some(Value::String(struct_name)) =
                    obj.get("__struct_type").or_else(|| obj.get("__struct"))
                {
                    self.eval_struct_instance_method(obj, struct_name.as_ref(), base_method, arg_values)
                }
                // Check if this is a `DataFrame` builder
                else if let Some(Value::String(type_str)) = obj.get("__type") {
                    if type_str.as_ref() == "DataFrameBuilder" {
                        self.eval_dataframe_builder_method(obj, base_method, arg_values)
                    } else {
                        self.eval_object_method(obj, base_method, arg_values, args_empty)
                    }
                } else {
                    self.eval_object_method(obj, base_method, arg_values, args_empty)
                }
            }
            Value::ObjectMut(cell_rc) => {
                // Dispatch mutable objects the same way as immutable ones
                // Safe borrow: We only read metadata fields to determine dispatch
                let obj = cell_rc.lock().unwrap();

                // Check if this is an actor instance
                if let Some(Value::String(actor_name)) = obj.get("__actor") {
                    let actor_name = actor_name.clone();
                    drop(obj); // Release borrow before recursive call
                    self.eval_actor_instance_method_mut(
                        cell_rc,
                        actor_name.as_ref(),
                        base_method,
                        arg_values,
                    )
                }
                // Check if this is a class instance
                else if let Some(Value::String(class_name)) = obj.get("__class") {
                    let class_name = class_name.clone();
                    drop(obj); // Release borrow before recursive call
                    self.eval_class_instance_method_mut(
                        cell_rc,
                        class_name.as_ref(),
                        base_method,
                        arg_values,
                    )
                }
                // Check if this is a struct instance with impl methods
                else if let Some(Value::String(struct_name)) =
                    obj.get("__struct_type").or_else(|| obj.get("__struct"))
                {
                    let struct_name = struct_name.clone();
                    drop(obj); // Release borrow before recursive call
                    self.eval_struct_instance_method_mut(
                        cell_rc,
                        struct_name.as_ref(),
                        base_method,
                        arg_values,
                    )
                } else {
                    drop(obj); // Release borrow before recursive call
                    self.eval_object_method_mut(cell_rc, base_method, arg_values, args_empty)
                }
            }
            Value::Class {
                class_name,
                fields,
                methods,
            } => {
                // Dispatch instance method call on Class
                self.eval_class_instance_method_on_class(class_name, fields, methods, base_method, arg_values)
            }
            #[cfg(not(target_arch = "wasm32"))]
            Value::HtmlDocument(doc) => self.eval_html_document_method(doc, base_method, arg_values),
            #[cfg(not(target_arch = "wasm32"))]
            Value::HtmlElement(elem) => self.eval_html_element_method(elem, base_method, arg_values),
            _ => self.eval_generic_method(receiver, base_method, args_empty),
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
        arg_values: &[Value],
    ) -> Result<Value, InterpreterError> {
        eval_method::eval_integer_method(n, method, arg_values)
    }

    fn eval_generic_method(
        &self,
        receiver: &Value,
        method: &str,
        args_empty: bool,
    ) -> Result<Value, InterpreterError> {
        eval_method::eval_generic_method(receiver, method, args_empty)
    }

    /// Evaluate `HtmlDocument` methods (HTTP-002-C)
    /// Complexity: 4 (within Toyota Way limits)
    #[cfg(not(target_arch = "wasm32"))]
    fn eval_html_document_method(
        &self,
        doc: &crate::stdlib::html::HtmlDocument,
        method: &str,
        arg_values: &[Value],
    ) -> Result<Value, InterpreterError> {
        match method {
            "select" => {
                if arg_values.len() != 1 {
                    return Err(InterpreterError::RuntimeError(
                        "select() expects 1 argument (selector)".to_string(),
                    ));
                }
                match &arg_values[0] {
                    Value::String(selector) => {
                        let elements = doc
                            .select(selector.as_ref())
                            .map_err(|e| InterpreterError::RuntimeError(format!("select() failed: {e}")))?;
                        let values: Vec<Value> = elements
                            .into_iter()
                            .map(Value::HtmlElement)
                            .collect();
                        Ok(Value::Array(values.into()))
                    }
                    _ => Err(InterpreterError::RuntimeError(
                        "select() expects a string selector".to_string(),
                    )),
                }
            }
            "query_selector" => {
                if arg_values.len() != 1 {
                    return Err(InterpreterError::RuntimeError(
                        "query_selector() expects 1 argument (selector)".to_string(),
                    ));
                }
                match &arg_values[0] {
                    Value::String(selector) => {
                        let element = doc
                            .query_selector(selector.as_ref())
                            .map_err(|e| InterpreterError::RuntimeError(format!("query_selector() failed: {e}")))?;
                        Ok(element.map_or(Value::Nil, Value::HtmlElement))
                    }
                    _ => Err(InterpreterError::RuntimeError(
                        "query_selector() expects a string selector".to_string(),
                    )),
                }
            }
            "query_selector_all" => {
                if arg_values.len() != 1 {
                    return Err(InterpreterError::RuntimeError(
                        "query_selector_all() expects 1 argument (selector)".to_string(),
                    ));
                }
                match &arg_values[0] {
                    Value::String(selector) => {
                        let elements = doc
                            .query_selector_all(selector.as_ref())
                            .map_err(|e| InterpreterError::RuntimeError(format!("query_selector_all() failed: {e}")))?;
                        let values: Vec<Value> = elements
                            .into_iter()
                            .map(Value::HtmlElement)
                            .collect();
                        Ok(Value::Array(values.into()))
                    }
                    _ => Err(InterpreterError::RuntimeError(
                        "query_selector_all() expects a string selector".to_string(),
                    )),
                }
            }
            _ => Err(InterpreterError::RuntimeError(format!(
                "Unknown method '{}' on HtmlDocument",
                method
            ))),
        }
    }

    /// Evaluate `HtmlElement` methods (HTTP-002-C)
    /// Complexity: 4 (within Toyota Way limits)
    #[cfg(not(target_arch = "wasm32"))]
    fn eval_html_element_method(
        &self,
        elem: &crate::stdlib::html::HtmlElement,
        method: &str,
        arg_values: &[Value],
    ) -> Result<Value, InterpreterError> {
        match method {
            "text" => {
                if !arg_values.is_empty() {
                    return Err(InterpreterError::RuntimeError(
                        "text() expects no arguments".to_string(),
                    ));
                }
                Ok(Value::from_string(elem.text()))
            }
            "attr" => {
                if arg_values.len() != 1 {
                    return Err(InterpreterError::RuntimeError(
                        "attr() expects 1 argument (attribute name)".to_string(),
                    ));
                }
                match &arg_values[0] {
                    Value::String(attr_name) => {
                        let value = elem.attr(attr_name.as_ref());
                        Ok(value.map_or(Value::Nil, Value::from_string))
                    }
                    _ => Err(InterpreterError::RuntimeError(
                        "attr() expects a string attribute name".to_string(),
                    )),
                }
            }
            "html" => {
                if !arg_values.is_empty() {
                    return Err(InterpreterError::RuntimeError(
                        "html() expects no arguments".to_string(),
                    ));
                }
                Ok(Value::from_string(elem.html()))
            }
            _ => Err(InterpreterError::RuntimeError(format!(
                "Unknown method '{}' on HtmlElement",
                method
            ))),
        }
    }

    // ObjectMut adapter methods - delegate to immutable versions via borrow
    // Complexity: 2 each (simple delegation)

    fn eval_actor_instance_method_mut(
        &mut self,
        cell_rc: &Arc<std::sync::Mutex<std::collections::HashMap<String, Value>>>,
        actor_name: &str,
        method: &str,
        arg_values: &[Value],
    ) -> Result<Value, InterpreterError> {
        // Special handling for send method - needs mutable state access
        if method == "send" {
            if arg_values.is_empty() {
                return Err(InterpreterError::RuntimeError(
                    "send() requires a message argument".to_string(),
                ));
            }
            return self.process_actor_message_sync_mut(cell_rc, &arg_values[0]);
        }

        // For other methods, delegate to non-mut version
        let instance = cell_rc.lock().unwrap();
        self.eval_actor_instance_method(&instance, actor_name, method, arg_values)
    }

    fn eval_class_instance_method_mut(
        &mut self,
        cell_rc: &Arc<std::sync::Mutex<std::collections::HashMap<String, Value>>>,
        class_name: &str,
        method: &str,
        arg_values: &[Value],
    ) -> Result<Value, InterpreterError> {
        // For mutable instances, we need to pass ObjectMut as self (not a copy)
        // This allows &mut self methods to mutate the instance in place

        // Look up the class definition
        let class_def = self.lookup_variable(class_name)?;

        if let Value::Object(ref class_info) = class_def {
            // Look for the method in the class definition
            if let Some(Value::Object(ref methods)) = class_info.get("__methods") {
                if let Some(Value::Object(ref method_meta)) = methods.get(method) {
                    // Get the method closure
                    if let Some(Value::Closure { params, body, .. }) = method_meta.get("closure") {
                        // Check if it's a static method
                        let is_static = method_meta
                            .get("is_static")
                            .and_then(|v| {
                                if let Value::Bool(b) = v {
                                    Some(*b)
                                } else {
                                    None
                                }
                            })
                            .unwrap_or(false);

                        if is_static {
                            return Err(InterpreterError::RuntimeError(format!(
                                "Cannot call static method {} on instance",
                                method
                            )));
                        }

                        // Create environment for method execution
                        let mut method_env = HashMap::new();

                        // CRITICAL: Pass ObjectMut as self, using the SAME Arc<RefCell<>>
                        // This enables &mut self methods to mutate the shared instance
                        method_env
                            .insert("self".to_string(), Value::ObjectMut(Arc::clone(cell_rc)));

                        // Bind method parameters to arguments
                        if arg_values.len() != params.len() {
                            return Err(InterpreterError::RuntimeError(format!(
                                "Method {} expects {} arguments, got {}",
                                method,
                                params.len(),
                                arg_values.len()
                            )));
                        }

                        for (param, arg) in params.iter().zip(arg_values) {
                            method_env.insert(param.clone(), arg.clone());
                        }

                        // Push method environment
                        self.env_push(method_env);

                        // Execute method body
                        let result = self.eval_expr(body)?;

                        // Pop environment
                        self.env_pop();

                        return Ok(result);
                    }
                }
            }

            // Method not found
            Err(InterpreterError::RuntimeError(format!(
                "Class {} has no method named {}",
                class_name, method
            )))
        } else {
            Err(InterpreterError::RuntimeError(format!(
                "{} is not a class",
                class_name
            )))
        }
    }

    /// Evaluate instance method on `Value::Class` variant
    /// This is for the new Class implementation with Arc<`RwLock`<HashMap>>
    fn eval_class_instance_method_on_class(
        &mut self,
        class_name: &str,
        fields: &Arc<std::sync::RwLock<HashMap<String, Value>>>,
        methods: &Arc<HashMap<String, Value>>,
        method: &str,
        arg_values: &[Value],
    ) -> Result<Value, InterpreterError> {
        // Look up the method in the methods map
        if let Some(method_closure) = methods.get(method) {
            if let Value::Closure { params, body, .. } = method_closure {
                // Check argument count
                if arg_values.len() != params.len() {
                    return Err(InterpreterError::RuntimeError(format!(
                        "Method {} expects {} arguments, got {}",
                        method,
                        params.len(),
                        arg_values.len()
                    )));
                }

                // Create environment for method execution
                let mut method_env = HashMap::new();

                // Bind 'self' to the class instance
                method_env.insert(
                    "self".to_string(),
                    Value::Class {
                        class_name: class_name.to_string(),
                        fields: Arc::clone(fields),
                        methods: Arc::clone(methods),
                    },
                );

                // Bind method parameters to arguments
                for (param, arg) in params.iter().zip(arg_values) {
                    method_env.insert(param.clone(), arg.clone());
                }

                // Push method environment
                self.env_push(method_env);

                // Execute method body
                let result = self.eval_expr(body)?;

                // Pop environment
                self.env_pop();

                Ok(result)
            } else {
                Err(InterpreterError::RuntimeError(format!(
                    "Method {} is not a closure",
                    method
                )))
            }
        } else {
            Err(InterpreterError::RuntimeError(format!(
                "Method '{}' not found for type class",
                method
            )))
        }
    }

    fn eval_struct_instance_method_mut(
        &mut self,
        cell_rc: &Arc<std::sync::Mutex<std::collections::HashMap<String, Value>>>,
        struct_name: &str,
        method: &str,
        arg_values: &[Value],
    ) -> Result<Value, InterpreterError> {
        let instance = cell_rc.lock().unwrap();
        self.eval_struct_instance_method(&instance, struct_name, method, arg_values)
    }

    fn eval_object_method_mut(
        &mut self,
        cell_rc: &Arc<std::sync::Mutex<std::collections::HashMap<String, Value>>>,
        method: &str,
        arg_values: &[Value],
        args_empty: bool,
    ) -> Result<Value, InterpreterError> {
        let instance = cell_rc.lock().unwrap();
        self.eval_object_method(&instance, method, arg_values, args_empty)
    }

    /// Evaluates actor instance methods like `send()` and `ask()`.
    ///
    /// This method handles message passing to actors using the `!` (send) and `<?` (ask) operators.
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::frontend::parser::Parser;
    /// use ruchy::runtime::interpreter::Interpreter;
    ///
    /// let mut interpreter = Interpreter::new();
    /// let code = r#"
    ///     actor Counter {
    ///         count: i32 = 0
    ///
    ///         receive {
    ///             Increment => 42
    ///         }
    ///     }
    ///
    ///     fn main() {
    ///         let counter = spawn Counter
    ///         counter ! Increment
    ///         counter
    ///     }
    /// "#;
    ///
    /// let mut parser = Parser::new(code);
    /// let expr = parser.parse().unwrap();
    /// interpreter.eval_expr(&expr).unwrap();
    /// let main_call = Parser::new("main()").parse().unwrap();
    /// let result = interpreter.eval_expr(&main_call).unwrap();
    /// // Actor instance returned
    /// ```
    fn eval_actor_instance_method(
        &mut self,
        instance: &std::collections::HashMap<String, Value>,
        _actor_name: &str,
        method: &str,
        arg_values: &[Value],
    ) -> Result<Value, InterpreterError> {
        match method {
            "send" => {
                // Send a message to the actor (fire-and-forget)
                if arg_values.is_empty() {
                    return Err(InterpreterError::RuntimeError(
                        "send() requires a message argument".to_string(),
                    ));
                }

                // Check if this is an async actor with runtime ID
                if let Some(Value::String(actor_id)) = instance.get("__actor_id") {
                    use crate::runtime::actor_runtime::{ActorMessage, ACTOR_RUNTIME};

                    // Extract message type and data
                    let message = &arg_values[0];
                    let (msg_type, msg_data) = if let Value::Object(msg_obj) = message {
                        if let Some(Value::String(type_str)) = msg_obj.get("__type") {
                            if type_str.as_ref() == "Message" {
                                let msg_type = msg_obj
                                    .get("type")
                                    .and_then(|v| {
                                        if let Value::String(s) = v {
                                            Some(s.to_string())
                                        } else {
                                            None
                                        }
                                    })
                                    .unwrap_or_else(|| "Unknown".to_string());
                                let msg_data = msg_obj
                                    .get("data")
                                    .and_then(|v| {
                                        if let Value::Array(arr) = v {
                                            Some(arr.to_vec())
                                        } else {
                                            None
                                        }
                                    })
                                    .unwrap_or_else(Vec::new);
                                (msg_type, msg_data)
                            } else {
                                ("Unknown".to_string(), vec![])
                            }
                        } else {
                            ("Unknown".to_string(), vec![])
                        }
                    } else {
                        // Simple message value
                        ("Message".to_string(), vec![message.clone()])
                    };

                    // Convert data to strings for thread safety
                    let str_data: Vec<String> =
                        msg_data.iter().map(|v| format!("{:?}", v)).collect();

                    // Send the message to the actor
                    let actor_msg = ActorMessage {
                        message_type: msg_type,
                        data: str_data,
                    };

                    ACTOR_RUNTIME.send_message(actor_id.as_ref(), actor_msg)?;
                    return Ok(Value::Nil);
                }

                // Synchronous actor - process message immediately
                self.process_actor_message_sync(instance, &arg_values[0])
            }
            "stop" => {
                // Stop the actor
                // In a real actor system, this would terminate the actor's mailbox processing
                Ok(Value::Bool(true))
            }
            "ask" => {
                // Send a message and wait for response
                // For now, we'll process the message synchronously
                if arg_values.is_empty() {
                    return Err(InterpreterError::RuntimeError(
                        "ask() requires a message argument".to_string(),
                    ));
                }

                // Get the message
                let message = &arg_values[0];

                // Try to extract message type and data
                if let Value::Object(msg_obj) = message {
                    // Check if this is a Message object we created
                    if let Some(Value::String(type_str)) = msg_obj.get("__type") {
                        if type_str.as_ref() == "Message" {
                            // Extract message type and data
                            if let Some(Value::String(msg_type)) = msg_obj.get("type") {
                                if let Some(Value::Array(data)) = msg_obj.get("data") {
                                    // Look up the handler for this message type
                                    if let Some(handlers) = instance.get("__handlers") {
                                        if let Value::Array(handler_list) = handlers {
                                            // Find matching handler
                                            for handler in handler_list.iter() {
                                                if let Value::Object(h) = handler {
                                                    if let Some(Value::String(h_type)) =
                                                        h.get("message_type")
                                                    {
                                                        if h_type.as_ref() == msg_type.as_ref() {
                                                            // Found matching handler - execute it
                                                            if let Some(Value::Closure {
                                                                params,
                                                                body,
                                                                env,
                                                            }) = h.get("handler")
                                                            {
                                                                // Push a new environment for handler execution
                                                                let mut handler_env =
                                                                    (**env).clone();

                                                                // Bind message parameters
                                                                for (i, param_name) in
                                                                    params.iter().enumerate()
                                                                {
                                                                    if let Some(value) = data.get(i)
                                                                    {
                                                                        handler_env.insert(
                                                                            param_name.clone(),
                                                                            value.clone(),
                                                                        );
                                                                    }
                                                                }

                                                                // Also bind 'self' to the actor instance
                                                                handler_env.insert(
                                                                    "self".to_string(),
                                                                    Value::Object(Arc::new(
                                                                        instance.clone(),
                                                                    )),
                                                                );

                                                                // Execute handler body
                                                                self.env_push(handler_env);
                                                                let result =
                                                                    self.eval_expr(body)?;
                                                                self.env_pop();

                                                                return Ok(result);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // No handler found - return a default response
                                    return Ok(Value::from_string(format!(
                                        "Received: {}",
                                        msg_type.as_ref()
                                    )));
                                }
                            }
                        }
                    }
                }

                // Default: return the message itself (echo)
                Ok(message.clone())
            }
            _ => Err(InterpreterError::RuntimeError(format!(
                "Unknown actor method: {}",
                method
            ))),
        }
    }

    /// Process a message for a synchronous (interpreted) actor.
    ///
    /// This method executes the appropriate message handler based on the message type.
    /// Complexity: 9
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::frontend::parser::Parser;
    /// use ruchy::runtime::interpreter::Interpreter;
    ///
    /// let mut interpreter = Interpreter::new();
    /// let code = r#"
    ///     actor Greeter {
    ///         greeting: String = "Hello"
    ///
    ///         receive {
    ///             Greet(name: String) => {
    ///                 "Hello, World!"
    ///             }
    ///         }
    ///     }
    ///
    ///     fn main() {
    ///         let greeter = spawn Greeter
    ///         greeter ! Greet("Alice")
    ///         greeter
    ///     }
    /// "#;
    ///
    /// let mut parser = Parser::new(code);
    /// let expr = parser.parse().unwrap();
    /// interpreter.eval_expr(&expr).unwrap();
    /// let main_call = Parser::new("main()").parse().unwrap();
    /// let result = interpreter.eval_expr(&main_call);
    /// assert!(result.is_ok());
    /// ```
    fn process_actor_message_sync(
        &mut self,
        instance: &std::collections::HashMap<String, Value>,
        message: &Value,
    ) -> Result<Value, InterpreterError> {
        // Parse the message to extract type and arguments
        // Messages come as function calls like Push(1) or SetCount(5)
        let (msg_type, msg_args) = if let Value::Object(msg_obj) = message {
            // Check if it's a Message object
            if let Some(Value::String(type_str)) = msg_obj.get("__type") {
                if type_str.as_ref() == "Message" {
                    let msg_type = msg_obj
                        .get("type")
                        .and_then(|v| {
                            if let Value::String(s) = v {
                                Some(s.to_string())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| "Unknown".to_string());
                    let msg_args = msg_obj
                        .get("data")
                        .and_then(|v| {
                            if let Value::Array(arr) = v {
                                Some(arr.to_vec())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(Vec::new);
                    (msg_type, msg_args)
                } else {
                    return Err(InterpreterError::RuntimeError(
                        "Invalid message format".to_string(),
                    ));
                }
            } else {
                return Err(InterpreterError::RuntimeError(
                    "Invalid message format".to_string(),
                ));
            }
        } else {
            return Err(InterpreterError::RuntimeError(
                "Message must be an object".to_string(),
            ));
        };

        // Find the matching handler
        if let Some(Value::Array(handlers)) = instance.get("__handlers") {
            for handler in handlers.iter() {
                if let Value::Object(handler_obj) = handler {
                    if let Some(Value::String(handler_type)) = handler_obj.get("message_type") {
                        if handler_type.as_ref() == msg_type {
                            // Found matching handler - execute it
                            if let Some(Value::Closure { params, body, env }) =
                                handler_obj.get("body")
                            {
                                // Create a new environment for handler execution
                                let mut handler_env = (**env).clone();

                                // Bind message parameters
                                for (i, param_name) in params.iter().enumerate() {
                                    if let Some(value) = msg_args.get(i) {
                                        handler_env.insert(param_name.clone(), value.clone());
                                    }
                                }

                                // Bind 'self' to the actor instance
                                // Create a mutable object for self that includes all fields
                                let mut self_obj = HashMap::new();
                                for (key, value) in instance {
                                    if !key.starts_with("__") {
                                        self_obj.insert(key.clone(), value.clone());
                                    }
                                }
                                handler_env
                                    .insert("self".to_string(), Value::Object(Arc::new(self_obj)));

                                // Execute the handler body
                                self.env_stack.push(handler_env);
                                let result = self.eval_expr(body);
                                self.env_stack.pop();

                                return result;
                            }
                        }
                    }
                }
            }
        }

        Err(InterpreterError::RuntimeError(format!(
            "No handler found for message type: {}",
            msg_type
        )))
    }

    /// Process a message for a synchronous (interpreted) actor with mutable state.
    ///
    /// This version accepts `Arc<Mutex<HashMap>>` and passes `ObjectMut` as self to enable mutations.
    /// Complexity: 9
    fn process_actor_message_sync_mut(
        &mut self,
        cell_rc: &Arc<std::sync::Mutex<std::collections::HashMap<String, Value>>>,
        message: &Value,
    ) -> Result<Value, InterpreterError> {
        let instance = cell_rc.lock().unwrap();

        // Parse the message to extract type and arguments
        let (msg_type, msg_args) = if let Value::Object(msg_obj) = message {
            if let Some(Value::String(type_str)) = msg_obj.get("__type") {
                if type_str.as_ref() == "Message" {
                    let msg_type = msg_obj
                        .get("type")
                        .and_then(|v| {
                            if let Value::String(s) = v {
                                Some(s.to_string())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| "Unknown".to_string());
                    let msg_args = msg_obj
                        .get("data")
                        .and_then(|v| {
                            if let Value::Array(arr) = v {
                                Some(arr.to_vec())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(Vec::new);
                    (msg_type, msg_args)
                } else {
                    return Err(InterpreterError::RuntimeError(
                        "Invalid message format".to_string(),
                    ));
                }
            } else {
                return Err(InterpreterError::RuntimeError(
                    "Invalid message format".to_string(),
                ));
            }
        } else {
            return Err(InterpreterError::RuntimeError(
                "Message must be an object".to_string(),
            ));
        };

        // Find the matching handler
        if let Some(Value::Array(handlers)) = instance.get("__handlers") {
            for handler in handlers.iter() {
                if let Value::Object(handler_obj) = handler {
                    if let Some(Value::String(handler_type)) = handler_obj.get("message_type") {
                        if handler_type.as_ref() == msg_type {
                            // Found matching handler - execute it
                            if let Some(Value::Closure { params, body, env }) =
                                handler_obj.get("body")
                            {
                                // Clone data before dropping instance borrow
                                let params_clone = params.clone();
                                let body_clone = body.clone();
                                let env_clone = env.clone();

                                // Get parameter types for validation
                                let param_types = handler_obj.get("param_types").and_then(|v| {
                                    if let Value::Array(types) = v {
                                        Some(types.clone())
                                    } else {
                                        None
                                    }
                                });

                                drop(instance); // Release borrow before executing handler

                                // Validate parameter types before execution
                                if let Some(types) = param_types {
                                    for (i, expected_type_val) in types.iter().enumerate() {
                                        if let Value::String(expected_type) = expected_type_val {
                                            if let Some(actual_value) = msg_args.get(i) {
                                                let actual_type = actual_value.type_name();
                                                // Map Ruchy type names to runtime type names
                                                let expected_runtime_type =
                                                    match expected_type.as_ref() {
                                                        "i32" | "i64" | "int" => "integer",
                                                        "f32" | "f64" | "float" => "float",
                                                        "String" | "string" | "str" => "string",
                                                        "bool" => "boolean",
                                                        _ => expected_type.as_ref(),
                                                    };

                                                if actual_type != expected_runtime_type
                                                    && expected_runtime_type != "Any"
                                                {
                                                    return Err(InterpreterError::RuntimeError(format!(
                                                        "Type error in message {}: parameter {} expects type '{}', got '{}'",
                                                        msg_type, i, expected_runtime_type, actual_type
                                                    )));
                                                }
                                            }
                                        }
                                    }
                                }

                                // Create a new environment for handler execution
                                let mut handler_env = (*env_clone).clone();

                                // Bind message parameters
                                for (i, param_name) in params_clone.iter().enumerate() {
                                    if let Some(value) = msg_args.get(i) {
                                        handler_env.insert(param_name.clone(), value.clone());
                                    }
                                }

                                // CRITICAL: Bind 'self' to ObjectMut (not immutable Object)
                                // This allows mutations in the handler to persist
                                handler_env.insert(
                                    "self".to_string(),
                                    Value::ObjectMut(Arc::clone(cell_rc)),
                                );

                                // Execute the handler body
                                self.env_stack.push(handler_env);
                                let result = self.eval_expr(&body_clone);
                                self.env_stack.pop();

                                return result;
                            }
                        }
                    }
                }
            }
        }

        Err(InterpreterError::RuntimeError(format!(
            "No handler found for message type: {}",
            msg_type
        )))
    }

    fn eval_struct_instance_method(
        &mut self,
        instance: &std::collections::HashMap<String, Value>,
        struct_name: &str,
        method: &str,
        arg_values: &[Value],
    ) -> Result<Value, InterpreterError> {
        // Look up impl method with qualified name
        let qualified_method_name = format!("{}::{}", struct_name, method);

        if let Ok(method_closure) = self.lookup_variable(&qualified_method_name) {
            if let Value::Closure { params, body, env } = method_closure {
                // Check argument count (including self)
                let expected_args = params.len();
                let provided_args = arg_values.len() + 1; // +1 for self

                if provided_args != expected_args {
                    return Err(InterpreterError::RuntimeError(format!(
                        "Method {} expects {} arguments, got {}",
                        method,
                        expected_args - 1, // -1 because self is implicit
                        arg_values.len()
                    )));
                }

                // Create new environment with method's captured environment as base
                let mut new_env = (*env).clone();

                // Bind self parameter (first parameter)
                if let Some(self_param) = params.first() {
                    new_env.insert(
                        self_param.clone(),
                        Value::Object(std::sync::Arc::new(instance.clone())),
                    );
                }

                // Bind other parameters
                for (i, arg_value) in arg_values.iter().enumerate() {
                    if let Some(param_name) = params.get(i + 1) {
                        // +1 to skip self
                        new_env.insert(param_name.clone(), arg_value.clone());
                    }
                }

                // Execute method body with new environment
                self.env_stack.push(new_env);
                let result = self.eval_expr(&body);
                self.env_stack.pop();

                result
            } else {
                Err(InterpreterError::RuntimeError(format!(
                    "Found {} but it's not a method closure",
                    qualified_method_name
                )))
            }
        } else {
            // Fall back to generic method handling
            self.eval_generic_method(
                &Value::Object(std::sync::Arc::new(instance.clone())),
                method,
                arg_values.is_empty(),
            )
        }
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
            &Value::Object(std::sync::Arc::new(obj.clone())),
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

        // Store state field definitions with default values
        let mut fields = HashMap::new();
        for field in state {
            let type_name = match &field.ty.kind {
                crate::frontend::ast::TypeKind::Named(n) => n.clone(),
                _ => "Any".to_string(),
            };

            // Create field metadata object
            let mut field_meta = HashMap::new();
            field_meta.insert("type".to_string(), Value::from_string(type_name));
            field_meta.insert("is_mut".to_string(), Value::Bool(field.is_mut));

            // Evaluate default value if present
            if let Some(ref default_expr) = field.default_value {
                match self.eval_expr(default_expr) {
                    Ok(default_val) => {
                        field_meta.insert("default".to_string(), default_val);
                    }
                    Err(_) => {
                        // If evaluation fails, use type default
                        field_meta.insert("default".to_string(), Value::Nil);
                    }
                }
            } else {
                // No default value specified, use Nil
                field_meta.insert("default".to_string(), Value::Nil);
            }

            fields.insert(field.name.clone(), Value::Object(Arc::new(field_meta)));
        }
        actor_type.insert(
            "__fields".to_string(),
            Value::Object(std::sync::Arc::new(fields)),
        );

        // Store message handlers as closures
        let mut handlers_array = Vec::new();
        for handler in handlers {
            // Create a closure for each handler
            let mut handler_obj = HashMap::new();
            handler_obj.insert(
                "message_type".to_string(),
                Value::from_string(handler.message_type.clone()),
            );

            // Store params as strings
            let param_names: Vec<String> = handler
                .params
                .iter()
                .map(crate::frontend::ast::Param::name)
                .collect();
            handler_obj.insert(
                "params".to_string(),
                Value::Array(Arc::from(
                    param_names
                        .iter()
                        .map(|n| Value::from_string(n.clone()))
                        .collect::<Vec<_>>(),
                )),
            );

            // Store parameter types for runtime type checking
            let param_types: Vec<String> = handler
                .params
                .iter()
                .map(|p| match &p.ty.kind {
                    crate::frontend::ast::TypeKind::Named(name) => name.clone(),
                    _ => "Any".to_string(),
                })
                .collect();
            handler_obj.insert(
                "param_types".to_string(),
                Value::Array(Arc::from(
                    param_types
                        .iter()
                        .map(|t| Value::from_string(t.clone()))
                        .collect::<Vec<_>>(),
                )),
            );

            // Store the handler body AST node (we'll evaluate it later)
            // For now, store as a closure with the current environment
            handler_obj.insert(
                "body".to_string(),
                Value::Closure {
                    params: param_names,
                    body: Arc::new(*handler.body.clone()),
                    env: Arc::new(self.current_env().clone()),
                },
            );

            handlers_array.push(Value::Object(Arc::new(handler_obj)));
        }
        actor_type.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(handlers_array)),
        );

        // Register this actor type in the environment
        let actor_obj = Value::Object(std::sync::Arc::new(actor_type));
        self.set_variable(name, actor_obj.clone());

        Ok(actor_obj)
    }

    /// Evaluate struct definition
    /// Creates a struct type descriptor that can be used for instantiation
    /// Complexity: 7
    fn eval_struct_definition(
        &mut self,
        name: &str,
        _type_params: &[String], // Generic type parameters (not yet used in runtime)
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
            field_info.insert(
                "is_pub".to_string(),
                Value::from_bool(field.visibility.is_public()),
            );
            field_info.insert("is_mut".to_string(), Value::from_bool(field.is_mut));
            // Store visibility for access control
            let visibility_str = match field.visibility {
                crate::frontend::ast::Visibility::Public => "pub",
                crate::frontend::ast::Visibility::PubCrate => "pub(crate)",
                crate::frontend::ast::Visibility::PubSuper => "pub(super)",
                crate::frontend::ast::Visibility::Private => "private",
                crate::frontend::ast::Visibility::Protected => "protected",
            };
            field_info.insert(
                "visibility".to_string(),
                Value::from_string(visibility_str.to_string()),
            );

            // Store default value if present
            if let Some(default_expr) = &field.default_value {
                let default_val = self.eval_expr(default_expr)?;
                field_info.insert("default".to_string(), default_val);
            }

            field_defs.insert(
                field.name.clone(),
                Value::Object(std::sync::Arc::new(field_info)),
            );
        }

        struct_type.insert(
            "__fields".to_string(),
            Value::Object(std::sync::Arc::new(field_defs)),
        );

        // Register this struct type in the environment
        let struct_obj = Value::Object(std::sync::Arc::new(struct_type));
        self.set_variable(name, struct_obj.clone());

        Ok(struct_obj)
    }

    /// Evaluate enum definition
    /// Stores enum type with variant definitions in the environment
    /// Complexity: 6
    fn eval_enum_definition(
        &mut self,
        name: &str,
        _type_params: &[String], // Generic type parameters (not yet used in runtime)
        variants: &[crate::frontend::ast::EnumVariant],
        _is_pub: bool,
    ) -> Result<Value, InterpreterError> {
        use std::collections::HashMap;

        // Create an enum type object
        let mut enum_type = HashMap::new();

        // Store enum metadata
        enum_type.insert(
            "__type".to_string(),
            Value::from_string("Enum".to_string()),
        );
        enum_type.insert("__name".to_string(), Value::from_string(name.to_string()));

        // Store variant definitions
        let mut variant_defs = HashMap::new();
        for variant in variants {
            let mut variant_info = HashMap::new();

            // Store variant kind
            let kind_str = match &variant.kind {
                crate::frontend::ast::EnumVariantKind::Unit => "Unit",
                crate::frontend::ast::EnumVariantKind::Tuple(_) => "Tuple",
                crate::frontend::ast::EnumVariantKind::Struct(_) => "Struct",
            };
            variant_info.insert("kind".to_string(), Value::from_string(kind_str.to_string()));

            // Store discriminant if present
            if let Some(disc) = variant.discriminant {
                variant_info.insert("discriminant".to_string(), Value::Integer(disc));
            }

            variant_defs.insert(
                variant.name.clone(),
                Value::Object(std::sync::Arc::new(variant_info)),
            );
        }

        enum_type.insert(
            "__variants".to_string(),
            Value::Object(std::sync::Arc::new(variant_defs)),
        );

        // Register this enum type in the environment
        let enum_obj = Value::Object(std::sync::Arc::new(enum_type));
        self.set_variable(name, enum_obj.clone());

        Ok(enum_obj)
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

        // Handle Actor types differently
        if type_name == "Actor" {
            // Convert field expressions to values for actor instantiation
            let mut field_values = Vec::new();
            for (field_name, field_expr) in fields {
                let value = self.eval_expr(field_expr)?;
                field_values.push((field_name.clone(), value));
            }

            // Create an object with the named fields to pass to actor instantiation
            let mut args_obj = HashMap::new();
            for (name, value) in field_values {
                args_obj.insert(name, value);
            }

            // Call the actor instantiation function
            return self.instantiate_actor_with_args(name, &[Value::Object(Arc::new(args_obj))]);
        }

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

        // Create struct instance fields (without metadata)
        let mut instance_fields = HashMap::new();

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
            instance_fields.insert(field_name.clone(), field_value);
        }

        // Check that all required fields are provided or have defaults
        for (field_name, field_def_value) in field_defs.iter() {
            if !instance_fields.contains_key(field_name) {
                // Check if this field has a default value
                if let Value::Object(field_info) = field_def_value {
                    if let Some(default_val) = field_info.get("default") {
                        // Use default value
                        instance_fields.insert(field_name.clone(), default_val.clone());
                    } else {
                        // No default, field is required
                        return Err(InterpreterError::RuntimeError(format!(
                            "Missing required field '{field_name}' for struct {name}"
                        )));
                    }
                } else {
                    return Err(InterpreterError::RuntimeError(format!(
                        "Invalid field definition for '{field_name}' in struct {name}"
                    )));
                }
            }
        }

        // Return Value::Struct variant (not Object)
        Ok(Value::Struct {
            name: name.to_string(),
            fields: Arc::new(instance_fields),
        })
    }

    /// Evaluate class definition
    ///
    /// Supports:
    /// - Class fields with types and defaults
    /// - Multiple constructors (including named constructors)
    /// - Instance methods with self binding
    /// - Static methods (no self binding)
    /// - Inheritance metadata (superclass stored but not fully implemented)
    ///
    /// Limitations:
    /// - Instance mutations don't persist between method calls (needs `RefCell`)
    /// - Inheritance not fully implemented (no `super()` calls or field merging)
    /// - Method overriding not implemented
    ///
    /// Complexity: 8
    fn eval_class_definition(
        &mut self,
        name: &str,
        _type_params: &[String],
        superclass: Option<&String>,
        _traits: &[String],
        fields: &[crate::frontend::ast::StructField],
        constructors: &[crate::frontend::ast::Constructor],
        methods: &[crate::frontend::ast::ClassMethod],
        constants: &[crate::frontend::ast::ClassConstant],
        _derives: &[String],
        _is_pub: bool,
    ) -> Result<Value, InterpreterError> {
        use std::collections::HashMap;
        use std::sync::Arc;

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
            field_info.insert(
                "is_pub".to_string(),
                Value::Bool(field.visibility.is_public()),
            );
            field_info.insert("is_mut".to_string(), Value::Bool(field.is_mut));

            // Store default value if present
            if let Some(ref default) = field.default_value {
                // Evaluate default value
                let default_val = self.eval_expr(default)?;
                field_info.insert("default".to_string(), default_val);
            }

            field_defs.insert(
                field.name.clone(),
                Value::Object(std::sync::Arc::new(field_info)),
            );
        }
        class_info.insert("__fields".to_string(), Value::Object(Arc::new(field_defs)));

        // Store constructors as closures
        let mut constructor_info = HashMap::new();
        for constructor in constructors {
            // Store constructor by name (default name is "new")
            let ctor_name = constructor
                .name
                .as_ref()
                .unwrap_or(&"new".to_string())
                .clone();

            // Extract parameter names from the constructor params
            let param_names: Vec<String> = constructor
                .params
                .iter()
                .map(|p| match &p.pattern {
                    crate::frontend::ast::Pattern::Identifier(name) => name.clone(),
                    _ => "_".to_string(),
                })
                .collect();

            // Create a closure for the constructor
            let ctor_closure = Value::Closure {
                params: param_names,
                body: Arc::new((*constructor.body).clone()),
                env: Arc::new(HashMap::new()), // Empty env for now
            };

            constructor_info.insert(ctor_name, ctor_closure);
        }

        // If no constructors defined, create a default "new" constructor
        if constructor_info.is_empty() {
            // Create a default constructor that initializes fields with defaults
            let default_body = Expr::new(
                ExprKind::Block(Vec::new()), // Empty block - fields get initialized with defaults
                crate::frontend::ast::Span::new(0, 0),
            );

            let default_constructor = Value::Closure {
                params: Vec::new(), // No parameters
                body: Arc::new(default_body),
                env: Arc::new(HashMap::new()),
            };

            constructor_info.insert("new".to_string(), default_constructor);
        }

        class_info.insert(
            "__constructors".to_string(),
            Value::Object(Arc::new(constructor_info)),
        );

        // Store methods as closures with metadata
        let mut method_info = HashMap::new();
        for method in methods {
            // Extract parameter names from the method params (excluding 'self')
            let param_names: Vec<String> = method
                .params
                .iter()
                .filter_map(|p| match &p.pattern {
                    crate::frontend::ast::Pattern::Identifier(name) if name != "self" => {
                        Some(name.clone())
                    }
                    crate::frontend::ast::Pattern::Identifier(_) => None, // Skip 'self'
                    _ => Some("_".to_string()),
                })
                .collect();

            // Create a closure for the method
            let method_closure = Value::Closure {
                params: param_names,
                body: Arc::new((*method.body).clone()),
                env: Arc::new(HashMap::new()),
            };

            // Store method with metadata
            let mut method_meta = HashMap::new();
            method_meta.insert("closure".to_string(), method_closure);
            method_meta.insert("is_static".to_string(), Value::Bool(method.is_static));
            method_meta.insert("is_override".to_string(), Value::Bool(method.is_override));

            method_info.insert(method.name.clone(), Value::Object(Arc::new(method_meta)));
        }
        class_info.insert(
            "__methods".to_string(),
            Value::Object(Arc::new(method_info)),
        );

        // Store class constants
        let mut constants_info = HashMap::new();
        for constant in constants {
            // Evaluate the constant value
            let const_value = self.eval_expr(&constant.value)?;

            // Store constant with metadata
            let mut const_meta = HashMap::new();
            const_meta.insert("value".to_string(), const_value.clone());
            const_meta.insert(
                "type".to_string(),
                Value::from_string(format!("{:?}", constant.ty)),
            );
            const_meta.insert("is_pub".to_string(), Value::Bool(constant.is_pub));

            constants_info.insert(constant.name.clone(), Value::Object(Arc::new(const_meta)));

            // Also store the constant directly on the class for easy access
            // e.g., MyClass::CONSTANT_NAME
            let qualified_name = format!("{}::{}", name, constant.name);
            self.set_variable(&qualified_name, const_value);
        }
        class_info.insert(
            "__constants".to_string(),
            Value::Object(Arc::new(constants_info)),
        );

        // Store the class definition in the environment
        let class_value = Value::Object(Arc::new(class_info));
        self.set_variable(name, class_value.clone());

        Ok(class_value)
    }

    fn eval_impl_block(
        &mut self,
        for_type: &str,
        methods: &[crate::frontend::ast::ImplMethod],
    ) -> Result<Value, InterpreterError> {
        use std::collections::HashMap;
        use std::sync::Arc;

        // For struct impl blocks, we need to register methods that can be called on instances
        // We'll store them in a special registry keyed by type name
        let mut impl_methods = HashMap::new();

        for method in methods {
            // Extract parameter names from Param structs
            let param_names: Vec<String> = method
                .params
                .iter()
                .map(|p| match &p.pattern {
                    crate::frontend::ast::Pattern::Identifier(name) => name.clone(),
                    _ => "_".to_string(), // For other patterns, use placeholder
                })
                .collect();

            // Convert ImplMethod to a Value::Closure
            let closure = Value::Closure {
                params: param_names,
                body: Arc::new(*method.body.clone()),
                env: Arc::new(HashMap::new()),
            };
            impl_methods.insert(method.name.clone(), closure);
        }

        // Store the impl methods in a global registry
        // For now, we'll just add them to the environment with qualified names
        for (method_name, method_closure) in impl_methods {
            let qualified_name = format!("{}::{}", for_type, method_name);
            self.set_variable(&qualified_name, method_closure);
        }

        Ok(Value::Nil) // impl blocks don't return values
    }

    /// Instantiates a class by calling its constructor.
    ///
    /// This method creates a new class instance, initializes fields with default values,
    /// and executes the constructor body to set field values from arguments.
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::frontend::parser::Parser;
    /// use ruchy::runtime::interpreter::{Interpreter, Value};
    ///
    /// let mut interpreter = Interpreter::new();
    /// let code = r#"
    ///     class Point {
    ///         x: i32,
    ///         y: i32
    ///
    ///         new(x: i32, y: i32) {
    ///             self.x = x
    ///             self.y = y
    ///         }
    ///
    ///         fn get_x(&self) -> i32 {
    ///             self.x
    ///         }
    ///     }
    ///
    ///     fn main() {
    ///         let p = Point::new(3, 4)
    ///         p.get_x()
    ///     }
    /// "#;
    ///
    /// let mut parser = Parser::new(code);
    /// let expr = parser.parse().unwrap();
    /// interpreter.eval_expr(&expr).unwrap();
    /// let main_call = Parser::new("main()").parse().unwrap();
    /// let result = interpreter.eval_expr(&main_call).unwrap();
    /// assert!(matches!(result, Value::Integer(3)));
    /// ```
    fn instantiate_class_with_constructor(
        &mut self,
        class_name: &str,
        constructor_name: &str,
        args: &[Value],
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

            // Helper function to collect fields from class and its parents
            fn collect_all_fields(
                class_info: &HashMap<String, Value>,
                interpreter: &Interpreter,
            ) -> HashMap<String, Value> {
                let mut all_fields = HashMap::new();

                // First, get parent fields if there's a superclass
                if let Some(Value::String(ref parent_name)) = class_info.get("__superclass") {
                    if let Ok(Value::Object(ref parent_info)) =
                        interpreter.lookup_variable(parent_name)
                    {
                        let parent_fields = collect_all_fields(parent_info, interpreter);
                        all_fields.extend(parent_fields);
                    }
                }

                // Then add this class's fields (overriding parent fields if they exist)
                if let Some(Value::Object(ref fields)) = class_info.get("__fields") {
                    for (field_name, field_info) in fields.iter() {
                        if let Value::Object(ref field_meta) = field_info {
                            // Use default value if present
                            if let Some(default) = field_meta.get("default") {
                                all_fields.insert(field_name.clone(), default.clone());
                            } else {
                                // Initialize with nil
                                all_fields.insert(field_name.clone(), Value::Nil);
                            }
                        }
                    }
                }

                all_fields
            }

            // Initialize fields with default values from this class and all parent classes
            let all_fields = collect_all_fields(class_info, self);
            for (field_name, field_value) in all_fields {
                instance.insert(field_name, field_value);
            }

            // Execute the constructor if present
            if let Some(Value::Object(ref constructors)) = class_info.get("__constructors") {
                // Look for the specified constructor
                if let Some(constructor) = constructors.get(constructor_name) {
                    if let Value::Closure {
                        params,
                        body,
                        env: _,
                    } = constructor
                    {
                        // Check argument count
                        if args.len() != params.len() {
                            return Err(InterpreterError::RuntimeError(format!(
                                "constructor expects {} arguments, got {}",
                                params.len(),
                                args.len()
                            )));
                        }

                        // Create environment for constructor
                        let mut ctor_env = HashMap::new();

                        // Bind 'self' to mutable instance for constructor
                        ctor_env.insert(
                            "self".to_string(),
                            Value::Object(Arc::new(instance.clone())),
                        );

                        // Bind constructor parameters
                        for (param, arg) in params.iter().zip(args) {
                            ctor_env.insert(param.clone(), arg.clone());
                        }

                        // Push constructor environment
                        self.env_stack.push(ctor_env);

                        // Execute constructor body
                        let _result = self.eval_expr(body)?;

                        // Extract updated self from environment after constructor execution
                        // Note: Handles immutability by copying updated fields back to instance
                        let updated_self = self.lookup_variable("self")?;
                        if let Value::Object(ref updated_instance) = updated_self {
                            // Copy all non-metadata fields from updated self back to instance
                            for (key, value) in updated_instance.iter() {
                                if !key.starts_with("__") {
                                    instance.insert(key.clone(), value.clone());
                                }
                            }
                        }

                        // Pop environment
                        self.env_stack.pop();
                    }
                }
            }

            // Return ObjectMut for mutable class instances (support &mut self methods)
            Ok(crate::runtime::object_helpers::new_mutable_object(instance))
        } else {
            Err(InterpreterError::RuntimeError(format!(
                "{} is not a class definition",
                class_name
            )))
        }
    }

    /// Instantiate a class with arguments (calls init constructor)
    /// Returns `Value::Class` with reference semantics
    fn instantiate_class_with_args(
        &mut self,
        class_name: &str,
        args: &[Value],
    ) -> Result<Value, InterpreterError> {
        use std::sync::RwLock;

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

            // Collect methods from the class definition
            let mut methods_map = HashMap::new();
            if let Some(Value::Object(ref methods_obj)) = class_info.get("__methods") {
                for (method_name, method_value) in methods_obj.iter() {
                    // Extract the closure from method metadata
                    if let Value::Object(ref method_meta) = method_value {
                        if let Some(closure) = method_meta.get("closure") {
                            methods_map.insert(method_name.clone(), closure.clone());
                        }
                    }
                }
            }

            // Create instance fields with default values
            let mut instance_fields = HashMap::new();
            if let Some(Value::Object(ref fields)) = class_info.get("__fields") {
                for (field_name, field_info) in fields.iter() {
                    if let Value::Object(ref field_meta) = field_info {
                        // Use default value if present
                        if let Some(default) = field_meta.get("default") {
                            instance_fields.insert(field_name.clone(), default.clone());
                        } else {
                            // Initialize with nil
                            instance_fields.insert(field_name.clone(), Value::Nil);
                        }
                    }
                }
            }

            // Create the Class instance
            let class_instance = Value::Class {
                class_name: class_name.to_string(),
                fields: Arc::new(RwLock::new(instance_fields.clone())),
                methods: Arc::new(methods_map),
            };

            // Execute the init constructor if present
            if let Some(Value::Object(ref constructors)) = class_info.get("__constructors") {
                // Look for "init" or "new" constructor
                let constructor = constructors
                    .get("init")
                    .or_else(|| constructors.get("new"));

                if let Some(constructor) = constructor {
                    if let Value::Closure {
                        params,
                        body,
                        env: _,
                    } = constructor
                    {
                        // Check argument count
                        if args.len() != params.len() {
                            return Err(InterpreterError::RuntimeError(format!(
                                "constructor expects {} arguments, got {}",
                                params.len(),
                                args.len()
                            )));
                        }

                        // Create environment for constructor
                        let mut ctor_env = HashMap::new();

                        // Bind 'self' to the class instance
                        ctor_env.insert("self".to_string(), class_instance.clone());

                        // Bind constructor parameters
                        for (param, arg) in params.iter().zip(args) {
                            ctor_env.insert(param.clone(), arg.clone());
                        }

                        // Push constructor environment
                        self.env_stack.push(ctor_env);

                        // Execute constructor body
                        let _result = self.eval_expr(body)?;

                        // Pop environment
                        self.env_stack.pop();
                    }
                }
            }

            Ok(class_instance)
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

            Ok(Value::Object(Arc::new(instance)))
        } else {
            Err(InterpreterError::RuntimeError(format!(
                "{} is not a struct definition",
                struct_name
            )))
        }
    }

    /// Instantiates an actor with initial field values.
    ///
    /// This method creates a new actor instance, initializes fields with default or provided values,
    /// and stores the message handlers for later use.
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::frontend::parser::Parser;
    /// use ruchy::runtime::interpreter::Interpreter;
    ///
    /// let mut interpreter = Interpreter::new();
    /// let code = r#"
    ///     actor Counter {
    ///         count: i32 = 0
    ///
    ///         receive {
    ///             Increment => 42
    ///         }
    ///     }
    ///
    ///     fn main() {
    ///         let counter = spawn Counter
    ///         counter
    ///     }
    /// "#;
    ///
    /// let mut parser = Parser::new(code);
    /// let expr = parser.parse().unwrap();
    /// interpreter.eval_expr(&expr).unwrap();
    /// let main_call = Parser::new("main()").parse().unwrap();
    /// let result = interpreter.eval_expr(&main_call);
    /// assert!(result.is_ok());
    /// ```
    fn instantiate_actor_with_args(
        &mut self,
        actor_name: &str,
        args: &[Value],
    ) -> Result<Value, InterpreterError> {
        // Look up the actor definition
        let actor_def = self.lookup_variable(actor_name)?;

        if let Value::Object(ref actor_info) = actor_def {
            // Verify this is an actor
            if let Some(Value::String(ref type_str)) = actor_info.get("__type") {
                if type_str.as_ref() != "Actor" {
                    return Err(InterpreterError::RuntimeError(format!(
                        "{} is not an actor",
                        actor_name
                    )));
                }
            }

            // Create actor instance
            let mut instance = HashMap::new();
            instance.insert(
                "__actor".to_string(),
                Value::from_string(actor_name.to_string()),
            );

            // Check if args is a single object literal (named arguments)
            let named_args = if args.len() == 1 {
                if let Value::Object(ref obj) = args[0] {
                    Some(obj)
                } else {
                    None
                }
            } else {
                None
            };

            // Initialize state fields with default values
            // Actors use __fields just like structs
            if let Some(Value::Object(ref fields)) = actor_info.get("__fields") {
                if let Some(named) = named_args {
                    // Use named arguments
                    for (field_name, _field_info) in fields.iter() {
                        if let Some(value) = named.get(field_name) {
                            instance.insert(field_name.clone(), value.clone());
                        } else {
                            // Initialize with default for type
                            instance.insert(field_name.clone(), Value::Nil);
                        }
                    }
                } else {
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
                        } else {
                            // Simple field without metadata
                            instance.insert(field_name.clone(), Value::Nil);
                        }
                    }
                }
            }

            // Store the actor's handlers for later message processing
            if let Some(handlers) = actor_info.get("__handlers") {
                instance.insert("__handlers".to_string(), handlers.clone());
            }

            // For simple interpreted actors, don't use async runtime - just store state directly
            // This allows synchronous message processing which is simpler and works for tests
            // Return ObjectMut for mutable actor state
            Ok(crate::runtime::object_helpers::new_mutable_object(instance))
        } else {
            Err(InterpreterError::RuntimeError(format!(
                "{} is not an actor definition",
                actor_name
            )))
        }
    }

    fn eval_class_instance_method(
        &mut self,
        instance: &HashMap<String, Value>,
        class_name: &str,
        method: &str,
        arg_values: &[Value],
    ) -> Result<Value, InterpreterError> {
        // Look up the class definition
        let class_def = self.lookup_variable(class_name)?;

        if let Value::Object(ref class_info) = class_def {
            // Look for the method in the class definition
            if let Some(Value::Object(ref methods)) = class_info.get("__methods") {
                if let Some(Value::Object(ref method_meta)) = methods.get(method) {
                    // Get the method closure
                    if let Some(Value::Closure { params, body, .. }) = method_meta.get("closure") {
                        // Check if it's a static method
                        let is_static = method_meta
                            .get("is_static")
                            .and_then(|v| {
                                if let Value::Bool(b) = v {
                                    Some(*b)
                                } else {
                                    None
                                }
                            })
                            .unwrap_or(false);

                        if is_static {
                            return Err(InterpreterError::RuntimeError(format!(
                                "Cannot call static method {} on instance",
                                method
                            )));
                        }

                        // Create environment for method execution
                        let mut method_env = HashMap::new();

                        // Add 'self' to the environment
                        method_env.insert(
                            "self".to_string(),
                            Value::Object(Arc::new(instance.clone())),
                        );

                        // Bind method parameters to arguments
                        // Note: We're not including 'self' in params count here
                        if arg_values.len() != params.len() {
                            return Err(InterpreterError::RuntimeError(format!(
                                "Method {} expects {} arguments, got {}",
                                method,
                                params.len(),
                                arg_values.len()
                            )));
                        }

                        for (param, arg) in params.iter().zip(arg_values) {
                            method_env.insert(param.clone(), arg.clone());
                        }

                        // Push method environment
                        self.env_push(method_env);

                        // Execute method body
                        let result = self.eval_expr(body)?;

                        // Pop environment
                        self.env_pop();

                        return Ok(result);
                    }
                }
            }

            // Method not found
            Err(InterpreterError::RuntimeError(format!(
                "Class {} has no method named {}",
                class_name, method
            )))
        } else {
            Err(InterpreterError::RuntimeError(format!(
                "{} is not a class",
                class_name
            )))
        }
    }

    fn call_static_method(
        &mut self,
        class_name: &str,
        method_name: &str,
        args: &[Value],
    ) -> Result<Value, InterpreterError> {
        // Look up the class definition
        let class_def = self.lookup_variable(class_name)?;

        if let Value::Object(ref class_info) = class_def {
            // Look for the method in the class definition
            if let Some(Value::Object(ref methods)) = class_info.get("__methods") {
                if let Some(Value::Object(ref method_meta)) = methods.get(method_name) {
                    // Verify it's a static method
                    let is_static = method_meta
                        .get("is_static")
                        .and_then(|v| {
                            if let Value::Bool(b) = v {
                                Some(*b)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(false);

                    if !is_static {
                        return Err(InterpreterError::RuntimeError(format!(
                            "{} is not a static method",
                            method_name
                        )));
                    }

                    // Get the method closure
                    if let Some(Value::Closure { params, body, .. }) = method_meta.get("closure") {
                        // Check parameter count
                        if args.len() != params.len() {
                            return Err(InterpreterError::RuntimeError(format!(
                                "Static method {} expects {} arguments, got {}",
                                method_name,
                                params.len(),
                                args.len()
                            )));
                        }

                        // Create environment for static method execution
                        let mut method_env = HashMap::new();

                        // Bind parameters to arguments (no self for static methods)
                        for (i, param) in params.iter().enumerate() {
                            method_env.insert(param.clone(), args[i].clone());
                        }

                        // Push the method environment
                        self.env_stack.push(method_env);

                        // Execute the method body
                        let result = self.eval_expr(body);

                        // Pop the method environment
                        self.env_stack.pop();

                        return result;
                    }
                }
            }

            Err(InterpreterError::RuntimeError(format!(
                "Static method {} not found in class {}",
                method_name, class_name
            )))
        } else {
            Err(InterpreterError::RuntimeError(format!(
                "{} is not a class",
                class_name
            )))
        }
    }

    /// Evaluate `DataFrame` builder methods (.column, .build)
    /// Complexity: 8 (within Toyota Way limits)
    fn eval_dataframe_builder_method(
        &self,
        builder: &std::collections::HashMap<String, Value>,
        method: &str,
        arg_values: &[Value],
    ) -> Result<Value, InterpreterError> {
        match method {
            "column" => {
                // .column(name, values) - add a column to the builder
                if arg_values.len() != 2 {
                    return Err(InterpreterError::RuntimeError(
                        "DataFrame builder .column() requires 2 arguments (name, values)"
                            .to_string(),
                    ));
                }

                // Extract column name
                let name = match &arg_values[0] {
                    Value::String(s) => s.to_string(),
                    _ => {
                        return Err(InterpreterError::RuntimeError(
                            "Column name must be a string".to_string(),
                        ))
                    }
                };

                // Extract values array
                let values = match &arg_values[1] {
                    Value::Array(arr) => arr.to_vec(),
                    _ => {
                        return Err(InterpreterError::RuntimeError(
                            "Column values must be an array".to_string(),
                        ))
                    }
                };

                // Get current columns
                let current_columns = match builder.get("__columns") {
                    Some(Value::Array(cols)) => cols.to_vec(),
                    _ => vec![],
                };

                // Create new column object
                let mut col_obj = std::collections::HashMap::new();
                col_obj.insert("name".to_string(), Value::from_string(name));
                col_obj.insert("values".to_string(), Value::from_array(values));

                // Add to columns array
                let mut new_columns = current_columns;
                new_columns.push(Value::Object(std::sync::Arc::new(col_obj)));

                // Create new builder with updated columns
                let mut new_builder = builder.clone();
                new_builder.insert("__columns".to_string(), Value::from_array(new_columns));

                Ok(Value::Object(std::sync::Arc::new(new_builder)))
            }
            "build" => {
                // .build() - convert builder to `DataFrame`
                if !arg_values.is_empty() {
                    return Err(InterpreterError::RuntimeError(
                        "DataFrame builder .build() takes no arguments".to_string(),
                    ));
                }

                // Extract columns from builder
                let columns_array = match builder.get("__columns") {
                    Some(Value::Array(cols)) => cols,
                    _ => return Ok(Value::DataFrame { columns: vec![] }),
                };

                // Convert column objects to `DataFrameColumn` structs
                let mut df_columns = Vec::new();
                for col_val in columns_array.as_ref() {
                    if let Value::Object(col_obj) = col_val {
                        let name = match col_obj.get("name") {
                            Some(Value::String(s)) => s.to_string(),
                            _ => continue,
                        };
                        let values = match col_obj.get("values") {
                            Some(Value::Array(vals)) => vals.to_vec(),
                            _ => vec![],
                        };
                        df_columns.push(DataFrameColumn { name, values });
                    }
                }

                Ok(Value::DataFrame {
                    columns: df_columns,
                })
            }
            _ => Err(InterpreterError::RuntimeError(format!(
                "Unknown builder method: {method}"
            ))),
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
    /// Complexity: 8 (within Toyota Way limits)
    fn eval_dataframe_filter_method(
        &mut self,
        receiver: &Value,
        args: &[Expr],
    ) -> Result<Value, InterpreterError> {
        if args.len() != 1 {
            return Err(InterpreterError::RuntimeError(
                "DataFrame.filter() requires exactly 1 argument (closure)".to_string(),
            ));
        }

        if let Value::DataFrame { columns } = receiver {
            let closure = &args[0];

            // Validate closure structure
            if !matches!(closure.kind, ExprKind::Lambda { .. }) {
                return Err(InterpreterError::RuntimeError(
                    "DataFrame.filter() expects a lambda expression".to_string(),
                ));
            }

            // Build keep_mask by evaluating closure for each row
            let num_rows = columns.first().map_or(0, |c| c.values.len());
            let mut keep_mask = Vec::with_capacity(num_rows);

            for row_idx in 0..num_rows {
                // Create row object with all column values for this row
                let mut row = HashMap::new();
                for col in columns {
                    if let Some(value) = col.values.get(row_idx) {
                        row.insert(col.name.clone(), value.clone());
                    }
                }
                let row_value = Value::Object(std::sync::Arc::new(row));

                // Evaluate closure with row object
                let result = self.eval_closure_with_value(closure, &row_value)?;

                // Check if result is boolean
                let keep = match result {
                    Value::Bool(b) => b,
                    _ => {
                        return Err(InterpreterError::RuntimeError(
                            "DataFrame.filter() closure must return boolean".to_string(),
                        ))
                    }
                };

                keep_mask.push(keep);
            }

            // Create new DataFrame with filtered rows
            let mut new_columns = Vec::new();
            for col in columns {
                let mut filtered_values = Vec::new();
                for (idx, &keep) in keep_mask.iter().enumerate() {
                    if keep {
                        if let Some(val) = col.values.get(idx) {
                            filtered_values.push(val.clone());
                        }
                    }
                }
                new_columns.push(DataFrameColumn {
                    name: col.name.clone(),
                    values: filtered_values,
                });
            }

            Ok(Value::DataFrame {
                columns: new_columns,
            })
        } else {
            Err(InterpreterError::RuntimeError(
                "filter method can only be called on DataFrame".to_string(),
            ))
        }
    }

    /// Special handler for `DataFrame` `with_column` method
    /// Complexity: 9 (within Toyota Way limits)
    fn eval_dataframe_with_column_method(
        &mut self,
        receiver: &Value,
        args: &[Expr],
    ) -> Result<Value, InterpreterError> {
        if args.len() != 2 {
            return Err(InterpreterError::RuntimeError(
                "DataFrame.with_column() requires exactly 2 arguments (name, closure)".to_string(),
            ));
        }

        // Evaluate the column name
        let col_name = match self.eval_expr(&args[0])? {
            Value::String(s) => s.to_string(),
            _ => {
                return Err(InterpreterError::RuntimeError(
                    "DataFrame.with_column() expects string column name".to_string(),
                ))
            }
        };

        if let Value::DataFrame { columns } = receiver {
            let closure = &args[1];

            // Extract parameter name from closure
            let param_name = if let ExprKind::Lambda { params, .. } = &closure.kind {
                if params.len() != 1 {
                    return Err(InterpreterError::RuntimeError(
                        "with_column closure must have exactly 1 parameter".to_string(),
                    ));
                }
                match &params[0].pattern {
                    crate::frontend::ast::Pattern::Identifier(name) => name.clone(),
                    _ => {
                        return Err(InterpreterError::RuntimeError(
                            "with_column closure must have simple identifier parameter".to_string(),
                        ))
                    }
                }
            } else {
                return Err(InterpreterError::RuntimeError(
                    "Expected lambda expression".to_string(),
                ));
            };

            // Check if parameter name matches a column
            let matching_col = columns.iter().find(|c| c.name == param_name);

            let mut new_values = Vec::new();
            let num_rows = columns.first().map_or(0, |c| c.values.len());

            for row_idx in 0..num_rows {
                let value_to_bind = if let Some(col) = matching_col {
                    // Parameter name matches a column - bind that column's value
                    col.values.get(row_idx).cloned().unwrap_or(Value::Nil)
                } else {
                    // Parameter name doesn't match - bind full row object
                    let mut row = HashMap::new();
                    for col in columns {
                        if let Some(value) = col.values.get(row_idx) {
                            row.insert(col.name.clone(), value.clone());
                        }
                    }
                    Value::Object(std::sync::Arc::new(row))
                };

                // Evaluate closure with the appropriate value
                let result = self.eval_closure_with_value(closure, &value_to_bind)?;
                new_values.push(result);
            }

            // Create new DataFrame with additional column
            let mut new_columns = columns.clone();
            new_columns.push(crate::runtime::DataFrameColumn {
                name: col_name,
                values: new_values,
            });

            Ok(Value::DataFrame {
                columns: new_columns,
            })
        } else {
            Err(InterpreterError::RuntimeError(
                "with_column method can only be called on DataFrame".to_string(),
            ))
        }
    }

    /// Special handler for `DataFrame` transform method
    /// Complexity: 9 (within Toyota Way limits)
    fn eval_dataframe_transform_method(
        &mut self,
        receiver: &Value,
        args: &[Expr],
    ) -> Result<Value, InterpreterError> {
        if args.len() != 2 {
            return Err(InterpreterError::RuntimeError(
                "DataFrame.transform() requires exactly 2 arguments (column, closure)".to_string(),
            ));
        }

        // Evaluate the column name
        let col_name = match self.eval_expr(&args[0])? {
            Value::String(s) => s.to_string(),
            _ => {
                return Err(InterpreterError::RuntimeError(
                    "DataFrame.transform() expects string column name".to_string(),
                ))
            }
        };

        if let Value::DataFrame { columns } = receiver {
            // Find the column to transform
            let col_idx = columns
                .iter()
                .position(|c| c.name == col_name)
                .ok_or_else(|| {
                    InterpreterError::RuntimeError(format!(
                        "Column '{col_name}' not found in DataFrame"
                    ))
                })?;

            let closure = &args[1];
            let mut new_columns = columns.clone();

            // Transform each value in the column
            let mut transformed_values = Vec::new();
            for value in &columns[col_idx].values {
                // Create a temporary environment with the value bound to the parameter
                let result = self.eval_closure_with_value(closure, value)?;
                transformed_values.push(result);
            }

            new_columns[col_idx].values = transformed_values;

            Ok(Value::DataFrame {
                columns: new_columns,
            })
        } else {
            Err(InterpreterError::RuntimeError(
                "transform method can only be called on DataFrame".to_string(),
            ))
        }
    }

    /// Evaluate a closure with a single value argument
    /// Complexity: 7 (within Toyota Way limits)
    fn eval_closure_with_value(
        &mut self,
        closure_expr: &Expr,
        value: &Value,
    ) -> Result<Value, InterpreterError> {
        match &closure_expr.kind {
            ExprKind::Lambda { params, body, .. } => {
                if params.len() != 1 {
                    return Err(InterpreterError::RuntimeError(
                        "Transform closure must have exactly 1 parameter".to_string(),
                    ));
                }

                // Extract parameter name from pattern
                let param_name = match &params[0].pattern {
                    crate::frontend::ast::Pattern::Identifier(name) => name.clone(),
                    _ => {
                        return Err(InterpreterError::RuntimeError(
                            "Transform closure must have simple identifier parameter".to_string(),
                        ))
                    }
                };

                // Create new environment with parameter binding
                let mut new_env = HashMap::new();
                new_env.insert(param_name, value.clone());

                // Push environment
                self.env_push(new_env);

                // Evaluate the body
                let result = self.eval_expr(body)?;

                // Pop environment
                self.env_pop();

                Ok(result)
            }
            _ => Err(InterpreterError::RuntimeError(
                "Expected lambda expression".to_string(),
            )),
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
        use crate::runtime::eval_string_interpolation::format_value_for_interpolation;

        let mut result = String::new();
        for part in parts {
            match part {
                StringPart::Text(text) => result.push_str(text),
                StringPart::Expr(expr) => {
                    let value = self.eval_expr(expr)?;
                    // Use format_value_for_interpolation to avoid adding quotes to strings
                    result.push_str(&format_value_for_interpolation(&value));
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
            body: Arc::new(body.clone()),
            env: Arc::new(self.current_env().clone()),
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
        // Handle static method calls: Type::method(args)
        // Parser represents these as Call { func: FieldAccess { object: Identifier("Type"), field: "method" }, args }
        if let ExprKind::FieldAccess { object, field } = &func.kind {
            if let ExprKind::Identifier(type_name) = &object.kind {
                // Detect Box::new(value) static method
                if type_name == "Box" && field == "new" {
                    if args.len() != 1 {
                        return Err(InterpreterError::RuntimeError(format!(
                            "Box::new() requires exactly 1 argument, got {}",
                            args.len()
                        )));
                    }
                    // Box::new(value) → just return the value (Box is transparent in Ruchy)
                    return self.eval_expr(&args[0]);
                }
                // Detect Vec::new() static method
                if type_name == "Vec" && field == "new" {
                    if !args.is_empty() {
                        return Err(InterpreterError::RuntimeError(format!(
                            "Vec::new() takes no arguments, got {}",
                            args.len()
                        )));
                    }
                    // Vec::new() → empty array
                    return Ok(Value::Array(Arc::from([])));
                }

                // REGRESSION-077: Check for user-defined struct impl methods
                // impl methods are stored with qualified names like "Logger::new_with_options"
                let qualified_method = format!("{}::{}", type_name, field);
                if let Ok(method_value) = self.lookup_variable(&qualified_method) {
                    // Found impl method - evaluate args and call it
                    let arg_vals: Result<Vec<Value>, InterpreterError> =
                        args.iter().map(|arg| self.eval_expr(arg)).collect();
                    let arg_vals = arg_vals?;
                    return self.call_function(method_value, &arg_vals);
                }
            }
        }

        // Try to evaluate the function normally
        let func_val_result = self.eval_expr(func);

        // If function lookup fails and it's an identifier, treat it as a message constructor
        let func_val = match func_val_result {
            Ok(val) => val,
            Err(InterpreterError::RuntimeError(msg)) if msg.starts_with("Undefined variable:") => {
                // Check if this is an identifier that could be a message constructor
                if let ExprKind::Identifier(name) = &func.kind {
                    // Create a message object
                    let arg_vals: Result<Vec<Value>, InterpreterError> =
                        args.iter().map(|arg| self.eval_expr(arg)).collect();
                    let arg_vals = arg_vals?;

                    let mut message = HashMap::new();
                    message.insert(
                        "__type".to_string(),
                        Value::from_string("Message".to_string()),
                    );
                    message.insert("type".to_string(), Value::from_string(name.clone()));
                    message.insert("data".to_string(), Value::Array(Arc::from(arg_vals)));

                    return Ok(Value::Object(Arc::new(message)));
                }
                return Err(InterpreterError::RuntimeError(msg));
            }
            Err(e) => return Err(e),
        };

        let arg_vals: Result<Vec<Value>, InterpreterError> =
            args.iter().map(|arg| self.eval_expr(arg)).collect();
        let arg_vals = arg_vals?;

        // Special handling for enum variant construction with arguments (tuple variants)
        if let Value::EnumVariant { variant_name, data: _ } = func_val {
            // This is a tuple variant constructor: Response::Error("msg")
            return Ok(Value::EnumVariant {
                variant_name,
                data: Some(arg_vals),
            });
        }

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
        // ISSUE-040 FIX: Use env_set_mut to search parent scopes for existing variables
        self.env_set_mut(name.to_string(), value);
    }

    /// Get a variable from the environment stack
    ///
    /// Searches the environment stack from innermost to outermost scope.
    /// Returns None if the variable is not found.
    pub fn get_variable(&self, name: &str) -> Option<Value> {
        // Search from innermost to outermost scope
        for env in self.env_stack.iter().rev() {
            if let Some(value) = env.get(name) {
                return Some(value.clone());
            }
        }
        None
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
            (Literal::Integer(a, _), Value::Integer(b)) => a == b,
            (Literal::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
            (Literal::String(a), Value::String(b)) => a == b.as_ref(),
            (Literal::Bool(a), Value::Bool(b)) => a == b,
            _ => false,
        }
    }

    // ========================================================================
    // EXTREME TDD: stdout Capture for WASM/REPL
    // Bug: https://github.com/paiml/ruchy/issues/PRINTLN_STDOUT
    // ========================================================================

    /// Capture println output to stdout buffer
    /// Complexity: 1 (single operation)
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::interpreter::Interpreter;
    ///
    /// let mut interpreter = Interpreter::new();
    /// interpreter.capture_stdout("Hello, World!".to_string());
    /// assert_eq!(interpreter.get_stdout(), "Hello, World!");
    /// ```
    pub fn capture_stdout(&mut self, output: String) {
        self.stdout_buffer.push(output);
    }

    /// Get captured stdout as a single string with newlines
    /// Complexity: 2 (join + conditional)
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::interpreter::Interpreter;
    ///
    /// let mut interpreter = Interpreter::new();
    /// interpreter.capture_stdout("Line 1".to_string());
    /// interpreter.capture_stdout("Line 2".to_string());
    /// assert_eq!(interpreter.get_stdout(), "Line 1\nLine 2");
    /// ```
    pub fn get_stdout(&self) -> String {
        self.stdout_buffer.join("\n")
    }

    /// Clear stdout buffer
    /// Complexity: 1 (single operation)
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::interpreter::Interpreter;
    ///
    /// let mut interpreter = Interpreter::new();
    /// interpreter.capture_stdout("test".to_string());
    /// interpreter.clear_stdout();
    /// assert_eq!(interpreter.get_stdout(), "");
    /// ```
    pub fn clear_stdout(&mut self) {
        self.stdout_buffer.clear();
    }

    /// Check if stdout has any captured output
    /// Complexity: 1 (single check)
    pub fn has_stdout(&self) -> bool {
        !self.stdout_buffer.is_empty()
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

#[cfg(test)]
mod lambda_tests {
    use super::*;

    #[test]
    fn test_lambda_variable_assignment_and_call() {
        let code = r"
            let double = x => x * 2
            double(5)
        ";
        let mut parser = crate::frontend::parser::Parser::new(code);
        let ast = parser.parse().expect("Parse failed");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_expr(&ast).expect("Eval failed");
        assert_eq!(result, Value::Integer(10));
    }

    #[test]
    fn test_lambda_pipe_syntax_variable_call() {
        let code = r"
            let triple = |x| x * 3
            triple(4)
        ";
        let mut parser = crate::frontend::parser::Parser::new(code);
        let ast = parser.parse().expect("Parse failed");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_expr(&ast).expect("Eval failed");
        assert_eq!(result, Value::Integer(12));
    }
}

#[cfg(test)]
mod negative_indexing_tests {
    use super::*;

    // FEATURE-042 (GitHub Issue #46): Negative indexing tests

    #[test]
    fn test_negative_array_indexing_last_element() {
        let code = r#"
            let fruits = ["apple", "banana", "cherry"]
            fruits[-1]
        "#;
        let mut parser = crate::frontend::parser::Parser::new(code);
        let ast = parser.parse().expect("Parse failed");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_expr(&ast).expect("Eval failed");
        assert_eq!(result, Value::from_string("cherry".to_string()));
    }

    #[test]
    fn test_negative_array_indexing_second_to_last() {
        let code = r#"
            let fruits = ["apple", "banana", "cherry"]
            fruits[-2]
        "#;
        let mut parser = crate::frontend::parser::Parser::new(code);
        let ast = parser.parse().expect("Parse failed");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_expr(&ast).expect("Eval failed");
        assert_eq!(result, Value::from_string("banana".to_string()));
    }

    #[test]
    fn test_negative_array_indexing_first_element() {
        let code = r#"
            let fruits = ["apple", "banana", "cherry"]
            fruits[-3]
        "#;
        let mut parser = crate::frontend::parser::Parser::new(code);
        let ast = parser.parse().expect("Parse failed");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_expr(&ast).expect("Eval failed");
        assert_eq!(result, Value::from_string("apple".to_string()));
    }

    #[test]
    fn test_negative_array_indexing_out_of_bounds() {
        let code = r#"
            let fruits = ["apple", "banana"]
            fruits[-5]
        "#;
        let mut parser = crate::frontend::parser::Parser::new(code);
        let ast = parser.parse().expect("Parse failed");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_expr(&ast);
        assert!(result.is_err(), "Should fail for out-of-bounds negative index");
    }

    #[test]
    fn test_negative_string_indexing() {
        let code = r#"
            let word = "hello"
            word[-1]
        "#;
        let mut parser = crate::frontend::parser::Parser::new(code);
        let ast = parser.parse().expect("Parse failed");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_expr(&ast).expect("Eval failed");
        assert_eq!(result, Value::from_string("o".to_string()));
    }

    #[test]
    fn test_negative_tuple_indexing() {
        let code = r#"
            let point = (10, 20, 30)
            point[-1]
        "#;
        let mut parser = crate::frontend::parser::Parser::new(code);
        let ast = parser.parse().expect("Parse failed");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_expr(&ast).expect("Eval failed");
        assert_eq!(result, Value::Integer(30));
    }

    #[test]
    fn test_negative_indexing_with_integers() {
        let code = r#"
            let numbers = [100, 200, 300, 400]
            numbers[-2]
        "#;
        let mut parser = crate::frontend::parser::Parser::new(code);
        let ast = parser.parse().expect("Parse failed");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_expr(&ast).expect("Eval failed");
        assert_eq!(result, Value::Integer(300));
    }
}

// Tests removed - moved to separate test files
