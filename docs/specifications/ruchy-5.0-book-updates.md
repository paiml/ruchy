# Ruchy 5.0: Book and Documentation Updates

**Version:** 1.0.0
**Status:** PROPOSED
**Date:** 2026-04-03

---

## 1. Overview

Nine documentation repositories require updates to cover the Ruchy 5.0 nine-pillar
architecture. Current books document v3.x-4.x features; there is zero coverage of the
sovereign stack, embedding, contracts, or simulation pillars.

**Goal:** Every pillar has at least one chapter in ruchy-book, one recipe in
ruchy-cookbook, and one demo in ruchy-repl-demos.

**Scope:**

| Repository | Current State | 5.0 Gap |
|------------|---------------|---------|
| ruchy-book | 24 chapters, v3.212.0, 96% examples pass | 0 of 9 pillars covered |
| ruchy-cli-tools-book | 4 Unix tools (cat/grep/wc/head) | 0 of 7 new subcommands documented |
| ruchy-cookbook | 1 of 600 recipes (0.17%) | 0 pillar-specific recipes |
| rosetta-ruchy | 145 files, 23 algorithms | No contract/sim/embed examples |
| ruchy-repl-demos | 148 demos, 100% pass | No 5.0 progression level |
| ruchyruchy | Bootstrap compiler, self-hosting | No 5.0 AST nodes or keywords |
| sovereign-ai-book | Stack architecture | No Ruchy frontend section |
| sovereign-ai-cookbook | forjar.yaml recipes | No Ruchy-native infra recipes |
| sovereign-ai-stack-book | Stack documentation | No unified scripting chapter |

**Dependency:** This spec depends on `ruchy-5.0-sovereign-platform.md` for pillar
definitions, syntax, and crate versions.

---

## 2. ruchy-book Updates

This is the primary, highest-impact documentation target. The ruchy-book is the canonical
learning resource for the language.

### 2.1 Current State

| Metric | Value |
|--------|-------|
| Chapters | 24 (Ch01-Ch24) |
| Documented version | v3.212.0 |
| Example pass rate | 96% |
| 18-tool validations | 2,628 |
| Pillar coverage | 0 of 9 |

### 2.2 New Chapters

| Chapter | Pillar | Title | Content Summary |
|---------|--------|-------|-----------------|
| Ch 25 | Correctness | "Design by Contract" | `requires`/`ensures`/`invariant`, SPARK levels (Gold/Silver/Bronze), blame tracking, runtime vs static checking |
| Ch 26 | Compute | "SIMD-Accelerated Computing" | trueno arrays, `ComputeBrick`, automatic GPU dispatch, threshold tuning, fallback to scalar |
| Ch 27 | Learning | "Machine Learning Pipelines" | `import ml`, `train`/`predict`/`score`, model persistence, quantization, drift detection |
| Ch 28 | Simulation | "Scientific Simulation" | `import sim`, Kepler orbit solver, Monte Carlo estimation, Bayesian optimization, Jidoka guards |
| Ch 29 | Visualization | "Reactive UI with Presentar" | `Column`/`Button`/`signal()`, `@brick` decorator, WASM build target, hot reload |
| Ch 30 | Infrastructure | "Infrastructure as Code" | `infra {}` blocks, `machine`/`resource` declarations, BLAKE3 state hashing, plan/apply/destroy |
| Ch 31 | Scripting | "Shell Scripting Target" | `--target shell`, `ruchy purify`, cross-shell compatibility (bash/zsh/fish), POSIX compliance |
| Ch 32 | Testing | "Probar Testing Framework" | `#[probar_test]`, playbooks, GUI coverage visualization, mutation testing integration |
| Ch 33 | Embedding | "Embedding Ruchy in Rust" | `Engine::new()`, `register_fn`, coroutine `yield`, hot reload, sandboxing, memory limits |

### 2.3 Chapter Structure Requirements

Each new chapter must follow the established ruchy-book conventions:

