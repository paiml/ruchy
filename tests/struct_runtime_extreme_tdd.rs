// EXTREME TDD: Struct Runtime Implementation Tests
// Following CLAUDE.md Toyota Way - ALL tests written FIRST before implementation
// This addresses RUCHY_RUNTIME_BUG_REPORT.md critical issues

use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::{Interpreter, Value};

fn eval_code(interpreter: &mut Interpreter, code: &str) -> Result<Value, String> {
    let mut parser = Parser::new(code);
    let expr = parser.parse().map_err(|e| e.to_string())?;

    // If the code contains a main function, evaluate the program then call main
    if code.contains("fn main()") {
        // Evaluate the parsed program (which defines functions and structs)
        interpreter.eval_expr(&expr).map_err(|e| e.to_string())?;

        // Now call main() function to get the actual result
        let main_call = Parser::new("main()").parse().map_err(|e| e.to_string())?;
        interpreter.eval_expr(&main_call).map_err(|e| e.to_string())
    } else {
        // For code without main function, just evaluate directly
        interpreter.eval_expr(&expr).map_err(|e| e.to_string())
    }
}

/// EXTREME TDD PHASE 1: Basic Struct Definition Tests (ALL SHOULD FAIL INITIALLY)
mod struct_definition_tests {
    use super::*;

    #[test]
    fn test_basic_struct_definition() {
        let mut interpreter = Interpreter::new();
        let code = r"
            struct Point {
                x: float,
                y: float
            }
        ";
        // EXTREME TDD: This MUST fail initially (runtime not implemented)
        let result = eval_code(&mut interpreter, code);
        assert!(result.is_ok(), "Struct definition should succeed");

        // Struct definition should return a type descriptor
        let value = result.unwrap();
        assert!(
            matches!(value, Value::Object(_)),
            "Struct definition should return object"
        );
    }

    #[test]
    fn test_struct_with_different_field_types() {
        let mut interpreter = Interpreter::new();
        let code = r"
            struct Person {
                name: string,
                age: i32,
                height: float,
                active: bool
            }
        ";
        let result = eval_code(&mut interpreter, code).expect("Should parse and evaluate");
        assert!(matches!(result, Value::Object(_)));
    }

    #[test]
    fn test_struct_with_single_field() {
        let mut interpreter = Interpreter::new();
        let code = r"
            struct Wrapper {
                value: i32
            }
        ";
        let result = eval_code(&mut interpreter, code).expect("Should parse and evaluate");
        assert!(matches!(result, Value::Object(_)));
    }

    #[test]
    fn test_empty_struct() {
        let mut interpreter = Interpreter::new();
        let code = r"
            struct Empty {
            }
        ";
        let result = eval_code(&mut interpreter, code).expect("Should parse and evaluate");
        assert!(matches!(result, Value::Object(_)));
    }

    #[test]
    fn test_public_struct() {
        let mut interpreter = Interpreter::new();
        let code = r"
            pub struct PublicStruct {
                field: i32
            }
        ";
        let result = eval_code(&mut interpreter, code).expect("Should parse and evaluate");
        assert!(matches!(result, Value::Object(_)));
    }
}

/// EXTREME TDD PHASE 2: Struct Instantiation Tests (ALL SHOULD FAIL INITIALLY)
mod struct_instantiation_tests {
    use super::*;

    #[test]
    fn test_basic_struct_instantiation() {
        let mut interpreter = Interpreter::new();
        let code = r"
            struct Point {
                x: float,
                y: float
            }

            fn main() {
                let p = Point { x: 3.0, y: 4.0 }
                p
            }
        ";
        // EXTREME TDD: This MUST fail initially (struct instantiation not implemented)
        let result = eval_code(&mut interpreter, code);
        assert!(result.is_ok(), "Struct instantiation should succeed");

        let value = result.unwrap();
        assert!(
            matches!(value, Value::Object(_)),
            "Struct instance should be object"
        );
    }

    #[test]
    fn test_struct_instantiation_with_different_types() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            struct Person {
                name: string,
                age: i32,
                active: bool
            }

            fn main() {
                let person = Person {
                    name: "Alice",
                    age: 30,
                    active: true
                }
                person
            }
        "#;
        let result = eval_code(&mut interpreter, code).expect("Should parse and evaluate");
        assert!(matches!(result, Value::Object(_)));
    }

    #[test]
    fn test_nested_struct_instantiation() {
        let mut interpreter = Interpreter::new();
        let code = r"
            struct Point {
                x: float,
                y: float
            }

            struct Line {
                start: Point,
                end: Point
            }

            fn main() {
                let line = Line {
                    start: Point { x: 0.0, y: 0.0 },
                    end: Point { x: 1.0, y: 1.0 }
                }
                line
            }
        ";
        let result = eval_code(&mut interpreter, code).expect("Should parse and evaluate");
        assert!(matches!(result, Value::Object(_)));
    }

    #[test]
    fn test_struct_field_access() {
        let mut interpreter = Interpreter::new();
        let code = r"
            struct Point {
                x: float,
                y: float
            }

            fn main() {
                let p = Point { x: 3.0, y: 4.0 }
                p.x
            }
        ";
        let result = eval_code(&mut interpreter, code).expect("Should parse and evaluate");
        assert!(matches!(result, Value::Float(_)));

        if let Value::Float(f) = result {
            assert_eq!(f, 3.0);
        }
    }

    #[test]
    fn test_struct_field_mutation() {
        let mut interpreter = Interpreter::new();
        let code = r"
            struct Counter {
                value: i32
            }

            fn main() {
                let mut counter = Counter { value: 0 }
                counter.value = 42
                counter.value
            }
        ";
        let result = eval_code(&mut interpreter, code).expect("Should parse and evaluate");
        assert!(matches!(result, Value::Integer(42)));
    }
}

