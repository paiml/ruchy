# 95% Coverage Target for Ruchy 2.0 Release

**Specification Version:** 1.0.0
**Created:** 2026-01-09
**Status:** Draft
**PMAT Ticket:** QUALITY-095

## Executive Summary

This specification defines the testing strategy to achieve 95% line coverage for the Ruchy 2.0 release. The approach is grounded in peer-reviewed software testing research and uses Popperian falsification criteria to validate test effectiveness.

## Current State

| Metric | Value |
|--------|-------|
| Current Tests | 15,848 |
| Estimated Coverage | ~87% |
| Target Coverage | 95% |
| Coverage Gap | ~8% |

## Peer-Reviewed Research Foundation

### 1. Code Coverage Effectiveness

**Inozemtseva & Holmes (2014)** - "Coverage is Not Strongly Correlated with Test Suite Effectiveness"
- *Proceedings of ICSE 2014*
- Finding: Coverage alone insufficient; mutation testing provides stronger quality signal
- Application: Combine line coverage with mutation testing (cargo-mutants)

**Gopinath et al. (2014)** - "Code Coverage for Suite Evaluation by Developers"
- *Proceedings of ICSE 2014*
- Finding: 80-90% coverage shows diminishing returns; focus on critical paths
- Application: Prioritize high-risk modules (interpreter, type inference)

### 2. Test-Driven Development Effectiveness

**Causevic et al. (2011)** - "Factors Limiting Industrial Adoption of TDD"
- *Journal of Systems and Software*
- Finding: TDD reduces defect density by 40-90% in controlled studies
- Application: EXTREME TDD methodology with RED-GREEN-REFACTOR

**Rafique & Misic (2013)** - "The Effects of TDD on External Quality and Productivity"
- *Empirical Software Engineering Journal*
- Finding: TDD improves external quality with minimal productivity cost
- Application: Write tests before implementation for new coverage

### 3. Mutation Testing

**Jia & Harman (2011)** - "An Analysis and Survey of the Development of Mutation Testing"
- *IEEE Transactions on Software Engineering*
- Finding: Mutation score correlates with fault detection capability
- Application: Target 75%+ mutation score via cargo-mutants

### 4. Property-Based Testing

**Claessen & Hughes (2000)** - "QuickCheck: A Lightweight Tool for Random Testing"
- *Proceedings of ICFP 2000*
- Finding: Random property testing finds edge cases missed by unit tests
- Application: proptest for all public APIs

---

## Module-by-Module Testing Strategy

### Module 1: `runtime/replay.rs`

**Current State:** 0 tests / 1196 lines (0%)
**Target:** 60 tests / 95% coverage
**Risk Level:** Medium

#### Strategy A: Struct Construction Tests (25 tests)

| Test ID | Function/Struct | Falsification Criterion |
|---------|-----------------|------------------------|
| REPLAY-001 | `EventId::new()` | Returns unique monotonic IDs |
| REPLAY-002 | `SemVer::parse()` | Rejects invalid version strings |
| REPLAY-003 | `SemVer::cmp()` | Ordering matches semantic versioning spec |
| REPLAY-004 | `ReplSession::new()` | Initializes with empty event list |
| REPLAY-005 | `SessionMetadata` | All fields accessible after construction |
| REPLAY-006 | `Environment::default()` | Returns sensible defaults |
| REPLAY-007 | `ResourceLimits::default()` | Limits are non-zero |
| REPLAY-008 | `TimestampedEvent` | Timestamp is monotonically increasing |

#### Strategy B: Event Enum Coverage (15 tests)

| Test ID | Variant | Falsification Criterion |
|---------|---------|------------------------|
| REPLAY-009 | `Event::Input` | Captures user input verbatim |
| REPLAY-010 | `Event::Output` | Preserves output formatting |
| REPLAY-011 | `Event::Error` | Error type preserved through serialization |
| REPLAY-012 | `Event` serde | Roundtrip serialization is lossless |

#### Strategy C: Validation & Divergence (20 tests)

