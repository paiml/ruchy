// TDD: GREEN - Minimal implementation for notebook execution
use anyhow::Result;
use axum::{
    Router, 
    routing::post,
    extract::Json,
    response::Json as ResponseJson,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct ExecuteRequest {
    pub code: String,
    pub cell_id: String,
}

#[derive(Debug, Serialize)]
pub struct ExecuteResponse {
    pub success: bool,
    pub result: String,
}

async fn execute_handler(Json(request): Json<ExecuteRequest>) -> ResponseJson<ExecuteResponse> {
    // TDD: Execute synchronously in blocking task to avoid REPL async issues
    let result = tokio::task::spawn_blocking(move || {
        // TDD: Use EXACT same execution path as REPL - no divergence
        use ruchy::runtime::repl::Repl;
        use std::time::{Duration, Instant};
        
        let mut repl = match Repl::new() {
            Ok(r) => r,
            Err(_) => return ExecuteResponse {
                success: false,
                result: "REPL creation failed".to_string(),
            },
        };
        
        // Use same method as REPL with timeout
        let deadline = Some(Instant::now() + Duration::from_secs(5));
        match repl.evaluate_expr_str(&request.code, deadline) {
            Ok(value) => ExecuteResponse {
                success: true,
                result: value.to_string(),
            },
            Err(e) => ExecuteResponse {
                success: false,
                result: format!("Error: {}", e),
            },
        }
    }).await;
    
    match result {
        Ok(response) => ResponseJson(response),
        Err(_) => ResponseJson(ExecuteResponse {
            success: false,
            result: "Task execution failed".to_string(),
        }),
    }
}

fn create_app() -> Router {
    Router::new().route("/api/execute", post(execute_handler))
}

pub async fn start_server(port: u16) -> Result<()> {
    let app = create_app();
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    println!("🚀 Notebook server running at http://{}", addr);
    
    axum::serve(listener, app).await?;
    Ok(())
}