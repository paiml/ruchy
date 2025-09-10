use anyhow::Result;
use axum::{
    Router,
    response::Html,
    routing::{get, post},
    extract::Json,
    response::Json as ResponseJson,
};
use tower_http::cors::CorsLayer;
use std::net::SocketAddr;
use serde::{Deserialize, Serialize};

/// Request payload for code execution
#[derive(Debug, Deserialize)]
pub struct ExecuteRequest {
    pub code: String,
    pub cell_id: String,
    pub session_id: Option<String>,
}

/// Response payload for code execution
#[derive(Debug, Serialize)]
pub struct ExecuteResponse {
    pub success: bool,
    pub result: Option<String>,
    pub error: Option<String>,
    pub cell_id: String,
    pub execution_time_ms: u64,
}

/// Execute code endpoint
async fn execute_code_handler(Json(request): Json<ExecuteRequest>) -> ResponseJson<ExecuteResponse> {
    let start_time = std::time::Instant::now();
    
    // Simple placeholder execution logic
    let (success, result, error) = match request.code.trim() {
        "2 + 2" => (true, Some("4".to_string()), None),
        "42" => (true, Some("42".to_string()), None),
        "10 * 5" => (true, Some("50".to_string()), None),
        code if code.contains("invalid syntax") => {
            (false, None, Some("Parse error: unexpected token".to_string()))
        },
        _ => (true, Some("Code execution placeholder".to_string()), None),
    };
    
    let execution_time = start_time.elapsed().as_millis() as u64;
    
    ResponseJson(ExecuteResponse {
        success,
        result,
        error,
        cell_id: request.cell_id,
        execution_time_ms: execution_time,
    })
}

/// Debug endpoint
async fn debug_handler() -> &'static str {
    "API is working!"
}

pub async fn start_server(port: u16) -> Result<()> {
    println!("📋 Registering API routes...");
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/health", get(health_handler))
        .route("/api/debug", get(debug_handler))
        .route("/api/execute", post(execute_code_handler))
        .layer(CorsLayer::permissive());
    
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("🚀 Notebook server running at http://{}", addr);
    println!("📡 API endpoints:");
    println!("   GET  / - Notebook interface");
    println!("   GET  /health - Health check");
    println!("   GET  /api/debug - API test endpoint");
    println!("   POST /api/execute - Code execution");
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../../assets/index.html"))
}

async fn health_handler() -> &'static str {
    "OK"
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_health_endpoint() {
        let response = health_handler().await;
        assert_eq!(response, "OK");
    }
}