// STDLIB-005 Example: Find Duplicate Files
//
// Demonstrates walk_parallel() + compute_hash() composition
// Use case: Find duplicate files in a directory (like rclean)
// Perfect composable API design

use std::fs;
use tempfile::TempDir;

fn main() {
    println!("🔍 Duplicate File Finder (Ruchy + rclean pattern)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Create temp directory with duplicate files
    let temp = TempDir::new().expect("Failed to create temp dir");

    // Create some duplicate files
    fs::write(temp.path().join("doc1.txt"), "Same content").unwrap();
    fs::write(temp.path().join("doc2.txt"), "Same content").unwrap();
    fs::write(temp.path().join("doc3.txt"), "Same content").unwrap();
    fs::write(temp.path().join("unique1.txt"), "Unique content A").unwrap();
    fs::write(temp.path().join("unique2.txt"), "Unique content B").unwrap();

    println!("Created test directory: {}", temp.path().display());
    println!("  3 duplicate files (doc1.txt, doc2.txt, doc3.txt)");
    println!("  2 unique files (unique1.txt, unique2.txt)");
    println!();

    // Ruchy code to find duplicates
    let code = format!(
        r#"
        # Step 1: Walk directory in parallel and compute hashes
        let dir = "{}"
        println("Scanning directory: {{}}", dir)

        let entries = walk_parallel(dir)
        let files = entries.filter(fn(e) {{ e.is_file }})

        println("Found {{}} files", files.len())
        println()

        # Step 2: Compute hashes for all files
        let with_hashes = files.map(fn(e) {{
            let hash = compute_hash(e.path)
            Object.merge(e, {{ hash: hash }})
        }})

        # Step 3: Group by hash to find duplicates
        # For now, we'll do this manually since we don't have group_by yet
        println("Files by hash:")
        let _ = with_hashes.map(fn(f) {{
            println("  {{}} → {{}}", f.name, f.hash)
        }})
        println()

        # Step 4: Find files with matching hashes
        # NOTE: This is a simplified version - proper duplicate detection would use
        # group_by() or mutable closure capture (not yet supported in Ruchy)

        # For demonstration, just show first duplicate group
        let first_hash = with_hashes.first().hash
        let matches = with_hashes.filter(fn(f) {{
            f.hash == first_hash
        }})

        if matches.len() > 1 {{
            println("🔍 Found {{}} files with hash {{}}", matches.len(), first_hash)
            let _ = matches.map(fn(m) {{
                println("    - {{}}", m.name)
            }})
            println()
        }}

        println("✅ Duplicate scan complete (showing first duplicate group only)!")
    "#,
        temp.path().display()
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

    println!();
    println!("💡 This demonstrates the perfect composable API:");
    println!("   walk_parallel() does parallel I/O");
    println!("   compute_hash() enables duplicate detection");
    println!("   Users compose with .filter(), .map(), array methods");
}
