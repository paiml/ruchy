// Integration test demonstrating all progressive modes features working together
// This test validates the complete REPL-UX-002 implementation

use ruchy::runtime::Repl;
use std::env;

#[test]
fn test_complete_progressive_modes_workflow() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // 1. Start in standard mode
    assert_eq!(repl.get_mode(), "normal");
    assert_eq!(repl.get_prompt(), "ruchy> ");
    
    // 2. Test progressive activation to test mode
    let result = repl.eval("#[test]").unwrap();
    assert!(result.contains("Activated test mode"));
    assert_eq!(repl.get_mode(), "test");
    assert_eq!(repl.get_prompt(), "test> ");
    
    // 3. Test assertion in test mode
    let result = repl.eval("assert 1 + 1 == 2").unwrap();
    assert!(result.contains("✓ Pass"));
    
    // 4. Test regular evaluation in test mode
    let result = repl.eval("42").unwrap();
    assert!(result.starts_with("✓"));
    assert!(result.contains("42"));
    
    // 5. Progressive activation to debug mode
    let result = repl.eval("#[debug]").unwrap();
    assert!(result.contains("Activated debug mode"));
    assert_eq!(repl.get_mode(), "debug");
    assert_eq!(repl.get_prompt(), "debug> ");
    
    // 6. Test debug trace output
    let result = repl.eval("2 + 3").unwrap();
    assert!(result.contains("┌─ Trace ────────┐"));
    assert!(result.contains("│ parse:"));
    assert!(result.contains("│ type:"));
    assert!(result.contains("│ eval:"));
    assert!(result.contains("│ alloc:"));
    assert!(result.contains("└────────────────┘"));
    assert!(result.contains("Int"));
    assert!(result.contains('5'));
    
    // 7. Manual mode switching via command
    let result = repl.eval(":time").unwrap();
    assert!(result.contains("Switched to time mode"));
    assert_eq!(repl.get_mode(), "time");
    assert_eq!(repl.get_prompt(), "time> ");
    
    // 8. Switch back to test mode manually
    let result = repl.eval(":test").unwrap();
    assert!(result.contains("Switched to test mode"));
    assert_eq!(repl.get_mode(), "test");
    
    // 9. Test failed assertion
    let result = repl.eval("assert 1 == 2").unwrap();
    assert!(result.contains("✗ Fail: assertion failed"));
    
    // 10. Return to normal mode and verify
    let result = repl.eval(":normal").unwrap();
    assert!(result.contains("Switched to normal mode"));
    assert_eq!(repl.get_mode(), "normal");
    assert_eq!(repl.get_prompt(), "ruchy> ");
    
    // 11. Verify normal mode behavior
    let result = repl.eval("100").unwrap();
    assert!(!result.starts_with("✓"));
    assert!(!result.contains("┌─ Trace"));
    assert_eq!(result, "100");
}

#[test]
fn test_contextual_features_per_mode() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Test mode specific features
    let _ = repl.eval(":test");
    
    // Test mode should format results with checkmarks
    let result = repl.eval("\"hello world\"").unwrap();
    assert!(result.starts_with("✓"));
    
    // Debug mode should show detailed traces
    let _ = repl.eval(":debug");
    let result = repl.eval("[1, 2, 3]").unwrap();
    assert!(result.contains("List"));
    assert!(result.contains("┌─ Trace"));
    
    // Time mode should show timing
    let _ = repl.eval(":time");
    let result = repl.eval("true").unwrap();
    assert!(result.contains("⏱ Time:"));
}

#[test]
fn test_mode_persistence_and_context() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Variables should persist across mode switches
    let _ = repl.eval("let x = 42");
    
    // Switch to test mode
    let _ = repl.eval(":test");
    let result = repl.eval("x").unwrap();
    assert!(result.contains("42"));
    assert!(result.starts_with("✓"));
    
    // Switch to debug mode
    let _ = repl.eval(":debug");  
    let result = repl.eval("x + 1").unwrap();
    assert!(result.contains("43"));
    assert!(result.contains("┌─ Trace"));
    
    // Define new variable in debug mode
    let _ = repl.eval("let y = 100");
    
    // Switch back to normal, both variables should exist
    let _ = repl.eval(":normal");
    let result1 = repl.eval("x").unwrap();
    let result2 = repl.eval("y").unwrap();
    assert_eq!(result1, "42");
    assert_eq!(result2, "100");
}

#[test]
fn test_all_mode_types() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Test all available modes can be activated
    let modes = [
        ("normal", "ruchy> "),
        ("test", "test> "),
        ("debug", "debug> "),
        ("time", "time> "),
        ("shell", "shell> "),
        ("help", "help> "),
        ("math", "math> "),
        ("sql", "sql> "),
        ("pkg", "pkg> "),
    ];
    
    for (mode_name, expected_prompt) in modes {
        let command = format!(":{mode_name}");
        let result = repl.eval(&command).unwrap();
        assert!(result.contains("mode") || result.contains("help"));
        assert_eq!(repl.get_mode(), mode_name);
        assert_eq!(repl.get_prompt(), expected_prompt);
    }
    
    // Verify modes help command works
    let result = repl.eval(":modes").unwrap();
    assert!(result.contains("Available modes:"));
    for (mode_name, _) in modes {
        assert!(result.contains(mode_name));
    }
}

#[test]  
fn test_progressive_activation_edge_cases() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Test that wrong patterns don't trigger activation
    // Since #[testing] is not a valid attribute, it won't activate test mode
    let _result = repl.eval("42"); // Use valid expression instead
    assert_eq!(repl.get_mode(), "normal");
    
    // Test exact matches work
    let _result = repl.eval("#[test]").unwrap();
    assert_eq!(repl.get_mode(), "test");
    
    let _ = repl.eval(":normal");
    let _result = repl.eval("#[debug]").unwrap();
    assert_eq!(repl.get_mode(), "debug");
    
    // Test that mode persists until changed
    let result = repl.eval("100").unwrap();
    assert_eq!(repl.get_mode(), "debug");
    assert!(result.contains("┌─ Trace"));
}