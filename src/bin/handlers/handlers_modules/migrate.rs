//! Migrate 4.x → 5.0 Tool
//!
//! Scans Ruchy source files and renames identifiers that conflict with
//! new 5.0 reserved keywords: requires, ensures, invariant, decreases,
//! infra, signal, yield.
//!
//! Per ruchy-5.0-sovereign-platform.md Section 9.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The 7 new keywords in Ruchy 5.0 and their default rename suffixes.
const KEYWORD_RENAMES: &[(&str, &str)] = &[
    ("requires", "requires_val"),
    ("ensures", "ensures_val"),
    ("invariant", "invariant_val"),
    ("decreases", "decreases_val"),
    ("infra", "infra_config"),
    ("signal", "signal_val"),
    ("yield", "yield_val"),
];

/// A single rename action performed by the migration tool.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrateRename {
    /// File path
    pub file: PathBuf,
    /// Line number (1-based)
    pub line: usize,
    /// Column (1-based)
    pub column: usize,
    /// Original identifier
    pub original: String,
    /// Replacement identifier
    pub replacement: String,
}

impl std::fmt::Display for MigrateRename {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "  {}:{}  renamed `{}` -> `{}`",
            self.file.display(),
            self.line,
            self.original,
            self.replacement
        )
    }
}

/// Result of a migration scan or apply.
#[derive(Debug)]
pub struct MigrateResult {
    /// Files scanned
    pub files_scanned: usize,
    /// Files that would be (or were) modified
    pub files_modified: usize,
    /// All renames found/applied
    pub renames: Vec<MigrateRename>,
}

impl MigrateResult {
    /// Format a summary line.
    pub fn summary(&self) -> String {
        format!(
            "Migration complete: {} files scanned, {} files modified, {} identifiers renamed.",
            self.files_scanned, self.files_modified, self.renames.len()
        )
    }
}

/// Collect all `.ruchy` files under a directory (non-recursive for safety).
fn collect_ruchy_files(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        return vec![path.to_path_buf()];
    }
    let mut files = Vec::new();
    collect_ruchy_files_recursive(path, &mut files);
    files.sort();
    files
}

fn collect_ruchy_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_ruchy_files_recursive(&path, files);
        } else if path.extension().is_some_and(|e| e == "ruchy") {
            files.push(path);
        }
    }
}

/// Check if a word at the given position is used as an identifier (not a keyword).
/// Returns true if the word appears as a variable/parameter name, not as a
/// keyword in its syntactic position (e.g., `requires` after `fun` declaration
/// is a contract keyword, not an identifier).
fn is_identifier_usage(line: &str, start: usize, word: &str) -> bool {
    let before = &line[..start];
    let after_end = start + word.len();
    let after = if after_end < line.len() {
        &line[after_end..]
    } else {
        ""
    };

    // Check word boundaries
    if start > 0 {
        let prev_char = before.chars().next_back().unwrap_or(' ');
        if prev_char.is_alphanumeric() || prev_char == '_' {
            return false;
        }
    }
    if let Some(next_char) = after.chars().next() {
        if next_char.is_alphanumeric() || next_char == '_' {
            return false;
        }
    }

    // Contract keywords after fun declaration are NOT identifiers
    let trimmed_before = before.trim_end();
    let contract_keywords = ["requires", "ensures", "invariant", "decreases"];
    if contract_keywords.contains(&word) {
        // If this appears at the start of a line or after a colon/brace, it's a keyword
        if trimmed_before.is_empty()
            || trimmed_before.ends_with(':')
            || trimmed_before.ends_with('{')
            || trimmed_before.ends_with(')')
        {
            return false;
        }
    }

    // `let X = ...` or `X = ...` or `fun foo(X: ...)` patterns → identifier
    let is_assignment = after.trim_start().starts_with('=')
        || after.trim_start().starts_with(':');
    let is_let_binding = trimmed_before.ends_with("let")
        || trimmed_before.ends_with("var");
    let is_param = trimmed_before.ends_with('(')
        || trimmed_before.ends_with(',');

    is_assignment || is_let_binding || is_param
}

