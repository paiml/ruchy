# CLAUDE.md - Ruchy Compiler Implementation Protocol

## IMPORTANT: Auto-Generated Files
**NEVER EDIT `deep_context.md`** - This file is auto-generated and will be overwritten. Any changes should be made to the source files instead.

## Prime Directive

**Generate correct code that compiles on first attempt. Quality is built-in, not bolted-on.**

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

### Testing Hierarchy (Idiot-Proof Prevention):
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

### Mandatory Testing Requirements for Quality Issues

**CRITICAL**: Any regression or quality problem REQUIRES this diverse testing approach:

1. **Doctests**: Every public function MUST have runnable documentation examples
2. **Property Tests**: Use proptest to verify invariants with 10,000+ random inputs
3. **Fuzz Tests**: Use cargo-fuzz or AFL to find edge cases with millions of inputs
4. **Examples**: Create working examples in examples/ directory demonstrating correct usage
5. **Integration Tests**: End-to-end scenarios covering real-world usage patterns
6. **Regression Tests**: Specific test case that reproduces and prevents the exact defect

**Code Coverage Requirements** (QUALITY-008 Implemented):
- **Current Baseline**: 33.34% overall (post-QUALITY-007 parser enhancements)
- **Regression Prevention**: Pre-commit hooks BLOCK commits below baseline
- **Direction**: Coverage must increase or stay same, NEVER decrease
- **Parser Improvements**: Character literals, tuple destructuring, rest patterns now working
- **Pattern Test Results**: 2 passing → 4 passing (100% improvement achieved)
- **Enforcement**: Automated coverage checking with clear error messages

**Complexity Requirements** (PMAT Enforced):
- **Maximum Cyclomatic Complexity**: 10 per function
- **Maximum Cognitive Complexity**: 15 per function  
- **Enforcement**: Pre-commit hook blocks commits exceeding limits
- **Refactoring**: Functions exceeding limits MUST be decomposed immediately

## Toyota Way Success Stories

### Property Testing Victory (2024-12)
- **545 systematic test cases**: 0 parser inconsistencies found
- **ROOT CAUSE**: Manual testing methodology error, NOT code defect
- **LESSON**: Property testing is objective - mathematically proves system behavior

### Language Completeness Achievement v1.9.1 (2025-08)
**BREAKTHROUGH**: Discovered that many "failing" features actually work perfectly!

**Systematic Testing Revealed**:
- ✅ **Fat arrow syntax**: Works perfectly (`x => x + 1`)
- ✅ **String interpolation**: Works perfectly (`f"Hello {name}"`)
- ✅ **Async/await**: Works perfectly (async fn and await expressions)
- ✅ **DataFrame literals**: Works perfectly (`df![]` macro)
- ✅ **Generics**: Works perfectly (`Vec<T>`, `Option<T>`)
- ✅ **Pipeline Operator**: `|>` for functional programming (v1.9.0)
- ✅ **Import/Export**: Module system evaluation (v1.9.1)
- ✅ **String Methods**: Complete suite (v1.8.9)

**ROOT CAUSE**: Manual testing created false negatives. Features were working all along!

### Quality Excellence Sprint v1.6.0 - COMPLETED
**RESULTS**: 107 comprehensive tests created, 287 tests passing, 80% coverage achieved
- LSP MODULE COVERAGE: 0% → 96-100% 
- MCP MODULE COVERAGE: 0% → 33%
- TYPE INFERENCE COVERAGE: 0% → 15%

### Complete Language Restoration - FINAL STATUS
**✅ ALL CORE FUNCTIONALITY RESTORED**:
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

## Task Execution Protocol

### MANDATORY: Roadmap and Ticket Tracking

**CRITICAL**: ALL development work MUST follow roadmap-driven development:

1. **ALWAYS Use Ticket Numbers**: Every commit, PR, and task MUST reference a ticket ID from docs/execution/roadmap.md
2. **Roadmap-First Development**: No work begins without a corresponding roadmap entry
3. **Ticket Format**: Use format "RUCHY-XXXX" or established ticket naming (e.g., QUALITY-001, USABLE-001)
4. **Traceability**: Every change must be traceable back to business requirements via ticket system
5. **Sprint Planning**: Work is organized by sprint with clear task dependencies and priorities

### Pre-Implementation Verification
```rust
// HALT. Before implementing ANY feature:
□ Check ../ruchy-book/INTEGRATION.md for latest compatibility report
□ Check ../ruchy-book/docs/bugs/ruchy-runtime-bugs.md for known issues
□ Locate specification section in SPECIFICATION.md
□ Find task ID in docs/execution/roadmap.md (MANDATORY)
□ Verify ticket dependencies completed via DAG
□ Reference ticket number in all commits/PRs
□ Check existing patterns in codebase
□ Confirm complexity budget (<10 cognitive)
```

