# Ruchy 5.0: The Sovereign Platform Language

**Version:** 1.0.0
**Status:** IN PROGRESS (alpha.1 started 2026-04-04)
**Date:** 2026-04-03

### Implementation Status (Ground Truth as of 2026-04-04)

| Milestone | Status | Evidence |
|-----------|--------|----------|
| Version bump 5.0.0-alpha.1 | DONE | `ruchy --version` → 5.0.0-alpha.1 |
| Parser: 7 new keywords | PARTIAL | requires/ensures/invariant/decreases (4.x), infra/signal/yield (5.0). All 7 reserved. |
| Unified decorator grammar | DONE (4.x) | `@decorator` and `#[attribute]` both parse |
| Feature gates (infra/simulation/shell-target) | DONE | Cargo.toml feature definitions |
| Optional deps (forjar/simular/bashrs) | DONE | Added as optional, feature-gated |
| New CLI subcommands | NOT STARTED | prove/infra/sim/widget/apr/model/purify/migrate-4to5 |
| Transpiler: contract → debug_assert! | NOT STARTED | contracts parse but don't transpile to assertions |
| trueno 0.16.5 upgrade | PARTIAL | 0.16 in Cargo.toml, spec says 0.16.5 |
| ruchy-embed Engine API | EXISTS (192 lines) | Engine/Value/compile/eval implemented |
| Stdlib bridges (forjar/simular/bashrs) | NOT STARTED | No bridge modules yet |

---

## 1. Version Rationale

**Current release:** v4.2.1 (transpiler-focused, stdlib bridges via trueno/aprender/presentar).

**Why 5.0, not 4.3:**

The nine-pillar architecture is a paradigm shift, not an incremental release. Ruchy 4.x
transpiles Ruchy source to Rust and delegates to crate bridges at the stdlib level. Ruchy 5.0
elevates those bridges to first-class language constructs -- new keywords, new decorator
grammar, new CLI subcommands, and a new embedding crate. This crosses every semantic
versioning boundary simultaneously:

| SemVer axis | What breaks |
|-------------|-------------|
| Parser | 7 new reserved keywords (`requires`, `ensures`, `invariant`, `decreases`, `infra`, `signal`, `yield`) |
| Cargo.toml | 3 new required crates, 2 upgraded major versions, 1 new workspace member |
| CLI | 10+ new subcommands (`ruchy prove`, `ruchy infra`, `ruchy sim`, `ruchy widget`, ...) |
| Runtime | Coroutine/yield semantics in interpreter and transpiler |

**Tagline:** *"Ruchy 5.0: The Sovereign Platform Language"*

A sovereign language owns its full stack -- from formal verification to infrastructure
provisioning, from ML training to UI rendering -- without shelling out to foreign toolchains.
Every pillar compiles to the same Rust backend, shares the same type system, and ships in
the same binary.

---

## 2. The Nine Pillars

Each pillar has a dedicated sub-spec in `docs/specifications/`. This table is the index.

| # | Pillar | Domain | Spec File | New Syntax | Crate | Version |
|---|--------|--------|-----------|------------|-------|---------|
| 1 | Correctness | Formal verification | `provable-contracts-language-integration.md` | `requires`/`ensures`/`invariant`/`decreases` | provable-contracts | workspace |
| 2 | Compute | SIMD/GPU numerics | `trueno-first-class-integration.md` | Array arithmetic emits SIMD | trueno | 0.16.5 |
| 3 | Infrastructure | IaC provisioning | `forjar-iac-language-integration.md` | `infra {}` blocks | forjar | 1.2 |
| 4 | Scripting | Shell transpilation | `bashrs-shell-transpilation-target.md` | `--target shell` | bashrs | 6.65.0 |
| 5 | Learning | ML training/inference | `aprender-deep-ml-integration.md` | `import ml` | aprender | 0.27.5 |
| 6 | Visualization | Reactive widgets/UI | `presentar-widget-integration.md` | `Column`/`Button`/`signal()` | presentar | 0.3.4 |
| 7 | Simulation | Discrete-event sim | `simular-simulation-integration.md` | `import sim` | simular | 0.3.1 |
| 8 | Testing | Property/mutation/fuzz | `probar-testing-integration.md` | `#[probar_test]` | jugar-probar | 1.0.4 |
| 9 | Embedding | Scripting engine API | (new -- see Section 7) | `Engine::new()` | ruchy-embed | 0.1.0 |

