// FUZZ Testing - Chaos Engineering for Lexer/Parser
// Target: Find edge cases with random/malformed input
// Sprint 80: ALL NIGHT Coverage Marathon Phase 19

use ruchy::frontend::lexer::Lexer;
use ruchy::Parser;
use quickcheck::{quickcheck, TestResult};
use quickcheck_macros::quickcheck;

// Fuzz lexer with totally random bytes
#[quickcheck]
fn fuzz_lexer_random_bytes(bytes: Vec<u8>) -> TestResult {
    // Convert random bytes to string (may be invalid UTF-8)
    let input = String::from_utf8_lossy(&bytes);

    let mut lexer = Lexer::new(&input);
    let _ = lexer.tokenize(); // Should not panic

    TestResult::passed()
}

// Fuzz lexer with random ASCII
#[quickcheck]
fn fuzz_lexer_ascii(input: String) -> TestResult {
    let ascii_only: String = input.chars()
        .filter(|c| c.is_ascii())
        .collect();

    let mut lexer = Lexer::new(&ascii_only);
    let _ = lexer.tokenize(); // Should not panic

    TestResult::passed()
}

// Fuzz parser with random tokens
#[quickcheck]
fn fuzz_parser_random_input(input: String) -> TestResult {
    let mut parser = Parser::new(&input);
    let _ = parser.parse(); // Should not panic

    TestResult::passed()
}

// Fuzz with malformed strings
#[test]
fn fuzz_malformed_strings() {
    let malformed = vec![
        r#""unterminated"#,
        r#""escaped\"still unterminated"#,
        r#""null\0byte""#,
        r#""newline
        in string""#,
        "\"\\x00\\x01\\x02\"",
        "\"\\u{D800}\"", // Invalid unicode
        "\"\\u{110000}\"", // Out of range unicode
    ];

    for input in malformed {
        let mut lexer = Lexer::new(input);
        let _ = lexer.tokenize();

        let mut parser = Parser::new(input);
        let _ = parser.parse();
    }
}

// Fuzz with extreme nesting
#[test]
fn fuzz_extreme_nesting() {
    // Deep parentheses
    let deep_parens = "(".repeat(1000) + "1" + &")".repeat(1000);
    let mut parser = Parser::new(&deep_parens);
    let _ = parser.parse();

    // Deep brackets
    let deep_brackets = "[".repeat(100) + "1" + &"]".repeat(100);
    let mut parser = Parser::new(&deep_brackets);
    let _ = parser.parse();

    // Deep braces
    let deep_braces = "{".repeat(100) + "}" .repeat(100);
    let mut parser = Parser::new(&deep_braces);
    let _ = parser.parse();
}

// Fuzz with mixed valid/invalid
#[test]
fn fuzz_mixed_valid_invalid() {
    let mixed = vec![
        "valid + @invalid",
        "1 + 2 + $$$ + 3",
        "fn() { @@@ }",
        "if true { ### }",
        "[1, 2, ???, 4]",
    ];

    for input in mixed {
        let mut parser = Parser::new(input);
        let _ = parser.parse();
    }
}

// Fuzz with unicode chaos
#[test]
fn fuzz_unicode_chaos() {
    let unicode_chaos = vec![
        "let ØÏ = 42",
        "fn ýp() { ÔÞ }",
        "=€ + <‰",
        "let emoji_var_=
 = true",
        "\u{200B}invisible\u{200B}",
        "\u{FEFF}BOM at start",
        "RTL text: E1-('",
        "Combining: a\u{0301}\u{0302}\u{0303}",
    ];

    for input in unicode_chaos {
        let mut lexer = Lexer::new(input);
        let _ = lexer.tokenize();

        let mut parser = Parser::new(input);
        let _ = parser.parse();
    }
}

// Fuzz with special characters
#[test]
fn fuzz_special_characters() {
    let special = vec![
        "\0\0\0",
        "\r\n\r\n",
        "\t\t\t",
        "\x1B[31mANSI\x1B[0m",
        "\\\\\\\\",
        "\"\"\"\"",
        "''''",
        "````",
    ];

    for input in special {
        let mut lexer = Lexer::new(input);
        let _ = lexer.tokenize();
    }
}

