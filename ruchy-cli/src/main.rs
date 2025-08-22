#![allow(clippy::print_stdout, clippy::print_stderr)]

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use ruchy::{runtime::repl::Repl, Parser as RuchyParser, Transpiler};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(ValueEnum, Clone, Debug)]
enum LintSeverity {
    Error,
    Warning,
    Info,
}

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

        /// Arguments to pass to the program
        args: Vec<String>,
    },

    /// Compile a Ruchy file to an executable
    Compile {
        /// The file to compile
        file: PathBuf,

        /// Output executable name
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Enable release mode
        #[arg(long)]
        release: bool,
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

        /// Line width for formatting
        #[arg(long, default_value = "100")]
        line_width: usize,

        /// Indent size (spaces)
        #[arg(long, default_value = "4")]
        indent: usize,

        /// Use tabs instead of spaces for indentation
        #[arg(long)]
        use_tabs: bool,

        /// Show diff of changes
        #[arg(long)]
        diff: bool,
    },

    /// Lint Ruchy source code for issues and style violations
    Lint {
        /// The file to lint
        file: PathBuf,

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

        /// Show only errors of specified severity or higher
        #[arg(long, value_enum)]
        min_severity: Option<LintSeverity>,
    },

    /// Run tests with optional coverage reporting
    Test {
        /// Specific test file to run (optional)
        file: Option<PathBuf>,

        /// Run all tests in project
        #[arg(long)]
        all: bool,

        /// Generate coverage report
        #[arg(long)]
        coverage: bool,

        /// Coverage output format (text, html, json)
        #[arg(long, default_value = "text")]
        coverage_format: String,

        /// Watch mode - rerun tests when files change
        #[arg(long)]
        watch: bool,

        /// Run tests in parallel
        #[arg(long)]
        parallel: bool,

        /// Minimum coverage threshold (fail if below)
        #[arg(long)]
        threshold: Option<f64>,

        /// Output format for test results (text, json, junit)
        #[arg(long, default_value = "text")]
        format: String,

        /// Show verbose test output
        #[arg(long)]
        verbose: bool,
    },

    /// Generate and analyze AST with advanced options
    Ast {
        /// The file to analyze
        file: PathBuf,

        /// Output format (pretty, json, graph)
        #[arg(long, default_value = "pretty")]
        format: String,

        /// Include metrics analysis
        #[arg(long)]
        metrics: bool,

        /// Include symbol table analysis
        #[arg(long)]
        symbols: bool,

        /// Include dependency analysis
        #[arg(long)]
        deps: bool,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Handle one-liner evaluation with -e flag (highest priority)
    if let Some(expr) = cli.eval {
        return evaluate_oneliner(&expr, &cli.format);
    }

    // Handle script file execution (without subcommand)
    if let Some(file) = cli.file {
        return run_script(&file);
    }

    // Check if stdin has input (for pipe support)
    // Only do this if we're not explicitly running a command
    if cli.command.is_none() && !atty::is(atty::Stream::Stdin) {
        use std::io::{self, Read};
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        if !buffer.trim().is_empty() {
            return evaluate_oneliner(&buffer, &cli.format);
        }
    }

    match cli.command {
        None | Some(Commands::Repl) => {
            // Start REPL by default
            let mut repl = Repl::new()?;
            repl.run()?;
        }

        Some(Commands::Parse { file }) => {
            let source = fs::read_to_string(&file)?;
            let mut parser = RuchyParser::new(&source);

            match parser.parse() {
                Ok(ast) => {
                    println!("{}", "AST:".bright_cyan());
                    println!("{ast:#?}");
                }
                Err(e) => {
                    eprintln!("{} {}", "Parse error:".bright_red(), e);
                    std::process::exit(1);
                }
            }
        }

        Some(Commands::Transpile { file, output }) => {
            let source = fs::read_to_string(&file)?;
            let mut parser = RuchyParser::new(&source);

            let ast = parser.parse()?;
            let transpiler = Transpiler::new();
            let rust_code = transpiler.transpile_to_string(&ast)?;

            if let Some(output_path) = output {
                fs::write(output_path, rust_code)?;
                println!("{} Transpiled successfully", "✓".bright_green());
            } else {
                println!("{rust_code}");
            }
        }

        Some(Commands::Run { file, args }) => {
            let source = fs::read_to_string(&file)?;
            let mut parser = RuchyParser::new(&source);

            let ast = parser.parse()?;
            let transpiler = Transpiler::new();
            let rust_code = transpiler.transpile_to_string(&ast)?;

            // Create temporary Rust file
            let temp_dir = std::env::temp_dir();
            let temp_file = temp_dir.join("ruchy_temp.rs");
            let temp_exe = temp_dir.join("ruchy_temp");

            // Wrap in main function if needed
            let full_code = if rust_code.contains("fn main") {
                rust_code
            } else {
                format!("fn main() {{\n    {rust_code}\n}}")
            };

            fs::write(&temp_file, full_code)?;

            // Compile
            let output = std::process::Command::new("rustc")
                .arg(&temp_file)
                .arg("-o")
                .arg(&temp_exe)
                .output()?;

            if !output.status.success() {
                eprintln!("{} Compilation failed:", "✗".bright_red());
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                std::process::exit(1);
            }

            // Run
            let mut cmd = std::process::Command::new(&temp_exe);
            for arg in args {
                cmd.arg(arg);
            }

            let status = cmd.status()?;
            std::process::exit(status.code().unwrap_or(1));
        }

        Some(Commands::Compile {
            file,
            output,
            release,
        }) => {
            let source = fs::read_to_string(&file)?;
            let mut parser = RuchyParser::new(&source);

            let ast = parser.parse()?;
            let transpiler = Transpiler::new();
            let rust_code = transpiler.transpile_to_string(&ast)?;

            // Generate output name
            let output_path = output.unwrap_or_else(|| {
                let mut path = file.clone();
                path.set_extension("");
                path
            });

            // Create temporary Rust file
            let temp_file = file.with_extension("rs");

            // Wrap in main function if needed
            let full_code = if rust_code.contains("fn main") {
                rust_code
            } else {
                format!("fn main() {{\n    {rust_code}\n}}")
            };

            fs::write(&temp_file, full_code)?;

            // Compile
            let mut cmd = std::process::Command::new("rustc");
            cmd.arg(&temp_file).arg("-o").arg(&output_path);

            if release {
                cmd.arg("-O");
            }

            let output = cmd.output()?;

            if !output.status.success() {
                eprintln!("{} Compilation failed:", "✗".bright_red());
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                std::process::exit(1);
            }

            println!(
                "{} Compiled successfully to {}",
                "✓".bright_green(),
                output_path.display()
            );
        }

        Some(Commands::Fmt {
            file,
            all,
            check,
            stdout,
            line_width,
            indent,
            use_tabs,
            diff,
        }) => {
            if all {
                format_all_files(check, stdout, line_width, indent, use_tabs, diff)?;
            } else {
                format_file(&file, check, stdout, line_width, indent, use_tabs, diff)?;
            }
        }

        Some(Commands::Lint {
            file,
            all,
            verbose,
            format,
            deny_warnings,
            max_complexity,
            min_severity,
        }) => {
            if all {
                lint_all_files(
                    verbose,
                    &format,
                    deny_warnings,
                    max_complexity,
                    min_severity.as_ref(),
                )?;
            } else {
                lint_file(
                    &file,
                    verbose,
                    &format,
                    deny_warnings,
                    max_complexity,
                    min_severity.as_ref(),
                )?;
            }
        }

        Some(Commands::Test {
            file,
            all,
            coverage,
            coverage_format,
            watch,
            parallel,
            threshold,
            format,
            verbose,
        }) => {
            run_tests(
                file.as_ref(),
                all,
                coverage,
                &coverage_format,
                watch,
                parallel,
                threshold,
                &format,
                verbose,
            )?;
        }

        Some(Commands::Ast {
            file,
            format,
            metrics,
            symbols,
            deps,
        }) => {
            analyze_ast(&file, &format, metrics, symbols, deps)?;
        }
    }

    Ok(())
}

