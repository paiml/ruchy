# Ruchy Classes and Object-Oriented Features Specification
Version 2.0 | Status: Final

## Executive Summary

Ruchy implements object-oriented patterns through mechanical syntax transformation to Rust's zero-cost abstractions. No runtime, no vtables, no inheritance hierarchy. Every construct has deterministic transpilation with observable assembly output via integrated MCP tooling. This specification serves as the authoritative reference for Ruchy's approach to encapsulation, composition, and message-passing concurrency.

## Core Architecture Philosophy

**Mechanical Transparency**: Every language construct must compile to predictable, inspectable machine code. The compiler serves as an intelligent translation layer, not an opaque optimization engine. Users can trace from source syntax through AST, MIR, WASM, to final assembly via MCP tools.

**Zero-Cost Composition**: Object-oriented patterns compile away entirely. Method dispatch resolves statically. Actor message passing inlines when possible. The abstraction penalty is precisely zero nanoseconds in release builds.

## 1. Design Rationale

### 1.1 Why No Classes

Traditional class hierarchies require one of three implementation strategies in Rust, each problematic:

| Strategy | Implementation | Runtime Cost | Compile Cost |
|----------|---------------|--------------|--------------|
| Trait Objects | `Box<dyn Trait>` | Virtual dispatch, heap allocation | Moderate |
| Enum Dispatch | Tagged unions | Branch prediction miss | Closed set |
| Macro Generation | Code generation | None | Exponential growth |

Ruchy instead leverages Rust's existing composition patterns with syntax sugar.

### 1.2 Mechanical Transformation Principle

Every Ruchy construct must have a deterministic, zero-cost Rust equivalent:

```rust
// Ruchy source
struct Point { x: f64, y: f64 }
impl Point {
    fun distance(&self) -> f64 = (self.x² + self.y²).sqrt()
}

// Generated Rust (character-for-character predictable)
struct Point { x: f64, y: f64 }
impl Point {
    fn distance(&self) -> f64 { (self.x.powi(2) + self.y.powi(2)).sqrt() }
}
```

## 2. Struct Definition

### 2.1 Basic Syntax

```rust
struct Name {
    field: Type,                // Private by default
    pub optional: Type? = None, // Public optional with default
    mutable: mut Type,          // Interior mutability hint
    pub public_field: Type,     // Explicit public field
}
```

### 2.2 Transpilation Rules

| Ruchy | Rust Output |
|-------|-------------|
| `field: Type` | `field: Type` (private by default) |
| `pub field: Type` | `pub field: Type` |
| `field: Type?` | `field: Option<Type>` |
| `pub field: Type?` | `pub field: Option<Type>` |
| `field: Type = value` | Constructor default |
| `mut field: Type` | `field: RefCell<Type>` |
| `pub mut field: Type` | `pub field: RefCell<Type>` |

### 2.3 Constructor Generation

Structs automatically get a builder pattern constructor:

```rust
// Ruchy
struct Config {
    host: String = "localhost",
    port: u16 = 8080,
    timeout: Duration? = None
}

// Generated Rust
impl Config {
    pub fn new() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            timeout: None,
        }
    }
    
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }
    
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
}
```

## 3. Implementation Blocks

### 3.1 Method Syntax

```rust
impl StructName {
    // Constructor convention
    fun new(...) -> Self { ... }
    
    // Instance methods
    fun method(&self) -> T { ... }
    fun mut_method(&mut self) { ... }
    
    // Associated functions
    fun static_fn() -> T { ... }
    
    // Property getter/setter sugar
    get property(&self) -> T { self.field }
    set property(&mut self, val: T) { self.field = val }
}
```

### 3.2 Method Transpilation

```rust
// Ruchy compact form
impl Point {
    fun distance(&self) = (self.x² + self.y²).sqrt()
}

// Expands to
impl Point {
    pub fn distance(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}
```

## 4. Trait System

### 4.1 Trait Definition

```rust
trait Drawable {
    // Required methods
    fun draw(&self);
    
    // Default implementations
    fun bounds(&self) -> Rect = {
        Rect::default()
    }
    
    // Associated types
    type Color;
    
    // Associated constants
    const MAX_SIZE: u32 = 1000;
}
```

