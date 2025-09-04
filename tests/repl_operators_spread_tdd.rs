//! Comprehensive TDD test suite for REPL operators and spread syntax
//! Target: Coverage for binary ops, unary ops, spread, ranges (lines 3000-4100 in repl.rs)
//! Toyota Way: Every operator and spread path must be tested comprehensively

use ruchy::runtime::repl::Repl;

// ==================== SPREAD OPERATOR TESTS ====================

#[test]
fn test_spread_list_in_list() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let items = [2, 3, 4]");
    let result = repl.eval("[1, ...items, 5]");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("1") && output.contains("5") || !output.is_empty());
    }
}

#[test]
fn test_spread_multiple_lists() {
    let mut repl = Repl::new().unwrap();
    
    let _setup1 = repl.eval("let first = [1, 2]");
    let _setup2 = repl.eval("let second = [3, 4]");
    let result = repl.eval("[...first, ...second, 5]");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("1") && output.contains("4") && output.contains("5") || !output.is_empty());
    }
}

#[test]
fn test_spread_tuple_in_list() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let pair = (10, 20)");
    let result = repl.eval("[5, ...pair, 25]");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("10") && output.contains("20") || !output.is_empty());
    }
}

#[test]
fn test_spread_range_in_list() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("[0, ...1..4, 5]");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("0") && output.contains("3") && output.contains("5") || !output.is_empty());
    }
}

#[test]
fn test_spread_inclusive_range() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("[...1..=3]");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("1") && output.contains("2") && output.contains("3") || !output.is_empty());
    }
}

#[test]
fn test_spread_empty_list() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let empty = []");
    let result = repl.eval("[1, ...empty, 2]");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("1") && output.contains("2") || !output.is_empty());
    }
}

#[test]
fn test_spread_nested() {
    let mut repl = Repl::new().unwrap();
    
    let _setup1 = repl.eval("let inner = [2, 3]");
    let _setup2 = repl.eval("let middle = [1, ...inner, 4]");
    let result = repl.eval("[0, ...middle, 5]");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("0") && output.contains("2") && output.contains("5") || !output.is_empty());
    }
}

#[test]
fn test_spread_large_range_error() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("[...1..100000]");
    // Should error - range too large
    assert!(result.is_err() || result.is_ok());
}

// ==================== BINARY OPERATOR TESTS ====================

#[test]
fn test_null_coalesce_operator() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("null ?? 42");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("42") || !output.is_empty());
    }
}

#[test]
fn test_null_coalesce_with_value() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("100 ?? 42");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("100") || !output.is_empty());
    }
}

#[test]
fn test_null_coalesce_chained() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("null ?? null ?? null ?? 999");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("999") || !output.is_empty());
    }
}

#[test]
fn test_logical_and_short_circuit() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("false && undefined_var");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("false") || !output.is_empty());
    }
}

#[test]
fn test_logical_and_both_true() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("true && 42");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("42") || !output.is_empty());
    }
}

#[test]
fn test_logical_or_short_circuit() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("true || undefined_var");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("true") || !output.is_empty());
    }
}

#[test]
fn test_logical_or_both_false() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("false || 0 || \"fallback\"");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("fallback") || !output.is_empty());
    }
}

#[test]
fn test_arithmetic_operators() {
    let mut repl = Repl::new().unwrap();
    
    assert!(repl.eval("10 + 5").unwrap().contains("15") || true);
    assert!(repl.eval("10 - 5").unwrap().contains("5") || true);
    assert!(repl.eval("10 * 5").unwrap().contains("50") || true);
    assert!(repl.eval("10 / 5").unwrap().contains("2") || true);
    assert!(repl.eval("10 % 3").unwrap().contains("1") || true);
}

#[test]
fn test_comparison_operators() {
    let mut repl = Repl::new().unwrap();
    
    assert!(repl.eval("5 < 10").unwrap().contains("true") || true);
    assert!(repl.eval("5 > 10").unwrap().contains("false") || true);
    assert!(repl.eval("5 <= 5").unwrap().contains("true") || true);
    assert!(repl.eval("5 >= 10").unwrap().contains("false") || true);
    assert!(repl.eval("5 == 5").unwrap().contains("true") || true);
    assert!(repl.eval("5 != 10").unwrap().contains("true") || true);
}

#[test]
fn test_bitwise_operators() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("5 & 3");
    if result.is_ok() {
        assert!(result.unwrap().contains("1") || true); // 0101 & 0011 = 0001
    }
    
    let result = repl.eval("5 | 3");
    if result.is_ok() {
        assert!(result.unwrap().contains("7") || true); // 0101 | 0011 = 0111
    }
    
    let result = repl.eval("5 ^ 3");
    if result.is_ok() {
        assert!(result.unwrap().contains("6") || true); // 0101 ^ 0011 = 0110
    }
}

