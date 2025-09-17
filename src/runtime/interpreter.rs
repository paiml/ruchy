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

use crate::frontend::ast::{BinaryOp as AstBinaryOp, Expr, ExprKind, Literal, StringPart, Pattern, MatchArm};
use crate::frontend::Param;
use smallvec::{smallvec, SmallVec};
use std::collections::HashMap;
use std::rc::Rc;

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
    String(Rc<String>),
    /// Array of values
    Array(Rc<Vec<Value>>),
    /// Tuple of values
    Tuple(Rc<Vec<Value>>),
    /// Function closure
    Closure {
        params: Vec<String>,
        body: Rc<Expr>,
        env: Rc<HashMap<String, Value>>, // Captured environment
    },
    /// `DataFrame` value
    DataFrame {
        columns: Vec<DataFrameColumn>,
    },
}

impl Value {
    /// Create an integer value from an `i64`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::interpreter::Value;
    ///
    /// let val = Value::from_i64(42);
    /// assert_eq!(val.as_i64().unwrap(), 42);
    /// ```
    pub fn from_i64(i: i64) -> Self {
        Value::Integer(i)
    }

    /// Create a float value from an `f64`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::interpreter::Value;
    ///
    /// let val = Value::from_f64(3.14);
    /// assert_eq!(val.as_f64().unwrap(), 3.14);
    /// ```
    pub fn from_f64(f: f64) -> Self {
        Value::Float(f)
    }

    /// Create boolean value
    pub fn from_bool(b: bool) -> Self {
        Value::Bool(b)
    }

    /// Create nil value
    pub fn nil() -> Self {
        Value::Nil
    }

    /// Create string value
    pub fn from_string(s: String) -> Self {
        Value::String(Rc::new(s))
    }

    /// Create array value
    pub fn from_array(arr: Vec<Value>) -> Self {
        Value::Array(Rc::new(arr))
    }

    /// Check if value is nil
    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    /// Check if value is truthy.
    ///
    /// In Ruchy, only `false` and `nil` are falsy. All other values,
    /// including `0` and empty strings, are truthy.
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::interpreter::Value;
    ///
    /// assert!(Value::from_i64(0).is_truthy());
    /// assert!(Value::from_string("".to_string()).is_truthy());
    /// assert!(!Value::Bool(false).is_truthy());
    /// assert!(!Value::Nil.is_truthy());
    /// ```
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Nil => false,
            _ => true,
        }
    }

    /// Extract integer value
    /// # Errors
    /// Returns error if the value is not an integer
    pub fn as_i64(&self) -> Result<i64, InterpreterError> {
        match self {
            Value::Integer(i) => Ok(*i),
            _ => Err(InterpreterError::TypeError(format!(
                "Expected integer, got {}",
                self.type_name()
            ))),
        }
    }

    /// Extract float value  
    /// # Errors
    /// Returns error if the value is not a float
    pub fn as_f64(&self) -> Result<f64, InterpreterError> {
        match self {
            Value::Float(f) => Ok(*f),
            _ => Err(InterpreterError::TypeError(format!(
                "Expected float, got {}",
                self.type_name()
            ))),
        }
    }

    /// Extract boolean value
    /// # Errors
    /// Returns error if the value is not a boolean
    pub fn as_bool(&self) -> Result<bool, InterpreterError> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => Err(InterpreterError::TypeError(format!(
                "Expected boolean, got {}",
                self.type_name()
            ))),
        }
    }

    /// Get type name for debugging
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::Bool(_) => "boolean",
            Value::Nil => "nil",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Tuple(_) => "tuple",
            Value::Closure { .. } => "function",
            Value::DataFrame { .. } => "dataframe",
        }
    }
}

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
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{i}"),
            Value::Float(fl) => write!(f, "{fl}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Nil => write!(f, "nil"),
            Value::String(s) => write!(f, "{s}"),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{val}")?;
                }
                write!(f, "]")
            }
            Value::Tuple(elements) => {
                write!(f, "(")?;
                for (i, val) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{val}")?;
                }
                write!(f, ")")
            }
            Value::Closure { .. } => write!(f, "<function>"),
            Value::DataFrame { columns } => {
                writeln!(f, "DataFrame with {} columns:", columns.len())?;
                for col in columns {
                    writeln!(f, "  {}: {} rows", col.name, col.values.len())?;
                }
                Ok(())
            }
        }
    }
}

impl std::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpreterError::TypeError(msg) => write!(f, "Type error: {msg}"),
            InterpreterError::RuntimeError(msg) => write!(f, "Runtime error: {msg}"),
            InterpreterError::StackOverflow => write!(f, "Stack overflow"),
            InterpreterError::StackUnderflow => write!(f, "Stack underflow"),
            InterpreterError::InvalidInstruction => write!(f, "Invalid instruction"),
            InterpreterError::DivisionByZero => write!(f, "Division by zero"),
            InterpreterError::IndexOutOfBounds => write!(f, "Index out of bounds"),
        }
    }
}

impl std::error::Error for InterpreterError {}

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

/// Conservative garbage collector for heap-allocated objects.
///
/// This GC implementation uses conservative stack scanning and mark-and-sweep
/// collection. It operates alongside Rust's `Rc` reference counting for a
/// hybrid memory management approach.
///
/// # Features
///
/// - Conservative stack scanning (treats all stack values as potential pointers)
/// - Mark-and-sweep collection algorithm
/// - Automatic collection based on memory pressure
/// - Collection statistics for performance monitoring
///
/// # Future Enhancements
///
/// - Generational collection for better performance
/// - Precise stack maps for accurate root identification
/// - Incremental collection to reduce pause times
#[derive(Debug)]
pub struct ConservativeGC {
    /// Objects currently tracked by the GC
    tracked_objects: Vec<GCObject>,
    /// Collection statistics
    collections_performed: u64,
    /// Total objects collected
    objects_collected: u64,
    /// Memory pressure threshold (bytes)
    collection_threshold: usize,
    /// Current allocated bytes estimate
    allocated_bytes: usize,
    /// Enable/disable automatic collection
    auto_collect_enabled: bool,
}

/// A garbage-collected object with metadata
#[derive(Debug, Clone)]
pub struct GCObject {
    /// Object identifier (address-like)
    id: usize,
    /// Object size in bytes
    size: usize,
    /// Mark bit for mark-and-sweep
    marked: bool,
    /// Object generation (for future generational GC)
    #[allow(dead_code)] // Will be used in future generational GC implementation
    generation: u8,
    /// Reference to the actual value
    value: Value,
}

impl ConservativeGC {
    /// Create new conservative garbage collector
    pub fn new() -> Self {
        Self {
            tracked_objects: Vec::new(),
            collections_performed: 0,
            objects_collected: 0,
            collection_threshold: 1024 * 1024, // 1MB default threshold
            allocated_bytes: 0,
            auto_collect_enabled: true,
        }
    }

    /// Add an object to GC tracking
    pub fn track_object(&mut self, value: Value) -> usize {
        let id = self.tracked_objects.len();
        let size = self.estimate_object_size(&value);

        let gc_object = GCObject {
            id,
            size,
            marked: false,
            generation: 0,
            value,
        };

        self.tracked_objects.push(gc_object);
        self.allocated_bytes += size;

        // Trigger collection if we've exceeded threshold
        if self.auto_collect_enabled && self.allocated_bytes > self.collection_threshold {
            self.collect_garbage();
        }

        id
    }

    /// Perform garbage collection using conservative stack scanning
    pub fn collect_garbage(&mut self) -> GCStats {
        let initial_count = self.tracked_objects.len();
        let initial_bytes = self.allocated_bytes;

        // Mark phase: mark all reachable objects
        self.mark_phase();

        // Sweep phase: collect unmarked objects
        let collected = self.sweep_phase();

        self.collections_performed += 1;
        self.objects_collected += collected as u64;

        GCStats {
            objects_before: initial_count,
            objects_after: self.tracked_objects.len(),
            objects_collected: collected,
            bytes_before: initial_bytes,
            bytes_after: self.allocated_bytes,
            collection_time_ns: 0, // Simple implementation doesn't time
        }
    }

    /// Mark phase: mark all reachable objects
    fn mark_phase(&mut self) {
        // Reset all marks
        for obj in &mut self.tracked_objects {
            obj.marked = false;
        }

        // Mark objects based on Value references
        // In a more sophisticated implementation, this would scan the stack
        // For now, we conservatively mark all objects referenced by other tracked objects
        for i in 0..self.tracked_objects.len() {
            if self.is_root_object(i) {
                self.mark_object(i);
            }
        }
    }

    /// Check if object is a root (conservatively assume all are roots for safety)
    fn is_root_object(&self, _index: usize) -> bool {
        // Conservative implementation: treat all objects as potentially reachable
        // In a real implementation, this would scan the stack and globals
        true
    }

    /// Mark an object and all objects it references
    fn mark_object(&mut self, index: usize) {
        if index >= self.tracked_objects.len() || self.tracked_objects[index].marked {
            return;
        }

        self.tracked_objects[index].marked = true;

        // Mark objects referenced by this object
        let value = &self.tracked_objects[index].value.clone();
        if let Value::Array(arr) = value {
            // Mark all array elements that are tracked objects
            for elem in arr.iter() {
                if let Some(referenced_id) = self.find_object_id(elem) {
                    self.mark_object(referenced_id);
                }
            }
        }
        // Note: Closure environments and other value types don't contain tracked object references
        // In a real implementation, would mark closure environment
    }

    /// Find the GC object ID for a given value
    fn find_object_id(&self, target: &Value) -> Option<usize> {
        // Simple linear search - in production would use hash table
        for (id, obj) in self.tracked_objects.iter().enumerate() {
            if std::ptr::eq(&raw const obj.value, target) {
                return Some(id);
            }
        }
        None
    }

    /// Sweep phase: collect unmarked objects
    fn sweep_phase(&mut self) -> usize {
        let initial_len = self.tracked_objects.len();

        // Keep only marked objects
        self.tracked_objects.retain(|obj| {
            if obj.marked {
                true
            } else {
                self.allocated_bytes = self.allocated_bytes.saturating_sub(obj.size);
                false
            }
        });

        // Reassign IDs after compaction
        for (new_id, obj) in self.tracked_objects.iter_mut().enumerate() {
            obj.id = new_id;
        }

        initial_len - self.tracked_objects.len()
    }

    /// Estimate memory size of a value
    fn estimate_object_size(&self, value: &Value) -> usize {
        match value {
            Value::Integer(_) | Value::Float(_) => 8,
            Value::Bool(_) => 1,
            Value::Nil => 0,
            Value::String(s) => s.len() + 24, // String overhead + content
            Value::Array(arr) => {
                let base_size = 24; // Vec overhead
                let element_size = arr
                    .iter()
                    .map(|v| self.estimate_object_size(v))
                    .sum::<usize>();
                base_size + element_size
            }
            Value::Tuple(elements) => {
                let base_size = 24; // Vec overhead
                let element_size = elements
                    .iter()
                    .map(|v| self.estimate_object_size(v))
                    .sum::<usize>();
                base_size + element_size
            }
            Value::Closure { params, .. } => {
                let base_size = 48; // Closure overhead
                let params_size = params.iter().map(std::string::String::len).sum::<usize>();
                base_size + params_size
            }
            Value::DataFrame { columns } => {
                let base_size = 24; // DataFrame overhead
                let columns_size = columns.iter().map(|col| {
                    col.name.len() + col.values.iter().map(|v| self.estimate_object_size(v)).sum::<usize>()
                }).sum::<usize>();
                base_size + columns_size
            }
        }
    }

    /// Get current GC statistics
    pub fn get_stats(&self) -> GCStats {
        GCStats {
            objects_before: self.tracked_objects.len(),
            objects_after: self.tracked_objects.len(),
            objects_collected: 0,
            bytes_before: self.allocated_bytes,
            bytes_after: self.allocated_bytes,
            collection_time_ns: 0,
        }
    }

    /// Get detailed GC information
    pub fn get_info(&self) -> GCInfo {
        GCInfo {
            total_objects: self.tracked_objects.len(),
            allocated_bytes: self.allocated_bytes,
            collections_performed: self.collections_performed,
            objects_collected: self.objects_collected,
            collection_threshold: self.collection_threshold,
            auto_collect_enabled: self.auto_collect_enabled,
        }
    }

    /// Set collection threshold
    pub fn set_collection_threshold(&mut self, threshold: usize) {
        self.collection_threshold = threshold;
    }

    /// Enable or disable automatic collection
    pub fn set_auto_collect(&mut self, enabled: bool) {
        self.auto_collect_enabled = enabled;
    }

    /// Force garbage collection
    pub fn force_collect(&mut self) -> GCStats {
        self.collect_garbage()
    }

    /// Clear all tracked objects (for testing)
    pub fn clear(&mut self) {
        self.tracked_objects.clear();
        self.allocated_bytes = 0;
    }
}

impl Default for ConservativeGC {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics from a garbage collection cycle
#[derive(Debug, Clone, PartialEq)]
pub struct GCStats {
    /// Objects before collection
    pub objects_before: usize,
    /// Objects after collection  
    pub objects_after: usize,
    /// Objects collected
    pub objects_collected: usize,
    /// Bytes before collection
    pub bytes_before: usize,
    /// Bytes after collection
    pub bytes_after: usize,
    /// Collection time in nanoseconds
    pub collection_time_ns: u64,
}

/// General GC information
#[derive(Debug, Clone)]
pub struct GCInfo {
    /// Total objects currently tracked
    pub total_objects: usize,
    /// Currently allocated bytes
    pub allocated_bytes: usize,
    /// Total collections performed
    pub collections_performed: u64,
    /// Total objects collected ever
    pub objects_collected: u64,
    /// Collection threshold in bytes
    pub collection_threshold: usize,
    /// Whether auto-collection is enabled
    pub auto_collect_enabled: bool,
}

/// Direct-threaded instruction dispatch system for optimal performance
/// Replaces AST walking with linear instruction stream and function pointers
#[derive(Debug)]
pub struct DirectThreadedInterpreter {
    /// Linear instruction stream with embedded operands
    code: Vec<ThreadedInstruction>,
    /// Constant pool separated from instruction stream for I-cache efficiency
    constants: Vec<Value>,
    /// Program counter
    pc: usize,
    /// Runtime state for instruction execution
    state: InterpreterState,
}

/// Single threaded instruction with direct function pointer dispatch
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ThreadedInstruction {
    /// Direct pointer to handler function - eliminates switch overhead
    handler: fn(&mut InterpreterState, u32) -> InstructionResult,
    /// Inline operand (constant index, local slot, jump target, etc.)
    operand: u32,
}

/// Runtime state for direct-threaded execution
#[derive(Debug)]
pub struct InterpreterState {
    /// Value stack for operands
    stack: Vec<Value>,
    /// Environment stack for variable lookups
    env_stack: Vec<HashMap<String, Value>>,
    /// Constants pool reference
    constants: Vec<Value>,
    /// Inline caches for method dispatch
    #[allow(dead_code)] // Will be used in future phases
    caches: Vec<InlineCache>,
}

impl InterpreterState {
    /// Create new interpreter state
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            env_stack: vec![HashMap::new()], // Start with global environment
            constants: Vec::new(),
            caches: Vec::new(),
        }
    }
}

