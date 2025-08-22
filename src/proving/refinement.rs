//! Refinement type system for property verification

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use super::smt::{SmtSolver, SmtBackend, SmtResult};

/// Refinement type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinementType {
    /// Base type
    pub base: BaseType,
    
    /// Refinement predicate
    pub predicate: Option<Predicate>,
    
    /// Type parameters
    pub params: Vec<String>,
}

/// Base types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BaseType {
    Int,
    Bool,
    String,
    Float,
    Array(Box<BaseType>),
    Tuple(Vec<BaseType>),
    Function(Vec<BaseType>, Box<BaseType>),
    Custom(String),
}

/// Refinement predicate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Predicate {
    /// Variable binding
    pub var: String,
    
    /// Predicate expression
    pub expr: String,
}

impl RefinementType {
    /// Create integer with bounds
    pub fn bounded_int(min: i64, max: i64) -> Self {
        Self {
            base: BaseType::Int,
            predicate: Some(Predicate {
                var: "x".to_string(),
                expr: format!("(and (>= x {}) (<= x {}))", min, max),
            }),
            params: Vec::new(),
        }
    }
    
    /// Create positive integer
    pub fn positive_int() -> Self {
        Self {
            base: BaseType::Int,
            predicate: Some(Predicate {
                var: "x".to_string(),
                expr: "(> x 0)".to_string(),
            }),
            params: Vec::new(),
        }
    }
    
    /// Create non-empty array
    pub fn non_empty_array(elem_type: BaseType) -> Self {
        Self {
            base: BaseType::Array(Box::new(elem_type)),
            predicate: Some(Predicate {
                var: "a".to_string(),
                expr: "(> (len a) 0)".to_string(),
            }),
            params: Vec::new(),
        }
    }
    
    /// Create sorted array
    pub fn sorted_array() -> Self {
        Self {
            base: BaseType::Array(Box::new(BaseType::Int)),
            predicate: Some(Predicate {
                var: "a".to_string(),
                expr: "(sorted a)".to_string(),
            }),
            params: Vec::new(),
        }
    }
}

impl fmt::Display for RefinementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(pred) = &self.predicate {
            write!(f, "{{ {}: {} | {} }}", pred.var, self.base, pred.expr)
        } else {
            write!(f, "{}", self.base)
        }
    }
}

impl fmt::Display for BaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int => write!(f, "Int"),
            Self::Bool => write!(f, "Bool"),
            Self::String => write!(f, "String"),
            Self::Float => write!(f, "Float"),
            Self::Array(t) => write!(f, "[{}]", t),
            Self::Tuple(ts) => {
                write!(f, "(")?;
                for (i, t) in ts.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
            Self::Function(params, ret) => {
                write!(f, "(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", p)?;
                }
                write!(f, ") -> {}", ret)
            }
            Self::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Type refinement
#[derive(Debug, Clone)]
pub struct TypeRefinement {
    /// Input type
    pub input: RefinementType,
    
    /// Output type
    pub output: RefinementType,
    
    /// Preconditions
    pub preconditions: Vec<String>,
    
    /// Postconditions
    pub postconditions: Vec<String>,
    
    /// Invariants
    pub invariants: Vec<String>,
}

impl TypeRefinement {
    /// Create new refinement
    pub fn new(input: RefinementType, output: RefinementType) -> Self {
        Self {
            input,
            output,
            preconditions: Vec::new(),
            postconditions: Vec::new(),
            invariants: Vec::new(),
        }
    }
    
    /// Add precondition
    pub fn add_precondition(&mut self, pred: &str) {
        self.preconditions.push(pred.to_string());
    }
    
    /// Add postcondition
    pub fn add_postcondition(&mut self, pred: &str) {
        self.postconditions.push(pred.to_string());
    }
    
    /// Add invariant
    pub fn add_invariant(&mut self, inv: &str) {
        self.invariants.push(inv.to_string());
    }
}

/// Refinement type checker
pub struct RefinementChecker {
    /// SMT backend
    backend: SmtBackend,
    
    /// Type environment
    env: HashMap<String, RefinementType>,
    
    /// Function signatures
    signatures: HashMap<String, TypeRefinement>,
}

impl RefinementChecker {
    /// Create new checker
    pub fn new() -> Self {
        Self {
            backend: SmtBackend::Z3,
            env: HashMap::new(),
            signatures: HashMap::new(),
        }
    }
    
    /// Set SMT backend
    pub fn set_backend(&mut self, backend: SmtBackend) {
        self.backend = backend;
    }
    
    /// Declare variable
    pub fn declare_var(&mut self, name: &str, ty: RefinementType) {
        self.env.insert(name.to_string(), ty);
    }
    
    /// Declare function
    pub fn declare_function(&mut self, name: &str, refinement: TypeRefinement) {
        self.signatures.insert(name.to_string(), refinement);
    }
    
