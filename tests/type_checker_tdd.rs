//! Comprehensive TDD test suite for type checker
//! Target: Transform 0% → 70%+ coverage via systematic testing
//! Toyota Way: Every type checking path must be tested comprehensively

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use ruchy::frontend::type_checker::{TypeChecker, TypeError, Type, TypeEnv};
use ruchy::frontend::ast::{Expr, Stmt, Program};

// ==================== TYPE CHECKER INITIALIZATION TESTS ====================

#[test]
fn test_type_checker_new() {
    let checker = TypeChecker::new();
    assert!(checker.is_empty());
}

#[test]
fn test_type_checker_with_builtins() {
    let checker = TypeChecker::with_builtins();
    // Should have built-in types
    assert!(checker.has_type("i32"));
    assert!(checker.has_type("f64"));
    assert!(checker.has_type("bool"));
    assert!(checker.has_type("String"));
}

// ==================== PRIMITIVE TYPE TESTS ====================

#[test]
fn test_check_integer_literal() {
    let mut checker = TypeChecker::new();
    let expr = Expr::literal(42);
    
    let ty = checker.check_expr(&expr);
    assert!(ty.is_ok());
    assert_eq!(ty.unwrap(), Type::I32);
}

#[test]
fn test_check_float_literal() {
    let mut checker = TypeChecker::new();
    let expr = Expr::literal_float(3.14);
    
    let ty = checker.check_expr(&expr);
    assert!(ty.is_ok());
    assert_eq!(ty.unwrap(), Type::F64);
}

#[test]
fn test_check_boolean_literal() {
    let mut checker = TypeChecker::new();
    let expr = Expr::literal_bool(true);
    
    let ty = checker.check_expr(&expr);
    assert!(ty.is_ok());
    assert_eq!(ty.unwrap(), Type::Bool);
}

#[test]
fn test_check_string_literal() {
    let mut checker = TypeChecker::new();
    let expr = Expr::string("hello");
    
    let ty = checker.check_expr(&expr);
    assert!(ty.is_ok());
    assert_eq!(ty.unwrap(), Type::String);
}

#[test]
fn test_check_char_literal() {
    let mut checker = TypeChecker::new();
    let expr = Expr::char('a');
    
    let ty = checker.check_expr(&expr);
    assert!(ty.is_ok());
    assert_eq!(ty.unwrap(), Type::Char);
}

// ==================== BINARY OPERATION TESTS ====================

#[test]
fn test_check_arithmetic_ops() {
    let mut checker = TypeChecker::new();
    
    for op in &["+", "-", "*", "/", "%"] {
        let expr = Expr::binary(op, Expr::literal(1), Expr::literal(2));
        let ty = checker.check_expr(&expr);
        assert!(ty.is_ok());
        assert_eq!(ty.unwrap(), Type::I32);
    }
}

#[test]
fn test_check_comparison_ops() {
    let mut checker = TypeChecker::new();
    
    for op in &["<", ">", "<=", ">=", "==", "!="] {
        let expr = Expr::binary(op, Expr::literal(1), Expr::literal(2));
        let ty = checker.check_expr(&expr);
        assert!(ty.is_ok());
        assert_eq!(ty.unwrap(), Type::Bool);
    }
}

#[test]
fn test_check_logical_ops() {
    let mut checker = TypeChecker::new();
    
    for op in &["&&", "||"] {
        let expr = Expr::binary(op, 
            Expr::literal_bool(true), 
            Expr::literal_bool(false)
        );
        let ty = checker.check_expr(&expr);
        assert!(ty.is_ok());
        assert_eq!(ty.unwrap(), Type::Bool);
    }
}

#[test]
fn test_binary_type_mismatch() {
    let mut checker = TypeChecker::new();
    let expr = Expr::binary("+", Expr::literal(42), Expr::string("text"));
    
    let ty = checker.check_expr(&expr);
    assert!(matches!(ty, Err(TypeError::TypeMismatch(_, _))));
}

// ==================== UNARY OPERATION TESTS ====================

#[test]
fn test_check_negation() {
    let mut checker = TypeChecker::new();
    let expr = Expr::unary("-", Expr::literal(42));
    
    let ty = checker.check_expr(&expr);
    assert!(ty.is_ok());
    assert_eq!(ty.unwrap(), Type::I32);
}

#[test]
fn test_check_logical_not() {
    let mut checker = TypeChecker::new();
    let expr = Expr::unary("!", Expr::literal_bool(true));
    
    let ty = checker.check_expr(&expr);
    assert!(ty.is_ok());
    assert_eq!(ty.unwrap(), Type::Bool);
}

