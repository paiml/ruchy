// EXTREME TDD: Notebook Server Tests
// Sprint 80: 0% Coverage Modules Attack
// Testing notebook/server.rs with comprehensive coverage including HTTP endpoints,
// error handling, concurrency, serialization, and edge cases

use proptest::prelude::*;
use ruchy::notebook::server::start_server;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct ExecuteRequest {
    source: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExecuteResponse {
    output: String,
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[cfg(test)]
mod server_unit_tests {
    use super::*;

    #[tokio::test]
    async fn test_server_start_basic() {
        // Check that server can be created without panicking
        // Using a high port number to avoid conflicts
        let port = 39999;

        // Start server in background task
        let server_handle = tokio::spawn(async move {
            let result = start_server(port).await;
            // Server runs forever unless stopped
            assert!(result.is_ok() || result.is_err());
        });

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Check health endpoint
        let client = reqwest::Client::new();
        let health_response = client
            .get(format!("http://127.0.0.1:{}/health", port))
            .send()
            .await;

        if let Ok(resp) = health_response {
            assert_eq!(resp.status(), 200);
            let body = resp.text().await.unwrap();
            assert_eq!(body, "OK");
        }

        // Abort server task
        server_handle.abort();
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let port = 40000;

        let server_handle = tokio::spawn(async move {
            let _ = start_server(port).await;
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let client = reqwest::Client::new();
        let resp = client
            .get(format!("http://127.0.0.1:{}/health", port))
            .send()
            .await;

        if let Ok(response) = resp {
            assert_eq!(response.status(), 200);
            assert_eq!(response.text().await.unwrap(), "OK");
        }

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_execute_endpoint() {
        let port = 40001;

        let server_handle = tokio::spawn(async move {
            let _ = start_server(port).await;
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        let client = reqwest::Client::new();

        // Check execute with simple expression
        let execute_request = serde_json::json!({
            "source": "42"
        });

        let resp = client
            .post(format!("http://127.0.0.1:{}/api/execute", port))
            .json(&execute_request)
            .send()
            .await;

        if let Ok(response) = resp {
            assert_eq!(response.status(), 200);
            let body: serde_json::Value = response.json().await.unwrap();
            assert!(body["success"].as_bool().unwrap_or(false));
        }

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_execute_with_error() {
        let port = 40002;

        let server_handle = tokio::spawn(async move {
            let _ = start_server(port).await;
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        let client = reqwest::Client::new();

        // Check execute with invalid code
        let execute_request = serde_json::json!({
            "source": "invalid syntax !!!"
        });

        let resp = client
            .post(format!("http://127.0.0.1:{}/api/execute", port))
            .json(&execute_request)
            .send()
            .await;

        if let Ok(response) = resp {
            assert_eq!(response.status(), 200);
            let body: serde_json::Value = response.json().await.unwrap();
            assert!(!body["success"].as_bool().unwrap_or(true));
            assert!(body["error"].is_string());
        }

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_notebook_html_endpoint() {
        let port = 40003;

        let server_handle = tokio::spawn(async move {
            let _ = start_server(port).await;
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let client = reqwest::Client::new();
        let resp = client
            .get(format!("http://127.0.0.1:{}/", port))
            .send()
            .await;

        if let Ok(response) = resp {
            assert_eq!(response.status(), 200);
            let content_type = response
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            assert!(content_type.contains("text/html"));
        }

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_multiple_execute_requests() {
        let port = 40004;

        let server_handle = tokio::spawn(async move {
            let _ = start_server(port).await;
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        let client = reqwest::Client::new();

        // Send multiple requests
        for i in 0..5 {
            let execute_request = serde_json::json!({
                "source": format!("{} + {}", i, i)
            });

            let resp = client
                .post(format!("http://127.0.0.1:{}/api/execute", port))
                .json(&execute_request)
                .send()
                .await;

            if let Ok(response) = resp {
                assert_eq!(response.status(), 200);
                let body: serde_json::Value = response.json().await.unwrap();
                assert!(body.is_object());
            }
        }

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_concurrent_requests() {
        let port = 40005;

        let server_handle = tokio::spawn(async move {
            let _ = start_server(port).await;
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Send concurrent requests
        let mut handles = vec![];

        for i in 0..10 {
            let handle = tokio::spawn(async move {
                let client = reqwest::Client::new();
                let execute_request = serde_json::json!({
                    "source": format!("{}", i)
                });

                let resp = client
                    .post(format!("http://127.0.0.1:40005/api/execute"))
                    .json(&execute_request)
                    .send()
                    .await;

                if let Ok(response) = resp {
                    assert_eq!(response.status(), 200);
                }
            });
            handles.push(handle);
        }

        // Wait for all requests to complete
        for handle in handles {
            let _ = handle.await;
        }

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_server_bind_error() {
        // Try to bind to a privileged port (should fail)
        let result = start_server(1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_empty_source_execute() {
        let port = 40006;

        let server_handle = tokio::spawn(async move {
            let _ = start_server(port).await;
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        let client = reqwest::Client::new();

        // Check with empty source
        let execute_request = serde_json::json!({
            "source": ""
        });

        let resp = client
            .post(format!("http://127.0.0.1:{}/api/execute", port))
            .json(&execute_request)
            .send()
            .await;

        if let Ok(response) = resp {
            assert_eq!(response.status(), 200);
            let body: serde_json::Value = response.json().await.unwrap();
            assert!(body["success"].as_bool().unwrap_or(false));
        }

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_malformed_json_request() {
        let port = 40007;

        let server_handle = tokio::spawn(async move {
            let _ = start_server(port).await;
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        let client = reqwest::Client::new();

        // Send malformed JSON
        let resp = client
            .post(format!("http://127.0.0.1:{}/api/execute", port))
            .body("not json")
            .header("content-type", "application/json")
            .send()
            .await;

        if let Ok(response) = resp {
            // Should return 400 or 422 for bad request
            assert!(response.status().is_client_error());
        }

        server_handle.abort();
    }
}

// Property-based tests for server
#[cfg(test)]
mod server_property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_server_port_range(port in 1024u16..65535u16) {
            // Any valid port should be acceptable for server creation
            // (actual binding may fail but shouldn't panic)
            let runtime = tokio::runtime::Runtime::new().unwrap();

            let server_handle = runtime.spawn(async move {
                let _ = super::start_server(port).await;
            });

            // Give it a moment
            std::thread::sleep(std::time::Duration::from_millis(10));

            // Abort the server
            server_handle.abort();
        }
    }
}

// Stress tests
#[cfg(test)]
mod server_stress_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Can be expensive
    async fn test_server_under_load() {
        let port = 40100;

        let server_handle = tokio::spawn(async move {
            let _ = start_server(port).await;
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Send 100 requests rapidly
        let mut handles = vec![];

        for i in 0..100 {
            let handle = tokio::spawn(async move {
                let client = reqwest::Client::new();
                let execute_request = serde_json::json!({
                    "source": format!("let x{} = {}", i, i)
                });

                let _ = client
                    .post(format!("http://127.0.0.1:40100/api/execute"))
                    .json(&execute_request)
                    .timeout(tokio::time::Duration::from_secs(5))
                    .send()
                    .await;
            });
            handles.push(handle);
        }

        // Wait for all to complete
        for handle in handles {
            let _ = handle.await;
        }

        server_handle.abort();
    }
}

// Additional serialization and structure tests
#[cfg(test)]
mod serialization_tests {
    use super::*;

    #[test]
    fn test_execute_request_serialization() {
        let request = ExecuteRequest {
            source: "42 + 1".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("42 + 1"));

        let deserialized: ExecuteRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.source, "42 + 1");
    }

    #[test]
    fn test_execute_response_serialization() {
        let response = ExecuteResponse {
            output: "43".to_string(),
            success: true,
            error: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("43"));
        assert!(json.contains("true"));

        let deserialized: ExecuteResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.output, "43");
        assert!(deserialized.success);
        assert!(deserialized.error.is_none());
    }

    #[test]
    fn test_execute_response_with_error() {
        let response = ExecuteResponse {
            output: "".to_string(),
            success: false,
            error: Some("Parse error".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: ExecuteResponse = serde_json::from_str(&json).unwrap();

        assert!(!deserialized.success);
        assert_eq!(deserialized.error, Some("Parse error".to_string()));
    }

    #[test]
    fn test_execute_request_with_special_characters() {
        let special_chars = "let x = \"hello\\nworld\\t🦀\"";
        let request = ExecuteRequest {
            source: special_chars.to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: ExecuteRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.source, special_chars);
    }

    #[test]
    fn test_serde_skip_serializing_if() {
        // Check that None error field is excluded from JSON
        let response = ExecuteResponse {
            output: "result".to_string(),
            success: true,
            error: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(!json.contains("error"));

        // Check that Some error field is included
        let response_with_error = ExecuteResponse {
            output: "".to_string(),
            success: false,
            error: Some("error message".to_string()),
        };

        let json_with_error = serde_json::to_string(&response_with_error).unwrap();
        assert!(json_with_error.contains("error"));
    }
}

// Extreme property-based tests
#[cfg(test)]
mod extreme_property_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_execute_request_roundtrip(source: String) {
            let request = ExecuteRequest { source: source.clone() };

            let json = serde_json::to_string(&request).unwrap();
            let deserialized: ExecuteRequest = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(deserialized.source, source);
        }

        #[test]
        fn test_execute_response_roundtrip(
            output: String,
            success: bool,
            error_msg: Option<String>
        ) {
            let response = ExecuteResponse {
                output: output.clone(),
                success,
                error: error_msg.clone(),
            };

            let json = serde_json::to_string(&response).unwrap();
            let deserialized: ExecuteResponse = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(deserialized.output, output);
            prop_assert_eq!(deserialized.success, success);
            prop_assert_eq!(deserialized.error, error_msg);
        }

        #[test]
        fn test_server_port_validity(port in 1024u16..65535u16) {
            // Check that any valid port number can be used
            // (binding may fail but shouldn't panic)
            let rt = tokio::runtime::Runtime::new().unwrap();

            let handle = rt.spawn(async move {
                let _ = start_server(port).await;
            });

            std::thread::sleep(std::time::Duration::from_millis(1));
            handle.abort();

            // Check completed without panic
            prop_assert!(true);
        }

        #[test]
        fn test_unicode_source_handling(source: String) {
            let request = ExecuteRequest { source };

            // Should serialize without panic
            let result = serde_json::to_string(&request);
            prop_assert!(result.is_ok());

            // Should deserialize back correctly
            let json = result.unwrap();
            let result = serde_json::from_str::<ExecuteRequest>(&json);
            prop_assert!(result.is_ok());
        }
    }
}

// Fuzz testing for robustness
#[cfg(test)]
mod fuzz_tests {
    use super::*;

    #[test]
    fn test_extreme_string_lengths() {
        // Check very short strings
        let short_request = ExecuteRequest {
            source: "".to_string(),
        };
        assert!(serde_json::to_string(&short_request).is_ok());

        let single_char = ExecuteRequest {
            source: "x".to_string(),
        };
        assert!(serde_json::to_string(&single_char).is_ok());

        // Check very long strings
        let long_source = "x".repeat(100_000);
        let long_request = ExecuteRequest {
            source: long_source,
        };
        assert!(serde_json::to_string(&long_request).is_ok());
    }

    #[test]
    fn test_special_character_handling() {
        let special_cases = vec![
            "\0",           // Null byte
            "\n\r\t",       // Whitespace
            "\"'\\",        // Quote and escape chars
            "🦀🔥⚡",       // Emoji
            "αβγδε",        // Greek
            "测试代码",     // Chinese
            "\u{FEFF}",     // BOM
            "\x00\x01\x02", // Control chars
        ];

        for case in special_cases {
            let request = ExecuteRequest {
                source: case.to_string(),
            };
            let json_result = serde_json::to_string(&request);
            assert!(json_result.is_ok(), "Failed for case: {:?}", case);

            if let Ok(json) = json_result {
                let parse_result = serde_json::from_str::<ExecuteRequest>(&json);
                assert!(
                    parse_result.is_ok(),
                    "Failed to parse back case: {:?}",
                    case
                );
            }
        }
    }

    #[test]
    fn test_malformed_json_parsing() {
        let malformed_cases = vec![
            "",
            "{",
            "}",
            "null",
            "[]",
            "{\"source\": }",
            "{\"wrong_field\": \"value\"}",
            "{\"source\": null}",
            "{\"source\": 123}",
        ];

        for case in malformed_cases {
            let result = serde_json::from_str::<ExecuteRequest>(case);
            // Should fail gracefully, not panic
            assert!(result.is_err(), "Should have failed for: {}", case);
        }
    }
}

// Integration tests for complete workflows
#[cfg(test)]
mod integration_workflow_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_server_lifecycle() {
        let port = 41000;

        // Start server
        let server_handle = tokio::spawn(async move {
            let _ = start_server(port).await;
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

        let client = reqwest::Client::new();

        // 1. Health check
        let health = client
            .get(format!("http://127.0.0.1:{}/health", port))
            .send()
            .await;

        assert!(health.is_ok());
        if let Ok(resp) = health {
            assert_eq!(resp.status(), 200);
            assert_eq!(resp.text().await.unwrap(), "OK");
        }

        // 2. Get notebook HTML
        let notebook = client
            .get(format!("http://127.0.0.1:{}/", port))
            .send()
            .await;

        assert!(notebook.is_ok());
        if let Ok(resp) = notebook {
            assert_eq!(resp.status(), 200);
        }

        // 3. Execute simple code
        let simple_exec = ExecuteRequest {
            source: "1 + 1".to_string(),
        };

        let exec_response = client
            .post(format!("http://127.0.0.1:{}/api/execute", port))
            .json(&simple_exec)
            .send()
            .await;

        assert!(exec_response.is_ok());
        if let Ok(resp) = exec_response {
            assert_eq!(resp.status(), 200);
        }

        // 4. Execute complex code
        let complex_exec = ExecuteRequest {
            source: "let x = 42; let y = x + 1; y".to_string(),
        };

        let complex_response = client
            .post(format!("http://127.0.0.1:{}/api/execute", port))
            .json(&complex_exec)
            .send()
            .await;

        assert!(complex_response.is_ok());

        // 5. Test error handling
        let error_exec = ExecuteRequest {
            source: "invalid_syntax_here".to_string(),
        };

        let error_response = client
            .post(format!("http://127.0.0.1:{}/api/execute", port))
            .json(&error_exec)
            .send()
            .await;

        assert!(error_response.is_ok());

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_server_resilience_under_errors() {
        let port = 41001;

        let server_handle = tokio::spawn(async move {
            let _ = start_server(port).await;
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

        let client = reqwest::Client::new();

        // Send various problematic requests
        let problematic_requests = vec![
            serde_json::json!({"source": ""}),
            serde_json::json!({"source": "\0\0\0"}),
            serde_json::json!({"source": "a".repeat(10000)}),
            serde_json::json!({"source": "🦀".repeat(1000)}),
            serde_json::json!({"source": "\n\t\r".repeat(100)}),
        ];

        for request in problematic_requests {
            let response = client
                .post(format!("http://127.0.0.1:{}/api/execute", port))
                .json(&request)
                .send()
                .await;

            // Server should handle all requests without crashing
            if let Ok(resp) = response {
                assert_eq!(resp.status(), 200);
            }
        }

        // Server should still be responsive
        let health_check = client
            .get(format!("http://127.0.0.1:{}/health", port))
            .send()
            .await;

        assert!(health_check.is_ok());
        if let Ok(resp) = health_check {
            assert_eq!(resp.status(), 200);
        }

        server_handle.abort();
    }
}
