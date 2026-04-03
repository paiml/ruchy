# Lua Embeddability Analysis: Lessons for Ruchy

## Research Report -- 2026-04-03

---

## 1. What Makes Lua Successful

Lua was born in 1993 at PUC-Rio (Brazil) from a specific constraint: Brazil's market
reserve policies prevented importing foreign software, forcing engineers to build their
own tools. The resulting design philosophy -- **mechanism over policy** -- produced a
language that provides extensible primitives rather than fixed features. This single
decision explains most of Lua's success.

**Key sources consulted:**
- https://www.lua.org/about.html (official design rationale)
- https://en.wikipedia.org/wiki/Lua_(programming_language) (adoption data)
- https://luau.org/ (Roblox's modern Lua derivative)
- https://luajit.org/luajit.html (JIT performance characteristics)
- https://docs.rs/mlua/latest/mlua/ (Rust-Lua bindings reference)
- https://rhai.rs/book/about/features.html (competing Rust-embeddable language)
- https://www.arewegameyet.rs/ecosystem/scripting/ (Rust game scripting ecosystem)
- https://luau.org/sandbox (Luau sandboxing model)
- https://luau.org/performance (Luau interpreter performance)
- https://luau.org/types (Luau gradual type system)

### Adoption Proof Points

| Domain | Users |
|--------|-------|
| Game engines | Roblox (Luau), World of Warcraft, Dota 2, Crysis, Garry's Mod |
| Infrastructure | Redis (server-side scripting), Nginx/OpenResty, ScyllaDB |
| Developer tools | Neovim (replaced VimScript), Wireshark, MediaWiki |
| Creative software | Adobe Lightroom, iClone |
| Operating systems | FreeBSD, NetBSD (system configuration) |

---

## 2. The Twelve Lua Benefits: Analysis and Ruchy Mapping

### 2.1 Tiny Footprint

**What makes it work in Lua:**
Lua 5.5.0 is ~32,000 lines of ANSI C. The compiled interpreter is 293KB and the library
is 484KB on 64-bit Linux. The entire tarball is 388KB compressed. This means Lua can be
statically linked into any application with negligible binary size impact.

**How Ruchy compares:**
Ruchy's compiler is ~460,000 lines of Rust -- roughly 14x larger than Lua. This is not
directly comparable because Ruchy is a transpiler+compiler+REPL+notebook+toolchain, not
just a language runtime. The critical question is: what is the minimum viable subset
needed for embedding?

**What Ruchy already has:**
- The transpiler produces standalone Rust code with zero runtime dependency
- WASM compilation target (`ruchy-wasm` crate) already strips the runtime

**What is missing and how to add it:**
A `ruchy-embed` crate that contains ONLY: lexer + parser + transpiler (no REPL, no
notebook, no CLI, no quality tooling). Estimated size: 30K-50K lines of Rust, which
would compile to a library of roughly 1-3MB. This is larger than Lua but comparable to
other Rust-native scripting solutions like Rhai.

**Actionable recommendation:**
Create a `ruchy-core` crate that extracts `frontend/` + `backend/transpiler/` +
`middleend/` into a minimal, feature-gated library. Target: <100KB WASM, <2MB native.

---

### 2.2 Embeddability

**What makes it work in Lua:**
Lua's C API uses a stack-based interface where the host pushes arguments, calls
functions, and pops results. The `lua_State` is fully self-contained -- multiple
independent Lua interpreters can coexist in a single process. The API resolves three
fundamental impedance mismatches: garbage-collected vs manual memory, dynamic vs static
types, and Lua values vs C values.

The Lua interpreter itself is under 500 lines of C wrapping the library. The library IS
the language. This means any C/C++ program can become a Lua host with minimal effort.

**How Ruchy compares:**
Ruchy's transpile-to-Rust architecture is fundamentally different. Instead of embedding
an interpreter, you embed a compiler. This has different tradeoffs:

| Aspect | Lua (interpreter) | Ruchy (transpiler) |
|--------|-------------------|-------------------|
| Startup latency | Microseconds | Compilation time (seconds) |
| Runtime performance | Interpreted (or JIT) | Native Rust speed |
| Memory overhead | Lua VM + GC | Zero (compiled away) |
| Hot code loading | Trivial (load string) | Requires recompilation |
| Host interaction | Stack-based C API | Rust trait system |
| Sandboxing | Environment tables | Rust type system |