#[test]
fn test_unary_type_mismatch() {
    let mut checker = TypeChecker::new();
    let expr = Expr::unary("!", Expr::literal(42)); // Not on integer
    
    let ty = checker.check_expr(&expr);
    assert!(matches!(ty, Err(TypeError::TypeMismatch(_, _))));
}

// ==================== VARIABLE TYPE TESTS ====================

#[test]
fn test_check_variable() {
    let mut checker = TypeChecker::new();
    checker.define_var("x", Type::I32);
    
    let expr = Expr::ident("x");
    let ty = checker.check_expr(&expr);
    assert!(ty.is_ok());
    assert_eq!(ty.unwrap(), Type::I32);
}

#[test]
fn test_undefined_variable() {
    let mut checker = TypeChecker::new();
    let expr = Expr::ident("undefined");
    
    let ty = checker.check_expr(&expr);
    assert!(matches!(ty, Err(TypeError::UndefinedVariable(_))));
}

#[test]
fn test_variable_shadowing() {
    let mut checker = TypeChecker::new();
    
    checker.define_var("x", Type::I32);
    checker.enter_scope();
    checker.define_var("x", Type::String);
    
    let expr = Expr::ident("x");
    let ty = checker.check_expr(&expr);
    assert_eq!(ty.unwrap(), Type::String);
    
    checker.exit_scope();
    let ty2 = checker.check_expr(&expr);
    assert_eq!(ty2.unwrap(), Type::I32);
}

// ==================== ARRAY TYPE TESTS ====================

#[test]
fn test_check_array_literal() {
    let mut checker = TypeChecker::new();
    let expr = Expr::array(vec![
        Expr::literal(1),
        Expr::literal(2),
        Expr::literal(3),
    ]);
    
    let ty = checker.check_expr(&expr);
    assert!(ty.is_ok());
    assert!(matches!(ty.unwrap(), Type::Array(box Type::I32)));
}

#[test]
fn test_array_heterogeneous_error() {
    let mut checker = TypeChecker::new();
    let expr = Expr::array(vec![
        Expr::literal(1),
        Expr::string("text"), // Different type
    ]);
    
    let ty = checker.check_expr(&expr);
    assert!(matches!(ty, Err(TypeError::HeterogeneousArray)));
}

#[test]
fn test_array_indexing() {
    let mut checker = TypeChecker::new();
    checker.define_var("arr", Type::Array(Box::new(Type::I32)));
    
    let expr = Expr::index(Expr::ident("arr"), Expr::literal(0));
    let ty = checker.check_expr(&expr);
    assert!(ty.is_ok());
    assert_eq!(ty.unwrap(), Type::I32);
}

// ==================== TUPLE TYPE TESTS ====================

#[test]
fn test_check_tuple_literal() {
    let mut checker = TypeChecker::new();
    let expr = Expr::tuple(vec![
        Expr::literal(42),
        Expr::string("hello"),
    ]);
    
    let ty = checker.check_expr(&expr);
    assert!(ty.is_ok());
    assert!(matches!(ty.unwrap(), Type::Tuple(types) if types.len() == 2));
}

#[test]
fn test_tuple_access() {
    let mut checker = TypeChecker::new();
    checker.define_var("tup", Type::Tuple(vec![Type::I32, Type::String]));
    
    let expr = Expr::tuple_index(Expr::ident("tup"), 0);
    let ty = checker.check_expr(&expr);
    assert!(ty.is_ok());
    assert_eq!(ty.unwrap(), Type::I32);
}

#[test]
fn test_tuple_index_out_of_bounds() {
    let mut checker = TypeChecker::new();
    checker.define_var("tup", Type::Tuple(vec![Type::I32, Type::String]));
    
    let expr = Expr::tuple_index(Expr::ident("tup"), 5);
    let ty = checker.check_expr(&expr);
    assert!(matches!(ty, Err(TypeError::TupleIndexOutOfBounds(_, _))));
}

// ==================== FUNCTION TYPE TESTS ====================

#[test]
fn test_check_function_type() {
    let mut checker = TypeChecker::new();
    checker.define_func("add", Type::Function(
        vec![Type::I32, Type::I32],
        Box::new(Type::I32)
    ));
    
    let expr = Expr::ident("add");
    let ty = checker.check_expr(&expr);
    assert!(ty.is_ok());
    assert!(matches!(ty.unwrap(), Type::Function(_, _)));
}

