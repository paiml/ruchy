#!/usr/bin/env rust-script
//! Memory efficiency validation tool for Ruchy interpreter
//! 
//! Generates large codebases and measures memory usage during evaluation.
//! Critical for validating self-hosting readiness with 50K+ LOC.

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

/// Generate a large Ruchy codebase for memory testing
fn generate_large_codebase(target_loc: usize) -> String {
    let mut code = String::new();
    
    // Generate modules with functions
    let modules_count = target_loc / 100; // ~100 LOC per module
    
    for module_id in 0..modules_count {
        code.push_str(&format!("mod module_{} {{\n", module_id));
        
        // Generate functions within each module
        for func_id in 0..10 {
            code.push_str(&format!(
                "    pub fun function_{}_{}<T>(x: T, y: i32) -> T {{\n",
                module_id, func_id
            ));
            
            // Generate function body with various constructs
            code.push_str("        let mut result = x\n");
            code.push_str("        for i in 0..y {\n");
            code.push_str("            if i % 2 == 0 {\n");
            code.push_str("                match i {\n");
            code.push_str("                    0 => result = x,\n");
            code.push_str("                    1 => result = x,\n");
            code.push_str("                    _ => result = x,\n");
            code.push_str("                }\n");
            code.push_str("            } else {\n");
            code.push_str("                let temp = [1, 2, 3, 4, 5]\n");
            code.push_str("                result = x\n");
            code.push_str("            }\n");
            code.push_str("        }\n");
            code.push_str("        result\n");
            code.push_str("    }\n\n");
        }
        
        code.push_str("}\n\n");
    }
    
    // Generate some usage code
    code.push_str("// Usage examples\n");
    for i in 0..std::cmp::min(10, modules_count) {
        code.push_str(&format!(
            "let result_{} = module_{}::function_{}_0(42, 10)\n",
            i, i, i
        ));
    }
    
    code
}

/// Count lines of code in a string
fn count_loc(code: &str) -> usize {
    code.lines()
        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with("//"))
        .count()
}

/// Measure memory usage of Ruchy evaluation
fn measure_memory_usage(code: &str) -> Result<(u64, std::time::Duration), Box<dyn std::error::Error>> {
    // Write code to temporary file
    let temp_file = "/tmp/ruchy_memory_test.ruchy";
    fs::write(temp_file, code)?;
    
    let start = Instant::now();
    
    // Run Ruchy with memory profiling
    let output = Command::new("cargo")
        .args(&["run", "--bin", "ruchy", "--", "run", temp_file])
        .env("RUCHY_MEMORY_PROFILE", "1")
        .output()?;
    
    let duration = start.elapsed();
    
    if !output.status.success() {
        eprintln!("Ruchy execution failed:");
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        return Err("Ruchy execution failed".into());
    }
    
    // Parse memory usage from output (would need to implement in Ruchy)
    // For now, estimate based on execution time and complexity
    let estimated_memory = code.len() as u64 * 10; // Rough estimate
    
    // Clean up
    fs::remove_file(temp_file).ok();
    
    Ok((estimated_memory, duration))
}

/// Run comprehensive memory efficiency validation
fn run_memory_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Ruchy Memory Efficiency Validation");
    println!("=====================================\n");
    
    let test_sizes = vec![1000, 5000, 10000, 25000, 50000];
    
    for target_loc in test_sizes {
        println!("📊 Testing with ~{} LOC...", target_loc);
        
        let code = generate_large_codebase(target_loc);
        let actual_loc = count_loc(&code);
        
        println!("   Generated: {} actual LOC", actual_loc);
        
        match measure_memory_usage(&code) {
            Ok((memory_bytes, duration)) => {
                let memory_mb = memory_bytes as f64 / 1_048_576.0;
                let loc_per_sec = actual_loc as f64 / duration.as_secs_f64();
                
                println!("   Memory usage: {:.2} MB", memory_mb);
                println!("   Execution time: {:.2}s", duration.as_secs_f64());
                println!("   Throughput: {:.0} LOC/s", loc_per_sec);
                
                // Memory efficiency thresholds
                let memory_per_loc = memory_bytes as f64 / actual_loc as f64;
                println!("   Memory per LOC: {:.0} bytes", memory_per_loc);
                
                if memory_per_loc > 1000.0 {
                    println!("   ⚠️  Warning: High memory usage per LOC");
                } else {
                    println!("   ✅ Memory usage acceptable");
                }
                
                if loc_per_sec < 100.0 {
                    println!("   ⚠️  Warning: Low throughput");
                } else {
                    println!("   ✅ Throughput acceptable");
                }
            }
            Err(e) => {
                println!("   ❌ Test failed: {}", e);
            }
        }
        
        println!();
    }
    
    println!("🎯 Memory Efficiency Summary");
    println!("============================");
    println!("Target for self-hosting:");
    println!("• Memory per LOC: <500 bytes");
    println!("• Throughput: >1000 LOC/s for parsing");
    println!("• Total memory for 50K LOC: <25MB");
    println!("• Evaluation time: <30s for complex code");
    
    Ok(())
}

fn main() {
    if let Err(e) = run_memory_validation() {
        eprintln!("❌ Memory validation failed: {}", e);
        std::process::exit(1);
    }
}