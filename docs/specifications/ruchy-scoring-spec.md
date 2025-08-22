This completes the developer experience toolchain, providing the essential capability of understanding program behavior through controlled execution.

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

## Metric Definitions

### Correctness (35%)

```rust
fn score_correctness(code: &Analysis) -> f64 {
    let mut score = 1.0;
    
    // Property test coverage (10% of correctness)
    let property_coverage = code.property_tests.coverage_ratio();
    score *= 0.9 + (property_coverage * 0.1);
    
    // Refinement type proofs (10%)
    let proof_ratio = code.proven_refinements / code.total_refinements.max(1);
    score *= 0.9 + (proof_ratio * 0.1);
    
    // Pattern match exhaustiveness (5%)
    let exhaustive_ratio = code.exhaustive_matches / code.total_matches.max(1);
    score *= 0.95 + (exhaustive_ratio * 0.05);
    
    // Mutation score (10%)
    if let Some(mutation_score) = code.mutation_score {
        score *= 0.9 + (mutation_score * 0.1);
    }
    
    score
}
```

### Performance (25%)

```rust
fn score_performance(code: &Analysis) -> f64 {
    // Allocation efficiency
    let alloc_score = 1.0 - (code.heap_allocs_per_op().min(0.3));
    
    // Complexity bounds
    let complexity_score = match code.max_cyclomatic {
        0..=10 => 1.0,
        11..=20 => 0.9 - ((code.max_cyclomatic - 10) as f64 * 0.01),
        _ => 0.7,
    };
    
    // Performance predictability
    let variance_score = 1.0 - (code.perf_coefficient_of_variation().min(0.3));
    
    (alloc_score * 0.4 + complexity_score * 0.4 + variance_score * 0.2)
}
```

### Maintainability (20%)

```rust
fn score_maintainability(code: &Analysis) -> f64 {
    // Afferent/efferent coupling
    let coupling_score = 1.0 / (1.0 + code.coupling_factor);
    
    // Module cohesion (LCOM4)
    let cohesion_score = code.average_cohesion;
    
    // Change amplification factor
    let amplification_score = 1.0 / code.change_amplification.max(1.0);
    
    (coupling_score * 0.3 + cohesion_score * 0.4 + amplification_score * 0.3)
}
```

### Safety (15%)

```rust
fn score_safety(code: &Analysis) -> f64 {
    let mut score = 1.0;
    
    // Unsafe block density
    let unsafe_density = code.unsafe_blocks as f64 / code.total_blocks as f64;
    score -= unsafe_density.min(0.5);
    
    // Error handling quality
    let unwrap_density = code.unwrap_count as f64 / code.result_types as f64;
    score -= (unwrap_density * 0.3).min(0.3);
    
    // Lifetime correctness proofs
    score += (code.lifetime_proofs as f64 * 0.01).min(0.2);
    
    score.max(0.0)
}
```

### Idiomaticity (5%)

```rust
fn score_idiomaticity(code: &Analysis) -> f64 {
    let iterator_score = code.iterator_usage_ratio();
    let error_score = 1.0 - code.panic_path_ratio();
    let pattern_score = code.pattern_match_ratio();
    
    (iterator_score * 0.3 + error_score * 0.5 + pattern_score * 0.2)
}
```

## Addressing Implementation Challenges

### 1. Mitigating "Garbage In, Garbage Out"

```rust
impl QualityScore {
    pub fn with_confidence(&self) -> (f64, f64) {
        // Return score with confidence interval
        let confidence = self.calculate_confidence();
        (self.value, confidence)
    }
    
    fn calculate_confidence(&self) -> f64 {
        // Confidence based on analysis completeness
        let coverage_confidence = self.test_coverage_known as u8 as f64;
        let static_confidence = self.static_analysis_depth as f64 / 3.0;
        let sample_confidence = (self.samples_analyzed as f64 / 1000.0).min(1.0);
        
        (coverage_confidence * 0.4 + static_confidence * 0.4 + sample_confidence * 0.2)
    }
}
```

### 2. Preventing Gaming

