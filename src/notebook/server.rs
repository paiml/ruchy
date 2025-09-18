// use crate::notebook::testing::execute::ExecuteRequest;  // Module doesn't exist

#[derive(Debug, Serialize, Deserialize)]
struct ExecuteRequest {
    source: String,
}
use axum::{response::Html, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Serialize, Deserialize)]
struct ExecuteResponse {
    output: String,
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

async fn execute_handler(Json(request): Json<ExecuteRequest>) -> Json<ExecuteResponse> {
    println!("🔧 TDD DEBUG: execute_handler called with: {request:?}");
    let result = tokio::task::spawn_blocking(move || {
        use crate::runtime::repl::Repl;
        use std::time::{Duration, Instant};

        let mut repl = match Repl::new(std::env::current_dir().unwrap_or_else(|_| "/tmp".into())) {
            Ok(r) => r,
            Err(e) => return ExecuteResponse {
                output: String::new(),
                success: false,
                error: Some(format!("Failed to create REPL: {e}")),
            },
        };
        let start = Instant::now();
        let timeout = Duration::from_secs(5);

        match repl.process_line(&request.source) {
            Ok(_should_exit) => {
                if start.elapsed() > timeout {
                    ExecuteResponse {
                        output: String::new(),
                        success: false,
                        error: Some("Execution timeout".to_string()),
                    }
                } else {
                    ExecuteResponse {
                        output: "Execution completed".to_string(),
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
        }
    })
    .await
    .unwrap_or_else(|e| ExecuteResponse {
        output: String::new(),
        success: false,
        error: Some(format!("Task panic: {e}")),
    });

    Json(result)
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
    let app = Router::new()
        .route("/", get(serve_notebook))
        .route("/api/execute", post(execute_handler))
        .route("/health", get(health));
    println!("🔧 TDD DEBUG: Creating app with /api/execute route");
    println!("🔧 TDD DEBUG: app created, binding to addr");
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("🚀 Notebook server running at http://127.0.0.1:{port}");
    axum::serve(listener, app).await?;
    Ok(())
}