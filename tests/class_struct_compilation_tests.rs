//! EXTREME TDD: Class/Struct Compilation Tests
//! Target: Complete OOP support with ALL functions ≤10 complexity
//!
//! Following the same EXTREME TDD approach as previous features:
//! 1. Write ALL tests FIRST (this file)
//! 2. All tests should FAIL initially
//! 3. Implement parser/transpiler to make tests pass
//! 4. Maintain Toyota Way: ≤10 complexity per function
//! 5. Add comprehensive property tests with 10,000 iterations

use ruchy::compile;

#[cfg(test)]
mod class_struct_compilation {
    use super::*;

    // =============================================================================
    // BASIC STRUCT TESTS
    // =============================================================================

    #[test]
    fn test_simple_struct_definition() {
        let code = r"
            struct Point {
                x: f64,
                y: f64
            }

            fun main() {
                Point { x: 1.0, y: 2.0 }
            }
        ";

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile simple struct: {result:?}"
        );
        let output = result.unwrap();
        assert!(output.contains("struct Point"));
        assert!(output.contains("x : f64"));
        assert!(output.contains("y : f64"));
    }

    #[test]
    fn test_struct_instantiation() {
        let code = r#"
            struct Person {
                name: String,
                age: i32
            }

            fun main() {
                let person = Person { name: "Alice", age: 30 };
                person
            }
        "#;

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile struct instantiation: {result:?}"
        );
        let output = result.unwrap();
        assert!(output.contains("Person"));
        assert!(output.contains("name :"));
        assert!(output.contains("age :"));
    }

    #[test]
    fn test_struct_field_access() {
        let code = r"
            struct Rectangle {
                width: f64,
                height: f64
            }

            fun main() {
                let rect = Rectangle { width: 10.0, height: 20.0 };
                rect.width + rect.height
            }
        ";

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile struct field access: {result:?}"
        );
        let output = result.unwrap();
        assert!(output.contains("width") && output.contains("height"));
    }

    // =============================================================================
    // CLASS DEFINITION TESTS
    // =============================================================================

    #[test]
    fn test_simple_class_definition() {
        let code = r"
            class Point {
                x: f64,
                y: f64
            }

            fun main() {
                Point { x: 1.0, y: 2.0 }
            }
        ";

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile simple class: {result:?}"
        );
        let output = result.unwrap();
        // Class should transpile to struct
        assert!(output.contains("struct Point"));
    }

    #[test]
    fn test_class_with_constructor() {
        let code = r"
            class Point {
                x: f64,
                y: f64,

                new(x: f64, y: f64) {
                    Self { x, y }
                }
            }

            fun main() {
                Point::new(1.0, 2.0)
            }
        ";

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile class with constructor: {result:?}"
        );
        let output = result.unwrap();
        // Constructor should become associated function
        assert!(output.contains("fn new"));
        assert!(output.contains("impl Point"));
    }

    #[test]
    fn test_class_with_methods() {
        let code = r"
            class Rectangle {
                width: f64,
                height: f64,

                new(width: f64, height: f64) {
                    Self { width, height }
                }

                fun area(&self) -> f64 {
                    self.width * self.height
                }

                fun perimeter(&self) -> f64 {
                    2.0 * (self.width + self.height)
                }
            }

            fun main() {
                let rect = Rectangle::new(5.0, 3.0);
                rect.area()
            }
        ";

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile class with methods: {result:?}"
        );
        let output = result.unwrap();
        assert!(output.contains("fn area"));
        assert!(output.contains("fn perimeter"));
        assert!(output.contains("impl Rectangle"));
    }

    // =============================================================================
    // GENERIC STRUCT/CLASS TESTS
    // =============================================================================

    #[test]
    fn test_generic_struct() {
        let code = r"
            struct Container<T> {
                value: T
            }

            fun main() {
                let int_container = Container { value: 42 };
                int_container.value
            }
        ";

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile generic struct: {result:?}"
        );
        let output = result.unwrap();
        assert!(output.contains("Container"));
        assert!(output.contains("<T>") || output.contains('T'));
    }

    #[test]
    fn test_generic_class_with_methods() {
        let code = r"
            class Box<T> {
                value: T,

                new(value: T) -> Self {
                    Self { value }
                }

                fun get(&self) -> &T {
                    &self.value
                }

                fun set(&mut self, value: T) {
                    self.value = value;
                }
            }

            fun main() {
                let mut box = Box::new(42);
                box.set(100);
                box.get()
            }
        ";

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile generic class: {result:?}"
        );
        let output = result.unwrap();
        assert!(output.contains("impl") && output.contains("Box"));
    }

    // =============================================================================
    // STRUCT/CLASS WITH DEFAULT VALUES
    // =============================================================================

    #[test]
    fn test_struct_with_default_values() {
        let code = r"
            struct Config {
                debug: bool = false,
                max_connections: i32 = 100,
                timeout: f64 = 30.0
            }

            fun main() {
                let config = Config { debug: true, ..Default::default() };
                config.max_connections
            }
        ";

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile struct with defaults: {result:?}"
        );
    }

    #[test]
    fn test_class_with_default_constructor() {
        let code = r#"
            class Settings {
                name: String = "default",
                version: i32 = 1,

                new() {
                    Self::default()
                }

                new(name: String) {
                    Self { name, version: 1 }
                }
            }

            fun main() {
                Settings::new()
            }
        "#;

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile class with default constructor: {result:?}"
        );
    }

    // =============================================================================
    // VISIBILITY MODIFIERS
    // =============================================================================

    #[test]
    fn test_struct_with_visibility() {
        let code = r#"
            pub struct PublicStruct {
                pub name: String,
                private_id: i32,
                pub(crate) internal_flag: bool
            }

            fun main() {
                PublicStruct {
                    name: "test",
                    private_id: 1,
                    internal_flag: true
                }
            }
        "#;

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile struct with visibility: {result:?}"
        );
        let output = result.unwrap();
        assert!(output.contains("pub struct"));
        assert!(output.contains("pub name"));
    }

    #[test]
    fn test_class_with_private_methods() {
        let code = r#"
            class BankAccount {
                pub balance: f64,
                account_number: String,

                pub new(balance: f64, account_number: String) {
                    Self { balance, account_number }
                }

                pub fun deposit(&mut self, amount: f64) {
                    self.balance += amount;
                }

                fun validate_amount(&self, amount: f64) -> bool {
                    amount > 0.0 && amount < 10000.0
                }
            }

            fun main() {
                let mut account = BankAccount::new(100.0, "123456");
                account.deposit(50.0);
                account.balance
            }
        "#;

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile class with private methods: {result:?}"
        );
    }

    // =============================================================================
    // TRAIT IMPLEMENTATIONS
    // =============================================================================

    #[test]
    fn test_class_with_trait_impl() {
        let code = r#"
            class Point {
                x: f64,
                y: f64,

                impl Display {
                    fun fmt(&self, f: &mut Formatter) -> fmt::Result {
                        write!(f, "Point({}, {})", self.x, self.y)
                    }
                }
            }

            fun main() {
                Point { x: 1.0, y: 2.0 }
            }
        "#;

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile class with trait impl: {result:?}"
        );
        let output = result.unwrap();
        assert!(
            output.contains("impl Display for Point")
                || output.contains("impl") && output.contains("Display")
        );
    }

    #[test]
    fn test_derive_attributes() {
        let code = r"
            #[derive(Debug, Clone, PartialEq)]
            struct Vector3 {
                x: f64,
                y: f64,
                z: f64
            }

            fun main() {
                Vector3 { x: 1.0, y: 2.0, z: 3.0 }
            }
        ";

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile struct with derive: {result:?}"
        );
        let output = result.unwrap();
        assert!(output.contains("derive") || output.contains("Debug"));
    }

    // =============================================================================
    // PROPERTIES (GETTERS/SETTERS)
    // =============================================================================

    #[test]
    fn test_class_with_properties() {
        let code = r"
            class Temperature {
                celsius: f64,

                new(celsius: f64) {
                    Self { celsius }
                }

                property fahrenheit: f64 {
                    get => self.celsius * 9.0/5.0 + 32.0,
                    set(value) => self.celsius = (value - 32.0) * 5.0/9.0
                }

                property kelvin: f64 {
                    get => self.celsius + 273.15
                }
            }

            fun main() {
                let temp = Temperature::new(25.0);
                temp.fahrenheit
            }
        ";

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile class with properties: {result:?}"
        );
        let output = result.unwrap();
        // Properties should become getter/setter methods
        assert!(output.contains("fahrenheit") && (output.contains("get") || output.contains("fn")));
    }

    // =============================================================================
    // STATIC METHODS AND ASSOCIATED CONSTANTS
    // =============================================================================

    #[test]
    fn test_class_with_static_methods() {
        let code = r"
            class MathUtils {
                const PI: f64 = 3.14159,

                static fun add(a: f64, b: f64) -> f64 {
                    a + b
                }

                static fun circle_area(radius: f64) -> f64 {
                    Self::PI * radius * radius
                }
            }

            fun main() {
                MathUtils::add(1.0, 2.0)
            }
        ";

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile class with static methods: {result:?}"
        );
        let output = result.unwrap();
        assert!(output.contains("const PI"));
    }

    // =============================================================================
    // NESTED STRUCTS/CLASSES
    // =============================================================================

    #[test]
    fn test_nested_structs() {
        let code = r#"
            struct Address {
                street: String,
                city: String,
                zip: String
            }

            struct Person {
                name: String,
                address: Address,
                age: i32
            }

            fun main() {
                let person = Person {
                    name: "Alice",
                    address: Address {
                        street: "123 Main St",
                        city: "Anytown",
                        zip: "12345"
                    },
                    age: 30
                };
                person.address.city
            }
        "#;

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile nested structs: {result:?}"
        );
    }

    // =============================================================================
    // PATTERN MATCHING WITH STRUCTS
    // =============================================================================

    #[test]
    fn test_struct_pattern_matching() {
        let code = r"
            struct Point {
                x: i32,
                y: i32
            }

            fun main() {
                let point = Point { x: 0, y: 5 };
                match point {
                    Point { x: 0, y } => y,
                    Point { x, y: 0 } => x,
                    Point { x, y } => x + y
                }
            }
        ";

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile struct pattern matching: {result:?}"
        );
    }

    // =============================================================================
    // ADVANCED FEATURES
    // =============================================================================

    #[test]
    fn test_struct_with_lifetime_parameters() {
        let code = r#"
            struct Borrower<'a> {
                data: &'a str
            }

            fun main() {
                let text = "hello";
                let borrower = Borrower { data: &text };
                borrower.data
            }
        "#;

        let result = compile(code);
        // This might not be implemented yet, but should not panic
        let _ = result;
    }

    #[test]
    fn test_sealed_class_enums() {
        let code = r"
            sealed class Shape {
                Circle { radius: f64 },
                Rectangle { width: f64, height: f64 },
                Triangle { base: f64, height: f64 },

                fun area(&self) -> f64 {
                    match self {
                        Shape::Circle { radius } => 3.14159 * radius * radius,
                        Shape::Rectangle { width, height } => width * height,
                        Shape::Triangle { base, height } => 0.5 * base * height,
                    }
                }
            }

            fun main() {
                let circle = Shape::Circle { radius: 5.0 };
                circle.area()
            }
        ";

        let result = compile(code);
        // Advanced feature - may not be implemented yet
        let _ = result;
    }

    // =============================================================================
    // ERROR CASES (Should fail gracefully)
    // =============================================================================

    #[test]
    fn test_malformed_struct_syntax() {
        let code = r"
            struct Point
                x: f64,
                y: f64
            }

            fun main() {
                Point { x: 1.0, y: 2.0 }
            }
        ";

        let result = compile(code);
        // Should fail parsing but not panic
        let _ = result;
    }

    #[test]
    fn test_missing_field_in_instantiation() {
        let code = r"
            struct Point {
                x: f64,
                y: f64
            }

            fun main() {
                Point { x: 1.0 }  // Missing y field
            }
        ";

        let result = compile(code);
        // May fail type checking, but should not panic
        let _ = result;
    }

    #[test]
    fn test_accessing_private_field() {
        let code = r#"
            class User {
                pub name: String,
                private_id: i32,

                pub new(name: String, id: i32) {
                    Self { name, private_id: id }
                }
            }

            fun main() {
                let user = User::new("Alice", 123);
                user.private_id  // Should not be accessible
            }
        "#;

        let result = compile(code);
        // May fail during compilation, but should not panic
        let _ = result;
    }
}
