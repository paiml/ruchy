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

/// Runtime value representation using safe enum approach
/// Alternative to tagged pointers that respects project's `unsafe_code = "forbid"`
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
}

impl Value {
    /// Create integer value
    pub fn from_i64(i: i64) -> Self {
        Value::Integer(i)
    }

    /// Create float value
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

    /// Check if value is truthy (everything except false and nil)
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
        }
    }
}

// Note: Complex object structures (ObjectHeader, Class, etc.) will be implemented
// in Phase 1 of the interpreter spec when we add proper GC and method dispatch.

/// Runtime interpreter state
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

/// Interpreter errors
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

/// Type feedback collection for JIT compilation decisions
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

/// Conservative garbage collector for heap-allocated objects
/// Currently operates alongside Rc-based memory management
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
            ExprKind::Literal(lit) => {
                if matches!(lit, Literal::Unit) {
                    // Nil doesn't need to be stored in constants
                    self.emit_instruction(op_load_nil, 0);
                } else {
                    let const_idx = self.add_constant(self.literal_to_value(lit));
                    self.emit_instruction(op_load_const, const_idx);
                }
            }

            ExprKind::Binary { left, op, right } => {
                // Compile operands first
                self.compile_expr(left)?;
                self.compile_expr(right)?;

                // Emit binary operation
                let op_code = match op {
                    crate::frontend::ast::BinaryOp::Add => op_add,
                    crate::frontend::ast::BinaryOp::Subtract => op_sub,
                    crate::frontend::ast::BinaryOp::Multiply => op_mul,
                    crate::frontend::ast::BinaryOp::Divide => op_div,
                    _ => {
                        return Err(InterpreterError::RuntimeError(format!(
                            "Unsupported binary operation: {:?}",
                            op
                        )));
                    }
                };
                self.emit_instruction(op_code, 0);
            }

            ExprKind::Identifier(name) => {
                let name_idx = self.add_constant(Value::String(Rc::new(name.clone())));
                self.emit_instruction(op_load_var, name_idx);
            }

            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                // Compile condition
                self.compile_expr(condition)?;

                // Jump if false to else branch
                let else_jump_addr = self.code.len();
                self.emit_instruction(op_jump_if_false, 0); // Placeholder

                // Compile then branch
                self.compile_expr(then_branch)?;

                if let Some(else_expr) = else_branch {
                    // Jump over else branch
                    let end_jump_addr = self.code.len();
                    self.emit_instruction(op_jump, 0); // Placeholder

                    // Fix else jump target
                    let else_target = self.code.len();
                    if let Some(instr) = self.code.get_mut(else_jump_addr) {
                        instr.operand = else_target as u32;
                    }

                    // Compile else branch
                    self.compile_expr(else_expr)?;

                    // Fix end jump target
                    let end_target = self.code.len();
                    if let Some(instr) = self.code.get_mut(end_jump_addr) {
                        instr.operand = end_target as u32;
                    }
                } else {
                    // No else branch - jump to end
                    let end_target = self.code.len();
                    if let Some(instr) = self.code.get_mut(else_jump_addr) {
                        instr.operand = end_target as u32;
                    }

                    // Push nil for missing else branch
                    self.emit_instruction(op_load_nil, 0);
                }
            }

            _ => {
                // For other expression types, fall back to AST evaluation
                // This is a hybrid approach during the transition
                let value_idx =
                    self.add_constant(Value::String(Rc::new("AST_FALLBACK".to_string())));
                self.emit_instruction(op_ast_fallback, value_idx);
            }
        }

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
        }
    }
}

