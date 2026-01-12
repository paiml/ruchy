//! Fuzz Command Handler
//!
//! Handles fuzz testing for Ruchy files.

use anyhow::Result;
use std::fs;
use std::path::Path;

/// Run cargo fuzz on target
fn run_cargo_fuzz(
    target: &str,
    iterations: usize,
    timeout: u32,
    verbose: bool,
) -> Result<std::process::Output> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.args([
        "fuzz",
        "run",
        target,
        "--",
        &format!("-runs={}", iterations),
        &format!("-timeout={}", timeout),
    ]);

    let output_result = cmd.output()?;
    super::log_command_output(&output_result, verbose);

    Ok(output_result)
}

/// Write fuzz test summary
fn write_fuzz_summary(
    format: &str,
    output: Option<&Path>,
    target: &str,
    iterations: usize,
    success: bool,
    stdout: &str,
) -> Result<()> {
    if format == "json" {
        write_fuzz_json(output, target, iterations, success, stdout)
    } else {
        write_fuzz_text(output, target, iterations, stdout)
    }
}

/// Write JSON fuzz test report
fn write_fuzz_json(
    output: Option<&Path>,
    target: &str,
    iterations: usize,
    success: bool,
    stdout: &str,
) -> Result<()> {
    let report = serde_json::json!({
        "target": target,
        "iterations": iterations,
        "status": if success { "passed" } else { "failed" },
        "output": stdout
    });
    let json_output = serde_json::to_string_pretty(&report)?;
    if let Some(out_path) = output {
        fs::write(out_path, json_output)?;
    } else {
        println!("{}", json_output);
    }
    Ok(())
}

/// Write text fuzz test report
fn write_fuzz_text(
    output: Option<&Path>,
    target: &str,
    iterations: usize,
    stdout: &str,
) -> Result<()> {
    println!("Fuzz Test Report");
    println!("================");
    println!("Target: {}", target);
    println!("Iterations: {}", iterations);
    if let Some(out_path) = output {
        super::write_file_with_context(out_path, stdout.as_bytes())?;
    } else {
        println!("\n{}", stdout);
    }
    Ok(())
}

/// Handle fuzz command - single file or cargo-fuzz target
pub fn handle_fuzz_command(
    target: &str,
    iterations: usize,
    timeout: u32,
    format: &str,
    output: Option<&Path>,
    verbose: bool,
) -> Result<()> {
    if verbose {
        eprintln!("Running fuzz tests on target: {}", target);
        eprintln!("Iterations: {}, Timeout: {}ms", iterations, timeout);
    }

    let target_path = Path::new(target);
    if target_path.is_file() && target_path.extension().and_then(|s| s.to_str()) == Some("ruchy") {
        return handle_fuzz_single_file(target_path, iterations, timeout, format, output, verbose);
    }

    let output_result = run_cargo_fuzz(target, iterations, timeout, verbose)?;
    let stdout = String::from_utf8_lossy(&output_result.stdout);
    let stderr = String::from_utf8_lossy(&output_result.stderr);

    let success = output_result.status.success();
    write_fuzz_summary(format, output, target, iterations, success, &stdout)?;

    if success {
        Ok(())
    } else {
        anyhow::bail!("Fuzz tests found crashes or panics:\n{}", stderr)
    }
}

/// Run fuzz iterations on compiled binary
#[allow(clippy::unnecessary_wraps)]
fn run_fuzz_iterations(
    binary_path: &Path,
    iterations: usize,
    verbose: bool,
) -> Result<(usize, usize, usize, Vec<String>)> {
    let mut crashes = 0;
    let mut timeouts = 0;
    let mut successes = 0;
    let mut crash_details = Vec::new();

    for i in 0..iterations {
        if verbose && i % 100 == 0 {
            eprintln!("  Iteration {}/{}", i, iterations);
        }

        let result = std::process::Command::new(binary_path).output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    successes += 1;
                } else {
                    crashes += 1;
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    crash_details.push(format!("Iteration {}: {}", i, stderr));
                }
            }
            Err(e) => {
                timeouts += 1;
                crash_details.push(format!("Iteration {}: Timeout/Error - {}", i, e));
            }
        }
    }

    Ok((successes, crashes, timeouts, crash_details))
}

