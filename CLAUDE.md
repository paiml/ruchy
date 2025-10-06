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

## EXTREME TDD Protocol (CRITICAL RESPONSE TO ANY BUG)

**ANY BUG IN ANY COMPONENT REQUIRES IMMEDIATE EXTREME TDD RESPONSE:**

### Critical Bug Response (MANDATORY - ZERO EXCEPTIONS):

**🛑 STOP THE LINE PROTOCOL:**

1. **HALT ALL OTHER WORK**: Stop everything when ANY bug found (parser, transpiler, runtime, linter, tooling, etc.)
2. **ROOT CAUSE ANALYSIS**: Use GENCHI GENBUTSU (go and see) to understand exact failure mode
3. **EXTREME TDD FIX** (RED→GREEN→REFACTOR):
   - **RED**: Write failing test that reproduces bug FIRST
   - **GREEN**: Fix bug with minimal code changes
   - **REFACTOR**: Apply PMAT quality gates (A- minimum, ≤10 complexity)
4. **EXTREME TEST COVERAGE**: Create comprehensive test suites immediately:
   - Unit tests for specific bug scenario
   - Integration tests for complete programs
   - Property tests with random inputs (10,000+ iterations)
   - Mutation tests (≥75% mutation coverage via cargo-mutants)
   - Fuzz tests for edge cases
   - Doctests in every public function
   - `cargo run --examples` MUST pass 100%
5. **REGRESSION PREVENTION**: Add failing test BEFORE fixing bug (TDD mandatory)
6. **PMAT QUALITY GATES**: ALL fixes MUST pass:
   - `pmat tdg <file> --min-grade A-` (≥85 points)
   - `pmat analyze complexity --max-cyclomatic 10 --max-cognitive 10`
   - `pmat analyze satd --fail-on-violation` (zero SATD)
7. **MUTATION VALIDATION**: Run mutation tests on fixed code:
   - `cargo mutants --file <fixed-file.rs> --timeout 300`
   - Target: ≥75% mutation coverage (CAUGHT/(CAUGHT+MISSED) ≥ 75%)
   - If <75%: Add more property tests and re-run
8. **COMPREHENSIVE VALIDATION**: Test all related features after fix

### Test Coverage Requirements (MANDATORY):
- **Parser Tests**: Every token, every grammar rule, every edge case
- **Transpiler Tests**: Every Ruchy construct → Rust construct mapping
- **Runtime Tests**: Every evaluation path, every error condition
- **Linter Tests**: Every lint rule, every scope scenario, every AST pattern
- **Tooling Tests**: Every CLI command, every flag combination, every output format
- **Integration Tests**: Full compile → execute → validate pipeline
- **Property Tests**: Automated generation of valid/invalid programs (10K+ cases)
- **Fuzz Tests**: Random input stress testing (AFL, cargo-fuzz)
- **Mutation Tests**: 75%+ mutation coverage via cargo-mutants (empirical validation)
- **Examples Tests**: All examples/ must compile and run

### Bug Categories (ALL Subject to EXTREME TDD):
- **Parser Bugs**: Grammar issues, tokenization errors, AST construction failures
- **Transpiler Bugs**: Incorrect Rust code generation, type mismatches, codegen errors
- **Runtime Bugs**: Evaluation errors, type system bugs, memory issues
- **Linter Bugs**: False positives, false negatives, scope tracking errors
- **Tooling Bugs**: CLI failures, invalid output, incorrect behavior
- **Quality Bugs**: PMAT violations, complexity explosions, technical debt accumulation

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

## QDD (Quality-Driven Development)

**Metrics**: Complexity ≤10, Coverage ≥80%, SATD=0, TDG A- minimum

**Setup**: `pmat quality-gates init; pmat hooks install`
**During**: `pmat tdg dashboard --port 8080 &`
**Commit**: Auto-validated via pre-commit hooks

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

## PMAT Quality Gates (v2.70+)

**Setup**: `pmat quality-gates init; pmat hooks install`
**Gates**: TDG A-, Complexity ≤10, SATD=0, Coverage ≥80%, Build clean
**Health**: `pmat maintain health` (~10s)

## PMAT TDG Quality Enforcement

**Standards**: A- (≥85), Complexity ≤10, SATD=0, Duplication <10%, Docs >70%

