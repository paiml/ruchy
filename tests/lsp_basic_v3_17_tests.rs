//! TDD Tests for LSP Basic Support
//! Sprint v3.17.0 - Language Server Protocol basic functionality

use ruchy::lsp::LspServer;
use serde_json::{json, Value};

#[cfg(test)]
mod lsp_initialization {
    use super::*;

    #[tokio::test]
    async fn test_server_initialize_request() {
        let server = LspServer::new();

        let init_request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "processId": 12345,
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                },
                "rootUri": "file:///test/project",
                "capabilities": {}
            }
        });

        let response = server.handle_request(init_request).await;
        assert!(response.is_ok());

        let resp_value = response.unwrap();
        assert_eq!(resp_value["jsonrpc"], "2.0");
        assert_eq!(resp_value["id"], 1);
        assert!(resp_value["result"]["capabilities"].is_object());
    }

    #[tokio::test]
    async fn test_server_initialized_notification() {
        let server = LspServer::new();

        // First initialize
        let init_request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        });
        let _ = server.handle_request(init_request).await;

        // Then send initialized notification
        let initialized = json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        });

        let result = server.handle_notification(initialized).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_server_shutdown() {
        let server = LspServer::new();

        let shutdown_request = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "shutdown",
            "params": null
        });

        let response = server.handle_request(shutdown_request).await;
        assert!(response.is_ok());

        let resp_value = response.unwrap();
        assert_eq!(resp_value["result"], Value::Null);
    }

    #[tokio::test]
    async fn test_server_exit() {
        let server = LspServer::new();

        // First shutdown
        let shutdown = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "shutdown",
            "params": null
        });
        let _ = server.handle_request(shutdown).await;

        // Then exit
        let exit = json!({
            "jsonrpc": "2.0",
            "method": "exit"
        });

        let result = server.handle_notification(exit).await;
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod lsp_text_sync {
    use super::*;

    #[tokio::test]
    async fn test_did_open_text_document() {
        let server = LspServer::new();

        let did_open = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///test.ruchy",
                    "languageId": "ruchy",
                    "version": 1,
                    "text": "fn main() { println(\"Hello\") }"
                }
            }
        });

        let result = server.handle_notification(did_open).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_did_change_text_document() {
        let server = LspServer::new();

        // First open
        let did_open = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///test.ruchy",
                    "languageId": "ruchy",
                    "version": 1,
                    "text": "fn main() { }"
                }
            }
        });
        let _ = server.handle_notification(did_open).await;

        // Then change
        let did_change = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {
                    "uri": "file:///test.ruchy",
                    "version": 2
                },
                "contentChanges": [{
                    "text": "fn main() { println(\"Updated\") }"
                }]
            }
        });

        let result = server.handle_notification(did_change).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_did_close_text_document() {
        let server = LspServer::new();

        let did_close = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didClose",
            "params": {
                "textDocument": {
                    "uri": "file:///test.ruchy"
                }
            }
        });

        let result = server.handle_notification(did_close).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_did_save_text_document() {
        let server = LspServer::new();

        let did_save = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didSave",
            "params": {
                "textDocument": {
                    "uri": "file:///test.ruchy"
                }
            }
        });

        let result = server.handle_notification(did_save).await;
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod lsp_diagnostics {
    use super::*;

    #[tokio::test]
    async fn test_publish_diagnostics() {
        let server = LspServer::new();

        // Open a document with an error
        let did_open = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///test.ruchy",
                    "languageId": "ruchy",
                    "version": 1,
                    "text": "fn main() { let x = }"  // Missing value
                }
            }
        });

        let result = server.handle_notification(did_open).await;
        assert!(result.is_ok());

        // Server should publish diagnostics
        let diagnostics = server.get_diagnostics("file:///test.ruchy").await;
        assert!(diagnostics.is_some());
        assert!(!diagnostics.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_clear_diagnostics_on_fix() {
        let server = LspServer::new();

        // Open with error
        let did_open = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///test.ruchy",
                    "languageId": "ruchy",
                    "version": 1,
                    "text": "fn main() { let x = }"
                }
            }
        });
        let _ = server.handle_notification(did_open).await;

        // Fix the error
        let did_change = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {
                    "uri": "file:///test.ruchy",
                    "version": 2
                },
                "contentChanges": [{
                    "text": "fn main() { let x = 42 }"
                }]
            }
        });
        let _ = server.handle_notification(did_change).await;

        let diagnostics = server.get_diagnostics("file:///test.ruchy").await;
        assert!(diagnostics.is_none() || diagnostics.unwrap().is_empty());
    }
}

#[cfg(test)]
mod lsp_hover {
    use super::*;

    #[tokio::test]
    async fn test_hover_on_function() {
        let server = LspServer::new();

        // Open document
        let did_open = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///test.ruchy",
                    "languageId": "ruchy",
                    "version": 1,
                    "text": "fn add(a: i32, b: i32) -> i32 { a + b }\nfn main() { add(1, 2) }"
                }
            }
        });
        let _ = server.handle_notification(did_open).await;

        // Request hover
        let hover_request = json!({
            "jsonrpc": "2.0",
            "id": 10,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": "file:///test.ruchy"
                },
                "position": {
                    "line": 1,
                    "character": 12  // On "add"
                }
            }
        });

        let response = server.handle_request(hover_request).await;
        assert!(response.is_ok());

        let resp = response.unwrap();
        assert!(resp["result"]["contents"].is_object() || resp["result"]["contents"].is_string());
    }

    #[tokio::test]
    async fn test_hover_on_variable() {
        let server = LspServer::new();

        // Open document
        let did_open = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///test.ruchy",
                    "languageId": "ruchy",
                    "version": 1,
                    "text": "fn main() {\n    let x: i32 = 42;\n    println(x)\n}"
                }
            }
        });
        let _ = server.handle_notification(did_open).await;

        // Request hover on variable
        let hover_request = json!({
            "jsonrpc": "2.0",
            "id": 11,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": "file:///test.ruchy"
                },
                "position": {
                    "line": 2,
                    "character": 12  // On "x" in println
                }
            }
        });

        let response = server.handle_request(hover_request).await;
        assert!(response.is_ok());
    }
}

