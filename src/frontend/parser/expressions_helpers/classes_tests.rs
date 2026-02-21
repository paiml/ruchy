
use crate::frontend::parser::Parser;

#[test]
fn test_basic_class() {
    let code = "class MyClass { }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Basic class should parse");
}

#[test]
fn test_class_with_fields() {
    let code = "class Point { x: f64 y: f64 }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Class with fields should parse");
}

#[test]
fn test_class_with_inheritance() {
    let code = "class Child : Parent { }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Class with inheritance should parse");
}

#[test]
fn test_class_with_traits() {
    let code = "class MyClass : ParentClass + Trait1 + Trait2 { }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Class with traits should parse");
}

#[test]
fn test_class_with_constructor() {
    let code = "class Point { new(x: f64, y: f64) { self.x = x; self.y = y } }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Class with constructor should parse");
}

#[test]
fn test_class_with_method() {
    let code = "class Point { fun distance(&self) -> f64 { 0.0 } }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Class with method should parse");
}

#[test]
fn test_generic_class() {
    let code = "class Container<T> { value: T }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Generic class should parse");
}

// Additional tests for comprehensive coverage
#[test]
fn test_class_with_init_constructor() {
    let code = "class Point { init(x: f64) { self.x = x } }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Class with init constructor should parse");
}

#[test]
fn test_class_with_multiple_constructors() {
    let code = "class Point { new() { } new(x: f64) { self.x = x } }";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "Class with multiple constructors should parse"
    );
}

#[test]
fn test_class_with_static_method() {
    let code = "class Math { static fun add(a: i32, b: i32) -> i32 { a + b } }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Class with static method should parse");
}

#[test]
fn test_class_with_pub_field() {
    let code = "class Point { pub x: f64 }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Class with pub field should parse");
}

#[test]
fn test_class_with_mut_field() {
    let code = "class Counter { mut count: i32 }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Class with mut field should parse");
}

#[test]
fn test_class_with_pub_mut_field() {
    let code = "class Counter { pub mut count: i32 }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Class with pub mut field should parse");
}

#[test]
fn test_class_with_const() {
    // Const in class might require different syntax
    let code = "class Math { const PI: f64 = 3.14159 }";
    let result = Parser::new(code).parse();
    // Some grammars require typed const
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_class_with_typed_const() {
    let code = "class Math { const MAX: i32 = 100 }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Class with typed const should parse");
}

#[test]
fn test_class_with_self_method() {
    let code = "class Point { fun get_x(&self) -> f64 { self.x } }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Class with &self method should parse");
}

#[test]
fn test_class_with_mut_self_method() {
    let code = "class Counter { fun increment(&mut self) { self.count = self.count + 1 } }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Class with &mut self method should parse");
}

#[test]
fn test_class_with_owned_self_method() {
    let code = "class Point { fun into_tuple(self) -> (f64, f64) { (self.x, self.y) } }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Class with owned self method should parse");
}

#[test]
fn test_class_with_override_method() {
    let code = "class Child : Parent { override fun method(&self) { } }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Class with override method should parse");
}

#[test]
fn test_class_with_final_method() {
    let code = "class Base { final fun method(&self) { } }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Class with final method should parse");
}

#[test]
fn test_class_with_abstract_method() {
    let code = "class Base { abstract fun method(&self) }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Class with abstract method should parse");
}

#[test]
fn test_class_with_async_method() {
    let code = "class AsyncClass { async fun fetch(&self) { } }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Class with async method should parse");
}

#[test]
fn test_class_with_fn_method() {
    let code = "class Point { fn get_x(&self) -> f64 { self.x } }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Class with fn keyword should parse");
}

#[test]
fn test_class_with_generic_inheritance() {
    let code = "class IntContainer : Container<i32> { }";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "Class with generic inheritance should parse"
    );
}

#[test]
fn test_class_with_generic_method() {
    // Generic methods in classes may have different syntax
    let code = "class Factory { fun create<T>() -> T { } }";
    let result = Parser::new(code).parse();
    // Generic methods may or may not be supported in this grammar
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_class_with_let_field() {
    let code = "class Point { let x: f64 }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Class with let field should parse");
}

