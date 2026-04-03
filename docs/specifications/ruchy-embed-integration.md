# Sub-spec: Ruchy-Embed -- Embeddable Scripting Engine for Rust Applications

**Parent:** docs/SPECIFICATION.md
**Ticket:** EMBED-001
**Status:** Proposed
**Priority:** High
**Target Version:** 5.0.0 | **Current Version:** 4.2.1

---

## 0. Prerequisites

- **No `ruchy-embed` crate exists.** The workspace contains `ruchy` (compiler/CLI) and
  `ruchy-wasm` (browser target). There is no embeddable library crate.
- **`Interpreter` is program-oriented.** The struct in `src/runtime/interpreter.rs`
  executes complete programs with a fixed `main()` entry point, not hosted scripts.
- **No embedding API.** No `Engine` type, no host function registration, no sandboxing
  layer, no coroutine support, no type marshaling traits.
- **No resource limiting.** The interpreter loop has no instruction counting, memory
  tracking, or timeout hooks.

## 1. Overview

### 1.1 Motivation

Lua's dominance in game scripting rests on six properties: fast startup (<1ms), small
footprint (~278KB), simple host API, cooperative coroutines, hot reload, and
configurable sandboxing. Ruchy must match all six to compete as an embeddable
scripting language for Rust applications.

### 1.2 Unique Value Proposition

Ruchy offers a **three-stage graduation path** no competitor provides:

| Stage | Mode | Startup | Speed | Use Case |
|-------|------|---------|-------|----------|
| 1 | REPL / Interpreter | <1ms | Baseline | Prototyping, live tuning |
| 2 | Bytecode VM | <1ms | 10-50x faster | Production embedding |
| 3 | Transpile to Rust | Seconds | Native | Final deployment |

No other embeddable language (Lua, Rhai, Rune, mruby, Wren) offers a
compile-to-native escape hatch.

### 1.3 Competitive Landscape

| Feature | mlua (Lua) | Rhai | Rune | ruchy-embed |
|---------|-----------|------|------|-------------|
| Type safety | Weak | Strong | Strong | Strong |
| Sandboxing | Manual | Built-in | Partial | Built-in (capability-based) |
| Coroutines | Yes (C stack) | No | Yes (async) | Yes (interpreter-level) |
| Hot reload | Manual | Yes | Yes | Yes (state preservation) |
| Compile to native | No | No | No | Yes (transpile to Rust) |
| Binary size | ~278KB | ~500KB | ~1MB | ~1MB target |
| Rust integration | FFI bridge | Native | Native | Native |

### 1.4 Crate Scope

`ruchy-embed` is a new workspace member re-exporting only the parser and interpreter
from `ruchy`. CLI, WASM, dataframes, actors, notebooks, MCP, and HTTP are excluded.

## 2. Engine API

The `Engine` struct is the primary embedding interface. All methods are safe and
panic-free (returning `Result` on failure).

### 2.1 Core API

```rust
use ruchy_embed::{Engine, Value, RuchyError};

fn main() -> Result<(), RuchyError> {
    let mut engine = Engine::new(); // sandboxed by default

    engine.register_fn("get_health", |id: i64| -> i64 { database_lookup(id) });
    engine.register_type::<PlayerState>();
    engine.set("bonus_hp", 50_i64);

    let result: i64 = engine.eval("get_health(42) + bonus_hp")?;

    // Compile once, execute many times
    let script = engine.compile("get_health(player_id) * multiplier")?;
    for id in 0..1000 {
        engine.set("player_id", id);
        engine.set("multiplier", 2_i64);
        let _hp: i64 = engine.eval_ast(&script)?;
    }
    Ok(())
}
```

### 2.2 API Surface

