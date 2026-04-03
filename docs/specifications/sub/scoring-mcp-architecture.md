# Sub-spec: Ruchy Scoring — MCP Integration and Core Architecture

**Parent:** [ruchy-scoring-spec.md](../ruchy-scoring-spec.md) Sections 1-5

---

## MCP Server Integration via PMCP

Leverage the existing PMCP SDK (Pragmatic Model Context Protocol) for protocol-compliant tool exposure.

### Core Architecture

```rust
use pmcp::{Server, Tool, ToolBuilder, transport::stdio::StdioTransport};
use pmcp::types::{ToolDefinition, ToolResult};

pub struct RuchyMCPServer {
    server: pmcp::Server,
    scorer: UnifiedScorer,
    debugger: RuchyDebugger,
    observatory: ActorObservatory,
    prover: InteractiveProver,
}

impl RuchyMCPServer {
    pub async fn start() -> Result<()> {
        let server = Server::builder()
            .transport(StdioTransport::new())
            .tool(Self::score_tool())
            .tool(Self::debug_tool())
            .tool(Self::observe_tool())
            .tool(Self::prove_tool())
            .tool(Self::optimize_tool())
            .tool(Self::dataflow_tool())
            .build()?;
            
        server.run().await
    }
}
```

### Tool Definitions

Each Ruchy tool maps to PMCP tool definition.

```rust
impl RuchyMCPServer {
    fn score_tool() -> impl Tool {
        ToolBuilder::new("ruchy_score")
            .description("Calculate unified quality score for Ruchy code")
            .input_schema(json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "depth": { 
                        "type": "string", 
                        "enum": ["shallow", "standard", "deep"] 
                    },
                    "config": { "type": "object" }
                },
                "required": ["path"]
            }))
            .handler(|params| async move {
                let path = params["path"].as_str().unwrap();
                let depth = params.get("depth")
                    .and_then(|d| d.as_str())
                    .and_then(|d| d.parse().ok())
                    .unwrap_or(AnalysisDepth::Standard);
                
                let scorer = UnifiedScorer::new();
                let score = scorer.score(path, depth)?;
                
                Ok(json!({
                    "score": score.value,
                    "grade": score.grade.to_string(),
                    "components": score.components,
                    "confidence": score.confidence,
                    "recommendations": score.generate_recommendations()
                }))
            })
            .build()
    }
    
    fn debug_tool() -> impl Tool {
        ToolBuilder::new("ruchy_debug")
            .description("Interactive debugging session control")
            .input_schema(json!({
                "type": "object",
                "properties": {
                    "action": { 
                        "type": "string",
                        "enum": ["start", "step", "continue", "break", "inspect", "eval"]
                    },
                    "target": { "type": "string" },
                    "expression": { "type": "string" },
                    "session_id": { "type": "string" }
                },
                "required": ["action"]
            }))
            .handler(|params| async move {
                let action = params["action"].as_str().unwrap();
                let session_id = params.get("session_id")
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| Uuid::new_v4().to_string());
                
                let debugger = DEBUGGER_SESSIONS.get_or_create(&session_id)?;
                
                match action {
                    "start" => {
                        let target = params["target"].as_str().unwrap();
                        debugger.start(target)?;
                        Ok(json!({ "session_id": session_id, "status": "started" }))
                    }
                    "step" => {
                        let frame = debugger.step()?;
                        Ok(json!({ 
                            "location": frame.location,
                            "locals": frame.locals,
                            "stack_depth": frame.depth
                        }))
                    }
                    "eval" => {
                        let expr = params["expression"].as_str().unwrap();
                        let value = debugger.eval(expr)?;
                        Ok(json!({ "result": value }))
                    }
                    _ => unimplemented!()
                }
            })
            .build()
    }
    
    fn observe_tool() -> impl Tool {
        ToolBuilder::new("ruchy_actor_observe")
            .description("Real-time actor system observation")
            .input_schema(json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "enum": ["list", "inspect", "trace", "metrics"]
                    },
                    "actor_id": { "type": "string" },
                    "filter": { "type": "object" }
                },
                "required": ["query"]
            }))
            .handler(|params| async move {
                let observatory = ACTOR_OBSERVATORY.get()?;
                let query = params["query"].as_str().unwrap();
                
                match query {
                    "list" => {
                        let actors = observatory.list_actors();
                        Ok(json!({ "actors": actors }))
                    }
                    "inspect" => {
                        let actor_id = params["actor_id"].as_str().unwrap();
                        let snapshot = observatory.pause_actor(actor_id.parse()?);
                        Ok(json!({
                            "state": snapshot.state,
                            "mailbox_depth": snapshot.mailbox.len(),
                            "reductions": snapshot.reduction_count
                        }))
                    }
                    "metrics" => {
                        Ok(json!({
                            "total_actors": observatory.actor_count(),
                            "total_messages": observatory.message_throughput(),
                            "deadlock_risk": observatory.detect_potential_deadlocks()
                        }))
                    }
                    _ => unimplemented!()
                }
            })
            .build()
    }
}
```