### 4.2 Trait Implementation

```rust
impl Drawable for Circle {
    type Color = RGB;
    
    fun draw(&self) {
        // Implementation
    }
}
```

## 5. Extension Methods

### 5.1 Syntax

```rust
// Extend existing types with new methods
extend String {
    fun is_palindrome(&self) -> bool {
        let clean = self.chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .to_lowercase();
        clean == clean.chars().rev().collect()
    }
}

// Usage
"racecar".is_palindrome()  // true
```

### 5.2 Transpilation

Extension methods generate trait implementations:

```rust
// Generated Rust
trait StringExt {
    fn is_palindrome(&self) -> bool;
}

impl StringExt for String {
    fn is_palindrome(&self) -> bool {
        let clean = self.chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .to_lowercase();
        clean == clean.chars().rev().collect::<String>()
    }
}
```

## 6. Actor Pattern

### 6.1 Actor Definition

Actors provide encapsulation similar to classes but with message-passing semantics:

```rust
actor Counter {
    // Private state
    count: i32 = 0,
    
    // Message handlers
    receive increment() {
        self.count += 1;
    }
    
    receive decrement() {
        self.count -= 1;
    }
    
    receive get() -> i32 {
        self.count
    }
    
    // Lifecycle hooks
    on_start() {
        println!("Counter starting");
    }
    
    on_stop() {
        println!("Final count: {}", self.count);
    }
}
```

### 6.2 Actor Transpilation

Actors generate approximately 200 lines of boilerplate:

```rust
// Message enum generation
enum CounterMessage {
    Increment,
    Decrement,
    Get { reply: oneshot::Sender<i32> },
}

// State struct
struct CounterState {
    count: i32,
}

// Actor implementation
struct CounterActor {
    state: CounterState,
    receiver: mpsc::Receiver<CounterMessage>,
}

// Handle struct for external interface
pub struct Counter {
    sender: mpsc::Sender<CounterMessage>,
}

impl Counter {
    pub async fn increment(&self) -> Result<()> {
        self.sender.send(CounterMessage::Increment).await
    }
    
    pub async fn get(&self) -> Result<i32> {
        let (tx, rx) = oneshot::channel();
        self.sender.send(CounterMessage::Get { reply: tx }).await?;
        rx.await
    }
}
```

## 7. Composition Patterns

### 7.1 Delegation

```rust
struct Engine {
    horsepower: u32
}

struct Car {
    engine: Engine,
    
    // Delegate to engine
    delegate horsepower to engine;
}

// Generates
impl Car {
    pub fn horsepower(&self) -> u32 {
        self.engine.horsepower
    }
}
```

### 7.2 Mixins via Traits

```rust
// Define reusable behavior
trait Timestamped {
    fun created_at(&self) -> DateTime;
    fun updated_at(&self) -> DateTime;
}

// Mix into structs
struct Post with Timestamped {
    title: String,
    content: String,
}

// Generates fields and impl
struct Post {
    title: String,
    content: String,
    created_at: DateTime,
    updated_at: DateTime,
}
```

## 8. Property System

### 8.1 Computed Properties

```rust
impl Rectangle {
    get area(&self) -> f64 {
        self.width * self.height
    }
    
    get perimeter(&self) -> f64 {
        2.0 * (self.width + self.height)
    }
}

// Usage
let r = Rectangle { width: 10.0, height: 20.0 };
println!("{}", r.area);  // Calls area()
```

### 8.2 Observable Properties

```rust
struct Model {
    @observable name: String,
}

// Generates
impl Model {
    pub fn set_name(&mut self, name: String) {
        let old = std::mem::replace(&mut self.name, name);
        self.notify_observers("name", &old, &self.name);
    }
}
```

## 9. Syntax Sugar Mappings

| Ruchy Feature | Rust Equivalent | Line Reduction |
|---------------|-----------------|----------------|
| `extend Type` | Trait + impl | 40% |
| `actor Name` | Struct + channels + tasks | 95% |
| `get/set` | Getter/setter methods | 60% |
| `delegate` | Forwarding methods | 80% |
| `with Trait` | Trait fields + impl | 70% |