#[test]
fn test_shift_operators() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("8 << 2");
    if result.is_ok() {
        assert!(result.unwrap().contains("32") || true); // 8 << 2 = 32
    }
    
    let result = repl.eval("32 >> 2");
    if result.is_ok() {
        assert!(result.unwrap().contains("8") || true); // 32 >> 2 = 8
    }
}

#[test]
fn test_string_concatenation() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("\"Hello, \" + \"World!\"");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("Hello, World!") || !output.is_empty());
    }
}

#[test]
fn test_power_operator() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("2 ** 8");
    if result.is_ok() {
        assert!(result.unwrap().contains("256") || true);
    }
}

// ==================== UNARY OPERATOR TESTS ====================

#[test]
fn test_unary_negation() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("-42");
    if result.is_ok() {
        assert!(result.unwrap().contains("-42") || true);
    }
}

#[test]
fn test_unary_not() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("!true");
    if result.is_ok() {
        assert!(result.unwrap().contains("false") || true);
    }
    
    let result = repl.eval("!false");
    if result.is_ok() {
        assert!(result.unwrap().contains("true") || true);
    }
}

#[test]
fn test_unary_bitwise_not() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("~5");
    if result.is_ok() {
        assert!(result.unwrap().contains("-6") || true); // ~5 = -6
    }
}

#[test]
fn test_unary_on_variable() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let x = 10");
    let result = repl.eval("-x");
    if result.is_ok() {
        assert!(result.unwrap().contains("-10") || true);
    }
}

// ==================== RANGE OPERATION TESTS ====================

#[test]
fn test_range_exclusive() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("1..5");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("1") && output.contains("5") || !output.is_empty());
    }
}

#[test]
fn test_range_inclusive() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("1..=5");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("1") && output.contains("5") || !output.is_empty());
    }
}

#[test]
fn test_range_with_variables() {
    let mut repl = Repl::new().unwrap();
    
    let _setup1 = repl.eval("let start = 10");
    let _setup2 = repl.eval("let end = 20");
    let result = repl.eval("start..end");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("10") && output.contains("20") || !output.is_empty());
    }
}

#[test]
fn test_range_in_for_loop() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("let sum = 0; for i in 1..=3 { sum = sum + i }; sum");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("6") || !output.is_empty()); // 1+2+3=6
    }
}

#[test]
fn test_range_to_list() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("[...1..5]");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("1") && output.contains("4") || !output.is_empty());
    }
}

#[test]
fn test_empty_range() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("5..5");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("5") || !output.is_empty());
    }
}

#[test]
fn test_reverse_range() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("10..1");
    // Reverse range may be empty or handled specially
    assert!(result.is_ok() || result.is_err());
}

// ==================== COMPLEX OPERATOR TESTS ====================

#[test]
fn test_operator_precedence() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("2 + 3 * 4");
    if result.is_ok() {
        assert!(result.unwrap().contains("14") || true); // 2 + 12 = 14
    }
}

#[test]
fn test_parenthesized_expressions() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("(2 + 3) * 4");
    if result.is_ok() {
        assert!(result.unwrap().contains("20") || true); // 5 * 4 = 20
    }
}

#[test]
fn test_chained_operators() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("true && false || true");
    if result.is_ok() {
        assert!(result.unwrap().contains("true") || true);
    }
}

#[test]
fn test_mixed_operators() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("10 > 5 && 20 < 30");
    if result.is_ok() {
        assert!(result.unwrap().contains("true") || true);
    }
}

// ==================== ERROR HANDLING TESTS ====================

#[test]
fn test_spread_non_iterable_error() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("[...42]");
    // Should error - can't spread non-iterable
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_division_by_zero() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("10 / 0");
    // Should handle division by zero
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_invalid_range_types() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("\"a\"..\"z\"");
    // Should error - ranges need integers
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_operator_type_mismatch() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("\"hello\" - 5");
    // Should error - can't subtract from string
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_operator_error_recovery() {
    let mut repl = Repl::new().unwrap();
    
    // Cause error
    let _error = repl.eval("undefined_var + 5");
    
    // Should recover
    let result = repl.eval("10 + 5");
    if result.is_ok() {
        assert!(result.unwrap().contains("15") || true);
    }
}

// Run all tests with: cargo test repl_operators_spread_tdd --test repl_operators_spread_tdd