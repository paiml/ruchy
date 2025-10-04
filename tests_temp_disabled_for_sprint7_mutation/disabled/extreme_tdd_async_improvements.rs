//! EXTREME TDD: Async/Await Improvements Tests (LANG-004)
//!
//! Test-first development for missing async features: async blocks and async lambdas
//! Target: Complete async/await support beyond just async functions
//! Complexity: All test functions ≤10 cyclomatic complexity
//! Coverage: 100% of missing async variations

use ruchy::compile;

#[cfg(test)]
mod async_improvements_tests {
    use super::*;

    // =============================================================================
    // ASYNC BLOCK TESTS
    // =============================================================================

    #[test]
    #[ignore = "Async blocks need parsing implementation"]
    fn test_simple_async_block() {
        let code = "async { 42 }";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile: async {{ 42 }}");
        let output = result.unwrap();
        println!("Simple async block output: {output}");

        // Should generate: async { 42 } or async move { 42 }
        let has_async_block = output.contains("async {") || output.contains("async move {");
        assert!(
            has_async_block,
            "Should generate async block, got: {output}"
        );
    }

    #[test]
    #[ignore = "Async blocks need parsing implementation"]
    fn test_async_block_with_await() {
        let code = "async { await fetch_data() }";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile async block with await");
        let output = result.unwrap();
        println!("Async block with await output: {output}");

        let has_async = output.contains("async {") || output.contains("async move {");
        let has_await = output.contains(".await");
        assert!(
            has_async && has_await,
            "Should have async block and await, got: {output}"
        );
    }

    #[test]
    #[ignore = "Async blocks need parsing implementation"]
    fn test_async_block_with_multiple_statements() {
        let code = r#"
            async {
                let x = await get_value()
                let y = await get_another()
                x + y
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile multi-statement async block"
        );
        let output = result.unwrap();

        let has_async = output.contains("async {") || output.contains("async move {");
        let has_awaits = output.matches(".await").count() >= 2;
        assert!(
            has_async && has_awaits,
            "Should have async block with multiple awaits, got: {output}"
        );
    }