**What Ruchy already has:**
- Transpiler that produces standalone Rust source
- WASM compilation for browser-based execution
- WIT interface generation (`src/wasm/wit.rs`)
- Module system with imports

**What is missing and how to add it:**
A `ruchy-embed` crate that provides:
```rust
pub struct RuchyEngine {
    transpiler: Transpiler,
    // Pre-compiled module cache
    cache: HashMap<String, CompiledModule>,
}

impl RuchyEngine {
    /// Create a new Ruchy scripting engine
    pub fn new() -> Self;

    /// Register a Rust function callable from Ruchy scripts
    pub fn register_fn<F>(&mut self, name: &str, f: F)
    where F: Fn(&[Value]) -> Result<Value>;

    /// Evaluate a Ruchy expression, returning transpiled Rust
    pub fn transpile(&self, source: &str) -> Result<String>;

    /// Compile and execute a Ruchy script (requires rustc)
    pub fn eval(&self, source: &str) -> Result<Value>;

    /// Load a Ruchy module from a file
    pub fn load_module(&mut self, path: &Path) -> Result<()>;
}
```

The key insight: Ruchy cannot match Lua's interpreter-based embedding model directly.
Instead, Ruchy should offer **two embedding modes**:

1. **Transpile mode**: Generate Rust source at build time (zero runtime overhead,
   suitable for game scripting where scripts are known at compile time)
