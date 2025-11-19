#![allow(missing_docs)]
// Test for BOOK-COMPAT-001: &str lifetime annotations in structs
// GitHub Issue: https://github.com/paiml/ruchy/issues/50
// Priority: HIGH - Blocks 100% book compatibility (Ch19 Ex2)

#[cfg(test)]
mod book_compat_001_tests {
    use ruchy::backend::transpiler::Transpiler;
    use ruchy::frontend::parser::Parser;

    #[test]
    fn test_book_compat_001_struct_with_str_reference() {
        // RED PHASE: This test SHOULD FAIL with current implementation
        // Ch19 Example 2 from ruchy-book
        let input = r#"
struct Person {
    name: &str,
    age: i32,
    height: f64
}

fun main() {
    let alice = Person {
        name: "Alice",
        age: 30,
        height: 5.6
    }
    println(alice.name)
}
"#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().expect("Failed to parse");

        let mut transpiler = Transpiler::new();
        let result = transpiler
            .transpile_to_program(&ast)
            .expect("Failed to transpile");
        let code = result.to_string();

        // ASSERTION 1: Struct should have lifetime parameter <'a> (with flexible spacing)
        let has_lifetime_param = code.contains("struct Person<'a>")
            || code.contains("struct Person < 'a >")
            || code.contains("struct Person<'a >");
        assert!(
            has_lifetime_param,
            "Expected struct Person<'a>, but got:\n{code}"
        );

        // ASSERTION 2: name field should have &'a str type (with flexible spacing)
        let has_lifetime_field = code.contains("name : &'a str")
            || code.contains("name: &'a str")
            || code.contains("name : & 'a str");
        assert!(
            has_lifetime_field,
            "Expected name field to have type &'a str, but got:\n{code}"
        );
    }

    #[test]
    fn test_book_compat_001_struct_without_references_unchanged() {
        // This should continue to work (no lifetimes needed)
        let input = r"
struct Point {
    x: i32,
    y: i32
}
";

        let mut parser = Parser::new(input);
        let ast = parser.parse().expect("Failed to parse");

        let mut transpiler = Transpiler::new();
        let result = transpiler
            .transpile_to_program(&ast)
            .expect("Failed to transpile");
        let code = result.to_string();

        // ASSERTION: Struct should NOT have lifetime parameter
        assert!(
            code.contains("struct Point") && !code.contains("struct Point<"),
            "Struct without references should not have lifetime parameter:\n{code}"
        );
    }

    #[test]
    fn test_book_compat_001_multiple_str_references() {
        // Test struct with multiple &str fields
        let input = r"
struct Book {
    title: &str,
    author: &str,
    pages: i32
}
";

        let mut parser = Parser::new(input);
        let ast = parser.parse().expect("Failed to parse");

        let mut transpiler = Transpiler::new();
        let result = transpiler
            .transpile_to_program(&ast)
            .expect("Failed to transpile");
        let code = result.to_string();

        // ASSERTION: Struct should have lifetime parameter (with flexible spacing)
        let has_lifetime_param = code.contains("struct Book<'a>")
            || code.contains("struct Book < 'a >")
            || code.contains("struct Book<'a >");
        assert!(
            has_lifetime_param,
            "Expected struct Book<'a>, but got:\n{code}"
        );
    }

    #[test]
    fn test_transpiler_001_string_literal_no_to_string() {
        // TRANSPILER-001: String literals should NOT call .to_string() in struct initialization
        // This is the bug found when trying to compile Ch19 Ex2
        let input = r#"
struct Person {
    name: &str,
    age: i32
}

fun main() {
    let alice = Person {
        name: "Alice",
        age: 30
    }
}
"#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().expect("Failed to parse");

        let mut transpiler = Transpiler::new();
        let result = transpiler
            .transpile_to_program(&ast)
            .expect("Failed to transpile");
        let code = result.to_string();

        // ASSERTION: String literals should NOT have .to_string() call
        assert!(
            !code.contains(r#""Alice" . to_string ()"#) && !code.contains(r#""Alice".to_string()"#),
            "String literals should not call .to_string() in struct initialization, but got:\n{code}"
        );

        // ASSERTION: Should contain just the string literal
        assert!(
            code.contains(r#""Alice""#),
            "Expected string literal \"Alice\" without .to_string(), but got:\n{code}"
        );
    }
}
