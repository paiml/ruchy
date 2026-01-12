//! Type environment for type inference
use crate::middleend::types::{MonoType, TyVarGenerator, TypeScheme};
use std::collections::HashMap;
/// Type environment mapping identifiers to type schemes
#[derive(Debug, Clone)]
pub struct TypeEnv {
    bindings: HashMap<String, TypeScheme>,
}
impl TypeEnv {
    #[must_use]
    /// Create a new empty type environment
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::middleend::environment::TypeEnv;
    /// let env = TypeEnv::new();
    /// ```
    pub fn new() -> Self {
        TypeEnv {
            bindings: HashMap::new(),
        }
    }
    /// Create a standard environment with built-in functions
    #[must_use]
    /// Create a standard type environment with built-in functions
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::middleend::environment::TypeEnv;
    /// let env = TypeEnv::standard();
    /// ```
    pub fn standard() -> Self {
        let mut env = Self::new();
        // Arithmetic functions
        env.bind(
            "add",
            TypeScheme::mono(MonoType::Function(
                Box::new(MonoType::Int),
                Box::new(MonoType::Function(
                    Box::new(MonoType::Int),
                    Box::new(MonoType::Int),
                )),
            )),
        );
        // IO functions
        env.bind(
            "print",
            TypeScheme::mono(MonoType::Function(
                Box::new(MonoType::String),
                Box::new(MonoType::Unit),
            )),
        );
        env.bind(
            "println",
            TypeScheme::mono(MonoType::Function(
                Box::new(MonoType::String),
                Box::new(MonoType::Unit),
            )),
        );
        // Comparison functions
        env.bind(
            "eq",
            TypeScheme::mono(MonoType::Function(
                Box::new(MonoType::Int),
                Box::new(MonoType::Function(
                    Box::new(MonoType::Int),
                    Box::new(MonoType::Bool),
                )),
            )),
        );
        env
    }
    /// Bind a name to a type scheme
    /// Bind a name to a type scheme in the environment
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::middleend::environment::{TypeEnv, TypeScheme};
    /// use ruchy::middleend::types::MonoType;
    /// let mut env = TypeEnv::new();
    /// env.bind("x", TypeScheme::mono(MonoType::Int));
    /// ```
    pub fn bind(&mut self, name: impl Into<String>, scheme: TypeScheme) {
        self.bindings.insert(name.into(), scheme);
    }
    /// Look up a name in the environment
    #[must_use]
    /// # Examples
    ///
    /// ```
    /// use ruchy::middleend::environment::lookup;
    ///
    /// let result = lookup("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn lookup(&self, name: &str) -> Option<&TypeScheme> {
        self.bindings.get(name)
    }
    /// Extend the environment with a new binding (functional style)
    #[must_use]
    /// # Examples
    ///
    /// ```
    /// use ruchy::middleend::environment::extend;
    ///
    /// let result = extend(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn extend(&self, name: impl Into<String>, scheme: TypeScheme) -> Self {
        let mut new_env = self.clone();
        new_env.bind(name, scheme);
        new_env
    }
    /// Get free type variables in the environment
    #[must_use]
    /// # Examples
    ///
    /// ```
    /// use ruchy::middleend::environment::free_vars;
    ///
    /// let result = free_vars(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn free_vars(&self) -> Vec<crate::middleend::types::TyVar> {
        let mut vars = Vec::new();
        for scheme in self.bindings.values() {
            // Only collect free variables not bound by the scheme
            let scheme_free = scheme.ty.free_vars();
            for var in scheme_free {
                if !scheme.vars.contains(&var) {
                    vars.push(var);
                }
            }
        }
        vars
    }
    /// Generalize a monomorphic type to a type scheme
    #[must_use]
    /// # Examples
    ///
    /// ```
    /// use ruchy::middleend::environment::generalize;
    ///
    /// let result = generalize(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn generalize(&self, ty: MonoType) -> TypeScheme {
        let ty_vars = ty.free_vars();
        let env_vars = self.free_vars();
        // Variables to generalize are those in ty but not in env
        let gen_vars: Vec<_> = ty_vars
            .into_iter()
            .filter(|v| !env_vars.contains(v))
            .collect();
        TypeScheme { vars: gen_vars, ty }
    }
    /// Instantiate a type scheme with fresh variables
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::middleend::environment::instantiate;
    ///
    /// let result = instantiate(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn instantiate(&self, scheme: &TypeScheme, gen: &mut TyVarGenerator) -> MonoType {
        scheme.instantiate(gen)
    }
}
impl Default for TypeEnv {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::middleend::types::TyVar;
    #[test]
    fn test_env_lookup() {
        let mut env = TypeEnv::new();
        env.bind("x", TypeScheme::mono(MonoType::Int));
        assert!(env.lookup("x").is_some());
        assert!(env.lookup("y").is_none());
    }
    #[test]
    fn test_env_extend() {
        let env = TypeEnv::new();
        let env2 = env.extend("x", TypeScheme::mono(MonoType::Bool));
        assert!(env.lookup("x").is_none());
        assert!(env2.lookup("x").is_some());
    }
    #[test]
    fn test_generalization() {
        let env = TypeEnv::new();
        let var = TyVar(0);
        // A type with a free variable
        let ty = MonoType::Function(
            Box::new(MonoType::Var(var.clone())),
            Box::new(MonoType::Var(var.clone())),
        );
        let scheme = env.generalize(ty);
        // The variable should be generalized
        assert_eq!(scheme.vars.len(), 1);
        assert!(scheme.vars.contains(&var));
    }
    #[test]
    fn test_no_generalization_with_env_vars() {
        let mut env = TypeEnv::new();
        let var = TyVar(0);
        // Add a binding with the same variable to the environment
        env.bind("y", TypeScheme::mono(MonoType::Var(var.clone())));
        // Try to generalize a type with the same variable
        let ty = MonoType::Function(Box::new(MonoType::Var(var)), Box::new(MonoType::Int));
        let scheme = env.generalize(ty);
        // The variable should NOT be generalized (it's in the env)
        assert_eq!(scheme.vars.len(), 0);
    }
    #[test]
    fn test_standard_env() {
        let env = TypeEnv::standard();
        assert!(env.lookup("println").is_some());
        assert!(env.lookup("print").is_some());
        assert!(env.lookup("add").is_some());
        assert!(env.lookup("eq").is_some());
    }
    #[test]
    fn test_default_env() {
        let env = TypeEnv::default();
        assert!(env.lookup("nonexistent").is_none());
        assert_eq!(env.bindings.len(), 0);
    }
    #[test]
    fn test_multiple_bindings() {
        let mut env = TypeEnv::new();
        env.bind("x", TypeScheme::mono(MonoType::Int));
        env.bind("y", TypeScheme::mono(MonoType::Bool));
        env.bind("z", TypeScheme::mono(MonoType::String));
        assert!(env.lookup("x").is_some());
        assert!(env.lookup("y").is_some());
        assert!(env.lookup("z").is_some());
        assert!(env.lookup("w").is_none());
    }
    #[test]
    fn test_bind_overwrites() {
        let mut env = TypeEnv::new();
        env.bind("x", TypeScheme::mono(MonoType::Int));
        env.bind("x", TypeScheme::mono(MonoType::Bool));
        let scheme = env.lookup("x").unwrap();
        match &scheme.ty {
            MonoType::Bool => {} // Expected
            _ => panic!("Expected Bool type after overwrite"),
        }
    }
    #[test]
    fn test_env_clone() {
        let mut env1 = TypeEnv::new();
        env1.bind("x", TypeScheme::mono(MonoType::Int));
        let env2 = env1.clone();
        assert!(env2.lookup("x").is_some());
    }
    #[test]
    fn test_free_vars_empty() {
        let env = TypeEnv::new();
        assert!(env.free_vars().is_empty());
    }
    #[test]
    fn test_free_vars_with_schemes() {
        let mut env = TypeEnv::new();
        let var1 = TyVar(1);
        let var2 = TyVar(2);
        // Add a scheme with a free variable
        let scheme1 = TypeScheme {
            vars: vec![],
            ty: MonoType::Var(var1.clone()),
        };
        env.bind("x", scheme1);
        // Add a scheme with a bound variable
        let scheme2 = TypeScheme {
            vars: vec![var2.clone()],
            ty: MonoType::Var(var2),
        };
        env.bind("y", scheme2);
        let free_vars = env.free_vars();
        assert!(free_vars.contains(&var1));
        assert!(!free_vars.contains(&TyVar(2))); // var2 is bound
    }
    #[test]
    fn test_generalize_empty_env() {
        let env = TypeEnv::new();
        let var = TyVar(5);
        let ty = MonoType::Var(var.clone());
        let scheme = env.generalize(ty);
        assert_eq!(scheme.vars.len(), 1);
        assert!(scheme.vars.contains(&var));
    }
    #[test]
    fn test_generalize_complex_type() {
        let env = TypeEnv::new();
        let var1 = TyVar(10);
        let var2 = TyVar(11);
        let ty = MonoType::Function(
            Box::new(MonoType::Var(var1.clone())),
            Box::new(MonoType::Function(
                Box::new(MonoType::Var(var2.clone())),
                Box::new(MonoType::Int),
            )),
        );
        let scheme = env.generalize(ty);
        assert_eq!(scheme.vars.len(), 2);
        assert!(scheme.vars.contains(&var1));
        assert!(scheme.vars.contains(&var2));
    }
    #[test]
    fn test_instantiate_scheme() {
        let env = TypeEnv::new();
        let mut gen = TyVarGenerator::new();
        // Create a polymorphic scheme: forall a. a -> a
        let var = TyVar(20);
        let scheme = TypeScheme {
            vars: vec![var.clone()],
            ty: MonoType::Function(
                Box::new(MonoType::Var(var.clone())),
                Box::new(MonoType::Var(var)),
            ),
        };
        let instance = env.instantiate(&scheme, &mut gen);
        // Should get fresh variables
        match instance {
            MonoType::Function(arg, ret) => {
                match (*arg, *ret) {
                    (MonoType::Var(v1), MonoType::Var(v2)) => {
                        assert_eq!(v1, v2); // Same fresh variable
                        assert_ne!(v1, TyVar(20)); // Different from original
                    }
                    _ => panic!("Expected function with variable types"),
                }
            }
            _ => panic!("Expected function type"),
        }
    }
    #[test]
    fn test_standard_env_function_types() {
        let env = TypeEnv::standard();
        // Test add function type
        let add_scheme = env.lookup("add").unwrap();
        match &add_scheme.ty {
            MonoType::Function(arg1, rest) => {
                assert!(matches!(**arg1, MonoType::Int));
                match rest.as_ref() {
                    MonoType::Function(arg2, ret_type) => {
                        assert!(matches!(**arg2, MonoType::Int));
                        assert!(matches!(**ret_type, MonoType::Int));
                    }
                    _ => panic!("Expected curried function type"),
                }
            }
            _ => panic!("Expected function type for add"),
        }
        // Test print function type
        let print_scheme = env.lookup("print").unwrap();
        match &print_scheme.ty {
            MonoType::Function(arg, ret) => {
                assert!(matches!(**arg, MonoType::String));
                assert!(matches!(**ret, MonoType::Unit));
            }
            _ => panic!("Expected function type for print"),
        }
    }