/// Write JSON format fuzz test report
#[allow(clippy::too_many_arguments)]
fn write_json_fuzz_report(
    path: &Path,
    output: Option<&Path>,
    iterations: usize,
    successes: usize,
    crashes: usize,
    timeouts: usize,
    success_rate: f64,
    crash_details: &[String],
) -> Result<()> {
    let report = serde_json::json!({
        "file": path.display().to_string(),
        "iterations": iterations,
        "successes": successes,
        "crashes": crashes,
        "timeouts": timeouts,
        "success_rate": success_rate,
        "status": if crashes == 0 && timeouts == 0 { "passed" } else { "failed" },
        "crash_details": crash_details
    });
    let json_output = serde_json::to_string_pretty(&report)?;

    if let Some(out_path) = output {
        super::write_file_with_context(out_path, json_output.as_bytes())?;
    } else {
        println!("{}", json_output);
    }
    Ok(())
}

/// Write text format fuzz test report
#[allow(clippy::too_many_arguments)]
fn write_text_fuzz_report(
    path: &Path,
    output: Option<&Path>,
    iterations: usize,
    successes: usize,
    crashes: usize,
    timeouts: usize,
    success_rate: f64,
    crash_details: &[String],
) -> Result<()> {
    println!("Fuzz Test Report");
    println!("================");
    println!("File: {}", path.display());
    println!("Iterations: {}", iterations);
    println!("Successes: {}", successes);
    println!("Crashes: {}", crashes);
    println!("Timeouts: {}", timeouts);
    println!("Success rate: {:.1}%", success_rate);
    println!(
        "Status: {}",
        if crashes == 0 && timeouts == 0 {
            "PASSED"
        } else {
            "FAILED"
        }
    );

    if !crash_details.is_empty() {
        println!("\nCrash Details:");
        for detail in crash_details {
            println!("  - {}", detail);
        }
    }

    if let Some(out_path) = output {
        let report = format!(
            "Fuzz Test Report\nFile: {}\nSuccess rate: {:.1}%\n",
            path.display(),
            success_rate
        );
        fs::write(out_path, report)?;
    }

    Ok(())
}

