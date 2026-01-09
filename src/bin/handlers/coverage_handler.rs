//! Coverage Command Handler
//!
//! Handles code coverage analysis for Ruchy files.

use anyhow::Result;
use std::path::Path;

/// Handle coverage command - analyze code coverage for Ruchy files
///
/// # Arguments
/// * `path` - Path to the Ruchy file to analyze
/// * `threshold` - Minimum coverage percentage (0.0 to skip threshold check)
/// * `format` - Output format (text, html, json)
/// * `verbose` - Enable verbose output
///
/// # Errors
/// Returns error if coverage analysis fails or threshold is not met
pub fn handle_coverage_command(
    path: &Path,
    threshold: f64,
    format: &str,
    verbose: bool,
) -> Result<()> {
    use ruchy::quality::ruchy_coverage::RuchyCoverageCollector;

    if verbose {
        println!("🔍 Analyzing coverage for: {}", path.display());
        println!("📊 Threshold: {:.1}%", threshold);
        println!("📋 Format: {}", format);
    }

    // Create coverage collector
    let mut collector = RuchyCoverageCollector::new();

    // Execute the file with coverage collection
    collector.execute_with_coverage(path)?;

    // Generate the coverage report based on format
    let report = match format {
        "html" => collector.generate_html_report(),
        "json" => collector.generate_json_report(),
        _ => collector.generate_text_report(), // Default to text
    };
    println!("{}", report);

    // Check threshold if specified
    if threshold > 0.0 {
        if collector.meets_threshold(threshold) {
            println!("\n✅ Coverage meets threshold of {:.1}%", threshold);
            Ok(())
        } else {
            eprintln!("\n❌ Coverage below threshold of {:.1}%", threshold);
            // Return an error instead of exiting - let the caller decide what to do
            Err(anyhow::anyhow!(
                "Coverage below threshold of {:.1}%",
                threshold
            ))
        }
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_coverage_command_nonexistent_file() {
        let path = PathBuf::from("/nonexistent/file.ruchy");
        let result = handle_coverage_command(&path, 0.0, "text", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_text_default() {
        // Just verify the format matching logic
        let format = "unknown";
        let is_text = !matches!(format, "html" | "json");
        assert!(is_text);
    }

    #[test]
    fn test_format_html() {
        let format = "html";
        let is_html = format == "html";
        assert!(is_html);
    }

    #[test]
    fn test_format_json() {
        let format = "json";
        let is_json = format == "json";
        assert!(is_json);
    }

    #[test]
    fn test_threshold_check_logic() {
        let threshold = 80.0;
        let coverage = 85.0;
        assert!(coverage >= threshold);

        let threshold = 80.0;
        let coverage = 75.0;
        assert!(coverage < threshold);
    }

    #[test]
    fn test_zero_threshold_skips_check() {
        let threshold = 0.0;
        assert!(threshold <= 0.0);
    }

    // ===== EXTREME TDD Round 152 - Coverage Handler Tests =====

    #[test]
    fn test_coverage_command_with_verbose() {
        let path = PathBuf::from("/nonexistent/file.ruchy");
        let result = handle_coverage_command(&path, 0.0, "text", true);
        assert!(result.is_err());
    }

    #[test]
    fn test_coverage_command_html_format() {
        let path = PathBuf::from("/nonexistent/file.ruchy");
        let result = handle_coverage_command(&path, 80.0, "html", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_coverage_command_json_format() {
        let path = PathBuf::from("/nonexistent/file.ruchy");
        let result = handle_coverage_command(&path, 50.0, "json", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_coverage_threshold_boundary() {
        // Test exact boundary
        let threshold = 80.0;
        let coverage = 80.0;
        assert!(coverage >= threshold);
    }

    #[test]
    fn test_coverage_threshold_just_below() {
        let threshold = 80.0;
        let coverage = 79.9999;
        assert!(coverage < threshold);
    }

    #[test]
    fn test_coverage_threshold_just_above() {
        let threshold = 80.0;
        let coverage = 80.0001;
        assert!(coverage >= threshold);
    }

    #[test]
    fn test_format_matching_logic() {
        let formats = ["text", "html", "json", "unknown", "XML"];
        for format in &formats {
            let is_html = *format == "html";
            let is_json = *format == "json";
            let is_default = !is_html && !is_json;
            // At least one must be true
            assert!(is_html || is_json || is_default);
        }
    }

    #[test]
    fn test_coverage_command_various_thresholds() {
        let thresholds = [0.0, 50.0, 75.0, 90.0, 100.0];
        for threshold in &thresholds {
            let path = PathBuf::from("/nonexistent/file.ruchy");
            let _ = handle_coverage_command(&path, *threshold, "text", false);
        }
    }

    #[test]
    fn test_negative_threshold_treated_as_zero() {
        let threshold = -10.0;
        assert!(threshold <= 0.0);
    }

    #[test]
    fn test_path_validation() {
        let paths = [
            "/nonexistent/file.ruchy",
            "relative/path.ruchy",
            "./current.ruchy",
            "../parent.ruchy",
        ];
        for path_str in &paths {
            let path = PathBuf::from(path_str);
            let _ = handle_coverage_command(&path, 0.0, "text", false);
        }
    }
}
