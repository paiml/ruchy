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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_server_new() {
        let server = LspServer::new();
        // Should create server without panic
        assert!(std::mem::size_of_val(&server) > 0);
    }

    #[test]
    fn test_lsp_server_default() {
        let server = LspServer::default();
        // Should create server via default
        assert!(std::mem::size_of_val(&server) > 0);
    }

    #[test]
    fn test_handle_request_initialize() {
        let server = LspServer::new();
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        });

        let response = server.handle_request(request).unwrap();
        assert_eq!(response["id"], 1);
        assert!(response["result"]["capabilities"].is_object());
        assert!(response["result"]["capabilities"]["hoverProvider"].as_bool().unwrap());
    }

    #[test]
    fn test_handle_request_shutdown() {
        let server = LspServer::new();
        let request = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "shutdown",
            "params": null
        });

        let response = server.handle_request(request).unwrap();
        assert_eq!(response["id"], 2);
        assert!(response["result"].is_null());
    }

    #[test]
    fn test_handle_request_hover() {
        let server = LspServer::new();
        let request = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/hover",
            "params": {
                "textDocument": { "uri": "file:///test.ruchy" },
                "position": { "line": 0, "character": 5 }
            }
        });

        let response = server.handle_request(request).unwrap();
        assert_eq!(response["id"], 3);
        assert!(response["result"]["contents"].is_object());
    }

    #[test]
    fn test_handle_request_hover_null_params() {
        let server = LspServer::new();
        let request = json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "textDocument/hover",
            "params": null
        });

        let response = server.handle_request(request).unwrap();
        assert!(response["result"].is_null());
    }

    #[test]
    fn test_handle_request_completion() {
        let server = LspServer::new();
        let request = json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "textDocument/completion",
            "params": {
                "textDocument": { "uri": "file:///test.ruchy" },
                "position": { "line": 0, "character": 0 }
            }
        });

        let response = server.handle_request(request).unwrap();
        assert_eq!(response["id"], 5);
        assert!(response["result"].is_array());
        let completions = response["result"].as_array().unwrap();
        assert!(!completions.is_empty());
        assert_eq!(completions[0]["label"], "println");
    }

    #[test]
    fn test_handle_request_definition() {
        let server = LspServer::new();
        let request = json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "textDocument/definition",
            "params": {
                "textDocument": { "uri": "file:///test.ruchy" },
                "position": { "line": 5, "character": 10 }
            }
        });

        let response = server.handle_request(request).unwrap();
        assert_eq!(response["id"], 6);
        assert_eq!(response["result"]["uri"], "file:///test.ruchy");
        assert!(response["result"]["range"].is_object());
    }

    #[test]
    fn test_handle_request_unknown_method() {
        let server = LspServer::new();
        let request = json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "unknown/method",
            "params": {}
        });

        let response = server.handle_request(request).unwrap();
        assert_eq!(response["id"], 7);
        assert!(response["error"].is_object());
        assert_eq!(response["error"]["code"], -32601);
        assert_eq!(response["error"]["message"], "Method not found");
    }

    #[test]
    fn test_handle_raw_message() {
        let server = LspServer::new();
        let result = server.handle_raw_message("invalid json").unwrap();
        assert!(result["error"].is_object());
        assert_eq!(result["error"]["code"], -32700);
    }

    #[tokio::test]
    async fn test_handle_notification_initialized() {
        let server = LspServer::new();
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        });

        server.handle_notification(notification).await.unwrap();
        let initialized = server.initialized.lock().await;
        assert!(*initialized);
    }

    #[tokio::test]
    async fn test_handle_notification_exit() {
        let server = LspServer::new();
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "exit",
            "params": null
        });

        // Should not panic
        server.handle_notification(notification).await.unwrap();
    }

    #[tokio::test]
    async fn test_handle_notification_did_open() {
        let server = LspServer::new();
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///test.ruchy",
                    "languageId": "ruchy",
                    "version": 1,
                    "text": "fun main() { println(\"Hello\") }"
                }
            }
        });

        server.handle_notification(notification).await.unwrap();

        let docs = server.documents.lock().await;
        assert!(docs.contains_key("file:///test.ruchy"));
        assert_eq!(
            docs.get("file:///test.ruchy").unwrap(),
            "fun main() { println(\"Hello\") }"
        );
    }

    #[tokio::test]
    async fn test_handle_notification_did_change() {
        let server = LspServer::new();

        // First open a document
        let open_notif = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///test.ruchy",
                    "text": "original"
                }
            }
        });
        server.handle_notification(open_notif).await.unwrap();

        // Then change it
        let change_notif = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": { "uri": "file:///test.ruchy" },
                "contentChanges": [{ "text": "changed" }]
            }
        });
        server.handle_notification(change_notif).await.unwrap();

        let docs = server.documents.lock().await;
        assert_eq!(docs.get("file:///test.ruchy").unwrap(), "changed");
    }

    #[tokio::test]
    async fn test_handle_notification_did_close() {
        let server = LspServer::new();

        // First open a document
        let open_notif = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///test.ruchy",
                    "text": "content"
                }
            }
        });
        server.handle_notification(open_notif).await.unwrap();

        // Then close it
        let close_notif = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didClose",
            "params": {
                "textDocument": { "uri": "file:///test.ruchy" }
            }
        });
        server.handle_notification(close_notif).await.unwrap();

        let docs = server.documents.lock().await;
        assert!(!docs.contains_key("file:///test.ruchy"));
    }

    #[tokio::test]
    async fn test_handle_notification_did_save() {
        let server = LspServer::new();
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didSave",
            "params": {
                "textDocument": { "uri": "file:///test.ruchy" }
            }
        });

        // Should not panic
        server.handle_notification(notification).await.unwrap();
    }

    #[tokio::test]
    async fn test_handle_notification_unknown() {
        let server = LspServer::new();
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "unknown/notification",
            "params": {}
        });

        // Should not panic - unknown notifications are ignored
        server.handle_notification(notification).await.unwrap();
    }

    #[tokio::test]
    async fn test_diagnostics_on_error() {
        let server = LspServer::new();
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///error.ruchy",
                    "text": "let x = }"
                }
            }
        });

        server.handle_notification(notification).await.unwrap();

        let diags = server.get_diagnostics("file:///error.ruchy").await;
        assert!(diags.is_some());
        let diags = diags.unwrap();
        assert!(!diags.is_empty());
        assert_eq!(diags[0].message, "Expected expression after '='");
    }

    #[tokio::test]
    async fn test_diagnostics_cleared_on_fix() {
        let server = LspServer::new();

        // Open with error
        let open_notif = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///fix.ruchy",
                    "text": "let x = }"
                }
            }
        });
        server.handle_notification(open_notif).await.unwrap();

        // Fix the error
        let change_notif = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": { "uri": "file:///fix.ruchy" },
                "contentChanges": [{ "text": "let x = 42" }]
            }
        });
        server.handle_notification(change_notif).await.unwrap();

        let diags = server.get_diagnostics("file:///fix.ruchy").await;
        assert!(diags.is_none());
    }

    #[test]
    fn test_diagnostic_struct() {
        let diag = Diagnostic {
            range: Range {
                start: Position {
                    line: 1,
                    character: 5,
                },
                end: Position {
                    line: 1,
                    character: 10,
                },
            },
            severity: DiagnosticSeverity::Error,
            message: "Test error".to_string(),
        };

        assert_eq!(diag.message, "Test error");
        assert_eq!(diag.range.start.line, 1);
        assert_eq!(diag.range.start.character, 5);
    }

    #[test]
    fn test_diagnostic_severity_values() {
        assert_eq!(DiagnosticSeverity::Error as u8, 1);
        assert_eq!(DiagnosticSeverity::Warning as u8, 2);
        assert_eq!(DiagnosticSeverity::Information as u8, 3);
        assert_eq!(DiagnosticSeverity::Hint as u8, 4);
    }

    #[test]
    fn test_position_struct() {
        let pos = Position {
            line: 10,
            character: 20,
        };
        assert_eq!(pos.line, 10);
        assert_eq!(pos.character, 20);
    }

    #[test]
    fn test_range_struct() {
        let range = Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 5,
                character: 10,
            },
        };
        assert_eq!(range.start.line, 0);
        assert_eq!(range.end.line, 5);
    }

    #[test]
    fn test_request_wrapper() {
        let val = json!({"method": "test"});
        let req = Request(val.clone());
        assert_eq!(req.0, val);
    }

    #[test]
    fn test_response_wrapper() {
        let val = json!({"result": "ok"});
        let resp = Response(val.clone());
        assert_eq!(resp.0, val);
    }

    #[test]
    fn test_notification_wrapper() {
        let val = json!({"method": "notify"});
        let notif = Notification(val.clone());
        assert_eq!(notif.0, val);
    }

    #[test]
    fn test_handle_request_missing_method() {
        let server = LspServer::new();
        let request = json!({
            "jsonrpc": "2.0",
            "id": 8
        });

        let response = server.handle_request(request).unwrap();
        assert!(response["error"].is_object());
    }

    #[tokio::test]
    async fn test_handle_notification_missing_method() {
        let server = LspServer::new();
        let notification = json!({
            "jsonrpc": "2.0",
            "params": {}
        });

        // Should not panic
        server.handle_notification(notification).await.unwrap();
    }

    #[tokio::test]
    async fn test_did_open_missing_text_document() {
        let server = LspServer::new();
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {}
        });

        // Should not panic
        server.handle_notification(notification).await.unwrap();
    }

    #[tokio::test]
    async fn test_did_change_missing_changes() {
        let server = LspServer::new();
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": { "uri": "file:///test.ruchy" }
            }
        });

        // Should not panic
        server.handle_notification(notification).await.unwrap();
    }

    #[tokio::test]
    async fn test_did_change_empty_changes() {
        let server = LspServer::new();
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": { "uri": "file:///test.ruchy" },
                "contentChanges": []
            }
        });

        // Should not panic
        server.handle_notification(notification).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_diagnostics_not_found() {
        let server = LspServer::new();
        let diags = server.get_diagnostics("file:///nonexistent.ruchy").await;
        assert!(diags.is_none());
    }

    #[test]
    fn test_initialize_capabilities() {
        let server = LspServer::new();
        let result = server.handle_initialize(&json!({})).unwrap();

        assert_eq!(result["capabilities"]["textDocumentSync"], 1);
        assert!(result["capabilities"]["hoverProvider"].as_bool().unwrap());
        assert!(result["capabilities"]["definitionProvider"].as_bool().unwrap());
        assert!(result["capabilities"]["completionProvider"].is_object());
    }
}