### Stateful Session Management

Debug sessions require state persistence across MCP calls.

```rust
pub struct DebuggerSessions {
    sessions: DashMap<SessionId, RuchyDebugger>,
    timeout: Duration,
}

impl DebuggerSessions {
    pub fn get_or_create(&self, id: &str) -> Result<&mut RuchyDebugger> {
        self.sessions.entry(id.to_string())
            .or_insert_with(|| RuchyDebugger::new())
            .deref_mut()
    }
    
    pub fn cleanup_expired(&self) {
        let now = Instant::now();
        self.sessions.retain(|_, debugger| {
            now - debugger.last_accessed < self.timeout
        });
    }
}

lazy_static! {
    static ref DEBUGGER_SESSIONS: DebuggerSessions = DebuggerSessions {
        sessions: DashMap::new(),
        timeout: Duration::from_secs(1800), // 30 minute timeout
    };
}
```

### Streaming Support

Actor observation and debug stepping use PMCP's streaming capabilities.

```rust
impl RuchyMCPServer {
    fn trace_stream_tool() -> impl Tool {
        ToolBuilder::new("ruchy_trace_stream")
            .description("Stream actor messages in real-time")
            .streaming(true)
            .handler(|params, stream| async move {
                let filter = MessageFilter::from(&params["filter"]);
                let observatory = ACTOR_OBSERVATORY.get()?;
                
                let mut message_stream = observatory.trace_messages(filter);
                
                while let Some(msg) = message_stream.next().await {
                    stream.send(json!({
                        "timestamp": msg.timestamp,
                        "from": msg.from.to_string(),
                        "to": msg.to.to_string(),
                        "type": msg.type_name(),
                        "payload": msg.payload
                    })).await?;
                }
                
                Ok(())
            })
            .build()
    }
}
```

### Composite Tool Operations

Complex workflows via tool composition.

```rust
impl RuchyMCPServer {
    fn analyze_and_fix_tool() -> impl Tool {
        ToolBuilder::new("ruchy_analyze_and_fix")
            .description("Analyze code quality and apply automatic fixes")
            .handler(|params| async move {
                let path = params["path"].as_str().unwrap();
                
                // Step 1: Deep analysis
                let score = UnifiedScorer::new().score(path, AnalysisDepth::Deep)?;
                
                if score.value >= 0.90 {
                    return Ok(json!({
                        "status": "no_fixes_needed",
                        "score": score.value
                    }));
                }
                
                // Step 2: Identify fixable issues
                let fixes = score.components.iter()
                    .filter(|(_, v)| *v < 0.8)
                    .flat_map(|(component, _)| generate_fixes(component, path))
                    .collect::<Vec<_>>();
                
                // Step 3: Apply fixes
                let applied = fixes.iter()
                    .filter_map(|fix| fix.apply().ok())
                    .count();
                
                // Step 4: Re-score
                let new_score = UnifiedScorer::new().score(path, AnalysisDepth::Standard)?;
                
                Ok(json!({
                    "fixes_applied": applied,
                    "old_score": score.value,
                    "new_score": new_score.value,
                    "improvement": new_score.value - score.value
                }))
            })
            .build()
    }
}
```

### CLI Integration

```bash
# Start MCP server
ruchy mcp-server

# Connect via stdio
echo '{"method": "tools/call", "params": {"name": "ruchy_score", "arguments": {"path": "src/"}}}' | ruchy mcp-server

# Use with Claude Desktop
# Add to claude_desktop_config.json:
{
  "mcpServers": {
    "ruchy": {
      "command": "ruchy",
      "args": ["mcp-server"],
      "env": {}
    }
  }
}
```

### Performance Optimizations

PMCP integration maintains sub-100ms response times via:

1. **Tool result caching**: Memoize expensive computations
2. **Lazy initialization**: Tools initialize on first use
3. **Connection pooling**: Reuse debugger/observatory connections
4. **Batch operations**: Combine multiple tool calls when possible

This MCP server architecture provides:
- **Protocol compliance**: Full MCP 1.0 specification support via PMCP
- **Tool unification**: All Ruchy tools accessible through single protocol
- **Session persistence**: Stateful debugging across multiple calls
- **Stream support**: Real-time actor observation and tracing
- **Zero configuration**: Works with Claude Desktop out-of-box

## MCP Server Integration

The Ruchy MCP server unifies all analysis tools into a coherent protocol-driven architecture.

### Core MCP Server Architecture

```rust
pub struct RuchyMCPServer {
    // Tool registry with automatic discovery
    tools: HashMap<String, Box<dyn MCPTool>>,
    
    // Session management for stateful interactions
    sessions: DashMap<SessionId, ToolSession>,
    
    // Unified telemetry across all tools
    telemetry: TelemetryAggregator,
    
    // Quality proxy for all operations
    quality_gate: QualityGate,
}

#[async_trait]
impl MCPServer for RuchyMCPServer {
    async fn handle_tool_call(&self, call: ToolCall) -> Result<ToolResult> {
        // Quality gate on input
        self.quality_gate.check_input(&call)?;
        
        // Route to appropriate tool
        let tool = self.tools.get(&call.tool_name)
            .ok_or(Error::ToolNotFound)?;
            
        // Execute with telemetry
        let result = self.telemetry.trace(&call.tool_name, async {
            tool.execute(call.params).await
        }).await?;
        
        // Quality gate on output
        self.quality_gate.check_output(&result)?;
        
        Ok(result)
    }
}
```

### Tool Registration

All Ruchy tools expose MCP interfaces automatically.

```rust
#[m# Ruchy Score Command Specification v1.0

## Executive Summary

The `ruchy score` command provides a single, deterministic quality metric (0.0-1.0) for Ruchy code. This unified score enables objective quality gates, MCP integration, and continuous improvement tracking while maintaining computational efficiency through incremental analysis.

## Critical Design Constraints

### Scope Management
The specification acknowledges the "astronomical scope" risk identified in review. Mitigation strategy:
- **Phase 0 enforcement**: No scoring features beyond base metrics until parser/type system complete
- **Tool priority**: Debugger first (4 months), then incremental tool delivery
- **Feature freeze**: Score algorithm locked at v1.0 to prevent scope creep

### Resource Reality
Given the "decades not years" implementation horizon:
- **Incremental value delivery**: Each tool provides standalone utility
- **Progressive enhancement**: Tools function at reduced fidelity initially
- **Expertise scaling**: Core team implements frameworks; community implements analyzers

## Core Architecture

### Score Composition

```rust
pub struct QualityScore {
    value: f64,           // 0.0-1.0 normalized score
    components: Components,
    grade: Grade,         // Human-readable grade
    confidence: f64,      // Confidence in score accuracy
    cache_hit_rate: f64,  // Percentage from cached analysis
}

pub struct Components {
    correctness: f64,     // 35% - Semantic correctness
    performance: f64,     // 25% - Runtime efficiency  
    maintainability: f64, // 20% - Change resilience
    safety: f64,          // 15% - Memory/type safety
    idiomaticity: f64,    // 5%  - Language conventions
}
```

### Incremental Analysis Engine

To address computational cost, the scorer employs three-tier analysis:

```rust
pub enum AnalysisDepth {
    Shallow,   // <100ms - AST metrics only
    Standard,  // <1s - AST + type checking + basic flow
    Deep,      // <30s - Full property/mutation testing
}

impl ScoreEngine {
    pub fn score(&self, path: &Path, depth: AnalysisDepth) -> QualityScore {
        match depth {
            Shallow => self.ast_only_score(path),      // Real-time feedback
            Standard => self.standard_score(path),      // Default CLI
            Deep => self.comprehensive_score(path),     // CI/CD gates
        }
    }
    
    pub fn incremental_score(&self, changes: &FileChanges) -> QualityScore {
        // Recompute only affected modules
        let affected = self.dependency_graph.affected_modules(changes);
        let cached = self.cache.get_unchanged_scores();
        let fresh = affected.par_iter().map(|m| self.score_module(m));
        
        self.merge_scores(cached, fresh)
    }
}
```

