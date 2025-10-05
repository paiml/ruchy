# CLAUDE.md - Ruchy Compiler Implementation Protocol

## IMPORTANT: Auto-Generated Files
**NEVER EDIT `deep_context.md`** - This file is auto-generated and will be overwritten. Any changes should be made to the source files instead.

## Prime Directive

**Generate correct code that compiles on first attempt. Quality is built-in, not bolted-on.**

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
fn process_data(items: Vec<Item>) -> Result<Output> {
    let mut results = Vec::new();
    for item in items {
        if item.valid {
            if item.type == "A" {
                // ... 20 more lines of nested logic
            }
        }
    }
    // ... more complexity
}

// ✅ GOOD: Complexity ≤10
fn process_data(items: Vec<Item>) -> Result<Output> {
    items.into_iter()
        .filter(|item| item.valid)
        .map(process_single_item)
        .collect()
}

fn process_single_item(item: Item) -> Result<ItemOutput> {
    match item.item_type {
        ItemType::A => process_type_a(item),
        ItemType::B => process_type_b(item),
    }
}
```

## EXTREME TDD Protocol (CRITICAL RESPONSE TO PARSER FAILURES)

**ANY PARSER OR TRANSPILER BUG REQUIRES IMMEDIATE EXTREME TDD RESPONSE:**

### Critical Bug Response (MANDATORY):
1. **HALT ALL OTHER WORK**: Stop everything when parser/transpiler bugs found
2. **EXTREME TEST COVERAGE**: Create comprehensive test suites immediately:
   - Unit tests for every parser rule
   - Integration tests for complete programs  
   - Property tests with random inputs (10,000+ iterations)
   - Fuzz tests for edge cases
   - Doctests in every public function
   - `cargo run --examples` MUST pass 100%
3. **REGRESSION PREVENTION**: Add failing test BEFORE fixing bug
4. **COMPREHENSIVE VALIDATION**: Test all language features after any fix

### Test Coverage Requirements (MANDATORY):
- **Parser Tests**: Every token, every grammar rule, every edge case
- **Transpiler Tests**: Every Ruchy construct → Rust construct mapping
- **Integration Tests**: Full compile → execute → validate pipeline
- **Property Tests**: Automated generation of valid/invalid programs
- **Fuzz Tests**: Random input stress testing (AFL, cargo-fuzz)
- **Mutation Tests**: 80%+ mutation coverage via cargo-mutants (empirical validation)
- **Examples Tests**: All examples/ must compile and run

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

#### Test Gap Patterns (From Sprint 8 Empirical Data):
1. **Match Arm Deletions** (most common): Test ALL match arms with assertions
2. **Function Stub Replacements**: Validate return values are real data, not None/empty
3. **Boundary Conditions**: Test <, <=, ==, >, >= explicitly
4. **Boolean Negations**: Test both true AND false branches
5. **Operator Changes**: Test +/-, */%, <=/>=, &&/|| alternatives

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

## QDD (Quality-Driven Development) Protocol

**QUALITY IS THE DRIVER, NOT AN AFTERTHOUGHT - BASED ON PMAT BOOK CH14**

### QDD Core Principles:
1. **Quality Metrics First**: Define quality metrics BEFORE writing code
2. **Continuous Monitoring**: Real-time quality tracking during development
3. **Automated Enforcement**: Quality gates that cannot be bypassed
4. **Data-Driven Decisions**: Let metrics guide development priorities
5. **Preventive Maintenance**: Fix quality issues before they become technical debt

### QDD Implementation with PMAT:
```bash
# BEFORE starting any task - establish quality baseline
pmat tdg . --min-grade A- --format=json > quality_baseline.json
pmat analyze complexity --format=csv > complexity_baseline.csv

# DURING development - continuous quality monitoring
pmat tdg dashboard --port 8080 --update-interval 5 &  # Real-time monitoring
watch -n 5 'pmat quality-gate --quiet || echo "QUALITY DEGRADATION DETECTED"'

# AFTER each function/module - verify quality maintained
pmat tdg <file> --compare-baseline quality_baseline.json
pmat analyze complexity <file> --max-cyclomatic 10 --max-cognitive 10

