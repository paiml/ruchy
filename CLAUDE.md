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

### 🚨 CRITICAL: Phantom UI Prevention Protocol (DEFECT-E2E-PHANTOM-UI Response)

**ROOT CAUSE LEARNED**: E2E tests were written for non-existent UI elements, causing 100% test failure rate.

**Reference**: `docs/issues/DEFECT-E2E-PHANTOM-UI.md`

**MANDATORY PROTOCOL** (Before Writing ANY E2E Test):

1. 🔍 **GENCHI GENBUTSU**: Read actual HTML file BEFORE writing tests
   - Example: Read `static/notebook.html` to see real element IDs
   - NEVER assume UI elements exist based on planned/future designs

2. ✅ **Selector Validation**: Verify EVERY selector exists in actual HTML
   ```typescript
   // ❌ FORBIDDEN: Writing tests for phantom UI
   await page.locator('#status').toHaveClass(/status-ready/); // Element doesn't exist!

   // ✅ CORRECT: Use actual notebook UI elements
   await page.waitForSelector('#notebook-cells', { timeout: 10000 });
   await expect(page.locator('.CodeMirror').first()).toBeVisible();
   ```

3. 🚫 **No Phantom UI**: NEVER write tests for planned/future UI elements
   - Tests describe CURRENT reality, not future plans
   - If UI doesn't exist yet, don't write E2E tests for it

4. ✅ **Manual Verification**: View page in browser to confirm elements exist
   - Open `static/notebook.html` in browser
   - Use DevTools to inspect actual element IDs and classes
   - Take screenshot if needed for reference

**Toyota Way**: GENCHI GENBUTSU - Go and see what's actually there, don't test phantom UI

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
4. ✅ **EXTREME TDD FIX** (RED→GREEN→REFACTOR):
   - **RED**: Write failing test that reproduces bug FIRST
   - **GREEN**: Fix bug with minimal code changes
   - **REFACTOR**: Apply PMAT quality gates (A- minimum, ≤10 complexity)
5. 🧪 **PROPERTY TESTS**: Verify invariants with 10K+ random inputs
6. 🧬 **MUTATION TESTS**: Prove tests catch real bugs (≥75% mutation coverage)
7. ✅ **COMMIT**: Document fix with ticket reference

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

### 🚀 DEBUGGING TOOLKIT (RuchyRuchy v1.9.0+ - PRODUCTION READY)

**✅ NOW AVAILABLE**: Comprehensive debugging tools via ruchydbg CLI (installed via `cargo install ruchyruchy`)

**What EXISTS Now (v1.9.0)**:
- ✅ RuchyRuchy repository: https://github.com/paiml/ruchyruchy
- ✅ `ruchydbg run` - Timeout detection + type-aware tracing (Ruchy v3.149.0+)
- ✅ `ruchydbg validate` - Validates debugging infrastructure
- ✅ Bug discovery: Property tests, differential tests, fuzz tests, code churn analysis
- ✅ Bug replication: Delta debugging, git bisection, test generation
- ✅ Bug reporting: GitHub integration, Five-Whys analysis
- ✅ Quality tools: 10 static analysis tools (TDG, dead code, ML defect prediction, etc.)

**Bug Detection Workflow** (Automated, 2-5 minutes):
1. **Create Test File**: Property-based test with expected behavior (1 minute)
   ```ruchy
   // test_issue.ruchy - Property: Must NOT hang
   struct Logger { level: LogLevel }
   impl Logger { fun test(&self) { self.level as i32; } }
   ```

2. **Run with ruchydbg**: Detect hangs + type-aware tracing (30 seconds)
   ```bash
   # NOW AVAILABLE: Automated timeout + tracing
   ruchydbg run test_issue.ruchy --timeout 1000 --trace
   # Exit 0 = pass, Exit 124 = TIMEOUT (bug detected!)
   # Shows: TRACE: → test(&self: Logger), types help identify root cause
   ```

3. **GENCHI GENBUTSU**: Read actual code to find root cause (3-10 minutes)
   ```bash
   # Use type information from trace to narrow search
   # grep for specific types/methods shown in trace output
   ```

4. **Validate Fix**: Use Ruchy's 15 native tools + type tracing (1 minute)
   ```bash
   ruchy check test.ruchy          # Syntax validation
   ruchy --trace run test.ruchy    # Runtime with type tracing
   ruchy ast test.ruchy            # AST visualization
   ```

**Time Savings**: Manual (30+ min/bug) → ruchydbg + tracing (2-5 min/bug) = **6-15x faster**

