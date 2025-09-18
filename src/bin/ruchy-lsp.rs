//! Ruchy Language Server binary
//!
//! This binary starts the Ruchy Language Server Protocol server for editor integration.

use clap::{Arg, Command};
#[cfg(feature = "mcp")]
use ruchy::lsp::start_server;
use std::process;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    // Initialize basic logging

    let matches = Command::new("ruchy-lsp")
        .version("0.3.2")
        .author("Ruchy Contributors")
        .about("Ruchy Language Server Protocol server")
        .arg(
            Arg::new("stdio")
                .long("stdio")
                .help("Use stdio for communication (default)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("tcp")
                .long("tcp")
                .help("Use TCP for communication")
                .value_name("PORT")
                .num_args(1),
        )
        .get_matches();

    if let Some(port) = matches.get_one::<String>("tcp") {
        let port: u16 = port.parse().unwrap_or_else(|_| {
            error!("Invalid port number: {port}");
            process::exit(1);
        });

        info!("Starting Ruchy LSP server on TCP port {port}");
        #[cfg(feature = "mcp")]
        {
            if let Err(err) = ruchy::lsp::start_tcp_server(port).await {
                error!("LSP server error: {err}");
                process::exit(1);
            }
        }
        #[cfg(not(feature = "mcp"))]
        {
            error!("LSP support requires the 'mcp' feature to be enabled");
            process::exit(1);
        }
    } else {
        info!("Starting Ruchy LSP server on stdio");
        #[cfg(feature = "mcp")]
        {
            if let Err(err) = start_server().await {
                error!("LSP server error: {err}");
                process::exit(1);
            }
        }
        #[cfg(not(feature = "mcp"))]
        {
            error!("LSP support requires the 'mcp' feature to be enabled");
            process::exit(1);
        }
    }
}
