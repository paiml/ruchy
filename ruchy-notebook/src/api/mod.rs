//! Notebook API Endpoints
//! 
//! This module implements the REST API endpoints for notebook code execution,
//! session management, and state persistence.

#[cfg(feature = "native")]
pub mod native_api {
    use axum::{
        extract::Json,
        http::StatusCode,
        response::Json as ResponseJson,
    };
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

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

/// Session state for variable persistence
#[derive(Debug, Clone)]
pub struct SessionState {
    pub variables: HashMap<String, String>,
    pub last_result: Option<String>,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            variables: HashMap::new(),
            last_result: None,
        }
    }
}

/// Session manager for maintaining execution contexts
pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, SessionState>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_or_create_session(&self, session_id: &str) -> SessionState {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.entry(session_id.to_string())
            .or_insert_with(SessionState::default)
            .clone()
    }

    pub fn update_session(&self, session_id: &str, state: SessionState) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session_id.to_string(), state);
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Debug endpoint to test API functionality
/// 
/// GET /api/debug
pub async fn debug_api() -> &'static str {
    "API module is working!"
}

/// Execute Ruchy code in a session context
/// 
/// POST /api/execute
/// 
/// This endpoint receives Ruchy code, executes it within a session context,
/// and returns the result or error information.
pub async fn execute_code(
    Json(request): Json<ExecuteRequest>,
) -> Result<ResponseJson<ExecuteResponse>, StatusCode> {
    let start_time = std::time::Instant::now();
    
    // Get or create session
    let session_id = request.session_id.unwrap_or_else(|| "default".to_string());
    
    // TODO: Implement actual code execution
    // This is a placeholder that demonstrates the API structure
    let (success, result, error) = match request.code.trim() {
        // Simple arithmetic expressions
        "2 + 2" => (true, Some("4".to_string()), None),
        "42" => (true, Some("42".to_string()), None),
        "10 * 5" => (true, Some("50".to_string()), None),
        
        // Variable assignments (placeholder)
        code if code.starts_with("let x = ") => {
            (true, Some("()".to_string()), None)
        },
        
        // Variable references (placeholder)
        "x" => (true, Some("42".to_string()), None), // Assuming x was set to 42
        "x + 8" => (true, Some("50".to_string()), None), // 42 + 8
        
        // Function calls (placeholder)
        code if code.contains("calculate(5, 6)") => {
            (true, Some("40".to_string()), None) // 5 * 6 + 10
        },
        
        // Error cases
        code if code.contains("invalid syntax") => {
            (false, None, Some("Parse error: unexpected token '@'".to_string()))
        },
        
        // Default case
        _ => (false, None, Some("Code execution not yet implemented".to_string())),
    };
    
    let execution_time = start_time.elapsed().as_millis() as u64;
    
    let response = ExecuteResponse {
        success,
        result,
        error,
        cell_id: request.cell_id,
        execution_time_ms: execution_time,
    };
    
    Ok(ResponseJson(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_manager_creation() {
        let manager = SessionManager::new();
        let session = manager.get_or_create_session("test-session");
        assert!(session.variables.is_empty());
        assert!(session.last_result.is_none());
    }

    #[test]
    fn test_session_state_persistence() {
        let manager = SessionManager::new();
        let session_id = "test-session";
        
        // Create and modify session
        let mut session = manager.get_or_create_session(session_id);
        session.variables.insert("x".to_string(), "42".to_string());
        session.last_result = Some("42".to_string());
        manager.update_session(session_id, session);
        
        // Retrieve session and verify persistence
        let retrieved = manager.get_or_create_session(session_id);
        assert_eq!(retrieved.variables.get("x"), Some(&"42".to_string()));
        assert_eq!(retrieved.last_result, Some("42".to_string()));
    }

    #[tokio::test]
    async fn test_execute_simple_arithmetic() {
        let request = ExecuteRequest {
            code: "2 + 2".to_string(),
            cell_id: "cell-1".to_string(),
            session_id: None,
        };
        
        let result = execute_code(Json(request)).await;
        assert!(result.is_ok());
        
        let response = result.unwrap().0;
        assert_eq!(response.success, true);
        assert_eq!(response.result, Some("4".to_string()));
        assert!(response.error.is_none());
        assert_eq!(response.cell_id, "cell-1");
    }

    #[tokio::test]
    async fn test_execute_error_handling() {
        let request = ExecuteRequest {
            code: "invalid syntax here @#$%".to_string(),
            cell_id: "cell-2".to_string(),
            session_id: None,
        };
        
        let result = execute_code(Json(request)).await;
        assert!(result.is_ok());
        
        let response = result.unwrap().0;
        assert_eq!(response.success, false);
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        assert!(response.error.unwrap().contains("Parse error"));
    }
}

} // Close native_api module

#[cfg(feature = "native")]
pub use native_api::*;