```toml
# .ruchy/score.toml - Per-project configuration
[scoring]
# Teams can adjust weights within bounds
correctness_weight = 0.35    # (0.25-0.45)
performance_weight = 0.25     # (0.15-0.35)
maintainability_weight = 0.20 # (0.10-0.30)
safety_weight = 0.15          # (0.10-0.25)
idiomaticity_weight = 0.05    # (0.00-0.15)

# Anti-gaming rules
[scoring.rules]
min_property_tests_per_function = 3
max_trivial_test_ratio = 0.2
min_mutation_kill_ratio = 0.75
```

### 3. Performance Optimization

```rust
pub struct IncrementalScorer {
    ast_cache: DashMap<PathBuf, AstAnalysis>,
    type_cache: DashMap<ModuleId, TypeAnalysis>,
    score_cache: LruCache<FileHash, ComponentScores>,
    
    // Dependency tracking for incremental updates
    dep_graph: DependencyGraph,
    
    // Background analysis thread pool
    analyzer_pool: ThreadPool,
}

impl IncrementalScorer {
    pub fn watch_mode(&self, path: &Path) -> impl Stream<Item = QualityScore> {
        // Return score stream with progressive refinement
        stream::unfold(AnalysisDepth::Shallow, |depth| async {
            let score = self.score(path, depth);
            let next_depth = depth.next().filter(|_| score.confidence < 0.95);
            Some((score, next_depth?))
        })
    }
}
```

### 4. Dimension Preservation

```rust
impl QualityScore {
    pub fn explain_delta(&self, previous: &QualityScore) -> Explanation {
        let mut explanation = Explanation::new();
        
        for (component, weight) in self.components.iter() {
            let delta = component.value - previous.get(component.name);
            if delta.abs() > 0.01 {
                explanation.add_change(component.name, delta, weight);
            }
        }
        
        explanation.add_tradeoffs(self.identify_tradeoffs(previous));
        explanation
    }
    
    fn identify_tradeoffs(&self, previous: &QualityScore) -> Vec<Tradeoff> {
        // Detect when improvements in one dimension hurt another
        let mut tradeoffs = vec![];
        
        if self.components.performance > previous.components.performance &&
           self.components.maintainability < previous.components.maintainability {
            tradeoffs.push(Tradeoff::PerformanceVsMaintainability {
                perf_gain: self.components.performance - previous.components.performance,
                maint_loss: previous.components.maintainability - self.components.maintainability,
            });
        }
        
        tradeoffs
    }
}
```

## CLI Interface

```bash
# Fast feedback (AST-only, <100ms)
ruchy score src/main.ruchy --fast

# Standard analysis (default, <1s)
ruchy score src/

# Deep analysis for CI (complete, <30s)
ruchy score src/ --deep

# Watch mode with progressive refinement
ruchy score src/ --watch

# Explain score changes
ruchy score src/ --explain --baseline=main

# Project-specific configuration
ruchy score src/ --config=.ruchy/score.toml

# MCP integration
ruchy score src/ --json | mcp-client analyze
```

## Grade Boundaries

```rust
pub enum Grade {
    APlus,  // [0.97, 1.00] - Ship to production
    A,       // [0.93, 0.97) - Ship with confidence
    AMinus,  // [0.90, 0.93) - Ship with review
    BPlus,   // [0.87, 0.90) - Acceptable
    B,       // [0.83, 0.87) - Needs work
    BMinus,  // [0.80, 0.83) - Minimum viable
    CPlus,   // [0.77, 0.80) - Technical debt
    C,       // [0.73, 0.77) - Refactor advised
    CMinus,  // [0.70, 0.73) - Refactor required
    D,       // [0.60, 0.70) - Major issues
    F,       // [0.00, 0.60) - Fundamental problems
}
```

## Integration Points

### CI/CD Pipeline

```yaml
- name: Quality Gate
  run: |
    # Fast check for PRs
    ruchy score src/ --fast --min=0.75
    
    # Deep check for main branch
    if [[ "$GITHUB_REF" == "refs/heads/main" ]]; then
      ruchy score src/ --deep --min=0.85
    fi
```

### MCP Tool