# BEFORE commit - comprehensive quality validation
pmat tdg . --min-grade A- --fail-on-violation
pmat quality-gate --fail-on-violation --format=detailed
```

### QDD Metrics Hierarchy:
1. **Code Quality Metrics** (via PMAT TDG):
   - Cyclomatic Complexity: ≤10 per function
   - Cognitive Complexity: ≤10 per function
   - Code Duplication: <10% across codebase
   - Documentation Coverage: >70% for public APIs
   - Technical Debt: 0 SATD comments allowed

2. **Test Quality Metrics** (via cargo llvm-cov):
   - Line Coverage: ≥80% per module
   - Branch Coverage: ≥75% per module
   - Function Coverage: 100% for public APIs
   - Test Diversity: Unit + Integration + Property + Fuzz

3. **Performance Metrics** (via cargo bench):
   - Regression Detection: ±5% performance variance allowed
   - Memory Usage: Track peak and average
   - Compilation Speed: <1s for incremental builds

### QDD Workflow Integration:
```yaml
Development Cycle:
1. DEFINE: Quality metrics for the task
2. MEASURE: Baseline quality before changes
3. DEVELOP: Write code with real-time monitoring
4. VALIDATE: Ensure all metrics maintained/improved
5. DOCUMENT: Record quality impact in commit message
```

## Toyota Way Implementation

**STOP THE LINE FOR ANY DEFECT. NO DEFECT IS TOO SMALL. NO SHORTCUT IS ACCEPTABLE.**

### Core Toyota Principles:
1. **Jidoka (Autonomation)**: Build quality into the process, detect problems immediately
2. **Genchi Genbutsu**: Go to the source, understand the root cause through direct observation
3. **Kaizen**: Continuous improvement through systematic problem-solving
4. **Respect for People**: Create systems that prevent human error, not blame people
5. **Long-term Philosophy**: Short-term fixes create long-term problems

### Defect Response Protocol (MANDATORY):
```
1. HALT DEVELOPMENT: Stop all forward progress when defect detected
2. ROOT CAUSE ANALYSIS: Use 5 Whys to find true cause, not symptoms  
3. POKA-YOKE: Design error-prevention into the system
4. SYSTEMATIC TESTING: Add comprehensive test coverage to prevent recurrence
5. PROCESS IMPROVEMENT: Update development process to catch similar issues earlier
6. VERIFICATION: Prove the fix works and cannot regress
```

### Testing Hierarchy (Systematic Defect Prevention):
```
Level 1: Unit Tests         - Function-level correctness
Level 2: Integration Tests  - Component interaction  
Level 3: E2E Tests          - Full system behavior
Level 4: Property Tests     - Mathematical invariants
Level 5: Fuzz Tests         - Random input robustness
Level 6: Regression Tests   - Historical defect prevention
Level 7: Performance Tests  - Non-functional requirements
```

**NEVER AGAIN RULE**: Every defect must be made impossible to repeat through systematic prevention.

### Mandatory Testing Requirements (80% Property Test Coverage)

**CRITICAL**: Following paiml-mcp-agent-toolkit Sprint 88 success pattern:

1. **Property Test Coverage**: Target 80% of all modules with property tests
2. **Doctests**: Every public function MUST have runnable documentation examples
3. **Property Tests**: Use proptest to verify invariants with 10,000+ random inputs
4. **Fuzz Tests**: Use cargo-fuzz or AFL to find edge cases with millions of inputs
5. **Examples**: Create working examples in examples/ directory demonstrating correct usage
6. **Integration Tests**: End-to-end scenarios covering real-world usage patterns
7. **Regression Tests**: Specific test case that reproduces and prevents the exact defect

**Property Test Injection Pattern** (from pmat):
```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_function_never_panics(input: String) {
            let _ = function_under_test(&input); // Should not panic
        }
        
        #[test]
        fn test_invariant_holds(a: i32, b: i32) {
            let result = add(a, b);
            prop_assert_eq!(result, b + a); // Commutative property
        }
    }
}
```

**Code Coverage Requirements** (QUALITY-008 Implemented):
- **Current Baseline**: 33.34% overall (post-QUALITY-007 parser enhancements)
- **Regression Prevention**: Pre-commit hooks BLOCK commits below baseline
- **Direction**: Coverage must increase or stay same, NEVER decrease
- **Parser Improvements**: Character literals, tuple destructuring, rest patterns now working
- **Pattern Test Results**: 2 passing → 4 passing (100% improvement achieved)
- **Enforcement**: Automated coverage checking with clear error messages

## PMAT TDG Quality Enforcement (MANDATORY - BLOCKING)

**CRITICAL**: PMAT TDG (Technical Debt Grading) v2.39.0+ quality gates are MANDATORY and BLOCKING. NO EXCEPTIONS.

### TDG Quality Standards (Zero Tolerance - v2.39.0):
- **Overall Grade**: Must maintain A- or higher (≥85 points) - HARD LIMIT
- **Structural Complexity**: ≤10 per function (enforced via TDG)
- **Semantic Complexity**: Cognitive complexity ≤10 (enforced via TDG) 
- **Code Duplication**: <10% code duplication (measured via TDG)
- **Documentation Coverage**: >70% for public APIs (tracked via TDG)
- **Technical Debt**: Zero SATD comments (zero-tolerance via TDG)
- **Coupling Analysis**: Module dependency limits (enforced via TDG)
- **Consistency Score**: Naming/style consistency ≥80% (enforced via TDG)

### MANDATORY TDG Commands (v2.39.0 - All Development):

#### Before ANY Code Changes:
```bash
# MANDATORY: TDG baseline check with comprehensive analysis
pmat tdg . --min-grade A- --fail-on-violation
pmat quality-gate --fail-on-violation --format=summary
```

#### During Development (After Each Function/Module):
```bash
# MANDATORY: File-level TDG analysis
pmat tdg <file.rs> --include-components --min-grade B+