1. Opening motivating example (runnable, under 20 lines)
2. Concept explanation with diagrams where applicable
3. Progressive code listings in `listings/chXX/` directory
4. At least 3 exercises per chapter (easy/medium/hard)
5. Cross-references to related chapters and pillar specs
6. All listings validated by `make validate-book`

### 2.4 Existing Chapter Updates

| Chapter | Update Required |
|---------|----------------|
| Ch 01 (Getting Started) | Add 5.0 installation notes, mention 9 pillars |
| Ch 02 (Variables) | Add contract annotation examples on variable declarations |
| Ch 07 (Functions) | Add `requires`/`ensures` examples on function signatures |
| Ch 15 (Modules) | Document `import ml`, `import sim` module paths |
| Ch 20 (WASM) | Add widget build pipeline via `ruchy widget build` |
| Ch 24 (Advanced) | Reference new chapters for each pillar deep-dive |

### 2.5 INTEGRATION.md Update

Add 5.0 compatibility matrix to `../ruchy-book/INTEGRATION.md`:

| Feature | Chapter | Status | Ruchy Version |
|---------|---------|--------|---------------|
| Contract annotations | Ch 25 | PENDING | 5.0.0 |
| SIMD arrays | Ch 26 | PENDING | 5.0.0 |
| ML pipelines | Ch 27 | PENDING | 5.0.0 |
| Simulation | Ch 28 | PENDING | 5.0.0 |
| Reactive widgets | Ch 29 | PENDING | 5.0.0 |
| Infrastructure blocks | Ch 30 | PENDING | 5.0.0 |
| Shell target | Ch 31 | PENDING | 5.0.0 |
| Probar testing | Ch 32 | PENDING | 5.0.0 |
| Embedding API | Ch 33 | PENDING | 5.0.0 |

---

## 3. ruchy-cli-tools-book Updates

### 3.1 Current State

| Metric | Value |
|--------|-------|
| Parts | 2 (Introduction, Unix Tools) |
| Chapters | 4 (cat, grep, wc, head) |
| 5.0 tool coverage | 0 of 7 new subcommands |

### 3.2 New Part III: "Ruchy 5.0 Tools"

| Chapter | Subcommand | Content |
|---------|------------|---------|
| Ch 05 | `ruchy infra` | `plan`/`apply`/`drift`/`status`/`destroy` lifecycle |
| Ch 06 | `ruchy sim` | `run`/`inspect`/`verify`/`export` simulation workflows |
| Ch 07 | `ruchy widget` | `serve`/`build`/`test`/`inspect` for Presentar UIs |
| Ch 08 | `ruchy apr` | `run`/`serve`/`quantize`/`inspect`/`bench`/`eval` for ML |
| Ch 09 | `ruchy purify` | Bash-to-Ruchy cleanup, lint, transform pipeline |
| Ch 10 | `ruchy prove` | Contract verification at Gold/Silver/Bronze levels |
| Ch 11 | `ruchy contracts sync` | YAML extraction, CI integration, contract registry |

### 3.3 Chapter Structure

Each CLI chapter must include:
- Synopsis block with all flags and options
- 3 progressive examples (basic, intermediate, advanced)
- Error message catalog with resolution steps
- Integration example showing how the tool fits into CI pipelines

---

## 4. ruchy-cookbook Updates

### 4.1 Current State

| Metric | Value |
|--------|-------|
| Total planned recipes | 600 |
| Completed recipes | 1 (0.17%) |
| Pillar-specific recipes | 0 |

### 4.2 New Pillar-Specific Recipe Sections

Each section contains 5-8 recipes. Recipes are self-contained, runnable, and under 50 lines.