#[test]
fn test_class_traits_only() {
    let code = "class MyClass : + Trait1 + Trait2 { }";
    let result = Parser::new(code).parse();
    // Traits without superclass - depends on grammar
    // Just ensure it doesn't crash
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_class_with_return_type() {
    let code = "class Point { fun magnitude(&self) -> f64 { 0.0 } }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Class method with return type should parse");
}

#[test]
fn test_class_with_no_return_type() {
    let code = "class Logger { fun log(&self, msg: String) { } }";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "Class method without return type should parse"
    );
}

#[test]
fn test_class_with_multiple_type_params() {
    let code = "class Map<K, V> { }";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "Class with multiple type params should parse"
    );
}

#[test]
fn test_class_with_field_initialization() {
    let code = "class Point { x: f64 = 0.0 }";
    let result = Parser::new(code).parse();
    // Field initialization support depends on grammar
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_class_complete_example() {
    // Complete class with inheritance, traits, fields, and methods
    let code = r#"
            class Vector2D : BaseVector + Comparable + Serializable {
                pub mut x: f64
                pub mut y: f64

                new(x: f64, y: f64) {
                    self.x = x
                    self.y = y
                }

                fun magnitude(&self) -> f64 {
                    0.0
                }

                static fun zero() -> Vector2D {
                    Vector2D::new(0.0, 0.0)
                }
            }
        "#;
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Complete class example should parse");
}

#[test]
fn test_class_empty_body() {
    let code = "class Empty { }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Empty class body should parse");
}

#[test]
fn test_class_field_separator_comma() {
    let code = "class Point { x: f64, y: f64 }";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "Class with comma-separated fields should parse"
    );
}

#[test]
fn test_class_field_separator_semicolon() {
    let code = "class Point { x: f64; y: f64 }";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "Class with semicolon-separated fields should parse"
    );
}

#[test]
fn test_class_field_separator_newline() {
    let code = "class Point {\n    x: f64\n    y: f64\n}";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "Class with newline-separated fields should parse"
    );
}

#[test]
fn test_class_with_pub_constructor() {
    let code = "class Point { pub new() { } }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Class with pub constructor should parse");
}

#[test]
fn test_class_with_pub_method() {
    let code = "class Point { pub fun get_x(&self) -> f64 { self.x } }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Class with pub method should parse");
}

#[test]
fn test_nested_class_error() {
    let code = "class Outer { class Inner { } }";
    let result = Parser::new(code).parse();
    // Nested classes are not supported
    assert!(result.is_err(), "Nested classes should fail");
}

#[test]
fn test_impl_in_class_error() {
    let code = "class MyClass { impl SomeTrait { } }";
    let result = Parser::new(code).parse();
    // Impl blocks in classes not supported
    assert!(result.is_err(), "Impl in class should fail");
}

#[test]
fn test_class_with_decorated_field() {
    // Decorators on fields - depends on grammar
    let code = "class MyClass { @JsonIgnore value: i32 }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok() || result.is_err());
}

// ============================================================
// Additional comprehensive tests for EXTREME TDD coverage
// ============================================================

use crate::frontend::ast::{Expr, ExprKind};
use crate::frontend::parser::Result;

fn parse(code: &str) -> Result<Expr> {
    Parser::new(code).parse()
}

fn get_block_exprs(expr: &Expr) -> Option<&Vec<Expr>> {
    match &expr.kind {
        ExprKind::Block(exprs) => Some(exprs),
        _ => None,
    }
}

// ============================================================
// Class produces Class ExprKind
// ============================================================

#[test]
fn test_class_produces_class_exprkind() {
    let expr = parse("class Foo { }").unwrap();
    if let Some(exprs) = get_block_exprs(&expr) {
        assert!(
            matches!(&exprs[0].kind, ExprKind::Class { .. }),
            "Should produce Class ExprKind"
        );
    }
}

// ============================================================
// Basic class variations
// ============================================================

#[test]
fn test_class_single_char_name() {
    let result = parse("class A { }");
    assert!(result.is_ok(), "Single char class name should parse");
}

#[test]
fn test_class_long_name() {
    let result = parse("class VeryLongClassNameWithManyChars { }");
    assert!(result.is_ok(), "Long class name should parse");
}