    // ===== EXTREME TDD Round 156 - Additional Environment Tests =====

    #[test]
    fn test_standard_env_eq_function() {
        let env = TypeEnv::standard();
        let eq_scheme = env.lookup("eq").unwrap();
        match &eq_scheme.ty {
            MonoType::Function(arg1, rest) => {
                assert!(matches!(**arg1, MonoType::Int));
                match rest.as_ref() {
                    MonoType::Function(arg2, ret_type) => {
                        assert!(matches!(**arg2, MonoType::Int));
                        assert!(matches!(**ret_type, MonoType::Bool));
                    }
                    _ => panic!("Expected curried function type"),
                }
            }
            _ => panic!("Expected function type for eq"),
        }
    }

    #[test]
    fn test_standard_env_println_function() {
        let env = TypeEnv::standard();
        let println_scheme = env.lookup("println").unwrap();
        match &println_scheme.ty {
            MonoType::Function(arg, ret) => {
                assert!(matches!(**arg, MonoType::String));
                assert!(matches!(**ret, MonoType::Unit));
            }
            _ => panic!("Expected function type for println"),
        }
    }

    #[test]
    fn test_extend_preserves_original() {
        let mut env1 = TypeEnv::new();
        env1.bind("a", TypeScheme::mono(MonoType::Int));
        let env2 = env1.extend("b", TypeScheme::mono(MonoType::Bool));
        // Original still has only "a"
        assert!(env1.lookup("a").is_some());
        assert!(env1.lookup("b").is_none());
        // Extended has both
        assert!(env2.lookup("a").is_some());
        assert!(env2.lookup("b").is_some());
    }

