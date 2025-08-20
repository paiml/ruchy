//! Performance validation harness for CI/CD
//! 
//! Validates that performance targets are met

use std::process::Command;
use std::fs;
use std::time::Instant;

const PERFORMANCE_TARGETS: &str = r#"
script_startup: 50      # ms - time to run simple -e command
repl_response: 100      # ms - time for REPL to evaluate expression
binary_size: 52428800   # 50MB max binary size
test_pass_rate: 0.90    # 90% minimum test pass rate
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Validating Ruchy Performance Targets...\n");
    
    let mut failures = Vec::new();
    
    // Test 1: Script startup time
    println!("Testing script startup time...");
    let start = Instant::now();
    let output = Command::new("target/release/ruchy")
        .args(&["-e", "42"])
        .output()?;
    let startup_ms = start.elapsed().as_millis() as f64;
    
    if !output.status.success() {
        failures.push("Script execution failed".to_string());
    } else if startup_ms > 50.0 {
        failures.push(format!(
            "Startup time {:.2}ms exceeds target 50ms", 
            startup_ms
        ));
    } else {
        println!("  ✅ Startup time: {:.2}ms", startup_ms);
    }
    
    // Test 2: Binary size
    println!("Checking binary size...");
    let metadata = fs::metadata("target/release/ruchy")?;
    let binary_size = metadata.len();
    let size_mb = binary_size as f64 / 1_048_576.0;
    
    if binary_size > 52_428_800 {
        failures.push(format!(
            "Binary size {:.2}MB exceeds target 50MB", 
            size_mb
        ));
    } else {
        println!("  ✅ Binary size: {:.2}MB", size_mb);
    }
    
    // Test 3: One-liner test pass rate
    println!("Running one-liner tests...");
    let output = Command::new("./tests/oneliner/suite.sh")
        .output()?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    
    // Parse test results from output
    if let Some(line) = output_str.lines().find(|l| l.contains("Results:")) {
        // Format: "Results: X passed, Y failed"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            let passed: f64 = parts[1].parse().unwrap_or(0.0);
            let failed: f64 = parts[3].parse().unwrap_or(0.0);
            let total = passed + failed;
            let pass_rate = if total > 0.0 { passed / total } else { 0.0 };
            
            if pass_rate < 0.90 {
                failures.push(format!(
                    "Test pass rate {:.1}% below target 90%", 
                    pass_rate * 100.0
                ));
            } else {
                println!("  ✅ Test pass rate: {:.1}%", pass_rate * 100.0);
            }
        }
    }
    
    // Test 4: REPL response time
    println!("Testing REPL response time...");
    let start = Instant::now();
    let mut child = Command::new("target/release/ruchy")
        .arg("repl")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;
    
    // Send a simple expression
    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        writeln!(stdin, "2 + 2")?;
        writeln!(stdin, ":quit")?;
    }
    
    let output = child.wait_with_output()?;
    let repl_ms = start.elapsed().as_millis() as f64;
    
    if !output.status.success() {
        failures.push("REPL execution failed".to_string());
    } else if repl_ms > 100.0 {
        failures.push(format!(
            "REPL response time {:.2}ms exceeds target 100ms", 
            repl_ms
        ));
    } else {
        println!("  ✅ REPL response: {:.2}ms", repl_ms);
    }
    
    // Report results
    println!("\n" + &"=".repeat(50));
    if failures.is_empty() {
        println!("✅ All performance targets met!");
        Ok(())
    } else {
        println!("❌ Performance validation failed:");
        for failure in &failures {
            eprintln!("  • {}", failure);
        }
        std::process::exit(1);
    }
}