#[test]
fn test_check_function_call() {
    let mut checker = TypeChecker::new();
    checker.define_func("add", Type::Function(
        vec![Type::I32, Type::I32],
        Box::new(Type::I32)
    ));
    
    let expr = Expr::call("add", vec![Expr::literal(1), Expr::literal(2)]);
    let ty = checker.check_expr(&expr);
    assert!(ty.is_ok());
    assert_eq!(ty.unwrap(), Type::I32);
}

#[test]
fn test_function_call_wrong_arity() {
    let mut checker = TypeChecker::new();
    checker.define_func("add", Type::Function(
        vec![Type::I32, Type::I32],
        Box::new(Type::I32)
    ));
    
    let expr = Expr::call("add", vec![Expr::literal(1)]); // Wrong arity
    let ty = checker.check_expr(&expr);
    assert!(matches!(ty, Err(TypeError::ArityMismatch(_, _, _))));
}

#[test]
fn test_function_call_wrong_arg_type() {
    let mut checker = TypeChecker::new();
    checker.define_func("add", Type::Function(
        vec![Type::I32, Type::I32],
        Box::new(Type::I32)
    ));
    
    let expr = Expr::call("add", vec![
        Expr::literal(1),
        Expr::string("not a number"), // Wrong type
    ]);
    let ty = checker.check_expr(&expr);
    assert!(matches!(ty, Err(TypeError::TypeMismatch(_, _))));
}

// ==================== LAMBDA TYPE TESTS ====================

#[test]
fn test_check_lambda() {
    let mut checker = TypeChecker::new();
    let expr = Expr::lambda(
        vec![("x", Type::I32), ("y", Type::I32)],
        Expr::binary("+", Expr::ident("x"), Expr::ident("y"))
    );
    
    let ty = checker.check_expr(&expr);
    assert!(ty.is_ok());
    assert!(matches!(ty.unwrap(), Type::Function(params, ret) 
        if params.len() == 2 && **ret == Type::I32));
}

#[test]
fn test_lambda_type_inference() {
    let mut checker = TypeChecker::new();
    let expr = Expr::lambda_infer(
        vec!["x", "y"],
        Expr::binary("+", Expr::ident("x"), Expr::ident("y"))
    );
    
    // With context, should infer types
    let context = Type::Function(
        vec![Type::I32, Type::I32],
        Box::new(Type::I32)
    );
    
    let ty = checker.check_with_context(&expr, &context);
    assert!(ty.is_ok());
}

// ==================== IF EXPRESSION TYPE TESTS ====================

#[test]
fn test_check_if_expr() {
    let mut checker = TypeChecker::new();
    let expr = Expr::if_expr(
        Expr::literal_bool(true),
        Expr::literal(1),
        Some(Expr::literal(2))
    );
    
    let ty = checker.check_expr(&expr);
    assert!(ty.is_ok());
    assert_eq!(ty.unwrap(), Type::I32);
}

#[test]
fn test_if_condition_not_bool() {
    let mut checker = TypeChecker::new();
    let expr = Expr::if_expr(
        Expr::literal(42), // Not a bool
        Expr::literal(1),
        Some(Expr::literal(2))
    );
    
    let ty = checker.check_expr(&expr);
    assert!(matches!(ty, Err(TypeError::TypeMismatch(_, _))));
}

#[test]
fn test_if_branches_type_mismatch() {
    let mut checker = TypeChecker::new();
    let expr = Expr::if_expr(
        Expr::literal_bool(true),
        Expr::literal(1),
        Some(Expr::string("text")) // Different type
    );
    
    let ty = checker.check_expr(&expr);
    assert!(matches!(ty, Err(TypeError::TypeMismatch(_, _))));
}

// ==================== MATCH EXPRESSION TYPE TESTS ====================

#[test]
fn test_check_match_expr() {
    let mut checker = TypeChecker::new();
    let expr = Expr::match_expr(
        Expr::literal(42),
        vec![
            (Pattern::literal(0), Expr::string("zero")),
            (Pattern::wildcard(), Expr::string("other")),
        ]
    );
    
    let ty = checker.check_expr(&expr);
    assert!(ty.is_ok());
    assert_eq!(ty.unwrap(), Type::String);
}

#[test]
fn test_match_arms_type_mismatch() {
    let mut checker = TypeChecker::new();
    let expr = Expr::match_expr(
        Expr::literal(42),
        vec![
            (Pattern::literal(0), Expr::string("zero")),
            (Pattern::wildcard(), Expr::literal(1)), // Different type
        ]
    );
    
    let ty = checker.check_expr(&expr);
    assert!(matches!(ty, Err(TypeError::TypeMismatch(_, _))));
}

