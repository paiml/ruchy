//! EXTREME TDD: Unit tests for named constructors in classes
//! Named constructors allow multiple constructor variants like: new square(size)

use ruchy::{Parser, Transpiler};

#[test]
fn test_simple_named_constructor() {
    let code = r"
        class Rectangle {
            width: f64,
            height: f64,

            new square(size: f64) {
                self.width = size
                self.height = size
            }
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Named constructor should become a static method
    assert!(result_str.contains("struct Rectangle"));
    assert!(result_str.contains("impl Rectangle"));
    // Named constructor becomes: fn square(size: f64) -> Self
    assert!(
        result_str.contains("fn square") || result_str.contains("fn new_square"),
        "Should have named constructor method"
    );
    assert!(
        result_str.contains("-> Self"),
        "Named constructor should return Self"
    );
}

#[test]
fn test_multiple_named_constructors() {
    let code = r"
        class Shape {
            x: f64,
            y: f64,
            radius: f64,

            new(x: f64, y: f64, radius: f64) {
                self.x = x
                self.y = y
                self.radius = radius
            }

            new circle(x: f64, y: f64, r: f64) {
                self.x = x
                self.y = y
                self.radius = r
            }

            new unit_circle() {
                self.x = 0.0
                self.y = 0.0
                self.radius = 1.0
            }

            new at_origin(radius: f64) {
                self.x = 0.0
                self.y = 0.0
                self.radius = radius
            }
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Should have all constructor variants
    assert!(
        result_str.contains("fn new"),
        "Should have primary constructor"
    );
    assert!(
        result_str.contains("fn circle") || result_str.contains("fn new_circle"),
        "Should have circle constructor"
    );
    assert!(
        result_str.contains("fn unit_circle") || result_str.contains("fn new_unit_circle"),
        "Should have unit_circle constructor"
    );
    assert!(
        result_str.contains("fn at_origin") || result_str.contains("fn new_at_origin"),
        "Should have at_origin constructor"
    );
}

#[test]
fn test_named_constructor_with_validation() {
    let code = r#"
        class PositiveNumber {
            value: f64,

            new checked(val: f64) -> Result<Self> {
                if val <= 0.0 {
                    Err("Value must be positive")
                } else {
                    Ok(PositiveNumber { value: val })
                }
            }
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Named constructor with Result return type
    assert!(result_str.contains("fn checked") || result_str.contains("fn new_checked"));
    assert!(
        result_str.contains("-> Result"),
        "Should have Result return type"
    );
}

#[test]
fn test_pub_named_constructor() {
    let code = r"
        class Point {
            x: i32,
            y: i32,

            pub new origin() {
                self.x = 0
                self.y = 0
            }

            new from_x(x: i32) {
                self.x = x
                self.y = 0
            }
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Public named constructor
    assert!(result_str.contains("pub fn origin") || result_str.contains("pub fn new_origin"));
    // Private named constructor should not have pub
    assert!(!result_str.contains("pub fn from_x") && !result_str.contains("pub fn new_from_x"));
}

#[test]
fn test_named_constructor_ast_structure() {
    let code = r"
        class Test {
            value: i32,

            new zero() {
                self.value = 0
            }
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    // Verify AST structure for named constructors
    if let ruchy::frontend::ast::ExprKind::Class { constructors, .. } = &ast.kind {
        assert_eq!(constructors.len(), 1, "Should have 1 constructor");
        let constructor = &constructors[0];
        assert!(
            constructor.name.is_some(),
            "Named constructor should have a name"
        );
        assert_eq!(constructor.name.as_ref().unwrap(), "zero");
    } else {
        panic!("AST should be a class");
    }
}

#[test]
fn test_mixed_regular_and_named_constructors() {
    let code = r#"
        class Config {
            host: String,
            port: i32,

            new(host: String, port: i32) {
                self.host = host
                self.port = port
            }

            new localhost(port: i32) {
                self.host = "localhost"
                self.port = port
            }

            new defaults() {
                self.host = "0.0.0.0"
                self.port = 8080
            }
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    // Should have 3 constructors total
    if let ruchy::frontend::ast::ExprKind::Class { constructors, .. } = &ast.kind {
        assert_eq!(constructors.len(), 3, "Should have 3 constructors");

        // Check constructor names
        assert!(
            constructors[0].name.is_none(),
            "First should be unnamed (primary)"
        );
        assert_eq!(constructors[1].name.as_ref().unwrap(), "localhost");
        assert_eq!(constructors[2].name.as_ref().unwrap(), "defaults");
    } else {
        panic!("AST should be a class");
    }
}
