//! PARSER-067: Struct pattern matching tests
//!
//! This test suite validates that struct patterns in match expressions
//! correctly bind field values to variables.

use ruchy::runtime::repl::Repl;

#[test]
fn test_parser_067_simple_struct_pattern() {
    let code = r#"
struct Person { name: String }
let p = Person { name: "Alice" };
match p {
    Person { name } => name
}
"#;

    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let result = repl.eval(code);
    assert!(result.is_ok(), "Struct pattern matching should work: {result:?}");
}

#[test]
fn test_parser_067_struct_pattern_multiple_fields() {
    let code = r#"
struct Person { name: String, age: Integer }
let p = Person { name: "Bob", age: 30 };
match p {
    Person { name, age } => name + " is " + age.to_string()
}
"#;

    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let result = repl.eval(code);
    assert!(result.is_ok(), "Multi-field struct pattern should work: {result:?}");
}

#[test]
fn test_parser_067_struct_pattern_nested() {
    let code = r#"
struct Address { city: String }
struct Person { name: String, addr: Address }
let p = Person { name: "Carol", addr: Address { city: "NYC" } };
match p {
    Person { name, addr } => name + " lives in " + addr.city
}
"#;

    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let result = repl.eval(code);
    assert!(result.is_ok(), "Nested struct pattern should work: {result:?}");
}
