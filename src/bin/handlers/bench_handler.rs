//! Benchmark Command Handler
//!
//! Handles benchmarking of Ruchy files with configurable iterations and warmup.

use anyhow::{Context, Result};
use ruchy::Parser as RuchyParser;
use std::fs;
use std::path::Path;
use std::time::Instant;

/// Handle bench command - benchmark execution of a Ruchy file
///
/// # Arguments
/// * `file` - Path to the Ruchy file to benchmark
/// * `iterations` - Number of benchmark iterations
/// * `warmup` - Number of warmup iterations
/// * `format` - Output format (text, json, csv)
/// * `output` - Optional output file path
/// * `verbose` - Enable verbose output
///
/// # Errors
/// Returns error if file cannot be read, parsed, or executed
pub fn handle_bench_command(
    file: &Path,
    iterations: usize,
    warmup: usize,
    format: &str,
    output: Option<&Path>,
    verbose: bool,
) -> Result<()> {
    // Read and parse the file
    let source = fs::read_to_string(file)
        .with_context(|| format!("Failed to read file: {}", file.display()))?;

    // Parse to validate syntax
    let _parser = RuchyParser::new(&source);

    if verbose {
        println!("📊 Benchmarking: {}", file.display());
        println!("🔥 Warmup: {} iterations", warmup);
        println!("🏃 Benchmark: {} iterations", iterations);
    }

    // Warmup phase
    if verbose && warmup > 0 {
        println!("\n⏱️  Running warmup...");
    }
    for i in 0..warmup {
        let mut repl = super::create_repl()?;
        repl.eval(&source)?;
        if verbose {
            println!("  Warmup iteration {}/{}", i + 1, warmup);
        }
    }

    // Benchmark phase
    if verbose {
        println!("\n⏱️  Running benchmark...");
    }

    let mut timings = Vec::with_capacity(iterations);
    for i in 0..iterations {
        let start = Instant::now();
        let mut repl = super::create_repl()?;
        repl.eval(&source)?;
        let duration = start.elapsed();
        timings.push(duration.as_secs_f64() * 1000.0); // Convert to milliseconds

        if verbose {
            println!("  Iteration {}/{}: {:.3} ms", i + 1, iterations, timings[i]);
        }
    }

    // Calculate statistics
    let min = timings.iter().copied().fold(f64::INFINITY, f64::min);
    let max = timings.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let sum: f64 = timings.iter().sum();
    let mean = sum / timings.len() as f64;

    // Calculate standard deviation
    let variance: f64 =
        timings.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / timings.len() as f64;
    let stddev = variance.sqrt();

    // Generate output based on format
    let report = match format {
        "json" => {
            generate_bench_json_output(file, iterations, warmup, &timings, min, max, mean, stddev)
        }
        "csv" => {
            generate_bench_csv_output(file, iterations, warmup, &timings, min, max, mean, stddev)
        }
        _ => generate_bench_text_output(file, iterations, warmup, &timings, min, max, mean, stddev),
    };

    // Write output
    if let Some(output_path) = output {
        fs::write(output_path, &report)
            .with_context(|| format!("Failed to write output to: {}", output_path.display()))?;
        if verbose {
            println!("\n💾 Results saved to: {}", output_path.display());
        }
    } else {
        println!("{}", report);
    }

    Ok(())
}

/// Generate text format benchmark output
pub fn generate_bench_text_output(
    file: &Path,
    iterations: usize,
    warmup: usize,
    _timings: &[f64],
    min: f64,
    max: f64,
    mean: f64,
    stddev: f64,
) -> String {
    format!(
        "=== Benchmark Results ===\n\
         File: {}\n\
         Warmup: {} iterations\n\
         Benchmark: {} iterations\n\
         \n\
         Statistics:\n\
         ├─ Min:     {:.3} ms\n\
         ├─ Max:     {:.3} ms\n\
         ├─ Average: {:.3} ms\n\
         └─ StdDev:  {:.3} ms\n",
        file.display(),
        warmup,
        iterations,
        min,
        max,
        mean,
        stddev
    )
}

/// Generate JSON format benchmark output
pub fn generate_bench_json_output(
    file: &Path,
    iterations: usize,
    warmup: usize,
    timings: &[f64],
    min: f64,
    max: f64,
    mean: f64,
    stddev: f64,
) -> String {
    format!(
        "{{\n\
           \"file\": \"{}\",\n\
           \"warmup\": {},\n\
           \"iterations\": {},\n\
           \"timings_ms\": {:?},\n\
           \"statistics\": {{\n\
             \"min_ms\": {:.3},\n\
             \"max_ms\": {:.3},\n\
             \"mean_ms\": {:.3},\n\
             \"stddev_ms\": {:.3}\n\
           }}\n\
         }}",
        file.display(),
        warmup,
        iterations,
        timings,
        min,
        max,
        mean,
        stddev
    )
}

