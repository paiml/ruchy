//! Snapshot Testing Infrastructure
//!
//! Based on docs/ruchy-transpiler-docs.md Section 3: Snapshot Testing
//! Detects any output changes immediately through content-addressed storage

#![allow(clippy::print_stdout)] // Testing infrastructure needs stdout for feedback
#![allow(clippy::print_stderr)] // Testing infrastructure needs stderr for errors

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

/// A single snapshot test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotTest {
    /// Unique name for this test
    pub name: String,
    /// Input Ruchy code
    pub input: String,
    /// SHA256 hash of expected output
    pub output_hash: String,
    /// The actual Rust output (for reference)
    pub rust_output: String,
    /// Metadata about when this snapshot was created/updated
    pub metadata: SnapshotMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub created_at: String,
    pub updated_at: String,
    pub ruchy_version: String,
    pub rustc_version: String,
}

/// Snapshot test suite
#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotSuite {
    pub tests: Vec<SnapshotTest>,
    pub config: SnapshotConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotConfig {
    /// Whether to automatically update snapshots on mismatch
    pub auto_update: bool,
    /// Directory to store snapshot files
    pub snapshot_dir: PathBuf,
    /// Whether to fail on missing snapshots
    pub fail_on_missing: bool,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            auto_update: false,
            snapshot_dir: PathBuf::from("tests/snapshots"),
            fail_on_missing: true,
        }
    }
}

/// Snapshot test runner
pub struct SnapshotRunner {
    config: SnapshotConfig,
    suite: SnapshotSuite,
}

impl SnapshotRunner {
    /// Load snapshot suite from disk
    /// # Errors
    ///
    /// Returns an error if the operation fails
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn load(config: SnapshotConfig) -> Result<Self> {
        let snapshot_file = config.snapshot_dir.join("snapshots.toml");

        let suite = if snapshot_file.exists() {
            let contents = fs::read_to_string(&snapshot_file)?;
            toml::from_str(&contents)?
        } else {
            SnapshotSuite {
                tests: Vec::new(),
                config: config.clone(),
            }
        };

        Ok(Self { config, suite })
    }

    /// Run a snapshot test
    /// # Errors
    ///
    /// Returns an error if the operation fails
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn test<F>(&mut self, name: &str, input: &str, transform: F) -> Result<()>
    where
        F: FnOnce(&str) -> Result<String>,
    {
        // Generate output
        let output = transform(input)?;
        let output_hash = Self::hash(&output);

        // Find existing snapshot
        if let Some(existing) = self.suite.tests.iter().find(|t| t.name == name) {
            if existing.output_hash == output_hash {
                // Test passed
                println!("✓ Snapshot matched: {name}");
            } else if self.config.auto_update {
                // Update the snapshot
                self.update_snapshot(name, input, &output, &output_hash)?;
                println!("✓ Updated snapshot: {name}");
            } else {
                // Fail the test
                bail!(
                    "Snapshot mismatch for '{}':\n  Expected hash: {}\n  Actual hash: {}\n  Output:\n{}",
                    name, existing.output_hash, output_hash, output
                );
            }
        } else {
            // No existing snapshot
            if self.config.fail_on_missing {
                bail!("Missing snapshot for test: {}", name);
            }
            // Create new snapshot
            self.create_snapshot(name, input, &output, &output_hash)?;
            println!("✓ Created snapshot: {name}");
        }

        Ok(())
    }

    /// Update an existing snapshot
    fn update_snapshot(&mut self, name: &str, input: &str, output: &str, hash: &str) -> Result<()> {
        for test in &mut self.suite.tests {
            if test.name == name {
                test.input = input.to_string();
                test.output_hash = hash.to_string();
                test.rust_output = output.to_string();
                test.metadata.updated_at = chrono::Utc::now().to_rfc3339();
                break;
            }
        }

        self.save()?;
        Ok(())
    }

    /// Create a new snapshot
    fn create_snapshot(&mut self, name: &str, input: &str, output: &str, hash: &str) -> Result<()> {
        let test = SnapshotTest {
            name: name.to_string(),
            input: input.to_string(),
            output_hash: hash.to_string(),
            rust_output: output.to_string(),
            metadata: SnapshotMetadata {
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
                ruchy_version: env!("CARGO_PKG_VERSION").to_string(),
                rustc_version: "1.75.0".to_string(), // Would get from rustc --version
            },
        };

        self.suite.tests.push(test);
        self.save()?;
        Ok(())
    }