# MANDATORY: Traditional complexity verification (backup)
pmat analyze complexity --max-cyclomatic 10 --max-cognitive 10 --fail-on-violation

# MANDATORY: SATD detection (zero tolerance)
pmat analyze satd --format=summary --fail-on-violation
```

#### End of Sprint (Before Commit):
```bash
# MANDATORY: Comprehensive TDG quality gate (BLOCKS commits)
pmat tdg . --min-grade A- --format=sarif --output=tdg-report.sarif
pmat quality-gate --fail-on-violation --format=detailed

# MANDATORY: Real-time dashboard check
pmat tdg dashboard --port 8080 --open  # Verify no regressions

# MANDATORY: Export comprehensive analysis
pmat tdg . --format=markdown --output=TDG_QUALITY_REPORT.md
```

### TDG Integration Protocol (Toyota Way v2.39.0):
1. **HALT DEVELOPMENT**: Stop on ANY TDG grade below A- (85 points)
2. **ROOT CAUSE**: Use `pmat tdg <file> --include-components` to identify exact issues
3. **REFACTOR IMMEDIATELY**: Address all TDG component failures systematically
4. **DASHBOARD MONITORING**: Use `pmat tdg dashboard` for real-time quality tracking
5. **VERIFY FIX**: Re-run `pmat tdg <file>` to prove A- grade achievement
6. **MCP INTEGRATION**: Use MCP tools for enterprise-grade external integration

### Complexity Decomposition Strategy:
```rust
// BEFORE (Complexity: 72 - VIOLATION)
fn giant_function() { /* 200 lines */ }

