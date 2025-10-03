// Property-based tests for interpreter evaluation semantics
// PROPTEST-004 Part 2: Evaluation semantics properties (15 tests)
//
// Properties tested:
// 1. Variable binding preserves value
// 2. Variable shadowing works correctly
// 3. If expression returns correct branch (then)
// 4. If expression returns correct branch (else)
// 5. For loops iterate correct number of times
// 6. While loops terminate correctly
// 7. Function calls return expected values
// 8. Array indexing returns correct elements
// 9. Array length is preserved
// 10. String indexing works correctly
// 11. Range expressions generate correct sequences
// 12. Boolean short-circuit evaluation (AND)
// 13. Boolean short-circuit evaluation (OR)
// 14. Arithmetic operator precedence respected
// 15. Comparison chaining works correctly

use proptest::prelude::*;
use ruchy::runtime::repl::Repl;
use std::path::PathBuf;

// ============================================================================
// Property 1: Variable binding preserves value
// ============================================================================

proptest! {
    #[test]
    fn prop_variable_binding_preserves_integer(n in -1000i64..1000) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        repl.eval(&format!("let x = {n}")).unwrap();
        let result = repl.eval("x").unwrap();

        prop_assert!(result.contains(&n.to_string()),
            "Variable binding should preserve integer value: {}", n);
    }

    #[test]
    fn prop_variable_binding_preserves_string(s in "[a-zA-Z]{1,20}") {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        repl.eval(&format!("let x = \"{s}\"")).unwrap();
        let result = repl.eval("x").unwrap();

        prop_assert!(result.contains(&s),
            "Variable binding should preserve string value");
    }
}

// ============================================================================
// Property 2: Variable shadowing works correctly
// ============================================================================

proptest! {
    #[test]
    fn prop_variable_shadowing(a in 1i64..100, b in 101i64..200) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        repl.eval(&format!("let x = {a}")).unwrap();
        repl.eval(&format!("let x = {b}")).unwrap();
        let result = repl.eval("x").unwrap();

        prop_assert!(result.contains(&b.to_string()),
            "Variable shadowing should use new value: {} not {}", b, a);
    }
}

// ============================================================================
// Property 3-4: If expression returns correct branch
// ============================================================================

proptest! {
    #[test]
    fn prop_if_then_branch(then_val in 1i64..1000, else_val in 1001i64..2000) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        let code = format!("if true {{ {then_val} }} else {{ {else_val} }}");
        let result = repl.eval(&code).unwrap();

        prop_assert!(result.contains(&then_val.to_string()),
            "If true should execute then branch: {}", then_val);
    }

    #[test]
    fn prop_if_else_branch(then_val in 1i64..1000, else_val in 1001i64..2000) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        let code = format!("if false {{ {then_val} }} else {{ {else_val} }}");
        let result = repl.eval(&code).unwrap();

        prop_assert!(result.contains(&else_val.to_string()),
            "If false should execute else branch: {}", else_val);
    }
}

// ============================================================================
// Property 5: For loops iterate correct number of times
// ============================================================================

proptest! {
    #[test]
    fn prop_for_loop_iteration_count(start in 0i64..10, count in 1i64..10) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        let end = start + count;
        let code = format!(
            "let mut sum = 0; for i in {start}..{end} {{ sum = sum + 1 }}; sum"
        );
        let result = repl.eval(&code).unwrap();

        prop_assert!(result.contains(&count.to_string()),
            "For loop should iterate {} times", count);
    }
}

// ============================================================================
// Property 6: While loops terminate correctly
// ============================================================================

proptest! {
    #[test]
    fn prop_while_loop_termination(n in 1i64..10) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        let code = format!(
            "let mut i = 0; while i < {n} {{ i = i + 1 }}; i"
        );
        let result = repl.eval(&code).unwrap();

        prop_assert!(result.contains(&n.to_string()),
            "While loop should terminate at: {}", n);
    }
}

// ============================================================================
// Property 7: Function calls return expected values
// ============================================================================

proptest! {
    #[test]
    fn prop_function_call_returns_value(return_val in 1i64..1000) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        repl.eval(&format!("fn test() {{ {return_val} }}")).unwrap();
        let result = repl.eval("test()").unwrap();

        prop_assert!(result.contains(&return_val.to_string()),
            "Function should return: {}", return_val);
    }

    #[test]
    fn prop_function_with_param(param_val in 1i64..1000) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        repl.eval("fn double(x) { x * 2 }").unwrap();
        let result = repl.eval(&format!("double({param_val})")).unwrap();

        let expected = param_val * 2;
        prop_assert!(result.contains(&expected.to_string()),
            "Function should double input: {} * 2 = {}", param_val, expected);
    }
}

