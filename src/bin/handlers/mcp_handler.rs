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
    use ruchy::mcp::StdioTransport;

    let server = prepare_server(name, verbose)?;
    let runtime = tokio::runtime::Runtime::new().context("Failed to create async runtime")?;
    runtime.block_on(async {
        if verbose {
            eprintln!("   Transport: stdio");
            eprintln!("✅ MCP server running");
        }
        server
            .run(StdioTransport::new())
            .await
            .context("MCP server error")
    })
}

/// Build the server `ruchy mcp` runs, with every tool registered
/// (`ruchy::mcp::all_tools`), without starting the stdio loop.
///
/// # Errors
/// Returns error if the server cannot be built
#[cfg(feature = "mcp")]
fn prepare_server(name: &str, verbose: bool) -> Result<ruchy::mcp::Server> {
    use anyhow::Context;
    use ruchy::mcp::{all_tools, create_named_mcp_server};

    let server = create_named_mcp_server(name).context("Failed to create MCP server")?;
    if verbose {
        let tools = all_tools();
        eprintln!("🚀 Starting Ruchy MCP Server: {name}");
        eprintln!("   Registered {} tools:", tools.len());
        for (tool_name, tool) in &tools {
            eprintln!("   - {}: {}", tool_name, tool.description());
        }
    }
    Ok(server)
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
    anyhow::bail!("MCP support not enabled. Rebuild with: cargo build --features mcp")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(feature = "mcp"))]
    fn test_handle_mcp_command_no_feature() {
        // Without the mcp feature the command must fail with an error the caller can
        // report (main prints it and exits 1) — never by exiting the process itself,
        // which killed every test binary that touched this function (PMAT-104).
        let err = handle_mcp_command("test-server", false, 3600, 0.8, 10, false, None)
            .expect_err("mcp stub must return Err without the mcp feature");
        assert!(
            err.to_string().contains("MCP support not enabled"),
            "unexpected error text: {err}"
        );
    }

    // MCPTOOLS-1: these build the server `ruchy mcp` runs but never start its
    // stdio loop, which blocks on standard input until the client hangs up.

    #[test]
    #[cfg(feature = "mcp")]
    fn test_mcptools1_prepare_server_registers_the_rhl_tools() {
        let server = prepare_server("test-server", false).expect("server");
        for name in ruchy::mcp::rhl_tools::RHL_TOOL_NAMES {
            assert!(server.has_tool(name), "`{name}` not registered");
        }
    }

    #[test]
    #[cfg(feature = "mcp")]
    fn test_mcptools1_prepare_server_registers_the_ruchy_tools() {
        let server = prepare_server("test-server", true).expect("server");
        for (name, _) in ruchy::mcp::create_ruchy_tools() {
            assert!(server.has_tool(name), "`{name}` not registered");
        }
        assert!(!server.has_tool("no-such-tool"));
    }

    #[test]
    #[cfg(feature = "mcp")]
    fn test_mcptools1_prepare_server_accepts_any_name() {
        for name in ["", "ruchy-mcp", "full-server"] {
            let server = prepare_server(name, false).expect("server");
            assert!(server.has_tool("rhl_check"));
        }
    }
}