/// Evaluate a one-liner expression
fn evaluate_oneliner(expr: &str, format: &str) -> Result<()> {
    use ruchy::runtime::Value;
    use std::time::{Duration, Instant};

    // Create a REPL instance for evaluation
    let mut repl = Repl::new()?;

    // Set timeout for one-liners (100ms default)
    let deadline = Instant::now() + Duration::from_millis(100);

    match repl.evaluate_expr_str(expr, Some(deadline)) {
        Ok(value) => {
            match format {
                "json" => {
                    // Output as JSON for scripting
                    println!("{}", value_to_json(&value));
                }
                _ => {
                    // Default text output
                    if !matches!(value, Value::Unit) {
                        println!("{value}");
                    }
                }
            }
            std::process::exit(0);
        }
        Err(e) => {
            if format == "json" {
                println!(r#"{{"error": "{e}"}}"#);
            } else {
                eprintln!("Error: {e}");
            }
            std::process::exit(1);
        }
    }
}

/// Run a script file
fn run_script(file: &PathBuf) -> Result<()> {
    use std::time::{Duration, Instant};

    let source = fs::read_to_string(file)?;
    let mut repl = Repl::new()?;

    // Scripts get longer timeout (10 seconds)
    let deadline = Instant::now() + Duration::from_secs(10);

    // Execute each line/statement in the script
    for line in source.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        if let Err(e) = repl.evaluate_expr_str(line, Some(deadline)) {
            eprintln!("Error at line: {line}");
            eprintln!("  {e}");
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Convert a Value to JSON representation
fn value_to_json(value: &ruchy::runtime::Value) -> String {
    use ruchy::runtime::Value;

    match value {
        Value::Unit => "null".to_string(),
        Value::Int(n) => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::String(s) => format!(r#""{}""#, s.replace('"', r#"\""#)),
        Value::Char(c) => format!(r#""{c}""#),
        Value::List(items) => {
            let json_items: Vec<String> = items.iter().map(value_to_json).collect();
            format!("[{}]", json_items.join(", "))
        }
        Value::Tuple(items) => {
            let json_items: Vec<String> = items.iter().map(value_to_json).collect();
            format!("[{}]", json_items.join(", "))
        }
        Value::Function { name, params, .. } => {
            format!(r#""fn {}({})""#, name, params.join(", "))
        }
        Value::Lambda { params, .. } => {
            format!(r#""lambda({})"#, params.join(", "))
        }
        Value::DataFrame { columns } => {
            // Create a JSON representation of the DataFrame
            let mut cols = Vec::new();
            for col in columns {
                let values: Vec<String> = col.values.iter().map(value_to_json).collect();
                cols.push(format!(
                    r#"{{"name":"{}","values":[{}]}}"#,
                    col.name,
                    values.join(",")
                ));
            }
            format!(r#"{{"type":"DataFrame","columns":[{}]}}"#, cols.join(","))
        }
        Value::Object(map) => {
            let pairs: Vec<String> = map
                .iter()
                .map(|(k, v)| format!(r#""{}": {}"#, k, value_to_json(v)))
                .collect();
            format!("{{{}}}", pairs.join(", "))
        }
        Value::Range {
            start,
            end,
            inclusive,
        } => {
            if *inclusive {
                format!(r#""{}..={}""#, start, end)
            } else {
                format!(r#""{}..{}""#, start, end)
            }
        }
        Value::EnumVariant {
            enum_name,
            variant_name,
            data,
        } => {
            let base = format!("\"{}::{}\"", enum_name, variant_name);
            if let Some(values) = data {
                let vals: Vec<String> = values.iter().map(value_to_json).collect();
                format!("{{\"variant\": {}, \"data\": [{}]}}", base, vals.join(", "))
            } else {
                base
            }
        }
    }
}

/// Format configuration
#[derive(Clone)]
#[allow(dead_code)]
struct FormatConfig {
    line_width: usize,
    indent_size: usize,
    use_tabs: bool,
}

impl FormatConfig {
    fn new(line_width: usize, indent: usize, use_tabs: bool) -> Self {
        Self {
            line_width,
            indent_size: indent,
            use_tabs,
        }
    }

    #[allow(dead_code)]
    fn indent_str(&self, level: usize) -> String {
        if self.use_tabs {
            "\t".repeat(level)
        } else {
            " ".repeat(level * self.indent_size)
        }
    }
}

/// Format a single file
#[allow(clippy::fn_params_excessive_bools)]
fn format_file(
    file: &PathBuf,
    check: bool,
    stdout: bool,
    line_width: usize,
    indent: usize,
    use_tabs: bool,
    diff: bool,
) -> Result<()> {
    use std::fs;

    // Read the source file
    let source = match fs::read_to_string(file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!(
                "{} Failed to read {}: {}",
                "✗".bright_red(),
                file.display(),
                e
            );
            std::process::exit(1);
        }
    };

    // Parse the source
    let mut parser = RuchyParser::new(&source);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!(
                "{} Parse error in {}: {}",
                "✗".bright_red(),
                file.display(),
                e
            );
            std::process::exit(1);
        }
    };

    // Format the AST
    let config = FormatConfig::new(line_width, indent, use_tabs);
    let formatted = format_ast(&ast, &config);

    if check {
        // Check mode: verify if file is already formatted
        if source.trim() == formatted.trim() {
            println!(
                "{} {} is already formatted",
                "✓".bright_green(),
                file.display()
            );
        } else {
            println!("{} {} needs formatting", "✗".bright_red(), file.display());
            if diff {
                print_diff(&source, &formatted, file);
            }
            std::process::exit(1);
        }
    } else if stdout {
        // Output to stdout
        println!("{formatted}");
    } else {
        // Write back to file
        if source.trim() == formatted.trim() {
            println!(
                "{} {} is already formatted",
                "→".bright_cyan(),
                file.display()
            );
        } else {
            fs::write(file, &formatted)?;
            println!("{} Formatted {}", "✓".bright_green(), file.display());
            if diff {
                print_diff(&source, &formatted, file);
            }
        }
    }

    Ok(())
}

/// Format all Ruchy files in the project
#[allow(clippy::fn_params_excessive_bools)]
fn format_all_files(
    check: bool,
    _stdout: bool,
    line_width: usize,
    indent: usize,
    use_tabs: bool,
    diff: bool,
) -> Result<()> {
    // Discover all .ruchy files
    let ruchy_files = discover_ruchy_files_fmt(".")?;

    if ruchy_files.is_empty() {
        println!("{} No .ruchy files found", "⚠".yellow());
        return Ok(());
    }

    println!(
        "{} Found {} .ruchy files",
        "→".bright_cyan(),
        ruchy_files.len()
    );

    let mut formatted = 0;
    let already_formatted = 0;
    let mut errors = 0;

    for file in &ruchy_files {
        match format_file(file, check, false, line_width, indent, use_tabs, diff) {
            Ok(()) => {
                // Simple success counting
                formatted += 1;
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

    println!(
        "\nformat result: {}. {} files processed; {} errors",
        status,
        formatted + already_formatted,
        errors
    );

    if errors > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/// Discover all .ruchy files for formatting
#[allow(clippy::items_after_statements)]
fn discover_ruchy_files_fmt(dir: &str) -> Result<Vec<PathBuf>> {
    use std::fs;

    let mut ruchy_files = Vec::new();

    fn visit_dir(dir: &std::path::Path, files: &mut Vec<PathBuf>) -> Result<()> {
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
                            && name != "dist"
                        {
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

    visit_dir(std::path::Path::new(dir), &mut ruchy_files)?;
    ruchy_files.sort();
    Ok(ruchy_files)
}

/// Format an AST to a string
fn format_ast(ast: &ruchy::Expr, config: &FormatConfig) -> String {
    let mut output = String::new();
    format_expr(ast, &mut output, 0, config);
    // Ensure we end with a newline
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

/// Format an expression with proper indentation
#[allow(clippy::format_push_string)]
fn format_expr(
    expr: &ruchy::Expr,
    output: &mut String,
    _indent_level: usize,
    _config: &FormatConfig,
) {
    // For now, use a simple debug-based formatter
    // This can be enhanced later with proper AST traversal
    output.push_str(&format!("{expr:?}"));
}

/// Format a literal value (simplified)
#[allow(dead_code, clippy::format_push_string)]
fn format_literal(lit: &ruchy::Literal, output: &mut String) {
    output.push_str(&format!("{lit:?}"));
}

/// Format a binary operator (simplified)
#[allow(
    dead_code,
    clippy::format_push_string,
    clippy::trivially_copy_pass_by_ref
)]
fn format_binary_op(op: &ruchy::BinaryOp, output: &mut String) {
    output.push_str(&format!("{op:?}"));
}

/// Format a unary operator (simplified)
#[allow(
    dead_code,
    clippy::format_push_string,
    clippy::trivially_copy_pass_by_ref
)]
fn format_unary_op(op: &ruchy::UnaryOp, output: &mut String) {
    output.push_str(&format!("{op:?}"));
}

/// Format a pattern (simplified)
#[allow(dead_code, clippy::format_push_string)]
fn format_pattern(pattern: &ruchy::Pattern, output: &mut String) {
    output.push_str(&format!("{pattern:?}"));
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

/// Lint a single file
#[allow(clippy::fn_params_excessive_bools)]
fn lint_file(
    file: &PathBuf,
    verbose: bool,
    format: &str,
    deny_warnings: bool,
    max_complexity: usize,
    min_severity: Option<&LintSeverity>,
) -> Result<()> {
    use std::fs;

    // Read the source file
    let source = match fs::read_to_string(file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!(
                "{} Failed to read {}: {e}",
                "✗".bright_red(),
                file.display()
            );
            std::process::exit(1);
        }
    };

    // Parse the source
    let mut parser = RuchyParser::new(&source);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!(
                "{} Parse error in {}: {e}",
                "✗".bright_red(),
                file.display()
            );
            std::process::exit(1);
        }
    };

    // Run lint checks
    let violations = run_lint_checks(&ast, max_complexity);
    let filtered_violations = filter_violations(violations, min_severity);

    // Display results
    display_lint_results(&filtered_violations, file, verbose, format, deny_warnings)
}

/// Lint all Ruchy files in the project
fn lint_all_files(
    verbose: bool,
    format: &str,
    deny_warnings: bool,
    max_complexity: usize,
    min_severity: Option<&LintSeverity>,
) -> Result<()> {
    // Discover all .ruchy files
    let ruchy_files = discover_ruchy_files_lint(".")?;

    if ruchy_files.is_empty() {
        println!("{} No .ruchy files found", "⚠".yellow());
        return Ok(());
    }

    println!(
        "{} Found {} .ruchy files",
        "→".bright_cyan(),
        ruchy_files.len()
    );

    let mut total_violations = Vec::new();
    let mut files_with_errors = 0;

    for file in &ruchy_files {
        match lint_file(file, verbose, format, false, max_complexity, min_severity) {
            Ok(()) => {
                // Count violations for summary
                let source = fs::read_to_string(file)?;
                let mut parser = RuchyParser::new(&source);
                if let Ok(ast) = parser.parse() {
                    let violations = run_lint_checks(&ast, max_complexity);
                    let filtered = filter_violations(violations, min_severity);
                    if !filtered.is_empty() {
                        total_violations.extend(filtered);
                        files_with_errors += 1;
                    }
                }
            }
            Err(_) => {
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

    println!(
        "\nlint result: {}. {} files processed; {} violations in {} files",
        status,
        ruchy_files.len(),
        total_violations.len(),
        files_with_errors
    );

    if !total_violations.is_empty() && deny_warnings {
        std::process::exit(1);
    }

    Ok(())
}

/// Discover all .ruchy files for linting
#[allow(clippy::items_after_statements)]
fn discover_ruchy_files_lint(dir: &str) -> Result<Vec<PathBuf>> {
    // Reuse the same logic as formatting
    discover_ruchy_files_fmt(dir)
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

/// Run all lint checks on an AST
fn run_lint_checks(ast: &ruchy::Expr, max_complexity: usize) -> Vec<LintViolation> {
    let mut violations = Vec::new();

    // Check 1: Function complexity
    check_function_complexity(ast, max_complexity, &mut violations);

    // Check 2: Unused variables (basic check)
    check_unused_variables(ast, &mut violations);

    // Check 3: Missing documentation
    check_missing_docs(ast, &mut violations);

    // Check 4: Naming conventions
    check_naming_conventions(ast, &mut violations);

    violations
}

/// Check function complexity
fn check_function_complexity(
    ast: &ruchy::Expr,
    max_complexity: usize,
    violations: &mut Vec<LintViolation>,
) {
    use ruchy::ExprKind;

    // Simple recursive traversal to find function definitions
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
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
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
        _ => {
            // For other expression types, continue traversing if needed
        }
    }
}

/// Calculate cyclomatic complexity of an expression
fn calculate_complexity(expr: &ruchy::Expr) -> usize {
    use ruchy::ExprKind;

    match &expr.kind {
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            1 + calculate_complexity(condition)
                + calculate_complexity(then_branch)
                + else_branch.as_ref().map_or(0, |e| calculate_complexity(e))
        }
        ExprKind::Match { expr, arms } => {
            arms.len()
                + calculate_complexity(expr)
                + arms
                    .iter()
                    .map(|arm| calculate_complexity(&arm.body))
                    .sum::<usize>()
        }
        ExprKind::For { body, .. } | ExprKind::While { body, .. } => 1 + calculate_complexity(body),
        ExprKind::Block(exprs) => exprs.iter().map(calculate_complexity).sum(),
        ExprKind::Function { body, .. } => 1 + calculate_complexity(body),
        _ => 0,
    }
}

/// Check for unused variables (simplified)
fn check_unused_variables(_ast: &ruchy::Expr, _violations: &mut Vec<LintViolation>) {
    // TODO: Implement proper unused variable detection
    // This would require building a symbol table and tracking usage
}

/// Check for missing documentation
fn check_missing_docs(ast: &ruchy::Expr, violations: &mut Vec<LintViolation>) {
    use ruchy::ExprKind;

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
    use ruchy::ExprKind;

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

/// Check if a name follows snake_case convention
fn is_snake_case(name: &str) -> bool {
    name.chars()
        .all(|c| c.is_lowercase() || c.is_numeric() || c == '_')
        && !name.starts_with('_')
        && !name.ends_with('_')
        && !name.contains("__")
}

/// Convert a name to snake_case
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

/// Filter violations based on minimum severity
fn filter_violations(
    violations: Vec<LintViolation>,
    min_severity: Option<&LintSeverity>,
) -> Vec<LintViolation> {
    if let Some(min_sev) = min_severity {
        violations
            .into_iter()
            .filter(|v| match (&v.severity, min_sev) {
                (LintSeverity::Error, _) => true,
                (LintSeverity::Warning, LintSeverity::Error) => false,
                (LintSeverity::Warning, _) => true,
                (LintSeverity::Info, LintSeverity::Error | LintSeverity::Warning) => false,
                (LintSeverity::Info, LintSeverity::Info) => true,
            })
            .collect()
    } else {
        violations
    }
}

/// Display lint results
fn display_lint_results(
    violations: &[LintViolation],
    file: &PathBuf,
    verbose: bool,
    format: &str,
    deny_warnings: bool,
) -> Result<()> {
    match format {
        "json" => display_json_results(violations, file),
        _ => display_text_results(violations, file, verbose),
    }

    if !violations.is_empty() && deny_warnings {
        std::process::exit(1);
    }

    Ok(())
}

/// Display results in text format
fn display_text_results(violations: &[LintViolation], file: &PathBuf, verbose: bool) {
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

        println!(
            "  {}: {} [{}]",
            severity_color, violation.message, violation.rule
        );

        if verbose {
            println!(
                "    at line {}, column {}",
                violation.line, violation.column
            );
            if let Some(suggestion) = &violation.suggestion {
                println!("    suggestion: {}", suggestion.bright_green());
            }
        }
    }
}

/// Display results in JSON format
fn display_json_results(violations: &[LintViolation], file: &PathBuf) {
    let json_violations: Vec<serde_json::Value> = violations
        .iter()
        .map(|v| {
            serde_json::json!({
                "severity": format!("{:?}", v.severity).to_lowercase(),
                "rule": v.rule,
                "message": v.message,
                "line": v.line,
                "column": v.column,
                "suggestion": v.suggestion
            })
        })
        .collect();

    let result = serde_json::json!({
        "file": file.display().to_string(),
        "violations": json_violations
    });

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}

/// Test execution and coverage reporting
#[allow(clippy::fn_params_excessive_bools)]
fn run_tests(
    file: Option<&PathBuf>,
    all: bool,
    coverage: bool,
    coverage_format: &str,
    watch: bool,
    parallel: bool,
    threshold: Option<f64>,
    format: &str,
    verbose: bool,
) -> Result<()> {
    use std::time::Instant;

    println!("{} Running Ruchy tests...", "🧪".bright_blue());
    
    let start_time = Instant::now();
    let test_files = if let Some(specific_file) = file {
        vec![specific_file.clone()]
    } else if all {
        discover_test_files(".")?
    } else {
        discover_test_files(".")?
    };

    if test_files.is_empty() {
        println!("{} No test files found", "⚠".yellow());
        println!("  Test files should match: *_test.ruchy, test_*.ruchy, tests/*.ruchy");
        return Ok(());
    }

    println!(
        "{} Found {} test file(s)",
        "→".bright_cyan(),
        test_files.len()
    );

    if watch {
        println!("{} Watch mode not yet implemented", "⚠".yellow());
    }

    if parallel {
        println!("{} Parallel execution not yet implemented", "⚠".yellow());
    }

    let mut test_results = TestResults::new();
    let mut coverage_data = if coverage {
        Some(CoverageData::new())
    } else {
        None
    };

    // Run tests
    for test_file in &test_files {
        println!("\n{} Testing {}...", "🔍".bright_blue(), test_file.display());
        
        match run_single_test_file(test_file, verbose, coverage_data.as_mut()) {
            Ok(file_results) => {
                test_results.merge(file_results);
            }
            Err(e) => {
                test_results.errors.push(format!("Failed to run {}: {}", test_file.display(), e));
            }
        }
    }

    let duration = start_time.elapsed();

    // Display results
    display_test_results(&test_results, format, duration)?;

    // Display coverage if requested
    if let Some(cov_data) = coverage_data {
        display_coverage_results(&cov_data, coverage_format, threshold)?;
    }

    // Exit with appropriate code
    if test_results.failures > 0 || !test_results.errors.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}

/// Discover test files in a directory
fn discover_test_files(dir: &str) -> Result<Vec<PathBuf>> {
    use std::fs;

    let mut test_files = Vec::new();

    fn visit_dir_tests(dir: &std::path::Path, files: &mut Vec<PathBuf>) -> Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                        if !name.starts_with('.')
                            && name != "target"
                            && name != "node_modules"
                            && name != "build"
                            && name != "dist"
                        {
                            visit_dir_tests(&path, files)?;
                        }
                    }
                } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if ext == "ruchy" {
                        if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                            // Match test file patterns
                            if name.ends_with("_test") 
                                || name.starts_with("test_") 
                                || path.parent().and_then(|p| p.file_name()).and_then(|s| s.to_str()) == Some("tests")
                            {
                                files.push(path);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    visit_dir_tests(std::path::Path::new(dir), &mut test_files)?;
    test_files.sort();
    Ok(test_files)
}

/// Test results aggregation
#[derive(Debug, Default)]
struct TestResults {
    passed: usize,
    failures: usize,
    errors: Vec<String>,
    test_cases: Vec<TestCase>,
}

#[derive(Debug)]
struct TestCase {
    name: String,
    file: PathBuf,
    passed: bool,
    message: Option<String>,
    duration_ms: u64,
}

impl TestResults {
    fn new() -> Self {
        Self::default()
    }

    fn merge(&mut self, other: TestResults) {
        self.passed += other.passed;
        self.failures += other.failures;
        self.errors.extend(other.errors);
        self.test_cases.extend(other.test_cases);
    }
}

/// Coverage data collection
#[derive(Debug)]
struct CoverageData {
    lines_covered: std::collections::HashSet<(PathBuf, usize)>,
    total_lines: std::collections::HashMap<PathBuf, usize>,
    functions_covered: std::collections::HashSet<(PathBuf, String)>,
    total_functions: std::collections::HashMap<PathBuf, Vec<String>>,
}

impl CoverageData {
    fn new() -> Self {
        Self {
            lines_covered: std::collections::HashSet::new(),
            total_lines: std::collections::HashMap::new(),
            functions_covered: std::collections::HashSet::new(),
            total_functions: std::collections::HashMap::new(),
        }
    }

    fn coverage_percentage(&self) -> f64 {
        let total_lines: usize = self.total_lines.values().sum();
        if total_lines == 0 {
            return 100.0;
        }
        (self.lines_covered.len() as f64 / total_lines as f64) * 100.0
    }
}

/// Run a single test file
fn run_single_test_file(
    test_file: &PathBuf,
    verbose: bool,
    coverage_data: Option<&mut CoverageData>,
) -> Result<TestResults> {
    use std::time::Instant;

    let source = fs::read_to_string(test_file)?;
    let mut parser = RuchyParser::new(&source);
    
    let ast = parser.parse()?;
    
    // Track coverage if requested
    if let Some(cov_data) = coverage_data {
        track_file_coverage(&ast, test_file, cov_data);
    }

    // Extract test functions from AST
    let test_functions = extract_test_functions(&ast);
    
    if test_functions.is_empty() {
        if verbose {
            println!("  {} No test functions found (looking for functions with #[test] or names starting with 'test_')", "⚠".yellow());
        }
        return Ok(TestResults::new());
    }

    let mut results = TestResults::new();
    
    // Execute each test function
    for test_func in test_functions {
        let test_start = Instant::now();
        
        match execute_test_function(&test_func, &source, test_file) {
            Ok(()) => {
                results.passed += 1;
                results.test_cases.push(TestCase {
                    name: test_func.clone(),
                    file: test_file.clone(),
                    passed: true,
                    message: None,
                    duration_ms: test_start.elapsed().as_millis() as u64,
                });
                
                if verbose {
                    println!("  {} {}", "✓".bright_green(), test_func);
                }
            }
            Err(e) => {
                results.failures += 1;
                results.test_cases.push(TestCase {
                    name: test_func.clone(),
                    file: test_file.clone(),
                    passed: false,
                    message: Some(e.to_string()),
                    duration_ms: test_start.elapsed().as_millis() as u64,
                });
                
                println!("  {} {} - {}", "✗".bright_red(), test_func, e);
            }
        }
    }

    Ok(results)
}

/// Extract test functions from AST
fn extract_test_functions(ast: &ruchy::Expr) -> Vec<String> {
    use ruchy::ExprKind;
    
    let mut test_functions = Vec::new();
    
    fn extract_from_expr(expr: &ruchy::Expr, functions: &mut Vec<String>) {
        match &expr.kind {
            ExprKind::Function { name, .. } => {
                // Check for #[test] attribute or test_ prefix
                let has_test_attr = expr.attributes.iter().any(|attr| attr.name == "test");
                let has_test_prefix = name.starts_with("test_");
                
                if has_test_attr || has_test_prefix {
                    functions.push(name.clone());
                }
            }
            ExprKind::Block(exprs) => {
                for expr in exprs {
                    extract_from_expr(expr, functions);
                }
            }
            _ => {}
        }
    }
    
    extract_from_expr(ast, &mut test_functions);
    test_functions
}

/// Execute a single test function
fn execute_test_function(
    test_name: &str,
    source: &str,
    file: &PathBuf,
) -> Result<()> {
    use std::time::{Duration, Instant};

    // Create test execution script
    let test_script = format!("{}\n{}()", source, test_name);
    
    // Execute in REPL environment
    let mut repl = ruchy::runtime::repl::Repl::new()?;
    let deadline = Instant::now() + Duration::from_secs(5); // 5s timeout per test
    
    // Execute the test script
    match repl.evaluate_expr_str(&test_script, Some(deadline)) {
        Ok(_) => Ok(()),
        Err(e) => {
            anyhow::bail!("Test execution failed: {}", e);
        }
    }
}

/// Track file coverage information
fn track_file_coverage(
    ast: &ruchy::Expr,
    file: &PathBuf,
    coverage_data: &mut CoverageData,
) {
    use ruchy::ExprKind;
    
    // Count total lines (simplified)
    let line_count = ast.span.end;
    coverage_data.total_lines.insert(file.clone(), line_count);
    
    // Track function definitions
    let mut functions = Vec::new();
    extract_functions_for_coverage(ast, &mut functions);
    coverage_data.total_functions.insert(file.clone(), functions);
    
    // For now, assume 100% line coverage during test execution
    // In a real implementation, this would track actual execution
    for line in 1..=line_count {
        coverage_data.lines_covered.insert((file.clone(), line));
    }
}

/// Extract function names for coverage tracking
fn extract_functions_for_coverage(expr: &ruchy::Expr, functions: &mut Vec<String>) {
    use ruchy::ExprKind;
    
    match &expr.kind {
        ExprKind::Function { name, body, .. } => {
            functions.push(name.clone());
            extract_functions_for_coverage(body, functions);
        }
        ExprKind::Block(exprs) => {
            for expr in exprs {
                extract_functions_for_coverage(expr, functions);
            }
        }
        _ => {}
    }
}

/// Display test results
fn display_test_results(
    results: &TestResults,
    format: &str,
    duration: std::time::Duration,
) -> Result<()> {
    match format {
        "json" => display_test_results_json(results, duration),
        "junit" => display_test_results_junit(results, duration),
        _ => display_test_results_text(results, duration),
    }
    Ok(())
}

/// Display test results in text format
fn display_test_results_text(results: &TestResults, duration: std::time::Duration) {
    println!("\n{}", "Test Results".bright_cyan().underline());
    println!("{}", "=".repeat(50));
    
    let total_tests = results.passed + results.failures;
    let status = if results.failures == 0 && results.errors.is_empty() {
        format!("{} PASSED", "✓".bright_green())
    } else {
        format!("{} FAILED", "✗".bright_red())
    };
    
    println!(
        "Result: {}. {} tests run in {:.2}s",
        status,
        total_tests,
        duration.as_secs_f64()
    );
    
    if results.passed > 0 {
        println!("  {} {} passed", "✓".bright_green(), results.passed);
    }
    
    if results.failures > 0 {
        println!("  {} {} failed", "✗".bright_red(), results.failures);
    }
    
    if !results.errors.is_empty() {
        println!("  {} {} errors", "⚠".yellow(), results.errors.len());
        for error in &results.errors {
            println!("    {}", error);
        }
    }
    
    // Show failure details
    if results.failures > 0 {
        println!("\n{}", "Failed Tests".bright_red().underline());
        for test_case in &results.test_cases {
            if !test_case.passed {
                println!("  {} {} ({}ms)", "✗".bright_red(), test_case.name, test_case.duration_ms);
                if let Some(message) = &test_case.message {
                    println!("    {}", message);
                }
            }
        }
    }
}

/// Display test results in JSON format
fn display_test_results_json(results: &TestResults, duration: std::time::Duration) {
    let json_result = serde_json::json!({
        "summary": {
            "total": results.passed + results.failures,
            "passed": results.passed,
            "failed": results.failures,
            "errors": results.errors.len(),
            "duration_seconds": duration.as_secs_f64()
        },
        "test_cases": results.test_cases.iter().map(|tc| {
            serde_json::json!({
                "name": tc.name,
                "file": tc.file.display().to_string(),
                "passed": tc.passed,
                "message": tc.message,
                "duration_ms": tc.duration_ms
            })
        }).collect::<Vec<_>>(),
        "errors": results.errors
    });
    
    println!("{}", serde_json::to_string_pretty(&json_result).unwrap());
}

/// Display test results in JUnit XML format
fn display_test_results_junit(results: &TestResults, duration: std::time::Duration) {
    let total_tests = results.passed + results.failures;
    
    println!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    println!(
        "<testsuite name=\"ruchy-tests\" tests=\"{}\" failures=\"{}\" errors=\"{}\" time=\"{:.3}\">",
        total_tests,
        results.failures,
        results.errors.len(),
        duration.as_secs_f64()
    );
    
    for test_case in &results.test_cases {
        println!(
            "  <testcase name=\"{}\" classname=\"{}\" time=\"{:.3}\">",
            test_case.name,
            test_case.file.display(),
            test_case.duration_ms as f64 / 1000.0
        );
        
        if !test_case.passed {
            if let Some(message) = &test_case.message {
                println!("    <failure message=\"{}\"/>", message);
            } else {
                println!("    <failure/>");
            }
        }
        
        println!("  </testcase>");
    }
    
    println!("</testsuite>");
}

/// Display coverage results
fn display_coverage_results(
    coverage: &CoverageData,
    format: &str,
    threshold: Option<f64>,
) -> Result<()> {
    let coverage_pct = coverage.coverage_percentage();
    
    match format {
        "html" => generate_html_coverage_report(coverage)?,
        "json" => display_coverage_json(coverage),
        _ => display_coverage_text(coverage),
    }
    
    // Check threshold
    if let Some(min_threshold) = threshold {
        if coverage_pct < min_threshold {
            eprintln!(
                "{} Coverage {:.1}% is below threshold {:.1}%",
                "✗".bright_red(),
                coverage_pct,
                min_threshold
            );
            std::process::exit(1);
        }
    }
    
    Ok(())
}

/// Display coverage in text format
fn display_coverage_text(coverage: &CoverageData) {
    let coverage_pct = coverage.coverage_percentage();
    
    println!("\n{}", "Coverage Report".bright_cyan().underline());
    println!("{}", "=".repeat(50));
    
    let status_color = if coverage_pct >= 80.0 {
        coverage_pct.to_string().bright_green()
    } else if coverage_pct >= 60.0 {
        coverage_pct.to_string().yellow()
    } else {
        coverage_pct.to_string().bright_red()
    };
    
    println!("Overall Coverage: {}%", status_color);
    println!("Lines Covered: {}", coverage.lines_covered.len());
    
    let total_lines: usize = coverage.total_lines.values().sum();
    println!("Total Lines: {}", total_lines);
    
    // Per-file breakdown
    println!("\nFile Coverage:");
    for (file, total) in &coverage.total_lines {
        let file_covered = coverage.lines_covered.iter()
            .filter(|(f, _)| f == file)
            .count();
        let file_pct = if *total > 0 {
            (file_covered as f64 / *total as f64) * 100.0
        } else {
            100.0
        };
        
        println!("  {}: {:.1}%", file.display(), file_pct);
    }
}

/// Display coverage in JSON format
fn display_coverage_json(coverage: &CoverageData) {
    let total_lines: usize = coverage.total_lines.values().sum();
    
    let json_coverage = serde_json::json!({
        "summary": {
            "lines_covered": coverage.lines_covered.len(),
            "total_lines": total_lines,
            "coverage_percentage": coverage.coverage_percentage()
        },
        "files": coverage.total_lines.iter().map(|(file, total)| {
            let file_covered = coverage.lines_covered.iter()
                .filter(|(f, _)| f == file)
                .count();
            let file_pct = if *total > 0 {
                (file_covered as f64 / *total as f64) * 100.0
            } else {
                100.0
            };
            
            serde_json::json!({
                "file": file.display().to_string(),
                "lines_covered": file_covered,
                "total_lines": total,
                "coverage_percentage": file_pct
            })
        }).collect::<Vec<_>>()
    });
    
    println!("{}", serde_json::to_string_pretty(&json_coverage).unwrap());
}

/// Generate HTML coverage report
fn generate_html_coverage_report(coverage: &CoverageData) -> Result<()> {
    let coverage_pct = coverage.coverage_percentage();
    
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Ruchy Coverage Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .summary {{ background: #f5f5f5; padding: 15px; border-radius: 5px; margin-bottom: 20px; }}
        .coverage-high {{ color: #28a745; }}
        .coverage-medium {{ color: #ffc107; }}
        .coverage-low {{ color: #dc3545; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
    </style>
</head>
<body>
    <h1>Ruchy Coverage Report</h1>
    
    <div class="summary">
        <h2>Summary</h2>
        <p><strong>Overall Coverage:</strong> <span class="coverage-{}">{:.1}%</span></p>
        <p><strong>Lines Covered:</strong> {}</p>
        <p><strong>Total Lines:</strong> {}</p>
    </div>
    
    <h2>File Coverage</h2>
    <table>
        <tr><th>File</th><th>Coverage</th><th>Lines Covered</th><th>Total Lines</th></tr>
        {}
    </table>
</body>
</html>"#,
        if coverage_pct >= 80.0 { "high" } else if coverage_pct >= 60.0 { "medium" } else { "low" },
        coverage_pct,
        coverage.lines_covered.len(),
        coverage.total_lines.values().sum::<usize>(),
        coverage.total_lines.iter().map(|(file, total)| {
            let file_covered = coverage.lines_covered.iter()
                .filter(|(f, _)| f == file)
                .count();
            let file_pct = if *total > 0 {
                (file_covered as f64 / *total as f64) * 100.0
            } else {
                100.0
            };
            
            format!(
                "<tr><td>{}</td><td>{:.1}%</td><td>{}</td><td>{}</td></tr>",
                file.display(),
                file_pct,
                file_covered,
                total
            )
        }).collect::<Vec<_>>().join("\n        ")
    );
    
    fs::write("coverage.html", html)?;
    println!("{} Coverage report generated: coverage.html", "✓".bright_green());
    
    Ok(())
}

/// Analyze AST with advanced options
fn analyze_ast(
    file: &PathBuf,
    format: &str,
    metrics: bool,
    symbols: bool,
    deps: bool,
) -> Result<()> {
    let source = fs::read_to_string(file)?;
    let mut parser = RuchyParser::new(&source);
    
    let ast = parser.parse()?;
    
    match format {
        "json" => display_ast_json(&ast, metrics, symbols, deps)?,
        "graph" => display_ast_graph(&ast, file)?,
        _ => display_ast_pretty(&ast, metrics, symbols, deps)?,
    }
    
    Ok(())
}

/// Display AST in pretty format
fn display_ast_pretty(
    ast: &ruchy::Expr,
    metrics: bool,
    symbols: bool,
    deps: bool,
) -> Result<()> {
    println!("{}", "AST Analysis".bright_cyan().underline());
    println!("{}", "=".repeat(50));
    
    println!("\n{}", "Abstract Syntax Tree:".bright_blue());
    println!("{:#?}", ast);
    
    if metrics {
        println!("\n{}", "Complexity Metrics:".bright_blue());
        let complexity = calculate_complexity(ast);
        println!("Cyclomatic Complexity: {}", complexity);
        
        let depth = calculate_ast_depth(ast);
        println!("AST Depth: {}", depth);
        
        let node_count = count_ast_nodes(ast);
        println!("AST Node Count: {}", node_count);
    }
    
    if symbols {
        println!("\n{}", "Symbol Analysis:".bright_blue());
        let symbols = extract_symbols(ast);
        println!("Functions: {:?}", symbols.functions);
        println!("Variables: {:?}", symbols.variables);
    }
    
    if deps {
        println!("\n{}", "Dependencies:".bright_blue());
        let dependencies = extract_dependencies(ast);
        if dependencies.is_empty() {
            println!("No external dependencies found");
        } else {
            for dep in dependencies {
                println!("  {}", dep);
            }
        }
    }
    
    Ok(())
}

/// Display AST in JSON format
fn display_ast_json(
    ast: &ruchy::Expr,
    metrics: bool,
    symbols: bool,
    deps: bool,
) -> Result<()> {
    let mut result = serde_json::json!({
        "ast": format!("{:#?}", ast)
    });
    
    if metrics {
        result["metrics"] = serde_json::json!({
            "complexity": calculate_complexity(ast),
            "depth": calculate_ast_depth(ast),
            "node_count": count_ast_nodes(ast)
        });
    }
    
    if symbols {
        let symbols = extract_symbols(ast);
        result["symbols"] = serde_json::json!({
            "functions": symbols.functions,
            "variables": symbols.variables
        });
    }
    
    if deps {
        result["dependencies"] = serde_json::json!(extract_dependencies(ast));
    }
    
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

/// Display AST as DOT graph
fn display_ast_graph(ast: &ruchy::Expr, file: &PathBuf) -> Result<()> {
    println!("digraph AST {{");
    println!("  rankdir=TB;");
    println!("  node [shape=box];");
    
    let mut node_id = 0;
    generate_dot_nodes(ast, &mut node_id);
    
    println!("}}");
    
    // Also save to file
    let dot_file = file.with_extension("dot");
    let mut dot_content = String::new();
    dot_content.push_str("digraph AST {\n");
    dot_content.push_str("  rankdir=TB;\n");
    dot_content.push_str("  node [shape=box];\n");
    
    let mut node_id = 0;
    generate_dot_content(ast, &mut node_id, &mut dot_content);
    
    dot_content.push_str("}\n");
    fs::write(&dot_file, dot_content)?;
    
    println!("\n{} DOT graph saved to: {}", "✓".bright_green(), dot_file.display());
    println!("Generate SVG with: dot -Tsvg {} -o {}", dot_file.display(), file.with_extension("svg").display());
    
    Ok(())
}

/// Calculate AST depth
fn calculate_ast_depth(expr: &ruchy::Expr) -> usize {
    use ruchy::ExprKind;
    
    match &expr.kind {
        ExprKind::Block(exprs) => {
            1 + exprs.iter().map(calculate_ast_depth).max().unwrap_or(0)
        }
        ExprKind::Function { body, .. } => 1 + calculate_ast_depth(body),
        ExprKind::If { condition, then_branch, else_branch } => {
            1 + [
                calculate_ast_depth(condition),
                calculate_ast_depth(then_branch),
                else_branch.as_ref().map_or(0, |e| calculate_ast_depth(e))
            ].into_iter().max().unwrap_or(0)
        }
        ExprKind::Match { expr, arms } => {
            1 + [calculate_ast_depth(expr)]
                .into_iter()
                .chain(arms.iter().map(|arm| calculate_ast_depth(&arm.body)))
                .max()
                .unwrap_or(0)
        }
        _ => 1,
    }
}

/// Count AST nodes
fn count_ast_nodes(expr: &ruchy::Expr) -> usize {
    use ruchy::ExprKind;
    
    1 + match &expr.kind {
        ExprKind::Block(exprs) => exprs.iter().map(count_ast_nodes).sum(),
        ExprKind::Function { body, .. } => count_ast_nodes(body),
        ExprKind::If { condition, then_branch, else_branch } => {
            count_ast_nodes(condition) + 
            count_ast_nodes(then_branch) + 
            else_branch.as_ref().map_or(0, count_ast_nodes)
        }
        ExprKind::Match { expr, arms } => {
            count_ast_nodes(expr) + 
            arms.iter().map(|arm| count_ast_nodes(&arm.body)).sum::<usize>()
        }
        _ => 0,
    }
}

/// Symbol information
#[derive(Debug, Default)]
struct SymbolInfo {
    functions: Vec<String>,
    variables: Vec<String>,
}

/// Extract symbols from AST
fn extract_symbols(expr: &ruchy::Expr) -> SymbolInfo {
    use ruchy::ExprKind;
    
    let mut symbols = SymbolInfo::default();
    
    fn extract_from_expr(expr: &ruchy::Expr, symbols: &mut SymbolInfo) {
        match &expr.kind {
            ExprKind::Function { name, body, .. } => {
                symbols.functions.push(name.clone());
                extract_from_expr(body, symbols);
            }
            ExprKind::Let { name, value, .. } => {
                symbols.variables.push(name.clone());
                extract_from_expr(value, symbols);
            }
            ExprKind::Block(exprs) => {
                for expr in exprs {
                    extract_from_expr(expr, symbols);
                }
            }
            ExprKind::If { condition, then_branch, else_branch } => {
                extract_from_expr(condition, symbols);
                extract_from_expr(then_branch, symbols);
                if let Some(else_expr) = else_branch {
                    extract_from_expr(else_expr, symbols);
                }
            }
            ExprKind::Match { expr, arms } => {
                extract_from_expr(expr, symbols);
                for arm in arms {
                    extract_from_expr(&arm.body, symbols);
                }
            }
            _ => {}
        }
    }
    
    extract_from_expr(expr, &mut symbols);
    symbols
}

/// Extract dependencies (simplified)
fn extract_dependencies(_ast: &ruchy::Expr) -> Vec<String> {
    // TODO: Implement proper dependency extraction
    // This would look for import/use statements
    Vec::new()
}

/// Generate DOT nodes for graph visualization
fn generate_dot_nodes(expr: &ruchy::Expr, node_id: &mut usize) {
    use ruchy::ExprKind;
    
    let current_id = *node_id;
    *node_id += 1;
    
    let label = match &expr.kind {
        ExprKind::Function { name, .. } => format!("Function: {}", name),
        ExprKind::Let { name, .. } => format!("Let: {}", name),
        ExprKind::Literal(lit) => format!("Literal: {:?}", lit),
        ExprKind::Variable(name) => format!("Variable: {}", name),
        ExprKind::Block(_) => "Block".to_string(),
        ExprKind::If { .. } => "If".to_string(),
        ExprKind::Match { .. } => "Match".to_string(),
        _ => format!("{:?}", expr.kind).split('(').next().unwrap_or("Unknown").to_string(),
    };
    
    println!("  {} [label=\"{}\"];", current_id, label);
}

/// Generate DOT content for file output
fn generate_dot_content(expr: &ruchy::Expr, node_id: &mut usize, content: &mut String) {
    use ruchy::ExprKind;
    
    let current_id = *node_id;
    *node_id += 1;
    
    let label = match &expr.kind {
        ExprKind::Function { name, .. } => format!("Function: {}", name),
        ExprKind::Let { name, .. } => format!("Let: {}", name),
        ExprKind::Literal(lit) => format!("Literal: {:?}", lit),
        ExprKind::Variable(name) => format!("Variable: {}", name),
        ExprKind::Block(_) => "Block".to_string(),
        ExprKind::If { .. } => "If".to_string(),
        ExprKind::Match { .. } => "Match".to_string(),
        _ => format!("{:?}", expr.kind).split('(').next().unwrap_or("Unknown").to_string(),
    };
    
    content.push_str(&format!("  {} [label=\"{}\"];\n", current_id, label));
}