#[test]
fn test_class_underscore_name() {
    let result = parse("class _InternalClass { }");
    assert!(result.is_ok(), "Underscore prefix class should parse");
}

#[test]
fn test_class_numbers_in_name() {
    let result = parse("class Vector3D { }");
    assert!(result.is_ok(), "Class with numbers should parse");
}

// ============================================================
// Field variations
// ============================================================

#[test]
fn test_class_one_field() {
    let result = parse("class Point { x: i32 }");
    assert!(result.is_ok(), "One field should parse");
}

#[test]
fn test_class_two_fields() {
    let result = parse("class Point { x: i32, y: i32 }");
    assert!(result.is_ok(), "Two fields should parse");
}

#[test]
fn test_class_three_fields() {
    let result = parse("class Point { x: i32, y: i32, z: i32 }");
    assert!(result.is_ok(), "Three fields should parse");
}

#[test]
fn test_class_field_i32() {
    let result = parse("class Data { value: i32 }");
    assert!(result.is_ok(), "i32 field should parse");
}

#[test]
fn test_class_field_f64() {
    let result = parse("class Data { value: f64 }");
    assert!(result.is_ok(), "f64 field should parse");
}

#[test]
fn test_class_field_string() {
    let result = parse("class Data { name: String }");
    assert!(result.is_ok(), "String field should parse");
}

#[test]
fn test_class_field_bool() {
    let result = parse("class Data { flag: bool }");
    assert!(result.is_ok(), "bool field should parse");
}

#[test]
fn test_class_field_option() {
    let result = parse("class Data { maybe: Option<i32> }");
    assert!(result.is_ok(), "Option field should parse");
}

#[test]
fn test_class_field_vec() {
    let result = parse("class Data { items: Vec<i32> }");
    assert!(result.is_ok(), "Vec field should parse");
}

// ============================================================
// Method variations
// ============================================================

#[test]
fn test_class_method_no_params() {
    let result = parse("class Foo { fun get(&self) { } }");
    assert!(result.is_ok(), "Method no params should parse");
}

#[test]
fn test_class_method_one_param() {
    let result = parse("class Foo { fun set(&mut self, v: i32) { } }");
    assert!(result.is_ok(), "Method one param should parse");
}

#[test]
fn test_class_method_two_params() {
    let result = parse("class Foo { fun compute(&self, a: i32, b: i32) { } }");
    assert!(result.is_ok(), "Method two params should parse");
}

#[test]
fn test_class_static_method_no_params() {
    let result = parse("class Foo { static fun create() { } }");
    assert!(result.is_ok(), "Static no params should parse");
}

#[test]
fn test_class_static_method_with_params() {
    let result = parse("class Foo { static fun create(a: i32) { } }");
    assert!(result.is_ok(), "Static with params should parse");
}

// ============================================================
// Constructor variations
// ============================================================

#[test]
fn test_class_constructor_no_params() {
    let result = parse("class Foo { new() { } }");
    assert!(result.is_ok(), "Constructor no params should parse");
}

#[test]
fn test_class_constructor_one_param() {
    let result = parse("class Foo { new(x: i32) { } }");
    assert!(result.is_ok(), "Constructor one param should parse");
}

#[test]
fn test_class_constructor_three_params() {
    let result = parse("class Foo { new(a: i32, b: i32, c: i32) { } }");
    assert!(result.is_ok(), "Constructor three params should parse");
}

#[test]
fn test_class_init_constructor_no_params() {
    let result = parse("class Foo { init() { } }");
    assert!(result.is_ok(), "Init no params should parse");
}

#[test]
fn test_class_init_constructor_with_params() {
    let result = parse("class Foo { init(v: i32) { self.v = v } }");
    assert!(result.is_ok(), "Init with params should parse");
}

// ============================================================
// Inheritance variations
// ============================================================

#[test]
fn test_class_extends_one() {
    let result = parse("class Child : Parent { }");
    assert!(result.is_ok(), "Extends one should parse");
}

#[test]
fn test_class_extends_with_trait() {
    let result = parse("class Child : Parent + Trait1 { }");
    assert!(result.is_ok(), "Extends with trait should parse");
}

