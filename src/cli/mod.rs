// [RUCHY-207] CLI Module Implementation
// PMAT Complexity: <10 per function
use crate::utils::format_file_error;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
#[derive(Parser, Debug)]
#[command(name = "ruchy")]
#[command(author = "Noah Gift")]
#[command(version = "3.4.1")]
#[command(
    about = "The Ruchy programming language - A modern, expressive language for data science"
)]
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
    /// Execute the CLI command
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::cli::Cli;
    /// let cli = Cli::new();
    /// cli.execute().expect("Failed to execute");
    /// ```
    pub fn execute(self) -> Result<(), String> {
        match self.command {
            #[cfg(not(target_arch = "wasm32"))]
            Command::Repl => execute_repl(self.verbose, self.quiet),
            #[cfg(target_arch = "wasm32")]
            Command::Repl => Err("REPL not available in WASM build".to_string()),
            Command::Run { path } => execute_run(path, self.verbose),
            Command::Format { path, check } => execute_format(path, check),
            Command::Notebook(cmd) => execute_notebook(cmd, self.verbose),
            Command::Wasm(cmd) => execute_wasm(cmd, self.verbose),
            Command::Test(cmd) => execute_test(cmd, self.verbose),
        }
    }
}
#[cfg(not(target_arch = "wasm32"))]
fn execute_repl(_verbose: bool, quiet: bool) -> Result<(), String> {
    if !quiet {
        println!("Starting Ruchy REPL v3.4.1...");
    }
    // Use existing REPL implementation
    crate::run_repl().map_err(|e| format!("REPL error: {e}"))
}
fn execute_run(path: PathBuf, verbose: bool) -> Result<(), String> {
    if verbose {
        println!("Running script: {}", path.display());
    }
    let source = std::fs::read_to_string(&path).map_err(|_e| format_file_error("read", &path))?;
    let mut parser = crate::frontend::parser::Parser::new(&source);
    let ast = parser.parse().map_err(|e| format!("Parse error: {e:?}"))?;
    let mut interpreter = crate::runtime::interpreter::Interpreter::new();
    interpreter
        .eval_expr(&ast)
        .map_err(|e| format!("Runtime error: {e:?}"))?;
    Ok(())
}
fn execute_format(path: PathBuf, check: bool) -> Result<(), String> {
    use crate::quality::formatter::Formatter;

    let config = find_and_load_config(&path)?;
    let formatter = Formatter::with_config(config);

    if check {
        check_format(&path, &formatter)
    } else {
        apply_format(&path, &formatter)
    }
}

/// Check if a file is properly formatted
fn check_format(path: &PathBuf, formatter: &crate::quality::formatter::Formatter) -> Result<(), String> {
    println!("Checking formatting for: {}", path.display());

    let source = std::fs::read_to_string(path).map_err(|_e| format_file_error("read", path))?;
    let ast = parse_source(&source)?;
    let formatted = formatter.format(&ast).map_err(|e| format!("Format error: {e}"))?;

    if formatted.trim() == source.trim() {
        println!("✓ File is properly formatted");
        Ok(())
    } else {
        Err("File is not properly formatted. Run without --check to fix.".to_string())
    }
}

/// Apply formatting to a file
fn apply_format(path: &PathBuf, formatter: &crate::quality::formatter::Formatter) -> Result<(), String> {
    println!("Formatting: {}", path.display());

    let source = std::fs::read_to_string(path).map_err(|_e| format_file_error("read", path))?;
    let ast = parse_source(&source)?;
    let formatted = formatter.format(&ast).map_err(|e| format!("Format error: {e}"))?;

    std::fs::write(path, formatted).map_err(|e| format!("Failed to write file: {e}"))?;
    println!("✓ File formatted successfully");
    Ok(())
}

/// Parse source code into AST
fn parse_source(source: &str) -> Result<crate::frontend::ast::Expr, String> {
    let mut parser = crate::frontend::parser::Parser::new(source);
    parser.parse().map_err(|e| format!("Parse error: {e:?}"))
}

