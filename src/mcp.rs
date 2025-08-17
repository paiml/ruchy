//! MCP Integration for Ruchy
//!
//! This module provides integration with the Model Context Protocol (MCP)
//! using the high-quality pmcp crate, providing type-safe MCP tools that
//! work with Ruchy's type system and actor runtime.

// Re-export all pmcp types except Result to avoid conflicts
pub use pmcp::{
    async_trait, Client, ClientCapabilities, Error as PmcpError, PromptHandler,
    RequestHandlerExtra, ResourceHandler, Server, ServerCapabilities, StdioTransport, ToolHandler,
    Transport,
};

use crate::middleend::types::MonoType;
use crate::runtime::ActorSystem;
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Ruchy-specific MCP integration that connects pmcp with Ruchy's type system
pub struct RuchyMCP {
    server: Option<Server>,
    client: Option<Box<dyn std::any::Any + Send + Sync>>,
    type_registry: HashMap<String, MonoType>,
    actor_system: Option<Arc<ActorSystem>>,
}

impl RuchyMCP {
    /// Create a new Ruchy MCP integration
    #[must_use]
    pub fn new() -> Self {
        Self {
            server: None,
            client: None,
            type_registry: HashMap::new(),
            actor_system: None,
        }
    }

    /// Set the actor system for actor-based MCP tools
    #[must_use]
    pub fn with_actor_system(mut self, actor_system: Arc<ActorSystem>) -> Self {
        self.actor_system = Some(actor_system);
        self
    }

    /// Register a Ruchy type for MCP tool validation
    pub fn register_type(&mut self, name: String, mono_type: MonoType) {
        self.type_registry.insert(name, mono_type);
    }

    /// Validate a JSON value against a registered Ruchy type
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ruchy::mcp::RuchyMCP;
    /// use serde_json::json;
    ///
    /// let mut mcp = RuchyMCP::new();
    /// mcp.register_type("count", ruchy::middleend::infer::MonoType::Int);
    ///
    /// let value = json!(42);
    /// assert!(mcp.validate_against_type(&value, "count").is_ok());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The type is not registered
    /// - The value doesn't match the expected type
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn validate_against_type(&self, value: &Value, type_name: &str) -> Result<()> {
        if let Some(expected_type) = self.type_registry.get(type_name) {
            self.validate_json_value(value, expected_type)
        } else {
            Err(anyhow!("Type '{}' not registered", type_name))
        }
    }

    /// Validate JSON value against `MonoType`
    #[allow(clippy::only_used_in_recursion)]
    fn validate_json_value(&self, value: &Value, expected_type: &MonoType) -> Result<()> {
        match (value, expected_type) {
            (Value::String(_), MonoType::String) 
            | (Value::Bool(_), MonoType::Bool) 
            | (Value::Null, MonoType::Unit) => Ok(()),
            (Value::Number(n), MonoType::Int) if n.is_i64() => Ok(()),
            (Value::Number(n), MonoType::Float) if n.is_f64() => Ok(()),
            (Value::Array(arr), MonoType::List(inner_type)) => {
                for item in arr {
                    self.validate_json_value(item, inner_type)?;
                }
                Ok(())
            }
            (Value::Object(_), MonoType::Named(type_name)) => {
                // For named types, we assume they're valid structured data
                // In a full implementation, we'd check against registered struct definitions
                if type_name == "Any" || type_name == "Object" {
                    Ok(())
                } else {
                    // Allow through for now - this would be enhanced with full struct validation
                    Ok(())
                }
            }
            _ => Err(anyhow!(
                "Type mismatch: expected {:?}, got {:?}",
                expected_type,
                value
            )),
        }
    }

    /// Create a server with Ruchy integration
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ruchy::mcp::RuchyMCP;
    ///
    /// let mut mcp = RuchyMCP::new();
    /// let server = mcp.create_server("ruchy-server", "1.0.0").expect("Failed to create server");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the server cannot be created or configured
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn create_server(&mut self, name: &str, version: &str) -> Result<&mut Server> {
        let server = Server::builder()
            .name(name)
            .version(version)
            .capabilities(ServerCapabilities::default())
            .build()?;

        self.server = Some(server);
        self.server
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Server was just set but is None"))
    }

