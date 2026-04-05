# Ruchy 5.0: The Sovereign Platform Language

**Version:** 1.0.0
**Status:** IN PROGRESS (beta.1 released 2026-04-04; rc.1 integration gate met)
**Date:** 2026-04-03

### Implementation Status (Ground Truth as of 2026-04-04)

| Milestone | Status | Evidence |
|-----------|--------|----------|
| Version bump 5.0.0-alpha.1 | DONE | `ruchy --version` → 5.0.0-alpha.1 |
| Parser: 7 new keywords | DONE | All 7 reserved in lexer (Token enum) and parser. 3 keyword reservation tests. |
| Parser: infra/signal/yield expressions | DONE | InfraBlock, Signal, Yield AST nodes parsed. 5 parser tests. |
| Unified decorator grammar | DONE (4.x) | `@decorator` and `#[attribute]` both parse |
| Feature gates (infra/simulation/shell-target) | DONE | Cargo.toml feature definitions |
| Optional deps (forjar/simular/bashrs) | DONE | Added as optional, feature-gated |
| New CLI subcommands | DONE | prove/infra/sim/widget/apr/model/purify/migrate-4to5/contracts/suggest-contracts all registered |
| CLI handler routing | DONE | All sovereign commands dispatched in command_router.rs (23 handler tests) |
| CLI integration tests | DONE | 29 assert_cmd tests for sovereign commands |
| Transpiler: contract → debug_assert! | DONE (Silver) | requires/ensures emit debug_assert! macros. 6 transpiler tests. |
| Transpiler: infra/signal/yield | DONE | InfraBlock→block, Signal→Signal::new(), Yield→yield |
| trueno 0.16.5 upgrade | PARTIAL | 0.16.5 in Cargo.lock, spec says 0.16.5 |
| ruchy-embed Engine API | DONE | Engine: new/eval/compile/load_file/load_source/call/reset. 15 tests + 2 doctests. |
| Stdlib bridges (forjar/simular/bashrs) | DONE | Bridge modules with types, builders, tests (24 tests). Feature-gated re-exports. |
| migrate-4to5 tool | DONE | Scans/renames keyword conflicts, dry-run support, 12 tests |
| 5.0 Examples | DONE | 30_contracts.ruchy, 31_sovereign_platform.ruchy, 32_migration_demo.ruchy |
| CHANGELOG 5.0 section | DONE | Full release notes for 5.0.0-alpha.1 |
| Test command: probar flags | DONE | --probar, --playbook, --visual-regression, --mutations registered |
| Contracts CLI (sync/list/check) | DONE | 3 subcommands + suggest-contracts + 8 handler tests |
| Alpha.1 gate: all 4.x tests pass | DONE | 20,429 lib tests pass, 0 failures |
| Alpha.2: trueno bridge expansion | DONE | 44 public functions (was 11), 44 tests. sub/div/fma/norms/trig/ML activations. |
| Alpha.2: array SIMD lowering module | DONE | array_simd.rs: try_lower_array_binary(), 10 tests. Lowers list+list to trueno. |
| Alpha.3: ruchy-embed sandboxing | DONE | Sandbox struct (timeout/recursion/fs/net/env). Engine::with_sandbox(). 21 tests. |
| Alpha.3: startup time tracking | DONE | Engine::startup_time(). Verified < 5ms target documented. |
| Criterion #2: release-mode startup < 5ms verified | DONE | ruchy-embed/tests/startup_benchmark.rs passes under `cargo test --release -- --ignored` |
| Alpha.3: yield/signal/infra in interpreter | DONE | Yield→Return(val), Signal→eval initial, InfraBlock→eval body. 20,456 tests pass. |
| 5.0 Examples (5 files) | DONE | 30_contracts, 31_sovereign_platform, 32_migration_demo, 33_simd_arrays, 34_embedding |
| Beta.1: aprender ML pipeline types | DONE | TrainingConfig, InferenceConfig, PipelineStage, TrainingResult. 19 tests. |
| Beta.1: presentar widget types | DONE | Widget (Column/Row/Text/Button/Input/Container), Alignment, RenderTarget. 31 tests. |
| Beta.2: shell transpilation types | DONE | ShellScript, ShellVar, QuoteStrategy. Injection-proof quoting. 16 tests. |
| Version bump to 5.0.0-beta.1 | DONE | Workspace + ruchy-wasm bumped 2026-04-04 |
| RC.1: 9-pillar acceptance gate | DONE | 16 integration tests in sovereign_nine_pillar_acceptance.rs (all passing) |
| Criterion #12: migrate-4to5 synthetic 4.x e2e | DONE | 4 tests in sovereign_migrate_4to5_e2e.rs covering all 7 keywords (all passing) |
| Criterion #11: zero unsafe in transpile output | DONE | 2 tests in sovereign_zero_unsafe_transpile.rs (required + sovereign examples, all passing) |
| Criterion #6: binary size (default) under +20% | DONE | 8.45 MB measured (target: <14.19 MB). 40% headroom. Automated in sovereign_binary_size_budget.rs. |
| Criterion #4: ruchy-book examples compile | PARTIAL | 15/16 critical chapters pass on 5.0.0-beta.1 (ch18 DataFrames failing — pre-existing, see DATAFRAMES-001). COMPILER-001 fix landed: ruchy compile now honours CARGO_TARGET_DIR. |

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