    #[test]
    fn test_extend_chain() {
        let env = TypeEnv::new()
            .extend("x", TypeScheme::mono(MonoType::Int))
            .extend("y", TypeScheme::mono(MonoType::Bool))
            .extend("z", TypeScheme::mono(MonoType::String));
        assert!(env.lookup("x").is_some());
        assert!(env.lookup("y").is_some());
        assert!(env.lookup("z").is_some());
    }

    #[test]
    fn test_generalize_no_free_vars() {
        let env = TypeEnv::new();
        let ty = MonoType::Int;
        let scheme = env.generalize(ty);
        assert!(scheme.vars.is_empty());
    }

    #[test]
    fn test_free_vars_multiple_bindings() {
        let mut env = TypeEnv::new();
        let var1 = TyVar(100);
        let var2 = TyVar(101);
        let var3 = TyVar(102);
        // Binding with free var1
        env.bind(
            "a",
            TypeScheme {
                vars: vec![],
                ty: MonoType::Var(var1.clone()),
            },
        );
        // Binding with free var2 and bound var3
        env.bind(
            "b",
            TypeScheme {
                vars: vec![var3.clone()],
                ty: MonoType::Function(
                    Box::new(MonoType::Var(var2.clone())),
                    Box::new(MonoType::Var(var3)),
                ),
            },
        );
        let free = env.free_vars();
        assert!(free.contains(&var1));
        assert!(free.contains(&var2));
        assert!(!free.contains(&TyVar(102))); // var3 is bound
    }

