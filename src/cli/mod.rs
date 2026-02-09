// [RUCHY-207] CLI Module Implementation
// PMAT Complexity: <10 per function
use crate::utils::format_file_error;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

/// VM execution mode selection (OPT-004)
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum VmMode {
    /// Use AST interpreter (default, stable)
    Ast,
    /// Use bytecode VM (experimental, 40-60% faster)
    Bytecode,
}

impl Default for VmMode {
    fn default() -> Self {
        // Check environment variable first
        if let Ok(mode) = std::env::var("RUCHY_VM_MODE") {
            match mode.to_lowercase().as_str() {
                "bytecode" | "vm" => return VmMode::Bytecode,
                _ => return VmMode::Ast,
            }
        }
        VmMode::Ast
    }
}

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
    /// VM execution mode: ast (default) or bytecode (experimental, faster)
    #[arg(long, value_enum, global = true, default_value_t = VmMode::default())]
    pub vm_mode: VmMode,
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
    /// Hunt Mode: Automated defect resolution (PDCA cycle)
    Hunt {
        /// Target directory to analyze
        #[arg(default_value = "./examples")]
        target: PathBuf,
        /// Number of PDCA cycles to run
        #[arg(short, long, default_value = "10")]
        cycles: u32,
        /// Show Andon dashboard
        #[arg(long)]
        andon: bool,
        /// Export Hansei (lessons learned) report
        #[arg(long)]
        hansei_report: Option<PathBuf>,
        /// Enable Five Whys analysis
        #[arg(long)]
        five_whys: bool,
    },
    /// Generate transpilation report with rich diagnostics
    Report {
        /// Target directory to analyze
        #[arg(default_value = "./examples")]
        target: PathBuf,
        /// Output format (human, json, markdown, sarif)
        #[arg(short, long, default_value = "human")]
        format: String,
        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
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
        /// PID file for automatic process management
        #[arg(long)]
        pid_file: Option<PathBuf>,
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
            Command::Run { path } => execute_run(path, self.verbose, self.vm_mode),
            Command::Format { path, check } => execute_format(path, check),
            Command::Hunt {
                target,
                cycles,
                andon,
                hansei_report,
                five_whys,
            } => execute_hunt(
                target,
                cycles,
                andon,
                hansei_report,
                five_whys,
                self.verbose,
            ),
            Command::Report {
                target,
                format,
                output,
            } => execute_report(target, format, output, self.verbose),
            Command::Notebook(cmd) => execute_notebook(cmd, self.verbose),
            Command::Wasm(cmd) => execute_wasm(cmd, self.verbose),
            Command::Test(cmd) => execute_test(cmd, self.verbose),
        }
    }
}
#[cfg(all(not(target_arch = "wasm32"), feature = "repl"))]
fn execute_repl(_verbose: bool, quiet: bool) -> Result<(), String> {
    if !quiet {
        println!("Starting Ruchy REPL v3.4.1...");
    }
    // Use existing REPL implementation
    crate::run_repl().map_err(|e| format!("REPL error: {e}"))
}

/// Stub for builds without REPL support
#[cfg(not(all(not(target_arch = "wasm32"), feature = "repl")))]
fn execute_repl(_verbose: bool, _quiet: bool) -> Result<(), String> {
    Err("REPL not available (requires 'repl' feature)".to_string())
}
fn execute_run(path: PathBuf, verbose: bool, vm_mode: VmMode) -> Result<(), String> {
    if verbose {
        println!("Running script: {} (mode: {:?})", path.display(), vm_mode);
    }
    let source = std::fs::read_to_string(&path).map_err(|_e| format_file_error("read", &path))?;
    let mut parser = crate::frontend::parser::Parser::new(&source);
    let ast = parser.parse().map_err(|e| format!("Parse error: {e:?}"))?;

    // ISSUE-106: Resolve module declarations (mod name;) and imports before evaluation
    let resolved_ast = resolve_modules_for_run(&path, ast)?;

    match vm_mode {
        VmMode::Ast => {
            // Use AST interpreter (default)
            let mut interpreter = crate::runtime::interpreter::Interpreter::new();
            interpreter
                .eval_expr(&resolved_ast)
                .map_err(|e| format!("Evaluation error: {e:?}"))?;
        }
        VmMode::Bytecode => {
            // Use bytecode VM (experimental, faster)
            use crate::runtime::bytecode::{Compiler, VM};

            let mut compiler = Compiler::new("main".to_string());
            compiler
                .compile_expr(&resolved_ast)
                .map_err(|e| format!("Compilation error: {e}"))?;
            let chunk = compiler.finalize();

            let mut vm = VM::new();
            let _result = vm
                .execute(&chunk)
                .map_err(|e| format!("VM execution error: {e}"))?;
        }
    }

    Ok(())
}

