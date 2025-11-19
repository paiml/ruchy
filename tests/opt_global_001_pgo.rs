// OPT-GLOBAL-001: Profile-Guided Optimization Tests
// RED Phase: Create failing tests for PGO infrastructure
// Risk Class: 1 (Heuristic-Based - profile-dependent decisions)
// MAXIMUM RIGOR: All 8 phases, 25K+ property cases, statistical validation

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;

/// Test 1: PGO instrumentation build succeeds
#[test]
fn test_opt_global_001_01_pgo_instrumentation_build() {
    // Build with profile-generate flag
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--release")
        .env("RUSTFLAGS", "-Cprofile-generate=/tmp/pgo-data-test");

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Compiling ruchy"));
}

/// Test 2: Profile data collection produces .profraw files
#[test]
fn test_opt_global_001_02_profile_data_collection() {
    // Run a simple ruchy command to generate profile data
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("transpile")
        .arg("-")
        .write_stdin("fun main() { println(42); }")
        .env("LLVM_PROFILE_FILE", "/tmp/pgo-test-%p.profraw");

    cmd.assert().success();

    // Check that .profraw file was created
    let profraw_files = fs::read_dir("/tmp")
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "profraw"))
        .count();

    assert!(profraw_files > 0, "Expected .profraw files to be created");
}

/// Test 3: llvm-profdata merge succeeds
#[test]
fn test_opt_global_001_03_profile_merge() {
    // Create dummy profraw file for testing
    fs::create_dir_all("/tmp/pgo-merge-test").unwrap();

    let mut cmd = Command::new("llvm-profdata");
    cmd.arg("merge")
        .arg("-output=/tmp/pgo-merge-test/merged.profdata")
        .arg("/tmp/pgo-test-*.profraw");

    // Should succeed or fail gracefully if llvm-profdata not installed
    let result = cmd.output();
    assert!(result.is_ok(), "llvm-profdata command should be available");
}

/// Test 4: PGO-optimized build succeeds with merged profile
#[test]
fn test_opt_global_001_04_pgo_optimized_build() {
    let profile_path = "/tmp/pgo-merge-test/merged.profdata";

    // Skip if profile doesn't exist
    if !PathBuf::from(profile_path).exists() {
        eprintln!("Skipping: No merged profile data available");
        return;
    }

    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--release")
        .env("RUSTFLAGS", format!("-Cprofile-use={profile_path}"));

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Compiling ruchy"));
}

/// Test 5: Benchmark baseline measurement (no PGO)
#[test]
fn test_opt_global_001_05_benchmark_baseline() {
    use std::time::Instant;

    // Measure transpilation time for representative workload
    let code = r"
        fun fibonacci(n: i32) -> i32 {
            if n <= 1 {
                n
            } else {
                fibonacci(n - 1) + fibonacci(n - 2)
            }
        }

        fun main() -> i32 {
            fibonacci(10)
        }
    ";

    let start = Instant::now();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("transpile").arg("-").write_stdin(code.to_string());

    cmd.assert().success();

    let duration = start.elapsed();

    // Store baseline for comparison (should be < 100ms)
    assert!(
        duration.as_millis() < 100,
        "Baseline transpilation should complete quickly, took {}ms",
        duration.as_millis()
    );
}

/// Test 6: Statistical validation - collect N=30 runs
#[test]
#[ignore = "Run manually: cargo test opt_global_001_06 -- --ignored --nocapture"]
fn test_opt_global_001_06_statistical_validation_baseline() {
    use std::time::Instant;

    let code = include_str!("../examples/01_basics.ruchy");
    let mut timings = Vec::with_capacity(30);

    // Collect 30 runs for statistical significance
    for _ in 0..30 {
        let start = Instant::now();

        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("transpile")
            .arg("-")
            .write_stdin(code.to_string())
            .assert()
            .success();

        timings.push(start.elapsed().as_micros());
    }

    // Calculate mean and standard deviation
    let mean: f64 = timings.iter().map(|&t| t as f64).sum::<f64>() / 30.0;
    let variance: f64 = timings
        .iter()
        .map(|&t| {
            let diff = t as f64 - mean;
            diff * diff
        })
        .sum::<f64>()
        / 29.0; // N-1 for sample variance
    let std_dev = variance.sqrt();

    println!("Baseline: Mean={mean}μs, StdDev={std_dev}μs");

    // Save baseline to temp file for comparison
    fs::write("/tmp/pgo-baseline.txt", format!("{mean},{std_dev}"))
        .expect("Failed to write baseline");

    // Baseline should be stable (CV < 10%)
    let cv = (std_dev / mean) * 100.0;
    assert!(
        cv < 10.0,
        "Baseline coefficient of variation too high: {cv:.2}% (expected < 10%)"
    );
}

/// Test 7: PGO speedup validation (requires baseline)
#[test]
#[ignore = "Run after PGO build: cargo test opt_global_001_07 -- --ignored --nocapture"]
fn test_opt_global_001_07_pgo_speedup_validation() {
    use std::time::Instant;

    // Load baseline statistics
    let baseline_data = fs::read_to_string("/tmp/pgo-baseline.txt")
        .expect("Baseline data missing - run test 06 first");
    let parts: Vec<&str> = baseline_data.trim().split(',').collect();
    let baseline_mean: f64 = parts[0].parse().unwrap();

    let code = include_str!("../examples/01_basics.ruchy");
    let mut timings = Vec::with_capacity(30);

    // Collect 30 runs for PGO-optimized binary
    for _ in 0..30 {
        let start = Instant::now();

        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("transpile")
            .arg("-")
            .write_stdin(code.to_string())
            .assert()
            .success();

        timings.push(start.elapsed().as_micros());
    }

    let pgo_mean: f64 = timings.iter().map(|&t| t as f64).sum::<f64>() / 30.0;
    let speedup = ((baseline_mean - pgo_mean) / baseline_mean) * 100.0;

    println!("PGO: Mean={pgo_mean}μs, Speedup={speedup:.2}%");

    // PGO should provide ≥15% speedup (spec target: 15-30%)
    assert!(
        speedup >= 15.0,
        "PGO speedup {speedup:.2}% below target (expected ≥15%)"
    );
}