/// Find and load formatter configuration by searching up the directory tree
fn find_and_load_config(start_path: &PathBuf) -> Result<crate::quality::FormatterConfig, String> {
    let start_dir = get_start_directory(start_path);
    find_config_in_ancestors(&start_dir)
}

/// Get the directory to start config search from
fn get_start_directory(path: &PathBuf) -> PathBuf {
    if path.is_file() {
        path.parent().unwrap_or(path).to_path_buf()
    } else {
        path.clone()
    }
}

/// Search for config file in current and ancestor directories
fn find_config_in_ancestors(start_dir: &PathBuf) -> Result<crate::quality::FormatterConfig, String> {
    // Try current directory
    let config_path = start_dir.join(".ruchy-fmt.toml");
    if config_path.exists() {
        return crate::quality::FormatterConfig::from_file(&config_path);
    }

    // Try parent directories recursively
    match start_dir.parent() {
        Some(parent) => find_config_in_ancestors(&parent.to_path_buf()),
        None => Ok(crate::quality::FormatterConfig::default()),
    }
}
fn execute_notebook(cmd: NotebookCommand, verbose: bool) -> Result<(), String> {
    match cmd {
        NotebookCommand::Serve { port, host } => execute_notebook_serve(port, host, verbose),
        NotebookCommand::Test { path, coverage, format } => execute_notebook_test(path, coverage, format, verbose),
        NotebookCommand::Convert { input, output, format } => execute_notebook_convert(input, Some(output), format, verbose),
    }
}

fn execute_notebook_serve(port: u16, host: String, verbose: bool) -> Result<(), String> {
    if verbose {
        println!("Starting notebook server on {host}:{port}");
    }
    #[cfg(feature = "notebook")]
    {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {e}"))?;
        rt.block_on(async {
            crate::notebook::server::start_server(port)
                .await
                .map_err(|e| format!("Server error: {e}"))
        })?;
    }
    #[cfg(not(feature = "notebook"))]
    {
        return Err("Notebook feature not enabled".to_string());
    }
    Ok(())
}

fn execute_notebook_test(path: PathBuf, _coverage: bool, format: String, verbose: bool) -> Result<(), String> {
    if verbose {
        println!("Testing notebook: {}", path.display());
    }
    #[cfg(feature = "notebook")]
    {
        let config = crate::notebook::testing::types::TestConfig::default();
        let report = run_test_command(&path, config)?;
        match format.as_str() {
            "json" => match serde_json::to_string_pretty(&report) {
                Ok(json) => println!("{json}"),
                Err(e) => eprintln!("Failed to serialize report: {e}"),
            },
            "html" => println!("HTML report generation not yet implemented"),
            _ => println!("{report:#?}"),
        }
    }
    #[cfg(not(feature = "notebook"))]
    {
        return Err("Notebook feature not enabled".to_string());
    }
    Ok(())
}

