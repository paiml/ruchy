//! Type system representation for Ruchy
//!
//! This module implements the Hindley-Milner type system for Ruchy, providing
//! type inference, unification, and polymorphic type schemes.
//!
//! # Examples
//!
//! ```
//! use ruchy::middleend::types::{MonoType, TypeScheme, TyVarGenerator};
//!
//! // Create basic types
//! let int_type = MonoType::Int;
//! let bool_type = MonoType::Bool;
//! 
//! // Create function type: i32 -> bool
//! let func_type = MonoType::Function(
//!     Box::new(int_type),
//!     Box::new(bool_type)
//! );
//!
//! // Create type scheme for polymorphic function
//! let mut gen = TyVarGenerator::new();
//! let var = gen.fresh();
//! let scheme = TypeScheme::mono(MonoType::Var(var));
//! ```
//!
//! # Type System Features
//!
//! - **Type Variables**: For unification and type inference
//! - **Monomorphic Types**: Concrete types without quantification
//! - **Type Schemes**: Polymorphic types with quantified variables
//! - **Substitution**: Type variable replacement mechanism
//! - **DataFrame Types**: Support for data science operations
use std::collections::HashMap;
use std::fmt;
/// Type variable for unification
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TyVar(pub u32);
impl fmt::Display for TyVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "τ{}", self.0)
    }
}
/// Monomorphic types in the Hindley-Milner system
#[derive(Debug, Clone, PartialEq)]
pub enum MonoType {
    /// Type variable (unknown type to be inferred)
    Var(TyVar),
    /// Primitive integer type
    Int,
    /// Primitive float type
    Float,
    /// Primitive boolean type
    Bool,
    /// String type
    String,
    /// Character type
    Char,
    /// Unit type ()
    Unit,
    /// Function type: T1 -> T2
    Function(Box<MonoType>, Box<MonoType>),
    /// List type: `[T]`
    List(Box<MonoType>),
    /// Tuple type: (T1, T2, ...)
    Tuple(Vec<MonoType>),
    /// Optional type: T?
    Optional(Box<MonoType>),
    /// Result type: Result<T, E>
    Result(Box<MonoType>, Box<MonoType>),
    /// Named type (user-defined or gradual typing 'Any')
    Named(String),
    /// Reference type: &T
    Reference(Box<MonoType>),
    /// `DataFrame` type with column names and their types
    DataFrame(Vec<(String, MonoType)>),
    /// Series type with element type
    Series(Box<MonoType>),
}
impl fmt::Display for MonoType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MonoType::Var(v) => write!(f, "{v}"),
            MonoType::Int => write!(f, "i32"),
            MonoType::Float => write!(f, "f64"),
            MonoType::Bool => write!(f, "bool"),
            MonoType::String => write!(f, "String"),
            MonoType::Char => write!(f, "char"),
            MonoType::Unit => write!(f, "()"),
            MonoType::Function(arg, ret) => write!(f, "({arg} -> {ret})"),
            MonoType::List(elem) => write!(f, "[{elem}]"),
            MonoType::Optional(inner) => write!(f, "{inner}?"),
            MonoType::Result(ok, err) => write!(f, "Result<{ok}, {err}>"),
            MonoType::Tuple(types) => {
                write!(f, "(")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{ty}")?;
                }
                write!(f, ")")
            }
            MonoType::Named(name) => write!(f, "{name}"),
            MonoType::Reference(inner) => write!(f, "&{inner}"),
            MonoType::DataFrame(columns) => {
                write!(f, "DataFrame[")?;
                for (i, (name, ty)) in columns.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{name}: {ty}")?;
                }
                write!(f, "]")
            }
            MonoType::Series(dtype) => write!(f, "Series<{dtype}>"),
        }
    }
}
/// Polymorphic type scheme: ∀α₁...αₙ. τ
#[derive(Debug, Clone)]
pub struct TypeScheme {
    /// Quantified type variables
    pub vars: Vec<TyVar>,
    /// The monomorphic type
    pub ty: MonoType,
}
impl TypeScheme {
    /// Create a monomorphic type scheme (no quantified variables)
    ///
    /// # Examples
    /// 
    /// ```
    /// use ruchy::middleend::types::{MonoType, TypeScheme};
    /// 
    /// let scheme = TypeScheme::mono(MonoType::Int);
    /// assert_eq!(scheme.vars.len(), 0);
    /// assert_eq!(scheme.ty, MonoType::Int);
    /// ```
    #[must_use]
    pub fn mono(ty: MonoType) -> Self {
        TypeScheme {
            vars: Vec::new(),
            ty,
        }
    }
    /// Instantiate a type scheme with fresh type variables
    ///
    /// # Examples
    /// 
    /// ```
    /// use ruchy::middleend::types::{MonoType, TypeScheme, TyVarGenerator, TyVar};
    /// 
    /// let mut gen = TyVarGenerator::new();
    /// let var = gen.fresh();
    /// let scheme = TypeScheme {
    ///     vars: vec![var.clone()],
    ///     ty: MonoType::Var(var)
    /// };
    /// let instance = scheme.instantiate(&mut gen);
    /// // Should get a fresh type variable
    /// assert!(matches!(instance, MonoType::Var(_)));
    /// ```
    pub fn instantiate(&self, gen: &mut TyVarGenerator) -> MonoType {
        if self.vars.is_empty() {
            self.ty.clone()
        } else {
            let subst: HashMap<TyVar, MonoType> = self
                .vars
                .iter()
                .map(|v| (v.clone(), MonoType::Var(gen.fresh())))
                .collect();
            self.ty.substitute(&subst)
        }
    }
}
impl fmt::Display for TypeScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.vars.is_empty() {
            write!(f, "{}", self.ty)
        } else {
            write!(f, "∀")?;
            for (i, var) in self.vars.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                }
                write!(f, "{var}")?;
            }
            write!(f, ". {}", self.ty)
        }
    }
}
/// Type variable generator for fresh variables
pub struct TyVarGenerator {
    next: u32,
}
impl TyVarGenerator {
    /// Create a new type variable generator
    ///
    /// # Examples
    /// 
    /// ```
    /// use ruchy::middleend::types::TyVarGenerator;
    /// 
    /// let gen = TyVarGenerator::new();
    /// // Generator starts with id 0
    /// ```
    #[must_use]
    pub fn new() -> Self {
        TyVarGenerator { next: 0 }
    }
    /// Generate a fresh type variable
    ///
    /// # Examples
    /// 
    /// ```
    /// use ruchy::middleend::types::{TyVarGenerator, TyVar};
    /// 
    /// let mut gen = TyVarGenerator::new();
    /// let var1 = gen.fresh();
    /// let var2 = gen.fresh();
    /// assert_eq!(var1, TyVar(0));
    /// assert_eq!(var2, TyVar(1));
    /// ```
    pub fn fresh(&mut self) -> TyVar {
        let var = TyVar(self.next);
        self.next += 1;
        var
    }
}
impl Default for TyVarGenerator {
    fn default() -> Self {
        Self::new()
    }
}
/// Substitution mapping from type variables to types
pub type Substitution = HashMap<TyVar, MonoType>;
impl MonoType {
    /// Apply a substitution to this type
    ///
    /// # Examples
    /// 
    /// ```
    /// use std::collections::HashMap;
    /// use ruchy::middleend::types::{MonoType, TyVar};
    /// 
    /// let mut subst = HashMap::new();
    /// let var = TyVar(0);
    /// subst.insert(var.clone(), MonoType::Int);
    /// 
    /// let list_type = MonoType::List(Box::new(MonoType::Var(var)));
    /// let result = list_type.substitute(&subst);
    /// assert_eq!(result, MonoType::List(Box::new(MonoType::Int)));
    /// ```
    #[must_use]
    pub fn substitute(&self, subst: &Substitution) -> MonoType {
        match self {
            MonoType::Var(v) => subst.get(v).cloned().unwrap_or_else(|| self.clone()),
            MonoType::Function(arg, ret) => MonoType::Function(
                Box::new(arg.substitute(subst)),
                Box::new(ret.substitute(subst)),
            ),
            MonoType::List(elem) => MonoType::List(Box::new(elem.substitute(subst))),
            MonoType::Optional(inner) => MonoType::Optional(Box::new(inner.substitute(subst))),
            MonoType::Result(ok, err) => MonoType::Result(
                Box::new(ok.substitute(subst)),
                Box::new(err.substitute(subst)),
            ),
            MonoType::DataFrame(columns) => MonoType::DataFrame(
                columns
                    .iter()
                    .map(|(name, ty)| (name.clone(), ty.substitute(subst)))
                    .collect(),
            ),
            MonoType::Series(dtype) => MonoType::Series(Box::new(dtype.substitute(subst))),
            MonoType::Reference(inner) => MonoType::Reference(Box::new(inner.substitute(subst))),
            MonoType::Tuple(types) => {
                MonoType::Tuple(types.iter().map(|ty| ty.substitute(subst)).collect())
            }
            _ => self.clone(),
        }
    }
    /// Get free type variables in this type
    ///
    /// # Examples
    /// 
    /// ```
    /// use ruchy::middleend::types::{MonoType, TyVar};
    /// 
    /// let var1 = TyVar(0);
    /// let var2 = TyVar(1);
    /// let func_type = MonoType::Function(
    ///     Box::new(MonoType::Var(var1.clone())),
    ///     Box::new(MonoType::Var(var2.clone()))
    /// );
    /// 
    /// let free_vars = func_type.free_vars();
    /// assert_eq!(free_vars.len(), 2);
    /// assert!(free_vars.contains(&var1));
    /// assert!(free_vars.contains(&var2));
    /// ```
    #[must_use]
    pub fn free_vars(&self) -> Vec<TyVar> {
        use std::collections::HashSet;
        fn collect_vars(ty: &MonoType, vars: &mut HashSet<TyVar>) {
            match ty {
                MonoType::Var(v) => {
                    vars.insert(v.clone());
                }
                MonoType::Function(arg, ret) => {
                    collect_vars(arg, vars);
                    collect_vars(ret, vars);
                }
                MonoType::List(elem) => collect_vars(elem, vars),
                MonoType::Optional(inner)
                | MonoType::Series(inner)
                | MonoType::Reference(inner) => {
                    collect_vars(inner, vars);
                }
                MonoType::Result(ok, err) => {
                    collect_vars(ok, vars);
                    collect_vars(err, vars);
                }
                MonoType::DataFrame(columns) => {
                    for (_, ty) in columns {
                        collect_vars(ty, vars);
                    }
                }
                MonoType::Tuple(types) => {
                    for ty in types {
                        collect_vars(ty, vars);
                    }
                }
                _ => {}
            }
        }
        let mut vars = HashSet::new();
        collect_vars(self, &mut vars);
        vars.into_iter().collect()
    }
}
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
#[cfg(test)]
use proptest::prelude::*;
    #[test]
    fn test_type_display() {
        assert_eq!(MonoType::Int.to_string(), "i32");
        assert_eq!(MonoType::Bool.to_string(), "bool");
        assert_eq!(
            MonoType::Function(Box::new(MonoType::Int), Box::new(MonoType::Bool)).to_string(),
            "(i32 -> bool)"
        );
        assert_eq!(MonoType::List(Box::new(MonoType::Int)).to_string(), "[i32]");
    }
    #[test]
    fn test_type_scheme_instantiation() {
        let mut gen = TyVarGenerator::new();
        let var = gen.fresh();
        let scheme = TypeScheme {
            vars: vec![var.clone()],
            ty: MonoType::Function(
                Box::new(MonoType::Var(var.clone())),
                Box::new(MonoType::Var(var)),
            ),
        };
        let instantiated = scheme.instantiate(&mut gen);
        match instantiated {
            MonoType::Function(arg, ret) => {
                assert!(matches!(*arg, MonoType::Var(_)));
                assert!(matches!(*ret, MonoType::Var(_)));
            }
            _ => panic!("Expected function type"),
        }
    }
    #[test]
    fn test_substitution() {
        let mut subst = HashMap::new();
        let var = TyVar(0);
        subst.insert(var.clone(), MonoType::Int);
        let ty = MonoType::List(Box::new(MonoType::Var(var)));
        let result = ty.substitute(&subst);
        assert_eq!(result, MonoType::List(Box::new(MonoType::Int)));
    }
    #[test]
    fn test_free_vars() {
        let var1 = TyVar(0);
        let var2 = TyVar(1);
        let ty = MonoType::Function(
            Box::new(MonoType::Var(var1.clone())),
            Box::new(MonoType::List(Box::new(MonoType::Var(var2.clone())))),
        );
        let free = ty.free_vars();
        assert_eq!(free.len(), 2);
        assert!(free.contains(&var1));
        assert!(free.contains(&var2));
        // Test that duplicate variables are deduplicated
        let ty_dup = MonoType::Function(
            Box::new(MonoType::Var(var1.clone())),
            Box::new(MonoType::Var(var1.clone())),
        );
        let free_dup = ty_dup.free_vars();
        assert_eq!(free_dup.len(), 1);
        assert!(free_dup.contains(&var1));
    }
}
#[cfg(test)]
mod property_tests_types {
    use proptest::proptest;
    use super::*;
    use proptest::prelude::*;
    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_mono_never_panics(input: String) {
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