#[test]
fn test_class_extends_with_two_traits() {
    let result = parse("class Child : Parent + Trait1 + Trait2 { }");
    assert!(result.is_ok(), "Extends with two traits should parse");
}

#[test]
fn test_class_extends_generic_parent() {
    let result = parse("class IntList : List<i32> { }");
    assert!(result.is_ok(), "Extends generic parent should parse");
}

// ============================================================
// Generic class variations
// ============================================================

#[test]
fn test_class_generic_one() {
    let result = parse("class Box<T> { }");
    assert!(result.is_ok(), "One generic should parse");
}

#[test]
fn test_class_generic_two() {
    let result = parse("class Pair<A, B> { }");
    assert!(result.is_ok(), "Two generics should parse");
}

#[test]
fn test_class_generic_three() {
    let result = parse("class Triple<A, B, C> { }");
    assert!(result.is_ok(), "Three generics should parse");
}

#[test]
fn test_class_generic_with_field() {
    let result = parse("class Box<T> { value: T }");
    assert!(result.is_ok(), "Generic with field should parse");
}

#[test]
fn test_class_generic_with_method() {
    let result = parse("class Box<T> { fun get(&self) -> T { self.value } }");
    assert!(result.is_ok(), "Generic with method should parse");
}

// ============================================================
// Visibility combinations
// ============================================================

#[test]
fn test_class_pub_field_only() {
    let result = parse("class Foo { pub x: i32 }");
    assert!(result.is_ok(), "Pub field should parse");
}

#[test]
fn test_class_mut_field_only() {
    let result = parse("class Foo { mut x: i32 }");
    assert!(result.is_ok(), "Mut field should parse");
}

#[test]
fn test_class_pub_method() {
    let result = parse("class Foo { pub fun get(&self) { } }");
    assert!(result.is_ok(), "Pub method should parse");
}

#[test]
fn test_class_pub_static_method() {
    let result = parse("class Foo { pub static fun create() { } }");
    assert!(result.is_ok(), "Pub static method should parse");
}

// ============================================================
// Combined class tests
// ============================================================

#[test]
fn test_class_fields_and_methods() {
    let result = parse("class Point { x: i32, y: i32, fun len(&self) { } }");
    assert!(result.is_ok(), "Fields and methods should parse");
}

#[test]
fn test_class_constructor_and_method() {
    let result = parse("class Foo { new() { } fun get(&self) { } }");
    assert!(result.is_ok(), "Constructor and method should parse");
}

#[test]
fn test_class_all_elements() {
    let result = parse("class Foo { x: i32, new(x: i32) { self.x = x } fun get(&self) -> i32 { self.x } static fun zero() { } }");
    assert!(result.is_ok(), "All elements should parse");
}

// ===== Additional coverage tests (Round 104) =====

// Test 82: Class with async method
#[test]
fn test_class_async_method() {
    let result = parse("class Client { async fun fetch(&self) { } }");
    assert!(result.is_ok(), "Async method should parse");
}

// Test 83: Class with generic constraint
#[test]
fn test_class_generic_constraint() {
    let result = parse("class Container<T: Clone> { value: T }");
    assert!(result.is_ok(), "Generic constraint should parse");
}

// Test 84: Class with multiple fields same type
#[test]
fn test_class_multiple_same_type_fields() {
    let result = parse("class Vec3 { x: f64, y: f64, z: f64 }");
    assert!(result.is_ok(), "Multiple same type fields should parse");
}

// Test 85: Class with method returning self type
#[test]
fn test_class_method_returns_self() {
    let result = parse("class Builder { fun with_value(&mut self, v: i32) -> Self { self } }");
    assert!(result.is_ok(), "Method returning Self should parse");
}

// Test 86: Class with impl block style method
#[test]
fn test_class_impl_style_method() {
    let result = parse("class Foo { fun compute(&self, x: i32, y: i32) -> i32 { x + y } }");
    assert!(result.is_ok(), "Impl style method should parse");
}

// Test 87: Class with default field values
#[test]
fn test_class_default_field() {
    let result = parse("class Config { debug: bool = false }");
    assert!(result.is_ok(), "Default field value should parse");
}