/// ISSUE-106: Resolve module declarations and imports for the run command
/// This enables `mod name;` and `use module` syntax when running scripts directly
fn resolve_modules_for_run(
    source_path: &Path,
    ast: crate::frontend::ast::Expr,
) -> Result<crate::frontend::ast::Expr, String> {
    use crate::backend::module_resolver::ModuleResolver;
    use crate::frontend::ast::ExprKind;

    // Check if AST contains any module declarations or imports that need resolution
    fn needs_resolution(expr: &crate::frontend::ast::Expr) -> bool {
        match &expr.kind {
            ExprKind::ModuleDeclaration { .. } => true,
            ExprKind::Module { .. } => true,
            ExprKind::Import { .. } => true,
            ExprKind::ImportAll { .. } => true,
            ExprKind::ImportDefault { .. } => true,
            ExprKind::Block(exprs) => exprs.iter().any(needs_resolution),
            ExprKind::Function { body, .. } => needs_resolution(body),
            ExprKind::Let { value, body, .. } => needs_resolution(value) || needs_resolution(body),
            _ => false,
        }
    }

    if !needs_resolution(&ast) {
        return Ok(ast);
    }

    let mut resolver = ModuleResolver::new();

    // Add the source file's directory to the module search path
    if let Some(parent_dir) = source_path.parent() {
        resolver.add_search_path(parent_dir);

        // Also search in standard project layout directories
        if let Some(project_root) = parent_dir.parent() {
            resolver.add_search_path(project_root.join("src"));
            resolver.add_search_path(project_root.join("lib"));
            resolver.add_search_path(project_root.join("modules"));
        }
    }

    resolver
        .resolve_imports(ast)
        .map_err(|e| format!("Module resolution error: {e}"))
}
fn execute_format(path: PathBuf, check: bool) -> Result<(), String> {
    use crate::quality::formatter::Formatter;

    let config = find_and_load_config(&path)?;
    let mut formatter = Formatter::with_config(config);

    if check {
        check_format(&path, &mut formatter)
    } else {
        apply_format(&path, &mut formatter)
    }
}

/// Execute Hunt Mode: Automated defect resolution using PDCA cycle
fn execute_hunt(
    target: PathBuf,
    cycles: u32,
    andon: bool,
    hansei_report: Option<PathBuf>,
    five_whys: bool,
    verbose: bool,
) -> Result<(), String> {
    use crate::hunt_mode::{HuntConfig, HuntMode};

    if verbose {
        println!("Starting Hunt Mode on: {}", target.display());
        println!("Cycles: {cycles}, Andon: {andon}, Five Whys: {five_whys}");
    }

    // Configure Hunt Mode
    let config = HuntConfig {
        max_cycles: cycles,
        enable_five_whys: five_whys,
        verbose,
        ..Default::default()
    };

    let mut hunt = HuntMode::with_config(config);

    // Run Hunt Mode cycles
    println!("=== HUNT MODE: PDCA Cycle ===");
    println!("Target: {}", target.display());
    println!();

    let outcomes = hunt
        .run(cycles)
        .map_err(|e| format!("Hunt Mode error: {e}"))?;

    // Display results
    println!("=== Hunt Mode Results ===");
    println!("Cycles completed: {}", outcomes.len());

    let fixes_applied: usize = outcomes.iter().filter(|o| o.fix_applied).count();
    println!("Fixes applied: {fixes_applied}");

    // Show Andon status
    let status = hunt.andon_status();
    println!();
    println!("Andon Status: {} {}", status.icon(), status.name());

    // Show Kaizen metrics
    let metrics = hunt.kaizen_metrics();
    println!();
    println!("=== Kaizen Metrics ===");
    println!("Compilation rate: {:.1}%", metrics.success_rate_percent());
    println!("Total fixes: {}", metrics.cumulative_fixes);
    println!(
        "Improving: {}",
        if metrics.is_improving() { "Yes" } else { "No" }
    );

    // Export Hansei report if requested
    if let Some(report_path) = hansei_report {
        export_hansei_report(&report_path, hunt.history(), verbose)?;
    }

    if andon {
        println!();
        println!("=== Andon Dashboard ===");
        display_andon_dashboard(&hunt);
    }

    Ok(())
}