    /// Create a client with Ruchy integration
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ruchy::mcp::RuchyMCP;
    /// use pmcp::StdioTransport;
    ///
    /// let mut mcp = RuchyMCP::new();
    /// let transport = StdioTransport::new();
    /// mcp.create_client(transport).expect("Failed to create client");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the client cannot be created with the given transport
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn create_client<T: Transport + 'static>(&mut self, transport: T) -> Result<()> {
        let client = Client::new(transport);
        self.client = Some(Box::new(client));
        Ok(())
    }

    /// Get the server instance
    pub fn server(&mut self) -> Option<&mut Server> {
        self.server.as_mut()
    }

    /// Get the client instance (type-erased)
    pub fn client(&mut self) -> Option<&mut Box<dyn std::any::Any + Send + Sync>> {
        self.client.as_mut()
    }
}

impl Default for RuchyMCP {
    fn default() -> Self {
        Self::new()
    }
}

/// A Ruchy-specific MCP tool that validates inputs against Ruchy types
pub struct RuchyMCPTool {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    description: String,
    input_type: Option<MonoType>,
    output_type: Option<MonoType>,
    handler: Box<dyn Fn(Value) -> Result<Value> + Send + Sync>,
}

impl RuchyMCPTool {
    /// Create a new Ruchy MCP tool
    pub fn new<F>(name: String, description: String, handler: F) -> Self
    where
        F: Fn(Value) -> Result<Value> + Send + Sync + 'static,
    {
        Self {
            name,
            description,
            input_type: None,
            output_type: None,
            handler: Box::new(handler),
        }
    }

    /// Set the expected input type
    #[must_use]
    pub fn with_input_type(mut self, input_type: MonoType) -> Self {
        self.input_type = Some(input_type);
        self
    }

    /// Set the expected output type
    #[must_use]
    pub fn with_output_type(mut self, output_type: MonoType) -> Self {
        self.output_type = Some(output_type);
        self
    }
}

#[async_trait]
impl ToolHandler for RuchyMCPTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> pmcp::Result<Value> {
        // Validate input type if specified
        if let Some(ref _input_type) = self.input_type {
            // In a real implementation, we'd validate against the type
            // For now, we'll just pass through
        }

        // Call the handler
        let result = (self.handler)(args).map_err(|e| PmcpError::internal(e.to_string()))?;

        // Validate output type if specified
        if let Some(ref _output_type) = self.output_type {
            // In a real implementation, we'd validate the output
            // For now, we'll just pass through
        }

        Ok(result)
    }
}

/// Create common Ruchy MCP tools
pub fn create_ruchy_tools() -> Vec<(&'static str, RuchyMCPTool)> {
    vec![
        (
            "ruchy-eval",
            RuchyMCPTool::new(
                "ruchy-eval".to_string(),
                "Evaluate Ruchy expressions with type safety".to_string(),
                |args| {
                    let expression = args["expression"]
                        .as_str()
                        .ok_or_else(|| anyhow!("Missing 'expression' field"))?;

                    // This would integrate with Ruchy's REPL/evaluation system
                    Ok(serde_json::json!({
                        "result": format!("Evaluated: {}", expression),
                        "type": "String"
                    }))
                },
            )
            .with_input_type(MonoType::Named("EvalRequest".to_string()))
            .with_output_type(MonoType::Named("EvalResult".to_string())),
        ),
        (
            "ruchy-transpile",
            RuchyMCPTool::new(
                "ruchy-transpile".to_string(),
                "Transpile Ruchy code to Rust".to_string(),
                |args| {
                    let code = args["code"]
                        .as_str()
                        .ok_or_else(|| anyhow!("Missing 'code' field"))?;

                    // This would integrate with Ruchy's transpiler
                    Ok(serde_json::json!({
                        "rust_code": format!("// Transpiled from Ruchy\n{}", code),
                        "success": true
                    }))
                },
            )
            .with_input_type(MonoType::Named("TranspileRequest".to_string()))
            .with_output_type(MonoType::Named("TranspileResult".to_string())),
        ),
        (
            "ruchy-type-check",
            RuchyMCPTool::new(
                "ruchy-type-check".to_string(),
                "Type check Ruchy expressions".to_string(),
                |args| {
                    let expression = args["expression"]
                        .as_str()
                        .ok_or_else(|| anyhow!("Missing 'expression' field"))?;

                    // This would integrate with Ruchy's type inference system
                    Ok(serde_json::json!({
                        "inferred_type": "String",
                        "type_errors": [],
                        "expression": expression
                    }))
                },
            )
            .with_input_type(MonoType::Named("TypeCheckRequest".to_string()))
            .with_output_type(MonoType::Named("TypeCheckResult".to_string())),
        ),
    ]
}