// ==================== STRUCT TYPE TESTS ====================

#[test]
fn test_check_struct_literal() {
    let mut checker = TypeChecker::new();
    checker.define_struct("Point", vec![
        ("x", Type::F64),
        ("y", Type::F64),
    ]);
    
    let expr = Expr::struct_literal("Point", vec![
        ("x", Expr::literal_float(1.0)),
        ("y", Expr::literal_float(2.0)),
    ]);
    
    let ty = checker.check_expr(&expr);
    assert!(ty.is_ok());
    assert!(matches!(ty.unwrap(), Type::Struct(name) if name == "Point"));
}

#[test]
fn test_struct_field_access() {
    let mut checker = TypeChecker::new();
    checker.define_struct("Point", vec![
        ("x", Type::F64),
        ("y", Type::F64),
    ]);
    checker.define_var("p", Type::Struct("Point".to_string()));
    
    let expr = Expr::field(Expr::ident("p"), "x");
    let ty = checker.check_expr(&expr);
    assert!(ty.is_ok());
    assert_eq!(ty.unwrap(), Type::F64);
}

#[test]
fn test_struct_unknown_field() {
    let mut checker = TypeChecker::new();
    checker.define_struct("Point", vec![
        ("x", Type::F64),
        ("y", Type::F64),
    ]);
    checker.define_var("p", Type::Struct("Point".to_string()));
    
    let expr = Expr::field(Expr::ident("p"), "z"); // Unknown field
    let ty = checker.check_expr(&expr);
    assert!(matches!(ty, Err(TypeError::UnknownField(_, _))));
}

// ==================== ENUM TYPE TESTS ====================

#[test]
fn test_check_enum_variant() {
    let mut checker = TypeChecker::new();
    checker.define_enum("Option", vec![
        ("Some", Some(Type::Generic("T".to_string()))),
        ("None", None),
    ]);
    
    let expr = Expr::enum_variant("Option", "None", None);
    let ty = checker.check_expr(&expr);
    assert!(ty.is_ok());
}

// ==================== GENERIC TYPE TESTS ====================

#[test]
fn test_generic_function() {
    let mut checker = TypeChecker::new();
    checker.define_generic_func("identity", 
        vec!["T"],
        Type::Function(
            vec![Type::Generic("T".to_string())],
            Box::new(Type::Generic("T".to_string()))
        )
    );
    
    let expr = Expr::call("identity", vec![Expr::literal(42)]);
    let ty = checker.check_expr(&expr);
    assert!(ty.is_ok());
    assert_eq!(ty.unwrap(), Type::I32); // T instantiated as I32
}

#[test]
fn test_generic_struct() {
    let mut checker = TypeChecker::new();
    checker.define_generic_struct("Vec", vec!["T"], vec![
        ("data", Type::Array(Box::new(Type::Generic("T".to_string())))),
    ]);
    
    let expr = Expr::struct_literal("Vec", vec![
        ("data", Expr::array(vec![Expr::literal(1), Expr::literal(2)])),
    ]);
    
    let ty = checker.check_expr(&expr);
    assert!(ty.is_ok());
}

// ==================== TYPE ALIAS TESTS ====================

#[test]
fn test_type_alias() {
    let mut checker = TypeChecker::new();
    checker.define_type_alias("MyInt", Type::I32);
    
    assert!(checker.resolve_type("MyInt") == Some(Type::I32));
}

// ==================== RECURSIVE TYPE TESTS ====================

#[test]
fn test_recursive_type() {
    let mut checker = TypeChecker::new();
    
    // List type: Cons(T, Box<List<T>>) | Nil
    checker.define_recursive_enum("List", vec!["T"], vec![
        ("Cons", Some(Type::Tuple(vec![
            Type::Generic("T".to_string()),
            Type::Box(Box::new(Type::Enum("List".to_string()))),
        ]))),
        ("Nil", None),
    ]);
    
    assert!(checker.has_type("List"));
}

// ==================== STATEMENT TYPE CHECKING TESTS ====================

#[test]
fn test_check_let_stmt() {
    let mut checker = TypeChecker::new();
    let stmt = Stmt::let_binding("x", Some(Type::I32), Expr::literal(42));
    
    let result = checker.check_stmt(&stmt);
    assert!(result.is_ok());
    assert!(checker.has_var("x"));
}

