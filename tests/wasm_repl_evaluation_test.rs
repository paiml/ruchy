#![allow(clippy::ignore_without_reason)] // Test file with known limitations

// WASM REPL Evaluation Tests
// Bug: WASM REPL returns AST debug string instead of evaluating code
// Discovered: Interactive.paiml.com project blocked by non-functional REPL
//
// Five Whys Analysis:
// 1. Why does WASM REPL show "Function { ... }" instead of executing code?
//    - Because it returns format!("{ast:?}") instead of evaluating
// 2. Why doesn't it evaluate?
//    - Because eval() method has stub implementation (line 114 in wasm/repl.rs)
// 3. Why is it a stub?
//    - Original implementation focused on parsing validation, not execution
// 4. Why wasn't evaluation added?
//    - Likely incomplete feature implementation
// 5. Why does this block interactive.paiml.com?
//    - Users expect REPL to execute code, not show AST
//
// ROOT CAUSE: WasmRepl.eval() never calls Interpreter::eval() - just formats AST
// SOLUTION: Integrate Interpreter to actually execute parsed code

// Note: These tests are for native builds that simulate WASM behavior
// Actual WASM tests require wasm-bindgen-test framework

use serde_json::Value as JsonValue;

// Helper to parse REPL JSON output
fn parse_repl_output(json: &str) -> Result<JsonValue, serde_json::Error> {
    serde_json::from_str(json)
}

/// RED TEST: Simple arithmetic should evaluate, not return AST
#[test]
fn test_wasm_repl_evaluates_arithmetic() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use ruchy::wasm::repl::WasmRepl;

        let mut repl = WasmRepl::new().expect("Failed to create REPL");
        let output = repl.eval("1 + 2").expect("Eval failed");

        let json: JsonValue = parse_repl_output(&output).expect("Invalid JSON");

        assert_eq!(json["success"], true, "Evaluation should succeed");
        assert_eq!(
            json["display"].as_str().unwrap().trim(),
            "3",
            "Should display evaluated result, not AST"
        );
        assert!(
            json["error"].is_null(),
            "Should have no error"
        );
    }
}

/// RED TEST: Function definition and call should execute
#[test]
fn test_wasm_repl_function_definition() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use ruchy::wasm::repl::WasmRepl;

        let mut repl = WasmRepl::new().expect("Failed to create REPL");
        let output = repl
            .eval("fun greet(name) { \"Hello, \" + name }; greet(\"World\")")
            .expect("Eval failed");

        let json: JsonValue = parse_repl_output(&output).expect("Invalid JSON");

        assert_eq!(json["success"], true, "Evaluation should succeed");
        assert_eq!(
            json["display"].as_str().unwrap().trim(),
            "Hello, World",
            "Should execute function and return result"
        );
    }
}

/// RED TEST: Variable assignment should persist... wait, no state!
/// Note: Current WASM REPL creates NEW interpreter each eval - no state persistence
/// This test documents expected behavior for future enhancement
#[test]
fn test_wasm_repl_single_expression_evaluation() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use ruchy::wasm::repl::WasmRepl;

        let mut repl = WasmRepl::new().expect("Failed to create REPL");

        // Single expression should evaluate
        let output = repl.eval("let x = 10; x + 5").expect("Eval failed");
        let json: JsonValue = parse_repl_output(&output).expect("Invalid JSON");

        assert_eq!(json["success"], true);
        assert_eq!(json["display"].as_str().unwrap().trim(), "15");
    }
}

/// RED TEST: Error handling for invalid syntax
#[test]
fn test_wasm_repl_syntax_error() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use ruchy::wasm::repl::WasmRepl;

        let mut repl = WasmRepl::new().expect("Failed to create REPL");
        let output = repl.eval("let x = ").expect("Eval should return error JSON");

        let json: JsonValue = parse_repl_output(&output).expect("Invalid JSON");

        assert_eq!(json["success"], false, "Should fail on syntax error");
        assert!(
            json["error"].as_str().unwrap().contains("Parse error"),
            "Error should mention parse error"
        );
    }
}

/// RED TEST: Runtime error handling
#[test]
fn test_wasm_repl_runtime_error() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use ruchy::wasm::repl::WasmRepl;

        let mut repl = WasmRepl::new().expect("Failed to create REPL");
        let output = repl
            .eval("undefined_variable")
            .expect("Eval should return error JSON");

        let json: JsonValue = parse_repl_output(&output).expect("Invalid JSON");

        assert_eq!(json["success"], false, "Should fail on undefined variable");
        assert!(
            json["error"].as_str().is_some(),
            "Should have error message"
        );
    }
}

