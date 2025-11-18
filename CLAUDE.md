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

**Rationale**: ruchydbg provides instant validation and isolation of bugs. Manual inspection wastes time.

**Prerequisites**: `cargo install ruchyruchy` (provides ruchydbg CLI)

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

**Reference**: See `../ruchyruchy/INTEGRATION_GUIDE.md` and `../ruchyruchy/DEBUGGING_GUIDE.md` for complete documentation.

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

## 🚨 CRITICAL: E2E Testing Protocol (DEFECT-001 Response)

**SACRED RULE**: NEVER commit frontend changes without E2E tests passing.

**Reference**: `docs/defects/CRITICAL-DEFECT-001-UI-EXECUTION-BROKEN.md`

### Mandatory E2E Testing Checklist

**Before ANY commit touching frontend code** (`static/**/*.html`, `*.js`, `*.css`):

1. ✅ **Run E2E smoke tests**: `./run-e2e-tests.sh tests/e2e/notebook/00-smoke-test.spec.ts`
2. ✅ **Verify selectors exist**: Use `validateSelectors()` helper (prevent phantom UI)
3. ✅ **Check coverage**: Frontend coverage ≥80% (enforced)
4. ✅ **Lint frontend**: `make lint-frontend` passes
5. ✅ **Visual check**: Manually verify in browser (Genchi Genbutsu)

### 🚨 CRITICAL: Phantom UI Prevention Protocol

**ROOT CAUSE**: E2E tests written for non-existent UI elements (100% failure rate).

**MANDATORY PROTOCOL**:
1. 🔍 **GENCHI GENBUTSU**: Read actual HTML before writing tests
2. ✅ **Selector Validation**: Verify EVERY selector exists in actual HTML
3. 🚫 **No Phantom UI**: Tests describe CURRENT reality, not future plans
4. ✅ **Manual Verification**: View page in browser, inspect with DevTools

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
   - **VALIDATE**: Full end-to-end verification (transpile→compile→execute, ruchydbg, cargo run examples)
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

### Test Coverage Requirements (MANDATORY - ALL Bug Categories):
- **Unit Tests**: Parser (tokens/grammar/edge cases), Transpiler (Ruchy→Rust mappings), Runtime (evaluation paths/errors), Linter (rules/scope/AST), Tooling (CLI/flags/output)
- **Integration Tests**: Full compile → execute → validate pipeline + All examples/
- **Property Tests**: 10K+ cases via proptest (verify invariants hold)
- **Fuzz Tests**: Millions of inputs via cargo-fuzz/AFL (find edge cases)
- **Mutation Tests**: ≥75% coverage via cargo-mutants (prove tests catch real bugs)
- **Regression Tests**: Every GitHub issue gets specific test case

### Mutation Testing Protocol (MANDATORY - Sprint 8)

**CRITICAL**: Mutation testing is the GOLD STANDARD for test quality validation.

