//! Sister project integration tests for ruchy-book
//!
//! Tests all examples from the Ruchy book to ensure compatibility

use std::path::Path;
use std::process::Command;

fn test_ruchy_file(path: &Path) -> Result<String, String> {
    let output = Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "run", path.to_str().unwrap()])
        .output()
        .map_err(|e| format!("Failed to execute: {e}"))?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[test]
#[ignore] // Run with --ignored flag
fn test_all_book_examples() {
    let book_dir = Path::new("../ruchy-book");
    if !book_dir.exists() {
        eprintln!("Skipping: ruchy-book not found");
        return;
    }
    
    let mut passed = 0;
    let mut failed = 0;
    let mut errors = Vec::new();
    
    // Find all .ruchy files
    for entry in walkdir::WalkDir::new(book_dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "ruchy"))
    {
        let path = entry.path();
        let relative_path = path.strip_prefix(book_dir).unwrap_or(path);
        
        match test_ruchy_file(path) {
            Ok(_) => {
                passed += 1;
                println!("✅ {}", relative_path.display());
            }
            Err(err) => {
                failed += 1;
                println!("❌ {}", relative_path.display());
                errors.push((relative_path.to_path_buf(), err));
            }
        }
    }
    
    println!("\n📊 Sister Project Test Results:");
    println!("   Passed: {passed}");
    println!("   Failed: {failed}");
    println!("   Total:  {}", passed + failed);
    println!("   Success Rate: {:.1}%", (f64::from(passed) / f64::from(passed + failed)) * 100.0);
    
    if !errors.is_empty() {
        println!("\n❌ Failed files:");
        for (path, _err) in &errors[..errors.len().min(10)] {
            println!("   - {}", path.display());
        }
        if errors.len() > 10 {
            println!("   ... and {} more", errors.len() - 10);
        }
    }
    
    assert!(failed == 0, "Sister project tests failed: {}/{}", failed, passed + failed);
}