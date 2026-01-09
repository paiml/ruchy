//! Replay-to-Tests Command Handler
//!
//! Converts REPL replay files to regression tests.

use anyhow::{Context, Result};
use ruchy::runtime::replay_converter::ConversionConfig;
use std::path::{Path, PathBuf};

/// Handle replay-to-tests command - convert .replay files to regression tests
///
/// # Arguments
/// * `input` - Input replay file or directory containing .replay files
/// * `output` - Optional output test file path
/// * `property_tests` - Whether to include property tests
/// * `benchmarks` - Whether to include benchmarks
/// * `timeout` - Test timeout in milliseconds
///
/// # Examples
/// ```
/// // Convert single replay file
/// handle_replay_to_tests_command(Path::new("demo.replay"), None, true, false, 5000);
///
/// // Convert directory of replay files
/// handle_replay_to_tests_command(Path::new("demos/"), Some(Path::new("tests/replays.rs")), true, true, 10000);
/// ```
///
/// # Errors
/// Returns error if replay files can't be read or test files can't be written
pub fn handle_replay_to_tests_command(
    input: &Path,
    output: Option<&Path>,
    property_tests: bool,
    benchmarks: bool,
    timeout: u64,
) -> Result<()> {
    use colored::Colorize;
    use ruchy::runtime::replay_converter::ReplayConverter;

    println!(
        "{}",
        "🔄 Converting REPL replay files to regression tests"
            .bright_cyan()
            .bold()
    );
    println!("Input: {}", input.display());

    let config = setup_conversion_config(property_tests, benchmarks, timeout);
    let converter = ReplayConverter::with_config(config);
    let mut all_tests = Vec::new();
    let mut processed_files = 0;
    let output_path = determine_output_path(output);

    process_input_path(input, &converter, &mut all_tests, &mut processed_files)?;

    if all_tests.is_empty() {
        println!("⚠️  No tests generated");
        return Ok(());
    }

    write_test_output(&converter, &all_tests, output_path)?;
    generate_summary_report(&all_tests, processed_files);
    Ok(())
}

/// Setup conversion configuration for replay-to-test conversion (complexity: 4)
fn setup_conversion_config(
    property_tests: bool,
    benchmarks: bool,
    timeout: u64,
) -> ConversionConfig {
    ConversionConfig {
        test_module_prefix: "replay_generated".to_string(),
        include_property_tests: property_tests,
        include_benchmarks: benchmarks,
        timeout_ms: timeout,
    }
}

/// Determine output path, using default if none provided (complexity: 3)
pub(crate) fn determine_output_path(output: Option<&Path>) -> &Path {
    let default_output = Path::new("tests/generated_from_replays.rs");
    output.unwrap_or(default_output)
}

/// Validate that file has .replay extension (complexity: 3)
fn validate_replay_file(path: &Path) -> Result<()> {
    if path.extension().and_then(|s| s.to_str()) == Some("replay") {
        Ok(())
    } else {
        eprintln!("❌ Input file must have .replay extension");
        Err(anyhow::anyhow!("Invalid file extension"))
    }
}

/// Process a single .replay file (complexity: 8)
fn process_single_file(
    input: &Path,
    converter: &ruchy::runtime::replay_converter::ReplayConverter,
    all_tests: &mut Vec<ruchy::runtime::replay_converter::GeneratedTest>,
    processed_files: &mut usize,
) -> Result<()> {
    validate_replay_file(input)?;
    println!("📄 Processing replay file: {}", input.display());
    match converter.convert_file(input) {
        Ok(tests) => {
            println!("  ✅ Generated {} tests", tests.len());
            all_tests.extend(tests);
            *processed_files += 1;
            Ok(())
        }
        Err(e) => {
            eprintln!("  ❌ Failed to process {}: {}", input.display(), e);
            Err(e)
        }
    }
}