| Test ID | Function | Falsification Criterion |
|---------|----------|------------------------|
| REPLAY-013 | `ValidationResult::is_valid()` | True iff no divergences |
| REPLAY-014 | `Divergence::Output` | Detects output mismatches |
| REPLAY-015 | `Divergence::Timing` | Detects timing violations |
| REPLAY-016 | `ReplayResult::success()` | Only true when all events match |

---

### Module 2: `runtime/resource_eval.rs`

**Current State:** 4 tests / 393 lines (1%)
**Target:** 37 tests / 95% coverage
**Risk Level:** High (security-sensitive)

#### Strategy A: Sandbox Operations (15 tests)

| Test ID | Function | Falsification Criterion |
|---------|----------|------------------------|
| RESOURCE-001 | `Sandbox::new()` | Creates isolated execution context |
| RESOURCE-002 | `Sandbox::enforce_limits()` | Terminates on limit exceeded |
| RESOURCE-003 | `Sandbox::checkpoint()` | State is capturable |
| RESOURCE-004 | `Sandbox::restore()` | State restoration is exact |

#### Strategy B: CheckpointHandle (10 tests)

| Test ID | Function | Falsification Criterion |
|---------|----------|------------------------|
| RESOURCE-005 | `CheckpointHandle::capture()` | Captures complete state |
| RESOURCE-006 | `CheckpointHandle::restore()` | Restores to exact prior state |
| RESOURCE-007 | `CheckpointHandle::drop()` | Resources freed on drop |

#### Strategy C: ResourceLimits Boundaries (12 tests)

| Test ID | Function | Falsification Criterion |
|---------|----------|------------------------|
| RESOURCE-008 | `ResourceLimits::memory(0)` | Zero memory limit rejects all allocations |
| RESOURCE-009 | `ResourceLimits::memory(MAX)` | Max limit allows all allocations |
| RESOURCE-010 | `ResourceLimits::time(0)` | Zero time limit causes immediate timeout |

---

### Module 3: `runtime/async_runtime.rs`

**Current State:** 2 tests / 141 lines (1.4%)
**Target:** 20 tests / 95% coverage
**Risk Level:** High (concurrency)

#### Strategy A: AsyncRuntime Lifecycle (8 tests)

| Test ID | Function | Falsification Criterion |
|---------|----------|------------------------|
| ASYNC-001 | `AsyncRuntime::new()` | Creates runtime without panic |
| ASYNC-002 | `AsyncRuntime::spawn()` | Task executes to completion |
| ASYNC-003 | `AsyncRuntime::shutdown()` | All tasks terminate gracefully |
| ASYNC-004 | `AsyncRuntime::block_on()` | Blocks until future completes |

#### Strategy B: JoinHandle (6 tests)

| Test ID | Function | Falsification Criterion |
|---------|----------|------------------------|
| ASYNC-005 | `JoinHandle::await` | Returns task result |
| ASYNC-006 | `JoinHandle::abort()` | Cancels running task |
| ASYNC-007 | `JoinHandle::is_finished()` | Accurate completion status |

#### Strategy C: Error Propagation (6 tests)

| Test ID | Function | Falsification Criterion |
|---------|----------|------------------------|
| ASYNC-008 | Task panic | Panic captured, not propagated to runtime |
| ASYNC-009 | Task timeout | Timeout error returned |
| ASYNC-010 | Task cancellation | Cancellation error distinct from panic |

---

### Module 4: `runtime/object_helpers.rs`

**Current State:** 9 tests / 612 lines (1.5%)
**Target:** 65 tests / 95% coverage
**Risk Level:** Medium

#### Strategy A: Predicate Functions (20 tests)

| Test ID | Function | Falsification Criterion |
|---------|----------|------------------------|
| OBJ-001 | `is_mutable_object(ObjectMut)` | Returns true |
| OBJ-002 | `is_mutable_object(Object)` | Returns false |
| OBJ-003 | `is_mutable_object(Integer)` | Returns false |
| OBJ-004 | `is_object(Object)` | Returns true |
| OBJ-005 | `is_object(ObjectMut)` | Returns true |
| OBJ-006 | `is_object(Array)` | Returns false |

#### Strategy B: Field Operations (25 tests)

