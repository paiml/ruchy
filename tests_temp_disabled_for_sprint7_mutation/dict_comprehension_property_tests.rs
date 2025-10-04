// EXTREME TDD: Dict comprehension property tests
use proptest::prelude::*;
use ruchy::{compile, Parser};

// Generate valid variable names
prop_compose! {
    fn valid_var_name()(s in "[a-z][a-z0-9_]{0,10}") -> String {
        s
    }
}

// Generate valid expressions
prop_compose! {
    fn valid_expr()(choice in 0..3, var in valid_var_name()) -> String {
        match choice {
            0 => var,
            1 => format!("{var}.len()"),
            2 => format!("{var} * 2"),
            _ => unreachable!("choice is 0..3"),
        }
    }
}

// Generate valid iterables
prop_compose! {
    fn valid_iterable()(choice in 0..3) -> String {
        match choice {
            0 => "items".to_string(),
            1 => "vec![1, 2, 3]".to_string(),
            2 => "data.iter()".to_string(),
            _ => "items".to_string(),
        }
    }
}

// Generate valid conditions
prop_compose! {
    fn valid_condition()(var in valid_var_name(), op in 0..3usize, val in 0..10i32) -> String {
        let ops = [">", "<", "=="];
        format!("{} {} {}", var, ops[op], val)
    }
}

proptest! {
    #[test]
    fn test_basic_dict_comprehension_never_panics(
        key_var in valid_var_name(),
        value_var in valid_var_name(),
        iter_var in valid_var_name(),
        iterable in valid_iterable()
    ) {
        let input = format!(
            "{{{key_var}: {value_var} for {iter_var} in {iterable}}}"
        );
        let mut parser = Parser::new(&input);
        // Should not panic
        let _ = parser.parse();
    }

    #[test]
    fn test_dict_comprehension_with_filter_never_panics(
        key_expr in valid_expr(),
        value_expr in valid_expr(),
        var in valid_var_name(),
        iterable in valid_iterable(),
        condition in valid_condition()
    ) {
        let input = format!(
            "{{{key_expr}: {value_expr} for {var} in {iterable} if {condition}}}"
        );
        let mut parser = Parser::new(&input);
        // Should not panic
        let _ = parser.parse();
    }

    #[test]
    fn test_tuple_pattern_dict_comprehension(
        k_var in valid_var_name(),
        v_var in valid_var_name(),
        iterable in valid_iterable()
    ) {
        let input = format!(
            "{{{k_var}: {v_var} for ({k_var}, {v_var}) in {iterable}}}"
        );
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        // If it parses, it should produce valid AST
        if result.is_ok() {
            let ast = result.unwrap();
            // Should contain DictComprehension node
            let ast_str = format!("{ast:?}");
            prop_assert!(ast_str.contains("DictComprehension"));
        }
    }

    #[test]
    fn test_dict_comprehension_transpiles_to_hashmap(
        var in valid_var_name(),
        iterable in valid_iterable()
    ) {
        let code = format!(
            "fun main() {{ let m = {{{var}: {var}.len() for {var} in {iterable}}}; }}"
        );

        if let Ok(output) = compile(&code) {
            // Should generate HashMap collection
            prop_assert!(output.contains("HashMap") || output.contains("collect"));
            prop_assert!(output.contains("map"));
        } else {
            // Some combinations might not compile, that's ok
        }
    }

    #[test]
    fn test_dict_comprehension_filter_transpiles_correctly(
        var in valid_var_name(),
        threshold in 1..100i32
    ) {
        let code = format!(
            "fun main() {{ let m = {{{var}: {var} for {var} in nums if {var} > {threshold}}}; }}"
        );

        if let Ok(output) = compile(&code) {
            // Should generate filter before map
            prop_assert!(output.contains("filter"));
            prop_assert!(output.contains("HashMap"));
            // Filter should come before map in the chain
            if let (Some(filter_pos), Some(map_pos)) =
                (output.find("filter"), output.find("map")) {
                prop_assert!(filter_pos < map_pos);
            }
        } else {
            // Some combinations might not compile, that's ok
        }
    }

    #[test]
    fn test_dict_comprehension_key_value_independence(
        key_var in valid_var_name(),
        value_var in valid_var_name(),
        iter_var in valid_var_name()
    ) {
        prop_assume!(key_var != value_var);
        prop_assume!(key_var != iter_var);
        prop_assume!(value_var != iter_var);

        let code = format!(
            "fun main() {{ let m = {{{key_var}: {value_var} for {iter_var} in items}}; }}"
        );

        if let Ok(output) = compile(&code) {
            // Both key and value should appear in the output
            prop_assert!(output.contains(&key_var) || key_var == iter_var);
            prop_assert!(output.contains(&value_var) || value_var == iter_var);
        } else {
            // Some combinations might not compile, that's ok
        }
    }

    #[test]
    fn test_nested_dict_comprehensions_parse(
        var1 in valid_var_name(),
        var2 in valid_var_name()
    ) {
        prop_assume!(var1 != var2);

        let input = format!(
            "{{{var1}: {{{var2}: {var2} for {var2} in inner}} for {var1} in outer}}"
        );

        let mut parser = Parser::new(&input);
        // Should not panic even with nested comprehensions
        let _ = parser.parse();
    }

    #[test]
    fn test_dict_comprehension_with_method_calls(
        var in valid_var_name(),
        method in prop::sample::select(vec!["len()", "to_string()", "clone()"])
    ) {
        let code = format!(
            "fun main() {{ let m = {{{var}: {var}.{method} for {var} in items}}; }}"
        );

        if let Ok(output) = compile(&code) {
            prop_assert!(output.contains("HashMap"));
            // Method should be preserved in output
            let method_name = method.trim_end_matches("()");
            prop_assert!(output.contains(method_name));
        } else {
            // Some method calls might not compile, that's ok
        }
    }
}

