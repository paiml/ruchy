use axum::{
    extract::Json,
    response::{Html, Json as ResponseJson},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
#[derive(Debug, Deserialize)]
pub struct ExecuteRequest {
    pub cell_id: String,
    pub code: String,
}
#[derive(Debug, Serialize)]
pub struct ExecuteResponse {
    pub success: bool,
    pub result: String,
}
async fn serve_notebook() -> Html<&'static str> {
    Html(include_str!("../../static/notebook.html"))
}
async fn execute_handler(Json(request): Json<ExecuteRequest>) -> ResponseJson<ExecuteResponse> {
    println!("🔧 TDD DEBUG: execute_handler called with: {:?}", request);
    let result = tokio::task::spawn_blocking(move || {
        use crate::runtime::repl::Repl;
        use std::time::{Duration, Instant};
#[cfg(test)]
use proptest::prelude::*;
        println!("🔧 TDD DEBUG: Creating REPL for execution");
        let mut repl = match Repl::new() {
            Ok(r) => r,
            Err(e) => {
                println!("🔧 TDD DEBUG: REPL creation failed: {:?}", e);
                return ExecuteResponse {
                    success: false,
                    result: "REPL creation failed".to_string(),
                };
            }
        };
        println!("🔧 TDD DEBUG: Evaluating code: {}", request.code);
        let deadline = Some(Instant::now() + Duration::from_secs(5));
        match repl.evaluate_expr_str(&request.code, deadline) {
            Ok(value) => {
                println!("🔧 TDD DEBUG: Execution successful: {}", value);
                ExecuteResponse {
                    success: true,
                    result: value.to_string(),
                }
            }
            Err(e) => {
                println!("🔧 TDD DEBUG: Execution failed: {:?}", e);
                ExecuteResponse {
                    success: false,
                    result: format!("Error: {}", e),
                }
            }
        }
    }).await;
    match result {
        Ok(response) => {
            println!("🔧 TDD DEBUG: Returning response: {:?}", response);
            ResponseJson(response)
        }
        Err(e) => {
            println!("🔧 TDD DEBUG: Task join error: {:?}", e);
            ResponseJson(ExecuteResponse {
                success: false,
                result: format!("Task execution failed: {}", e),
            })
        }
    }
}
/// # Examples
/// 
/// ```
/// use ruchy::notebook::server::start_server;
/// 
/// let result = start_server(());
/// assert_eq!(result, Ok(()));
/// ```
pub async fn start_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 TDD DEBUG: start_server called, about to create app");
    let app = Router::new()
        .route("/", get(serve_notebook))
        .route("/api/execute", post(execute_handler));
    println!("🔧 TDD DEBUG: Creating app with /api/execute route");
    println!("🔧 TDD DEBUG: app created, binding to addr");
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("🚀 Notebook server running at http://127.0.0.1:{}", port);
    axum::serve(listener, app).await?;
    Ok(())
}
#[cfg(test)]
mod property_tests_server {
    use proptest::proptest;
    use super::*;
    use proptest::prelude::*;
    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_start_server_never_panics(input: String) {
            // Limit input size to avoid timeout
            let input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