## 10. Non-Features (Deliberate Omissions)

### 10.1 No Inheritance

```rust
// NOT SUPPORTED
struct Animal { name: String }
struct Dog extends Animal { breed: String }  // ❌

// INSTEAD USE
trait Animal {
    fun name(&self) -> &str;
}

struct Dog {
    name: String,
    breed: String,
}

impl Animal for Dog {
    fun name(&self) -> &str { &self.name }
}
```

### 10.2 No Method Overloading

```rust
// NOT SUPPORTED
impl Calculator {
    fun add(x: i32, y: i32) -> i32 { x + y }      // ❌
    fun add(x: f64, y: f64) -> f64 { x + y }      // ❌
}

// INSTEAD USE
impl Calculator {
    fun add_i32(x: i32, y: i32) -> i32 { x + y }
    fun add_f64(x: f64, y: f64) -> f64 { x + y }
}
```

### 10.3 No Dynamic Dispatch by Default

```rust
// NOT SUPPORTED (implicit boxing)
let shapes: Vec<Shape> = vec![Circle::new(), Square::new()];  // ❌

// EXPLICIT WHEN NEEDED
let shapes: Vec<Box<dyn Shape>> = vec![
    Box::new(Circle::new()),
    Box::new(Square::new()),
];
```

## 11. Implementation Timeline

| Feature | Status | Target | Complexity |
|---------|--------|--------|------------|
| Structs | ✅ Complete | - | Low |
| Impl blocks | ✅ Complete | - | Low |
| Traits | ✅ Complete | - | Medium |
| Extension methods | [x] In Progress | v0.7.12 | Medium |
| Actors | 📅 Planned | v0.4 | High |
| Properties | 📅 Planned | v0.5 | Medium |
| Delegation | 📅 Planned | v0.6 | Low |

## 12. Performance Guarantees

All object-oriented features maintain zero-cost abstraction:

```rust
// Ruchy struct method call
point.distance()

// Compiles to identical assembly as
Point::distance(&point)

// No vtable lookup, no indirection
```

## 13. Testing Requirements

Each OOP feature requires:

1. **Transpilation tests**: Ruchy input → expected Rust output
2. **Semantic tests**: Behavior preservation
3. **Performance tests**: Zero overhead verification
4. **Error tests**: Meaningful error messages

## Appendix A: Actor Implementation Detail

Actor implementation details to be documented.

## Appendix B: Extension Method Resolution

Resolution follows Rust's trait rules:
1. Inherent methods first
2. Extension methods in scope
3. Error on ambiguity

## 14. MCP-Native Introspection

### 14.1 AST Inspection

Every Ruchy construct exposes its AST via MCP tools:

```rust
#[mcp::tool("inspect_ast")]
fun show_ast(target: TypePath) -> AstNode {
    // Returns serialized AST for any type/function
    compiler::ast_for(target)
}

// Usage via MCP
> mcp.inspect_ast("Counter")
{
  "node": "Actor",
  "name": "Counter",
  "fields": [{"name": "count", "type": "i32", "default": 0}],
  "handlers": [
    {"name": "increment", "params": [], "body": {...}},
    {"name": "get", "returns": "i32", "body": {...}}
  ]
}
```

### 14.2 Type Graph Visualization

```rust
#[mcp::tool("type_graph")]  
fun show_relationships(root: TypePath) -> Graph {
    // Generates DOT/Mermaid graph of type relationships
    TypeGraph::from(root)
        .with_traits()
        .with_impls()
        .with_dependencies()
        .render()
}

// Output format
digraph {
    Counter -> CounterMessage [label="generates"];
    Counter -> mpsc::Sender [label="contains"];
    Counter -> Supervision [label="implements"];
}
```

## 15. Provability via PMAT

### 15.1 Property Specifications

```rust
#[pmat::spec]
impl Counter {
    // Invariant: count never negative
    #[invariant]
    fun count_non_negative(&self) -> bool {
        self.count >= 0
    }
    
    // Property: increment increases by exactly 1
    #[property]
    fun increment_adds_one(&self, initial: i32) -> bool {
        let before = self.get();
        self.increment();
        self.get() == before + 1
    }
}
```