```rust
#[mcp::tool]
async fn analyze_code_quality(path: String, depth: Option<String>) -> QualityReport {
    let depth = depth.map(|d| d.parse()).transpose()?.unwrap_or_default();
    let score = ruchy::score(&path, depth).await?;
    
    QualityReport {
        score: score.value,
        confidence: score.confidence,
        grade: score.grade,
        components: score.components,
        recommendations: score.generate_recommendations(),
        tradeoffs: score.identify_tradeoffs(&baseline),
    }
}
```

## Success Metrics

- **Adoption**: 80% of Ruchy projects using score in CI
- **Accuracy**: <5% false positive rate on quality issues
- **Performance**: <100ms for incremental scoring
- **Trust**: 90% developer agreement with score assessments
- **Impact**: 25% reduction in production defects

## Future Enhancements

1. **Machine Learning Calibration**: Train weights on defect correlation data
2. **Team Profiles**: Learn team-specific quality preferences
3. **Historical Trending**: Score velocity and trajectory analysis
4. **Cross-Language Scoring**: Extend to transpiled Rust code
5. **Distributed Analysis**: Cluster-based deep analysis for large codebases

## Extended Toolchain Integration

### 1. Mechanical Sympathy Tuner (`ruchy optimize`)

Profile-guided optimization with hardware-aware cost modeling.

```rust
pub struct MechanicalSympathyTuner {
    cpu_model: CpuCostModel,        // Cycle-accurate predictions via llvm-mca
    profile_data: ProfileDatabase,   // PGO data from previous runs
    cache_simulator: CacheSimulator, // Abstract interpretation for cache behavior
}

impl MechanicalSympathyTuner {
    pub fn analyze_abstraction_cost(&self, feature: AbstractionType) -> CostReport {
        // Track monomorphization costs
        match feature {
            AbstractionType::Option(inner) => {
                let niche_cost = self.calculate_niche_optimization(inner);
                let branch_cost = self.cpu_model.predict_branch_overhead();
                CostReport::new(niche_cost + branch_cost)
            }
            AbstractionType::Iterator(chain) => {
                self.verify_zero_cost_iteration(chain)
            }
        }
    }
    
    pub fn hardware_lint(&self, ast: &AST) -> Vec<HardwareLint> {
        vec![
            self.detect_cache_line_contention(ast),
            self.find_missed_vectorization(ast),
            self.identify_branch_misprediction_hotspots(ast),
        ].into_iter().flatten().collect()
    }
}
```

**Integration with Score:**
```rust
// Hardware efficiency becomes a component of performance score
fn score_performance_with_sympathy(code: &Analysis, tuner: &MechanicalSympathyTuner) -> f64 {
    let base_score = score_performance(code);
    let hw_efficiency = tuner.calculate_hardware_efficiency(code);
    base_score * 0.8 + hw_efficiency * 0.2
}
```

### 2. Actor Observatory (`ruchy actor:observe`)

Live introspection of concurrent systems without performance degradation.

```rust
pub struct ActorObservatory {
    telemetry_buffer: LockFreeRingBuffer<ActorEvent>,
    shadow_states: DashMap<ActorId, ShadowState>,
    supervision_tree: Arc<RwLock<SupervisionTree>>,
}

#[cfg_attr(observe, instrument)]
trait ObservableActor: Actor {
    fn observe_state(&self) -> ShadowState {
        // Copy-on-observe semantics
        ShadowState {
            mailbox_depth: self.mailbox.len(),
            memory_usage: self.estimate_memory(),
            reduction_count: self.reductions,
            generation: self.state_generation.fetch_add(1, Ordering::SeqCst),
        }
    }
}

impl ActorObservatory {
    pub fn trace_messages(&self, filter: MessageFilter) -> impl Stream<Item = Message> {
        self.telemetry_buffer
            .stream()
            .filter(move |msg| filter.matches(msg))
    }
    
    pub fn render_tui(&self) -> Result<()> {
        // Terminal UI with real-time updates
        let ui = tui::Terminal::new()?;
        ui.draw(|f| {
            f.render_widget(ActorDashboard::new(&self.shadow_states), f.size());
        })
    }
}
```

