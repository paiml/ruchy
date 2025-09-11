// [RUCHY-207] CLI Module Implementation
// PMAT Complexity: <10 per function

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "ruchy")]
#[command(author = "Noah Gift")]
#[command(version = "3.0.3")]
#[command(about = "The Ruchy programming language - A modern, expressive language for data science")]
#[command(long_about = None)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress all output except errors
    #[arg(short, long, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Start the interactive REPL
    Repl,
    
    /// Run a Ruchy script
    Run {
        /// Path to the script file
        path: PathBuf,
    },
    
    /// Format Ruchy code
    #[command(visible_alias = "fmt")]
    Format {
        /// Path to format (file or directory)
        path: PathBuf,
        
        /// Check formatting without making changes
        #[arg(long)]
        check: bool,
    },
    
    /// Notebook operations
    #[command(subcommand)]
    Notebook(NotebookCommand),
    
    /// WebAssembly compilation
    #[command(subcommand)]
    Wasm(WasmCommand),
    
    /// Testing utilities
    #[command(subcommand)]
    Test(TestCommand),
}

#[derive(Subcommand, Debug)]
pub enum NotebookCommand {
    /// Start the notebook server
    Serve {
        /// Port to serve on
        #[arg(short, long, default_value = "8888")]
        port: u16,
        
        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
    
    /// Test a notebook
    Test {
        /// Path to the notebook file
        path: PathBuf,
        
        /// Generate coverage report
        #[arg(long)]
        coverage: bool,
        
        /// Output format (json, html, text)
        #[arg(long, default_value = "text")]
        format: String,
    },
    
    /// Convert notebook to different format
    Convert {
        /// Input notebook path
        input: PathBuf,
        
        /// Output path
        output: PathBuf,
        
        /// Output format (html, markdown, script)
        #[arg(long, default_value = "html")]
        format: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum WasmCommand {
    /// Compile Ruchy code to WebAssembly
    Compile {
        /// Input Ruchy file
        input: PathBuf,
        
        /// Output WASM file
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Optimize output
        #[arg(long)]
        optimize: bool,
        
        /// Validate generated WASM
        #[arg(long, default_value = "true")]
        validate: bool,
    },
    
    /// Run WASM module
    Run {
        /// WASM module to run
        module: PathBuf,
        
        /// Arguments to pass to main function
        args: Vec<String>,
    },
    
    /// Validate WASM module
    Validate {
        /// WASM module to validate
        module: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
pub enum TestCommand {
    /// Run tests
    Run {
        /// Path to test (file or directory)
        path: PathBuf,
        
        /// Generate coverage report
        #[arg(long)]
        coverage: bool,
        
        /// Run tests in parallel
        #[arg(long, default_value = "true")]
        parallel: bool,
        
        /// Filter tests by name
        #[arg(long)]
        filter: Option<String>,
    },
    
    /// Generate test report
    Report {
        /// Output format (json, html, junit)
        #[arg(long, default_value = "html")]
        format: String,
        
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

// Implementation functions with complexity <10
impl Cli {
    pub fn execute(self) -> Result<(), String> {
        match self.command {
            Command::Repl => execute_repl(self.verbose, self.quiet),
            Command::Run { path } => execute_run(path, self.verbose),
            Command::Format { path, check } => execute_format(path, check),
            Command::Notebook(cmd) => execute_notebook(cmd, self.verbose),
            Command::Wasm(cmd) => execute_wasm(cmd, self.verbose),
            Command::Test(cmd) => execute_test(cmd, self.verbose),
        }
    }
}

fn execute_repl(verbose: bool, quiet: bool) -> Result<(), String> {
    if !quiet {
        println!("Starting Ruchy REPL v3.0.3...");
    }
    
    // Use existing REPL implementation
    crate::run_repl()
        .map_err(|e| format!("REPL error: {}", e))
}

fn execute_run(path: PathBuf, verbose: bool) -> Result<(), String> {
    if verbose {
        println!("Running script: {:?}", path);
    }
    
    let source = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let mut parser = crate::frontend::parser::Parser::new(&source);
    let ast = parser.parse()
        .map_err(|e| format!("Parse error: {:?}", e))?;
    
    let mut interpreter = crate::runtime::interpreter::Interpreter::new();
    interpreter.eval_expr(&ast)
        .map_err(|e| format!("Runtime error: {:?}", e))?;
    
    Ok(())
}

fn execute_format(path: PathBuf, check: bool) -> Result<(), String> {
    if check {
        println!("Checking formatting for: {:?}", path);
        // TODO: Implement format checking
        Ok(())
    } else {
        println!("Formatting: {:?}", path);
        // TODO: Implement formatting
        Ok(())
    }
}

fn execute_notebook(cmd: NotebookCommand, verbose: bool) -> Result<(), String> {
    match cmd {
        NotebookCommand::Serve { port, host } => {
            if verbose {
                println!("Starting notebook server on {}:{}", host, port);
            }
            
            // Use existing notebook server
            #[cfg(feature = "notebook")]
            {
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| format!("Failed to create runtime: {}", e))?;
                
                rt.block_on(async {
                    crate::notebook::server::start_server(port).await
                        .map_err(|e| format!("Server error: {}", e))
                })?;
            }
            
            #[cfg(not(feature = "notebook"))]
            {
                return Err("Notebook feature not enabled".to_string());
            }
            
            Ok(())
        }
        NotebookCommand::Test { path, coverage, format } => {
            if verbose {
                println!("Testing notebook: {:?}", path);
            }
            
            #[cfg(feature = "notebook")]
            {
                let config = crate::notebook::testing::types::TestConfig::default();
                let report = run_test_command(&path, config)?;
                
                match format.as_str() {
                    "json" => println!("{}", serde_json::to_string_pretty(&report).unwrap()),
                    "html" => println!("HTML report generation not yet implemented"),
                    _ => println!("{:#?}", report),
                }
            }
            
            #[cfg(not(feature = "notebook"))]
            {
                let _ = (coverage, format);
                return Err("Notebook feature not enabled".to_string());
            }
            
            Ok(())
        }
        NotebookCommand::Convert { input, output, format } => {
            if verbose {
                println!("Converting {:?} to {} format", input, format);
            }
            // TODO: Implement notebook conversion
            Ok(())
        }
    }
}

fn execute_wasm(cmd: WasmCommand, verbose: bool) -> Result<(), String> {
    match cmd {
        WasmCommand::Compile { input, output, optimize, validate } => {
            if verbose {
                println!("Compiling {:?} to WASM", input);
            }
            
            let source = std::fs::read_to_string(&input)
                .map_err(|e| format!("Failed to read file: {}", e))?;
            
            let output_path = output.unwrap_or_else(|| {
                let mut path = input.clone();
                path.set_extension("wasm");
                path
            });
            
            // Use existing WASM compiler
            #[cfg(feature = "wasm-compile")]
            {
                let mut parser = crate::frontend::parser::Parser::new(&source);
                let ast = parser.parse()
                    .map_err(|e| format!("Parse error: {:?}", e))?;
                
                let emitter = crate::backend::wasm::WasmEmitter::new();
                let wasm_bytes = emitter.emit(&ast)
                    .map_err(|e| format!("WASM compilation error: {}", e))?;
                
                if validate {
                    wasmparser::validate(&wasm_bytes)
                        .map_err(|e| format!("WASM validation error: {}", e))?;
                }
                
                std::fs::write(&output_path, wasm_bytes)
                    .map_err(|e| format!("Failed to write WASM file: {}", e))?;
                
                if verbose {
                    println!("Successfully compiled to {:?}", output_path);
                }
            }
            
            #[cfg(not(feature = "wasm-compile"))]
            {
                return Err("WASM compilation feature not enabled".to_string());
            }
            
            Ok(())
        }
        WasmCommand::Run { module, args } => {
            if verbose {
                println!("Running WASM module: {:?}", module);
            }
            // TODO: Implement WASM execution
            Ok(())
        }
        WasmCommand::Validate { module } => {
            if verbose {
                println!("Validating WASM module: {:?}", module);
            }
            
            let bytes = std::fs::read(&module)
                .map_err(|e| format!("Failed to read WASM file: {}", e))?;
            
            wasmparser::validate(&bytes)
                .map_err(|e| format!("WASM validation error: {}", e))?;
            
            println!("✓ WASM module is valid");
            Ok(())
        }
    }
}

fn execute_test(cmd: TestCommand, verbose: bool) -> Result<(), String> {
    match cmd {
        TestCommand::Run { path, coverage, parallel, filter } => {
            if verbose {
                println!("Running tests in {:?}", path);
            }
            
            // TODO: Implement test runner
            println!("Test runner not yet implemented");
            Ok(())
        }
        TestCommand::Report { format, output } => {
            if verbose {
                println!("Generating test report in {} format", format);
            }
            // TODO: Implement test reporting
            Ok(())
        }
    }
}

// Keep the existing run_test_command function
#[cfg(feature = "notebook")]
pub fn run_test_command(_notebook_path: &std::path::Path, _config: crate::notebook::testing::types::TestConfig) -> Result<crate::notebook::testing::types::TestReport, String> {
    // Stub implementation for Sprint 0
    Ok(crate::notebook::testing::types::TestReport {
        total_tests: 1,
        passed_tests: 1,
        failed_tests: 0,
        skipped_tests: 0,
        execution_time: std::time::Duration::from_millis(100),
        coverage: None,
        failures: Vec::new(),
        results: vec![crate::notebook::testing::types::TestResult::Pass],
    })
}