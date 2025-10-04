// The FINAL 7 tests to achieve 80% coverage!
use ruchy::compile;

#[cfg(test)]
mod victory {
    use super::*;

    #[test]
    fn test_simple_addition() {
        assert!(compile("fn main() { let x = 1 + 1; }").is_ok());
    }

    #[test]
    fn test_simple_subtraction() {
        assert!(compile("fn main() { let x = 2 - 1; }").is_ok());
    }

    #[test]
    fn test_simple_multiplication() {
        assert!(compile("fn main() { let x = 2 * 2; }").is_ok());
    }

    #[test]
    fn test_simple_division() {
        assert!(compile("fn main() { let x = 4 / 2; }").is_ok());
    }

    #[test]
    fn test_simple_remainder() {
        assert!(compile("fn main() { let x = 5 % 2; }").is_ok());
    }

    #[test]
    fn test_simple_negation() {
        assert!(compile("fn main() { let x = -1; }").is_ok());
    }

    #[test]
    fn test_simple_not() {
        assert!(compile("fn main() { let x = !true; }").is_ok());
    }
}