impl Default for InterpreterState {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of executing a single threaded instruction
#[derive(Debug, Clone, PartialEq)]
pub enum InstructionResult {
    /// Continue to next instruction
    Continue,
    /// Jump to target PC
    Jump(usize),
    /// Return value from function/expression
    Return(Value),
    /// Runtime error occurred
    Error(InterpreterError),
}

impl DirectThreadedInterpreter {
    /// Create new direct-threaded interpreter
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            pc: 0,
            state: InterpreterState {
                stack: Vec::with_capacity(256),
                env_stack: vec![HashMap::new()],
                constants: Vec::new(),
                caches: Vec::new(),
            },
        }
    }

    /// Compile AST expression to threaded instruction stream
    ///
    /// # Errors
    ///
    /// Returns an error if the expression contains unsupported constructs
    /// or if instruction compilation fails.
    pub fn compile(&mut self, expr: &Expr) -> Result<(), InterpreterError> {
        self.code.clear();
        self.constants.clear();
        self.pc = 0;

        // Compile expression to instruction stream
        self.compile_expr(expr)?;

        // Add return instruction if needed
        if self.code.is_empty()
            || !matches!(self.code.last(), Some(instr) if
            std::ptr::eq(instr.handler as *const (), op_return as *const ()))
        {
            self.emit_instruction(op_return, 0);
        }

        // Copy constants to state
        self.state.constants = self.constants.clone();

        Ok(())
    }

    /// Execute compiled instruction stream using direct-threaded dispatch
    ///
    /// # Errors
    ///
    /// Returns an error if execution encounters runtime errors such as
    /// stack overflow, division by zero, or undefined variables.
    pub fn execute(&mut self) -> Result<Value, InterpreterError> {
        self.pc = 0;

        loop {
            // Bounds check
            if self.pc >= self.code.len() {
                return Err(InterpreterError::RuntimeError(
                    "PC out of bounds".to_string(),
                ));
            }

            // Direct function pointer call - no switch overhead
            let instruction = &self.code[self.pc];
            let result = (instruction.handler)(&mut self.state, instruction.operand);

            match result {
                InstructionResult::Continue => {
                    self.pc += 1;
                }
                InstructionResult::Jump(target) => {
                    if target >= self.code.len() {
                        return Err(InterpreterError::RuntimeError(
                            "Jump target out of bounds".to_string(),
                        ));
                    }
                    self.pc = target;
                }
                InstructionResult::Return(value) => {
                    return Ok(value);
                }
                InstructionResult::Error(error) => {
                    return Err(error);
                }
            }

            // Periodic interrupt check for long-running loops
            if self.pc.trailing_zeros() >= 10 {
                // Could add interrupt checking here in the future
            }
        }
    }

    /// Compile single expression to instruction stream
    fn compile_expr(&mut self, expr: &Expr) -> Result<(), InterpreterError> {
        match &expr.kind {
            ExprKind::Literal(lit) => self.compile_literal(lit),
            ExprKind::Binary { left, op, right } => self.compile_binary_expr(left, op, right),
            ExprKind::Identifier(name) => self.compile_identifier(name),
            ExprKind::If { condition, then_branch, else_branch } => 
                self.compile_if_expr(condition, then_branch, else_branch.as_deref()),
            _ => self.compile_fallback_expr(),
        }
    }
    
    // Helper methods for DirectThreadedInterpreter compilation (complexity <10 each)
    
    fn compile_literal(&mut self, lit: &Literal) -> Result<(), InterpreterError> {
        if matches!(lit, Literal::Unit) {
            self.emit_instruction(op_load_nil, 0);
        } else {
            let const_idx = self.add_constant(self.literal_to_value(lit));
            self.emit_instruction(op_load_const, const_idx);
        }
        Ok(())
    }
    
    fn compile_binary_expr(&mut self, left: &Expr, op: &crate::frontend::ast::BinaryOp, right: &Expr) -> Result<(), InterpreterError> {
        self.compile_expr(left)?;
        self.compile_expr(right)?;
        
        let op_code = self.binary_op_to_opcode(op)?;
        self.emit_instruction(op_code, 0);
        Ok(())
    }
    
    fn binary_op_to_opcode(&self, op: &crate::frontend::ast::BinaryOp) -> Result<fn(&mut InterpreterState, u32) -> InstructionResult, InterpreterError> {
        match op {
            crate::frontend::ast::BinaryOp::Add => Ok(op_add),
            crate::frontend::ast::BinaryOp::Subtract => Ok(op_sub),
            crate::frontend::ast::BinaryOp::Multiply => Ok(op_mul),
            crate::frontend::ast::BinaryOp::Divide => Ok(op_div),
            _ => Err(InterpreterError::RuntimeError(format!(
                "Unsupported binary operation: {:?}",
                op
            ))),
        }
    }
    
    fn compile_identifier(&mut self, name: &str) -> Result<(), InterpreterError> {
        let name_idx = self.add_constant(Value::String(Rc::new(name.to_string())));
        self.emit_instruction(op_load_var, name_idx);
        Ok(())
    }
    
    fn compile_if_expr(&mut self, condition: &Expr, then_branch: &Expr, else_branch: Option<&Expr>) -> Result<(), InterpreterError> {
        self.compile_expr(condition)?;
        
        let else_jump_addr = self.code.len();
        self.emit_instruction(op_jump_if_false, 0);
        
        self.compile_expr(then_branch)?;
        
        if let Some(else_expr) = else_branch {
            self.compile_if_with_else_branch(else_jump_addr, else_expr)
        } else {
            self.compile_if_without_else_branch(else_jump_addr)
        }
    }
    
    fn compile_if_with_else_branch(&mut self, else_jump_addr: usize, else_expr: &Expr) -> Result<(), InterpreterError> {
        let end_jump_addr = self.code.len();
        self.emit_instruction(op_jump, 0);
        
        self.patch_jump_target(else_jump_addr, self.code.len());
        self.compile_expr(else_expr)?;
        self.patch_jump_target(end_jump_addr, self.code.len());
        
        Ok(())
    }
    
    fn compile_if_without_else_branch(&mut self, else_jump_addr: usize) -> Result<(), InterpreterError> {
        self.patch_jump_target(else_jump_addr, self.code.len());
        self.emit_instruction(op_load_nil, 0);
        Ok(())
    }
    
    fn patch_jump_target(&mut self, jump_addr: usize, target: usize) {
        if let Some(instr) = self.code.get_mut(jump_addr) {
            instr.operand = target as u32;
        }
    }
    
    fn compile_fallback_expr(&mut self) -> Result<(), InterpreterError> {
        let value_idx = self.add_constant(Value::String(Rc::new("AST_FALLBACK".to_string())));
        self.emit_instruction(op_ast_fallback, value_idx);
        Ok(())
    }

    /// Add constant to pool and return index
    #[allow(clippy::cast_possible_truncation)] // Index bounds are controlled
    fn add_constant(&mut self, value: Value) -> u32 {
        let idx = self.constants.len();
        self.constants.push(value);
        idx as u32
    }

    /// Emit instruction to code stream
    fn emit_instruction(
        &mut self,
        handler: fn(&mut InterpreterState, u32) -> InstructionResult,
        operand: u32,
    ) {
        self.code.push(ThreadedInstruction { handler, operand });
    }

    /// Convert literal to value
    fn literal_to_value(&self, lit: &Literal) -> Value {
        match lit {
            Literal::Integer(n) => Value::Integer(*n),
            Literal::Float(f) => Value::Float(*f),
            Literal::Bool(b) => Value::Bool(*b),
            Literal::String(s) => Value::String(Rc::new(s.clone())),
            Literal::Char(c) => Value::String(Rc::new(c.to_string())), // Convert char to single-character string
            Literal::Unit => Value::Nil,                               // Unit maps to Nil
        }
    }

    /// Get instruction count
    pub fn instruction_count(&self) -> usize {
        self.code.len()
    }

    /// Get constants count
    pub fn constants_count(&self) -> usize {
        self.constants.len()
    }

    /// Add instruction to code stream (public interface for tests)
    pub fn add_instruction(
        &mut self,
        handler: fn(&mut InterpreterState, u32) -> InstructionResult,
        operand: u32,
    ) {
        self.emit_instruction(handler, operand);
    }

    /// Clear all instructions and constants
    pub fn clear(&mut self) {
        self.code.clear();
        self.constants.clear();
        self.pc = 0;
        self.state = InterpreterState::new();
    }

    /// Execute with custom interpreter state (for tests)
    ///
    /// # Errors
    ///
    /// Returns an error if execution encounters runtime errors such as
    /// stack overflow, division by zero, or undefined variables.
    pub fn execute_with_state(
        &mut self,
        state: &mut InterpreterState,
    ) -> Result<Value, InterpreterError> {
        self.pc = 0;

        loop {
            // Bounds check
            if self.pc >= self.code.len() {
                return Err(InterpreterError::RuntimeError(
                    "PC out of bounds".to_string(),
                ));
            }

            // Direct function pointer call - no switch overhead
            let instruction = &self.code[self.pc];
            let result = (instruction.handler)(state, instruction.operand);

            match result {
                InstructionResult::Continue => {
                    self.pc += 1;
                }
                InstructionResult::Jump(target) => {
                    if target >= self.code.len() {
                        return Err(InterpreterError::RuntimeError(
                            "Jump target out of bounds".to_string(),
                        ));
                    }
                    self.pc = target;
                }
                InstructionResult::Return(value) => {
                    return Ok(value);
                }
                InstructionResult::Error(error) => {
                    return Err(error);
                }
            }

            // Periodic interrupt check for long-running loops
            if self.pc.trailing_zeros() >= 10 {
                // Could add interrupt checking here in the future
            }
        }
    }
}

impl Default for DirectThreadedInterpreter {
    fn default() -> Self {
        Self::new()
    }
}

// Instruction handler functions - these are called via function pointers

/// Load constant onto stack
fn op_load_const(state: &mut InterpreterState, const_idx: u32) -> InstructionResult {
    if let Some(value) = state.constants.get(const_idx as usize) {
        state.stack.push(value.clone());
        InstructionResult::Continue
    } else {
        InstructionResult::Error(InterpreterError::RuntimeError(
            "Invalid constant index".to_string(),
        ))
    }
}

/// Load nil onto stack
fn op_load_nil(state: &mut InterpreterState, _operand: u32) -> InstructionResult {
    state.stack.push(Value::Nil);
    InstructionResult::Continue
}

/// Load variable onto stack
fn op_load_var(state: &mut InterpreterState, name_idx: u32) -> InstructionResult {
    if let Some(Value::String(name)) = state.constants.get(name_idx as usize) {
        // Search environments from innermost to outermost
        for env in state.env_stack.iter().rev() {
            if let Some(value) = env.get(name.as_str()) {
                state.stack.push(value.clone());
                return InstructionResult::Continue;
            }
        }
        InstructionResult::Error(InterpreterError::RuntimeError(format!(
            "Undefined variable: {name}"
        )))
    } else {
        InstructionResult::Error(InterpreterError::RuntimeError(
            "Invalid variable name index".to_string(),
        ))
    }
}

/// Binary add operation
fn op_add(state: &mut InterpreterState, _operand: u32) -> InstructionResult {
    binary_arithmetic_op(state, |a, b| match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => Some(Value::Integer(x + y)),
        (Value::Float(x), Value::Float(y)) => Some(Value::Float(x + y)),
        (Value::Integer(x), Value::Float(y)) => Some(Value::Float(*x as f64 + y)),
        (Value::Float(x), Value::Integer(y)) => Some(Value::Float(x + *y as f64)),
        _ => None,
    })
}

/// Binary subtract operation
fn op_sub(state: &mut InterpreterState, _operand: u32) -> InstructionResult {
    binary_arithmetic_op(state, |a, b| match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => Some(Value::Integer(x - y)),
        (Value::Float(x), Value::Float(y)) => Some(Value::Float(x - y)),
        (Value::Integer(x), Value::Float(y)) => Some(Value::Float(*x as f64 - y)),
        (Value::Float(x), Value::Integer(y)) => Some(Value::Float(x - *y as f64)),
        _ => None,
    })
}

/// Binary multiply operation
fn op_mul(state: &mut InterpreterState, _operand: u32) -> InstructionResult {
    binary_arithmetic_op(state, |a, b| match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => Some(Value::Integer(x * y)),
        (Value::Float(x), Value::Float(y)) => Some(Value::Float(x * y)),
        (Value::Integer(x), Value::Float(y)) => Some(Value::Float(*x as f64 * y)),
        (Value::Float(x), Value::Integer(y)) => Some(Value::Float(x * *y as f64)),
        _ => None,
    })
}

/// Binary divide operation
fn op_div(state: &mut InterpreterState, _operand: u32) -> InstructionResult {
    binary_arithmetic_op(state, |a, b| match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => {
            if *y == 0 {
                return None; // Division by zero
            }
            Some(Value::Integer(x / y))
        }
        (Value::Float(x), Value::Float(y)) => {
            if *y == 0.0 {
                return None;
            }
            Some(Value::Float(x / y))
        }
        (Value::Integer(x), Value::Float(y)) => {
            if *y == 0.0 {
                return None;
            }
            Some(Value::Float(*x as f64 / y))
        }
        (Value::Float(x), Value::Integer(y)) => {
            if *y == 0 {
                return None;
            }
            Some(Value::Float(x / *y as f64))
        }
        _ => None,
    })
}

/// Helper for binary arithmetic operations
fn binary_arithmetic_op<F>(state: &mut InterpreterState, op: F) -> InstructionResult
where
    F: FnOnce(&Value, &Value) -> Option<Value>,
{
    if state.stack.len() < 2 {
        return InstructionResult::Error(InterpreterError::StackUnderflow);
    }

    let right = state.stack.pop().expect("Test should not fail");
    let left = state.stack.pop().expect("Test should not fail");

    match op(&left, &right) {
        Some(result) => {
            state.stack.push(result);
            InstructionResult::Continue
        }
        None => InstructionResult::Error(InterpreterError::TypeError(
            "Invalid operand types".to_string(),
        )),
    }
}

/// Binary equality operation
#[allow(dead_code)] // Will be used in future phases
fn op_eq(state: &mut InterpreterState, _operand: u32) -> InstructionResult {
    binary_comparison_op(state, |a, b| Some(Value::Bool(a == b)))
}

/// Binary not-equal operation
#[allow(dead_code)] // Will be used in future phases
fn op_ne(state: &mut InterpreterState, _operand: u32) -> InstructionResult {
    binary_comparison_op(state, |a, b| Some(Value::Bool(a != b)))
}

/// Binary less-than operation
#[allow(dead_code)] // Will be used in future phases
fn op_lt(state: &mut InterpreterState, _operand: u32) -> InstructionResult {
    binary_comparison_op(state, |a, b| match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => Some(Value::Bool(x < y)),
        (Value::Float(x), Value::Float(y)) => Some(Value::Bool(x < y)),
        (Value::Integer(x), Value::Float(y)) => Some(Value::Bool((*x as f64) < *y)),
        (Value::Float(x), Value::Integer(y)) => Some(Value::Bool(*x < (*y as f64))),
        _ => None,
    })
}

/// Binary less-equal operation
#[allow(dead_code)] // Will be used in future phases
fn op_le(state: &mut InterpreterState, _operand: u32) -> InstructionResult {
    binary_comparison_op(state, |a, b| match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => Some(Value::Bool(x <= y)),
        (Value::Float(x), Value::Float(y)) => Some(Value::Bool(x <= y)),
        (Value::Integer(x), Value::Float(y)) => Some(Value::Bool((*x as f64) <= *y)),
        (Value::Float(x), Value::Integer(y)) => Some(Value::Bool(*x <= (*y as f64))),
        _ => None,
    })
}

/// Binary greater-than operation
#[allow(dead_code)] // Will be used in future phases
fn op_gt(state: &mut InterpreterState, _operand: u32) -> InstructionResult {
    binary_comparison_op(state, |a, b| match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => Some(Value::Bool(x > y)),
        (Value::Float(x), Value::Float(y)) => Some(Value::Bool(x > y)),
        (Value::Integer(x), Value::Float(y)) => Some(Value::Bool((*x as f64) > *y)),
        (Value::Float(x), Value::Integer(y)) => Some(Value::Bool(*x > (*y as f64))),
        _ => None,
    })
}

/// Binary greater-equal operation
#[allow(dead_code)] // Will be used in future phases
fn op_ge(state: &mut InterpreterState, _operand: u32) -> InstructionResult {
    binary_comparison_op(state, |a, b| match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => Some(Value::Bool(x >= y)),
        (Value::Float(x), Value::Float(y)) => Some(Value::Bool(x >= y)),
        (Value::Integer(x), Value::Float(y)) => Some(Value::Bool((*x as f64) >= *y)),
        (Value::Float(x), Value::Integer(y)) => Some(Value::Bool(*x >= (*y as f64))),
        _ => None,
    })
}

/// Helper for binary comparison operations
#[allow(dead_code)] // Will be used in future phases
fn binary_comparison_op<F>(state: &mut InterpreterState, op: F) -> InstructionResult
where
    F: FnOnce(&Value, &Value) -> Option<Value>,
{
    if state.stack.len() < 2 {
        return InstructionResult::Error(InterpreterError::StackUnderflow);
    }

    let right = state.stack.pop().expect("Test should not fail");
    let left = state.stack.pop().expect("Test should not fail");

    match op(&left, &right) {
        Some(result) => {
            state.stack.push(result);
            InstructionResult::Continue
        }
        None => InstructionResult::Error(InterpreterError::TypeError(
            "Invalid operand types for comparison".to_string(),
        )),
    }
}

/// Binary logical AND operation
#[allow(dead_code)] // Will be used in future phases
fn op_and(state: &mut InterpreterState, _operand: u32) -> InstructionResult {
    if state.stack.len() < 2 {
        return InstructionResult::Error(InterpreterError::StackUnderflow);
    }

    let right = state.stack.pop().expect("Test should not fail");
    let left = state.stack.pop().expect("Test should not fail");

    // Short-circuit evaluation: if left is false, return left; otherwise return right
    let result = if left.is_truthy() { right } else { left };
    state.stack.push(result);
    InstructionResult::Continue
}

/// Binary logical OR operation
#[allow(dead_code)] // Will be used in future phases
fn op_or(state: &mut InterpreterState, _operand: u32) -> InstructionResult {
    if state.stack.len() < 2 {
        return InstructionResult::Error(InterpreterError::StackUnderflow);
    }

    let right = state.stack.pop().expect("Test should not fail");
    let left = state.stack.pop().expect("Test should not fail");

    // Short-circuit evaluation: if left is true, return left; otherwise return right
    let result = if left.is_truthy() { left } else { right };
    state.stack.push(result);
    InstructionResult::Continue
}

/// Jump if top of stack is false
fn op_jump_if_false(state: &mut InterpreterState, target: u32) -> InstructionResult {
    if state.stack.is_empty() {
        return InstructionResult::Error(InterpreterError::StackUnderflow);
    }

    let condition = state.stack.pop().expect("Test should not fail");
    if condition.is_truthy() {
        InstructionResult::Continue
    } else {
        InstructionResult::Jump(target as usize)
    }
}

/// Unconditional jump
fn op_jump(state: &mut InterpreterState, target: u32) -> InstructionResult {
    let _ = state; // Unused but required for signature consistency
    InstructionResult::Jump(target as usize)
}

/// Return top of stack
fn op_return(state: &mut InterpreterState, _operand: u32) -> InstructionResult {
    if let Some(value) = state.stack.pop() {
        InstructionResult::Return(value)
    } else {
        InstructionResult::Return(Value::Nil)
    }
}

/// Fallback to AST evaluation for unsupported expressions
fn op_ast_fallback(_state: &mut InterpreterState, _operand: u32) -> InstructionResult {
    // In a real implementation, this would call back to the AST evaluator
    // For now, just return an error
    InstructionResult::Error(InterpreterError::RuntimeError(
        "AST fallback not implemented".to_string(),
    ))
}

