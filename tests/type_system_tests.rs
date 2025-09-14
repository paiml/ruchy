//! Tests for type system modules
//! Focus on type inference and checking

use ruchy::middleend::types::{Type, TypeScheme};
use ruchy::middleend::environment::Environment;
use std::collections::HashMap;

#[test]
fn test_type_creation() {
    // Test basic type creation
    let int_type = Type::Int;
    let bool_type = Type::Bool;
    let string_type = Type::String;
    let float_type = Type::Float;
    
    assert_eq!(format!("{:?}", int_type), "Int");
    assert_eq!(format!("{:?}", bool_type), "Bool");
    assert_eq!(format!("{:?}", string_type), "String");
    assert_eq!(format!("{:?}", float_type), "Float");
}

#[test]
fn test_function_type() {
    // Test function type creation
    let param_types = vec![Type::Int, Type::Int];
    let return_type = Type::Int;
    let func_type = Type::Function(Box::new(Type::Tuple(param_types)), Box::new(return_type));
    
    match func_type {
        Type::Function(params, ret) => {
            match *params {
                Type::Tuple(ref p) => {
                    assert_eq!(p.len(), 2);
                    assert!(matches!(p[0], Type::Int));
                    assert!(matches!(p[1], Type::Int));
                }
                _ => panic!("Expected tuple type for parameters"),
            }
            assert!(matches!(*ret, Type::Int));
        }
        _ => panic!("Expected function type"),
    }
}

#[test]
fn test_array_type() {
    let elem_type = Type::Int;
    let array_type = Type::Array(Box::new(elem_type));
    
    match array_type {
        Type::Array(elem) => {
            assert!(matches!(*elem, Type::Int));
        }
        _ => panic!("Expected array type"),
    }
}

#[test]
fn test_option_type() {
    let inner_type = Type::String;
    let option_type = Type::Option(Box::new(inner_type));
    
    match option_type {
        Type::Option(inner) => {
            assert!(matches!(*inner, Type::String));
        }
        _ => panic!("Expected option type"),
    }
}

#[test]
fn test_result_type() {
    let ok_type = Type::Int;
    let err_type = Type::String;
    let result_type = Type::Result(Box::new(ok_type), Box::new(err_type));
    
    match result_type {
        Type::Result(ok, err) => {
            assert!(matches!(*ok, Type::Int));
            assert!(matches!(*err, Type::String));
        }
        _ => panic!("Expected result type"),
    }
}

#[test]
fn test_tuple_type() {
    let types = vec![Type::Int, Type::Bool, Type::String];
    let tuple_type = Type::Tuple(types);
    
    match tuple_type {
        Type::Tuple(elems) => {
            assert_eq!(elems.len(), 3);
            assert!(matches!(elems[0], Type::Int));
            assert!(matches!(elems[1], Type::Bool));
            assert!(matches!(elems[2], Type::String));
        }
        _ => panic!("Expected tuple type"),
    }
}

#[test]
fn test_type_variable() {
    let type_var = Type::Var(42);
    
    match type_var {
        Type::Var(id) => {
            assert_eq!(id, 42);
        }
        _ => panic!("Expected type variable"),
    }
}

#[test]
fn test_type_scheme_creation() {
    // Test TypeScheme with no quantified variables
    let ty = Type::Int;
    let scheme = TypeScheme::new(vec![], ty.clone());
    
    assert_eq!(scheme.vars.len(), 0);
    assert!(matches!(scheme.ty, Type::Int));
}

#[test]
fn test_type_scheme_with_vars() {
    // Test TypeScheme with quantified variables
    let ty = Type::Function(
        Box::new(Type::Var(0)),
        Box::new(Type::Var(0))
    );
    let scheme = TypeScheme::new(vec![0], ty);
    
    assert_eq!(scheme.vars.len(), 1);
    assert_eq!(scheme.vars[0], 0);
}

#[test]
fn test_environment_creation() {
    let env = Environment::<Type>::new();
    
    assert!(env.bindings.is_empty());
}

#[test]
fn test_environment_insert_lookup() {
    let mut env = Environment::<Type>::new();
    
    // Insert a binding
    env.insert("x".to_string(), Type::Int);
    
    // Lookup the binding
    let ty = env.lookup("x");
    assert!(ty.is_some());
    assert!(matches!(ty.unwrap(), Type::Int));
    
    // Lookup non-existent binding
    let missing = env.lookup("y");
    assert!(missing.is_none());
}

#[test]
fn test_environment_extend() {
    let mut env = Environment::<Type>::new();
    env.insert("x".to_string(), Type::Int);
    
    let mut bindings = HashMap::new();
    bindings.insert("y".to_string(), Type::Bool);
    bindings.insert("z".to_string(), Type::String);
    
    let new_env = env.extend(bindings);
    
    // Check all bindings exist
    assert!(matches!(new_env.lookup("x"), Some(Type::Int)));
    assert!(matches!(new_env.lookup("y"), Some(Type::Bool)));
    assert!(matches!(new_env.lookup("z"), Some(Type::String)));
}

#[test]
fn test_type_equality() {
    assert_eq!(Type::Int, Type::Int);
    assert_eq!(Type::Bool, Type::Bool);
    assert_ne!(Type::Int, Type::Bool);
    
    let array1 = Type::Array(Box::new(Type::Int));
    let array2 = Type::Array(Box::new(Type::Int));
    let array3 = Type::Array(Box::new(Type::Bool));
    
    assert_eq!(array1, array2);
    assert_ne!(array1, array3);
}

#[test]
fn test_nested_types() {
    // Test Option<Array<Int>>
    let nested = Type::Option(Box::new(
        Type::Array(Box::new(Type::Int))
    ));
    
    match nested {
        Type::Option(inner) => {
            match *inner {
                Type::Array(elem) => {
                    assert!(matches!(*elem, Type::Int));
                }
                _ => panic!("Expected array inside option"),
            }
        }
        _ => panic!("Expected option type"),
    }
}

#[test]
fn test_generic_type() {
    // Test a generic list type
    let list_type = Type::Generic("List".to_string(), vec![Type::Int]);
    
    match list_type {
        Type::Generic(name, args) => {
            assert_eq!(name, "List");
            assert_eq!(args.len(), 1);
            assert!(matches!(args[0], Type::Int));
        }
        _ => panic!("Expected generic type"),
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
            let ty = Type::Var(id);
            match ty {
                Type::Var(n) => prop_assert_eq!(n, id),
                _ => prop_assert!(false, "Expected type variable"),
            }
        }
        
        #[test]
        fn prop_environment_insert_retrieve(key in "[a-z]{1,10}", val in 0u32..100) {
            let mut env = Environment::<u32>::new();
            env.insert(key.clone(), val);
            
            let retrieved = env.lookup(&key);
            prop_assert_eq!(retrieved, Some(val));
        }
        
        #[test]
        fn prop_type_scheme_preserves_vars(vars in prop::collection::vec(0u32..100, 0..10)) {
            let ty = Type::Int; // Simple type for testing
            let scheme = TypeScheme::new(vars.clone(), ty);
            
            prop_assert_eq!(scheme.vars.len(), vars.len());
            for (i, var) in vars.iter().enumerate() {
                prop_assert_eq!(scheme.vars[i], *var);
            }
        }
    }
}