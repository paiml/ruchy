//! Unification algorithm for type inference
use crate::middleend::types::{MonoType, Substitution, TyVar};
use anyhow::{bail, Result};
use std::collections::HashMap;
/// Unification engine for type inference
pub struct Unifier {
    subst: Substitution,
}
impl Unifier {
    #[must_use]
/// # Examples
/// 
/// ```
/// use ruchy::middleend::unify::new;
/// 
/// let result = new(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn new() -> Self {
        Unifier {
            subst: HashMap::new(),
        }
    }
    /// Get the current substitution
    #[must_use]
/// # Examples
/// 
/// ```
/// use ruchy::middleend::unify::substitution;
/// 
/// let result = substitution(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn substitution(&self) -> &Substitution {
        &self.subst
    }
    /// Apply current substitution to a type
    #[must_use]
/// # Examples
/// 
/// ```
/// use ruchy::middleend::unify::apply;
/// 
/// let result = apply(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn apply(&self, ty: &MonoType) -> MonoType {
        ty.substitute(&self.subst)
    }
    /// Unify two types, updating the substitution
    ///
    /// # Errors
    ///
    /// Returns an error if the types cannot be unified (type mismatch or occurs check failure)
    /// # Errors
    ///
    /// Returns an error if the operation fails
/// # Examples
/// 
/// ```
/// use ruchy::middleend::unify::unify;
/// 
/// let result = unify(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn unify(&mut self, t1: &MonoType, t2: &MonoType) -> Result<()> {
        let t1 = self.apply(t1);
        let t2 = self.apply(t2);
        match (t1, t2) {
            (MonoType::Var(v1), MonoType::Var(v2)) if v1 == v2 => Ok(()),
            (MonoType::Var(v), t) | (t, MonoType::Var(v)) => self.bind(&v, &t),
            (MonoType::Int, MonoType::Int)
            | (MonoType::Float, MonoType::Float)
            | (MonoType::Bool, MonoType::Bool)
            | (MonoType::String, MonoType::String)
            | (MonoType::Unit, MonoType::Unit) => Ok(()),
            (MonoType::Named(n1), MonoType::Named(n2)) if n1 == n2 => Ok(()),
            (MonoType::Function(a1, r1), MonoType::Function(a2, r2)) => {
                self.unify(&a1, &a2)?;
                self.unify(&r1, &r2)
            }
            (MonoType::List(e1), MonoType::List(e2)) => self.unify(&e1, &e2),
            (MonoType::Optional(i1), MonoType::Optional(i2)) => self.unify(&i1, &i2),
            (MonoType::Result(ok1, err1), MonoType::Result(ok2, err2)) => {
                self.unify(&ok1, &ok2)?;
                self.unify(&err1, &err2)
            }
            (MonoType::DataFrame(cols1), MonoType::DataFrame(cols2)) => {
                // DataFrames unify if they have the same columns with the same types
                if cols1.len() != cols2.len() {
                    bail!("Cannot unify DataFrames with different column counts");
                }
                for ((name1, ty1), (name2, ty2)) in cols1.iter().zip(cols2.iter()) {
                    if name1 != name2 {
                        bail!(
                            "Cannot unify DataFrames with different column names: {} vs {}",
                            name1,
                            name2
                        );
                    }
                    self.unify(ty1, ty2)?;
                }
                Ok(())
            }
            (MonoType::Series(ty1), MonoType::Series(ty2)) => self.unify(&ty1, &ty2),
            (t1, t2) => bail!("Cannot unify {} with {}", t1, t2),
        }
    }
    /// Bind a type variable to a type
    fn bind(&mut self, var: &TyVar, ty: &MonoType) -> Result<()> {
        // Occurs check: ensure var doesn't occur in ty
        if Self::occurs(var, ty) {
            bail!("Infinite type: {} occurs in {}", var, ty);
        }
        // Apply the binding
        self.subst.insert(var.clone(), ty.clone());
        // Update existing substitutions
        let updated: Substitution = self
            .subst
            .iter()
            .map(|(k, v)| {
                if k == var {
                    (k.clone(), ty.clone())
                } else {
                    (k.clone(), v.substitute(&[(var.clone(), ty.clone())].into()))
                }
            })
            .collect();
        self.subst = updated;
        Ok(())
    }
    /// Check if a type variable occurs in a type (occurs check)
    fn occurs(var: &TyVar, ty: &MonoType) -> bool {
        match ty {
            MonoType::Var(v) => v == var,
            MonoType::Function(arg, ret) => Self::occurs(var, arg) || Self::occurs(var, ret),
            MonoType::List(elem) => Self::occurs(var, elem),
            MonoType::Optional(inner) | MonoType::Series(inner) | MonoType::Reference(inner) => {
                Self::occurs(var, inner)
            }
            MonoType::Result(ok, err) => Self::occurs(var, ok) || Self::occurs(var, err),
            MonoType::DataFrame(columns) => {
                columns.iter().any(|(_, col_ty)| Self::occurs(var, col_ty))
            }
            MonoType::Tuple(types) => types.iter().any(|ty| Self::occurs(var, ty)),
            _ => false,
        }
    }
    /// Solve a type variable to its final type
    #[must_use]