| Test ID | Function | Falsification Criterion |
|---------|----------|------------------------|
| OBJ-007 | `get_object_field(obj, "exists")` | Returns Some(value) |
| OBJ-008 | `get_object_field(obj, "missing")` | Returns None |
| OBJ-009 | `get_object_field(non_obj, _)` | Returns None |
| OBJ-010 | `set_object_field(mut_obj, k, v)` | Field updated |
| OBJ-011 | `set_object_field(immut_obj, _, _)` | Returns error |

#### Strategy C: Conversion Functions (20 tests)

| Test ID | Function | Falsification Criterion |
|---------|----------|------------------------|
| OBJ-012 | `to_mutable(Object)` | Returns ObjectMut with same fields |
| OBJ-013 | `to_mutable(ObjectMut)` | Returns clone |
| OBJ-014 | `to_immutable(ObjectMut)` | Returns Object with same fields |
| OBJ-015 | `new_mutable_object({})` | Creates empty ObjectMut |
| OBJ-016 | `new_immutable_object({})` | Creates empty Object |

---

### Module 5: `quality/scoring.rs`

**Current State:** 40 tests / 2483 lines (1.6%)
**Target:** 80 tests / 95% coverage
**Risk Level:** Medium

#### Strategy A: Score Functions (40 tests)

| Test ID | Function | Falsification Criterion |
|---------|----------|------------------------|
| SCORE-001 | `score_correctness(empty_ast)` | Returns 1.0 (perfect) |
| SCORE-002 | `score_correctness(type_error_ast)` | Returns < 0.5 |
| SCORE-003 | `score_performance(O(1)_ast)` | Returns > 0.9 |
| SCORE-004 | `score_performance(O(n^2)_ast)` | Returns < 0.7 |
| SCORE-005 | `score_maintainability(clean_ast)` | Returns > 0.8 |
| SCORE-006 | `score_maintainability(complex_ast)` | Returns < 0.5 |
| SCORE-007 | `score_safety(safe_ast)` | Returns 1.0 |
| SCORE-008 | `score_safety(unsafe_ast)` | Returns 0.0 |

#### Strategy B: Grade Calculations (15 tests)

| Test ID | Function | Falsification Criterion |
|---------|----------|------------------------|
| SCORE-009 | `Grade::from_score(0.95)` | Returns Grade::APlus |
| SCORE-010 | `Grade::from_score(0.90)` | Returns Grade::A |
| SCORE-011 | `Grade::from_score(0.85)` | Returns Grade::AMinus |
| SCORE-012 | `Grade::from_score(0.59)` | Returns Grade::F |
| SCORE-013 | `Grade::APlus > Grade::A` | Returns true |

#### Strategy C: ScoreEngine & Caching (25 tests)

| Test ID | Function | Falsification Criterion |
|---------|----------|------------------------|
| SCORE-014 | `ScoreEngine::new()` | Initializes with empty cache |
| SCORE-015 | `ScoreEngine::score(ast)` twice | Second call hits cache |
| SCORE-016 | `ScoreEngine::invalidate(file)` | Clears relevant cache entries |
| SCORE-017 | `CacheStats::hit_rate()` | Accurate after mixed access |

---

### Module 6: `runtime/interpreter.rs`

**Current State:** 164 tests / 9787 lines (1.7%)
**Target:** 230 tests / 95% coverage
**Risk Level:** Critical

#### Strategy A: Expression Evaluation (100 tests)

| Test ID | ExprKind | Falsification Criterion |
|---------|----------|------------------------|
| INTERP-001 | `Literal(Integer)` | Evaluates to Value::Integer |
| INTERP-002 | `Literal(Float)` | Evaluates to Value::Float |
| INTERP-003 | `Literal(String)` | Evaluates to Value::String |
| INTERP-004 | `Literal(Bool)` | Evaluates to Value::Bool |
| INTERP-005 | `BinaryOp(Add, Int, Int)` | Returns sum |
| INTERP-006 | `BinaryOp(Add, Str, Str)` | Returns concatenation |
| INTERP-007 | `BinaryOp(Div, _, 0)` | Returns DivisionByZero error |
| INTERP-008 | `UnaryOp(Neg, Int)` | Returns negated value |
| INTERP-009 | `UnaryOp(Not, Bool)` | Returns inverted bool |
| INTERP-010 | `Call(fn, args)` | Executes function with args |