    /// Check subtyping
    pub fn is_subtype(&self, sub_type: &RefinementType, super_type: &RefinementType) -> Result<bool> {
        if sub_type.base != super_type.base {
            return Ok(false);
        }
        
        match (&sub_type.predicate, &super_type.predicate) {
            (Some(sub_pred), Some(super_pred)) => {
                self.check_implication(&sub_pred.expr, &super_pred.expr)
            }
            (Some(_), None) => Ok(true),
            (None, Some(_)) => Ok(false),
            (None, None) => Ok(true),
        }
    }
    
    /// Check implication using SMT
    fn check_implication(&self, antecedent: &str, consequent: &str) -> Result<bool> {
        let mut solver = SmtSolver::new(self.backend);
        
        solver.assert(antecedent);
        solver.assert(&format!("(not {})", consequent));
        
        match solver.check_sat()? {
            SmtResult::Unsat => Ok(true),
            _ => Ok(false),
        }
    }
    
    /// Verify function refinement
    pub fn verify_function(&self, name: &str, body: &str) -> Result<VerificationResult> {
        let refinement = self.signatures.get(name)
            .ok_or_else(|| anyhow::anyhow!("Unknown function: {}", name))?;
        
        let mut solver = SmtSolver::new(self.backend);
        
        for pre in &refinement.preconditions {
            solver.assert(pre);
        }
        
        solver.assert(body);
        
        for post in &refinement.postconditions {
            solver.assert(&format!("(not {})", post));
        }
        
        match solver.check_sat()? {
            SmtResult::Unsat => Ok(VerificationResult::Valid),
            SmtResult::Sat => Ok(VerificationResult::Invalid("Postcondition violation".to_string())),
            _ => Ok(VerificationResult::Unknown),
        }
    }
    
    /// Check invariant preservation
    pub fn check_invariant(&self, invariant: &str, body: &str) -> Result<bool> {
        let mut solver = SmtSolver::new(self.backend);
        
        solver.assert(invariant);
        
        solver.assert(body);
        
        solver.assert(&format!("(not {})", invariant));
        
        match solver.check_sat()? {
            SmtResult::Unsat => Ok(true),
            _ => Ok(false),
        }
    }
}

impl Default for RefinementChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationResult {
    Valid,
    Invalid(String),
    Unknown,
}

impl VerificationResult {
    /// Check if valid
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }
    
    /// Get error message
    pub fn error(&self) -> Option<&str> {
        match self {
            Self::Invalid(msg) => Some(msg),
            _ => None,
        }
    }
}

/// Liquid type inference
pub struct LiquidTypeInference {
    checker: RefinementChecker,
    constraints: Vec<String>,
}

impl LiquidTypeInference {
    /// Create new inference engine
    pub fn new() -> Self {
        Self {
            checker: RefinementChecker::new(),
            constraints: Vec::new(),
        }
    }
    
    /// Infer refinement type
    pub fn infer(&mut self, expr: &str) -> Result<RefinementType> {
        match expr {
            s if s.parse::<i64>().is_ok() => {
                let n = s.parse::<i64>().unwrap();
                Ok(RefinementType {
                    base: BaseType::Int,
                    predicate: Some(Predicate {
                        var: "x".to_string(),
                        expr: format!("(= x {})", n),
                    }),
                    params: Vec::new(),
                })
            }
            "true" | "false" => Ok(RefinementType {
                base: BaseType::Bool,
                predicate: None,
                params: Vec::new(),
            }),
            _ => Ok(RefinementType {
                base: BaseType::Custom("Unknown".to_string()),
                predicate: None,
                params: Vec::new(),
            }),
        }
    }
    
    /// Add constraint
    pub fn add_constraint(&mut self, constraint: &str) {
        self.constraints.push(constraint.to_string());
    }
    
    /// Solve constraints
    pub fn solve(&self) -> Result<bool> {
        let mut solver = SmtSolver::new(self.checker.backend);
        
        for constraint in &self.constraints {
            solver.assert(constraint);
        }
        
        match solver.check_sat()? {
            SmtResult::Sat => Ok(true),
            _ => Ok(false),
        }
    }
}

impl Default for LiquidTypeInference {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_refinement_type_display() {
        let ty = RefinementType::positive_int();
        assert_eq!(ty.to_string(), "{ x: Int | (> x 0) }");
        
        let bounded = RefinementType::bounded_int(0, 100);
        assert_eq!(bounded.to_string(), "{ x: Int | (and (>= x 0) (<= x 100)) }");
    }
    
    #[test]
    fn test_base_type_display() {
        assert_eq!(BaseType::Int.to_string(), "Int");
        assert_eq!(BaseType::Array(Box::new(BaseType::Int)).to_string(), "[Int]");
        
        let func = BaseType::Function(
            vec![BaseType::Int, BaseType::Bool],
            Box::new(BaseType::String)
        );
        assert_eq!(func.to_string(), "(Int, Bool) -> String");
    }
}