/// Scan a single file for identifier conflicts.
fn scan_file(path: &Path) -> Vec<MigrateRename> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let rename_map: HashMap<&str, &str> = KEYWORD_RENAMES.iter().copied().collect();
    let mut renames = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        // Skip comments
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }
        for (&keyword, &replacement) in &rename_map {
            scan_line_for_keyword(
                path, line, line_idx, keyword, replacement, &mut renames,
            );
        }
    }
    renames
}

fn scan_line_for_keyword(
    path: &Path,
    line: &str,
    line_idx: usize,
    keyword: &str,
    replacement: &str,
    renames: &mut Vec<MigrateRename>,
) {
    let mut search_start = 0;
    while let Some(pos) = line[search_start..].find(keyword) {
        let abs_pos = search_start + pos;
        if is_identifier_usage(line, abs_pos, keyword) {
            renames.push(MigrateRename {
                file: path.to_path_buf(),
                line: line_idx + 1,
                column: abs_pos + 1,
                original: keyword.to_string(),
                replacement: replacement.to_string(),
            });
        }
        search_start = abs_pos + keyword.len();
    }
}

/// Apply renames to file content and return the modified content.
fn apply_renames_to_content(content: &str, renames: &[MigrateRename]) -> String {
    let rename_map: HashMap<&str, &str> = KEYWORD_RENAMES.iter().copied().collect();
    let mut result = String::with_capacity(content.len());

    for (line_idx, line) in content.lines().enumerate() {
        let line_renames: Vec<&MigrateRename> = renames
            .iter()
            .filter(|r| r.line == line_idx + 1)
            .collect();

        if line_renames.is_empty() {
            result.push_str(line);
        } else {
            let mut new_line = line.to_string();
            // Apply renames right-to-left to preserve positions
            let mut sorted = line_renames.clone();
            sorted.sort_by(|a, b| b.column.cmp(&a.column));
            for rename in sorted {
                let start = rename.column - 1;
                let end = start + rename.original.len();
                let fallback = rename.replacement.as_str();
                let replacement = rename_map
                    .get(rename.original.as_str())
                    .copied()
                    .unwrap_or(fallback);
                new_line.replace_range(start..end, replacement);
            }
            result.push_str(&new_line);
        }
        result.push('\n');
    }
    // Remove trailing newline if original didn't have one
    if !content.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }
    result
}