### Commit Message Format (MANDATORY)
```
[TICKET-ID] Brief description

Detailed explanation of changes
- Specific improvements made
- Test coverage added
- Performance impact
- Breaking changes (if any)

Closes: TICKET-ID
```

**Example**:
```
[QUALITY-001] Refactor parse_prefix complexity from 161 to 8

Extracted 8 helper functions following single responsibility principle:
- parse_literal_prefix (complexity: 5)
- parse_string_prefix (complexity: 4)
- parse_identifier_prefix (complexity: 7)

Added comprehensive doctests and property tests for all helpers.
Performance trade-off: ~100ms -> ~200ms acceptable for maintainability.

Closes: QUALITY-001
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

echo "🔒 MANDATORY Quality Gates..."

# GATE 1: Basic functionality (FATAL if fails)
echo 'println("Hello")' | timeout 5s ruchy repl | grep -q "Hello" || {
    echo "❌ FATAL: Can't even print 'Hello' in REPL"
    exit 1
}

# GATE 2: Language Feature Compatibility (CRITICAL)
cargo test test_one_liners --test compatibility_suite --quiet || {
    echo "❌ FATAL: One-liner compatibility regression detected"
    exit 1
}

# GATE 3: Complexity enforcement
pmat agent analyze --max-complexity 10 --auto-fix || {
    echo "❌ BLOCKED: Complexity exceeds 10"
    exit 1
}

# GATE 4: Zero SATD policy
! grep -r "TODO\|FIXME\|HACK" src/ --include="*.rs" || {
    echo "❌ BLOCKED: SATD comments found"
    exit 1
}

# GATE 5: Lint zero tolerance
cargo clippy --all-targets --all-features -- -D warnings || {
    echo "❌ BLOCKED: Lint warnings found" 
    exit 1
}

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

**Current Standards (v1.0.0 - PERFECT COMPATIBILITY ACHIEVED!)**:
- ✅ **One-liners**: 100% (15/15) - BASELINE - never regress
- ✅ **Basic Language Features**: 100% (5/5) - Core syntax complete  
- ✅ **Control Flow**: 100% (5/5) - if/match/for/while/pattern-guards
- ✅ **Data Structures**: 100% (7/7) - Objects fully functional
- ✅ **String Operations**: 100% (5/5) - All string methods working
- ✅ **Numeric Operations**: 100% (4/4) - Integer.to_string() + all math ops
- ✅ **Advanced Features**: 100% (4/4) - Pattern guards COMPLETED!

**🏆 ULTIMATE ACHIEVEMENT: 100% PERFECT LANGUAGE COMPATIBILITY! 🏆
🎯 TOTAL: 41/41 FEATURES WORKING - NO DEFECTS LEFT BEHIND**

### Test Organization (Industry Standard)
```
tests/
├── compatibility_suite.rs      # Main feature compatibility (100% required)
├── properties/                 # Property-based testing (Haskell style)
├── regression/                 # Bug prevention (every GitHub issue)
└── benchmarks/                # Performance baselines (SQLite style)
```

Language compatibility testing is **GATE 2** in our mandatory pre-commit hooks - more critical than complexity or linting because **language regressions break user code**.

## The Development Flow

```
1. LOCATE specification section
2. IDENTIFY task in execution roadmap
3. VERIFY dependencies complete
4. IMPLEMENT with <10 complexity
5. VALIDATE performance invariants
6. COMMIT with task reference
```

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

**Key Rules**:
- No "cruft" at root of repo - always clean up temp files before committing
- If fixing documentation, always ensure a doctest exists
- Language compatibility tests are MANDATORY quality gates - never bypass
- ruchy-cli is deprecated, stop publishing it
- When increasing test coverage ensure proper mix of unit/doctests/property-tests/fuzz
- Always look at ../ruchy-book and ../rosetta-ruchy to ensure quality at sprint start
- Any time we fail more than once, add more testing - mandatory sign this code path needs more testing
- Check ../ruchyruchy for integration reports at beginning of each sprint
- all bugs MUST be solved with TDD.  we don't do manual "hacks".  We write the test, then prove it fixes.
- This type of behavior is never tolerated. "The quality gate might be too strict. Let me try bypassing for now since our changes are good".  This means you need to practice Five Whys and the Toyota Way and fix root cause.