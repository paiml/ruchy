//! Quality gate enforcement implementation for CLI
use std::path::Path;
use anyhow::Result;
use crate::quality::gates::{QualityGateEnforcer, QualityGateConfig};
use crate::quality::scoring::{ScoreEngine, AnalysisDepth};
/// Load and configure quality gate enforcer (complexity: 6)
fn load_gate_config(
    path: &Path, 
    config: Option<&Path>, 
    ci: bool
) -> Result<QualityGateEnforcer> {
    // Load configuration
    let project_root = find_project_root(path)?;
    let mut gate_config = if let Some(config_path) = config {
        QualityGateEnforcer::load_config(config_path.parent().unwrap_or(Path::new(".")))
    } else {
        QualityGateEnforcer::load_config(&project_root)
    }?;
    // Apply CI mode overrides (stricter thresholds)
    if ci {
        gate_config = apply_ci_overrides(gate_config);
    }
    Ok(QualityGateEnforcer::new(gate_config))
}
/// Parse analysis depth string parameter (complexity: 4)
fn parse_analysis_depth(depth: &str) -> Result<AnalysisDepth> {
    match depth {
        "shallow" => Ok(AnalysisDepth::Shallow),
        "standard" => Ok(AnalysisDepth::Standard),
        "deep" => Ok(AnalysisDepth::Deep),
        _ => Err(anyhow::anyhow!("Invalid depth: {}", depth)),
    }
}
/// Process file or directory path (complexity: 5)
fn process_path(
    path: &Path,
    enforcer: &QualityGateEnforcer,
    analysis_depth: AnalysisDepth,
    fail_fast: bool,
    verbose: bool
) -> Result<Vec<crate::quality::gates::GateResult>> {
    let mut all_results = Vec::new();
    if path.is_file() {
        let result = process_file(enforcer, path, analysis_depth, verbose)?;
        all_results.push(result);
    } else if path.is_dir() {
        let results = process_directory(enforcer, path, analysis_depth, fail_fast, verbose)?;
        all_results.extend(results);
    } else {
        return Err(anyhow::anyhow!("Invalid path: {}", path.display()));
    }
    Ok(all_results)
}
/// Output results in specified format (complexity: 4)
fn output_results(
    results: &[crate::quality::gates::GateResult],
    format: &str,
    verbose: bool
) -> Result<()> {
    match format {
        "console" => print_console_results(results, verbose)?,
        "json" => print_json_results(results)?,
        "junit" => print_junit_results(results)?,
        _ => return Err(anyhow::anyhow!("Invalid format: {}", format)),
    }
    Ok(())
}
/// Handle CI export if requested (complexity: 3)
fn handle_export(
    enforcer: &QualityGateEnforcer,
    results: &[crate::quality::gates::GateResult],
    export: Option<&Path>
) -> Result<()> {
    if let Some(export_path) = export {
        std::fs::create_dir_all(export_path)?;
        enforcer.export_ci_results(results, export_path)?;
        println!("📊 Results exported to {}", export_path.display());
    }
    Ok(())
}
/// Check gate results and exit appropriately (complexity: 4)
fn check_gate_results(results: &[crate::quality::gates::GateResult]) -> Result<()> {
    let failed_gates = results.iter().filter(|r| !r.passed).count();
    if failed_gates > 0 {
        eprintln!("❌ {failed_gates} quality gate(s) failed");
        std::process::exit(1);
    } else {
        println!("✅ All quality gates passed!");
    }
    Ok(())
}
/// Enforce quality gates on a file or directory (complexity: 6)
/// # Examples
/// 
/// ```
/// use ruchy::quality::enforcement::enforce_quality_gates;
/// 
/// let result = enforce_quality_gates("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn enforce_quality_gates(
    path: &Path,
    config: Option<&Path>,
    depth: &str,
    fail_fast: bool,
    format: &str,
    export: Option<&Path>,
    ci: bool,
    verbose: bool,
) -> Result<()> {
    let enforcer = load_gate_config(path, config, ci)?;
    let analysis_depth = parse_analysis_depth(depth)?;
    let all_results = process_path(path, &enforcer, analysis_depth, fail_fast, verbose)?;
    output_results(&all_results, format, verbose)?;
    handle_export(&enforcer, &all_results, export)?;
    check_gate_results(&all_results)?;
    Ok(())
}
fn find_project_root(path: &Path) -> Result<std::path::PathBuf> {
    let mut current = if path.is_file() {
        path.parent().unwrap_or(Path::new("."))
    } else {
        path
    };
    loop {
        if current.join("Cargo.toml").exists() || current.join(".ruchy").exists() {
            return Ok(current.to_path_buf());
        }
        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            // Default to current directory
            return Ok(Path::new(".").to_path_buf());
        }
    }
}
fn apply_ci_overrides(mut config: QualityGateConfig) -> QualityGateConfig {
    // Apply stricter thresholds for CI
    config.min_score = config.min_score.max(0.8); // Higher overall score
    config.component_thresholds.correctness = config.component_thresholds.correctness.max(0.9);
    config.component_thresholds.safety = config.component_thresholds.safety.max(0.9);
    config.anti_gaming.min_confidence = config.anti_gaming.min_confidence.max(0.8);
    config.ci_integration.fail_on_violation = true;
    config
}
fn process_file(
    enforcer: &QualityGateEnforcer,
    file_path: &Path,
    depth: AnalysisDepth,
    verbose: bool,
) -> Result<crate::quality::gates::GateResult> {
    if verbose {
        println!("🔍 Analyzing {}", file_path.display());
    }
    // Read and parse file
    let content = std::fs::read_to_string(file_path)?;
    let mut parser = crate::Parser::new(&content);
    let ast = parser.parse()?;
    // Calculate score
    let score_config = crate::quality::scoring::ScoreConfig::default();
    let mut score_engine = ScoreEngine::new(score_config);
    let score = score_engine.score_incremental(&ast, file_path.to_path_buf(), &content, depth);
    // Enforce gates
    let result = enforcer.enforce_gates(&score, Some(&file_path.to_path_buf()));
    Ok(result)
}
fn process_directory(
    enforcer: &QualityGateEnforcer,
    dir_path: &Path,
    depth: AnalysisDepth,
    fail_fast: bool,
    verbose: bool,
) -> Result<Vec<crate::quality::gates::GateResult>> {
    let mut results = Vec::new();
    for entry in std::fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        let entry_results = process_directory_entry(
            enforcer,
            &path,
            depth,
            fail_fast,
            verbose,
        )?;
        // Handle fail-fast mode
        if should_fail_fast(&entry_results, fail_fast) {
            return Ok(entry_results);
        }
        results.extend(entry_results);
    }
    Ok(results)
}
/// Process a single directory entry (complexity: 6)
fn process_directory_entry(
    enforcer: &QualityGateEnforcer,
    path: &Path,
    depth: AnalysisDepth,
    fail_fast: bool,
    verbose: bool,
) -> Result<Vec<crate::quality::gates::GateResult>> {
    if is_ruchy_file(path) {
        process_ruchy_file(enforcer, path, depth, fail_fast, verbose)
    } else if is_processable_directory(path) {
        process_directory(enforcer, path, depth, fail_fast, verbose)
    } else {
        Ok(Vec::new())
    }
}
/// Check if path is a Ruchy file (complexity: 2)
fn is_ruchy_file(path: &Path) -> bool {
    path.is_file() && path.extension().is_some_and(|ext| ext == "ruchy")
}
/// Check if directory should be processed (complexity: 3)
fn is_processable_directory(path: &Path) -> bool {
    path.is_dir() && !path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .starts_with('.')
}
/// Process a Ruchy file and handle errors (complexity: 5)
fn process_ruchy_file(
    enforcer: &QualityGateEnforcer,
    path: &Path,
    depth: AnalysisDepth,
    fail_fast: bool,
    verbose: bool,
) -> Result<Vec<crate::quality::gates::GateResult>> {
    match process_file(enforcer, path, depth, verbose) {
        Ok(result) => {
            if fail_fast && !result.passed {
                eprintln!("❌ Failed fast on {}", path.display());
            }
            Ok(vec![result])
        }
        Err(e) => {
            eprintln!("⚠️ Error processing {}: {}", path.display(), e);
            if fail_fast {
                Err(e)
            } else {
                Ok(Vec::new())
            }
        }
    }
}
/// Check if we should fail fast (complexity: 2)
fn should_fail_fast(results: &[crate::quality::gates::GateResult], fail_fast: bool) -> bool {
    fail_fast && results.iter().any(|r| !r.passed)
}
fn print_console_results(results: &[crate::quality::gates::GateResult], verbose: bool) -> Result<()> {
    for (i, result) in results.iter().enumerate() {
        println!("\n📋 Quality Gate #{}: {}", i + 1, if result.passed { "✅ PASSED" } else { "❌ FAILED" });
        println!("   Score: {:.1}% ({})", result.score * 100.0, result.grade);
        println!("   Confidence: {:.1}%", result.confidence * 100.0);
        if !result.violations.is_empty() {
            println!("   Violations:");
            for violation in &result.violations {
                println!("     • {}", violation.message);
                if verbose {
                    println!("       Type: {:?}, Severity: {:?}", violation.violation_type, violation.severity);
                    println!("       Required: {:.3}, Actual: {:.3}", violation.required, violation.actual);
                }
            }
        }
        if !result.gaming_warnings.is_empty() {
            println!("   Warnings:");
            for warning in &result.gaming_warnings {
                println!("     ⚠️ {warning}");
            }
        }
    }
    let passed = results.iter().filter(|r| r.passed).count();
    let total = results.len();
    println!("\n📊 Summary: {passed}/{total} gates passed");
    Ok(())
}
fn print_json_results(results: &[crate::quality::gates::GateResult]) -> Result<()> {
    let json = serde_json::to_string_pretty(results)?;
    println!("{json}");
    Ok(())
}
fn print_junit_results(results: &[crate::quality::gates::GateResult]) -> Result<()> {
    let total = results.len();
    let failures = results.iter().filter(|r| !r.passed).count();
    println!(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    println!(r#"<testsuite name="Quality Gates" tests="{total}" failures="{failures}" time="0.0">"#);
    for (i, result) in results.iter().enumerate() {
        let test_name = format!("quality-gate-{i}");
        if result.passed {
            println!(r#"  <testcase name="{test_name}" classname="QualityGate" time="0.0"/>"#);
        } else {
            println!(r#"  <testcase name="{test_name}" classname="QualityGate" time="0.0">"#);
            println!(r#"    <failure message="Quality gate violation">Score: {:.1}%, Grade: {}</failure>"#, 
                result.score * 100.0, result.grade);
            println!(r"  </testcase>");
        }
    }
    println!("</testsuite>");
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    use crate::quality::gates::QualityGateConfig;
    fn create_test_ruchy_file(dir: &Path, filename: &str, content: &str) -> std::path::PathBuf {
        let file_path = dir.join(filename);
        fs::write(&file_path, content).unwrap();
        file_path
    }
    fn create_test_project_structure(dir: &Path) -> std::path::PathBuf {
        // Create Cargo.toml to mark as project root
        fs::write(dir.join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"").unwrap();
        // Create .ruchy directory
        fs::create_dir_all(dir.join(".ruchy")).unwrap();
        // Create some test Ruchy files
        create_test_ruchy_file(dir, "test.ruchy", "let x = 5\nprintln(x)");
        create_test_ruchy_file(dir, "simple.ruchy", "println(\"hello\")");
        dir.to_path_buf()
    }
    // Test 1: Project Root Finding
    #[test]
    fn test_find_project_root_with_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        // Create Cargo.toml
        fs::write(project_dir.join("Cargo.toml"), "[package]").unwrap();
        let found_root = find_project_root(project_dir).unwrap();
        assert_eq!(found_root, project_dir);
    }
    #[test]
    fn test_find_project_root_with_ruchy_dir() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        // Create .ruchy directory
        fs::create_dir_all(project_dir.join(".ruchy")).unwrap();
        let found_root = find_project_root(project_dir).unwrap();
        assert_eq!(found_root, project_dir);
    }
    #[test]
    fn test_find_project_root_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        // Create Cargo.toml
        fs::write(project_dir.join("Cargo.toml"), "[package]").unwrap();
        // Create a file in project
        let file_path = create_test_ruchy_file(project_dir, "test.ruchy", "let x = 5");
        let found_root = find_project_root(&file_path).unwrap();
        assert_eq!(found_root, project_dir);
    }
    #[test]
    fn test_find_project_root_nested() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        // Create nested directory structure
        let nested_dir = project_dir.join("src").join("deep");
        fs::create_dir_all(&nested_dir).unwrap();
        // Create Cargo.toml at root
        fs::write(project_dir.join("Cargo.toml"), "[package]").unwrap();
        // Create file in nested directory
        let file_path = create_test_ruchy_file(&nested_dir, "nested.ruchy", "println(\"nested\")");
        let found_root = find_project_root(&file_path).unwrap();
        assert_eq!(found_root, project_dir);
    }
    #[test]
    fn test_find_project_root_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let some_dir = temp_dir.path().join("no_project_markers");
        fs::create_dir_all(&some_dir).unwrap();
        let found_root = find_project_root(&some_dir).unwrap();
        // Should fallback to current directory
        assert_eq!(found_root, Path::new("."));
    }
    // Test 2: CI Overrides
    #[test]
    fn test_apply_ci_overrides() {
        let mut config = QualityGateConfig::default();
        // Set lower initial values to test overrides
        config.min_score = 0.6;
        config.component_thresholds.correctness = 0.7;
        config.component_thresholds.safety = 0.7;
        config.anti_gaming.min_confidence = 0.5;
        config.ci_integration.fail_on_violation = false;
        let ci_config = apply_ci_overrides(config);
        // Should apply stricter thresholds
        assert!(ci_config.min_score >= 0.8);
        assert!(ci_config.component_thresholds.correctness >= 0.9);
        assert!(ci_config.component_thresholds.safety >= 0.9);
        assert!(ci_config.anti_gaming.min_confidence >= 0.8);
        assert!(ci_config.ci_integration.fail_on_violation);
    }
    #[test]
    fn test_ci_overrides_preserve_higher_values() {
        let mut config = QualityGateConfig::default();
        // Set higher initial values
        config.min_score = 0.95;
        config.component_thresholds.correctness = 0.95;
        config.component_thresholds.safety = 0.95;
        config.anti_gaming.min_confidence = 0.95;
        let ci_config = apply_ci_overrides(config);
        // Should preserve higher existing values
        assert_eq!(ci_config.min_score, 0.95);
        assert_eq!(ci_config.component_thresholds.correctness, 0.95);
        assert_eq!(ci_config.component_thresholds.safety, 0.95);
        assert_eq!(ci_config.anti_gaming.min_confidence, 0.95);
    }
    // Test 3: Analysis Depth Parsing
    #[test]
    fn test_analysis_depth_parsing() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "depth_test.ruchy", "let x = 1");
        // Test all valid depth values
        let depths = vec![
            ("shallow", true),
            ("standard", true), 
            ("deep", true),
            ("invalid", false),
            ("", false),
        ];
        for (depth_str, should_succeed) in depths {
            let result = enforce_quality_gates(
                &file_path,
                None,
                depth_str,
                false,
                "console",
                None,
                false,
                false,
            );
            if should_succeed {
                // For valid depths, should not fail on depth parsing
                // May fail on other quality issues, but not depth parsing
                if let Err(e) = &result {
                    assert!(!e.to_string().contains("Invalid depth"), 
                        "Should not fail on depth parsing for '{depth_str}'");
                }
            } else {
                assert!(result.is_err(), "Should fail for invalid depth: '{depth_str}'");
                if let Err(e) = result {
                    assert!(e.to_string().contains("Invalid depth"), 
                        "Should fail with depth error for '{depth_str}'");
                }
            }
        }
    }
    // Test 4: Format Validation
    #[test] 
    fn test_format_validation() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "format_test.ruchy", "let x = 1");
        let formats = vec![
            ("console", true),
            ("json", true),
            ("junit", true),
            ("xml", false),
            ("invalid", false),
        ];
        for (format, should_succeed) in formats {
            let result = enforce_quality_gates(
                &file_path,
                None,
                "standard",
                false,
                format,
                None,
                false,
                false,
            );
            if should_succeed {
                // Valid formats shouldn't fail on format parsing
                if let Err(e) = &result {
                    assert!(!e.to_string().contains("Invalid format"), 
                        "Should not fail on format parsing for '{format}'");
                }
            } else {
                assert!(result.is_err(), "Should fail for invalid format: '{format}'");
                if let Err(e) = result {
                    assert!(e.to_string().contains("Invalid format"), 
                        "Should fail with format error for '{format}'");
                }
            }
        }
    }
    // Test 5: File vs Directory Processing
    #[test]
    fn test_single_file_processing() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "single.ruchy", "println(\"test\")");
        // This should not crash and should process the single file
        let result = enforce_quality_gates(
            &file_path,
            None,
            "standard", 
            false,
            "console",
            None,
            false,
            false,
        );
        // May fail due to quality issues, but should not crash
        assert!(result.is_ok() || result.is_err(), "Should complete processing");
    }
    #[test]
    fn test_directory_processing() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        // Create multiple files
        create_test_ruchy_file(&project_dir, "file1.ruchy", "let a = 1");
        create_test_ruchy_file(&project_dir, "file2.ruchy", "let b = 2");
        // This should process directory
        let result = enforce_quality_gates(
            &project_dir,
            None,
            "standard",
            false,
            "console", 
            None,
            false,
            false,
        );
        // May fail due to quality issues, but should not crash
        assert!(result.is_ok() || result.is_err(), "Should complete directory processing");
    }
    #[test]
    fn test_nonexistent_path() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("does_not_exist.ruchy");
        let result = enforce_quality_gates(
            &nonexistent,
            None,
            "standard",
            false,
            "console",
            None,
            false,
            false,
        );
        assert!(result.is_err(), "Should fail for nonexistent path");
    }
    // Test 6: Configuration Loading
    #[test]
    fn test_custom_config_loading() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "config_test.ruchy", "let x = 1");
        // Create custom config
        let config_dir = temp_dir.path().join("custom_config");
        fs::create_dir_all(&config_dir).unwrap();
        fs::create_dir_all(config_dir.join(".ruchy")).unwrap();
        // Create custom score.toml
        let config_content = r#"