// Fuzz with repeated patterns
#[quickcheck]
fn fuzz_repeated_patterns(pattern: char, count: u8) -> TestResult {
    if count > 100 {
        return TestResult::discard();
    }

    let input = pattern.to_string().repeat(count as usize);
    let mut parser = Parser::new(&input);
    let _ = parser.parse();

    TestResult::passed()
}

// Fuzz with random operators
#[test]
fn fuzz_random_operators() {
    let ops = vec![
        "++++++",
        "------",
        "******",
        "//////",
        "======",
        "&&&&&&",
        "||||||",
        "<<<<<<",
        ">>>>>>",
        "......",
        ":::::::",
    ];

    for op in ops {
        let mut lexer = Lexer::new(op);
        let _ = lexer.tokenize();

        let mut parser = Parser::new(op);
        let _ = parser.parse();
    }
}

// Fuzz with large inputs
#[test]
fn fuzz_large_inputs() {
    // Large identifier
    let large_ident = "a".repeat(10000);
    let mut lexer = Lexer::new(&large_ident);
    let _ = lexer.tokenize();

    // Large number
    let large_number = "9".repeat(1000);
    let mut lexer = Lexer::new(&large_number);
    let _ = lexer.tokenize();

    // Large string
    let large_string = format!(r#""{}""#, "x".repeat(10000));
    let mut lexer = Lexer::new(&large_string);
    let _ = lexer.tokenize();
}

// Fuzz with zero-width and control characters
#[test]
fn fuzz_zero_width_control() {
    let zero_width = vec![
        "\u{200B}", // Zero-width space
        "\u{200C}", // Zero-width non-joiner
        "\u{200D}", // Zero-width joiner
        "\u{FEFF}", // Zero-width no-break space
        "\u{0000}", // Null
        "\u{0001}", // Start of heading
        "\u{001F}", // Unit separator
    ];

    for zw in zero_width {
        let input = format!("let x{} = 42", zw);
        let mut lexer = Lexer::new(&input);
        let _ = lexer.tokenize();
    }
}

// Fuzz with mixed line endings
#[test]
fn fuzz_line_endings() {
    let endings = vec![
        "line1\nline2\nline3",
        "line1\rline2\rline3",
        "line1\r\nline2\r\nline3",
        "line1\n\rline2\n\rline3", // Mixed wrong
    ];

    for input in endings {
        let mut lexer = Lexer::new(input);
        let _ = lexer.tokenize();

        let mut parser = Parser::new(input);
        let _ = parser.parse();
    }
}

// Fuzz with comment edge cases
#[test]
fn fuzz_comment_edge_cases() {
    let comments = vec![
        "// comment without newline at EOF",
        "/* unterminated comment",
        "/* /* nested */ comment */",
        "/** /* deeply /* nested */ */ */",
        "//! doc comment",
        "/// triple slash",
        "/* multi\n * line\n * comment */",
    ];

    for input in comments {
        let mut lexer = Lexer::new(input);
        let _ = lexer.tokenize();
    }
}

// Fuzz with number edge cases
#[test]
fn fuzz_number_edge_cases() {
    let numbers = vec![
        "0",
        "-0",
        "+0",
        "00000",
        "1e308", // Near f64::MAX
        "1e-308", // Near f64::MIN_POSITIVE
        "0x0",
        "0xFFFFFFFFFFFFFFFF",
        "0b0",
        "0b11111111111111111111111111111111",
        "0o0",
        "0o777777",
        "123.456.789", // Multiple dots
        "1e",  // Incomplete scientific
        "0x",  // Incomplete hex
        "0b",  // Incomplete binary
    ];

    for num in numbers {
        let mut lexer = Lexer::new(num);
        let _ = lexer.tokenize();

        let mut parser = Parser::new(num);
        let _ = parser.parse();
    }
}

// Fuzz with random valid Ruchy code
#[quickcheck]
fn fuzz_random_valid_code(
    vars: Vec<String>,
    nums: Vec<i32>,
    ops: Vec<bool>
) -> TestResult {
    if vars.is_empty() || nums.is_empty() {
        return TestResult::discard();
    }

    let mut code = String::new();
    for (i, var) in vars.iter().take(5).enumerate() {
        if let Some(num) = nums.get(i) {
            code.push_str(&format!("let {} = {};\n",
                var.chars().filter(|c| c.is_alphanumeric()).collect::<String>(),
                num));
        }
    }

    let mut parser = Parser::new(&code);
    let _ = parser.parse();

    TestResult::passed()
}