**Tagline:** *"Ruchy 5.0: The Sovereign Platform Language — provable by construction."*

A sovereign language owns its full stack -- from formal verification to infrastructure
provisioning, from ML training to UI rendering -- without shelling out to foreign toolchains.
Every pillar compiles to the same Rust backend, shares the same type system, and ships in
the same binary.

**Provability mandate** (see §14): Ruchy 5.x commits to **becoming one of the most
provable transpiled-to-Rust systems-scripting languages in the world** (bounded claim —
see §14.1 and §14.F-Audit-7). By 5.2, every non-test `fun` is required to carry a
contract; the build gate refuses to produce a binary when contracts are missing or stubbed.
This is the SPARK/GNATprove model, ported.

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
| 9 | Embedding | Scripting engine API | `ruchy-embed-pillar9-integration.md` | `Engine::new()` | ruchy-embed | 0.1.0 |

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
| 4 | Downstream book repos compile | 100% on each book | See Appendix B: seven book repos must pass on 5.0 |
| 5 | WASM target functional | All WASM tests pass | `cargo test --target wasm32-unknown-unknown` |
| 6 | Binary size (default features) | < +20% vs 4.x | `ls -la target/release/ruchy` |
| 7 | Compile time (default features) | < +30% vs 4.x | `cargo build --release --timings` |
| 8 | TDG grade | >= A- across all new code | `pmat tdg . --min-grade A- --fail-on-violation` |
| 9 | Test coverage | >= 80% for new pillar code | `cargo llvm-cov` |
| 10 | Mutation coverage | >= 75% for new pillar code | `cargo mutants --file <pillar>` |
| 11 | Zero unsafe in transpiler output | 0 occurrences | `pmat query --literal "unsafe {" --files-with-matches` |
| 12 | `ruchy migrate-4to5` handles all keyword conflicts | 100% | Test suite with synthetic 4.x code |
| 13 | Provability Mandate (F1-F5 in §14.5) | All 5 metrics in-range | See §14.5 falsifiability table |

### Go/No-Go Decision

Release 5.0.0 requires ALL 13 criteria met. Any single failure blocks the release.
The release committee (maintainers) reviews at the RC.1 milestone.

---

## 14. Provability Mandate

> *"It must be impossible to ship code that violates a contract.
>  Not difficult. Not caught in CI. Impossible. Like a type error."*
> — PAIML provable-contracts §13, ported here as core policy.

### 14.1 Vision (bounded claim)

**Present fact (5.0.0-beta.1):** Ruchy has 18 `#[contract]`-enforced functions
and a Silver-tier transpiler that emits `debug_assert!` from contract clauses.
This is not yet "one of the most provable languages" by any existing standard
(Lean's mathlib, CompCert, SPARK's Tokeneer kernel all dwarf us).

**Forward commitment:** By release 5.2, Ruchy commits to being
**one of the most provable transpiled-to-Rust systems-scripting languages**.
The scope is bounded deliberately: we do not compete with Lean or Coq on
mathematical depth; we compete on *pervasiveness* — every `pub fn` in
stdlib discharged, zero `#[contract_exempt]` escape-hatches in public API.

This section draws from six reference systems surveyed April 2026: Eiffel,
SPARK Ada, Verus, Lean 4, Liquid Haskell, and PAIML `provable-contracts`.
Consensus across 4 of 6: **compile-time gating + graduated proof strength
per function**. Liquid Haskell's 2025 adoption study (arXiv 2509.15005) warns
against pure refinement-type approaches due to ergonomic cliffs; we heed
that warning.

### 14.2 The Four Tiers

