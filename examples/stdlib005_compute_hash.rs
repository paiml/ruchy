// STDLIB-005 Example: File Hashing with MD5
//
// Demonstrates compute_hash() for file integrity verification
// Use case: Generate checksums for files

use std::fs;
use tempfile::TempDir;

fn main() {
    println!("🔐 File Hashing Example");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Create temp directory with test files
    let temp = TempDir::new().expect("Failed to create temp dir");
    let file1 = temp.path().join("file1.txt");
    let file2 = temp.path().join("file2.txt");
    let file3 = temp.path().join("file3.txt");

    fs::write(&file1, "Hello World").unwrap();
    fs::write(&file2, "Hello World").unwrap(); // Identical to file1
    fs::write(&file3, "Different content").unwrap();

    println!("Created 3 test files:");
    println!("  file1.txt: 'Hello World'");
    println!("  file2.txt: 'Hello World' (identical)");
    println!("  file3.txt: 'Different content'");
    println!();

    // Ruchy code to execute
    let code = format!(
        r#"
        let hash1 = compute_hash("{}")
        let hash2 = compute_hash("{}")
        let hash3 = compute_hash("{}")

        println("Hash results:")
        println("  file1.txt: {{}}", hash1)
        println("  file2.txt: {{}}", hash2)
        println("  file3.txt: {{}}", hash3)
        println()

        if hash1 == hash2 {{
            println("✅ file1 and file2 are identical (same hash)")
        }} else {{
            println("❌ file1 and file2 differ")
        }}

        if hash1 == hash3 {{
            println("❌ file1 and file3 are identical")
        }} else {{
            println("✅ file1 and file3 differ (different hash)")
        }}

        # Known MD5 of "Hello World"
        let expected = "b10a8db164e0754105b7a99be72e3fe5"
        if hash1 == expected {{
            println("✅ Hash matches known MD5 for 'Hello World'")
        }}
    "#,
        file1.display(),
        file2.display(),
        file3.display()
    );

    // Run via ruchy interpreter
    let output = std::process::Command::new("cargo")
        .args(["run", "--release", "--bin", "ruchy", "--", "-e", &code])
        .output()
        .expect("Failed to execute ruchy");

    if output.status.success() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }
}
