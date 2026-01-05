use anyhow::{Context, Result};
pub mod add;
pub mod bench_handler;
pub mod build;
pub mod check_handler;
mod commands;
pub mod coverage_handler;
pub mod doc_handler;
pub mod eval;
pub mod execution_handler;
mod handlers_modules;
pub mod new;
pub mod parse_handler;
pub mod repl_handler;
pub mod run_handler;
pub mod transpile_handler;
pub mod wasm_handler;

// Testing handlers
pub mod fuzz_handler;
pub mod mutations_handler;
pub mod property_tests_handler;

// Other extracted handlers
pub mod actor_handler;
pub mod compile_handler;
pub mod dataflow_handler;
pub mod optimize_handler;
pub mod oracle_handler;
pub mod publish_handler;

// Re-export from extracted modules
pub use bench_handler::handle_bench_command;
pub use check_handler::handle_check_command;
pub use coverage_handler::handle_coverage_command;
pub use doc_handler::handle_doc_command;
pub use eval::handle_eval_command;
pub use execution_handler::{handle_file_execution, handle_stdin_input};
pub use parse_handler::handle_parse_command;
pub use repl_handler::handle_repl_command;
pub use run_handler::{
    compile_rust_code, handle_run_command, prepare_compilation, transpile_for_execution, VmMode,
};
pub use transpile_handler::handle_transpile_command;
pub use wasm_handler::{compile_ruchy_to_wasm, handle_wasm_command};

// Testing re-exports
pub use fuzz_handler::handle_fuzz_command;
pub use mutations_handler::handle_mutations_command;
pub use property_tests_handler::handle_property_tests_command;

// Other re-exports
pub use actor_handler::handle_actor_observe_command;
pub use compile_handler::handle_compile_command;
pub use dataflow_handler::handle_dataflow_debug_command;
pub use optimize_handler::handle_optimize_command;
pub use oracle_handler::handle_oracle_command;
pub use publish_handler::handle_publish_command;

// Import for internal use
use transpile_handler::parse_source;
use ruchy::runtime::replay_converter::ConversionConfig;
use ruchy::runtime::Repl;
// RuchyParser, Transpiler, WasmEmitter moved to individual handler modules
// Replay functionality imports removed - not needed in handler, used directly in REPL
// PARSER-077: Add syn and prettyplease for proper TokenStream formatting
use std::fs;
use std::path::{Path, PathBuf};

// handle_eval_command moved to eval.rs
// handle_file_execution, handle_stdin_input moved to execution_handler.rs
// handle_parse_command moved to parse_handler.rs
// handle_transpile_command moved to transpile_handler.rs

// ============================================================================
// Common Helper Functions (Complexity ≤5, reused across handlers)
// ============================================================================

/// Check if a result should be printed (filters out Unit values)
/// Complexity: 2
fn should_print_result(result: &str) -> bool {
    result != "Unit" && result != "()"
}

/// Read file contents with detailed error context
/// Complexity: 2
fn read_file_with_context(file: &Path) -> Result<String> {
    fs::read_to_string(file).map_err(|e| {
        // Include the OS error message (e.g., "No such file or directory")
        anyhow::anyhow!("{}: {}", file.display(), e)
    })
}

/// Create a REPL instance with temp directory
/// Complexity: 1
fn create_repl() -> Result<Repl> {
    Repl::new(std::env::temp_dir())
}

/// Log command output if verbose mode is enabled
/// Complexity: 2
fn log_command_output(output: &std::process::Output, verbose: bool) {
    if verbose {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Command output:\n{}", stderr);
    }
}

/// Write file with detailed error context
/// Complexity: 2
fn write_file_with_context(path: &Path, content: &[u8]) -> Result<()> {
    fs::write(path, content).with_context(|| format!("Failed to write file: {}", path.display()))
}

// ============================================================================

// handle_run_command and VmMode moved to run_handler.rs
/// Handle interactive theorem prover (RUCHY-0820) - delegated to refactored module
pub fn handle_prove_command(
    file: Option<&std::path::Path>,
    backend: &str,
    ml_suggestions: bool,
    timeout: u64,
    script: Option<&std::path::Path>,
    export: Option<&std::path::Path>,
    check: bool,
    counterexample: bool,
    verbose: bool,
    format: &str,
) -> anyhow::Result<()> {
    // Delegate to refactored module with ≤10 complexity
    handlers_modules::prove::handle_prove_command(
        file,
        backend,
        ml_suggestions,
        timeout,
        script,
        export,
        check,
        counterexample,
        verbose,
        format,
    )
}

// print_prover_help moved to repl_handler.rs
// handle_repl_command moved to repl_handler.rs
// handle_compile_command moved to compile_handler.rs

// handle_check_command moved to check_handler.rs
/// Handle test command - run tests with various options (delegated to refactored module)
///
/// # Arguments
/// * `path` - Optional path to test directory
/// * `watch` - Enable watch mode
/// * `verbose` - Enable verbose output
/// * `filter` - Optional test filter
/// * `coverage` - Enable coverage reporting
/// * `coverage_format` - Coverage report format
/// * `parallel` - Number of parallel test threads
/// * `threshold` - Coverage threshold
/// * `format` - Output format
///
/// # Examples
/// ```
/// // This function is typically called by the CLI test command
/// // handle_test_command(None, false, false, None, false, &"text".to_string(), 1, 80.0, &"text".to_string());
/// ```
///
/// # Errors
/// Returns error if tests fail to run or coverage threshold is not met
pub fn handle_test_command(
    path: Option<PathBuf>,
    watch: bool,
    verbose: bool,
    filter: Option<&str>,
    coverage: bool,
    coverage_format: &str,
    parallel: usize,
    threshold: f64,
    format: &str,
) -> Result<()> {
    // Delegate to refactored module with ≤10 complexity
    handlers_modules::test::handle_test_command(
        path,
        watch,
        verbose,
        filter,
        coverage,
        coverage_format,
        parallel,
        threshold,
        format,
    )
}

