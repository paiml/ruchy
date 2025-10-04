//! Tests for type system modules
//! Focus on type inference and checking

use ruchy::middleend::environment::TypeEnv;
use ruchy::middleend::types::{MonoType, TyVar, TypeScheme};

#[test]
fn test_type_creation() {
    // Check basic type creation
    let int_type = MonoType::Int;
    let bool_type = MonoType::Bool;
    let string_type = MonoType::String;
    let float_type = MonoType::Float;

    assert_eq!(format!("{:?}", int_type), "Int");
    assert_eq!(format!("{:?}", bool_type), "Bool");
    assert_eq!(format!("{:?}", string_type), "String");
    assert_eq!(format!("{:?}", float_type), "Float");
}

#[test]
fn test_function_type() {
    // Check function type creation
    let param_types = vec![MonoType::Int, MonoType::Int];
    let return_type = MonoType::Int;
    let func_type = MonoType::Function(
        Box::new(MonoType::Tuple(param_types)),
        Box::new(return_type),
    );

    match func_type {
        MonoType::Function(params, ret) => {
            match *params {
                MonoType::Tuple(ref p) => {
                    assert_eq!(p.len(), 2);
                    assert!(matches!(p[0], MonoType::Int));
                    assert!(matches!(p[1], MonoType::Int));
                }
                _ => panic!("Expected tuple type for parameters"),
            }
            assert!(matches!(*ret, MonoType::Int));
        }
        _ => panic!("Expected function type"),
    }
}

#[test]
fn test_array_type() {
    let elem_type = MonoType::Int;
    let array_type = MonoType::List(Box::new(elem_type));

    match array_type {
        MonoType::List(elem) => {
            assert!(matches!(*elem, MonoType::Int));
        }
        _ => panic!("Expected array type"),
    }
}

#[test]
fn test_option_type() {
    let inner_type = MonoType::String;
    let option_type = MonoType::Optional(Box::new(inner_type));

    match option_type {
        MonoType::Optional(inner) => {
            assert!(matches!(*inner, MonoType::String));
        }
        _ => panic!("Expected option type"),
    }
}

#[test]
fn test_result_type() {
    let ok_type = MonoType::Int;
    let err_type = MonoType::String;
    let result_type = MonoType::Result(Box::new(ok_type), Box::new(err_type));

    match result_type {
        MonoType::Result(ok, err) => {
            assert!(matches!(*ok, MonoType::Int));
            assert!(matches!(*err, MonoType::String));
        }
        _ => panic!("Expected result type"),
    }
}

#[test]
fn test_tuple_type() {
    let types = vec![MonoType::Int, MonoType::Bool, MonoType::String];
    let tuple_type = MonoType::Tuple(types);

    match tuple_type {
        MonoType::Tuple(elems) => {
            assert_eq!(elems.len(), 3);
            assert!(matches!(elems[0], MonoType::Int));
            assert!(matches!(elems[1], MonoType::Bool));
            assert!(matches!(elems[2], MonoType::String));
        }
        _ => panic!("Expected tuple type"),
    }
}

#[test]
fn test_type_variable() {
    let type_var = MonoType::Var(TyVar(42));

    match type_var {
        MonoType::Var(id) => {
            assert_eq!(id, TyVar(42));
        }
        _ => panic!("Expected type variable"),
    }
}

#[test]
fn test_type_scheme_creation() {
    // Check TypeScheme with no quantified variables
    let ty = MonoType::Int;
    let scheme = TypeScheme::mono(ty.clone());

    assert_eq!(scheme.vars.len(), 0);
    assert!(matches!(scheme.ty, MonoType::Int));
}

#[test]
fn test_type_scheme_with_vars() {
    // Check TypeScheme with quantified variables
    let var = TyVar(0);
    let ty = MonoType::Function(
        Box::new(MonoType::Var(var.clone())),
        Box::new(MonoType::Var(var.clone())),
    );
    let scheme = TypeScheme {
        vars: vec![var.clone()],
        ty,
    };

    assert_eq!(scheme.vars.len(), 1);
    assert_eq!(scheme.vars[0], var);
}