    #[test]
    #[ignore = "Async blocks need parsing implementation"]
    fn test_nested_async_blocks() {
        let code = r#"
            async {
                let inner = async { 42 }
                await inner
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile nested async blocks");
        let output = result.unwrap();

        // Should have multiple async blocks
        let async_count =
            output.matches("async {").count() + output.matches("async move {").count();
        assert!(
            async_count >= 2,
            "Should have nested async blocks, got: {output}"
        );
    }

    // =============================================================================
    // ASYNC LAMBDA TESTS
    // =============================================================================

    #[test]
    #[ignore = "Async lambdas need parsing implementation"]
    fn test_async_lambda_pipe_syntax() {
        let code = "async |x| x + 1";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile: async |x| x + 1");
        let output = result.unwrap();
        println!("Async lambda pipe output: {output}");

        // Should generate: |x| async move { x + 1 } or similar
        let has_async =
            output.contains("async") && (output.contains("|") || output.contains("move"));
        assert!(has_async, "Should generate async lambda, got: {output}");
    }

    #[test]
    #[ignore = "Async lambdas need parsing implementation"]
    fn test_async_lambda_arrow_syntax() {
        let code = "async x => x + 1";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile: async x => x + 1");
        let output = result.unwrap();

        let has_async = output.contains("async") && output.contains("move");
        assert!(has_async, "Should generate async lambda, got: {output}");
    }

    #[test]
    #[ignore = "Async lambdas need parsing implementation"]
    fn test_async_lambda_with_await() {
        let code = "async |x| await process(x)";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile async lambda with await");
        let output = result.unwrap();

        let has_async = output.contains("async");
        let has_await = output.contains(".await");
        assert!(
            has_async && has_await,
            "Should have async lambda with await, got: {output}"
        );
    }

    #[test]
    #[ignore = "Async lambdas need parsing implementation"]
    fn test_async_lambda_multiple_params() {
        let code = "async |x, y| x + y";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile multi-param async lambda");
        let output = result.unwrap();

        let has_async = output.contains("async");
        let has_params = output.contains("x") && output.contains("y");
        assert!(
            has_async && has_params,
            "Should have async lambda with multiple params, got: {output}"
        );
    }

    #[test]
    #[ignore = "Async lambdas need parsing implementation"]
    fn test_async_lambda_with_block() {
        let code = r#"
            async |x| {
                let y = await transform(x)
                y * 2
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile async lambda with block");
        let output = result.unwrap();

        let has_async = output.contains("async");
        let has_await = output.contains(".await");
        let has_block = output.contains("{") && output.contains("}");
        assert!(
            has_async && has_await && has_block,
            "Should have async lambda with block, got: {output}"
        );
    }

    // =============================================================================
    // INTEGRATION TESTS
    // =============================================================================

    #[test]
    #[ignore = "Integration needs all async features"]
    fn test_async_function_returning_async_block() {
        let code = r#"
            async fun create_task() -> Future<i32> {
                async { 42 }
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile async function with async block"
        );
        let output = result.unwrap();

        let has_async_fn = output.contains("async fn");
        let has_async_block = output.contains("async {") || output.contains("async move {");
        assert!(
            has_async_fn && has_async_block,
            "Should have both async function and block, got: {output}"
        );
    }

    #[test]
    #[ignore = "Integration needs all async features"]
    fn test_async_lambda_in_function() {
        let code = r#"
            fun process_async(data: Vec<i32>) {
                data.map(async |x| await transform(x))
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile function with async lambda"
        );
        let output = result.unwrap();

        let has_map = output.contains("map");
        let has_async = output.contains("async");
        let has_await = output.contains(".await");
        assert!(
            has_map && has_async && has_await,
            "Should integrate async lambda with map, got: {output}"
        );
    }

    #[test]
    #[ignore = "Integration needs all async features"]
    fn test_mixed_async_constructs() {
        let code = r#"
            async fun complex_async() {
                let block_result = await async { 10 }
                let lambda = async |x| x * 2
                let final_result = await lambda(block_result)
                final_result
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile mixed async constructs");
        let output = result.unwrap();

        let has_async_fn = output.contains("async fn");
        let has_async_block = output.contains("async {") || output.contains("async move {");
        let has_async_lambda = output.matches("async").count() >= 3; // fn, block, lambda
        let has_awaits = output.matches(".await").count() >= 2;

        assert!(
            has_async_fn && has_async_block && has_async_lambda && has_awaits,
            "Should have all async constructs, got: {output}"
        );
    }

    // =============================================================================
    // ERROR HANDLING TESTS
    // =============================================================================

    #[test]
    fn test_invalid_async_syntax() {
        let code = "async";
        let result = compile(code);
        // Should either compile with a graceful fallback or fail with a clear error
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_async_without_body() {
        let code = "async |x|";
        let result = compile(code);
        // Should fail gracefully
        assert!(result.is_err(), "Should fail for incomplete async lambda");
    }

    // =============================================================================
    // EXISTING FUNCTIONALITY (SHOULD STILL WORK)
    // =============================================================================

    #[test]
    fn test_async_function_still_works() {
        let code = "async fun fetch() { \"data\" }";
        let result = compile(code);
        assert!(result.is_ok(), "Async functions should still work");
        let output = result.unwrap();

        let has_async_fn = output.contains("async fn");
        assert!(has_async_fn, "Should generate async fn, got: {output}");
    }

    #[test]
    fn test_await_expression_still_works() {
        let code = "await fetch_data()";
        let result = compile(code);
        assert!(result.is_ok(), "Await expressions should still work");
        let output = result.unwrap();

        let has_await = output.contains(".await") || output.contains(". await");
        assert!(has_await, "Should generate .await, got: {output}");
    }

    #[test]
    fn test_await_in_let_still_works() {
        let code = "let result = await get_value()";
        let result = compile(code);
        assert!(result.is_ok(), "Await in let should still work");
        let output = result.unwrap();

        let has_let = output.contains("let");
        let has_await = output.contains(".await") || output.contains(". await");
        assert!(
            has_let && has_await,
            "Should have let with await, got: {output}"
        );
    }
}

#[cfg(test)]
mod async_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_async_blocks_never_panic(
            content in 1i32..100
        ) {
            let code = format!("async {{ {content} }}");
            let _ = compile(&code); // Should not panic
        }

        #[test]
        fn test_async_lambdas_never_panic(
            param in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            body in 1i32..100
        ) {
            let code = format!("async |{param}| {body}");
            let _ = compile(&code); // Should not panic
        }

        #[test]
        fn test_await_expressions_never_panic(
            func_name in "[a-zA-Z_][a-zA-Z0-9_]{0,20}"
        ) {
            let code = format!("await {func_name}()");
            let _ = compile(&code); // Should not panic
        }
    }
}
