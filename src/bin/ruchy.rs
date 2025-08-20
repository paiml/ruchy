#![allow(clippy::print_stdout)]
#![allow(clippy::print_stderr)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::format_push_string)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::fn_params_excessive_bools)]
#![allow(clippy::too_many_lines)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use ruchy::{runtime::repl::Repl, Parser as RuchyParser, Transpiler, ExprKind};
use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(name = "ruchy")]
#[command(author, version, about = "The Ruchy programming language", long_about = None)]
struct Cli {
    /// Evaluate a one-liner expression
    #[arg(short = 'e', long = "eval", value_name = "EXPR")]
    eval: Option<String>,

    /// Output format for evaluation results (text, json)
    #[arg(long, default_value = "text")]
    format: String,

    /// Enable verbose output
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Script file to execute (alternative to subcommands)
    file: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the interactive REPL
    Repl,

    /// Parse a Ruchy file and show the AST
    Parse {
        /// The file to parse
        file: PathBuf,
    },

    /// Transpile a Ruchy file to Rust
    Transpile {
        /// The file to transpile
        file: PathBuf,

        /// Output file (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Compile and run a Ruchy file
    Run {
        /// The file to run
        file: PathBuf,
    },

    /// Check syntax without running
    Check {
        /// The file to check
        file: PathBuf,
        
        /// Watch for changes and re-check automatically
        #[arg(long)]
        watch: bool,
    },
    
    /// Run tests for Ruchy code
    Test {
        /// The test file or directory to run
        path: Option<PathBuf>,
        
        /// Watch for changes and re-run tests automatically
        #[arg(long)]
        watch: bool,
        
        /// Show verbose output
        #[arg(long)]
        verbose: bool,
        
        /// Filter tests by name pattern
        #[arg(long)]
        filter: Option<String>,
    },

    /// Show AST for a file
    Ast {
        /// The file to parse
        file: PathBuf,
    },

    /// Format Ruchy source code
    Fmt {
        /// The file to format
        file: PathBuf,

        /// Format all files in project
        #[arg(long)]
        all: bool,

        /// Check if files are formatted without modifying them
        #[arg(long)]
        check: bool,

        /// Write formatted output to stdout instead of modifying files
        #[arg(long)]
        stdout: bool,

        /// Show diff of changes
        #[arg(long)]
        diff: bool,
    },

    /// Generate documentation from Ruchy source code
    Doc {
        /// The file or directory to document
        path: PathBuf,

        /// Output directory for generated documentation
        #[arg(long, default_value = "./docs")]
        output: PathBuf,

        /// Documentation format (html, markdown, json)
        #[arg(long, default_value = "html")]
        format: String,

        /// Include private items in documentation
        #[arg(long)]
        private: bool,

        /// Open documentation in browser after generation
        #[arg(long)]
        open: bool,

        /// Generate documentation for all files in project
        #[arg(long)]
        all: bool,

        /// Show verbose output
        #[arg(long)]
        verbose: bool,
    },

    /// Benchmark Ruchy code performance
    Bench {
        /// The file to benchmark
        file: PathBuf,

        /// Number of iterations to run
        #[arg(long, default_value = "100")]
        iterations: usize,

        /// Number of warmup iterations
        #[arg(long, default_value = "10")]
        warmup: usize,

        /// Output format (text, json, csv)
        #[arg(long, default_value = "text")]
        format: String,

        /// Save results to file
        #[arg(long)]
        output: Option<PathBuf>,

        /// Show verbose output including individual runs
        #[arg(long)]
        verbose: bool,
    },

    /// Lint Ruchy source code for issues and style violations
    Lint {
        /// The file to lint (ignored if --all is used)
        file: Option<PathBuf>,

        /// Lint all files in project
        #[arg(long)]
        all: bool,

        /// Show additional context for violations
        #[arg(long)]
        verbose: bool,

        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        format: String,

        /// Fail on warnings as well as errors
        #[arg(long)]
        deny_warnings: bool,

        /// Maximum allowed complexity for functions
        #[arg(long, default_value = "10")]
        max_complexity: usize,
        
        /// Path to custom lint rules configuration file
        #[arg(long)]
        config: Option<PathBuf>,
        
        /// Generate default lint configuration file
        #[arg(long)]
        init_config: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle one-liner evaluation with -e flag
    if let Some(expr) = cli.eval {
        if cli.verbose {
            eprintln!("Parsing expression: {expr}");
        }

        let mut repl = Repl::new()?;
        match repl.eval(&expr) {
            Ok(result) => {
                if cli.verbose {
                    eprintln!("Evaluation successful");
                }

                if cli.format == "json" {
                    // Output as JSON
                    println!(
                        "{}",
                        serde_json::json!({
                            "success": true,
                            "result": format!("{result}")
                        })
                    );
                } else {
                    // Default text output
                    println!("{result}");
                }
                return Ok(());
            }
            Err(e) => {
                if cli.verbose {
                    eprintln!("Evaluation failed: {e}");
                }

                if cli.format == "json" {
                    println!(
                        "{}",
                        serde_json::json!({
                            "success": false,
                            "error": e.to_string()
                        })
                    );
                } else {
                    eprintln!("Error: {e}");
                }
                std::process::exit(1);
            }
        }
    }

    // Handle script file execution (without subcommand)
    if let Some(file) = cli.file {
        return run_file(&file);
    }

    // Check if stdin has input (piped mode) - but only for non-REPL commands
    if !io::stdin().is_terminal() && !matches!(cli.command, Some(Commands::Repl)) {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;

        if !input.trim().is_empty() {
            let mut repl = Repl::new()?;
            match repl.eval(&input) {
                Ok(result) => {
                    println!("{result}");
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }
    }

    // Handle subcommands
    match cli.command {
        Some(Commands::Repl) | None => {
            let version_msg = format!("Welcome to Ruchy REPL v{}", env!("CARGO_PKG_VERSION"));
            println!("{}", version_msg.bright_cyan().bold());
            println!(
                "Type {} for commands, {} to exit\n",
                ":help".green(),
                ":quit".yellow()
            );

            let mut repl = Repl::new()?;
            repl.run()?;
        }
        Some(Commands::Parse { file }) => {
            let source = fs::read_to_string(&file)?;
            let mut parser = RuchyParser::new(&source);
            match parser.parse() {
                Ok(ast) => println!("{ast:#?}"),
                Err(e) => {
                    eprintln!("Parse error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Transpile { file, output }) => {
            let source = if file.as_os_str() == "-" {
                // Read from stdin
                let mut input = String::new();
                io::stdin().read_to_string(&mut input)?;
                input
            } else {
                fs::read_to_string(&file)?
            };
            
            let mut parser = RuchyParser::new(&source);
            let ast = parser.parse()?;
            let transpiler = Transpiler::new();
            let rust_code = transpiler.transpile(&ast)?;
            let rust_code_str = rust_code.to_string();

            if let Some(output_path) = output {
                fs::write(output_path, rust_code_str)?;
            } else {
                println!("{rust_code_str}");
            }
        }
        Some(Commands::Run { file }) => {
            run_file(&file)?;
        }
        Some(Commands::Check { file, watch }) => {
            if watch {
                watch_and_check(&file)?;
            } else {
                check_syntax(&file)?;
            }
        }
        Some(Commands::Test { path, watch, verbose, filter }) => {
            if watch {
                let test_path = path.unwrap_or_else(|| PathBuf::from("."));
                watch_and_test(&test_path, verbose, filter.as_deref())?;
            } else {
                let test_path = path.unwrap_or_else(|| PathBuf::from("."));
                run_tests(&test_path, verbose, filter.as_deref())?;
            }
        }
        Some(Commands::Ast { file }) => {
            let source = fs::read_to_string(&file)?;
            let mut parser = RuchyParser::new(&source);
            match parser.parse() {
                Ok(ast) => println!("{ast:#?}"),
                Err(e) => {
                    eprintln!("Parse error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Fmt { file, all, check, stdout, diff }) => {
            format_ruchy_code(&file, all, check, stdout, diff)?;
        }
        Some(Commands::Doc { path, output, format, private, open, all, verbose }) => {
            generate_documentation(&path, &output, &format, private, open, all, verbose)?;
        }
        Some(Commands::Bench { file, iterations, warmup, format, output, verbose }) => {
            benchmark_ruchy_code(&file, iterations, warmup, &format, output.as_deref(), verbose)?;
        }
        Some(Commands::Lint { file, all, verbose, format, deny_warnings, max_complexity, config, init_config }) => {
            if init_config {
                generate_default_lint_config()?;
            } else {
                // Load custom rules if config provided
                let custom_rules = if let Some(config_path) = config {
                    load_custom_lint_rules(&config_path)?
                } else {
                    CustomLintRules::default()
                };
                
                if all {
                    lint_ruchy_code(&PathBuf::from("."), all, verbose, &format, deny_warnings, max_complexity, &custom_rules)?;
                } else if let Some(file) = file {
                    lint_ruchy_code(&file, false, verbose, &format, deny_warnings, max_complexity, &custom_rules)?;
                } else {
                    eprintln!("Error: Either provide a file or use --all flag");
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

fn run_file(file: &Path) -> Result<()> {
    let source = fs::read_to_string(file)?;

    // Use REPL to evaluate the file
    let mut repl = Repl::new()?;
    match repl.eval(&source) {
        Ok(result) => {
            // Only print non-unit results
            if result != "Unit" && result != "()" {
                println!("{result}");
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

/// Check syntax of a file
fn check_syntax(file: &Path) -> Result<()> {
    let source = fs::read_to_string(file)?;
    let mut parser = RuchyParser::new(&source);
    match parser.parse() {
        Ok(_) => {
            println!("{}", "✓ Syntax is valid".green());
            Ok(())
        }
        Err(e) => {
            eprintln!("{}", format!("✗ Syntax error: {e}").red());
            std::process::exit(1);
        }
    }
}

/// Watch a file and check syntax on changes
fn watch_and_check(file: &Path) -> Result<()> {
    println!("{} Watching {} for changes...", "👁".bright_cyan(), file.display());
    println!("Press Ctrl+C to stop watching\n");
    
    // Initial check
    check_syntax(file)?;
    
    // Simple file watching using polling
    let mut last_modified = fs::metadata(file)?.modified()?;
    
    loop {
        thread::sleep(Duration::from_millis(500));
        
        let Ok(metadata) = fs::metadata(file) else {
            continue; // File might be temporarily unavailable
        };
        
        let Ok(modified) = metadata.modified() else {
            continue;
        };
        
        if modified != last_modified {
            last_modified = modified;
            println!("\n{} File changed, checking...", "→".bright_cyan());
            let _ = check_syntax(file); // Don't exit on error, keep watching
        }
    }
}

/// Run tests from a path
fn run_tests(path: &Path, verbose: bool, filter: Option<&str>) -> Result<()> {
    println!("{} Running tests...", "🧪".bright_cyan());
    
    // Discover test files
    let test_files = if path.is_dir() {
        discover_test_files(path)?
    } else {
        vec![path.to_path_buf()]
    };
    
    if test_files.is_empty() {
        println!("{} No test files found", "⚠".yellow());
        return Ok(());
    }
    
    let mut total_tests = 0;
    let mut passed_tests = 0;
    let mut failed_tests = 0;
    
    for test_file in &test_files {
        if verbose {
            println!("\n{} Running {}", "→".bright_cyan(), test_file.display());
        }
        
        let source = fs::read_to_string(test_file)?;
        
        // Parse and find test functions
        let mut parser = RuchyParser::new(&source);
        match parser.parse() {
            Ok(ast) => {
                let tests = extract_test_functions(&ast, filter);
                
                for test in tests {
                    total_tests += 1;
                    
                    if verbose {
                        print!("  {} {}... ", "→".bright_cyan(), test.name);
                    }
                    
                    // Run test in REPL
                    let mut repl = Repl::new()?;
                    match repl.eval(&source) {
                        Ok(_) => {
                            passed_tests += 1;
                            if verbose {
                                println!("{}", "✓".green());
                            } else {
                                print!("{}", ".".green());
                            }
                        }
                        Err(e) => {
                            failed_tests += 1;
                            if verbose {
                                println!("{} {}", "✗".red(), e);
                            } else {
                                print!("{}", "F".red());
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("{} Parse error in {}: {e}", "✗".bright_red(), test_file.display());
                failed_tests += 1;
            }
        }
    }
    
    // Print summary
    println!("\n\n{}", "─".repeat(40));
    let status = if failed_tests == 0 {
        format!("{} All tests passed!", "✓".bright_green())
    } else {
        format!("{} Some tests failed", "✗".bright_red())
    };
    
    println!("{status}");
    println!("Total: {total_tests}, Passed: {}, Failed: {}", 
        format!("{passed_tests}").green(),
        if failed_tests > 0 { format!("{failed_tests}").red() } else { format!("{failed_tests}").white() }
    );
    
    if failed_tests > 0 {
        std::process::exit(1);
    }
    
    Ok(())
}

/// Watch and run tests on changes
fn watch_and_test(path: &Path, verbose: bool, filter: Option<&str>) -> Result<()> {
    println!("{} Watching {} for changes...", "👁".bright_cyan(), path.display());
    println!("Press Ctrl+C to stop watching\n");
    
    // Initial test run
    let _ = run_tests(path, verbose, filter);
    
    // Watch for changes
    let mut last_run = Instant::now();
    
    loop {
        thread::sleep(Duration::from_millis(500));
        
        // Check if any test files have changed
        let test_files = if path.is_dir() {
            discover_test_files(path)?
        } else {
            vec![path.to_path_buf()]
        };
        
        let mut should_run = false;
        for file in &test_files {
            if let Ok(metadata) = fs::metadata(file) {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(duration) = modified.duration_since(std::time::SystemTime::UNIX_EPOCH) {
                        let file_time = Instant::now().checked_sub(Duration::from_secs(duration.as_secs())).unwrap();
                        if file_time > last_run {
                            should_run = true;
                            break;
                        }
                    }
                }
            }
        }
        
        if should_run {
            last_run = Instant::now();
            println!("\n{} Files changed, re-running tests...", "→".bright_cyan());
            let _ = run_tests(path, verbose, filter);
        }
    }
}

/// Test function info
struct TestFunction {
    name: String,
}

/// Extract test functions from AST
fn extract_test_functions(ast: &ruchy::Expr, filter: Option<&str>) -> Vec<TestFunction> {
    let mut tests = Vec::new();
    extract_tests_recursive(ast, &mut tests, filter);
    tests
}

/// Recursively extract test functions
fn extract_tests_recursive(ast: &ruchy::Expr, tests: &mut Vec<TestFunction>, filter: Option<&str>) {
    match &ast.kind {
        ExprKind::Function { name, .. } => {
            // Test functions start with "test_" or have @test attribute
            if name.starts_with("test_") || ast.attributes.iter().any(|a| a.name == "test") {
                if let Some(f) = filter {
                    if !name.contains(f) {
                        return;
                    }
                }
                tests.push(TestFunction {
                    name: name.clone(),
                });
            }
        }
        ExprKind::Block(exprs) => {
            for expr in exprs {
                extract_tests_recursive(expr, tests, filter);
            }
        }
        _ => {}
    }
}

/// Discover test files in a directory
fn discover_test_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut test_files = Vec::new();
    visit_dir_for_tests(dir, &mut test_files)?;
    test_files.sort();
    Ok(test_files)
}

/// Recursively visit directories to find test files
fn visit_dir_for_tests(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                // Skip hidden directories and common build/dependency directories
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    if !name.starts_with('.') 
                        && name != "target" 
                        && name != "node_modules" 
                        && name != "build" 
                        && name != "dist" {
                        visit_dir_for_tests(&path, files)?;
                    }
                }
            } else if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                // Test files end with _test.ruchy or test.ruchy or are in a tests/ directory
                if (name.ends_with("_test.ruchy") || name.ends_with("test.ruchy") || 
                    path.parent().and_then(|p| p.file_name()).and_then(|s| s.to_str()) == Some("tests"))
                    && path.extension().and_then(|s| s.to_str()) == Some("ruchy") {
                    files.push(path);
                }
            }
        }
    }
    Ok(())
}

/// Format Ruchy code
#[allow(clippy::fn_params_excessive_bools)]
fn format_ruchy_code(file: &Path, all: bool, check: bool, stdout: bool, diff: bool) -> Result<()> {
    if all {
        // Format all .ruchy files
        let ruchy_files = discover_ruchy_files(".")?;
        
        if ruchy_files.is_empty() {
            println!("{} No .ruchy files found", "⚠".yellow());
            return Ok(());
        }
        
        println!("{} Found {} .ruchy files", "→".bright_cyan(), ruchy_files.len());
        
        let mut formatted_count = 0;
        let mut errors = 0;
        
        for file in &ruchy_files {
            match format_single_file(file, check, false, diff) {
                Ok(was_formatted) => {
                    if was_formatted {
                        formatted_count += 1;
                    }
                }
                Err(_) => {
                    errors += 1;
                }
            }
        }
        
        // Print summary
        let status = if errors == 0 {
            format!("{} PASSED", "✓".bright_green())
        } else {
            format!("{} FAILED", "✗".bright_red())
        };
        
        println!("\nformat result: {}. {} files processed; {} formatted, {} errors",
            status, ruchy_files.len(), formatted_count, errors);
        
        if errors > 0 {
            std::process::exit(1);
        }
    } else {
        // Format single file
        format_single_file(file, check, stdout, diff)?;
    }
    
    Ok(())
}

/// Format a single file
fn format_single_file(file: &Path, check: bool, stdout: bool, diff: bool) -> Result<bool> {
    let source = fs::read_to_string(file)?;
    let mut parser = RuchyParser::new(&source);
    
    match parser.parse() {
        Ok(ast) => {
            // For now, use debug formatting as a placeholder for proper formatting
            // In a full implementation, this would traverse the AST and produce
            // properly formatted Ruchy code
            let formatted = format_ast(&ast);
            
            if check {
                if source.trim() == formatted.trim() {
                    println!("{} {} is already formatted", "✓".bright_green(), file.display());
                    Ok(false)
                } else {
                    println!("{} {} needs formatting", "✗".bright_red(), file.display());
                    if diff {
                        print_diff(&source, &formatted, file);
                    }
                    std::process::exit(1);
                }
            } else if stdout {
                println!("{formatted}");
                Ok(true)
            } else {
                // Write back to file
                if source.trim() == formatted.trim() {
                    println!("{} {} is already formatted", "→".bright_cyan(), file.display());
                    Ok(false)
                } else {
                    fs::write(file, &formatted)?;
                    println!("{} Formatted {}", "✓".bright_green(), file.display());
                    if diff {
                        print_diff(&source, &formatted, file);
                    }
                    Ok(true)
                }
            }
        }
        Err(e) => {
            eprintln!("{} Parse error in {}: {e}", "✗".bright_red(), file.display());
            std::process::exit(1);
        }
    }
}

/// Format an AST to a string (placeholder implementation)
fn format_ast(ast: &ruchy::Expr) -> String {
    // For now, use debug representation
    // A full implementation would traverse the AST and produce formatted Ruchy code
    format!("{ast:#?}\n")
}

/// Print diff between original and formatted content
fn print_diff(original: &str, formatted: &str, file: &Path) {
    println!("\n{} Diff for {}:", "📝".bright_blue(), file.display());
    println!("{}", "--- Original".bright_red());
    println!("{}", "+++ Formatted".bright_green());
    
    let original_lines: Vec<&str> = original.lines().collect();
    let formatted_lines: Vec<&str> = formatted.lines().collect();
    
    // Simple diff display - just show different lines
    let max_lines = original_lines.len().max(formatted_lines.len());
    for i in 0..max_lines {
        let orig = original_lines.get(i).unwrap_or(&"");
        let fmt = formatted_lines.get(i).unwrap_or(&"");
        
        if orig != fmt {
            if !orig.is_empty() {
                println!("{} {}", "-".bright_red(), orig);
            }
            if !fmt.is_empty() {
                println!("{} {}", "+".bright_green(), fmt);
            }
        }
    }
    println!();
}

/// Benchmark Ruchy code performance
fn benchmark_ruchy_code(
    file: &Path,
    iterations: usize,
    warmup: usize,
    format: &str,
    output: Option<&Path>,
    verbose: bool,
) -> Result<()> {
    let source = fs::read_to_string(file)?;
    
    println!("{} Benchmarking: {}", "→".bright_cyan(), file.display());
    println!("  Iterations: {iterations}, Warmup: {warmup}");
    
    // Warmup phase
    if verbose {
        println!("\n{} Warmup phase...", "→".bright_cyan());
    }
    for i in 0..warmup {
        let mut parser = RuchyParser::new(&source);
        let _ = parser.parse();
        if verbose {
            println!("  Warmup {}/{warmup}", i + 1);
        }
    }
    
    // Benchmark parsing
    let mut parse_times = Vec::new();
    if verbose {
        println!("\n{} Parse benchmark...", "→".bright_cyan());
    }
    
    for i in 0..iterations {
        let start = Instant::now();
        let mut parser = RuchyParser::new(&source);
        match parser.parse() {
            Ok(_) => {
                let elapsed = start.elapsed();
                parse_times.push(elapsed);
                if verbose {
                    println!("  Run {}/{iterations}: {:?}", i + 1, elapsed);
                }
            }
            Err(e) => {
                eprintln!("{} Parse error: {e}", "✗".bright_red());
                std::process::exit(1);
            }
        }
    }
    
    // Benchmark transpilation
    let mut transpile_times = Vec::new();
    if verbose {
        println!("\n{} Transpile benchmark...", "→".bright_cyan());
    }
    
    for i in 0..iterations {
        let mut parser = RuchyParser::new(&source);
        let ast = parser.parse()?;
        
        let start = Instant::now();
        let transpiler = Transpiler::new();
        let _ = transpiler.transpile(&ast);
        let elapsed = start.elapsed();
        
        transpile_times.push(elapsed);
        if verbose {
            println!("  Run {}/{iterations}: {:?}", i + 1, elapsed);
        }
    }
    
    // Calculate statistics
    let parse_stats = calculate_stats(&parse_times);
    let transpile_stats = calculate_stats(&transpile_times);
    
    // Display results
    match format {
        "json" => display_bench_json(&parse_stats, &transpile_stats, file, iterations),
        "csv" => display_bench_csv(&parse_stats, &transpile_stats, file, iterations),
        _ => display_bench_text(&parse_stats, &transpile_stats, file, iterations, &source),
    }
    
    // Save to file if requested
    if let Some(output_path) = output {
        save_bench_results(output_path, &parse_stats, &transpile_stats, file, iterations, format)?;
        println!("\n{} Results saved to {}", "✓".bright_green(), output_path.display());
    }
    
    Ok(())
}

/// Statistics for benchmark results
#[derive(Debug)]
#[allow(dead_code)]
struct BenchStats {
    mean: Duration,
    median: Duration,
    min: Duration,
    max: Duration,
    std_dev: Duration,
    throughput_mb_per_sec: f64,
}

/// Calculate statistics from timing data
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
fn calculate_stats(times: &[Duration]) -> BenchStats {
    if times.is_empty() {
        return BenchStats {
            mean: Duration::ZERO,
            median: Duration::ZERO,
            min: Duration::ZERO,
            max: Duration::ZERO,
            std_dev: Duration::ZERO,
            throughput_mb_per_sec: 0.0,
        };
    }
    
    let mut sorted_times = times.to_vec();
    sorted_times.sort();
    
    let sum: Duration = times.iter().sum();
    let mean = sum / times.len() as u32;
    
    let median = if times.len() % 2 == 0 {
        (sorted_times[times.len() / 2 - 1] + sorted_times[times.len() / 2]) / 2
    } else {
        sorted_times[times.len() / 2]
    };
    
    let min = sorted_times.first().copied().unwrap_or(Duration::ZERO);
    let max = sorted_times.last().copied().unwrap_or(Duration::ZERO);
    
    // Calculate standard deviation
    let variance: f64 = times.iter()
        .map(|t| {
            let diff = t.as_secs_f64() - mean.as_secs_f64();
            diff * diff
        })
        .sum::<f64>() / times.len() as f64;
    
    let std_dev = Duration::from_secs_f64(variance.sqrt());
    
    // Placeholder for throughput (would need file size)
    let throughput_mb_per_sec = 0.0;
    
    BenchStats {
        mean,
        median,
        min,
        max,
        std_dev,
        throughput_mb_per_sec,
    }
}

/// Display benchmark results in text format
#[allow(clippy::cast_precision_loss)]
fn display_bench_text(
    parse_stats: &BenchStats,
    transpile_stats: &BenchStats,
    file: &Path,
    iterations: usize,
    source: &str,
) {
    let source_lines = source.lines().count();
    let source_bytes = source.len();
    
    println!("\n{} Benchmark Results", "📊".bright_blue());
    println!("  File: {}", file.display());
    println!("  Size: {source_bytes} bytes, {source_lines} lines");
    println!("  Iterations: {iterations}");
    
    println!("\n{} Parse Performance:", "→".bright_cyan());
    println!("  Mean:     {:?}", parse_stats.mean);
    println!("  Median:   {:?}", parse_stats.median);
    println!("  Min:      {:?}", parse_stats.min);
    println!("  Max:      {:?}", parse_stats.max);
    println!("  Std Dev:  {:?}", parse_stats.std_dev);
    
    if source_bytes > 0 {
        let throughput = source_bytes as f64 / parse_stats.mean.as_secs_f64() / 1_000_000.0;
        println!("  Throughput: {throughput:.2} MB/s");
    }
    
    println!("\n{} Transpile Performance:", "→".bright_cyan());
    println!("  Mean:     {:?}", transpile_stats.mean);
    println!("  Median:   {:?}", transpile_stats.median);
    println!("  Min:      {:?}", transpile_stats.min);
    println!("  Max:      {:?}", transpile_stats.max);
    println!("  Std Dev:  {:?}", transpile_stats.std_dev);
    
    if source_bytes > 0 {
        let throughput = source_bytes as f64 / transpile_stats.mean.as_secs_f64() / 1_000_000.0;
        println!("  Throughput: {throughput:.2} MB/s");
    }
    
    println!("\n{} Total Time:", "→".bright_cyan());
    let total_mean = parse_stats.mean + transpile_stats.mean;
    println!("  Mean:     {:?}", total_mean);
    
    if source_lines > 0 {
        let lines_per_sec = source_lines as f64 / total_mean.as_secs_f64();
        println!("  Lines/sec: {:.0}", lines_per_sec);
    }
}

/// Display benchmark results in JSON format
fn display_bench_json(
    parse_stats: &BenchStats,
    transpile_stats: &BenchStats,
    file: &Path,
    iterations: usize,
) {
    let result = serde_json::json!({
        "file": file.display().to_string(),
        "iterations": iterations,
        "parse": {
            "mean_ms": parse_stats.mean.as_millis(),
            "median_ms": parse_stats.median.as_millis(),
            "min_ms": parse_stats.min.as_millis(),
            "max_ms": parse_stats.max.as_millis(),
            "std_dev_ms": parse_stats.std_dev.as_millis(),
        },
        "transpile": {
            "mean_ms": transpile_stats.mean.as_millis(),
            "median_ms": transpile_stats.median.as_millis(),
            "min_ms": transpile_stats.min.as_millis(),
            "max_ms": transpile_stats.max.as_millis(),
            "std_dev_ms": transpile_stats.std_dev.as_millis(),
        },
        "total": {
            "mean_ms": (parse_stats.mean + transpile_stats.mean).as_millis(),
        }
    });
    
    println!("{}", serde_json::to_string_pretty(&result).unwrap_or_else(|_| "Invalid JSON".to_string()));
}

/// Display benchmark results in CSV format
fn display_bench_csv(
    parse_stats: &BenchStats,
    transpile_stats: &BenchStats,
    file: &Path,
    iterations: usize,
) {
    println!("file,iterations,parse_mean_ms,parse_median_ms,parse_min_ms,parse_max_ms,transpile_mean_ms,transpile_median_ms,transpile_min_ms,transpile_max_ms");
    println!("{},{},{},{},{},{},{},{},{},{}",
        file.display(),
        iterations,
        parse_stats.mean.as_millis(),
        parse_stats.median.as_millis(),
        parse_stats.min.as_millis(),
        parse_stats.max.as_millis(),
        transpile_stats.mean.as_millis(),
        transpile_stats.median.as_millis(),
        transpile_stats.min.as_millis(),
        transpile_stats.max.as_millis(),
    );
}

/// Save benchmark results to file
fn save_bench_results(
    output_path: &Path,
    parse_stats: &BenchStats,
    transpile_stats: &BenchStats,
    file: &Path,
    iterations: usize,
    format: &str,
) -> Result<()> {
    let content = match format {
        "json" => {
            let result = serde_json::json!({
                "file": file.display().to_string(),
                "iterations": iterations,
                "parse": {
                    "mean_ms": parse_stats.mean.as_millis(),
                    "median_ms": parse_stats.median.as_millis(),
                    "min_ms": parse_stats.min.as_millis(),
                    "max_ms": parse_stats.max.as_millis(),
                },
                "transpile": {
                    "mean_ms": transpile_stats.mean.as_millis(),
                    "median_ms": transpile_stats.median.as_millis(),
                    "min_ms": transpile_stats.min.as_millis(),
                    "max_ms": transpile_stats.max.as_millis(),
                }
            });
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "Invalid JSON".to_string())
        }
        "csv" => {
            format!("file,iterations,parse_mean_ms,parse_median_ms,transpile_mean_ms,transpile_median_ms\n{},{},{},{},{},{}",
                file.display(),
                iterations,
                parse_stats.mean.as_millis(),
                parse_stats.median.as_millis(),
                transpile_stats.mean.as_millis(),
                transpile_stats.median.as_millis(),
            )
        }
        _ => {
            format!("Benchmark Results\nFile: {}\nIterations: {}\n\nParse:\n  Mean: {:?}\n  Median: {:?}\n\nTranspile:\n  Mean: {:?}\n  Median: {:?}",
                file.display(),
                iterations,
                parse_stats.mean,
                parse_stats.median,
                transpile_stats.mean,
                transpile_stats.median,
            )
        }
    };
    
    fs::write(output_path, content)?;
    Ok(())
}

/// Represents a lint severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum LintSeverity {
    Error,
    Warning,
    Info,
}

/// Represents a lint violation
#[derive(Debug, Clone)]
struct LintViolation {
    severity: LintSeverity,
    rule: String,
    message: String,
    line: usize,
    column: usize,
    suggestion: Option<String>,
}

/// Custom lint rules configuration
#[derive(Debug, Clone, Default)]
struct CustomLintRules {
    rules: Vec<CustomRule>,
    disabled_rules: Vec<String>,
    custom_patterns: Vec<PatternRule>,
}

/// A custom lint rule
#[derive(Debug, Clone)]
struct CustomRule {
    name: String,
    severity: LintSeverity,
    enabled: bool,
    config: serde_json::Value,
}

/// A pattern-based lint rule
#[derive(Debug, Clone)]
struct PatternRule {
    name: String,
    pattern: String,
    message: String,
    severity: LintSeverity,
    suggestion: Option<String>,
}

/// Load custom lint rules from configuration file
fn load_custom_lint_rules(config_path: &Path) -> Result<CustomLintRules> {
    let content = fs::read_to_string(config_path)?;
    let config: serde_json::Value = serde_json::from_str(&content)?;
    
    let mut rules = CustomLintRules::default();
    
    // Parse disabled rules
    if let Some(disabled) = config.get("disabled_rules").and_then(|v| v.as_array()) {
        for rule in disabled {
            if let Some(name) = rule.as_str() {
                rules.disabled_rules.push(name.to_string());
            }
        }
    }
    
    // Parse pattern rules
    if let Some(patterns) = config.get("pattern_rules").and_then(|v| v.as_array()) {
        for pattern_config in patterns {
            if let (Some(name), Some(pattern), Some(message)) = (
                pattern_config.get("name").and_then(|v| v.as_str()),
                pattern_config.get("pattern").and_then(|v| v.as_str()),
                pattern_config.get("message").and_then(|v| v.as_str()),
            ) {
                let severity = pattern_config.get("severity")
                    .and_then(|v| v.as_str())
                    .and_then(|s| match s {
                        "error" => Some(LintSeverity::Error),
                        "warning" => Some(LintSeverity::Warning),
                        "info" => Some(LintSeverity::Info),
                        _ => None,
                    })
                    .unwrap_or(LintSeverity::Warning);
                
                let suggestion = pattern_config.get("suggestion")
                    .and_then(|v| v.as_str())
                    .map(std::string::ToString::to_string);
                
                rules.custom_patterns.push(PatternRule {
                    name: name.to_string(),
                    pattern: pattern.to_string(),
                    message: message.to_string(),
                    severity,
                    suggestion,
                });
            }
        }
    }
    
    // Parse custom rules
    if let Some(custom) = config.get("custom_rules").and_then(|v| v.as_array()) {
        for rule_config in custom {
            if let Some(name) = rule_config.get("name").and_then(|v| v.as_str()) {
                let enabled = rule_config.get("enabled")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true);
                
                let severity = rule_config.get("severity")
                    .and_then(|v| v.as_str())
                    .and_then(|s| match s {
                        "error" => Some(LintSeverity::Error),
                        "warning" => Some(LintSeverity::Warning),
                        "info" => Some(LintSeverity::Info),
                        _ => None,
                    })
                    .unwrap_or(LintSeverity::Warning);
                
                rules.rules.push(CustomRule {
                    name: name.to_string(),
                    severity,
                    enabled,
                    config: rule_config.clone(),
                });
            }
        }
    }
    
    Ok(rules)
}

/// Generate a default lint configuration file
fn generate_default_lint_config() -> Result<()> {
    let default_config = r#"{
  "disabled_rules": [
    "unused_variables"
  ],
  "pattern_rules": [
    {
      "name": "no_debug_print",
      "pattern": "debug_print|dbg!",
      "message": "Debug print statements should not be committed",
      "severity": "warning",
      "suggestion": "Remove debug print statement or use proper logging"
    },
    {
      "name": "no_panic",
      "pattern": "panic!",
      "message": "Avoid using panic! in production code",
      "severity": "error",
      "suggestion": "Use Result type for error handling instead"
    }
  ],
  "custom_rules": [
    {
      "name": "max_line_length",
      "enabled": true,
      "severity": "warning",
      "max_length": 120
    },
    {
      "name": "max_function_length",
      "enabled": true,
      "severity": "warning",
      "max_lines": 50
    },
    {
      "name": "require_doc_comments",
      "enabled": false,
      "severity": "info",
      "public_only": true
    }
  ],
  "complexity": {
    "max_cyclomatic": 10,
    "max_cognitive": 15
  }
}
"#;
    
    let config_path = PathBuf::from(".ruchy-lint.json");
    
    if config_path.exists() {
        eprintln!("{} Lint configuration already exists at {}", "⚠".yellow(), config_path.display());
        eprintln!("Use --config flag to specify a different configuration file");
        return Ok(());
    }
    
    fs::write(&config_path, default_config)?;
    println!("{} Created default lint configuration at {}", "✓".bright_green(), config_path.display());
    println!("Edit this file to customize your lint rules");
    
    Ok(())
}

/// Lint Ruchy code
fn lint_ruchy_code(file: &Path, all: bool, verbose: bool, format: &str, deny_warnings: bool, max_complexity: usize, custom_rules: &CustomLintRules) -> Result<()> {
    if all {
        // Discover and lint all .ruchy files
        let ruchy_files = discover_ruchy_files(".")?;
        
        if ruchy_files.is_empty() {
            println!("{} No .ruchy files found", "⚠".yellow());
            return Ok(());
        }
        
        println!("{} Found {} .ruchy files", "→".bright_cyan(), ruchy_files.len());
        
        let mut total_violations = Vec::new();
        let mut files_with_errors = 0;
        
        for file in &ruchy_files {
            let source = fs::read_to_string(file)?;
            let mut parser = RuchyParser::new(&source);
            
            match parser.parse() {
                Ok(ast) => {
                    let violations = run_lint_checks(&ast, max_complexity, custom_rules, &source);
                    if !violations.is_empty() {
                        total_violations.extend(violations.clone());
                        files_with_errors += 1;
                        display_lint_results(&violations, file, verbose, format);
                    }
                }
                Err(e) => {
                    eprintln!("{} Parse error in {}: {e}", "✗".bright_red(), file.display());
                    files_with_errors += 1;
                }
            }
        }
        
        // Print summary
        let status = if total_violations.is_empty() {
            format!("{} PASSED", "✓".bright_green())
        } else {
            format!("{} FAILED", "✗".bright_red())
        };
        
        println!("\nlint result: {}. {} files processed; {} violations in {} files",
            status, ruchy_files.len(), total_violations.len(), files_with_errors);
        
        if !total_violations.is_empty() && deny_warnings {
            std::process::exit(1);
        }
    } else {
        // Lint single file
        let source = fs::read_to_string(file)?;
        let mut parser = RuchyParser::new(&source);
        
        match parser.parse() {
            Ok(ast) => {
                let violations = run_lint_checks(&ast, max_complexity, custom_rules, &source);
                display_lint_results(&violations, file, verbose, format);
                
                if !violations.is_empty() && deny_warnings {
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("{} Parse error in {}: {e}", "✗".bright_red(), file.display());
                std::process::exit(1);
            }
        }
    }
    
    Ok(())
}

/// Run all lint checks on an AST
fn run_lint_checks(ast: &ruchy::Expr, max_complexity: usize, custom_rules: &CustomLintRules, source: &str) -> Vec<LintViolation> {
    let mut violations = Vec::new();
    
    // Built-in checks (skip if disabled)
    if !custom_rules.disabled_rules.contains(&"complexity".to_string()) {
        check_function_complexity(ast, max_complexity, &mut violations);
    }
    
    if !custom_rules.disabled_rules.contains(&"unused_variables".to_string()) {
        check_unused_variables(ast, &mut violations);
    }
    
    if !custom_rules.disabled_rules.contains(&"missing_docs".to_string()) {
        check_missing_docs(ast, &mut violations);
    }
    
    if !custom_rules.disabled_rules.contains(&"naming_conventions".to_string()) {
        check_naming_conventions(ast, &mut violations);
    }
    
    if !custom_rules.disabled_rules.contains(&"line_length".to_string()) {
        check_line_length(ast, &mut violations);
    }
    
    // Apply custom pattern rules
    for pattern_rule in &custom_rules.custom_patterns {
        check_pattern_rule(source, pattern_rule, &mut violations);
    }
    
    // Apply custom configurable rules
    for custom_rule in &custom_rules.rules {
        if custom_rule.enabled {
            apply_custom_rule(ast, source, custom_rule, &mut violations);
        }
    }
    
    violations
}

/// Check for pattern-based violations in source
fn check_pattern_rule(source: &str, rule: &PatternRule, violations: &mut Vec<LintViolation>) {
    // Simple pattern matching - could be enhanced with regex
    for (line_num, line) in source.lines().enumerate() {
        if line.contains(&rule.pattern) {
            violations.push(LintViolation {
                severity: rule.severity,
                rule: rule.name.clone(),
                message: rule.message.clone(),
                line: line_num + 1,
                column: line.find(&rule.pattern).unwrap_or(0),
                suggestion: rule.suggestion.clone(),
            });
        }
    }
}

/// Apply a custom configurable rule
fn apply_custom_rule(ast: &ruchy::Expr, source: &str, rule: &CustomRule, violations: &mut Vec<LintViolation>) {
    match rule.name.as_str() {
        "max_line_length" => {
            if let Some(max_length) = rule.config.get("max_length").and_then(serde_json::Value::as_u64) {
                for (line_num, line) in source.lines().enumerate() {
                    if line.len() > max_length as usize {
                        violations.push(LintViolation {
                            severity: rule.severity,
                            rule: rule.name.clone(),
                            message: format!("Line exceeds maximum length of {} characters", max_length),
                            line: line_num + 1,
                            column: max_length as usize,
                            suggestion: Some("Break line into multiple lines".to_string()),
                        });
                    }
                }
            }
        }
        "max_function_length" => {
            if let Some(max_lines) = rule.config.get("max_lines").and_then(serde_json::Value::as_u64) {
                check_function_length(ast, max_lines as usize, rule, violations);
            }
        }
        "require_doc_comments" => {
            let public_only = rule.config.get("public_only")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true);
            check_doc_comments(ast, public_only, rule, violations);
        }
        _ => {
            // Unknown custom rule - skip
        }
    }
}

/// Check function length
fn check_function_length(ast: &ruchy::Expr, max_lines: usize, rule: &CustomRule, violations: &mut Vec<LintViolation>) {
    match &ast.kind {
        ExprKind::Function { name, body, .. } => {
            // Approximate function length by span
            let function_lines = (body.span.end - ast.span.start) / 80; // Rough estimate
            if function_lines > max_lines {
                violations.push(LintViolation {
                    severity: rule.severity,
                    rule: rule.name.clone(),
                    message: format!("Function '{}' exceeds maximum length of {} lines", name, max_lines),
                    line: ast.span.start,
                    column: 0,
                    suggestion: Some("Consider breaking this function into smaller functions".to_string()),
                });
            }
            check_function_length(body, max_lines, rule, violations);
        }
        ExprKind::Block(exprs) => {
            for expr in exprs {
                check_function_length(expr, max_lines, rule, violations);
            }
        }
        _ => {}
    }
}

/// Check for documentation comments
fn check_doc_comments(ast: &ruchy::Expr, public_only: bool, rule: &CustomRule, violations: &mut Vec<LintViolation>) {
    match &ast.kind {
        ExprKind::Function { name, .. } => {
            let is_public = !name.starts_with('_');
            if (!public_only || is_public) && ast.attributes.is_empty() {
                violations.push(LintViolation {
                    severity: rule.severity,
                    rule: rule.name.clone(),
                    message: format!("Function '{}' is missing documentation", name),
                    line: ast.span.start,
                    column: 0,
                    suggestion: Some(format!("Add documentation comment: /// {} does...", name)),
                });
            }
        }
        ExprKind::Block(exprs) => {
            for expr in exprs {
                check_doc_comments(expr, public_only, rule, violations);
            }
        }
        _ => {}
    }
}

/// Check function complexity
fn check_function_complexity(ast: &ruchy::Expr, max_complexity: usize, violations: &mut Vec<LintViolation>) {
    match &ast.kind {
        ExprKind::Function { name, body, .. } => {
            let complexity = calculate_complexity(body);
            if complexity > max_complexity {
                violations.push(LintViolation {
                    severity: LintSeverity::Warning,
                    rule: "complexity".to_string(),
                    message: format!("Function '{name}' has complexity {complexity}, max allowed is {max_complexity}"),
                    line: ast.span.start,
                    column: 0,
                    suggestion: Some("Consider breaking this function into smaller functions".to_string()),
                });
            }
            check_function_complexity(body, max_complexity, violations);
        }
        ExprKind::Block(exprs) => {
            for expr in exprs {
                check_function_complexity(expr, max_complexity, violations);
            }
        }
        ExprKind::If { condition, then_branch, else_branch } => {
            check_function_complexity(condition, max_complexity, violations);
            check_function_complexity(then_branch, max_complexity, violations);
            if let Some(else_expr) = else_branch {
                check_function_complexity(else_expr, max_complexity, violations);
            }
        }
        ExprKind::Match { expr, arms } => {
            check_function_complexity(expr, max_complexity, violations);
            for arm in arms {
                check_function_complexity(&arm.body, max_complexity, violations);
            }
        }
        ExprKind::For { body, .. } | ExprKind::While { body, .. } => {
            check_function_complexity(body, max_complexity, violations);
        }
        _ => {}
    }
}

/// Calculate cyclomatic complexity of an expression
fn calculate_complexity(expr: &ruchy::Expr) -> usize {
    match &expr.kind {
        ExprKind::If { condition, then_branch, else_branch } => {
            1 + calculate_complexity(condition) 
              + calculate_complexity(then_branch)
              + else_branch.as_ref().map_or(0, |e| calculate_complexity(e))
        }
        ExprKind::Match { expr, arms } => {
            arms.len() + calculate_complexity(expr) + 
            arms.iter().map(|arm| calculate_complexity(&arm.body)).sum::<usize>()
        }
        ExprKind::For { body, .. } | ExprKind::While { body, .. } => {
            1 + calculate_complexity(body)
        }
        ExprKind::Block(exprs) => {
            exprs.iter().map(calculate_complexity).sum()
        }
        ExprKind::Function { body, .. } => {
            1 + calculate_complexity(body)
        }
        _ => 0,
    }
}

/// Check for unused variables (placeholder for future implementation)
#[allow(clippy::ptr_arg)]
fn check_unused_variables(_ast: &ruchy::Expr, _violations: &mut Vec<LintViolation>) {
    // This requires building a symbol table and tracking usage
    // Implementation deferred to future iteration
}

/// Check for missing documentation
fn check_missing_docs(ast: &ruchy::Expr, violations: &mut Vec<LintViolation>) {
    match &ast.kind {
        ExprKind::Function { name, .. } => {
            // Check if function has documentation attributes
            if ast.attributes.is_empty() {
                violations.push(LintViolation {
                    severity: LintSeverity::Info,
                    rule: "missing_docs".to_string(),
                    message: format!("Function '{name}' is missing documentation"),
                    line: ast.span.start,
                    column: 0,
                    suggestion: Some(format!("Add documentation comment: /// {name} does...")),
                });
            }
        }
        ExprKind::Block(exprs) => {
            for expr in exprs {
                check_missing_docs(expr, violations);
            }
        }
        _ => {}
    }
}

/// Check naming conventions
fn check_naming_conventions(ast: &ruchy::Expr, violations: &mut Vec<LintViolation>) {
    match &ast.kind {
        ExprKind::Function { name, .. } => {
            if !is_snake_case(name) {
                violations.push(LintViolation {
                    severity: LintSeverity::Warning,
                    rule: "naming_convention".to_string(),
                    message: format!("Function name '{name}' should use snake_case"),
                    line: ast.span.start,
                    column: 0,
                    suggestion: Some(format!("Rename to '{}'", to_snake_case(name))),
                });
            }
        }
        ExprKind::Let { name, .. } => {
            if !is_snake_case(name) {
                violations.push(LintViolation {
                    severity: LintSeverity::Warning,
                    rule: "naming_convention".to_string(),
                    message: format!("Variable name '{name}' should use snake_case"),
                    line: ast.span.start,
                    column: 0,
                    suggestion: Some(format!("Rename to '{}'", to_snake_case(name))),
                });
            }
        }
        ExprKind::Block(exprs) => {
            for expr in exprs {
                check_naming_conventions(expr, violations);
            }
        }
        _ => {}
    }
}

/// Check line length (placeholder - would need source text)
#[allow(clippy::ptr_arg)]
fn check_line_length(_ast: &ruchy::Expr, _violations: &mut Vec<LintViolation>) {
    // This would require access to the original source text
    // For now, this is a placeholder
}

/// Check if a name follows `snake_case` convention
fn is_snake_case(name: &str) -> bool {
    name.chars().all(|c| c.is_lowercase() || c.is_numeric() || c == '_') 
        && !name.starts_with('_') 
        && !name.ends_with('_')
        && !name.contains("__")
}

/// Convert a name to `snake_case`
fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    let mut prev_was_lower = false;
    
    for c in name.chars() {
        if c.is_uppercase() {
            if prev_was_lower {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_was_lower = false;
        } else {
            result.push(c);
            prev_was_lower = c.is_lowercase();
        }
    }
    
    result
}

/// Display lint results
fn display_lint_results(violations: &[LintViolation], file: &Path, verbose: bool, format: &str) {
    if format == "json" {
        display_json_results(violations, file);
    } else {
        display_text_results(violations, file, verbose);
    }
}

/// Display results in text format
fn display_text_results(violations: &[LintViolation], file: &Path, verbose: bool) {
    if violations.is_empty() {
        println!("{} {} is clean", "✓".bright_green(), file.display());
        return;
    }
    
    println!("\n{} Issues found in {}:", "⚠".yellow(), file.display());
    
    for violation in violations {
        let severity_color = match violation.severity {
            LintSeverity::Error => "error".bright_red(),
            LintSeverity::Warning => "warning".yellow(),
            LintSeverity::Info => "info".bright_blue(),
        };
        
        println!("  {}: {} [{}]", severity_color, violation.message, violation.rule);
        
        if verbose {
            println!("    at line {}, column {}", violation.line, violation.column);
            if let Some(suggestion) = &violation.suggestion {
                println!("    suggestion: {}", suggestion.bright_green());
            }
        }
    }
}

/// Display results in JSON format
fn display_json_results(violations: &[LintViolation], file: &Path) {
    let json_violations: Vec<serde_json::Value> = violations.iter().map(|v| {
        serde_json::json!({
            "severity": format!("{:?}", v.severity).to_lowercase(),
            "rule": v.rule,
            "message": v.message,
            "line": v.line,
            "column": v.column,
            "suggestion": v.suggestion
        })
    }).collect();
    
    let result = serde_json::json!({
        "file": file.display().to_string(),
        "violations": json_violations
    });
    
    println!("{}", serde_json::to_string_pretty(&result).unwrap_or_else(|_| "Invalid JSON".to_string()));
}

/// Documentation item extracted from AST
#[derive(Debug, Clone)]
struct DocItem {
    name: String,
    kind: DocItemKind,
    description: Option<String>,
    signature: String,
    line: usize,
    is_public: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum DocItemKind {
    Function,
    Variable,
    Type,
    Module,
}

/// Generate documentation from Ruchy source code
fn generate_documentation(
    path: &Path,
    output_dir: &Path,
    format: &str,
    include_private: bool,
    open_browser: bool,
    all_files: bool,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("{} Generating documentation...", "→".bright_cyan());
    }
    
    // Collect all files to document
    let files_to_document = if all_files || path.is_dir() {
        let base_dir = if path.is_dir() { path.to_str().unwrap() } else { "." };
        let files = discover_ruchy_files(base_dir)?;
        if files.is_empty() {
            println!("{} No .ruchy files found", "⚠".yellow());
            return Ok(());
        }
        files
    } else {
        vec![path.to_path_buf()]
    };
    
    if verbose {
        println!("{} Processing {} files", "→".bright_cyan(), files_to_document.len());
    }
    
    // Extract documentation from all files
    let mut all_docs = Vec::new();
    for file in &files_to_document {
        if verbose {
            println!("  Processing {}", file.display());
        }
        
        let source = fs::read_to_string(file)?;
        let mut parser = RuchyParser::new(&source);
        
        match parser.parse() {
            Ok(ast) => {
                let docs = extract_documentation(&ast, include_private);
                all_docs.extend(docs);
            }
            Err(e) => {
                eprintln!("{} Parse error in {}: {e}", "✗".bright_red(), file.display());
            }
        }
    }
    
    // Create output directory
    fs::create_dir_all(output_dir)?;
    
    // Generate documentation in the requested format
    match format {
        "html" => {
            generate_html_docs(&all_docs, output_dir, verbose)?;
            let index_path = output_dir.join("index.html");
            println!("{} Generated HTML documentation in {}", "✓".bright_green(), output_dir.display());
            
            if open_browser {
                open_in_browser(&index_path)?;
            }
        }
        "markdown" | "md" => {
            generate_markdown_docs(&all_docs, output_dir, verbose)?;
            println!("{} Generated Markdown documentation in {}", "✓".bright_green(), output_dir.display());
        }
        "json" => {
            generate_json_docs(&all_docs, output_dir, verbose)?;
            println!("{} Generated JSON documentation in {}", "✓".bright_green(), output_dir.display());
        }
        _ => {
            eprintln!("Unsupported format: {}. Use 'html', 'markdown', or 'json'", format);
            std::process::exit(1);
        }
    }
    
    if verbose {
        println!("{} Documentation generated: {} items", "✓".bright_green(), all_docs.len());
    }
    
    Ok(())
}

/// Extract documentation from AST
fn extract_documentation(ast: &ruchy::Expr, include_private: bool) -> Vec<DocItem> {
    let mut docs = Vec::new();
    extract_docs_recursive(ast, &mut docs, include_private);
    docs
}

/// Recursively extract documentation items from AST
fn extract_docs_recursive(ast: &ruchy::Expr, docs: &mut Vec<DocItem>, include_private: bool) {
    match &ast.kind {
        ExprKind::Function { name, params, .. } => {
            let is_public = !name.starts_with('_');
            if is_public || include_private {
                // Format parameters for signature
                let param_list: Vec<String> = params.iter()
                    .map(|p| format!("{}: {}", p.name, format_type(&p.ty)))
                    .collect();
                let signature = format!("fn {}({})", name, param_list.join(", "));
                let description = extract_doc_comment(&ast.attributes);
                
                docs.push(DocItem {
                    name: name.clone(),
                    kind: DocItemKind::Function,
                    description,
                    signature,
                    line: ast.span.start,
                    is_public,
                });
            }
        }
        ExprKind::Let { name, .. } => {
            let is_public = !name.starts_with('_');
            if is_public || include_private {
                let signature = format!("let {}", name);
                let description = extract_doc_comment(&ast.attributes);
                
                docs.push(DocItem {
                    name: name.clone(),
                    kind: DocItemKind::Variable,
                    description,
                    signature,
                    line: ast.span.start,
                    is_public,
                });
            }
        }
        ExprKind::Block(exprs) => {
            for expr in exprs {
                extract_docs_recursive(expr, docs, include_private);
            }
        }
        _ => {}
    }
}

/// Format a type for display
fn format_type(ty: &ruchy::frontend::ast::Type) -> String {
    // Simple type formatting - can be enhanced later
    format!("{:?}", ty)
}

/// Extract documentation comment from attributes
fn extract_doc_comment(attributes: &[ruchy::frontend::ast::Attribute]) -> Option<String> {
    // Look for documentation attributes (e.g., /// comments)
    for attr in attributes {
        if attr.name == "doc" && !attr.args.is_empty() {
            return Some(attr.args.join(" "));
        }
    }
    None
}

/// Generate HTML documentation
fn generate_html_docs(docs: &[DocItem], output_dir: &Path, verbose: bool) -> Result<()> {
    let mut html = String::from(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Ruchy Documentation</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; margin: 40px; }
        h1 { color: #333; border-bottom: 2px solid #4CAF50; padding-bottom: 10px; }
        h2 { color: #555; margin-top: 30px; }
        .item { margin: 20px 0; padding: 15px; background: #f5f5f5; border-left: 3px solid #4CAF50; }
        .signature { font-family: monospace; background: #eee; padding: 5px; }
        .description { margin-top: 10px; color: #666; }
        .private { opacity: 0.7; border-left-color: #FFA500; }
        .kind { display: inline-block; padding: 2px 8px; background: #4CAF50; color: white; border-radius: 3px; font-size: 12px; }
    </style>
</head>
<body>
    <h1>Ruchy Documentation</h1>
"#);
    
    // Group items by kind
    let mut functions = Vec::new();
    let mut variables = Vec::new();
    
    for doc in docs {
        match doc.kind {
            DocItemKind::Function => functions.push(doc),
            DocItemKind::Variable => variables.push(doc),
            _ => {}
        }
    }
    
    // Generate functions section
    if !functions.is_empty() {
        html.push_str("<h2>Functions</h2>\n");
        for func in &functions {
            let class = if func.is_public { "item" } else { "item private" };
            html.push_str(&format!(r#"<div class="{}">
    <span class="kind">function</span>
    <h3>{}</h3>
    <div class="signature">{}</div>"#, class, func.name, func.signature));
            
            if let Some(desc) = &func.description {
                html.push_str(&format!(r#"
    <div class="description">{}</div>"#, desc));
            }
            
            html.push_str("\n</div>\n");
        }
    }
    
    // Generate variables section
    if !variables.is_empty() {
        html.push_str("<h2>Variables</h2>\n");
        for var in &variables {
            let class = if var.is_public { "item" } else { "item private" };
            html.push_str(&format!(r#"<div class="{}">
    <span class="kind">variable</span>
    <h3>{}</h3>
    <div class="signature">{}</div>"#, class, var.name, var.signature));
            
            if let Some(desc) = &var.description {
                html.push_str(&format!(r#"
    <div class="description">{}</div>"#, desc));
            }
            
            html.push_str("\n</div>\n");
        }
    }
    
    html.push_str("</body>\n</html>");
    
    // Write HTML file
    let index_path = output_dir.join("index.html");
    fs::write(&index_path, html)?;
    
    if verbose {
        println!("  Generated {}", index_path.display());
    }
    
    Ok(())
}

/// Generate Markdown documentation
fn generate_markdown_docs(docs: &[DocItem], output_dir: &Path, verbose: bool) -> Result<()> {
    let mut markdown = String::from("# Ruchy Documentation\n\n");
    
    // Group items by kind
    let mut functions = Vec::new();
    let mut variables = Vec::new();
    
    for doc in docs {
        match doc.kind {
            DocItemKind::Function => functions.push(doc),
            DocItemKind::Variable => variables.push(doc),
            _ => {}
        }
    }
    
    // Generate functions section
    if !functions.is_empty() {
        markdown.push_str("## Functions\n\n");
        for func in &functions {
            markdown.push_str(&format!("### {}\n\n", func.name));
            markdown.push_str(&format!("```ruchy\n{}\n```\n\n", func.signature));
            
            if let Some(desc) = &func.description {
                markdown.push_str(&format!("{}\n\n", desc));
            }
            
            if !func.is_public {
                markdown.push_str("*Private function*\n\n");
            }
        }
    }
    
    // Generate variables section
    if !variables.is_empty() {
        markdown.push_str("## Variables\n\n");
        for var in &variables {
            markdown.push_str(&format!("### {}\n\n", var.name));
            markdown.push_str(&format!("```ruchy\n{}\n```\n\n", var.signature));
            
            if let Some(desc) = &var.description {
                markdown.push_str(&format!("{}\n\n", desc));
            }
            
            if !var.is_public {
                markdown.push_str("*Private variable*\n\n");
            }
        }
    }
    
    // Write Markdown file
    let docs_path = output_dir.join("README.md");
    fs::write(&docs_path, markdown)?;
    
    if verbose {
        println!("  Generated {}", docs_path.display());
    }
    
    Ok(())
}

/// Generate JSON documentation
fn generate_json_docs(docs: &[DocItem], output_dir: &Path, verbose: bool) -> Result<()> {
    let json_docs: Vec<serde_json::Value> = docs.iter().map(|doc| {
        serde_json::json!({
            "name": doc.name,
            "kind": format!("{:?}", doc.kind).to_lowercase(),
            "signature": doc.signature,
            "description": doc.description,
            "line": doc.line,
            "is_public": doc.is_public,
        })
    }).collect();
    
    let result = serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "items": json_docs,
    });
    
    // Write JSON file
    let json_path = output_dir.join("docs.json");
    let json_str = serde_json::to_string_pretty(&result)?;
    fs::write(&json_path, json_str)?;
    
    if verbose {
        println!("  Generated {}", json_path.display());
    }
    
    Ok(())
}

/// Open HTML documentation in browser
fn open_in_browser(path: &Path) -> Result<()> {
    let url = format!("file://{}", path.canonicalize()?.display());
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(&["/C", "start", &url])
            .spawn()?;
    }
    
    println!("{} Opened documentation in browser", "✓".bright_green());
    Ok(())
}

/// Discover all .ruchy files in a directory
fn discover_ruchy_files(dir: &str) -> Result<Vec<PathBuf>> {
    let mut ruchy_files = Vec::new();
    visit_dir(Path::new(dir), &mut ruchy_files)?;
    ruchy_files.sort();
    Ok(ruchy_files)
}

/// Recursively visit directories to find .ruchy files
fn visit_dir(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                // Skip hidden directories and common build/dependency directories
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    if !name.starts_with('.') 
                        && name != "target" 
                        && name != "node_modules" 
                        && name != "build" 
                        && name != "dist" {
                        visit_dir(&path, files)?;
                    }
                }
            } else if path.extension().and_then(|s| s.to_str()) == Some("ruchy") {
                files.push(path);
            }
        }
    }
    Ok(())
}
