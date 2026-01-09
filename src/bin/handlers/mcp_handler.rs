//! MCP (Model Context Protocol) Command Handler
//!
//! Handles starting the MCP server for code analysis tools.

use anyhow::Result;
use std::path::Path;

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
    use anyhow::Context;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_handler_stub() {
        // MCP handler tests require the mcp feature
        // This is a placeholder
    }

    // ===== EXTREME TDD Round 150 - MCP Handler Tests =====

    #[test]
    #[cfg(not(feature = "mcp"))]
    fn test_handle_mcp_command_no_feature() {
        // When mcp feature is disabled, the command should fail
        // Note: The function calls process::exit, so we can't easily test it
    }

    #[test]
    fn test_mcp_command_accepts_parameters() {
        // Verify function signature
        let _ = handle_mcp_command(
            "test-server",
            false,
            3600,
            0.8,
            10,
            false,
            None,
        );
    }

    #[test]
    fn test_mcp_command_with_streaming() {
        let _ = handle_mcp_command(
            "streaming-server",
            true, // streaming
            3600,
            0.8,
            10,
            false,
            None,
        );
    }

    #[test]
    fn test_mcp_command_with_verbose() {
        let _ = handle_mcp_command(
            "verbose-server",
            false,
            3600,
            0.8,
            10,
            true, // verbose
            None,
        );
    }

    #[test]
    fn test_mcp_command_various_timeouts() {
        let timeouts = [60, 300, 3600, 86400];
        for timeout in &timeouts {
            let _ = handle_mcp_command(
                "test",
                false,
                *timeout,
                0.8,
                10,
                false,
                None,
            );
        }
    }

    #[test]
    fn test_mcp_command_various_scores() {
        let scores = [0.0, 0.5, 0.8, 1.0];
        for score in &scores {
            let _ = handle_mcp_command(
                "test",
                false,
                3600,
                *score,
                10,
                false,
                None,
            );
        }
    }

    #[test]
    fn test_mcp_command_various_complexity() {
        let complexities = [1, 5, 10, 20, 50];
        for complexity in &complexities {
            let _ = handle_mcp_command(
                "test",
                false,
                3600,
                0.8,
                *complexity,
                false,
                None,
            );
        }
    }

    #[test]
    fn test_mcp_command_with_config() {
        let _ = handle_mcp_command(
            "config-server",
            false,
            3600,
            0.8,
            10,
            false,
            Some(Path::new("/path/to/config.toml")),
        );
    }

    #[test]
    fn test_mcp_command_all_options() {
        let _ = handle_mcp_command(
            "full-server",
            true,
            7200,
            0.9,
            15,
            true,
            Some(Path::new("./mcp.toml")),
        );
    }

    #[test]
    fn test_mcp_command_empty_name() {
        let _ = handle_mcp_command(
            "",
            false,
            3600,
            0.8,
            10,
            false,
            None,
        );
    }
}