/// Process directory of replay files (complexity: 4 - reduced from 11)
fn process_directory(
    input: &Path,
    converter: &ruchy::runtime::replay_converter::ReplayConverter,
    all_tests: &mut Vec<ruchy::runtime::replay_converter::GeneratedTest>,
    processed_files: &mut usize,
) -> Result<()> {
    println!("📁 Processing replay directory: {}", input.display());
    let replay_files = find_replay_files(input)?;
    if replay_files.is_empty() {
        println!("⚠️  No .replay files found in directory");
        return Ok(());
    }
    println!("🔍 Found {} replay files", replay_files.len());
    process_replay_files(&replay_files, converter, all_tests, processed_files);
    Ok(())
}

/// Find all .replay files in directory (complexity: 3)
fn find_replay_files(dir: &Path) -> Result<Vec<PathBuf>> {
    use std::fs;
    Ok(fs::read_dir(dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() && path.extension()? == "replay" {
                Some(path)
            } else {
                None
            }
        })
        .collect())
}

/// Process all replay files in sequence (complexity: 4)
fn process_replay_files(
    replay_files: &[PathBuf],
    converter: &ruchy::runtime::replay_converter::ReplayConverter,
    all_tests: &mut Vec<ruchy::runtime::replay_converter::GeneratedTest>,
    processed_files: &mut usize,
) {
    for replay_file in replay_files {
        println!("📄 Processing: {}", replay_file.display());
        match converter.convert_file(replay_file) {
            Ok(tests) => {
                println!("  ✅ Generated {} tests", tests.len());
                all_tests.extend(tests);
                *processed_files += 1;
            }
            Err(e) => {
                eprintln!("  ⚠️  Failed to process {}: {}", replay_file.display(), e);
                // Continue with other files instead of failing completely
            }
        }
    }
}

/// Write test output to file, creating directories if needed (complexity: 4)
fn write_test_output(
    converter: &ruchy::runtime::replay_converter::ReplayConverter,
    all_tests: &[ruchy::runtime::replay_converter::GeneratedTest],
    output_path: &Path,
) -> Result<()> {
    use std::fs;

    // Create output directory if needed
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    println!("📝 Writing tests to: {}", output_path.display());
    converter
        .write_tests(all_tests, output_path)
        .context("Failed to write test file")?;
    Ok(())
}

/// Generate comprehensive summary report of conversion results (complexity: 8)
fn generate_summary_report(
    all_tests: &[ruchy::runtime::replay_converter::GeneratedTest],
    processed_files: usize,
) {
    use colored::Colorize;
    use std::collections::{HashMap, HashSet};

    println!("\n{}", "🎉 Conversion Summary".bright_green().bold());
    println!("=====================================");
    println!("📊 Files processed: {}", processed_files);
    println!("✅ Tests generated: {}", all_tests.len());

    // Breakdown by test category
    let mut category_counts = HashMap::new();
    let mut coverage_areas = HashSet::new();
    for test in all_tests {
        *category_counts.entry(&test.category).or_insert(0) += 1;
        coverage_areas.extend(test.coverage_areas.iter().cloned());
    }

    println!("\n📋 Test Breakdown:");
    for (category, count) in category_counts {
        println!("   {:?}: {}", category, count);
    }

    println!("\n🎯 Coverage Areas: {} unique areas", coverage_areas.len());
    if !coverage_areas.is_empty() {
        let mut areas: Vec<_> = coverage_areas.into_iter().collect();
        areas.sort();
        for area in areas.iter().take(10) {
            // Show first 10
            println!("   • {}", area);
        }
        if areas.len() > 10 {
            println!("   ... and {} more", areas.len() - 10);
        }
    }

    println!("\n💡 Next Steps:");
    println!("   1. Run tests: cargo test");
    println!("   2. Measure coverage: cargo test -- --test-threads=1");
    println!("   3. Validate replay determinism");
    println!(
        "\n🚀 {}",
        "Replay-to-test conversion complete!".bright_green()
    );
}

