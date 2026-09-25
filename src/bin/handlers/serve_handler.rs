//! HTTP Static File Server Handler (HTTP-001)
//!
//! Handles serving static files over HTTP with optional watch mode.

use anyhow::Result;
use std::path::Path;

/// Reject a serve root that is missing or is not a directory.
///
/// Runs in every build so `ruchy serve ./missing` reports the path problem
/// whether or not the HTTP server itself was compiled in.
fn validate_serve_directory(directory: &Path) -> Result<()> {
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
    Ok(())
}

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
    let request = ServeRequest {
        directory,
        port,
        host,
        verbose,
        watch,
        debounce,
        pid_file,
        watch_wasm,
    };
    serve_until(&request, shutdown_on_signal)
}

/// The arguments of one `ruchy serve` invocation.
#[cfg(feature = "notebook")]
struct ServeRequest<'a> {
    directory: &'a Path,
    port: u16,
    host: &'a str,
    verbose: bool,
    watch: bool,
    debounce: u64,
    pid_file: Option<&'a Path>,
    watch_wasm: bool,
}

/// Serve `request` until the channel returned by `install_shutdown` receives.
///
/// `install_shutdown` runs once the server is set up. The CLI passes
/// [`shutdown_on_signal`] (Ctrl+C / SIGTERM); tests pass a channel they
/// control, so they drive the whole serve path without serving forever.
/// Complexity: 4 (Toyota Way: <10)
#[cfg(feature = "notebook")]
fn serve_until(
    request: &ServeRequest<'_>,
    install_shutdown: impl FnOnce() -> std::sync::mpsc::Receiver<()>,
) -> Result<()> {
    validate_serve_directory(request.directory)?;

    // Initialize PID file if requested
    let _pid_guard = request
        .pid_file
        .map(ruchy::server::PidFile::create)
        .transpose()?;

    // World-class UX: Colored startup banner (vite-style)
    print_startup_banner(
        request.host,
        request.port,
        request.directory,
        request.watch,
        request.watch_wasm,
    );
    let app = build_static_app(request.directory);

    // PERFORMANCE: Create optimized tokio runtime (multi-threaded, CPU-bound)
    let num_cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num_cpus)
        .enable_all()
        .build()?;

    let shutdown_rx = install_shutdown();
    if request.watch {
        run_watch_mode(
            &runtime,
            &app,
            request.directory,
            request.host,
            request.port,
            request.verbose,
            request.debounce,
            request.watch_wasm,
            num_cpus,
            &shutdown_rx,
        )
    } else {
        run_normal_mode(
            &runtime,
            app,
            request.host,
            request.port,
            request.verbose,
            num_cpus,
            &shutdown_rx,
        )
    }
}

/// Static file service for `directory` with the WASM isolation headers.
/// Complexity: 1 (Toyota Way: <10)
#[cfg(feature = "notebook")]
fn build_static_app(directory: &Path) -> axum::Router {
    use axum::{http::HeaderValue, Router};
    use tower::ServiceBuilder;
    use tower_http::{services::ServeDir, set_header::SetResponseHeaderLayer};

    // Build the Axum app with static file serving + WASM headers
    let serve_dir = ServeDir::new(directory)
        .precompressed_gzip() // Serve .gz files if available (faster)
        .precompressed_br(); // Serve .br files if available (faster)

    // Add WASM-specific headers for SharedArrayBuffer support (HTTP-003)
    // Required for: WebAssembly threading, SharedArrayBuffer, Atomics
    // Reference: https://web.dev/coop-coep/
    Router::new().fallback_service(serve_dir).layer(
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
    )
}

