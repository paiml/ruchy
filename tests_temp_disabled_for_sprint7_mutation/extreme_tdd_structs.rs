//! Extreme TDD tests for struct functionality
//! These tests are written BEFORE implementation to drive development
//! Target: 100% struct feature coverage

use ruchy::compile;

// ==================== DEFAULT VALUES ====================

#[test]
fn test_struct_default_values_basic() {
    let code = r#"
        struct Config {
            host: String = "localhost",
            port: i32 = 8080,
            debug: bool = false
        }

        fn main() {
            let cfg = Config {}
            println(cfg.host)
            println(cfg.port)
            println(cfg.debug)
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Struct with default values should compile");
    let rust_code = result.unwrap();
    assert!(rust_code.contains("impl Default"));
    assert!(rust_code.contains(r#""localhost""#));
}

#[test]
fn test_struct_partial_defaults() {
    let code = r#"
        struct User {
            name: String,
            age: i32 = 18,
            active: bool = true
        }

        fn main() {
            let user = User { name: "Alice" }
            println(user.age)  // Should be 18
        }
    "#;
    let result = compile(code);
    assert!(
        result.is_ok(),
        "Struct with partial defaults should compile"
    );
}

#[test]
fn test_struct_override_defaults() {
    let code = r"
        struct Settings {
            timeout: i32 = 30,
            retries: i32 = 3
        }

        fn main() {
            let s1 = Settings {}
            let s2 = Settings { timeout: 60 }
            let s3 = Settings { timeout: 90, retries: 5 }
            println(s1.timeout)  // 30
            println(s2.timeout)  // 60
            println(s3.retries)  // 5
        }
    ";
    let result = compile(code);
    assert!(result.is_ok(), "Overriding defaults should work");
}

// ==================== VISIBILITY MODIFIERS ====================

#[test]
fn test_struct_visibility_modifiers() {
    let code = r"
        pub struct PublicUser {
            pub name: String,
            pub(crate) internal_id: i32,
            private_key: String
        }

        struct PrivateData {
            data: Vec<i32>
        }
    ";
    let result = compile(code);
    assert!(result.is_ok(), "Visibility modifiers should compile");
    let rust_code = result.unwrap();
    assert!(rust_code.contains("pub struct PublicUser"));
    // Check for pub name (with or without space before colon)
    assert!(rust_code.contains("pub name") && rust_code.contains("String"));
    // Check for pub(crate) internal_id (allowing space between pub and (crate))
    assert!(
        (rust_code.contains("pub(crate) internal_id")
            || rust_code.contains("pub (crate) internal_id"))
            && rust_code.contains("i32")
    );
}

#[test]
fn test_struct_pub_crate_visibility() {
    let code = r"
        pub(crate) struct InternalConfig {
            pub(crate) setting: String,
            value: i32
        }
    ";
    let result = compile(code);
    assert!(result.is_ok(), "pub(crate) visibility should work");
}

// ==================== PATTERN MATCHING ====================

#[test]
fn test_struct_pattern_matching_basic() {
    let code = r#"
        struct Point { x: f64, y: f64 }

        fn main() {
            let p = Point { x: 3.0, y: 4.0 }
            match p {
                Point { x: 0.0, y } => println("on y-axis"),
                Point { x, y: 0.0 } => println("on x-axis"),
                Point { x, y } => println(f"at {x}, {y}")
            }
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Struct pattern matching should work");
}

#[test]
fn test_struct_pattern_matching_nested() {
    let code = r#"
        struct Color { r: u8, g: u8, b: u8 }
        struct Pixel { pos: Point, color: Color }
        struct Point { x: i32, y: i32 }

        fn main() {
            let pixel = Pixel {
                pos: Point { x: 10, y: 20 },
                color: Color { r: 255, g: 0, b: 0 }
            }

            match pixel {
                Pixel { pos: Point { x: 0, y: 0 }, .. } => println("origin"),
                Pixel { color: Color { r: 255, .. }, .. } => println("red pixel"),
                _ => println("other")
            }
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Nested struct pattern matching should work");
}

#[test]
fn test_struct_pattern_with_guard() {
    let code = r#"
        struct Range { start: i32, end: i32 }

        fn main() {
            let r = Range { start: 5, end: 10 }
            match r {
                Range { start, end } if end > start => println("valid range"),
                Range { start, end } if start == end => println("single point"),
                _ => println("invalid range")
            }
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Pattern matching with guards should work");
}

#[test]
fn test_struct_destructuring_in_let() {
    let code = r"
        struct Vec3 { x: f64, y: f64, z: f64 }

        fn main() {
            let v = Vec3 { x: 1.0, y: 2.0, z: 3.0 }
            let Vec3 { x, y, z } = v
            println(x + y + z)
        }
    ";
    let result = compile(code);
    assert!(result.is_ok(), "Struct destructuring in let should work");
}

// ==================== DERIVE ATTRIBUTES ====================

#[test]
fn test_struct_derive_debug() {
    let code = r#"
        #[derive(Debug)]
        struct User {
            name: String,
            age: i32
        }

        fn main() {
            let user = User { name: "Bob", age: 30 }
            println(f"{user:?}")
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Derive Debug should work");
    let rust_code = result.unwrap();
    // Check for derive Debug (with or without spaces)
    assert!(
        rust_code.contains("derive") && rust_code.contains("Debug"),
        "Should contain derive Debug attribute"
    );
}

#[test]
fn test_struct_derive_multiple() {
    let code = r#"
        #[derive(Debug, Clone, PartialEq)]
        struct Data {
            value: i32,
            label: String
        }

        fn main() {
            let d1 = Data { value: 42, label: "test" }
            let d2 = d1.clone()
            assert(d1 == d2)
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Multiple derive attributes should work");
}

#[test]
fn test_struct_custom_derive() {
    let code = r"
        #[derive(Serialize, Deserialize)]
        struct ApiResponse {
            status: i32,
            message: String
        }
    ";
    let result = compile(code);
    assert!(result.is_ok(), "Custom derive macros should be supported");
}

// ==================== GENERIC STRUCTS ====================

#[test]
fn test_struct_generic_with_constraints() {
    let code = r#"
        struct Container<T: Display> {
            value: T,
            label: String
        }

        fn main() {
            let c1 = Container { value: 42, label: "number" }
            let c2 = Container { value: "text", label: "string" }
        }
    "#;
    let result = compile(code);
    assert!(
        result.is_ok(),
        "Generic structs with constraints should work"
    );
}

#[test]
fn test_struct_multiple_generics() {
    let code = r#"
        struct Pair<T, U> {
            first: T,
            second: U
        }

        fn main() {
            let p = Pair { first: 10, second: "ten" }
            println(p.first)
            println(p.second)
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Multiple generic parameters should work");
}

// ==================== METHODS ON STRUCTS ====================

#[test]
fn test_struct_impl_block() {
    let code = r"
        struct Rectangle {
            width: f64,
            height: f64
        }

        impl Rectangle {
            fn area(self) -> f64 {
                self.width * self.height
            }

            fn perimeter(self) -> f64 {
                2.0 * (self.width + self.height)
            }

            fn new(w: f64, h: f64) -> Rectangle {
                Rectangle { width: w, height: h }
            }
        }

        fn main() {
            let rect = Rectangle::new(10.0, 20.0)
            println(rect.area())
            println(rect.perimeter())
        }
    ";
    let result = compile(code);
    assert!(result.is_ok(), "Struct impl blocks should work");
}

#[test]
fn test_struct_mutable_methods() {
    let code = r"
        struct Counter {
            count: i32
        }

        impl Counter {
            fn increment(mut self) {
                self.count += 1
            }

            fn reset(mut self) {
                self.count = 0
            }

            fn value(self) -> i32 {
                self.count
            }
        }

        fn main() {
            let mut c = Counter { count: 0 }
            c.increment()
            c.increment()
            println(c.value())  // Should be 2
        }
    ";
    let result = compile(code);
    assert!(result.is_ok(), "Mutable methods should work");
}

// ==================== LIFETIME PARAMETERS ====================

#[test]
fn test_struct_lifetime_basic() {
    let code = r#"
        struct Reference<'a> {
            data: &'a str
        }

        fn main() {
            let text = "hello"
            let r = Reference { data: &text }
            println(r.data)
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Basic lifetime parameters should work");
}

#[test]
fn test_struct_multiple_lifetimes() {
    let code = r"
        struct TwoRefs<'a, 'b> {
            first: &'a str,
            second: &'b str
        }
    ";
    let result = compile(code);
    assert!(result.is_ok(), "Multiple lifetime parameters should work");
}

// ==================== TUPLE STRUCTS ====================

#[test]
fn test_tuple_struct_basic() {
    let code = r"
        struct Color(u8, u8, u8)

        fn main() {
            let red = Color(255, 0, 0)
            println(red.0)  // Access first element
        }
    ";
    let result = compile(code);
    assert!(result.is_ok(), "Tuple structs should work");
}

#[test]
fn test_newtype_pattern() {
    let code = r#"
        struct UserId(i32)
        struct Username(String)

        fn main() {
            let id = UserId(42)
            let name = Username("alice")
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Newtype pattern should work");
}

// ==================== UNIT STRUCTS ====================

#[test]
fn test_unit_struct() {
    let code = r"
        struct Marker
        struct Empty

        fn main() {
            let m = Marker
            let e = Empty
        }
    ";
    let result = compile(code);
    assert!(result.is_ok(), "Unit structs should work");
}

// ==================== ADVANCED FEATURES ====================

#[test]
fn test_struct_with_const_generics() {
    let code = r"
        struct Array<T, const N: usize> {
            data: [T; N]
        }

        fn main() {
            let arr = Array { data: [1, 2, 3, 4, 5] }
        }
    ";
    let result = compile(code);
    assert!(result.is_ok(), "Const generics should work");
}

#[test]
fn test_struct_field_init_shorthand() {
    let code = r#"
        struct Config {
            host: String,
            port: i32
        }

        fn main() {
            let host = "localhost"
            let port = 8080
            let cfg = Config { host, port }  // Field init shorthand
            println(cfg.host)
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Field init shorthand should work");
}

#[test]
fn test_struct_update_syntax() {
    let code = r"
        struct Settings {
            timeout: i32,
            retries: i32,
            verbose: bool
        }

        fn main() {
            let default = Settings { timeout: 30, retries: 3, verbose: false }
            let custom = Settings { timeout: 60, ..default }
            println(custom.retries)  // Should be 3
        }
    ";
    let result = compile(code);
    assert!(result.is_ok(), "Struct update syntax should work");
}