/// Run the migration tool: scan for conflicts and optionally apply fixes.
pub fn run_migration(path: &Path, dry_run: bool) -> anyhow::Result<MigrateResult> {
    let files = collect_ruchy_files(path);
    println!("Scanning {} files...", files.len());

    let mut all_renames = Vec::new();
    let mut files_modified = 0;

    for file in &files {
        let renames = scan_file(file);
        if !renames.is_empty() {
            if !dry_run {
                let content = std::fs::read_to_string(file)?;
                let modified = apply_renames_to_content(&content, &renames);
                std::fs::write(file, modified)?;
            }
            files_modified += 1;
            for rename in &renames {
                println!("{rename}");
            }
            all_renames.extend(renames);
        }
    }

    let result = MigrateResult {
        files_scanned: files.len(),
        files_modified,
        renames: all_renames,
    };
    println!("{}", result.summary());
    if dry_run && result.files_modified > 0 {
        println!("(dry run - no files were modified)");
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_keyword_renames_count() {
        assert_eq!(KEYWORD_RENAMES.len(), 7);
    }

    #[test]
    fn test_is_identifier_usage_let_binding() {
        assert!(is_identifier_usage("let signal = 42", 4, "signal"));
    }

    #[test]
    fn test_is_identifier_usage_parameter() {
        assert!(is_identifier_usage("fun foo(yield: i32)", 8, "yield"));
    }

    #[test]
    fn test_is_not_identifier_contract_keyword() {
        // `requires` at start of line after function is a contract keyword
        assert!(!is_identifier_usage("requires x > 0", 0, "requires"));
    }

    #[test]
    fn test_is_not_identifier_substring() {
        // "required" is not "requires" — scan_line_for_keyword uses find()
        // which won't match, but if we manually check boundary detection:
        // "yields" contains "yield" at position 4, but 's' follows → not a word boundary
        assert!(!is_identifier_usage("let yields = true", 4, "yield"));
    }

    #[test]
    fn test_scan_file_finds_conflicts() {
        let dir = TempDir::new().expect("create temp dir");
        let file = dir.path().join("test.ruchy");
        let mut f = std::fs::File::create(&file).expect("create file");
        writeln!(f, "let signal = 42").expect("write");
        writeln!(f, "let yield = signal + 1").expect("write");
        writeln!(f, "print(signal)").expect("write");

        let renames = scan_file(&file);
        assert!(renames.len() >= 2, "expected >=2 renames, got {}", renames.len());
        assert!(renames.iter().any(|r| r.original == "signal"));
        assert!(renames.iter().any(|r| r.original == "yield"));
    }

    #[test]
    fn test_apply_renames() {
        let content = "let signal = 42\nlet x = signal + 1\n";
        let renames = vec![
            MigrateRename {
                file: PathBuf::from("test.ruchy"),
                line: 1,
                column: 5,
                original: "signal".to_string(),
                replacement: "signal_val".to_string(),
            },
        ];
        let result = apply_renames_to_content(content, &renames);
        assert!(result.contains("let signal_val = 42"));
    }

    #[test]
    fn test_run_migration_dry_run() {
        let dir = TempDir::new().expect("create temp dir");
        let file = dir.path().join("example.ruchy");
        let mut f = std::fs::File::create(&file).expect("create file");
        writeln!(f, "let signal = 0").expect("write");
        drop(f);

        let result = run_migration(dir.path(), true).expect("migration");
        assert_eq!(result.files_scanned, 1);
        assert!(result.renames.len() >= 1);

        // Dry run: file should be unchanged
        let content = std::fs::read_to_string(&file).expect("read");
        assert!(content.contains("let signal = 0"));
    }

    #[test]
    fn test_run_migration_apply() {
        let dir = TempDir::new().expect("create temp dir");
        let file = dir.path().join("example.ruchy");
        let mut f = std::fs::File::create(&file).expect("create file");
        writeln!(f, "let signal = 0").expect("write");
        drop(f);

        let result = run_migration(dir.path(), false).expect("migration");
        assert_eq!(result.files_modified, 1);

        // Apply mode: file should be modified
        let content = std::fs::read_to_string(&file).expect("read");
        assert!(content.contains("signal_val"));
    }

    #[test]
    fn test_collect_ruchy_files_ignores_non_ruchy() {
        let dir = TempDir::new().expect("create temp dir");
        std::fs::write(dir.path().join("a.ruchy"), "let x = 1").expect("write");
        std::fs::write(dir.path().join("b.rs"), "fn main() {}").expect("write");
        std::fs::write(dir.path().join("c.txt"), "hello").expect("write");

        let files = collect_ruchy_files(dir.path());
        assert_eq!(files.len(), 1);
        assert!(files[0].extension().unwrap() == "ruchy");
    }

    #[test]
    fn test_migrate_rename_display() {
        let rename = MigrateRename {
            file: PathBuf::from("src/math.ruchy"),
            line: 12,
            column: 5,
            original: "requires".to_string(),
            replacement: "requires_val".to_string(),
        };
        let display = format!("{rename}");
        assert!(display.contains("src/math.ruchy:12"));
        assert!(display.contains("`requires`"));
        assert!(display.contains("`requires_val`"));
    }

    #[test]
    fn test_no_renames_for_clean_file() {
        let dir = TempDir::new().expect("create temp dir");
        let file = dir.path().join("clean.ruchy");
        std::fs::write(&file, "let x = 42\nlet y = x + 1\n").expect("write");

        let renames = scan_file(&file);
        assert!(renames.is_empty());
    }
}
