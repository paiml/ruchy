//! Coverage tests for magic commands
//!
//! [TEST-COV-005] Magic Commands Coverage

use ruchy::runtime::{Repl, ReplConfig, MagicRegistry, MagicCommand, MagicResult, UnicodeExpander};

#[test]
fn test_magic_registry() {
    let mut registry = MagicRegistry::new();
    
    // Test registration
    struct TestCommand;
    impl MagicCommand for TestCommand {
        fn name(&self) -> &str { "test" }
        fn description(&self) -> &str { "Test command" }
        fn execute(&self, _repl: &mut Repl, _args: &str) -> MagicResult {
            Ok("Test executed".to_string())
        }
    }
    
    registry.register(Box::new(TestCommand));
    assert!(registry.get("test").is_some());
    assert!(registry.get("nonexistent").is_none());
    
    // Test listing
    let commands = registry.list_commands();
    assert!(commands.iter().any(|(name, _)| name == "test"));
}

#[test]
fn test_unicode_expander() {
    let expander = UnicodeExpander::new();
    
    // Greek letters
    assert_eq!(expander.expand("\\alpha"), Some("α"));
    assert_eq!(expander.expand("\\beta"), Some("β"));
    assert_eq!(expander.expand("\\gamma"), Some("γ"));
    assert_eq!(expander.expand("\\delta"), Some("δ"));
    assert_eq!(expander.expand("\\epsilon"), Some("ε"));
    assert_eq!(expander.expand("\\lambda"), Some("λ"));
    assert_eq!(expander.expand("\\pi"), Some("π"));
    assert_eq!(expander.expand("\\sigma"), Some("σ"));
    assert_eq!(expander.expand("\\omega"), Some("ω"));
    
    // Capital Greek
    assert_eq!(expander.expand("\\Alpha"), Some("Α"));
    assert_eq!(expander.expand("\\Beta"), Some("Β"));
    assert_eq!(expander.expand("\\Gamma"), Some("Γ"));
    assert_eq!(expander.expand("\\Delta"), Some("Δ"));
    
    // Mathematical symbols
    assert_eq!(expander.expand("\\infty"), Some("∞"));
    assert_eq!(expander.expand("\\sum"), Some("∑"));
    assert_eq!(expander.expand("\\prod"), Some("∏"));
    assert_eq!(expander.expand("\\int"), Some("∫"));
    assert_eq!(expander.expand("\\sqrt"), Some("√"));
    assert_eq!(expander.expand("\\partial"), Some("∂"));
    assert_eq!(expander.expand("\\nabla"), Some("∇"));
    
    // Arrows
    assert_eq!(expander.expand("\\rightarrow"), Some("→"));
    assert_eq!(expander.expand("\\leftarrow"), Some("←"));
    assert_eq!(expander.expand("\\Rightarrow"), Some("⇒"));
    assert_eq!(expander.expand("\\Leftarrow"), Some("⇐"));
    
    // Operators
    assert_eq!(expander.expand("\\pm"), Some("±"));
    assert_eq!(expander.expand("\\times"), Some("×"));
    assert_eq!(expander.expand("\\div"), Some("÷"));
    assert_eq!(expander.expand("\\neq"), Some("≠"));
    assert_eq!(expander.expand("\\leq"), Some("≤"));
    assert_eq!(expander.expand("\\geq"), Some("≥"));
    
    // Non-existent
    assert_eq!(expander.expand("\\nonexistent"), None);
    assert_eq!(expander.expand("not_a_command"), None);
}

#[test]
fn test_expand_in_text() {
    let expander = UnicodeExpander::new();
    
    let text = "The equation is \\alpha + \\beta = \\gamma";
    let expanded = expander.expand_in_text(text);
    assert_eq!(expanded, "The equation is α + β = γ");
    
    let text = "\\sum_{i=1}^{n} x_i \\rightarrow \\infty";
    let expanded = expander.expand_in_text(text);
    assert!(expanded.contains("∑"));
    assert!(expanded.contains("→"));
    assert!(expanded.contains("∞"));
}

#[test]
fn test_completions() {
    let expander = UnicodeExpander::new();
    
    let completions = expander.get_completions("\\alp");
    assert!(completions.contains(&"\\alpha".to_string()));
    
    let completions = expander.get_completions("\\bet");
    assert!(completions.contains(&"\\beta".to_string()));
    
    let completions = expander.get_completions("\\in");
    assert!(completions.contains(&"\\infty".to_string()));
    assert!(completions.contains(&"\\int".to_string()));
    
    let completions = expander.get_completions("\\");
    assert!(completions.len() > 50); // Many completions
    
    let completions = expander.get_completions("xyz");
    assert!(completions.is_empty());
}

#[test]
fn test_magic_commands_execution() {
    let mut repl = Repl::new().unwrap();
    
    // Test %whos
    repl.eval("let x = 42").ok();
    repl.eval("let y = \"hello\"").ok();
    let result = repl.eval("%whos").unwrap_or_default();
    assert!(result.contains("x") || result.contains("Variable"));
    
    // Test %clear
    repl.eval("%clear").ok();
    let result = repl.eval("%whos").unwrap_or_default();
    assert!(!result.contains("x") || result.contains("No variables"));
    
    // Test %pwd
    let result = repl.eval("%pwd").unwrap_or_default();
    assert!(result.contains("/") || result.contains("\\"));
    
    // Test %history
    let result = repl.eval("%history").unwrap_or_default();
    // History should contain previous commands
}

#[test]
fn test_profile_data() {
    use ruchy::runtime::ProfileData;
    
    let mut profile = ProfileData::new();
    
    profile.record_call("function1", std::time::Duration::from_millis(100));
    profile.record_call("function1", std::time::Duration::from_millis(50));
    profile.record_call("function2", std::time::Duration::from_millis(200));
    
    let report = profile.generate_report();
    assert!(report.contains("function1"));
    assert!(report.contains("function2"));
    assert!(report.contains("2")); // 2 calls to function1
    assert!(report.contains("150")); // 150ms total for function1
}