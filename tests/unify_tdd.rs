//! Comprehensive TDD test suite for unify.rs  
//! Target: Transform 0% → 70%+ coverage via systematic testing
//! Toyota Way: Every unification path must be tested comprehensively

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use ruchy::middleend::unify::Unifier;
use ruchy::middleend::types::{MonoType, TyVar};

// ==================== UNIFIER CREATION TESTS ====================

#[test]
fn test_unifier_new() {
    let unifier = Unifier::new();
    assert!(unifier.substitution().is_empty());
}

// ==================== BASIC UNIFICATION TESTS ====================

#[test]
fn test_unify_same_primitive_types() {
    let mut unifier = Unifier::new();
    
    assert!(unifier.unify(&MonoType::Int, &MonoType::Int).is_ok());
    assert!(unifier.unify(&MonoType::Float, &MonoType::Float).is_ok());
    assert!(unifier.unify(&MonoType::Bool, &MonoType::Bool).is_ok());
    assert!(unifier.unify(&MonoType::String, &MonoType::String).is_ok());
    assert!(unifier.unify(&MonoType::Unit, &MonoType::Unit).is_ok());
    assert!(unifier.unify(&MonoType::Char, &MonoType::Char).is_ok());
}

#[test]
fn test_unify_different_primitive_types() {
    let mut unifier = Unifier::new();
    
    assert!(unifier.unify(&MonoType::Int, &MonoType::Float).is_err());
    assert!(unifier.unify(&MonoType::Bool, &MonoType::String).is_err());
    assert!(unifier.unify(&MonoType::Int, &MonoType::Bool).is_err());
    assert!(unifier.unify(&MonoType::Char, &MonoType::String).is_err());
}

// ==================== TYPE VARIABLE UNIFICATION TESTS ====================

#[test]
fn test_unify_type_variable_with_concrete() {
    let mut unifier = Unifier::new();
    let tv = MonoType::Var(TyVar(0));
    
    assert!(unifier.unify(&tv, &MonoType::Int).is_ok());
    assert_eq!(unifier.apply(&tv), MonoType::Int);
}

#[test]
fn test_unify_two_type_variables() {
    let mut unifier = Unifier::new();
    let tv1 = MonoType::Var(TyVar(0));
    let tv2 = MonoType::Var(TyVar(1));
    
    assert!(unifier.unify(&tv1, &tv2).is_ok());
    // They should be unified
}

#[test]
fn test_unify_type_variable_transitivity() {
    let mut unifier = Unifier::new();
    let tv1 = MonoType::Var(TyVar(0));
    let tv2 = MonoType::Var(TyVar(1));
    let tv3 = MonoType::Var(TyVar(2));
    
    assert!(unifier.unify(&tv1, &tv2).is_ok());
    assert!(unifier.unify(&tv2, &tv3).is_ok());
    assert!(unifier.unify(&tv3, &MonoType::Int).is_ok());
    
    assert_eq!(unifier.apply(&tv1), MonoType::Int);
    assert_eq!(unifier.apply(&tv2), MonoType::Int);
    assert_eq!(unifier.apply(&tv3), MonoType::Int);
}

// ==================== FUNCTION TYPE UNIFICATION TESTS ====================

#[test]
fn test_unify_function_types() {
    let mut unifier = Unifier::new();
    
    let fn1 = MonoType::Function(
        Box::new(MonoType::Int),
        Box::new(MonoType::Int)
    );
    let fn2 = MonoType::Function(
        Box::new(MonoType::Int),
        Box::new(MonoType::Int)
    );
    
    assert!(unifier.unify(&fn1, &fn2).is_ok());
}

#[test]
fn test_unify_function_types_different_params() {
    let mut unifier = Unifier::new();
    
    let fn1 = MonoType::Function(
        Box::new(MonoType::Int),
        Box::new(MonoType::Int)
    );
    let fn2 = MonoType::Function(
        Box::new(MonoType::String),
        Box::new(MonoType::Int)
    );
    
    assert!(unifier.unify(&fn1, &fn2).is_err());
}

#[test]
fn test_unify_function_types_different_return() {
    let mut unifier = Unifier::new();
    
    let fn1 = MonoType::Function(
        Box::new(MonoType::Int),
        Box::new(MonoType::String)
    );
    let fn2 = MonoType::Function(
        Box::new(MonoType::Int),
        Box::new(MonoType::Int)
    );
    
    assert!(unifier.unify(&fn1, &fn2).is_err());
}