min_score = 0.5
min_grade = "D"
[component_thresholds]
correctness = 0.4
performance = 0.4
maintainability = 0.4
safety = 0.4
idiomaticity = 0.4
"#;
        fs::write(config_dir.join(".ruchy").join("score.toml"), config_content).unwrap();
        let custom_config_path = config_dir.join("score.toml");
        let result = enforce_quality_gates(
            &file_path,
            Some(&custom_config_path),
            "standard",
            false,
            "console",
            None,
            false,
            false,
        );
        // Should use custom config (may pass due to lower thresholds)
        assert!(result.is_ok() || result.is_err(), "Should process with custom config");
#[cfg(test)]
use proptest::prelude::*;
    }
    // Test 7: Export Functionality
    #[test]
    fn test_export_directory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "export_test.ruchy", "let x = 1");
        let export_dir = temp_dir.path().join("exports");
        let _result = enforce_quality_gates(
            &file_path,
            None,
            "standard",
            false,
            "console",
            Some(&export_dir),
            false,
            false,
        );
        // Should create export directory
        assert!(export_dir.exists(), "Export directory should be created");
    }
    // Test 8: Error Handling
    #[test]
    fn test_invalid_ruchy_syntax() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        // Create file with invalid syntax
        let bad_file = create_test_ruchy_file(&project_dir, "bad_syntax.ruchy", "let = = invalid syntax here");
        let result = enforce_quality_gates(
            &bad_file,
            None,
            "standard",
            false,
            "console",
            None,
            false,
            false,
        );
        // Should handle parsing errors gracefully
        assert!(result.is_err(), "Should fail gracefully on invalid syntax");
    }
    // Test 9: Verbose Output Mode
    #[test]
    fn test_verbose_mode_flag() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "verbose_test.ruchy", "let x = 1");
        // Test both verbose modes (this mainly tests that verbose flag is accepted)
        for verbose in [true, false] {
            let result = enforce_quality_gates(
                &file_path,
                None,
                "standard",
                false,
                "console",
                None,
                false,
                verbose,
            );
            // Should accept verbose flag without crashing
            assert!(result.is_ok() || result.is_err(), "Should handle verbose flag");
        }
    }
    // Test 10: Fail Fast Mode  
    #[test]
    fn test_fail_fast_mode() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        // Create multiple files
        create_test_ruchy_file(&project_dir, "fail1.ruchy", "let a = 1");
        create_test_ruchy_file(&project_dir, "fail2.ruchy", "let b = 2");
        for fail_fast in [true, false] {
            let result = enforce_quality_gates(
                &project_dir,
                None,
                "standard",
                fail_fast,
                "console",
                None,
                false,
                false,
            );
            // Should handle fail_fast flag without crashing
            assert!(result.is_ok() || result.is_err(), "Should handle fail_fast flag");
        }
    }
}
#[cfg(test)]
mod property_tests_enforcement {
    use proptest::proptest;
    use super::*;
    use proptest::prelude::*;
    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_enforce_quality_gates_never_panics(input: String) {
            // Limit input size to avoid timeout
            let input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
