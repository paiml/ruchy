//! Property-based defect hunting for qualified name parsing
#![allow(clippy::expect_used, clippy::print_stdout, clippy::uninlined_format_args, clippy::manual_let_else, clippy::needless_borrows_for_generic_args, clippy::single_char_pattern)] // Test code allows these
//! 
//! This systematically tests the boundary conditions where parsing fails
//! in REPL vs unit tests to find the EXACT root cause

use std::process::{Command, Stdio};
use std::io::Write;
use ruchy::frontend::parser::Parser;

/// Property: Any qualified name that parses in unit tests MUST parse in REPL
#[test]
fn property_unit_test_repl_parsing_equivalence() {
    // Generate systematic test cases
    let test_cases = generate_qualified_name_test_cases();
    
    let mut unit_pass_repl_fail = Vec::new();
    let mut both_pass = Vec::new();
    let mut both_fail = Vec::new();
    let mut unit_fail_repl_pass = Vec::new();
    
    for input in test_cases {
        let unit_result = test_unit_parsing(&input);
        let repl_result = test_repl_parsing(&input);
        
        match (unit_result, repl_result) {
            (true, false) => {
                println!("🚨 DEFECT: '{}' passes unit but fails REPL", input);
                unit_pass_repl_fail.push(input);
            }
            (true, true) => both_pass.push(input),
            (false, false) => both_fail.push(input),
            (false, true) => {
                println!("🤔 WEIRD: '{}' fails unit but passes REPL", input);
                unit_fail_repl_pass.push(input);
            }
        }
    }
    
    println!("\n=== PROPERTY TEST RESULTS ===");
    println!("Both pass: {}", both_pass.len());
    println!("Both fail: {}", both_fail.len());
    println!("Unit pass, REPL fail: {} ← DEFECTS", unit_pass_repl_fail.len());
    println!("Unit fail, REPL pass: {} ← WEIRD", unit_fail_repl_pass.len());
    
    if !unit_pass_repl_fail.is_empty() {
        println!("\n🚨 DEFECT PATTERNS:");
        for defect in &unit_pass_repl_fail {
            println!("  - {}", defect);
        }
        
        // Analyze the patterns
        analyze_defect_patterns(&unit_pass_repl_fail);
    }
    
    // Property violation: Unit tests and REPL must be equivalent
    assert!(
        unit_pass_repl_fail.is_empty(),
        "PROPERTY VIOLATION: {} inputs parse in unit tests but fail in REPL. This indicates architectural inconsistency.",
        unit_pass_repl_fail.len()
    );
}

fn generate_qualified_name_test_cases() -> Vec<String> {
    let mut cases = Vec::new();
    
    // Systematic generation by segment count
    cases.extend(generate_by_segments(1)); // a
    cases.extend(generate_by_segments(2)); // a::b
    cases.extend(generate_by_segments(3)); // a::b::c  ← suspected boundary
    cases.extend(generate_by_segments(4)); // a::b::c::d
    cases.extend(generate_by_segments(5)); // a::b::c::d::e
    
    // Special token combinations
    cases.extend(generate_special_token_cases());
    
    // Function call variations
    cases.extend(generate_function_call_cases());
    
    // Constructor variations  
    cases.extend(generate_constructor_cases());
    
    cases
}

fn generate_by_segments(count: usize) -> Vec<String> {
    let segments: Vec<&str> = vec!["std", "fs", "io", "collections", "result", "option", "string"];
    let mut cases = Vec::new();
    
    if count == 0 { return cases; }
    
    // Generate all combinations of `count` segments
    for i in 0..segments.len().min(count) {
        let mut path = segments[i].to_string();
        for j in 1..count {
            let segment_idx = (i + j) % segments.len();
            path = format!("{}::{}", path, segments[segment_idx]);
        }
        cases.push(path.clone());
        
        // Add function call version
        cases.push(format!("{}::function()", path));
        
        // Add with arguments
        cases.push(format!("{}::method(42)", path));
        cases.push(format!("{}::method(\"test\")", path));
    }
    
    cases
}

fn generate_special_token_cases() -> Vec<String> {
    let special_tokens = vec!["Result", "Option", "Ok", "Err", "Some", "None"];
    let prefixes = vec!["std", "core", "std::result", "std::option"];
    let mut cases = Vec::new();
    
    for prefix in prefixes {
        for token in &special_tokens {
            // Qualified special token
            cases.push(format!("{}::{}", prefix, token));
            
            // With constructor
            cases.push(format!("{}::{}(42)", prefix, token));
            
            // Nested special tokens
            for token2 in &special_tokens {
                cases.push(format!("{}::{}::{}", prefix, token, token2));
                cases.push(format!("{}::{}::{}(42)", prefix, token, token2));
            }
        }
    }
    
    cases
}

fn generate_function_call_cases() -> Vec<String> {
    vec![
        "std::fs::read_file(\"test.txt\")".to_string(),
        "std::io::println(\"hello\")".to_string(),
        "std::collections::HashMap::new()".to_string(),
        "std::result::Result::Ok(42)".to_string(),
        "std::option::Option::Some(42)".to_string(),
        "a::b::c::deeply::nested::function()".to_string(),
        "super::parent::module::method()".to_string(),
        "crate::internal::utils::helper()".to_string(),
    ]
}