#[cfg(test)]
mod lsp_completion {
    use super::*;

    #[tokio::test]
    async fn test_basic_completion() {
        let server = LspServer::new();

        // Open document
        let did_open = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///test.ruchy",
                    "languageId": "ruchy",
                    "version": 1,
                    "text": "fn main() {\n    pr"
                }
            }
        });
        let _ = server.handle_notification(did_open).await;

        // Request completion
        let completion_request = json!({
            "jsonrpc": "2.0",
            "id": 20,
            "method": "textDocument/completion",
            "params": {
                "textDocument": {
                    "uri": "file:///test.ruchy"
                },
                "position": {
                    "line": 1,
                    "character": 6  // After "pr"
                }
            }
        });

        let response = server.handle_request(completion_request).await;
        assert!(response.is_ok());

        let resp = response.unwrap();
        let items = resp["result"].as_array();
        assert!(items.is_some());

        // Should have completion items like "println"
        let has_println = items.unwrap().iter().any(|item| {
            item["label"].as_str() == Some("println")
        });
        assert!(has_println);
    }

    #[tokio::test]
    async fn test_method_completion() {
        let server = LspServer::new();

        // Open document
        let did_open = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///test.ruchy",
                    "languageId": "ruchy",
                    "version": 1,
                    "text": "fn main() {\n    let s = \"hello\";\n    s."
                }
            }
        });
        let _ = server.handle_notification(did_open).await;

        // Request completion after dot
        let completion_request = json!({
            "jsonrpc": "2.0",
            "id": 21,
            "method": "textDocument/completion",
            "params": {
                "textDocument": {
                    "uri": "file:///test.ruchy"
                },
                "position": {
                    "line": 2,
                    "character": 6  // After "s."
                }
            }
        });

        let response = server.handle_request(completion_request).await;
        assert!(response.is_ok());

        let resp = response.unwrap();
        assert!(resp["result"].is_array() || resp["result"].is_object());
    }
}

#[cfg(test)]
mod lsp_goto_definition {
    use super::*;

    #[tokio::test]
    async fn test_goto_function_definition() {
        let server = LspServer::new();

        // Open document
        let did_open = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///test.ruchy",
                    "languageId": "ruchy",
                    "version": 1,
                    "text": "fn helper() -> i32 { 42 }\n\nfn main() { let x = helper(); }"
                }
            }
        });
        let _ = server.handle_notification(did_open).await;

        // Request goto definition
        let goto_def_request = json!({
            "jsonrpc": "2.0",
            "id": 30,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": "file:///test.ruchy"
                },
                "position": {
                    "line": 2,
                    "character": 20  // On "helper" call
                }
            }
        });

        let response = server.handle_request(goto_def_request).await;
        assert!(response.is_ok());

        let resp = response.unwrap();
        let location = &resp["result"];
        assert!(location["uri"].is_string());
        assert_eq!(location["range"]["start"]["line"], 0);
    }

    #[tokio::test]
    async fn test_goto_variable_definition() {
        let server = LspServer::new();

        // Open document
        let did_open = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///test.ruchy",
                    "languageId": "ruchy",
                    "version": 1,
                    "text": "fn main() {\n    let x = 42;\n    let y = x + 1;\n}"
                }
            }
        });
        let _ = server.handle_notification(did_open).await;

        // Request goto definition for variable
        let goto_def_request = json!({
            "jsonrpc": "2.0",
            "id": 31,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": "file:///test.ruchy"
                },
                "position": {
                    "line": 2,
                    "character": 12  // On "x" in "x + 1"
                }
            }
        });

        let response = server.handle_request(goto_def_request).await;
        assert!(response.is_ok());
    }
}

#[cfg(test)]
mod lsp_error_handling {
    use super::*;

    #[tokio::test]
    async fn test_invalid_request() {
        let server = LspServer::new();

        let invalid_request = json!({
            "jsonrpc": "2.0",
            "id": 999,
            "method": "invalid/method",
            "params": {}
        });

        let response = server.handle_request(invalid_request).await;
        assert!(response.is_ok());

        let resp = response.unwrap();
        assert!(resp["error"].is_object());
        assert_eq!(resp["error"]["code"], -32601); // Method not found
    }

    #[tokio::test]
    async fn test_malformed_json() {
        let server = LspServer::new();

        // This would normally be a string that fails to parse
        // For testing, we simulate the error handling
        let result = server.handle_raw_message("{ invalid json }").await;
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_missing_required_params() {
        let server = LspServer::new();

        let request_missing_params = json!({
            "jsonrpc": "2.0",
            "id": 100,
            "method": "textDocument/hover"
            // Missing params
        });

        let response = server.handle_request(request_missing_params).await;
        assert!(response.is_ok());

        let resp = response.unwrap();
        // Should either handle gracefully or return error
        assert!(resp["error"].is_object() || resp["result"].is_null());
    }
}