2. **Interpret mode**: Use the existing bytecode interpreter (`src/runtime/`) for
   dynamic scripting (matches Lua's model but with Ruchy syntax)

---

### 2.3 Fast Startup

**What makes it work in Lua:**
Lua starts in microseconds because it only needs to allocate a `lua_State` and
initialize a small set of standard library functions. There is no JIT warmup, no
bytecode verification pass, and no complex initialization.

**How Ruchy compares:**
Ruchy has three execution paths with different startup characteristics:

| Path | Startup | Use case |
|------|---------|----------|
| REPL interpreter | ~10ms (per design spec) | Interactive development |
| Transpile + rustc | Seconds | Production compilation |
| WASM | ~50ms (module instantiation) | Browser execution |
| JIT (Cranelift, POC) | ~10ms + compilation | Hot path optimization |

**What Ruchy already has:**
- The bytecode interpreter (`src/runtime/bytecode/`) provides fast startup for
  interpreted execution
- The REPL targets <10ms startup

**What is missing and how to add it:**
For embedding, the interpret mode should be the default for interactive/scripting use
cases. The Cranelift JIT (`src/jit/`) could provide a tiered approach: interpret first
(microsecond startup), JIT-compile hot functions (millisecond warmup). This is exactly
the model LuaJIT uses, and Ruchy's architecture already supports it.

**Actionable recommendation:**
Finish the JIT POC (`src/jit/mod.rs`, currently Phase 1). Implement tiered execution:
bytecode interpret -> profile -> Cranelift JIT for hot functions. Target: <1ms cold
start for interpreted mode, <10ms for JIT warmup.

---

### 2.4 Simple Host API (C API equivalent)

**What makes it work in Lua:**
Lua's stack-based C API is famous for its simplicity. All data exchange happens through
a virtual stack. The host pushes values, calls functions, and pops results. This design
elegantly handles the impedance mismatch between garbage-collected Lua and manually-
managed C memory.

Key API patterns:
```c
lua_State *L = luaL_newstate();    // Create interpreter
luaL_openlibs(L);                   // Load standard library
luaL_dostring(L, "x = 42");       // Execute Lua code
lua_getglobal(L, "x");             // Get value
int x = lua_tointeger(L, -1);     // Convert to C type
lua_pop(L, 1);                     // Clean up stack
```

**How Ruchy compares:**
Ruchy does not currently have a host API. The closest analog is the `mlua` crate
(4.2M downloads) which provides safe Rust bindings to Lua. Ruchy should learn from
mlua's API design, which uses Rust's type system instead of a stack:

```rust
// mlua approach (what Ruchy should emulate)
let lua = Lua::new();
lua.globals().set("x", 42)?;
let result: i32 = lua.load("return x + 1").eval()?;
```

**What Ruchy already has:**
- The interpreter (`src/runtime/`) can evaluate expressions and return `Value` types
- The `Value` enum already handles type conversion between Ruchy and Rust
- WASM bindings (`src/wasm_bindings.rs`) show how to expose Ruchy to external callers

**What is missing and how to add it:**
Design a `ruchy-embed` API that follows mlua's pattern but leverages Ruchy's
transpile-to-Rust architecture:

```rust
use ruchy_embed::Engine;

let mut engine = Engine::new();

// Register Rust functions
engine.register_fn("greet", |name: String| -> String {
    format!("Hello, {}!", name)
});

// Execute Ruchy code that calls the registered function
let result: String = engine.eval(r#"
    let msg = greet("World")
    msg
"#)?;

// Or: transpile to Rust source for AOT compilation
let rust_code = engine.transpile("let x = 2 + 2")?;
```

**Actionable recommendation:**
Create `ruchy-embed` as a new workspace member crate. The API should support:
- `Engine::new()` -- create a scripting engine instance
- `engine.register_fn()` -- expose Rust functions to scripts
- `engine.register_type::<T>()` -- expose Rust types with methods
- `engine.eval::<T>()` -- interpret and return typed result
- `engine.transpile()` -- generate Rust source code
- `engine.compile()` -- transpile + rustc for native performance
- Multiple independent `Engine` instances (like multiple `lua_State`)

---

### 2.5 Coroutines

**What makes it work in Lua:**
Lua implements **asymmetric coroutines** (also called semi-coroutines). Each coroutine
has its own stack, local variables, and instruction pointer, but shares globals with
other coroutines. Key characteristics:

- `coroutine.create(f)` creates a coroutine from a function
- `coroutine.resume(co, ...)` starts/continues execution
- `coroutine.yield(...)` suspends execution, returning values to the resumer
- Only one coroutine runs at a time (cooperative, not preemptive)
- Coroutines enable producer-consumer patterns, iterators, and state machines

Lua coroutines are "first-class" -- they can be stored in variables, passed as
arguments, and returned from functions. This makes them a powerful building block for
async I/O, game AI, and workflow engines.

**How Ruchy compares:**
Ruchy has `async/await` syntax (example `09_async_await.ruchy`) and an actor model
(example `21_concurrency.ruchy`) but does not have Lua-style coroutines.

| Feature | Lua | Ruchy |
|---------|-----|-------|
| Coroutines (yield/resume) | First-class | Not implemented |
| Async/await | Via coroutines | Yes (transpiles to Rust async) |
| Actors | Not built-in | Yes (actor keyword) |
| Channels | Not built-in | Yes (via actor messages) |
| Threads | Via coroutines + C | Via Rust std::thread |

**What Ruchy already has:**
- `async fn`, `await` syntax in the AST (`ExprKind::Await`, `ExprKind::Spawn`,
  `ExprKind::AsyncBlock`)
- Actor model with message passing (`actor Counter { handler Increment... }`)
- The bytecode VM could support coroutines (it already has stack frames)

**What is missing and how to add it:**
Lua-style coroutines would map naturally to Rust generators (unstable) or to
`async/await` with manual poll. For embedding, coroutines are essential because they
let the host control execution granularity:

```ruchy
// Proposed Ruchy coroutine syntax
let co = coroutine fun() {
    yield 1
    yield 2
    yield 3
}

for value in co {
    println(value)  // 1, 2, 3
}
```

This would transpile to Rust iterators or `async` streams.

**Actionable recommendation:**
Implement coroutines as syntactic sugar over Rust iterators. `yield` becomes an iterator
`next()` return, and `coroutine fun` becomes a struct implementing `Iterator`. For async
coroutines, transpile to `Stream`. This gives Ruchy coroutine ergonomics with zero-cost
Rust performance.

---

### 2.6 Metatables

**What makes it work in Lua:**
Metatables are Lua's single extensibility mechanism. Any table can have a metatable that
intercepts operations via metamethods:

| Metamethod | Intercepts |
|-----------|-----------|
| `__index` | Property access (missing keys) |
| `__newindex` | Property assignment |
| `__call` | Function call syntax on non-functions |
| `__add`, `__sub`, etc. | Arithmetic operators |
| `__tostring` | String conversion |
| `__len` | Length operator |
| `__eq`, `__lt`, `__le` | Comparison operators |
| `__gc` | Garbage collection finalization |

This single mechanism enables: classes, inheritance, proxies, read-only tables, default
values, operator overloading, and domain-specific languages. It is the embodiment of
"mechanism over policy."

**How Ruchy compares:**
Ruchy uses Rust's trait system for similar functionality, which is more structured but
less flexible:

| Lua metatable | Ruchy equivalent |
|---------------|-----------------|
| `__index` | Struct fields, `impl` blocks |
| `__newindex` | `mut` fields |
| `__call` | `Fn` trait (via closures) |
| `__add` etc. | `std::ops::Add` trait (via transpilation) |
| `__tostring` | `Display` trait |
| `__eq` | `PartialEq` trait |

**What Ruchy already has:**
- Classes with `impl` blocks (example `12_classes_structs.ruchy`)
- Operator overloading (transpiles to Rust `std::ops` traits)
- Pattern matching for structural dispatch
- Pipeline operator (`|>`) for method chaining

**What is missing and how to add it:**
Ruchy does not need metatables because Rust's trait system is strictly more powerful for
static dispatch. However, for dynamic embedding scenarios, Ruchy could add a
**protocol system** inspired by Python's dunder methods:

```ruchy
class Vector {
    x: Float
    y: Float

    // Protocol methods (like Lua metamethods / Python dunders)
    fun __add__(self, other: Vector) -> Vector {
        Vector { x: self.x + other.x, y: self.y + other.y }
    }

    fun __str__(self) -> String {
        f"({self.x}, {self.y})"
    }
}
```

This already partially exists via operator overloading in Ruchy. The gap is the dynamic
dispatch case -- making it work with the interpreter, not just the transpiler.

**Actionable recommendation:**
For the transpiler path, the current trait-based approach is correct and superior to
metatables. For the interpreter/embedding path, add a `protocols` map to the `Value`
enum that allows runtime dispatch of operators and property access. This gives Lua-like
flexibility when running in interpreted mode.

---

### 2.7 Tables as Universal Data Structure

**What makes it work in Lua:**
Lua's table is simultaneously: an array (integer keys), a hash map (any key), a
namespace (string keys), a record (fixed string keys), a set (keys with boolean values),
and an object (with metatable). One data structure, learned once, used everywhere.

This simplicity is a core reason for Lua's success in embedding: there is exactly one
thing to learn about data structures.

**How Ruchy compares:**
Ruchy has multiple data structure types:

| Lua | Ruchy |
|-----|-------|
| Table (array mode) | `Vec<T>` / `[1, 2, 3]` |
| Table (hash mode) | `HashMap<K,V>` / `{key: value}` |
| Table (record mode) | Structs / classes |
| Table (namespace) | Modules |
| Table (set mode) | `HashSet<T>` |

**What Ruchy already has:**
- Array literals: `[1, 2, 3]`
- Object/record literals: `{name: "Alice", age: 30}`
- DataFrame macro: `df![...]`
- Tuples: `(x, y)`

**What is missing and how to add it:**
Ruchy should NOT adopt Lua's single-table approach. Having distinct types (Vec, HashMap,
struct) is a strength because it enables Rust's zero-cost abstractions and compile-time
type checking. However, for the interpreter/embedding mode, Ruchy could provide a
unified `Table` type that acts like Lua's table but compiles to the appropriate Rust type
when transpiled:

```ruchy
// In interpreter mode: behaves like a Lua table
let t = {
    1: "first",          // array-like
    name: "Alice",       // record-like
    "key": "value",      // hash-like
}

// When transpiled: becomes a struct or HashMap depending on usage
```

**Actionable recommendation:**
Keep the current multi-type approach for transpilation (it produces better Rust code).
For the embedding/interpreter API, provide a `Value::Table` variant that can hold mixed
key-value pairs, similar to how serde_json::Value works. This gives embedders a
flexible interchange format without sacrificing transpilation quality.

---

### 2.8 Gradual Typing

**What makes it work in Lua/Luau:**
Standard Lua is dynamically typed. Roblox's Luau adds gradual typing with three modes:

- `--!nocheck`: No type analysis (fully dynamic)
- `--!nonstrict`: Unknown types default to `any` (permissive)
- `--!strict`: Full type tracking and error reporting

Luau uses **structural typing** (shape-based, not name-based) and provides type
inference so most code needs no annotations. Type annotations use colon syntax:
`local x: number = 42`. Type casts use `::` and are validated.

**How Ruchy compares:**
Ruchy already has gradual typing that is more advanced than Luau's:

| Feature | Luau | Ruchy |
|---------|------|-------|
| Type inference | Yes | Yes (bidirectional + HM) |
| Type annotations | `x: number` | `x: Int` |
| Generics | Yes | Yes |
| Union types | Yes | Yes |
| Refinement types | No | Yes (Z3-backed) |
| Effect tracking | No | Yes (IO, Async, Pure) |
| Linear/affine types | No | Yes |

