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
#![allow(clippy::redundant_field_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::format_in_format_args)]
#![allow(clippy::items_after_statements)]
#![allow(dead_code)]
use anyhow::Result;
use clap::{Parser, Subcommand};
// use colored::Colorize; // Unused after refactoring
use ruchy::{runtime::repl::Repl, Parser as RuchyParser};
use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::{Path, PathBuf};
mod handlers;
use handlers::{
    handle_check_command, handle_compile_command, handle_complex_command, handle_eval_command,
    handle_file_execution, handle_fuzz_command, handle_mutations_command, handle_parse_command,
    handle_property_tests_command, handle_repl_command, handle_run_command, handle_stdin_input,
    handle_test_command, handle_transpile_command, VmMode,
};
/// Configuration for code formatting
#[derive(Debug, Clone)]
struct FormatConfig {
    #[allow(dead_code)]
    line_width: usize,
    indent: usize,
    use_tabs: bool,
}
impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            line_width: 100,
            indent: 4,
            use_tabs: false,
        }
    }
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
    /// Enable verbose output
    #[arg(short = 'v', long)]
    verbose: bool,
    /// Enable execution tracing (DEBUGGER-014, Issue #84)
    #[arg(long)]
    trace: bool,
    /// VM execution mode: ast (default) or bytecode (experimental, faster)
    #[arg(long, value_enum, default_value = "ast")]
    vm_mode: VmMode,
    /// Script file to execute (alternative to subcommands)
    file: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Commands>,
}
#[derive(Subcommand)]
enum Commands {
    /// Start the interactive REPL
    Repl {
        /// Record REPL session to a .replay file
        #[arg(long, value_name = "FILE")]
        record: Option<PathBuf>,
    },
    /// Create a new Ruchy project with Cargo integration
    New {
        /// Project name
        name: String,
        /// Create a library instead of a binary
        #[arg(long)]
        lib: bool,
    },
    /// Build a Ruchy project (wrapper around cargo build)
    Build {
        /// Build in release mode with optimizations
        #[arg(long)]
        release: bool,
    },
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
        /// Use minimal codegen for self-hosting (direct Rust mapping, no optimization)
        #[arg(long)]
        minimal: bool,
    },
    /// Compile and run a Ruchy file
    Run {
        /// The file to run
        file: PathBuf,
    },
    /// Compile a Ruchy file to a standalone binary (RUCHY-0801)
    Compile {
        /// The file to compile
        file: PathBuf,
        /// Output binary path
        #[arg(short, long, default_value = "a.out")]
        output: PathBuf,
        /// Optimization level (0-3, or 's' for size)
        #[arg(short = 'O', long, default_value = "2")]
        opt_level: String,
        /// High-level optimization preset (OPTIMIZATION-001)
        #[arg(long)]
        optimize: Option<String>,
        /// Strip debug symbols
        #[arg(long)]
        strip: bool,
        /// Static linking
        #[arg(long)]
        static_link: bool,
        /// Target triple (e.g., x86_64-unknown-linux-gnu)
        #[arg(long)]
        target: Option<String>,
        /// Show verbose compilation details
        #[arg(long)]
        verbose: bool,
        /// Output compilation metrics to JSON file
        #[arg(long)]
        json: Option<PathBuf>,
        /// Show profile characteristics before compilation (PERF-002 Phase 3)
        #[arg(long)]
        show_profile_info: bool,
        /// Enable Profile-Guided Optimization (two-step build) (PERF-002 Phase 4)
        #[arg(long)]
        pgo: bool,
    },
    /// Check syntax without running
    Check {
        /// The file(s) to check
        files: Vec<PathBuf>,
        /// Watch for changes and re-check automatically
        #[arg(long)]
        watch: bool,
    },
    /// Run tests for Ruchy code with optional coverage reporting
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
        /// Generate coverage report
        #[arg(long)]
        coverage: bool,
        /// Coverage output format (text, html, json)
        #[arg(long, default_value = "text")]
        coverage_format: String,
        /// Run tests in parallel
        #[arg(long)]
        parallel: bool,
        /// Minimum coverage threshold (fail if below)
        #[arg(long)]
        threshold: Option<f64>,
        /// Output format for test results (text, json, junit)
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Launch interactive notebook server
    Notebook {
        /// Optional file to validate in non-interactive mode (TOOL-VALIDATION-003)
        file: Option<PathBuf>,
        /// Port to run the server on
        #[arg(short, long, default_value = "8080")]
        port: u16,
        /// Open browser automatically
        #[arg(long)]
        open: bool,
        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
    /// Serve static files over HTTP (HTTP-001)
    Serve {
        /// Directory to serve (defaults to current directory)
        #[arg(default_value = ".")]
        directory: PathBuf,
        /// Port to run the server on
        #[arg(short, long, default_value = "8080")]
        port: u16,
        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Show verbose logging
        #[arg(long)]
        verbose: bool,
        /// Watch files and auto-restart on changes
        #[arg(long)]
        watch: bool,
        /// Debounce delay in milliseconds (default: 300)
        #[arg(long, default_value = "300")]
        debounce: u64,
        /// PID file for process management
        #[arg(long)]
        pid_file: Option<PathBuf>,
        /// Watch .ruchy files and rebuild WASM
        #[arg(long)]
        watch_wasm: bool,
    },
    /// Generate coverage report for Ruchy code
    Coverage {
        /// The file or directory to analyze  
        path: PathBuf,
        /// Minimum coverage threshold (fail if below)
        #[arg(long)]
        threshold: Option<f64>,
        /// Output format for coverage report (text, html, json)
        #[arg(long, default_value = "text")]
        format: String,
        /// Show verbose coverage output
        #[arg(long)]
        verbose: bool,
    },
    /// Show AST for a file (Enhanced for v0.9.12)
    Ast {
        /// The file to parse
        file: PathBuf,
        /// Output AST in JSON format for tooling integration
        #[arg(long)]
        json: bool,
        /// Generate visual AST graph in DOT format
        #[arg(long)]
        graph: bool,
        /// Calculate and show complexity metrics
        #[arg(long)]
        metrics: bool,
        /// Perform symbol table analysis
        #[arg(long)]
        symbols: bool,
        /// Analyze module dependencies
        #[arg(long)]
        deps: bool,
        /// Show verbose analysis output
        #[arg(long)]
        verbose: bool,
        /// Output file for graph/analysis results
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Formal verification and correctness analysis (RUCHY-0754)
    Provability {
        /// The file to analyze
        file: PathBuf,
        /// Perform full formal verification
        #[arg(long)]
        verify: bool,
        /// Contract verification (pre/post-conditions, invariants)
        #[arg(long)]
        contracts: bool,
        /// Loop invariant checking
        #[arg(long)]
        invariants: bool,
        /// Termination analysis for loops and recursion
        #[arg(long)]
        termination: bool,
        /// Array bounds checking and memory safety
        #[arg(long)]
        bounds: bool,
        /// Show verbose verification output
        #[arg(long)]
        verbose: bool,
        /// Output file for verification results
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Performance analysis and `BigO` complexity detection (RUCHY-0755)
    Runtime {
        /// The file to analyze
        file: PathBuf,
        /// Perform detailed execution profiling
        #[arg(long)]
        profile: bool,
        /// Profile transpiled binary instead of interpreter (PROFILING-001, Issue #138)
        #[arg(long)]
        binary: bool,
        /// Number of profiling iterations (default: 1 for binary, 10 for interpreter)
        #[arg(long)]
        iterations: Option<usize>,
        /// Automatic `BigO` algorithmic complexity analysis
        #[arg(long)]
        bigo: bool,
        /// Benchmark execution with statistical analysis
        #[arg(long)]
        bench: bool,
        /// Compare performance between two files
        #[arg(long)]
        compare: Option<PathBuf>,
        /// Memory usage and allocation analysis
        #[arg(long)]
        memory: bool,
        /// Show verbose performance output
        #[arg(long)]
        verbose: bool,
        /// Output file for performance results
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Unified quality scoring (RUCHY-0810)
    Score {
        /// The file or directory to score
        path: PathBuf,
        /// Analysis depth (shallow/standard/deep)
        #[arg(long, default_value = "standard")]
        depth: String,
        /// Fast feedback mode (AST-only, <100ms)
        #[arg(long)]
        fast: bool,
        /// Deep analysis for CI (complete, <30s)
        #[arg(long)]
        deep: bool,
        /// Watch mode with progressive refinement
        #[arg(long)]
        watch: bool,
        /// Explain score changes from baseline
        #[arg(long)]
        explain: bool,
        /// Baseline branch/commit for comparison
        #[arg(long)]
        baseline: Option<String>,
        /// Minimum score threshold (0.0-1.0)
        #[arg(long)]
        min: Option<f64>,
        /// Configuration file
        #[arg(long)]
        config: Option<PathBuf>,
        /// Output format (text/json/html)
        #[arg(long, default_value = "text")]
        format: String,
        /// Verbose output
        #[arg(long)]
        verbose: bool,
        /// Output file for score report
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Quality gate enforcement (RUCHY-0815)
    QualityGate {
        /// The file or directory to check
        path: PathBuf,
        /// Configuration file (.ruchy/score.toml)
        #[arg(long)]
        config: Option<PathBuf>,
        /// Analysis depth (shallow/standard/deep)
        #[arg(long, default_value = "standard")]
        depth: String,
        /// Fail fast on first violation
        #[arg(long)]
        fail_fast: bool,
        /// Output format (console/json/junit)
        #[arg(long, default_value = "console")]
        format: String,
        /// Export CI/CD results
        #[arg(long)]
        export: Option<PathBuf>,
        /// Run in CI mode (strict thresholds)
        #[arg(long)]
        ci: bool,
        /// Show detailed violation information
        #[arg(long)]
        verbose: bool,
    },
    /// Format Ruchy source code (Enhanced for v0.9.12)
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
        /// Configuration file for formatting rules
        #[arg(long)]
        config: Option<PathBuf>,
        /// Maximum line width for formatting
        #[arg(long, default_value = "100")]
        line_width: usize,
        /// Indentation size (spaces)
        #[arg(long, default_value = "4")]
        indent: usize,
        /// Use tabs instead of spaces for indentation
        #[arg(long)]
        use_tabs: bool,
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
    /// Lint Ruchy source code for issues and style violations (Enhanced for v0.9.12)
    Lint {
        /// The file to lint (ignored if --all is used)
        file: Option<PathBuf>,
        /// Lint all files in project
        #[arg(long)]
        all: bool,
        /// Auto-fix issues where possible
        #[arg(long)]
        fix: bool,
        /// Strict mode with all rules enabled
        #[arg(long)]
        strict: bool,
        /// Show additional context for violations
        #[arg(long)]
        verbose: bool,
        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        format: String,
        /// Specific rule categories to check (comma-separated: unused,style,complexity,safety,performance)
        #[arg(long)]
        rules: Option<String>,
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
    /// Add a package dependency
    Add {
        /// Package name to add
        package: String,
        /// Specific version to add (default: latest)
        #[arg(long)]
        version: Option<String>,
        /// Add as development dependency
        #[arg(long)]
        dev: bool,
        /// Registry URL to use
        #[arg(long, default_value = "https://ruchy.dev/registry")]
        registry: String,
    },
    /// Publish a package to the registry
    Publish {
        /// Registry URL to publish to
        #[arg(long, default_value = "https://ruchy.dev/registry")]
        registry: String,
        /// Package version to publish (reads from Ruchy.toml if not specified)
        #[arg(long)]
        version: Option<String>,
        /// Perform a dry run without actually publishing
        #[arg(long)]
        dry_run: bool,
        /// Allow publishing dirty working directory
        #[arg(long)]
        allow_dirty: bool,
    },
    /// Start MCP server for real-time quality analysis (RUCHY-0811)
    Mcp {
        /// Server name for MCP identification
        #[arg(long, default_value = "ruchy-mcp")]
        name: String,
        /// Enable streaming updates
        #[arg(long)]
        streaming: bool,
        /// Session timeout in seconds
        #[arg(long, default_value = "3600")]
        timeout: u64,
        /// Minimum quality score threshold
        #[arg(long, default_value = "0.8")]
        min_score: f64,
        /// Maximum complexity threshold
        #[arg(long, default_value = "10")]
        max_complexity: u32,
        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
        /// Configuration file path
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    /// Hardware-aware optimization analysis (RUCHY-0816)
    Optimize {
        /// The file to analyze for optimization opportunities
        file: PathBuf,
        /// Hardware profile to use (detect, intel, amd, arm)
        #[arg(long, default_value = "detect")]
        hardware: String,
        /// Analysis depth (quick, standard, deep)
        #[arg(long, default_value = "standard")]
        depth: String,
        /// Show cache behavior analysis
        #[arg(long)]
        cache: bool,
        /// Show branch prediction analysis
        #[arg(long)]
        branches: bool,
        /// Show vectorization opportunities
        #[arg(long)]
        vectorization: bool,
        /// Show abstraction cost analysis
        #[arg(long)]
        abstractions: bool,
        /// Benchmark hardware characteristics
        #[arg(long)]
        benchmark: bool,
        /// Output format (text, json, html)
        #[arg(long, default_value = "text")]
        format: String,
        /// Save analysis to file
        #[arg(long)]
        output: Option<PathBuf>,
        /// Show verbose optimization details
        #[arg(long)]
        verbose: bool,
        /// Minimum impact threshold for recommendations (0.0-1.0)
        #[arg(long, default_value = "0.05")]
        threshold: f64,
    },
    /// Actor observatory for live system introspection (RUCHY-0817)
    #[command(name = "actor:observe")]
    ActorObserve {
        /// Actor system configuration file
        #[arg(long)]
        config: Option<PathBuf>,
        /// Observatory refresh interval in milliseconds
        #[arg(long, default_value = "1000")]
        refresh_interval: u64,
        /// Maximum number of message traces to display
        #[arg(long, default_value = "50")]
        max_traces: usize,
        /// Maximum number of actors to display
        #[arg(long, default_value = "20")]
        max_actors: usize,
        /// Enable deadlock detection
        #[arg(long)]
        enable_deadlock_detection: bool,
        /// Deadlock detection interval in milliseconds
        #[arg(long, default_value = "1000")]
        deadlock_interval: u64,
        /// Start in a specific view mode (overview, actors, messages, metrics, deadlocks)
        #[arg(long, default_value = "overview")]
        start_mode: String,
        /// Disable color output
        #[arg(long)]
        no_color: bool,
        /// Output format (interactive, json, text)
        #[arg(long, default_value = "interactive")]
        format: String,
        /// Export observations to file
        #[arg(long)]
        export: Option<PathBuf>,
        /// Duration to observe in seconds (0 for infinite)
        #[arg(long, default_value = "0")]
        duration: u64,
        /// Show verbose output
        #[arg(long)]
        verbose: bool,
        /// Add message filter by actor name pattern
        #[arg(long)]
        filter_actor: Option<String>,
        /// Add message filter for failed messages only
        #[arg(long)]
        filter_failed: bool,
        /// Add message filter for delayed messages (minimum microseconds)
        #[arg(long)]
        filter_slow: Option<u64>,
    },
    /// Dataflow debugger for `DataFrame` pipeline debugging (RUCHY-0818)
    #[command(name = "dataflow:debug")]
    DataflowDebug {
        /// Pipeline configuration file
        #[arg(long)]
        config: Option<PathBuf>,
        /// Maximum rows to materialize per stage
        #[arg(long, default_value = "1000")]
        max_rows: usize,
        /// Auto-materialize data at each stage
        #[arg(long)]
        auto_materialize: bool,
        /// Enable performance profiling
        #[arg(long, default_value = "true")]
        enable_profiling: bool,
        /// Stage execution timeout in milliseconds
        #[arg(long, default_value = "30000")]
        timeout: u64,
        /// Enable memory tracking
        #[arg(long)]
        track_memory: bool,
        /// Compute diffs between stages
        #[arg(long)]
        compute_diffs: bool,
        /// Sample rate for large datasets (0.0-1.0)
        #[arg(long, default_value = "1.0")]
        sample_rate: f64,
        /// UI refresh interval in milliseconds
        #[arg(long, default_value = "1000")]
        refresh_interval: u64,
        /// Disable color output
        #[arg(long)]
        no_color: bool,
        /// Output format (interactive, json, text)
        #[arg(long, default_value = "interactive")]
        format: String,
        /// Export debug data to file
        #[arg(long)]
        export: Option<PathBuf>,
        /// Show verbose debugging output
        #[arg(long)]
        verbose: bool,
        /// Add breakpoint at stage (can be used multiple times)
        #[arg(long)]
        breakpoint: Vec<String>,
        /// Start mode (overview, stages, data, metrics, history)
        #[arg(long, default_value = "overview")]
        start_mode: String,
    },
    /// WebAssembly component toolkit (RUCHY-0819)
    Wasm {
        /// The source file to compile to WASM
        file: PathBuf,
        /// Output file for the WASM component
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Target platform (wasm32, wasi, browser, nodejs, cloudflare-workers)
        #[arg(long, default_value = "wasm32")]
        target: String,
        /// Generate WIT interface definition
        #[arg(long)]
        wit: bool,
        /// Deploy to target platform
        #[arg(long)]
        deploy: bool,
        /// Deployment target (cloudflare, fastly, aws-lambda, vercel, deno)
        #[arg(long)]
        deploy_target: Option<String>,
        /// Analyze portability across platforms
        #[arg(long)]
        portability: bool,
        /// Optimization level (none, O1, O2, O3, Os, Oz)
        #[arg(long, default_value = "O2")]
        opt_level: String,
        /// Include debug information
        #[arg(long)]
        debug: bool,
        /// Enable SIMD instructions
        #[arg(long)]
        simd: bool,
        /// Enable threads and atomics
        #[arg(long)]
        threads: bool,
        /// Enable component model
        #[arg(long, default_value = "true")]
        component_model: bool,
        /// Component name
        #[arg(long)]
        name: Option<String>,
        /// Component version
        #[arg(long, default_value = "0.1.0")]
        version: String,
        /// Show verbose output
        #[arg(long)]
        verbose: bool,
    },
    /// Interactive theorem prover (RUCHY-0820)
    Prove {
        /// The file to verify (optional, starts REPL if not provided)
        file: Option<PathBuf>,
        /// SMT backend (z3, cvc5, yices2)
        #[arg(long, default_value = "z3")]
        backend: String,
        /// Enable ML-powered tactic suggestions
        #[arg(long)]
        ml_suggestions: bool,
        /// Timeout for SMT queries in milliseconds
        #[arg(long, default_value = "5000")]
        timeout: u64,
        /// Load proof script
        #[arg(long)]
        script: Option<PathBuf>,
        /// Export proof to file
        #[arg(long)]
        export: Option<PathBuf>,
        /// Non-interactive mode (check proofs only)
        #[arg(long)]
        check: bool,
        /// Generate counterexamples for failed proofs
        #[arg(long)]
        counterexample: bool,
        /// Show verbose proof output
        #[arg(long)]
        verbose: bool,
        /// Output format (text, json, coq, lean)
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Convert REPL replay files to regression tests
    ReplayToTests {
        /// Input replay file or directory containing .replay files
        input: PathBuf,
        /// Output test file (defaults to `tests/generated_from_replays.rs`)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Include property tests for invariants
        #[arg(long)]
        property_tests: bool,
        /// Include performance benchmarks
        #[arg(long)]
        benchmarks: bool,
        /// Test timeout in milliseconds
        #[arg(long, default_value = "5000")]
        timeout: u64,
    },
    /// Run property-based tests with configurable case count
    PropertyTests {
        /// Path to test file or directory
        path: PathBuf,
        /// Number of test cases per property
        #[arg(long, default_value = "10000")]
        cases: usize,
        /// Output format (text, json, markdown)
        #[arg(long, default_value = "text")]
        format: String,
        /// Output file
        #[arg(long)]
        output: Option<PathBuf>,
        /// Random seed for reproducibility
        #[arg(long)]
        seed: Option<u64>,
        /// Show verbose output
        #[arg(long)]
        verbose: bool,
    },
    /// Run mutation tests to validate test suite quality
    Mutations {
        /// Path to source file or directory
        path: PathBuf,
        /// Timeout per mutation (seconds)
        #[arg(long, default_value = "300")]
        timeout: u32,
        /// Output format (text, json, markdown, sarif)
        #[arg(long, default_value = "text")]
        format: String,
        /// Output file
        #[arg(long)]
        output: Option<PathBuf>,
        /// Minimum mutation coverage (0.0-1.0)
        #[arg(long, default_value = "0.75")]
        min_coverage: f64,
        /// Show verbose output
        #[arg(long)]
        verbose: bool,
    },
    /// Run fuzz tests to find crashes and panics
    Fuzz {
        /// Fuzz target name or path
        target: String,
        /// Number of iterations
        #[arg(long, default_value = "1000000")]
        iterations: usize,
        /// Timeout per iteration (ms)
        #[arg(long, default_value = "1000")]
        timeout: u32,
        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        format: String,
        /// Output file
        #[arg(long)]
        output: Option<PathBuf>,
        /// Show verbose output
        #[arg(long)]
        verbose: bool,
    },
}
fn main() -> Result<()> {
    // CLI-UNIFY-001: If no args provided, open REPL directly
    // This matches behavior of python, ruby, node, deno
    // Check before clap parsing to avoid showing help
    if std::env::args().len() == 1 {
        return handle_repl_command(None);
    }

    let cli = Cli::parse();
    // Try to handle direct evaluation first
    if let Some(result) = try_handle_direct_evaluation(&cli) {
        return result;
    }
    // Try to handle stdin input
    if let Some(result) = try_handle_stdin(cli.command.as_ref())? {
        return result;
    }
    // Handle subcommands
    handle_command_dispatch(cli.command, cli.verbose, cli.vm_mode)
}
/// Handle direct evaluation via -e flag or file argument (complexity: 4)
fn try_handle_direct_evaluation(cli: &Cli) -> Option<Result<()>> {
    // Handle one-liner evaluation with -e flag
    if let Some(expr) = &cli.eval {
        return Some(handle_eval_command(
            expr,
            cli.verbose,
            &cli.format,
            cli.trace,
        ));
    }
    // Handle script file execution (without subcommand)
    if let Some(file) = &cli.file {
        return Some(handle_file_execution(file));
    }
    None
}
/// Handle stdin input if present (complexity: 5)
fn try_handle_stdin(command: Option<&Commands>) -> Result<Option<Result<()>>> {
    // Check if stdin has input (piped mode) - but only when no command is specified
    if !io::stdin().is_terminal() && command.is_none() {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        if !input.trim().is_empty() {
            return Ok(Some(handle_stdin_input(&input)));
        }
    }
    Ok(None)
}
/// Dispatch commands to appropriate handlers (complexity: 6)
fn handle_command_dispatch(
    command: Option<Commands>,
    verbose: bool,
    vm_mode: VmMode,
) -> Result<()> {
    match command {
        Some(Commands::Repl { record }) => handle_repl_command(record),
        Some(Commands::New { name, lib }) => handlers::new::handle_new_command(&name, lib, verbose),
        Some(Commands::Build { release }) => {
            handlers::build::handle_build_command(release, verbose)
        }
        Some(Commands::Publish {
            registry,
            version,
            dry_run,
            allow_dirty,
        }) => handlers::handle_publish_command(
            &registry,
            version.as_deref(),
            dry_run,
            allow_dirty,
            verbose,
        ),
        None => handle_repl_command(None),
        Some(Commands::Parse { file }) => handle_parse_command(&file, verbose),
        Some(Commands::Transpile {
            file,
            output,
            minimal,
        }) => handle_transpile_command(&file, output.as_deref(), minimal, verbose),
        Some(Commands::Run { file }) => handle_run_command(&file, verbose, vm_mode),
        Some(Commands::Compile {
            file,
            output,
            opt_level,
            optimize,
            strip,
            static_link,
            target,
            verbose,
            json,
            show_profile_info,
            pgo,
        }) => handle_compile_command(
            &file,
            output,
            opt_level,
            optimize.as_deref(),
            strip,
            static_link,
            target,
            verbose,
            json.as_deref(),
            show_profile_info,
            pgo,
        ),
        Some(Commands::Check { files, watch }) => handle_check_command(&files, watch),
        Some(Commands::Test {
            path,
            watch,
            verbose,
            filter,
            coverage,
            coverage_format,
            parallel,
            threshold,
            format,
        }) => handle_test_dispatch(
            path,
            watch,
            verbose,
            filter.as_ref(),
            coverage,
            &coverage_format,
            parallel,
            threshold,
            &format,
        ),
        Some(Commands::PropertyTests {
            path,
            cases,
            format,
            output,
            seed,
            verbose,
        }) => {
            handle_property_tests_command(&path, cases, &format, output.as_deref(), seed, verbose)
        }
        Some(Commands::Mutations {
            path,
            timeout,
            format,
            output,
            min_coverage,
            verbose,
        }) => handle_mutations_command(
            &path,
            timeout,
            &format,
            output.as_deref(),
            min_coverage,
            verbose,
        ),
        Some(Commands::Fuzz {
            target,
            iterations,
            timeout,
            format,
            output,
            verbose,
        }) => handle_fuzz_command(
            &target,
            iterations,
            timeout,
            &format,
            output.as_deref(),
            verbose,
        ),
        Some(command) => handle_advanced_command(command),
    }
}
/// Handle test command with all its parameters (complexity: 3)
fn handle_test_dispatch(
    path: Option<PathBuf>,
    watch: bool,
    verbose: bool,
    filter: Option<&String>,
    coverage: bool,
    coverage_format: &str,
    parallel: bool,
    threshold: Option<f64>,
    format: &str,
) -> Result<()> {
    handle_test_command(
        path,
        watch,
        verbose,
        filter.map(String::as_str),
        coverage,
        coverage_format,
        usize::from(parallel),
        threshold.unwrap_or(0.0),
        format,
    )
}
fn handle_advanced_command(command: Commands) -> Result<()> {
    // Delegate to the existing handle_complex_command from cli module
    handle_complex_command(command)
}
fn run_file(file: &Path) -> Result<()> {
    let source = fs::read_to_string(file)?;
    // Use REPL to evaluate the file
    let mut repl = Repl::new(std::env::temp_dir())?;
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
    use colored::Colorize;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    #[test]
    fn test_format_config_default() {
        let config = FormatConfig::default();
        assert_eq!(config.line_width, 100);
        assert_eq!(config.indent, 4);
        assert!(!config.use_tabs);
    }

    #[test]
    fn test_format_config_creation() {
        let config = FormatConfig {
            line_width: 120,
            indent: 2,
            use_tabs: true,
        };
        assert_eq!(config.line_width, 120);
        assert_eq!(config.indent, 2);
        assert!(config.use_tabs);
    }

    #[test]
    fn test_try_handle_direct_evaluation_with_eval() {
        let cli = Cli {
            eval: Some("1 + 1".to_string()),
            format: "text".to_string(),
            verbose: false,
            vm_mode: VmMode::Ast,
            file: None,
            command: None,
            trace: false,
        };
        let result = try_handle_direct_evaluation(&cli);
        assert!(result.is_some());
    }

    #[test]
    fn test_try_handle_direct_evaluation_with_file() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "println(\"Hello World\")").unwrap();

        let cli = Cli {
            eval: None,
            format: "text".to_string(),
            verbose: false,
            vm_mode: VmMode::Ast,
            file: Some(temp_file.path().to_path_buf()),
            command: None,
            trace: false,
        };
        let result = try_handle_direct_evaluation(&cli);
        assert!(result.is_some());
    }

    #[test]
    fn test_try_handle_direct_evaluation_none() {
        let cli = Cli {
            eval: None,
            format: "text".to_string(),
            verbose: false,
            vm_mode: VmMode::Ast,
            file: None,
            command: None,
            trace: false,
        };
        let result = try_handle_direct_evaluation(&cli);
        assert!(result.is_none());
    }

    #[test]
    fn test_try_handle_stdin_no_command() {
        let result = try_handle_stdin(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_try_handle_stdin_with_command() {
        let command = Commands::Repl { record: None };
        let result = try_handle_stdin(Some(&command));
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_file_valid_syntax() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "let x = 42").unwrap();

        let result = run_file(temp_file.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_file_nonexistent() {
        let nonexistent_path = PathBuf::from("/nonexistent/file.ruchy");
        let result = run_file(&nonexistent_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_syntax_valid() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "let x = 42").unwrap();

        let result = check_syntax(temp_file.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_syntax_nonexistent_file() {
        let nonexistent_path = PathBuf::from("/nonexistent/file.ruchy");
        let result = check_syntax(&nonexistent_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_test_dispatch_basic() {
        let result =
            handle_test_dispatch(None, false, false, None, false, "text", false, None, "text");
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_test_dispatch_with_path() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "// Test file").unwrap();

        let result = handle_test_dispatch(
            Some(temp_file.path().to_path_buf()),
            false,
            true,
            None,
            false,
            "text",
            false,
            Some(0.8),
            "json",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_test_dispatch_with_filter() {
        let filter = "test_name".to_string();
        let result = handle_test_dispatch(
            None,
            false,
            false,
            Some(&filter),
            true,
            "html",
            true,
            Some(0.5),
            "junit",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_advanced_command_repl() {
        let command = Commands::Repl { record: None };
        let result = handle_advanced_command(command);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_advanced_command_parse() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "let x = 42").unwrap();

        let command = Commands::Parse {
            file: temp_file.path().to_path_buf(),
        };
        let result = handle_advanced_command(command);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_advanced_command_transpile() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "let x = 42").unwrap();

        let command = Commands::Transpile {
            file: temp_file.path().to_path_buf(),
            output: None,
            minimal: false,
        };
        let result = handle_advanced_command(command);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_advanced_command_compile() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "let x = 42").unwrap();

        let command = Commands::Compile {
            file: temp_file.path().to_path_buf(),
            output: PathBuf::from("test.out"),
            opt_level: "2".to_string(),
            optimize: None,
            strip: false,
            static_link: false,
            target: None,
            verbose: false,
            json: None,
            show_profile_info: false,
            pgo: false,
        };
        let result = handle_advanced_command(command);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_advanced_command_check() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "let x = 42").unwrap();

        let command = Commands::Check {
            files: vec![temp_file.path().to_path_buf()],
            watch: false,
        };
        let result = handle_advanced_command(command);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_advanced_command_notebook() {
        let command = Commands::Notebook {
            file: None,
            port: 8080,
            open: false,
            host: "127.0.0.1".to_string(),
        };
        let result = handle_advanced_command(command);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_advanced_command_coverage() {
        let temp_dir = tempfile::tempdir().unwrap();
        // Create a test file with some content
        let test_file = temp_dir.path().join("test.ruchy");
        fs::write(&test_file, "let x = 42;").unwrap();

        let command = Commands::Coverage {
            path: test_file, // Use the file path, not directory
            threshold: None, // Don't set threshold for test
            format: "html".to_string(),
            verbose: false,
        };
        let result = handle_advanced_command(command);
        if let Err(e) = &result {
            eprintln!("Coverage test error: {}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_advanced_command_ast() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "let x = 42").unwrap();

        let command = Commands::Ast {
            file: temp_file.path().to_path_buf(),
            json: false,
            graph: false,
            metrics: false,
            symbols: false,
            deps: false,
            verbose: false,
            output: None,
        };
        let result = handle_advanced_command(command);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_advanced_command_ast_with_options() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "let x = 42").unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let command = Commands::Ast {
            file: temp_file.path().to_path_buf(),
            json: true,
            graph: true,
            metrics: true,
            symbols: true,
            deps: true,
            verbose: true,
            output: Some(output_file.path().to_path_buf()),
        };
        let result = handle_advanced_command(command);
        assert!(result.is_ok());
    }

    // Note: fmt command testing removed - redundant with comprehensive formatter tests
    // in tests/cli_contract_fmt*.rs and tests/formatter_*.rs

    #[test]
    fn test_handle_advanced_command_doc() {
        let temp_file = NamedTempFile::new().unwrap();
        // TEST-FIX-002: Use valid Ruchy code instead of comment-only (empty program)
        fs::write(
            &temp_file,
            "/// Documentation test\nfun add(a, b) { a + b }",
        )
        .unwrap();

        let output_dir = tempfile::tempdir().unwrap();
        let command = Commands::Doc {
            path: temp_file.path().to_path_buf(),
            output: output_dir.path().to_path_buf(),
            format: "html".to_string(),
            private: false,
            open: false,
            all: false,
            verbose: false,
        };
        let result = handle_advanced_command(command);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_advanced_command_bench() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "let x = 42").unwrap();

        let command = Commands::Bench {
            file: temp_file.path().to_path_buf(),
            iterations: 10,
            warmup: 5,
            format: "json".to_string(),
            output: None,
            verbose: false,
        };
        let result = handle_advanced_command(command);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_advanced_command_lint() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "let x = 42").unwrap();

        let command = Commands::Lint {
            file: Some(temp_file.path().to_path_buf()),
            all: false,
            fix: false,
            strict: false,
            verbose: false,
            format: "text".to_string(),
            rules: None,
            deny_warnings: false,
            max_complexity: 10,
            config: None,
            init_config: false,
        };
        let result = handle_advanced_command(command);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_advanced_command_add() {
        let command = Commands::Add {
            package: "test_package".to_string(),
            version: Some("1.0.0".to_string()),
            dev: false,
            registry: "https://ruchy.dev/registry".to_string(),
        };
        let result = handle_advanced_command(command);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_advanced_command_publish() {
        let command = Commands::Publish {
            registry: "https://ruchy.dev/registry".to_string(),
            version: Some("1.0.0".to_string()),
            dry_run: true,
            allow_dirty: false,
        };
        let result = handle_advanced_command(command);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_advanced_command_score() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "let x = 42").unwrap();

        let command = Commands::Score {
            path: temp_file.path().to_path_buf(),
            depth: "standard".to_string(),
            fast: false,
            deep: false,
            watch: false,
            explain: false,
            baseline: None,
            min: Some(0.8),
            config: None,
            format: "text".to_string(),
            verbose: false,
            output: None,
        };
        let result = handle_advanced_command(command);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_advanced_command_wasm() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "let x = 42").unwrap();

        let command = Commands::Wasm {
            file: temp_file.path().to_path_buf(),
            output: None,
            target: "wasm32".to_string(),
            wit: false,
            deploy: false,
            deploy_target: None,
            portability: false,
            opt_level: "O2".to_string(),
            debug: false,
            simd: false,
            threads: false,
            component_model: true,
            name: None,
            version: "0.1.0".to_string(),
            verbose: false,
        };
        let result = handle_advanced_command(command);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_command_dispatch_repl() {
        let result =
            handle_command_dispatch(Some(Commands::Repl { record: None }), false, VmMode::Ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_command_dispatch_none() {
        let result = handle_command_dispatch(None, false, VmMode::Ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_command_dispatch_parse() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "let x = 42").unwrap();

        let result = handle_command_dispatch(
            Some(Commands::Parse {
                file: temp_file.path().to_path_buf(),
            }),
            false,
            VmMode::Ast,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_command_dispatch_transpile() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "let x = 42").unwrap();

        let result = handle_command_dispatch(
            Some(Commands::Transpile {
                file: temp_file.path().to_path_buf(),
                output: None,
                minimal: false,
            }),
            true,
            VmMode::Ast,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_command_dispatch_run() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "let x = 42").unwrap();

        let result = handle_command_dispatch(
            Some(Commands::Run {
                file: temp_file.path().to_path_buf(),
            }),
            false,
            VmMode::Ast,
        );
        assert!(result.is_ok());
    }
}