#### Why Mutation Testing Matters:
- **Line coverage measures execution, mutation coverage measures effectiveness**
- 99% line coverage can have 20% mutation coverage (tests run code but don't validate it)
- Mutation testing empirically proves tests catch real bugs, not just exercise code paths
- Each mutation simulates a real bug - if tests don't catch it, they're inadequate

#### Incremental Mutation Testing Strategy:
```bash
# NEVER run full baseline (10+ hours) - use incremental file-by-file approach
cargo mutants --file src/frontend/parser/core.rs --timeout 300  # 5-30 min

# Analyze gaps immediately
grep "MISSED" core_mutations.txt

# Write tests targeting SPECIFIC mutations
# Re-run to validate 80%+ coverage achieved
```

#### Mutation-Driven TDD:
1. Run mutation test FIRST to identify gaps
2. Write test targeting SPECIFIC mutation
3. Re-run mutation test to verify fix
4. Repeat until 80%+ coverage achieved
5. No speculative tests - only evidence-based

#### Acceptable Mutations (Rare):
- **Semantically Equivalent**: Mutation produces identical observable behavior
- **Example**: `Vec::leak(Vec::new())` vs `self.state.get_errors()` both return empty slice
- Document why mutation is uncatchable and mark ACCEPTABLE

## Scientific Method Protocol

**WE DON'T GUESS, WE PROVE VIA QUANTITATIVE METHODS AND TESTING.**

### Evidence-Based Development Rules:
1. **NO ASSUMPTIONS**: Every claim must be backed by concrete evidence
2. **MEASURE EVERYTHING**: Use tests, benchmarks, and metrics to validate behavior
3. **REPRODUCE ISSUES**: Create minimal test cases that demonstrate problems
4. **QUANTIFY IMPROVEMENTS**: Before/after metrics prove effectiveness
5. **DOCUMENT EVIDENCE**: All findings must be recorded with reproducible steps

### Investigation Protocol:
1. **Hypothesis**: State what you believe is happening
2. **Test**: Create specific tests that prove/disprove the hypothesis
3. **Measure**: Collect concrete data (test results, timings, coverage)
4. **Analyze**: Draw conclusions only from the evidence
5. **Document**: Record findings and next steps

## Mandatory Testing Requirements

**Target**: 80% property test coverage across all modules (Sprint 88 pattern)

**Requirements**:
1. **Doctests**: Every public function has runnable documentation examples
2. **Property Tests**: `proptest!` macro with 10K+ random inputs (80% of modules)
3. **Examples**: Working code in examples/ directory demonstrating usage
4. **Code Coverage**: ≥33.34% baseline (enforced by pre-commit hooks, NEVER decrease)
   - Current: 33.34% (post-QUALITY-007: character literals, tuple destructuring, rest patterns)
   - Direction: Must increase or stay same

### 🚨 MANDATORY: PROPTEST_CASES=100 Standard (Institutionalized)

**SACRED RULE**: Property tests MUST use 100+ test cases for statistical significance.

**Rationale**:
- 5 cases = insufficient for edge case discovery (may hit 3-4 branches out of 10)
- 100 cases = statistically significant coverage of conditional branches, error paths, boundary conditions
- Empirical evidence: bashrs achieved ~90% coverage using PROPTEST_CASES=100 (validated)
- Scientific backing: docs/specifications/90-percent-coverage-strategy-spec.md (10 peer-reviewed papers)

**Implementation**: Makefile line 340 (NEVER change back to 5)
```makefile
@env PROPTEST_CASES=100 QUICKCHECK_TESTS=100 cargo llvm-cov ...
```

**Expected Impact**: +3-5% coverage increase (Phase 1, Task 1.2 from 90% strategy)

**Verification**: Coverage percentage displayed in `make coverage` output

### 90% Coverage Strategy (Scientific + Empirical)

**Command**: `make prompt-coverage` (generates AI-ready prompts for coverage work)

**Full Strategy**: `docs/specifications/90-percent-coverage-strategy-spec.md`
- Empirical analysis of bashrs (sister project at ~90% coverage)
- 10 peer-reviewed computer science papers (IEEE, ACM, PLDI, ICSE)
- 3-phase actionable plan: 70%→80%→90%→95%
- bashrs patterns: 13.5 tests/file, PROPTEST_CASES=100, SQLite exhaustive testing

**Implementation**: Script auto-detects current coverage phase and generates context-aware prompts with:
- Priority modules for improvement
- Specific tasks with time estimates
- Code patterns from bashrs
- Success criteria
- Copy-paste ready prompts

## 🚀 Certeza Three-Tiered Testing Framework (DOCS-CERTEZA-001)

**CRITICAL**: Ruchy implements the Certeza testing framework for systematic quality assurance.

**Philosophy**: "Testing can prove the presence of bugs, not their absence" (Dijkstra). Maximize practical confidence through tiered verification, not theoretical perfection.

**Specification**: `docs/specifications/improve-testing-quality-using-certeza-concepts.md` (47KB, 10 peer-reviewed papers)

### Three-Tiered Workflow

#### Tier 1: On-Save (Sub-Second Feedback)

**Goal**: Enable developer flow state through instant feedback.

**Time Budget**: <1 second per save.

**Usage**:
```bash
# Manual execution
make tier1-on-save

# Auto-watch mode (recommended for development)
make tier1-watch    # Runs on every file save
```

**What it does**:
- `cargo check` - Syntax and type checking (0.1-0.5s)
- `cargo clippy` - Linting (0.2-0.8s)
- Fast unit tests - Critical path only (0.1-0.3s)

**When to use**: During active development for continuous validation.

---

#### Tier 2: On-Commit (1-5 Minutes)

**Goal**: Prevent problematic commits from entering repository.

**Time Budget**: 1-5 minutes per commit.

**Usage**:
```bash
make tier2-on-commit
```

**What it does**:
- Full unit test suite
- Property-based tests (PROPTEST_CASES=100)
- Integration tests
- Coverage analysis (≥95% line target, ≥90% branch target)
- PMAT quality gates (TDG ≥A-, complexity ≤10)

**When to use**: Before every commit (can be added to pre-commit hook).

**Integration with PMAT**: This tier implements existing PMAT pre-commit hooks with enhanced coverage tracking.

---

#### Tier 3: Nightly/Pre-Merge (Hours)

**Goal**: Maximum confidence before main branch integration.

**Time Budget**: Hours (run overnight or in CI).

**Usage**:
```bash
# Manual execution (long-running)
make tier3-nightly

# GitHub Actions (automated)
# Runs nightly at 2 AM UTC via .github/workflows/certeza-tier3-nightly.yml
```

**What it does**:
- **Mutation testing**: Incremental, file-by-file (5-30 min per file)
  - Parser modules (High-Risk: ≥85% mutation score target)
  - Type inference modules (High-Risk: ≥85% mutation score target)
  - Code generation modules (High-Risk: ≥85% mutation score target)
- **Performance benchmarks**: Regression detection
- **RuchyRuchy smoke testing**: 14,000+ property test cases
- **Cross-platform validation**: Linux, macOS, Windows

**When to use**: Nightly CI (automated), before major releases, pre-merge to main.

---

### Risk-Based Resource Allocation

Certeza allocates verification effort based on risk level: **"Spend 40% of verification time on the 5-10% highest-risk code"**.

**Risk Stratification**:

| Risk Level | Components | Coverage Target | Mutation Target | Allocation |
|------------|-----------|-----------------|-----------------|------------|
| **Very High** | Unsafe blocks, globals, FFI | 100% line/branch | 95% mutation | 40% effort |
| **High** | Parser, type inference, codegen | 95% line, 90% branch | 85% mutation | 35% effort |
| **Medium** | REPL, CLI, linter, runtime | 85% line, 80% branch | As time permits | 20% effort |
| **Low** | Utilities, formatters, docs | 70% line | Doctests only | 5% effort |

**Application to Ruchy**:
- **Very High-Risk**: `src/codegen/unsafe_globals.rs` (if any), WASM FFI bindings
- **High-Risk**: `src/frontend/parser/`, `src/typechecker/`, `src/codegen/`
- **Medium-Risk**: `src/runtime/`, `src/cli/`, `src/linter/`
- **Low-Risk**: `src/utils/`, `src/formatter/`

---

### Target Metrics (Phase 1 Complete, Phases 2-5 In Progress)

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| **Line Coverage** | 70.31% | 95%+ | 🟡 In Progress |
| **Branch Coverage** | Not tracked | 90%+ | 🟡 Infrastructure added |
| **Mutation Score** | Ad-hoc | ≥85% (High-Risk) | 🟡 Tier 3 CI running |
| **Property Test Coverage** | ~40% modules | 80% modules | 🟡 Phase 3 pending |
| **Tier 1 Feedback** | N/A | <1 second | ✅ cargo-watch installed |
| **Tier 2 Feedback** | ~5 min | 1-5 min | ✅ Makefile target added |
| **Tier 3 CI** | None | Nightly | ✅ GitHub Actions workflow |

---

### Implementation Status

**Phase 1: Infrastructure (COMPLETE - 2025-11-18)**:
- ✅ Installed cargo-watch for Tier 1 automation
- ✅ Added branch coverage tracking to Tier 2
- ✅ Created GitHub Actions nightly CI for Tier 3
- ✅ Documented three-tiered workflow in CLAUDE.md
- ✅ Created Makefile targets: `tier1-on-save`, `tier1-watch`, `tier2-on-commit`, `tier3-nightly`

**Next Phases**:
- Phase 2: Risk Stratification (Sprint 3-4)
- Phase 3: Property Testing Expansion (Sprint 5-6)
- Phase 4: Mutation Testing Systematic Coverage (Sprint 7-8)
- Phase 5: Selective Formal Verification (Sprint 9-10)

---

### Certeza Help

**Quick Reference**:
```bash
# Show Certeza framework overview
make certeza-help

# Development workflow (Tier 1)
make tier1-watch              # Start auto-watch mode

# Pre-commit validation (Tier 2)
make tier2-on-commit          # Run before git commit

# Check nightly results (Tier 3)
# View GitHub Actions: .github/workflows/certeza-tier3-nightly.yml
```

**Scientific Foundation**: 10 peer-reviewed publications (IEEE TSE, ICSE, ACM, NASA FM)
- Google mutation testing at scale (16.9M mutants)
- Jane Street property-based testing (30 interviews)
- Microsoft/IBM TDD case studies (40-90% defect reduction)
- Prusti formal verification for Rust

**Economic Reality**:
- **Time Investment**: 25% of development time (10 hours per 40-hour sprint)
- **Defect Reduction**: 40-90% (empirical evidence)
- **Break-even**: 3-6 months (amortized over reduced debugging)

---

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

## bashrs Quality Enforcement (Shell Scripts & Makefiles)

**SACRED RULE**: ALL shell scripts MUST either:
1. **Use bashrs validation** (for infrastructure/build scripts like Makefile, pre-commit hooks), OR
2. **Be replaced with Ruchy scripts** (PREFERRED for all new scripting)

**MANDATORY**: All remaining shell scripts and Makefiles MUST pass bashrs quality checks before commit.

**Policy**:
- **New Scripts**: Write in Ruchy (`.ruchy` files) whenever possible
- **Existing Scripts**: Either migrate to Ruchy OR enforce bashrs quality gates
- **Infrastructure**: Build tools, CI/CD, and git hooks may remain in bash with bashrs validation
- **Zero Exceptions**: No raw bash without bashrs validation

### Installation
```bash
cargo install bashrs  # Already installed at ~/.cargo/bin/bashrs
```

### Usage

**Manual Linting**:
```bash
# Lint single shell script
bashrs lint <script.sh>

# Lint Makefile
bashrs make lint Makefile

# Lint all via Makefile targets
make lint-scripts    # All .sh files
make lint-make       # Makefile only
make lint-bashrs     # Everything (scripts + Makefile)
```

**Pre-commit Hook**: bashrs validation runs automatically on staged files
- ✅ **Errors**: Block commit (exit 1)
- ⚠️  **Warnings**: Allow commit (non-blocking)

**Quality Standards**: Errors block commits, warnings are non-blocking. Run `make lint-bashrs` before changes.

### Documented Exceptions

**DET002 (Non-deterministic timestamps)** - Acceptable in logging/testing scripts:
- `.pmat/run_overnight_mutations.sh`: Timestamps are INTENTIONAL for test logging
- Rationale: Logging scripts need timestamps for debugging and audit trails
- Pattern: Add header comment explaining why timestamps are required
- Pre-commit: Excluded from bashrs validation (documented in .git/hooks/pre-commit)

**Exception Criteria**:
- Script purpose is logging, testing, or debugging (not build/deployment)
- Timestamps provide valuable debugging information
- Script is NOT used in reproducible builds or CI/CD pipelines
- Exception is documented in script header AND CLAUDE.md

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

## Absolute Rules (From paiml-mcp-agent-toolkit)

1. **NEVER Leave Stub Implementations**: Every feature must be fully functional. No "TODO" or "not yet implemented".
2. **NEVER Add SATD Comments**: Zero tolerance for TODO, FIXME, HACK comments. Complete implementation only.
3. **NEVER Use Simple Heuristics**: Always use proper AST-based analysis and accurate algorithms.
4. **NEVER Duplicate Core Logic**: ONE implementation per feature. All consumers use same underlying logic.
5. **ALWAYS Dogfood via MCP First**: Use our own MCP tools as primary interface when available.
6. **NEVER Bypass Quality Gates**: Zero tolerance for `--no-verify`. Fix issues, don't bypass.
7. **NEVER Use Git Branches**: Work directly on main branch. Continuous integration prevents conflicts.
8. **ALWAYS Apply Kaizen**: Small, incremental improvements. One file at a time.
9. **ALWAYS Use Genchi Genbutsu**: Don't guess. Use PMAT to find actual root causes.
10. **ALWAYS Apply Jidoka**: Automate with human verification. Use `pmat refactor auto`.

## Task Execution Protocol

### MANDATORY: Roadmap and Ticket Tracking

**CRITICAL**: ALL development work MUST follow roadmap-driven development:

1. **ALWAYS Use Ticket Numbers**: Every commit, PR, and task MUST reference a ticket ID from docs/execution/roadmap.yaml
2. **Roadmap-First Development**: No work begins without a corresponding roadmap entry
3. **Ticket Format**: Use format "QUALITY-XXX", "PARSER-XXX", "DF-XXX", "WASM-XXX" per roadmap
4. **Traceability**: Every change must be traceable back to business requirements via ticket system
5. **Sprint Planning**: Work is organized by sprint with clear task dependencies and priorities

### CLI Testing Standard
**Pattern**: Use `assert_cmd::Command` with predicates, NOT `std::process::Command`
**Reference**: See `tests/fifteen_tool_validation.rs` for examples

### MANDATORY: Test Naming Convention (TRACEABILITY)

**CRITICAL**: Every test MUST be traceable to its documentation/specification via naming convention.

**Naming Pattern** (MANDATORY - NO EXCEPTIONS):
```
test_<TICKET_ID>_<section>_<feature>_<scenario>

Examples:
- test_langcomp_003_01_if_expression_true_branch()
- test_langcomp_003_01_if_expression_example_file()
- test_langcomp_003_02_match_literal_pattern()
- test_langcomp_003_05_break_exits_loop()
```

**Component Breakdown**:
1. `TICKET_ID`: LANG-COMP-003 → `langcomp_003` (lowercase, underscored)
2. `section`: File number (01, 02, 03, 04, 05)
3. `feature`: What is being tested (if_expression, match_pattern, for_loop)
4. `scenario`: Specific test case (true_branch, example_file, literal_pattern)

**Why Naming Convention is Mandatory**:
- **Traceability**: Instantly map test → documentation → ticket → requirement
- **Findability**: `grep "langcomp_003_01"` finds all if-expression tests
- **Validation**: Prove documentation examples are tested (not just written)
- **Toyota Way**: Standard work enables quality - no standard = no quality

**Generic Names are DEFECTS**: `test_if_true_branch()` provides ZERO traceability. Use ticket-based naming.

### Commit Message Format
```
[TICKET-ID] Brief description

- Specific changes
- Test coverage
- TDG Score: src/file.rs: 68.2→82.5 (C+→B+)

Closes: TICKET-ID
```

### 🚨 MANDATORY: Update Roadmap & Changelog on EVERY Commit

**ABSOLUTE REQUIREMENT**: EVERY commit MUST update documentation to prevent stale roadmap.

**Pre-Commit Checklist (NO EXCEPTIONS)**:
1. ✅ **Update docs/execution/roadmap.yaml**:
   - Add completed task to appropriate sprint section
   - Update progress percentages
   - Document test metrics (unit/property/mutation counts)
   - Mark dependencies as complete

2. ✅ **Update CHANGELOG.md**:
   - Add entry under appropriate version header
   - Document what changed (Added/Fixed/Changed/Removed)
   - Include ticket reference and file paths
   - Note breaking changes if any

**Why This Matters**:
- **Single Source of Truth**: roadmap.yaml is machine-readable and programmatically validated
- **Historical Record**: CHANGELOG.md provides human-readable version history
- **No Stale Documentation**: Every commit keeps documentation current
- **Toyota Way**: Jidoka (quality built-in) - documentation is part of the commit, not an afterthought

**Workflow**: Code changes → Update roadmap.yaml → Update CHANGELOG.md → Commit atomically

**Forbidden**: Code commits without documentation updates

### End-of-Sprint Git Commit Protocol (MANDATORY)

**Sprint Completion Checklist**:
1. ✅ All sprint tasks complete + unit tests passing
2. ✅ **Property tests**: `cargo test <module>::property_tests -- --ignored --nocapture` (report results)
3. ✅ **Mutation tests**: `cargo mutants --file tests/lang_comp/<module>.rs --timeout 300` (≥75% CAUGHT/MISSED)
4. ✅ **15-Tool Validation**: All examples work with ALL 15 native tools
5. ✅ **Roadmap + Documentation updated** with sprint status + test metrics
6. ✅ **GIT COMMIT IMMEDIATELY**: `git add . && git commit` with sprint summary, test metrics, ticket list

**Toyota Way**: Atomic sprint commits prevent integration hell and data loss

## TDG Scoring & Enforcement

**Grades**: A+ (95-100), A (90-94), A- (85-89), B (80-84), C (70-79), D (60-69), F (<60)
**Pre-Commit**: `pmat tdg . --min-grade A- --fail-on-violation` (BLOCKING)

## Compiler Architecture Patterns

**Parser**: Pratt parsing with error recovery, operator precedence via binding power
**Type Inference**: Bidirectional checking (check vs infer), unification for type matching

## Quality Gates Enforcement (BLOCKING - Not Advisory)

### SACRED RULE: NEVER BYPASS QUALITY GATES
- ❌ **FORBIDDEN**: `git commit --no-verify`, skipping tests, ignoring checks, dismissing warnings
- ✅ **Toyota Way**: Stop the line for ANY defect, fix root cause via Five Whys

**Pre-commit Hooks** (AUTO-INSTALLED via `pmat hooks install`):
- TDG A-, Function complexity ≤10, Basic REPL test, bashrs validation, book validation

**When Clippy Blocks** - Fix root cause:
- Missing `# Errors` → Add documentation with examples
- `unwrap()` → Replace with `expect()` + meaningful messages
- Dead code → Remove or prefix with underscore
- Missing doctests → Add runnable examples

**Make Lint Contract**: `cargo clippy --all-targets --all-features -- -D warnings` (EVERY warning = error)

### 🚨 CRITICAL: Pre-commit Hook Management

**ABSOLUTE PROHIBITION**: Quality gate checks MUST NEVER be "temporarily disabled"

**Problem**: PMAT hooks may have commented-out checks (violates "Stop the Line")

**Correct Approach**:
1. 🛑 **STOP THE LINE**: If quality gate fails, halt work immediately
2. 🔍 **FIVE WHYS**: Perform root cause analysis
3. 🔧 **FIX VIOLATIONS**: Refactor to meet standards (≤10 complexity, zero SATD)
4. ✅ **RE-ENABLE**: `pmat hooks refresh` to regenerate
5. 📝 **VERIFY**: Test commit confirms gates block violations

**Toyota Way**: Temporary bypasses become permanent. Fix root causes, never mask symptoms.

### 🚨 MANDATORY: RuchyRuchy Smoke Testing Protocol (Pre-Release)

**SACRED RULE**: NEVER release without RuchyRuchy smoke testing.

**Prerequisites**: `cargo install ruchyruchy` (provides ruchydbg CLI v1.12.0+)

**MANDATORY Steps**:
1. **Create smoke test for EACH fix**: Test actual bug pattern
2. **Run individual tests**: `ruchydbg run /tmp/smoke_test.ruchy --timeout 5000 --trace`
3. **Integration test**: Combine all fixes, run with ruchydbg
4. **Performance check**: `ruchydbg detect /tmp/smoke_integration.ruchy --threshold 15`
5. **Real project validation**: `timeout 60 ruchy compile /path/to/project/main.ruchy`
6. **Generate report**: Document all test results

**Checklist** (Before Publishing):
- [ ] Individual smoke tests pass
- [ ] Integration test passes
- [ ] Performance analysis complete
- [ ] Real project validation passes
- [ ] Report generated

**Success Metrics**: ~15 min investment prevents hours of debugging

### 🚀 PRE-RELEASE VALIDATION PROTOCOL (4 Gates)

**Gate 0: Smoke Testing**
- `cargo test --lib --release`
- `cargo test --test --release`
- `cargo build --release`

**Gate 1: Debugging Tools (ruchydbg v1.13.0+)**
- Timeout detection: Test all examples with `ruchydbg run --timeout 5000 --trace`
- Regression: `ruchydbg regression determinism examples/**/*.ruchy --runs 10`
- Stack profiling: `ruchydbg profile --stack examples/recursion/*.ruchy`

**Gate 2: Property-Based Testing**
- Location: `../ruchyruchy`
- Run: `cargo test --test property_based_tests --release`
- Expected: 7 properties, 14,000+ cases, all PASS

**Gate 3: Real-World Project**
- Re-transpile project with latest Ruchy
- Verify compilation: `cargo clean && cargo build --release`
- Test execution: `ruchydbg run src/main.ruchy --timeout 10000`
- Verify publication: `ruchy publish --dry-run`

**Gate 4: PMAT Quality**
- `pmat tdg . --min-grade A- --fail-on-violation`
- `pmat maintain health`

**Time**: ~15-20 min per release. **ROI**: Prevents hours of debugging.

### DUAL-RELEASE PUBLISHING PROTOCOL
**After version bump**: Publish `ruchy` first, wait 30s, then `ruchy-wasm` (same version)
**Checklist**: Tests pass, CHANGELOG updated, git commit/push, both crates build

### 📅 CRATES.IO RELEASE SCHEDULE (MANDATORY)

**🚨 SACRED RULE: Crates.io releases happen on FRIDAYS ONLY**

**Rationale**:
- **Weekend buffer**: Issues can be addressed before Monday
- **User impact**: Minimal disruption to production users
- **Quality assurance**: Full week for testing and validation
- **Predictability**: Consistent release cadence builds trust

**Workflow**:
1. **Monday-Thursday**: Development, testing, bug fixes, quality gates
2. **Friday Morning**: Final validation (4 gates: smoke, ruchydbg, property tests, real-world)
3. **Friday Afternoon**: Dual-release publishing (ruchy → wait 30s → ruchy-wasm)
4. **Weekend**: Monitor for issues, hotfix if critical

**Exceptions** (RARE - require explicit justification):
- Critical security vulnerability (CVE-level)
- Data loss bug affecting production users
- Compiler crash blocking all users

**Version Bumps During Week**:
- ✅ ALLOWED: Cargo.toml version bumps, git tags, CHANGELOG updates
- ✅ ALLOWED: Development commits to main branch
- ❌ FORBIDDEN: `cargo publish` commands (wait for Friday)

**Enforcement**: Claude will remind you if you attempt to publish on non-Friday

## Language Feature Testing Protocol

### CRITICAL REQUIREMENT: Language Compatibility First

**NO CODE CHANGES can be committed without passing language feature compatibility tests.**

#### Compatibility Test Suite (MANDATORY)
```bash
# Run before EVERY commit - no exceptions
make compatibility  # Or: cargo test compatibility_report --test compatibility_suite -- --nocapture --ignored
```

**Current Standards (v1.0.0)**:
- ✅ **One-liners**: 100% (15/15) - Baseline
- ✅ **Basic Language Features**: 100% (5/5) - Core syntax complete  
- ✅ **Control Flow**: 100% (5/5) - if/match/for/while/pattern-guards
- ✅ **Data Structures**: 100% (7/7) - Objects functional
- ✅ **String Operations**: 100% (5/5) - String methods working
- ✅ **Numeric Operations**: 100% (4/4) - Integer.to_string() + math ops
- ✅ **Advanced Features**: 100% (4/4) - Pattern guards complete

**Total: 41/41 features working**

### Test Organization (Industry Standard)
```
tests/
├── compatibility_suite.rs      # Main feature compatibility (100% required)
├── properties/                 # Property-based testing (Haskell style)
├── regression/                 # Bug prevention (every GitHub issue)
└── benchmarks/                # Performance baselines (SQLite style)
```

Language compatibility testing is **GATE 2** in our mandatory pre-commit hooks - more critical than complexity or linting because **language regressions break user code**.

## 15 Native Tool Validation Protocol (LANG-COMP MANDATORY)

**🚨 SACRED PRINCIPLE: LANG-COMP TESTS ARE DEFECT-FINDING TOOLS**

**Purpose**: Find compiler/interpreter defects, NOT documentation
- **NO WORKAROUNDS EVER**: LANG-COMP test fails → FIX THE COMPILER (Stop The Line)
- **Success**: DEFECT-POW (pow() missing in eval), DEFECT-REF (& operator missing), DEFECT-TUPLE (tuple field access)
- **Sacred Rule**: LANG-COMP failure = Compiler bug

**Tool Support** (TOOL-VALIDATION sprint 2025-10-07):
- **REPL**: `ruchy -e "$(cat file.ruchy)"` for eval validation
- **Notebook**: Accepts file parameter for non-interactive mode
- **WASM**: Validate with simple code (some feature limitations)

### 15-Tool Validation Requirements (MANDATORY - NO EXCEPTIONS)

#### Mandatory Test Pattern (ALL 15 TOOLS - NO SKIPS):

**ALL 15 tools**: check, transpile, -e (eval), lint, compile, run, coverage, runtime --bigo, ast, wasm, provability, property-tests, mutations, fuzz, notebook

**Test Structure**:
- **Naming**: `fn test_langcomp_XXX_YY_feature_name()` (XXX = ticket, YY = section)
- **Pattern**: Invoke all 15 tools via `ruchy_cmd().arg(<tool>).arg(&example).assert().success()`
- **Acceptance**: Test passes ONLY if ALL 15 tools succeed

**Makefile Target**: `make test-lang-comp` runs `cargo test --test lang_comp_suite`
- Tests LANG-COMP-006 (Data Structures), 007 (Type Annotations), 008 (Methods), 009 (Pattern Matching)
- Current: 4 test modules with full 15-tool coverage

**Reference**: docs/SPECIFICATION.md Section 31

## Development Flow (PMAT-Enforced)

**MANDATORY Steps**:
1. **BASELINE**: `pmat quality-gate --fail-on-violation --checks=all`
2. **LOCATE**: Find spec in SPECIFICATION.md + task in docs/execution/roadmap.yaml (MANDATORY ticket)
3. **VERIFY**: Dependencies complete via roadmap DAG
4. **CHECK PATTERNS**: Use GENCHI GENBUTSU - examine existing code patterns (assert_cmd, proptest, cargo-mutants)
5. **IMPLEMENT**: <10 complexity (verify via `pmat analyze complexity`) + Zero SATD
6. **VALIDATE**: `pmat quality-gate` before commit (TDG ≥A-, 85 points)
7. **COMMIT**: With ticket reference (only if PMAT passes)

**TDG Violation Response**: HALT → ANALYZE (`pmat tdg <file> --include-components`) → REFACTOR → VERIFY A-

## Sprint Hygiene Protocol

### Pre-Sprint Cleanup (MANDATORY)
```bash
# Remove all debug binaries before starting sprint
rm -f test_* debug_* 
find . -type f -executable -not -path "./target/*" -not -path "./.git/*" -delete

# Verify no large files
find . -type f -size +100M -not -path "./target/*" -not -path "./.git/*"
```

---

**Remember**: Compiler engineering is about systematic transformation, not clever hacks. Every abstraction must have zero runtime cost. Every error must be actionable. Every line of code must justify its complexity budget.

## Documentation Standards

**Professional Documentation Requirements**:
- Use precise, factual language without hyperbole or marketing speak
- Avoid excessive exclamation marks and celebratory language
- State achievements and features objectively
- Focus on technical accuracy over promotional language
- Never create documentation files proactively unless explicitly requested
- Documentation should be maintainable and verifiable