// handle_coverage_command moved to coverage_handler.rs
// handle_bench_command moved to bench_handler.rs
// handle_doc_command moved to doc_handler.rs
// handle_dataflow_debug_command moved to dataflow_handler.rs
// handle_actor_observe_command moved to actor_handler.rs
// handle_optimize_command moved to optimize_handler.rs

/// Watch and run tests on changes - delegated to refactored module
fn handle_watch_and_test(path: &Path, verbose: bool, filter: Option<&str>) -> Result<()> {
    handlers_modules::test::handle_test_command(
        Some(path.to_path_buf()),
        true, // watch mode
        verbose,
        filter,
        false, // coverage
        "text",
        1,
        0.0,
        "text",
    )
}
/// Run enhanced tests - delegated to refactored module
#[allow(clippy::unnecessary_wraps)]
fn handle_run_enhanced_tests(
    path: &Path,
    verbose: bool,
    filter: Option<&str>,
    coverage: bool,
    coverage_format: &str,
    parallel: usize,
    threshold: f64,
    format: &str,
) -> Result<()> {
    handlers_modules::test::handle_test_command(
        Some(path.to_path_buf()),
        false, // not watch mode
        verbose,
        filter,
        coverage,
        coverage_format,
        parallel,
        threshold,
        format,
    )
}
/// Run a single .ruchy test file - delegated to `test_helpers` module
fn run_ruchy_test_file(test_file: &Path, verbose: bool) -> Result<()> {
    handlers_modules::test_helpers::run_test_file(test_file, verbose)
}
/// Verify proofs extracted from AST - delegated to `prove_helpers` module
fn verify_proofs_from_ast(
    ast: &ruchy::frontend::ast::Expr,
    file_path: &std::path::Path,
    format: &str,
    counterexample: bool,
    verbose: bool,
) -> Result<()> {
    handlers_modules::prove_helpers::verify_proofs_from_ast(
        ast,
        file_path,
        format,
        counterexample,
        verbose,
    )
}
///
/// # Arguments
/// * `command` - The command to execute
///
/// # Examples
/// ```
/// // This function is typically called by the main dispatcher for complex commands
/// ```
///
/// # Errors
/// Returns error if command execution fails
#[allow(clippy::unnecessary_wraps)]
pub fn handle_complex_command(command: crate::Commands) -> Result<()> {
    match command {
        crate::Commands::Ast {
            file,
            json,
            graph,
            metrics,
            symbols,
            deps,
            verbose,
            output,
        } => commands::handle_ast_command(
            &file,
            json,
            graph,
            metrics,
            symbols,
            deps,
            verbose,
            output.as_deref(),
        ),
        crate::Commands::Provability {
            file,
            verify,
            contracts,
            invariants,
            termination,
            bounds,
            verbose,
            output,
        } => commands::handle_provability_command(
            &file,
            verify,
            contracts,
            invariants,
            termination,
            bounds,
            verbose,
            output.as_deref(),
        ),
        crate::Commands::Runtime {
            file,
            profile,
            binary,
            iterations,
            bigo,
            bench,
            compare,
            memory,
            verbose,
            output,
        } => commands::handle_runtime_command(
            &file,
            profile,
            binary,
            iterations,
            bigo,
            bench,
            compare.as_deref(),
            memory,
            verbose,
            output.as_deref(),
        ),
        crate::Commands::Score {
            path,
            depth,
            fast,
            deep,
            watch,
            explain,
            baseline,
            min,
            config,
            format,
            verbose,
            output,
        } => commands::handle_score_command(
            &path,
            &depth,
            fast,
            deep,
            watch,
            explain,
            baseline.as_deref(),
            min,
            config.as_deref(),
            &format,
            verbose,
            output.as_deref(),
        ),
        crate::Commands::QualityGate {
            path,
            config,
            depth: _,
            fail_fast,
            format,
            export,
            ci: _,
            verbose,
        } => {
            // Simplified quality gate handling
            commands::handle_quality_gate_command(
                &path,
                config.as_deref(),
                fail_fast, // Use as strict
                !verbose,  // Use as quiet
                format == "json",
                verbose,
                None, // No output field
                export.as_deref(),
            )
        }
        crate::Commands::Fmt {
            file,
            all,
            check,
            stdout,
            diff,
            config,
            line_width: _,
            indent: _,
            use_tabs: _,
        } => {
            // Simplified fmt handling
            commands::handle_fmt_command(
                &file,
                check,
                !check && !stdout, // write if not check or stdout
                config.as_deref(),
                all,
                diff,
                stdout,
                false, // verbose not available
            )
        }
        crate::Commands::Lint {
            file,
            all: _,
            fix,
            strict,
            verbose,
            format,
            rules,
            deny_warnings: _,
            max_complexity: _,
            config,
            init_config,
        } => {
            if init_config {
                // Create default lint config
                println!("Creating default lint configuration...");
                Ok(())
            } else if let Some(file_path) = file {
                commands::handle_lint_command(
                    &file_path,
                    fix,
                    strict,
                    rules.as_deref(),
                    format == "json",
                    verbose,
                    None, // ignore not available
                    config.as_deref(),
                )
            } else {
                Err(anyhow::anyhow!(
                    "Error: Either provide a file or use --all flag"
                ))
            }
        }
        crate::Commands::Prove {
            file,
            backend,
            ml_suggestions,
            timeout,
            script,
            export,
            check,
            counterexample,
            verbose,
            format,
        } => handle_prove_command(
            file.as_deref(),
            &backend,
            ml_suggestions,
            timeout,
            script.as_deref(),
            export.as_deref(),
            check,
            counterexample,
            verbose,
            &format,
        ),
        crate::Commands::Coverage {
            path,
            threshold,
            format,
            verbose,
        } => handle_coverage_command(&path, threshold.unwrap_or(80.0), &format, verbose),
        crate::Commands::Notebook {
            file,
            port,
            open,
            host,
        } => handle_notebook_command(file.as_deref(), port, open, &host),
        crate::Commands::Serve {
            directory,
            port,
            host,
            verbose,
            watch,
            debounce,
            pid_file,
            watch_wasm,
        } => handle_serve_command(
            &directory,
            port,
            &host,
            verbose,
            watch,
            debounce,
            pid_file.as_deref(),
            watch_wasm,
        ),
        crate::Commands::ReplayToTests {
            input,
            output,
            property_tests,
            benchmarks,
            timeout,
        } => handle_replay_to_tests_command(
            &input,
            output.as_deref(),
            property_tests,
            benchmarks,
            timeout,
        ),
        crate::Commands::Wasm {
            file,
            output,
            target,
            wit,
            deploy,
            deploy_target,
            portability,
            opt_level,
            debug,
            simd,
            threads,
            component_model,
            name,
            version,
            verbose,
        } => handle_wasm_command(
            &file,
            output.as_deref(),
            &target,
            wit,
            deploy,
            deploy_target.as_deref(),
            portability,
            &opt_level,
            debug,
            simd,
            threads,
            component_model,
            name.as_deref(),
            &version,
            verbose,
        ),
        crate::Commands::Mcp {
            name,
            streaming,
            timeout,
            min_score,
            max_complexity,
            verbose,
            config,
        } => handle_mcp_command(
            &name,
            streaming,
            timeout,
            min_score,
            max_complexity,
            verbose,
            config.as_deref(),
        ),
        crate::Commands::Add {
            package,
            version,
            dev,
            registry: _registry,
        } => {
            // Use our new add::handle_add_command (CARGO-003)
            // Note: registry parameter ignored for now - using cargo's default (crates.io)
            add::handle_add_command(&package, version.as_deref(), dev, false)
        }
        crate::Commands::Bench {
            file,
            iterations,
            warmup,
            format,
            output,
            verbose,
        } => handle_bench_command(
            &file,
            iterations,
            warmup,
            &format,
            output.as_deref(),
            verbose,
        ),
        crate::Commands::Doc {
            path,
            output,
            format,
            private,
            open,
            all,
            verbose,
        } => handle_doc_command(&path, &output, &format, private, open, all, verbose),
        crate::Commands::Optimize {
            file,
            hardware,
            depth,
            cache,
            branches,
            vectorization,
            abstractions,
            benchmark,
            format,
            output,
            verbose,
            threshold,
        } => handle_optimize_command(
            &file,
            &hardware,
            &depth,
            cache,
            branches,
            vectorization,
            abstractions,
            benchmark,
            &format,
            output.as_deref(),
            verbose,
            threshold,
        ),
        crate::Commands::ActorObserve {
            config,
            refresh_interval,
            max_traces,
            max_actors,
            enable_deadlock_detection,
            deadlock_interval,
            start_mode,
            no_color,
            format,
            export,
            duration,
            verbose,
            filter_actor,
            filter_failed,
            filter_slow,
        } => handle_actor_observe_command(
            config.as_deref(),
            refresh_interval,
            max_traces,
            max_actors,
            enable_deadlock_detection,
            deadlock_interval,
            &start_mode,
            !no_color,
            &format,
            export.as_deref(),
            duration,
            verbose,
            filter_actor.as_deref(),
            filter_failed,
            filter_slow,
        ),
        crate::Commands::DataflowDebug {
            config,
            max_rows,
            auto_materialize,
            enable_profiling,
            timeout,
            track_memory,
            compute_diffs,
            sample_rate,
            refresh_interval,
            no_color,
            format,
            export,
            verbose,
            breakpoint,
            start_mode,
        } => handle_dataflow_debug_command(
            config.as_deref(),
            max_rows,
            auto_materialize,
            enable_profiling,
            timeout,
            track_memory,
            compute_diffs,
            sample_rate,
            refresh_interval,
            !no_color,
            &format,
            export.as_deref(),
            verbose,
            &breakpoint,
            &start_mode,
        ),
        _ => {
            // Other commands not yet implemented
            eprintln!("Command not yet implemented");
            Ok(())
        }
    }
    /*
    // Original complex command handling - commented out until handlers implemented
    match command {
        crate::Commands::Ast {
            file,
            json,
            graph,
            metrics,
            symbols,
            deps,
            verbose,
            output
        } => {
            // AST analysis implementation planned
            eprintln!("AST analysis not yet implemented");
            Ok(())
        }
        crate::Commands::Provability {
            file,
            verify,
            contracts,
            invariants,
            termination,
            bounds,
            verbose,
            output
        } => {
            // Provability analysis implementation planned
            eprintln!("Provability analysis not yet implemented");
            Ok(())
        }
        crate::Commands::Runtime {
            file,
            profile,
            bigo,
            bench,
            compare,
            memory,
            verbose,
            output
        } => {
            // Runtime analysis implementation planned
            eprintln!("Runtime analysis not yet implemented");
            Ok(())
        }
        crate::Commands::Score {
            path,
            depth,
            fast,
            deep,
            watch,
            explain,
            baseline,
            min,
            config,
            format,
            verbose,
            output,
        } => {
            let baseline_str = baseline.as_deref();
            let config_str = config.as_ref().and_then(|p| p.to_str());
            let output_str = output.as_ref().and_then(|p| p.to_str());
            // Quality score calculation implementation planned
            eprintln!("Quality score calculation not yet implemented");
            Ok(())
        }
        crate::Commands::QualityGate {
            path,
            config,
            depth,
            fail_fast,
            format,
            export,
            ci,
            verbose,
        } => {
            // Quality gates implementation planned
            eprintln!("Quality gates enforcement not yet implemented");
            Ok(())
        }
        crate::Commands::Fmt {
            file: _,
            all: _,
            check: _,
            stdout: _,
            diff: _,
            config: _,
            line_width: _,
            indent: _,
            use_tabs: _,
        } => {
            // Code formatting implementation planned
            eprintln!("Code formatting not yet implemented");
            Ok(())
        }
        crate::Commands::Doc {
            path,
            output,
            format,
            private,
            open,
            all,
            verbose,
        } => handle_doc_command(&path, &output, &format, private, open, all, verbose),
        crate::Commands::Bench {
            file,
            iterations,
            warmup,
            format,
            output,
            verbose,
        } => {
            crate::benchmark_ruchy_code(
                &file,
                iterations,
                warmup,
                &format,
                output.as_deref(),
                verbose,
            )
        }
        crate::Commands::Lint {
            file,
            all: _,
            fix,
            strict,
            verbose,
            format,
            rules,
            deny_warnings: _,
            max_complexity: _,
            config,
            init_config,
        } => {
            if init_config {
                crate::generate_default_lint_config()
            } else {
                // Load custom rules if config provided
                let custom_rules = if let Some(config_path) = config {
                    crate::load_custom_lint_rules(&config_path)?
                } else {
                    // Custom lint rules implementation planned
                    Default::default()
                };
                if all {
                    crate::lint_ruchy_code(
                        &PathBuf::from("."),
                        all,
                        fix,
                        strict,
                        verbose,
                        &format,
                        rules.as_deref(),
                        deny_warnings,
                        max_complexity,
                        &custom_rules,
                    )
                } else if let Some(file) = file {
                    crate::lint_ruchy_code(
                        &file,
                        false,
                        fix,
                        strict,
                        verbose,
                        &format,
                        rules.as_deref(),
                        deny_warnings,
                        max_complexity,
                        &custom_rules,
                    )
                } else {
                    return Err(anyhow::anyhow!("Error: Either provide a file or use --all flag"));
                }
            }
        }
        crate::Commands::Publish {
            registry,
            version,
            dry_run,
            allow_dirty,
        } => {
            crate::publish_package(&registry, version.as_deref(), dry_run, allow_dirty)
        }
        crate::Commands::Mcp {
            name,
            streaming,
            timeout,
            min_score,
            max_complexity,
            verbose,
            config,
        } => {
            let config_str = config.as_ref().and_then(|p| p.to_str());
            crate::start_mcp_server(&name, streaming, timeout, min_score, max_complexity, verbose, config_str)
        }
        crate::Commands::Optimize {
            file,
            hardware,
            depth,
            cache,
            branches,
            vectorization,
            abstractions,
            benchmark,
            format,
            output,
            verbose,
            threshold,
        } => {
            crate::optimize_file(
                &file,
                &hardware,
                &depth,
                cache,
                branches,
                vectorization,
                abstractions,
                benchmark,
                &format,
                output.as_deref(),
                verbose,
                threshold,
            )
        }
        crate::Commands::ActorObserve {
            config,
            refresh_interval,
            max_traces,
            max_actors,
            enable_deadlock_detection,
            deadlock_interval,
            start_mode,
            no_color,
            format,
            export,
            duration,
            verbose,
            filter_actor,
            filter_failed,
            filter_slow,
        } => {
            crate::start_actor_observatory(
                config.as_ref(),
                refresh_interval,
                max_traces,
                max_actors,
                enable_deadlock_detection,
                deadlock_interval,
                &start_mode,
                !no_color,
                &format,
                export.as_ref(),
                duration,
                verbose,
                filter_actor.as_ref(),
                filter_failed,
                filter_slow,
            )
        }
        crate::Commands::DataflowDebug {
            config,
            max_rows,
            auto_materialize,
            enable_profiling,
            timeout,
            track_memory,
            compute_diffs,
            sample_rate,
            refresh_interval,
            no_color,
            format,
            export,
            verbose,
            breakpoint,
            start_mode,
        } => {
            crate::start_dataflow_debugger(
                config.as_ref(),
                max_rows,
                auto_materialize,
                enable_profiling,
                timeout,
                track_memory,
                compute_diffs,
                sample_rate,
                refresh_interval,
                !no_color,
                &format,
                export.as_ref(),
                verbose,
                &breakpoint,
                &start_mode,
            )
        }
        crate::Commands::Wasm {
            file,
            output,
            target,
            wit,
            deploy,
            deploy_target,
            portability,
            opt_level,
            debug,
            simd,
            threads,
            component_model,
            name,
            version,
            verbose,
        } => {
            crate::handle_wasm_command(
                &file,
                output.as_deref(),
                &target,
                wit,
                deploy,
                deploy_target.as_deref(),
                portability,
                &opt_level,
                debug,
                simd,
                threads,
                component_model,
                name.as_deref(),
                &version,
                verbose,
            )
        }
        _ => {
            // This should not be reached since handled commands are processed elsewhere
            eprintln!("Error: Command not implemented in complex handler");
            std::process::exit(1);
        }
    }
    */
}

