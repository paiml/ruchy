/// ERROR-004: TDD tests for parser issues with let statements in function bodies
///
/// Tests that functions containing multiple let statements and early returns
/// parse correctly without "Expected RightBrace" errors.
use ruchy::frontend::Parser;
use ruchy::runtime::Interpreter;

fn parse(code: &str) -> Result<String, String> {
    let mut parser = Parser::new(code);
    parser
        .parse()
        .map(|_| "OK".to_string())
        .map_err(|e| e.to_string())
}

fn eval(code: &str) -> Result<String, String> {
    let mut interp = Interpreter::new();
    let mut parser = Parser::new(code);
    let expr = parser.parse().map_err(|e| e.to_string())?;
    interp
        .eval_expr(&expr)
        .map(|v| v.to_string())
        .map_err(|e| e.to_string())
}

#[test]
fn test_function_with_single_let_mut() {
    let code = r#"
        fun test() -> i32 {
            let mut x = 5;
            x
        }
        test()
    "#;

    let result = parse(code);
    assert!(result.is_ok(), "Single let mut should parse: {:?}", result);
}

#[test]
fn test_function_with_multiple_let_mut() {
    let code = r#"
        fun test() -> i32 {
            let mut result = 0;
            let mut i = 0;
            result
        }
        test()
    "#;

    let result = parse(code);
    assert!(
        result.is_ok(),
        "Multiple let mut should parse: {:?}",
        result
    );
}

#[test]
fn test_function_with_let_and_if() {
    let code = r#"
        fun test(input: &str) -> i32 {
            let mut result = 0;

            if input.len() == 0 {
                return 0;
            }

            result
        }
        test("")
    "#;

    let result = parse(code);
    assert!(result.is_ok(), "Let with if should parse: {:?}", result);
}

#[test]
fn test_function_with_multiple_lets_and_if() {
    let code = r#"
        fun test(input: &str) -> i32 {
            let mut result = 0;
            let mut i = 0;
            let chars = input.as_bytes();

            if input.len() == 0 {
                println("Error: Empty input");
                return 0;
            }

            result
        }
        test("")
    "#;

    let result = parse(code);
    assert!(
        result.is_ok(),
        "Multiple lets with if should parse: {:?}",
        result
    );
}

#[test]
fn test_simplified_ex08() {
    // Simplified version of Chapter 17 Example 8
    let code = r#"
        fun parse_positive_integer(input: &str) -> i32 {
            let mut result = 0;
            let mut i = 0;
            let chars = input.as_bytes();

            if input.len() == 0 {
                println("Error: Empty input, using 0");
                return 0;
            }

            if chars[0] == b'-' {
                println("Error: Negative numbers not allowed, using 0");
                return 0;
            }

            result
        }

        parse_positive_integer("123")
    "#;

    let result = parse(code);
    assert!(result.is_ok(), "Simplified ex08 should parse: {:?}", result);
}

#[test]
fn test_ex08_with_while_loop() {
    // Full version of the problematic function from Example 8
    let code = r#"
        fun parse_positive_integer(input: &str) -> i32 {
            let mut result = 0;
            let mut i = 0;
            let chars = input.as_bytes();

            if input.len() == 0 {
                println("Error: Empty input, using 0");
                return 0;
            }

            if chars[0] == b'-' {
                println("Error: Negative numbers not allowed, using 0");
                return 0;
            }

            while i < input.len() {
                let ch = chars[i];
                if ch >= b'0' && ch <= b'9' {
                    let digit = (ch - b'0') as i32;
                    result = result * 10 + digit;
                } else {
                    println("Error: Invalid character in number, stopping at {}", result);
                    break;
                }
                i = i + 1;
            }

            if result > 1000 {
                println("Warning: Value {} too large, capping at 1000", result);
                return 1000;
            }

            result
        }

        parse_positive_integer("123")
    "#;

    let result = parse(code);
    assert!(
        result.is_ok(),
        "Full ex08 parse_positive_integer should parse: {:?}",
        result
    );
}

#[test]
fn test_ex08_full_with_two_functions() {
    // Both functions from Example 8
    let code = r#"
        fun parse_positive_integer(input: &str) -> i32 {
            let mut result = 0;
            let mut i = 0;
            let chars = input.as_bytes();

            if input.len() == 0 {
                println("Error: Empty input, using 0");
                return 0;
            }

            if chars[0] == b'-' {
                println("Error: Negative numbers not allowed, using 0");
                return 0;
            }

            while i < input.len() {
                let ch = chars[i];
                if ch >= b'0' && ch <= b'9' {
                    let digit = (ch - b'0') as i32;
                    result = result * 10 + digit;
                } else {
                    println("Error: Invalid character in number, stopping at {}", result);
                    break;
                }
                i = i + 1;
            }

            if result > 1000 {
                println("Warning: Value {} too large, capping at 1000", result);
                return 1000;
            }

            result
        }

        fun calculate_score(correct: &str, total: &str) -> f64 {
            let correct_num = parse_positive_integer(correct);
            let total_num = parse_positive_integer(total);

            if total_num == 0 {
                println("Error: Cannot calculate score with zero total");
                return 0.0;
            }

            if correct_num > total_num {
                println("Error: Correct answers cannot exceed total");
                return 0.0;
            }

            (correct_num as f64) / (total_num as f64) * 100.0
        }

        calculate_score("8", "10")
    "#;

    let result = parse(code);
    assert!(
        result.is_ok(),
        "Full ex08 with both functions should parse: {:?}",
        result
    );
}