/// Process input path (file or directory) with replay files (complexity: 5)
fn process_input_path(
    input: &Path,
    converter: &ruchy::runtime::replay_converter::ReplayConverter,
    all_tests: &mut Vec<ruchy::runtime::replay_converter::GeneratedTest>,
    processed_files: &mut usize,
) -> Result<()> {
    if input.is_file() {
        process_single_file(input, converter, all_tests, processed_files)
    } else if input.is_dir() {
        process_directory(input, converter, all_tests, processed_files)
    } else {
        eprintln!("❌ Input path must be a file or directory");
        Err(anyhow::anyhow!("Invalid input path"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_conversion_config() {
        let config = setup_conversion_config(true, false, 5000);
        assert!(config.include_property_tests);
        assert!(!config.include_benchmarks);
        assert_eq!(config.timeout_ms, 5000);
    }

    #[test]
    fn test_determine_output_path_default() {
        let path = determine_output_path(None);
        assert_eq!(path, Path::new("tests/generated_from_replays.rs"));
    }

    #[test]
    fn test_determine_output_path_custom() {
        let custom = Path::new("custom/path.rs");
        let path = determine_output_path(Some(custom));
        assert_eq!(path, custom);
    }

    #[test]
    fn test_validate_replay_file_valid() {
        let path = Path::new("test.replay");
        assert!(validate_replay_file(path).is_ok());
    }

    #[test]
    fn test_validate_replay_file_invalid() {
        let path = Path::new("test.txt");
        assert!(validate_replay_file(path).is_err());
    }

    // ===== EXTREME TDD Round 152 - Replay Handler Tests =====

    #[test]
    fn test_setup_conversion_config_all_true() {
        let config = setup_conversion_config(true, true, 10000);
        assert!(config.include_property_tests);
        assert!(config.include_benchmarks);
        assert_eq!(config.timeout_ms, 10000);
    }

    #[test]
    fn test_setup_conversion_config_all_false() {
        let config = setup_conversion_config(false, false, 1000);
        assert!(!config.include_property_tests);
        assert!(!config.include_benchmarks);
        assert_eq!(config.timeout_ms, 1000);
    }

    #[test]
    fn test_determine_output_path_various_customs() {
        let paths = [
            Path::new("custom/path.rs"),
            Path::new("./relative.rs"),
            Path::new("/absolute/path.rs"),
        ];
        for custom in &paths {
            let result = determine_output_path(Some(custom));
            assert_eq!(result, *custom);
        }
    }

    #[test]
    fn test_validate_replay_file_various_extensions() {
        let valid = ["test.replay", "a.b.replay", "/path/to/file.replay"];
        let invalid = ["test.rs", "test.txt", "test", "test.replay.bak"];

        for path in &valid {
            assert!(validate_replay_file(Path::new(path)).is_ok());
        }
        for path in &invalid {
            assert!(validate_replay_file(Path::new(path)).is_err());
        }
    }

    #[test]
    fn test_handle_replay_to_tests_nonexistent() {
        let result = handle_replay_to_tests_command(
            Path::new("/nonexistent/file.replay"),
            None,
            true,
            false,
            5000,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_replay_to_tests_invalid_extension() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let invalid_file = temp_dir.path().join("test.txt");
        std::fs::write(&invalid_file, "content").unwrap();

        let result = handle_replay_to_tests_command(
            &invalid_file,
            None,
            false,
            false,
            5000,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_setup_conversion_config_various_timeouts() {
        let timeouts = [100, 1000, 5000, 30000, 60000];
        for timeout in &timeouts {
            let config = setup_conversion_config(false, false, *timeout);
            assert_eq!(config.timeout_ms, *timeout);
        }
    }

    #[test]
    fn test_config_module_prefix() {
        let config = setup_conversion_config(true, true, 5000);
        assert_eq!(config.test_module_prefix, "replay_generated");
    }

    #[test]
    fn test_handle_replay_directory_nonexistent() {
        let result = handle_replay_to_tests_command(
            Path::new("/nonexistent/directory/"),
            None,
            false,
            false,
            5000,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_replay_empty_directory() {
        let temp_dir = tempfile::TempDir::new().unwrap();

        let result = handle_replay_to_tests_command(
            temp_dir.path(),
            None,
            false,
            false,
            5000,
        );
        // Should succeed with "No tests generated" message
        assert!(result.is_ok());
    }
}