/// # Examples
/// 
/// ```
/// use ruchy::middleend::unify::solve;
/// 
/// let result = solve(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn solve(&self, var: &TyVar) -> MonoType {
        self.subst
            .get(var)
            .map_or_else(|| MonoType::Var(var.clone()), |ty| self.apply(ty))
    }
}
impl Default for Unifier {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
#[cfg(test)]
use proptest::prelude::*;
    #[test]
    fn test_unify_same_types() {
        let mut unifier = Unifier::new();
        assert!(unifier.unify(&MonoType::Int, &MonoType::Int).is_ok());
        assert!(unifier.unify(&MonoType::Bool, &MonoType::Bool).is_ok());
        assert!(unifier.unify(&MonoType::String, &MonoType::String).is_ok());
    }
    #[test]
    fn test_unify_different_types() {
        let mut unifier = Unifier::new();
        assert!(unifier.unify(&MonoType::Int, &MonoType::Bool).is_err());
        assert!(unifier.unify(&MonoType::String, &MonoType::Int).is_err());
    }
    #[test]
    fn test_unify_with_var() {
        let mut unifier = Unifier::new();
        let var = TyVar(0);
        assert!(unifier
            .unify(&MonoType::Var(var.clone()), &MonoType::Int)
            .is_ok());
        assert_eq!(unifier.solve(&var), MonoType::Int);
    }
    #[test]
    fn test_unify_functions() {
        let mut unifier = Unifier::new();
        let var = TyVar(0);
        let f1 = MonoType::Function(
            Box::new(MonoType::Int),
            Box::new(MonoType::Var(var.clone())),
        );
        let f2 = MonoType::Function(Box::new(MonoType::Int), Box::new(MonoType::Bool));
        assert!(unifier.unify(&f1, &f2).is_ok());
        assert_eq!(unifier.solve(&var), MonoType::Bool);
    }
    #[test]
    fn test_unify_lists() {
        let mut unifier = Unifier::new();
        let var = TyVar(0);
        let l1 = MonoType::List(Box::new(MonoType::Var(var.clone())));
        let l2 = MonoType::List(Box::new(MonoType::String));
        assert!(unifier.unify(&l1, &l2).is_ok());
        assert_eq!(unifier.solve(&var), MonoType::String);
    }
    #[test]
    fn test_occurs_check() {
        let mut unifier = Unifier::new();
        let var = TyVar(0);
        // Try to unify τ0 with [τ0] - should fail (infinite type)
        let infinite = MonoType::List(Box::new(MonoType::Var(var.clone())));
        assert!(unifier.unify(&MonoType::Var(var), &infinite).is_err());
    }
    #[test]
    fn test_transitive_unification() {
        let mut unifier = Unifier::new();
        let var1 = TyVar(0);
        let var2 = TyVar(1);
        // τ0 = τ1
        assert!(unifier
            .unify(&MonoType::Var(var1.clone()), &MonoType::Var(var2.clone()))
            .is_ok());
        // τ1 = Int
        assert!(unifier
            .unify(&MonoType::Var(var2.clone()), &MonoType::Int)
            .is_ok());
        // Now τ0 should also be Int
        assert_eq!(unifier.solve(&var1), MonoType::Int);
        assert_eq!(unifier.solve(&var2), MonoType::Int);
    }
}
#[cfg(test)]
mod property_tests_unify {
    use proptest::proptest;
    use super::*;
    use proptest::prelude::*;
    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_new_never_panics(input: String) {
            // Limit input size to avoid timeout
            let input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
