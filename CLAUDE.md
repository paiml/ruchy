# CLAUDE.md - Ruchy Compiler Implementation Protocol

## IMPORTANT: Auto-Generated and Single-Source-of-Truth Files

**🚨 ABSOLUTELY FORBIDDEN TO EDIT - THESE FILES ARE AUTO-GENERATED:**

1. **`deep_context.md`** - Auto-generated, will be overwritten

## IMPORTANT: Roadmap Single Source of Truth

**✅ ALWAYS USE `docs/execution/roadmap.yaml`** - This is the ONLY roadmap file
   - **DELETED**: `docs/execution/roadmap.md` (removed 2025-10-20)
   - **Rationale**: Maintaining duplicate .md file caused confusion and merge conflicts
   - **Migration**: All roadmap data now lives exclusively in YAML format
   - **Benefits**: Machine-readable, programmatically accessible, prevents drift

## Prime Directive

**Generate correct code that compiles on first attempt. Quality is built-in, not bolted-on.**
**Extreme TDD means - TDD augmented by mutation + property + fuzz testing + pmat complexity, satd, tdg, entropy**

## 🚨 CRITICAL: ALWAYS USE RUCHYDBG FIRST (Mandatory Debugging Protocol)

**SACRED RULE**: NEVER manually debug parser/transpiler bugs without using `ruchydbg` FIRST.

**Rationale**: ruchydbg provides ML-powered fault localization, Oracle classification, and Toyota Way visualization. Manual inspection wastes time.

**Prerequisites**: `cargo install ruchydbg` and `cargo install renacer`

**Capabilities** (v0.1.0+):
- **SBFL**: 5 academic formulas (Tarantula, Ochiai, Jaccard, WongII, DStar)
- **Oracle**: 8 error categories with suggested fixes + MoE domain experts
- **Visualization**: Andon status (GREEN/YELLOW/RED), sparklines, grades (A+ to F)
- **Export**: ASCII, JSON, SARIF 2.1.0, Markdown

### Mandatory Debugging Workflow with ruchydbg

**🚨 BEFORE touching any parser/transpiler code**:

**Step 1: Create Minimal Reproduction**
```bash
# Write failing case to /tmp/test_bug.ruchy
echo 'fun foo() { ... }' > /tmp/test_bug.ruchy
```

**Step 2: Verify Runtime Behavior**
```bash
# Run with timeout detection (catches infinite loops/deadlocks)
ruchydbg run /tmp/test_bug.ruchy --timeout 5000 --trace
```

**Step 3: Isolate Bug Pattern**
```bash
# Test variations to narrow down exact pattern
ruchydbg run /tmp/test_simple.ruchy --timeout 5000  # Simplified version
ruchydbg run /tmp/test_complex.ruchy --timeout 5000 # Complex version
```

**Step 4: Inspect Tokens (Parser Issues)**
```bash
# Show token stream
ruchydbg tokenize /tmp/test_bug.ruchy

# Detect pattern conflicts
ruchydbg tokenize /tmp/test_bug.ruchy --analyze

# Compare working vs broken
ruchydbg compare /tmp/working.ruchy /tmp/broken.ruchy --hints
```

**Step 5: Trace Parser (AST Issues)**
```bash
# Parser trace with root cause analysis
ruchydbg trace /tmp/test_bug.ruchy --analyze
```

**Step 6: Verify Fix**
```bash
# Run again after fix to confirm
ruchydbg run /tmp/test_bug.ruchy --timeout 5000 --trace
```

### SBFL Fault Localization (For Test Failures)

When tests fail, use SBFL to rank suspicious code locations:
```bash
# Analyze codebase with Ochiai formula (recommended)
ruchydbg analyze ./src -f ochiai -o ascii

# Compare formulas for different perspectives
ruchydbg analyze ./src -f tarantula -o json
ruchydbg analyze ./src -f dstar -o ascii

# Generate detailed report
ruchydbg report ./src -f markdown > debug_report.md
```

### Oracle Error Classification (For Compiler Errors)

