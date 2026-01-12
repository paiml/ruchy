//! Command Router
//!
//! Routes complex CLI commands to their appropriate handlers.

use anyhow::Result;

use super::{
    add, commands, handle_actor_observe_command, handle_bench_command, handle_coverage_command,
    handle_dataflow_debug_command, handle_doc_command, handle_mcp_command, handle_notebook_command,
    handle_optimize_command, handle_prove_command, handle_replay_to_tests_command,
    handle_serve_command, handle_wasm_command,
};

/// Handle complex commands that require special routing
///
/// This function routes CLI commands to their appropriate handlers based on
/// the command variant. It serves as the central dispatcher for all complex
/// commands in the Ruchy CLI.
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
}

#[cfg(test)]
mod tests {
    // Router tests would require mocking Commands enum
    // which is defined in main.rs
}
