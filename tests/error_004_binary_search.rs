/// ERROR-004: Binary search to find which part of ex08 causes parser error
use ruchy::frontend::Parser;

fn parse(code: &str) -> Result<String, String> {
    let mut parser = Parser::new(code);
    parser
        .parse()
        .map(|_| "OK".to_string())
        .map_err(|e| e.to_string())
}

#[test]
fn test_just_first_function() {
    // Just parse_positive_integer without calculate_score or main
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
        "Just first function should parse: {:?}",
        result
    );
}

#[test]
fn test_first_two_functions() {
    // parse_positive_integer + calculate_score (no main)
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
        "First two functions should parse: {:?}",
        result
    );
}
