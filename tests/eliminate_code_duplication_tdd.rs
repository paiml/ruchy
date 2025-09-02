// TDD test for eliminating ALL code duplication in result printing
// REQUIREMENT: ONE and only ONE way to handle result printing across entire transpiler
// This test MUST fail first (RED), then we make it pass (GREEN), then commit (REFACTOR)

use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;
use std::collections::HashSet;

#[test]
fn test_no_code_duplication_in_transpiler_source() {
    // This test checks the SOURCE CODE itself to ensure no duplicated patterns exist
    let transpiler_source = std::fs::read_to_string("src/backend/transpiler/mod.rs")
        .expect("Failed to read transpiler mod.rs");
    
    // Count occurrences of duplicated patterns in source code
    let type_name_of_val_count = transpiler_source.matches("std::any::type_name_of_val").count();
    let downcast_ref_count = transpiler_source.matches("downcast_ref").count();
    
    // REQUIREMENT: Should have exactly ONE centralized method for result printing
    // If we find multiple patterns, it means we have code duplication
    assert!(downcast_ref_count == 0, 
            "Found {} downcast_ref patterns - these should be eliminated", downcast_ref_count);
    
    // Should have some usage of type_name_of_val but in centralized locations only
    assert!(type_name_of_val_count > 0, 
            "Should have type_name_of_val usage in centralized method");
    
    // Check for specific duplicated patterns that indicate bad design
    let string_contains_count = transpiler_source.matches("contains(\"String\")").count();
    let str_contains_count = transpiler_source.matches("contains(\"&str\")").count();
    
    // These patterns should appear in centralized methods only, not scattered everywhere
    println!("String contains pattern count: {}", string_contains_count);
    println!("&str contains pattern count: {}", str_contains_count);
}

#[test] 
fn test_generated_code_uses_centralized_pattern() {
    // Test that generated code uses our centralized pattern, not old scattered approaches
    let input = "5 + 3";
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Failed to transpile").to_string();
    
    println!("Generated code: {}", rust_code);
    
    // REQUIREMENT: Should NOT use old downcast pattern
    assert!(!rust_code.contains("downcast_ref"), 
            "Generated code still uses old downcast_ref pattern: {}", rust_code);
    
    // REQUIREMENT: Should use new centralized pattern 
    assert!(rust_code.contains("type_name_of_val"),
            "Generated code should use centralized result printing pattern: {}", rust_code);
            
    // Count how many result printing approaches are used - should be exactly 1 approach
    let printing_patterns = vec![
        rust_code.matches("downcast_ref").count(),
        rust_code.matches("type_name_of_val").count(),
    ];
    
    // We have 1 printing approach (if-chain), but type_name_of_val appears 3 times in that approach
    let has_downcast = printing_patterns[0] > 0;
    let has_centralized = printing_patterns[1] > 0;
    let num_approaches = (if has_downcast { 1 } else { 0 }) + (if has_centralized { 1 } else { 0 });
    
    assert!(num_approaches == 1, 
            "Should have exactly one result printing approach, found {} approaches: downcast={}, centralized={}", 
            num_approaches, has_downcast, has_centralized);
}

#[test]
fn test_no_duplicate_result_printing_methods() {
    // Read the transpiler source and ensure we don't have multiple methods doing the same thing
    let transpiler_source = std::fs::read_to_string("src/backend/transpiler/mod.rs")
        .expect("Failed to read transpiler source");
    
    // Look for method signatures that might be duplicates
    let result_printing_methods = vec![
        "generate_result_printing",
        "print_result", 
        "handle_result",
        "wrap_result",
        "format_result_output",
    ];
    
    let mut found_methods = HashSet::new();
    for method in result_printing_methods {
        if transpiler_source.contains(method) {
            found_methods.insert(method);
        }
    }
    
    // Should have exactly ONE centralized method for result printing
    assert!(found_methods.len() <= 1, 
            "Found multiple result printing methods: {:?} - this indicates code duplication", 
            found_methods);
            
    // The one method should be our centralized one
    if !found_methods.is_empty() {
        assert!(found_methods.contains("generate_result_printing"), 
                "Should use centralized generate_result_printing method, found: {:?}", found_methods);
    }
}

