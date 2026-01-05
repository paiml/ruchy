//! JIT Type Feedback System - Extracted for 100% Coverage
//!
//! Collects runtime type information to guide JIT optimization decisions.
//! Includes inline caching for field access and type specialization candidates.

use crate::runtime::Value;
use smallvec::{smallvec, SmallVec};
use std::collections::HashMap;

/// State of an inline cache for field access optimization
#[derive(Clone, Debug, PartialEq)]
pub enum CacheState {
    /// Cache has not been used yet
    Uninitialized,
    /// Cache has seen only one type (best for optimization)
    Monomorphic,
    /// Cache has seen 2-4 types (still optimizable)
    Polymorphic,
    /// Cache has seen too many types (megamorphic - fallback to slow path)
    Megamorphic,
}

/// Single entry in the inline cache
#[derive(Clone, Debug)]
pub struct CacheEntry {
    /// Type ID of the object this entry is for
    pub type_id: std::any::TypeId,
    /// Field name being accessed
    pub field_name: String,
    /// Cached result of the field access
    pub cached_result: Value,
    /// Number of times this entry was hit
    pub hit_count: u32,
}

/// Inline cache for field access operations
///
/// Uses polymorphic inline caching (PIC) with up to 4 entries.
/// Transitions through states: Uninitialized -> Monomorphic -> Polymorphic -> Megamorphic
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

    /// Get current cache state
    pub fn state(&self) -> &CacheState {
        &self.state
    }

    /// Get number of entries in cache
    pub fn entry_count(&self) -> usize {
        self.entries.len()
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

    /// Get total hits
    pub fn total_hits(&self) -> u32 {
        self.total_hits
    }

    /// Get total misses
    pub fn total_misses(&self) -> u32 {
        self.total_misses
    }
}

impl Default for InlineCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Feedback for a specific operation site (binary ops, field access, etc.)
#[derive(Clone, Debug)]
pub struct OperationFeedback {
    /// Types observed for left operand
    pub left_types: SmallVec<[std::any::TypeId; 4]>,
    /// Types observed for right operand (for binary ops)
    pub right_types: SmallVec<[std::any::TypeId; 4]>,
    /// Result types observed
    pub result_types: SmallVec<[std::any::TypeId; 4]>,
    /// Hit counts for each type combination
    pub type_counts: HashMap<(std::any::TypeId, std::any::TypeId), u32>,
    /// Total operation count
    pub total_count: u32,
}

impl OperationFeedback {
    /// Create new operation feedback
    pub fn new() -> Self {
        Self {
            left_types: smallvec![],
            right_types: smallvec![],
            result_types: smallvec![],
            type_counts: HashMap::new(),
            total_count: 0,
        }
    }

    /// Check if operation is monomorphic (single type pair)
    pub fn is_monomorphic(&self) -> bool {
        self.left_types.len() == 1 && self.right_types.len() == 1
    }
}

impl Default for OperationFeedback {
    fn default() -> Self {
        Self::new()
    }
}

/// Type feedback for variables across their lifetime
#[derive(Clone, Debug)]
pub struct VariableTypeFeedback {
    /// Types assigned to this variable
    pub assigned_types: SmallVec<[std::any::TypeId; 4]>,
    /// Type transitions (`from_type` -> `to_type`)
    pub transitions: HashMap<std::any::TypeId, HashMap<std::any::TypeId, u32>>,
    /// Most common type (for specialization)
    pub dominant_type: Option<std::any::TypeId>,
    /// Type stability score (0.0 = highly polymorphic, 1.0 = monomorphic)
    pub stability_score: f64,
}

impl VariableTypeFeedback {
    /// Create new variable type feedback
    pub fn new() -> Self {
        Self {
            assigned_types: smallvec![],
            transitions: HashMap::new(),
            dominant_type: None,
            stability_score: 1.0,
        }
    }

    /// Check if variable is stable (good candidate for specialization)
    pub fn is_stable(&self) -> bool {
        self.stability_score > 0.8
    }
}