// AFTER (Each <10 complexity - COMPLIANT)  
fn orchestrator() { /* calls helpers */ }
fn helper_one() { /* focused responsibility */ }
fn helper_two() { /* focused responsibility */ }
fn helper_three() { /* focused responsibility */ }
```

## Toyota Way Success Stories

### Property Testing Victory (2024-12)
- **545 systematic test cases**: 0 parser inconsistencies found
- **ROOT CAUSE**: Manual testing methodology error, NOT code defect
- **LESSON**: Property testing is objective - mathematically proves system behavior

### PMAT Enforcement Success (2025-08-30)
**Discovery**: 3,557 quality violations found
- **Finding**: Functions with 72x complexity limit (720 vs 10)
- **SATD debt**: 1,280 technical debt comments
- **Dead code**: 6 violations indicating maintenance debt
- **Root cause**: PMAT quality gates not enforced during development
- **Solution**: Mandatory PMAT enforcement at every development step
- **Lesson**: Quality must be built-in from start, not bolted-on later

### Language Completeness Achievement v1.9.1 (2025-08)
**Systematic Testing Revealed**:
- ✅ **Fat arrow syntax**: Functional (`x => x + 1`)
- ✅ **String interpolation**: Functional (`f"Hello {name}"`)
- ✅ **Async/await**: Functional (async fn and await expressions)
- ✅ **DataFrame literals**: Functional (`df![]` macro)
- ✅ **Generics**: Functional (`Vec<T>`, `Option<T>`)
- ✅ **Pipeline Operator**: `|>` for functional programming (v1.9.0)
- ✅ **Import/Export**: Module system evaluation (v1.9.1)
- ✅ **String Methods**: Complete suite (v1.8.9)

**ROOT CAUSE**: Manual testing created false negatives. Features were already implemented.

### Quality Excellence Sprint v1.6.0
**Results**: 107 tests created, 287 tests passing, 80% coverage achieved
- LSP module coverage: 0% → 96-100% 
- MCP module coverage: 0% → 33%
- Type inference coverage: 0% → 15%

### Complete Language Restoration - Status
**Core Functionality Status**:
- Basic Math, Float Math, Variables ✅
- String Concatenation, Method Calls ✅  
- Boolean Operations, Complex Expressions ✅
- Reserved Keywords: final → r#final (automatic) ✅

## Scripting Policy

**CRITICAL**: Use ONLY Ruchy scripts for adhoc scripting and testing. No Python, Bash scripts, or other languages for testing Ruchy functionality.

✅ **Allowed**: `*.ruchy` files loaded via `:load` command in REPL
❌ **Forbidden**: Python scripts, shell scripts, or any non-Ruchy testing code

## Implementation Hierarchy

```yaml
Navigation:
1. SPECIFICATION.md     # What to build (reference)
2. docs/execution/roadmap.md  # Strategic priorities and current tasks
3. docs/execution/      # Tactical work breakdown
4. ../ruchy-book/INTEGRATION.md  # Book compatibility tracking
5. CHANGELOG.md         # Version history and release notes
```

## Book Compatibility Monitoring

**CRITICAL**: Check `../ruchy-book/INTEGRATION.md` FREQUENTLY for:
- Current compatibility: 19% (49/259 examples) + 100% one-liners (20/20)
- v1.9.1 Language Completeness: Pipeline operator, Import/Export, String methods
- Regression detection from previous versions

## Quality Status (v1.9.3)

**INTERPRETER COMPLEXITY**: 
- evaluate_expr: 138 (was 209, target <50)
- Value::fmt: 66 (target <30)
- Value::format_dataframe: 69 (target <30)
- **Latest Features**: Math functions (sqrt, pow, abs, min, max, floor, ceil, round)

## Critical Quality Gate Defect (Toyota Way Investigation)

**DEFECT**: Pre-commit hook hangs at dogfooding test

**ROOT CAUSE**: Transpiler generates different code in debug vs release mode, violating determinism

**PREVENTION**: 
- Add property test: `assert!(transpile_debug(x) == transpile_release(x))`
- Quality gates must use consistent binary paths
- Never allow behavioral differences between debug/release

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

1. **ALWAYS Use Ticket Numbers**: Every commit, PR, and task MUST reference a ticket ID from docs/execution/roadmap.md
2. **Roadmap-First Development**: No work begins without a corresponding roadmap entry
3. **Ticket Format**: Use format "QUALITY-XXX", "PARSER-XXX", "DF-XXX", "WASM-XXX" per roadmap
4. **Traceability**: Every change must be traceable back to business requirements via ticket system
5. **Sprint Planning**: Work is organized by sprint with clear task dependencies and priorities

### Pre-Implementation Verification (PMAT-Enforced)
```rust
// HALT. Before implementing ANY feature:
□ Run PMAT baseline: `pmat quality-gate --fail-on-violation --checks=all`
□ Check ../ruchy-book/INTEGRATION.md for latest compatibility report
□ Check ../ruchy-book/docs/bugs/ruchy-runtime-bugs.md for known issues
□ Locate specification section in SPECIFICATION.md
□ Find task ID in docs/execution/roadmap.md (MANDATORY)
□ Verify ticket dependencies completed via DAG
□ Reference ticket number in all commits/PRs
□ Check existing patterns in codebase
□ PMAT complexity check: `pmat analyze complexity --max-cyclomatic 10`
□ Confirm complexity budget (<10 cognitive) via PMAT verification
□ Zero SATD: `pmat analyze satd --fail-on-violation`
```

### Commit Message Format (MANDATORY with TDG Tracking)
```
[TICKET-ID] Brief description

Detailed explanation of changes
- Specific improvements made
- Test coverage added
- Performance impact
- Breaking changes (if any)

TDG Score Changes (MANDATORY):
- src/file1.rs: 85.3→87.1 (B+→A-) [+1.8 improvement]
- src/file2.rs: 72.5→72.5 (B-→B-) [stable]
- File Hash: abc123def456...

Closes: TICKET-ID
```

**Example**:
```
[QUALITY-001] Refactor parse_prefix complexity from 161 to 8

Extracted 8 helper functions following single responsibility principle:
- parse_literal_prefix (complexity: 5)
- parse_string_prefix (complexity: 4)
- parse_identifier_prefix (complexity: 7)

TDG Score Changes:
- src/frontend/parser.rs: 68.2→82.5 (C+→B+) [+14.3 improvement]
- tests/parser_test.rs: 91.0→92.1 (A→A) [+1.1 improvement]
- File Hash: 7f3a9b2c4e8d1a5f6b9c2d4e8f1a3b5c7d9e1f3a

PMAT Verification:
- Complexity: 161→8 (95% reduction)
- SATD: 0 violations maintained
- Dead Code: <5% threshold maintained