**Type-Aware Tracing (Ruchy v3.149.0+)**:
```bash
# Shows argument and return types for debugging
ruchy --trace run test.ruchy
# Or via ruchydbg wrapper:
ruchydbg run test.ruchy --trace
# Output: TRACE: → square(5: integer), TRACE: ← square = 25: integer
```

**Stack Depth Profiler (RuchyRuchy v1.11.0+ - DEBUGGER-041)**:
```bash
# Profile recursion depth and function hotspots
ruchydbg profile --stack recursive_code.ruchy

# Example output:
# === Stack Depth Profile ===
# Max depth: 10 (deepest call stack)
# Total calls: 77
# Call counts:
#   fibonacci: 67 calls ← HOTSPOT (exponential recursion)
#   factorial: 10 calls
# Deepest call stack: Shows full call chain at max depth
```

**Pathological Performance Detector (DEBUGGER-042 - Documented, Integration Pending)**:
```bash
# NOTE: Book chapter complete, CLI integration pending
# Will be available in future ruchydbg version
# ruchydbg profile --pathological code.ruchy

# Example output:
# === Pathological Performance Analysis ===
# Baseline (n=10): 0.15ms
# Scaled (n=100): 245.32ms
# Performance cliff detected! 16,354x slowdown (expected ~100x for O(n²))
#
# Likely causes:
# - Algorithmic complexity worse than O(n²)
# - Nested loops with exponential growth
# - Memory allocation churn

# Use cases:
# ✓ Find performance cliffs (fuzzing finds crashes, this finds slowdowns)
# ✓ Validate algorithmic complexity matches expectations
# ✓ Detect accidental exponential behavior
# ✓ Complement benchmarking (average-case) with worst-case analysis
```

**Use Cases**:
- **Find recursion issues BEFORE stack overflow** (catches depth > 30 early)
- **Identify hotspots** (fibonacci with 67 calls vs factorial with 10)
- **Debug deep recursion patterns** (view exact call chain at max depth)
- **Performance**: <1% overhead, zero overhead when disabled

**Real-World Example**:
```bash
# Before fix: test_deep_recursion_within_limit crashed (stack overflow at depth 150)
# After profiling: Detected max_depth=10 for factorial(10)
# Used profile data to set safe MAX_CALL_DEPTH=30 (prevents Rust stack overflow)
```

**Resources**:
- Installation: `cargo install ruchyruchy` (includes ruchydbg CLI)
- Integration Guide: `../ruchyruchy/INTEGRATION_GUIDE.md` (comprehensive usage)
- Quick Start: `../ruchyruchy/QUICK_START_FOR_RUCHY_DEVS.md` (10-minute tutorial)
- Bug Patterns: `../ruchyruchy/WHACK_A_MOLE_BUG_HUNTERS_GUIDE.md` (1,200+ lines)
- Stack Profiler: `../ruchyruchy/book/src/phase4_debugger/debugger-041-stack-profiler.md` (DEBUGGER-041)
- Repository: https://github.com/paiml/ruchyruchy

**Success Story - Issue #79 (2025-10-29)**:
- ✅ Used `ruchydbg run --timeout` to detect enum cast hang instantly
- ✅ Applied timeout methodology: detected bug in 1 second
- ✅ Used GENCHI GENBUTSU to find root causes (dispatch_method_call, eval_struct_instance_method)
- ✅ Fixed 2 bugs (RUNTIME-093, RUNTIME-094) with 8/8 tests passing
- ⏱️ **Time**: ~2 hours total (vs estimated 4-6 hours manual debugging)
- 📊 **Detection**: Bug found in 1 second with timeout vs 30+ minutes manually

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

## 🚨 CRITICAL: Claude Zombie Process Prevention

**ROOT CAUSE OF PERSISTENT HANGS**: Zombie Claude Code processes accumulate over time, consuming CPU and memory.

**Discovered 2025-10-23**: 6 zombie Claude processes:
- 3 processes: **194-197% CPU** (2 full cores each), running **111-231 DAYS** (3-7 months!)
- 3 processes: **8-10% CPU**, running **9-10 days**, 576-987MB RAM each
- **Impact**: System load 7.4, 22GB swap usage, competing for resources with active sessions

### Prevention Protocol (MANDATORY):

**Before starting ANY Claude session:**
```bash
# 1. Check for zombie Claude processes (>1 day old)
ps aux | grep claude | grep -v grep

# 2. Kill zombies older than 1 day
pkill -f "claude --dangerously-skip-permissions" --older 86400

# 3. Verify system health
uptime  # Load should be <4 on this system
free -h # Swap usage should be <5GB
```

