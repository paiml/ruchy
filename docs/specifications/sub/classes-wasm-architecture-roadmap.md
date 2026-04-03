# Sub-spec: Classes & OOP — WASM Architecture, Migration & Roadmap

**Parent:** [ruchy_classes_spec.md](../ruchy_classes_spec.md) Sections 18-24

---

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
