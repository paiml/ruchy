//! Type feedback for JIT optimization
//! Extracted from interpreter.rs for modularity (complexity: ≤10 per function)

use super::cache::TypeId;
use std::collections::HashMap;

/// Type feedback for optimization decisions
pub struct TypeFeedback {
    /// Binary operation feedback
    binary_ops: HashMap<usize, OperationFeedback>,
    /// Variable type feedback
    variables: HashMap<String, VariableTypeFeedback>,
    /// Function call feedback
    call_sites: HashMap<usize, CallSiteFeedback>,
    /// Specialization candidates
    candidates: Vec<SpecializationCandidate>,
}

impl TypeFeedback {
    /// Create new type feedback tracker
    pub fn new() -> Self {
        Self {
            binary_ops: HashMap::new(),
            variables: HashMap::new(),
            call_sites: HashMap::new(),
            candidates: Vec::new(),
        }
    }

    /// Record binary operation types
    pub fn record_binary_op(&mut self, location: usize, left: TypeId, right: TypeId) {
        let entry = self.binary_ops.entry(location).or_insert_with(|| {
            OperationFeedback::new(location)
        });
        entry.record(left, right);
        
        // Check for specialization opportunity
        if entry.is_stable() {
            self.consider_specialization(location, entry);
        }
    }

    /// Record variable assignment
    pub fn record_variable_assignment(&mut self, name: String, type_id: TypeId) {
        let entry = self.variables.entry(name.clone()).or_insert_with(|| {
            VariableTypeFeedback::new(name.clone())
        });
        entry.record(type_id);
    }

    /// Record function call
    pub fn record_function_call(&mut self, location: usize, arg_types: Vec<TypeId>) {
        let entry = self.call_sites.entry(location).or_insert_with(|| {
            CallSiteFeedback::new(location)
        });
        entry.record(arg_types);
    }

    /// Consider creating specialization
    fn consider_specialization(&mut self, location: usize, feedback: &OperationFeedback) {
        if let Some((left, right)) = feedback.get_dominant_types() {
            let candidate = SpecializationCandidate {
                location,
                kind: SpecializationKind::BinaryOp { left, right },
                confidence: feedback.confidence(),
            };
            
            if candidate.confidence > 0.9 {
                self.candidates.push(candidate);
            }
        }
    }

    /// Get specialization candidates
    pub fn get_specialization_candidates(&self) -> Vec<SpecializationCandidate> {
        self.candidates
            .iter()
            .filter(|c| c.confidence > 0.9)
            .cloned()
            .collect()
    }

    /// Get statistics
    pub fn get_statistics(&self) -> TypeFeedbackStats {
        TypeFeedbackStats {
            binary_ops_tracked: self.binary_ops.len(),
            variables_tracked: self.variables.len(),
            call_sites_tracked: self.call_sites.len(),
            specialization_candidates: self.candidates.len(),
        }
    }

    /// Clear all feedback
    pub fn clear(&mut self) {
        self.binary_ops.clear();
        self.variables.clear();
        self.call_sites.clear();
        self.candidates.clear();
    }
}

impl Default for TypeFeedback {
    fn default() -> Self {
        Self::new()
    }
}

/// Feedback for binary operations
#[derive(Debug, Clone)]
pub struct OperationFeedback {
    location: usize,
    observations: Vec<(TypeId, TypeId)>,
    type_counts: HashMap<(TypeId, TypeId), usize>,
}

impl OperationFeedback {
    /// Create new operation feedback
    pub fn new(location: usize) -> Self {
        Self {
            location,
            observations: Vec::new(),
            type_counts: HashMap::new(),
        }
    }

    /// Record an observation
    pub fn record(&mut self, left: TypeId, right: TypeId) {
        self.observations.push((left, right));
        *self.type_counts.entry((left, right)).or_insert(0) += 1;
        
        // Keep only recent observations
        if self.observations.len() > 1000 {
            self.observations.drain(0..500);
        }
    }