Every non-test `fun` in Ruchy is assigned exactly one tier. The tier is
written in the function signature (not as an attribute) so it is visible at
use-site and impossible to erase accidentally.

| Tier | Syntax | Discharge | When required |
|------|--------|-----------|---------------|
| **Bronze** | `@bronze fun f(x) { ... }` | `rustc` types only (baseline type safety) | Migration only; banned in stdlib after 5.2 |
| **Silver** | `fun f(x) requires P ensures Q { ... }` | `assert!` (release) + `debug_assert!` (debug) + Kani harness in CI | Default tier for all user code |
| **Gold** | `@gold fun f(x) requires P ensures Q { ... }` | SMT-discharged (Kani BMC) at compile time | Required for all `pub` stdlib fns |
| **Platinum** | `@platinum` + YAML contract + Lean theorem | Quorum: rustc + Kani + probar(10K) + Lean + human review | Required for safety-critical kernels |

**Default is Silver.** A bare `fun f(x) { body }` without `requires`/`ensures`
is a *Bronze* function and emits a warning. After version 5.2, unmarked `pub`
functions in stdlib are **compile errors** (see §14.6 deadline).

**Silver-is-not-stripped rule (§14.F-Audit-2 fix):** unlike Rust's
`debug_assert!`, Silver contracts emit `assert!` in release builds by
default. Authors may opt down to `debug_assert!` with an explicit
`@silver_debug_only` marker — which, critically, flips that function into
the Bronze tier in metric F1 (no hiding).

### 14.3 Escape-Proof Build Gate

Adapted from SPARK GNATprove and PAIML provable-contracts §13:

```
A. Parse:         fun f(...) requires P ensures Q { body }   ← §14.2 syntax
   ↓
B. AST:           Expr has non-empty `contracts: Vec<Contract>` slot
   ↓
C. Lint:          `ruchy contracts check` (pmat comply CB-1400)
   ↓ (must pass)
D. Tier resolve:  decides Bronze/Silver/Gold/Platinum from annotations
   ↓
E. Codegen:       Silver → debug_assert!, Gold → Kani harness, Platinum → YAML
   ↓
F. Release gate:  stdlib forbids Bronze, CI forbids >20% Bronze in user code
```

Skipping any stage = `ruchy check` exits non-zero = `rustc` is never invoked.
This is the SPARK "static prior to build" model, not the Eiffel "runtime flag"
model.

### 14.4 N-of-M Quorum for Platinum

A Platinum function is "discharged" only when **N of M independent oracles
agree**:

| Oracle | Technology | Verdict |
|--------|-----------|---------|
| 1 | `rustc` type checker | constraint satisfied |
| 2 | Kani BMC | no counter-example ≤ bound |
| 3 | Z3 SMT via Verus-style ghost | postcondition entailed |
| 4 | probar property test (10K cases) | no falsifier found |
| 5 | Lean 4 theorem (no `sorry`) | mathematically proved |
| 6 | Human review | LGTM with reason string |

**Stratified quorum (§14.F-Audit-4 fix):** oracles are grouped into three
epistemic strata. Discharge requires **≥1 vote from each stratum**:

| Stratum | Oracles | Epistemic source |
|---------|---------|------------------|
| Symbolic | rustc, Kani, Z3 | first-order logic over AST |
| Semantic | probar, Lean | random sampling / dependent types |
| Extrinsic | human review | out-of-band judgment |

A 3-of-6 quorum of {rustc, Kani, Z3} is **refused**: all three share the
same epistemic source. Minimum: 1 symbolic + 1 semantic + 1 extrinsic.
For `#[safety_critical]`: 2 symbolic + 2 semantic + 1 extrinsic (5 total).

**Anti-illusion safeguard** (see §14.5 Falsifier F3): pairwise verdict
correlation is continuously measured. If any two oracles exceed 0.95
correlation on a 100-kernel sample, they collapse to a single vote.

**Human-bottleneck acknowledgment:** the extrinsic stratum does not
scale. Platinum is therefore capped at ~200 functions/release (human
review budget). Non-safety-critical code uses Gold (no human stratum).

### 14.5 Falsifiability Commitments (Popper)

A provability claim that cannot be falsified is not science. We publish
three metrics in every release and pre-commit to the thresholds that
would refute the claim:

| # | Metric | Target | Falsifier (we're wrong if...) |
|---|--------|--------|-------------------------------|
| F1 | % of `fun` defs with SMT-non-trivial contracts (Z3 cannot prove `P ↔ true` from empty context in <100 ms) | ≥ 95 % at 5.2 | drops below 50 % — the "mandatory" gate became performative |
| F2 | Density of `#[contract_exempt]` escape-hatches per KLoC | ≤ 0.5 / KLoC | exceeds 5 / KLoC — the gate is being routinely bypassed |
| F3 | Pairwise oracle-verdict correlation on a 100-kernel sample | each pair < 0.95 | any pair ≥ 0.95 — quorum is an illusion |
| F4 | Stdlib Bronze-tier function count | 0 after 5.2 | ≥ 1 — the deadline slipped, stdlib isn't provable |
| F5 | Contract-free code ships to crates.io | 0 occurrences | ≥ 1 — the gate has a hole |

If any metric enters falsified range, the release committee MUST open a
spec-round ticket and cannot ship the next minor version until the metric
is restored.

### 14.6 Deadline Schedule

| Release | Gate level |
|---------|-----------|
| 5.0.0 | Silver *opt-in*. Bronze warned but allowed everywhere. |
| 5.1.0 | Silver *default*. Bronze warned in user code, errored in stdlib. |
| 5.2.0 | Silver *required* on all `pub` fns. Bronze banned in stdlib entirely. Gold required on `unsafe`-equivalent ops. |
| 5.3.0 | Platinum quorum available for any user fn. `#[safety_critical]` marker enforced at 5-of-6. |

### 14.7 Escape-Hatch Policy

One canonical hatch, and only one:

```rust
#[contract_exempt(reason = "ffi boundary with legacy C header",
                  until = "5.3.0",
                  ticket = "COMPAT-042")]
fun call_legacy() { ... }
```

Rules:
- `reason` is a **required** human-readable string.
- `until` is a **required** version pinning the exemption.
- `ticket` is a **required** tracking ID.
- Forbidden on `pub` functions (public API stays clean).
- Tracked by `pmat tdg` as a -5 grade penalty.
- Counted in F2 metric; CI fails if density exceeds threshold.
- **Time-bound enforcement (§14.F-Audit-3 fix):** `build.rs` compares `until`
  against `env!("CARGO_PKG_VERSION")` using semver. If the current version
  is ≥ the `until` version, build fails with a hard error pointing at the
  tracking ticket. Exemptions cannot silently outlive their deadline.

This is the explicit dual of SPARK's `pragma Annotate (GNATprove, False_Positive)`:
visible, justified, time-bounded, measured.

### 14.8 Why this makes Ruchy competitive

| Language | Contract model | Enforced at | Escape hatch |
|----------|---------------|-------------|--------------|
| Eiffel | require/ensure built in | runtime flag | compile option |
| SPARK Ada | pre/post/invariant | **compile-time (GNATprove)** | `pragma Annotate` |
| Verus | ghost code | compile-time SMT | `assume` |
| Liquid Haskell | refinement types | typecheck | `assume` |
| Lean 4 | dependent types | typecheck | `sorry` / `axiom` |
| **Ruchy 5.2+** | **signature-level tier + build gate** | **compile-time + build.rs** | **`#[contract_exempt]` with ticket + deadline** |

Ruchy's unique contribution: **the tier is visible in the function definition
(§14.F-Audit-1 correction)**. Callers discover it via `ruchy contracts show f`
or tooling hover (LSP), not at the call site — but authors cannot forget it
because the tier IS the syntax of `fun`. SPARK and Verus bury tiers in
separate ghost blocks. Eiffel hides them in runtime toggles. This is the
"label on the package at author time" ergonomic bet.

### 14.9 Migration Feasibility (§14.F-Audit-5 fix)

Naïve manual migration of stdlib (~20k functions) to Silver requires
~10 FTE-weeks. That is infeasible without automation. The 5.2 deadline is
conditioned on:

1. `ruchy suggest-contracts` (already registered as CLI in 5.0.0-alpha.1
   status table) must reach ≥ 80 % acceptance rate on its inferred
   contracts before 5.1 branches.
2. An auto-migration PR stream (target: 100 functions/day, human-reviewed
   in batches of 10) runs during the 5.1 → 5.2 window.
3. If acceptance rate < 50 % at 5.1 RC, the 5.2 deadline slips to 5.3 and
   we publicly update §14.6. Hiding slippage is a §14.5 falsification.

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
| 9 - Embedding | `docs/specifications/ruchy-embed-pillar9-integration.md` | EMBED-XXX |

---

## Appendix B: Downstream Book Repos (paiml org)