### 15.2 SMT-Backed Verification

```rust
actor BankAccount {
    balance: i64 = 0,
    
    #[requires(amount > 0)]
    #[ensures(self.balance == old(self.balance) + amount)]
    receive deposit(amount: i64) {
        self.balance += amount;
    }
    
    #[requires(amount > 0 && amount <= self.balance)]
    #[ensures(self.balance == old(self.balance) - amount)]
    receive withdraw(amount: i64) -> Result<()> {
        self.balance -= amount;
        Ok(())
    }
}

// Generates Z3 constraints for verification
```

## 16. Disassembly Integration

### 16.1 Assembly Inspection

```rust
#[mcp::tool("show_assembly")]
fun disassemble(target: FunctionPath) -> Assembly {
    compiler::emit_asm(target, OptLevel::Release)
}

// Example output
> mcp.disassemble("Point::distance")
Point::distance:
    mulss  xmm0, xmm0      ; x²
    mulss  xmm1, xmm1      ; y²
    addss  xmm0, xmm1      ; x² + y²
    sqrtss xmm0, xmm0      ; √(x² + y²)
    ret
```

### 16.2 Optimization Verification

```rust
#[test]
#[pmat::verify_zero_cost]
fun test_actor_overhead() {
    // Verify actor message passing compiles to direct call
    let counter = Counter::spawn();
    counter.increment();  
    
    // PMAT verifies this produces identical assembly to:
    // counter_state.count += 1;
}
```

## 17. Metaprogramming

### 17.1 Compile-Time Reflection

```rust
// Type-level computation
meta fun generate_builder<T: Struct>() -> impl Builder<T> {
    let fields = T::fields();
    
    struct ${T}Builder {
        ${for field in fields {
            ${field.name}: Option<${field.type}>,
        }}
    }
    
    impl ${T}Builder {
        ${for field in fields {
            fun with_${field.name}(mut self, val: ${field.type}) -> Self {
                self.${field.name} = Some(val);
                self
            }
        }}
    }
}

// Usage
#[derive_builder]
struct Config { host: String, port: u16 }
```

### 17.2 Hygenic Macros

```rust
// Ruchy macros are hygenic and type-aware
macro define_enum_matcher($enum_type) {
    impl $enum_type {
        fun match_all<R>(&self, ${arms}) -> R {
            match self {
                ${for variant in $enum_type::variants() {
                    Self::${variant} => ${arms[variant]},
                }}
            }
        }
    }
}

// Expansion happens at Rust AST level, not text
```

### 17.3 Const Evaluation

```rust
// Compile-time execution
const TABLE: DataFrame = {
    let data = include_csv!("data.csv");
    data.filter(|row| row.valid)
        .select(["id", "name"])
        .collect()
};

// Const functions for metaprogramming
const fun type_size<T>() -> usize {
    std::mem::size_of::<T>()
}

// Used in type-level assertions
#[static_assert(type_size<Handle>() <= 8)]
struct Handle { ... }
```

### 17.4 Code Generation Hooks

```rust
// Pre-transpilation transformers
#[transformer(phase = "pre_typecheck")]
fun optimize_dataframe_ops(ast: &mut Ast) {
    // Rewrite df.filter().map().collect() to single pass
    ast.visit_mut(|node| {
        if let Chain(ops) = node {
            fuse_operations(ops);
        }
    });
}

// Post-transpilation Rust manipulation
#[transformer(phase = "post_transpile")]
fun add_telemetry(rust_ast: &mut syn::File) {
    // Inject performance counters
    for func in rust_ast.items.iter_mut() {
        inject_timer(func);
    }
}
```

## 18. WASM-First Architecture

### 18.1 Dual Compilation Targets

```rust
// Every Ruchy construct compiles to both native and WASM
#[target(wasm32, native)]
actor Calculator {
    // WASM: Uses atomics + SharedArrayBuffer
    // Native: Uses std::sync primitives
    state: i32 = 0,
    
    receive compute(input: i32) -> i32 {
        self.state = complex_calculation(input);
        self.state
    }
}
```

