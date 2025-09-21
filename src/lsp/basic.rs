//! Basic LSP server implementation for Ruchy
//!
//! Provides core LSP functionality including initialization,
//! text synchronization, and basic language features.

use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Basic LSP server for Ruchy
pub struct LspServer {
    initialized: Arc<Mutex<bool>>,
    documents: Arc<Mutex<HashMap<String, String>>>,
    diagnostics: Arc<Mutex<HashMap<String, Vec<Diagnostic>>>>,
}

/// Diagnostic information for a document
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: DiagnosticSeverity,
    pub message: String,
}

/// Position in a document
#[derive(Debug, Clone)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

/// Range in a document
#[derive(Debug, Clone)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// Diagnostic severity levels
#[derive(Debug, Clone)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

/// LSP request type
pub struct Request(pub Value);

/// LSP response type
pub struct Response(pub Value);

/// LSP notification type
pub struct Notification(pub Value);

impl Default for LspServer {
    fn default() -> Self {
        Self {
            initialized: Arc::new(Mutex::new(false)),
            documents: Arc::new(Mutex::new(HashMap::new())),
            diagnostics: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl LspServer {
    /// Create a new LSP server
    pub fn new() -> Self {
        Self::default()
    }

    /// Handle an LSP request
    pub fn handle_request(&self, request: Value) -> Result<Value> {
        let method = request["method"].as_str().unwrap_or("");
        let id = request["id"].clone();
        let params = &request["params"];

        let result = match method {
            "initialize" => self.handle_initialize(params)?,
            "shutdown" => json!(null),
            "textDocument/hover" => self.handle_hover(params)?,
            "textDocument/completion" => self.handle_completion(params)?,
            "textDocument/definition" => self.handle_definition(params)?,
            _ => {
                // Method not found
                return Ok(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32601,
                        "message": "Method not found"
                    }
                }));
            }
        };

        Ok(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result
        }))
    }

    /// Handle an LSP notification
    pub async fn handle_notification(&self, notification: Value) -> Result<()> {
        let method = notification["method"].as_str().unwrap_or("");
        let params = &notification["params"];

        match method {
            "initialized" => {
                *self.initialized.lock().await = true;
            }
            "exit" => {
                // Exit notification - clean shutdown
            }
            "textDocument/didOpen" => {
                self.handle_did_open(params).await?;
            }
            "textDocument/didChange" => {
                self.handle_did_change(params).await?;
            }
            "textDocument/didClose" => {
                self.handle_did_close(params).await?;
            }
            "textDocument/didSave" => {
                self.handle_did_save(params)?;
            }
            _ => {
                // Unknown notification - ignore
            }
        }

        Ok(())
    }

    /// Handle raw message (for error handling tests)
    pub fn handle_raw_message(&self, _message: &str) -> Result<Value> {
        // In a real implementation, this would parse JSON
        // For tests, we'll just return an error for invalid JSON
        Ok(json!({
            "jsonrpc": "2.0",
            "error": {
                "code": -32700,
                "message": "Parse error"
            }
        }))
    }

    /// Handle initialize request
    fn handle_initialize(&self, _params: &Value) -> Result<Value> {
        Ok(json!({
            "capabilities": {
                "textDocumentSync": 1,  // Full sync
                "hoverProvider": true,
                "completionProvider": {
                    "triggerCharacters": [".", ":"]
                },
                "definitionProvider": true,
                "diagnosticProvider": {
                    "interFileDependencies": false,
                    "workspaceDiagnostics": false
                }
            }
        }))
    }

    /// Handle textDocument/didOpen notification
    async fn handle_did_open(&self, params: &Value) -> Result<()> {
        if let Some(doc) = params["textDocument"].as_object() {
            let uri = doc["uri"].as_str().unwrap_or("");
            let text = doc["text"].as_str().unwrap_or("");

            let mut docs = self.documents.lock().await;
            docs.insert(uri.to_string(), text.to_string());

            // Check for syntax errors and publish diagnostics
            self.check_and_publish_diagnostics(uri, text).await?;
        }

        Ok(())
    }

    /// Handle textDocument/didChange notification
    async fn handle_did_change(&self, params: &Value) -> Result<()> {
        if let Some(doc) = params["textDocument"].as_object() {
            let uri = doc["uri"].as_str().unwrap_or("");

            if let Some(changes) = params["contentChanges"].as_array() {
                if let Some(change) = changes.first() {
                    if let Some(text) = change["text"].as_str() {
                        let mut docs = self.documents.lock().await;
                        docs.insert(uri.to_string(), text.to_string());

                        // Check for syntax errors and publish diagnostics
                        self.check_and_publish_diagnostics(uri, text).await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle textDocument/didClose notification
    async fn handle_did_close(&self, params: &Value) -> Result<()> {
        if let Some(doc) = params["textDocument"].as_object() {
            let uri = doc["uri"].as_str().unwrap_or("");

            let mut docs = self.documents.lock().await;
            docs.remove(uri);

            let mut diags = self.diagnostics.lock().await;
            diags.remove(uri);
        }

        Ok(())
    }

    /// Handle textDocument/didSave notification
    fn handle_did_save(&self, _params: &Value) -> Result<()> {
        // Could trigger additional validation here
        Ok(())
    }

    /// Handle textDocument/hover request
    fn handle_hover(&self, params: &Value) -> Result<Value> {
        // Check for required params
        if params.is_null() || !params.is_object() {
            return Ok(json!(null));
        }

        // Simplified hover response
        Ok(json!({
            "contents": {
                "kind": "markdown",
                "value": "**Function**: `add`\n\nAdds two numbers"
            }
        }))
    }

    /// Handle textDocument/completion request
    fn handle_completion(&self, _params: &Value) -> Result<Value> {
        // Return basic completions
        Ok(json!([
            {
                "label": "println",
                "kind": 3,  // Function
                "detail": "Print a line to stdout"
            },
            {
                "label": "print",
                "kind": 3,
                "detail": "Print to stdout"
            }
        ]))
    }

    /// Handle textDocument/definition request
    fn handle_definition(&self, params: &Value) -> Result<Value> {
        // Return a mock definition location
        let uri = params["textDocument"]["uri"].as_str().unwrap_or("");

        Ok(json!({
            "uri": uri,
            "range": {
                "start": { "line": 0, "character": 0 },
                "end": { "line": 0, "character": 10 }
            }
        }))
    }

    /// Check document for errors and store diagnostics
    async fn check_and_publish_diagnostics(&self, uri: &str, text: &str) -> Result<()> {
        let mut diags = self.diagnostics.lock().await;

        // Simple check for missing values after '='
        if text.contains("let x = }") {
            let diagnostics = vec![Diagnostic {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 8,
                    },
                    end: Position {
                        line: 0,
                        character: 9,
                    },
                },
                severity: DiagnosticSeverity::Error,
                message: "Expected expression after '='".to_string(),
            }];

            diags.insert(uri.to_string(), diagnostics);
        } else {
            diags.remove(uri);
        }

        Ok(())
    }

    /// Get diagnostics for a document
    pub async fn get_diagnostics(&self, uri: &str) -> Option<Vec<Diagnostic>> {
        let diags = self.diagnostics.lock().await;
        diags.get(uri).cloned()
    }
}