/// A channel that receives once the process gets SIGINT or SIGTERM (Ctrl+C).
///
/// On non-Unix targets no handler is installed: the channel never receives
/// and Ctrl+C ends the process through the default handler.
/// Complexity: 2 (Toyota Way: <10)
#[cfg(feature = "notebook")]
fn shutdown_on_signal() -> std::sync::mpsc::Receiver<()> {
    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel::<()>();

    #[cfg(unix)]
    {
        use signal_hook::consts::{SIGINT, SIGTERM};
        use signal_hook::iterator::Signals;

        std::thread::spawn(move || {
            let mut signals =
                Signals::new([SIGINT, SIGTERM]).expect("Failed to register signal handlers");
            if let Some(_sig) = signals.forever().next() {
                let _ = shutdown_tx.send(());
            }
        });
    }
    #[cfg(not(unix))]
    drop(shutdown_tx);

    shutdown_rx
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

/// Run server in watch mode with file change detection.
///
/// A server that ends on its own (e.g. the address cannot be bound) returns
/// its error, as in normal mode.
/// Complexity: 3 (Toyota Way: <10)
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
    shutdown_rx: &std::sync::mpsc::Receiver<()>,
) -> Result<()> {
    // One watcher for the whole session: re-creating it on every restart spent
    // an inotify instance each time, and a failure to get one killed the server.
    let Some(mut watcher) = open_watcher(directory, debounce) else {
        return run_normal_mode(
            runtime,
            app.clone(),
            host,
            port,
            verbose,
            num_cpus,
            shutdown_rx,
        );
    };
    loop {
        let server_handle = spawn_server(runtime, app.clone(), host, port, verbose, num_cpus);
        if let Some(outcome) = serve_until_change(
            runtime,
            server_handle,
            &mut watcher,
            watch_wasm,
            verbose,
            shutdown_rx,
        ) {
            return outcome;
        }
    }
}