Success Criterion #4 requires every paiml-org book/cookbook/demo repo that
teaches or exercises Ruchy to compile cleanly against 5.0. These repos are
downstream consumers of the compiler and the most authentic regression
surface for breaking changes. Each must pass its own validation harness on
the released 5.0.0 binary before Go/No-Go.

| # | Repo | Kind | What it validates | Local path |
|---|------|------|-------------------|------------|
| B1 | `paiml/ruchy-book` | Official language book | Chapters 01..23 parse/compile/run; `make validate-book` | `../ruchy-book` |
| B2 | `paiml/ruchy-cookbook` | Language cookbook | Recipe-style examples across stdlib surface | `../ruchy-cookbook` |
| B3 | `paiml/ruchy-cli-tools-book` | CLI tools book | Building CLI tools in Ruchy (clap-equivalents, argv parsing) | `../ruchy-cli-tools-book` |
| B4 | `paiml/tooling-with-ruchy` | Tooling book | Using `ruchy` native tools (check/lint/fmt/test/coverage/...) | `../tooling-with-ruchy` |
| B5 | `paiml/ruchy-repl-demos` | REPL demos | `ruchy -e` one-liners + REPL transcripts | `../ruchy-repl-demos` |
| B6 | `paiml/rosetta-ruchy` | Polyglot benchmarks | Ruchy-vs-Rust-vs-Python performance parity examples | `../rosetta-ruchy` |
| B7 | `paiml/ruchyruchy` | Self-hosting corpus | Ruchy compiling Ruchy (bootstrap reference) | `../ruchyruchy` |

### Validation workflow

For each book repo `$B`:

```bash
# 1. Clone alongside the ruchy repo (sibling path)
( cd .. && [ -d "$B" ] || git clone "https://github.com/paiml/$B" )

# 2. Install current ruchy binary
cargo install --path . --force

# 3. Run the book's own validation harness
( cd ../$B && make validate 2>&1 )   # or the repo's documented target

# 4. Record pass/fail in the 5.0 spec status table
```

### Current status on 5.0.0-beta.1 (2026-04-05)

| # | Repo | Status | Notes |
|---|------|--------|-------|
| B1 | `ruchy-book` | PARTIAL | 15/16 critical chapters pass. ch18 DataFrames blocked by DATAFRAMES-001 (pre-existing transpiler defect, not 5.0 regression). |
| B2 | `ruchy-cookbook` | PARTIAL (77%) | 35/45 sampled `.ruchy` files pass. Recipe test-file parsing (`Expected RightBrace, found Let`) blocks the rest — separate pre-existing issue. Improved from 42% to 77% via PARSER-ATTR-001. |
| B3 | `ruchy-cli-tools-book` | PASS (100%) | 12/12 sampled files pass. |
| B4 | `tooling-with-ruchy` | SKIPPED (empty) | Repo has only README; no .ruchy corpus yet. |
| B5 | `ruchy-repl-demos` | PASS (100%) | 50/50 sampled files pass. |
| B6 | `rosetta-ruchy` | PASS (98%) | 49/50 sampled files pass. 1 pre-existing failure. |
| B7 | `ruchyruchy` | PASS (92%) | 46/50 sampled files pass. 4 pre-existing failures in debugger test corpus. |

### Gate for RC.1 → 5.0.0

RC.1 cannot be tagged until **all seven** book repos report either PASS or
a documented, ticketed, separate-from-5.0 failure. Missing validation
(UNVALIDATED) blocks the release.

### Known blockers

| Ticket | Repo | Description | Scope |
|--------|------|-------------|-------|
| DATAFRAMES-001 | ruchy-book ch18 | `df!` macro emits `HashMap<String,Vec<String>>` but transpiled code calls `.lazy()` expecting a Polars DataFrame | Transpiler DataFrame type inference rework — pre-existing, not a 5.0 regression |
| COMPILER-001 | ruchy-book (all compile-requiring chapters) | `ruchy compile` ignored `CARGO_TARGET_DIR` → "Expected binary not found" | **FIXED** in 5.0.0-beta.1 (`tests/compiler_cargo_target_dir.rs`) |
| PARSER-ATTR-001 | ruchy-cookbook + every 5.0 attribute user | Parser bail!'d on `#[...]` with "Attributes are not supported", contradicting spec Section 3 (unified decorator grammar) | **FIXED** in 5.0.0-beta.1 (`tests/parser_attribute_syntax.rs`). Moves cookbook from 42% → 77%. |