### Daily Cleanup (MANDATORY):

```bash
# Add to crontab: Kill Claude processes older than 24 hours
0 */6 * * * pkill -f "claude --dangerously-skip-permissions" --older 86400

# Check system load
0 */1 * * * uptime >> /tmp/system-load-monitor.log
```

### When to Manually Kill Claude Sessions:

1. **High CPU** (>50% for >10 minutes) → Old background session, kill it
2. **Old sessions** (>24 hours) → Likely zombie, kill it
3. **High memory** (>2GB for single session) → Memory leak, kill it
4. **High system load** (>4 sustained) → Check for zombies: `ps aux | grep claude`

**Why This Matters:**
- Zombie Claude processes accumulate like zombie ruchy processes
- Each zombie consumes 1-2 CPU cores indefinitely
- Memory and swap usage grow over time
- Active sessions compete with zombies, causing hangs

**Reference Incident**: 2025-10-23
- Before: 6 zombies, load 7.4, 22GB swap, sessions freezing
- After: All killed, load 5.5 → 2.0, 7GB swap, sessions stable

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

### 🚨 CRITICAL: Pre-commit Hook Management (PMAT-Generated)

**ABSOLUTE PROHIBITION**: Quality gate checks MUST NEVER be "temporarily disabled"

**Reference Issue**: `docs/issues/PMAT-HOOK-DISABLED-CHECKS.md` - Comprehensive Five Whys analysis

**Problem**: PMAT-generated pre-commit hooks (`.git/hooks/pre-commit`) may contain commented-out checks with "⚠️ (temporarily disabled)" warnings. This violates Toyota Way "Stop the Line" principle.

**Toyota Way Violation**:
```bash
# ❌ FORBIDDEN PATTERN (found in auto-generated hooks)
# 1. Complexity analysis (TEMPORARILY DISABLED for PARSER-055 commit)
# TODO: Re-enable after fixing cognitive complexity violations
echo -n "  Complexity check... "
echo "⚠️  (temporarily disabled)"
```

**Correct Approach (MANDATORY)**:
1. 🛑 **STOP THE LINE**: If quality gate fails, halt work immediately
2. 🔍 **FIVE WHYS**: Perform root cause analysis (documented in tracking issue)
3. 🔧 **FIX VIOLATIONS**: Refactor code to meet quality standards (≤10 complexity, zero SATD)
4. ✅ **RE-ENABLE CHECKS**: Use `pmat hooks refresh` to regenerate with checks enabled
5. 📝 **VERIFY**: Test commit to confirm gates block violations

**When Pre-commit Hook Shows Disabled Checks**:
```bash
# 1. Identify disabled checks
grep "temporarily disabled" .git/hooks/pre-commit

# 2. Document issue (if not already tracked)
# Create docs/issues/PMAT-HOOK-DISABLED-CHECKS-<DATE>.md

# 3. Fix underlying violations
pmat analyze complexity --max-cyclomatic 10  # Find violations
pmat analyze satd                             # Find SATD comments
# Refactor files until all violations fixed

# 4. Regenerate hooks (after PMAT upstream fix)
pmat hooks refresh

# 5. Verify gates work
git commit  # Should block if violations exist
```

**If PMAT Not Yet Fixed** (Temporary Workaround):
1. Create tracking issue in PMAT repository
2. Manually uncomment checks in `.git/hooks/pre-commit`
3. Add header comment: `# TEMPORARY FIX: Re-enabled until PMAT upstream fix`
4. Fix all violations before committing
5. Document in commit message why manual hook edit was necessary

**FORBIDDEN RESPONSES**:
- ❌ "Let's just disable the check for now"
- ❌ "We'll fix it later"
- ❌ "Use `--no-verify` to bypass"
- ❌ "It's only temporary"

**Toyota Way**: Temporary bypasses become permanent. Fix root causes, never mask symptoms.

### 🚨 MANDATORY: RuchyRuchy Smoke Testing Protocol (Pre-Release)

**SACRED RULE**: NEVER release without RuchyRuchy smoke testing. All transpiler fixes MUST be verified end-to-end.

**Reference**: Issue #111 (v3.168.0) - Smoke testing caught 0 regressions, verified all 4 fixes

**MANDATORY PROTOCOL** (Before EVERY Release):

**Prerequisites**:
```bash
# Install ruchydbg (if not already installed)
cargo install ruchyruchy  # Provides ruchydbg CLI
ruchydbg --version  # Verify v1.12.0+
```

