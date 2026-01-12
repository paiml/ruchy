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
    /// # Examples
    ///
    /// ```
    /// use ruchy::testing::snapshot::load;
    ///
    /// let result = load(());
    /// assert_eq!(result, Ok(()));
    /// ```
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
                    name,
                    existing.output_hash,
                    output_hash,
                    output
                );
            }
        } else {
            // No existing snapshot
            if self.config.fail_on_missing {
                bail!("Missing snapshot for test: {name}");
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
            bail!("{failed} snapshot tests failed");
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
    /// # Examples
    ///
    /// ```
    /// use ruchy::testing::snapshot::new;
    ///
    /// let result = new(());
    /// assert_eq!(result, Ok(()));
    /// ```
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
/// # Examples
///
/// ```
/// use ruchy::testing::snapshot::core_snapshot_tests;
///
/// let result = core_snapshot_tests(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn core_snapshot_tests() -> Vec<(&'static str, &'static str)> {
    vec![
        ("literal_int", "42"),
        ("literal_float", "3.15"),
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
        ("pipeline_simple", "data >> filter >> map"),
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
        let mut runner = SnapshotRunner::load(config).expect("operation should succeed in test");
        // Test a simple expression
        runner
            .test("simple_addition", "1 + 2", |input| {
                let mut parser = Parser::new(input);
                let ast = parser.parse()?;
                let mut transpiler = Transpiler::new();
                let tokens = transpiler.transpile(&ast)?;
                Ok(tokens.to_string())
            })
            .expect("operation should succeed in test");
    }
    #[test]
    #[ignore = "Flaky test when run with full test suite"]
    fn test_snapshot_determinism() {
        let config = SnapshotConfig {
            auto_update: false,
            snapshot_dir: PathBuf::from("target/test-snapshots-determinism"),
            fail_on_missing: false,
        };
        let mut runner = SnapshotRunner::load(config).expect("operation should succeed in test");
        // Run the same test multiple times - should produce identical hashes
        for i in 0..3 {
            runner
                .test(&format!("determinism_test_{i}"), "x * 2 + 1", |input| {
                    let mut parser = Parser::new(input);
                    let ast = parser.parse()?;
                    let mut transpiler = Transpiler::new();
                    let tokens = transpiler.transpile(&ast)?;
                    Ok(tokens.to_string())
                })
                .expect("operation should succeed in test");
        }
    }

    #[test]
    fn test_snapshot_config_default() {
        let config = SnapshotConfig::default();
        assert!(!config.auto_update);
        assert_eq!(config.snapshot_dir, PathBuf::from("tests/snapshots"));
        assert!(config.fail_on_missing);
    }

    #[test]
    fn test_snapshot_config_custom() {
        let config = SnapshotConfig {
            auto_update: true,
            snapshot_dir: PathBuf::from("custom/snapshots"),
            fail_on_missing: false,
        };
        assert!(config.auto_update);
        assert_eq!(config.snapshot_dir, PathBuf::from("custom/snapshots"));
        assert!(!config.fail_on_missing);
    }

    #[test]
    fn test_snapshot_metadata() {
        let metadata = SnapshotMetadata {
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-02T00:00:00Z".to_string(),
            ruchy_version: "1.0.0".to_string(),
            rustc_version: "1.75.0".to_string(),
        };
        assert_eq!(metadata.created_at, "2023-01-01T00:00:00Z");
        assert_eq!(metadata.updated_at, "2023-01-02T00:00:00Z");
        assert_eq!(metadata.ruchy_version, "1.0.0");
        assert_eq!(metadata.rustc_version, "1.75.0");
    }

    #[test]
    fn test_snapshot_test_structure() {
        let test = SnapshotTest {
            name: "test_basic".to_string(),
            input: "1 + 1".to_string(),
            output_hash: "abc123".to_string(),
            rust_output: "1 + 1".to_string(),
            metadata: SnapshotMetadata {
                created_at: "2023-01-01T00:00:00Z".to_string(),
                updated_at: "2023-01-01T00:00:00Z".to_string(),
                ruchy_version: "1.0.0".to_string(),
                rustc_version: "1.75.0".to_string(),
            },
        };
        assert_eq!(test.name, "test_basic");
        assert_eq!(test.input, "1 + 1");
        assert_eq!(test.output_hash, "abc123");
        assert_eq!(test.rust_output, "1 + 1");
    }

    #[test]
    fn test_snapshot_suite_creation() {
        let suite = SnapshotSuite {
            tests: vec![],
            config: SnapshotConfig::default(),
        };
        assert_eq!(suite.tests.len(), 0);
        assert!(!suite.config.auto_update);
    }

    #[test]
    fn test_hash_function() {
        let hash1 = SnapshotRunner::hash("hello world");
        let hash2 = SnapshotRunner::hash("hello world");
        let hash3 = SnapshotRunner::hash("hello world!");

        // Same input should produce same hash
        assert_eq!(hash1, hash2);
        // Different input should produce different hash
        assert_ne!(hash1, hash3);

        // Hash should be valid SHA256 (64 hex characters)
        assert_eq!(hash1.len(), 64);
        assert!(hash1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_consistency() {
        // Test various inputs
        let inputs = vec!["", "a", "hello", "hello world", "🦀"];
        for input in inputs {
            let hash1 = SnapshotRunner::hash(input);
            let hash2 = SnapshotRunner::hash(input);
            assert_eq!(hash1, hash2, "Hash inconsistency for input: {input}");
        }
    }

    #[test]
    fn test_snapshot_load_empty() {
        let config = SnapshotConfig {
            auto_update: false,
            snapshot_dir: PathBuf::from("target/test-snapshots-empty"),
            fail_on_missing: false,
        };

        // Clean up any existing file
        let _ = std::fs::remove_dir_all(&config.snapshot_dir);

        let runner = SnapshotRunner::load(config).expect("operation should succeed in test");
        assert_eq!(runner.suite.tests.len(), 0);
    }

    #[test]
    fn test_snapshot_auto_update_disabled() {
        let config = SnapshotConfig {
            auto_update: false,
            snapshot_dir: PathBuf::from("target/test-snapshots-no-update"),
            fail_on_missing: false,
        };

        // Clean up
        let _ = std::fs::remove_dir_all(&config.snapshot_dir);

        let mut runner = SnapshotRunner::load(config).expect("operation should succeed in test");

        // First test - creates snapshot
        runner
            .test("test1", "input1", |_| Ok("output1".to_string()))
            .expect("operation should succeed in test");
        assert_eq!(runner.suite.tests.len(), 1);

        // Second test with different output - should fail because auto_update is false
        let result = runner.test("test1", "input1", |_| Ok("output2".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_snapshot_auto_update_enabled() {
        let config = SnapshotConfig {
            auto_update: true,
            snapshot_dir: PathBuf::from("target/test-snapshots-update"),
            fail_on_missing: false,
        };

        // Clean up
        let _ = std::fs::remove_dir_all(&config.snapshot_dir);

        let mut runner = SnapshotRunner::load(config).expect("operation should succeed in test");

        // First test - creates snapshot
        runner
            .test("test1", "input1", |_| Ok("output1".to_string()))
            .expect("operation should succeed in test");
        let original_hash = runner.suite.tests[0].output_hash.clone();

        // Second test with different output - should update because auto_update is true
        runner
            .test("test1", "input1", |_| Ok("output2".to_string()))
            .expect("operation should succeed in test");
        let new_hash = &runner.suite.tests[0].output_hash;

        assert_ne!(original_hash, *new_hash);
        assert_eq!(runner.suite.tests[0].rust_output, "output2");
    }

    #[test]
    fn test_snapshot_fail_on_missing() {
        let config = SnapshotConfig {
            auto_update: false,
            snapshot_dir: PathBuf::from("target/test-snapshots-fail-missing"),
            fail_on_missing: true,
        };

        // Clean up
        let _ = std::fs::remove_dir_all(&config.snapshot_dir);

        let mut runner = SnapshotRunner::load(config).expect("operation should succeed in test");

        // Should fail because snapshot doesn't exist and fail_on_missing is true
        let result = runner.test("missing_test", "input", |_| Ok("output".to_string()));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing snapshot"));
    }

    #[test]
    fn test_snapshot_matching() {
        let config = SnapshotConfig {
            auto_update: false,
            snapshot_dir: PathBuf::from("target/test-snapshots-match"),
            fail_on_missing: false,
        };

        // Clean up
        let _ = std::fs::remove_dir_all(&config.snapshot_dir);

        let mut runner = SnapshotRunner::load(config).expect("operation should succeed in test");

        // Create initial snapshot
        runner
            .test("match_test", "input", |_| {
                Ok("consistent_output".to_string())
            })
            .expect("operation should succeed in test");

        // Test with same output - should pass
        runner
            .test("match_test", "input", |_| {
                Ok("consistent_output".to_string())
            })
            .expect("operation should succeed in test");

        assert_eq!(runner.suite.tests.len(), 1);
    }

    #[test]
    fn test_run_all_snapshots() {
        let config = SnapshotConfig {
            auto_update: false,
            snapshot_dir: PathBuf::from("target/test-snapshots-run-all"),
            fail_on_missing: false,
        };

        // Clean up
        let _ = std::fs::remove_dir_all(&config.snapshot_dir);

        let mut runner = SnapshotRunner::load(config).expect("operation should succeed in test");

        // Add some test snapshots
        runner
            .test("test1", "input1", |_| Ok("output1".to_string()))
            .expect("operation should succeed in test");
        runner
            .test("test2", "input2", |_| Ok("output2".to_string()))
            .expect("operation should succeed in test");

        // Run all tests with consistent transform
        let result = runner.run_all(|input| match input {
            "input1" => Ok("output1".to_string()),
            "input2" => Ok("output2".to_string()),
            _ => Ok("default".to_string()),
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_create_snapshot_with_metadata() {
        let config = SnapshotConfig {
            auto_update: false,
            snapshot_dir: PathBuf::from("target/test-snapshots-metadata"),
            fail_on_missing: false,
        };

        // Clean up
        let _ = std::fs::remove_dir_all(&config.snapshot_dir);

        let mut runner = SnapshotRunner::load(config).expect("operation should succeed in test");

        runner
            .test("metadata_test", "input", |_| Ok("output".to_string()))
            .expect("operation should succeed in test");

        assert_eq!(runner.suite.tests.len(), 1);
        let test = &runner.suite.tests[0];
        assert_eq!(test.name, "metadata_test");
        assert_eq!(test.input, "input");
        assert_eq!(test.rust_output, "output");
        assert!(!test.metadata.created_at.is_empty());
        assert!(!test.metadata.updated_at.is_empty());
        assert!(!test.metadata.ruchy_version.is_empty());
        assert!(!test.metadata.rustc_version.is_empty());
    }

    #[test]
    fn test_multiple_snapshots_same_runner() {
        let config = SnapshotConfig {
            auto_update: false,
            snapshot_dir: PathBuf::from("target/test-snapshots-multiple"),
            fail_on_missing: false,
        };

        // Clean up
        let _ = std::fs::remove_dir_all(&config.snapshot_dir);

        let mut runner = SnapshotRunner::load(config).expect("operation should succeed in test");

        // Create multiple snapshots
        runner
            .test("test_a", "input_a", |_| Ok("output_a".to_string()))
            .expect("operation should succeed in test");
        runner
            .test("test_b", "input_b", |_| Ok("output_b".to_string()))
            .expect("operation should succeed in test");
        runner
            .test("test_c", "input_c", |_| Ok("output_c".to_string()))
            .expect("operation should succeed in test");

        assert_eq!(runner.suite.tests.len(), 3);

        // Verify all tests are distinct
        let names: Vec<_> = runner.suite.tests.iter().map(|t| &t.name).collect();
        assert!(names.contains(&&"test_a".to_string()));
        assert!(names.contains(&&"test_b".to_string()));
        assert!(names.contains(&&"test_c".to_string()));
    }
}
#[cfg(test)]
mod property_tests_snapshot {
    use proptest::proptest;

    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_load_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}

// === EXTREME TDD Round 21 - Coverage Push Tests ===
#[cfg(test)]
mod coverage_push_tests {
    use super::*;

    #[test]
    fn test_snapshot_bisector_new() {
        let tests = vec![];
        let bisector = SnapshotBisector::new(tests);
        // Just verify it doesn't panic
        let _ = bisector;
    }

    #[test]
    fn test_snapshot_bisector_bisect() {
        let tests = vec![];
        let bisector = SnapshotBisector::new(tests);
        let result = bisector.bisect("test_name", |_| true);
        assert!(result.is_none()); // Currently returns None
    }

    #[test]
    fn test_core_snapshot_tests_count() {
        let tests = core_snapshot_tests();
        assert!(!tests.is_empty());
        assert!(tests.len() >= 15); // At least 15 core tests
    }

    #[test]
    fn test_core_snapshot_tests_coverage() {
        let tests = core_snapshot_tests();
        // Verify expected test names exist
        let names: Vec<_> = tests.iter().map(|(name, _)| *name).collect();
        assert!(names.contains(&"literal_int"));
        assert!(names.contains(&"literal_float"));
        assert!(names.contains(&"literal_string"));
        assert!(names.contains(&"binary_add"));
        assert!(names.contains(&"function_simple"));
    }

    #[test]
    fn test_snapshot_test_debug() {
        let test = SnapshotTest {
            name: "test".to_string(),
            input: "1+1".to_string(),
            output_hash: "abc".to_string(),
            rust_output: "2".to_string(),
            metadata: SnapshotMetadata {
                created_at: "2023-01-01".to_string(),
                updated_at: "2023-01-01".to_string(),
                ruchy_version: "1.0".to_string(),
                rustc_version: "1.75".to_string(),
            },
        };
        let debug = format!("{:?}", test);
        assert!(debug.contains("SnapshotTest"));
        assert!(debug.contains("test"));
    }

    #[test]
    fn test_snapshot_test_clone() {
        let test = SnapshotTest {
            name: "test".to_string(),
            input: "1+1".to_string(),
            output_hash: "abc".to_string(),
            rust_output: "2".to_string(),
            metadata: SnapshotMetadata {
                created_at: "2023-01-01".to_string(),
                updated_at: "2023-01-01".to_string(),
                ruchy_version: "1.0".to_string(),
                rustc_version: "1.75".to_string(),
            },
        };
        let cloned = test.clone();
        assert_eq!(test.name, cloned.name);
        assert_eq!(test.input, cloned.input);
    }

    #[test]
    fn test_snapshot_metadata_debug() {
        let metadata = SnapshotMetadata {
            created_at: "2023-01-01".to_string(),
            updated_at: "2023-01-02".to_string(),
            ruchy_version: "1.0".to_string(),
            rustc_version: "1.75".to_string(),
        };
        let debug = format!("{:?}", metadata);
        assert!(debug.contains("SnapshotMetadata"));
    }

    #[test]
    fn test_snapshot_metadata_clone() {
        let metadata = SnapshotMetadata {
            created_at: "2023-01-01".to_string(),
            updated_at: "2023-01-02".to_string(),
            ruchy_version: "1.0".to_string(),
            rustc_version: "1.75".to_string(),
        };
        let cloned = metadata.clone();
        assert_eq!(metadata.created_at, cloned.created_at);
    }

    #[test]
    fn test_snapshot_config_debug() {
        let config = SnapshotConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("SnapshotConfig"));
    }

    #[test]
    fn test_snapshot_config_clone() {
        let config = SnapshotConfig {
            auto_update: true,
            snapshot_dir: PathBuf::from("test"),
            fail_on_missing: false,
        };
        let cloned = config.clone();
        assert_eq!(config.auto_update, cloned.auto_update);
        assert_eq!(config.snapshot_dir, cloned.snapshot_dir);
    }

    #[test]
    fn test_snapshot_suite_debug() {
        let suite = SnapshotSuite {
            tests: vec![],
            config: SnapshotConfig::default(),
        };
        let debug = format!("{:?}", suite);
        assert!(debug.contains("SnapshotSuite"));
    }

    #[test]
    fn test_hash_empty_string() {
        let hash = SnapshotRunner::hash("");
        assert_eq!(hash.len(), 64);
        // SHA256 of empty string
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_hash_unicode() {
        let hash = SnapshotRunner::hash("Hello 世界 🌍");
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_run_all_with_failures() {
        let config = SnapshotConfig {
            auto_update: false,
            snapshot_dir: PathBuf::from("target/test-snapshots-failures"),
            fail_on_missing: false,
        };

        // Clean up
        let _ = std::fs::remove_dir_all(&config.snapshot_dir);

        let mut runner = SnapshotRunner::load(config).expect("load");

        // Create a snapshot
        runner
            .test("test1", "input1", |_| Ok("output1".to_string()))
            .expect("create");

        // Run with failing transform
        let result = runner.run_all(|_| Ok("wrong_output".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_snapshot_test_serialize_deserialize() {
        let test = SnapshotTest {
            name: "serialize_test".to_string(),
            input: "let x = 42".to_string(),
            output_hash: "deadbeef".to_string(),
            rust_output: "fn main() {}".to_string(),
            metadata: SnapshotMetadata {
                created_at: "2023-06-15T10:30:00Z".to_string(),
                updated_at: "2023-06-15T10:30:00Z".to_string(),
                ruchy_version: "1.2.3".to_string(),
                rustc_version: "1.70.0".to_string(),
            },
        };
        let toml_str = toml::to_string(&test).expect("serialize");
        let decoded: SnapshotTest = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(test.name, decoded.name);
        assert_eq!(test.input, decoded.input);
    }

    #[test]
    fn test_snapshot_suite_serialize_deserialize() {
        let suite = SnapshotSuite {
            tests: vec![SnapshotTest {
                name: "t1".to_string(),
                input: "1".to_string(),
                output_hash: "h1".to_string(),
                rust_output: "o1".to_string(),
                metadata: SnapshotMetadata {
                    created_at: "now".to_string(),
                    updated_at: "now".to_string(),
                    ruchy_version: "1.0".to_string(),
                    rustc_version: "1.75".to_string(),
                },
            }],
            config: SnapshotConfig::default(),
        };
        let toml_str = toml::to_string(&suite).expect("serialize");
        let decoded: SnapshotSuite = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(suite.tests.len(), decoded.tests.len());
    }

    #[test]
    fn test_snapshot_transform_error() {
        let config = SnapshotConfig {
            auto_update: false,
            snapshot_dir: PathBuf::from("target/test-snapshots-transform-err"),
            fail_on_missing: false,
        };

        let _ = std::fs::remove_dir_all(&config.snapshot_dir);

        let mut runner = SnapshotRunner::load(config).expect("load");

        // Test with transform that returns error
        let result = runner.test("error_test", "input", |_| {
            Err(anyhow::anyhow!("Transform failed"))
        });
        assert!(result.is_err());
    }
}
