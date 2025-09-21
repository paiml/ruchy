// use crate::notebook::testing::execute::ExecuteRequest;  // Module doesn't exist

#[derive(Debug, Serialize, Deserialize)]
struct ExecuteRequest {
    source: String,
}
use axum::{
    response::Html,
    routing::{get, post},
    Json, Router,
};
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
            Err(e) => {
                return ExecuteResponse {
                    output: String::new(),
                    success: false,
                    error: Some(format!("Failed to create REPL: {e}")),
                }
            }
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
        let app = Router::new()
            .route("/", get(serve_notebook))
            .route("/api/execute", post(execute_handler))
            .route("/health", get(health));

        // Test health endpoint
        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_execute_handler_valid_request() {
        let app = Router::new().route("/api/execute", post(execute_handler));

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
    }

    #[tokio::test]
    async fn test_execute_handler_invalid_json() {
        let app = Router::new().route("/api/execute", post(execute_handler));

        let request = Request::builder()
            .uri("/api/execute")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from("invalid json"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        // Should return an error status for invalid JSON
        assert_ne!(response.status(), StatusCode::OK);
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
}