**Design invariant:** Every pillar is optional via Cargo feature gates except Pillar 1
(Correctness) and Pillar 2 (Compute), which are always-on because safety and performance
are non-negotiable.

---

## 3. Unified Decorator Grammar

The 8 existing sub-specs independently invented decorator syntax. Ruchy 5.0 unifies them
under one grammar with two forms:

| Form | Syntax | Semantics | Resolved at |
|------|--------|-----------|-------------|
| Decorator | `@name(args)` | Runtime behavior modification | Transpile time (emits wrapper code) |
| Attribute | `#[name(args)]` | Compile-time metadata | Parse time (AST annotation) |

**Decision rule:** Use `@decorator` for Ruchy-native features that alter generated code.
Use `#[attribute]` for metadata consumed by external tools or Rust interop.

### Complete Decorator/Attribute Map

| Annotation | Form | Pillar | Semantics |
|------------|------|--------|-----------|
| `@verified` | Decorator | 1 - Correctness | Enable contract checking for function |
| `@gpu` | Decorator | 2 - Compute | Emit GPU kernel via trueno |
| `@tuned(metric)` | Decorator | 5 - Learning | Hyperparameter auto-tuning |
| `@brick(name)` | Decorator | 2 - Compute | Register as ComputeBrick |
| `@anomaly_checked` | Decorator | 5 - Learning | Wrap with anomaly detection |
| `@falsifiable` | Decorator | 8 - Testing | Generate property-based counter-examples |
| `@quantized(bits)` | Decorator | 5 - Learning | Quantize model weights |
| `@pipeline` | Decorator | 5 - Learning | Define ML pipeline stage |
| `#[prove(level)]` | Attribute | 1 - Correctness | Set verification level (bronze/silver/gold) |
| `#[probar_test]` | Attribute | 8 - Testing | Mark as probar test case |
| `#[playbook(name)]` | Attribute | 8 - Testing | Attach test playbook |
| `#[brick_budget(ms)]` | Attribute | 2 - Compute | Set compute time budget |
| `#[zero_js]` | Attribute | 6 - Visualization | Verify no JS emission in widget |
| `#[infra_policy(p)]` | Attribute | 3 - Infrastructure | Attach compliance policy |

### Grammar (EBNF)

```ebnf
decorator    ::= '@' IDENT ( '(' arg_list ')' )?
attribute    ::= '#[' IDENT ( '(' arg_list ')' )? ']'
arg_list     ::= arg ( ',' arg )*
arg          ::= IDENT '=' literal | literal | IDENT
```

---

## 4. Parser Changes Summary

### New Reserved Keywords

| Keyword | Pillar | Context | Example |
|---------|--------|---------|---------|
| `requires` | 1 | Function precondition | `requires x > 0` |
| `ensures` | 1 | Function postcondition | `ensures result > 0` |
| `invariant` | 1 | Loop/class invariant | `invariant len > 0` |
| `decreases` | 1 | Termination measure | `decreases n` |
| `infra` | 3 | Infrastructure block | `infra { machine("web") { ... } }` |
| `signal` | 6 | Reactive state | `let count = signal(0)` |
| `yield` | 9 | Coroutine suspension | `yield value` |

### New Block Syntax

| Block | Pillar | Transpiles To |
|-------|--------|---------------|
| `infra { ... }` | 3 | `forjar::InfraSpec` builder |
| `Column { ... }` | 6 | `presentar::Column::new()` builder |
| `Row { ... }` | 6 | `presentar::Row::new()` builder |
| `contract { requires ...; ensures ... }` | 1 | `provable_contracts::Contract` |

### New Expression Forms

| Expression | Pillar | Example |
|------------|--------|---------|
| `signal(init)` | 6 | `let count = signal(0)` |
| `derived(fn)` | 6 | `let double = derived(|| count() * 2)` |
| Array arithmetic | 2 | `let c = a + b` (element-wise SIMD) |

### Migration Concern

Any user code using `requires`, `ensures`, `invariant`, `decreases`, `infra`, `signal`, or
`yield` as variable names will break. The `ruchy migrate-4to5` tool (Section 9) renames
these automatically.