#### Strategy B: Statement Execution (80 tests)

| Test ID | Statement | Falsification Criterion |
|---------|-----------|------------------------|
| INTERP-011 | `let x = 1` | Binds x in environment |
| INTERP-012 | `x = 2` (after let mut) | Updates binding |
| INTERP-013 | `x = 2` (immutable) | Returns error |
| INTERP-014 | `if true { a } else { b }` | Evaluates a |
| INTERP-015 | `if false { a } else { b }` | Evaluates b |
| INTERP-016 | `while cond { body }` | Loops until cond false |
| INTERP-017 | `for x in arr { body }` | Iterates all elements |
| INTERP-018 | `match val { pat => expr }` | Matches correct arm |

#### Strategy C: Error Handling (50 tests)

| Test ID | Error Type | Falsification Criterion |
|---------|------------|------------------------|
| INTERP-019 | `TypeError` | Includes type names |
| INTERP-020 | `NameError` | Includes undefined name |
| INTERP-021 | `DivisionByZero` | Distinct error type |
| INTERP-022 | `StackOverflow` | Triggered by deep recursion |
| INTERP-023 | `IndexOutOfBounds` | Includes index and length |

---

### Module 7: `frontend/ast.rs`

**Current State:** 65 tests / 3495 lines (1.9%)
**Target:** 60 tests / 95% coverage
**Risk Level:** Low

#### Strategy A: Span & Comment (15 tests)

| Test ID | Function | Falsification Criterion |
|---------|----------|------------------------|
| AST-001 | `Span::new(0, 10)` | start=0, end=10 |
| AST-002 | `Span::merge(a, b)` | Covers both spans |
| AST-003 | `Span::contains(offset)` | True iff within range |
| AST-004 | `Comment::line(text)` | Preserves text |
| AST-005 | `Comment::block(text)` | Preserves multiline |

#### Strategy B: Pattern Variants (25 tests)

| Test ID | Pattern | Falsification Criterion |
|---------|---------|------------------------|
| AST-006 | `Pattern::Wildcard` | Matches any value |
| AST-007 | `Pattern::Literal(42)` | Matches only 42 |
| AST-008 | `Pattern::Identifier("x")` | Binds x |
| AST-009 | `Pattern::Tuple([a, b])` | Destructures tuple |
| AST-010 | `Pattern::Struct{..}` | Matches struct fields |

#### Strategy C: Display Implementations (20 tests)

| Test ID | Type | Falsification Criterion |
|---------|------|------------------------|
| AST-011 | `BinaryOp::Add.to_string()` | Returns "+" |
| AST-012 | `BinaryOp::Sub.to_string()` | Returns "-" |
| AST-013 | `UnaryOp::Neg.to_string()` | Returns "-" |
| AST-014 | `UnaryOp::Not.to_string()` | Returns "!" |

---

### Module 8: `middleend/infer.rs`

**Current State:** 57 tests / 2673 lines (2.1%)
**Target:** 90 tests / 95% coverage
**Risk Level:** High

#### Strategy A: Type Inference (40 tests)

| Test ID | Expression | Falsification Criterion |
|---------|------------|------------------------|
| INFER-001 | `42` | Infers i64 |
| INFER-002 | `3.14` | Infers f64 |
| INFER-003 | `"hello"` | Infers String |
| INFER-004 | `true` | Infers bool |
| INFER-005 | `a + b` (both i64) | Infers i64 |
| INFER-006 | `a + b` (i64 + f64) | Infers f64 (coercion) |
| INFER-007 | `fn(x) { x }` | Infers generic T -> T |
| INFER-008 | `if c { a } else { b }` | Infers common type |

#### Strategy B: Unification (30 tests)

| Test ID | Unification | Falsification Criterion |
|---------|-------------|------------------------|
| INFER-009 | `unify(i64, i64)` | Succeeds |
| INFER-010 | `unify(i64, f64)` | Fails (no implicit coercion in unify) |
| INFER-011 | `unify(T, i64)` | T resolves to i64 |
| INFER-012 | `unify(Vec<T>, Vec<i64>)` | T resolves to i64 |
| INFER-013 | `unify(T, T)` | Succeeds trivially |