**Before**: `pmat tdg . --min-grade A- --fail-on-violation`
**During**: `pmat tdg <file> --include-components`
**Commit**: Blocked by pre-commit hooks if <A-

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

### Commit Message Format
```
[TICKET-ID] Brief description

- Specific changes
- Test coverage
- TDG Score: src/file.rs: 68.2→82.5 (C+→B+)

Closes: TICKET-ID
```

### End-of-Sprint Git Commit Protocol (MANDATORY)

**CRITICAL**: After EVERY sprint completion, you MUST commit all changes immediately.

**Sprint Completion Checklist**:
1. ✅ All sprint tasks complete and verified
2. ✅ All tests passing (unit, integration, property, mutation where applicable)
3. ✅ Roadmap updated with sprint completion status
4. ✅ Documentation updated (examples, tests, validation results)
5. ✅ **GIT COMMIT IMMEDIATELY** - Don't wait, commit now!

**Commit Protocol**:
```bash
# After sprint completion, ALWAYS run:
git add .
git status  # Verify changes
git commit -m "[SPRINT-ID] Sprint completion: <brief summary>

- All tasks complete: <list ticket IDs>
- Tests: X/X passing
- Examples: X files created
- Validation: X-tool protocol verified
- Roadmap: Updated with completion status

🤖 Generated with Claude Code
Co-Authored-By: Claude <noreply@anthropic.com>"

git status  # Verify commit success
```

**Why This Matters (Toyota Way)**:
- **Jidoka**: Each sprint is a complete unit of work - commit it atomically
- **Genchi Genbutsu**: Working code on disk = empirical evidence of progress
- **Kaizen**: Small, verified increments prevent integration hell
- **Risk Mitigation**: Never lose completed work due to session interruption

## MANDATORY: TDG Transactional Tracking

**Scoring**: A+ (95-100), A (90-94), A- (85-89), B (80-84), C (70-79), D (60-69), F (<60)

**Pre-Commit**: `pmat tdg . --min-grade A- --fail-on-violation` (BLOCKING)

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

### Pre-commit Hooks (AUTO-INSTALLED via `pmat hooks install`)
Gates: TDG A-, Function complexity ≤10, Basic REPL test
Install: `pmat hooks install`

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

## 15 Native Tool Validation Protocol (LANG-COMP MANDATORY)

**CRITICAL**: All language completeness documentation (LANG-COMP tickets) MUST validate examples using ALL 15 native Ruchy tools.

### 15-Tool Validation Requirements (MANDATORY/BLOCKING)

Every LANG-COMP example MUST pass: `lint`, `compile`, `run`, `test`, `coverage`, `bench`, `doc`, `fmt`, `check`, `big-o`, `ast`, `wasm`, `provability`, `mutation`, `fuzz`

See: docs/SPECIFICATION.md Section 31

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

### TDG Violation Response (IMMEDIATE):
1. **HALT**: Stop when TDG < A- (85 points)
2. **ANALYZE**: `pmat tdg <file> --include-components`
3. **REFACTOR**: Fix specific component issues
4. **VERIFY**: Re-run to prove A- achievement

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


## PMAT v2.68.0+ Advanced Features

### Daily Workflow
**Before Work**: `pmat tdg . --top-files 10; pmat tdg dashboard --port 8080 --open &`
**During**: Monitor dashboard, check files: `pmat tdg <file> --include-components`
**Before Commit**: `pmat tdg . --min-grade A- --fail-on-violation`

### Key Rules
- **PMAT FIRST**: Run quality gates before ANY task
- **NO BYPASS**: Never `--no-verify`, fix root cause via Five Whys
- **TDD MANDATORY**: Write test first, prove fix works
- Use cargo-llvm-cov (not tarpaulin)
- All bugs solved with TDD, never manual hacks
- ruchy-cli deprecated (don't publish)
- Mix: unit/doctests/property-tests/fuzz tests
- Check ../ruchy-book and ../rosetta-ruchy at sprint start

## Documentation Standards

**Professional Documentation Requirements**:
- Use precise, factual language without hyperbole or marketing speak
- Avoid excessive exclamation marks and celebratory language
- State achievements and features objectively
- Focus on technical accuracy over promotional language
- Never create documentation files proactively unless explicitly requested
- Documentation should be maintainable and verifiable