---

## 5. Dependency Impact

### Cargo.toml Changes

| Crate | 4.x Version | 5.0 Version | Feature Gate | Always On | Est. Size Impact |
|-------|------------|-------------|--------------|-----------|------------------|
| trueno | 0.15 | 0.16.5 | -- | Yes | +200 KB (SIMD tables) |
| aprender | 0.26 | 0.27.5 | -- | Yes | +150 KB (quantize) |
| presentar | 0.3.1 | 0.3.4 | `widgets` | No | +80 KB |
| forjar | -- | 1.2 | `infra` | No | +300 KB |
| simular | -- | 0.3.1 | `simulation` | No | +120 KB |
| bashrs | -- | 6.65.0 | `shell-target` | No | +250 KB |
| jugar-probar | 1.0.2 | 1.0.4 | dev-dependency | N/A | 0 (test only) |
| ruchy-embed | -- | 0.1.0 | workspace member | N/A | +400 KB |
| provable-contracts | -- | workspace | -- | Yes | +100 KB |

### Default Feature Set

```toml
[features]
default = []  # Minimal: correctness + compute only
full = ["widgets", "infra", "simulation", "shell-target"]
widgets = ["dep:presentar"]
infra = ["dep:forjar"]
simulation = ["dep:simular"]
shell-target = ["dep:bashrs"]
```

### Binary Size Budget

| Configuration | 4.x Baseline | 5.0 Target | Delta |
|---------------|-------------|------------|-------|
| Default features | 12.4 MB | 12.9 MB | +4% |
| Full features | N/A | 14.8 MB | +19% |

Hard limit: `full` build must stay under +20% of 4.x default.

---

## 6. CLI Architecture v5.0

### New Subcommands

| Subcommand | Pillar | Description |
|------------|--------|-------------|
| `ruchy prove` | 1 | Run contract verification (bronze/silver/gold) |
| `ruchy contracts sync` | 1 | Generate YAML contract manifests |
| `ruchy suggest-contracts` | 1 | LLM-inferred contract suggestions |
| `ruchy infra plan` | 3 | Preview infrastructure changes |
| `ruchy infra apply` | 3 | Apply infrastructure changes |
| `ruchy infra drift` | 3 | Detect configuration drift |
| `ruchy infra status` | 3 | Show current infrastructure state |
| `ruchy infra destroy` | 3 | Tear down infrastructure |
| `ruchy purify` | 4 | Analyze/clean legacy shell scripts |
| `ruchy sim run` | 7 | Execute simulation |
| `ruchy sim inspect` | 7 | Inspect simulation state |
| `ruchy sim verify` | 7 | Verify simulation invariants |
| `ruchy sim export` | 7 | Export simulation results |
| `ruchy widget serve` | 6 | Dev server for widget preview |
| `ruchy widget build` | 6 | Production widget build |
| `ruchy widget test` | 6 | Widget visual regression tests |
| `ruchy widget inspect` | 6 | Widget tree inspector |
| `ruchy apr run` | 5 | Train/run ML model |
| `ruchy apr serve` | 5 | Serve model via HTTP |
| `ruchy apr quantize` | 5 | Quantize model weights |
| `ruchy apr inspect` | 5 | Inspect model architecture |
| `ruchy apr bench` | 5 | Benchmark model performance |
| `ruchy apr eval` | 5 | Evaluate model accuracy |
| `ruchy model save` | 5 | Save model checkpoint |
| `ruchy model load` | 5 | Load model checkpoint |
| `ruchy model export` | 5 | Export to ONNX/SafeTensors |
| `ruchy model import` | 5 | Import from ONNX/SafeTensors |
| `ruchy model inspect` | 5 | Inspect model metadata |
| `ruchy model verify` | 5 | Verify model integrity |
| `ruchy test --probar` | 8 | Run probar test suite |
| `ruchy test --playbook` | 8 | Run test playbook |
| `ruchy test --visual-regression` | 8 | Visual regression testing |
| `ruchy test --mutations` | 8 | Mutation testing |
| `ruchy migrate-4to5` | -- | Automated 4.x to 5.0 migration |

### Subcommand Conflict Avoidance

