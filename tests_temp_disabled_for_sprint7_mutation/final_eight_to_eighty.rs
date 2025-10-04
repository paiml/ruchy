// The final 8 tests to reach 80% coverage!
// Simple, high-quality tests with single responsibility

use ruchy::compile;

#[cfg(test)]
mod final_eight {
    use super::*;

    #[test]
    fn test_continue_in_loop() {
        let code = "fn main() { for i in 0..10 { continue; } }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_break_in_loop() {
        let code = "fn main() { for i in 0..10 { break; } }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_return_in_function() {
        let code = "fn test() { return; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_char_literal() {
        let code = "fn main() { let c = 'a'; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_byte_literal() {
        let code = "fn main() { let b = b'A'; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_hex_literal() {
        let code = "fn main() { let x = 0xFF; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_binary_literal() {
        let code = "fn main() { let x = 0b1010; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_octal_literal() {
        let code = "fn main() { let x = 0o777; }";
        assert!(compile(code).is_ok());
    }
}