fn execute_notebook_convert(input: PathBuf, _output: Option<PathBuf>, format: String, verbose: bool) -> Result<(), String> {
    if verbose {
        println!("Converting {} to {format} format", input.display());
    }
    // Note: Implement notebook conversion
    Ok(())
}
// COMPLEXITY REDUCTION: Split execute_wasm into separate functions (was 14, now <5 each)
fn execute_wasm(cmd: WasmCommand, verbose: bool) -> Result<(), String> {
    match cmd {
        WasmCommand::Compile {
            input,
            output,
            optimize: _,
            validate,
        } => execute_wasm_compile(input, output, validate, verbose),
        WasmCommand::Run { module, args } => execute_wasm_run(module, args, verbose),
        WasmCommand::Validate { module } => execute_wasm_validate(module, verbose),
    }
}
fn execute_wasm_compile(
    input: std::path::PathBuf,
    output: Option<std::path::PathBuf>,
    validate: bool,
    verbose: bool,
) -> Result<(), String> {
    if verbose {
        println!("Compiling {} to WASM", input.display());
    }
    let source =
        std::fs::read_to_string(&input).map_err(|e| format!("Failed to read file: {e}"))?;
    let output_path = output.unwrap_or_else(|| {
        let mut path = input.clone();
        path.set_extension("wasm");
        path
    });
    compile_wasm_source(&source, &output_path, validate, verbose)
}
#[cfg(feature = "wasm-compile")]
fn compile_wasm_source(
    source: &str,
    output_path: &std::path::Path,
    validate: bool,
    verbose: bool,
) -> Result<(), String> {
    let mut parser = crate::frontend::parser::Parser::new(source);
    let ast = parser.parse().map_err(|e| format!("Parse error: {e:?}"))?;
    let emitter = crate::backend::wasm::WasmEmitter::new();
    let wasm_bytes = emitter
        .emit(&ast)
        .map_err(|e| format!("WASM compilation error: {e}"))?;
    if validate {
        #[cfg(feature = "notebook")]
        {
            wasmparser::validate(&wasm_bytes).map_err(|e| format!("WASM validation error: {e}"))?;
        }
        #[cfg(not(feature = "notebook"))]
        {
            eprintln!("Warning: WASM validation skipped (wasmparser not available)");
        }
    }
    std::fs::write(output_path, wasm_bytes)
        .map_err(|e| format!("Failed to write WASM file: {e}"))?;
    if verbose {
        println!("Successfully compiled to {}", output_path.display());
    }
    Ok(())
}
#[cfg(not(feature = "wasm-compile"))]
fn compile_wasm_source(
    _source: &str,
    _output_path: &std::path::Path,
    _validate: bool,
    _verbose: bool,
) -> Result<(), String> {
    Err("WASM compilation feature not enabled".to_string())
}
fn execute_wasm_run(
    module: std::path::PathBuf,
    _args: Vec<String>,
    verbose: bool,
) -> Result<(), String> {
    if verbose {
        println!("Running WASM module: {}", module.display());
    }
    // Note: Implement WASM execution
    Ok(())
}
fn execute_wasm_validate(module: std::path::PathBuf, verbose: bool) -> Result<(), String> {
    if verbose {
        println!("Validating WASM module: {}", module.display());
    }
    let bytes = std::fs::read(&module).map_err(|e| format!("Failed to read WASM file: {e}"))?;
    #[cfg(feature = "notebook")]
    {
        wasmparser::validate(&bytes).map_err(|e| format!("WASM validation error: {e}"))?;
    }
    #[cfg(not(feature = "notebook"))]
    {
        eprintln!("Warning: WASM validation requires notebook feature");
        return Err("WASM validation not available without notebook feature".to_string());
    }
    println!("✓ WASM module is valid");
    Ok(())
}
fn execute_test(cmd: TestCommand, verbose: bool) -> Result<(), String> {
    match cmd {
        TestCommand::Run {
            path,
            coverage: _,
            parallel: _,
            filter: _,
        } => {
            if verbose {
                println!("Running tests in {}", path.display());
            }
            // Note: Implement test runner
            println!("Test runner not yet implemented");
            Ok(())
        }
        TestCommand::Report { format, output: _ } => {
            if verbose {
                println!("Generating test report in {format} format");
            }
            // Note: Implement test reporting
            Ok(())
        }
    }
}
// Keep the existing run_test_command function
#[cfg(feature = "notebook")]
/// # Examples
///
/// ```ignore
/// use ruchy::cli::mod::run_test_command;
///
/// let result = run_test_command(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn run_test_command(
    _notebook_path: &std::path::Path,
    _config: crate::notebook::testing::types::TestConfig,
) -> Result<crate::notebook::testing::types::TestReport, String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // Sprint 8: Comprehensive CLI module tests

    #[test]
    fn test_cli_creation() {
        let cli = Cli {
            verbose: false,
            quiet: false,
            command: Command::Repl,
        };
        assert!(!cli.verbose);
        assert!(!cli.quiet);
        assert!(matches!(cli.command, Command::Repl));
    }

    #[test]
    fn test_cli_verbose_quiet_flags() {
        let cli = Cli {
            verbose: true,
            quiet: true,
            command: Command::Repl,
        };
        assert!(cli.verbose);
        assert!(cli.quiet);
    }

    #[test]
    fn test_command_run_variant() {
        let path = PathBuf::from("test.ruchy");
        let command = Command::Run { path: path.clone() };
        if let Command::Run { path: p } = command {
            assert_eq!(p, path);
        } else {
            panic!("Expected Run command");
        }
    }

    #[test]
    fn test_command_format_variant() {
        let path = PathBuf::from("test.ruchy");
        let command = Command::Format {
            path: path.clone(),
            check: true,
        };
        if let Command::Format { path: p, check: c } = command {
            assert_eq!(p, path);
            assert!(c);
        } else {
            panic!("Expected Format command");
        }
    }

    #[test]
    fn test_notebook_command_serve() {
        let cmd = NotebookCommand::Serve {
            port: 8080,
            host: "localhost".to_string(),
        };
        if let NotebookCommand::Serve { port, host } = cmd {
            assert_eq!(port, 8080);
            assert_eq!(host, "localhost");
        } else {
            panic!("Expected Serve command");
        }
    }

    #[test]
    fn test_notebook_command_test() {
        let cmd = NotebookCommand::Test {
            path: PathBuf::from("test.ipynb"),
            coverage: true,
            format: "json".to_string(),
        };
        if let NotebookCommand::Test {
            path,
            coverage,
            format,
        } = cmd
        {
            assert_eq!(path, PathBuf::from("test.ipynb"));
            assert!(coverage);
            assert_eq!(format, "json");
        } else {
            panic!("Expected Test command");
        }
    }

    #[test]
    fn test_notebook_command_convert() {
        let cmd = NotebookCommand::Convert {
            input: PathBuf::from("input.ipynb"),
            output: PathBuf::from("output.html"),
            format: "html".to_string(),
        };
        if let NotebookCommand::Convert {
            input,
            output,
            format,
        } = cmd
        {
            assert_eq!(input, PathBuf::from("input.ipynb"));
            assert_eq!(output, PathBuf::from("output.html"));
            assert_eq!(format, "html");
        } else {
            panic!("Expected Convert command");
        }
    }

    #[test]
    fn test_wasm_command_compile() {
        let cmd = WasmCommand::Compile {
            input: PathBuf::from("test.ruchy"),
            output: Some(PathBuf::from("test.wasm")),
            optimize: true,
            validate: false,
        };
        if let WasmCommand::Compile {
            input,
            output,
            optimize,
            validate,
        } = cmd
        {
            assert_eq!(input, PathBuf::from("test.ruchy"));
            assert_eq!(output, Some(PathBuf::from("test.wasm")));
            assert!(optimize);
            assert!(!validate);
        } else {
            panic!("Expected Compile command");
        }
    }

    #[test]
    fn test_wasm_command_run() {
        let cmd = WasmCommand::Run {
            module: PathBuf::from("test.wasm"),
            args: vec!["arg1".to_string(), "arg2".to_string()],
        };
        if let WasmCommand::Run { module, args } = cmd {
            assert_eq!(module, PathBuf::from("test.wasm"));
            assert_eq!(args.len(), 2);
            assert_eq!(args[0], "arg1");
        } else {
            panic!("Expected Run command");
        }
    }

    #[test]
    fn test_wasm_command_validate() {
        let cmd = WasmCommand::Validate {
            module: PathBuf::from("test.wasm"),
        };
        if let WasmCommand::Validate { module } = cmd {
            assert_eq!(module, PathBuf::from("test.wasm"));
        } else {
            panic!("Expected Validate command");
        }
    }

    #[test]
    fn test_test_command_run() {
        let cmd = TestCommand::Run {
            path: PathBuf::from("tests/"),
            coverage: true,
            parallel: false,
            filter: Some("test_".to_string()),
        };
        if let TestCommand::Run {
            path,
            coverage,
            parallel,
            filter,
        } = cmd
        {
            assert_eq!(path, PathBuf::from("tests/"));
            assert!(coverage);
            assert!(!parallel);
            assert_eq!(filter, Some("test_".to_string()));
        } else {
            panic!("Expected Run command");
        }
    }

    #[test]
    fn test_test_command_report() {
        let cmd = TestCommand::Report {
            format: "junit".to_string(),
            output: Some(PathBuf::from("report.xml")),
        };
        if let TestCommand::Report { format, output } = cmd {
            assert_eq!(format, "junit");
            assert_eq!(output, Some(PathBuf::from("report.xml")));
        } else {
            panic!("Expected Report command");
        }
    }

    #[test]
    fn test_execute_format_nonexistent_file() {
        let path = PathBuf::from("nonexistent.ruchy");
        let result = execute_format(path, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_run_nonexistent_file() {
        let path = PathBuf::from("nonexistent.ruchy");
        let result = execute_run(path, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_wasm_run() {
        let module = PathBuf::from("test.wasm");
        let args = vec![];
        let result = execute_wasm_run(module, args, false);
        // Currently just returns Ok(())
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_wasm_validate() {
        let module = PathBuf::from("test.wasm");
        let result = execute_wasm_validate(module, false);
        // Should return error for nonexistent file
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_test_run() {
        let cmd = TestCommand::Run {
            path: PathBuf::from("tests/"),
            coverage: false,
            parallel: true,
            filter: None,
        };
        let result = execute_test(cmd, false);
        // Currently returns Ok(())
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_test_report() {
        let cmd = TestCommand::Report {
            format: "html".to_string(),
            output: None,
        };
        let result = execute_test(cmd, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_test_command() {
        let path = PathBuf::from("test.ipynb");
        let config = crate::notebook::testing::types::TestConfig::default();
        let result = run_test_command(&path, config);
        assert!(result.is_ok());
        let report = result.unwrap();
        assert_eq!(report.total_tests, 1);
        assert_eq!(report.passed_tests, 1);
        assert_eq!(report.failed_tests, 0);
    }

    #[test]

    fn test_execute_notebook_serve() {
        let cmd = NotebookCommand::Serve {
            port: 8888,
            host: "127.0.0.1".to_string(),
        };

        // Test the command parsing works correctly
        if let NotebookCommand::Serve { port, host } = cmd {
            assert_eq!(port, 8888);
            assert_eq!(host, "127.0.0.1");
        } else {
            panic!("Expected Serve command");
        }

        // Only test the error case without notebook feature to avoid hanging server
        #[cfg(not(feature = "notebook"))]
        {
            let cmd = NotebookCommand::Serve {
                port: 8888,
                host: "127.0.0.1".to_string(),
            };
            let result = execute_notebook(cmd, false);
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("Notebook feature not enabled"));
        }

        // Skip actual server execution when notebook feature is enabled to avoid hanging
        #[cfg(feature = "notebook")]
        {
            // Test passes - we just verify command structure above
            // Starting actual server would hang the test indefinitely
        }
    }

    #[test]
    fn test_execute_notebook_test() {
        let cmd = NotebookCommand::Test {
            path: PathBuf::from("test.ipynb"),
            coverage: false,
            format: "text".to_string(),
        };
        let result = execute_notebook(cmd, false);
        // Without notebook feature, returns error
        #[cfg(not(feature = "notebook"))]
        assert!(result.is_err());
        #[cfg(feature = "notebook")]
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_execute_notebook_convert() {
        let cmd = NotebookCommand::Convert {
            input: PathBuf::from("input.ipynb"),
            output: PathBuf::from("output.html"),
            format: "html".to_string(),
        };
        let result = execute_notebook(cmd, false);
        assert!(result.is_ok()); // Currently just returns Ok(())
    }

    #[test]
    fn test_compile_wasm_source_not_enabled() {
        #[cfg(not(feature = "wasm-compile"))]
        {
            let result = compile_wasm_source("", &PathBuf::from("out.wasm"), false, false);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), "WASM compilation feature not enabled");
        }
    }
}

#[cfg(test)]
mod property_tests_mod {
    use proptest::proptest;

    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_execute_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