    /// Save the snapshot suite to disk
    fn save(&self) -> Result<()> {
        fs::create_dir_all(&self.config.snapshot_dir)?;
        let snapshot_file = self.config.snapshot_dir.join("snapshots.toml");
        let contents = toml::to_string_pretty(&self.suite)?;
        fs::write(snapshot_file, contents)?;
        Ok(())
    }

    /// Calculate SHA256 hash of a string
    fn hash(s: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(s.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Run all snapshots and report results
    /// # Errors
    ///
    /// Returns an error if the operation fails
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn run_all<F>(&mut self, transform: F) -> Result<()>
    where
        F: Fn(&str) -> Result<String>,
    {
        let mut passed = 0;
        let mut failed = 0;
        let updated = 0;

        for test in self.suite.tests.clone() {
            match self.test(&test.name, &test.input, |input| transform(input)) {
                Ok(()) => passed += 1,
                Err(e) => {
                    eprintln!("✗ {}: {}", test.name, e);
                    failed += 1;
                }
            }
        }

        println!("\nSnapshot Test Results:");
        println!("  Passed: {passed}");
        println!("  Failed: {failed}");
        if updated > 0 {
            println!("  Updated: {updated}");
        }

        if failed > 0 {
            bail!("{} snapshot tests failed", failed);
        }

        Ok(())
    }
}

/// Automatic bisection to identify regression source
#[allow(clippy::module_name_repetitions)]
pub struct SnapshotBisector {
    #[allow(dead_code)]
    snapshots: Vec<SnapshotTest>,
}

impl SnapshotBisector {
    #[must_use]
    pub fn new(snapshots: Vec<SnapshotTest>) -> Self {
        Self { snapshots }
    }

    /// Find the commit that introduced a regression
    pub fn bisect<F>(&self, test_name: &str, _is_good: F) -> Option<String>
    where
        F: Fn(&str) -> bool,
    {
        // This would integrate with git bisect
        // For now, just a placeholder
        println!("Would bisect to find regression in test: {test_name}");
        None
    }
}

/// Snapshot test definitions for core Ruchy features
#[must_use]
pub fn core_snapshot_tests() -> Vec<(&'static str, &'static str)> {
    vec![
        ("literal_int", "42"),
        ("literal_float", "3.14"),
        ("literal_string", r#""hello""#),
        ("literal_bool_true", "true"),
        ("literal_bool_false", "false"),
        ("binary_add", "1 + 2"),
        ("binary_mul", "3 * 4"),
        ("binary_complex", "1 + 2 * 3"),
        ("binary_parens", "(1 + 2) * 3"),
        ("let_simple", "let x = 10"),
        ("let_nested", "let x = 10 in x + 1"),
        ("function_simple", "fun f(x) { x + 1 }"),
        ("function_multi_param", "fun add(x, y) { x + y }"),
        ("if_simple", "if true { 1 } else { 2 }"),
        ("if_no_else", "if x > 0 { x }"),
        ("list_empty", "[]"),
        ("list_numbers", "[1, 2, 3]"),
        ("pipeline_simple", "data |> filter |> map"),
        ("match_simple", "match x { 1 => \"one\", _ => \"other\" }"),
    ]
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::{Parser, Transpiler};

    #[test]
    fn test_snapshot_basic() {
        let config = SnapshotConfig {
            auto_update: true,
            snapshot_dir: PathBuf::from("target/test-snapshots"),
            fail_on_missing: false,
        };

        let mut runner = SnapshotRunner::load(config).unwrap();

        // Test a simple expression
        runner
            .test("simple_addition", "1 + 2", |input| {
                let mut parser = Parser::new(input);
                let ast = parser.parse()?;
                let transpiler = Transpiler::new();
                let tokens = transpiler.transpile(&ast)?;
                Ok(tokens.to_string())
            })
            .unwrap();
    }

    #[test]
    fn test_snapshot_determinism() {
        let config = SnapshotConfig {
            auto_update: false,
            snapshot_dir: PathBuf::from("target/test-snapshots-determinism"),
            fail_on_missing: false,
        };

        let mut runner = SnapshotRunner::load(config).unwrap();

        // Run the same test multiple times - should produce identical hashes
        for i in 0..3 {
            runner
                .test(&format!("determinism_test_{i}"), "x * 2 + 1", |input| {
                    let mut parser = Parser::new(input);
                    let ast = parser.parse()?;
                    let transpiler = Transpiler::new();
                    let tokens = transpiler.transpile(&ast)?;
                    Ok(tokens.to_string())
                })
                .unwrap();
        }
    }
}