| Section | Recipes | Example Titles |
|---------|---------|----------------|
| Contracts | 6 | Annotating functions, Gold verification, blame debugging, invariant loops, decreases proofs, contract inheritance |
| SIMD/Compute | 5 | Array dot product, matrix multiply, GPU threshold tuning, SIMD reduction, ComputeBrick composition |
| Machine Learning | 6 | Classification pipeline, regression, model quantization, drift detection, cross-validation, feature importance |
| Simulation | 5 | Kepler orbit, Monte Carlo Pi, Bayesian optimization, population dynamics, queueing theory |
| Widgets/UI | 6 | Counter app, dashboard layout, data table, ModelCard, form validation, theme switching |
| Infrastructure | 5 | Web server stack, database provisioning, GPU cluster, drift detection, multi-region deploy |
| Shell Scripting | 5 | Installer script, CI deploy pipeline, system config, log rotation, backup cron |
| Testing | 5 | Playbook creation, visual regression, mutation testing, property-based testing, coverage gates |
| Embedding | 5 | Game scripting host, hot reload loop, sandboxed eval, coroutine scheduler, custom FFI bridge |

**Total new recipes:** 48 pillar-specific recipes (priority over generic recipes).

### 4.3 Recipe Template

Every recipe follows this structure:
1. Problem statement (1-2 sentences)
2. Solution code (runnable, under 50 lines)
3. Discussion (explanation of key techniques)
4. Cross-references (related recipes, book chapters)

---

## 5. rosetta-ruchy Updates

### 5.1 Current State

| Metric | Value |
|--------|-------|
| Files | 145 |
| Algorithms | 23 |
| 5.0 feature examples | 0 |

### 5.2 Contract-Annotated Algorithms

Add contract annotations to the top 10 most-used algorithms:

| Algorithm | Contract Additions |
|-----------|--------------------|
| binary_search | `requires(arr.is_sorted())`, `ensures(result.is_none() or arr[result] == target)` |
| quicksort | `ensures(result.is_sorted())`, `decreases(arr.len())` |
| merge_sort | `ensures(result.len() == input.len())`, `ensures(result.is_sorted())` |
| fibonacci | `requires(n >= 0)`, `ensures(result >= 0)`, `decreases(n)` |
| gcd | `requires(a > 0 and b > 0)`, `ensures(result > 0)`, `decreases(b)` |
| dijkstra | `requires(graph.is_connected())`, `ensures(dist[source] == 0)` |
| bfs | `requires(start in graph)`, `ensures(all nodes reachable)` |
| insertion_sort | `invariant(arr[0..i].is_sorted())` |
| matrix_multiply | `requires(a.cols == b.rows)`, `ensures(result.shape == (a.rows, b.cols))` |
| newton_raphson | `requires(tolerance > 0)`, `decreases(error)` |

### 5.3 Simulation Examples

| Example | Description |
|---------|-------------|
| n_body.ruchy | N-body gravitational simulation using `import sim` |
| monte_carlo_pi.ruchy | Pi estimation with Monte Carlo sampling |
| lorenz_attractor.ruchy | Chaotic system simulation with Jidoka guards |

### 5.4 Embedding Examples

| Example | Description |
|---------|-------------|
| game_loop.ruchy + host.rs | Ruchy scripted NPC behavior in a Rust game loop |
| plugin_system.ruchy + host.rs | Hot-reloadable plugin architecture |
| sandboxed_eval.rs | Evaluating untrusted Ruchy code with memory limits |

---

## 6. Sovereign AI Ecosystem Updates

### 6.1 sovereign-ai-book

Add new section: "Ruchy as Stack Frontend"

| Subsection | Content |
|------------|---------|
| Pillar-to-component mapping | Table showing how each of the 9 pillars maps to a stack component |
| Unified type system | How Ruchy types flow through the stack without serialization boundaries |
| Single-binary deployment | How `ruchy build --release` produces one binary for the entire stack |

### 6.2 sovereign-ai-cookbook

Convert existing `forjar.yaml` recipes to Ruchy-native `infra {}` equivalents:

| Existing Recipe | Ruchy-Native Version |
|-----------------|---------------------|
| web-server.yaml | web_server.ruchy using `infra { machine(...) }` |
| gpu-cluster.yaml | gpu_cluster.ruchy using `infra { resource("gpu", ...) }` |
| database-setup.yaml | database.ruchy using `infra { service("postgres", ...) }` |

### 6.3 sovereign-ai-stack-book

Add new chapter: "Ruchy 5.0 as the Unified Scripting Layer"

