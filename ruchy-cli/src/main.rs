use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use ruchy::{runtime::repl::Repl, Parser as RuchyParser, Transpiler};
use std::fs;
use std::path::PathBuf;

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
    if !atty::is(atty::Stream::Stdin) {
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
                    println!("{:#?}", ast);
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
                println!("{}", rust_code);
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
                format!("fn main() {{\n    {}\n}}", rust_code)
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
                format!("fn main() {{\n    {}\n}}", rust_code)
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
                        println!("{}", value);
                    }
                }
            }
            std::process::exit(0);
        }
        Err(e) => {
            if format == "json" {
                println!(r#"{{"error": "{}"}}"#, e);
            } else {
                eprintln!("Error: {}", e);
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
            eprintln!("Error at line: {}", line);
            eprintln!("  {}", e);
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
        Value::Char(c) => format!(r#""{}""#, c),
    }
}
