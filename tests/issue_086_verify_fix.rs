#![allow(missing_docs)]
//! Verification test for Issue #86 fix: Run 100 iterations to verify determinism

use ruchy::runtime::replay::DeterministicRepl;
use ruchy::runtime::Repl;

#[test]
fn test_issue_086_fix_verification_100_iterations() {
    use tempfile::TempDir;

    println!("\n=== Running 100 iterations to verify fix for Issue #86 ===\n");

    let mut hash_frequency = std::collections::HashMap::new();
    let mut mismatch_count = 0;
    let mut match_count = 0;

    for i in 1..=100 {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();
        let mut repl1 = Repl::new(temp_dir1.path().to_path_buf()).unwrap();
        let mut repl2 = Repl::new(temp_dir2.path().to_path_buf()).unwrap();

        let result1 = repl1.execute_with_seed("let x = 42", 12345);
        let result2 = repl2.execute_with_seed("let x = 42", 12345);

        *hash_frequency
            .entry(result1.state_hash.clone())
            .or_insert(0) += 1;
        *hash_frequency
            .entry(result2.state_hash.clone())
            .or_insert(0) += 1;

        if result1.state_hash == result2.state_hash {
            match_count += 1;
        } else {
            mismatch_count += 1;
            if mismatch_count <= 5 {
                println!(
                    "Run {}: ✗ MISMATCH (hash1: {}..., hash2: {}...)",
                    i,
                    &result1.state_hash[..16],
                    &result2.state_hash[..16]
                );
            }
        }

        // Print progress every 10 iterations
        if i % 10 == 0 {
            println!("Progress: {i}/100 iterations complete ({match_count} matches, {mismatch_count} mismatches)");
        }
    }

    println!("\n=== Final Results ===");
    println!("Total iterations: 100");
    println!("Matches: {} ({:.1}%)", match_count, f64::from(match_count));
    println!(
        "Mismatches: {} ({:.1}%)",
        mismatch_count,
        f64::from(mismatch_count)
    );

    println!("\n=== Hash Frequency Analysis ===");
    for (hash, count) in &hash_frequency {
        println!("{}... appeared {} times", &hash[..16], count);
    }

    if hash_frequency.len() > 1 {
        println!(
            "\n⚠️  NON-DETERMINISM STILL PRESENT: {} different hashes",
            hash_frequency.len()
        );
        panic!("Fix did not work - still non-deterministic");
    } else {
        println!("\n✅ DETERMINISTIC: All hashes identical - Issue #86 FIXED!");
    }

    assert_eq!(mismatch_count, 0, "All runs should match");
    assert_eq!(
        match_count, 100,
        "All 100 runs should produce identical hashes"
    );
}