Content:
- Why a single language for the entire stack reduces cognitive load
- How `ruchy prove` replaces external verification tools
- How `ruchy sim` replaces separate simulation frameworks
- Migration guide from polyglot scripts to Ruchy 5.0

---

## 7. ruchy-repl-demos Updates

### 7.1 Current State

| Metric | Value |
|--------|-------|
| Total demos | 148 |
| Pass rate | 100% |
| Progression levels | 9 (levels 1-9) |
| 5.0 feature demos | 0 |

### 7.2 New Progression Level 10: "Ruchy 5.0 Features"

| Demo | Pillar | Description | Expected Output |
|------|--------|-------------|-----------------|
| 10_01_contract_basic.ruchy | Correctness | `requires`/`ensures` on a factorial function | Contract verification pass message |
| 10_02_signal_reactive.ruchy | Visualization | `signal()` state with derived computation | Reactive update trace |
| 10_03_monte_carlo.ruchy | Simulation | `import sim`, Monte Carlo Pi estimation | Pi approximation to 2 decimal places |
| 10_04_ml_classify.ruchy | Learning | `import ml`, train/predict on toy dataset | Accuracy score output |
| 10_05_coroutine_yield.ruchy | Embedding | Coroutine `yield` pattern with resumption | Interleaved yield/resume trace |
| 10_06_simd_array.ruchy | Compute | trueno array operations with timing | Performance comparison output |
| 10_07_infra_plan.ruchy | Infrastructure | `infra {}` block with plan output | Resource plan summary |
| 10_08_shell_target.ruchy | Scripting | Simple script transpiled to shell | Generated bash output |
| 10_09_probar_test.ruchy | Testing | `#[probar_test]` with assertion | Test pass/fail result |
| 10_10_contract_blame.ruchy | Correctness | Blame tracking on a violated precondition | Blame trace with caller info |

### 7.3 Demo Constraints

- Each demo must run in under 2 seconds
- Each demo must be self-contained (no external files)
- Each demo must produce deterministic output (seed RNG where needed)
- All demos validated by the existing `make validate-demos` infrastructure

---

## 8. ruchyruchy Updates

### 8.1 Current State

The ruchyruchy repository contains the bootstrap (self-hosting) compiler written in Ruchy.

### 8.2 Parser Updates

Add recognition for 7 new reserved keywords introduced in 5.0:

| Keyword | AST Node | Pillar |
|---------|----------|--------|
| `requires` | `ContractClause::Requires` | Correctness |
| `ensures` | `ContractClause::Ensures` | Correctness |
| `invariant` | `ContractClause::Invariant` | Correctness |
| `decreases` | `ContractClause::Decreases` | Correctness |
| `infra` | `InfraBlock` | Infrastructure |
| `signal` | `SignalDecl` | Visualization |
| `yield` | `YieldExpr` | Embedding |

### 8.3 Conformance Tests

Add 5.0 conformance test suite to ruchyruchy:

| Test Category | Count | Validates |
|---------------|-------|-----------|
| Contract parsing | 10 | All 4 contract keywords parse correctly |
| Infra block parsing | 5 | `infra {}` with nested declarations |
| Signal expressions | 5 | `signal()` and derived computations |
| Yield expressions | 5 | Coroutine yield/resume patterns |
| Keyword conflicts | 5 | New keywords do not break existing code |

### 8.4 JIT Updates

Extend the JIT backend for new AST node types:

| Node Type | JIT Behavior |
|-----------|-------------|
| `ContractClause` | Emit runtime check or static annotation depending on SPARK level |
| `InfraBlock` | Emit call to forjar runtime |
| `SignalDecl` | Emit reactive subscription setup |
| `YieldExpr` | Emit coroutine state machine transition |

---

## 9. Cross-Book Consistency

### 9.1 Version References

All documentation must reference Ruchy 5.0 consistently:

| Item | Required Value |
|------|----------------|
| Version string | "Ruchy 5.0" or "v5.0.0" (not "3.x", "4.x") |
| Minimum compiler version | v5.0.0 |
| Cargo.toml example version | `ruchy = "5.0"` |
| README badge | `![Ruchy 5.0](badge-url)` |

