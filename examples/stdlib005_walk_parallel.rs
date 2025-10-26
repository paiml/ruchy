// STDLIB-005 Example: Parallel Directory Walking
//
// Demonstrates walk_parallel() for fast directory traversal using rayon
// Use case: Find all Rust files in a large codebase

use std::env;

fn main() {
    // Get directory from args or use current directory
    let dir = env::args().nth(1).unwrap_or_else(|| ".".to_string());

    println!("🔍 Walking directory: {}", dir);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Ruchy code to execute
    let code = format!(
        r#"
        let entries = walk_parallel("{}")
        let rust_files = entries.filter(fn(e) {{
            e.is_file && e.name.ends_with(".rs")
        }})

        println("Found {{}} Rust files", rust_files.len())

        # Show first 10 files using map (each doesn't work with mutation)
        let sample = rust_files.slice(0, 10)
        let _ = sample.map(fn(f) {{
            println("  - {{}} ({{}} bytes)", f.name, f.size)
        }})

        if rust_files.len() > 10 {{
            println("  ... and {{}} more", rust_files.len() - 10)
        }}

        rust_files.len()
    "#,
        dir
    );

    // Run via ruchy interpreter
    let output = std::process::Command::new("cargo")
        .args(&["run", "--release", "--bin", "ruchy", "--", "-e", &code])
        .output()
        .expect("Failed to execute ruchy");

    if output.status.success() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }
}