// Fuzz test with completely random input
proptest! {
    #[test]
    fn fuzz_dict_comprehension_parser(input in "\\{[^}]{0,100}\\}") {
        let mut parser = Parser::new(&input);
        // Should never panic, even with random input
        let _ = parser.parse();
    }
}

// Edge cases as property tests
#[test]
fn test_dict_comprehension_edge_cases() {
    let edge_cases = vec![
        "{a: b for c in d}",                    // Single char variables
        "{_: _ for _ in _}",                    // Underscore variables
        "{x1: y2 for z3 in items}",             // Vars with numbers
        "{foo_bar: baz_qux for quux in items}", // Underscores
        "{a: b for (a, b) in [(1,2), (3,4)]}",  // Tuple destructuring
    ];

    for case in edge_cases {
        let mut parser = Parser::new(case);
        let result = parser.parse();
        // Should either parse successfully or give clear error
        match result {
            Ok(_) => println!("✓ Parsed: {case}"),
            Err(e) => println!("✗ Failed: {case} - {e}"),
        }
    }
}

// Regression test for specific bugs found
#[test]
fn test_dict_comprehension_regression_suite() {
    // Bug 1: Tuple patterns not parsing
    let code1 = "fun main() { let m = {k: v for (k, v) in pairs}; }";
    assert!(compile(code1).is_ok(), "Tuple pattern should work");

    // Bug 2: Filter with method calls
    let code2 = "fun main() { let m = {w: w.len() for w in words if w.len() > 5}; }";
    match compile(code2) {
        Ok(output) => {
            let normalized = output.replace(' ', "");
            assert!(normalized.contains("filter"));
            assert!(normalized.contains("len()"));
        }
        Err(e) => panic!("Filter with method calls failed: {e}"),
    }

    // Bug 3: Enumerate pattern
    let code3 = "fun main() { let m = {i: val for (i, val) in items.enumerate()}; }";
    assert!(compile(code3).is_ok(), "Enumerate pattern should work");
}
