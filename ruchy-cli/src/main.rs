#![allow(clippy::print_stdout, clippy::print_stderr)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use ruchy::{runtime::repl::Repl, Parser as RuchyParser, Transpiler};
use std::fs;
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
        Value::Function { name, params, .. } => {
            format!(r#""fn {}({})""#, name, params.join(", "))
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