    /// Check if types are stable
    pub fn is_stable(&self) -> bool {
        if self.observations.len() < 10 {
            return false;
        }
        
        // Check if one type pair dominates
        if let Some((_, count)) = self.type_counts.iter().max_by_key(|(_, c)| *c) {
            let total = self.observations.len();
            *count as f64 / total as f64 > 0.9
        } else {
            false
        }
    }

    /// Get dominant type pair
    pub fn get_dominant_types(&self) -> Option<(TypeId, TypeId)> {
        self.type_counts
            .iter()
            .max_by_key(|(_, c)| *c)
            .map(|((l, r), _)| (*l, *r))
    }

    /// Get confidence level
    pub fn confidence(&self) -> f64 {
        if self.observations.is_empty() {
            return 0.0;
        }
        
        if let Some((_, count)) = self.type_counts.iter().max_by_key(|(_, c)| *c) {
            *count as f64 / self.observations.len() as f64
        } else {
            0.0
        }
    }
}

/// Feedback for variable types
#[derive(Debug, Clone)]
pub struct VariableTypeFeedback {
    name: String,
    observations: Vec<TypeId>,
    type_counts: HashMap<TypeId, usize>,
}

impl VariableTypeFeedback {
    /// Create new variable feedback
    pub fn new(name: String) -> Self {
        Self {
            name,
            observations: Vec::new(),
            type_counts: HashMap::new(),
        }
    }

    /// Record type observation
    pub fn record(&mut self, type_id: TypeId) {
        self.observations.push(type_id);
        *self.type_counts.entry(type_id).or_insert(0) += 1;
        
        // Keep only recent observations
        if self.observations.len() > 100 {
            self.observations.drain(0..50);
        }
    }

    /// Check if variable has stable type
    pub fn is_monomorphic(&self) -> bool {
        self.type_counts.len() == 1 && self.observations.len() > 5
    }

    /// Get dominant type
    pub fn dominant_type(&self) -> Option<TypeId> {
        self.type_counts
            .iter()
            .max_by_key(|(_, c)| *c)
            .map(|(t, _)| *t)
    }
}

/// Feedback for call sites
#[derive(Debug, Clone)]
pub struct CallSiteFeedback {
    location: usize,
    observations: Vec<Vec<TypeId>>,
    signature_counts: HashMap<Vec<TypeId>, usize>,
}

impl CallSiteFeedback {
    /// Create new call site feedback
    pub fn new(location: usize) -> Self {
        Self {
            location,
            observations: Vec::new(),
            signature_counts: HashMap::new(),
        }
    }

    /// Record call signature
    pub fn record(&mut self, arg_types: Vec<TypeId>) {
        self.observations.push(arg_types.clone());
        *self.signature_counts.entry(arg_types).or_insert(0) += 1;
        
        // Keep only recent observations
        if self.observations.len() > 100 {
            self.observations.drain(0..50);
        }
    }

    /// Check if call site has stable signature
    pub fn is_monomorphic(&self) -> bool {
        self.signature_counts.len() == 1 && self.observations.len() > 5
    }

    /// Get dominant signature
    pub fn dominant_signature(&self) -> Option<&Vec<TypeId>> {
        self.signature_counts
            .iter()
            .max_by_key(|(_, c)| *c)
            .map(|(sig, _)| sig)
    }
}

/// Specialization candidate
#[derive(Debug, Clone)]
pub struct SpecializationCandidate {
    pub location: usize,
    pub kind: SpecializationKind,
    pub confidence: f64,
}

/// Kind of specialization
#[derive(Debug, Clone)]
pub enum SpecializationKind {
    BinaryOp { left: TypeId, right: TypeId },
    UnaryOp { operand: TypeId },
    FunctionCall { signature: Vec<TypeId> },
    PropertyAccess { object: TypeId, property: String },
}

/// Type feedback statistics
#[derive(Debug, Clone)]
pub struct TypeFeedbackStats {
    pub binary_ops_tracked: usize,
    pub variables_tracked: usize,
    pub call_sites_tracked: usize,
    pub specialization_candidates: usize,
}