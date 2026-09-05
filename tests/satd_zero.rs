use regex::Regex;
use std::fs;
use walkdir::WalkDir;

/// Test PMAT-099: Verify zero SATD violations in src/**/*.rs
/// Walks source tree and asserts:
/// 1. No lines match (TODO|FIXME|HACK|XXX): patterns
/// 2. No PARSER-XXX placeholder comments anywhere in src/
#[test]
fn test_pmat_099_satd_markers_forbidden() {
    let satd_regex = Regex::new(r"(TODO|FIXME|HACK|XXX):").expect("Invalid regex");
    let parser_placeholder_regex = Regex::new(r"PARSER-XXX").expect("Invalid regex");

    let mut violations = Vec::new();
    let mut parser_violations = Vec::new();

    // Walk src/ directory, excluding test files and test directories
    for entry in WalkDir::new("src")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            // Skip directories and test files
            path.is_file()
                && path.extension().map(|ext| ext == "rs").unwrap_or(false)
                && !path.to_string_lossy().contains("_tests.rs")
                && !path.to_string_lossy().contains("/tests/")
        })
    {
        let path = entry.path();
        if let Ok(content) = fs::read_to_string(path) {
            for (line_no, line) in content.lines().enumerate() {
                let line_num = line_no + 1;

                // Check for SATD markers (TODO, FIXME, HACK, XXX)
                if satd_regex.is_match(line) {
                    violations.push(format!("{}:{}: {}", path.display(), line_num, line.trim()));
                }

                // Check for PARSER-XXX placeholders
                if parser_placeholder_regex.is_match(line) {
                    parser_violations.push(format!(
                        "{}:{}: {}",
                        path.display(),
                        line_num,
                        line.trim()
                    ));
                }
            }
        }
    }

    // Print violations for debugging
    if !violations.is_empty() {
        eprintln!("\n=== SATD Marker Violations (TODO|FIXME|HACK|XXX:) ===");
        for violation in &violations {
            eprintln!("{}", violation);
        }
    }

    if !parser_violations.is_empty() {
        eprintln!("\n=== PARSER-XXX Placeholder Violations ===");
        for violation in &parser_violations {
            eprintln!("{}", violation);
        }
    }

    // Assert no violations
    assert!(
        violations.is_empty() && parser_violations.is_empty(),
        "Found {} SATD marker violations and {} PARSER-XXX violations",
        violations.len(),
        parser_violations.len()
    );
}