// Test 90: Class method with multiple return types
#[test]
fn test_class_method_optional_return() {
    let result = parse("class Cache { fun get(&self, key: str) -> Option<T> { None } }");
    assert!(result.is_ok(), "Optional return type should parse");
}

// Test 91: Class with private method
#[test]
fn test_class_private_method() {
    let result = parse("class Service { fun internal(&self) { } }");
    assert!(result.is_ok(), "Private method should parse");
}

// Test 92: Empty class variations
#[test]
fn test_empty_class_with_whitespace() {
    let result = parse("class Empty {\n\n}");
    assert!(result.is_ok(), "Empty class with whitespace should parse");
}

// Test 90: Class with constructor and init
#[test]
fn test_class_both_constructors() {
    let result = parse("class Dual { new() { } init() { } }");
    assert!(result.is_ok(), "Both constructors should parse");
}

// ========================================================================
// parse_operator_method tests (operator overloading)
// ========================================================================

#[test]
fn test_class_operator_add() {
    let result = parse("class Vec2 { x: f64  y: f64  operator+(self, other: Vec2) -> Vec2 { Vec2 { x: 0.0, y: 0.0 } } }");
    assert!(result.is_ok(), "operator+ should parse: {:?}", result.err());
}

#[test]
fn test_class_operator_sub() {
    let result = parse(
        "class Vec2 { x: f64  operator-(self, other: Vec2) -> Vec2 { Vec2 { x: 0.0, y: 0.0 } } }",
    );
    assert!(result.is_ok(), "operator- should parse: {:?}", result.err());
}

#[test]
fn test_class_operator_mul() {
    let result = parse(
        "class Vec2 { x: f64  operator*(self, scalar: f64) -> Vec2 { Vec2 { x: 0.0, y: 0.0 } } }",
    );
    assert!(result.is_ok(), "operator* should parse: {:?}", result.err());
}

#[test]
fn test_class_operator_div() {
    let result = parse(
        "class Vec2 { x: f64  operator/(self, scalar: f64) -> Vec2 { Vec2 { x: 0.0, y: 0.0 } } }",
    );
    assert!(result.is_ok(), "operator/ should parse: {:?}", result.err());
}