| Method | Signature | Description |
|--------|-----------|-------------|
| `new()` | `fn new() -> Engine` | Sandboxed engine (safe defaults) |
| `new_unrestricted()` | `fn new_unrestricted() -> Engine` | Full capabilities |
| `with_capabilities()` | `fn with_capabilities(Capabilities) -> Engine` | Custom capability set |
| `register_fn()` | `fn register_fn<A, R>(&mut self, name, f)` | Host function (up to 8 params) |
| `register_type::<T>()` | `fn register_type<T: RuchyType>(&mut self)` | Expose Rust type |
| `compile()` | `fn compile(&self, src: &str) -> Result<CompiledScript>` | Parse, reusable |
| `eval()` | `fn eval<T: FromRuchyValue>(&mut self, expr: &str) -> Result<T>` | Evaluate expression |
| `eval_ast()` | `fn eval_ast<T: FromRuchyValue>(&mut self, s: &CompiledScript) -> Result<T>` | Execute compiled |
| `set()` / `get()` | `fn set/get<T>(&mut self, name: &str, ...)` | Global variables |
| `call_fn()` | `fn call_fn<T>(&mut self, name: &str, args: &[Value]) -> Result<T>` | Call script function |

### 2.3 Type Conversion Traits

```rust
pub trait IntoRuchyValue { fn into_ruchy_value(self) -> Value; }
pub trait FromRuchyValue: Sized { fn from_ruchy_value(v: Value) -> Result<Self, TypeError>; }
```

Built-in implementations: `i64`, `f64`, `bool`, `String`, `&str`, `()`,
`Vec<Value>`, `HashMap<String, Value>`, `Option<T>`, `Result<T, E>`.

## 3. Sandboxing (Capability-Based)

Scripts run in a restricted sandbox by default. Capabilities are granted at engine
construction and cannot be expanded after creation.

```rust
let caps = Capabilities {
    allow_fs: false,           allow_net: false,
    allow_process: false,      allow_env: false,
    allow_ffi: false,          allow_actors: false,
    instruction_limit: Some(1_000_000),
    memory_limit_bytes: Some(64 * 1024 * 1024),
    timeout: Some(Duration::from_secs(5)),
};
let engine = Engine::with_capabilities(caps);
```

### 3.1 Capability Table

| Capability | Default | Controls |
|------------|---------|----------|
| `allow_fs` | `false` | `open()`, `read()`, `write()`, path operations |
| `allow_net` | `false` | `http_get()`, `http_post()`, socket operations |
| `allow_process` | `false` | `exec()`, `spawn()`, process management |
| `allow_env` | `false` | `env_get()`, `env_set()`, environment access |
| `allow_ffi` | `false` | Calling into native C/Rust libraries |
| `allow_actors` | `false` | Actor spawning, message passing |
| `instruction_limit` | `None` | Counter checked in interpreter loop |
| `memory_limit_bytes` | `None` | Tracked in `Value` constructors |
| `timeout` | `None` | Wall-clock deadline at loop back-edges |

### 3.2 Enforcement Mechanisms

- **Instruction counting:** Counter incremented each instruction; exceeding limit
  returns `Err(InstructionLimitExceeded)`. Cost: ~2% overhead.
- **Memory tracking:** `Value` allocations update a per-engine byte counter;
  exceeding limit returns `Err(MemoryLimitExceeded)`.
- **Timeout:** `Instant::now() >= deadline` checked every 1024 instructions.
  No OS signals; pure Rust. Cost: ~5% amortized.
- **Capability checks:** I/O builtins consult the capability set; denied operations
  return `Err(CapabilityDenied { capability, operation })`.

## 4. Hot Reload

Hot reload updates script logic without restarting the host application.

```rust
let script = Arc::new(RwLock::new(engine.compile(initial_source)?));

// Game loop reads current script
let r = Arc::clone(&script);
std::thread::spawn(move || loop {
    let compiled = r.read().unwrap();
    let result: f64 = engine.eval_ast(&compiled).unwrap();
    apply_game_logic(result);
});

// File watcher replaces script atomically
let r = Arc::clone(&script);
notify::recommended_watcher(move |_| {
    if let Ok(src) = std::fs::read_to_string(&path) {
        if let Ok(new) = engine.compile(&src) { *r.write().unwrap() = new; }
    }
});
```

**Semantics:**
- `CompiledScript` holds `Arc<Expr>` + source hash. Swap is atomic to readers.
- Global state persists across reloads (variables set via `set()` are preserved).
- Module cache invalidated on reload; dependents re-resolved on next access.
- Parse/compile failure keeps old script running; error returned to caller.

## 5. Coroutines (Lua-Style Cooperative)

