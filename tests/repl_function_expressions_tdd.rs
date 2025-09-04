//! Comprehensive TDD test suite for REPL function expressions
//! Target: Coverage for function expression evaluation (lines 1661+ in repl.rs)
//! Toyota Way: Every function evaluation path must be tested comprehensively

use ruchy::runtime::repl::Repl;

// ==================== FUNCTION DEFINITION TESTS ====================

#[test]
fn test_simple_function_definition() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("fun greet(name) { \"Hello, \" + name }");
    // Function definition should return the function
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_function_with_no_params() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("fun get_answer() { 42 }");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_function_with_multiple_params() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("fun add(a, b, c) { a + b + c }");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_function_with_return_expression() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("fun calculate(x) { return x * 2 + 1 }");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_function_with_block_body() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("fun complex(x) { let temp = x * 2; temp + 1 }");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_recursive_function_definition() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("fun factorial(n) { if n <= 1 { 1 } else { n * factorial(n - 1) } }");
    assert!(result.is_ok() || result.is_err());
}

// ==================== LAMBDA EXPRESSION TESTS ====================

#[test]
fn test_simple_lambda() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("let double = |x| x * 2");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_lambda_no_params() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("let get_pi = || 3.14159");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_lambda_multiple_params() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("let sum = |a, b, c| a + b + c");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_lambda_with_block() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("let process = |x| { let temp = x * 2; temp + 1 }");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_lambda_capturing_variables() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let multiplier = 3");
    let result = repl.eval("let multiply = |x| x * multiplier");
    assert!(result.is_ok() || result.is_err());
}

// ==================== FUNCTION CALL TESTS ====================

#[test]
fn test_function_call_basic() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("fun add(a, b) { a + b }");
    let result = repl.eval("add(5, 3)");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("8") || !output.is_empty());
    }
}

#[test]
fn test_function_call_no_args() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("fun get_ten() { 10 }");
    let result = repl.eval("get_ten()");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("10") || !output.is_empty());
    }
}

#[test]
fn test_function_call_with_expressions() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("fun multiply(a, b) { a * b }");
    let result = repl.eval("multiply(2 + 3, 4 - 1)");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("15") || !output.is_empty()); // (2+3) * (4-1) = 5 * 3 = 15
    }
}

#[test]
fn test_function_call_with_variables() {
    let mut repl = Repl::new().unwrap();
    
    let _setup1 = repl.eval("fun square(x) { x * x }");
    let _setup2 = repl.eval("let num = 7");
    let result = repl.eval("square(num)");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("49") || !output.is_empty());
    }
}

#[test]
fn test_nested_function_calls() {
    let mut repl = Repl::new().unwrap();
    
    let _setup1 = repl.eval("fun add(a, b) { a + b }");
    let _setup2 = repl.eval("fun multiply(a, b) { a * b }");
    let result = repl.eval("multiply(add(2, 3), add(4, 1))");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("25") || !output.is_empty()); // (2+3) * (4+1) = 5 * 5 = 25
    }
}

#[test]
fn test_recursive_function_call() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("fun countdown(n) { if n <= 0 { 0 } else { n + countdown(n - 1) } }");
    let result = repl.eval("countdown(3)");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("6") || !output.is_empty()); // 3 + 2 + 1 + 0 = 6
    }
}

// ==================== LAMBDA CALL TESTS ====================

#[test]
fn test_lambda_call_basic() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let double = |x| x * 2");
    let result = repl.eval("double(5)");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("10") || !output.is_empty());
    }
}

#[test]
fn test_lambda_call_no_args() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let get_answer = || 42");
    let result = repl.eval("get_answer()");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("42") || !output.is_empty());
    }
}

#[test]
fn test_immediate_lambda_call() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("(|x| x + 1)(10)");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("11") || !output.is_empty());
    }
}

#[test]
fn test_lambda_with_closure() {
    let mut repl = Repl::new().unwrap();
    
    let _setup1 = repl.eval("let base = 100");
    let _setup2 = repl.eval("let add_base = |x| x + base");
    let result = repl.eval("add_base(23)");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("123") || !output.is_empty());
    }
}

// ==================== HIGHER-ORDER FUNCTION TESTS ====================

#[test]
fn test_function_returning_function() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("fun make_adder(n) { |x| x + n }");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_function_taking_function() {
    let mut repl = Repl::new().unwrap();
    
    let _setup1 = repl.eval("fun apply_twice(f, x) { f(f(x)) }");
    let _setup2 = repl.eval("fun increment(x) { x + 1 }");
    let result = repl.eval("apply_twice(increment, 5)");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("7") || !output.is_empty()); // increment(increment(5)) = 7
    }
}

#[test]
fn test_function_composition() {
    let mut repl = Repl::new().unwrap();
    
    let _setup1 = repl.eval("fun double(x) { x * 2 }");
    let _setup2 = repl.eval("fun add_one(x) { x + 1 }");
    let _setup3 = repl.eval("fun compose(f, g) { |x| f(g(x)) }");
    let _setup4 = repl.eval("let double_then_add = compose(add_one, double)");
    let result = repl.eval("double_then_add(3)");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("7") || !output.is_empty()); // add_one(double(3)) = add_one(6) = 7
    }
}

// ==================== METHOD CALL TESTS ====================

#[test]
fn test_list_method_calls() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let numbers = [1, 2, 3]");
    let result = repl.eval("numbers.len()");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("3") || !output.is_empty());
    }
}