New subcommands must not shadow user script names. Rule: if `ruchy foo` is invoked and
`foo.ruchy` exists in the current directory, the script takes precedence. The subcommand
is available via `ruchy --builtin foo`.

---

## 7. The "Graduate" Workflow

This is Ruchy's unique value proposition. The same source file works in three execution
modes with zero modifications:

```
  +-------------------+     +-------------------+     +-------------------+
  |   1. INTERPRET    | --> |    2. EMBED       | --> |    3. COMPILE     |
  |   ruchy repl      |     |   ruchy-embed     |     |   ruchy compile   |
  |   Instant feedback|     |   Hot reload      |     |   Native binary   |
  |   Prototyping     |     |   Sandboxing      |     |   Zero overhead   |
  +-------------------+     +-------------------+     +-------------------+
```

### Stage 1: REPL (Interpreter)

```bash
$ ruchy repl
>>> fun greet(name: str) -> str:
...     return f"Hello, {name}!"
>>> greet("world")
"Hello, world!"
```

Instant iteration. No compilation. Full stdlib access. Ideal for prototyping.

### Stage 2: Embed (ruchy-embed)

```rust
use ruchy_embed::{Engine, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::new();           // < 5ms startup
    engine.load_file("greet.ruchy")?;     // Hot-reloadable
    let result = engine.call("greet", &[Value::Str("world".into())])?;
    println!("{}", result);               // "Hello, world!"
    Ok(())
}
```

Embed Ruchy as a scripting layer in any Rust application. Sandboxed execution, capability-
based permissions, hot reload without restart.

### Stage 3: Compile (Transpiler)

```bash
$ ruchy compile greet.ruchy -o greet
$ ./greet world
Hello, world!
```

Full transpilation to Rust, then `rustc` compilation. Zero runtime overhead. Same
performance as hand-written Rust.

### Why This Matters

No other language offers all three modes from a single source file:

| Language | Interpret | Embed | Compile |
|----------|-----------|-------|---------|
| Python | Yes | Yes (CPython API) | No (Cython is different source) |
| Lua | Yes | Yes | No |
| Rust | No | N/A | Yes |
| Go | No | No | Yes |
| **Ruchy** | **Yes** | **Yes** | **Yes** |

---

## 8. Release Plan

| Milestone | Target | Scope | Gate |
|-----------|--------|-------|------|
| 5.0-alpha.1 | 2026-Q2 | Parser: 7 new keywords + unified decorator grammar. Provable contracts Silver level. | All 4.x tests pass. New keyword parser tests green. |
| 5.0-alpha.2 | 2026-Q2 | trueno 0.16.5 upgrade. Transpiler emits SIMD for array ops. | trueno bridge tests. Benchmark no regression. |
| 5.0-alpha.3 | 2026-Q3 | ruchy-embed crate: `Engine` API, sandboxing, coroutines/`yield`. | Embed smoke tests. Startup < 5ms. |
| 5.0-beta.1 | 2026-Q3 | aprender 0.27.5 + presentar 0.3.4 widgets + simular 0.3.1 bridge. | ML pipeline e2e. Widget render tests. |
| 5.0-beta.2 | 2026-Q4 | forjar 1.2 + bashrs 6.65.0 + jugar-probar 1.0.4 integration. | `ruchy infra plan` e2e. Shell transpile tests. |
| 5.0-rc.1 | 2026-Q4 | Full integration testing. Doc generation. ruchy-book updates. | All 9 pillars pass acceptance. Book compiles. |
| 5.0.0 | 2027-Q1 | Release. | Success criteria (Section 10). |

### Parallel Workstreams

Alpha milestones are sequential (parser changes must land first). Beta milestones can
proceed in parallel across pillar teams. The RC phase is integration-only -- no new
features.

---

## 9. Breaking Changes and Migration

### What Breaks from 4.x