fn generate_constructor_cases() -> Vec<String> {
    vec![
        "std::result::Result::Ok(42)".to_string(),
        "std::result::Result::Err(\"error\")".to_string(),
        "std::option::Option::Some(42)".to_string(),
        "std::option::Option::None".to_string(),
        "Result::Ok(std::option::Option::Some(42))".to_string(),
        "Option::Some(std::result::Result::Ok(42))".to_string(),
    ]
}

fn test_unit_parsing(input: &str) -> bool {
    let mut parser = Parser::new(input);
    parser.parse().is_ok()
}

fn test_repl_parsing(input: &str) -> bool {
    let mut child = match Command::new("cargo")
        .args(&["run", "--bin", "ruchy", "repl"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => return false,
    };
    
    if let Some(stdin) = child.stdin.as_mut() {
        let _ = writeln!(stdin, "{}", input);
    }
    
    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(_) => return false,
    };
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Parse succeeded if no "Failed to parse input" message
    !stderr.contains("Failed to parse input")
}

fn analyze_defect_patterns(defects: &[String]) {
    println!("\n🔍 DEFECT PATTERN ANALYSIS:");
    
    // Count by segment count
    let mut by_segments: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for defect in defects {
        let segment_count = defect.matches("::").count() + 1;
        *by_segments.entry(segment_count).or_insert(0) += 1;
    }
    
    println!("By segment count:");
    for (count, freq) in by_segments {
        println!("  {} segments: {} cases", count, freq);
    }
    
    // Check for common prefixes
    let common_prefixes = vec!["std::", "std::result::", "std::option::", "std::fs::", "a::b::"];
    println!("\nBy common prefixes:");
    for prefix in common_prefixes {
        let count = defects.iter().filter(|d| d.starts_with(prefix)).count();
        if count > 0 {
            println!("  '{}': {} cases", prefix, count);
        }
    }
    
    // Check for function calls vs bare names
    let function_calls = defects.iter().filter(|d| d.contains("(")).count();
    let bare_names = defects.len() - function_calls;
    println!("\nBy type:");
    println!("  Function calls: {}", function_calls);
    println!("  Bare names: {}", bare_names);
}

/// Fuzz test: Generate random qualified names and test consistency
#[test]
fn fuzz_qualified_names_consistency() {
    use std::collections::HashSet;
    
    let segments = vec!["std", "core", "alloc", "fs", "io", "collections", "result", "option", 
                       "string", "vec", "hash", "map", "Result", "Option", "Ok", "Err", "Some", "None"];
    let functions = vec!["new", "read", "write", "open", "close", "get", "set", "push", "pop"];
    
    let mut tested = HashSet::new();
    let mut inconsistencies = Vec::new();
    
    // Generate 1000 random qualified names
    for seed in 0..1000 {
        let qualified_name = generate_random_qualified_name(seed, &segments, &functions);
        
        if tested.contains(&qualified_name) {
            continue; // Skip duplicates
        }
        tested.insert(qualified_name.clone());
        
        let unit_result = test_unit_parsing(&qualified_name);
        let repl_result = test_repl_parsing(&qualified_name);
        
        if unit_result != repl_result {
            inconsistencies.push((qualified_name.clone(), unit_result, repl_result));
            
            if inconsistencies.len() <= 10 { // Only print first 10
                println!("🎯 FUZZ FOUND: '{}' unit:{} repl:{}", 
                        qualified_name, unit_result, repl_result);
            }
        }
    }
    
    println!("\n🎯 FUZZ RESULTS: Found {} inconsistencies out of {} tests", 
            inconsistencies.len(), tested.len());
    
    if !inconsistencies.is_empty() {
        println!("Sample inconsistencies:");
        for (name, unit, repl) in inconsistencies.iter().take(5) {
            println!("  '{}' → unit:{} repl:{}", name, unit, repl);
        }
    }
    
    // Fuzz property: Inconsistencies should be minimal (allowing for some edge cases)
    assert!(inconsistencies.len() < tested.len() / 10,
           "Too many inconsistencies: {}/{} ({}%) - indicates systematic parsing differences",
           inconsistencies.len(), tested.len(), 
           inconsistencies.len() * 100 / tested.len());
}

fn generate_random_qualified_name(seed: usize, segments: &[&str], functions: &[&str]) -> String {
    // Simple deterministic "randomness" based on seed
    let segment_count = (seed % 5) + 1; // 1-5 segments
    let mut name = String::new();
    
    for i in 0..segment_count {
        if i > 0 { name.push_str("::"); }
        let segment_idx = (seed + i) % segments.len();
        name.push_str(segments[segment_idx]);
    }
    
    // Sometimes add function call
    if seed % 3 == 0 {
        let func_idx = seed % functions.len();
        name = format!("{}::{}()", name, functions[func_idx]);
    }
    
    // Sometimes add constructor args
    if seed % 4 == 0 {
        if name.ends_with("()") {
            name = name[..name.len()-2].to_string(); // Remove ()
        }
        let arg_type = seed % 3;
        match arg_type {
            0 => name = format!("{}(42)", name),
            1 => name = format!("{}(\"test\")", name),
            _ => name = format!("{}(x)", name),
        }
    }
    
    name
}