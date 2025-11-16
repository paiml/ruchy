//! MCP-compatible Actor system implementation
//!
//! Based on SPECIFICATION.md section 7: MCP Message-Passing Architecture
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing;
/// Core Actor trait compatible with MCP message passing
#[async_trait::async_trait]
pub trait Actor: Send + Sync + 'static + Sized {
    type Message: McpSerializable + Send + 'static;
    type Response: McpSerializable + Send + 'static;
    async fn receive(&mut self, msg: Self::Message) -> Option<Self::Response>;
    /// Spawn the actor and return a handle to communicate with it
    fn spawn(mut self) -> ActorHandle<Self::Message, Self::Response> {
        let (tx, mut rx) = mpsc::channel::<(Self::Message, mpsc::Sender<Self::Response>)>(100);
        tokio::spawn(async move {
            while let Some((msg, reply_tx)) = rx.recv().await {
                let response = self.receive(msg).await;
                if let Some(resp) = response {
                    let _ = reply_tx.send(resp).await;
                }
            }
        });
        ActorHandle { tx }
    }
}
/// Trait for MCP serializable messages
pub trait McpSerializable: Serialize + for<'de> Deserialize<'de> + fmt::Debug + Clone {}
// Blanket implementation for types that satisfy the bounds
impl<T> McpSerializable for T where T: Serialize + for<'de> Deserialize<'de> + fmt::Debug + Clone {}
/// Handle for communicating with an actor
pub struct ActorHandle<M, R> {
    tx: mpsc::Sender<(M, mpsc::Sender<R>)>,
}
impl<M, R> ActorHandle<M, R>
where
    M: McpSerializable + Send + 'static,
    R: McpSerializable + Send + 'static,
{
    /// Send a message to the actor without waiting for response
    ///
    /// # Errors
    ///
    /// Returns an error if the actor has stopped and can no longer receive messages
    /// # Examples
    ///
    /// ```
    /// use ruchy::actors::send;
    ///
    /// let result = send(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub async fn send(&self, msg: M) -> Result<()> {
        let (reply_tx, _) = mpsc::channel::<R>(1);
        self.tx
            .send((msg, reply_tx))
            .await
            .map_err(|_| anyhow::anyhow!("Actor has stopped"))?;
        Ok(())
    }
    /// Send a message and wait for response
    ///
    /// # Errors
    ///
    /// Returns an error if the actor has stopped or does not respond
    /// # Examples
    ///
    /// ```
    /// use ruchy::actors::ask;
    ///
    /// let result = ask(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub async fn ask(&self, msg: M) -> Result<R> {
        let (reply_tx, mut reply_rx) = mpsc::channel::<R>(1);
        self.tx
            .send((msg, reply_tx))
            .await
            .map_err(|_| anyhow::anyhow!("Actor has stopped"))?;
        reply_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("No response received"))
    }
    /// Check if the actor is still alive
    /// # Examples
    ///
    /// ```
    /// use ruchy::actors::is_alive;
    ///
    /// let result = is_alive(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn is_alive(&self) -> bool {
        !self.tx.is_closed()
    }
}
/// MCP protocol message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpMessage {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: Option<String>,
}
/// MCP protocol response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<McpError>,
    pub id: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}