#### Strategy C: Error Messages (20 tests)

| Test ID | Error | Falsification Criterion |
|---------|-------|------------------------|
| INFER-014 | Type mismatch | Shows expected vs actual |
| INFER-015 | Undefined variable | Shows variable name |
| INFER-016 | Infinite type | Detected and reported |

---

### Module 9: `quality/formatter.rs`

**Current State:** 55 tests / 2485 lines (2.2%)
**Target:** 110 tests / 95% coverage
**Risk Level:** Low

#### Strategy A: Expression Formatting (50 tests)

| Test ID | Expression | Falsification Criterion |
|---------|------------|------------------------|
| FMT-001 | `42` | Formats as "42" |
| FMT-002 | `a + b` | Formats with spaces: "a + b" |
| FMT-003 | `a + b * c` | Respects precedence: "a + b * c" |
| FMT-004 | `(a + b) * c` | Preserves parens: "(a + b) * c" |
| FMT-005 | `fn() { x }` | Proper brace formatting |

#### Strategy B: Statement Formatting (40 tests)

| Test ID | Statement | Falsification Criterion |
|---------|-----------|------------------------|
| FMT-006 | `let x = 1` | Formats with spaces |
| FMT-007 | `struct S { x: i64 }` | Proper indentation |
| FMT-008 | `enum E { A, B }` | Variants on separate lines |
| FMT-009 | `impl T { fn f() {} }` | Nested indentation |

#### Strategy C: Roundtrip Tests (20 tests)

| Test ID | Property | Falsification Criterion |
|---------|----------|------------------------|
| FMT-010 | `parse(format(ast)) == ast` | AST preserved |
| FMT-011 | `format(format(code)) == format(code)` | Idempotent |
| FMT-012 | Comments preserved | Roundtrip keeps comments |

---

### Module 10: `quality/gates.rs`

**Current State:** 22 tests / 1033 lines (2.1%)
**Target:** 45 tests / 95% coverage
**Risk Level:** Medium

#### Strategy A: Gate Configuration (15 tests)

| Test ID | Function | Falsification Criterion |
|---------|----------|------------------------|
| GATE-001 | `QualityGateConfig::default()` | Uses sensible defaults |
| GATE-002 | `QualityGateConfig::strict()` | All thresholds at A- |
| GATE-003 | `QualityGateConfig::lenient()` | All thresholds at C |
| GATE-004 | Invalid config | Validation rejects |

#### Strategy B: Enforcer Logic (20 tests)

| Test ID | Function | Falsification Criterion |
|---------|----------|------------------------|
| GATE-005 | `enforce(A+, min=A-)` | Passes |
| GATE-006 | `enforce(B, min=A-)` | Fails |
| GATE-007 | `enforce(A-, min=A-)` | Passes (boundary) |
| GATE-008 | Multiple gates | All must pass |

#### Strategy C: Grade Ordering (10 tests)

| Test ID | Comparison | Falsification Criterion |
|---------|------------|------------------------|
| GATE-009 | `A+ > A` | True |
| GATE-010 | `A > A-` | True |
| GATE-011 | `A- > B+` | True |
| GATE-012 | `F < D` | True |
| GATE-013 | `A == A` | True |

---

## 100-Point Popperian Falsification Criteria

### Falsifiability Requirements

Each test must be **falsifiable** per Popper's criterion - it must be possible for the test to fail if the implementation is incorrect.

### Scoring Rubric (100 points)

| Category | Points | Criterion |
|----------|--------|-----------|
| **Determinism** | 20 | Test produces same result on every run |
| **Isolation** | 15 | Test does not depend on external state |
| **Specificity** | 15 | Test targets exactly one behavior |
| **Boundary Coverage** | 15 | Test covers edge cases and boundaries |
| **Error Paths** | 15 | Test verifies error conditions |
| **Documentation** | 10 | Test name describes expected behavior |
| **Performance** | 10 | Test completes in < 100ms |

### Falsification Checklist

For each test, verify:

- [ ] **F1**: If implementation returns wrong value, test fails
- [ ] **F2**: If implementation throws unexpected error, test fails
- [ ] **F3**: If implementation has off-by-one error, test fails
- [ ] **F4**: If implementation has null/nil handling bug, test fails
- [ ] **F5**: If implementation has type confusion, test fails

