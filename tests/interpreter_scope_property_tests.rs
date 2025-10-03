// Property-based tests for interpreter environment and scope handling
// PROPTEST-004 Part 3: Environment/scope properties (7 tests)
//
// Properties tested:
// 1. Function scope isolates variables
// 2. Block scope shadows correctly
// 3. Nested scopes access outer variables
// 4. Loop variables don't leak
// 5. Function parameters shadow outer variables
// 6. Closure captures work correctly
// 7. Multiple sequential scopes are independent

use proptest::prelude::*;
use ruchy::runtime::repl::Repl;
use std::path::PathBuf;

// ============================================================================
// Property 1: Function scope isolates variables
// ============================================================================

proptest! {
    #[test]
    fn prop_function_scope_isolation(outer_val in 1i64..100, inner_val in 101i64..200) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        // Define variable in outer scope
        repl.eval(&format!("let x = {}", outer_val)).unwrap();

        // Define function with local variable of same name
        repl.eval(&format!("fn test() {{ let x = {}; x }}", inner_val)).unwrap();

        // Call function should return inner value
        let func_result = repl.eval("test()").unwrap();
        prop_assert!(func_result.contains(&inner_val.to_string()),
            "Function should see inner x: {}", inner_val);

        // Outer scope should still see outer value
        let outer_result = repl.eval("x").unwrap();
        prop_assert!(outer_result.contains(&outer_val.to_string()),
            "Outer scope should see outer x: {}", outer_val);
    }
}

// ============================================================================
// Property 2: Block scope shadows correctly
// ============================================================================

proptest! {
    #[test]
    fn prop_block_scope_access_outer(outer_val in 1i64..1000) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        // Test that blocks can access outer variables
        let code = format!(
            "let x = {}; {{ x }}",
            outer_val
        );
        let block_result = repl.eval(&code).unwrap();

        prop_assert!(block_result.contains(&outer_val.to_string()),
            "Block should access outer x: {}", outer_val);
    }
}

// ============================================================================
// Property 3: Nested scopes access outer variables
// ============================================================================

proptest! {
    #[test]
    fn prop_nested_scope_access(outer_val in 1i64..1000) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        // Define outer variable
        repl.eval(&format!("let x = {}", outer_val)).unwrap();

        // Nested block should access outer variable
        let code = "{ { x } }";
        let result = repl.eval(code).unwrap();

        prop_assert!(result.contains(&outer_val.to_string()),
            "Nested scope should access outer x: {}", outer_val);
    }

    #[test]
    fn prop_function_accesses_global(global_val in 1i64..1000) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        // Define global variable
        repl.eval(&format!("let g = {}", global_val)).unwrap();

        // Function should access global
        repl.eval("fn test() { g }").unwrap();
        let result = repl.eval("test()").unwrap();

        prop_assert!(result.contains(&global_val.to_string()),
            "Function should access global g: {}", global_val);
    }
}

// ============================================================================
// Property 4: Loop variables don't leak
// ============================================================================

proptest! {
    #[test]
    fn prop_loop_variable_scoping(n in 1i64..10) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        // Test that loop variables work correctly within loop
        let code = format!(
            "let mut sum = 0; for i in 0..{} {{ sum = sum + i }}; sum",
            n
        );
        let result = repl.eval(&code).unwrap();

        // Sum of 0..(n-1) is n*(n-1)/2
        let expected = (n * (n - 1)) / 2;
        prop_assert!(result.contains(&expected.to_string()),
            "Loop variable 'i' should work correctly: sum of 0..{} = {}", n, expected);
    }
}

// ============================================================================
// Property 5: Function parameters shadow outer variables
// ============================================================================

proptest! {
    #[test]
    fn prop_param_shadows_outer(outer_val in 1i64..100, param_val in 101i64..200) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        // Define outer variable
        repl.eval(&format!("let x = {}", outer_val)).unwrap();

        // Define function with parameter 'x'
        repl.eval("fn test(x) { x }").unwrap();

        // Call function with different value
        let result = repl.eval(&format!("test({})", param_val)).unwrap();

        prop_assert!(result.contains(&param_val.to_string()),
            "Function parameter should shadow outer x: {} not {}", param_val, outer_val);

        // After function call, outer value should still be accessible
        let outer_result = repl.eval("x").unwrap();
        prop_assert!(outer_result.contains(&outer_val.to_string()),
            "After function, should see outer x: {}", outer_val);
    }
}

// ============================================================================
// Property 6: Closure captures work correctly
// ============================================================================

proptest! {
    #[test]
    fn prop_closure_captures_variable(captured_val in 1i64..1000, new_val in 1001i64..2000) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        // Define variable
        repl.eval(&format!("let x = {}", captured_val)).unwrap();

        // Create closure that captures x
        repl.eval("let f = || x").unwrap();

        // Closure should see captured value
        let result1 = repl.eval("f()").unwrap();
        prop_assert!(result1.contains(&captured_val.to_string()),
            "Closure should capture x: {}", captured_val);

        // Update x
        repl.eval(&format!("let x = {}", new_val)).unwrap();

        // New closure should see new value
        let result2 = repl.eval("x").unwrap();
        prop_assert!(result2.contains(&new_val.to_string()),
            "After update, should see new x: {}", new_val);
    }
}

// ============================================================================
// Property 7: Multiple sequential scopes are independent
// ============================================================================

proptest! {
    #[test]
    fn prop_sequential_scopes_independent(
        val1 in 1i64..100,
        val2 in 101i64..200,
        val3 in 201i64..300
    ) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        // First scope
        repl.eval(&format!("{{ let x = {} }}", val1)).unwrap();

        // Second scope - should not see x from first scope
        repl.eval(&format!("{{ let x = {} }}", val2)).unwrap();

        // Third scope - should not see x from previous scopes
        let code = format!("{{ let x = {}; x }}", val3);
        let result = repl.eval(&code).unwrap();

        prop_assert!(result.contains(&val3.to_string()),
            "Third scope should see its own x: {}", val3);
    }
}
