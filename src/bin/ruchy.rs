#![allow(clippy::print_stdout)]
#![allow(clippy::print_stderr)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use ruchy::{runtime::repl::Repl, Parser as RuchyParser, Transpiler, ExprKind};
use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::{Path, PathBuf};

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

    // Check if stdin has input (piped mode)
    if !io::stdin().is_terminal() {
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
            let source = fs::read_to_string(&file)?;
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
        Some(Commands::Check { file }) => {
            let source = fs::read_to_string(&file)?;
            let mut parser = RuchyParser::new(&source);
            match parser.parse() {
                Ok(_) => {
                    println!("{}", "✓ Syntax is valid".green());
                }
                Err(e) => {
                    eprintln!("{}", format!("✗ Syntax error: {e}").red());
                    std::process::exit(1);
                }
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
        Some(Commands::Lint { file, all, verbose, format, deny_warnings, max_complexity }) => {
            if all {
                lint_ruchy_code(&PathBuf::from("."), all, verbose, &format, deny_warnings, max_complexity)?;
            } else if let Some(file) = file {
                lint_ruchy_code(&file, false, verbose, &format, deny_warnings, max_complexity)?;
            } else {
                eprintln!("Error: Either provide a file or use --all flag");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn run_file(file: &PathBuf) -> Result<()> {
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

/// Format Ruchy code
#[allow(clippy::fn_params_excessive_bools)]
fn format_ruchy_code(file: &PathBuf, all: bool, check: bool, stdout: bool, diff: bool) -> Result<()> {
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
fn format_single_file(file: &PathBuf, check: bool, stdout: bool, diff: bool) -> Result<bool> {
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
fn print_diff(original: &str, formatted: &str, file: &PathBuf) {
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

/// Lint Ruchy code
fn lint_ruchy_code(file: &PathBuf, all: bool, verbose: bool, format: &str, deny_warnings: bool, max_complexity: usize) -> Result<()> {
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
                    let violations = run_lint_checks(&ast, max_complexity);
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
                let violations = run_lint_checks(&ast, max_complexity);
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
fn run_lint_checks(ast: &ruchy::Expr, max_complexity: usize) -> Vec<LintViolation> {
    let mut violations = Vec::new();
    
    // Check 1: Function complexity
    check_function_complexity(ast, max_complexity, &mut violations);
    
    // Check 2: Unused variables (basic check - placeholder for now)
    check_unused_variables(ast, &mut violations);
    
    // Check 3: Missing documentation
    check_missing_docs(ast, &mut violations);
    
    // Check 4: Naming conventions
    check_naming_conventions(ast, &mut violations);
    
    // Check 5: Line length
    check_line_length(ast, &mut violations);
    
    violations
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
fn check_unused_variables(_ast: &ruchy::Expr, _violations: &mut Vec<LintViolation>) {
    // TODO: Implement proper unused variable detection
    // This requires building a symbol table and tracking usage
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
fn check_line_length(_ast: &ruchy::Expr, _violations: &mut Vec<LintViolation>) {
    // This would require access to the original source text
    // For now, this is a placeholder
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

/// Display lint results
fn display_lint_results(violations: &[LintViolation], file: &PathBuf, verbose: bool, format: &str) {
    if format == "json" {
        display_json_results(violations, file);
    } else {
        display_text_results(violations, file, verbose);
    }
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
    
    println!("{}", serde_json::to_string_pretty(&result).unwrap_or_else(|_| "Invalid JSON".to_string()));
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