### 5.1 Ruchy Syntax

```ruchy
fun ai_patrol():
    move_to(waypoint_a)
    yield
    move_to(waypoint_b)
    yield
    attack_nearest_enemy()
    yield
    retreat_to(base)
```

### 5.2 Host API

```rust
let mut co = engine.create_coroutine("counter", &[Value::Int(5)])?;
loop {
    match co.resume(&mut engine)? {
        CoroutineStatus::Yielded(v) => println!("Got: {v}"),
        CoroutineStatus::Completed(v) => { println!("Done: {v}"); break; }
    }
}
```

### 5.3 Implementation

- **Stack save/restore:** `yield` saves call frames, locals, and program counter
  into the `Coroutine` struct; `resume()` restores and continues.
- **No Tokio dependency.** Explicit state machines at interpreter level.
- **Game loop pattern:** Host calls `resume()` per frame for cooperative multitasking.

### 5.4 Coroutine Lifecycle

| State | Description | Transitions |
|-------|-------------|-------------|
| `Created` | Allocated, not started | `resume()` -> `Running` |
| `Running` | Executing | `yield` -> `Suspended`, return -> `Completed` |
| `Suspended` | Yielded | `resume()` -> `Running` |
| `Completed` | Final value returned | Terminal |
| `Failed` | Runtime error | Terminal |

## 6. Performance Tiers

| Tier | Engine | Startup | Throughput | Use Case |
|------|--------|---------|------------|----------|
| Interpreter | AST walk | <1ms | 1x | Development, REPL |
| Bytecode VM | Opcodes | <1ms | 10-50x | Production embedding |
| Transpile | Rust codegen | Seconds | 100-1000x | Final deployment |

**Graduation workflow:** Prototype in REPL -> embed with bytecode VM -> transpile hot
paths to native Rust via `ruchy transpile`. No other embeddable language supports this.

### 6.1 Benchmark Targets

| Operation | Target | Fail Threshold |
|-----------|--------|----------------|
| `Engine::new()` | <1ms | >5ms |
| `eval("1 + 1")` | <1ms | >5ms |
| `eval_ast(&compiled)` | <10us | >100us |
| Coroutine `resume()` | <5us | >50us |
| Host function overhead | <100ns | >1us |

## 7. Minimal Footprint

### 7.1 Feature Gates

```toml
[features]
default = ["interpreter"]
interpreter = []
bytecode = []
coroutines = []
serde = ["dep:serde"]
```

Excluded entirely (not compiled): dataframes, actors, notebooks, MCP, HTTP, WASM,
CLI (clap), transpiler.

### 7.2 Size Comparison

| Runtime | Binary Size | Notes |
|---------|-------------|-------|
| Lua 5.4 | ~278KB | C, minimal stdlib |
| Rhai | ~500KB | Rust, no-std capable |
| Rune | ~1MB | Rust, async runtime |
| **ruchy-embed** | **~1MB** | Rust, interpreter + parser |

Dependency budget: under 30 transitive crates for default features.
No tokio, serde (unless gated), clap, wasm-bindgen, polars, reqwest, hyper.

## 8. FFI and Interop

### 8.1 Host Function Registration

```rust
engine.register_fn("get_time", || -> f64 { get_elapsed_time() });
engine.register_fn("lerp", |a: f64, b: f64, t: f64| -> f64 { a + (b - a) * t });
engine.register_fn("read_config", |path: String| -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
});
```

Closures capturing host state (via `Arc<Mutex<T>>`) are supported. Up to 8 parameters.

### 8.2 Type Marshaling

| Rust Type | Ruchy Type | Direction |
|-----------|-----------|-----------|
| `i64` | `int` | Bidirectional |
| `f64` | `float` | Bidirectional |
| `bool` | `bool` | Bidirectional |
| `String` / `&str` | `str` | Bidirectional |
| `Vec<Value>` | `list` | Bidirectional |
| `HashMap<String, Value>` | `dict` | Bidirectional |
| `Option<T>` | `T \| None` | Bidirectional |
| `()` | `None` | Rust -> Ruchy |

### 8.3 Custom Types