/// Handle MCP server command
///
/// Starts a Model Context Protocol server that exposes Ruchy's code analysis,
/// scoring, linting, formatting, and transpilation capabilities as MCP tools.
///
/// # Arguments
/// * `name` - Server name for MCP identification
/// * `streaming` - Enable streaming updates
/// * `timeout` - Session timeout in seconds
/// * `min_score` - Minimum quality score threshold
/// * `max_complexity` - Maximum complexity threshold
/// * `verbose` - Enable verbose logging
/// * `config` - Optional configuration file path
///
/// # Examples
/// ```no_run
/// // This function is typically called by the CLI
/// // handle_mcp_command("ruchy-mcp", false, 3600, 0.8, 10, false, None);
/// ```
///
/// # Errors
/// Returns error if MCP server cannot be started or configured
#[cfg(feature = "mcp")]
pub fn handle_mcp_command(
    name: &str,
    _streaming: bool,
    _timeout: u64,
    _min_score: f64,
    _max_complexity: u32,
    verbose: bool,
    _config: Option<&Path>,
) -> Result<()> {
    use ruchy::mcp::{create_ruchy_mcp_server, create_ruchy_tools, StdioTransport};

    if verbose {
        eprintln!("🚀 Starting Ruchy MCP Server: {}", name);
    }

    // Create the MCP server with tools
    let server = create_ruchy_mcp_server().context("Failed to create MCP server")?;

    // Register all Ruchy tools
    let tools = create_ruchy_tools();
    if verbose {
        eprintln!("   Registered {} tools:", tools.len());
        for (tool_name, tool) in &tools {
            eprintln!("   - {}: {}", tool_name, tool.description());
        }
    }

    if verbose {
        eprintln!("   Transport: stdio");
        eprintln!("   Awaiting MCP client connection...");
    }

    // Create async runtime for the server
    let runtime = tokio::runtime::Runtime::new().context("Failed to create async runtime")?;

    runtime.block_on(async {
        let transport = StdioTransport::new();

        if verbose {
            eprintln!("✅ MCP server running");
        }

        // Run the server with stdio transport
        server.run(transport).await.context("MCP server error")
    })
}