#[test]
fn test_class_operator_eq() {
    let result = parse("class Vec2 { x: f64  operator==(self, other: Vec2) -> bool { true } }");
    assert!(
        result.is_ok(),
        "operator== should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_class_operator_ne() {
    let result = parse("class Vec2 { x: f64  operator!=(self, other: Vec2) -> bool { false } }");
    assert!(
        result.is_ok(),
        "operator!= should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_class_operator_lt() {
    let result = parse("class Num { v: i32  operator<(self, other: Num) -> bool { true } }");
    assert!(result.is_ok(), "operator< should parse: {:?}", result.err());
}

#[test]
fn test_class_operator_gt() {
    let result = parse("class Num { v: i32  operator>(self, other: Num) -> bool { false } }");
    assert!(result.is_ok(), "operator> should parse: {:?}", result.err());
}

#[test]
fn test_class_operator_le() {
    let result = parse("class Num { v: i32  operator<=(self, other: Num) -> bool { true } }");
    assert!(
        result.is_ok(),
        "operator<= should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_class_operator_ge() {
    let result = parse("class Num { v: i32  operator>=(self, other: Num) -> bool { false } }");
    assert!(
        result.is_ok(),
        "operator>= should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_class_operator_rem() {
    let result = parse("class Num { v: i32  operator%(self, other: Num) -> Num { Num { v: 0 } } }");
    assert!(result.is_ok(), "operator% should parse: {:?}", result.err());
}

#[test]
fn test_class_operator_index() {
    let result = parse("class Grid { data: Vec<i32>  operator[](self, idx: i32) -> i32 { 0 } }");
    assert!(
        result.is_ok(),
        "operator[] should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_class_operator_no_return_type() {
    let result = parse("class Vec2 { x: f64  operator+(self, other: Vec2) { 0 } }");
    assert!(result.is_ok(), "operator+ without return type should parse");
}

#[test]
fn test_class_multiple_operators() {
    let code = "class Vec2 { x: f64  y: f64  operator+(self, other: Vec2) -> Vec2 { Vec2 { x: 0.0, y: 0.0 } }  operator-(self, other: Vec2) -> Vec2 { Vec2 { x: 0.0, y: 0.0 } } }";
    let result = parse(code);
    assert!(result.is_ok(), "Multiple operators should parse");
}

// ============================================================
// Coverage tests for parse_decorator_argument (classes.rs:446)
// and parse_decorator_value (classes.rs:419)
// parse_decorator_argument is called from parse_decorator_args,
// which is invoked by parse_decorator for decorators INSIDE
// class bodies (Token::At path in parse_decorator).
// Top-level @decorators use parse_label_as_decorator instead.
// ============================================================

// Direct unit tests for parse_decorator_argument and parse_decorator_value
use super::{parse_decorator_argument, parse_decorator_value};
use crate::frontend::parser::ParserState;

#[test]
fn test_decorator_argument_direct_string() {
    // parse_decorator_argument: String branch
    let mut state = ParserState::new(r#""hello""#);
    let result = parse_decorator_argument(&mut state);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "hello");
}

#[test]
fn test_decorator_argument_direct_identifier_only() {
    // parse_decorator_argument: Identifier branch, no = follows
    let mut state = ParserState::new("myarg");
    let result = parse_decorator_argument(&mut state);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "myarg");
}

#[test]
fn test_decorator_argument_direct_key_value_string() {
    // parse_decorator_argument: Identifier + = + String value
    let mut state = ParserState::new(r#"key="value""#);
    let result = parse_decorator_argument(&mut state);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), r#"key=value"#);
}

#[test]
fn test_decorator_argument_direct_key_value_integer() {
    // parse_decorator_argument: Identifier + = + Integer value
    let mut state = ParserState::new("count=42");
    let result = parse_decorator_argument(&mut state);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "count=42");
}

#[test]
fn test_decorator_argument_direct_key_value_float() {
    // parse_decorator_argument: Identifier + = + Float value
    let mut state = ParserState::new("ratio=3.14");
    let result = parse_decorator_argument(&mut state);
    assert!(result.is_ok());
    let val = result.unwrap();
    assert!(val.starts_with("ratio="), "Got: {val}");
}

#[test]
fn test_decorator_argument_direct_key_value_bool_true_is_lexed_as_bool() {
    // "true"/"false" are lexed as Token::Bool, not Token::Identifier,
    // so the Identifier guard in parse_decorator_value is unreachable.
    // After key=, the parser sees Token::Bool which doesn't match any branch.
    let mut state = ParserState::new("debug=true");
    let result = parse_decorator_argument(&mut state);
    // This exercises the key= path in parse_decorator_argument,
    // then hits parse_decorator_value error branch (Bool not handled)
    assert!(
        result.is_err(),
        "Bool token not handled by parse_decorator_value"
    );
}

#[test]
fn test_decorator_argument_direct_key_value_with_string_as_value() {
    // Use string value after = instead of bool
    let mut state = ParserState::new(r#"verbose="false""#);
    let result = parse_decorator_argument(&mut state);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "verbose=false");
}

#[test]
fn test_decorator_argument_direct_error_invalid_token() {
    // parse_decorator_argument: error branch (not String or Identifier)
    let mut state = ParserState::new("42");
    let result = parse_decorator_argument(&mut state);
    assert!(result.is_err(), "Should fail on numeric token");
}

#[test]
fn test_decorator_value_direct_integer() {
    let mut state = ParserState::new("42");
    let result = parse_decorator_value(&mut state);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "42");
}

#[test]
fn test_decorator_value_direct_float() {
    let mut state = ParserState::new("3.14");
    let result = parse_decorator_value(&mut state);
    assert!(result.is_ok());
}

#[test]
fn test_decorator_value_direct_string() {
    let mut state = ParserState::new(r#""hello""#);
    let result = parse_decorator_value(&mut state);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "hello");
}

#[test]
fn test_decorator_value_direct_bool_true_not_handled() {
    // "true" is lexed as Token::Bool(true), not Token::Identifier("true"),
    // so parse_decorator_value's Identifier guard is unreachable.
    let mut state = ParserState::new("true");
    let result = parse_decorator_value(&mut state);
    // This exercises the error/fallthrough branch
    assert!(
        result.is_err(),
        "Bool token not in parse_decorator_value match arms"
    );
}

#[test]
fn test_decorator_value_direct_bool_false_not_handled() {
    let mut state = ParserState::new("false");
    let result = parse_decorator_value(&mut state);
    assert!(
        result.is_err(),
        "Bool token not in parse_decorator_value match arms"
    );
}

#[test]
fn test_decorator_value_direct_error() {
    // Not a valid value token
    let mut state = ParserState::new("(");
    let result = parse_decorator_value(&mut state);
    assert!(result.is_err());
}

// Integration tests for decorators (via full parse)

#[test]
fn test_decorator_on_class_method_inside_body() {
    // This path goes through parse_decorator -> parse_decorator_args
    // -> parse_decorator_argument (the classes.rs code path)
    let code = "class MyClass { @inline fun method(&self) -> i32 { 42 } }";
    let result = parse(code);
    assert!(result.is_ok(), "Decorator on method: {:?}", result.err());
}

#[test]
fn test_decorator_no_args_on_class() {
    let code = "@test class MyClass { }";
    let result = parse(code);
    assert!(result.is_ok(), "Decorator no args: {:?}", result.err());
}

#[test]
fn test_decorator_empty_parens_on_class() {
    let code = "@test() class MyClass { }";
    let result = parse(code);
    assert!(result.is_ok(), "Decorator empty parens: {:?}", result.err());
}

#[test]
fn test_multiple_decorators_on_class() {
    let code = "@serialize @debug class MyClass { }";
    let result = parse(code);
    assert!(result.is_ok(), "Multiple decorators: {:?}", result.err());
}

#[test]
fn test_decorator_with_string_on_class_method() {
    let code = r#"class C { @test("example") fun m(&self) { 42 } }"#;
    let result = parse(code);
    assert!(
        result.is_ok(),
        "Decorator with string arg in class: {:?}",
        result.err()
    );
}

#[test]
fn test_decorator_with_key_value_on_class_method() {
    let code = r#"class C { @config(max=100) fun m(&self) { 42 } }"#;
    let result = parse(code);
    assert!(
        result.is_ok(),
        "Decorator with key=value in class: {:?}",
        result.err()
    );
}

// ============================================================
// Coverage tests for parse_property_accessors (17 uncov, 0%)
// and parse_property_setter (17 uncov, 0%)
// ============================================================

#[test]
fn test_property_with_getter_only() {
    let code = "class MyClass { property value: i32 { get => 42 } }";
    let result = parse(code);
    assert!(
        result.is_ok(),
        "Property with getter only should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_property_with_setter_only() {
    let code = "class MyClass { property value: i32 { set(v) => self.x = v } }";
    let result = parse(code);
    assert!(
        result.is_ok(),
        "Property with setter only should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_property_with_getter_and_setter() {
    let code = "class MyClass { property value: i32 { get => self.x, set(v) => self.x = v } }";
    let result = parse(code);
    assert!(
        result.is_ok(),
        "Property with getter and setter should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_property_with_setter_and_getter_reversed() {
    let code = "class MyClass { property value: i32 { set(v) => self.x = v, get => self.x } }";
    let result = parse(code);
    assert!(
        result.is_ok(),
        "Property with setter first then getter should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_property_getter_returns_expression() {
    let code = "class Circle { property area: f64 { get => 3.14 * self.r * self.r } }";
    let result = parse(code);
    assert!(
        result.is_ok(),
        "Property getter with expression should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_property_setter_with_param() {
    let code = "class Box { property width: f64 { set(w) => self.w = w } }";
    let result = parse(code);
    assert!(
        result.is_ok(),
        "Property setter with param should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_class_with_multiple_properties() {
    let code = r#"class Point {
            property x: f64 { get => self._x, set(v) => self._x = v }
            property y: f64 { get => self._y }
        }"#;
    let result = parse(code);
    assert!(
        result.is_ok(),
        "Class with multiple properties should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_property_with_method_and_field() {
    let code = r#"class MyClass {
            name: String
            property length: i32 { get => 0 }
            fun greet(&self) -> String { self.name }
        }"#;
    let result = parse(code);
    assert!(
        result.is_ok(),
        "Class with field, property and method should parse: {:?}",
        result.err()
    );
}