Added comprehensive doctests and property tests for all helpers.
Performance trade-off: ~100ms -> ~200ms acceptable for maintainability.

Closes: QUALITY-001
```

## MANDATORY: TDG (Technical Debt Gradient) Transactional Tracking

**CRITICAL**: Every commit MUST include TDG score tracking to prevent debt drift.

### TDG Scoring System
- **A+ (95-100)**: Excellent - No action needed
- **A (90-94)**: Very Good - Maintain quality
- **B (80-89)**: Good - Monitor for degradation  
- **C (70-79)**: Fair - Requires improvement
- **D (60-69)**: Poor - Priority refactoring needed
- **F (<60)**: Critical - MUST fix immediately

### TDG Pre-Commit Protocol (MANDATORY)
```bash
# Run before EVERY commit to track debt changes:
./.tdg_tracking.sh || {
    echo "❌ TDG degradation detected - commit blocked"
    echo "Fix the code or add [TDG-OVERRIDE] with justification"
    exit 1
}
```

### TDG Components Tracked
1. **Structural Complexity** (25%): Cyclomatic/cognitive complexity
2. **Semantic Complexity** (20%): Type complexity, generics usage
3. **Duplication** (20%): Code duplication percentage
4. **Coupling** (15%): Module dependencies
5. **Documentation** (10%): Doc coverage percentage
6. **Consistency** (10%): Style/pattern adherence

### File Hash Tracking
- Each commit includes SHA256 hash of modified files
- Enables detection of uncommitted changes
- Prevents TDG score gaming by partial commits

### TDG Override Protocol
Only allowed with explicit justification:
```
[P0-EMERGENCY][TDG-OVERRIDE] Critical production fix

TDG Score Changes:
- src/runtime/repl.rs: 67.4→65.2 (C+→D+) [-2.2 degradation]

Override Justification: 
- Emergency fix for production outage
- Debt payback ticket created: DEBT-XXX
- Scheduled for next sprint

File Hash: 9a8b7c6d5e4f3a2b1c0d9e8f7a6b5c4d3e2f1a0b
```

### MANDATORY: TDG Commit Verification Protocol v2.39.0
**CRITICAL**: NO commits allowed without TDG A- grade verification

```bash
# MANDATORY before every commit (TDG v2.39.0):
pmat tdg . --min-grade A- --fail-on-violation || {
    echo "❌ COMMIT BLOCKED: TDG grade below A- threshold"
    echo "Run: pmat tdg . --include-components --top-files 5"
    echo "Use: pmat tdg dashboard --open for real-time analysis"
    echo "ALL TDG violations must achieve A- grade before commit"
    exit 1
}

# BACKUP: Traditional quality gate verification
pmat quality-gate --fail-on-violation --format=summary || {
    echo "❌ COMMIT BLOCKED: Traditional quality gate failed"
    exit 1
}
```

## Compiler Architecture Patterns

### Parser Pattern - Pratt with Error Recovery
```rust
impl Parser {
    fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_prefix()?;
        while let Some(&op) = self.peek() {
            let (l_bp, r_bp) = op.binding_power();
            if l_bp < min_bp { break; }
            self.advance();
            let rhs = self.parse_expr(r_bp)?;
            lhs = Expr::binary(op, lhs, rhs, self.span());
        }
        Ok(lhs)
    }
}
```

### Type Inference - Bidirectional Checking
```rust
impl TypeChecker {
    fn check(&mut self, expr: &Expr, expected: Type) -> Result<(), TypeError> {
        match (&expr.kind, expected) {
            (ExprKind::Lambda(params, body), Type::Function(arg_tys, ret_ty)) => {
                self.check_params(params, arg_tys)?;
                self.check(body, *ret_ty)
            }
            _ => {
                let inferred = self.infer(expr)?;
                self.unify(inferred, expected)
            }
        }
    }
}
```

## MANDATORY Quality Gates (BLOCKING - Not Advisory)

### SACRED RULE: NEVER BYPASS QUALITY GATES

**ABSOLUTELY FORBIDDEN**:
- `git commit --no-verify` - NEVER use this - NO EXCEPTIONS EVER
- Skipping tests "temporarily" - NO exceptions
- Ignoring failing quality checks - Must fix EVERY defect
- Dismissing warnings as "unrelated" - All defects matter

**Toyota Way Principle**: Stop the line for ANY defect. No defect is too small. No shortcut is acceptable.

**WHEN CLIPPY BLOCKS**: Always fix the root cause:
- Missing `# Errors` sections → Add proper documentation with examples
- Using `unwrap()` → Replace with `expect()` with meaningful messages  
- Dead code warnings → Remove or prefix with underscore
- Missing doctests → Add runnable examples to documentation

