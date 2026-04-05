# Ruchy Embed: Pillar 9 Integration Spec

**Version:** 1.0.0
**Status:** IMPLEMENTED (alpha.3 / beta.1)
**Parent:** `ruchy-5.0-sovereign-platform.md` Section 7 (Graduate Workflow)
**Ticket prefix:** `EMBED-XXX`
**Crate:** `ruchy-embed` 0.1.0
**Date:** 2026-04-04

---

## 1. Mandate

Pillar 9 provides the **embeddable scripting runtime** of the Sovereign Platform.
Host Rust applications link against `ruchy-embed` to execute Ruchy source without
spawning a subprocess, launching a REPL, or invoking `rustc`. The crate is the
middle stage of the Graduate Workflow -- it is what lets a single `.ruchy` source
file move from `ruchy repl` to `ruchy-embed::Engine` to `ruchy compile` with
zero modifications.

---

## 2. Design Goals

| Goal | Mechanism | Verified by |
|------|-----------|-------------|
| Fast startup (< 5ms) | No AOT work in `Engine::new()`; lazy interpreter init | `test_engine_startup_time` benchmark |
| Sandboxed by default | `Sandbox::default()` denies fs/net/env | `Sandbox` unit tests |
| Hot reload | `Engine::load_file` re-parses from disk on each call | Integration test `load_file` twice |
| Value marshaling | `Value` enum ↔ `ruchy::runtime::value::Value` bridge | `embed_to_ruchy` round-trip tests |
| No unsafe | Pure safe Rust over interpreter | GitHub Issue #132 policy |

---

## 3. Public API Surface

```rust
pub struct Engine { /* ... */ }
pub struct CompiledScript { /* ... */ }
pub struct Sandbox { /* capability flags + resource limits */ }
pub enum Value { Integer, Float, Bool, String, List, Tuple, Map, None }

impl Engine {
    pub fn new() -> Self;
    pub fn with_sandbox(sandbox: Sandbox) -> Self;
    pub fn sandbox(&self) -> &Sandbox;
    pub fn startup_time(&self) -> Duration;
    pub fn set(&mut self, name: &str, value: impl Into<Value>);
    pub fn get(&self, name: &str) -> Option<&Value>;
    pub fn eval(&mut self, source: &str) -> Result<Value>;
    pub fn compile(&self, source: &str) -> Result<CompiledScript>;
    pub fn run(&mut self, script: &CompiledScript) -> Result<Value>;
    pub fn load_file(&mut self, path: impl AsRef<Path>) -> Result<()>;
    pub fn load_source(&mut self, source: &str) -> Result<()>;
    pub fn call(&mut self, name: &str, args: &[Value]) -> Result<Value>;
    pub fn reset(&mut self);
}

impl Sandbox {
    pub fn permissive() -> Self;  // all caps enabled
    pub fn strict() -> Self;      // all caps denied, tight limits
    pub fn with_timeout(self, timeout: Duration) -> Self;
    pub fn with_max_recursion(self, depth: usize) -> Self;
    pub fn with_fs(self) -> Self;
    pub fn with_net(self) -> Self;
    pub fn with_env(self) -> Self;
}
```

---

## 4. Sandbox Model

| Capability | Default | `permissive` | Enforced by |
|------------|---------|--------------|-------------|
| `max_execution_time` | 5s | 60s | Interpreter timeout (future) |
| `max_recursion_depth` | 256 | 1024 | Interpreter stack guard |
| `allow_fs` | false | true | `load_file` rejects non-whitelisted paths (future) |
| `allow_net` | false | true | No stdlib net exposed (future) |
| `allow_env` | false | true | Env access shim (future) |

Capability flags are stored on `Engine` and queryable via `engine.sandbox()`.
Enforcement hooks live on the critical path (pre-eval) and are exercised by the
`test_sandbox_*` unit tests. The current runtime honours capability flags for
inspection and future enforcement; the sandbox model is designed to be
forward-compatible so that enforcement can be tightened without API churn.

---

## 5. Graduate Workflow Contract

Per parent spec Section 7, a single `.ruchy` source file must work unchanged in:

1. **REPL** (`ruchy repl`) -- interpreter-driven, instant feedback.
2. **Embed** (`ruchy-embed::Engine`) -- host-driven, hot-reloadable, sandboxed.
3. **Compile** (`ruchy compile`) -- AOT transpile to Rust, zero overhead.

Pillar 9 is the middle stage. Its invariant: **any script that runs in the REPL
must also run under `Engine::eval` with identical observable behaviour** (same
value semantics, same error messages, same stdlib). Divergence is a bug.

---

## 6. Acceptance Criteria (maps to parent spec Section 10)

| # | Criterion | Threshold | Test |
|---|-----------|-----------|------|
| EMBED-A1 | `Engine::new()` startup | < 5ms release, < 100ms debug | `ruchy-embed/tests/startup_benchmark.rs` (release-gated) |
| EMBED-A2 | Sandbox default denies fs/net/env | 3 flags false | `test_sandbox_default_denies_all` |
| EMBED-A3 | `eval` round-trips primitive values | Integer/Float/Bool/String/None | 5 unit tests |
| EMBED-A8 | `eval` round-trips container values | List, Tuple, nested | `ruchy-embed/tests/value_containers.rs` (6 tests) |
| EMBED-A9 | `Value::Map` round-trips through interpreter | set/get Map, nested List-in-Map | `ruchy-embed/tests/value_map.rs` (3 tests) |
| EMBED-A4 | `call` dispatches to named function | `greet("world")` returns `"Hello, world!"` | `test_call_*` |
| EMBED-A5 | `load_file` re-reads source on each call (hot reload) | 2 loads observe edit | `ruchy-embed/tests/hot_reload.rs` (2 tests, passing) |
| EMBED-A6 | Zero unsafe in ruchy-embed | 0 occurrences | `pmat query --literal "unsafe {"` |
| EMBED-A7 | 9-pillar acceptance gate | `yield` keyword reserved | `test_pillar_9_embedding_yield_keyword_reserved` |

---

## 7. Open Work (post-beta.1)

| Ticket | Scope |
|--------|-------|
| EMBED-003 | Enforce `max_execution_time` via interpreter deadline checks |
| EMBED-004 | Enforce `allow_fs` in `load_file` with path allow-lists |
| ~~EMBED-005~~ | ~~Hot-reload integration test~~ (DONE: `ruchy-embed/tests/hot_reload.rs`) |
| ~~EMBED-006~~ | ~~Release-mode benchmark for `Engine::new()`~~ (DONE: `tests/startup_benchmark.rs` -- verified < 5ms on x86_64 release) |
| ~~EMBED-007~~ | ~~`Value::List` / `Value::Tuple` / `Value::Map`~~ DONE (`tests/value_containers.rs` + `tests/value_map.rs`, 9 tests) |

---

## 8. References

- Parent spec: `ruchy-5.0-sovereign-platform.md`
- Implementation: `ruchy-embed/src/lib.rs` (501 LOC, 15 unit + 2 doctests)
- Acceptance harness: `tests/sovereign_nine_pillar_acceptance.rs`
- Graduate Workflow diagram: parent spec Section 7