impl Default for VariableTypeFeedback {
    fn default() -> Self {
        Self::new()
    }
}

/// Feedback for function call sites
#[derive(Clone, Debug)]
pub struct CallSiteFeedback {
    /// Argument type patterns observed
    pub arg_type_patterns: SmallVec<[Vec<std::any::TypeId>; 4]>,
    /// Return types observed
    pub return_types: SmallVec<[std::any::TypeId; 4]>,
    /// Call frequency
    pub call_count: u32,
    /// Functions called at this site (for polymorphic calls)
    pub called_functions: HashMap<String, u32>,
}

impl CallSiteFeedback {
    /// Create new call site feedback
    pub fn new() -> Self {
        Self {
            arg_type_patterns: smallvec![],
            return_types: smallvec![],
            call_count: 0,
            called_functions: HashMap::new(),
        }
    }

    /// Check if call site is monomorphic
    pub fn is_monomorphic(&self) -> bool {
        self.arg_type_patterns.len() == 1 && self.return_types.len() == 1
    }
}

impl Default for CallSiteFeedback {
    fn default() -> Self {
        Self::new()
    }
}

/// Type feedback collection for JIT compilation decisions.
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
            .or_default();

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
            .or_default();

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
            .or_default();

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
    pub fn get_specialization_candidates(&self) -> Vec<SpecializationCandidate> {
        let mut candidates = Vec::new();

        // Find monomorphic operation sites
        for (&site_id, feedback) in &self.operation_sites {
            if feedback.is_monomorphic() && feedback.total_count > 10 {
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
            if feedback.is_stable() {
                if let Some(dominant_type) = feedback.dominant_type {
                    candidates.push(SpecializationCandidate {
                        kind: SpecializationKind::Variable {
                            name: var_name.clone(),
                            specialized_type: dominant_type,
                        },
                        confidence: feedback.stability_score,
                        benefit_score: feedback.stability_score * 100.0,
                    });
                }
            }
        }

        // Find monomorphic call sites
        for (&site_id, feedback) in &self.call_sites {
            if feedback.is_monomorphic() && feedback.call_count > 5 {
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
            .filter(|f| f.is_monomorphic())
            .count();

        let stable_variables = self
            .variable_types
            .values()
            .filter(|f| f.is_stable())
            .count();

        let monomorphic_calls = self
            .call_sites
            .values()
            .filter(|f| f.is_monomorphic())
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

    /// Get total samples collected
    pub fn total_samples(&self) -> u64 {
        self.total_samples
    }
}

impl Default for TypeFeedback {
    fn default() -> Self {
        Self::new()
    }
}

/// Specialization candidate for JIT compilation
#[derive(Clone, Debug)]
pub struct SpecializationCandidate {
    /// Type of specialization
    pub kind: SpecializationKind,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f64,
    /// Expected benefit score
    pub benefit_score: f64,
}

/// Kind of JIT specialization
#[derive(Clone, Debug)]
pub enum SpecializationKind {
    /// Specialize a binary operation for specific types
    BinaryOperation {
        site_id: usize,
        left_type: std::any::TypeId,
        right_type: std::any::TypeId,
    },
    /// Specialize a variable for a specific type
    Variable {
        name: String,
        specialized_type: std::any::TypeId,
    },
    /// Specialize a function call site
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

#[cfg(test)]
mod tests {
    use super::*;

    // ============ InlineCache Tests ============

    #[test]
    fn test_inline_cache_new() {
        let cache = InlineCache::new();
        assert_eq!(*cache.state(), CacheState::Uninitialized);
        assert_eq!(cache.entry_count(), 0);
        assert_eq!(cache.total_hits(), 0);
        assert_eq!(cache.total_misses(), 0);
    }

    #[test]
    fn test_inline_cache_default() {
        let cache = InlineCache::default();
        assert_eq!(*cache.state(), CacheState::Uninitialized);
    }

    #[test]
    fn test_inline_cache_lookup_miss() {
        let mut cache = InlineCache::new();
        let value = Value::Integer(42);
        assert!(cache.lookup(&value, "field").is_none());
        assert_eq!(cache.total_misses(), 1);
    }

    #[test]
    fn test_inline_cache_insert_monomorphic() {
        let mut cache = InlineCache::new();
        let obj = Value::Integer(1);
        cache.insert(&obj, "x".to_string(), Value::Integer(10));

        assert_eq!(*cache.state(), CacheState::Monomorphic);
        assert_eq!(cache.entry_count(), 1);
    }

    #[test]
    fn test_inline_cache_lookup_hit() {
        let mut cache = InlineCache::new();
        let obj = Value::Integer(1);
        cache.insert(&obj, "x".to_string(), Value::Integer(10));

        let result = cache.lookup(&obj, "x");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), Value::Integer(10));
        assert_eq!(cache.total_hits(), 1);
    }

    #[test]
    fn test_inline_cache_polymorphic() {
        let mut cache = InlineCache::new();
        cache.insert(&Value::Integer(1), "a".to_string(), Value::Integer(1));
        cache.insert(&Value::Float(1.0), "b".to_string(), Value::Float(2.0));

        assert_eq!(*cache.state(), CacheState::Polymorphic);
        assert_eq!(cache.entry_count(), 2);
    }

    #[test]
    fn test_inline_cache_megamorphic() {
        let mut cache = InlineCache::new();
        // Insert 5 different entries to trigger megamorphic
        for i in 0..5 {
            cache.insert(
                &Value::Integer(i),
                format!("field{i}"),
                Value::Integer(i * 10),
            );
        }

        assert_eq!(*cache.state(), CacheState::Megamorphic);
    }

    #[test]
    fn test_inline_cache_hit_rate_zero() {
        let cache = InlineCache::new();
        assert_eq!(cache.hit_rate(), 0.0);
    }

    #[test]
    fn test_inline_cache_hit_rate() {
        let mut cache = InlineCache::new();
        let obj = Value::Integer(1);
        cache.insert(&obj, "x".to_string(), Value::Integer(10));

        // 1 hit, 0 misses initially (insert doesn't count as miss)
        cache.lookup(&obj, "x"); // hit
        cache.lookup(&obj, "y"); // miss

        assert!(cache.hit_rate() > 0.0 && cache.hit_rate() < 1.0);
    }

    // ============ CacheState Tests ============

    #[test]
    fn test_cache_state_equality() {
        assert_eq!(CacheState::Uninitialized, CacheState::Uninitialized);
        assert_eq!(CacheState::Monomorphic, CacheState::Monomorphic);
        assert_ne!(CacheState::Monomorphic, CacheState::Polymorphic);
    }

    // ============ OperationFeedback Tests ============

    #[test]
    fn test_operation_feedback_new() {
        let feedback = OperationFeedback::new();
        assert!(feedback.left_types.is_empty());
        assert!(feedback.right_types.is_empty());
        assert_eq!(feedback.total_count, 0);
    }

    #[test]
    fn test_operation_feedback_default() {
        let feedback = OperationFeedback::default();
        assert!(feedback.left_types.is_empty());
    }

    #[test]
    fn test_operation_feedback_is_monomorphic_empty() {
        let feedback = OperationFeedback::new();
        assert!(!feedback.is_monomorphic());
    }

    // ============ VariableTypeFeedback Tests ============

    #[test]
    fn test_variable_type_feedback_new() {
        let feedback = VariableTypeFeedback::new();
        assert!(feedback.assigned_types.is_empty());
        assert!(feedback.dominant_type.is_none());
        assert_eq!(feedback.stability_score, 1.0);
    }

    #[test]
    fn test_variable_type_feedback_default() {
        let feedback = VariableTypeFeedback::default();
        assert!(feedback.assigned_types.is_empty());
    }

    #[test]
    fn test_variable_type_feedback_is_stable() {
        let mut feedback = VariableTypeFeedback::new();
        feedback.stability_score = 0.9;
        assert!(feedback.is_stable());

        feedback.stability_score = 0.5;
        assert!(!feedback.is_stable());
    }

    // ============ CallSiteFeedback Tests ============

    #[test]
    fn test_call_site_feedback_new() {
        let feedback = CallSiteFeedback::new();
        assert!(feedback.arg_type_patterns.is_empty());
        assert!(feedback.return_types.is_empty());
        assert_eq!(feedback.call_count, 0);
    }

    #[test]
    fn test_call_site_feedback_default() {
        let feedback = CallSiteFeedback::default();
        assert_eq!(feedback.call_count, 0);
    }

    #[test]
    fn test_call_site_feedback_is_monomorphic_empty() {
        let feedback = CallSiteFeedback::new();
        assert!(!feedback.is_monomorphic());
    }

    // ============ TypeFeedback Tests ============

    #[test]
    fn test_type_feedback_new() {
        let feedback = TypeFeedback::new();
        assert_eq!(feedback.total_samples(), 0);
    }

    #[test]
    fn test_type_feedback_default() {
        let feedback = TypeFeedback::default();
        assert_eq!(feedback.total_samples(), 0);
    }

    #[test]
    fn test_type_feedback_record_binary_op() {
        let mut feedback = TypeFeedback::new();
        let left = Value::Integer(1);
        let right = Value::Integer(2);
        let result = Value::Integer(3);

        feedback.record_binary_op(0, &left, &right, &result);

        assert_eq!(feedback.total_samples(), 1);
        let stats = feedback.get_statistics();
        assert_eq!(stats.total_operation_sites, 1);
    }

    #[test]
    fn test_type_feedback_record_binary_op_monomorphic() {
        let mut feedback = TypeFeedback::new();

        // Record same types multiple times
        for _ in 0..15 {
            feedback.record_binary_op(
                0,
                &Value::Integer(1),
                &Value::Integer(2),
                &Value::Integer(3),
            );
        }

        let stats = feedback.get_statistics();
        assert_eq!(stats.monomorphic_operation_sites, 1);
    }

    #[test]
    fn test_type_feedback_record_variable_assignment() {
        let mut feedback = TypeFeedback::new();
        let type_id = Value::Integer(1).type_id();

        feedback.record_variable_assignment("x", type_id);

        let stats = feedback.get_statistics();
        assert_eq!(stats.total_variables, 1);
        assert_eq!(stats.stable_variables, 1);
    }

    #[test]
    fn test_type_feedback_variable_type_transition() {
        let mut feedback = TypeFeedback::new();
        let int_type = Value::Integer(1).type_id();
        let float_type = Value::Float(1.0).type_id();

        feedback.record_variable_assignment("x", int_type);
        feedback.record_variable_assignment("x", float_type);

        let stats = feedback.get_statistics();
        assert_eq!(stats.total_variables, 1);
        // After transition, stability decreases
        assert_eq!(stats.stable_variables, 0);
    }

    #[test]
    fn test_type_feedback_record_function_call() {
        let mut feedback = TypeFeedback::new();
        let args = vec![Value::Integer(1)];
        let result = Value::Integer(2);

        feedback.record_function_call(0, "add", &args, &result);

        let stats = feedback.get_statistics();
        assert_eq!(stats.total_call_sites, 1);
    }

    #[test]
    fn test_type_feedback_monomorphic_call_site() {
        let mut feedback = TypeFeedback::new();
        let args = vec![Value::Integer(1)];
        let result = Value::Integer(2);

        for _ in 0..10 {
            feedback.record_function_call(0, "add", &args, &result);
        }

        let stats = feedback.get_statistics();
        assert_eq!(stats.monomorphic_call_sites, 1);
    }

    #[test]
    fn test_type_feedback_get_specialization_candidates_empty() {
        let feedback = TypeFeedback::new();
        let candidates = feedback.get_specialization_candidates();
        assert!(candidates.is_empty());
    }

    #[test]
    fn test_type_feedback_get_specialization_candidates() {
        let mut feedback = TypeFeedback::new();

        // Record monomorphic binary op (>10 times)
        for _ in 0..15 {
            feedback.record_binary_op(
                0,
                &Value::Integer(1),
                &Value::Integer(2),
                &Value::Integer(3),
            );
        }

        // Record stable variable
        let type_id = Value::Integer(1).type_id();
        feedback.record_variable_assignment("x", type_id);

        // Record monomorphic call (>5 times)
        let args = vec![Value::Integer(1)];
        for _ in 0..10 {
            feedback.record_function_call(1, "func", &args, &Value::Integer(2));
        }

        let candidates = feedback.get_specialization_candidates();
        assert!(!candidates.is_empty());
        // Should be sorted by benefit score
        for i in 1..candidates.len() {
            assert!(candidates[i - 1].benefit_score >= candidates[i].benefit_score);
        }
    }

    #[test]
    fn test_type_feedback_statistics() {
        let mut feedback = TypeFeedback::new();

        // Add some data
        feedback.record_binary_op(
            0,
            &Value::Integer(1),
            &Value::Integer(2),
            &Value::Integer(3),
        );
        feedback.record_variable_assignment("x", Value::Integer(1).type_id());
        feedback.record_function_call(1, "f", &[], &Value::Nil);

        let stats = feedback.get_statistics();
        assert_eq!(stats.total_operation_sites, 1);
        assert_eq!(stats.total_variables, 1);
        assert_eq!(stats.total_call_sites, 1);
        assert_eq!(stats.total_samples, 1);
    }

    // ============ SpecializationKind Tests ============

    #[test]
    fn test_specialization_kind_binary_op() {
        let kind = SpecializationKind::BinaryOperation {
            site_id: 0,
            left_type: Value::Integer(1).type_id(),
            right_type: Value::Integer(2).type_id(),
        };
        // Just verify it can be created and debug printed
        let _ = format!("{kind:?}");
    }

    #[test]
    fn test_specialization_kind_variable() {
        let kind = SpecializationKind::Variable {
            name: "x".to_string(),
            specialized_type: Value::Integer(1).type_id(),
        };
        let _ = format!("{kind:?}");
    }

    #[test]
    fn test_specialization_kind_function_call() {
        let kind = SpecializationKind::FunctionCall {
            site_id: 0,
            arg_types: vec![Value::Integer(1).type_id()],
            return_type: Value::Integer(2).type_id(),
        };
        let _ = format!("{kind:?}");
    }

    // ============ SpecializationCandidate Tests ============

    #[test]
    fn test_specialization_candidate() {
        let candidate = SpecializationCandidate {
            kind: SpecializationKind::Variable {
                name: "x".to_string(),
                specialized_type: Value::Integer(1).type_id(),
            },
            confidence: 0.95,
            benefit_score: 100.0,
        };
        assert_eq!(candidate.confidence, 0.95);
        assert_eq!(candidate.benefit_score, 100.0);
    }

    // ============ TypeFeedbackStats Tests ============

    #[test]
    fn test_type_feedback_stats() {
        let stats = TypeFeedbackStats {
            total_operation_sites: 10,
            monomorphic_operation_sites: 5,
            total_variables: 20,
            stable_variables: 15,
            total_call_sites: 8,
            monomorphic_call_sites: 4,
            total_samples: 100,
        };

        assert_eq!(stats.total_operation_sites, 10);
        assert_eq!(stats.monomorphic_operation_sites, 5);
        assert_eq!(stats.total_variables, 20);
        assert_eq!(stats.stable_variables, 15);
        assert_eq!(stats.total_call_sites, 8);
        assert_eq!(stats.monomorphic_call_sites, 4);
        assert_eq!(stats.total_samples, 100);
    }
}