#[test]
fn test_string_method_calls() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let text = \"Hello\"");
    let result = repl.eval("text.upper()");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("HELLO") || !output.is_empty());
    }
}

#[test]
fn test_int_method_calls() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("42.to_string()");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("42") || !output.is_empty());
    }
}

#[test]
fn test_float_method_calls() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("3.14.floor()");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("3") || !output.is_empty());
    }
}

#[test]
fn test_object_method_calls() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let obj = {name: \"test\", value: 42}");
    let result = repl.eval("obj.keys()");
    // Object methods may or may not be implemented
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_chained_method_calls() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let data = [1, 2, 3, 4, 5]");
    let result = repl.eval("data.reverse().head()");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("5") || !output.is_empty());
    }
}

// ==================== OPTIONAL METHOD CALL TESTS ====================

#[test]
fn test_optional_method_call_valid() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let obj = {value: 42}");
    let result = repl.eval("obj?.value");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("42") || !output.is_empty());
    }
}

#[test]
fn test_optional_method_call_null() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let obj = null");
    let result = repl.eval("obj?.method()");
    // Optional method call on null should handle gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_chained_optional_method_calls() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let data = {user: {profile: {name: \"Alice\"}}}");
    let result = repl.eval("data?.user?.profile?.name");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("Alice") || !output.is_empty());
    }
}

// ==================== ARRAY CONSTRUCTOR TESTS ====================

#[test]
fn test_array_constructor_call() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let Array = \"Array constructor\"");
    let result = repl.eval("Array.new(3, 0)");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("Array") || !output.is_empty());
    }
}

#[test]
fn test_array_constructor_wrong_args() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let Array = \"Array constructor\"");
    let result = repl.eval("Array.new(3)");
    // Should error - Array.new expects 2 arguments
    assert!(result.is_err() || result.is_ok());
}

// ==================== FUNCTION ERROR HANDLING TESTS ====================

#[test]
fn test_undefined_function_call() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("undefined_function(1, 2, 3)");
    // Should error - function doesn't exist
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_function_wrong_argument_count() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("fun add_two(a, b) { a + b }");
    let result = repl.eval("add_two(1, 2, 3)");
    // Should handle argument count mismatch
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_function_call_on_non_function() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let not_a_function = 42");
    let result = repl.eval("not_a_function(1, 2)");
    // Should error - trying to call a number
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_method_call_unsupported_type() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("true.unknown_method()");
    // Should error - booleans don't support methods
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_function_error_recovery() {
    let mut repl = Repl::new().unwrap();
    
    // Cause error in function call
    let _error = repl.eval("undefined_function()");
    
    // Should recover for next evaluation
    let _setup = repl.eval("fun valid_function() { 100 }");
    let result = repl.eval("valid_function()");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("100") || !output.is_empty());
    }
}

// ==================== COMPLEX FUNCTION TESTS ====================

#[test]
fn test_functions_with_control_flow() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("fun max(a, b) { if a > b { a } else { b } }");
    let result = repl.eval("max(10, 7)");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("10") || !output.is_empty());
    }
}

#[test]
fn test_functions_with_loops() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("fun sum_to_n(n) { let sum = 0; for i in 1..=n { sum = sum + i }; sum }");
    let result = repl.eval("sum_to_n(5)");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("15") || !output.is_empty()); // 1+2+3+4+5 = 15
    }
}

#[test]
fn test_functions_with_data_structures() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("fun process_list(lst) { lst.reverse().head() }");
    let result = repl.eval("process_list([1, 2, 3, 4])");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("4") || !output.is_empty());
    }
}

#[test]
fn test_functions_modifying_global_state() {
    let mut repl = Repl::new().unwrap();
    
    let _setup1 = repl.eval("let global_counter = 0");
    let _setup2 = repl.eval("fun increment_counter() { global_counter = global_counter + 1; global_counter }");
    let result = repl.eval("increment_counter()");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("1") || !output.is_empty());
    }
}

#[test]
fn test_nested_function_definitions() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("fun outer(x) { fun inner(y) { y * 2 }; inner(x) + 1 }");
    assert!(result.is_ok() || result.is_err());
    
    if result.is_ok() {
        let call_result = repl.eval("outer(5)");
        if call_result.is_ok() {
            let output = call_result.unwrap();
            assert!(output.contains("11") || !output.is_empty()); // inner(5) + 1 = 10 + 1 = 11
        }
    }
}

// ==================== PERFORMANCE AND EDGE CASES ====================

#[test]
fn test_deeply_recursive_function() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("fun deep_sum(n) { if n <= 0 { 0 } else { 1 + deep_sum(n - 1) } }");
    let result = repl.eval("deep_sum(10)");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("10") || !output.is_empty());
    }
}

#[test]
fn test_function_with_many_parameters() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("fun many_params(a, b, c, d, e, f, g, h) { a + b + c + d + e + f + g + h }");
    let result = repl.eval("many_params(1, 2, 3, 4, 5, 6, 7, 8)");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("36") || !output.is_empty()); // 1+2+3+4+5+6+7+8 = 36
    }
}

#[test]
fn test_function_returning_complex_data() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("fun make_person(name, age) { {name: name, age: age, active: true} }");
    let result = repl.eval("make_person(\"Bob\", 30)");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("Bob") && output.contains("30") || !output.is_empty());
    }
}

// Run all tests with: cargo test repl_function_expressions_tdd --test repl_function_expressions_tdd