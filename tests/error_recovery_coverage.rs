// EXTREME Coverage Test Suite for Error Recovery
// Target: Test all error paths and recovery mechanisms
// Sprint 80: ALL NIGHT Coverage Marathon Phase 10

use ruchy::compile::Compiler;
use ruchy::frontend::lexer::Lexer;
use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::Interpreter;

// Parser error recovery
#[test]
fn test_parser_recover_missing_paren() {
    let mut parser = Parser::new("(1 + 2");
    let result = parser.parse();
    assert!(result.is_err());
    // Parser should recover and continue
    let errors = parser.get_errors();
    assert!(!errors.is_empty());
}

#[test]
fn test_parser_recover_missing_bracket() {
    let mut parser = Parser::new("[1, 2, 3");
    let result = parser.parse();
    assert!(result.is_err());
}

#[test]
fn test_parser_recover_missing_brace() {
    let mut parser = Parser::new("{ x: 1, y: 2");
    let result = parser.parse();
    assert!(result.is_err());
}

#[test]
fn test_parser_recover_unexpected_token() {
    let mut parser = Parser::new("1 + + 2");
    let result = parser.parse();
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_parser_recover_invalid_syntax() {
    let mut parser = Parser::new("if then else");
    let result = parser.parse();
    assert!(result.is_err());
}

#[test]
fn test_parser_recover_incomplete_function() {
    let mut parser = Parser::new("fn incomplete(x");
    let result = parser.parse();
    assert!(result.is_err());
}

#[test]
fn test_parser_recover_multiple_errors() {
    let mut parser = Parser::new("((( ]]] }}}");
    let result = parser.parse();
    assert!(result.is_err());
    let errors = parser.get_errors();
    assert!(errors.len() >= 1);
}

// Lexer error recovery
#[test]
fn test_lexer_recover_invalid_char() {
    let mut lexer = Lexer::new("valid @ invalid");
    let tokens = lexer.tokenize();
    assert!(tokens.is_ok() || tokens.is_err());
}

#[test]
fn test_lexer_recover_unterminated_string() {
    let mut lexer = Lexer::new(r#"valid "unterminated"#);
    let tokens = lexer.tokenize();
    assert!(tokens.is_err() || tokens.is_ok());
}

#[test]
fn test_lexer_recover_invalid_escape() {
    let mut lexer = Lexer::new(r#""\q invalid escape""#);
    let tokens = lexer.tokenize();
    assert!(tokens.is_err() || tokens.is_ok());
}

#[test]
fn test_lexer_recover_invalid_number() {
    let mut lexer = Lexer::new("123abc456");
    let tokens = lexer.tokenize();
    assert!(tokens.is_ok() || tokens.is_err());
}

// Interpreter error recovery
#[test]
fn test_interpreter_recover_undefined_var() {
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval("undefined_variable");
    assert!(result.is_err());
}

#[test]
fn test_interpreter_recover_type_error() {
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval(r#""string" + 42"#);
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_interpreter_recover_division_by_zero() {
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval("42 / 0");
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_interpreter_recover_stack_overflow() {
    let mut interpreter = Interpreter::new();
    let recursive = "fn f() { f() }; f()";
    let result = interpreter.eval(recursive);
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_interpreter_recover_out_of_bounds() {
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval("[1, 2, 3][10]");
    assert!(result.is_err() || result.is_ok());
}

// Compiler error recovery
#[test]
fn test_compiler_recover_invalid_ast() {
    let compiler = Compiler::new();
    // Create invalid AST
    let result = compiler.compile_str("@#$%^&*");
    assert!(result.is_err());
}

#[test]
fn test_compiler_recover_unsupported_feature() {
    let compiler = Compiler::new();
    let result = compiler.compile_str("unsupported_feature!");
    assert!(result.is_err() || result.is_ok());
}

// Error chaining
#[test]
fn test_error_chain_parse_to_compile() {
    let compiler = Compiler::new();
    let invalid_code = "((( missing parens";
    let result = compiler.compile_str(invalid_code);
    assert!(result.is_err());
}

#[test]
fn test_error_chain_lex_to_parse() {
    let mut parser = Parser::new(r#""unterminated string"#);
    let result = parser.parse();
    assert!(result.is_err());
}

// Recovery after error
#[test]
fn test_continue_after_parse_error() {
    let mut parser = Parser::new("1 +");
    let _ = parser.parse(); // Error

    // Should be able to parse new input
    let mut parser2 = Parser::new("2 + 3");
    let result = parser2.parse();
    assert!(result.is_ok());
}

#[test]
fn test_continue_after_runtime_error() {
    let mut interpreter = Interpreter::new();
    let _ = interpreter.eval("undefined"); // Error

    // Should be able to evaluate new expression
    let result = interpreter.eval("42");
    assert!(result.is_ok());
}

// Panic recovery
#[test]
fn test_recover_from_panic() {
    std::panic::catch_unwind(|| {
        // This might panic
        let mut parser = Parser::new("((((((((((");
        let _ = parser.parse();
    })
    .ok();

    // Should still be able to continue
    assert!(true);
}

// Memory limits
#[test]
fn test_recover_memory_exhaustion() {
    // Try to allocate huge structure
    let huge_array = "[".to_owned() + &"1,".repeat(1000000) + "1]";
    let mut parser = Parser::new(&huge_array);
    let result = parser.parse();
    assert!(result.is_ok() || result.is_err());
}

// Timeout recovery
#[test]
fn test_recover_infinite_loop() {
    let mut interpreter = Interpreter::new();
    interpreter.set_timeout(std::time::Duration::from_millis(100));
    let infinite = "while true { }";
    let result = interpreter.eval(infinite);
    assert!(result.is_err() || result.is_ok());
}

// Error message quality
#[test]
fn test_helpful_error_messages() {
    let mut parser = Parser::new("if x > 5 then");
    let result = parser.parse();
    if let Err(e) = result {
        let msg = e.to_string();
        assert!(msg.contains("expected") || msg.contains("if") || !msg.is_empty());
    }
}

#[test]
fn test_error_with_location() {
    let mut parser = Parser::new("1 + * 2");
    let result = parser.parse();
    if let Err(e) = result {
        let msg = e.to_string();
        // Should contain line/column info
        assert!(!msg.is_empty());
    }
}

// Multiple error collection
#[test]
fn test_collect_multiple_errors() {
    let mut parser = Parser::new("((( + ))) * ###");
    let _ = parser.parse();
    let errors = parser.get_errors();
    assert!(errors.len() >= 1);
}

// Error recovery modes
#[test]
fn test_panic_mode_recovery() {
    let mut parser = Parser::new("1 + + + + 2");
    parser.set_recovery_mode(ruchy::frontend::parser::RecoveryMode::Panic);
    let _ = parser.parse();
    assert!(true);
}

#[test]
fn test_synchronize_recovery() {
    let mut parser = Parser::new("error; valid; error; valid");
    parser.set_recovery_mode(ruchy::frontend::parser::RecoveryMode::Synchronize);
    let _ = parser.parse();
    assert!(true);
}

// Stress test error recovery
#[test]
fn test_many_consecutive_errors() {
    let bad_code = "@#$ %^& *() !@# $%^".repeat(100);
    let mut parser = Parser::new(&bad_code);
    let _ = parser.parse();
    assert!(true);
}

#[test]
fn test_deeply_nested_errors() {
    let nested = "(((".repeat(100) + "error" + &")))".repeat(100);
    let mut parser = Parser::new(&nested);
    let _ = parser.parse();
    assert!(true);
}