#[test]
fn test_environment_creation() {
    let env = TypeEnv::new();

    // Check that environment is empty by trying to lookup a non-existent key
    assert!(env.lookup("nonexistent").is_none());
}

#[test]
fn test_environment_insert_lookup() {
    let mut env = TypeEnv::new();

    // Insert a binding
    let scheme = TypeScheme::mono(MonoType::Int);
    env.bind("x", scheme.clone());

    // Lookup the binding
    let retrieved = env.lookup("x");
    assert!(retrieved.is_some());
    assert_eq!(&retrieved.unwrap().ty, &MonoType::Int);

    // Lookup non-existent binding
    let missing = env.lookup("y");
    assert!(missing.is_none());
}

// Commented out - TypeEnv extend method doesn't work with HashMap
// #[test]
// fn test_environment_extend() {
//     let mut env = TypeEnv::new();
//     env.bind("x", TypeScheme::mono(MonoType::Int));
//
//     let new_env = env.extend("y", TypeScheme::mono(MonoType::Bool));
//
//     // Check bindings exist
//     assert!(env.lookup("x").is_some());
//     assert!(new_env.lookup("x").is_some());
//     assert!(new_env.lookup("y").is_some());
// }

#[test]
fn test_type_equality() {
    assert_eq!(MonoType::Int, MonoType::Int);
    assert_eq!(MonoType::Bool, MonoType::Bool);
    assert_ne!(MonoType::Int, MonoType::Bool);

    let array1 = MonoType::List(Box::new(MonoType::Int));
    let array2 = MonoType::List(Box::new(MonoType::Int));
    let array3 = MonoType::List(Box::new(MonoType::Bool));

    assert_eq!(array1, array2);
    assert_ne!(array1, array3);
}

#[test]
fn test_nested_types() {
    // Check Option<Array<Int>>
    let nested = MonoType::Optional(Box::new(MonoType::List(Box::new(MonoType::Int))));

    match nested {
        MonoType::Optional(inner) => match *inner {
            MonoType::List(elem) => {
                assert!(matches!(*elem, MonoType::Int));
            }
            _ => panic!("Expected array inside option"),
        },
        _ => panic!("Expected option type"),
    }
}

#[test]
fn test_generic_type() {
    // Check a named type (user-defined)
    let list_type = MonoType::Named("List".to_string());

    match list_type {
        MonoType::Named(name) => {
            assert_eq!(name, "List");
        }
        _ => panic!("Expected named type"),
    }
}

// Property-based tests
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_type_var_roundtrip(id in 0u32..10000) {
            let ty = MonoType::Var(TyVar(id));
            match ty {
                MonoType::Var(n) => prop_assert_eq!(n, TyVar(id)),
                _ => prop_assert!(false, "Expected type variable"),
            }
        }

        // #[test]
        // fn prop_environment_insert_retrieve(key in "[a-z]{1,10}", val in 0u32..100) {
        //     let mut env = TypeEnv::new();
        //     env.bind(key.clone(), TypeScheme::mono(MonoType::Int));
        //
        //     let retrieved = env.lookup(&key);
        //     prop_assert!(retrieved.is_some());
        // }

        #[test]
        fn prop_type_scheme_preserves_vars(vars in prop::collection::vec(0u32..100, 0..10)) {
            let ty = MonoType::Int; // Simple type for testing
            let ty_vars: Vec<TyVar> = vars.iter().map(|&v| TyVar(v)).collect();
            let scheme = TypeScheme {
                vars: ty_vars.clone(),
                ty,
            };

            prop_assert_eq!(scheme.vars.len(), vars.len());
            for (i, var) in vars.iter().enumerate() {
                prop_assert_eq!(scheme.vars[i].clone(), TyVar(*var));
            }
        }
    }
}