// ============================================================================
// Property 8-9: Array operations
// ============================================================================

proptest! {
    #[test]
    fn prop_array_indexing(
        elem1 in 1i64..100,
        elem2 in 101i64..200,
        elem3 in 201i64..300
    ) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        repl.eval(&format!("let arr = [{elem1}, {elem2}, {elem3}]")).unwrap();

        // Test each index
        let result0 = repl.eval("arr[0]").unwrap();
        let result1 = repl.eval("arr[1]").unwrap();
        let result2 = repl.eval("arr[2]").unwrap();

        prop_assert!(result0.contains(&elem1.to_string()), "arr[0] should be {}", elem1);
        prop_assert!(result1.contains(&elem2.to_string()), "arr[1] should be {}", elem2);
        prop_assert!(result2.contains(&elem3.to_string()), "arr[2] should be {}", elem3);
    }

    #[test]
    fn prop_array_length_preserved(len in 1usize..20) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        // Create array of given length with zeros
        let elements = vec!["0"; len];
        let array_code = format!("[{}]", elements.join(", "));

        repl.eval(&format!("let arr = {array_code}")).unwrap();
        let result = repl.eval("arr.len()").unwrap();

        prop_assert!(result.contains(&len.to_string()),
            "Array length should be {}", len);
    }
}

// ============================================================================
// Property 10: String indexing
// ============================================================================

proptest! {
    #[test]
    fn prop_string_indexing(s in "[a-z]{5,10}") {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        repl.eval(&format!("let s = \"{s}\"")).unwrap();

        // Test first character
        let result = repl.eval("s[0]").unwrap();
        let first_char = s.chars().next().unwrap();

        prop_assert!(result.contains(first_char),
            "String[0] should be {}", first_char);
    }
}

// ============================================================================
// Property 11: Range expressions generate correct sequences
// ============================================================================

proptest! {
    #[test]
    fn prop_range_sequence_generates(start in 0i64..10, count in 1i64..10) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        let end = start + count;
        // Use range in for loop to verify it generates correct iterations
        let code = format!(
            "let mut sum = 0; for i in {start}..{end} {{ sum = sum + 1 }}; sum"
        );
        let result = repl.eval(&code).unwrap();

        prop_assert!(result.contains(&count.to_string()),
            "Range {}..{} should iterate {} times", start, end, count);
    }
}

// ============================================================================
// Property 12-13: Boolean short-circuit evaluation
// ============================================================================

#[test]
fn prop_and_short_circuit_false() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // false && (side effect) should not execute side effect
    repl.eval("let mut x = 0").unwrap();
    repl.eval("false && { x = 1; true }").unwrap();
    let result = repl.eval("x").unwrap();

    assert!(
        result.contains('0'),
        "AND should short-circuit on false, x should remain 0"
    );
}

#[test]
fn prop_or_short_circuit_true() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // true || (side effect) should not execute side effect
    repl.eval("let mut x = 0").unwrap();
    repl.eval("true || { x = 1; false }").unwrap();
    let result = repl.eval("x").unwrap();

    assert!(
        result.contains('0'),
        "OR should short-circuit on true, x should remain 0"
    );
}

// ============================================================================
// Property 14: Arithmetic operator precedence respected
// ============================================================================

proptest! {
    #[test]
    fn prop_operator_precedence_mul_before_add(
        a in 1i64..10,
        b in 1i64..10,
        c in 1i64..10
    ) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        // a + b * c should be a + (b * c)
        let code = format!("{a} + {b} * {c}");
        let result = repl.eval(&code).unwrap();

        let expected = a + (b * c);
        prop_assert!(result.contains(&expected.to_string()),
            "Operator precedence: {} + {} * {} = {} (not {})",
            a, b, c, expected, (a + b) * c);
    }
}

// ============================================================================
// Property 15: Comparison chaining
// ============================================================================

proptest! {
    #[test]
    fn prop_comparison_chain_transitive(
        a in 1i64..10,
        b in 11i64..20,
        c in 21i64..30
    ) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        // Since a < b < c by construction, a < c should be true
        let code = format!("{a} < {b} && {b} < {c}");
        let result = repl.eval(&code).unwrap();

        prop_assert!(result.contains("true"),
            "Comparison transitivity: {} < {} < {} should be true", a, b, c);
    }
}