**Score Integration:**
```rust
// Actor health contributes to safety score
fn score_actor_safety(observatory: &ActorObservatory) -> f64 {
    let deadlock_risk = observatory.detect_potential_deadlocks();
    let mailbox_health = observatory.average_mailbox_depth() / MAX_HEALTHY_MAILBOX;
    1.0 - (deadlock_risk * 0.5 + mailbox_health.min(1.0) * 0.5)
}
```

### 3. Dataflow Debugger (`ruchy dataflow:debug`)

Interactive debugging for DataFrame pipelines with materialization on demand.

```rust
pub struct DataflowDebugger {
    pipeline_dag: DataflowGraph,
    breakpoints: HashSet<NodeId>,
    materialization_cache: LruCache<NodeId, DataFrame>,
}

enum DataflowNode {
    Lazy(Box<dyn Fn() -> DataFrame>),
    Materialized(Arc<DataFrame>), // Arc for zero-copy viewing
    Breakpoint(Box<DataflowNode>),
}

impl DataflowDebugger {
    pub fn step_into(&mut self, node_id: NodeId) -> DebugContext {
        let df = self.materialize_at(node_id);
        DebugContext {
            data: df.clone(),
            schema: df.schema(),
            prev_diff: self.compute_diff(node_id),
            query_plan: self.pipeline_dag.explain_at(node_id),
        }
    }
    
    pub fn set_breakpoint(&mut self, location: &str) -> Result<()> {
        let node_id = self.pipeline_dag.find_by_location(location)?;
        self.breakpoints.insert(node_id);
        Ok(())
    }
}
```

**Score Integration:**
```rust
// Pipeline complexity affects maintainability
fn score_dataflow_maintainability(debugger: &DataflowDebugger) -> f64 {
    let pipeline_depth = debugger.pipeline_dag.max_depth();
    let branch_factor = debugger.pipeline_dag.average_branching();
    1.0 / (1.0 + (pipeline_depth * branch_factor) / 100.0)
}
```

### 4. Component Toolkit (`ruchy wasm`)

Comprehensive WebAssembly component lifecycle management.

```rust
pub struct ComponentToolkit {
    wit_generator: WitGenerator,
    wasi_runtime: WasiRuntime,
    deployment_profiles: HashMap<Platform, DeploymentProfile>,
}

impl ComponentToolkit {
    pub fn generate_wit(&self, ruchy_type: &RuchyType) -> WitInterface {
        match ruchy_type {
            RuchyType::Actor(actor) => {
                WitInterface {
                    exports: actor.public_methods.map(|m| self.method_to_wit(m)),
                    imports: actor.dependencies.map(|d| self.dep_to_wit(d)),
                }
            }
            RuchyType::Refinement { base, predicate } => {
                let wit_base = self.generate_wit(base);
                self.inject_validation(wit_base, predicate)
            }
        }
    }
    
    pub fn deploy(&self, component: Component, platform: Platform) -> Result<DeploymentUrl> {
        let profile = self.deployment_profiles.get(&platform)?;
        let optimized = profile.optimize(component);
        let polyfilled = profile.polyfill_missing_features(optimized);
        profile.deploy(polyfilled).await
    }
}
```

**Score Integration:**
```rust
// Component portability affects idiomaticity score
fn score_wasm_idiomaticity(toolkit: &ComponentToolkit, component: &Component) -> f64 {
    let wit_compliance = toolkit.validate_wit_compliance(component);
    let platform_coverage = toolkit.count_supported_platforms(component) as f64 / 5.0;
    wit_compliance * 0.7 + platform_coverage * 0.3
}
```

### 5. Interactive Prover (`ruchy prove`)

REPL-based refinement type verification with developer-friendly tactics.

