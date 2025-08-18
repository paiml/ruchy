//! Type system representation for Ruchy

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
    /// Unit type ()
    Unit,
    /// Function type: T1 -> T2
    Function(Box<MonoType>, Box<MonoType>),
    /// List type: [T]
    List(Box<MonoType>),
    /// Optional type: T?
    Optional(Box<MonoType>),
    /// Result type: Result<T, E>
    Result(Box<MonoType>, Box<MonoType>),
    /// Tuple type: (T1, T2, ...)
    Tuple(Vec<MonoType>),
    /// Named type (user-defined or gradual typing 'Any')
    Named(String),
}

impl fmt::Display for MonoType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MonoType::Var(v) => write!(f, "{v}"),
            MonoType::Int => write!(f, "i32"),
            MonoType::Float => write!(f, "f64"),
            MonoType::Bool => write!(f, "bool"),
            MonoType::String => write!(f, "String"),
            MonoType::Unit => write!(f, "()"),
            MonoType::Function(arg, ret) => write!(f, "({arg} -> {ret})"),
            MonoType::List(elem) => write!(f, "[{elem}]"),
            MonoType::Optional(inner) => write!(f, "{inner}?"),
            MonoType::Result(ok, err) => write!(f, "Result<{ok}, {err}>"),
            MonoType::Tuple(types) => {
                write!(f, "(")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{ty}")?;
                }
                write!(f, ")")
            }
            MonoType::Named(name) => write!(f, "{name}"),
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
    #[must_use]
    pub fn mono(ty: MonoType) -> Self {
        TypeScheme {
            vars: Vec::new(),
            ty,
        }
    }

    /// Instantiate a type scheme with fresh type variables
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
    #[must_use]
    pub fn new() -> Self {
        TyVarGenerator { next: 0 }
    }

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
            _ => self.clone(),
        }
    }

    /// Get free type variables in this type
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
                MonoType::Optional(inner) => collect_vars(inner, vars),
                MonoType::Result(ok, err) => {
                    collect_vars(ok, vars);
                    collect_vars(err, vars);
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