```rust
#[derive(RuchyType)]
struct PlayerState { pub name: String, pub health: i64, pub position: (f64, f64) }

engine.register_type::<PlayerState>();
engine.set("player", PlayerState { name: "Hero".into(), health: 100, position: (0.0, 0.0) });
let hp: i64 = engine.eval("player.health")?;
```

### 8.4 Error Propagation

Host functions returning `Result<T, E>` where `E: Display` convert errors to Ruchy
exceptions. Scripts catch them with `try/except`.

## 9. Crate Structure

```
ruchy-embed/
├── Cargo.toml          # Workspace member, minimal deps
├── src/
│   ├── lib.rs          # Public API: Engine, CompiledScript, Value, errors
│   ├── engine.rs       # Core engine: eval/compile/set/get
│   ├── types.rs        # IntoRuchyValue, FromRuchyValue
│   ├── sandbox.rs      # Capabilities, enforcement hooks
│   ├── coroutine.rs    # Cooperative coroutines
│   └── ffi.rs          # Host function registration
├── tests/              # engine, sandbox, coroutine, ffi, hot_reload, proptest
├── benches/            # startup, eval, coroutine resume benchmarks
└── examples/           # game_scripting.rs, config_engine.rs
```

### 9.1 Cargo.toml

```toml
[package]
name = "ruchy-embed"
version = "5.0.0"
edition = "2021"
description = "Embed the Ruchy scripting language in Rust applications"
license = "MIT OR Apache-2.0"

[dependencies]
ruchy = { path = "../", default-features = false, features = ["embed"] }

[dev-dependencies]
proptest = "1"
criterion = { version = "0.5", features = ["html_reports"] }
```

The `ruchy` crate exposes an `embed` feature re-exporting parser and interpreter
modules without CLI, WASM, or heavyweight dependencies.

## 10. Testing Requirements

### 10.1 Test Categories

| Category | Framework | Target |
|----------|-----------|--------|
| Unit tests | `#[test]` | 50+ |
| Property tests (marshaling round-trips) | `proptest` | 10K+ cases |
| Sandbox escape tests | `#[test]` | 20+ |
| Coroutine lifecycle tests | `#[test]` | 15+ |
| Hot reload state tests | `#[test]` | 10+ |
| Integration tests | `#[test]` | 20+ |
| Benchmarks | `criterion` | 6 |

### 10.2 Critical Test Scenarios

**Sandbox escape** (must return `CapabilityDenied`):

```rust
#[test]
fn test_sandbox_denies_fs() {
    let engine = Engine::new();
    assert!(matches!(engine.eval::<String>("open('x').read()"), Err(RuchyError::CapabilityDenied { .. })));
}

#[test]
fn test_instruction_limit() {
    let caps = Capabilities { instruction_limit: Some(1000), ..Default::default() };
    let engine = Engine::with_capabilities(caps);
    assert!(matches!(engine.eval::<()>("while true: pass"), Err(RuchyError::InstructionLimitExceeded)));
}
```

**Type round-trip property tests:**

```rust
proptest! {
    #[test]
    fn roundtrip_i64(v: i64) {
        let back = i64::from_ruchy_value(v.into_ruchy_value()).unwrap();
        prop_assert_eq!(v, back);
    }
}
```

**Coroutine lifecycle:**

```rust
#[test]
fn test_coroutine_yield_resume() {
    let mut engine = Engine::new();
    engine.eval("fun count(): yield 1; yield 2; yield 3").unwrap();
    let mut co = engine.create_coroutine("count", &[]).unwrap();
    assert_eq!(co.resume(&mut engine), Ok(CoroutineStatus::Yielded(Value::Int(1))));
    assert_eq!(co.resume(&mut engine), Ok(CoroutineStatus::Yielded(Value::Int(2))));
    assert_eq!(co.resume(&mut engine), Ok(CoroutineStatus::Yielded(Value::Int(3))));
    assert_eq!(co.resume(&mut engine), Ok(CoroutineStatus::Completed(Value::None)));
}
```

### 10.3 Mutation Testing

Target: 75% killed/missed ratio on `ruchy-embed/src/` via `cargo-mutants`.
Priority: `sandbox.rs` (escape-proof), `types.rs` (correctness), `coroutine.rs`
(state transitions).