**What Ruchy already has:**
- Full gradual type system with inference
- Type annotations are optional (Python-like ergonomics)
- Refinement types backed by Z3 SMT solver
- Effect system for tracking side effects

**What is missing:**
Ruchy's type system is already superior to Luau's. The gap is in **documentation and
developer experience**: Luau makes it trivially easy to start untyped and add types
incrementally. Ruchy should ensure the same experience.

**Actionable recommendation:**
Ensure that Ruchy scripts with ZERO type annotations work perfectly in both interpreter
and transpiler modes. The type inference engine should handle all common cases without
requiring annotations. Document the "add types incrementally" workflow prominently.

---

### 2.9 Hot Reloading

**What makes it work in Lua:**
Lua scripts can be loaded and replaced at runtime without restarting the host
application. The pattern:
```c
luaL_dofile(L, "script.lua");  // Load initial script
// ... later, when file changes:
luaL_dofile(L, "script.lua");  // Reload, replacing definitions
```

This works because Lua scripts define functions and values in a global environment that
can be overwritten. Neovim uses this extensively: editing your Lua config reloads it
instantly.

**How Ruchy compares:**
Ruchy already has a file watcher (`src/server/watcher.rs`) that detects file changes
with debouncing. However, the transpile-to-Rust model means hot reloading requires
recompilation:

| Approach | Latency | Suitable for |
|----------|---------|-------------|
| Lua reload | <1ms | Game scripting, config |
| Ruchy interpret | ~10ms | Development, prototyping |
| Ruchy transpile + rustc | Seconds | Production builds |
| Ruchy WASM reload | ~100ms | Browser applications |

**What Ruchy already has:**
- `FileWatcher` with debouncing (`src/server/watcher.rs`, feature-gated behind
  `watch-mode`)
- REPL with `:load` command for loading scripts
- WASM REPL for browser-based execution

**What is missing and how to add it:**
For the embedding use case, Ruchy needs a **interpreted hot-reload path**:

```rust
let mut engine = Engine::new();
engine.load_file("game_logic.ruchy")?;

// File watcher detects change
engine.reload_file("game_logic.ruchy")?; // Re-interpret, ~10ms

// Functions are updated, state is preserved
engine.call::<i32>("on_update", &[delta_time])?;
```

The bytecode interpreter already supports this conceptually. The gap is:
1. A stable module identity system (so reloaded modules replace the old version)
2. State preservation across reloads (keep global variables, replace functions)
3. An `engine.reload()` API that re-parses and re-interprets without restarting

**Actionable recommendation:**
Implement `Engine::reload_file()` that re-parses and updates the function table while
preserving global state. Use the existing bytecode interpreter for sub-10ms reload.
For production, offer a `--watch` flag that monitors files and triggers `transpile +
cargo build` automatically (the `FileWatcher` already exists).

---

### 2.10 Sandboxing

**What makes it work in Lua/Luau:**
Lua sandboxes code by manipulating the environment table. Luau goes further:

1. **Library restrictions**: `io`, `os`, `package` libraries removed entirely.
   `debug` limited to `traceback` and `info`. `loadstring` cannot process bytecode.
2. **Environmental isolation**: All standard libraries marked read-only. Scripts
   get their own global table that delegates to builtins via `__index`.
3. **Memory safety**: `__gc` metamethod removed (prevents use-after-free). Custom
   allocator tracks memory per-script.
4. **Runtime control**: Interrupt mechanism terminates runaway scripts at function
   calls and loop iterations. CPU limits without formal termination proofs.
5. **Bytecode validation**: `string.dump` and `load` removed to prevent smuggling
   invalid bytecode past the validator.

**How Ruchy compares:**
Ruchy has a fundamentally different security model because it transpiles to Rust:

| Security aspect | Lua | Ruchy |
|----------------|-----|-------|
| Memory safety | GC + careful C | Rust borrow checker |
| Type safety | Dynamic checks | Compile-time verification |
| Code injection | Environment control | Transpiler restricts output |
| Resource limits | Interrupt hooks | Not implemented |
| Library restriction | Environment tables | Feature gates |