### 18.2 WASM-Specific Optimizations

```rust
// Automatic WASM size optimization
#[wasm::optimize(size)]
struct LightweightHandle {
    // In WASM: u32 index into table
    // In native: *const Actor
    id: ActorId,
}

// Zero-copy between JS and Ruchy
#[wasm::bindgen]
impl DataFrame {
    // Exposes typed array views, no serialization
    #[wasm::zero_copy]
    fun as_float_array(&self) -> Float64Array {
        // Direct memory view into WASM linear memory
        unsafe { Float64Array::view(&self.data) }
    }
}
```

### 18.3 WASI Component Model

```rust
// First-class WASI component support
#[wasm::component]
interface DataProcessor {
    // Automatically generates WIT interface
    process: fun(input: DataFrame) -> DataFrame;
    validate: fun(data: Series) -> Result<()>;
}

// Compiles to:
// interface data-processor {
//   record dataframe { ... }
//   process: func(input: dataframe) -> dataframe;
//   validate: func(data: series) -> result<_, error>;
// }
```

### 18.4 WASM Actor Runtime

```rust
// Actors map to WASM threads (when available) or green threads
actor WebWorker {
    #[wasm::worker]  // Spawns in Web Worker
    receive process_chunk(data: ArrayBuffer) {
        // Runs in isolated worker context
    }
    
    #[wasm::shared_memory]  // Uses SharedArrayBuffer
    receive atomic_increment(&mut self) {
        // Atomic operations on shared memory
    }
}

// Runtime automatically handles:
// - Worker spawn/terminate
// - SharedArrayBuffer when available
// - Fallback to postMessage
```

### 18.5 Module Federation

```rust
// WASM modules as first-class imports
import wasm "https://cdn.example.com/analytics.wasm" as analytics;
import wasm "./local_module.wasm" as local;

// Type-safe FFI without bindings
fun analyze(data: DataFrame) -> Report {
    // Direct WASM-to-WASM call, no JS bridge
    analytics::process(data)
}
```

## 18. WASM-First Architecture

### 18.1 Dual-Target Compilation

Every Ruchy construct mechanically transpiles to both Rust and WAT (WebAssembly Text):

```rust
actor Counter {
    count: i32 = 0,
    receive increment() { self.count += 1 }
}

// Generates parallel artifacts:
// 1. counter.rs → rustc → native binary
// 2. counter.wat → wasm-opt → counter.wasm (3KB)
```

Compilation strategy:
- **Native**: Full Rust std library
- **WASM**: no_std + wasm-specific allocator
- **Size target**: <10KB base runtime

### 18.2 Memory Model Unification

```rust
// Ruchy's memory model maps to WASM linear memory
struct WasmActor {
    // All actor state in linear memory segment
    state_offset: u32,      // Offset into memory.pages
    mailbox_offset: u32,    // Ring buffer in SharedArrayBuffer
    
    // No indirection for WASM builds
    #[cfg(target = "wasm32")]
    direct_memory: &'static mut [u8; 65536],
}

// Zero-copy DataFrame operations
impl DataFrame {
    #[wasm::export]
    fun as_memory_view(&self) -> (u32, u32) {
        // Returns (offset, length) for direct JS TypedArray access
        (self.as_ptr() as u32, self.len() as u32)
    }
}
```

### 18.3 Component Model Integration

```rust
// WIT (WebAssembly Interface Types) generation
#[wit::interface]
trait DataProcessor {
    fn process(input: DataFrame) -> DataFrame;
}

// Generates WIT:
interface data-processor {
    use types.{dataframe}
    process: func(input: dataframe) -> dataframe
}

// And component metadata:
(component
  (import "types" (instance $types
    (export "dataframe" (type))))
  (core func $process (param i32 i32) (result i32))
  (func (export "process") 
    (param "input" (type $types.dataframe))
    (result (type $types.dataframe))
    (canon lift (core func $process))))
```

### 18.4 WASM-Specific Actor Runtime