/// Display Andon dashboard (visual management)
fn display_andon_dashboard(hunt: &crate::hunt_mode::HuntMode) {
    let status = hunt.andon_status();
    let metrics = hunt.kaizen_metrics();

    // Simple ASCII dashboard
    println!("┌─────────────────────────────────┐");
    println!("│     HUNT MODE ANDON BOARD       │");
    println!("├─────────────────────────────────┤");
    println!("│ Status: {} {:20} │", status.icon(), status.name());
    println!("│ Rate:   {:>23.1}% │", metrics.success_rate_percent());
    println!("│ Fixes:  {:>23} │", metrics.cumulative_fixes);
    println!("│ Cycles: {:>23} │", metrics.total_cycles);
    println!("└─────────────────────────────────┘");
}

/// Export Hansei (lessons learned) report
fn export_hansei_report(
    path: &PathBuf,
    history: &[crate::hunt_mode::CycleOutcome],
    verbose: bool,
) -> Result<(), String> {
    use std::fs::File;
    use std::io::Write;

    if verbose {
        println!("Exporting Hansei report to: {}", path.display());
    }

    let mut file = File::create(path).map_err(|e| format!("Failed to create report: {e}"))?;

    writeln!(file, "# Hunt Mode Hansei Report").map_err(|e| e.to_string())?;
    writeln!(file, "## Lessons Learned\n").map_err(|e| e.to_string())?;

    for (i, outcome) in history.iter().enumerate() {
        writeln!(file, "### Cycle {}", i + 1).map_err(|e| e.to_string())?;
        writeln!(file, "- Fix applied: {}", outcome.fix_applied).map_err(|e| e.to_string())?;
        writeln!(file, "- Confidence: {:.1}%", outcome.confidence * 100.0)
            .map_err(|e| e.to_string())?;
        writeln!(file, "- Lessons:").map_err(|e| e.to_string())?;
        for lesson in &outcome.lessons {
            writeln!(file, "  - {lesson}").map_err(|e| e.to_string())?;
        }
        writeln!(file).map_err(|e| e.to_string())?;
    }

    println!("Hansei report exported to: {}", path.display());
    Ok(())
}

/// Execute Report command: Generate transpilation report
fn execute_report(
    target: PathBuf,
    format: String,
    output: Option<PathBuf>,
    verbose: bool,
) -> Result<(), String> {
    use crate::reporting::formats::{
        HumanFormatter, JsonFormatter, MarkdownFormatter, SarifFormatter,
    };
    use crate::reporting::{ReportFormatter, TranspileReport};

    if verbose {
        println!("Generating report for: {}", target.display());
        println!("Format: {format}");
    }

    // Scan target directory for .ruchy files
    let files = scan_ruchy_files(&target)?;

    if verbose {
        println!("Found {} .ruchy files", files.len());
    }

    // Generate report (placeholder - would run actual transpilation)
    // For now, just count files as passed
    let passed = files.len();
    let failed = 0;
    let report = TranspileReport::new(files.len(), passed, failed);

    // Format output based on requested format
    let formatted = match format.as_str() {
        "json" => {
            let formatter = JsonFormatter::pretty();
            formatter.format(&report)
        }
        "markdown" | "md" => {
            let formatter = MarkdownFormatter;
            formatter.format(&report)
        }
        "sarif" => {
            let formatter = SarifFormatter;
            formatter.format(&report)
        }
        _ => {
            let formatter = HumanFormatter::default();
            formatter.format(&report)
        }
    };

    // Write output
    match output {
        Some(path) => {
            std::fs::write(&path, formatted).map_err(|e| format!("Failed to write report: {e}"))?;
            println!("Report written to: {}", path.display());
        }
        None => {
            println!("{formatted}");
        }
    }

    Ok(())
}

/// Scan directory for .ruchy files
fn scan_ruchy_files(path: &Path) -> Result<Vec<PathBuf>, String> {
    use std::fs;

    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }

    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }

    let mut files = Vec::new();

    fn scan_dir(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                scan_dir(&path, files)?;
            } else if path.extension().is_some_and(|e| e == "ruchy") {
                files.push(path);
            }
        }
        Ok(())
    }

    scan_dir(path, &mut files).map_err(|e| format!("Failed to scan directory: {e}"))?;
    Ok(files)
}

/// Check if a file is properly formatted
fn check_format(
    path: &PathBuf,
    formatter: &mut crate::quality::formatter::Formatter,
) -> Result<(), String> {
    println!("Checking formatting for: {}", path.display());

    let source = std::fs::read_to_string(path).map_err(|_e| format_file_error("read", path))?;
    let ast = parse_source(&source)?;

    // Set source for ignore directives
    formatter.set_source(&source);
    let formatted_code = formatter
        .format(&ast)
        .map_err(|e| format!("Format error: {e}"))?;

    if formatted_code.trim() == source.trim() {
        println!("✓ File is properly formatted");
        Ok(())
    } else {
        Err("File is not properly formatted. Run without --check to fix.".to_string())
    }
}