### MANDATORY RELEASE PUBLISHING PROTOCOL

**CRITICAL**: After EVERY version update, you MUST publish to crates.io immediately.

```bash
# MANDATORY after version bump and git push:
cargo publish                    # Publish main package only
# NOTE: ruchy-cli is DEPRECATED - do NOT publish (MUDA/waste)
```

### Pre-commit Hooks (MANDATORY)
```bash
#!/bin/bash
# .git/hooks/pre-commit - BLOCKS commits that violate quality
set -e

echo "🔒 MANDATORY Quality Gates Running..."

# GATE 1: TDG A- Grade Verification (PRIMARY)
echo "📊 MANDATORY: TDG A- grade verification..."
TDG_SCORE=$(timeout 60s pmat tdg . --quiet 2>/dev/null || echo "0")
if [ -n "$TDG_SCORE" ] && (( $(echo "$TDG_SCORE >= 85" | bc -l) )); then
    echo "✅ TDG Grade: $TDG_SCORE (≥85 A- required) - PASSED"
else
    echo "❌ BLOCKED: TDG grade $TDG_SCORE below A- threshold (85 points)"
    echo "Run: pmat tdg . --include-components --format=table"
    exit 1
fi

# GATE 2: Function-level quality checks
echo "📊 MANDATORY: Function-level quality gate..."
if ./scripts/quality-gate.sh src; then
    echo "✅ Quality gate passed"
else
    echo "❌ BLOCKED: Quality gate failed"
    echo "Fix all functions with complexity >10 and remove SATD comments"
    exit 1
fi

# GATE 3: Basic functionality test
echo "🧪 MANDATORY: Basic functionality test..."
if echo 'println("Hello")' | timeout 5s ruchy repl | grep -q "Hello"; then
    echo "✅ Basic functionality test passed"
else
    echo "❌ BLOCKED: Basic functionality test failed"
    echo "REPL cannot execute simple println"
    exit 1
fi

echo "✅ All quality gates passed"
```

## The Make Lint Contract (Zero Warnings Allowed)
```bash
# make lint command from Makefile:
cargo clippy --all-targets --all-features -- -D warnings
```

**Critical**: The `-D warnings` flag treats EVERY clippy warning as a hard error.

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

## The Development Flow (PMAT-Enforced)

### MANDATORY: PMAT Quality at Every Step
```
1. BASELINE CHECK: Run `pmat quality-gate --fail-on-violation --checks=all`
2. LOCATE specification section in SPECIFICATION.md
3. IDENTIFY task in execution roadmap with ticket number
4. VERIFY dependencies complete via roadmap DAG
5. IMPLEMENT with <10 complexity (verified by `pmat analyze complexity`)
6. VALIDATE: Run `pmat quality-gate` before ANY commit
7. COMMIT with task reference (only if PMAT passes)
```

### MANDATORY TDG Protocol v2.39.0 (BLOCKING):
```bash
# STEP 1: Pre-development TDG baseline (MANDATORY)
pmat tdg . --min-grade A- --format=table
pmat quality-gate --fail-on-violation --format=summary

# STEP 2: Real-time development monitoring (continuous)
pmat tdg dashboard --port 8080 --update-interval 5 &  # Background monitoring

# STEP 3: File-level verification (after each function/module)
pmat tdg <modified-file.rs> --include-components --min-grade B+

# STEP 4: Pre-commit TDG verification (MANDATORY - BLOCKS COMMITS)
pmat tdg . --min-grade A- --fail-on-violation
pmat quality-gate --fail-on-violation --format=detailed

# STEP 5: Sprint completion comprehensive analysis (MANDATORY)
pmat tdg . --format=markdown --output=TDG_SPRINT_REPORT_$(date +%Y%m%d).md
pmat tdg export . --all-formats --output-dir ./tdg-reports/
```

### TDG Violation Response v2.39.0 (IMMEDIATE):
1. **HALT**: Stop ALL development when TDG grade falls below A- (85 points)
2. **ANALYZE**: Use `pmat tdg <file> --include-components` to identify exact component failures
3. **DASHBOARD**: Use `pmat tdg dashboard` for real-time hotspot identification
4. **TARGETED REFACTOR**: Address specific TDG component issues (structural, semantic, etc.)
5. **VERIFY**: Re-run `pmat tdg <file>` to prove A- grade achievement
6. **TRENDING**: Use dashboard to verify no regression in other files
7. **MCP INTEGRATION**: Use enterprise MCP tools for external quality integration

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