```rust
// Actors compile to different primitives per target
actor FileProcessor {
    #[cfg(wasm32)]
    spawn_mode: WasmSpawn::Worker,  // Web Worker
    
    #[cfg(not(wasm32))]  
    spawn_mode: NativeSpawn::Thread, // OS thread
    
    receive process(file: Blob) {
        // WASM: Blob API
        // Native: std::fs
    }
}

// Actor mailboxes use appropriate primitives:
enum MailboxImpl {
    SharedArrayBuffer(Sab<Message>),  // WASM threads
    Atomics(AtomicRingBuffer),        // WASM atomics
    PostMessage(Channel),              // WASM workers
    Mpsc(mpsc::Receiver),              // Native
}
```

### 18.5 WASM Optimization Directives

```rust
// Size optimization via compile-time stripping
#[wasm::optimize(size)]
impl HeavyComputation {
    // Stripped in WASM unless explicitly retained
    #[wasm::strip(unless = "debug")]
    fn debug_info(&self) -> String { ... }
    
    // Inlined aggressively in WASM
    #[wasm::inline(always)]
    fn hot_path(&self) -> i32 { ... }
}

// SIMD detection and polyfill
#[wasm::simd(fallback = scalar)]
fun matrix_multiply(a: Matrix, b: Matrix) -> Matrix {
    // Uses v128 SIMD if available
    // Falls back to scalar in older browsers
}
```

### 18.6 JavaScript Interop

```rust
// Zero-cost JS bindings via extern
extern "js" {
    #[wasm::import("console", "log")]
    fn console_log(s: &str);
    
    #[wasm::import("Math", "random")]  
    fn math_random() -> f64;
}

// Direct DOM manipulation
#[wasm::bindgen]
impl Component {
    fun render(&self) -> Node {
        // Compiles to direct DOM calls, no VDOM
        document.create_element("div")
            .set_text(&self.content)
    }
}
```

### 18.7 Module Federation

```rust
// WASM modules as packages
import wasm "https://unpkg.com/analytics-wasm" as analytics;
import wasm "./compute.wasm" as compute;

// Type-safe cross-module calls
fun pipeline(data: DataFrame) -> Report {
    data
    |> compute::transform    // WASM-to-WASM, no JS
    |> analytics::analyze     // Direct memory passing
}
```

### 18.8 Performance Guarantees

| Operation | Native | WASM | Overhead |
|-----------|--------|------|----------|
| Actor message | 15ns | 25ns | 1.6x |
| DataFrame alloc | 200ns | 250ns | 1.25x |
| SIMD operation | 1 cycle | 1 cycle | 1.0x |
| JS interop | N/A | 5ns | - |

Target: WASM within 2x of native for all operations.

## 19. Streaming Compilation

### 19.1 Incremental WASM Generation

```rust
// WASM modules generated per actor for streaming
#[wasm::streaming]
actor StreamProcessor {
    // Compiles to separate .wasm file
    // Loaded on-demand via WebAssembly.instantiateStreaming
}

// Enables progressive loading:
// 1. Load core runtime (5KB)
// 2. Stream actor modules as needed
// 3. Total time to first paint: <50ms
```

### 19.2 Edge Deployment

```rust
// Cloudflare Workers / Deno Deploy optimization
#[wasm::edge]
actor EdgeHandler {
    // No filesystem, no threads
    // Pure request/response model
    
    receive handle(req: Request) -> Response {
        // Compiles to CF Workers compatible WASM
    }
}
```

## 20. Architectural Integration Points

### 20.1 Grand Unified Theory

This specification is not isolated—it integrates with the complete Ruchy platform:

```rust
// Cross-specification dependencies
dependencies {
    parser: "docs/architecture/grammar.md",
    types: "docs/specification/type-system.md",
    runtime: "docs/specification/runtime-architecture.md",
    quality: "docs/specification/quality-gates.md",
    mcp: "docs/architecture/message-passing-mcp.md",
}
```

### 20.2 Vertical Integration

Every OOP construct participates in the full stack:

| Layer | Integration Point | Specification |
|-------|------------------|---------------|
| Syntax | Grammar rules | `grammar.md` |
| Types | Inference engine | `type-system.md` |
| Runtime | Actor scheduler | `runtime-architecture.md` |
| Verification | PMAT properties | `quality-gates.md` |
| Distribution | WASM components | `wasm-architecture.md` |
| Introspection | MCP tools | `message-passing-mcp.md` |

### 20.3 Correctness by Construction

Object patterns enforce quality invariants:

```rust
#[invariant("actors never deadlock")]
#[invariant("no data races possible")]
#[invariant("message ordering preserved")]
actor CriticalSystem {
    // Compiler proves invariants via SMT
}
```

## 21. Performance Transparency

### 21.1 Cost Model

Every feature has measurable overhead:

| Feature | Source Lines | Generated Lines | Runtime Cost |
|---------|-------------|-----------------|--------------|
| `struct` | 1 | 1 | 0ns |
| `impl` method | 1 | 1.2 | 0ns |
| `trait` | 1 | 1.5 | 0ns (static) |
| `extend` | 1 | 3 | 0ns |
| `actor` basic | 5 | 200 | 25ns/msg |
| `actor` supervised | 10 | 500 | 50ns/msg |

### 21.2 Observable Compilation

```rust
#[mcp::trace_compilation]
actor Example {
    // Emits compilation trace:
    // 1. Parser: 2ms
    // 2. Type check: 5ms
    // 3. Actor expansion: 15ms
    // 4. Rust codegen: 8ms
    // 5. LLVM opt: 45ms
    // Total: 75ms
}
```

## 22. Migration Strategy

### 22.1 From Other Languages

| Source | Pattern | Ruchy Equivalent |
|--------|---------|------------------|
| Python class | Inheritance | Trait composition |
| Java interface | Abstract methods | Trait |
| C++ virtual | Dynamic dispatch | Trait object (explicit) |
| JavaScript class | Prototype | Struct + impl |
| Go interface | Duck typing | Trait (structural) |

### 22.2 Progressive Adoption

```rust
// Stage 1: Direct port
struct User {
    name: String,
    age: i32,
}

// Stage 2: Add methods
impl User {
    fun validate(&self) -> bool {
        self.age >= 0 && self.age < 150
    }
}

// Stage 3: Extract traits
trait Validatable {
    fun validate(&self) -> bool;
}

// Stage 4: Actor for concurrency
actor UserService {
    users: Vec<User>,
    receive get(id: UserId) -> Option<User> {
        self.users.find(|u| u.id == id)
    }
}
```

## 23. Implementation Roadmap

### Phase 1: Foundation (Q1 2025)
- ✅ Struct definitions
- ✅ Basic impl blocks
- ✅ Trait definitions
- 🚧 Extension methods

### Phase 2: Composition (Q2 2025)
- Actor message generation
- Supervision trees
- Property getters/setters
- Delegation syntax

### Phase 3: Verification (Q3 2025)
- PMAT trait properties
- SMT invariant checking
- Assembly verification
- Zero-cost proofs

### Phase 4: Distribution (Q4 2025)
- WASM actor compilation
- Component model
- Edge deployment
- Module federation

## 24. Design Decisions Record

### DDR-001: No Inheritance
**Decision**: Omit class inheritance
**Rationale**: No mechanical translation to Rust
**Alternative**: Trait composition
**Trade-off**: Familiarity vs performance

### DDR-002: Actor Priority
**Decision**: Actors as P0 feature
**Rationale**: 95% boilerplate reduction
**Alternative**: Manual channels
**Trade-off**: Compiler complexity vs user value

### DDR-003: WASM Co-equal
**Decision**: WASM as primary target
**Rationale**: Future deployment reality
**Alternative**: WASM as secondary
**Trade-off**: Design constraints vs portability

## Appendices

### Appendix A: Complete Actor Expansion
[Full 200-line expansion example]

### Appendix B: Trait Resolution Algorithm
[Detailed resolution rules]

### Appendix C: WASM Memory Layout
[Linear memory mapping]

### Appendix D: MCP Tool Protocol
[Complete tool specifications]

---

*This specification is part of the Ruchy Grand Unified Architecture. For the complete system design, see SPECIFICATION.md sections 1-27.*