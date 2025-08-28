// Test for REPL-MAGIC-003: Session export to clean script
// Tests the :export command for generating production scripts

use ruchy::runtime::Repl;
use std::fs;

#[test]
fn test_export_command_basic() {
    let mut repl = Repl::new().unwrap();
    
    // Create a simple session
    repl.eval("let x = 42").unwrap();
    repl.eval("let y = x + 8").unwrap();
    repl.eval("println(y)").unwrap();
    
    // Export the session
    let result = repl.eval(":export test_basic_export.ruchy").unwrap();
    assert!(result.contains("Session exported to clean script"), 
            "Should confirm export success");
    
    // Check the exported file exists and contains expected content
    let exported_content = fs::read_to_string("test_basic_export.ruchy").unwrap();
    
    assert!(exported_content.contains("let x = 42;"), "Should contain x assignment");
    assert!(exported_content.contains("let y = x + 8;"), "Should contain y assignment");
    assert!(exported_content.contains("println(y);"), "Should contain println");
    assert!(exported_content.contains("fn main()"), "Should be wrapped in main function");
    assert!(exported_content.contains("Ok(())"), "Should have Result return type");
    
    // Clean up
    let _ = fs::remove_file("test_basic_export.ruchy");
}

#[test]
fn test_export_command_filters_display_only() {
    let mut repl = Repl::new().unwrap();
    
    // Create session with mix of productive and display-only commands
    repl.eval("let x = 10").unwrap();
    repl.eval("x").unwrap(); // Display only
    repl.eval("let y = 20").unwrap();
    repl.eval("y").unwrap(); // Display only
    repl.eval("let sum = x + y").unwrap();
    
    // Export the session
    repl.eval(":export test_filtered_export.ruchy").unwrap();
    
    // Check the exported content
    let exported_content = fs::read_to_string("test_filtered_export.ruchy").unwrap();
    
    // Should contain productive statements
    assert!(exported_content.contains("let x = 10;"), "Should contain x assignment");
    assert!(exported_content.contains("let y = 20;"), "Should contain y assignment");
    assert!(exported_content.contains("let sum = x + y;"), "Should contain sum assignment");
    
    // Should note filtered display-only commands
    assert!(exported_content.contains("// x (display only - removed)"), 
            "Should note filtered x display");
    assert!(exported_content.contains("// y (display only - removed)"), 
            "Should note filtered y display");
    
    // Clean up
    let _ = fs::remove_file("test_filtered_export.ruchy");
}

#[test]
fn test_export_command_with_regular_statements() {
    let mut repl = Repl::new().unwrap();
    
    // Create session with regular statements
    repl.eval("let x = 5").unwrap();
    repl.eval("let y = 10").unwrap(); 
    repl.eval("let z = x + y").unwrap();
    
    // Export the session
    repl.eval(":export test_regular_statements_export.ruchy").unwrap();
    
    // Check the exported content
    let exported_content = fs::read_to_string("test_regular_statements_export.ruchy").unwrap();
    
    // Should contain all regular statements
    assert!(exported_content.contains("let x = 5;"), "Should contain x assignment");
    assert!(exported_content.contains("let y = 10;"), "Should contain y assignment");
    assert!(exported_content.contains("let z = x + y;"), "Should contain z assignment");
    
    // Should be wrapped in main function
    assert!(exported_content.contains("fn main()"), "Should have main function");
    
    // Clean up
    let _ = fs::remove_file("test_regular_statements_export.ruchy");
}

#[test]
fn test_export_command_no_args() {
    let mut repl = Repl::new().unwrap();
    
    // Test with no arguments
    let result = repl.eval(":export").unwrap();
    
    assert!(result.contains("Usage: :export <filename>"), 
            "Should show usage when no arguments provided");
}

#[test]
fn test_export_command_empty_session() {
    let mut repl = Repl::new().unwrap();
    
    // Export empty session
    let result = repl.eval(":export test_empty_export.ruchy").unwrap();
    assert!(result.contains("Session exported to clean script"), 
            "Should handle empty session gracefully");
    
    // Check the exported content
    let exported_content = fs::read_to_string("test_empty_export.ruchy").unwrap();
    
    assert!(exported_content.contains("// No executable statements to export"), 
            "Should note empty session");
    assert!(exported_content.contains("fn main() {"), "Should still have main function");
    assert!(exported_content.contains("println!(\"Hello, Ruchy!\");"), 
            "Should have placeholder content");
    
    // Clean up
    let _ = fs::remove_file("test_empty_export.ruchy");
}

#[test]
fn test_export_command_with_functions() {
    let mut repl = Repl::new().unwrap();
    
    // Create session with function definition and call
    repl.eval("fn double(n) { n * 2 }").unwrap();
    repl.eval("let result = double(21)").unwrap();
    repl.eval("println(result)").unwrap();
    
    // Export the session
    repl.eval(":export test_function_export.ruchy").unwrap();
    
    // Check the exported content
    let exported_content = fs::read_to_string("test_function_export.ruchy").unwrap();
    
    assert!(exported_content.contains("fn double(n) { n * 2 };"), 
            "Should contain function definition");
    assert!(exported_content.contains("let result = double(21);"), 
            "Should contain function call");
    assert!(exported_content.contains("println(result);"), 
            "Should contain println statement");
    
    // Clean up
    let _ = fs::remove_file("test_function_export.ruchy");
}

#[test]
fn test_export_command_header_and_metadata() {
    let mut repl = Repl::new().unwrap();
    
    // Create a simple session
    repl.eval("let greeting = \"Hello, World!\"").unwrap();
    
    // Export the session
    repl.eval(":export test_metadata_export.ruchy").unwrap();
    
    // Check the exported content has proper header
    let exported_content = fs::read_to_string("test_metadata_export.ruchy").unwrap();
    
    assert!(exported_content.contains("// Ruchy Script - Exported from REPL Session"), 
            "Should have proper header");
    assert!(exported_content.contains("// Generated:"), "Should have timestamp");
    assert!(exported_content.contains("// Total commands:"), "Should show command count");
    
    // Clean up
    let _ = fs::remove_file("test_metadata_export.ruchy");
}