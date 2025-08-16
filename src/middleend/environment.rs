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
    pub fn new() -> Self {
        TypeEnv {
            bindings: HashMap::new(),
        }
    }

    /// Create a standard environment with built-in functions
    #[must_use]
    pub fn standard() -> Self {
        let mut env = Self::new();

        // Arithmetic functions
        env.bind(
            "add",
            TypeScheme::mono(MonoType::Function(
                Box::new(MonoType::Int),
                Box::new(MonoType::Function(Box::new(MonoType::Int), Box::new(MonoType::Int))),
            )),
        );

        // IO functions
        env.bind("print", TypeScheme::mono(MonoType::Function(
            Box::new(MonoType::String),
            Box::new(MonoType::Unit),
        )));

        env.bind("println", TypeScheme::mono(MonoType::Function(
            Box::new(MonoType::String),
            Box::new(MonoType::Unit),
        )));

        // Comparison functions
        env.bind(
            "eq",
            TypeScheme::mono(MonoType::Function(
                Box::new(MonoType::Int),
                Box::new(MonoType::Function(Box::new(MonoType::Int), Box::new(MonoType::Bool))),
            )),
        );

        env
    }

    /// Bind a name to a type scheme
    pub fn bind(&mut self, name: impl Into<String>, scheme: TypeScheme) {
        self.bindings.insert(name.into(), scheme);
    }

    /// Look up a name in the environment
    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<&TypeScheme> {
        self.bindings.get(name)
    }

    /// Extend the environment with a new binding (functional style)
    #[must_use]
    pub fn extend(&self, name: impl Into<String>, scheme: TypeScheme) -> Self {
        let mut new_env = self.clone();
        new_env.bind(name, scheme);
        new_env
    }

    /// Get free type variables in the environment
    #[must_use] pub fn free_vars(&self) -> Vec<crate::middleend::types::TyVar> {
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
    pub fn generalize(&self, ty: MonoType) -> TypeScheme {
        let ty_vars = ty.free_vars();
        let env_vars = self.free_vars();

        // Variables to generalize are those in ty but not in env
        let gen_vars: Vec<_> = ty_vars
            .into_iter()
            .filter(|v| !env_vars.contains(v))
            .collect();

        TypeScheme {
            vars: gen_vars,
            ty,
        }
    }

    /// Instantiate a type scheme with fresh variables
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
        env.bind(
            "y",
            TypeScheme::mono(MonoType::Var(var.clone())),
        );

        // Try to generalize a type with the same variable
        let ty = MonoType::Function(
            Box::new(MonoType::Var(var)),
            Box::new(MonoType::Int),
        );

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
}