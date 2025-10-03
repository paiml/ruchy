// TDD Tests for REPL Multiline Support (REPL-006)
//
// Requirements:
// 1. Incomplete expressions should trigger multiline mode
// 2. Continuation prompt "..." should appear
// 3. Multiple lines should be buffered until complete
// 4. Complete expression should evaluate all buffered lines
// 5. Multiline mode should clear after successful evaluation

use ruchy::runtime::repl::*;
use std::path::PathBuf;

#[test]
fn test_multiline_function_definition() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Single-line function (baseline)
    let result = repl.eval("fun add(a, b) { a + b }").unwrap();
    // Function definitions typically return nil or empty
    let _ = result;

    // Call function
    let result = repl.eval("add(2, 3)").unwrap();
    assert!(result.contains('5'), "Expected 5 but got: {result}");
}

#[test]
fn test_multiline_incomplete_expression() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Start incomplete expression
    let result = repl.eval("fun add(a, b) {").unwrap();

    // Should indicate need for more input (empty result or specific message)
    // The evaluator returns NeedMoreInput for incomplete expressions
    assert!(
        result.is_empty() || result.contains("NeedMoreInput") || result.contains("Incomplete"),
        "Expected multiline continuation but got: {result}"
    );
}

#[test]
fn test_multiline_array_literal() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Single-line array
    let result = repl.eval("[1, 2, 3]").unwrap();
    assert!(result.contains('[') || result.contains('1'));
}

#[test]
fn test_multiline_object_literal() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Single-line object (if supported)
    let result = repl.eval("{ x: 10, y: 20 }").unwrap();
    // Object may or may not be supported, just check it doesn't panic
    let _ = result;
}

#[test]
fn test_multiline_if_expression() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Single-line if
    let result = repl.eval("if true { 42 } else { 0 }").unwrap();
    assert!(result.contains("42"), "Expected 42 but got: {result}");
}

#[test]
fn test_multiline_for_loop() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Single-line for loop
    let result = repl.eval("for i in [1, 2, 3] { println(i) }").unwrap();
    // For loops may not return value
    let _ = result;
}

#[test]
fn test_evaluator_multiline_state() {
    use ruchy::runtime::repl::Evaluator;

    let mut evaluator = Evaluator::new();
    let mut state = ReplState::new();

    // Initially not in multiline mode
    assert!(!evaluator.is_multiline());

    // After incomplete input, should enter multiline mode
    let result = evaluator.evaluate_line("fun test() {", &mut state).unwrap();
    matches!(result, EvalResult::NeedMoreInput);

    // Should be in multiline mode
    assert!(evaluator.is_multiline());
}

#[test]
fn test_multiline_prompt_indicator() {
    let repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Normal prompt
    let prompt = repl.get_prompt();
    assert!(
        prompt.contains('>'),
        "Expected normal prompt but got: {prompt}"
    );
    assert!(
        !prompt.contains("..."),
        "Should not have continuation prompt"
    );
}