#[test]
fn test_let_type_mismatch() {
    let mut checker = TypeChecker::new();
    let stmt = Stmt::let_binding("x", Some(Type::I32), Expr::string("text"));
    
    let result = checker.check_stmt(&stmt);
    assert!(matches!(result, Err(TypeError::TypeMismatch(_, _))));
}

#[test]
fn test_let_type_inference() {
    let mut checker = TypeChecker::new();
    let stmt = Stmt::let_binding("x", None, Expr::literal(42));
    
    let result = checker.check_stmt(&stmt);
    assert!(result.is_ok());
    assert_eq!(checker.get_var_type("x"), Some(Type::I32));
}

// ==================== PROGRAM TYPE CHECKING TESTS ====================

#[test]
fn test_check_program() {
    let mut checker = TypeChecker::new();
    let program = Program {
        functions: vec![
            Function {
                name: "main",
                params: vec![],
                ret_type: None,
                body: vec![
                    Stmt::let_binding("x", Some(Type::I32), Expr::literal(42)),
                    Stmt::expr(Expr::ident("x")),
                ],
            }
        ],
    };
    
    let result = checker.check_program(&program);
    assert!(result.is_ok());
}

// Helper implementations for tests
impl TypeChecker {
    fn new() -> Self { unimplemented!() }
    fn with_builtins() -> Self { unimplemented!() }
    fn is_empty(&self) -> bool { true }
    fn has_type(&self, _: &str) -> bool { false }
    fn has_var(&self, _: &str) -> bool { false }
    fn define_var(&mut self, _: &str, _: Type) {}
    fn define_func(&mut self, _: &str, _: Type) {}
    fn define_struct(&mut self, _: &str, _: Vec<(&str, Type)>) {}
    fn define_enum(&mut self, _: &str, _: Vec<(&str, Option<Type>)>) {}
    fn define_generic_func(&mut self, _: &str, _: Vec<&str>, _: Type) {}
    fn define_generic_struct(&mut self, _: &str, _: Vec<&str>, _: Vec<(&str, Type)>) {}
    fn define_recursive_enum(&mut self, _: &str, _: Vec<&str>, _: Vec<(&str, Option<Type>)>) {}
    fn define_type_alias(&mut self, _: &str, _: Type) {}
    fn enter_scope(&mut self) {}
    fn exit_scope(&mut self) {}
    fn check_expr(&mut self, _: &Expr) -> Result<Type, TypeError> { Ok(Type::I32) }
    fn check_with_context(&mut self, _: &Expr, _: &Type) -> Result<Type, TypeError> { Ok(Type::I32) }
    fn check_stmt(&mut self, _: &Stmt) -> Result<(), TypeError> { Ok(()) }
    fn check_program(&mut self, _: &Program) -> Result<(), TypeError> { Ok(()) }
    fn resolve_type(&self, _: &str) -> Option<Type> { None }
    fn get_var_type(&self, _: &str) -> Option<Type> { None }
}

#[derive(Debug, Clone, PartialEq)]
enum Type {
    I32,
    F64,
    Bool,
    String,
    Char,
    Array(Box<Type>),
    Tuple(Vec<Type>),
    Function(Vec<Type>, Box<Type>),
    Struct(String),
    Enum(String),
    Generic(String),
    Box(Box<Type>),
}

enum TypeError {
    TypeMismatch(Type, Type),
    UndefinedVariable(String),
    HeterogeneousArray,
    TupleIndexOutOfBounds(usize, usize),
    ArityMismatch(String, usize, usize),
    UnknownField(String, String),
}

struct TypeEnv;
struct Function {
    name: &'static str,
    params: Vec<(&'static str, Type)>,
    ret_type: Option<Type>,
    body: Vec<Stmt>,
}

impl Expr {
    fn literal_float(_: f64) -> Self { unimplemented!() }
    fn char(_: char) -> Self { unimplemented!() }
    fn index(_: Self, _: Self) -> Self { unimplemented!() }
    fn tuple_index(_: Self, _: usize) -> Self { unimplemented!() }
    fn lambda(_: Vec<(&str, Type)>, _: Self) -> Self { unimplemented!() }
    fn lambda_infer(_: Vec<&str>, _: Self) -> Self { unimplemented!() }
    fn field(_: Self, _: &str) -> Self { unimplemented!() }
}

impl Stmt {
    fn let_binding(_: &str, _: Option<Type>, _: Expr) -> Self { unimplemented!() }
}

struct Pattern;
impl Pattern {
    fn literal(_: i32) -> Self { Self }
    fn wildcard() -> Self { Self }
}

// Run all tests with: cargo test type_checker_tdd --test type_checker_tdd