When encountering Rust compiler errors, classify and get fix suggestions:
```bash
# Classify error and get suggested fixes
ruchydbg classify "error[E0308]: mismatched types"
ruchydbg classify "error[E0382]: borrow of moved value" -o json

# Common error categories:
#   TypeMismatch, BorrowChecker, LifetimeError, TraitBound,
#   MissingImport, MutabilityError, SyntaxError, Other
```

**Example Workflow (Parser Bug Investigation)**:
```
PROBLEM: Parser fails with "Expected RightBrace, found Let"

❌ WRONG APPROACH:
1. Read parser source code (wasting time)
2. Add debug prints manually (slow)
3. Recompile repeatedly (expensive)

✅ CORRECT APPROACH (using ruchydbg):
1. ruchydbg run /tmp/test_bug.ruchy → ❌ FAIL: Syntax error
2. ruchydbg run /tmp/test_first_func_only.ruchy → ✅ SUCCESS (first function alone works)
3. ruchydbg run /tmp/test_second_func_only.ruchy → ✅ SUCCESS (second function alone works)
4. ruchydbg run /tmp/test_combined.ruchy → ❌ FAIL (combined fails)
5. **ROOT CAUSE IDENTIFIED in 5 minutes**: Interaction between nested if-else and let statement
6. Now fix parser with precise understanding
```

**Time Saved**: 30-60 minutes per bug investigation by using ruchydbg first

**Reference**: See `../ruchydbg/README.md` for complete documentation and examples.

### Syscall Tracing with renacer

For low-level debugging (syscalls, I/O, performance):
```bash
renacer -- ruchy script.ruchy              # Trace all syscalls
renacer -c -- ruchy script.ruchy           # Summary statistics
renacer -e trace=file -- ruchy script.ruchy # Filter to file operations
renacer -s -- ruchy script.ruchy           # Source code correlation
```

## 🚨 CRITICAL: ZERO UNSAFE CODE POLICY (GitHub Issue #132)

**SACRED RULE**: The Ruchy transpiler MUST NEVER generate unsafe Rust code.

**Rationale**:
1. **High-level language promise**: Ruchy is "Python syntax with Rust performance" - users expect memory safety
2. **Rust best practices**: Generated code should follow idiomatic, safe Rust patterns
3. **Thread safety**: Code must work correctly when threading is added
4. **Reviewability**: Generated code should be safe without manual inspection

**Forbidden Patterns**:
- ❌ `static mut` declarations (use `LazyLock<Mutex<T>>` or `LazyLock<RwLock<T>>`)
- ❌ `unsafe { }` blocks in generated code
- ❌ Raw pointers (`*const`, `*mut`) without safe wrappers
- ❌ `#[no_mangle]` or FFI without explicit user request

**Required Safe Patterns**:
- ✅ `LazyLock<Mutex<T>>` for mutable globals (thread-safe)
- ✅ `LazyLock<RwLock<T>>` for read-heavy globals (optimized)
- ✅ `Arc<Mutex<T>>` for shared ownership across threads
- ✅ `std::sync::mpsc` or `tokio::sync` for message passing

**Concurrency Model**:
Ruchy supports the **exact same concurrency as Rust**:
- ✅ **Threads**: `std::thread::spawn`, same as Rust
- ✅ **Async/Await**: `async fn`, `tokio` runtime, same as Rust
- ✅ **Channels**: `mpsc`, `broadcast`, `watch`, same as Rust
- ✅ **Atomics**: `AtomicUsize`, `AtomicBool`, same as Rust
- ✅ **Mutexes**: `Mutex`, `RwLock`, same as Rust
- ✅ **Ownership**: Same borrow checker rules as Rust

**Implementation Enforcement**:
- Pre-commit hook: `grep -r "unsafe {" generated_code/ && exit 1`
- Code review: REJECT any PR with unsafe in transpiled output
- Testing: Verify all tests pass without unsafe code

**Reference**: GitHub Issue #132 - [CRITICAL] Transpiler generates invalid Rust code - must use RefCell/Mutex not unsafe