```rust
pub struct InteractiveProver {
    smt_solver: SmtSolver,
    proof_state: ProofState,
    tactic_library: TacticLibrary,
}

enum Tactic {
    Induction(Pattern),
    CaseAnalysis(Expression),
    Simplify,
    UseHypothesis(Name),
    Unfold(Definition),
    Auto, // Automated proof search
}

impl InteractiveProver {
    pub fn apply_tactic(&mut self, tactic: Tactic) -> ProofResult {
        let transformed = self.proof_state.apply(tactic);
        match self.smt_solver.check(transformed) {
            SmtResult::Proved => ProofResult::Success,
            SmtResult::Counterexample(cex) => {
                ProofResult::Failed(self.generate_test_case(cex))
            }
            SmtResult::Unknown => ProofResult::Incomplete(transformed)
        }
    }
    
    pub fn suggest_tactics(&self) -> Vec<(Tactic, f64)> {
        // ML-based tactic suggestion with confidence scores
        self.tactic_library.rank_by_similarity(&self.proof_state)
    }
    
    fn generate_test_case(&self, cex: Counterexample) -> String {
        // Convert SMT model to executable Ruchy test
        format!(
            "#[test]\nfn counterexample_{}_{}() {{\n    {}\n}}",
            self.proof_state.function_name,
            cex.hash(),
            cex.to_ruchy_code()
        )
    }
}
```

**Score Integration:**
```rust
// Proven properties dramatically increase correctness score
fn score_correctness_with_proofs(code: &Analysis, prover: &InteractiveProver) -> f64 {
    let base_score = score_correctness(code);
    let proof_coverage = prover.proven_properties() as f64 / prover.total_properties() as f64;
    
    // Proofs are weighted heavily - they provide mathematical certainty
    base_score * 0.6 + proof_coverage * 0.4
}
```

## Unified Score Architecture

All tools contribute to the unified quality score through specialized analyzers:

```rust
pub struct UnifiedScorer {
    base_analyzer: BaseAnalyzer,
    mechanical_sympathy: Option<MechanicalSympathyTuner>,
    actor_observatory: Option<ActorObservatory>,
    dataflow_debugger: Option<DataflowDebugger>,
    component_toolkit: Option<ComponentToolkit>,
    interactive_prover: Option<InteractiveProver>,
}

impl UnifiedScorer {
    pub fn calculate_comprehensive_score(&self, code: &Code) -> QualityScore {
        let mut components = self.base_analyzer.analyze(code);
        
        // Each specialized tool refines the score with additional insights
        if let Some(tuner) = &self.mechanical_sympathy {
            components.performance = components.performance * 0.8 + 
                                    tuner.hardware_efficiency(code) * 0.2;
        }
        
        if let Some(observatory) = &self.actor_observatory {
            components.safety = components.safety * 0.9 + 
                              observatory.actor_health(code) * 0.1;
        }
        
        if let Some(prover) = &self.interactive_prover {
            components.correctness = components.correctness * 0.6 + 
                                   prover.proof_coverage(code) * 0.4;
        }
        
        QualityScore::from_components(components)
    }
}
```

## Implementation Timeline

| Tool | Priority | Duration | Dependencies | Risk Mitigation |
|------|----------|----------|--------------|-----------------|
| Interactive Debugger | 1 | 4 months | Source map generation | Phase 1: Interpreter-only (2mo) |
| Actor Observatory | 2 | 3 months | Runtime instrumentation | Copy-on-observe, no blocking |
| Dataflow Debugger | 3 | 2 months | DataFrame API stability | Reuse breakpoint infrastructure |
| Component Toolkit | 4 | 4 months | WIT spec finalization | Leverage wasm-pack patterns |
| Interactive Prover | 5 | 6 months | SMT solver integration | Z3 bindings, defer custom solver |
| Mechanical Sympathy | 6 | 8 months | LLVM-MCA bindings | Start with static analysis only |

### Expertise Requirements

Per review feedback on "unicorn" team requirements, the implementation strategy:

1. **Core Team Focus**: Parser, type system, transpiler (Rust expertise required)
2. **Community Contributions**: Tool analyzers, MCP tools (domain expertise sufficient)
3. **External Dependencies**: PMCP for MCP, Z3 for SMT, LLVM for optimization
4. **Incremental Expertise**: Hire specialists as tools mature, not upfront

### Discipline Enforcement

Per "Foundation First" risk assessment:

```rust
// Compile-time quality gates prevent feature development until foundations complete
#[cfg(not(phase_0_complete))]
compile_error!("Phase 0 (parser, type system, test coverage) must be complete");

pub struct QualityGate {
    phase_0_metrics: Phase0Metrics,
    enforcement: EnforcementLevel,
}

impl QualityGate {
    pub fn check_phase_0(&self) -> Result<()> {
        // Hard requirements - no bypass possible
        assert!(self.phase_0_metrics.parser_coverage > 0.95);
        assert!(self.phase_0_metrics.type_system_complete);
        assert!(self.phase_0_metrics.cyclomatic_max < 10);
        assert!(self.phase_0_metrics.satd_count == 0);
        Ok(())
    }
}
```

## Success Metrics

Realistic targets acknowledging scope challenges:

- **Phase 0 Completion**: 6 months (non-negotiable blocker)
- **MVP Scoring**: Month 7 (AST metrics only)
- **Tool Integration**: 1 tool per quarter after MVP
- **Production Ready**: 24 months for core + 3 tools
- **Full Vision**: 5+ years (acknowledged, not promised)

## Ruchy Interactive Debugger (`ridb`)

The missing critical component for dynamic analysis: step-by-step execution control with complete state inspection.

### Core Architecture

```rust
pub struct RuchyDebugger {
    // Multi-tier execution backends
    interpreter_backend: InterpreterDebugger,
    compiled_backend: DwarfDebugger,
    jit_backend: Option<JitDebugger>,
    
    // Source mapping infrastructure
    source_mapper: SourceMapper,
    
    // Runtime expression evaluator
    eval_engine: ExpressionEvaluator,
    
    // Actor system integration
    actor_inspector: ActorInspector,
}

pub struct DebuggerState {
    breakpoints: BTreeMap<Location, Breakpoint>,
    call_stack: Vec<StackFrame>,
    local_vars: HashMap<String, Value>,
    actor_states: DashMap<ActorId, ActorSnapshot>,
}
```

### Source-Level Debugging

The fundamental challenge: maintaining source fidelity across transpilation.

```rust
impl SourceMapper {
    pub fn map_to_ruchy(&self, addr: MachineAddress) -> SourceLocation {
        // Three-level mapping: machine -> rust -> ruchy
        let rust_loc = self.dwarf_reader.addr_to_line(addr)?;
        let ruchy_loc = self.transpiler_map.rust_to_ruchy(rust_loc)?;
        ruchy_loc
    }
    
    pub fn inject_debug_info(&mut self, mir: &MIR) -> MIR {
        // Embed source locations at MIR level for preservation through optimization
        mir.instructions.map(|inst| {
            inst.with_debug_location(self.current_source_loc())
        })
    }
}
```

### Expression Evaluation

Runtime compilation within debugger context.

```rust
impl ExpressionEvaluator {
    pub fn eval(&self, expr: &str, context: &StackFrame) -> Result<Value> {
        // Parse expression in current lexical context
        let ast = self.parser.parse_expr(expr)?;
        
        // Type check against captured locals
        let typed_ast = self.type_checker.check_with_env(
            ast, 
            &context.type_environment()
        )?;
        
        // Execute via interpreter (fast path)
        self.interpreter.eval_in_context(typed_ast, context.locals())
    }
}
```

### Actor-Aware Debugging

Non-invasive concurrent system inspection.

```rust
impl ActorInspector {
    pub fn pause_actor(&self, id: ActorId) -> ActorSnapshot {
        // Copy-on-pause semantics
        let actor = self.registry.get(id)?;
        
        ActorSnapshot {
            state: actor.shadow_state.clone(),
            mailbox: actor.mailbox.snapshot(),
            reduction_count: actor.reductions.load(Ordering::Acquire),
            supervision_tree: self.get_supervision_context(id),
        }
    }
    
    pub fn trace_messages(&self, filter: MessageFilter) -> MessageStream {
        // Zero-copy message inspection via ring buffer
        self.message_bus
            .tap()
            .filter(move |msg| filter.matches(msg))
    }
}
```

### Phased Implementation

#### Phase 1: Interpreter Debugger (Month 1-2)
```rust
impl InterpreterDebugger {
    pub fn step(&mut self) -> StepResult {
        // Direct AST traversal - no source mapping needed
        let next_node = self.interpreter.next_node();
        
        if self.should_break_at(next_node) {
            StepResult::Breakpoint(self.capture_state())
        } else {
            self.interpreter.execute_node(next_node);
            StepResult::Continue
        }
    }
}
```