/// Generate CSV format benchmark output
pub fn generate_bench_csv_output(
    file: &Path,
    iterations: usize,
    warmup: usize,
    _timings: &[f64],
    min: f64,
    max: f64,
    mean: f64,
    stddev: f64,
) -> String {
    format!(
        "file,warmup,iterations,min_ms,max_ms,mean_ms,stddev_ms\n\
         \"{}\",{},{},{:.3},{:.3},{:.3},{:.3}\n",
        file.display(),
        warmup,
        iterations,
        min,
        max,
        mean,
        stddev
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_generate_text_output() {
        let file = Path::new("test.ruchy");
        let output = generate_bench_text_output(file, 10, 3, &[], 1.0, 5.0, 2.5, 1.0);
        assert!(output.contains("Benchmark Results"));
        assert!(output.contains("test.ruchy"));
        assert!(output.contains("10"));
        assert!(output.contains("3"));
    }

    #[test]
    fn test_generate_json_output() {
        let file = Path::new("test.ruchy");
        let timings = vec![1.0, 2.0, 3.0];
        let output = generate_bench_json_output(file, 3, 1, &timings, 1.0, 3.0, 2.0, 0.8);
        assert!(output.contains("\"file\":"));
        assert!(output.contains("\"iterations\":"));
        assert!(output.contains("\"statistics\":"));
    }

    #[test]
    fn test_generate_csv_output() {
        let file = Path::new("test.ruchy");
        let output = generate_bench_csv_output(file, 5, 2, &[], 1.0, 5.0, 2.5, 1.0);
        assert!(output.contains("file,warmup,iterations"));
        assert!(output.contains("test.ruchy"));
    }

    #[test]
    fn test_csv_header() {
        let file = Path::new("any.ruchy");
        let output = generate_bench_csv_output(file, 1, 1, &[], 0.0, 0.0, 0.0, 0.0);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(
            lines[0],
            "file,warmup,iterations,min_ms,max_ms,mean_ms,stddev_ms"
        );
    }

    // ===== EXTREME TDD Round 149 - Bench Handler Tests =====

    #[test]
    fn test_generate_text_output_zero_values() {
        let file = Path::new("test.ruchy");
        let output = generate_bench_text_output(file, 0, 0, &[], 0.0, 0.0, 0.0, 0.0);
        assert!(output.contains("Benchmark Results"));
    }

    #[test]
    fn test_generate_text_output_large_values() {
        let file = Path::new("test.ruchy");
        let output =
            generate_bench_text_output(file, 1000000, 100000, &[], 0.001, 10000.0, 500.0, 100.0);
        assert!(output.contains("Benchmark Results"));
    }

    #[test]
    fn test_generate_json_output_empty_timings() {
        let file = Path::new("test.ruchy");
        let output = generate_bench_json_output(file, 0, 0, &[], 0.0, 0.0, 0.0, 0.0);
        assert!(output.contains("\"timings_ms\": []"));
    }

    #[test]
    fn test_generate_json_output_many_timings() {
        let file = Path::new("test.ruchy");
        let timings: Vec<f64> = (0..100).map(|x| x as f64 * 0.1).collect();
        let output = generate_bench_json_output(file, 100, 10, &timings, 0.0, 9.9, 4.95, 2.0);
        assert!(output.contains("\"iterations\": 100"));
    }

    #[test]
    fn test_generate_csv_output_special_chars_in_path() {
        let file = Path::new("path with spaces/test.ruchy");
        let output = generate_bench_csv_output(file, 5, 2, &[], 1.0, 5.0, 2.5, 1.0);
        assert!(output.contains("path with spaces"));
    }

    #[test]
    fn test_handle_bench_command_nonexistent_file() {
        let result = handle_bench_command(
            Path::new("/nonexistent/file.ruchy"),
            1,
            0,
            "text",
            None,
            false,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_bench_command_basic() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "42").unwrap();
        let result = handle_bench_command(temp.path(), 1, 0, "text", None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_bench_command_with_warmup() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "42").unwrap();
        let result = handle_bench_command(
            temp.path(),
            2,
            1, // warmup
            "text",
            None,
            false,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_bench_command_json_format() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "42").unwrap();
        let result = handle_bench_command(temp.path(), 1, 0, "json", None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_bench_command_csv_format() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "42").unwrap();
        let result = handle_bench_command(temp.path(), 1, 0, "csv", None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_bench_command_with_output() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "42").unwrap();
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("output.txt");
        let result = handle_bench_command(temp.path(), 1, 0, "text", Some(&output_path), false);
        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_handle_bench_command_verbose() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "42").unwrap();
        let result = handle_bench_command(
            temp.path(),
            1,
            0,
            "text",
            None,
            true, // verbose
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_bench_command_multiple_iterations() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "42").unwrap();
        let result = handle_bench_command(
            temp.path(),
            5, // multiple iterations
            2,
            "text",
            None,
            false,
        );
        assert!(result.is_ok());
    }
}