## MANDATORY TDG Real-Time Monitoring (v2.39.0 - CONTINUOUS)

**NEW REQUIREMENT**: Real-time quality monitoring via TDG dashboard is MANDATORY for all development.

### TDG Dashboard Integration:
```bash
# MANDATORY: Start real-time monitoring at beginning of each session
pmat tdg dashboard --port 8080 --update-interval 5 --open

# Features in v2.39.0:
# - Real-time system metrics with 5-second updates
# - Storage backend monitoring (Hot/Warm/Cold tiers)
# - Performance profiling with flame graphs
# - Bottleneck detection (CPU, I/O, Memory, Lock contention)
# - Interactive analysis with Server-Sent Events
```

### TDG MCP Enterprise Integration:
```bash
# OPTIONAL: External tool integration via MCP server
pmat mcp serve --port 3000

# Available MCP tools for external integration:
# - tdg_analyze_with_storage: Enterprise-grade analysis with persistence
# - tdg_system_diagnostics: System health and performance monitoring
# - tdg_storage_management: Storage backend control and optimization
# - tdg_performance_profiling: Advanced profiling with flame graphs
# - tdg_alert_management: Configurable alert system
# - tdg_export_data: Multi-format export (JSON, CSV, SARIF, HTML, XML)
```

## MANDATORY TDG Rules (BLOCKING - NO EXCEPTIONS)

**CRITICAL**: ALL development MUST achieve TDG A- grade (≥85 points) before proceeding.

### TDG Enforcement Rules v2.39.0:
- **TDG BASELINE**: Run `pmat tdg . --min-grade A-` before ANY code changes
- **REAL-TIME MONITORING**: Keep `pmat tdg dashboard` running during development
- **COMPLEXITY LIMIT**: ≤10 cyclomatic complexity per function (TDG structural component)
- **COGNITIVE LIMIT**: ≤10 cognitive complexity per function (TDG semantic component)
- **ZERO SATD**: No TODO/FIXME/HACK comments (TDG technical debt component)
- **DOCUMENTATION**: >70% API documentation coverage (TDG documentation component)
- **DUPLICATION**: <10% code duplication (TDG duplication component)
- **CONSISTENCY**: ≥80% naming/style consistency (TDG consistency component)
- **ENTERPRISE MCP**: Use MCP tools for external integration and advanced analytics
- **COMMIT BLOCKING**: Pre-commit hooks MUST include TDG A- grade verification

### PMAT Violation Response (IMMEDIATE ACTION REQUIRED):
```bash
# When PMAT fails:
1. HALT: Stop ALL development immediately
2. ANALYZE: pmat analyze complexity --top-files 5 --format=detailed
3. IDENTIFY: Find functions >10 complexity 
4. REFACTOR: Extract functions to reduce complexity
5. VERIFY: Re-run pmat quality-gate to confirm fix
6. CONTINUE: Only proceed when ALL violations resolved
```

## 🔥 PMAT v2.68.0+ Advanced Quality Features (MANDATORY INTEGRATION)

### 📊 TDG Persistent Storage (NEW - Dogfooding Excellence)
**MANDATORY**: Use TDG persistent scoring for continuous quality tracking and historical analysis.

```bash
# Install latest PMAT with TDG storage features
cargo install pmat --force --version ">=2.68.0"

# Analyze files (automatically stores scores to ~/.pmat/tdg-*)
pmat tdg src/runtime/repl.rs --include-components

# Check storage statistics (monitor quality trends)
pmat tdg storage stats

# View historical scores (uses cached results for performance)
pmat tdg src/runtime/repl.rs  # Instant retrieval from storage

# Export quality trends
pmat tdg export . --format markdown --output quality-trends.md
```

**Storage Benefits**:
- **📈 Historical Tracking**: Every TDG score persistently stored
- **⚡ Performance**: Cache hits avoid re-analysis on unchanged files
- **🎯 Quality Trends**: Track improvements/regressions over time
- **💾 Tiered Storage**: Hot/warm/cold optimization for 100K+ files

### 🎯 Actionable Entropy Analysis (v2.70.0+)
**MANDATORY**: Replace noisy character entropy with AST-based actionable patterns.

```bash
# Analyze entropy for actionable refactoring opportunities
pmat analyze entropy . --min-severity high --top-violations 10

# Pattern types detected:
# - ErrorHandling: Repetitive error handling patterns
# - DataValidation: Duplicate validation logic
# - ResourceManagement: Repeated resource patterns
# - ConfigurationAccess: Multiple config accesses
# - StateManagement: Redundant state checks
# - AlgorithmicComplexity: Similar algorithm implementations

# Each violation includes:
# - Specific fix suggestion
# - LOC reduction estimate
# - Refactoring strategy
```

