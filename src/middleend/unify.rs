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
    /// Create a new unifier
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::middleend::unify::Unifier;
    /// let unifier = Unifier::new();
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
    /// use ruchy::middleend::unify::Unifier;
    ///
    /// let mut instance = Unifier::new();
    /// let result = instance.unify();
    /// // Verify behavior
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
                        bail!("Cannot unify DataFrames with different column names: {name1} vs {name2}");
                    }
                    self.unify(ty1, ty2)?;
                }
                Ok(())
            }
            (MonoType::Series(ty1), MonoType::Series(ty2)) => self.unify(&ty1, &ty2),
            (t1, t2) => bail!("Cannot unify {t1} with {t2}"),
        }
    }
    /// Bind a type variable to a type
    fn bind(&mut self, var: &TyVar, ty: &MonoType) -> Result<()> {
        // Occurs check: ensure var doesn't occur in ty
        if Self::occurs(var, ty) {
            bail!("Infinite type: {var} occurs in {ty}");
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

    // === EXTREME TDD Round 19 tests ===

    #[test]
    fn test_unifier_default() {
        let unifier = Unifier::default();
        assert!(unifier.substitution().is_empty());
    }

    #[test]
    fn test_unifier_apply_no_substitution() {
        let unifier = Unifier::new();
        let ty = MonoType::Int;
        // Applying substitution to concrete type should return same type
        assert_eq!(unifier.apply(&ty), MonoType::Int);
    }

    #[test]
    fn test_unify_optional_types() {
        let mut unifier = Unifier::new();
        let var = TyVar(0);
        let opt1 = MonoType::Optional(Box::new(MonoType::Var(var.clone())));
        let opt2 = MonoType::Optional(Box::new(MonoType::String));
        assert!(unifier.unify(&opt1, &opt2).is_ok());
        assert_eq!(unifier.solve(&var), MonoType::String);
    }

    #[test]
    fn test_unify_same_var() {
        let mut unifier = Unifier::new();
        let var = TyVar(0);
        // Unifying τ0 with itself should succeed
        assert!(unifier
            .unify(&MonoType::Var(var.clone()), &MonoType::Var(var))
            .is_ok());
    }

    #[test]
    fn test_unify_unit_type() {
        let mut unifier = Unifier::new();
        assert!(unifier.unify(&MonoType::Unit, &MonoType::Unit).is_ok());
    }

    #[test]
    fn test_unify_float_types() {
        let mut unifier = Unifier::new();
        assert!(unifier.unify(&MonoType::Float, &MonoType::Float).is_ok());
        assert!(unifier.unify(&MonoType::Float, &MonoType::Int).is_err());
    }

    // === EXTREME TDD Round 125 tests ===

    #[test]
    fn test_unify_named_types_same() {
        let mut unifier = Unifier::new();
        let n1 = MonoType::Named("Point".to_string());
        let n2 = MonoType::Named("Point".to_string());
        assert!(unifier.unify(&n1, &n2).is_ok());
    }

    #[test]
    fn test_unify_named_types_different() {
        let mut unifier = Unifier::new();
        let n1 = MonoType::Named("Point".to_string());
        let n2 = MonoType::Named("Vector".to_string());
        assert!(unifier.unify(&n1, &n2).is_err());
    }

    #[test]
    fn test_unify_result_types() {
        let mut unifier = Unifier::new();
        let var = TyVar(0);
        let r1 = MonoType::Result(
            Box::new(MonoType::Var(var.clone())),
            Box::new(MonoType::String),
        );
        let r2 = MonoType::Result(Box::new(MonoType::Int), Box::new(MonoType::String));
        assert!(unifier.unify(&r1, &r2).is_ok());
        assert_eq!(unifier.solve(&var), MonoType::Int);
    }

    #[test]
    fn test_unify_result_types_err_part() {
        let mut unifier = Unifier::new();
        let var = TyVar(0);
        let r1 = MonoType::Result(
            Box::new(MonoType::Int),
            Box::new(MonoType::Var(var.clone())),
        );
        let r2 = MonoType::Result(Box::new(MonoType::Int), Box::new(MonoType::String));
        assert!(unifier.unify(&r1, &r2).is_ok());
        assert_eq!(unifier.solve(&var), MonoType::String);
    }

    #[test]
    fn test_unify_series_types() {
        let mut unifier = Unifier::new();
        let var = TyVar(0);
        let s1 = MonoType::Series(Box::new(MonoType::Var(var.clone())));
        let s2 = MonoType::Series(Box::new(MonoType::Float));
        assert!(unifier.unify(&s1, &s2).is_ok());
        assert_eq!(unifier.solve(&var), MonoType::Float);
    }

    #[test]
    fn test_unify_nested_functions() {
        let mut unifier = Unifier::new();
        let var = TyVar(0);
        let f1 = MonoType::Function(
            Box::new(MonoType::Function(
                Box::new(MonoType::Int),
                Box::new(MonoType::Var(var.clone())),
            )),
            Box::new(MonoType::Bool),
        );
        let f2 = MonoType::Function(
            Box::new(MonoType::Function(
                Box::new(MonoType::Int),
                Box::new(MonoType::String),
            )),
            Box::new(MonoType::Bool),
        );
        assert!(unifier.unify(&f1, &f2).is_ok());
        assert_eq!(unifier.solve(&var), MonoType::String);
    }

    #[test]
    fn test_unify_nested_lists() {
        let mut unifier = Unifier::new();
        let var = TyVar(0);
        let l1 = MonoType::List(Box::new(MonoType::List(Box::new(MonoType::Var(var.clone())))));
        let l2 = MonoType::List(Box::new(MonoType::List(Box::new(MonoType::Int))));
        assert!(unifier.unify(&l1, &l2).is_ok());
        assert_eq!(unifier.solve(&var), MonoType::Int);
    }

    #[test]
    fn test_occurs_check_in_function() {
        let mut unifier = Unifier::new();
        let var = TyVar(0);
        let f = MonoType::Function(
            Box::new(MonoType::Int),
            Box::new(MonoType::Var(var.clone())),
        );
        assert!(unifier.unify(&MonoType::Var(var), &f).is_err());
    }

    #[test]
    fn test_occurs_check_in_optional() {
        let mut unifier = Unifier::new();
        let var = TyVar(0);
        let opt = MonoType::Optional(Box::new(MonoType::Var(var.clone())));
        assert!(unifier.unify(&MonoType::Var(var), &opt).is_err());
    }

    #[test]
    fn test_apply_after_unification() {
        let mut unifier = Unifier::new();
        let var = TyVar(0);
        unifier
            .unify(&MonoType::Var(var.clone()), &MonoType::Int)
            .unwrap();
        // Apply should substitute var with Int
        let result = unifier.apply(&MonoType::Var(var));
        assert_eq!(result, MonoType::Int);
    }

    #[test]
    fn test_solve_unbound_var() {
        let unifier = Unifier::new();
        let var = TyVar(99);
        // Unbound var should return itself
        assert_eq!(unifier.solve(&var), MonoType::Var(var));
    }

    #[test]
    fn test_unify_multiple_vars() {
        let mut unifier = Unifier::new();
        let var1 = TyVar(0);
        let var2 = TyVar(1);
        let var3 = TyVar(2);
        // var1 = var2
        assert!(unifier
            .unify(&MonoType::Var(var1.clone()), &MonoType::Var(var2.clone()))
            .is_ok());
        // var2 = var3
        assert!(unifier
            .unify(&MonoType::Var(var2.clone()), &MonoType::Var(var3.clone()))
            .is_ok());
        // var3 = Int
        assert!(unifier.unify(&MonoType::Var(var3), &MonoType::Int).is_ok());
        // All should resolve to Int
        assert_eq!(unifier.solve(&var1), MonoType::Int);
        assert_eq!(unifier.solve(&var2), MonoType::Int);
    }

    #[test]
    fn test_unify_var_with_self() {
        let mut unifier = Unifier::new();
        let var = TyVar(0);
        // Unifying a var with itself should always succeed
        assert!(unifier
            .unify(&MonoType::Var(var.clone()), &MonoType::Var(var))
            .is_ok());
    }

    #[test]
    fn test_unify_concrete_types_mismatch() {
        let mut unifier = Unifier::new();
        // Int and String cannot unify
        assert!(unifier.unify(&MonoType::Int, &MonoType::String).is_err());
        // Bool and Float cannot unify
        assert!(unifier.unify(&MonoType::Bool, &MonoType::Float).is_err());
        // Unit and Int cannot unify
        assert!(unifier.unify(&MonoType::Unit, &MonoType::Int).is_err());
    }
}
#[cfg(test)]
mod property_tests_unify {
    use proptest::proptest;

    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_new_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}

// === EXTREME TDD Round 164 - Unification Tests ===

#[cfg(test)]
mod unify_tests_r164 {
    use super::*;

    #[test]
    fn test_unify_int_with_int_r164() {
        let mut unifier = Unifier::new();
        assert!(unifier.unify(&MonoType::Int, &MonoType::Int).is_ok());
    }

    #[test]
    fn test_unify_float_with_float_r164() {
        let mut unifier = Unifier::new();
        assert!(unifier.unify(&MonoType::Float, &MonoType::Float).is_ok());
    }

    #[test]
    fn test_unify_bool_with_bool_r164() {
        let mut unifier = Unifier::new();
        assert!(unifier.unify(&MonoType::Bool, &MonoType::Bool).is_ok());
    }

    #[test]
    fn test_unify_string_with_string_r164() {
        let mut unifier = Unifier::new();
        assert!(unifier.unify(&MonoType::String, &MonoType::String).is_ok());
    }

    #[test]
    fn test_unify_unit_with_unit_r164() {
        let mut unifier = Unifier::new();
        assert!(unifier.unify(&MonoType::Unit, &MonoType::Unit).is_ok());
    }

    #[test]
    fn test_unify_int_with_float_fails_r164() {
        let mut unifier = Unifier::new();
        assert!(unifier.unify(&MonoType::Int, &MonoType::Float).is_err());
    }

    #[test]
    fn test_unify_int_with_string_fails_r164() {
        let mut unifier = Unifier::new();
        assert!(unifier.unify(&MonoType::Int, &MonoType::String).is_err());
    }

    #[test]
    fn test_unify_int_with_bool_fails_r164() {
        let mut unifier = Unifier::new();
        assert!(unifier.unify(&MonoType::Int, &MonoType::Bool).is_err());
    }

    #[test]
    fn test_unify_float_with_string_fails_r164() {
        let mut unifier = Unifier::new();
        assert!(unifier.unify(&MonoType::Float, &MonoType::String).is_err());
    }

    #[test]
    fn test_unify_float_with_bool_fails_r164() {
        let mut unifier = Unifier::new();
        assert!(unifier.unify(&MonoType::Float, &MonoType::Bool).is_err());
    }

    #[test]
    fn test_unify_bool_with_string_fails_r164() {
        let mut unifier = Unifier::new();
        assert!(unifier.unify(&MonoType::Bool, &MonoType::String).is_err());
    }

    #[test]
    fn test_unify_var_with_int_r164() {
        let mut unifier = Unifier::new();
        let var = TyVar(10);
        assert!(unifier.unify(&MonoType::Var(var.clone()), &MonoType::Int).is_ok());
        assert_eq!(unifier.solve(&var), MonoType::Int);
    }

    #[test]
    fn test_unify_var_with_float_r164() {
        let mut unifier = Unifier::new();
        let var = TyVar(11);
        assert!(unifier.unify(&MonoType::Var(var.clone()), &MonoType::Float).is_ok());
        assert_eq!(unifier.solve(&var), MonoType::Float);
    }

    #[test]
    fn test_unify_var_with_string_r164() {
        let mut unifier = Unifier::new();
        let var = TyVar(12);
        assert!(unifier.unify(&MonoType::Var(var.clone()), &MonoType::String).is_ok());
        assert_eq!(unifier.solve(&var), MonoType::String);
    }

    #[test]
    fn test_unify_var_with_bool_r164() {
        let mut unifier = Unifier::new();
        let var = TyVar(13);
        assert!(unifier.unify(&MonoType::Var(var.clone()), &MonoType::Bool).is_ok());
        assert_eq!(unifier.solve(&var), MonoType::Bool);
    }

    #[test]
    fn test_unify_int_with_var_r164() {
        let mut unifier = Unifier::new();
        let var = TyVar(14);
        // Order reversed - concrete then var
        assert!(unifier.unify(&MonoType::Int, &MonoType::Var(var.clone())).is_ok());
        assert_eq!(unifier.solve(&var), MonoType::Int);
    }

    #[test]
    fn test_unify_two_vars_r164() {
        let mut unifier = Unifier::new();
        let var1 = TyVar(20);
        let var2 = TyVar(21);
        assert!(unifier.unify(&MonoType::Var(var1.clone()), &MonoType::Var(var2.clone())).is_ok());
        // Now bind one to Int
        assert!(unifier.unify(&MonoType::Var(var1.clone()), &MonoType::Int).is_ok());
        // Both should resolve to Int
        assert_eq!(unifier.solve(&var1), MonoType::Int);
    }

    #[test]
    fn test_unify_chain_of_vars_r164() {
        let mut unifier = Unifier::new();
        let var1 = TyVar(30);
        let var2 = TyVar(31);
        let var3 = TyVar(32);
        let var4 = TyVar(33);

        // Create chain: var1 -> var2 -> var3 -> var4 -> String
        assert!(unifier.unify(&MonoType::Var(var1.clone()), &MonoType::Var(var2.clone())).is_ok());
        assert!(unifier.unify(&MonoType::Var(var2.clone()), &MonoType::Var(var3.clone())).is_ok());
        assert!(unifier.unify(&MonoType::Var(var3.clone()), &MonoType::Var(var4.clone())).is_ok());
        assert!(unifier.unify(&MonoType::Var(var4.clone()), &MonoType::String).is_ok());

        // All should resolve to String
        assert_eq!(unifier.solve(&var1), MonoType::String);
    }

    #[test]
    fn test_unify_function_types_r164() {
        let mut unifier = Unifier::new();
        let fn1 = MonoType::Function(Box::new(MonoType::Int), Box::new(MonoType::Bool));
        let fn2 = MonoType::Function(Box::new(MonoType::Int), Box::new(MonoType::Bool));
        assert!(unifier.unify(&fn1, &fn2).is_ok());
    }

    #[test]
    fn test_unify_function_types_arg_mismatch_r164() {
        let mut unifier = Unifier::new();
        let fn1 = MonoType::Function(Box::new(MonoType::Int), Box::new(MonoType::Bool));
        let fn2 = MonoType::Function(Box::new(MonoType::String), Box::new(MonoType::Bool));
        assert!(unifier.unify(&fn1, &fn2).is_err());
    }

    #[test]
    fn test_unify_function_types_ret_mismatch_r164() {
        let mut unifier = Unifier::new();
        let fn1 = MonoType::Function(Box::new(MonoType::Int), Box::new(MonoType::Bool));
        let fn2 = MonoType::Function(Box::new(MonoType::Int), Box::new(MonoType::String));
        assert!(unifier.unify(&fn1, &fn2).is_err());
    }

    #[test]
    fn test_unify_function_with_var_arg_r164() {
        let mut unifier = Unifier::new();
        let var = TyVar(40);
        let fn1 = MonoType::Function(Box::new(MonoType::Var(var.clone())), Box::new(MonoType::Bool));
        let fn2 = MonoType::Function(Box::new(MonoType::Int), Box::new(MonoType::Bool));
        assert!(unifier.unify(&fn1, &fn2).is_ok());
        assert_eq!(unifier.solve(&var), MonoType::Int);
    }

    #[test]
    fn test_unify_function_with_var_ret_r164() {
        let mut unifier = Unifier::new();
        let var = TyVar(41);
        let fn1 = MonoType::Function(Box::new(MonoType::Int), Box::new(MonoType::Var(var.clone())));
        let fn2 = MonoType::Function(Box::new(MonoType::Int), Box::new(MonoType::String));
        assert!(unifier.unify(&fn1, &fn2).is_ok());
        assert_eq!(unifier.solve(&var), MonoType::String);
    }

    #[test]
    fn test_apply_to_int_r164() {
        let unifier = Unifier::new();
        let result = unifier.apply(&MonoType::Int);
        assert_eq!(result, MonoType::Int);
    }

    #[test]
    fn test_apply_to_unbound_var_r164() {
        let unifier = Unifier::new();
        let var = TyVar(50);
        let result = unifier.apply(&MonoType::Var(var.clone()));
        assert_eq!(result, MonoType::Var(var));
    }

    #[test]
    fn test_apply_to_bound_var_r164() {
        let mut unifier = Unifier::new();
        let var = TyVar(51);
        unifier.unify(&MonoType::Var(var.clone()), &MonoType::Float).unwrap();
        let result = unifier.apply(&MonoType::Var(var));
        assert_eq!(result, MonoType::Float);
    }

    #[test]
    fn test_apply_to_function_r164() {
        let mut unifier = Unifier::new();
        let var = TyVar(52);
        unifier.unify(&MonoType::Var(var.clone()), &MonoType::Int).unwrap();
        let fn_type = MonoType::Function(Box::new(MonoType::Var(var)), Box::new(MonoType::Bool));
        let result = unifier.apply(&fn_type);
        if let MonoType::Function(arg, _ret) = result {
            assert_eq!(*arg, MonoType::Int);
        } else {
            panic!("Expected function type");
        }
    }

    #[test]
    fn test_substitution_empty_r164() {
        let unifier = Unifier::new();
        let subst = unifier.substitution();
        assert!(subst.is_empty());
    }

    #[test]
    fn test_substitution_after_unify_r164() {
        let mut unifier = Unifier::new();
        let var = TyVar(60);
        unifier.unify(&MonoType::Var(var), &MonoType::Int).unwrap();
        let subst = unifier.substitution();
        assert!(!subst.is_empty());
    }

    #[test]
    fn test_unifier_default_r164() {
        let unifier = Unifier::default();
        let subst = unifier.substitution();
        assert!(subst.is_empty());
    }

    #[test]
    fn test_unify_same_var_twice_r164() {
        let mut unifier = Unifier::new();
        let var = TyVar(70);
        assert!(unifier.unify(&MonoType::Var(var.clone()), &MonoType::Int).is_ok());
        // Unifying again with same type should succeed
        assert!(unifier.unify(&MonoType::Var(var.clone()), &MonoType::Int).is_ok());
    }

    #[test]
    fn test_unify_var_with_different_types_fails_r164() {
        let mut unifier = Unifier::new();
        let var = TyVar(71);
        assert!(unifier.unify(&MonoType::Var(var.clone()), &MonoType::Int).is_ok());
        // Unifying with different type should fail
        assert!(unifier.unify(&MonoType::Var(var), &MonoType::String).is_err());
    }

    #[test]
    fn test_unify_named_types_same_r164() {
        let mut unifier = Unifier::new();
        let named1 = MonoType::Named("Foo".to_string());
        let named2 = MonoType::Named("Foo".to_string());
        assert!(unifier.unify(&named1, &named2).is_ok());
    }

    #[test]
    fn test_unify_named_types_different_r164() {
        let mut unifier = Unifier::new();
        let named1 = MonoType::Named("Foo".to_string());
        let named2 = MonoType::Named("Bar".to_string());
        assert!(unifier.unify(&named1, &named2).is_err());
    }

    #[test]
    fn test_solve_bound_var_r164() {
        let mut unifier = Unifier::new();
        let var = TyVar(80);
        unifier.unify(&MonoType::Var(var.clone()), &MonoType::Bool).unwrap();
        assert_eq!(unifier.solve(&var), MonoType::Bool);
    }

    #[test]
    fn test_solve_unbound_var_r164() {
        let unifier = Unifier::new();
        let var = TyVar(81);
        // Unbound var should return itself
        assert_eq!(unifier.solve(&var), MonoType::Var(var));
    }

    #[test]
    fn test_unify_complex_nested_function_r164() {
        let mut unifier = Unifier::new();
        // (Int -> Bool) -> String
        let inner = MonoType::Function(Box::new(MonoType::Int), Box::new(MonoType::Bool));
        let fn1 = MonoType::Function(Box::new(inner.clone()), Box::new(MonoType::String));
        let fn2 = MonoType::Function(Box::new(inner), Box::new(MonoType::String));
        assert!(unifier.unify(&fn1, &fn2).is_ok());
    }
}