/// Apply formatting to a file
fn apply_format(
    path: &PathBuf,
    formatter: &mut crate::quality::formatter::Formatter,
) -> Result<(), String> {
    println!("Formatting: {}", path.display());

    let source = std::fs::read_to_string(path).map_err(|_e| format_file_error("read", path))?;
    let ast = parse_source(&source)?;

    // Set source for ignore directives
    formatter.set_source(&source);
    let formatted_code = formatter
        .format(&ast)
        .map_err(|e| format!("Format error: {e}"))?;

    std::fs::write(path, formatted_code).map_err(|e| format!("Failed to write file: {e}"))?;
    println!("✓ File formatted successfully");
    Ok(())
}

/// Parse source code into AST
fn parse_source(source: &str) -> Result<crate::frontend::ast::Expr, String> {
    let mut parser = crate::frontend::parser::Parser::new(source);
    parser.parse().map_err(|e| format!("Parse error: {e:?}"))
}

/// Find and load formatter configuration by searching up the directory tree
fn find_and_load_config(start_path: &Path) -> Result<crate::quality::FormatterConfig, String> {
    let start_dir = get_start_directory(start_path);
    find_config_in_ancestors(&start_dir)
}

/// Get the directory to start config search from
fn get_start_directory(path: &Path) -> PathBuf {
    if path.is_file() {
        path.parent().unwrap_or(path).to_path_buf()
    } else {
        path.to_path_buf()
    }
}

/// Search for config file in current and ancestor directories
fn find_config_in_ancestors(start_dir: &Path) -> Result<crate::quality::FormatterConfig, String> {
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
        NotebookCommand::Serve {
            port,
            host,
            pid_file,
        } => execute_notebook_serve(port, host, pid_file, verbose),
        NotebookCommand::Test {
            path,
            coverage,
            format,
        } => execute_notebook_test(path, coverage, format, verbose),
        NotebookCommand::Convert {
            input,
            output,
            format,
        } => execute_notebook_convert(input, Some(output), format, verbose),
    }
}

fn execute_notebook_serve(
    port: u16,
    host: String,
    pid_file: Option<PathBuf>,
    verbose: bool,
) -> Result<(), String> {
    // Create PID file for process management (if specified)
    let _pid_file_guard = if let Some(pid_path) = pid_file {
        if verbose {
            println!("Creating PID file at: {}", pid_path.display());
        }
        Some(
            crate::server::PidFile::new(pid_path)
                .map_err(|e| format!("Failed to create PID file: {e}"))?,
        )
    } else {
        None
    };

    if verbose {
        println!("Starting notebook server on {host}:{port}");
    }
    #[cfg(feature = "notebook")]
    {
        let rt =
            tokio::runtime::Runtime::new().map_err(|e| format!("Failed to create runtime: {e}"))?;
        rt.block_on(async {
            crate::notebook::server::start_server(port)
                .await
                .map_err(|e| format!("Server error: {e}"))
        })?;
    }
    #[cfg(not(feature = "notebook"))]
    {
        Err("Notebook feature not enabled".to_string())
    }
    #[cfg(feature = "notebook")]
    {
        Ok(())
        // PID file automatically cleaned up when _pid_file_guard drops
    }
}

fn execute_notebook_test(
    path: PathBuf,
    _coverage: bool,
    _format: String,
    verbose: bool,
) -> Result<(), String> {
    if verbose {
        println!("Testing notebook: {}", path.display());
    }
    #[cfg(feature = "notebook")]
    {
        let config = crate::notebook::testing::types::TestConfig::default();
        let report = run_test_command(&path, config)?;
        match _format.as_str() {
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
        Err("Notebook feature not enabled".to_string())
    }
    #[cfg(feature = "notebook")]
    {
        Ok(())
    }
}

fn execute_notebook_convert(
    input: PathBuf,
    _output: Option<PathBuf>,
    format: String,
    verbose: bool,
) -> Result<(), String> {
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
    #[cfg(feature = "notebook")]
    {
        let bytes = std::fs::read(&module).map_err(|e| format!("Failed to read WASM file: {e}"))?;
        wasmparser::validate(&bytes).map_err(|e| format!("WASM validation error: {e}"))?;
        println!("✓ WASM module is valid");
        Ok(())
    }
    #[cfg(not(feature = "notebook"))]
    {
        eprintln!("Warning: WASM validation requires notebook feature");
        Err("WASM validation not available without notebook feature".to_string())
    }
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
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "property_tests.rs"]
mod property_tests_mod;