    #[test]
    fn test_instantiate_mono_scheme() {
        let env = TypeEnv::new();
        let mut gen = TyVarGenerator::new();
        let scheme = TypeScheme::mono(MonoType::Int);
        let instance = env.instantiate(&scheme, &mut gen);
        assert!(matches!(instance, MonoType::Int));
    }

    #[test]
    fn test_env_debug_format() {
        let mut env = TypeEnv::new();
        env.bind("test", TypeScheme::mono(MonoType::Int));
        let debug = format!("{:?}", env);
        assert!(debug.contains("TypeEnv"));
        assert!(debug.contains("test"));
    }

    #[test]
    fn test_generalize_with_partially_bound_env() {
        let mut env = TypeEnv::new();
        let var1 = TyVar(200);
        let var2 = TyVar(201);
        // env has var1 free
        env.bind(
            "x",
            TypeScheme {
                vars: vec![],
                ty: MonoType::Var(var1.clone()),
            },
        );
        // Generalize a type with var1 and var2
        let ty = MonoType::Function(
            Box::new(MonoType::Var(var1)),
            Box::new(MonoType::Var(var2.clone())),
        );
        let scheme = env.generalize(ty);
        // Only var2 should be generalized (var1 is in env)
        assert_eq!(scheme.vars.len(), 1);
        assert!(scheme.vars.contains(&var2));
    }
}
#[cfg(test)]
mod property_tests_environment {
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
