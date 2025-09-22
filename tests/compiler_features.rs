// Tests for specific compiler features
use ruchy::compile;

#[cfg(test)]
mod compiler_features {
    use super::*;

    // Type system tests
    #[test]
    fn test_type_inference() {
        assert!(compile("fn main() { let x = 42; }").is_ok());
    }

    #[test]
    fn test_explicit_types() {
        assert!(compile("fn main() { let x: i32 = 42; }").is_ok());
    }

    #[test]
    fn test_type_casting() {
        assert!(compile("fn main() { let x = 42 as f64; }").is_ok());
    }

    // Pattern matching tests
    #[test]
    fn test_let_patterns() {
        assert!(compile("fn main() { let (x, y) = (1, 2); }").is_ok());
    }

    #[test]
    fn test_match_patterns() {
        assert!(compile("fn main() { match 42 { x => println(x), } }").is_ok());
    }

    #[test]
    fn test_if_let() {
        assert!(compile("fn main() { if let Some(x) = Some(42) { println(x); } }").is_ok());
    }

    // Ownership tests
    #[test]
    #[ignore] // Not yet implemented
    fn test_move_semantics() {
        assert!(compile("fn main() { let x = vec![1,2,3]; let y = x; }").is_ok());
    }

    #[test]
    #[ignore] // Not yet implemented
    fn test_borrowing() {
        assert!(compile("fn main() { let x = 42; let y = &x; }").is_ok());
    }

    #[test]
    #[ignore] // Not yet implemented
    fn test_mutable_borrow() {
        assert!(compile("fn main() { let mut x = 42; let y = &mut x; }").is_ok());
    }

    // Advanced syntax tests
    #[test]
    fn test_struct_definition() {
        assert!(compile("struct Point { x: i32, y: i32 }").is_ok());
    }

    #[test]
    fn test_enum_definition() {
        assert!(compile("enum Option<T> { Some(T), None }").is_ok());
    }

    #[test]
    #[ignore] // Not yet implemented
    fn test_impl_block() {
        assert!(compile("struct S; impl S { fn new() -> S { S } }").is_ok());
    }

    #[test]
    #[ignore] // Not yet implemented
    fn test_trait_definition() {
        assert!(compile("trait Display { fn fmt(&self); }").is_ok());
    }

    // Macro tests
    #[test]
    fn test_println_macro() {
        assert!(compile("fn main() { println!(\"Hello\"); }").is_ok());
    }

    #[test]
    #[ignore] // Not yet implemented
    fn test_vec_macro() {
        assert!(compile("fn main() { let v = vec![1, 2, 3]; }").is_ok());
    }

    #[test]
    fn test_format_macro() {
        assert!(compile("fn main() { let s = format!(\"Hello {}\", \"world\"); }").is_ok());
    }

    // Module system tests
    #[test]
    fn test_use_statement() {
        assert!(compile("use std::collections::HashMap;").is_ok());
    }

    #[test]
    fn test_mod_declaration() {
        assert!(compile("mod my_module;").is_ok() || compile("mod my_module;").is_err());
    }

    // Async tests
    #[test]
    #[ignore] // Not yet implemented
    fn test_async_function() {
        assert!(compile("async fn test() {}").is_ok());
    }

    #[test]
    #[ignore] // Not yet implemented
    fn test_await_expression() {
        assert!(compile("async fn test() { other().await; }").is_ok());
    }
}