#[test]
fn test_unify_function_with_type_variables() {
    let mut unifier = Unifier::new();
    
    let tv1 = MonoType::Var(TyVar(0));
    let tv2 = MonoType::Var(TyVar(1));
    
    let fn1 = MonoType::Function(
        Box::new(tv1.clone()),
        Box::new(tv2.clone())
    );
    let fn2 = MonoType::Function(
        Box::new(MonoType::Int),
        Box::new(MonoType::String)
    );
    
    assert!(unifier.unify(&fn1, &fn2).is_ok());
    assert_eq!(unifier.apply(&tv1), MonoType::Int);
    assert_eq!(unifier.apply(&tv2), MonoType::String);
}

#[test]
fn test_unify_curried_function() {
    let mut unifier = Unifier::new();
    
    // (Int -> Int) -> Int
    let fn1 = MonoType::Function(
        Box::new(MonoType::Function(
            Box::new(MonoType::Int),
            Box::new(MonoType::Int)
        )),
        Box::new(MonoType::Int)
    );
    
    let fn2 = MonoType::Function(
        Box::new(MonoType::Function(
            Box::new(MonoType::Int),
            Box::new(MonoType::Int)
        )),
        Box::new(MonoType::Int)
    );
    
    assert!(unifier.unify(&fn1, &fn2).is_ok());
}

// ==================== LIST TYPE UNIFICATION TESTS ====================

#[test]
fn test_unify_list_types() {
    let mut unifier = Unifier::new();
    
    let list1 = MonoType::List(Box::new(MonoType::Int));
    let list2 = MonoType::List(Box::new(MonoType::Int));
    
    assert!(unifier.unify(&list1, &list2).is_ok());
}

#[test]
fn test_unify_list_types_different_elements() {
    let mut unifier = Unifier::new();
    
    let list1 = MonoType::List(Box::new(MonoType::Int));
    let list2 = MonoType::List(Box::new(MonoType::String));
    
    assert!(unifier.unify(&list1, &list2).is_err());
}

#[test]
fn test_unify_list_with_type_variable() {
    let mut unifier = Unifier::new();
    
    let tv = MonoType::Var(TyVar(0));
    let list1 = MonoType::List(Box::new(tv.clone()));
    let list2 = MonoType::List(Box::new(MonoType::Int));
    
    assert!(unifier.unify(&list1, &list2).is_ok());
    assert_eq!(unifier.apply(&tv), MonoType::Int);
}

// ==================== TUPLE TYPE UNIFICATION TESTS ====================

#[test]
fn test_unify_tuple_types() {
    let mut unifier = Unifier::new();
    
    let tup1 = MonoType::Tuple(vec![MonoType::Int, MonoType::String]);
    let tup2 = MonoType::Tuple(vec![MonoType::Int, MonoType::String]);
    
    assert!(unifier.unify(&tup1, &tup2).is_ok());
}

#[test]
fn test_unify_tuple_types_different_length() {
    let mut unifier = Unifier::new();
    
    let tup1 = MonoType::Tuple(vec![MonoType::Int]);
    let tup2 = MonoType::Tuple(vec![MonoType::Int, MonoType::String]);
    
    assert!(unifier.unify(&tup1, &tup2).is_err());
}

#[test]
fn test_unify_tuple_types_different_elements() {
    let mut unifier = Unifier::new();
    
    let tup1 = MonoType::Tuple(vec![MonoType::Int, MonoType::Int]);
    let tup2 = MonoType::Tuple(vec![MonoType::Int, MonoType::String]);
    
    assert!(unifier.unify(&tup1, &tup2).is_err());
}

#[test]
fn test_unify_tuple_with_type_variables() {
    let mut unifier = Unifier::new();
    
    let tv1 = MonoType::Var(TyVar(0));
    let tv2 = MonoType::Var(TyVar(1));
    
    let tup1 = MonoType::Tuple(vec![tv1.clone(), tv2.clone()]);
    let tup2 = MonoType::Tuple(vec![MonoType::Int, MonoType::String]);
    
    assert!(unifier.unify(&tup1, &tup2).is_ok());
    assert_eq!(unifier.apply(&tv1), MonoType::Int);
    assert_eq!(unifier.apply(&tv2), MonoType::String);
}

#[test]
fn test_unify_empty_tuple() {
    let mut unifier = Unifier::new();
    
    let tup1 = MonoType::Tuple(vec![]);
    let tup2 = MonoType::Tuple(vec![]);
    
    assert!(unifier.unify(&tup1, &tup2).is_ok());
}

#[test]
fn test_unify_single_element_tuple() {
    let mut unifier = Unifier::new();
    
    let tup1 = MonoType::Tuple(vec![MonoType::Int]);
    let tup2 = MonoType::Tuple(vec![MonoType::Int]);
    
    assert!(unifier.unify(&tup1, &tup2).is_ok());
}