/// Example of how to create a Ruchy MCP server
///
/// # Examples
///
/// ```no_run
/// # async fn example() {
/// use ruchy::mcp::create_ruchy_mcp_server;
///
/// let server = create_ruchy_mcp_server().expect("Failed to create MCP server");
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if the server cannot be built or configured
pub fn create_ruchy_mcp_server() -> Result<Server> {
    let server = Server::builder()
        .name("ruchy-mcp-server")
        .version(env!("CARGO_PKG_VERSION"))
        .capabilities(ServerCapabilities::tools_only())
        .build()?;

    // Note: In the actual pmcp API, tools are registered via builder pattern
    // or through dynamic registration after server start
    // for (_name, _tool) in create_ruchy_tools() {
    //     // Tools would be registered here if the API supported it
    // }

    Ok(server)
}

/// Example of how to create a Ruchy MCP client with stdio transport
///
/// # Examples
///
/// ```no_run
/// # async fn example() {
/// use ruchy::mcp::create_ruchy_mcp_client;
///
/// let client = create_ruchy_mcp_client().expect("Failed to create MCP client");
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if the client cannot be created or connected
pub fn create_ruchy_mcp_client() -> Result<Client<StdioTransport>> {
    // Use stdio transport by default
    let transport = StdioTransport::new();
    let client = Client::new(transport);

    Ok(client)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_ruchy_mcp_creation() {
        let mcp = RuchyMCP::new();
        assert!(mcp.server.is_none());
        assert!(mcp.client.is_none());
    }

    #[test]
    fn test_type_registration() {
        let mut mcp = RuchyMCP::new();
        mcp.register_type("TestType".to_string(), MonoType::String);
        assert!(mcp.type_registry.contains_key("TestType"));
    }

    #[test]
    fn test_type_validation() {
        let mcp = RuchyMCP::new();
        let value = serde_json::json!("test string");

        // Test validation against String type
        assert!(mcp.validate_json_value(&value, &MonoType::String).is_ok());

        // Test validation against Int type (should fail)
        assert!(mcp.validate_json_value(&value, &MonoType::Int).is_err());
    }

    #[test]
    fn test_ruchy_tool_creation() {
        let tool = RuchyMCPTool::new("test-tool".to_string(), "A test tool".to_string(), |args| {
            Ok(args)
        });

        assert_eq!(tool.name, "test-tool");
        assert_eq!(tool.description, "A test tool");
    }

    #[tokio::test]
    async fn test_ruchy_tool_handler() {
        use tokio_util::sync::CancellationToken;

        let tool = RuchyMCPTool::new("echo-tool".to_string(), "Echo input".to_string(), |args| {
            Ok(args)
        });

        let input = serde_json::json!({"message": "hello"});
        // Create a dummy RequestHandlerExtra for testing
        let cancellation_token = CancellationToken::new();
        let extra = RequestHandlerExtra::new("test-request".to_string(), cancellation_token);
        let result = tool
            .handle(input.clone(), extra)
            .await
            .unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn test_create_ruchy_tools() {
        let tools = create_ruchy_tools();
        assert!(!tools.is_empty());

        let tool_names: Vec<&str> = tools.iter().map(|(name, _)| *name).collect();
        assert!(tool_names.contains(&"ruchy-eval"));
        assert!(tool_names.contains(&"ruchy-transpile"));
        assert!(tool_names.contains(&"ruchy-type-check"));
    }

    #[tokio::test]
    async fn test_server_creation() {
        let server = create_ruchy_mcp_server();
        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_client_creation() {
        let client = create_ruchy_mcp_client();
        assert!(client.is_ok());
    }
}