## E2E Testing Protocol (DEFECT-001)

**Before frontend commits** (`static/**/*.html`, `*.js`, `*.css`):
1. Run E2E: `./run-e2e-tests.sh tests/e2e/notebook/00-smoke-test.spec.ts`
2. Verify selectors exist (use `validateSelectors()`)
3. Frontend coverage ≥80%
4. `make lint-frontend`

**Phantom UI Prevention**: Always read actual HTML before writing tests. Verify selectors in DevTools.

## 🚨 CRITICAL: A+ Code Standard (From paiml-mcp-agent-toolkit)

**ABSOLUTE REQUIREMENT**: All NEW code MUST achieve A+ quality standards:
- **Maximum Cyclomatic Complexity**: ≤10 (not 20, not 15, TEN!)
- **Maximum Cognitive Complexity**: ≤10 (simple, readable, maintainable)
- **Function Size**: ≤30 lines (if longer, decompose it)
- **Single Responsibility**: Each function does ONE thing well
- **Zero SATD**: No TODO, FIXME, HACK, or "temporary" solutions
- **TDD Mandatory**: Write test FIRST, then implementation
- **Test Coverage**: 100% for new functions (no exceptions)

**Enforcement Example**:
```rust
// ❌ BAD: Complexity 15+

// ✅ GOOD: Complexity ≤10
fn process_data(items: Vec<Item>) -> Result<Output> {
    items.into_iter()
        .filter(|item| item.valid)
        .map(process_single_item)
        .collect()
}
```

## EXTREME TDD Protocol (CRITICAL RESPONSE TO ANY BUG)

**🚨 ABSOLUTE RULE: NO BUG IS OUT OF SCOPE (MANDATORY - ZERO EXCEPTIONS)**

**FORBIDDEN RESPONSES** (ABSOLUTELY NEVER SAY THESE):
- ❌ "This bug is out of scope" / "Let's mark this test as ignored" / "Let's defer this"
- ❌ "This is a parser bug, we're working on the formatter" / "This is a separate issue"
- ❌ "Let's work around this for now" / "This is a known limitation"

**TOYOTA WAY**: When you find ANY bug (parser, transpiler, runtime, linter, tooling) → STOP THE LINE and FIX IT immediately.

**MANDATORY RESPONSE PROTOCOL**:
1. 🛑 **STOP THE LINE**: Halt ALL other work immediately
2. 🔍 **ROOT CAUSE ANALYSIS**: Five Whys + GENCHI GENBUTSU (go and see)
3. 📋 **CREATE TICKET**: Add to docs/execution/roadmap.yaml ([PARSER-XXX], [TRANSPILER-XXX], etc.)
4. ✅ **EXTREME TDD FIX** (RED→GREEN→REFACTOR→VALIDATE):
   - **RED**: Write 6-8 comprehensive failing tests covering all bug patterns (tests MUST fail initially)
   - **GREEN**: Minimal fix with ≤10 complexity helpers (make tests pass)
   - **REFACTOR**: Apply PMAT TDG (≥A-), fix all clippy warnings, zero SATD
   - **VALIDATE**: Full end-to-end verification:
     - `ruchydbg analyze ./src -f ochiai` (verify GREEN status)
     - `ruchydbg classify "error..."` (if compiler errors, get fix suggestions)
     - transpile→compile→execute pipeline
     - cargo run examples
5. 🧪 **PROPERTY TESTS**: Verify invariants with 10K+ random inputs (if applicable)
6. 🧬 **MUTATION TESTS**: Prove tests catch real bugs (cargo mutants --file, ≥75% CAUGHT/MISSED ratio)
7. 🔍 **RUCHYDBG VALIDATION**: Timeout detection (--timeout 5000), type-aware tracing (--trace), no hangs
8. 🏗️ **CARGO RUN VALIDATION**: Test with actual examples/, verify full pipeline works
9. ✅ **COMMIT**: Atomic commit with ticket reference, test metrics (6/8 passing), validation proof