### 🌐 Real-time TDG Dashboard (v2.39.0+)
```bash
# Start interactive web dashboard for continuous monitoring
pmat tdg dashboard --port 8080 --open

# Features:
# - Real-time quality metrics with 5-second updates
# - Storage backend monitoring (Hot/Warm/Cold tiers)
# - Performance profiling with flame graphs
# - Bottleneck detection (CPU, I/O, Memory, Lock contention)
# - Server-Sent Events for live updates
```

### 🔌 MCP Server Integration (Enterprise Features)
```bash
# Start MCP server for external tool integration
pmat mcp serve --port 3000

# Available MCP tools:
# - tdg_analyze_with_storage: Analysis with persistent storage
# - tdg_system_diagnostics: System health monitoring
# - tdg_storage_management: Storage optimization
# - tdg_performance_profiling: Advanced profiling
# - tdg_alert_management: Configurable alerts
# - tdg_export_data: Multi-format export
# - analyze_entropy: Actionable pattern detection
```

### 🔒 Pre-commit Hooks Management (v2.66.0+)
```bash
# Install PMAT-managed pre-commit hooks
pmat tdg hooks install --backup

# Features:
# - Dynamic generation from pmat.toml
# - Single source of truth for thresholds
# - Automatic TDG scoring on commit
# - Entropy analysis integration
# - Zero configuration duplication
```

### 📋 Daily PMAT Workflow (MANDATORY)

**Before Starting Work**:
```bash
# 1. Check baseline quality with persistent storage
pmat tdg . --top-files 10  # Uses cached scores for speed

# 2. Start real-time dashboard
pmat tdg dashboard --port 8080 --open &

# 3. Analyze entropy patterns
pmat analyze entropy . --min-severity medium
```

**During Development**:
```bash
# 1. Monitor dashboard for real-time quality feedback
# 2. Check specific files after changes
pmat tdg src/modified_file.rs --include-components

# 3. Validate entropy patterns
pmat analyze entropy --file src/modified_file.rs
```

**Before Committing**:
```bash
# 1. Full TDG validation (uses persistent storage)
pmat tdg . --min-grade A- --fail-on-violation

# 2. Entropy check for new patterns
pmat analyze entropy --changed-files --min-severity high

# 3. Export quality report
pmat tdg export . --format json > commit-quality.json

# 4. Storage statistics (track quality trends)
pmat tdg storage stats
```

### 🏆 Quality Metrics Integration

**Ruchy-Specific Thresholds**:
- **TDG Grade**: Maintain A- (≥85 points) minimum
- **Complexity**: ≤10 cyclomatic (strict Toyota Way)
- **Entropy Violations**: ≤10 high-severity patterns
- **Storage Growth**: Monitor ~/.pmat/tdg-* weekly
- **Cache Hit Ratio**: >80% for unchanged files

**Key Rules**:
- **PMAT FIRST**: Always run PMAT quality gates before starting any task
- **COMPLEXITY BUDGET**: Every function must justify its complexity via PMAT metrics
- **NO BYPASS**: Quality gates cannot be bypassed or temporarily ignored
- No "cruft" at root of repo - always clean up temp files before committing
- If fixing documentation, always ensure a doctest exists
- Language compatibility tests are MANDATORY quality gates - never bypass
- ruchy-cli is deprecated, stop publishing it
- When increasing test coverage ensure proper mix of unit/doctests/property-tests/fuzz
- Always look at ../ruchy-book and ../rosetta-ruchy to ensure quality at sprint start
- Any time we fail more than once, add more testing - mandatory sign this code path needs more testing
- Check ../ruchyruchy for integration reports at beginning of each sprint
- all bugs MUST be solved with TDD.  we don't do manual "hacks".  We write the test, then prove it fixes.
- **PMAT VIOLATION RESPONSE**: "The quality gate might be too strict. Let me try bypassing for now since our changes are good" is NEVER TOLERATED. Use Five Whys and Toyota Way to fix root cause.
- we use cargo-llvm not tarpualin for coverage
- **PMAT MANDATORY**: Every commit MUST pass PMAT quality gates - no exceptions

## Documentation Standards

**Professional Documentation Requirements**:
- Use precise, factual language without hyperbole or marketing speak
- Avoid excessive exclamation marks and celebratory language
- State achievements and features objectively
- Focus on technical accuracy over promotional language
- Never create documentation files proactively unless explicitly requested
- Documentation should be maintainable and verifiable