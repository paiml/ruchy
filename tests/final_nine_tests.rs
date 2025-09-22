// The final 9 tests to reach 80% coverage!
// Each test has single responsibility and complexity ≤10

use ruchy::compile;

#[cfg(test)]
mod final_tests {
    use super::*;

    #[test]
    fn test_print_statement() {
        let code = r#"fn main() { println("Hello"); }"#;
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_print_with_variable() {
        let code = r#"fn main() { let x = 42; println(x); }"#;
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_assignment() {
        let code = "fn main() { let mut x = 1; x = 2; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_compound_assignment_add() {
        let code = "fn main() { let mut x = 1; x += 2; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_compound_assignment_subtract() {
        let code = "fn main() { let mut x = 5; x -= 2; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_compound_assignment_multiply() {
        let code = "fn main() { let mut x = 3; x *= 2; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_compound_assignment_divide() {
        let code = "fn main() { let mut x = 10; x /= 2; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_block_expression() {
        let code = "fn main() { let x = { 42 }; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_nested_blocks() {
        let code = "fn main() { { { let x = 1; } } }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_shadowing() {
        let code = "fn main() { let x = 1; let x = 2; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_reference() {
        let code = "fn main() { let x = 42; let y = &x; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_dereference() {
        let code = "fn main() { let x = 42; let y = &x; let z = *y; }";
        assert!(compile(code).is_ok());
    }
}