**Step 1: Create Smoke Tests for Each Fix**

For EACH transpiler/runtime fix in the release, create a smoke test file:

```bash
# Example: DEFECT-018 (moved value in loop)
cat > /tmp/smoke_defect_018.ruchy << 'EOF'
// SMOKE TEST: DEFECT-018 - Auto-clone in nested loops
struct Process { pid: i32, name: String }
struct Rule { id: i32, enabled: bool }

fun rule_matches(rule: Rule, proc: Process) -> bool {
    rule.enabled && proc.pid > 0
}

fun find_matches() -> i32 {
    let procs = vec![Process { pid: 1, name: "init" }];
    let rules = vec![Rule { id: 1, enabled: true }];
    let mut i = 0;
    let mut matches = 0;

    while i < procs.len() {
        let proc = procs[i];
        let mut j = 0;
        while j < rules.len() {
            let rule = rules[j];
            if rule_matches(rule, proc) {  // Auto-clone test
                matches = matches + 1;
            }
            j = j + 1;
        }
        i = i + 1;
    }
    matches
}

println(find_matches());
EOF
```

**Step 2: Run Individual Fix Smoke Tests**

```bash
# Test each fix individually with timeout detection + tracing
ruchydbg run /tmp/smoke_defect_018.ruchy --timeout 5000 --trace

# Expected output:
# ✅ SUCCESS
# ⏱️  Execution time: <10ms
# 🔍 Type-aware tracing: enabled
# Correct output verified
```

**Step 3: Create Integration Test (All Fixes Combined)**

```bash
# Combine all fixes into one comprehensive test
cat > /tmp/smoke_integration.ruchy << 'EOF'
// INTEGRATION TEST: All fixes from this release
// Include patterns from EACH fix combined
EOF

ruchydbg run /tmp/smoke_integration.ruchy --timeout 5000 --trace
```

**Step 4: Pathological Input Detection**

```bash
# Check for performance regressions
ruchydbg detect /tmp/smoke_integration.ruchy --threshold 15

# ⚠️ If pathological input detected:
# - Verify it's expected algorithmic complexity (O(n²) is OK for nested loops)
# - Ensure execution completes (no hangs)
# - Performance should be <1s for smoke tests
```

**Step 5: End-to-End Verification with Real Project**

```bash
# Test with actual user project (e.g., reaper, ruchy-book examples)
timeout 60 ruchy compile /path/to/real/project/main.ruchy

# Verify:
# ✅ Compiles with 0 errors
# ✅ Binary executes correctly
# ✅ No regressions in existing functionality
```

**Step 6: Generate Smoke Test Report**

```bash
cat > /tmp/ruchydbg_smoke_test_report.md << 'EOF'
# RuchyRuchy Smoke Test Report
## Version: vX.Y.Z

### Individual Fix Tests
- ✅ DEFECT-XXX: [description] - SUCCESS (Xms)
- ✅ DEFECT-YYY: [description] - SUCCESS (Xms)

### Integration Test
- ✅ All fixes combined: SUCCESS (Xms)

### Performance Analysis
- ⚠️ Pathological inputs: [analysis]

### End-to-End Verification
- ✅ Project: [name] - Compiles + Executes

### Summary
ALL SMOKE TESTS PASSED ✅
EOF
```

**MANDATORY Checklist** (Before Publishing):
- [ ] Individual smoke tests created for each fix
- [ ] All smoke tests pass with ruchydbg
- [ ] Integration test passes
- [ ] Performance analysis complete (no unexpected regressions)
- [ ] End-to-end verification with real project
- [ ] Smoke test report generated

**FORBIDDEN RESPONSES**:
- ❌ "The fix looks good, let's skip smoke testing"
- ❌ "We already have unit tests, smoke tests are redundant"
- ❌ "This is a small fix, doesn't need smoke testing"
- ❌ "Let's smoke test after release"

**Toyota Way**: Smoke testing is GATE 0 for releases. No smoke tests = no release.

**Success Metrics** (v3.168.0):
- 3 individual smoke tests: ALL PASSED (3-4ms each)
- 1 integration test: PASSED (4ms)
- Pathological detection: Expected O(n²), no hangs
- Reaper (5,100 LOC): 0 errors, executes correctly
- Time investment: ~15 minutes vs hours of debugging production issues

### DUAL-RELEASE PUBLISHING PROTOCOL
**After version bump**: Publish `ruchy` first, wait 30s, then `ruchy-wasm` (same version)
**Checklist**: Tests pass, CHANGELOG updated, git commit/push, both crates build

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