---

## PMAT Work Tracking

### Sprint Allocation

| Sprint | Modules | Est. Tests | Est. Hours |
|--------|---------|------------|------------|
| S1 | object_helpers, resource_eval | 102 | 8 |
| S2 | scoring, gates | 125 | 10 |
| S3 | replay, async_runtime | 80 | 6 |
| S4 | ast, formatter | 170 | 12 |
| S5 | infer | 90 | 8 |
| S6 | interpreter (part 1) | 115 | 10 |
| S7 | interpreter (part 2) | 115 | 10 |

### Quality Gates Per Sprint

Each sprint must pass:

1. `pmat tdg . --min-grade A- --fail-on-violation`
2. `cargo test --lib` - 100% pass rate
3. `cargo mutants --file <module>` - 75%+ mutation score
4. Coverage delta: +1% per sprint minimum

### PMAT Metrics Tracking

```yaml
# docs/execution/roadmap.yaml entry
QUALITY-095:
  title: "95% Coverage for 2.0 Release"
  status: in_progress
  sprints:
    - id: S1
      modules: [object_helpers, resource_eval]
      tests_added: 0
      coverage_delta: 0
      mutation_score: 0
    - id: S2
      modules: [scoring, gates]
      tests_added: 0
      coverage_delta: 0
      mutation_score: 0
    # ... etc
```

### TDG Score Targets

| Module | Current TDG | Target TDG |
|--------|-------------|------------|
| runtime/replay.rs | F (0%) | A- (85%) |
| runtime/resource_eval.rs | D (1%) | A- (85%) |
| runtime/object_helpers.rs | D (1.5%) | A- (85%) |
| quality/scoring.rs | D (1.6%) | A- (85%) |
| runtime/interpreter.rs | D (1.7%) | B+ (80%) |

---

## Implementation Timeline

### Phase 1: Quick Wins (Week 1)
- `object_helpers.rs`: 65 tests
- `resource_eval.rs`: 37 tests
- `async_runtime.rs`: 20 tests

### Phase 2: Quality Modules (Week 2)
- `scoring.rs`: 80 tests
- `gates.rs`: 45 tests
- `formatter.rs`: 110 tests

### Phase 3: Core Systems (Week 3-4)
- `replay.rs`: 60 tests
- `infer.rs`: 90 tests
- `interpreter.rs`: 230 tests
- `ast.rs`: 60 tests

### Total Estimated Tests: 797

---

## References

1. Inozemtseva, L., & Holmes, R. (2014). Coverage is not strongly correlated with test suite effectiveness. *ICSE '14*.

2. Gopinath, R., Jensen, C., & Groce, A. (2014). Code coverage for suite evaluation by developers. *ICSE '14*.

3. Causevic, A., Sundmark, D., & Punnekkat, S. (2011). Factors limiting industrial adoption of test driven development. *JSS*.

4. Rafique, Y., & Misic, V. B. (2013). The effects of test-driven development on external quality and productivity. *ESE Journal*.

5. Jia, Y., & Harman, M. (2011). An analysis and survey of the development of mutation testing. *IEEE TSE*.

6. Claessen, K., & Hughes, J. (2000). QuickCheck: A lightweight tool for random testing of Haskell programs. *ICFP '00*.

7. Popper, K. (1959). *The Logic of Scientific Discovery*. Hutchinson.

---

## Appendix A: Test Naming Convention

```
test_<MODULE>_<FUNCTION>_<SCENARIO>_r<ROUND>

Examples:
- test_object_helpers_is_mutable_object_with_object_mut_r162
- test_scoring_score_correctness_empty_ast_r162
- test_interpreter_eval_binary_add_integers_r162
```

## Appendix B: EXTREME TDD Protocol

For each module:

1. **RED**: Write failing tests first
2. **GREEN**: Implement minimum code to pass
3. **REFACTOR**: Improve code quality (TDG A-)
4. **VALIDATE**: Run mutation testing
5. **COMMIT**: Atomic commit with metrics

---

*Document maintained by: PMAT Quality Team*
*Last updated: 2026-01-09*
