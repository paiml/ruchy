#![allow(missing_docs)]
//! Investigation test for Issue #86: Non-deterministic state hashing
//!
//! This test attempts to identify the source of non-determinism by:
//! 1. Capturing bindings from both Repl instances
//! 2. Comparing their Debug output
//! 3. Identifying any differences that cause hash mismatches

use ruchy::runtime::Repl;
use ruchy::runtime::replay::DeterministicRepl;

#[test]
#[ignore = "Diagnostic test - run manually to investigate Issue #86"]
fn test_issue_086_diagnose_non_determinism() {
    use tempfile::TempDir;

    // Create two independent Repl instances with isolated temp dirs
    let temp_dir1 = TempDir::new().unwrap();
    let temp_dir2 = TempDir::new().unwrap();
    let mut repl1 = Repl::new(temp_dir1.path().to_path_buf()).unwrap();
    let mut repl2 = Repl::new(temp_dir2.path().to_path_buf()).unwrap();

    // Execute identical code with same seed
    let code = "let x = 42";
    let seed = 12345;

    let result1 = repl1.execute_with_seed(code, seed);
    let result2 = repl2.execute_with_seed(code, seed);

    // Print diagnostic information
    println!("\n=== REPL 1 ===");
    println!("Hash: {}", result1.state_hash);
    println!("Bindings:");
    for (name, value) in repl1.get_bindings() {
        println!("  {name} = {value:?}");
        println!("    to_string: '{value}'");
        println!("    debug: {value:?}");
    }

    println!("\n=== REPL 2 ===");
    println!("Hash: {}", result2.state_hash);
    println!("Bindings:");
    for (name, value) in repl2.get_bindings() {
        println!("  {name} = {value:?}");
        println!("    to_string: '{value}'");
        println!("    debug: {value:?}");
    }

    println!("\n=== COMPARISON ===");
    println!("Hash 1: {}", result1.state_hash);
    println!("Hash 2: {}", result2.state_hash);
    println!("Hashes match: {}", result1.state_hash == result2.state_hash);

    // Check if bindings are identical
    let bindings1 = repl1.get_bindings();
    let bindings2 = repl2.get_bindings();

    println!("\nBinding count: {} vs {}", bindings1.len(), bindings2.len());

    for (name, value1) in bindings1 {
        if let Some(value2) = bindings2.get(name) {
            if value1 == value2 {
                println!("  MATCH: {name} = {value1:?}");
            } else {
                println!("  MISMATCH: {name} = {value1:?} vs {value2:?}");
            }
        } else {
            println!("  MISSING in repl2: {name}");
        }
    }

    for name in bindings2.keys() {
        if !bindings1.contains_key(name) {
            println!("  MISSING in repl1: {name}");
        }
    }
}

/// Run this test multiple times to check for non-determinism patterns
#[test]
#[ignore = "Run manually: cargo test issue_086_repeated --ignored -- --nocapture"]
fn test_issue_086_repeated_runs() {
    use tempfile::TempDir;

    println!("\n=== Running 10 iterations to check for hash patterns ===\n");

    let mut hash_frequency = std::collections::HashMap::new();

    for i in 1..=10 {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();
        let mut repl1 = Repl::new(temp_dir1.path().to_path_buf()).unwrap();
        let mut repl2 = Repl::new(temp_dir2.path().to_path_buf()).unwrap();

        let result1 = repl1.execute_with_seed("let x = 42", 12345);
        let result2 = repl2.execute_with_seed("let x = 42", 12345);

        *hash_frequency.entry(result1.state_hash.clone()).or_insert(0) += 1;
        *hash_frequency.entry(result2.state_hash.clone()).or_insert(0) += 1;

        let match_status = if result1.state_hash == result2.state_hash {
            "✓ MATCH"
        } else {
            "✗ MISMATCH"
        };

        println!("Run {}: {} (hash1: {}..., hash2: {}...)",
            i,
            match_status,
            &result1.state_hash[..8],
            &result2.state_hash[..8]
        );
    }

    println!("\n=== Hash Frequency Analysis ===");
    for (hash, count) in &hash_frequency {
        println!("{}... appeared {} times", &hash[..16], count);
    }

    if hash_frequency.len() > 1 {
        println!("\n⚠️  NON-DETERMINISM DETECTED: {} different hashes", hash_frequency.len());
    } else {
        println!("\n✓ DETERMINISTIC: All hashes identical");
    }
}