/// RED TEST: String operations
#[test]
fn test_wasm_repl_string_operations() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use ruchy::wasm::repl::WasmRepl;

        let mut repl = WasmRepl::new().expect("Failed to create REPL");
        let output = repl
            .eval("\"Hello\" + \" \" + \"World\"")
            .expect("Eval failed");

        let json: JsonValue = parse_repl_output(&output).expect("Invalid JSON");

        assert_eq!(json["success"], true);
        assert_eq!(json["display"].as_str().unwrap().trim(), "Hello World");
    }
}

/// RED TEST: Array operations
#[test]
fn test_wasm_repl_array_operations() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use ruchy::wasm::repl::WasmRepl;

        let mut repl = WasmRepl::new().expect("Failed to create REPL");
        let output = repl.eval("[1, 2, 3].length()").expect("Eval failed");

        let json: JsonValue = parse_repl_output(&output).expect("Invalid JSON");

        assert_eq!(json["success"], true);
        assert_eq!(json["display"].as_str().unwrap().trim(), "3");
    }
}

/// RED TEST: Match expressions (Issue #40 related)
#[test]
fn test_wasm_repl_match_expression() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use ruchy::wasm::repl::WasmRepl;

        let mut repl = WasmRepl::new().expect("Failed to create REPL");
        let output = repl
            .eval("match 1 { 1 => \"one\" _ => \"other\" }")
            .expect("Eval failed");

        let json: JsonValue = parse_repl_output(&output).expect("Invalid JSON");

        assert_eq!(json["success"], true);
        assert_eq!(json["display"].as_str().unwrap().trim(), "one");
    }
}

/// RED TEST: Loop execution
#[test]
fn test_wasm_repl_loop_execution() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use ruchy::wasm::repl::WasmRepl;

        let mut repl = WasmRepl::new().expect("Failed to create REPL");
        let code = r"
let mut sum = 0;
let mut i = 0;
loop {
    if i >= 5 { break; }
    sum = sum + i;
    i = i + 1;
}
sum
";
        let output = repl.eval(code).expect("Eval failed");

        let json: JsonValue = parse_repl_output(&output).expect("Invalid JSON");

        assert_eq!(json["success"], true);
        assert_eq!(
            json["display"].as_str().unwrap().trim(),
            "10",
            "0+1+2+3+4 = 10"
        );
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;

    /// Property test: All valid integers should evaluate correctly
    #[test]
    #[ignore] // Run with: cargo test property_tests -- --ignored
    fn proptest_integer_evaluation() {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use ruchy::wasm::repl::WasmRepl;

            for n in -100..100 {
                let mut repl = WasmRepl::new().expect("Failed to create REPL");
                let output = repl.eval(&n.to_string()).expect("Eval failed");
                let json: JsonValue = parse_repl_output(&output).expect("Invalid JSON");

                assert_eq!(json["success"], true, "Integer {n} should evaluate");
                assert_eq!(
                    json["display"].as_str().unwrap().trim(),
                    n.to_string(),
                    "Integer {n} should display correctly"
                );
            }
        }
    }

    /// Property test: Arithmetic commutativity
    #[test]
    #[ignore]
    fn proptest_arithmetic_commutativity() {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use ruchy::wasm::repl::WasmRepl;

            for a in 0..20 {
                for b in 0..20 {
                    let mut repl1 = WasmRepl::new().expect("Failed to create REPL");
                    let mut repl2 = WasmRepl::new().expect("Failed to create REPL");

                    let expr1 = format!("{a} + {b}");
                    let expr2 = format!("{b} + {a}");

                    let out1 = repl1.eval(&expr1).expect("Eval failed");
                    let out2 = repl2.eval(&expr2).expect("Eval failed");

                    let json1: JsonValue = parse_repl_output(&out1).expect("Invalid JSON");
                    let json2: JsonValue = parse_repl_output(&out2).expect("Invalid JSON");

                    assert_eq!(
                        json1["display"], json2["display"],
                        "{a} + {b} should equal {b} + {a}"
                    );
                }
            }
        }
    }
}