impl Interpreter {
    /// Create new interpreter instance
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(1024), // Pre-allocate stack
            env_stack: vec![HashMap::new()], // Start with global environment
            frames: Vec::new(),
            execution_counts: HashMap::new(),
            field_caches: HashMap::new(),
            type_feedback: TypeFeedback::new(),
            gc: ConservativeGC::new(),
        }
    }

    /// Evaluate an AST expression directly
    /// # Errors
    /// Returns error if evaluation fails (type errors, runtime errors, etc.)
    pub fn eval_expr(&mut self, expr: &Expr) -> Result<Value, InterpreterError> {
        self.eval_expr_kind(&expr.kind)
    }

    /// Evaluate an expression kind directly (main AST walker)
    /// # Errors
    /// Returns error if evaluation fails
    fn eval_expr_kind(&mut self, expr_kind: &ExprKind) -> Result<Value, InterpreterError> {
        match expr_kind {
            ExprKind::Literal(lit) => Ok(self.eval_literal(lit)),

            ExprKind::Identifier(name) => self.lookup_variable(name),

            ExprKind::Binary { left, op, right } => {
                let left_val = self.eval_expr(left)?;
                let right_val = self.eval_expr(right)?;
                let result = self.eval_binary_op(*op, &left_val, &right_val)?;
                // Collect type feedback for this binary operation
                let site_id = left.span.start; // Use span start as site ID
                self.record_binary_op_feedback(site_id, &left_val, &right_val, &result);
                Ok(result)
            }

            ExprKind::Unary { op, operand } => {
                let operand_val = self.eval_expr(operand)?;
                self.eval_unary_op(*op, &operand_val)
            }

            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_val = self.eval_expr(condition)?;
                if cond_val.is_truthy() {
                    self.eval_expr(then_branch)
                } else if let Some(else_branch) = else_branch {
                    self.eval_expr(else_branch)
                } else {
                    Ok(Value::nil())
                }
            }

            ExprKind::Let {
                name, value, body, ..
            } => {
                let val = self.eval_expr(value)?;
                // Collect type feedback for variable assignment
                self.record_variable_assignment_feedback(name, &val);
                // Store in current environment
                self.env_set(name.clone(), val);
                let result = self.eval_expr(body)?;
                // Remove binding
                self.env_remove(name);
                Ok(result)
            }

            ExprKind::Function {
                name, params, body, ..
            } => self.eval_function(name, params, body),

            ExprKind::Lambda { params, body } => self.eval_lambda(params, body),

            ExprKind::Call { func, args } => self.eval_function_call(func, args),

            ExprKind::List(elements) => {
                let mut values = Vec::new();
                for element in elements {
                    values.push(self.eval_expr(element)?);
                }
                Ok(Value::Array(Rc::new(values)))
            }

            ExprKind::Block(statements) => {
                if statements.is_empty() {
                    Ok(Value::nil())
                } else {
                    let mut result = Value::nil();
                    for stmt in statements {
                        result = self.eval_expr(stmt)?;
                    }
                    Ok(result)
                }
            }
            
            ExprKind::MethodCall { receiver, method, args } => {
                self.eval_method_call(receiver, method, args)
            }
            
            ExprKind::StringInterpolation { parts } => {
                self.eval_string_interpolation(parts)
            }
            
            ExprKind::Range { start, end, inclusive } => {
                let start_val = self.eval_expr(start)?;
                let end_val = self.eval_expr(end)?;
                
                match (start_val, end_val) {
                    (Value::Integer(s), Value::Integer(e)) => {
                        let range_end = if *inclusive { e + 1 } else { e };
                        let values: Vec<Value> = (s..range_end).map(Value::Integer).collect();
                        Ok(Value::Array(Rc::new(values)))
                    }
                    _ => Err(InterpreterError::RuntimeError(
                        "Range bounds must be integers".to_string()
                    )),
                }
            }
            
            ExprKind::Tuple(elements) => {
                let mut values = Vec::new();
                for element in elements {
                    values.push(self.eval_expr(element)?);
                }
                Ok(Value::Tuple(Rc::new(values)))
            }
            
            ExprKind::For { var, pattern, iter, body } => {
                self.eval_for_loop(var, pattern.as_ref(), iter, body)
            }
            
            ExprKind::While { condition, body } => {
                self.eval_while_loop(condition, body)
            }
            
            ExprKind::Break { label: _ } => {
                // For now, ignore labels and just break
                Err(InterpreterError::RuntimeError("break".to_string()))
            }
            
            ExprKind::Continue { label: _ } => {
                // For now, ignore labels and just continue
                Err(InterpreterError::RuntimeError("continue".to_string()))
            }
            
            ExprKind::Return { value } => {
                let return_value = if let Some(val_expr) = value {
                    self.eval_expr(val_expr)?
                } else {
                    Value::nil()
                };
                Err(InterpreterError::RuntimeError(format!("return:{}", return_value)))
            }
            
            ExprKind::Match { expr, arms } => {
                self.eval_match(expr, arms)
            }
            
            ExprKind::Assign { target, value } => {
                self.eval_assign(target, value)
            }
            
            ExprKind::CompoundAssign { target, op, value } => {
                self.eval_compound_assign(target, *op, value)
            }

            // Placeholder implementations for other expression types
            _ => Err(InterpreterError::RuntimeError(format!(
                "Expression type not yet implemented: {expr_kind:?}"
            ))),
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
        let env = self
            .env_stack
            .last_mut()
            .expect("Environment stack should never be empty");
        env.insert(name, value);
    }

    /// Remove a variable from the current environment
    #[allow(clippy::expect_used)] // Environment stack invariant ensures this never panics
    fn env_remove(&mut self, name: &str) {
        let env = self
            .env_stack
            .last_mut()
            .expect("Environment stack should never be empty");
        env.remove(name);
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

    /// Call a function with given arguments
    fn call_function(&mut self, func: Value, args: &[Value]) -> Result<Value, InterpreterError> {
        match func {
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

    /// Evaluate a binary operation from AST
    fn eval_binary_op(
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
            AstBinaryOp::Equal => Ok(Value::from_bool(self.equal_values(left, right))),
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
            AstBinaryOp::NotEqual => Ok(Value::from_bool(!self.equal_values(left, right))),
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
            AstBinaryOp::Modulo => self.modulo_values(left, right),
            AstBinaryOp::Power => self.power_values(left, right),
            _ => Err(InterpreterError::RuntimeError(format!(
                "Binary operator not yet implemented: {op:?}"
            ))),
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

    /// Helper function for testing - evaluate a string expression via parser
    /// # Errors
    /// Returns error if parsing or evaluation fails
    #[cfg(test)]
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
            Pattern::Literal(lit) => {
                let lit_value = self.eval_literal(lit);
                Ok(lit_value == *value)
            }
            Pattern::Identifier(_name) => {
                // Identifier patterns always match and bind the value
                // Binding is handled separately
                Ok(true)
            }
            Pattern::Tuple(patterns) => {
                if let Value::Tuple(elements) = value {
                    if patterns.len() != elements.len() {
                        return Ok(false);
                    }
                    for (pat, val) in patterns.iter().zip(elements.iter()) {
                        if !self.pattern_matches(pat, val)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Pattern::List(patterns) => {
                if let Value::Array(elements) = value {
                    if patterns.len() != elements.len() {
                        return Ok(false);
                    }
                    for (pat, val) in patterns.iter().zip(elements.iter()) {
                        if !self.pattern_matches(pat, val)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Pattern::Or(patterns) => {
                for pat in patterns {
                    if self.pattern_matches(pat, value)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Pattern::Range { start, end, inclusive } => {
                if let Value::Integer(i) = value {
                    let start_val = if let Pattern::Literal(Literal::Integer(s)) = &**start {
                        *s
                    } else {
                        return Ok(false);
                    };
                    let end_val = if let Pattern::Literal(Literal::Integer(e)) = &**end {
                        *e
                    } else {
                        return Ok(false);
                    };
                    
                    if *inclusive {
                        Ok(*i >= start_val && *i <= end_val)
                    } else {
                        Ok(*i >= start_val && *i < end_val)
                    }
                } else {
                    Ok(false)
                }
            }
            _ => {
                // Other patterns not yet implemented
                Ok(false)
            }
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

    /// Compute field access result (slow path)
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
    
    /// Evaluate a method call
    fn eval_method_call(&mut self, receiver: &Expr, method: &str, args: &[Expr]) -> Result<Value, InterpreterError> {
        let receiver_value = self.eval_expr(receiver)?;
        let arg_values: Result<Vec<_>, _> = args.iter().map(|arg| self.eval_expr(arg)).collect();
        let arg_values = arg_values?;
        
        match (method, &receiver_value) {
            // Float methods
            ("sqrt", Value::Float(f)) if args.is_empty() => Ok(Value::Float(f.sqrt())),
            ("abs", Value::Float(f)) if args.is_empty() => Ok(Value::Float(f.abs())),
            ("round", Value::Float(f)) if args.is_empty() => Ok(Value::Float(f.round())),
            ("floor", Value::Float(f)) if args.is_empty() => Ok(Value::Float(f.floor())),
            ("ceil", Value::Float(f)) if args.is_empty() => Ok(Value::Float(f.ceil())),
            
            // String methods
            ("len", Value::String(s)) if args.is_empty() => Ok(Value::Integer(s.len() as i64)),
            ("to_upper", Value::String(s)) if args.is_empty() => Ok(Value::from_string(s.to_uppercase())),
            ("to_lower", Value::String(s)) if args.is_empty() => Ok(Value::from_string(s.to_lowercase())),
            ("trim", Value::String(s)) if args.is_empty() => Ok(Value::from_string(s.trim().to_string())),
            
            // Array methods
            ("len", Value::Array(arr)) if args.is_empty() => Ok(Value::Integer(arr.len() as i64)),
            ("push", Value::Array(arr)) if args.len() == 1 => {
                let mut new_arr = (**arr).clone();
                new_arr.push(arg_values[0].clone());
                Ok(Value::Array(Rc::new(new_arr)))
            }
            ("pop", Value::Array(arr)) if args.is_empty() => {
                let mut new_arr = (**arr).clone();
                new_arr.pop().unwrap_or(Value::nil());
                Ok(Value::Array(Rc::new(new_arr)))
            }
            
            // Generic to_string method
            ("to_string", _) if args.is_empty() => Ok(Value::from_string(receiver_value.to_string())),
            
            _ => Err(InterpreterError::RuntimeError(format!(
                "Method '{}' not found for type or wrong number of arguments",
                method
            ))),
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
            }
        }
        Ok(Value::from_string(result))
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