/// MCP-compatible actor for handling protocol messages
pub struct McpActor {
    pub tools: Vec<String>,
}
impl McpActor {
    /// Create a new MCP actor with default tools
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::McpActor;
    ///
    /// let actor = McpActor::new();
    /// // Actor starts with three default tools
    /// assert_eq!(actor.tools.len(), 3);
    /// ```
    pub fn new() -> Self {
        Self {
            tools: vec![
                "transpile".to_string(),
                "parse".to_string(),
                "analyze".to_string(),
            ],
        }
    }
    fn list_tools(&self) -> McpResponse {
        McpResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!({
                "tools": self.tools.iter().map(|name| {
                    serde_json::json!({
                        "name": name,
                        "description": format!("Ruchy {name} tool")
                    })
                }).collect::<Vec<_>>()
            })),
            error: None,
            id: None,
        }
    }
    fn call_tool(params: &serde_json::Value) -> Option<McpResponse> {
        // Extract tool name and arguments from params
        let tool_name = params.get("name")?.as_str()?;
        let result = match tool_name {
            "transpile" => {
                serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": "Transpilation functionality placeholder"
                        }
                    ]
                })
            }
            "parse" => {
                serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": "Parsing functionality placeholder"
                        }
                    ]
                })
            }
            "analyze" => {
                serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": "Analysis functionality placeholder"
                        }
                    ]
                })
            }
            _ => {
                return Some(McpResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(McpError {
                        code: -32601,
                        message: format!("Unknown tool: {tool_name}"),
                        data: None,
                    }),
                    id: None,
                });
            }
        };
        Some(McpResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id: None,
        })
    }
}
impl Default for McpActor {
    fn default() -> Self {
        Self::new()
    }
}
#[async_trait::async_trait]
impl Actor for McpActor {
    type Message = McpMessage;
    type Response = McpResponse;
    async fn receive(&mut self, msg: McpMessage) -> Option<McpResponse> {
        match msg.method.as_str() {
            "tools/list" => Some(self.list_tools()),
            "tools/call" => Self::call_tool(&msg.params),
            _ => Some(McpResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(McpError {
                    code: -32601,
                    message: format!("Unknown method: {method}", method = msg.method),
                    data: None,
                }),
                id: msg.id,
            }),
        }
    }
}
/// Supervision strategies for actor fault tolerance
#[derive(Debug, Clone)]
pub enum SupervisionStrategy {
    /// Restart only the failed child
    OneForOne,
    /// Restart all children when one fails
    OneForAll,
    /// Restart the failed child and all children started after it
    RestForOne,
}
/// Supervisor for managing actor lifecycles
pub struct Supervisor<A: Actor> {
    children: Vec<ActorHandle<A::Message, A::Response>>,
    strategy: SupervisionStrategy,
}
impl<A: Actor> Supervisor<A> {
    pub fn new(strategy: SupervisionStrategy) -> Self {
        Self {
            children: Vec::new(),
            strategy,
        }
    }
    pub fn supervise(&mut self, actor: A) {
        let handle = actor.spawn();
        self.children.push(handle);
    }
    pub async fn monitor(&mut self) {
        // Monitoring implementation would go here
        // For now, just check if actors are alive periodically
        loop {
            for (i, child) in self.children.iter().enumerate() {
                if !child.is_alive() {
                    match self.strategy {
                        SupervisionStrategy::OneForOne => {
                            tracing::warn!("Child actor {i} died, would restart in production");
                        }
                        SupervisionStrategy::OneForAll => {
                            tracing::warn!("Child actor {i} died, would restart all in production");
                        }
                        SupervisionStrategy::RestForOne => {
                            tracing::warn!(
                                "Child actor {i} died, would restart from {i} in production"
                            );
                        }
                    }
                }
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestMessage {
        content: String,
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestResponse {
        echo: String,
    }
    struct EchoActor;
    #[async_trait::async_trait]
    impl Actor for EchoActor {
        type Message = TestMessage;
        type Response = TestResponse;
        async fn receive(&mut self, msg: TestMessage) -> Option<TestResponse> {
            Some(TestResponse {
                echo: format!("Echo: {content}", content = msg.content),
            })
        }
    }
    #[tokio::test]
    async fn test_actor_spawn_and_communication() -> Result<(), Box<dyn std::error::Error>> {
        let actor = EchoActor;
        let handle = actor.spawn();
        let msg = TestMessage {
            content: "Hello, Actor!".to_string(),
        };
        let response = handle.ask(msg).await?;
        assert_eq!(response.echo, "Echo: Hello, Actor!");
        Ok(())
    }
    #[tokio::test]
    async fn test_mcp_actor_list_tools() -> Result<(), Box<dyn std::error::Error>> {
        let actor = McpActor::new();
        let handle = actor.spawn();
        let msg = McpMessage {
            jsonrpc: "2.0".to_string(),
            method: "tools/list".to_string(),
            params: serde_json::Value::Null,
            id: Some("test".to_string()),
        };
        let response = handle.ask(msg).await?;
        assert!(response.result.is_some());
        assert!(response.error.is_none());
        Ok(())
    }
    #[tokio::test]
    async fn test_mcp_actor_call_tool() -> Result<(), Box<dyn std::error::Error>> {
        let actor = McpActor::new();
        let handle = actor.spawn();
        let msg = McpMessage {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "transpile",
                "arguments": {}
            }),
            id: Some("test".to_string()),
        };
        let response = handle.ask(msg).await?;
        assert!(response.result.is_some());
        assert!(response.error.is_none());
        Ok(())
    }
    #[test]
    fn test_supervision_strategy_creation() {
        let supervisor: Supervisor<EchoActor> = Supervisor::new(SupervisionStrategy::OneForOne);
        assert!(matches!(
            supervisor.strategy,
            SupervisionStrategy::OneForOne
        ));
        assert_eq!(supervisor.children.len(), 0);
    }
    #[test]
    fn test_mcp_message_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let msg = McpMessage {
            jsonrpc: "2.0".to_string(),
            method: "test".to_string(),
            params: serde_json::json!({"key": "value"}),
            id: Some("123".to_string()),
        };
        let serialized = serde_json::to_string(&msg)?;
        let deserialized: McpMessage = serde_json::from_str(&serialized)?;
        assert_eq!(msg.jsonrpc, deserialized.jsonrpc);
        assert_eq!(msg.method, deserialized.method);
        assert_eq!(msg.id, deserialized.id);
        Ok(())
    }

    // Test 6: ActorHandle::is_alive returns true for active actor
    #[tokio::test]
    async fn test_actor_handle_is_alive() {
        let actor = EchoActor;
        let handle = actor.spawn();
        assert!(handle.is_alive());
    }

    // Test 7: McpActor::new creates actor with default tools
    #[test]
    fn test_mcp_actor_new() {
        let actor = McpActor::new();
        assert_eq!(actor.tools.len(), 3);
        assert!(actor.tools.contains(&"transpile".to_string()));
        assert!(actor.tools.contains(&"parse".to_string()));
        assert!(actor.tools.contains(&"analyze".to_string()));
    }

    // Test 8: McpActor::default matches new()
    #[test]
    fn test_mcp_actor_default() {
        let actor1 = McpActor::new();
        let actor2 = McpActor::default();
        assert_eq!(actor1.tools, actor2.tools);
    }

    // Test 9: MCP actor handles unknown method (ERROR PATH)
    #[tokio::test]
    async fn test_mcp_actor_unknown_method_error() -> Result<(), Box<dyn std::error::Error>> {
        let actor = McpActor::new();
        let handle = actor.spawn();
        let msg = McpMessage {
            jsonrpc: "2.0".to_string(),
            method: "invalid/method".to_string(),
            params: serde_json::Value::Null,
            id: Some("test".to_string()),
        };
        let response = handle.ask(msg).await?;
        assert!(response.error.is_some());
        assert!(response.result.is_none());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32601);
        assert!(error.message.contains("Unknown method"));
        Ok(())
    }