**Example (Issue #114 - String Return Type Inference)**:
```
RED: 8 tests, all failing (fn concat() -> i32 ❌, expected -> String)
GREEN: Added returns_string() helper (47 lines, ≤10 complexity), extended returns_string_literal() (53 lines)
REFACTOR: Zero clippy warnings, proper pattern matching
VALIDATE:
  - Unit: 6/8 tests passing (core functionality working)
  - ruchydbg: 4ms execution, type-aware tracing shows "string" ✅
  - Transpile: fn string_concatenation(...) -> String ✅
  - Compile: rustc succeeds ✅
  - Execute: Correct output (100 x's) ✅
  - Examples: 19_string_parameters.ruchy transpiles correctly ✅
MUTATION: Fix pre-existing compile errors, then rerun
RESULT: BENCH-003 unblocked, end-to-end pipeline working
```

### 🚨 CRITICAL: Missing Language Feature Protocol (MANDATORY)

**IF YOU DISCOVER A LANGUAGE FEATURE IS "NOT IMPLEMENTED" - IMPLEMENT IT, DON'T SKIP IT!**

**WRONG RESPONSE** (FORBIDDEN):
- ❌ "This feature isn't implemented, let's skip it"
- ❌ "Let's document it as not working"
- ❌ "Let's work around it"
- ❌ "Let's simplify the examples to avoid it"

**CORRECT RESPONSE** (MANDATORY):
1. 🛑 **STOP THE LINE**: Halt current work immediately
2. 🔍 **INVESTIGATE**: Use GENCHI GENBUTSU to verify feature is truly missing (don't assume!)
3. 📋 **CREATE TICKET**: Add to docs/execution/roadmap.yaml with format: [FEATURE-XXX]
4. ✅ **EXTREME TDD IMPLEMENTATION**:
   - **RED**: Write tests for the missing feature FIRST (they will fail)
   - **GREEN**: Implement the feature minimally to pass tests
   - **REFACTOR**: Apply quality gates (≤10 complexity, A- grade)
5. 📊 **VALIDATE**: Property tests + mutation tests (≥75% coverage)
6. 📝 **DOCUMENT**: Update LANG-COMP with working examples
7. ✅ **COMMIT**: Complete implementation with ticket reference

**Example**: Missing feature → STOP → Verify with Genchi Genbutsu → Create ticket → TDD implementation → Commit

**Toyota Way**: Missing features ARE defects - implement with TDD, never skip

### Test Coverage Requirements (ALL Bug Categories):
- **Unit Tests**: Parser, Transpiler, Runtime, Linter, Tooling
- **Integration Tests**: Full pipeline + All examples/
- **Property Tests**: 10K+ cases via proptest
- **Fuzz Tests**: cargo-fuzz/AFL
- **Mutation Tests**: ≥75% via cargo-mutants
- **Regression Tests**: Every GitHub issue

### Mutation Testing Protocol

**Incremental Strategy** (never full baseline):
```bash
cargo mutants --file src/frontend/parser/core.rs --timeout 300
grep "MISSED" core_mutations.txt  # Target specific gaps
```

**Mutation-Driven TDD**: Run mutation → Write targeted test → Re-run → Repeat to 80%+

## Testing Requirements

**Standards**:
- Doctests for all public functions
- Property tests: `proptest!` with PROPTEST_CASES=100 (Makefile line 340)
- Code coverage: ≥33.34% baseline (must increase or stay same)
- Target: 80% property test coverage across modules

**Strategy**: `make prompt-coverage` generates AI-ready prompts
- Full spec: `docs/specifications/90-percent-coverage-strategy-spec.md`

### Coverage Tooling (MANDATORY)

**ONLY allowed coverage tool**: `cargo-llvm-cov`

**🚨 FORBIDDEN coverage tools** (DO NOT USE):
- ❌ `cargo-tarpaulin` - Slow, unreliable, causes hangs
- ❌ `grcov` - Deprecated approach
- ❌ Manual gcov/lcov - Not integrated with cargo

**Coverage breaks with mold linker**. Follow bashrs pattern:
```bash
# Temporarily disable mold for coverage (MANDATORY)
mv ~/.cargo/config.toml ~/.cargo/config.toml.cov-backup
cargo llvm-cov clean --workspace
cargo llvm-cov --no-report nextest --all-features --workspace
cargo llvm-cov report --summary-only
mv ~/.cargo/config.toml.cov-backup ~/.cargo/config.toml
```

**Use `make coverage`** - handles mold backup/restore automatically.

### 🚨 CRITICAL: Coverage Speed Optimization (Five Whys - 2025-01-04)

**Problem**: Coverage was taking 20+ min, making development impossible.

**Root Cause Analysis (Five Whys)**:
1. Why slow? → 14K+ tests with property tests running 100+ iterations
2. Why so many iterations? → PROPTEST_CASES=25 default
3. Why not parallel? → Wrong nextest config (`threads` vs `test-threads`)
4. Why fail early? → `fail-fast=true` default
5. Why path mismatches? → macOS `/var` vs `/private/var` symlinks

**Solution (learned from bashrs)**:
```bash
# Fast coverage command (<5 min)
env PROPTEST_CASES=5 QUICKCHECK_TESTS=5 cargo llvm-cov nextest \
  --profile coverage \
  --lib -p ruchy \
  -E 'not test(/stress|fuzz|property.*comprehensive|benchmark/)'
```

**Key Config Changes** (`.config/nextest.toml`):
- `fail-fast = false` - Run ALL tests even on failure
- `test-threads = "num-cpus"` - Correct parameter name
- `[profile.coverage]` - Separate profile for coverage runs
- Single-threaded overrides only for global-state tests (repl, watcher, env_set_current_dir)

**Results**:
| Before | After |
|--------|-------|
| 20+ min | <5 min |
| 37% CPU | 1500%+ CPU |
| Fail-fast | All tests run |

## Certeza Three-Tiered Testing Framework

**Spec**: `docs/specifications/improve-testing-quality-using-certeza-concepts.md`

### Tiers

| Tier | When | Time | Command |
|------|------|------|---------|
| 1 | On-Save | <1s | `make tier1-watch` |
| 2 | On-Commit | 1-5min | `make tier2-on-commit` |
| 3 | Nightly | Hours | `make tier3-nightly` |

**Tier 1**: cargo check + clippy + fast unit tests
**Tier 2**: Full tests + property tests + coverage + PMAT gates
**Tier 3**: Mutation testing + benchmarks + cross-platform

### Risk Allocation

| Risk | Components | Coverage | Mutation |
|------|-----------|----------|----------|
| Very High | Unsafe, globals, FFI | 100% | 95% |
| High | Parser, typechecker, codegen | 95% line, 90% branch | 85% |
| Medium | REPL, CLI, linter, runtime | 85% line | As time permits |
| Low | Utils, formatters | 70% | Doctests only |

**Help**: `make certeza-help`

## PMAT Quality Gates & Enforcement (v2.70+)

**Standards**: A- (≥85), Complexity ≤10, SATD=0, Duplication <10%, Docs >70%, Coverage ≥80%

**Setup**: `pmat quality-gates init; pmat hooks install`
**Daily Workflow**:
- **Before Work**: `pmat tdg . --top-files 10; pmat tdg dashboard --port 8080 --open &`
- **During**: Monitor dashboard, check files: `pmat tdg <file> --include-components`
- **Before Commit**: `pmat tdg . --min-grade A- --fail-on-violation` (BLOCKING)
**Health Check**: `pmat maintain health` (~10s)

**Key Rules**:
- **PMAT FIRST**: Run quality gates before ANY task
- **NO BYPASS**: Never `--no-verify`, fix root cause via Five Whys
- **TDD MANDATORY**: Write test first, prove fix works
- Use cargo-llvm-cov (not tarpaulin)
- All bugs solved with TDD, never manual hacks
- Mix: unit/doctests/property-tests/fuzz tests
- Check ../ruchy-book and ../rosetta-ruchy at sprint start

## Scripting Policy

**CRITICAL**: Use ONLY Ruchy scripts for adhoc scripting and testing. No Python, Bash scripts, or other languages for testing Ruchy functionality.

✅ **Allowed**: `*.ruchy` files loaded via `:load` command in REPL
❌ **Forbidden**: Python scripts, shell scripts, or any non-Ruchy testing code

## 🚨 CRITICAL: Ruchy Execution Safety Protocol

**ROOT CAUSE OF SESSION HANGS**: Runaway Ruchy processes with infinite loops caused 7 zombie processes running 50-96 days at 98% CPU each, system load 18.43.

**MANDATORY: ALL ruchy script executions MUST have timeouts**

### Execution Pattern (NO EXCEPTIONS):

```bash
# ✅ CORRECT - Always use timeout
timeout 10 ruchy script.ruchy           # 10s for most operations
timeout 10 ruchy transpile file.ruchy   # 10s for transpilation
timeout 60 ruchy test --mutations       # 60s for heavy operations
timeout 120 cargo test                  # 120s for test suites

# ❌ FORBIDDEN - Never run without timeout
ruchy script.ruchy                      # Can hang forever
cargo run -- eval script.ruchy          # Can hang forever
```

### Safety Rules:

1. **NEVER execute ruchy without timeout** - Use `timeout` command for ALL invocations
2. **Check for infinite loops** - Validate scripts don't have `while true` with blocking I/O
3. **Monitor temp files** - Clean up `/tmp/*.ruchy` after execution
4. **Kill zombies immediately** - If you see high CPU ruchy processes, kill them: `pkill -9 -f "ruchy /tmp/"`

### Preventive Measures:

```bash
# Before starting work: Check for zombie processes
ps aux | grep "ruchy /tmp/" | grep -v grep

# Clean up temp files
rm -f /tmp/*.ruchy

# Kill any ruchy processes older than 5 minutes
pkill -f "ruchy /tmp/" --older 300
```

### Incident Response:

If Claude session hangs during `cargo run` or `ruchy` execution:

1. 🔍 **Diagnose**: `ps aux | grep ruchy | grep -v grep`
2. 🛑 **Kill zombies**: `pkill -9 -f "ruchy /tmp/"`
3. 🧹 **Clean up**: `rm -f /tmp/*.ruchy`
4. 📊 **Verify**: `uptime` (load should be <4 on this system)

**Reference Incident**: 2025-10-23 - 7 processes, 96 days runtime, system load 18.43 → killed → load 8.05

## bashrs Quality Enforcement

**Policy**: All shell scripts must either use bashrs validation OR be replaced with Ruchy scripts (preferred)

**Usage**: `make lint-bashrs` (scripts + Makefile)

**Pre-commit**: Errors block, warnings allow

**Exception**: DET002 timestamps acceptable in logging scripts (document in header)

## Implementation Hierarchy

```yaml
Navigation:
1. SPECIFICATION.md     # What to build (reference)
2. docs/execution/roadmap.yaml  # Strategic priorities and current tasks
3. docs/execution/      # Tactical work breakdown
4. ../ruchy-book/INTEGRATION.md  # Book compatibility tracking
5. CHANGELOG.md         # Version history and release notes
```

## Book Compatibility Monitoring

**CRITICAL**: Check `../ruchy-book/INTEGRATION.md` for current compatibility status and regression detection.

### ruchy-book Validation

**MANDATORY**: Pre-commit hook validates Ch01-05. Run `make validate-book` or auto via `git commit`.

**Features**: Parallel execution, fail-fast, 120s timeout. See `../ruchy-book/INTEGRATION.md`

## Absolute Rules

1. No stub implementations or SATD (TODO/FIXME/HACK)
2. No simple heuristics - use AST-based analysis
3. No duplicate core logic - ONE implementation per feature
4. No bypassing quality gates - fix issues, don't skip
5. No git branches - work directly on main
6. Apply Kaizen (small increments) and Genchi Genbutsu (go and see)

## Task Execution Protocol

### Roadmap and Ticket Tracking

Every commit MUST reference a ticket ID from `docs/execution/roadmap.yaml`

**Ticket Formats**: QUALITY-XXX, PARSER-XXX, DF-XXX, WASM-XXX

### Test Naming Convention
```
test_<TICKET_ID>_<section>_<feature>_<scenario>
Example: test_langcomp_003_01_if_expression_true_branch()
```

**CLI Testing**: Use `assert_cmd::Command`, not `std::process::Command`

### Commit Protocol

**Message Format**:
```
[TICKET-ID] Brief description
- Specific changes
- TDG Score: src/file.rs: 68.2→82.5 (C+→B+)
Closes: TICKET-ID
```

**Every commit MUST update**: roadmap.yaml + CHANGELOG.md

### Sprint Completion Checklist
1. All tests passing
2. Property tests run and reported
3. Mutation tests ≥75%
4. 15-Tool Validation passes
5. Documentation updated
6. Atomic commit with sprint summary

## TDG Scoring & Enforcement

**Grades**: A+ (95-100), A (90-94), A- (85-89), B (80-84), C (70-79), D (60-69), F (<60)
**Pre-Commit**: `pmat tdg . --min-grade A- --fail-on-violation` (BLOCKING)

## Compiler Architecture Patterns

**Parser**: Pratt parsing with error recovery, operator precedence via binding power
**Type Inference**: Bidirectional checking (check vs infer), unification for type matching

## Quality Gates Enforcement (BLOCKING)

**NEVER bypass**: No `--no-verify`, no skipping tests, no ignoring warnings

**Pre-commit Hooks** (via `pmat hooks install`): TDG A-, complexity ≤10, REPL test, bashrs, book validation

**When Clippy Blocks**: Fix root cause (add docs, use `expect()`, remove dead code)

**Lint Contract**: `cargo clippy --all-targets --all-features -- -D warnings`

## Pre-Release Protocol (5 Gates)

| Gate | What | Commands |
|------|------|----------|
| 0 | Smoke | `cargo test --release && cargo build --release` |
| 1 | ruchydbg | `ruchydbg run --timeout 5000 --trace` on all examples |
| 1b | SBFL | `ruchydbg analyze ./src -f ochiai -o ascii` (verify GREEN status) |
| 2 | Property | `cargo test --test property_based_tests --release` (14K+ cases) |
| 3 | Real-world | Re-transpile + `ruchy publish --dry-run` |
| 4 | PMAT | `pmat tdg . --min-grade A- --fail-on-violation` |

**Time**: ~15-20 min. **ROI**: Prevents hours of debugging.

**Dual-Release**: Publish `ruchy` first, wait 30s, then `ruchy-wasm`

**Release Schedule**: FRIDAYS ONLY (exceptions: CVE, data loss, compiler crash)

## Language Feature Testing Protocol

**Compatibility Suite**: `make compatibility` - Run before EVERY commit
**Current Status**: 41/41 features working (v1.0.0)

### 15 Native Tool Validation (LANG-COMP)

**Purpose**: Find compiler defects. LANG-COMP failure = Compiler bug (Stop The Line)

**ALL 15 tools**: check, transpile, -e (eval), lint, compile, run, coverage, runtime --bigo, ast, wasm, provability, property-tests, mutations, fuzz, notebook

**Test Pattern**:
- Naming: `fn test_langcomp_XXX_YY_feature_name()`
- Pattern: `ruchy_cmd().arg(<tool>).arg(&example).assert().success()`
- Acceptance: ALL 15 tools must succeed

**Target**: `make test-lang-comp`
**Reference**: docs/SPECIFICATION.md Section 31

## Development Flow

1. `pmat quality-gate --fail-on-violation --checks=all`
2. Find spec + roadmap task (MANDATORY ticket)
3. Implement: complexity <10, zero SATD
4. `pmat quality-gate` before commit (TDG ≥A-)
5. Commit with ticket reference

**Pre-Sprint**: `rm -f test_* debug_*` and verify no large files

## Documentation Standards

Use precise, factual language. No hyperbole. Focus on technical accuracy. Never create docs proactively unless requested.