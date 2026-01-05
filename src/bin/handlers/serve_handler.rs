//! HTTP Static File Server Handler (HTTP-001)
//!
//! Handles serving static files over HTTP with optional watch mode.

use anyhow::Result;
use std::path::Path;

/// Handle serve command - serve static files over HTTP
///
/// # Arguments
/// * `directory` - Directory to serve
/// * `port` - Port to bind to
/// * `host` - Host address to bind to
/// * `verbose` - Enable verbose logging
/// * `watch` - Enable watch mode for file changes
/// * `debounce` - Debounce interval for file changes in ms
/// * `pid_file` - Optional PID file path
/// * `watch_wasm` - Enable WASM hot reload
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
    print_startup_banner(host, port, directory, watch, watch_wasm);

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
        run_watch_mode(
            &runtime,
            &app,
            directory,
            host,
            port,
            verbose,
            debounce,
            watch_wasm,
            num_cpus,
            #[cfg(unix)]
            &shutdown_rx,
        )
    } else {
        run_normal_mode(
            &runtime,
            app,
            host,
            port,
            verbose,
            num_cpus,
            #[cfg(unix)]
            &shutdown_rx,
        )
    }
}

/// Print colored startup banner (vite-style)
/// Complexity: 3 (Toyota Way: <10)
#[cfg(feature = "notebook")]
fn print_startup_banner(host: &str, port: u16, directory: &Path, watch: bool, watch_wasm: bool) {
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
}

/// Run server in watch mode with file change detection
/// Complexity: 8 (Toyota Way: <10)
#[cfg(feature = "notebook")]
#[allow(clippy::too_many_arguments)]
fn run_watch_mode(
    runtime: &tokio::runtime::Runtime,
    app: &axum::Router,
    directory: &Path,
    host: &str,
    port: u16,
    verbose: bool,
    debounce: u64,
    watch_wasm: bool,
    num_cpus: usize,
    #[cfg(unix)] shutdown_rx: &std::sync::mpsc::Receiver<()>,
) -> Result<()> {
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
                print_shutdown_message();
                server_handle.abort();
                return Ok(());
            }

            if let Some(changed_files) = watcher.check_changes() {
                handle_file_changes(&changed_files, watch_wasm, verbose);
                server_handle.abort();
                print_restart_message();
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

/// Handle file changes in watch mode
/// Complexity: 5 (Toyota Way: <10)
#[cfg(feature = "notebook")]
fn handle_file_changes(changed_files: &[std::path::PathBuf], watch_wasm: bool, verbose: bool) {
    use super::compile_ruchy_to_wasm;

    #[cfg(not(target_arch = "wasm32"))]
    {
        use colored::Colorize;

        // WASM hot reload: compile .ruchy files to .wasm
        if watch_wasm {
            for file in changed_files {
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
            for file in changed_files {
                println!("  📝 {}: {}", "Changed".yellow(), file.display());
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        if verbose {
            for file in changed_files {
                println!("  📝 Changed: {}", file.display());
            }
        }
    }
}

/// Print shutdown message
/// Complexity: 1 (Toyota Way: <10)
#[cfg(feature = "notebook")]
fn print_shutdown_message() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use colored::Colorize;
        println!("\n  {} Shutting down gracefully...\n", "✓".green());
    }
    #[cfg(target_arch = "wasm32")]
    {
        println!("\n  ✓ Shutting down gracefully...\n");
    }
}

/// Print restart message
/// Complexity: 1 (Toyota Way: <10)
#[cfg(feature = "notebook")]
fn print_restart_message() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use colored::Colorize;
        println!("\n  {} Restarting server...\n", "↻".cyan());
    }
    #[cfg(target_arch = "wasm32")]
    {
        println!("\n  ↻ Restarting server...\n");
    }
}

/// Run server in normal mode (no watch)
/// Complexity: 6 (Toyota Way: <10)
#[cfg(feature = "notebook")]
fn run_normal_mode(
    runtime: &tokio::runtime::Runtime,
    app: axum::Router,
    host: &str,
    port: u16,
    verbose: bool,
    num_cpus: usize,
    #[cfg(unix)] shutdown_rx: &std::sync::mpsc::Receiver<()>,
) -> Result<()> {
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
                print_shutdown_message();
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_serve_handler_stub() {
        // Serve handler tests require the notebook feature
        // This is a placeholder
    }
}