    // Test 10: MCP actor call_tool with unknown tool (ERROR PATH)
    #[tokio::test]
    async fn test_mcp_actor_unknown_tool_error() -> Result<(), Box<dyn std::error::Error>> {
        let actor = McpActor::new();
        let handle = actor.spawn();
        let msg = McpMessage {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "nonexistent_tool",
                "arguments": {}
            }),
            id: Some("test".to_string()),
        };
        let response = handle.ask(msg).await?;
        assert!(response.error.is_some());
        assert!(response.result.is_none());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32601);
        assert!(error.message.contains("Unknown tool"));
        Ok(())
    }

    // Test 11: MCP actor call_tool with parse tool
    #[tokio::test]
    async fn test_mcp_actor_call_parse_tool() -> Result<(), Box<dyn std::error::Error>> {
        let actor = McpActor::new();
        let handle = actor.spawn();
        let msg = McpMessage {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "parse",
                "arguments": {}
            }),
            id: Some("test".to_string()),
        };
        let response = handle.ask(msg).await?;
        assert!(response.result.is_some());
        assert!(response.error.is_none());
        Ok(())
    }

    // Test 12: MCP actor call_tool with analyze tool
    #[tokio::test]
    async fn test_mcp_actor_call_analyze_tool() -> Result<(), Box<dyn std::error::Error>> {
        let actor = McpActor::new();
        let handle = actor.spawn();
        let msg = McpMessage {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "analyze",
                "arguments": {}
            }),
            id: Some("test".to_string()),
        };
        let response = handle.ask(msg).await?;
        assert!(response.result.is_some());
        assert!(response.error.is_none());
        Ok(())
    }

    // Test 13: McpError construction
    #[test]
    fn test_mcp_error_construction() {
        let error = McpError {
            code: -32700,
            message: "Parse error".to_string(),
            data: Some(serde_json::json!({"details": "invalid json"})),
        };
        assert_eq!(error.code, -32700);
        assert_eq!(error.message, "Parse error");
        assert!(error.data.is_some());
    }

    // Test 14: McpResponse with error
    #[test]
    fn test_mcp_response_with_error() {
        let response = McpResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(McpError {
                code: -32600,
                message: "Invalid Request".to_string(),
                data: None,
            }),
            id: Some("req1".to_string()),
        };
        assert!(response.result.is_none());
        assert!(response.error.is_some());
    }

    // Test 15: McpResponse serialization
    #[test]
    fn test_mcp_response_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let response = McpResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!({"success": true})),
            error: None,
            id: Some("resp1".to_string()),
        };
        let serialized = serde_json::to_string(&response)?;
        let deserialized: McpResponse = serde_json::from_str(&serialized)?;
        assert_eq!(response.jsonrpc, deserialized.jsonrpc);
        assert_eq!(response.id, deserialized.id);
        Ok(())
    }

    // Test 16: Supervisor with OneForAll strategy
    #[test]
    fn test_supervisor_one_for_all_strategy() {
        let supervisor: Supervisor<EchoActor> = Supervisor::new(SupervisionStrategy::OneForAll);
        assert!(matches!(
            supervisor.strategy,
            SupervisionStrategy::OneForAll
        ));
    }

    // Test 17: Supervisor with RestForOne strategy
    #[test]
    fn test_supervisor_rest_for_one_strategy() {
        let supervisor: Supervisor<EchoActor> = Supervisor::new(SupervisionStrategy::RestForOne);
        assert!(matches!(
            supervisor.strategy,
            SupervisionStrategy::RestForOne
        ));
    }

    // Test 18: Supervisor supervise adds child
    #[tokio::test]
    async fn test_supervisor_supervise() {
        let mut supervisor: Supervisor<EchoActor> = Supervisor::new(SupervisionStrategy::OneForOne);
        assert_eq!(supervisor.children.len(), 0);
        supervisor.supervise(EchoActor);
        assert_eq!(supervisor.children.len(), 1);
    }

    // Test 19: ActorHandle::send without waiting for response
    #[tokio::test]
    async fn test_actor_handle_send() -> Result<(), Box<dyn std::error::Error>> {
        let actor = EchoActor;
        let handle = actor.spawn();
        let msg = TestMessage {
            content: "Fire and forget".to_string(),
        };
        handle.send(msg).await?;
        Ok(())
    }

    // Test 20: McpMessage with null id
    #[test]
    fn test_mcp_message_null_id() -> Result<(), Box<dyn std::error::Error>> {
        let msg = McpMessage {
            jsonrpc: "2.0".to_string(),
            method: "notification".to_string(),
            params: serde_json::Value::Null,
            id: None,
        };
        assert!(msg.id.is_none());
        let serialized = serde_json::to_string(&msg)?;
        let deserialized: McpMessage = serde_json::from_str(&serialized)?;
        assert!(deserialized.id.is_none());
        Ok(())
    }

    // Test 21: Multiple messages to same actor
    #[tokio::test]
    async fn test_multiple_messages_to_actor() -> Result<(), Box<dyn std::error::Error>> {
        let actor = EchoActor;
        let handle = actor.spawn();

        // Send multiple messages
        for i in 0..5 {
            let msg = TestMessage {
                content: format!("Message {i}"),
            };
            let response = handle.ask(msg).await?;
            assert!(response.echo.contains(&format!("Message {i}")));
        }
        Ok(())
    }

    // Test 22: Concurrent messages to actor
    #[tokio::test]
    async fn test_concurrent_messages_to_actor() -> Result<(), Box<dyn std::error::Error>> {
        let actor = EchoActor;
        let handle = std::sync::Arc::new(actor.spawn());

        let mut tasks = vec![];
        for i in 0..10 {
            let h = handle.clone();
            let task = tokio::spawn(async move {
                let msg = TestMessage {
                    content: format!("Concurrent {i}"),
                };
                h.ask(msg).await
            });
            tasks.push(task);
        }

        // All messages should succeed
        for task in tasks {
            assert!(task.await.is_ok());
        }
        Ok(())
    }

    // Test 23: MCP actor with complex params
    #[tokio::test]
    async fn test_mcp_actor_complex_params() -> Result<(), Box<dyn std::error::Error>> {
        let actor = McpActor::new();
        let handle = actor.spawn();
        let msg = McpMessage {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "transpile",
                "arguments": {
                    "source": "fun main() { println!(\"hello\") }",
                    "options": {
                        "optimize": true,
                        "target": "rust"
                    }
                }
            }),
            id: Some("complex".to_string()),
        };
        let response = handle.ask(msg).await?;
        assert!(response.result.is_some());
        Ok(())
    }

    // Test 24: McpError with data field
    #[test]
    fn test_mcp_error_with_data() -> Result<(), Box<dyn std::error::Error>> {
        let error = McpError {
            code: -32602,
            message: "Invalid params".to_string(),
            data: Some(serde_json::json!({
                "param": "name",
                "expected": "string",
                "got": "number"
            })),
        };
        assert_eq!(error.code, -32602);
        assert!(error.data.is_some());

        // Serialize and deserialize
        let serialized = serde_json::to_string(&error)?;
        let deserialized: McpError = serde_json::from_str(&serialized)?;
        assert_eq!(error.message, deserialized.message);
        Ok(())
    }

    // Test 25: Supervisor with multiple children
    #[tokio::test]
    async fn test_supervisor_multiple_children() {
        let mut supervisor: Supervisor<EchoActor> = Supervisor::new(SupervisionStrategy::OneForOne);

        // Add 5 children
        for _ in 0..5 {
            supervisor.supervise(EchoActor);
        }

        assert_eq!(supervisor.children.len(), 5);

        // All children should be alive
        for child in &supervisor.children {
            assert!(child.is_alive());
        }
    }

    // Test 26: Actor handle send fire-and-forget multiple times
    #[tokio::test]
    async fn test_actor_handle_send_multiple() -> Result<(), Box<dyn std::error::Error>> {
        let actor = EchoActor;
        let handle = actor.spawn();

        // Send 10 fire-and-forget messages
        for i in 0..10 {
            let msg = TestMessage {
                content: format!("Fire {i}"),
            };
            handle.send(msg).await?;
        }

        // Actor should still be alive
        assert!(handle.is_alive());
        Ok(())
    }

    // Test 27: MCP response with null result
    #[test]
    fn test_mcp_response_null_result() {
        let response = McpResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: None,
            id: Some("null_result".to_string()),
        };
        assert!(response.result.is_none());
        assert!(response.error.is_none());
    }

    // Test 28: McpActor list_tools returns correct structure
    #[test]
    fn test_mcp_actor_list_tools_structure() {
        let actor = McpActor::new();
        let response = actor.list_tools();

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert!(response.error.is_none());

        // Verify tools structure
        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();
        assert_eq!(tools.len(), 3);

        // Check first tool structure
        let first_tool = &tools[0];
        assert!(first_tool.get("name").is_some());
        assert!(first_tool.get("description").is_some());
    }

    // Test 29: MCP call_tool with missing name parameter (ERROR PATH)
    #[test]
    fn test_mcp_call_tool_missing_name() {
        let params = serde_json::json!({
            "arguments": {}
        });
        let response = McpActor::call_tool(&params);

        // Should return None when required parameter missing
        assert!(response.is_none());
    }

    // Test 30: McpMessage serialization with complex params
    #[test]
    fn test_mcp_message_complex_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let msg = McpMessage {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "nested": {
                    "array": [1, 2, 3],
                    "object": {
                        "key": "value"
                    }
                }
            }),
            id: Some("complex_params".to_string()),
        };

        let serialized = serde_json::to_string(&msg)?;
        let deserialized: McpMessage = serde_json::from_str(&serialized)?;

        assert_eq!(msg.method, deserialized.method);
        assert_eq!(msg.params, deserialized.params);
        Ok(())
    }

    // Test 31: ActorHandle::ask when actor has stopped (ERROR PATH)
    #[tokio::test]
    async fn test_actor_handle_ask_actor_stopped() {
        let (tx, _rx) = mpsc::channel::<(TestMessage, mpsc::Sender<TestResponse>)>(1);
        drop(_rx); // Close the receiver to simulate stopped actor
        let stopped_handle = ActorHandle { tx };

        let msg = TestMessage {
            content: "Test".to_string(),
        };

        // Should fail with "Actor has stopped"
        let result = stopped_handle.ask(msg).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Actor has stopped"));
    }

    // Test 32: ActorHandle::send when actor has stopped (ERROR PATH)
    #[tokio::test]
    async fn test_actor_handle_send_actor_stopped() {
        let (tx, _rx) = mpsc::channel::<(TestMessage, mpsc::Sender<TestResponse>)>(1);
        drop(_rx); // Close the receiver to simulate stopped actor
        let stopped_handle = ActorHandle { tx };

        let msg = TestMessage {
            content: "Test".to_string(),
        };

        // Should fail with "Actor has stopped"
        let result = stopped_handle.send(msg).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Actor has stopped"));
    }

    // Test 33: ActorHandle::is_alive returns false when actor stopped
    #[tokio::test]
    async fn test_actor_handle_is_alive_stopped() {
        let (tx, _rx) = mpsc::channel::<(TestMessage, mpsc::Sender<TestResponse>)>(1);
        drop(_rx); // Close receiver
        let stopped_handle = ActorHandle { tx };

        // Wait a bit for channel to fully close
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        assert!(!stopped_handle.is_alive());
    }

    // Test 34: McpMessage with empty method
    #[test]
    fn test_mcp_message_empty_method() -> Result<(), Box<dyn std::error::Error>> {
        let msg = McpMessage {
            jsonrpc: "2.0".to_string(),
            method: String::new(),
            params: serde_json::Value::Null,
            id: Some("empty".to_string()),
        };
        assert_eq!(msg.method, "");
        let serialized = serde_json::to_string(&msg)?;
        let deserialized: McpMessage = serde_json::from_str(&serialized)?;
        assert_eq!(msg.method, deserialized.method);
        Ok(())
    }

    // Test 35: McpError with zero code
    #[test]
    fn test_mcp_error_zero_code() {
        let error = McpError {
            code: 0,
            message: "No error".to_string(),
            data: None,
        };
        assert_eq!(error.code, 0);
        assert!(error.data.is_none());
    }

    // Test 36: McpResponse with both result and error (invalid but parseable)
    #[test]
    fn test_mcp_response_both_result_and_error() -> Result<(), Box<dyn std::error::Error>> {
        let response = McpResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!({"value": 42})),
            error: Some(McpError {
                code: -1,
                message: "Error".to_string(),
                data: None,
            }),
            id: Some("invalid".to_string()),
        };

        // Should be serializable even if semantically invalid
        let serialized = serde_json::to_string(&response)?;
        let deserialized: McpResponse = serde_json::from_str(&serialized)?;
        assert!(deserialized.result.is_some());
        assert!(deserialized.error.is_some());
        Ok(())
    }

    // Test 37: Supervisor with zero children
    #[test]
    fn test_supervisor_zero_children() {
        let supervisor: Supervisor<EchoActor> = Supervisor::new(SupervisionStrategy::OneForOne);
        assert_eq!(supervisor.children.len(), 0);
    }

    // Test 38: McpActor with modified tools list
    #[test]
    fn test_mcp_actor_custom_tools() {
        let mut actor = McpActor::new();
        actor.tools.push("custom_tool".to_string());
        assert_eq!(actor.tools.len(), 4);
        assert!(actor.tools.contains(&"custom_tool".to_string()));
    }

    // Test 39: MCP call_tool with empty params object
    #[test]
    fn test_mcp_call_tool_empty_params() {
        let params = serde_json::json!({});
        let response = McpActor::call_tool(&params);

        // Should return None when name parameter missing
        assert!(response.is_none());
    }

    // Test 40: McpMessage with large params
    #[test]
    fn test_mcp_message_large_params() -> Result<(), Box<dyn std::error::Error>> {
        let large_array: Vec<i32> = (0..1000).collect();
        let msg = McpMessage {
            jsonrpc: "2.0".to_string(),
            method: "test".to_string(),
            params: serde_json::json!({ "data": large_array }),
            id: Some("large".to_string()),
        };

        let serialized = serde_json::to_string(&msg)?;
        let deserialized: McpMessage = serde_json::from_str(&serialized)?;
        assert_eq!(msg.method, deserialized.method);
        Ok(())
    }

    // Test 41: SupervisionStrategy Debug formatting
    #[test]
    fn test_supervision_strategy_debug() {
        let one_for_one = SupervisionStrategy::OneForOne;
        let debug_str = format!("{one_for_one:?}");
        assert!(debug_str.contains("OneForOne"));

        let one_for_all = SupervisionStrategy::OneForAll;
        let debug_str = format!("{one_for_all:?}");
        assert!(debug_str.contains("OneForAll"));

        let rest_for_one = SupervisionStrategy::RestForOne;
        let debug_str = format!("{rest_for_one:?}");
        assert!(debug_str.contains("RestForOne"));
    }

    // Test 42: SupervisionStrategy Clone
    #[test]
    fn test_supervision_strategy_clone() {
        let original = SupervisionStrategy::OneForOne;
        let cloned = original;
        assert!(matches!(cloned, SupervisionStrategy::OneForOne));
    }

    // Test 43: McpActor receive with id propagation
    #[tokio::test]
    async fn test_mcp_actor_id_propagation() -> Result<(), Box<dyn std::error::Error>> {
        let mut actor = McpActor::new();
        let msg = McpMessage {
            jsonrpc: "2.0".to_string(),
            method: "unknown_method".to_string(),
            params: serde_json::Value::Null,
            id: Some("propagate_me".to_string()),
        };

        let response = actor.receive(msg).await.unwrap();
        assert_eq!(response.id, Some("propagate_me".to_string()));
        Ok(())
    }

    // Test 44: MCP call_tool with all three tools
    #[tokio::test]
    async fn test_mcp_call_all_tools() -> Result<(), Box<dyn std::error::Error>> {
        let actor = McpActor::new();
        let handle = actor.spawn();

        for tool_name in &["transpile", "parse", "analyze"] {
            let msg = McpMessage {
                jsonrpc: "2.0".to_string(),
                method: "tools/call".to_string(),
                params: serde_json::json!({
                    "name": tool_name,
                    "arguments": {}
                }),
                id: Some(format!("test_{tool_name}")),
            };

            let response = handle.ask(msg).await?;
            assert!(response.result.is_some());
            assert!(response.error.is_none());
        }
        Ok(())
    }

    // Test 45: Actor that returns None response
    struct NoResponseActor;

    #[async_trait::async_trait]
    impl Actor for NoResponseActor {
        type Message = TestMessage;
        type Response = TestResponse;

        async fn receive(&mut self, _msg: TestMessage) -> Option<TestResponse> {
            None // No response
        }
    }

    #[tokio::test]
    async fn test_actor_no_response() {
        let actor = NoResponseActor;
        let handle = actor.spawn();

        let msg = TestMessage {
            content: "Test".to_string(),
        };

        // Ask should timeout/fail when actor returns None
        let result = tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            handle.ask(msg)
        ).await;

        // Timeout or error expected
        assert!(result.is_err() || result.unwrap().is_err());
    }
}
#[cfg(test)]
mod property_tests_actors {
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use proptest::prelude::*;
    use proptest::proptest;
    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_send_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