| Category | Breaking Change | Impact | Mitigation |
|----------|----------------|--------|------------|
| Keywords | 7 new reserved words | Code using `requires`, `ensures`, `invariant`, `decreases`, `infra`, `signal`, `yield` as identifiers | `ruchy migrate-4to5` auto-renames |
| Crate API | trueno 0.15 to 0.16.5 | Bridge function signatures may change | Changelog documents all changes |
| Crate API | aprender 0.26 to 0.27.5 | Quantization API restructured | Migration guide in aprender CHANGELOG |
| CLI | New subcommands | `ruchy prove`, `ruchy sim`, etc. may shadow user scripts | Script-first resolution (Section 6) |
| Runtime | `yield` keyword | Cannot use `yield` as variable name | `ruchy migrate-4to5` |
| Transpiler | Decorator grammar | Old `@decorator` usage without parens may parse differently | Warnings in 4.3 pre-release |

### Migration Tool

```bash
$ ruchy migrate-4to5 src/
Scanning 42 files...
  src/math.ruchy:12  renamed variable `requires` -> `requires_val`
  src/gen.ruchy:8    renamed variable `yield` -> `yield_val`
  src/infra.ruchy:3  renamed variable `infra` -> `infra_config`
Migration complete: 3 files modified, 3 identifiers renamed.
```

The migration tool is conservative: it only renames identifiers that conflict with new
keywords. It does not modify semantics.

### Deprecation Timeline

| Version | Action |
|---------|--------|
| 4.3.0 | Emit warnings for identifiers that will become keywords |
| 4.4.0 | Emit warnings for deprecated stdlib patterns |
| 5.0-alpha.1 | Keywords reserved; old code requires migration |
| 5.0.0 | Full break; `ruchy migrate-4to5` available |

---

## 10. Success Criteria

| # | Criterion | Threshold | Measurement |
|---|-----------|-----------|-------------|
| 1 | All 9 pillar specs pass acceptance | 100% | Each spec defines its own acceptance tests |
| 2 | ruchy-embed startup latency | < 5ms | Benchmark: `Engine::new()` on x86_64, release build |
| 3 | Zero regressions in 4.x test suite | 0 failures | `cargo test --all-features` (existing tests) |
| 4 | ruchy-book examples compile | 100% | `make validate-book` passes |
| 5 | WASM target functional | All WASM tests pass | `cargo test --target wasm32-unknown-unknown` |
| 6 | Binary size (default features) | < +20% vs 4.x | `ls -la target/release/ruchy` |
| 7 | Compile time (default features) | < +30% vs 4.x | `cargo build --release --timings` |
| 8 | TDG grade | >= A- across all new code | `pmat tdg . --min-grade A- --fail-on-violation` |
| 9 | Test coverage | >= 80% for new pillar code | `cargo llvm-cov` |
| 10 | Mutation coverage | >= 75% for new pillar code | `cargo mutants --file <pillar>` |
| 11 | Zero unsafe in transpiler output | 0 occurrences | `pmat query --literal "unsafe {" --files-with-matches` |
| 12 | `ruchy migrate-4to5` handles all keyword conflicts | 100% | Test suite with synthetic 4.x code |

### Go/No-Go Decision

Release 5.0.0 requires ALL 12 criteria met. Any single failure blocks the release.
The release committee (maintainers) reviews at the RC.1 milestone.

---

## Appendix A: Cross-Reference to Sub-Specs

Each pillar spec is self-contained but must conform to the unified grammar (Section 3)
and CLI architecture (Section 6) defined in this document. In case of conflict, this
document takes precedence.

| Pillar | Spec Path | Ticket Prefix |
|--------|-----------|---------------|
| 1 - Correctness | `docs/specifications/provable-contracts-language-integration.md` | CONTRACTS-XXX |
| 2 - Compute | `docs/specifications/trueno-first-class-integration.md` | TRUENO-XXX |
| 3 - Infrastructure | `docs/specifications/forjar-iac-language-integration.md` | FORJAR-XXX |
| 4 - Scripting | `docs/specifications/bashrs-shell-transpilation-target.md` | BASHRS-XXX |
| 5 - Learning | `docs/specifications/aprender-deep-ml-integration.md` | APRENDER-XXX |
| 6 - Visualization | `docs/specifications/presentar-widget-integration.md` | PRESENTAR-XXX |
| 7 - Simulation | `docs/specifications/simular-simulation-integration.md` | SIMULAR-XXX |
| 8 - Testing | `docs/specifications/probar-testing-integration.md` | PROBAR-XXX |
| 9 - Embedding | (to be created) | EMBED-XXX |