/// Serve until shutdown is requested or the server task ends (`Some`, the
/// outcome) or a watched file changes (`None`, the server has stopped and its
/// address is free to bind again).
/// Complexity: 3 (Toyota Way: <10)
#[cfg(feature = "notebook")]
fn serve_until_change(
    runtime: &tokio::runtime::Runtime,
    mut server_handle: tokio::task::JoinHandle<std::io::Result<()>>,
    watcher: &mut ruchy::server::watcher::FileWatcher,
    watch_wasm: bool,
    verbose: bool,
    shutdown_rx: &std::sync::mpsc::Receiver<()>,
) -> Option<Result<()>> {
    while !stopped_on_change(runtime, &mut server_handle, watcher, watch_wasm, verbose) {
        if let Some(outcome) = poll_server(runtime, &mut server_handle, shutdown_rx) {
            return Some(outcome);
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    None
}

/// When a watched file changed: handle it, stop the server and wait for the
/// aborted task so its listener is closed before the restart rebinds (`true`).
/// Complexity: 2 (Toyota Way: <10)
#[cfg(feature = "notebook")]
fn stopped_on_change(
    runtime: &tokio::runtime::Runtime,
    server_handle: &mut tokio::task::JoinHandle<std::io::Result<()>>,
    watcher: &mut ruchy::server::watcher::FileWatcher,
    watch_wasm: bool,
    verbose: bool,
) -> bool {
    let Some(changed_files) = watcher.check_changes() else {
        return false;
    };
    handle_file_changes(&changed_files, watch_wasm, verbose);
    server_handle.abort();
    let _ = runtime.block_on(server_handle);
    print_restart_message();
    true
}

/// Open the file watcher for `directory`, or `None` (with a warning on stderr)
/// when the host cannot watch it.
///
/// `--watch` used to exit the server when the watcher could not be created. On a
/// busy host the per-user inotify instance limit (`fs.inotify.max_user_instances`,
/// 128 by default) is exhausted and creation fails with EMFILE ("Too many open
/// files"); the server then keeps serving without reload instead (G2B-S2).
#[cfg(feature = "notebook")]
fn open_watcher(directory: &Path, debounce: u64) -> Option<ruchy::server::watcher::FileWatcher> {
    match ruchy::server::watcher::FileWatcher::new(vec![directory.to_path_buf()], debounce) {
        Ok(watcher) => Some(watcher),
        Err(err) => {
            eprintln!("{}", watch_unavailable_message(directory, &err));
            None
        }
    }
}

/// Warning printed when `--watch` cannot watch `directory`.
#[cfg(feature = "notebook")]
fn watch_unavailable_message(directory: &Path, err: &dyn std::fmt::Display) -> String {
    format!(
        "  warning: --watch disabled, cannot watch {}: {err}; serving without reload \
         (if this is \"Too many open files\", raise fs.inotify.max_user_instances)",
        directory.display()
    )
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
                    println!("  🦀 {}: {}", "Compiling".cyan().bold(), file.display());

                    match compile_ruchy_to_wasm(file, verbose) {
                        Ok(wasm_path) => {
                            println!("  ✅ {}: {}", "Compiled".green(), wasm_path.display());
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

/// Run server in normal mode (no watch) until shutdown is requested.
///
/// A server that ends on its own (e.g. the address cannot be bound) returns
/// its error instead of leaving the process waiting on a dead server.
/// Complexity: 4 (Toyota Way: <10)
#[cfg(feature = "notebook")]
fn run_normal_mode(
    runtime: &tokio::runtime::Runtime,
    app: axum::Router,
    host: &str,
    port: u16,
    verbose: bool,
    num_cpus: usize,
    shutdown_rx: &std::sync::mpsc::Receiver<()>,
) -> Result<()> {
    let server_handle = spawn_server(runtime, app, host, port, verbose, num_cpus);
    wait_for_shutdown_or_exit(runtime, server_handle, shutdown_rx)
}

/// Spawn the server task: bind `host:port`, then serve `app` on it.
/// Complexity: 2 (Toyota Way: <10)
#[cfg(feature = "notebook")]
fn spawn_server(
    runtime: &tokio::runtime::Runtime,
    app: axum::Router,
    host: &str,
    port: u16,
    verbose: bool,
    num_cpus: usize,
) -> tokio::task::JoinHandle<std::io::Result<()>> {
    let addr = format!("{}:{}", host, port);
    runtime.spawn(async move {
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        if verbose {
            println!("✅ Server started ({} workers)", num_cpus);
        }

        axum::serve(listener, app).await
    })
}

/// Block until shutdown is requested (abort the server, `Ok`) or the server
/// task ends on its own (return its result).
/// Complexity: 2 (Toyota Way: <10)
#[cfg(feature = "notebook")]
fn wait_for_shutdown_or_exit(
    runtime: &tokio::runtime::Runtime,
    mut server_handle: tokio::task::JoinHandle<std::io::Result<()>>,
    shutdown_rx: &std::sync::mpsc::Receiver<()>,
) -> Result<()> {
    loop {
        if let Some(outcome) = poll_server(runtime, &mut server_handle, shutdown_rx) {
            return outcome;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

/// One poll: `Some` once shutdown was requested or the server task ended.
/// Complexity: 3 (Toyota Way: <10)
#[cfg(feature = "notebook")]
fn poll_server(
    runtime: &tokio::runtime::Runtime,
    server_handle: &mut tokio::task::JoinHandle<std::io::Result<()>>,
    shutdown_rx: &std::sync::mpsc::Receiver<()>,
) -> Option<Result<()>> {
    if shutdown_rx.try_recv().is_ok() {
        print_shutdown_message();
        server_handle.abort();
        return Some(Ok(()));
    }
    if !server_handle.is_finished() {
        return None;
    }
    let joined = runtime.block_on(server_handle);
    Some(
        joined
            .map_err(anyhow::Error::from)
            .and_then(|served| Ok(served?)),
    )
}

#[cfg(not(feature = "notebook"))]
pub fn handle_serve_command(
    directory: &Path,
    _port: u16,
    _host: &str,
    _verbose: bool,
    _watch: bool,
    _debounce: u64,
    _pid_file: Option<&Path>,
    _watch_wasm: bool,
) -> Result<()> {
    validate_serve_directory(directory)?;
    Err(anyhow::anyhow!(
        "HTTP server requires notebook feature. Rebuild with --features notebook"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// G2B-S2: a watcher that cannot be created must not end the server.
    #[test]
    #[cfg(feature = "notebook")]
    fn test_open_watcher_on_unwatchable_path_degrades_to_none() {
        let dir = tempfile::tempdir().expect("tempdir");
        let missing = dir.path().join("absent");
        assert!(open_watcher(&missing, 100).is_none());
    }

    #[test]
    #[cfg(feature = "notebook")]
    fn test_watch_unavailable_message_names_the_cause_and_remedy() {
        let msg = watch_unavailable_message(Path::new("site"), &"Too many open files");
        assert!(msg.contains("--watch disabled"), "{msg}");
        assert!(msg.contains("site"), "{msg}");
        assert!(msg.contains("Too many open files"), "{msg}");
        assert!(msg.contains("max_user_instances"), "{msg}");
    }

    /// A channel that has already received the shutdown request.
    #[cfg(feature = "notebook")]
    fn stopped() -> std::sync::mpsc::Receiver<()> {
        let (tx, rx) = std::sync::mpsc::channel();
        tx.send(()).expect("send shutdown");
        rx
    }

    /// Drive the serve path for these arguments without serving forever.
    ///
    /// With the notebook feature the server is set up (PID file, banner, app,
    /// runtime, listener task, watcher) and stopped through the shutdown seam;
    /// without it the stub is called directly.
    fn serve_then_stop(
        directory: &Path,
        port: u16,
        host: &str,
        verbose: bool,
        watch: bool,
        debounce: u64,
        pid_file: Option<&Path>,
        watch_wasm: bool,
    ) -> Result<()> {
        #[cfg(feature = "notebook")]
        {
            let request = ServeRequest {
                directory,
                port,
                host,
                verbose,
                watch,
                debounce,
                pid_file,
                watch_wasm,
            };
            serve_until(&request, stopped)
        }
        #[cfg(not(feature = "notebook"))]
        handle_serve_command(
            directory, port, host, verbose, watch, debounce, pid_file, watch_wasm,
        )
    }

    /// A valid serve request shuts down cleanly with the notebook feature and
    /// names the missing feature without it.
    fn assert_served(result: Result<()>) {
        #[cfg(feature = "notebook")]
        assert!(result.is_ok(), "{result:?}");
        #[cfg(not(feature = "notebook"))]
        assert!(result
            .expect_err("stub must refuse")
            .to_string()
            .contains("notebook feature"),);
    }

    /// A free local port (bound, then released).
    #[cfg(feature = "notebook")]
    fn free_port() -> u16 {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind port 0");
        listener.local_addr().expect("local addr").port()
    }

    /// GET `path` from 127.0.0.1:`port`, retrying until the server listens.
    #[cfg(feature = "notebook")]
    fn http_get(port: u16, path: &str) -> String {
        use std::io::{Read, Write};
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        loop {
            if let Ok(mut stream) = std::net::TcpStream::connect(("127.0.0.1", port)) {
                let request =
                    format!("GET {path} HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n");
                stream.write_all(request.as_bytes()).expect("write request");
                let mut response = String::new();
                stream.read_to_string(&mut response).expect("read response");
                return response;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "server never listened"
            );
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    /// RHLGA-1: the server really serves the directory, then stops when the
    /// shutdown channel receives, and the call returns.
    #[test]
    #[cfg(feature = "notebook")]
    fn test_serve_until_serves_files_then_stops_on_shutdown() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("hello.txt"), "hello from serve").unwrap();
        let port = free_port();
        let (tx, rx) = std::sync::mpsc::channel();
        let client = std::thread::spawn(move || {
            let response = http_get(port, "/hello.txt");
            tx.send(()).expect("send shutdown");
            response
        });
        let request = ServeRequest {
            directory: temp_dir.path(),
            port,
            host: "127.0.0.1",
            verbose: false,
            watch: false,
            debounce: 100,
            pid_file: None,
            watch_wasm: false,
        };
        serve_until(&request, move || rx).expect("serve until shutdown");
        let response = client.join().expect("client thread");
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
        assert!(response.contains("hello from serve"), "{response}");
        assert!(
            response.contains("cross-origin-opener-policy: same-origin"),
            "{response}"
        );
    }

    /// RHLGA-1: an address that cannot be bound is reported, not waited on.
    #[test]
    #[cfg(feature = "notebook")]
    fn test_serve_until_reports_a_port_in_use() {
        let temp_dir = TempDir::new().unwrap();
        let taken = std::net::TcpListener::bind("127.0.0.1:0").expect("bind port 0");
        let port = taken.local_addr().expect("local addr").port();
        let (_tx, rx) = std::sync::mpsc::channel::<()>();
        let request = ServeRequest {
            directory: temp_dir.path(),
            port,
            host: "127.0.0.1",
            verbose: false,
            watch: false,
            debounce: 100,
            pid_file: None,
            watch_wasm: false,
        };
        let err = serve_until(&request, move || rx).expect_err("port is taken");
        assert!(err.to_string().to_lowercase().contains("in use"), "{err}");
    }

    /// RHLGA-1: watch mode also stops when the shutdown channel receives.
    #[test]
    #[cfg(feature = "notebook")]
    fn test_serve_until_watch_mode_stops_on_shutdown() {
        let temp_dir = TempDir::new().unwrap();
        let request = ServeRequest {
            directory: temp_dir.path(),
            port: free_port(),
            host: "127.0.0.1",
            verbose: true,
            watch: true,
            debounce: 50,
            pid_file: None,
            watch_wasm: false,
        };
        serve_until(&request, stopped).expect("watch mode stops");
    }

    /// RHLGA-1: watch mode on an address it cannot bind returns the bind error
    /// promptly instead of watching files with no server behind them.
    #[test]
    #[cfg(feature = "notebook")]
    fn test_serve_until_watch_mode_reports_a_port_in_use() {
        let temp_dir = TempDir::new().unwrap();
        let taken = std::net::TcpListener::bind("127.0.0.1:0").expect("bind port 0");
        let port = taken.local_addr().expect("local addr").port();
        let dir = temp_dir.path().to_path_buf();
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let (_tx, rx) = std::sync::mpsc::channel::<()>();
            let request = ServeRequest {
                directory: &dir,
                port,
                host: "127.0.0.1",
                verbose: false,
                watch: true,
                debounce: 50,
                pid_file: None,
                watch_wasm: false,
            };
            let outcome = serve_until(&request, move || rx).map_err(|e| e.to_string());
            let _ = done_tx.send(outcome);
        });
        let outcome = done_rx
            .recv_timeout(std::time::Duration::from_secs(20))
            .expect("watch mode must return when the server cannot bind");
        let err = outcome.expect_err("port is taken");
        assert!(err.to_lowercase().contains("in use"), "{err}");
        drop(taken);
    }

    // ===== EXTREME TDD Round 145 - Serve Handler Tests =====

    #[test]
    fn test_handle_serve_command_nonexistent_dir() {
        let result = handle_serve_command(
            Path::new("/nonexistent/dir"),
            8080,
            "127.0.0.1",
            false,
            false,
            500,
            None,
            false,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_handle_serve_command_file_not_dir() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "test").unwrap();
        let result = handle_serve_command(
            &file_path,
            8080,
            "127.0.0.1",
            false,
            false,
            500,
            None,
            false,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a directory"));
    }

    #[test]
    #[cfg(not(feature = "notebook"))]
    fn test_handle_serve_command_no_notebook_feature() {
        let result = handle_serve_command(
            Path::new("."),
            8080,
            "127.0.0.1",
            false,
            false,
            500,
            None,
            false,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("notebook feature"));
    }

    #[test]
    fn test_serve_command_requires_valid_path() {
        // Test that both feature and non-feature paths work
        let _ = handle_serve_command(
            Path::new("/invalid"),
            8080,
            "localhost",
            false,
            false,
            100,
            None,
            false,
        );
    }

    #[test]
    fn test_serve_command_default_parameters() {
        // Verify function signature accepts expected parameters
        let temp_dir = TempDir::new().unwrap();
        assert_served(serve_then_stop(
            temp_dir.path(),
            3000,
            "0.0.0.0",
            true,
            true,
            1000,
            Some(&temp_dir.path().join("server.pid")),
            true,
        ));
    }

    #[test]
    fn test_serve_command_with_verbose() {
        let temp_dir = TempDir::new().unwrap();
        assert_served(serve_then_stop(
            temp_dir.path(),
            8081,
            "127.0.0.1",
            true, // verbose
            false,
            500,
            None,
            false,
        ));
    }

    #[test]
    fn test_serve_command_various_hosts() {
        let temp_dir = TempDir::new().unwrap();
        // Test localhost
        assert_served(serve_then_stop(
            temp_dir.path(),
            8082,
            "localhost",
            false,
            false,
            100,
            None,
            false,
        ));
        // Test 0.0.0.0
        assert_served(serve_then_stop(
            temp_dir.path(),
            8083,
            "0.0.0.0",
            false,
            false,
            100,
            None,
            false,
        ));
    }

    #[test]
    fn test_serve_command_various_ports() {
        let temp_dir = TempDir::new().unwrap();
        assert_served(serve_then_stop(
            temp_dir.path(),
            80,
            "127.0.0.1",
            false,
            false,
            100,
            None,
            false,
        ));
        assert_served(serve_then_stop(
            temp_dir.path(),
            443,
            "127.0.0.1",
            false,
            false,
            100,
            None,
            false,
        ));
        assert_served(serve_then_stop(
            temp_dir.path(),
            65535,
            "127.0.0.1",
            false,
            false,
            100,
            None,
            false,
        ));
    }

    #[test]
    fn test_serve_command_debounce_values() {
        let temp_dir = TempDir::new().unwrap();
        // Minimum debounce
        assert_served(serve_then_stop(
            temp_dir.path(),
            8084,
            "127.0.0.1",
            false,
            false,
            1, // 1ms
            None,
            false,
        ));
        // Large debounce
        assert_served(serve_then_stop(
            temp_dir.path(),
            8085,
            "127.0.0.1",
            false,
            false,
            10000, // 10s
            None,
            false,
        ));
    }

    #[test]
    fn test_serve_command_with_wasm_watch() {
        let temp_dir = TempDir::new().unwrap();
        assert_served(serve_then_stop(
            temp_dir.path(),
            8086,
            "127.0.0.1",
            false,
            true, // watch
            500,
            None,
            true, // watch_wasm
        ));
    }

    // ===== EXTREME TDD Round 153 - Serve Handler Tests =====

    #[test]
    fn test_serve_command_all_flags() {
        let temp_dir = TempDir::new().unwrap();
        let pid_path = temp_dir.path().join("server.pid");
        assert_served(serve_then_stop(
            temp_dir.path(),
            8087,
            "127.0.0.1",
            true, // verbose
            true, // watch
            250,  // debounce
            Some(&pid_path),
            true, // watch_wasm
        ));
    }

    #[test]
    fn test_serve_command_zero_debounce() {
        let temp_dir = TempDir::new().unwrap();
        assert_served(serve_then_stop(
            temp_dir.path(),
            8088,
            "127.0.0.1",
            false,
            false,
            0, // zero debounce
            None,
            false,
        ));
    }

    #[test]
    fn test_serve_command_nested_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested = temp_dir.path().join("a").join("b").join("c");
        std::fs::create_dir_all(&nested).unwrap();
        assert_served(serve_then_stop(
            &nested,
            8089,
            "127.0.0.1",
            false,
            false,
            100,
            None,
            false,
        ));
    }
}