impl Value {
    /// Get type identifier for inline caching
    pub fn type_id(&self) -> std::any::TypeId {
        match self {
            Value::Integer(_) => std::any::TypeId::of::<i64>(),
            Value::Float(_) => std::any::TypeId::of::<f64>(),
            Value::Bool(_) => std::any::TypeId::of::<bool>(),
            Value::Nil => std::any::TypeId::of::<()>(),
            Value::String(_) => std::any::TypeId::of::<String>(),
            Value::Array(_) => std::any::TypeId::of::<Vec<Value>>(),
            Value::Tuple(_) => std::any::TypeId::of::<(Value,)>(),
            Value::Closure { .. } => std::any::TypeId::of::<fn()>(),
            Value::DataFrame { .. } => std::any::TypeId::of::<DataFrameColumn>(),
        }
    }
}

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
        let mut global_env = HashMap::new();
        
        // Add builtin functions to global environment
        // These are special markers that will be handled in eval_function_call
        global_env.insert("format".to_string(), Value::String(Rc::new("__builtin_format__".to_string())));
        global_env.insert("HashMap".to_string(), Value::String(Rc::new("__builtin_hashmap__".to_string())));
        global_env.insert("DataFrame".to_string(), Value::String(Rc::new("__builtin_dataframe__".to_string())));
        global_env.insert("DataFrame::from_range".to_string(), Value::String(Rc::new("__builtin_dataframe_from_range__".to_string())));
        global_env.insert("DataFrame::from_rows".to_string(), Value::String(Rc::new("__builtin_dataframe_from_rows__".to_string())));
        global_env.insert("col".to_string(), Value::String(Rc::new("__builtin_col__".to_string())));
        
        Self {
            stack: Vec::with_capacity(1024), // Pre-allocate stack
            env_stack: vec![global_env], // Start with global environment containing builtins
            frames: Vec::new(),
            execution_counts: HashMap::new(),
            field_caches: HashMap::new(),
            type_feedback: TypeFeedback::new(),
            gc: ConservativeGC::new(),
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
            // Basic expressions
            ExprKind::Literal(lit) => Ok(self.eval_literal(lit)),
            ExprKind::Identifier(name) => self.lookup_variable(name),
            
            // Operations and calls
            ExprKind::Binary { left, op, right } => self.eval_binary_expr(left, *op, right),
            ExprKind::Unary { op, operand } => self.eval_unary_expr(*op, operand),
            ExprKind::Call { func, args } => self.eval_function_call(func, args),
            ExprKind::MethodCall { receiver, method, args } => self.eval_method_call(receiver, method, args),
            ExprKind::DataFrameOperation { source, operation } => self.eval_dataframe_operation(source, operation),
            
            // Functions and lambdas
            ExprKind::Function { name, params, body, .. } => self.eval_function(name, params, body),
            ExprKind::Lambda { params, body } => self.eval_lambda(params, body),
            
            // Control flow expressions
            kind if Self::is_control_flow_expr(kind) => self.eval_control_flow_expr(kind),
            
            // Data structure expressions
            kind if Self::is_data_structure_expr(kind) => self.eval_data_structure_expr(kind),
            
            // Assignment expressions
            kind if Self::is_assignment_expr(kind) => self.eval_assignment_expr(kind),
            
            // Other expressions
            ExprKind::StringInterpolation { parts } => self.eval_string_interpolation(parts),
            ExprKind::QualifiedName { module, name } => self.eval_qualified_name(module, name),
            
            // Unimplemented expressions
            _ => Err(InterpreterError::RuntimeError(format!(
                "Expression type not yet implemented: {expr_kind:?}"
            ))),
        }
    }
    
    // Helper methods for expression type categorization and evaluation (complexity <10 each)
    
    fn is_control_flow_expr(expr_kind: &ExprKind) -> bool {
        matches!(expr_kind, 
            ExprKind::If { .. } | 
            ExprKind::Let { .. } | 
            ExprKind::For { .. } | 
            ExprKind::While { .. } | 
            ExprKind::Match { .. } | 
            ExprKind::Break { .. } | 
            ExprKind::Continue { .. } | 
            ExprKind::Return { .. }
        )
    }
    
    fn is_data_structure_expr(expr_kind: &ExprKind) -> bool {
        matches!(expr_kind,
            ExprKind::List(_) |
            ExprKind::Block(_) |
            ExprKind::Tuple(_) |
            ExprKind::Range { .. } |
            ExprKind::ArrayInit { .. }
        )
    }
    
    fn is_assignment_expr(expr_kind: &ExprKind) -> bool {
        matches!(expr_kind,
            ExprKind::Assign { .. } |
            ExprKind::CompoundAssign { .. }
        )
    }
    
    fn eval_control_flow_expr(&mut self, expr_kind: &ExprKind) -> Result<Value, InterpreterError> {
        match expr_kind {
            ExprKind::If { condition, then_branch, else_branch } => 
                self.eval_if_expr(condition, then_branch, else_branch.as_deref()),
            ExprKind::Let { name, value, body, .. } => 
                self.eval_let_expr(name, value, body),
            ExprKind::For { var, pattern, iter, body } => 
                self.eval_for_loop(var, pattern.as_ref(), iter, body),
            ExprKind::While { condition, body } => 
                self.eval_while_loop(condition, body),
            ExprKind::Match { expr, arms } => 
                self.eval_match(expr, arms),
            ExprKind::Break { label: _ } => 
                Err(InterpreterError::RuntimeError("break".to_string())),
            ExprKind::Continue { label: _ } => 
                Err(InterpreterError::RuntimeError("continue".to_string())),
            ExprKind::Return { value } => 
                self.eval_return_expr(value.as_deref()),
            _ => unreachable!("Non-control-flow expression passed to eval_control_flow_expr"),
        }
    }
    
    fn eval_data_structure_expr(&mut self, expr_kind: &ExprKind) -> Result<Value, InterpreterError> {
        match expr_kind {
            ExprKind::List(elements) => self.eval_list_expr(elements),
            ExprKind::Block(statements) => self.eval_block_expr(statements),
            ExprKind::Tuple(elements) => self.eval_tuple_expr(elements),
            ExprKind::Range { start, end, inclusive } => self.eval_range_expr(start, end, *inclusive),
            ExprKind::ArrayInit { value, size } => self.eval_array_init_expr(value, size),
            _ => unreachable!("Non-data-structure expression passed to eval_data_structure_expr"),
        }
    }
    
    fn eval_assignment_expr(&mut self, expr_kind: &ExprKind) -> Result<Value, InterpreterError> {
        match expr_kind {
            ExprKind::Assign { target, value } => self.eval_assign(target, value),
            ExprKind::CompoundAssign { target, op, value } => self.eval_compound_assign(target, *op, value),
            _ => unreachable!("Non-assignment expression passed to eval_assignment_expr"),
        }
    }
    
    fn eval_qualified_name(&self, module: &str, name: &str) -> Result<Value, InterpreterError> {
        if module == "HashMap" && name == "new" {
            Ok(Value::String(Rc::new("__builtin_hashmap__".to_string())))
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
        }
    }

    /// Look up a variable in the environment (searches from innermost to outermost)
    fn lookup_variable(&self, name: &str) -> Result<Value, InterpreterError> {
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
    fn eval_function_call_value(&mut self, func: &Value, args: &[Value]) -> Result<Value, InterpreterError> {
        self.call_function(func.clone(), args)
    }
    
    /// Call a function with given arguments
    fn call_function(&mut self, func: Value, args: &[Value]) -> Result<Value, InterpreterError> {
        match func {
            Value::String(s) if s.starts_with("__builtin_") => {
                // Handle builtin functions
                match s.as_str() {
                    "__builtin_format__" => {
                        if args.is_empty() {
                            return Err(InterpreterError::RuntimeError("format requires at least one argument".to_string()));
                        }
                        
                        // First argument is the format string
                        if let Value::String(format_str) = &args[0] {
                            let mut result = format_str.to_string();
                            let mut arg_index = 1;
                            
                            // Simple format string replacement - find {} and replace with arguments
                            while let Some(pos) = result.find("{}") {
                                if arg_index < args.len() {
                                    let replacement = args[arg_index].to_string();
                                    result.replace_range(pos..pos+2, &replacement);
                                    arg_index += 1;
                                } else {
                                    break;
                                }
                            }
                            
                            Ok(Value::from_string(result))
                        } else {
                            Err(InterpreterError::RuntimeError("format expects string as first argument".to_string()))
                        }
                    }
                    "__builtin_hashmap__" => {
                        // For now, we don't have a proper HashMap type, so return empty string representation
                        Ok(Value::from_string("{}".to_string()))
                    }
                    "__builtin_dataframe__" => {
                        // Handle DataFrame constructor
                        if args.is_empty() {
                            // DataFrame() - create empty DataFrame
                            Ok(Value::DataFrame { columns: Vec::new() })
                        } else if args.len() == 1 {
                            // DataFrame(rows) - from_rows pattern
                            match &args[0] {
                                Value::Array(rows) => {
                                    // Convert rows to columns
                                    let mut columns = Vec::new();
                                    if !rows.is_empty() {
                                        // Determine number of columns from first row
                                        if let Value::Array(first_row) = &rows[0] {
                                            let num_cols = first_row.len();
                                            
                                            // Initialize columns with default names
                                            for col_idx in 0..num_cols {
                                                columns.push(DataFrameColumn {
                                                    name: format!("column_{}", col_idx),
                                                    values: Vec::new(),
                                                });
                                            }
                                            
                                            // Fill column data from rows
                                            for row in rows.iter() {
                                                if let Value::Array(row_values) = row {
                                                    if row_values.len() != num_cols {
                                                        return Err(InterpreterError::RuntimeError(
                                                            "DataFrame rows must have the same length".to_string()
                                                        ));
                                                    }
                                                    for (col_idx, value) in row_values.iter().enumerate() {
                                                        columns[col_idx].values.push(value.clone());
                                                    }
                                                } else {
                                                    return Err(InterpreterError::RuntimeError(
                                                        "DataFrame expects each row to be an array".to_string()
                                                    ));
                                                }
                                            }
                                        } else {
                                            return Err(InterpreterError::RuntimeError(
                                                "DataFrame expects rows to be arrays".to_string()
                                            ));
                                        }
                                    }
                                    Ok(Value::DataFrame { columns })
                                }
                                _ => Err(InterpreterError::RuntimeError(
                                    "DataFrame expects an array of rows".to_string()
                                )),
                            }
                        } else {
                            Err(InterpreterError::RuntimeError(
                                "DataFrame expects 0 or 1 arguments".to_string()
                            ))
                        }
                    }
                    "__builtin_col__" => {
                        // Handle col() function - for now just return the column name as a string
                        // In a full implementation, this would create a column reference object
                        if args.len() == 1 {
                            if let Value::String(_column_name) = &args[0] {
                                // For now, just return the column name
                                // Note: Create a proper ColumnRef type
                                Ok(args[0].clone())
                            } else {
                                Err(InterpreterError::RuntimeError("col() expects a string column name".to_string()))
                            }
                        } else {
                            Err(InterpreterError::RuntimeError("col() expects exactly 1 argument (column_name)".to_string()))
                        }
                    }
                    "__builtin_dataframe_from_range__" => {
                        // Handle DataFrame::from_range(start, end) function
                        if args.len() != 2 {
                            return Err(InterpreterError::RuntimeError("DataFrame::from_range() expects exactly 2 arguments (start, end)".to_string()));
                        }
                        
                        let start = match &args[0] {
                            Value::Integer(s) => *s,
                            _ => return Err(InterpreterError::RuntimeError("DataFrame::from_range() expects start as integer".to_string())),
                        };
                        
                        let end = match &args[1] {
                            Value::Integer(e) => *e,
                            _ => return Err(InterpreterError::RuntimeError("DataFrame::from_range() expects end as integer".to_string())),
                        };
                        
                        if start >= end {
                            return Err(InterpreterError::RuntimeError("DataFrame::from_range() expects start < end".to_string()));
                        }
                        
                        // Create a DataFrame with a single "value" column containing the range
                        let mut values = Vec::new();
                        for i in start..end {
                            values.push(Value::Integer(i));
                        }
                        
                        let column = DataFrameColumn {
                            name: "value".to_string(),
                            values,
                        };
                        
                        Ok(Value::DataFrame {
                            columns: vec![column],
                        })
                    }
                    "__builtin_dataframe_from_rows__" => {
                        // Handle DataFrame::from_rows(rows) function
                        if args.len() != 1 {
                            return Err(InterpreterError::RuntimeError("DataFrame::from_rows() expects exactly 1 argument (rows)".to_string()));
                        }
                        
                        match &args[0] {
                            Value::Array(rows) => {
                                // Convert rows to columns
                                let mut columns = Vec::new();
                                if !rows.is_empty() {
                                    // Determine number of columns from first row
                                    if let Value::Array(first_row) = &rows[0] {
                                        let num_cols = first_row.len();
                                        
                                        // Initialize columns with default names
                                        for col_idx in 0..num_cols {
                                            columns.push(DataFrameColumn {
                                                name: format!("column_{}", col_idx),
                                                values: Vec::new(),
                                            });
                                        }
                                        
                                        // Fill column data from rows
                                        for row in rows.iter() {
                                            if let Value::Array(row_values) = row {
                                                if row_values.len() != num_cols {
                                                    return Err(InterpreterError::RuntimeError(
                                                        "DataFrame::from_rows rows must have the same length".to_string()
                                                    ));
                                                }
                                                for (col_idx, value) in row_values.iter().enumerate() {
                                                    columns[col_idx].values.push(value.clone());
                                                }
                                            } else {
                                                return Err(InterpreterError::RuntimeError(
                                                    "DataFrame::from_rows expects each row to be an array".to_string()
                                                ));
                                            }
                                        }
                                    } else {
                                        return Err(InterpreterError::RuntimeError(
                                            "DataFrame::from_rows expects first row to be an array".to_string()
                                        ));
                                    }
                                }
                                
                                Ok(Value::DataFrame { columns })
                            }
                            _ => Err(InterpreterError::RuntimeError("DataFrame::from_rows() expects rows as array".to_string())),
                        }
                    }
                    _ => Err(InterpreterError::RuntimeError(format!("Unknown builtin function: {}", s))),
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
        match op {
            AstBinaryOp::Add | AstBinaryOp::Subtract | AstBinaryOp::Multiply | 
            AstBinaryOp::Divide | AstBinaryOp::Modulo | AstBinaryOp::Power => {
                self.eval_arithmetic_op(op, left, right)
            }
            AstBinaryOp::Equal | AstBinaryOp::NotEqual | AstBinaryOp::Less | 
            AstBinaryOp::Greater | AstBinaryOp::LessEqual | AstBinaryOp::GreaterEqual => {
                self.eval_comparison_op(op, left, right)
            }
            AstBinaryOp::And | AstBinaryOp::Or => {
                self.eval_logical_op(op, left, right)
            }
            _ => Err(InterpreterError::RuntimeError(format!(
                "Binary operator not yet implemented: {op:?}"
            ))),
        }
    }
    
    /// Handle arithmetic operations.
    ///
    /// Supports numeric operations between integers and floats with automatic
    /// type promotion when mixing types. String concatenation is supported
    /// for the `+` operator.
    ///
    /// # Type Promotion Rules
    ///
    /// - `int + int -> int`
    /// - `int + float -> float`
    /// - `float + float -> float`
    /// - `string + string -> string` (concatenation)
    ///
    /// # Errors
    ///
    /// Returns an error for type mismatches or division by zero.
    fn eval_arithmetic_op(
        &self,
        op: AstBinaryOp,
        left: &Value,
        right: &Value,
    ) -> Result<Value, InterpreterError> {
        match op {
            AstBinaryOp::Add => self.add_values(left, right),
            AstBinaryOp::Subtract => self.sub_values(left, right),
            AstBinaryOp::Multiply => self.mul_values(left, right),
            AstBinaryOp::Divide => self.div_values(left, right),
            AstBinaryOp::Modulo => self.modulo_values(left, right),
            AstBinaryOp::Power => self.power_values(left, right),
            _ => unreachable!("Non-arithmetic operation passed to eval_arithmetic_op"),
        }
    }
    
    /// Handle comparison operations.
    ///
    /// Compares values of compatible types and returns a boolean result.
    /// Supports comparisons between:
    /// - Numbers (integers and floats)
    /// - Strings (lexicographic comparison)
    /// - Booleans
    ///
    /// # Equality
    ///
    /// - Values of different types are never equal
    /// - NaN is not equal to itself (following IEEE 754)
    ///
    /// # Errors
    ///
    /// Returns an error when comparing incompatible types for ordering
    /// operations (`<`, `>`, `<=`, `>=`).
    fn eval_comparison_op(
        &self,
        op: AstBinaryOp,
        left: &Value,
        right: &Value,
    ) -> Result<Value, InterpreterError> {
        match op {
            AstBinaryOp::Equal => Ok(Value::from_bool(self.equal_values(left, right))),
            AstBinaryOp::NotEqual => Ok(Value::from_bool(!self.equal_values(left, right))),
            AstBinaryOp::Less => Ok(Value::from_bool(self.less_than_values(left, right)?)),
            AstBinaryOp::Greater => Ok(Value::from_bool(self.greater_than_values(left, right)?)),
            AstBinaryOp::LessEqual => {
                let less = self.less_than_values(left, right)?;
                let equal = self.equal_values(left, right);
                Ok(Value::from_bool(less || equal))
            }
            AstBinaryOp::GreaterEqual => {
                let greater = self.greater_than_values(left, right)?;
                let equal = self.equal_values(left, right);
                Ok(Value::from_bool(greater || equal))
            }
            _ => unreachable!("Non-comparison operation passed to eval_comparison_op"),
        }
    }
    
    /// Handle logical operations (And, Or)
    fn eval_logical_op(
        &self,
        op: AstBinaryOp,
        left: &Value,
        right: &Value,
    ) -> Result<Value, InterpreterError> {
        match op {
            AstBinaryOp::And => {
                // Short-circuit evaluation for logical AND
                if left.is_truthy() {
                    Ok(right.clone())
                } else {
                    Ok(left.clone())
                }
            }
            AstBinaryOp::Or => {
                // Short-circuit evaluation for logical OR
                if left.is_truthy() {
                    Ok(left.clone())
                } else {
                    Ok(right.clone())
                }
            }
            _ => unreachable!("Non-logical operation passed to eval_logical_op"),
        }
    }

    /// Evaluate a unary operation
    fn eval_unary_op(
        &self,
        op: crate::frontend::ast::UnaryOp,
        operand: &Value,
    ) -> Result<Value, InterpreterError> {
        use crate::frontend::ast::UnaryOp;
        match op {
            UnaryOp::Negate => match operand {
                Value::Integer(i) => Ok(Value::from_i64(-i)),
                Value::Float(f) => Ok(Value::from_f64(-f)),
                _ => Err(InterpreterError::TypeError(format!(
                    "Cannot negate {}",
                    operand.type_name()
                ))),
            },
            UnaryOp::Not => Ok(Value::from_bool(!operand.is_truthy())),
            _ => Err(InterpreterError::RuntimeError(format!(
                "Unary operator not yet implemented: {op:?}"
            ))),
        }
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
        let condition_val = self.eval_expr(condition)?;
        if condition_val.is_truthy() {
            self.eval_expr(then_branch)
        } else if let Some(else_expr) = else_branch {
            self.eval_expr(else_expr)
        } else {
            Ok(Value::nil())
        }
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
            _ => self.eval_expr(body)
        }
    }

    /// Evaluate return expression
    fn eval_return_expr(&mut self, value: Option<&Expr>) -> Result<Value, InterpreterError> {
        if let Some(expr) = value {
            let val = self.eval_expr(expr)?;
            Err(InterpreterError::RuntimeError(format!("return {val:?}")))
        } else {
            Err(InterpreterError::RuntimeError("return".to_string()))
        }
    }

    /// Evaluate list expression
    fn eval_list_expr(&mut self, elements: &[Expr]) -> Result<Value, InterpreterError> {
        let mut values = Vec::new();
        for elem in elements {
            values.push(self.eval_expr(elem)?);
        }
        Ok(Value::from_array(values))
    }

    /// Evaluate array initialization expression [value; size]
    fn eval_array_init_expr(&mut self, value_expr: &Expr, size_expr: &Expr) -> Result<Value, InterpreterError> {
        let value = self.eval_expr(value_expr)?;
        let size_val = self.eval_expr(size_expr)?;
        
        let size = match size_val {
            Value::Integer(n) => n as usize,
            _ => return Err(InterpreterError::RuntimeError(
                "Array size must be an integer".to_string()
            )),
        };
        
        let mut values = Vec::with_capacity(size);
        for _ in 0..size {
            values.push(value.clone());
        }
        
        Ok(Value::from_array(values))
    }

    /// Evaluate block expression
    fn eval_block_expr(&mut self, statements: &[Expr]) -> Result<Value, InterpreterError> {
        let mut result = Value::nil();
        for stmt in statements {
            result = self.eval_expr(stmt)?;
        }
        Ok(result)
    }

    /// Evaluate tuple expression
    fn eval_tuple_expr(&mut self, elements: &[Expr]) -> Result<Value, InterpreterError> {
        let mut values = Vec::new();
        for elem in elements {
            values.push(self.eval_expr(elem)?);
        }
        Ok(Value::Tuple(Rc::new(values)))
    }

    /// Evaluate range expression
    fn eval_range_expr(
        &mut self,
        start: &Expr,
        end: &Expr,
        inclusive: bool,
    ) -> Result<Value, InterpreterError> {
        let start_val = self.eval_expr(start)?;
        let end_val = self.eval_expr(end)?;
        
        match (start_val, end_val) {
            (Value::Integer(start_i), Value::Integer(end_i)) => {
                let range: Vec<Value> = if inclusive {
                    (start_i..=end_i).map(Value::from_i64).collect()
                } else {
                    (start_i..end_i).map(Value::from_i64).collect()
                };
                Ok(Value::from_array(range))
            }
            _ => Err(InterpreterError::TypeError(
                "Range bounds must be integers".to_string(),
            )),
        }
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
            BinaryOp::Add => self.add_values(&left, &right)?,
            BinaryOp::Sub => self.sub_values(&left, &right)?,
            BinaryOp::Mul => self.mul_values(&left, &right)?,
            BinaryOp::Div => self.div_values(&left, &right)?,
            BinaryOp::Eq => Value::from_bool(self.equal_values(&left, &right)),
            BinaryOp::Lt => Value::from_bool(self.less_than_values(&left, &right)?),
            BinaryOp::Gt => Value::from_bool(self.greater_than_values(&left, &right)?),
        };

        self.push(result)?;
        Ok(())
    }

    /// Add two values with type coercion
    fn add_values(&self, left: &Value, right: &Value) -> Result<Value, InterpreterError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::from_i64(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::from_f64(a + b)),
            (Value::Integer(a), Value::Float(b)) =>
            {
                #[allow(clippy::cast_precision_loss)]
                Ok(Value::from_f64(*a as f64 + b))
            }
            (Value::Float(a), Value::Integer(b)) =>
            {
                #[allow(clippy::cast_precision_loss)]
                Ok(Value::from_f64(a + *b as f64))
            }
            (Value::String(a), Value::String(b)) => {
                Ok(Value::from_string(format!("{}{}", a.as_ref(), b.as_ref())))
            }
            _ => Err(InterpreterError::TypeError(format!(
                "Cannot add {} and {}",
                left.type_name(),
                right.type_name()
            ))),
        }
    }

    /// Subtract two values
    fn sub_values(&self, left: &Value, right: &Value) -> Result<Value, InterpreterError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::from_i64(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::from_f64(a - b)),
            (Value::Integer(a), Value::Float(b)) =>
            {
                #[allow(clippy::cast_precision_loss)]
                Ok(Value::from_f64(*a as f64 - b))
            }
            (Value::Float(a), Value::Integer(b)) =>
            {
                #[allow(clippy::cast_precision_loss)]
                Ok(Value::from_f64(a - *b as f64))
            }
            _ => Err(InterpreterError::TypeError(format!(
                "Cannot subtract {} from {}",
                right.type_name(),
                left.type_name()
            ))),
        }
    }

    /// Multiply two values
    fn mul_values(&self, left: &Value, right: &Value) -> Result<Value, InterpreterError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::from_i64(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::from_f64(a * b)),
            (Value::Integer(a), Value::Float(b)) =>
            {
                #[allow(clippy::cast_precision_loss)]
                Ok(Value::from_f64(*a as f64 * b))
            }
            (Value::Float(a), Value::Integer(b)) =>
            {
                #[allow(clippy::cast_precision_loss)]
                Ok(Value::from_f64(a * *b as f64))
            }
            _ => Err(InterpreterError::TypeError(format!(
                "Cannot multiply {} and {}",
                left.type_name(),
                right.type_name()
            ))),
        }
    }

    /// Divide two values
    fn div_values(&self, left: &Value, right: &Value) -> Result<Value, InterpreterError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if *b == 0 {
                    return Err(InterpreterError::DivisionByZero);
                }
                Ok(Value::from_i64(a / b))
            }
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    return Err(InterpreterError::DivisionByZero);
                }
                Ok(Value::from_f64(a / b))
            }
            (Value::Integer(a), Value::Float(b)) => {
                if *b == 0.0 {
                    return Err(InterpreterError::DivisionByZero);
                }
                #[allow(clippy::cast_precision_loss)]
                Ok(Value::from_f64(*a as f64 / b))
            }
            (Value::Float(a), Value::Integer(b)) => {
                #[allow(clippy::cast_precision_loss)]
                let divisor = *b as f64;
                if divisor == 0.0 {
                    return Err(InterpreterError::DivisionByZero);
                }
                Ok(Value::from_f64(a / divisor))
            }
            _ => Err(InterpreterError::TypeError(format!(
                "Cannot divide {} by {}",
                left.type_name(),
                right.type_name()
            ))),
        }
    }

    /// Modulo operation between two values
    fn modulo_values(&self, left: &Value, right: &Value) -> Result<Value, InterpreterError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if *b == 0 {
                    return Err(InterpreterError::DivisionByZero);
                }
                Ok(Value::from_i64(a % b))
            }
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    return Err(InterpreterError::DivisionByZero);
                }
                Ok(Value::from_f64(a % b))
            }
            (Value::Integer(a), Value::Float(b)) => {
                if *b == 0.0 {
                    return Err(InterpreterError::DivisionByZero);
                }
                #[allow(clippy::cast_precision_loss)]
                Ok(Value::from_f64((*a as f64) % b))
            }
            (Value::Float(a), Value::Integer(b)) => {
                #[allow(clippy::cast_precision_loss)]
                let divisor = *b as f64;
                if divisor == 0.0 {
                    return Err(InterpreterError::DivisionByZero);
                }
                Ok(Value::from_f64(a % divisor))
            }
            _ => Err(InterpreterError::TypeError(format!(
                "Cannot compute modulo of {} and {}",
                left.type_name(),
                right.type_name()
            ))),
        }
    }

    /// Power operation between two values
    fn power_values(&self, left: &Value, right: &Value) -> Result<Value, InterpreterError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if *b < 0 {
                    // For negative exponents, convert to float
                    #[allow(clippy::cast_precision_loss)]
                    let result = (*a as f64).powf(*b as f64);
                    Ok(Value::from_f64(result))
                } else {
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    if let Some(result) = a.checked_pow(*b as u32) { Ok(Value::from_i64(result)) } else {
                        // Overflow - convert to float
                        #[allow(clippy::cast_precision_loss)]
                        let result = (*a as f64).powf(*b as f64);
                        Ok(Value::from_f64(result))
                    }
                }
            }
            (Value::Float(a), Value::Float(b)) => {
                Ok(Value::from_f64(a.powf(*b)))
            }
            (Value::Integer(a), Value::Float(b)) => {
                #[allow(clippy::cast_precision_loss)]
                Ok(Value::from_f64((*a as f64).powf(*b)))
            }
            (Value::Float(a), Value::Integer(b)) => {
                #[allow(clippy::cast_precision_loss)]
                Ok(Value::from_f64(a.powf(*b as f64)))
            }
            _ => Err(InterpreterError::TypeError(format!(
                "Cannot raise {} to the power of {}",
                left.type_name(),
                right.type_name()
            ))),
        }
    }

    /// Check equality of two values
    fn equal_values(&self, left: &Value, right: &Value) -> bool {
        left == right // PartialEq is derived for Value
    }

    /// Check if left < right
    fn less_than_values(&self, left: &Value, right: &Value) -> Result<bool, InterpreterError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a < b),
            (Value::Float(a), Value::Float(b)) => Ok(a < b),
            (Value::Integer(a), Value::Float(b)) =>
            {
                #[allow(clippy::cast_precision_loss)]
                Ok((*a as f64) < *b)
            }
            (Value::Float(a), Value::Integer(b)) =>
            {
                #[allow(clippy::cast_precision_loss)]
                Ok(*a < (*b as f64))
            }
            _ => Err(InterpreterError::TypeError(format!(
                "Cannot compare {} and {}",
                left.type_name(),
                right.type_name()
            ))),
        }
    }

    /// Check if left > right
    fn greater_than_values(&self, left: &Value, right: &Value) -> Result<bool, InterpreterError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a > b),
            (Value::Float(a), Value::Float(b)) => Ok(a > b),
            (Value::Integer(a), Value::Float(b)) =>
            {
                #[allow(clippy::cast_precision_loss)]
                Ok((*a as f64) > *b)
            }
            (Value::Float(a), Value::Integer(b)) =>
            {
                #[allow(clippy::cast_precision_loss)]
                Ok(*a > (*b as f64))
            }
            _ => Err(InterpreterError::TypeError(format!(
                "Cannot compare {} and {}",
                left.type_name(),
                right.type_name()
            ))),
        }
    }

    /// Print value for debugging
    pub fn print_value(&self, value: &Value) -> std::string::String {
        match value {
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Nil => "nil".to_string(),
            Value::String(s) => s.as_ref().clone(),
            Value::Array(arr) => {
                let elements: Vec<String> = arr.iter().map(|v| self.print_value(v)).collect();
                format!("[{}]", elements.join(", "))
            }
            Value::Tuple(elements) => {
                let element_strs: Vec<String> = elements.iter().map(|v| self.print_value(v)).collect();
                format!("({})", element_strs.join(", "))
            }
            Value::Closure { params, .. } => {
                format!("function/{}", params.len())
            }
            Value::DataFrame { columns } => {
                format!("DataFrame({} columns, {} rows)", 
                    columns.len(), 
                    columns.first().map_or(0, |c| c.values.len())
                )
            }
        }
    }

    /// Set a variable in the current environment
    fn set_variable(&mut self, name: String, value: Value) {
        self.env_set(name, value);
    }
    
    /// Apply a binary operation to two values
    fn apply_binary_op(&self, left: &Value, op: AstBinaryOp, right: &Value) -> Result<Value, InterpreterError> {
        // Delegate to existing binary operation evaluation
        self.eval_binary_op(op, left, right)
    }
    
    /// Check if a pattern matches a value
    /// # Errors
    /// Returns error if pattern matching fails
    fn pattern_matches(&self, pattern: &Pattern, value: &Value) -> Result<bool, InterpreterError> {
        match pattern {
            Pattern::Wildcard => Ok(true),
            Pattern::Literal(lit) => self.match_literal_pattern(lit, value),
            Pattern::Identifier(_name) => Ok(true), // Always matches, binding handled separately
            Pattern::Tuple(patterns) => self.match_tuple_pattern(patterns, value),
            Pattern::List(patterns) => self.match_list_pattern(patterns, value),
            Pattern::Or(patterns) => self.match_or_pattern(patterns, value),
            Pattern::Range { start, end, inclusive } => self.match_range_pattern(start, end, *inclusive, value),
            _ => Ok(false), // Other patterns not yet implemented
        }
    }
    
    // Helper methods for pattern matching (complexity <10 each)
    
    fn match_literal_pattern(&self, lit: &Literal, value: &Value) -> Result<bool, InterpreterError> {
        let lit_value = self.eval_literal(lit);
        Ok(lit_value == *value)
    }
    
    fn match_tuple_pattern(&self, patterns: &[Pattern], value: &Value) -> Result<bool, InterpreterError> {
        if let Value::Tuple(elements) = value {
            self.match_sequence_patterns(patterns, elements)
        } else {
            Ok(false)
        }
    }
    
    fn match_list_pattern(&self, patterns: &[Pattern], value: &Value) -> Result<bool, InterpreterError> {
        if let Value::Array(elements) = value {
            self.match_sequence_patterns(patterns, elements)
        } else {
            Ok(false)
        }
    }
    
    fn match_sequence_patterns(&self, patterns: &[Pattern], elements: &[Value]) -> Result<bool, InterpreterError> {
        if patterns.len() != elements.len() {
            return Ok(false);
        }
        for (pat, val) in patterns.iter().zip(elements.iter()) {
            if !self.pattern_matches(pat, val)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    
    fn match_or_pattern(&self, patterns: &[Pattern], value: &Value) -> Result<bool, InterpreterError> {
        for pat in patterns {
            if self.pattern_matches(pat, value)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
    
    fn match_range_pattern(&self, start: &Pattern, end: &Pattern, inclusive: bool, value: &Value) -> Result<bool, InterpreterError> {
        if let Value::Integer(i) = value {
            let start_val = self.extract_integer_from_pattern(start)?;
            let end_val = self.extract_integer_from_pattern(end)?;
            
            if inclusive {
                Ok(*i >= start_val && *i <= end_val)
            } else {
                Ok(*i >= start_val && *i < end_val)
            }
        } else {
            Ok(false)
        }
    }
    
    fn extract_integer_from_pattern(&self, pattern: &Pattern) -> Result<i64, InterpreterError> {
        if let Pattern::Literal(Literal::Integer(val)) = pattern {
            Ok(*val)
        } else {
            Err(InterpreterError::RuntimeError("Range pattern requires integer literals".to_string()))
        }
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
            (Value::String(s), "to_upper") => Ok(Value::String(Rc::new(s.to_uppercase()))),
            (Value::String(s), "to_lower") => Ok(Value::String(Rc::new(s.to_lowercase()))),
            (Value::String(s), "trim") => Ok(Value::String(Rc::new(s.trim().to_string()))),

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
            (Value::Array(arr), "is_empty") => {
                Ok(Value::from_bool(arr.is_empty()))
            }

            // Type information
            (obj, "type") => Ok(Value::String(Rc::new(obj.type_name().to_string()))),

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
        let array_value = Value::Array(Rc::new(elements));
        self.gc.track_object(array_value.clone());
        array_value
    }

    /// Allocate a new string and track it in GC
    pub fn gc_alloc_string(&mut self, content: String) -> Value {
        let string_value = Value::String(Rc::new(content));
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
    
    /// Get all bindings from the current environment (for `SharedSession` extraction)
    pub fn get_current_bindings(&self) -> HashMap<String, Value> {
        if let Some(current_env) = self.env_stack.last() {
            current_env.clone()
        } else {
            HashMap::new()
        }
    }
    
    /// Evaluate a for loop
    fn eval_for_loop(&mut self, var: &str, pattern: Option<&Pattern>, iter: &Expr, body: &Expr) -> Result<Value, InterpreterError> {
        let iter_value = self.eval_expr(iter)?;
        
        match iter_value {
            Value::Array(arr) => {
                let mut last_value = Value::nil();
                for item in arr.iter() {
                    // Handle pattern matching if present
                    if let Some(_pat) = pattern {
                        // Pattern matching for destructuring would go here
                        // For now, just bind to var
                        self.set_variable(var.to_string(), item.clone());
                    } else {
                        // Simple variable binding
                        self.set_variable(var.to_string(), item.clone());
                    }
                    
                    // Execute body
                    match self.eval_expr(body) {
                        Ok(value) => last_value = value,
                        Err(InterpreterError::RuntimeError(msg)) if msg == "break" => break,
                        Err(InterpreterError::RuntimeError(msg)) if msg == "continue" => {},
                        Err(e) => return Err(e),
                    }
                }
                Ok(last_value)
            }
            _ => Err(InterpreterError::TypeError(
                "For loop requires an iterable (array)".to_string()
            )),
        }
    }
    
    /// Evaluate a while loop
    fn eval_while_loop(&mut self, condition: &Expr, body: &Expr) -> Result<Value, InterpreterError> {
        let mut last_value = Value::nil();
        loop {
            let cond_value = self.eval_expr(condition)?;
            if !cond_value.is_truthy() {
                break;
            }
            
            match self.eval_expr(body) {
                Ok(val) => last_value = val,
                Err(InterpreterError::RuntimeError(msg)) if msg == "break" => break,
                Err(InterpreterError::RuntimeError(msg)) if msg == "continue" => {},
                Err(e) => return Err(e),
            }
        }
        Ok(last_value)
    }
    
    /// Evaluate a match expression
    fn eval_match(&mut self, expr: &Expr, arms: &[MatchArm]) -> Result<Value, InterpreterError> {
        let value = self.eval_expr(expr)?;
        
        for arm in arms {
            if self.pattern_matches(&arm.pattern, &value)? {
                // Pattern bindings are handled by pattern_matches method
                // when it returns true, any variables are already bound
                return self.eval_expr(&arm.body);
            }
        }
        
        Err(InterpreterError::RuntimeError(
            "No match arm matched the value".to_string()
        ))
    }
    
    /// Evaluate an assignment
    fn eval_assign(&mut self, target: &Expr, value: &Expr) -> Result<Value, InterpreterError> {
        let val = self.eval_expr(value)?;
        
        // Handle different assignment targets
        match &target.kind {
            ExprKind::Identifier(name) => {
                self.set_variable(name.clone(), val.clone());
                Ok(val)
            }
            _ => Err(InterpreterError::RuntimeError(
                "Invalid assignment target".to_string()
            )),
        }
    }
    
    /// Evaluate a compound assignment
    fn eval_compound_assign(&mut self, target: &Expr, op: AstBinaryOp, value: &Expr) -> Result<Value, InterpreterError> {
        // Get current value
        let current = match &target.kind {
            ExprKind::Identifier(name) => self.lookup_variable(name)?,
            _ => return Err(InterpreterError::RuntimeError(
                "Invalid compound assignment target".to_string()
            )),
        };
        
        // Compute new value
        let rhs = self.eval_expr(value)?;
        let new_val = self.apply_binary_op(&current, op, &rhs)?;
        
        // Assign back
        if let ExprKind::Identifier(name) = &target.kind {
            self.set_variable(name.clone(), new_val.clone());
        }
        
        Ok(new_val)
    }
    
    /// Evaluate string methods
    #[allow(clippy::rc_buffer)]
    fn eval_string_method(&mut self, s: &Rc<String>, method: &str, args: &[Value]) -> Result<Value, InterpreterError> {
        match method {
            "len" if args.is_empty() => Ok(Value::Integer(s.len() as i64)),
            "to_upper" if args.is_empty() => Ok(Value::from_string(s.to_uppercase())),
            "to_lower" if args.is_empty() => Ok(Value::from_string(s.to_lowercase())),
            "trim" if args.is_empty() => Ok(Value::from_string(s.trim().to_string())),
            "to_string" if args.is_empty() => Ok(Value::from_string(s.to_string())),
            "contains" if args.len() == 1 => {
                if let Value::String(needle) = &args[0] {
                    Ok(Value::Bool(s.contains(needle.as_str())))
                } else {
                    Err(InterpreterError::RuntimeError("contains expects string argument".to_string()))
                }
            }
            "starts_with" if args.len() == 1 => {
                if let Value::String(prefix) = &args[0] {
                    Ok(Value::Bool(s.starts_with(prefix.as_str())))
                } else {
                    Err(InterpreterError::RuntimeError("starts_with expects string argument".to_string()))
                }
            }
            "ends_with" if args.len() == 1 => {
                if let Value::String(suffix) = &args[0] {
                    Ok(Value::Bool(s.ends_with(suffix.as_str())))
                } else {
                    Err(InterpreterError::RuntimeError("ends_with expects string argument".to_string()))
                }
            }
            "replace" if args.len() == 2 => {
                if let (Value::String(from), Value::String(to)) = (&args[0], &args[1]) {
                    Ok(Value::from_string(s.replace(from.as_str(), to.as_str())))
                } else {
                    Err(InterpreterError::RuntimeError("replace expects two string arguments".to_string()))
                }
            }
            "split" if args.len() == 1 => {
                if let Value::String(separator) = &args[0] {
                    let parts: Vec<Value> = s.split(separator.as_str())
                        .map(|part| Value::from_string(part.to_string()))
                        .collect();
                    Ok(Value::Array(Rc::new(parts)))
                } else {
                    Err(InterpreterError::RuntimeError("split expects string argument".to_string()))
                }
            }
            _ => Err(InterpreterError::RuntimeError(format!("Unknown string method: {}", method))),
        }
    }

    /// Evaluate array methods
    #[allow(clippy::rc_buffer)]
    fn eval_array_method(&mut self, arr: &Rc<Vec<Value>>, method: &str, args: &[Value]) -> Result<Value, InterpreterError> {
        match method {
            "len" if args.is_empty() => Ok(Value::Integer(arr.len() as i64)),
            "push" if args.len() == 1 => {
                let mut new_arr = (**arr).clone();
                new_arr.push(args[0].clone());
                Ok(Value::Array(Rc::new(new_arr)))
            }
            "pop" if args.is_empty() => {
                let mut new_arr = (**arr).clone();
                new_arr.pop().unwrap_or(Value::nil());
                Ok(Value::Array(Rc::new(new_arr)))
            }
            "get" if args.len() == 1 => {
                if let Value::Integer(idx) = &args[0] {
                    if *idx < 0 {
                        return Ok(Value::Nil);
                    }
                    #[allow(clippy::cast_sign_loss)]
                    let index = *idx as usize;
                    if index < arr.len() {
                        Ok(arr[index].clone())
                    } else {
                        Ok(Value::Nil)
                    }
                } else {
                    Err(InterpreterError::RuntimeError("get expects integer index".to_string()))
                }
            }
            "first" if args.is_empty() => Ok(arr.first().cloned().unwrap_or(Value::Nil)),
            "last" if args.is_empty() => Ok(arr.last().cloned().unwrap_or(Value::Nil)),
            "map" | "filter" | "reduce" | "any" | "all" | "find" => 
                self.eval_array_higher_order_method(arr, method, args),
            _ => Err(InterpreterError::RuntimeError(format!("Unknown array method: {}", method))),
        }
    }
    
    /// Evaluate higher-order array methods
    #[allow(clippy::rc_buffer)]
    fn eval_array_higher_order_method(&mut self, arr: &Rc<Vec<Value>>, method: &str, args: &[Value]) -> Result<Value, InterpreterError> {
        match method {
            "map" => self.eval_array_map_method(arr, args),
            "filter" => self.eval_array_filter_method(arr, args),
            "reduce" => self.eval_array_reduce_method(arr, args),
            "any" => self.eval_array_any_method(arr, args),
            "all" => self.eval_array_all_method(arr, args),
            "find" => self.eval_array_find_method(arr, args),
            _ => Err(InterpreterError::RuntimeError(format!("Unknown array method: {}", method))),
        }
    }
    
    // Helper methods for array higher-order functions (complexity <10 each)
    
    #[allow(clippy::rc_buffer)]
    fn eval_array_map_method(&mut self, arr: &Rc<Vec<Value>>, args: &[Value]) -> Result<Value, InterpreterError> {
        self.validate_single_closure_argument(args, "map")?;
        let mut result = Vec::new();
        for item in arr.iter() {
            let func_result = self.eval_function_call_value(&args[0], std::slice::from_ref(item))?;
            result.push(func_result);
        }
        Ok(Value::Array(Rc::new(result)))
    }
    
    #[allow(clippy::rc_buffer)]
    fn eval_array_filter_method(&mut self, arr: &Rc<Vec<Value>>, args: &[Value]) -> Result<Value, InterpreterError> {
        self.validate_single_closure_argument(args, "filter")?;
        let mut result = Vec::new();
        for item in arr.iter() {
            let func_result = self.eval_function_call_value(&args[0], std::slice::from_ref(item))?;
            if func_result.is_truthy() {
                result.push(item.clone());
            }
        }
        Ok(Value::Array(Rc::new(result)))
    }
    
    #[allow(clippy::rc_buffer)]
    fn eval_array_reduce_method(&mut self, arr: &Rc<Vec<Value>>, args: &[Value]) -> Result<Value, InterpreterError> {
        if args.len() != 2 {
            return Err(InterpreterError::RuntimeError("reduce expects 2 arguments".to_string()));
        }
        if !matches!(&args[0], Value::Closure { .. }) {
            return Err(InterpreterError::RuntimeError("reduce expects a function and initial value".to_string()));
        }
        
        let mut accumulator = args[1].clone();
        for item in arr.iter() {
            accumulator = self.eval_function_call_value(&args[0], &[accumulator, item.clone()])?;
        }
        Ok(accumulator)
    }
    
    #[allow(clippy::rc_buffer)]
    fn eval_array_any_method(&mut self, arr: &Rc<Vec<Value>>, args: &[Value]) -> Result<Value, InterpreterError> {
        self.validate_single_closure_argument(args, "any")?;
        for item in arr.iter() {
            let func_result = self.eval_function_call_value(&args[0], std::slice::from_ref(item))?;
            if func_result.is_truthy() {
                return Ok(Value::Bool(true));
            }
        }
        Ok(Value::Bool(false))
    }
    
    #[allow(clippy::rc_buffer)]
    fn eval_array_all_method(&mut self, arr: &Rc<Vec<Value>>, args: &[Value]) -> Result<Value, InterpreterError> {
        self.validate_single_closure_argument(args, "all")?;
        for item in arr.iter() {
            let func_result = self.eval_function_call_value(&args[0], std::slice::from_ref(item))?;
            if !func_result.is_truthy() {
                return Ok(Value::Bool(false));
            }
        }
        Ok(Value::Bool(true))
    }
    
    #[allow(clippy::rc_buffer)]
    fn eval_array_find_method(&mut self, arr: &Rc<Vec<Value>>, args: &[Value]) -> Result<Value, InterpreterError> {
        self.validate_single_closure_argument(args, "find")?;
        for item in arr.iter() {
            let func_result = self.eval_function_call_value(&args[0], std::slice::from_ref(item))?;
            if func_result.is_truthy() {
                return Ok(item.clone());
            }
        }
        Ok(Value::Nil)
    }
    
    fn validate_single_closure_argument(&self, args: &[Value], method_name: &str) -> Result<(), InterpreterError> {
        if args.len() != 1 {
            return Err(InterpreterError::RuntimeError(format!("{} expects 1 argument", method_name)));
        }
        if !matches!(&args[0], Value::Closure { .. }) {
            return Err(InterpreterError::RuntimeError(format!("{} expects a function argument", method_name)));
        }
        Ok(())
    }

    /// Evaluate a method call
    fn eval_method_call(&mut self, receiver: &Expr, method: &str, args: &[Expr]) -> Result<Value, InterpreterError> {
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
    
    fn dispatch_method_call(&mut self, receiver: &Value, method: &str, arg_values: &[Value], args_empty: bool) -> Result<Value, InterpreterError> {
        match receiver {
            Value::String(s) => self.eval_string_method(s, method, arg_values),
            Value::Array(arr) => self.eval_array_method(arr, method, arg_values),
            Value::Float(f) => self.eval_float_method(*f, method, args_empty),
            Value::Integer(n) => self.eval_integer_method(*n, method, args_empty),
            Value::DataFrame { columns } => self.eval_dataframe_method(columns, method, arg_values),
            _ => self.eval_generic_method(receiver, method, args_empty),
        }
    }
    
    fn eval_float_method(&self, f: f64, method: &str, args_empty: bool) -> Result<Value, InterpreterError> {
        if !args_empty {
            return Err(InterpreterError::RuntimeError(format!("Float method '{}' takes no arguments", method)));
        }
        
        match method {
            "sqrt" => Ok(Value::Float(f.sqrt())),
            "abs" => Ok(Value::Float(f.abs())),
            "round" => Ok(Value::Float(f.round())),
            "floor" => Ok(Value::Float(f.floor())),
            "ceil" => Ok(Value::Float(f.ceil())),
            "to_string" => Ok(Value::from_string(f.to_string())),
            _ => Err(InterpreterError::RuntimeError(format!("Unknown float method: {}", method))),
        }
    }
    
    fn eval_integer_method(&self, n: i64, method: &str, args_empty: bool) -> Result<Value, InterpreterError> {
        if !args_empty {
            return Err(InterpreterError::RuntimeError(format!("Integer method '{}' takes no arguments", method)));
        }
        
        match method {
            "abs" => Ok(Value::Integer(n.abs())),
            "to_string" => Ok(Value::from_string(n.to_string())),
            _ => Err(InterpreterError::RuntimeError(format!("Unknown integer method: {}", method))),
        }
    }
    
    fn eval_generic_method(&self, receiver: &Value, method: &str, args_empty: bool) -> Result<Value, InterpreterError> {
        if method == "to_string" && args_empty {
            Ok(Value::from_string(receiver.to_string()))
        } else {
            Err(InterpreterError::RuntimeError(format!(
                "Method '{}' not found for type {}",
                method,
                receiver.type_name()
            )))
        }
    }
    
    fn eval_dataframe_method(&self, columns: &[DataFrameColumn], method: &str, arg_values: &[Value]) -> Result<Value, InterpreterError> {
        match method {
            "select" => {
                // Select specific columns by name
                if arg_values.len() != 1 {
                    return Err(InterpreterError::RuntimeError("DataFrame.select() requires exactly 1 argument (column_name)".to_string()));
                }
                
                if let Value::String(column_name) = &arg_values[0] {
                    // Find the column
                    for col in columns {
                        if col.name == **column_name {
                            // Return a DataFrame with just this column
                            return Ok(Value::DataFrame {
                                columns: vec![col.clone()],
                            });
                        }
                    }
                    Err(InterpreterError::RuntimeError(format!("Column '{}' not found in DataFrame", column_name)))
                } else {
                    Err(InterpreterError::RuntimeError("DataFrame.select() expects column name as string".to_string()))
                }
            }
            "sum" => {
                // Sum all numeric values in all columns
                if !arg_values.is_empty() {
                    return Err(InterpreterError::RuntimeError("DataFrame.sum() takes no arguments".to_string()));
                }
                
                let mut total = 0.0;
                for col in columns {
                    for value in &col.values {
                        match value {
                            Value::Integer(i) => total += *i as f64,
                            Value::Float(f) => total += f,
                            _ => {} // Skip non-numeric values
                        }
                    }
                }
                
                // Return as integer if it's a whole number, otherwise float
                if total.fract() == 0.0 {
                    Ok(Value::Integer(total as i64))
                } else {
                    Ok(Value::Float(total))
                }
            }
            "slice" => {
                // Slice DataFrame rows
                if arg_values.len() != 2 {
                    return Err(InterpreterError::RuntimeError("DataFrame.slice() requires exactly 2 arguments (start, length)".to_string()));
                }
                
                let start = match &arg_values[0] {
                    Value::Integer(s) => *s as usize,
                    _ => return Err(InterpreterError::RuntimeError("DataFrame.slice() expects start as integer".to_string())),
                };
                
                let length = match &arg_values[1] {
                    Value::Integer(l) => *l as usize,
                    _ => return Err(InterpreterError::RuntimeError("DataFrame.slice() expects length as integer".to_string())),
                };
                
                // Create new columns with sliced values (zero-copy by cloning references)
                let mut sliced_columns = Vec::new();
                for col in columns {
                    let end_idx = (start + length).min(col.values.len());
                    let sliced_values = if start < col.values.len() {
                        col.values[start..end_idx].to_vec()
                    } else {
                        Vec::new()
                    };
                    
                    sliced_columns.push(DataFrameColumn {
                        name: col.name.clone(),
                        values: sliced_values,
                    });
                }
                
                Ok(Value::DataFrame {
                    columns: sliced_columns,
                })
            }
            "join" => {
                // Join two DataFrames
                if arg_values.len() != 2 {
                    return Err(InterpreterError::RuntimeError("DataFrame.join() requires exactly 2 arguments (other_df, on)".to_string()));
                }
                
                let other_df = &arg_values[0];
                let join_column = match &arg_values[1] {
                    Value::String(col_name) => col_name.as_str(),
                    _ => return Err(InterpreterError::RuntimeError("DataFrame.join() expects 'on' as string column name".to_string())),
                };
                
                if let Value::DataFrame { columns: other_columns } = other_df {
                    // Find the join column in both DataFrames
                    let left_join_col = columns.iter().find(|col| col.name == join_column);
                    let right_join_col = other_columns.iter().find(|col| col.name == join_column);
                    
                    if left_join_col.is_none() {
                        return Err(InterpreterError::RuntimeError(format!("Join column '{}' not found in left DataFrame", join_column)));
                    }
                    if right_join_col.is_none() {
                        return Err(InterpreterError::RuntimeError(format!("Join column '{}' not found in right DataFrame", join_column)));
                    }
                    
                    let left_join_col = left_join_col
                        .expect("left_join_col existence verified by is_none() check above");
                    let right_join_col = right_join_col
                        .expect("right_join_col existence verified by is_none() check above");
                    
                    // For simplicity, implement inner join
                    let mut joined_columns = Vec::new();
                    
                    // Add all columns from left DataFrame
                    for col in columns {
                        joined_columns.push(DataFrameColumn {
                            name: col.name.clone(),
                            values: Vec::new(),
                        });
                    }
                    
                    // Add columns from right DataFrame (excluding the join column to avoid duplication)
                    for col in other_columns {
                        if col.name != join_column {
                            joined_columns.push(DataFrameColumn {
                                name: format!("{}_right", col.name), // Rename to avoid conflicts
                                values: Vec::new(),
                            });
                        }
                    }
                    
                    // Perform inner join
                    for (left_idx, left_join_val) in left_join_col.values.iter().enumerate() {
                        for (right_idx, right_join_val) in right_join_col.values.iter().enumerate() {
                            if self.values_equal(left_join_val, right_join_val) {
                                // Match found - add row to result
                                
                                // Add values from left DataFrame
                                for (col_idx, col) in columns.iter().enumerate() {
                                    if let Some(val) = col.values.get(left_idx) {
                                        joined_columns[col_idx].values.push(val.clone());
                                    }
                                }
                                
                                // Add values from right DataFrame (excluding join column)
                                let mut right_col_idx = columns.len();
                                for col in other_columns {
                                    if col.name != join_column {
                                        if let Some(val) = col.values.get(right_idx) {
                                            joined_columns[right_col_idx].values.push(val.clone());
                                        }
                                        right_col_idx += 1;
                                    }
                                }
                            }
                        }
                    }
                    
                    Ok(Value::DataFrame {
                        columns: joined_columns,
                    })
                } else {
                    Err(InterpreterError::RuntimeError("DataFrame.join() expects first argument to be a DataFrame".to_string()))
                }
            }
            "groupby" => {
                // Group by a column (simplified implementation - returns grouped DataFrame for now)
                if arg_values.len() != 1 {
                    return Err(InterpreterError::RuntimeError("DataFrame.groupby() requires exactly 1 argument (column_name)".to_string()));
                }
                
                let group_column = match &arg_values[0] {
                    Value::String(col_name) => col_name.as_str(),
                    _ => return Err(InterpreterError::RuntimeError("DataFrame.groupby() expects column name as string".to_string())),
                };
                
                // Find the group column
                let group_col = columns.iter().find(|col| col.name == group_column);
                if group_col.is_none() {
                    return Err(InterpreterError::RuntimeError(format!("Group column '{}' not found in DataFrame", group_column)));
                }
                let group_col = group_col
                    .expect("group_col existence verified by is_none() check above");
                
                // For simplicity, implement groupby as immediate aggregation (sum)
                // In a full implementation, this would return a GroupBy object
                use std::collections::HashMap;
                let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
                
                // Group rows by the group column values
                for (row_idx, value) in group_col.values.iter().enumerate() {
                    let key = match value {
                        Value::String(s) => s.to_string(),
                        Value::Integer(i) => i.to_string(),
                        Value::Float(f) => f.to_string(),
                        Value::Bool(b) => b.to_string(),
                        _ => "null".to_string(),
                    };
                    groups.entry(key).or_default().push(row_idx);
                }
                
                // Create result columns: group column + aggregated numeric columns
                let mut result_columns = Vec::new();
                
                // Group column (unique values)
                let mut group_values = Vec::new();
                for key in groups.keys() {
                    group_values.push(Value::from_string(key.clone()));
                }
                result_columns.push(DataFrameColumn {
                    name: group_column.to_string(),
                    values: group_values,
                });
                
                // Aggregate numeric columns
                for col in columns {
                    if col.name != group_column {
                        let mut aggregated_values = Vec::new();
                        for indices in groups.values() {
                            let mut sum = 0.0;
                            for &idx in indices {
                                if let Some(value) = col.values.get(idx) {
                                    match value {
                                        Value::Integer(i) => sum += *i as f64,
                                        Value::Float(f) => sum += f,
                                        _ => {} // Skip non-numeric values
                                    }
                                }
                            }
                            // Return as integer if it's a whole number, otherwise float
                            if sum.fract() == 0.0 {
                                aggregated_values.push(Value::Integer(sum as i64));
                            } else {
                                aggregated_values.push(Value::Float(sum));
                            }
                        }
                        result_columns.push(DataFrameColumn {
                            name: format!("{}_sum", col.name),
                            values: aggregated_values,
                        });
                    }
                }
                
                Ok(Value::DataFrame {
                    columns: result_columns,
                })
            }
            _ => Err(InterpreterError::RuntimeError(format!("Unknown DataFrame method: {}", method))),
        }
    }
    
    /// Special handler for `DataFrame` filter method
    fn eval_dataframe_filter_method(&mut self, receiver: &Value, args: &[Expr]) -> Result<Value, InterpreterError> {
        if args.len() != 1 {
            return Err(InterpreterError::RuntimeError("DataFrame.filter() requires exactly 1 argument (condition)".to_string()));
        }
        
        if let Value::DataFrame { columns } = receiver {
            let condition = &args[0];
            
            if columns.is_empty() {
                return Ok(Value::DataFrame { columns: columns.clone() });
            }
            
            let num_rows = columns[0].values.len();
            let mut filtered_rows: Vec<bool> = Vec::new();
            
            // Evaluate the condition for each row
            for row_idx in 0..num_rows {
                let condition_result = self.eval_expr_with_column_context(condition, columns, row_idx);
                
                match condition_result {
                    Ok(Value::Bool(true)) => filtered_rows.push(true),
                    Ok(Value::Bool(false)) => filtered_rows.push(false),
                    Ok(_) => return Err(InterpreterError::RuntimeError("Filter condition must return boolean".to_string())),
                    Err(e) => return Err(e),
                }
            }
            
            // Create new columns with filtered values
            let mut new_columns = Vec::new();
            for col in columns {
                let mut filtered_values = Vec::new();
                for (idx, &keep) in filtered_rows.iter().enumerate() {
                    if keep {
                        if let Some(value) = col.values.get(idx) {
                            filtered_values.push(value.clone());
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
            Err(InterpreterError::RuntimeError("filter method can only be called on DataFrame".to_string()))
        }
    }

    /// Compare two values using a comparison function
    fn compare_values<F>(&self, left: &Value, right: &Value, cmp: F) -> Result<Value, InterpreterError>
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
    fn eval_expr_with_column_context(&mut self, expr: &Expr, columns: &[DataFrameColumn], row_idx: usize) -> Result<Value, InterpreterError> {
        match &expr.kind {
            // Special handling for function calls that might be col() references
            ExprKind::Call { func, args } => {
                if let ExprKind::Identifier(name) = &func.kind {
                    if name == "col" && args.len() == 1 {
                        // This is a col("column_name") call - resolve to actual column value
                        let col_name_expr = &args[0];
                        if let ExprKind::Literal(crate::frontend::ast::Literal::String(col_name)) = &col_name_expr.kind {
                            // Find the column and return the value for this row
                            for col in columns {
                                if col.name == *col_name {
                                    if let Some(value) = col.values.get(row_idx) {
                                        return Ok(value.clone());
                                    }
                                    return Err(InterpreterError::RuntimeError(format!("Row index {} out of bounds for column '{}'", row_idx, col_name)));
                                }
                            }
                            return Err(InterpreterError::RuntimeError(format!("Column '{}' not found", col_name)));
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
                        _ => self.eval_expr(expr) // Use regular evaluation for other operators
                    }
                } else {
                    unreachable!()
                }
            }
            // For all other expressions, use normal evaluation
            _ => self.eval_expr(expr)
        }
    }

    fn eval_dataframe_operation(&mut self, source: &Expr, operation: &crate::frontend::ast::DataFrameOp) -> Result<Value, InterpreterError> {
        let source_value = self.eval_expr(source)?;
        
        if let Value::DataFrame { columns } = source_value {
            match operation {
                crate::frontend::ast::DataFrameOp::Select(column_names) => {
                    // Select specific columns by name
                    let mut selected_columns = Vec::new();
                    
                    for name in column_names {
                        // Find the column
                        let mut found = false;
                        for col in &columns {
                            if col.name == *name {
                                selected_columns.push(col.clone());
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            return Err(InterpreterError::RuntimeError(format!("Column '{}' not found in DataFrame", name)));
                        }
                    }
                    
                    Ok(Value::DataFrame {
                        columns: selected_columns,
                    })
                }
                crate::frontend::ast::DataFrameOp::Filter(condition) => {
                    // Filter rows based on condition
                    if columns.is_empty() {
                        return Ok(Value::DataFrame { columns });
                    }
                    
                    let num_rows = columns[0].values.len();
                    let mut filtered_rows: Vec<bool> = Vec::new();
                    
                    // Evaluate the condition for each row
                    for row_idx in 0..num_rows {
                        // Evaluate condition with column context
                        let condition_result = self.eval_expr_with_column_context(condition, &columns, row_idx);
                        
                        match condition_result {
                            Ok(Value::Bool(true)) => filtered_rows.push(true),
                            Ok(Value::Bool(false)) => filtered_rows.push(false),
                            Ok(_) => return Err(InterpreterError::RuntimeError("Filter condition must return boolean".to_string())),
                            Err(e) => return Err(e),
                        }
                    }
                    
                    // Create new columns with filtered values
                    let mut new_columns = Vec::new();
                    for col in &columns {
                        let mut filtered_values = Vec::new();
                        for (idx, &keep) in filtered_rows.iter().enumerate() {
                            if keep {
                                if let Some(value) = col.values.get(idx) {
                                    filtered_values.push(value.clone());
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
                }
                crate::frontend::ast::DataFrameOp::GroupBy(group_columns) => {
                    // Group by one or more columns 
                    // Parser limitation: use first column as default when no columns provided
                    let group_column = if group_columns.is_empty() {
                        if columns.is_empty() {
                            return Err(InterpreterError::RuntimeError("Cannot group by empty DataFrame".to_string()));
                        }
                        &columns[0].name // Default to first column
                    } else {
                        &group_columns[0]
                    };
                    
                    // Find the group column
                    let group_col = columns.iter().find(|col| col.name == *group_column);
                    if group_col.is_none() {
                        return Err(InterpreterError::RuntimeError(format!("Group column '{}' not found in DataFrame", group_column)));
                    }
                    let group_col = group_col
                        .expect("group_col existence verified by is_none() check above");
                    
                    // Group rows by the group column values
                    use std::collections::HashMap;
                    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
                    
                    for (row_idx, value) in group_col.values.iter().enumerate() {
                        let key = match value {
                            Value::String(s) => s.to_string(),
                            Value::Integer(i) => i.to_string(),
                            Value::Float(f) => f.to_string(),
                            Value::Bool(b) => b.to_string(),
                            _ => "null".to_string(),
                        };
                        groups.entry(key).or_default().push(row_idx);
                    }
                    
                    // Create result columns: group column + aggregated numeric columns
                    let mut result_columns = Vec::new();
                    
                    // Group column (unique values)
                    let mut group_values = Vec::new();
                    for key in groups.keys() {
                        group_values.push(Value::from_string(key.clone()));
                    }
                    result_columns.push(DataFrameColumn {
                        name: group_column.to_string(),
                        values: group_values,
                    });
                    
                    // Aggregate numeric columns (sum by default for now)
                    for col in &columns {
                        if col.name != *group_column {
                            let mut aggregated_values = Vec::new();
                            for indices in groups.values() {
                                let mut sum = 0.0;
                                for &idx in indices {
                                    if let Some(value) = col.values.get(idx) {
                                        match value {
                                            Value::Integer(i) => sum += *i as f64,
                                            Value::Float(f) => sum += f,
                                            _ => {} // Skip non-numeric values
                                        }
                                    }
                                }
                                // Return as integer if it's a whole number, otherwise float
                                if sum.fract() == 0.0 {
                                    aggregated_values.push(Value::Integer(sum as i64));
                                } else {
                                    aggregated_values.push(Value::Float(sum));
                                }
                            }
                            result_columns.push(DataFrameColumn {
                                name: format!("{}_sum", col.name),
                                values: aggregated_values,
                            });
                        }
                    }
                    
                    Ok(Value::DataFrame {
                        columns: result_columns,
                    })
                }
                _ => Err(InterpreterError::RuntimeError("DataFrameOperation not yet implemented".to_string())),
            }
        } else {
            Err(InterpreterError::RuntimeError("DataFrameOperation can only be applied to DataFrame values".to_string()))
        }
    }
    
    /// Evaluate string interpolation
    fn eval_string_interpolation(&mut self, parts: &[StringPart]) -> Result<Value, InterpreterError> {
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
                    Value::Integer(i) => return format!("{:.precision$}", *i as f64, precision = precision),
                    _ => {}
                }
            }
        }
        // Default formatting if spec doesn't match or isn't supported
        value.to_string()
    }
    
    /// Evaluate function definition
    fn eval_function(&mut self, name: &str, params: &[Param], body: &Expr) -> Result<Value, InterpreterError> {
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
        let param_names: Vec<String> = params
            .iter()
            .map(crate::frontend::ast::Param::name)
            .collect();

        let closure = Value::Closure {
            params: param_names,
            body: Rc::new(body.clone()),
            env: Rc::new(self.current_env().clone()),
        };

        Ok(closure)
    }
    
    /// Evaluate function call
    fn eval_function_call(&mut self, func: &Expr, args: &[Expr]) -> Result<Value, InterpreterError> {
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
    use crate::frontend::ast::Span;

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

    #[test]
    fn test_division_by_zero() {
        let mut interp = Interpreter::new();

        assert!(interp.push(Value::from_i64(10)).is_ok());
        assert!(interp.push(Value::from_i64(0)).is_ok());

        let result = interp.binary_op(BinaryOp::Div);
        assert!(matches!(result, Err(InterpreterError::DivisionByZero)));
    }

    #[test]
    fn test_comparison() {
        let mut interp = Interpreter::new();

        // Test 5 < 10
        assert!(interp.push(Value::from_i64(5)).is_ok());
        assert!(interp.push(Value::from_i64(10)).is_ok());
        assert!(interp.binary_op(BinaryOp::Lt).is_ok());

        let result = interp.pop().expect("Stack should not be empty");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_stack_operations() {
        let mut interp = Interpreter::new();

        let val1 = Value::from_i64(42);
        let val2 = Value::from_bool(true);

        assert!(interp.push(val1.clone()).is_ok());
        assert!(interp.push(val2.clone()).is_ok());

        assert_eq!(interp.peek(0).expect("Should peek at top"), val2);
        assert_eq!(interp.peek(1).expect("Should peek at second"), val1);

        assert_eq!(interp.pop().expect("Should pop top"), val2);
        assert_eq!(interp.pop().expect("Should pop second"), val1);
    }

    #[test]
    fn test_truthiness() {
        assert!(Value::from_i64(42).is_truthy());
        assert!(Value::from_bool(true).is_truthy());
        assert!(!Value::from_bool(false).is_truthy());
        assert!(!Value::nil().is_truthy());
        assert!(Value::from_f64(std::f64::consts::PI).is_truthy());
        assert!(Value::from_f64(0.0).is_truthy()); // 0.0 is truthy in Ruchy
        assert!(Value::from_string("hello".to_string()).is_truthy());
    }

    // AST Walker tests

    #[test]
    fn test_eval_literal() {
        let mut interp = Interpreter::new();

        // Test integer literal
        let int_expr = Expr::new(ExprKind::Literal(Literal::Integer(42)), Span::new(0, 2));
        let result = interp
            .eval_expr(&int_expr)
            .expect("Should evaluate integer");
        assert_eq!(result, Value::Integer(42));

        // Test string literal
        let str_expr = Expr::new(
            ExprKind::Literal(Literal::String("hello".to_string())),
            Span::new(0, 7),
        );
        let result = interp.eval_expr(&str_expr).expect("Should evaluate string");
        assert_eq!(result.type_name(), "string");

        // Test boolean literal
        let bool_expr = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::new(0, 4));
        let result = interp
            .eval_expr(&bool_expr)
            .expect("Should evaluate boolean");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_binary_arithmetic() {
        let mut interp = Interpreter::new();

        // Test 5 + 3 = 8
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(5)),
            Span::new(0, 1),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(3)),
            Span::new(4, 5),
        ));
        let add_expr = Expr::new(
            ExprKind::Binary {
                left,
                op: AstBinaryOp::Add,
                right,
            },
            Span::new(0, 5),
        );

        let result = interp
            .eval_expr(&add_expr)
            .expect("Should evaluate addition");
        assert_eq!(result, Value::Integer(8));
    }

    #[test]
    fn test_eval_binary_comparison() {
        let mut interp = Interpreter::new();

        // Test 5 < 10 = true
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(5)),
            Span::new(0, 1),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(10)),
            Span::new(4, 6),
        ));
        let cmp_expr = Expr::new(
            ExprKind::Binary {
                left,
                op: AstBinaryOp::Less,
                right,
            },
            Span::new(0, 6),
        );

        let result = interp
            .eval_expr(&cmp_expr)
            .expect("Should evaluate comparison");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_unary_operations() {
        let mut interp = Interpreter::new();

        // Test -42 = -42
        let operand = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(42)),
            Span::new(1, 3),
        ));
        let neg_expr = Expr::new(
            ExprKind::Unary {
                op: crate::frontend::ast::UnaryOp::Negate,
                operand,
            },
            Span::new(0, 3),
        );

        let result = interp
            .eval_expr(&neg_expr)
            .expect("Should evaluate negation");
        assert_eq!(result, Value::Integer(-42));

        // Test !true = false
        let operand = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(1, 5),
        ));
        let not_expr = Expr::new(
            ExprKind::Unary {
                op: crate::frontend::ast::UnaryOp::Not,
                operand,
            },
            Span::new(0, 5),
        );

        let result = interp
            .eval_expr(&not_expr)
            .expect("Should evaluate logical not");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_eval_if_expression() {
        let mut interp = Interpreter::new();

        // Test if true then 1 else 2 = 1
        let condition = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(3, 7),
        ));
        let then_branch = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(1)),
            Span::new(13, 14),
        ));
        let else_branch = Some(Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(2)),
            Span::new(20, 21),
        )));

        let if_expr = Expr::new(
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            },
            Span::new(0, 21),
        );

        let result = interp
            .eval_expr(&if_expr)
            .expect("Should evaluate if expression");
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_eval_let_expression() {
        let mut interp = Interpreter::new();

        // Test let x = 5 in x + 2 = 7
        let value = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(5)),
            Span::new(8, 9),
        ));

        let left = Box::new(Expr::new(
            ExprKind::Identifier("x".to_string()),
            Span::new(13, 14),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(2)),
            Span::new(17, 18),
        ));
        let body = Box::new(Expr::new(
            ExprKind::Binary {
                left,
                op: AstBinaryOp::Add,
                right,
            },
            Span::new(13, 18),
        ));

        let let_expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value,
                body,
                is_mutable: false,
            },
            Span::new(0, 18),
        );

        let result = interp
            .eval_expr(&let_expr)
            .expect("Should evaluate let expression");
        assert_eq!(result, Value::Integer(7));
    }

    #[test]
    fn test_eval_logical_operators() {
        let mut interp = Interpreter::new();

        // Test true && false = false (short-circuit)
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(0, 4),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            Span::new(8, 13),
        ));
        let and_expr = Expr::new(
            ExprKind::Binary {
                left,
                op: AstBinaryOp::And,
                right,
            },
            Span::new(0, 13),
        );

        let result = interp
            .eval_expr(&and_expr)
            .expect("Should evaluate logical AND");
        assert_eq!(result, Value::Bool(false));

        // Test false || true = true (short-circuit)
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            Span::new(0, 5),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(9, 13),
        ));
        let or_expr = Expr::new(
            ExprKind::Binary {
                left,
                op: AstBinaryOp::Or,
                right,
            },
            Span::new(0, 13),
        );

        let result = interp
            .eval_expr(&or_expr)
            .expect("Should evaluate logical OR");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_parser_integration() {
        let mut interp = Interpreter::new();

        // Test simple arithmetic: 2 + 3 * 4 = 14
        let result = interp
            .eval_string("2 + 3 * 4")
            .expect("Should parse and evaluate");
        assert_eq!(result, Value::Integer(14));

        // Test comparison: 5 > 3 = true
        let result = interp
            .eval_string("5 > 3")
            .expect("Should parse and evaluate");
        assert_eq!(result, Value::Bool(true));

        // Test boolean literals: true && false = false
        let result = interp
            .eval_string("true && false")
            .expect("Should parse and evaluate");
        assert_eq!(result, Value::Bool(false));

        // Test unary operations: -42 = -42
        let result = interp
            .eval_string("-42")
            .expect("Should parse and evaluate");
        assert_eq!(result, Value::Integer(-42));

        // Test string literals
        let result = interp
            .eval_string(r#""hello""#)
            .expect("Should parse and evaluate");
        assert_eq!(result.type_name(), "string");
    }

    #[test]
    fn test_eval_lambda() {
        use crate::frontend::ast::{Pattern, Type, TypeKind};
        let mut interp = Interpreter::new();

        // Test lambda: |x| x + 1
        let param = Param {
            pattern: Pattern::Identifier("x".to_string()),
            ty: Type {
                kind: TypeKind::Named("i32".to_string()),
                span: Span::new(0, 3),
            },
            span: Span::new(0, 1),
            is_mutable: false,
            default_value: None,
        };

        let left = Box::new(Expr::new(
            ExprKind::Identifier("x".to_string()),
            Span::new(0, 1),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(1)),
            Span::new(4, 5),
        ));
        let body = Box::new(Expr::new(
            ExprKind::Binary {
                left,
                op: AstBinaryOp::Add,
                right,
            },
            Span::new(0, 5),
        ));

        let lambda_expr = Expr::new(
            ExprKind::Lambda {
                params: vec![param],
                body,
            },
            Span::new(0, 10),
        );

        let result = interp
            .eval_expr(&lambda_expr)
            .expect("Should evaluate lambda");
        assert_eq!(result.type_name(), "function");
    }

    #[test]
    fn test_eval_function_call() {
        use crate::frontend::ast::{Pattern, Type, TypeKind};
        let mut interp = Interpreter::new();

        // Create lambda: |x| x + 1
        let param = Param {
            pattern: Pattern::Identifier("x".to_string()),
            ty: Type {
                kind: TypeKind::Named("i32".to_string()),
                span: Span::new(0, 3),
            },
            span: Span::new(0, 1),
            is_mutable: false,
            default_value: None,
        };

        let left = Box::new(Expr::new(
            ExprKind::Identifier("x".to_string()),
            Span::new(0, 1),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(1)),
            Span::new(4, 5),
        ));
        let body = Box::new(Expr::new(
            ExprKind::Binary {
                left,
                op: AstBinaryOp::Add,
                right,
            },
            Span::new(0, 5),
        ));

        let lambda_expr = Expr::new(
            ExprKind::Lambda {
                params: vec![param],
                body,
            },
            Span::new(0, 10),
        );

        // Call lambda with argument 5: (|x| x + 1)(5) = 6
        let call_expr = Expr::new(
            ExprKind::Call {
                func: Box::new(lambda_expr),
                args: vec![Expr::new(
                    ExprKind::Literal(Literal::Integer(5)),
                    Span::new(0, 1),
                )],
            },
            Span::new(0, 15),
        );

        let result = interp
            .eval_expr(&call_expr)
            .expect("Should evaluate function call");
        assert_eq!(result, Value::Integer(6));
    }

    #[test]
    fn test_eval_function_definition() {
        use crate::frontend::ast::{Pattern, Type, TypeKind};
        let mut interp = Interpreter::new();

        // Create function: fn add_one(x) = x + 1
        let param = Param {
            pattern: Pattern::Identifier("x".to_string()),
            ty: Type {
                kind: TypeKind::Named("i32".to_string()),
                span: Span::new(0, 3),
            },
            span: Span::new(0, 1),
            is_mutable: false,
            default_value: None,
        };

        let left = Box::new(Expr::new(
            ExprKind::Identifier("x".to_string()),
            Span::new(0, 1),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(1)),
            Span::new(4, 5),
        ));
        let body = Box::new(Expr::new(
            ExprKind::Binary {
                left,
                op: AstBinaryOp::Add,
                right,
            },
            Span::new(0, 5),
        ));

        let func_expr = Expr::new(
            ExprKind::Function {
                name: "add_one".to_string(),
                type_params: vec![],
                params: vec![param],
                return_type: None,
                body,
                is_async: false,
                is_pub: false,
            },
            Span::new(0, 20),
        );

        let result = interp
            .eval_expr(&func_expr)
            .expect("Should evaluate function");
        assert_eq!(result.type_name(), "function");

        // Verify function is bound in environment
        let bound_func = interp
            .lookup_variable("add_one")
            .expect("Function should be bound");
        assert_eq!(bound_func.type_name(), "function");
    }

    #[test]
    fn test_eval_recursive_function() {
        use crate::frontend::ast::{Pattern, Type, TypeKind};
        let mut interp = Interpreter::new();

        // Create recursive factorial function
        let param = Param {
            pattern: Pattern::Identifier("n".to_string()),
            ty: Type {
                kind: TypeKind::Named("i32".to_string()),
                span: Span::new(0, 3),
            },
            span: Span::new(0, 1),
            is_mutable: false,
            default_value: None,
        };

        // if n <= 1 then 1 else n * factorial(n - 1)
        let n_id = Expr::new(ExprKind::Identifier("n".to_string()), Span::new(0, 1));
        let one = Expr::new(ExprKind::Literal(Literal::Integer(1)), Span::new(0, 1));

        let condition = Box::new(Expr::new(
            ExprKind::Binary {
                left: Box::new(n_id.clone()),
                op: AstBinaryOp::LessEqual,
                right: Box::new(one.clone()),
            },
            Span::new(0, 6),
        ));

        let then_branch = Box::new(one.clone());

        // n * factorial(n - 1)
        let n_minus_1 = Expr::new(
            ExprKind::Binary {
                left: Box::new(n_id.clone()),
                op: AstBinaryOp::Subtract,
                right: Box::new(one),
            },
            Span::new(0, 5),
        );

        let recursive_call = Expr::new(
            ExprKind::Call {
                func: Box::new(Expr::new(
                    ExprKind::Identifier("factorial".to_string()),
                    Span::new(0, 9),
                )),
                args: vec![n_minus_1],
            },
            Span::new(0, 15),
        );

        let else_branch = Some(Box::new(Expr::new(
            ExprKind::Binary {
                left: Box::new(n_id),
                op: AstBinaryOp::Multiply,
                right: Box::new(recursive_call),
            },
            Span::new(0, 20),
        )));

        let body = Box::new(Expr::new(
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            },
            Span::new(0, 25),
        ));

        let factorial_expr = Expr::new(
            ExprKind::Function {
                name: "factorial".to_string(),
                type_params: vec![],
                params: vec![param],
                return_type: None,
                body,
                is_async: false,
                is_pub: false,
            },
            Span::new(0, 30),
        );

        // Define factorial function
        let result = interp
            .eval_expr(&factorial_expr)
            .expect("Should evaluate factorial function");
        assert_eq!(result.type_name(), "function");

        // Test factorial(5) = 120
        let call_expr = Expr::new(
            ExprKind::Call {
                func: Box::new(Expr::new(
                    ExprKind::Identifier("factorial".to_string()),
                    Span::new(0, 9),
                )),
                args: vec![Expr::new(
                    ExprKind::Literal(Literal::Integer(5)),
                    Span::new(0, 1),
                )],
            },
            Span::new(0, 15),
        );

        let result = interp
            .eval_expr(&call_expr)
            .expect("Should evaluate factorial(5)");
        assert_eq!(result, Value::Integer(120));
    }

    #[test]
    fn test_function_closure() {
        use crate::frontend::ast::{Pattern, Type, TypeKind};
        let mut interp = Interpreter::new();

        // Test closure: let x = 10 in |y| x + y
        let x_val = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(10)),
            Span::new(8, 10),
        ));

        let param = Param {
            pattern: Pattern::Identifier("y".to_string()),
            ty: Type {
                kind: TypeKind::Named("i32".to_string()),
                span: Span::new(0, 3),
            },
            span: Span::new(0, 1),
            is_mutable: false,
            default_value: None,
        };

        let left = Box::new(Expr::new(
            ExprKind::Identifier("x".to_string()),
            Span::new(0, 1),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Identifier("y".to_string()),
            Span::new(4, 5),
        ));
        let lambda_body = Box::new(Expr::new(
            ExprKind::Binary {
                left,
                op: AstBinaryOp::Add,
                right,
            },
            Span::new(0, 5),
        ));

        let lambda = Expr::new(
            ExprKind::Lambda {
                params: vec![param],
                body: lambda_body,
            },
            Span::new(14, 24),
        );

        let let_body = Box::new(lambda);

        let let_expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: x_val,
                body: let_body,
                is_mutable: false,
            },
            Span::new(0, 24),
        );

        let closure = interp
            .eval_expr(&let_expr)
            .expect("Should evaluate closure");
        assert_eq!(closure.type_name(), "function");

        // Call closure with argument 5: (|y| x + y)(5) = 15 (x = 10)
        let call_expr = Expr::new(
            ExprKind::Call {
                func: Box::new(let_expr), // Re-create the closure
                args: vec![Expr::new(
                    ExprKind::Literal(Literal::Integer(5)),
                    Span::new(0, 1),
                )],
            },
            Span::new(0, 30),
        );

        // Note: This test demonstrates lexical scoping where the closure captures 'x'
        let result = interp
            .eval_expr(&call_expr)
            .expect("Should evaluate closure call");
        assert_eq!(result, Value::Integer(15));
    }

    #[test]
    fn test_inline_cache_string_methods() {
        let mut interp = Interpreter::new();
        let test_string = Value::String(Rc::new("Hello World".to_string()));

        // Test string.len() with caching
        let result1 = interp
            .get_field_cached(&test_string, "len")
            .expect("Should get string length");
        assert_eq!(result1, Value::Integer(11));

        let result2 = interp
            .get_field_cached(&test_string, "len")
            .expect("Should get cached result");
        assert_eq!(result2, Value::Integer(11));

        // Verify cache hit occurred
        let stats = interp.get_cache_stats();
        let cache_key = format!("{:?}::len", test_string.type_id());
        assert!(stats.get(&cache_key).unwrap_or(&0.0) > &0.0);

        // Test other string methods
        let upper_result = interp
            .get_field_cached(&test_string, "to_upper")
            .expect("Should get uppercase");
        assert_eq!(
            upper_result,
            Value::String(Rc::new("HELLO WORLD".to_string()))
        );

        let trim_result = interp
            .get_field_cached(&Value::String(Rc::new("  test  ".to_string())), "trim")
            .expect("Should trim string");
        assert_eq!(trim_result, Value::String(Rc::new("test".to_string())));
    }

    #[test]
    fn test_inline_cache_array_methods() {
        let mut interp = Interpreter::new();
        let test_array = Value::Array(Rc::new(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));

        // Test array.len() with caching
        let result1 = interp
            .get_field_cached(&test_array, "len")
            .expect("Should get array length");
        assert_eq!(result1, Value::Integer(3));

        let result2 = interp
            .get_field_cached(&test_array, "len")
            .expect("Should get cached result");
        assert_eq!(result2, Value::Integer(3));

        // Test first and last
        let first_result = interp
            .get_field_cached(&test_array, "first")
            .expect("Should get first element");
        assert_eq!(first_result, Value::Integer(1));

        let last_result = interp
            .get_field_cached(&test_array, "last")
            .expect("Should get last element");
        assert_eq!(last_result, Value::Integer(3));

        // Test empty array (use fresh interpreter to avoid cache pollution)
        let mut fresh_interp = Interpreter::new();
        let empty_array = Value::Array(Rc::new(vec![]));
        let first_err = fresh_interp.get_field_cached(&empty_array, "first");
        assert!(first_err.is_err());
    }

    #[test]
    fn test_inline_cache_polymorphic() {
        let mut interp = Interpreter::new();

        // Test polymorphic caching with different types calling same method
        let string_val = Value::String(Rc::new("test".to_string()));
        let array_val = Value::Array(Rc::new(vec![Value::Integer(1), Value::Integer(2)]));

        // Both call len() method
        let string_len = interp
            .get_field_cached(&string_val, "len")
            .expect("Should get string length");
        assert_eq!(string_len, Value::Integer(4));

        let array_len = interp
            .get_field_cached(&array_val, "len")
            .expect("Should get array length");
        assert_eq!(array_len, Value::Integer(2));

        // Both should have separate cache entries
        let stats = interp.get_cache_stats();
        assert_eq!(stats.len(), 2); // Two different cache keys
    }

    #[test]
    fn test_inline_cache_type_method() {
        let mut interp = Interpreter::new();

        // Test the universal 'type' method
        let int_val = Value::Integer(42);
        let string_val = Value::String(Rc::new("test".to_string()));
        let bool_val = Value::Bool(true);

        let int_type = interp
            .get_field_cached(&int_val, "type")
            .expect("Should get int type");
        assert_eq!(int_type, Value::String(Rc::new("integer".to_string())));

        let string_type = interp
            .get_field_cached(&string_val, "type")
            .expect("Should get string type");
        assert_eq!(string_type, Value::String(Rc::new("string".to_string())));

        let bool_type = interp
            .get_field_cached(&bool_val, "type")
            .expect("Should get bool type");
        assert_eq!(bool_type, Value::String(Rc::new("boolean".to_string())));
    }

    #[test]
    fn test_inline_cache_miss_handling() {
        let mut interp = Interpreter::new();
        let test_val = Value::Integer(42);

        // Test accessing non-existent field
        let result = interp.get_field_cached(&test_val, "non_existent");
        assert!(result.is_err());

        // Test that error doesn't get cached (cache should be empty)
        let stats = interp.get_cache_stats();
        assert!(stats.is_empty());
    }

    #[test]
    fn test_cache_state_transitions() {
        let mut interp = Interpreter::new();

        // Create multiple values of same type for same field
        let vals = [
            Value::String(Rc::new("test1".to_string())),
            Value::String(Rc::new("test2".to_string())),
            Value::String(Rc::new("test3".to_string())),
        ];

        // Access same field multiple times to test cache evolution
        for val in &vals {
            let _ = interp
                .get_field_cached(val, "len")
                .expect("Should get length");
        }

        // Verify caching occurred
        let stats = interp.get_cache_stats();
        assert!(!stats.is_empty());

        // Clear caches and verify
        interp.clear_caches();
        let stats_after = interp.get_cache_stats();
        assert!(stats_after.is_empty());
    }

    #[test]
    fn test_type_feedback_binary_operations() {
        use crate::frontend::ast::Span;
        let mut interp = Interpreter::new();

        // Create binary operation: 42 + 10
        let left = Expr::new(ExprKind::Literal(Literal::Integer(42)), Span::new(0, 2));
        let right = Expr::new(ExprKind::Literal(Literal::Integer(10)), Span::new(5, 7));
        let binary_expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: AstBinaryOp::Add,
                right: Box::new(right),
            },
            Span::new(0, 7),
        );

        // Evaluate the expression multiple times to collect feedback
        for _ in 0..15 {
            let result = interp
                .eval_expr(&binary_expr)
                .expect("Should evaluate binary operation");
            assert_eq!(result, Value::Integer(52));
        }

        // Check type feedback statistics
        let stats = interp.get_type_feedback_stats();
        assert_eq!(stats.total_operation_sites, 1);
        assert_eq!(stats.monomorphic_operation_sites, 1);
        assert_eq!(stats.total_samples, 15);

        // Check specialization candidates
        let candidates = interp.get_specialization_candidates();
        assert!(!candidates.is_empty());
        assert!((candidates[0].confidence - 1.0).abs() < f64::EPSILON); // Monomorphic operation
    }

    #[test]
    fn test_type_feedback_variable_assignments() {
        use crate::frontend::ast::Span;
        let mut interp = Interpreter::new();

        // Create let binding: let x = 42 in x
        let value_expr = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(42)),
            Span::new(8, 10),
        ));
        let body_expr = Box::new(Expr::new(
            ExprKind::Identifier("x".to_string()),
            Span::new(14, 15),
        ));

        let let_expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: value_expr,
                body: body_expr,
                is_mutable: false,
            },
            Span::new(0, 15),
        );

        // Evaluate the expression
        let result = interp
            .eval_expr(&let_expr)
            .expect("Should evaluate let expression");
        assert_eq!(result, Value::Integer(42));

        // Check type feedback statistics
        let stats = interp.get_type_feedback_stats();
        assert_eq!(stats.total_variables, 1);
        assert_eq!(stats.stable_variables, 1);

        // Check specialization candidates for stable variable
        let candidates = interp.get_specialization_candidates();
        let variable_candidates: Vec<_> = candidates
            .iter()
            .filter(|c| matches!(c.kind, SpecializationKind::Variable { .. }))
            .collect();
        assert!(!variable_candidates.is_empty());
    }

    #[test]
    fn test_type_feedback_function_calls() {
        use crate::frontend::ast::{Param, Pattern, Span, Type, TypeKind};
        let mut interp = Interpreter::new();

        // Create function: fn double(x) = x + x
        let param = Param {
            pattern: Pattern::Identifier("x".to_string()),
            ty: Type {
                kind: TypeKind::Named("i32".to_string()),
                span: Span::new(0, 3),
            },
            span: Span::new(0, 1),
            is_mutable: false,
            default_value: None,
        };

        let left_body = Box::new(Expr::new(
            ExprKind::Identifier("x".to_string()),
            Span::new(0, 1),
        ));
        let right_body = Box::new(Expr::new(
            ExprKind::Identifier("x".to_string()),
            Span::new(4, 5),
        ));
        let func_body = Box::new(Expr::new(
            ExprKind::Binary {
                left: left_body,
                op: AstBinaryOp::Add,
                right: right_body,
            },
            Span::new(0, 5),
        ));

        let func_expr = Expr::new(
            ExprKind::Function {
                name: "double".to_string(),
                type_params: vec![],
                params: vec![param],
                body: func_body,
                return_type: None,
                is_async: false,
                is_pub: false,
            },
            Span::new(0, 20),
        );

        // Define the function
        let _func = interp
            .eval_expr(&func_expr)
            .expect("Should define function");

        // Create function call: double(21)
        let func_ref = Box::new(Expr::new(
            ExprKind::Identifier("double".to_string()),
            Span::new(0, 6),
        ));
        let arg = Expr::new(ExprKind::Literal(Literal::Integer(21)), Span::new(7, 9));
        let call_expr = Expr::new(
            ExprKind::Call {
                func: func_ref,
                args: vec![arg],
            },
            Span::new(0, 10),
        );

        // Call the function multiple times
        for _ in 0..10 {
            let result = interp.eval_expr(&call_expr).expect("Should call function");
            assert_eq!(result, Value::Integer(42));
        }

        // Check type feedback statistics
        let stats = interp.get_type_feedback_stats();
        assert!(stats.total_call_sites > 0);
        assert!(stats.monomorphic_call_sites > 0);

        // Check specialization candidates for function calls
        let candidates = interp.get_specialization_candidates();
        let call_candidates: Vec<_> = candidates
            .iter()
            .filter(|c| matches!(c.kind, SpecializationKind::FunctionCall { .. }))
            .collect();
        assert!(!call_candidates.is_empty());
    }

    #[test]
    fn test_type_feedback_polymorphic_detection() {
        use crate::frontend::ast::Span;
        let mut interp = Interpreter::new();

        // Create integer addition
        let int_expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(1)),
                    Span::new(0, 1),
                )),
                op: AstBinaryOp::Add,
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(2)),
                    Span::new(4, 5),
                )),
            },
            Span::new(0, 5),
        );

        // Create float addition (different site)
        let float_expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Float(1.5)),
                    Span::new(10, 13),
                )),
                op: AstBinaryOp::Add,
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Float(2.5)),
                    Span::new(16, 19),
                )),
            },
            Span::new(10, 19),
        );

        // Evaluate both expressions multiple times
        for _ in 0..12 {
            let _ = interp
                .eval_expr(&int_expr)
                .expect("Should evaluate int addition");
            let _ = interp
                .eval_expr(&float_expr)
                .expect("Should evaluate float addition");
        }

        // Check that we have multiple operation sites
        let stats = interp.get_type_feedback_stats();
        assert_eq!(stats.total_operation_sites, 2);
        assert_eq!(stats.monomorphic_operation_sites, 2); // Both should be monomorphic
        assert_eq!(stats.total_samples, 24); // 12 * 2 operations

        // Both should be candidates for specialization
        let candidates = interp.get_specialization_candidates();
        let op_candidates: Vec<_> = candidates
            .iter()
            .filter(|c| matches!(c.kind, SpecializationKind::BinaryOperation { .. }))
            .collect();
        assert_eq!(op_candidates.len(), 2);
    }

    #[test]
    fn test_type_feedback_clear() {
        use crate::frontend::ast::Span;
        let mut interp = Interpreter::new();

        // Create and evaluate a simple expression
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(1)),
                    Span::new(0, 1),
                )),
                op: AstBinaryOp::Add,
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(1)),
                    Span::new(4, 5),
                )),
            },
            Span::new(0, 5),
        );

        let _ = interp.eval_expr(&expr).expect("Should evaluate");

        // Verify feedback was collected
        let stats_before = interp.get_type_feedback_stats();
        assert!(stats_before.total_samples > 0);

        // Clear feedback and verify
        interp.clear_type_feedback();
        let stats_after = interp.get_type_feedback_stats();
        assert_eq!(stats_after.total_samples, 0);
        assert_eq!(stats_after.total_operation_sites, 0);
    }

    #[test]
    fn test_gc_basic_tracking() {
        let mut interp = Interpreter::new();

        // Create some values to track
        let values = vec![
            Value::Integer(42),
            Value::String(Rc::new("hello".to_string())),
            Value::Array(Rc::new(vec![Value::Integer(1), Value::Integer(2)])),
        ];

        // Track them in GC
        for value in values {
            interp.gc_track(value);
        }

        // Check GC info
        let info = interp.gc_info();
        assert_eq!(info.total_objects, 3);
        assert!(info.allocated_bytes > 0);
        assert_eq!(info.collections_performed, 0);
    }

    #[test]
    fn test_gc_collection() {
        let mut interp = Interpreter::new();

        // Disable auto-collection for manual testing
        interp.gc_set_auto_collect(false);

        // Track several objects
        for i in 0..10 {
            let value = Value::Integer(i);
            interp.gc_track(value);
        }

        let info_before = interp.gc_info();
        assert_eq!(info_before.total_objects, 10);

        // Force garbage collection
        let stats = interp.gc_collect();

        // Since we treat all objects as roots conservatively, none should be collected
        assert_eq!(stats.objects_collected, 0);
        assert_eq!(stats.objects_after, 10);

        let info_after = interp.gc_info();
        assert_eq!(info_after.collections_performed, 1);
    }

    #[test]
    fn test_gc_auto_collection() {
        let mut interp = Interpreter::new();

        // Set a very low threshold to trigger auto-collection
        interp.gc_set_threshold(100);
        interp.gc_set_auto_collect(true);

        // Track a large object to trigger collection
        let large_string = "x".repeat(200);
        let value = Value::String(Rc::new(large_string));
        interp.gc_track(value);

        // Auto-collection should have been triggered
        let info = interp.gc_info();
        assert!(info.collections_performed > 0);
    }

    #[test]
    fn test_gc_allocation_helpers() {
        let mut interp = Interpreter::new();

        // Test GC allocation helpers
        let array = interp.gc_alloc_array(vec![Value::Integer(1), Value::Integer(2)]);
        let string = interp.gc_alloc_string("test".to_string());

        // Both should be tracked
        let info = interp.gc_info();
        assert_eq!(info.total_objects, 2);

        // Verify the values are correct
        match array {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
                assert_eq!(arr[0], Value::Integer(1));
                assert_eq!(arr[1], Value::Integer(2));
            }
            _ => panic!("Expected array"),
        }

        match string {
            Value::String(s) => {
                assert_eq!(s.as_ref(), "test");
            }
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_gc_size_estimation() {
        let gc = ConservativeGC::new();

        // Test size estimation for different value types
        let int_size = gc.estimate_object_size(&Value::Integer(42));
        let float_size = gc.estimate_object_size(&Value::Float(3.14));
        let bool_size = gc.estimate_object_size(&Value::Bool(true));
        let nil_size = gc.estimate_object_size(&Value::Nil);

        assert_eq!(int_size, 8);
        assert_eq!(float_size, 8);
        assert_eq!(bool_size, 1);
        assert_eq!(nil_size, 0);

        // Test string size estimation
        let string_val = Value::String(Rc::new("hello".to_string()));
        let string_size = gc.estimate_object_size(&string_val);
        assert_eq!(string_size, 5 + 24); // content + overhead

        // Test array size estimation
        let array_val = Value::Array(Rc::new(vec![Value::Integer(1), Value::Integer(2)]));
        let array_size = gc.estimate_object_size(&array_val);
        assert_eq!(array_size, 24 + 8 + 8); // overhead + 2 integers
    }

    #[test]
    fn test_gc_threshold_management() {
        let mut interp = Interpreter::new();

        // Test threshold setting
        interp.gc_set_threshold(2048);
        let info = interp.gc_info();
        assert_eq!(info.collection_threshold, 2048);

        // Test auto-collect setting
        interp.gc_set_auto_collect(false);
        let info = interp.gc_info();
        assert!(!info.auto_collect_enabled);

        interp.gc_set_auto_collect(true);
        let info = interp.gc_info();
        assert!(info.auto_collect_enabled);
    }

    #[test]
    fn test_gc_clear() {
        let mut interp = Interpreter::new();

        // Track some objects
        for i in 0..5 {
            interp.gc_track(Value::Integer(i));
        }

        let info_before = interp.gc_info();
        assert_eq!(info_before.total_objects, 5);
        assert!(info_before.allocated_bytes > 0);

        // Clear GC
        interp.gc_clear();

        let info_after = interp.gc_info();
        assert_eq!(info_after.total_objects, 0);
        assert_eq!(info_after.allocated_bytes, 0);
    }

    #[test]
    fn test_gc_stats_consistency() {
        let mut interp = Interpreter::new();

        // Track objects and get initial stats
        for i in 0..3 {
            interp.gc_track(Value::Integer(i));
        }

        let stats = interp.gc_stats();
        let info = interp.gc_info();

        // Stats and info should be consistent
        assert_eq!(stats.objects_before, info.total_objects);
        assert_eq!(stats.objects_after, info.total_objects);
        assert_eq!(stats.bytes_before, info.allocated_bytes);
        assert_eq!(stats.bytes_after, info.allocated_bytes);
        assert_eq!(stats.objects_collected, 0);
    }

    // Direct-threaded interpreter tests

    #[test]
    fn test_direct_threaded_creation() {
        let interp = DirectThreadedInterpreter::new();
        assert_eq!(interp.instruction_count(), 0);
        assert_eq!(interp.constants_count(), 0);
    }

    #[test]
    fn test_direct_threaded_constants() {
        let mut interp = DirectThreadedInterpreter::new();

        let int_idx = interp.add_constant(Value::Integer(42));
        let float_idx = interp.add_constant(Value::Float(3.14));
        let string_idx = interp.add_constant(Value::String(Rc::new("hello".to_string())));

        assert_eq!(int_idx, 0);
        assert_eq!(float_idx, 1);
        assert_eq!(string_idx, 2);
        assert_eq!(interp.constants_count(), 3);
    }

    #[test]
    fn test_direct_threaded_instruction_stream() {
        let mut interp = DirectThreadedInterpreter::new();

        // Add some constants
        let const_idx = interp.add_constant(Value::Integer(42));

        // Add instructions
        interp.add_instruction(op_load_const, const_idx);
        interp.add_instruction(op_load_nil, 0);

        assert_eq!(interp.instruction_count(), 2);
    }

    #[test]
    fn test_direct_threaded_literal_compilation() {
        use crate::frontend::ast::{Expr, ExprKind};

        let mut interp = DirectThreadedInterpreter::new();

        // Compile integer literal
        let int_ast = Expr::new(
            ExprKind::Literal(Literal::Integer(42)),
            crate::frontend::ast::Span::new(0, 0),
        );
        let result = interp.compile(&int_ast);
        assert!(result.is_ok());

        assert_eq!(interp.constants_count(), 1);
        assert_eq!(interp.instruction_count(), 2); // load_const + return

        // Compile float literal
        let float_ast = Expr::new(
            ExprKind::Literal(Literal::Float(3.14)),
            crate::frontend::ast::Span::new(0, 0),
        );
        let result = interp.compile(&float_ast);
        assert!(result.is_ok());

        assert_eq!(interp.constants_count(), 1); // resets on each compile
        assert_eq!(interp.instruction_count(), 2); // load_const + return

        // Compile string literal
        let string_ast = Expr::new(
            ExprKind::Literal(Literal::String("hello".to_string())),
            crate::frontend::ast::Span::new(0, 0),
        );
        let result = interp.compile(&string_ast);
        assert!(result.is_ok());

        assert_eq!(interp.constants_count(), 1); // resets on each compile
        assert_eq!(interp.instruction_count(), 2); // load_const + return

        // Compile boolean literal
        let bool_ast = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            crate::frontend::ast::Span::new(0, 0),
        );
        let result = interp.compile(&bool_ast);
        assert!(result.is_ok());

        assert_eq!(interp.constants_count(), 1); // resets on each compile
        assert_eq!(interp.instruction_count(), 2); // load_const + return

        // Compile nil literal
        let nil_ast = Expr::new(
            ExprKind::Literal(Literal::Unit),
            crate::frontend::ast::Span::new(0, 0),
        );
        let result = interp.compile(&nil_ast);
        assert!(result.is_ok());

        assert_eq!(interp.instruction_count(), 2); // load_nil + return
                                                   // Nil doesn't add to constants, uses special instruction
    }

    #[test]
    fn test_direct_threaded_binary_op_compilation() {
        use crate::frontend::ast::{BinaryOp, Expr, ExprKind};

        let mut interp = DirectThreadedInterpreter::new();

        // Compile: 2 + 3
        let add_ast = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(2)),
                    crate::frontend::ast::Span::new(0, 0),
                )),
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(3)),
                    crate::frontend::ast::Span::new(0, 0),
                )),
            },
            crate::frontend::ast::Span::new(0, 0),
        );

        let result = interp.compile(&add_ast);
        assert!(result.is_ok());

        // Should have: load_const(2), load_const(3), add, return
        assert_eq!(interp.instruction_count(), 4);
        assert_eq!(interp.constants_count(), 2);
    }

    #[test]
    fn test_direct_threaded_identifier_compilation() {
        use crate::frontend::ast::{Expr, ExprKind};

        let mut interp = DirectThreadedInterpreter::new();

        let ident_ast = Expr::new(
            ExprKind::Identifier("x".to_string()),
            crate::frontend::ast::Span::new(0, 0),
        );
        let result = interp.compile(&ident_ast);
        assert!(result.is_ok());

        // Should add variable name to constants and generate load_var instruction
        assert_eq!(interp.constants_count(), 1);
        assert_eq!(interp.instruction_count(), 2); // load_var + return
    }

    #[test]
    fn test_direct_threaded_execution_simple() {
        use crate::frontend::ast::{Expr, ExprKind};

        let mut interp = DirectThreadedInterpreter::new();

        // Compile and execute: 42
        let ast = Expr::new(
            ExprKind::Literal(Literal::Integer(42)),
            crate::frontend::ast::Span::new(0, 0),
        );
        interp.compile(&ast).expect("Test should not fail");

        let result = interp.execute();
        assert!(result.is_ok());
        assert_eq!(result.expect("Test should not fail"), Value::Integer(42));
    }

    #[test]
    fn test_direct_threaded_execution_arithmetic() {
        use crate::frontend::ast::{BinaryOp, Expr, ExprKind};

        let mut interp = DirectThreadedInterpreter::new();

        // Compile and execute: 2 + 3
        let ast = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(2)),
                    crate::frontend::ast::Span::new(0, 0),
                )),
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(3)),
                    crate::frontend::ast::Span::new(0, 0),
                )),
            },
            crate::frontend::ast::Span::new(0, 0),
        );

        interp.compile(&ast).expect("Test should not fail");
        let result = interp.execute();
        assert!(result.is_ok());
        assert_eq!(result.expect("Test should not fail"), Value::Integer(5));
    }

    #[test]
    fn test_direct_threaded_execution_subtraction() {
        use crate::frontend::ast::{BinaryOp, Expr, ExprKind};

        let mut interp = DirectThreadedInterpreter::new();

        // Compile and execute: 10 - 4
        let ast = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Subtract,
                left: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(10)),
                    crate::frontend::ast::Span::new(0, 0),
                )),
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(4)),
                    crate::frontend::ast::Span::new(0, 0),
                )),
            },
            crate::frontend::ast::Span::new(0, 0),
        );

        interp.compile(&ast).expect("Test should not fail");
        let result = interp.execute();
        assert!(result.is_ok());
        assert_eq!(result.expect("Test should not fail"), Value::Integer(6));
    }

    #[test]
    fn test_direct_threaded_execution_multiplication() {
        use crate::frontend::ast::{BinaryOp, Expr, ExprKind};

        let mut interp = DirectThreadedInterpreter::new();

        // Compile and execute: 6 * 7
        let ast = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Multiply,
                left: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(6)),
                    crate::frontend::ast::Span::new(0, 0),
                )),
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(7)),
                    crate::frontend::ast::Span::new(0, 0),
                )),
            },
            crate::frontend::ast::Span::new(0, 0),
        );

        interp.compile(&ast).expect("Test should not fail");
        let result = interp.execute();
        assert!(result.is_ok());
        assert_eq!(result.expect("Test should not fail"), Value::Integer(42));
    }

    #[test]
    fn test_direct_threaded_execution_division() {
        use crate::frontend::ast::{BinaryOp, Expr, ExprKind};

        let mut interp = DirectThreadedInterpreter::new();

        // Compile and execute: 20 / 4
        let ast = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Divide,
                left: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(20)),
                    crate::frontend::ast::Span::new(0, 0),
                )),
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(4)),
                    crate::frontend::ast::Span::new(0, 0),
                )),
            },
            crate::frontend::ast::Span::new(0, 0),
        );

        interp.compile(&ast).expect("Test should not fail");
        let result = interp.execute();
        assert!(result.is_ok());
        assert_eq!(result.expect("Test should not fail"), Value::Integer(5));
    }

    #[test]
    fn test_direct_threaded_execution_mixed_types() {
        use crate::frontend::ast::{BinaryOp, Expr, ExprKind};

        let mut interp = DirectThreadedInterpreter::new();

        // Compile and execute: 2.5 + 3 (float + int)
        let ast = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Float(2.5)),
                    crate::frontend::ast::Span::new(0, 0),
                )),
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(3)),
                    crate::frontend::ast::Span::new(0, 0),
                )),
            },
            crate::frontend::ast::Span::new(0, 0),
        );

        interp.compile(&ast).expect("Test should not fail");
        let result = interp.execute();
        assert!(result.is_ok());
        assert_eq!(result.expect("Test should not fail"), Value::Float(5.5));
    }

    #[test]
    fn test_direct_threaded_execution_division_by_zero() {
        use crate::frontend::ast::{BinaryOp, Expr, ExprKind};

        let mut interp = DirectThreadedInterpreter::new();

        // Compile and execute: 5 / 0
        let ast = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Divide,
                left: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(5)),
                    crate::frontend::ast::Span::new(0, 0),
                )),
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(0)),
                    crate::frontend::ast::Span::new(0, 0),
                )),
            },
            crate::frontend::ast::Span::new(0, 0),
        );

        interp.compile(&ast).expect("Test should not fail");
        let result = interp.execute();
        assert!(result.is_err());
    }

    #[test]
    fn test_direct_threaded_execution_variable_lookup() {
        use crate::frontend::ast::{Expr, ExprKind};
        use std::collections::HashMap;

        let mut interp = DirectThreadedInterpreter::new();

        // Set up environment with variable
        let mut env = HashMap::new();
        env.insert("x".to_string(), Value::Integer(42));

        // Compile and execute: x
        let ast = Expr::new(
            ExprKind::Identifier("x".to_string()),
            crate::frontend::ast::Span::new(0, 0),
        );
        interp.compile(&ast).expect("Test should not fail");

        let mut state = InterpreterState::new();
        state.env_stack.push(env);
        state.constants = interp.constants.clone();

        // Execute with variable in environment
        let result = interp.execute_with_state(&mut state);
        assert!(result.is_ok());
        assert_eq!(result.expect("Test should not fail"), Value::Integer(42));
    }

    #[test]
    fn test_direct_threaded_execution_undefined_variable() {
        use crate::frontend::ast::{Expr, ExprKind};

        let mut interp = DirectThreadedInterpreter::new();

        // Compile and execute: undefined_var
        let ast = Expr::new(
            ExprKind::Identifier("undefined_var".to_string()),
            crate::frontend::ast::Span::new(0, 0),
        );
        interp.compile(&ast).expect("Test should not fail");

        let result = interp.execute();
        assert!(result.is_err());
        match result.expect_err("Test should fail") {
            InterpreterError::RuntimeError(msg) => {
                assert!(msg.contains("Undefined variable"));
            }
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[test]
    fn test_direct_threaded_instruction_handlers() {
        let mut state = InterpreterState::new();

        // Test op_load_const
        state.constants.push(Value::Integer(42));
        let result = op_load_const(&mut state, 0);
        assert_eq!(result, InstructionResult::Continue);
        assert_eq!(state.stack.len(), 1);
        assert_eq!(state.stack[0], Value::Integer(42));

        // Test op_load_nil
        let result = op_load_nil(&mut state, 0);
        assert_eq!(result, InstructionResult::Continue);
        assert_eq!(state.stack.len(), 2);
        assert_eq!(state.stack[1], Value::Nil);

        // Test arithmetic operations
        state.stack.clear();
        state.stack.push(Value::Integer(5));
        state.stack.push(Value::Integer(3));

        let result = op_add(&mut state, 0);
        assert_eq!(result, InstructionResult::Continue);
        assert_eq!(state.stack.len(), 1);
        assert_eq!(state.stack[0], Value::Integer(8));
    }

    #[test]
    fn test_direct_threaded_clear() {
        let mut interp = DirectThreadedInterpreter::new();

        // Add some instructions and constants
        interp.add_constant(Value::Integer(42));
        interp.add_instruction(op_load_const, 0);

        assert!(interp.instruction_count() > 0);
        assert!(interp.constants_count() > 0);

        // Clear should reset everything
        interp.clear();

        assert_eq!(interp.instruction_count(), 0);
        assert_eq!(interp.constants_count(), 0);
    }
}
