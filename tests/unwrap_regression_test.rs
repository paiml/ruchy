//! Regression test to ensure unwrap() usage doesn't increase
//! This test enforces the baseline unwrap count and prevents regression

use std::fs;
use std::path::Path;
use std::process::Command;

const MAX_PRODUCTION_UNWRAPS: usize = 553; // Current baseline after cleanup

/// Count unwrap() calls in a file, excluding tests and doc comments
fn count_unwraps_in_file(path: &Path) -> usize {
    if !path.exists() {
        return 0;
    }
    
    let content = fs::read_to_string(path).unwrap_or_default();
    let lines: Vec<&str> = content.lines().collect();
    
    let mut count = 0;
    let mut in_test_block = false;
    let mut in_doc_comment = false;
    
    for line in lines {
        // Skip doc comments
        if line.trim_start().starts_with("///") {
            continue;
        }
        
        // Track if we're in a doc comment block
        if line.contains("/**") {
            in_doc_comment = true;
        }
        if in_doc_comment {
            if line.contains("*/") {
                in_doc_comment = false;
            }
            continue;
        }
        
        // Skip test attributes and functions
        if line.contains("#[test]") || line.contains("#[cfg(test)]") {
            in_test_block = true;
        }
        if line.contains("fn test_") || line.contains("mod tests") {
            in_test_block = true;
        }
        
        // Count unwraps in non-test code
        if !in_test_block && !in_doc_comment {
            count += line.matches(".unwrap()").count();
        }
        
        // Reset test block at function end
        if in_test_block && line.trim() == "}" {
            // Simple heuristic - may need refinement
            in_test_block = false;
        }
    }
    
    count
}

/// Count all production unwrap() calls in the src directory
fn count_all_production_unwraps() -> usize {
    let mut total = 0;
    
    // Walk through all Rust files in src
    for entry in walkdir::WalkDir::new("src")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        .filter(|e| !e.path().to_string_lossy().contains("test"))
    {
        total += count_unwraps_in_file(entry.path());
    }
    
    total
}

#[test]
fn test_unwrap_count_regression() {
    let current_count = count_all_production_unwraps();
    
    println!("Current production unwrap() count: {}", current_count);
    println!("Maximum allowed: {}", MAX_PRODUCTION_UNWRAPS);
    
    assert!(
        current_count <= MAX_PRODUCTION_UNWRAPS,
        "Unwrap count regression detected! Count increased from {} to {}. \
         Please use proper error handling:\n\
         - Use ? operator for error propagation\n\
         - Use .expect() with descriptive messages\n\
         - Use .unwrap_or() / .unwrap_or_else() for defaults\n\
         - Use .context() from anyhow for better errors",
        MAX_PRODUCTION_UNWRAPS,
        current_count
    );
    
    // Warn if we're getting close to the limit
    if current_count > MAX_PRODUCTION_UNWRAPS - 50 {
        eprintln!(
            "Warning: Approaching unwrap limit ({}/{})! \
             Consider refactoring to reduce unwrap usage.",
            current_count,
            MAX_PRODUCTION_UNWRAPS
        );
    }
}

#[test]
fn test_new_files_avoid_unwrap() {
    // Check that newer modules use proper error handling
    let newer_modules = [
        "src/backend/arrow_integration.rs",
        "src/wasm_bindings.rs",
    ];
    
    for module in &newer_modules {
        let path = Path::new(module);
        if path.exists() {
            let count = count_unwraps_in_file(path);
            assert!(
                count < 10,
                "New module {} has {} unwraps. New code should use proper error handling!",
                module,
                count
            );
        }
    }
}

#[test]
fn test_critical_modules_use_expect() {
    // Ensure critical modules use .expect() instead of .unwrap()
    let critical_modules = [
        "src/wasm/notebook.rs",
        "src/runtime/repl.rs",
        "src/backend/compiler.rs",
    ];
    
    for module in &critical_modules {
        let path = Path::new(module);
        if path.exists() {
            let content = fs::read_to_string(path).unwrap_or_default();
            
            // Count expect vs unwrap usage
            let expect_count = content.matches(".expect(").count();
            let unwrap_count = content.matches(".unwrap()").count();
            
            // We want more expects than unwraps in critical modules
            if unwrap_count > 0 {
                let ratio = expect_count as f64 / (unwrap_count as f64 + 0.01);
                assert!(
                    ratio > 0.5,
                    "Critical module {} should use .expect() more than .unwrap(). \
                     Found {} unwraps and {} expects (ratio: {:.2})",
                    module,
                    unwrap_count,
                    expect_count,
                    ratio
                );
            }
        }
    }
}

#[test]
#[ignore] // Run with --ignored to check unwrap patterns
fn analyze_unwrap_patterns() {
    use std::collections::HashMap;
    
    let mut patterns: HashMap<String, usize> = HashMap::new();
    
    for entry in walkdir::WalkDir::new("src")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            // Find common unwrap patterns
            for line in content.lines() {
                if line.contains(".unwrap()") {
                    if line.contains("lock().unwrap()") {
                        *patterns.entry("lock().unwrap()".to_string()).or_insert(0) += 1;
                    } else if line.contains("parse().unwrap()") {
                        *patterns.entry("parse().unwrap()".to_string()).or_insert(0) += 1;
                    } else if line.contains("join().unwrap()") {
                        *patterns.entry("join().unwrap()".to_string()).or_insert(0) += 1;
                    } else if line.contains("to_string().unwrap()") {
                        *patterns.entry("to_string().unwrap()".to_string()).or_insert(0) += 1;
                    } else {
                        *patterns.entry("other".to_string()).or_insert(0) += 1;
                    }
                }
            }
        }
    }
    
    println!("\n=== Unwrap Pattern Analysis ===");
    let mut sorted: Vec<_> = patterns.iter().collect();
    sorted.sort_by_key(|&(_, count)| std::cmp::Reverse(count));
    
    for (pattern, count) in sorted {
        println!("{}: {}", pattern, count);
    }
    
    println!("\nRecommendations:");
    if patterns.get("lock().unwrap()").unwrap_or(&0) > &0 {
        println!("- Replace lock().unwrap() with lock().expect(\"Failed to acquire lock\")");
    }
    if patterns.get("parse().unwrap()").unwrap_or(&0) > &0 {
        println!("- Replace parse().unwrap() with parse()? or parse().expect(\"Failed to parse\")");
    }
}