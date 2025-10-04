/// ERROR-004: Exact reproduction of Chapter 17 Example 8
/// Testing the exact code from the failing test
use ruchy::frontend::Parser;
use ruchy::runtime::Interpreter;

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
fn test_exact_ex08_code() {
    // This is the EXACT code from chapter_17_error_handling_tests.rs test_ex08_numeric_parsing
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

        fun main() {
            let score1 = calculate_score("8", "10");
            let score2 = calculate_score("abc", "10");
            let score3 = calculate_score("15", "10");
            let score4 = calculate_score("5", "0");

            println("Scores: {:.1}%, {:.1}%, {:.1}%, {:.1}%", score1, score2, score3, score4);
        }

        main()
    "#;

    let result = eval(code);
    assert!(result.is_ok(), "Example 8 should work: {:?}", result.err());
}