#### Phase 2: Compiled Code Debugger (Month 3-4)
```rust
impl DwarfDebugger {
    pub fn attach(&mut self, pid: ProcessId) -> Result<()> {
        // OS-level process control
        #[cfg(unix)]
        unsafe { ptrace::attach(pid)?; }
        
        // Load debug symbols
        self.dwarf_reader.load_symbols(pid)?;
        
        // Install breakpoint handlers
        self.install_signal_handlers()?;
        
        Ok(())
    }
    
    pub fn single_step(&mut self) -> Result<()> {
        #[cfg(unix)]
        unsafe { 
            ptrace::single_step(self.pid)?;
            self.wait_for_signal()?;
        }
        
        self.update_state()
    }
}
```

### Post-Mortem Debugging

Panic capture with full state preservation.

```rust
impl PanicHandler {
    pub fn capture_crash_state(&self) -> CrashDump {
        CrashDump {
            stack_trace: self.unwind_stack(),
            local_variables: self.capture_all_locals(),
            heap_snapshot: self.capture_heap(),
            actor_states: self.snapshot_all_actors(),
            timestamp: Instant::now(),
        }
    }
    
    pub fn enter_post_mortem(&self, dump: CrashDump) -> RidbSession {
        // Reconstruct debugging session from crash dump
        RidbSession::from_dump(dump)
    }
}
```

### Score Integration

Debuggability becomes a component of maintainability.

```rust
fn score_debuggability(code: &Analysis) -> f64 {
    let mut score = 1.0;
    
    // Penalize stripped symbols
    if !code.has_debug_symbols { score -= 0.3; }
    
    // Penalize macro-heavy code (harder to debug)
    score -= (code.macro_expansion_ratio * 0.2).min(0.2);
    
    // Reward explicit error handling (easier post-mortem)
    score += (code.result_types_ratio * 0.1).min(0.1);
    
    // Penalize async complexity (harder to trace)
    score -= (code.async_depth / 10.0).min(0.2);
    
    score.max(0.0)
}

// Integrate into maintainability score
fn score_maintainability_with_debug(code: &Analysis) -> f64 {
    let base = score_maintainability(code);
    let debug = score_debuggability(code);
    base * 0.85 + debug * 0.15
}
```

### CLI Interface

```bash
# Start with debugger
ruchy debug script.ruchy

# Attach to running process
ruchy debug --attach <pid>

# Post-mortem on panic
ruchy run script.ruchy --panic=debug

# Debug specific actor
ruchy debug script.ruchy --actor=worker-1

# Debug with expression watches
ruchy debug script.ruchy --watch="data.len()" --watch="state.phase"
```

### Debugger Commands

```
ridb> b 42                    # Breakpoint at line 42
ridb> b main if x > 100       # Conditional breakpoint
ridb> n                       # Next line
ridb> s                       # Step into
ridb> c                       # Continue
ridb> p variable              # Print variable
ridb> inspect dataframe       # Rich inspection
ridb> eval x * 2 + y         # Evaluate expression
ridb> bt                      # Backtrace
ridb> up/down                 # Navigate stack
ridb> actor ls                # List actors
ridb> actor pause worker-1    # Pause specific actor
ridb> actor trace worker-*    # Trace actor messages
ridb> save dump.ridb         # Save session
ridb> q                       # Quit
```

### Performance Considerations

Debug builds maintain <5% overhead through:

1. **Lazy symbol loading**: DWARF info loaded on-demand
2. **Incremental source mapping**: Cache hot paths
3. **Copy-on-write snapshots**: Zero-cost until accessed
4. **Ring buffer telemetry**: Lock-free message tracing

### Integration Points

The debugger integrates with all other tools:

- **Score**: Debuggability metrics feed maintainability score
- **Observatory**: Share actor telemetry infrastructure
- **Dataflow Debugger**: Unified breakpoint system
- **Prover**: Import counterexamples as debug scenarios
- **Mechanical Sympathy**: Show assembly alongside source

This completes the developer experience toolchain, providing the essential capability of understanding program behavior through controlled execution.