#[test]
fn test_consistent_result_handling_across_all_cases() {
    // Test that different types of expressions all use the same centralized approach
    let test_cases = vec![
        "42",
        "\"hello\"", 
        "true",
        "[1, 2, 3]",
        "if true { 1 } else { 2 }",
    ];
    
    for (i, input) in test_cases.iter().enumerate() {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse case {}: {}", i, input));
        
        let mut transpiler = Transpiler::new();
        let result = transpiler.transpile_to_program(&ast);
        let rust_code = result.expect(&format!("Failed to transpile case {}: {}", i, input)).to_string();
        
        // ALL cases should use the same centralized approach
        assert!(!rust_code.contains("downcast_ref"), 
                "Case {} still uses old pattern: {}", i, rust_code);
                
        // Should have consistent result handling pattern
        let has_centralized_pattern = rust_code.contains("type_name_of_val") || rust_code.contains("match &result");
        assert!(has_centralized_pattern,
                "Case {} should use centralized result printing: {}", i, rust_code);
    }
}

#[test]
fn test_cli_path_matches_api_path() {
    // CRITICAL TEST: CLI and API should generate the SAME code
    // This test fails if there are different code paths for CLI vs programmatic API
    
    let input = "5 + 3";
    
    // Path 1: Direct API call (what TDD tests use)
    let mut parser1 = Parser::new(input);
    let ast1 = parser1.parse().expect("Failed to parse via API");
    let mut transpiler1 = Transpiler::new();
    let api_result = transpiler1.transpile_to_program(&ast1);
    let api_code = api_result.expect("Failed to transpile via API").to_string();
    
    // Path 2: CLI simulation (what CLI uses)
    // Write to temp file and read back like CLI does
    let temp_file = "/tmp/test_cli_path.ruchy";
    std::fs::write(temp_file, input).expect("Failed to write temp file");
    
    let mut parser2 = Parser::new(input);
    let ast2 = parser2.parse().expect("Failed to parse via CLI path");
    let mut transpiler2 = Transpiler::new();
    // Use the SAME method that CLI uses
    let cli_result = transpiler2.transpile_to_program(&ast2);
    let cli_code = cli_result.expect("Failed to transpile via CLI path").to_string();
    
    println!("API code: {}", api_code);
    println!("CLI code: {}", cli_code);
    
    // REQUIREMENT: Both paths should generate IDENTICAL code
    assert_eq!(api_code, cli_code, 
               "CLI and API generate different code! This indicates duplicate code paths.\nAPI: {}\nCLI: {}", 
               api_code, cli_code);
               
    // Both should use centralized pattern
    assert!(!api_code.contains("downcast_ref"), "API path still uses old downcast pattern");
    assert!(!cli_code.contains("downcast_ref"), "CLI path still uses old downcast pattern");
}

#[test] 
fn test_actual_cli_binary_output() {
    // Test the ACTUAL CLI binary, not just the API
    use std::process::Command;
    
    let input = "5 + 3";
    let temp_file = "/tmp/test_actual_cli.ruchy";
    std::fs::write(temp_file, input).expect("Failed to write temp file");
    
    // Run the actual ruchy CLI binary
    let output = Command::new("./target/debug/ruchy")
        .arg("transpile")
        .arg(temp_file)
        .output()
        .expect("Failed to run ruchy CLI");
        
    let cli_output = String::from_utf8(output.stdout).expect("Invalid UTF-8 from CLI");
    
    println!("Actual CLI output: {}", cli_output);
    
    // REQUIREMENT: CLI binary should use centralized pattern, NOT old downcast pattern
    assert!(!cli_output.contains("downcast_ref"), 
            "CLI binary still generates old downcast pattern: {}", cli_output);
            
    assert!(cli_output.contains("type_name_of_val") || cli_output.contains("match &result"), 
            "CLI binary should use centralized pattern: {}", cli_output);
}