#[cfg(not(feature = "mcp"))]
pub fn handle_mcp_command(
    _name: &str,
    _streaming: bool,
    _timeout: u64,
    _min_score: f64,
    _max_complexity: u32,
    _verbose: bool,
    _config: Option<&Path>,
) -> Result<()> {
    eprintln!("Error: MCP support not enabled");
    eprintln!("Rebuild with: cargo build --features mcp");
    std::process::exit(1);
}

/// Handle notebook command
#[cfg(feature = "notebook")]
/// Validate notebook file can be parsed and executed
/// Complexity: 3 (Toyota Way: <10 ✓)
fn validate_notebook_file(path: &Path) -> Result<()> {
    println!("📓 Notebook validation mode for: {}", path.display());

    // Validate the file can be parsed and executed
    let source = read_file_with_context(path)?;
    let ast = parse_source(&source)?;
    let rust_code = transpile_for_execution(&ast, path)?;
    let (temp_source, binary_path) = prepare_compilation(&rust_code, false)?;
    compile_rust_code(temp_source.path(), &binary_path)?;

    // Execute the file to validate it runs
    let result = std::process::Command::new(&binary_path).output()?;

    // Cleanup
    let _ = fs::remove_file(&binary_path);

    if result.status.success() {
        println!("✅ Notebook validation: PASSED");
        println!("   File can be loaded and executed in notebook environment");
        Ok(())
    } else {
        anyhow::bail!(
            "Notebook validation: FAILED\n{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

/// Open browser for notebook interface
/// Complexity: 2 (Toyota Way: <10 ✓)
fn open_browser_for_notebook(url: &str) -> Result<()> {
    use std::process::Command;

    println!("   Opening browser at {}", url);
    #[cfg(target_os = "macos")]
    Command::new("open").arg(url).spawn()?;
    #[cfg(target_os = "linux")]
    Command::new("xdg-open").arg(url).spawn()?;
    #[cfg(target_os = "windows")]
    Command::new("cmd").args(["/C", "start", url]).spawn()?;
    Ok(())
}

/// Handle notebook command - start server or validate file
/// Complexity: 4 (Toyota Way: <10 ✓) [Reduced from 14]
pub fn handle_notebook_command(
    file: Option<&Path>,
    port: u16,
    open_browser: bool,
    host: &str,
) -> Result<()> {
    // TOOL-VALIDATION-003: Non-interactive file validation mode
    if let Some(path) = file {
        return validate_notebook_file(path);
    }

    // Interactive server mode (original behavior)
    println!("🚀 Starting Ruchy Notebook server...");
    println!("   Host: {}:{}", host, port);

    // Create async runtime for the server
    let runtime = tokio::runtime::Runtime::new()?;

    // Open browser if requested
    if open_browser {
        let url = format!("http://{}:{}", host, port);
        open_browser_for_notebook(&url)?;
    }

    // Start the notebook server
    println!(
        "🔧 DEBUG: About to call ruchy::notebook::start_server({})",
        port
    );
    let result = runtime.block_on(async { ruchy::notebook::start_server(port).await });
    println!("🔧 DEBUG: Server returned: {:?}", result);
    result.map_err(|e| anyhow::anyhow!("Notebook server error: {}", e))
}
#[cfg(not(feature = "notebook"))]
pub fn handle_notebook_command(
    _file: Option<&Path>,
    _port: u16,
    _open_browser: bool,
    _host: &str,
) -> Result<()> {
    Err(anyhow::anyhow!(
        "Notebook feature not enabled. Rebuild with --features notebook"
    ))
}

// ============================================================================
// HTTP Static File Server (HTTP-001)
// ============================================================================

/// Handle serve command - serve static files over HTTP
///
/// # Arguments
/// * `directory` - Directory to serve
/// * `port` - Port to bind to
/// * `host` - Host address to bind to
/// * `verbose` - Enable verbose logging
#[cfg(feature = "notebook")]
pub fn handle_serve_command(
    directory: &Path,
    port: u16,
    host: &str,
    verbose: bool,
    watch: bool,
    debounce: u64,
    pid_file: Option<&Path>,
    watch_wasm: bool,
) -> Result<()> {
    use axum::{http::HeaderValue, Router};
    use tower::ServiceBuilder;
    use tower_http::{services::ServeDir, set_header::SetResponseHeaderLayer};

    // Verify directory exists
    if !directory.exists() {
        return Err(anyhow::anyhow!(
            "Directory not found: {}",
            directory.display()
        ));
    }
    if !directory.is_dir() {
        return Err(anyhow::anyhow!(
            "Path is not a directory: {}",
            directory.display()
        ));
    }

    // Initialize PID file if requested
    let _pid_guard = if let Some(pid_path) = pid_file {
        Some(ruchy::server::PidFile::create(pid_path)?)
    } else {
        None
    };

    // World-class UX: Colored startup banner (vite-style)
    #[cfg(not(target_arch = "wasm32"))]
    {
        use colored::Colorize;

        println!(
            "\n  🚀 {} {}\n",
            "Ruchy Dev Server".bright_cyan().bold(),
            format!("v{}", env!("CARGO_PKG_VERSION")).dimmed()
        );

        println!(
            "  {}  http://{}:{}",
            "➜  Local:".green(),
            host,
            port.to_string().bold()
        );

        // Show network IP if available
        if let Ok(ip) = local_ip_address::local_ip() {
            println!("  {}  http://{}:{}", "➜  Network:".green(), ip, port);
        }

        println!(
            "  📁 {}: {}",
            "Serving".dimmed(),
            directory.display().to_string().bold()
        );

        if watch {
            println!(
                "  👀 {}: {}/**/*",
                "Watching".dimmed(),
                directory.display().to_string().bold()
            );
            if watch_wasm {
                println!(
                    "  🦀 {}: Hot reload enabled for .ruchy files",
                    "WASM".dimmed()
                );
            }
        }

        println!("\n  {} Press Ctrl+C to stop\n", "Ready".green().bold());
    }

    #[cfg(target_arch = "wasm32")]
    {
        println!("🚀 Ruchy HTTP Server v{}", env!("CARGO_PKG_VERSION"));
        println!("📁 Serving: {}", directory.display());
        println!("🌐 Listening: http://{}:{}", host, port);
        if watch {
            println!("👀 Watching: {}/**/*", directory.display());
        }
        println!("Press Ctrl+C to stop\n");
    }

    // Build the Axum app with static file serving + WASM headers
    let serve_dir = ServeDir::new(directory)
        .precompressed_gzip() // Serve .gz files if available (faster)
        .precompressed_br(); // Serve .br files if available (faster)

    // Add WASM-specific headers for SharedArrayBuffer support (HTTP-003)
    // Required for: WebAssembly threading, SharedArrayBuffer, Atomics
    // Reference: https://web.dev/coop-coep/
    let app = Router::new().fallback_service(serve_dir).layer(
        ServiceBuilder::new()
            // Cross-Origin-Opener-Policy: Isolate browsing context
            .layer(SetResponseHeaderLayer::if_not_present(
                axum::http::header::HeaderName::from_static("cross-origin-opener-policy"),
                HeaderValue::from_static("same-origin"),
            ))
            // Cross-Origin-Embedder-Policy: Require CORP for cross-origin resources
            .layer(SetResponseHeaderLayer::if_not_present(
                axum::http::header::HeaderName::from_static("cross-origin-embedder-policy"),
                HeaderValue::from_static("require-corp"),
            )),
    );

    // PERFORMANCE: Create optimized tokio runtime (multi-threaded, CPU-bound)
    let num_cpus = num_cpus::get();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num_cpus)
        .enable_all()
        .build()?;

    // Setup signal handling for graceful shutdown (Ctrl+C)
    #[cfg(unix)]
    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel::<()>();

    #[cfg(unix)]
    {
        use signal_hook::consts::{SIGINT, SIGTERM};
        use signal_hook::iterator::Signals;

        let shutdown_tx_clone = shutdown_tx;
        std::thread::spawn(move || {
            let mut signals =
                Signals::new([SIGINT, SIGTERM]).expect("Failed to register signal handlers");
            if let Some(_sig) = signals.forever().next() {
                let _ = shutdown_tx_clone.send(());
            }
        });
    }

    #[allow(unreachable_code)] // Watch mode and Unix signal handling both return early
    if watch {
        // Watch mode: Monitor file changes and restart server
        loop {
            let mut watcher =
                ruchy::server::watcher::FileWatcher::new(vec![directory.to_path_buf()], debounce)?;

            let addr = format!("{}:{}", host, port);
            let app_clone = app.clone();
            let server_handle = runtime.spawn(async move {
                let listener = tokio::net::TcpListener::bind(&addr).await?;

                if verbose {
                    println!("✅ Server started ({} workers)", num_cpus);
                }

                axum::serve(listener, app_clone).await
            });

            // Poll for file changes AND shutdown signal
            loop {
                // Check for shutdown signal
                #[cfg(unix)]
                if shutdown_rx.try_recv().is_ok() {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        use colored::Colorize;
                        println!("\n  {} Shutting down gracefully...\n", "✓".green());
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        println!("\n  ✓ Shutting down gracefully...\n");
                    }
                    server_handle.abort();
                    return Ok(());
                }

                if let Some(changed_files) = watcher.check_changes() {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        use colored::Colorize;

                        // WASM hot reload: compile .ruchy files to .wasm
                        if watch_wasm {
                            for file in &changed_files {
                                if file.extension().and_then(|s| s.to_str()) == Some("ruchy") {
                                    println!(
                                        "  🦀 {}: {}",
                                        "Compiling".cyan().bold(),
                                        file.display()
                                    );

                                    match compile_ruchy_to_wasm(file, verbose) {
                                        Ok(wasm_path) => {
                                            println!(
                                                "  ✅ {}: {}",
                                                "Compiled".green(),
                                                wasm_path.display()
                                            );
                                        }
                                        Err(e) => {
                                            println!("  ❌ {}: {}", "Failed".red(), e);
                                        }
                                    }
                                }
                            }
                        }

                        if verbose {
                            for file in &changed_files {
                                println!("  📝 {}: {}", "Changed".yellow(), file.display());
                            }
                        }

                        // Gracefully shutdown server
                        server_handle.abort();

                        println!("\n  {} Restarting server...\n", "↻".cyan());
                    }

                    #[cfg(target_arch = "wasm32")]
                    {
                        if verbose {
                            for file in &changed_files {
                                println!("  📝 Changed: {}", file.display());
                            }
                        }
                        server_handle.abort();
                        println!("\n  ↻ Restarting server...\n");
                    }

                    break;
                }

                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    } else {
        // Normal mode: Run server once with graceful shutdown
        let addr = format!("{}:{}", host, port);

        #[cfg(unix)]
        {
            let addr_clone = addr;
            let verbose_clone = verbose;
            let num_cpus_clone = num_cpus;
            let server_future = async move {
                let listener = tokio::net::TcpListener::bind(&addr_clone).await?;

                if verbose_clone {
                    println!("✅ Server started ({} workers)", num_cpus_clone);
                }

                axum::serve(listener, app).await
            };

            // Spawn server task
            let server_handle = runtime.spawn(server_future);

            // Wait for shutdown signal
            loop {
                if shutdown_rx.try_recv().is_ok() {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        use colored::Colorize;
                        println!("\n  {} Shutting down gracefully...\n", "✓".green());
                    }
                    server_handle.abort();
                    return Ok(());
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }

        #[cfg(not(unix))]
        runtime.block_on(async {
            let listener = tokio::net::TcpListener::bind(&addr).await?;

            if verbose {
                println!("✅ Server started ({} workers)", num_cpus);
            }

            axum::serve(listener, app).await
        })?;
    }

    #[allow(unreachable_code)]
    Ok(())
}

#[cfg(not(feature = "notebook"))]
pub fn handle_serve_command(
    _directory: &Path,
    _port: u16,
    _host: &str,
    _verbose: bool,
    _watch: bool,
    _debounce: u64,
    _pid_file: Option<&Path>,
    _watch_wasm: bool,
) -> Result<()> {
    Err(anyhow::anyhow!(
        "HTTP server requires notebook feature. Rebuild with --features notebook"
    ))
}

/// Handle replay-to-tests command - convert .replay files to regression tests
///
/// # Arguments
/// * `input` - Input replay file or directory containing .replay files
/// * `output` - Optional output test file path
/// * `property_tests` - Whether to include property tests
/// * `benchmarks` - Whether to include benchmarks
/// * `timeout` - Test timeout in milliseconds
///
/// # Examples
/// ```
/// // Convert single replay file
/// handle_replay_to_tests_command(Path::new("demo.replay"), None, true, false, 5000);
///
/// // Convert directory of replay files
/// handle_replay_to_tests_command(Path::new("demos/"), Some(Path::new("tests/replays.rs")), true, true, 10000);
/// ```
///
/// # Errors
/// Returns error if replay files can't be read or test files can't be written
/// Setup conversion configuration for replay-to-test conversion (complexity: 4)
fn setup_conversion_config(
    property_tests: bool,
    benchmarks: bool,
    timeout: u64,
) -> ConversionConfig {
    ConversionConfig {
        test_module_prefix: "replay_generated".to_string(),
        include_property_tests: property_tests,
        include_benchmarks: benchmarks,
        timeout_ms: timeout,
    }
}
/// Determine output path, using default if none provided (complexity: 3)
fn determine_output_path(output: Option<&Path>) -> &Path {
    let default_output = Path::new("tests/generated_from_replays.rs");
    output.unwrap_or(default_output)
}
/// Validate that file has .replay extension (complexity: 3)
fn validate_replay_file(path: &Path) -> Result<()> {
    if path.extension().and_then(|s| s.to_str()) == Some("replay") {
        Ok(())
    } else {
        eprintln!("❌ Input file must have .replay extension");
        Err(anyhow::anyhow!("Invalid file extension"))
    }
}
/// Process a single .replay file (complexity: 8)
fn process_single_file(
    input: &Path,
    converter: &ruchy::runtime::replay_converter::ReplayConverter,
    all_tests: &mut Vec<ruchy::runtime::replay_converter::GeneratedTest>,
    processed_files: &mut usize,
) -> Result<()> {
    validate_replay_file(input)?;
    println!("📄 Processing replay file: {}", input.display());
    match converter.convert_file(input) {
        Ok(tests) => {
            println!("  ✅ Generated {} tests", tests.len());
            all_tests.extend(tests);
            *processed_files += 1;
            Ok(())
        }
        Err(e) => {
            eprintln!("  ❌ Failed to process {}: {}", input.display(), e);
            Err(e)
        }
    }
}
/// Process directory containing .replay files (complexity: 10)
/// Process directory of replay files (complexity: 4 - reduced from 11)
fn process_directory(
    input: &Path,
    converter: &ruchy::runtime::replay_converter::ReplayConverter,
    all_tests: &mut Vec<ruchy::runtime::replay_converter::GeneratedTest>,
    processed_files: &mut usize,
) -> Result<()> {
    println!("📁 Processing replay directory: {}", input.display());
    let replay_files = find_replay_files(input)?;
    if replay_files.is_empty() {
        println!("⚠️  No .replay files found in directory");
        return Ok(());
    }
    println!("🔍 Found {} replay files", replay_files.len());
    process_replay_files(&replay_files, converter, all_tests, processed_files);
    Ok(())
}

/// Find all .replay files in directory (complexity: 3)
fn find_replay_files(dir: &Path) -> Result<Vec<PathBuf>> {
    use std::fs;
    Ok(fs::read_dir(dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() && path.extension()? == "replay" {
                Some(path)
            } else {
                None
            }
        })
        .collect())
}

/// Process all replay files in sequence (complexity: 4)
fn process_replay_files(
    replay_files: &[PathBuf],
    converter: &ruchy::runtime::replay_converter::ReplayConverter,
    all_tests: &mut Vec<ruchy::runtime::replay_converter::GeneratedTest>,
    processed_files: &mut usize,
) {
    for replay_file in replay_files {
        println!("📄 Processing: {}", replay_file.display());
        match converter.convert_file(replay_file) {
            Ok(tests) => {
                println!("  ✅ Generated {} tests", tests.len());
                all_tests.extend(tests);
                *processed_files += 1;
            }
            Err(e) => {
                eprintln!("  ⚠️  Failed to process {}: {}", replay_file.display(), e);
                // Continue with other files instead of failing completely
            }
        }
    }
}
/// Write test output to file, creating directories if needed (complexity: 4)
fn write_test_output(
    converter: &ruchy::runtime::replay_converter::ReplayConverter,
    all_tests: &[ruchy::runtime::replay_converter::GeneratedTest],
    output_path: &Path,
) -> Result<()> {
    use anyhow::Context;
    use std::fs;
    // Create output directory if needed
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    println!("📝 Writing tests to: {}", output_path.display());
    converter
        .write_tests(all_tests, output_path)
        .context("Failed to write test file")?;
    Ok(())
}
/// Generate comprehensive summary report of conversion results (complexity: 8)
fn generate_summary_report(
    all_tests: &[ruchy::runtime::replay_converter::GeneratedTest],
    processed_files: usize,
) {
    use colored::Colorize;
    use std::collections::{HashMap, HashSet};
    println!("\n{}", "🎉 Conversion Summary".bright_green().bold());
    println!("=====================================");
    println!("📊 Files processed: {}", processed_files);
    println!("✅ Tests generated: {}", all_tests.len());
    // Breakdown by test category
    let mut category_counts = HashMap::new();
    let mut coverage_areas = HashSet::new();
    for test in all_tests {
        *category_counts.entry(&test.category).or_insert(0) += 1;
        coverage_areas.extend(test.coverage_areas.iter().cloned());
    }
    println!("\n📋 Test Breakdown:");
    for (category, count) in category_counts {
        println!("   {:?}: {}", category, count);
    }
    println!("\n🎯 Coverage Areas: {} unique areas", coverage_areas.len());
    if !coverage_areas.is_empty() {
        let mut areas: Vec<_> = coverage_areas.into_iter().collect();
        areas.sort();
        for area in areas.iter().take(10) {
            // Show first 10
            println!("   • {}", area);
        }
        if areas.len() > 10 {
            println!("   ... and {} more", areas.len() - 10);
        }
    }
    println!("\n💡 Next Steps:");
    println!("   1. Run tests: cargo test");
    println!("   2. Measure coverage: cargo test -- --test-threads=1");
    println!("   3. Validate replay determinism");
    println!(
        "\n🚀 {}",
        "Replay-to-test conversion complete!".bright_green()
    );
}
/// Process input path (file or directory) with replay files (complexity: 5)
fn process_input_path(
    input: &Path,
    converter: &ruchy::runtime::replay_converter::ReplayConverter,
    all_tests: &mut Vec<ruchy::runtime::replay_converter::GeneratedTest>,
    processed_files: &mut usize,
) -> Result<()> {
    if input.is_file() {
        process_single_file(input, converter, all_tests, processed_files)
    } else if input.is_dir() {
        process_directory(input, converter, all_tests, processed_files)
    } else {
        eprintln!("❌ Input path must be a file or directory");
        Err(anyhow::anyhow!("Invalid input path"))
    }
}
/// Convert REPL replay files to regression tests (complexity: 7)
pub fn handle_replay_to_tests_command(
    input: &Path,
    output: Option<&Path>,
    property_tests: bool,
    benchmarks: bool,
    timeout: u64,
) -> Result<()> {
    use colored::Colorize;
    use ruchy::runtime::replay_converter::ReplayConverter;
    println!(
        "{}",
        "🔄 Converting REPL replay files to regression tests"
            .bright_cyan()
            .bold()
    );
    println!("Input: {}", input.display());
    let config = setup_conversion_config(property_tests, benchmarks, timeout);
    let converter = ReplayConverter::with_config(config);
    let mut all_tests = Vec::new();
    let mut processed_files = 0;
    let output_path = determine_output_path(output);
    process_input_path(input, &converter, &mut all_tests, &mut processed_files)?;
    if all_tests.is_empty() {
        println!("⚠️  No tests generated");
        return Ok(());
    }
    write_test_output(&converter, &all_tests, output_path)?;
    generate_summary_report(&all_tests, processed_files);
    Ok(())
}

// handle_wasm_command moved to wasm_handler.rs

// handle_property_tests_command moved to property_tests_handler.rs
// handle_mutations_command moved to mutations_handler.rs
// handle_fuzz_command moved to fuzz_handler.rs
// handle_oracle_command moved to oracle_handler.rs
// handle_publish_command moved to publish_handler.rs


#[cfg(test)]
mod tests;