/// EXTREME TDD PHASE 3: Integration with Functions Tests (ALL SHOULD FAIL INITIALLY)
mod struct_function_integration_tests {
    use super::*;

    #[test]
    fn test_struct_as_function_parameter() {
        let mut interpreter = Interpreter::new();
        let code = r"
            struct Point {
                x: float,
                y: float
            }

            fn distance_from_origin(p: Point) {
                sqrt(p.x * p.x + p.y * p.y)
            }

            fn main() {
                let p = Point { x: 3.0, y: 4.0 }
                distance_from_origin(p)
            }
        ";
        let result = eval_code(&mut interpreter, code).expect("Should parse and evaluate");
        assert!(matches!(result, Value::Float(_)));

        if let Value::Float(f) = result {
            assert!((f - 5.0).abs() < 0.001); // 3-4-5 triangle
        }
    }

    #[test]
    fn test_function_returning_struct() {
        let mut interpreter = Interpreter::new();
        let code = r"
            struct Point {
                x: float,
                y: float
            }

            fn make_point(x: float, y: float) {
                Point { x: x, y: y }
            }

            fn main() {
                let p = make_point(5.0, 10.0)
                p.x + p.y
            }
        ";
        let result = eval_code(&mut interpreter, code).expect("Should parse and evaluate");
        assert!(matches!(result, Value::Float(15.0)));
    }

    #[test]
    fn test_struct_method_calls() {
        let mut interpreter = Interpreter::new();
        let code = r"
            struct Point {
                x: float,
                y: float
            }

            impl Point {
                fn new(x: float, y: float) {
                    Point { x: x, y: y }
                }

                fn distance_from_origin(self) {
                    sqrt(self.x * self.x + self.y * self.y)
                }
            }

            fn main() {
                let p = Point::new(3.0, 4.0)
                p.distance_from_origin()
            }
        ";
        let result = eval_code(&mut interpreter, code).expect("Should parse and evaluate");
        assert!(matches!(result, Value::Float(_)));

        if let Value::Float(f) = result {
            assert!((f - 5.0).abs() < 0.001);
        }
    }
}

/// EXTREME TDD PHASE 4: Error Handling Tests (ALL SHOULD FAIL INITIALLY)
mod struct_error_handling_tests {
    use super::*;

    #[test]
    fn test_struct_with_undefined_field_type() {
        let mut interpreter = Interpreter::new();
        let code = r"
            struct Bad {
                field: UndefinedType
            }
        ";
        let result = eval_code(&mut interpreter, code);
        assert!(result.is_err(), "Should fail with undefined type");
    }

    #[test]
    fn test_struct_instantiation_missing_field() {
        let mut interpreter = Interpreter::new();
        let code = r"
            struct Point {
                x: float,
                y: float
            }

            fn main() {
                let p = Point { x: 3.0 }  // Missing y field
                p
            }
        ";
        let result = eval_code(&mut interpreter, code);
        assert!(result.is_err(), "Should fail with missing field");
    }

    #[test]
    fn test_struct_instantiation_extra_field() {
        let mut interpreter = Interpreter::new();
        let code = r"
            struct Point {
                x: float,
                y: float
            }

            fn main() {
                let p = Point { x: 3.0, y: 4.0, z: 5.0 }  // Extra z field
                p
            }
        ";
        let result = eval_code(&mut interpreter, code);
        assert!(result.is_err(), "Should fail with extra field");
    }

    #[test]
    fn test_access_nonexistent_field() {
        let mut interpreter = Interpreter::new();
        let code = r"
            struct Point {
                x: float,
                y: float
            }

            fn main() {
                let p = Point { x: 3.0, y: 4.0 }
                p.z  // Nonexistent field
            }
        ";
        let result = eval_code(&mut interpreter, code);
        assert!(result.is_err(), "Should fail accessing nonexistent field");
    }
}

/// EXTREME TDD PHASE 5: Complex Struct Scenarios (ALL SHOULD FAIL INITIALLY)
mod struct_advanced_tests {
    use super::*;