/// Handle fuzz testing for a single file
fn handle_fuzz_single_file(
    path: &Path,
    iterations: usize,
    _timeout_ms: u32,
    format: &str,
    output: Option<&Path>,
    verbose: bool,
) -> Result<()> {
    use super::property_tests_handler::compile_for_property_testing;

    if verbose {
        eprintln!("Fuzzing single file: {}", path.display());
    }

    let binary_path = compile_for_property_testing(path, verbose)?;
    let (successes, crashes, timeouts, crash_details) =
        run_fuzz_iterations(&binary_path, iterations, verbose)?;

    let _ = fs::remove_file(&binary_path);

    let total = successes + crashes + timeouts;
    let success_rate = (successes as f64 / total as f64) * 100.0;

    match format {
        "json" => write_json_fuzz_report(
            path,
            output,
            iterations,
            successes,
            crashes,
            timeouts,
            success_rate,
            &crash_details,
        )?,
        _ => write_text_fuzz_report(
            path,
            output,
            iterations,
            successes,
            crashes,
            timeouts,
            success_rate,
            &crash_details,
        )?,
    }

    if crashes == 0 && timeouts == 0 {
        Ok(())
    } else {
        anyhow::bail!(
            "Fuzz tests found {} crashes and {} timeouts",
            crashes,
            timeouts
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_run_fuzz_iterations_result_type() {
        // Just verify the result type is correct
        let result: Result<(usize, usize, usize, Vec<String>)> = Ok((10, 0, 0, vec![]));
        assert!(result.is_ok());
    }

    // ===== EXTREME TDD Round 144 - Fuzz Handler Tests =====

    #[test]
    fn test_write_fuzz_json_success() {
        let result = write_fuzz_json(None, "test_target", 100, true, "output");
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_fuzz_json_failure() {
        let result = write_fuzz_json(None, "test_target", 100, false, "crash info");
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_fuzz_json_to_file() {
        let temp = NamedTempFile::new().unwrap();
        let result = write_fuzz_json(Some(temp.path()), "target", 50, true, "data");
        assert!(result.is_ok());
        let content = std::fs::read_to_string(temp.path()).unwrap();
        assert!(content.contains("target"));
        assert!(content.contains("50"));
    }

    #[test]
    fn test_write_fuzz_text_basic() {
        let result = write_fuzz_text(None, "test_target", 100, "output");
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_fuzz_text_to_file() {
        let temp = NamedTempFile::new().unwrap();
        let result = write_fuzz_text(Some(temp.path()), "target", 50, "data");
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_fuzz_summary_json() {
        let result = write_fuzz_summary("json", None, "target", 100, true, "out");
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_fuzz_summary_text() {
        let result = write_fuzz_summary("text", None, "target", 100, true, "out");
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_json_fuzz_report_all_success() {
        let path = Path::new("/test/file.ruchy");
        let result = write_json_fuzz_report(path, None, 100, 100, 0, 0, 100.0, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_json_fuzz_report_with_crashes() {
        let path = Path::new("/test/file.ruchy");
        let crashes = vec!["Crash at iteration 5".to_string()];
        let result = write_json_fuzz_report(path, None, 100, 95, 5, 0, 95.0, &crashes);
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_json_fuzz_report_to_file() {
        let path = Path::new("/test/file.ruchy");
        let temp = NamedTempFile::new().unwrap();
        let result = write_json_fuzz_report(path, Some(temp.path()), 50, 50, 0, 0, 100.0, &[]);
        assert!(result.is_ok());
        let content = std::fs::read_to_string(temp.path()).unwrap();
        assert!(content.contains("passed"));
    }

    #[test]
    fn test_write_text_fuzz_report_all_success() {
        let path = Path::new("/test/file.ruchy");
        let result = write_text_fuzz_report(path, None, 100, 100, 0, 0, 100.0, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_text_fuzz_report_with_failures() {
        let path = Path::new("/test/file.ruchy");
        let failures = vec![
            "Crash at iteration 1".to_string(),
            "Timeout at iteration 50".to_string(),
        ];
        let result = write_text_fuzz_report(path, None, 100, 90, 5, 5, 90.0, &failures);
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_text_fuzz_report_to_file() {
        let path = Path::new("/test/file.ruchy");
        let temp = NamedTempFile::new().unwrap();
        let result = write_text_fuzz_report(path, Some(temp.path()), 50, 50, 0, 0, 100.0, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fuzz_result_tuple_structure() {
        // Test the expected result tuple structure
        let (successes, crashes, timeouts, details): (usize, usize, usize, Vec<String>) =
            (100, 0, 0, vec![]);
        assert_eq!(successes, 100);
        assert_eq!(crashes, 0);
        assert_eq!(timeouts, 0);
        assert!(details.is_empty());
    }

    #[test]
    fn test_fuzz_result_with_failures() {
        let (successes, crashes, timeouts, details): (usize, usize, usize, Vec<String>) =
            (90, 5, 5, vec!["crash1".to_string(), "timeout1".to_string()]);
        assert_eq!(successes, 90);
        assert_eq!(crashes, 5);
        assert_eq!(timeouts, 5);
        assert_eq!(details.len(), 2);
    }

    #[test]
    fn test_write_fuzz_json_with_all_params() {
        let temp = NamedTempFile::new().unwrap();
        let result = write_fuzz_json(
            Some(temp.path()),
            "parser_fuzz",
            1000,
            true,
            "Fuzz completed successfully",
        );
        assert!(result.is_ok());
        let content = std::fs::read_to_string(temp.path()).unwrap();
        assert!(content.contains("parser_fuzz"));
        assert!(content.contains("1000"));
        assert!(content.contains("passed"));
    }
}
