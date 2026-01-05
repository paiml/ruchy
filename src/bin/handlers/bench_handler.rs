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
        assert_eq!(lines[0], "file,warmup,iterations,min_ms,max_ms,mean_ms,stddev_ms");
    }
}