    #[test]
    fn test_recursive_struct_definition() {
        let mut interpreter = Interpreter::new();
        let code = r"
            struct Node {
                value: i32,
                next: Option<Node>
            }

            fn main() {
                let node = Node {
                    value: 42,
                    next: None
                }
                node.value
            }
        ";
        let result = eval_code(&mut interpreter, code).expect("Should parse and evaluate");
        assert!(matches!(result, Value::Integer(42)));
    }

    #[test]
    fn test_struct_with_array_field() {
        let mut interpreter = Interpreter::new();
        let code = r"
            struct Container {
                items: [i32]
            }

            fn main() {
                let container = Container {
                    items: [1, 2, 3, 4, 5]
                }
                container.items.length()
            }
        ";
        let result = eval_code(&mut interpreter, code).expect("Should parse and evaluate");
        assert!(matches!(result, Value::Integer(5)));
    }

    #[test]
    fn test_struct_equality() {
        let mut interpreter = Interpreter::new();
        let code = r"
            struct Point {
                x: float,
                y: float
            }

            fn main() {
                let p1 = Point { x: 3.0, y: 4.0 }
                let p2 = Point { x: 3.0, y: 4.0 }
                p1 == p2
            }
        ";
        let result = eval_code(&mut interpreter, code).expect("Should parse and evaluate");
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_struct_string_representation() {
        let mut interpreter = Interpreter::new();
        let code = r"
            struct Point {
                x: float,
                y: float
            }

            fn main() {
                let p = Point { x: 3.0, y: 4.0 }
                p.to_string()
            }
        ";
        let result = eval_code(&mut interpreter, code).expect("Should parse and evaluate");
        assert!(matches!(result, Value::String(_)));
    }
}

/// EXTREME TDD PHASE 6: Real-World Usage Tests (ALL SHOULD FAIL INITIALLY)
mod struct_real_world_tests {
    use super::*;

    #[test]
    fn test_point_and_rectangle_example() {
        let mut interpreter = Interpreter::new();
        let code = r"
            struct Point {
                x: float,
                y: float
            }

            struct Rectangle {
                top_left: Point,
                bottom_right: Point
            }

            fn main() {
                let rect = Rectangle {
                    top_left: Point { x: 0.0, y: 0.0 },
                    bottom_right: Point { x: 10.0, y: 5.0 }
                }

                let width = rect.bottom_right.x - rect.top_left.x
                let height = rect.bottom_right.y - rect.top_left.y
                width * height  // Area calculation
            }
        ";
        let result = eval_code(&mut interpreter, code).expect("Should parse and evaluate");
        assert!(matches!(result, Value::Float(50.0)));
    }

    #[test]
    fn test_person_database_example() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            struct Person {
                name: string,
                age: i32,
                email: string
            }

            fn main() {
                let people = [
                    Person { name: "Alice", age: 30, email: "alice@example.com" },
                    Person { name: "Bob", age: 25, email: "bob@example.com" },
                    Person { name: "Charlie", age: 35, email: "charlie@example.com" }
                ]

                people.length()
            }
        "#;
        let result = eval_code(&mut interpreter, code).expect("Should parse and evaluate");
        assert!(matches!(result, Value::Integer(3)));
    }
}

/// EXTREME TDD PHASE 7: Performance and Edge Cases (ALL SHOULD FAIL INITIALLY)
mod struct_edge_cases {
    use super::*;

    #[test]
    fn test_large_struct_with_many_fields() {
        let mut interpreter = Interpreter::new();
        let code = r"
            struct LargeStruct {
                f1: i32, f2: i32, f3: i32, f4: i32, f5: i32,
                f6: i32, f7: i32, f8: i32, f9: i32, f10: i32
            }

            fn main() {
                let large = LargeStruct {
                    f1: 1, f2: 2, f3: 3, f4: 4, f5: 5,
                    f6: 6, f7: 7, f8: 8, f9: 9, f10: 10
                }
                large.f1 + large.f10
            }
        ";
        let result = eval_code(&mut interpreter, code).expect("Should parse and evaluate");
        assert!(matches!(result, Value::Integer(11)));
    }

    #[test]
    fn test_deeply_nested_struct_access() {
        let mut interpreter = Interpreter::new();
        let code = r"
            struct A { b: B }
            struct B { c: C }
            struct C { d: D }
            struct D { value: i32 }

            fn main() {
                let a = A {
                    b: B {
                        c: C {
                            d: D { value: 42 }
                        }
                    }
                }
                a.b.c.d.value
            }
        ";
        let result = eval_code(&mut interpreter, code).expect("Should parse and evaluate");
        assert!(matches!(result, Value::Integer(42)));
    }

    #[test]
    fn test_struct_with_zero_sized_fields() {
        let mut interpreter = Interpreter::new();
        let code = r"
            struct Unit {}
            struct WithUnit {
                unit: Unit,
                value: i32
            }

            fn main() {
                let with_unit = WithUnit {
                    unit: Unit {},
                    value: 123
                }
                with_unit.value
            }
        ";
        let result = eval_code(&mut interpreter, code).expect("Should parse and evaluate");
        assert!(matches!(result, Value::Integer(123)));
    }
}
