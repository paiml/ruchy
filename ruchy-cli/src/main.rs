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
                lint_all_files(verbose, &format, deny_warnings, max_complexity, min_severity.as_ref())?;
            } else {
                lint_file(&file, verbose, &format, deny_warnings, max_complexity, min_severity.as_ref())?;
            }
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
        Value::Range { start, end, inclusive } => {
            if *inclusive {
                format!(r#""{}..={}""#, start, end)
            } else {
                format!(r#""{}..{}""#, start, end)
            }
        }
        Value::EnumVariant { enum_name, variant_name, data } => {
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
            eprintln!("{} Failed to read {}: {}", "✗".bright_red(), file.display(), e);
            std::process::exit(1);
        }
    };
    
    // Parse the source
    let mut parser = RuchyParser::new(&source);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("{} Parse error in {}: {}", "✗".bright_red(), file.display(), e);
            std::process::exit(1);
        }
    };
    
    // Format the AST
    let config = FormatConfig::new(line_width, indent, use_tabs);
    let formatted = format_ast(&ast, &config);
    
    if check {
        // Check mode: verify if file is already formatted
        if source.trim() == formatted.trim() {
            println!("{} {} is already formatted", "✓".bright_green(), file.display());
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
            println!("{} {} is already formatted", "→".bright_cyan(), file.display());
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
    
    println!("{} Found {} .ruchy files", "→".bright_cyan(), ruchy_files.len());
    
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
    
    println!("\nformat result: {}. {} files processed; {} errors",
        status, formatted + already_formatted, errors);
    
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
fn format_expr(expr: &ruchy::Expr, output: &mut String, _indent_level: usize, _config: &FormatConfig) {
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
#[allow(dead_code, clippy::format_push_string, clippy::trivially_copy_pass_by_ref)]
fn format_binary_op(op: &ruchy::BinaryOp, output: &mut String) {
    output.push_str(&format!("{op:?}"));
}

/// Format a unary operator (simplified)
#[allow(dead_code, clippy::format_push_string, clippy::trivially_copy_pass_by_ref)]
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
            eprintln!("{} Failed to read {}: {e}", "✗".bright_red(), file.display());
            std::process::exit(1);
        }
    };
    
    // Parse the source
    let mut parser = RuchyParser::new(&source);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("{} Parse error in {}: {e}", "✗".bright_red(), file.display());
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
    
    println!("{} Found {} .ruchy files", "→".bright_cyan(), ruchy_files.len());
    
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
    
    println!("\nlint result: {}. {} files processed; {} violations in {} files",
        status, ruchy_files.len(), total_violations.len(), files_with_errors);
    
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
fn check_function_complexity(ast: &ruchy::Expr, max_complexity: usize, violations: &mut Vec<LintViolation>) {
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
        _ => {
            // For other expression types, continue traversing if needed
        }
    }
}

/// Calculate cyclomatic complexity of an expression
fn calculate_complexity(expr: &ruchy::Expr) -> usize {
    use ruchy::ExprKind;
    
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
    name.chars().all(|c| c.is_lowercase() || c.is_numeric() || c == '_') 
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
fn filter_violations(violations: Vec<LintViolation>, min_severity: Option<&LintSeverity>) -> Vec<LintViolation> {
    if let Some(min_sev) = min_severity {
        violations.into_iter().filter(|v| {
            match (&v.severity, min_sev) {
                (LintSeverity::Error, _) => true,
                (LintSeverity::Warning, LintSeverity::Error) => false,
                (LintSeverity::Warning, _) => true,
                (LintSeverity::Info, LintSeverity::Error | LintSeverity::Warning) => false,
                (LintSeverity::Info, LintSeverity::Info) => true,
            }
        }).collect()
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
fn display_json_results(violations: &[LintViolation], file: &PathBuf) {
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
    
    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