// ==================== SUBSTITUTION TESTS ====================

#[test]
fn test_apply_substitution() {
    let mut unifier = Unifier::new();
    let tv = MonoType::Var(TyVar(0));
    
    unifier.unify(&tv, &MonoType::Int).unwrap();
    
    let result = unifier.apply(&tv);
    assert_eq!(result, MonoType::Int);
}

#[test]
fn test_apply_substitution_nested() {
    let mut unifier = Unifier::new();
    let tv = MonoType::Var(TyVar(0));
    
    let list_type = MonoType::List(Box::new(tv.clone()));
    unifier.unify(&tv, &MonoType::Int).unwrap();
    
    let result = unifier.apply(&list_type);
    assert_eq!(result, MonoType::List(Box::new(MonoType::Int)));
}

#[test]
fn test_apply_substitution_function() {
    let mut unifier = Unifier::new();
    let tv1 = MonoType::Var(TyVar(0));
    let tv2 = MonoType::Var(TyVar(1));
    
    let fn_type = MonoType::Function(
        Box::new(tv1.clone()),
        Box::new(tv2.clone())
    );
    
    unifier.unify(&tv1, &MonoType::Int).unwrap();
    unifier.unify(&tv2, &MonoType::String).unwrap();
    
    let result = unifier.apply(&fn_type);
    assert_eq!(
        result,
        MonoType::Function(
            Box::new(MonoType::Int),
            Box::new(MonoType::String)
        )
    );
}

// ==================== SOLVE TESTS ====================

#[test]
fn test_solve_type_variable() {
    let mut unifier = Unifier::new();
    let var = TyVar(0);
    let tv = MonoType::Var(var.clone());
    
    unifier.unify(&tv, &MonoType::Int).unwrap();
    
    let result = unifier.solve(&var);
    assert_eq!(result, MonoType::Int);
}

#[test]
fn test_solve_unbound_variable() {
    let unifier = Unifier::new();
    let var = TyVar(0);
    
    let result = unifier.solve(&var);
    assert_eq!(result, MonoType::Var(var));
}

// ==================== COMPLEX UNIFICATION TESTS ====================

#[test]
fn test_unify_complex_nested() {
    let mut unifier = Unifier::new();
    
    let tv = MonoType::Var(TyVar(0));
    
    // List of functions: [a -> Int]
    let complex1 = MonoType::List(Box::new(
        MonoType::Function(
            Box::new(tv.clone()),
            Box::new(MonoType::Int)
        )
    ));
    
    // List of functions: [String -> Int]
    let complex2 = MonoType::List(Box::new(
        MonoType::Function(
            Box::new(MonoType::String),
            Box::new(MonoType::Int)
        )
    ));
    
    assert!(unifier.unify(&complex1, &complex2).is_ok());
    assert_eq!(unifier.apply(&tv), MonoType::String);
}

#[test]
fn test_unify_multiple_constraints() {
    let mut unifier = Unifier::new();
    
    let tv1 = MonoType::Var(TyVar(0));
    let tv2 = MonoType::Var(TyVar(1));
    
    // Create constraints
    assert!(unifier.unify(&tv1, &tv2).is_ok());
    assert!(unifier.unify(&tv2, &MonoType::Int).is_ok());
    
    // Both should resolve to Int
    assert_eq!(unifier.apply(&tv1), MonoType::Int);
    assert_eq!(unifier.apply(&tv2), MonoType::Int);
}

#[test]
fn test_unify_polymorphic_function() {
    let mut unifier = Unifier::new();
    
    // id :: a -> a
    let tv = MonoType::Var(TyVar(0));
    let id_type = MonoType::Function(
        Box::new(tv.clone()),
        Box::new(tv.clone())
    );
    
    // Unify with Int -> Int
    let concrete = MonoType::Function(
        Box::new(MonoType::Int),
        Box::new(MonoType::Int)
    );
    
    assert!(unifier.unify(&id_type, &concrete).is_ok());
    assert_eq!(unifier.apply(&tv), MonoType::Int);
}

// ==================== ERROR RECOVERY TESTS ====================

#[test]
fn test_unify_error_recovery() {
    let mut unifier = Unifier::new();
    
    // This should fail
    let result = unifier.unify(&MonoType::Int, &MonoType::String);
    assert!(result.is_err());
    
    // But unifier should still be usable
    assert!(unifier.unify(&MonoType::Bool, &MonoType::Bool).is_ok());
}

// Run all tests with: cargo test unify_tdd --test unify_tdd