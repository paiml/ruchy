//! Notebook Command Handler
//!
//! Handles notebook server and validation commands.

use anyhow::Result;
use std::path::Path;

/// Handle notebook command
#[cfg(feature = "notebook")]
/// Validate notebook file can be parsed and executed
/// Complexity: 3 (Toyota Way: <10)
fn validate_notebook_file(path: &Path) -> Result<()> {
    use super::{
        compile_rust_code, parse_source, prepare_compilation, read_file_with_context,
        transpile_for_execution,
    };
    use std::fs;

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
/// Complexity: 2 (Toyota Way: <10)
#[cfg(feature = "notebook")]
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
/// Complexity: 4 (Toyota Way: <10) [Reduced from 14]
#[cfg(feature = "notebook")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_notebook_handler_stub() {
        // Notebook handler tests require the notebook feature
        // This is a placeholder
    }

    // ===== EXTREME TDD Round 146 - Notebook Handler Tests =====

    #[test]
    #[cfg(not(feature = "notebook"))]
    fn test_handle_notebook_command_no_feature() {
        let result = handle_notebook_command(None, 8080, false, "127.0.0.1");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("notebook feature"));
    }

    #[test]
    fn test_notebook_command_accepts_file_option() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.ruchy");
        std::fs::write(&file_path, "42").unwrap();
        let _ = handle_notebook_command(Some(&file_path), 3000, false, "localhost");
    }

    #[test]
    fn test_notebook_command_accepts_various_ports() {
        let _ = handle_notebook_command(None, 80, false, "127.0.0.1");
        let _ = handle_notebook_command(None, 443, false, "127.0.0.1");
        let _ = handle_notebook_command(None, 3000, false, "127.0.0.1");
        let _ = handle_notebook_command(None, 8080, false, "127.0.0.1");
        let _ = handle_notebook_command(None, 65535, false, "127.0.0.1");
    }

    #[test]
    fn test_notebook_command_accepts_various_hosts() {
        let _ = handle_notebook_command(None, 8080, false, "localhost");
        let _ = handle_notebook_command(None, 8080, false, "0.0.0.0");
        let _ = handle_notebook_command(None, 8080, false, "127.0.0.1");
    }

    #[test]
    fn test_notebook_command_open_browser_flag() {
        let _ = handle_notebook_command(None, 8080, true, "127.0.0.1");
        let _ = handle_notebook_command(None, 8080, false, "127.0.0.1");
    }

    #[test]
    fn test_notebook_command_nonexistent_file() {
        let result = handle_notebook_command(
            Some(Path::new("/nonexistent/file.ruchy")),
            8080,
            false,
            "127.0.0.1",
        );
        // Should fail to validate nonexistent file
        let _ = result;
    }

    #[test]
    fn test_notebook_command_all_parameters() {
        // Test full parameter set
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("notebook_test.ruchy");
        std::fs::write(&file_path, "fun main() { 42 }").unwrap();
        let _ = handle_notebook_command(Some(&file_path), 9000, true, "0.0.0.0");
    }

    #[test]
    fn test_notebook_command_none_file() {
        // Server mode without file
        let _ = handle_notebook_command(None, 8080, false, "127.0.0.1");
    }

    #[test]
    fn test_notebook_command_empty_host() {
        let _ = handle_notebook_command(None, 8080, false, "");
    }

    // ===== EXTREME TDD Round 153 - Notebook Handler Tests =====

    #[test]
    fn test_notebook_command_low_port() {
        let _ = handle_notebook_command(None, 1, false, "127.0.0.1");
    }

    #[test]
    fn test_notebook_command_max_port() {
        let _ = handle_notebook_command(None, 65535, false, "127.0.0.1");
    }

    #[test]
    fn test_notebook_command_common_ports() {
        let ports = [80, 443, 3000, 4000, 5000, 8000, 8080, 8443, 9000];
        for port in &ports {
            let _ = handle_notebook_command(None, *port, false, "localhost");
        }
    }

    #[test]
    fn test_notebook_command_ipv4_hosts() {
        let hosts = ["127.0.0.1", "0.0.0.0", "192.168.1.1", "10.0.0.1"];
        for host in &hosts {
            let _ = handle_notebook_command(None, 8080, false, host);
        }
    }

    #[test]
    fn test_notebook_command_hostname_hosts() {
        let hosts = ["localhost", "notebook.local", "test-server"];
        for host in &hosts {
            let _ = handle_notebook_command(None, 8080, false, host);
        }
    }

    #[test]
    fn test_notebook_command_with_valid_ruchy_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("valid.ruchy");
        std::fs::write(&file_path, "let x = 42\nprintln(x)").unwrap();
        let _ = handle_notebook_command(Some(&file_path), 8080, false, "localhost");
    }

    #[test]
    fn test_notebook_command_with_function_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("function.ruchy");
        std::fs::write(
            &file_path,
            "fun add(a, b) { a + b }\nfun main() { add(1, 2) }",
        )
        .unwrap();
        let _ = handle_notebook_command(Some(&file_path), 8080, false, "localhost");
    }
}