### 9.2 Shared Glossary

All books must use consistent terminology for the 9 pillars:

| Term | Definition |
|------|-----------|
| Pillar | One of 9 architectural domains in Ruchy 5.0 |
| Contract | A formal `requires`/`ensures`/`invariant` specification on a function |
| Blame tracking | Identifying which caller violated a contract precondition |
| SPARK level | Verification depth: Bronze (runtime), Silver (static), Gold (formal proof) |
| ComputeBrick | A SIMD-accelerated computation unit backed by trueno |
| Jidoka guard | A simulation safety check that halts on invariant violation |
| Infrastructure block | An `infra {}` declaration compiled to forjar provisioning |
| Signal | A reactive state primitive in the Presentar widget system |
| Engine | The `ruchy-embed` API entry point for hosting Ruchy in Rust |

### 9.3 Code Example Validation

| Requirement | Enforcement |
|-------------|-------------|
| All examples compile against 5.0 | `make validate-book` in CI |
| No deprecated 3.x/4.x patterns | Lint pass in pre-commit hook |
| Consistent import paths | `import ml`, `import sim` (not `import aprender`) |
| Consistent CLI invocations | `ruchy prove`, `ruchy sim` (not raw cargo commands) |

---

## 10. Priority Order and Effort Estimates

### 10.1 Priority Ranking

| Priority | Repository | Work Items | Estimated Effort | Rationale |
|----------|------------|------------|-----------------|-----------|
| 1 | ruchy-book | 9 new chapters + 6 chapter updates | 8-10 weeks | User-facing, highest learning impact |
| 2 | ruchy-repl-demos | 10 new demos | 1 week | Quick wins, verifiable, builds confidence |
| 3 | ruchy-cookbook | 48 new recipes (9 sections) | 4-5 weeks | Practical reference, complements book |
| 4 | ruchy-cli-tools-book | 7 new chapters | 3-4 weeks | Tool documentation, essential for adoption |
| 5 | rosetta-ruchy | 10 contract algorithms + 6 examples | 2 weeks | Showcase, community visibility |
| 6 | ruchyruchy | Parser + JIT + 30 conformance tests | 3-4 weeks | Self-hosting parity |
| 7 | sovereign-ai-book | 1 new section | 1 week | Ecosystem alignment |
| 8 | sovereign-ai-cookbook | 3 recipe conversions | 3 days | Ecosystem alignment |
| 9 | sovereign-ai-stack-book | 1 new chapter | 1 week | Ecosystem alignment |

**Total estimated effort:** 24-29 weeks across all repositories.

### 10.2 Dependencies

```
ruchy-5.0-sovereign-platform.md (spec finalized)
    |
    +---> ruchy-book Ch25-33 (must come first -- defines canonical examples)
    |         |
    |         +---> ruchy-cookbook recipes (reference book examples)
    |         +---> ruchy-repl-demos (simplified versions of book examples)
    |         +---> ruchy-cli-tools-book (CLI docs reference book concepts)
    |
    +---> rosetta-ruchy (independent, can parallelize with book)
    +---> ruchyruchy (independent, can parallelize with book)
    +---> sovereign-ai-* (depends on ruchy-book for cross-references)
```

### 10.3 Acceptance Criteria

| Criterion | Measurement |
|-----------|------------|
| All 9 pillars documented in ruchy-book | 9/9 new chapters merged |
| All new chapter examples pass | `make validate-book` returns 100% for Ch25-33 |
| All REPL demos pass | 158/158 demos (148 existing + 10 new) at 100% |
| All cookbook recipes runnable | `make validate-cookbook` returns 100% for new recipes |
| All CLI chapters have synopsis blocks | Manual review checklist |
| Cross-book glossary consistent | Automated term extraction and diff |
| No 3.x/4.x version references in new content | `grep -r "v3\.\|v4\." --include="*.md"` returns 0 in new files |
| INTEGRATION.md updated | 9 new rows in compatibility matrix |
