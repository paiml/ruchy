//! Language Server Protocol implementation for Ruchy
//!
//! Based on SPECIFICATION.md section 8: LSP Specification

mod analyzer;
mod capabilities;
mod formatter;
mod server;

pub use analyzer::SemanticAnalyzer;
pub use capabilities::{ruchy_token_to_lsp, RuchyTokenType, SEMANTIC_TOKEN_LEGEND};
pub use formatter::Formatter;
pub use server::RuchyLanguageServer;

use anyhow;
use std::collections::HashMap;
use tokio::net::TcpListener;
use tower_lsp::lsp_types::Url;
use tower_lsp::{LspService, Server};

/// Workspace management for LSP
pub struct Workspace {
    documents: HashMap<Url, String>,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    pub fn add_document(&mut self, uri: Url, content: String) {
        self.documents.insert(uri, content);
    }

    pub fn update_document(&mut self, uri: &Url, content: String) {
        self.documents.insert(uri.clone(), content);
    }

    /// Get document content by URI
    ///
    /// # Errors
    ///
    /// Returns an error if the document is not found in the workspace
    pub fn get_document(&self, uri: &Url) -> anyhow::Result<&String> {
        self.documents
            .get(uri)
            .ok_or_else(|| anyhow::anyhow!("Document not found: {uri}"))
    }

    pub fn remove_document(&mut self, uri: &Url) {
        self.documents.remove(uri);
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

/// Start the LSP server
///
/// # Errors
///
/// Returns an error if the LSP server fails to start or encounters I/O issues
pub async fn start_server() -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(RuchyLanguageServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}

/// Start the LSP server on a TCP port for debugging
///
/// # Errors
///
/// Returns an error if the TCP listener fails to bind or connection handling fails
pub async fn start_tcp_server(port: u16) -> anyhow::Result<()> {
    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).await?;

    while let Ok((stream, _)) = listener.accept().await {
        let (read, write) = tokio::io::split(stream);

        let (service, socket) = LspService::new(RuchyLanguageServer::new);

        tokio::spawn(async move {
            Server::new(read, write, socket).serve(service).await;
        });
    }

    Ok(())
}