**What Ruchy already has:**
- WASM sandbox (`src/notebook/testing/sandbox.rs`) for notebook execution
- Zero unsafe code policy (GitHub Issue #132)
- Rust's type system prevents memory corruption in transpiled output
- Feature-gated standard library modules

**What is missing and how to add it:**
For embedding untrusted scripts, Ruchy needs:

1. **Execution limits**: Maximum instruction count, maximum memory allocation,
   maximum recursion depth (partially exists via `RUNTIME-001`)
2. **API allowlisting**: A way for the host to specify which built-in functions
   are available to scripts
3. **No filesystem/network by default**: Scripts should not have `io` or `http`
   unless the host explicitly enables them
4. **Timeout enforcement**: Already mandated by project policy, needs API support

```rust
let mut engine = Engine::new();
engine.set_max_instructions(1_000_000);
engine.set_max_memory(10 * 1024 * 1024); // 10MB
engine.set_timeout(Duration::from_secs(5));
engine.allow_module("math");
engine.deny_module("io");
engine.deny_module("http");

// Untrusted code runs in sandbox
engine.eval_sandboxed(untrusted_source)?;
```

**Actionable recommendation:**
Implement a `SandboxConfig` that controls: allowed modules, instruction limits, memory
limits, and timeout. The bytecode interpreter should check limits at loop boundaries
and function calls (same as Luau's interrupt mechanism). For the transpiler path,
generate Rust code that includes resource-checking wrappers.

---

### 2.11 LuaJIT Performance

**What makes it work in LuaJIT:**
LuaJIT combines a hand-written assembly interpreter with a trace-based JIT compiler
using SSA-based optimizations. It achieves near-native performance on numeric code and
is consistently ranked as one of the fastest dynamic language runtimes.

Key technical details:
- Trace compiler identifies hot loops and compiles them to native code
- FFI library enables zero-overhead C function calls
- Register-based VM (shared with standard Lua) reduces stack copying
- Custom memory allocator for low overhead

Luau (Roblox) takes a different approach: a highly optimized C interpreter that achieves
performance comparable to LuaJIT's interpreter (without JIT). Luau's interpreter core
compiles to ~16KB on x64, fitting in half the instruction cache. Luau also compiles
950K lines of code per second on a single core.

**How Ruchy compares:**
Ruchy's transpile-to-Rust model is fundamentally faster than any JIT for steady-state
performance, because the output is statically compiled native code optimized by LLVM:

| Metric | LuaJIT | Luau | Ruchy (transpiled) | Ruchy (interpreted) |
|--------|--------|------|-------------------|-------------------|
| Startup | ~1ms | ~1ms | Seconds (rustc) | ~10ms |
| Steady-state | ~0.5-2x C | ~5-10x C | ~1.0x Rust (= C) | ~50-100x C |
| Memory | Low + GC | Low + GC | Zero overhead | Moderate |
| JIT warmup | ~10ms | N/A | N/A | N/A |

The Cranelift JIT POC (`src/jit/`) targets 50x improvement over the AST interpreter,
which would put it in the same ballpark as LuaJIT for numeric workloads.

**What Ruchy already has:**
- Transpiled output achieves 98-102% of hand-written Rust performance (per design spec)
- Cranelift JIT proof-of-concept (Phase 1, arithmetic only)
- Bytecode interpreter for fast-startup scenarios

**What is missing:**
The JIT is incomplete (Phase 1 of 3). For embedding, the performance story should be:
"Ruchy scripts interpret fast and compile to native speed."

**Actionable recommendation:**
Complete JIT Phase 2 (control flow, data structures) to enable a viable tiered execution
model: interpret -> JIT hot paths -> AOT compile for production. This matches LuaJIT's
strategy but with Rust as the final compilation target.

---

### 2.12 Portability

**What makes it work in Lua:**
Lua is written in ANSI C89, which compiles on virtually every platform with a C
compiler. The entire runtime is a single `.c` file plus headers that can be dropped into
any project. No build system, no dependencies, no configuration.

**How Ruchy compares:**
Ruchy targets Rust (widespread but not as universal as C) and WASM (for browser and
edge deployment). The portability matrix:

| Platform | Lua | Ruchy (transpiled) | Ruchy (WASM) |
|----------|-----|-------------------|-------------|
| Linux x86_64 | Yes | Yes | Yes |
| macOS ARM64 | Yes | Yes | Yes |
| Windows | Yes | Yes | Yes |
| Embedded (ARM) | Yes | Yes (cross-compile) | Maybe |
| Browser | Via Emscripten | No (needs rustc) | Yes |
| iOS/Android | Yes | Yes (cross-compile) | Yes |
| Microcontrollers | Yes (eLua) | No (too large) | No |

**What Ruchy already has:**
- WASM compilation target (`ruchy-wasm` crate)
- Portability analyzer (`src/wasm/portability.rs`)
- WIT interface generation for WASM component model
- Cross-platform Rust compilation via `cargo`

**What is missing:**
Ruchy cannot match Lua on deeply embedded platforms (microcontrollers, 8-bit systems).
This is an acceptable tradeoff -- Ruchy targets a different niche.

**Actionable recommendation:**
Focus portability efforts on WASM (broadest reach) and Tier 1 Rust targets. The WASM
path gives Ruchy universal browser deployment. Do not attempt to match Lua's
microcontroller portability; it is not the right design tradeoff for a
transpile-to-Rust language.

---

## 3. Practical Questions

### 3.1 Could Ruchy be used as an embedded scripting language in Rust applications?

**Yes, with the right architecture.** The model would be:

```
                    +-----------------------+
                    |   Host Rust App       |
                    |                       |
                    |  +--ruchy-embed----+  |
                    |  | Parser          |  |
                    |  | Type Checker    |  |
                    |  | Interpreter     |  |
                    |  | (opt) Transpiler|  |
                    |  +-----------------+  |
                    +-----------------------+
```

The `ruchy-embed` crate would contain the parser and bytecode interpreter for immediate
execution, with an optional transpiler for generating optimized Rust code at build time.

Comparison with existing Rust-embeddable options:

| Feature | mlua (Lua) | Rhai | Proposed ruchy-embed |
|---------|-----------|------|---------------------|
| Downloads (crates.io) | 4.2M | 5.8M | N/A (new) |
| Language familiarity | Lua syntax | JS+Rust syntax | Python syntax |
| Type safety | Dynamic | Dynamic | Gradual (optional types) |
| Performance (interpreted) | Good (LuaJIT: excellent) | Good | Good (bytecode VM) |
| Performance (compiled) | N/A | N/A | Excellent (native Rust) |
| Sandboxing | Environment tables | Engine restrictions | Module allowlisting |
| Async support | Via coroutines/mlua async | Limited | Native async/await |
| Dependencies | Lua C library | Minimal (6 crates) | Pure Rust |
| Binary size | ~500KB | ~1MB | ~2MB (estimated) |

Ruchy's unique selling point: it is the only option that can interpret for fast iteration
AND transpile to native Rust for production performance. No other embeddable scripting
language offers this dual-mode execution.

### 3.2 Could Ruchy scripts be hot-reloaded in a running application?

**Yes, using the bytecode interpreter path.** The workflow:

1. Host loads `script.ruchy` via `Engine::load_file()`
2. Parser produces AST, bytecode compiler generates bytecode
3. Interpreter executes bytecode
4. File watcher detects change (existing `FileWatcher`)
5. Engine re-parses and re-compiles bytecode (~10ms)
6. Function table is updated, global state preserved
7. Next host call to `engine.call()` uses new code

For the transpiler path, hot-reload means `transpile -> cargo build -> dlopen`, which
takes seconds. This is suitable for development (like `cargo-watch`) but not for
production hot-reload.

### 3.3 Could Ruchy achieve "configure once, script forever" for games?

**Yes, and better than Lua.** The pattern:

```
Build Time:                          Runtime:
.ruchy files                         Native speed
    |                                    ^
    v                                    |
ruchy transpile                      cargo build
    |                                    ^
    v                                    |
.rs files ───────────────────────> compiled into game binary
```

Game developers write Ruchy scripts. The build system transpiles them to Rust. The game
compiles with the transpiled code. Result: scripting ergonomics with zero runtime
overhead. No interpreter, no GC, no JIT warmup. This is strictly better than Lua for
known-at-build-time scripts.

For runtime-loaded scripts (user mods, live updates), use the interpreter path.

### 3.4 What would a `ruchy-embed` crate look like?

```rust
// Cargo.toml
[dependencies]
ruchy-embed = { version = "1.0", features = ["interpreter"] }
// Optional: features = ["transpiler", "jit", "async"]

// main.rs
use ruchy_embed::{Engine, Value, SandboxConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create engine with sandbox
    let sandbox = SandboxConfig::new()
        .allow_module("math")
        .allow_module("string")
        .deny_module("io")
        .deny_module("http")
        .max_instructions(1_000_000)
        .max_memory_bytes(10 * 1024 * 1024)
        .timeout(Duration::from_secs(5));

    let mut engine = Engine::with_sandbox(sandbox);

    // Register host functions
    engine.register_fn("get_player_health", |id: i64| -> f64 {
        game_state.get_player(id).health
    });

    engine.register_fn("damage_player", |id: i64, amount: f64| {
        game_state.get_player_mut(id).health -= amount;
    });

    // Load game scripts
    engine.load_file("scripts/game_logic.ruchy")?;

    // Game loop
    loop {
        let delta = get_delta_time();
        engine.call::<()>("on_update", &[Value::Float(delta)])?;
        engine.call::<()>("on_render", &[])?;

        // Hot reload on file change
        if file_watcher.changed("scripts/game_logic.ruchy") {
            engine.reload_file("scripts/game_logic.ruchy")?;
            println!("Scripts reloaded!");
        }
    }
}
```

---

## 4. Strategic Recommendations (Priority Order)

### P0: Create `ruchy-core` crate (Foundation)

Extract the minimal compilation pipeline into a standalone crate:
- `frontend/` (lexer + parser)
- `middleend/` (type inference)
- `backend/transpiler/` (code generation)
- `runtime/bytecode/` (interpreter)

This enables all other embedding work. Current estimate: the compiler is deeply
coupled to the monolithic `ruchy` crate. Extraction requires interface stabilization.

### P1: Implement `ruchy-embed` API

Build the `Engine` API described in Section 3.4 on top of `ruchy-core`. Start with
interpreter-only mode (fastest path to usability). Key deliverables:
- `Engine::new()` and `Engine::with_sandbox()`
- `engine.register_fn()` for host function binding
- `engine.eval()` and `engine.call()` for script execution
- `engine.load_file()` and `engine.reload_file()` for hot-reload

### P2: Complete JIT (Phase 2)

Finish the Cranelift JIT to enable tiered execution. This closes the performance gap
with LuaJIT for long-running scripts and makes the embedding story compelling:
"Start fast, run fast."

### P3: Implement coroutines

Add `yield`/coroutine syntax that transpiles to Rust iterators. This is essential for
game scripting (AI behavior trees, cutscenes, tutorials) and is one of Lua's most
beloved features.

### P4: Build sandbox infrastructure

Implement `SandboxConfig` with instruction limits, memory limits, timeout enforcement,
and module allowlisting. Required for running untrusted code (user mods, plugin systems).

### P5: Publish `ruchy-embed` to crates.io

Once P0-P4 are complete, publish with documentation, examples, and benchmarks comparing
to mlua and Rhai. Target the "Are We Game Yet?" scripting ecosystem listing.

---

## 5. Ruchy's Unique Advantages Over Lua

Despite Lua's dominance, Ruchy has structural advantages that Lua cannot match:

1. **Type safety at zero cost**: Gradual typing catches bugs at compile time without
   runtime overhead. Lua and LuaJIT are fully dynamic.

2. **Native Rust performance**: Transpiled Ruchy IS Rust. No interpreter overhead,
   no GC pauses, no JIT deoptimization. Lua can approach this with LuaJIT but never
   match it.

3. **Rust ecosystem access**: Transpiled Ruchy can use any crate on crates.io directly.
   Lua requires C bindings for every library.

4. **Memory safety guarantee**: Rust's borrow checker applies to transpiled output.
   Lua depends on careful C API usage to avoid memory bugs.

5. **Modern syntax**: Python-like syntax with pattern matching, pipeline operators,
   and async/await. Lua's syntax is rooted in 1993 design.

6. **Dual execution**: Interpret for development, compile for production. No other
   embeddable language offers this.

7. **WASM first-class**: Ruchy compiles to WASM natively. Lua requires Emscripten or
   Fengari (incomplete Lua in JS).

The pitch is not "Ruchy replaces Lua." It is: **"Ruchy gives you Lua's scripting
ergonomics with Rust's performance and safety guarantees."**

---

## 6. Competitive Landscape Summary

| Language | Footprint | Performance | Type Safety | Rust Integration | Hot Reload |
|----------|-----------|-------------|-------------|-----------------|------------|
| Lua 5.4 | 293KB | Good | None | Via mlua (FFI) | Excellent |
| LuaJIT | ~500KB | Excellent | None | Via mlua (FFI) | Excellent |
| Luau | ~500KB | Very Good | Gradual | Not available | Excellent |
| Rhai | ~1MB | Good | Dynamic | Native Rust | Good |
| Rune | ~1MB | Good | Dynamic | Native Rust | Good |
| **Ruchy** | ~2MB (est.) | **Excellent** (transpiled) | **Gradual** | **Native Rust** | Good (interpreter) |

Ruchy's position: larger footprint than Lua, but better performance (transpiled), better
type safety, and native Rust integration without FFI overhead.
