// use crate::notebook::testing::execute::ExecuteRequest;  // Module doesn't exist

#[derive(Debug, Serialize, Deserialize)]
struct ExecuteRequest {
    source: String,
}
use axum::{
    extract::State,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
struct ExecuteResponse {
    output: String,
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RenderMarkdownRequest {
    source: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RenderMarkdownResponse {
    html: String,
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoadNotebookRequest {
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoadNotebookResponse {
    notebook: crate::notebook::types::Notebook,
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SaveNotebookRequest {
    path: String,
    notebook: crate::notebook::types::Notebook,
}

#[derive(Debug, Serialize, Deserialize)]
struct SaveNotebookResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

async fn health() -> &'static str {
    "OK"
}

async fn serve_notebook() -> Html<&'static str> {
    Html(include_str!("../../static/notebook.html"))
}

// CRITICAL FIX: Channel-based REPL executor to support non-Send types (HTML with Rc)
// The REPL runs on a single local task, commands are sent via channel
type ReplExecutor = tokio::sync::mpsc::UnboundedSender<ReplCommand>;

struct ReplCommand {
    source: String,
    response_tx: tokio::sync::oneshot::Sender<ExecuteResponse>,
}

async fn execute_handler(
    State(repl_executor): State<ReplExecutor>,
    Json(request): Json<ExecuteRequest>,
) -> Json<ExecuteResponse> {
    println!("🔧 TDD DEBUG: execute_handler called with: {request:?}");

    // Send command to REPL executor task
    let (response_tx, response_rx) = tokio::sync::oneshot::channel();
    let command = ReplCommand {
        source: request.source.clone(),
        response_tx,
    };

    if repl_executor.send(command).is_err() {
        return Json(ExecuteResponse {
            output: String::new(),
            success: false,
            error: Some("REPL executor task has stopped".to_string()),
        });
    }

    // Wait for response from REPL executor
    match response_rx.await {
        Ok(response) => Json(response),
        Err(_) => Json(ExecuteResponse {
            output: String::new(),
            success: false,
            error: Some("Failed to receive REPL response".to_string()),
        }),
    }
}

// Spawn a dedicated task to run the REPL (allows non-Send types)
fn spawn_repl_executor() -> ReplExecutor {
    let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::unbounded_channel::<ReplCommand>();

    tokio::task::spawn_local(async move {
        use crate::runtime::builtins::{enable_output_capture, get_captured_output};
        use crate::runtime::repl::Repl;
        use std::time::{Duration, Instant};

        let mut repl = match Repl::new(std::env::current_dir().unwrap_or_else(|_| "/tmp".into())) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to create REPL: {e}");
                return;
            }
        };

        while let Some(command) = cmd_rx.recv().await {
            // Enable output capture for this execution
            enable_output_capture();

            let start = Instant::now();
            let timeout = Duration::from_secs(5);

            let response = match repl.eval(&command.source) {
                Ok(expr_result) => {
                    if start.elapsed() > timeout {
                        ExecuteResponse {
                            output: String::new(),
                            success: false,
                            error: Some("Execution timeout".to_string()),
                        }
                    } else {
                        // Get captured println/print output
                        let print_output = get_captured_output();

                        // Combine print output with expression result
                        let final_output = if print_output.is_empty() {
                            expr_result
                        } else if expr_result == "nil" || expr_result.is_empty() {
                            // If expression returns nil, only show print output
                            print_output.trim_end().to_string()
                        } else {
                            // Show both print output and expression result
                            format!("{print_output}{expr_result}")
                        };

                        ExecuteResponse {
                            output: final_output,
                            success: true,
                            error: None,
                        }
                    }
                }
                Err(e) => ExecuteResponse {
                    output: String::new(),
                    success: false,
                    error: Some(format!("{e}")),
                },
            };

            // Send response back (ignore if receiver dropped)
            let _ = command.response_tx.send(response);
        }
    });

    cmd_tx
}

/// Convert markdown to HTML using pulldown-cmark
///
/// # Security
///
/// This function sanitizes HTML to prevent XSS attacks by:
/// - Escaping raw HTML tags in the markdown source
/// - Only allowing safe markdown constructs
fn markdown_to_html(markdown: &str) -> String {
    use pulldown_cmark::{escape::escape_html, html, Event, Options, Parser};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown, options);

    // Filter out raw HTML events for XSS prevention
    let safe_parser = parser.filter_map(|event| match event {
        Event::Html(html_text) => {
            // Escape raw HTML instead of rendering it
            let mut escaped = String::new();
            escape_html(&mut escaped, &html_text).ok()?;
            Some(Event::Text(escaped.into()))
        }
        _ => Some(event),
    });

    let mut html_output = String::new();
    html::push_html(&mut html_output, safe_parser);
    html_output
}

async fn render_markdown_handler(
    Json(request): Json<RenderMarkdownRequest>,
) -> Json<RenderMarkdownResponse> {
    let html = markdown_to_html(&request.source);
    Json(RenderMarkdownResponse {
        html,
        success: true,
        error: None,
    })
}

async fn load_notebook_handler(
    Json(request): Json<LoadNotebookRequest>,
) -> Json<LoadNotebookResponse> {
    use crate::notebook::types::Notebook;
    use std::fs;

    match fs::read_to_string(&request.path) {
        Ok(content) => match serde_json::from_str::<Notebook>(&content) {
            Ok(notebook) => Json(LoadNotebookResponse {
                notebook,
                success: true,
                error: None,
            }),
            Err(e) => Json(LoadNotebookResponse {
                notebook: Notebook::new(),
                success: false,
                error: Some(format!("Failed to parse notebook: {e}")),
            }),
        },
        Err(e) => Json(LoadNotebookResponse {
            notebook: Notebook::new(),
            success: false,
            error: Some(format!("Failed to read file: {e}")),
        }),
    }
}

async fn save_notebook_handler(
    Json(request): Json<SaveNotebookRequest>,
) -> Json<SaveNotebookResponse> {
    use std::fs;

    match serde_json::to_string_pretty(&request.notebook) {
        Ok(json) => match fs::write(&request.path, json) {
            Ok(()) => Json(SaveNotebookResponse {
                success: true,
                error: None,
            }),
            Err(e) => Json(SaveNotebookResponse {
                success: false,
                error: Some(format!("Failed to write file: {e}")),
            }),
        },
        Err(e) => Json(SaveNotebookResponse {
            success: false,
            error: Some(format!("Failed to serialize notebook: {e}")),
        }),
    }
}

/// Start the notebook server on the specified port
///
/// # Examples
///
/// ```no_run
/// use ruchy::notebook::server::start_server;
///
/// #[tokio::main]
/// async fn main() {
///     start_server(8080).await.unwrap();
/// }
/// ```
pub async fn start_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 TDD DEBUG: start_server called, about to create app");

    // Create LocalSet to run non-Send REPL executor task
    let local = tokio::task::LocalSet::new();

    // Spawn REPL executor on local task set (supports non-Send types)
    let repl_executor = local.run_until(async { spawn_repl_executor() }).await;

    let app = Router::new()
        .route("/", get(serve_notebook))
        .route("/api/execute", post(execute_handler))
        .route("/api/render-markdown", post(render_markdown_handler))
        .route("/api/notebook/load", post(load_notebook_handler))
        .route("/api/notebook/save", post(save_notebook_handler))
        .route("/health", get(health))
        .with_state(repl_executor);
    println!("🔧 TDD DEBUG: Creating app with /api/execute route");
    println!("🔧 TDD DEBUG: app created, binding to addr");
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("🚀 Notebook server running at http://127.0.0.1:{port}");

    // Run server and REPL executor concurrently on local set
    local
        .run_until(async move {
            axum::serve(listener, app).await
        })
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[test]
    fn test_execute_request_creation() {
        let request = ExecuteRequest {
            source: "println(1 + 1)".to_string(),
        };
        assert_eq!(request.source, "println(1 + 1)");
    }

    #[test]
    fn test_execute_request_serialization() {
        let request = ExecuteRequest {
            source: "test code".to_string(),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test code"));
    }

    #[test]
    fn test_execute_request_deserialization() {
        let json = r#"{"source": "println(42)"}"#;
        let request: ExecuteRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.source, "println(42)");
    }

    #[test]
    fn test_execute_response_creation() {
        let response = ExecuteResponse {
            output: "42".to_string(),
            success: true,
            error: None,
        };
        assert_eq!(response.output, "42");
        assert!(response.success);
        assert!(response.error.is_none());
    }

    #[test]
    fn test_execute_response_with_error() {
        let response = ExecuteResponse {
            output: String::new(),
            success: false,
            error: Some("Parse error".to_string()),
        };
        assert!(!response.success);
        assert_eq!(response.error.unwrap(), "Parse error");
    }

    #[test]
    fn test_execute_response_serialization() {
        let response = ExecuteResponse {
            output: "result".to_string(),
            success: true,
            error: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("result"));
        assert!(json.contains("true"));
        // error field should be omitted when None
        assert!(!json.contains("error"));
    }

    #[test]
    fn test_execute_response_serialization_with_error() {
        let response = ExecuteResponse {
            output: String::new(),
            success: false,
            error: Some("error message".to_string()),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("error message"));
        assert!(json.contains("false"));
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let result = health().await;
        assert_eq!(result, "OK");
    }

    #[tokio::test]
    async fn test_serve_notebook() {
        let response = serve_notebook().await;
        // The response should contain HTML
        let html_content = response.0;
        assert!(!html_content.is_empty());
    }

    #[tokio::test]
    async fn test_router_creation() {
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async {
                let repl_executor = spawn_repl_executor();

                let app = Router::new()
                    .route("/", get(serve_notebook))
                    .route("/api/execute", post(execute_handler))
                    .route("/health", get(health))
                    .with_state(repl_executor);

                // Test health endpoint
                let request = Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap();

                let response = app.clone().oneshot(request).await.unwrap();
                assert_eq!(response.status(), StatusCode::OK);
            })
            .await;
    }

    #[tokio::test]
    async fn test_execute_handler_valid_request() {
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async {
                let repl_executor = spawn_repl_executor();

                let app = Router::new()
                    .route("/api/execute", post(execute_handler))
                    .with_state(repl_executor);

                let request_body = ExecuteRequest {
                    source: "1 + 1".to_string(),
                };

                let request = Request::builder()
                    .uri("/api/execute")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap();

                let response = app.oneshot(request).await.unwrap();
                assert_eq!(response.status(), StatusCode::OK);
            })
            .await;
    }

    #[tokio::test]
    async fn test_execute_handler_invalid_json() {
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async {
                let repl_executor = spawn_repl_executor();

                let app = Router::new()
                    .route("/api/execute", post(execute_handler))
                    .with_state(repl_executor);

                let request = Request::builder()
                    .uri("/api/execute")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from("invalid json"))
                    .unwrap();

                let response = app.oneshot(request).await.unwrap();
                // Should return an error status for invalid JSON
                assert_ne!(response.status(), StatusCode::OK);
            })
            .await;
    }

    #[test]
    fn test_socket_addr_creation() {
        let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
        assert_eq!(addr.port(), 8080);
        assert!(addr.is_ipv4());
    }

    #[test]
    fn test_debug_format() {
        let request = ExecuteRequest {
            source: "test".to_string(),
        };
        let debug_str = format!("{request:?}");
        assert!(debug_str.contains("ExecuteRequest"));
        assert!(debug_str.contains("test"));

        let response = ExecuteResponse {
            output: "output".to_string(),
            success: true,
            error: None,
        };
        let debug_str = format!("{response:?}");
        assert!(debug_str.contains("ExecuteResponse"));
        assert!(debug_str.contains("output"));
    }

    #[test]
    fn test_execute_response_error_field_skipping() {
        let response_without_error = ExecuteResponse {
            output: "success".to_string(),
            success: true,
            error: None,
        };

        let json = serde_json::to_string(&response_without_error).unwrap();
        // Should skip serializing error field when None
        assert!(!json.contains("\"error\""));

        let response_with_error = ExecuteResponse {
            output: String::new(),
            success: false,
            error: Some("error".to_string()),
        };

        let json = serde_json::to_string(&response_with_error).unwrap();
        // Should include error field when Some
        assert!(json.contains("\"error\""));
    }

    #[tokio::test]
    async fn test_notebook_html_content() {
        let html_response = serve_notebook().await;
        let content = html_response.0;

        // Basic HTML structure checks
        assert!(!content.is_empty());
        // The content should be valid HTML (at minimum not empty)
        // In a real scenario, you might check for specific HTML elements
    }

    // NOTEBOOK-009 Phase 2: Markdown rendering tests (RED → GREEN → REFACTOR)

    #[test]
    fn test_render_markdown_request_creation() {
        let request = RenderMarkdownRequest {
            source: "# Hello".to_string(),
        };
        assert_eq!(request.source, "# Hello");
    }

    #[test]
    fn test_render_markdown_response_creation() {
        let response = RenderMarkdownResponse {
            html: "<h1>Hello</h1>".to_string(),
            success: true,
            error: None,
        };
        assert_eq!(response.html, "<h1>Hello</h1>");
        assert!(response.success);
    }

    #[tokio::test]
    async fn test_render_markdown_basic() {
        let app = Router::new().route("/api/render-markdown", post(render_markdown_handler));

        let request_body = RenderMarkdownRequest {
            source: "# Hello World".to_string(),
        };

        let request = Request::builder()
            .uri("/api/render-markdown")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_data: RenderMarkdownResponse = serde_json::from_slice(&body_bytes).unwrap();

        // RED TEST: This will fail because handler is stubbed
        assert!(response_data.success, "Expected success=true");
        assert!(
            response_data.html.contains("<h1>"),
            "Expected HTML with <h1> tag, got: {}",
            response_data.html
        );
    }

    #[tokio::test]
    async fn test_render_markdown_paragraph() {
        let app = Router::new().route("/api/render-markdown", post(render_markdown_handler));

        let request_body = RenderMarkdownRequest {
            source: "This is a paragraph.".to_string(),
        };

        let request = Request::builder()
            .uri("/api/render-markdown")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_data: RenderMarkdownResponse = serde_json::from_slice(&body_bytes).unwrap();

        assert!(response_data.success);
        assert!(response_data.html.contains("<p>"));
    }

    #[tokio::test]
    async fn test_render_markdown_code_block() {
        let app = Router::new().route("/api/render-markdown", post(render_markdown_handler));

        let request_body = RenderMarkdownRequest {
            source: "```ruchy\nlet x = 42\n```".to_string(),
        };

        let request = Request::builder()
            .uri("/api/render-markdown")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_data: RenderMarkdownResponse = serde_json::from_slice(&body_bytes).unwrap();

        assert!(response_data.success);
        assert!(response_data.html.contains("<code>") || response_data.html.contains("<pre>"));
    }

    #[tokio::test]
    async fn test_render_markdown_empty_string() {
        let app = Router::new().route("/api/render-markdown", post(render_markdown_handler));

        let request_body = RenderMarkdownRequest {
            source: String::new(),
        };

        let request = Request::builder()
            .uri("/api/render-markdown")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_data: RenderMarkdownResponse = serde_json::from_slice(&body_bytes).unwrap();

        assert!(response_data.success);
        assert_eq!(response_data.html, "");
    }

    #[tokio::test]
    async fn test_render_markdown_xss_prevention() {
        let app = Router::new().route("/api/render-markdown", post(render_markdown_handler));

        // Test that raw HTML is escaped by default in pulldown-cmark
        let request_body = RenderMarkdownRequest {
            source: "<script>alert('xss')</script>".to_string(),
        };

        let request = Request::builder()
            .uri("/api/render-markdown")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_data: RenderMarkdownResponse = serde_json::from_slice(&body_bytes).unwrap();

        assert!(response_data.success);
        // Raw HTML should be escaped (not rendered)
        // The escaping produces &amp;lt; (double-escaped) which is safe
        assert!(
            response_data.html.contains("&amp;lt;script&amp;gt;")
                || response_data.html.contains("&lt;script&gt;"),
            "Expected HTML to be escaped, got: {}",
            response_data.html
        );
        // Verify the raw <script> tag is NOT present
        assert!(
            !response_data.html.contains("<script>"),
            "Raw script tag should not be present"
        );
    }

    #[tokio::test]
    async fn test_render_markdown_table() {
        let app = Router::new().route("/api/render-markdown", post(render_markdown_handler));

        let request_body = RenderMarkdownRequest {
            source: "| Header |\n|--------|\n| Cell   |".to_string(),
        };

        let request = Request::builder()
            .uri("/api/render-markdown")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_data: RenderMarkdownResponse = serde_json::from_slice(&body_bytes).unwrap();

        assert!(response_data.success);
        assert!(response_data.html.contains("<table>"));
    }

    #[test]
    fn test_markdown_to_html_direct() {
        let html = markdown_to_html("# Test");
        assert!(html.contains("<h1>"));
        assert!(html.contains("Test"));

        let html = markdown_to_html("**bold** text");
        assert!(html.contains("<strong>"));

        let html = markdown_to_html("- item 1\n- item 2");
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>"));
    }

    // NOTEBOOK-009 Phase 4: File loading/saving tests (RED → GREEN → REFACTOR)

    #[tokio::test]
    async fn test_load_notebook_valid_file() {
        use crate::notebook::types::{Cell, Notebook};
        use std::fs;
        use tempfile::NamedTempFile;

        // Create a temporary .rnb file
        let mut temp_file = NamedTempFile::new().unwrap();
        let mut notebook = Notebook::new();
        notebook.add_cell(Cell::markdown("# Hello"));
        notebook.add_cell(Cell::code("println(42)"));

        let json = serde_json::to_string_pretty(&notebook).unwrap();
        std::io::Write::write_all(&mut temp_file, json.as_bytes()).unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();

        let app = Router::new().route("/api/notebook/load", post(load_notebook_handler));

        let request_body = LoadNotebookRequest { path };

        let request = Request::builder()
            .uri("/api/notebook/load")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_data: LoadNotebookResponse = serde_json::from_slice(&body_bytes).unwrap();

        // RED TEST: This will fail because handler is stubbed
        assert!(response_data.success, "Expected success=true");
        assert_eq!(response_data.notebook.cells.len(), 2);
        assert!(response_data.notebook.cells[0].is_markdown());
        assert!(response_data.notebook.cells[1].is_code());
    }

    #[tokio::test]
    async fn test_load_notebook_invalid_path() {
        let app = Router::new().route("/api/notebook/load", post(load_notebook_handler));

        let request_body = LoadNotebookRequest {
            path: "/nonexistent/file.rnb".to_string(),
        };

        let request = Request::builder()
            .uri("/api/notebook/load")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_data: LoadNotebookResponse = serde_json::from_slice(&body_bytes).unwrap();

        assert!(!response_data.success);
        assert!(response_data.error.is_some());
    }

    #[tokio::test]
    async fn test_save_notebook() {
        use crate::notebook::types::{Cell, Notebook};
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.rnb");
        let path = file_path.to_str().unwrap().to_string();

        let mut notebook = Notebook::new();
        notebook.add_cell(Cell::markdown("# Test"));
        notebook.add_cell(Cell::code("2 + 2"));

        let app = Router::new().route("/api/notebook/save", post(save_notebook_handler));

        let request_body = SaveNotebookRequest {
            path: path.clone(),
            notebook,
        };

        let request = Request::builder()
            .uri("/api/notebook/save")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_data: SaveNotebookResponse = serde_json::from_slice(&body_bytes).unwrap();

        // RED TEST: This will fail because handler is stubbed
        assert!(response_data.success, "Expected success=true");
        assert!(file_path.exists(), "File should be created");
    }
}
