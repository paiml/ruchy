/// TDD Tests for Float Comparison Operations
/// 
/// These tests MUST pass to fix the critical float comparison bug
/// affecting multiple chapters in the Ruchy book.

#[cfg(test)]
mod float_comparison_tests {
    use ruchy::runtime::repl::Repl;

    #[test]
    fn test_float_greater_than() {
        let mut repl = Repl::new().unwrap();
        let result = repl.eval("3.14 > 2.5").unwrap();
        assert_eq!(result, "true");
    }

    #[test]
    fn test_float_less_than() {
        let mut repl = Repl::new().unwrap();
        let result = repl.eval("2.5 < 3.14").unwrap();
        assert_eq!(result, "true");
    }

    #[test]
    fn test_float_greater_than_false() {
        let mut repl = Repl::new().unwrap();
        let result = repl.eval("2.5 > 3.14").unwrap();
        assert_eq!(result, "false");
    }

    #[test]
    fn test_float_less_than_false() {
        let mut repl = Repl::new().unwrap();
        let result = repl.eval("3.14 < 2.5").unwrap();
        assert_eq!(result, "false");
    }

    #[test]
    fn test_float_greater_equal() {
        let mut repl = Repl::new().unwrap();
        let result = repl.eval("3.14 >= 3.14").unwrap();
        assert_eq!(result, "true");
        
        let result2 = repl.eval("3.14 >= 2.5").unwrap();
        assert_eq!(result2, "true");
    }

    #[test]
    fn test_float_less_equal() {
        let mut repl = Repl::new().unwrap();
        let result = repl.eval("2.5 <= 2.5").unwrap();
        assert_eq!(result, "true");
        
        let result2 = repl.eval("2.5 <= 3.14").unwrap();
        assert_eq!(result2, "true");
    }

    #[test]
    fn test_float_equal() {
        let mut repl = Repl::new().unwrap();
        let result = repl.eval("3.14 == 3.14").unwrap();
        assert_eq!(result, "true");
        
        let result2 = repl.eval("3.14 == 2.5").unwrap();
        assert_eq!(result2, "false");
    }

    #[test]
    fn test_float_not_equal() {
        let mut repl = Repl::new().unwrap();
        let result = repl.eval("3.14 != 2.5").unwrap();
        assert_eq!(result, "true");
        
        let result2 = repl.eval("3.14 != 3.14").unwrap();
        assert_eq!(result2, "false");
    }

    #[test]
    fn test_mixed_int_float_comparison() {
        let mut repl = Repl::new().unwrap();
        
        // Int < Float
        let result = repl.eval("3 < 3.14").unwrap();
        assert_eq!(result, "true");
        
        // Float > Int
        let result2 = repl.eval("3.14 > 3").unwrap();
        assert_eq!(result2, "true");
        
        // Int == Float (exact)
        let result3 = repl.eval("3 == 3.0").unwrap();
        assert_eq!(result3, "true");
    }

    #[test]
    fn test_float_comparison_with_variables() {
        let mut repl = Repl::new().unwrap();
        repl.eval("let x = 3.14").unwrap();
        repl.eval("let y = 2.5").unwrap();
        
        let result = repl.eval("x > y").unwrap();
        assert_eq!(result, "true");
        
        let result2 = repl.eval("y < x").unwrap();
        assert_eq!(result2, "true");
    }

    #[test]
    fn test_float_comparison_edge_cases() {
        let mut repl = Repl::new().unwrap();
        
        // Very close floats (epsilon comparison)
        let result = repl.eval("1.0000000001 == 1.0").unwrap();
        // This should use epsilon comparison
        
        // Negative floats
        let result2 = repl.eval("-3.14 < -2.5").unwrap();
        assert_eq!(result2, "true");
        
        // Zero comparisons
        let result3 = repl.eval("0.0 < 0.1").unwrap();
        assert_eq!(result3, "true");
    }
}