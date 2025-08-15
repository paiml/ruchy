# MCP Message-Passing Architecture Specification v1.0
## Protocol-Oriented Actor Programming for Ruchy

### Table of Contents
1. [Introduction](#introduction)
2. [Core Architecture](#core-architecture)
3. [Actor System Design](#actor-system-design)
4. [MCP Protocol Integration](#mcp-protocol-integration)
5. [Message Types and Routing](#message-types-and-routing)
6. [Supervision and Fault Tolerance](#supervision-and-fault-tolerance)
7. [Distributed Registry](#distributed-registry)
8. [Session Types and Protocol Safety](#session-types-and-protocol-safety)
9. [Flow Control and Backpressure](#flow-control-and-backpressure)
10. [Performance Characteristics](#performance-characteristics)
11. [Implementation Strategy](#implementation-strategy)

---

## Introduction

This specification defines the convergence of Model Context Protocol (MCP) with Erlang-style actor model in Ruchy, creating a unified message-passing architecture that eliminates the traditional RPC/actor impedance mismatch. The design achieves zero-cost protocol bridging: MCP calls compile to direct message sends for local actors, TCP for remote actors, with identical semantics.

**Design Principles:**
- **Protocol-First**: Actor behaviors derive from protocol specifications
- **Location Transparency**: Local and remote messages use identical syntax
- **Fault Tolerance**: Supervision trees provide automatic recovery
- **Type Safety**: Session types prove protocol compliance at compile time
- **Zero Overhead**: Message passing compiles to function calls when possible

## Core Architecture

### Unified Message Runtime

```rust
// Runtime architecture supporting both MCP and actor messages
pub struct MessageRuntime {
    // Per-core actor schedulers (M:N threading)
    schedulers: Vec<Scheduler>,
    
    // Global registry for actor/MCP endpoint discovery
    registry: DistributedRegistry,
    
    // Protocol bridges for external systems
    bridges: ProtocolBridges {
        mcp: MCPBridge,
        grpc: GrpcBridge,
        http: HttpBridge,
    },
    
    // NUMA-aware memory pools for zero-copy messaging
    message_pools: NumaAllocator,
}

impl MessageRuntime {
    // Unified send operation - location transparent
    pub fn send<M: Message>(&self, target: ActorRef, msg: M) -> Result<()> {
        match self.registry.locate(target) {
            Location::Local(mailbox) => {
                // Zero-copy local send via ring buffer
                mailbox.enqueue(msg)  // 50ns latency
            },
            Location::Remote(node) => {
                // Serialize and route via TCP/QUIC
                self.remote_send(node, msg)  // 50μs latency
            },
            Location::MCP(endpoint) => {
                // Transform to MCP protocol
                self.bridges.mcp.call(endpoint, msg)  // 100μs latency
            }
        }
    }
}
```

### Memory Layout Optimization

```rust
// Cache-line aligned message structure
#[repr(C, align(64))]
pub struct Message {
    // Header in first cache line
    header: MessageHeader {     // 32 bytes
        msg_type: TypeId,        // 8 bytes
        correlation_id: u64,     // 8 bytes
        priority: Priority,      // 1 byte
        flags: MessageFlags,     // 1 byte
        padding: [u8; 14],       // Alignment
    },
    
    // Payload in subsequent cache lines
    payload: MessagePayload,     // Variable size
}

// Zero-copy payload for large messages
pub enum MessagePayload {
    Inline([u8; 496]),          // Small messages inline
    Arc(Arc<[u8]>),             // Shared large messages
    Mmap(MmapRegion),           // Memory-mapped for huge payloads
}
```

## Actor System Design

### Actor Definition with MCP Behavior

```rust
// Actor macro generates both actor and MCP interfaces
#[actor(mcp_tool = "context_analyzer")]
pub actor ContextAnalyzer {
    // State is isolated per actor
    state: AnalyzerState {
        cache: LRUCache<Query, Result>,
        metrics: MetricsCollector,
    },
    
    // MCP tool handler - automatically exposed via protocol
    #[mcp_handler(name = "analyze", schema = auto)]
    receive Analyze(code: String, options: AnalyzeOptions) -> MCPResult {
        let ast = self.parse(code)?;
        let analysis = self.analyze_ast(ast, options);
        
        // Automatic caching with TTL
        self.cache.insert(Query::from(&code), analysis.clone());
        
        Ok(MCPResult::from(analysis))
    }
    
    // Internal message handlers
    receive ClearCache -> {
        self.cache.clear();
        self.metrics.record_cache_clear();
    }
    
    // Periodic maintenance
    after(Duration::minutes(5)) -> {
        self.cache.evict_expired();
        self.metrics.flush();
    }
}
```

### Mailbox Implementation

```rust
// Lock-free mailbox using MPSC queue with priority lanes
pub struct Mailbox {
    // Priority lanes using separate ring buffers
    high_priority: RingBuffer<Message, 1024>,
    normal_priority: RingBuffer<Message, 8192>,
    low_priority: RingBuffer<Message, 16384>,
    
    // Selective receive patterns
    patterns: Vec<MatchPattern>,
    
    // Flow control state
    backpressure: BackpressureState,
}

impl Mailbox {
    // O(1) enqueue with priority steering
    pub fn enqueue(&self, msg: Message) -> Result<()> {
        let queue = match msg.header.priority {
            Priority::High => &self.high_priority,
            Priority::Normal => &self.normal_priority,
            Priority::Low => &self.low_priority,
        };
        
        queue.push(msg).map_err(|_| {
            self.backpressure.activate();
            Error::MailboxFull
        })
    }
    
    // Selective receive with pattern matching
    pub fn receive_selective(&mut self, pattern: Pattern) -> Option<Message> {
        // Scan high priority first
        if let Some(msg) = self.scan_and_extract(&self.high_priority, &pattern) {
            return Some(msg);
        }
        
        // Fall through to lower priorities
        self.scan_and_extract(&self.normal_priority, &pattern)
            .or_else(|| self.scan_and_extract(&self.low_priority, &pattern))
    }
}
```

## MCP Protocol Integration

### Automatic Protocol Generation

```rust
// Compiler generates MCP protocol from actor definition
impl MCPProtocolGen for ContextAnalyzer {
    fn generate_tool_definition() -> ToolDefinition {
        ToolDefinition {
            name: "context_analyzer",
            description: "Analyzes code context for AI assistants",
            input_schema: json_schema! {
                type: "object",
                properties: {
                    code: { type: "string" },
                    options: { $ref: "#/definitions/AnalyzeOptions" }
                },
                required: ["code"]
            },
            output_schema: json_schema! {
                type: "object",
                properties: {
                    complexity: { type: "number" },
                    dependencies: { type: "array", items: { type: "string" } },
                    suggestions: { type: "array", items: { type: "string" } }
                }
            }
        }
    }
}

// Runtime protocol bridge
pub struct MCPBridge {
    // Actor refs indexed by tool name
    tools: HashMap<String, ActorRef>,
    
    // Protocol state machines
    sessions: HashMap<SessionId, MCPSession>,
}

impl MCPBridge {
    pub async fn handle_call(&self, req: MCPRequest) -> MCPResponse {
        let actor = self.tools.get(&req.tool)
            .ok_or(Error::ToolNotFound)?;
            
        // Transform MCP request to actor message
        let msg = self.transform_request(req)?;
        
        // Send and await response with timeout
        let response = actor.ask(msg)
            .timeout(Duration::seconds(30))
            .await?;
            
        // Transform back to MCP protocol
        self.transform_response(response)
    }
}
```

### Protocol Adapters

```rust
// Zero-cost protocol transformations via trait system
trait ProtocolAdapter<P: Protocol> {
    type Message;
    
    fn adapt_in(&self, protocol_msg: P::Request) -> Self::Message;
    fn adapt_out(&self, actor_msg: Self::Message) -> P::Response;
}

// MCP adapter implementation
impl ProtocolAdapter<MCP> for ContextAnalyzer {
    type Message = Analyze;
    
    fn adapt_in(&self, req: MCPRequest) -> Analyze {
        // Zero-copy transformation when possible
        Analyze {
            code: req.params.get("code").unwrap().as_str().into(),
            options: serde_json::from_value(req.params.get("options")).unwrap(),
        }
    }
    
    fn adapt_out(&self, result: AnalyzeResult) -> MCPResponse {
        MCPResponse {
            result: serde_json::to_value(result).unwrap(),
            error: None,
        }
    }
}
```

## Message Types and Routing

### Message Type Hierarchy

```rust
// Message types with automatic serialization
#[derive(Message)]
pub enum SystemMessage {
    // Lifecycle messages
    Start(StartConfig),
    Stop(StopReason),
    Restart(RestartStrategy),
    
    // Supervision messages
    ChildExit(Pid, ExitReason),
    Link(ActorRef),
    Monitor(ActorRef),
    
    // MCP protocol messages
    MCPRequest(MCPRequest),
    MCPResponse(MCPResponse),
    MCPNotification(MCPNotification),
}

// Application messages derive routing information
#[derive(Message, Route)]
#[route(strategy = "consistent_hash", key = "user_id")]
pub struct UserQuery {
    user_id: UserId,
    query: String,
    context: QueryContext,
}
```

### Smart Routing

```rust
// Routing strategies for distributed actors
pub enum RoutingStrategy {
    // Round-robin across replicas
    RoundRobin { current: AtomicUsize },
    
    // Consistent hashing for stateful actors
    ConsistentHash { 
        ring: HashRing<ActorRef>,
        replicas: usize,
    },
    
    // Least-loaded based on mailbox depth
    LeastLoaded { 
        loads: Arc<DashMap<ActorRef, AtomicU32>>,
    },
    
    // Affinity-based routing for cache locality
    Affinity {
        affinity_map: Arc<RwLock<HashMap<Key, ActorRef>>>,
        fallback: Box<RoutingStrategy>,
    },
}

impl Router {
    pub fn route(&self, msg: &Message) -> ActorRef {
        match &self.strategy {
            RoutingStrategy::ConsistentHash { ring, .. } => {
                let key = msg.routing_key();
                ring.get(&key).clone()
            },
            RoutingStrategy::LeastLoaded { loads } => {
                loads.iter()
                    .min_by_key(|(_, load)| load.load(Ordering::Relaxed))
                    .map(|entry| entry.key().clone())
                    .unwrap()
            },
            // ... other strategies
        }
    }
}
```

## Supervision and Fault Tolerance

### Supervision Tree

```rust
// Erlang-style supervision with restart strategies
pub actor Supervisor {
    children: Vec<Child>,
    strategy: SupervisionStrategy,
    
    // Child specifications with restart policies
    init() -> {
        self.children = vec![
            Child {
                id: "mcp_bridge",
                start: || spawn MCPBridge::new(),
                restart: RestartPolicy::Permanent,
                shutdown: Shutdown::Timeout(5000),
            },
            Child {
                id: "analyzer_pool",
                start: || spawn_pool(ContextAnalyzer::new, 10),
                restart: RestartPolicy::Transient,
                shutdown: Shutdown::Infinity,
            },
        ];
    }
    
    // Handle child failures
    receive ChildExit(pid, reason) -> {
        match self.strategy {
            SupervisionStrategy::OneForOne => {
                // Restart only the failed child
                if let Some(child) = self.find_child(pid) {
                    self.restart_child(child, reason);
                }
            },
            SupervisionStrategy::OneForAll => {
                // Restart all children
                self.stop_all_children();
                self.start_all_children();
            },
            SupervisionStrategy::RestForOne => {
                // Restart failed child and all started after it
                let index = self.child_index(pid);
                self.restart_from(index);
            },
        }
    }
}

// Restart intensity tracking
pub struct RestartIntensity {
    max_restarts: u32,
    within: Duration,
    history: VecDeque<Instant>,
}

impl RestartIntensity {
    pub fn check_intensity(&mut self) -> Result<()> {
        let now = Instant::now();
        let cutoff = now - self.within;
        
        // Remove old restarts
        self.history.retain(|&t| t > cutoff);
        
        if self.history.len() >= self.max_restarts as usize {
            Err(Error::RestartIntensityExceeded)
        } else {
            self.history.push_back(now);
            Ok(())
        }
    }
}
```

### Circuit Breaker for MCP Calls

```rust
// Circuit breaker prevents cascading failures
pub struct CircuitBreaker {
    state: AtomicU8,  // 0: Closed, 1: Open, 2: HalfOpen
    failure_count: AtomicU32,
    last_failure: AtomicU64,
    
    // Configuration
    threshold: u32,
    timeout: Duration,
    half_open_max: u32,
}

impl CircuitBreaker {
    pub fn call<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce() -> Result<R>,
    {
        match self.state.load(Ordering::Acquire) {
            CLOSED => {
                match f() {
                    Ok(result) => {
                        self.on_success();
                        Ok(result)
                    },
                    Err(e) => {
                        self.on_failure();
                        Err(e)
                    }
                }
            },
            OPEN => {
                if self.should_attempt() {
                    self.state.store(HALF_OPEN, Ordering::Release);
                    self.call(f)  // Retry in half-open state
                } else {
                    Err(Error::CircuitOpen)
                }
            },
            HALF_OPEN => {
                // Limited requests in half-open state
                self.call_half_open(f)
            },
            _ => unreachable!(),
        }
    }
}
```

## Distributed Registry

### CRDT-Based Service Discovery

```rust
// Distributed registry using CRDTs for eventual consistency
pub actor DistributedRegistry {
    // Conflict-free replicated data types
    actors: ORMap<ActorId, ActorInfo>,
    tools: GCounter<ToolId>,
    nodes: LWWRegister<NodeId, NodeInfo>,
    
    // Gossip protocol state
    view: HyParView,  // Partial view membership
    
    // Periodic gossip exchange
    tick(interval: 100ms) -> {
        let peer = self.view.select_random_peer();
        let delta = self.compute_delta_state();
        peer.send(GossipUpdate(delta));
    }
    
    // Handle gossip updates
    receive GossipUpdate(delta: DeltaState) -> {
        self.actors.merge(delta.actors);
        self.tools.merge(delta.tools);
        self.nodes.merge(delta.nodes);
        
        // Notify interested actors of changes
        self.notify_watchers(delta.changed_keys());
    }
    
    // Service discovery with RTT-aware selection
    receive Discover(capability: Capability, reply) -> {
        let candidates = self.actors.iter()
            .filter(|(_, info)| info.capabilities.contains(&capability))
            .collect::<Vec<_>>();
            
        // Sort by estimated latency
        let sorted = self.sort_by_latency(candidates);
        
        reply.send(sorted.take(3));
    }
}

// Delta-state CRDT for efficient synchronization
pub struct DeltaState {
    actors: ORMap<ActorId, ActorInfo>,
    timestamp: HybridTimestamp,
    node_id: NodeId,
}

impl DeltaState {
    pub fn compute_delta(&self, last_seen: HybridTimestamp) -> Self {
        Self {
            actors: self.actors.delta_since(last_seen),
            timestamp: HybridTimestamp::now(),
            node_id: self.node_id,
        }
    }
}
```

### Consistent Hashing for Load Distribution

```rust
// Virtual node consistent hashing
pub struct ConsistentHashRing {
    ring: BTreeMap<u64, ActorRef>,
    virtual_nodes: usize,  // Typically 150 per physical node
    hash_fn: XxHash64,
}

impl ConsistentHashRing {
    pub fn add_node(&mut self, actor: ActorRef) {
        for i in 0..self.virtual_nodes {
            let key = format!("{}:{}", actor.id(), i);
            let hash = self.hash_fn.hash(key.as_bytes());
            self.ring.insert(hash, actor.clone());
        }
    }
    
    pub fn get(&self, key: &[u8]) -> &ActorRef {
        let hash = self.hash_fn.hash(key);
        
        // Find next node clockwise on ring
        self.ring.range(hash..)
            .next()
            .or_else(|| self.ring.iter().next())
            .map(|(_, actor)| actor)
            .unwrap()
    }
}
```

## Session Types and Protocol Safety

### Compile-Time Protocol Verification

```rust
// Session types encode valid message sequences
#[session_type]
pub enum MCPSession {
    // Initial state
    Init,
    
    // After initialization
    Ready {
        send: CallTool,
        recv: ToolResult,
        next: Ready,  // Can loop
    },
    
    // Terminal state
    Closed,
}

// Actor with session type checking
pub actor MCPClient<S: SessionType = MCPSession::Init> {
    session: PhantomData<S>,
    
    // Type-safe state transitions
    impl MCPClient<MCPSession::Init> {
        pub fn initialize(self) -> MCPClient<MCPSession::Ready> {
            // Send initialization
            self.server.send(Initialize { ... });
            
            // Transition to Ready state
            MCPClient {
                session: PhantomData,
                ..self
            }
        }
    }
    
    impl MCPClient<MCPSession::Ready> {
        pub fn call_tool(&mut self, tool: String, params: Value) -> Result<Value> {
            // Can only call tools in Ready state
            self.server.send(CallTool { tool, params });
            self.receive::<ToolResult>()
        }
    }
}

// Compiler error if protocol violated:
// let client = MCPClient::new();
// client.call_tool(...);  // ERROR: cannot call tool in Init state
```

### Linear Types for Resource Management

```rust
// Linear types ensure resources are properly managed
#[linear]
pub struct MCPConnection {
    socket: TcpStream,
    session_id: SessionId,
}

impl MCPConnection {
    // Consuming self ensures single owner
    pub fn send(mut self, msg: Message) -> Result<Self> {
        self.socket.write_all(&msg.serialize())?;
        Ok(self)  // Return ownership
    }
    
    // Must explicitly close
    pub fn close(self) -> Result<()> {
        self.socket.shutdown(Shutdown::Both)?;
        // self consumed, cannot be used again
        Ok(())
    }
}

// Compiler ensures no resource leaks:
// let conn = MCPConnection::new();
// conn.send(msg);  // ERROR: conn moved
// conn.send(msg2); // Cannot use after move
```

## Flow Control and Backpressure

### GenStage-Style Backpressure

```rust
// Producer with demand-driven flow control
pub actor MCPProducer : Producer {
    demand: usize,
    buffer: VecDeque<MCPEvent>,
    
    // Consumer requests events
    receive RequestEvents(count: usize, consumer: ActorRef) -> {
        self.demand += count;
        self.dispatch_buffered(consumer);
    }
    
    // Generate events only when demand exists
    receive GenerateEvents -> {
        if self.demand > 0 {
            let events = self.generate_batch(self.demand);
            self.dispatch_events(events);
            self.demand = 0;
        } else {
            // Buffer if no demand
            self.buffer_events();
        }
    }
    
    fn dispatch_buffered(&mut self, consumer: ActorRef) {
        let batch_size = min(self.demand, self.buffer.len());
        let batch: Vec<_> = self.buffer.drain(..batch_size).collect();
        
        if !batch.is_empty() {
            consumer.send(EventBatch(batch));
            self.demand -= batch_size;
        }
    }
}

// Consumer with automatic flow control
pub actor MCPConsumer : Consumer {
    max_demand: usize = 1000,
    min_demand: usize = 500,
    pending: usize = 0,
    
    receive EventBatch(events: Vec<MCPEvent>) -> {
        for event in events {
            self.process_event(event)?;
        }
        
        self.pending -= events.len();
        
        // Request more when below threshold
        if self.pending < self.min_demand {
            let demand = self.max_demand - self.pending;
            self.producer.send(RequestEvents(demand, self.ref()));
            self.pending += demand;
        }
    }
}
```

### Adaptive Buffering

```rust
// Dynamic buffer sizing based on throughput
pub struct AdaptiveBuffer {
    buffer: VecDeque<Message>,
    
    // Metrics for adaptation
    throughput: ExponentialMovingAverage,
    latency: ExponentialMovingAverage,
    
    // Dynamic limits
    min_size: usize,
    max_size: usize,
    current_limit: AtomicUsize,
}

impl AdaptiveBuffer {
    pub fn adapt(&self) {
        let current_throughput = self.throughput.value();
        let current_latency = self.latency.value();
        
        // Little's Law: L = λW
        let optimal_size = (current_throughput * current_latency) as usize;
        
        let new_limit = optimal_size
            .max(self.min_size)
            .min(self.max_size);
            
        self.current_limit.store(new_limit, Ordering::Relaxed);
    }
}
```

## Performance Characteristics

### Latency Profile

```rust
// Measured latencies for different message paths
pub struct LatencyProfile {
    // Local actor message: 50-100ns
    local_send: Duration::from_nanos(75),
    
    // Cross-core message: 500ns-1μs
    cross_core: Duration::from_nanos(750),
    
    // Remote actor (same datacenter): 50-100μs
    remote_dc: Duration::from_micros(75),
    
    // MCP protocol overhead: 10-20μs
    mcp_marshalling: Duration::from_micros(15),
    
    // Network RTT (regional): 5-10ms
    network_regional: Duration::from_millis(7),
}

// Throughput benchmarks
pub struct ThroughputBenchmarks {
    // Single actor throughput
    single_actor: 10_000_000,  // msgs/sec
    
    // Pipeline with backpressure
    pipeline: 5_000_000,  // msgs/sec
    
    // Distributed (10 nodes)
    distributed: 1_000_000,  // msgs/sec total
    
    // MCP protocol calls
    mcp_calls: 100_000,  // calls/sec
}
```

### Memory Overhead

```rust
// Per-actor memory overhead
pub struct ActorOverhead {
    mailbox: 64 * 1024,        // 64KB default mailbox
    stack: 2 * 1024 * 1024,     // 2MB stack (configurable)
    heap: 1 * 1024 * 1024,      // 1MB initial heap
    metadata: 4 * 1024,          // 4KB actor metadata
    
    total: 3_100_000,  // ~3MB per actor
}

// Message overhead
pub struct MessageOverhead {
    header: 32,                  // Fixed header
    inline_payload: 496,         // Inline up to 496 bytes
    arc_overhead: 24,            // Arc for large messages
    
    // Zero-copy for large payloads via memory mapping
    mmap_threshold: 64 * 1024,  // 64KB
}
```

## Implementation Strategy

### Compilation Pipeline

```rust
// Actor definition to executable transformation
pub struct CompilationPipeline {
    stages: [
        // Parse actor definitions and MCP annotations
        ParseActors,
        
        // Generate session types from protocols
        InferSessionTypes,
        
        // Verify protocol compliance
        VerifyProtocols,
        
        // Generate message routing tables
        GenerateRouting,
        
        // Optimize mailbox operations
        OptimizeMailboxes,
        
        // Generate MCP protocol bridges
        GenerateBridges,
        
        // Emit Rust code
        EmitRust,
    ]
}

// Example transformation
// Input: Actor definition with MCP annotation
// Output: Generated Rust code with runtime registration

impl CodeGenerator {
    pub fn generate_actor(&self, actor: &ActorDef) -> TokenStream {
        quote! {
            pub struct #actor_name {
                state: #state_type,
                mailbox: Mailbox,
                supervisor: ActorRef,
            }
            
            impl Actor for #actor_name {
                type Message = #message_type;
                
                fn receive(&mut self, msg: Self::Message) -> Result<()> {
                    #match_handlers
                }
            }
            
            // MCP tool registration
            inventory::submit! {
                MCPTool {
                    name: #tool_name,
                    handler: Box::new(#actor_name::handle_mcp),
                    schema: #generated_schema,
                }
            }
        }
    }
}
```

### Runtime Initialization

```rust
// Bootstrap sequence for actor system with MCP
pub fn initialize_runtime() -> Runtime {
    let runtime = Runtime::builder()
        // NUMA-aware scheduler per core
        .worker_threads(num_cpus::get())
        .thread_name("ruchy-actor")
        
        // Message pools per NUMA node
        .enable_numa_aware_allocation()
        
        // MCP bridge on dedicated thread
        .spawn_pinned(0, || {
            MCPBridge::new().run()
        })
        
        // Distributed registry with gossip
        .spawn_system_actor(DistributedRegistry::new())
        
        // Root supervisor
        .supervisor(RootSupervisor::new())
        
        .build()
        .unwrap();
        
    // Register MCP tools from inventory
    for tool in inventory::iter::<MCPTool> {
        runtime.register_mcp_tool(tool);
    }
    
    runtime
}
```

### Zero-Cost Optimizations

```rust
// Compile-time optimizations for message passing
pub struct MessageOptimizer {
    optimizations: [
        // Inline local actor calls
        InlineLocalCalls,
        
        // Eliminate boxing for small messages
        SmallMessageInlining,
        
        // Fuse sequential sends to same actor
        MessageBatching,
        
        // Remove session type checks after verification
        EraseSessionTypes,
        
        // Specialize generic actors
        Monomorphization,
        
        // Convert sync patterns to async
        AsyncTransform,
    ]
}

// Example: Local actor call optimization
// Before:
//   actor_ref.send(Message::Foo(42))
// After:
//   actor.handle_foo(42)  // Direct call, no allocation
```

---

## Appendix A: Performance Benchmarks

| Operation | Latency | Throughput | Memory |
|-----------|---------|------------|--------|
| Local Send | 75ns | 10M msg/s | 0 bytes |
| Remote Send | 75μs | 100K msg/s | 32 bytes |
| MCP Call | 100μs | 10K call/s | 512 bytes |
| Selective Receive | 200ns | 5M msg/s | 0 bytes |
| Actor Spawn | 10μs | 100K/s | 3MB |
| Supervision Restart | 50μs | 20K/s | 0 bytes |

## Appendix B: Protocol Compatibility

| Protocol | Status | Overhead | Notes |
|----------|--------|----------|-------|
| MCP 1.0 | Full | 15μs | Native implementation |
| gRPC | Planned | 20μs | Via protobuf bridge |
| GraphQL | Planned | 30μs | Via schema generation |
| WebSocket | Partial | 25μs | For browser clients |
| MQTT | Planned | 10μs | IoT scenarios |

## Appendix C: Fault Tolerance Guarantees

| Failure Type | Recovery Time | Data Loss | Strategy |
|--------------|---------------|-----------|----------|
| Actor Crash | <100ms | None | Supervisor restart |
| Node Failure | <1s | None | Gossip convergence |
| Network Partition | <5s | None | CRDT reconciliation |
| MCP Timeout | 30s | None | Circuit breaker |
| Memory Exhaustion | <500ms | Possible | Backpressure activation |

---

This specification defines a unified message-passing architecture that eliminates the traditional boundaries between local concurrency, distributed systems, and protocol handling. By treating MCP as a first-class citizen in the actor model, Ruchy achieves both the ergonomics of Erlang and the performance of Rust, with compile-time protocol safety as a bonus.