# Property Testing Specification - Ruchy v3.66.5+

## Executive Summary

**Goal**: Achieve 80% property test coverage for P0 modules (Parser, Interpreter, Type Checker)
**Pattern**: Proven pmat/pforge Sprint 88 methodology
**Current**: 169 tests (52% coverage)
**Target**: 232 tests (80% coverage for P0 modules)
**Timeline**: 2-week sprint

## Motivation

From CLAUDE.md:
> Target 80% of all modules with property tests [...] Use proptest to verify invariants with 10,000+ random inputs

Property-based testing provides:
1. **Systematic edge case discovery**: Finds bugs manual testing misses
2. **Mathematical invariant verification**: Proves correctness properties
3. **Regression prevention**: Auto-generated test cases catch regressions
4. **Documentation**: Properties serve as executable specifications

## Current State Analysis

### Coverage Metrics (Baseline - v3.66.5)

```
Module                  Tests    Coverage    Status
====================================================
backend/transpiler       140        85%      ✅ Excellent
runtime/eval              13        30%      ⚠️ Needs work
runtime/transformation     4        60%      ⚠️ Needs work
runtime/builtins           3        40%      ⚠️ Needs work
frontend/parser            4        10%      🚨 Critical gap
====================================================
TOTAL                    169        52%      ⚠️ Below target
```

### Critical Gaps (P0 Modules)

1. **Frontend Parser** (10% coverage)
   - Missing: Token stream properties, AST invariants, error recovery
   - Impact: Parser bugs can corrupt entire compilation pipeline
   - Priority: **HIGHEST**

2. **Runtime Interpreter** (30% coverage)
   - Missing: Type system properties, evaluation semantics, error propagation
   - Impact: Runtime bugs cause incorrect program behavior
   - Priority: **HIGH**

3. **Type Checker** (0% coverage)
   - Missing: All type checking properties
   - Impact: Type errors not caught at compile time
   - Priority: **HIGH**

## Property Testing Principles

### 1. Invariant-Based Testing

**Definition**: Properties that must ALWAYS hold, regardless of input

**Example** (Parser):
```rust
#[quickcheck]
fn prop_parse_then_format_is_stable(expr: ValidExpr) -> bool {
    let parsed = parse(&expr.0);
    let formatted = format_ast(&parsed);
    let reparsed = parse(&formatted);

    // Invariant: parse(format(parse(x))) == parse(x)
    parsed == reparsed
}
```

### 2. Round-Trip Properties

**Definition**: Encoding then decoding returns original value

**Example** (Interpreter):
```rust
#[quickcheck]
fn prop_value_serialization_round_trip(value: Value) -> bool {
    let json = value.to_json();
    let decoded = Value::from_json(&json).unwrap();

    // Invariant: from_json(to_json(v)) == v
    value == decoded
}
```

### 3. Oracle-Based Properties

**Definition**: Compare against known-good reference implementation

**Example** (Evaluator):
```rust
#[quickcheck]
fn prop_arithmetic_matches_rust(a: i32, b: i32) -> bool {
    let ruchy_result = eval(&format!("{} + {}", a, b));
    let rust_result = a + b;

    // Oracle: Ruchy arithmetic == Rust arithmetic
    ruchy_result == Value::Integer(rust_result)
}
```

### 4. Error Resilience Properties

**Definition**: Invalid inputs should fail gracefully (no panics)

**Example** (Parser):
```rust
#[quickcheck]
fn prop_parser_never_panics(input: String) -> bool {
    // Property: Parser should never panic on any input
    std::panic::catch_unwind(|| {
        let _ = Parser::new(&input).parse();
    }).is_ok()
}
```

## Implementation Plan

### Sprint 1: Parser Properties (PROPTEST-003)

**Duration**: 5 days
**Goal**: 10% → 80% coverage (+26 tests)

#### Day 1-2: Expression Parsing (10 tests)

```rust
// tests/properties/parser/expressions.rs

#[quickcheck]
fn prop_literal_parsing_preserves_value(lit: Literal) -> bool {
    let code = format_literal(&lit);
    let parsed = parse_expr(&code);
    extract_literal(&parsed) == lit
}

#[quickcheck]
fn prop_binary_op_parsing_preserves_precedence(op1: BinOp, op2: BinOp) -> bool {
    let expr = format!("a {} b {} c", op1.symbol(), op2.symbol());
    let parsed = parse_expr(&expr);
    // Verify precedence rules maintained
    verify_precedence(&parsed, op1, op2)
}

#[quickcheck]
fn prop_nested_expressions_balance_correctly(depth: u8) -> TestResult {
    if depth > 10 { return TestResult::discard(); }

    let expr = generate_nested_expr(depth);
    let parsed = parse_expr(&expr);

    // Property: Nesting depth preserved
    TestResult::from_bool(measure_depth(&parsed) == depth)
}
```

**Properties to test**:
1. Literal parsing preserves values
2. Binary operators respect precedence
3. Unary operators bind correctly
4. Parentheses override precedence
5. Nested expressions balance
6. String literals handle escapes
7. Array literals preserve order
8. Tuple literals preserve arity
9. Range expressions parse correctly
10. Lambda expressions capture correctly

#### Day 3-4: Statement Parsing (10 tests)

```rust
// tests/properties/parser/statements.rs

#[quickcheck]
fn prop_let_binding_preserves_mutability(is_mut: bool, name: ValidIdent) -> bool {
    let code = if is_mut {
        format!("let mut {} = 42", name.0)
    } else {
        format!("let {} = 42", name.0)
    };

    let parsed = parse_stmt(&code);
    extract_mutability(&parsed) == is_mut
}

#[quickcheck]
fn prop_if_else_branches_parsed_correctly(
    cond: ValidExpr,
    then_block: Vec<ValidStmt>,
    else_block: Vec<ValidStmt>
) -> bool {
    let code = format_if_else(&cond, &then_block, &else_block);
    let parsed = parse_stmt(&code);

    verify_if_structure(&parsed, &then_block, &else_block)
}
```

**Properties to test**:
1. Let bindings preserve mutability
2. Function definitions preserve parameters
3. If/else branches parse correctly
4. For loops preserve iteration variable
5. While loops preserve condition
6. Match arms preserve exhaustiveness
7. Return statements preserve value
8. Break/continue preserve labels
9. Struct definitions preserve fields
10. Impl blocks preserve methods

#### Day 5: Error Recovery & Token Stream (6 tests)

```rust
// tests/properties/parser/error_recovery.rs

#[quickcheck]
fn prop_parser_recovers_from_single_error(valid_code: Vec<ValidStmt>) -> bool {
    let broken_code = introduce_single_error(&valid_code);
    let result = parse_with_recovery(&broken_code);

    // Property: Should recover and parse remaining valid statements
    result.errors.len() == 1 && result.valid_stmts.len() > 0
}

#[quickcheck]
fn prop_token_stream_round_trip(code: String) -> bool {
    let tokens = tokenize(&code);
    let reconstructed = tokens_to_string(&tokens);
    let retokenized = tokenize(&reconstructed);

    // Property: Tokenization is stable
    tokens == retokenized
}
```

**Properties to test**:
1. Parser recovers from single syntax errors
2. Token stream can be reconstructed
3. EOF handling is consistent
4. Comment preservation works
5. Whitespace handling is correct
6. Invalid UTF-8 fails gracefully

### Sprint 2: Interpreter Properties (PROPTEST-004)

**Duration**: 5 days
**Goal**: 30% → 80% coverage (+37 tests)

#### Day 1-2: Value Type Properties (15 tests)

```rust
// tests/properties/interpreter/values.rs

#[quickcheck]
fn prop_value_type_name_consistency(value: Value) -> bool {
    let type_name = value.type_name();

    // Property: type_name() matches actual variant
    match value {
        Value::Integer(_) => type_name == "Integer",
        Value::Float(_) => type_name == "Float",
        Value::String(_) => type_name == "String",
        // ... etc
    }
}

#[quickcheck]
fn prop_value_equality_is_reflexive(value: Value) -> bool {
    // Property: v == v (reflexivity)
    value == value.clone()
}

#[quickcheck]
fn prop_value_equality_is_symmetric(v1: Value, v2: Value) -> bool {
    // Property: if v1 == v2, then v2 == v1 (symmetry)
    (v1 == v2) == (v2 == v1)
}

#[quickcheck]
fn prop_integer_arithmetic_no_overflow_panic(a: i32, b: i32) -> bool {
    // Property: Arithmetic should handle overflow gracefully
    std::panic::catch_unwind(|| {
        let _ = eval(&format!("{} + {}", a, b));
        let _ = eval(&format!("{} * {}", a, b));
    }).is_ok()
}
```

**Properties to test**:
1. Type names are consistent
2. Equality is reflexive
3. Equality is symmetric
4. Equality is transitive
5. Integer arithmetic handles overflow
6. Float arithmetic handles NaN/Inf
7. String concatenation preserves length
8. Array indexing bounds checked
9. Nil propagates correctly
10. Boolean logic follows truth tables
11. Type conversions are consistent
12. Value cloning is deep
13. Value hashing is consistent
14. Value ordering is total
15. Value display is invertible

#### Day 3-4: Evaluation Semantics (15 tests)

```rust
// tests/properties/interpreter/evaluation.rs

#[quickcheck]
fn prop_variable_binding_preserves_value(name: ValidIdent, value: Value) -> bool {
    let code = format!("let {} = {:?}; {}", name.0, value, name.0);
    let result = eval(&code);

    // Property: Binding then retrieving returns same value
    result == value
}

#[quickcheck]
fn prop_function_call_pure(
    params: Vec<ValidIdent>,
    body: ValidExpr,
    args: Vec<Value>
) -> TestResult {
    if params.len() != args.len() { return TestResult::discard(); }

    let code = format_function(&params, &body, &args);

    let result1 = eval(&code);
    let result2 = eval(&code);

    // Property: Pure functions are deterministic
    TestResult::from_bool(result1 == result2)
}

#[quickcheck]
fn prop_if_expression_returns_correct_branch(
    cond: bool,
    then_val: Value,
    else_val: Value
) -> bool {
    let code = format!(
        "if {} {{ {:?} }} else {{ {:?} }}",
        cond, then_val, else_val
    );
    let result = eval(&code);

    // Property: If selects correct branch
    result == if cond { then_val } else { else_val }
}
```

**Properties to test**:
1. Variable binding preserves values
2. Function calls are pure (when applicable)
3. If expressions select correct branch
4. Match expressions are exhaustive
5. For loops execute correct iterations
6. While loops terminate on false condition
7. Array operations preserve element count
8. Struct field access is consistent
9. Method calls preserve receiver
10. Closure capture is correct
11. Recursion terminates (depth-limited)
12. Lazy evaluation defers correctly
13. Short-circuit evaluation works
14. Pattern matching is sound
15. Error values propagate

#### Day 5: Error Handling Properties (7 tests)

```rust
// tests/properties/interpreter/errors.rs

#[quickcheck]
fn prop_undefined_variable_errors_consistently(name: ValidIdent) -> bool {
    let code = format!("{}", name.0);
    let result = eval(&code);

    // Property: Undefined variables always produce errors
    matches!(result, Err(InterpreterError::UndefinedVariable(_)))
}

#[quickcheck]
fn prop_type_errors_dont_panic(op: BinOp, v1: Value, v2: Value) -> bool {
    let code = format!("{:?} {} {:?}", v1, op.symbol(), v2);

    // Property: Type errors caught gracefully (no panic)
    std::panic::catch_unwind(|| {
        let _ = eval(&code);
    }).is_ok()
}

#[quickcheck]
fn prop_error_messages_are_descriptive(error: InterpreterError) -> bool {
    let msg = error.to_string();

    // Property: Error messages are non-empty and descriptive
    msg.len() > 10 && !msg.contains("error")
}
```

**Properties to test**:
1. Undefined variables error consistently
2. Type errors don't panic
3. Division by zero errors gracefully
4. Array out of bounds errors
5. Invalid method calls error
6. Error messages are descriptive
7. Error stack traces are complete

### Sprint 3: Type Checker Properties (PROPTEST-005)

**Duration**: 3 days (if type checker exists)
**Goal**: 0% → 80% coverage

**Note**: Type checker implementation status to be verified. If not implemented, defer to future sprint.

## Success Metrics

### Quantitative Goals

| Metric | Baseline | Target | Measurement |
|--------|----------|--------|-------------|
| Total property tests | 169 | 232 | `cargo test --lib property \| grep "passed"` |
| Parser coverage | 10% | 80% | Property tests / total functions |
| Interpreter coverage | 30% | 80% | Property tests / total functions |
| P0 module coverage | 52% | 80% | Weighted average |
| Test runtime | 0.07s | <1.0s | `cargo test --lib property` time |
| Zero regressions | ✅ | ✅ | All existing tests pass |

### Qualitative Goals

1. **Invariant Documentation**: Each property test includes comments explaining the invariant
2. **Edge Case Discovery**: Find and fix at least 3 bugs via property testing
3. **Regression Prevention**: All discovered bugs have property tests preventing recurrence
4. **Code Examples**: Property tests serve as executable documentation

## Testing Tools & Configuration

### Proptest Configuration

```rust
// proptest-regressions/ for reproducible failures
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10000,  // 10,000+ inputs per test (CLAUDE.md requirement)
        max_shrink_iters: 1000,
        timeout: 5000,  // 5s per test case
        .. ProptestConfig::default()
    })]

    #[test]
    fn my_property_test(input: ValidInput) {
        // Test implementation
    }
}
```

### Custom Generators

```rust
// Generate valid Ruchy expressions
impl Arbitrary for ValidExpr {
    fn arbitrary(g: &mut Gen) -> Self {
        // Generate syntactically valid expressions
        // Avoid infinite recursion (max depth 10)
        // Cover all expression variants
    }
}

// Generate valid identifiers
impl Arbitrary for ValidIdent {
    fn arbitrary(g: &mut Gen) -> Self {
        // Start with letter/underscore
        // Continue with alphanumeric/underscore
        // Exclude reserved keywords
    }
}
```

## Integration with CI/CD

### Pre-commit Hooks

```bash
# .git/hooks/pre-commit
cargo test --lib property || {
    echo "❌ Property tests failed - commit blocked"
    exit 1
}
```

### GitHub Actions

```yaml
# .github/workflows/property-tests.yml
name: Property Tests
on: [push, pull_request]
jobs:
  proptest:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run property tests
        run: cargo test --lib property --release
      - name: Upload regression files
        uses: actions/upload-artifact@v2
        with:
          name: proptest-regressions
          path: proptest-regressions/
```

## Maintenance & Evolution

### Regression File Management

1. **Commit Regression Files**: Always commit `proptest-regressions/` directory
2. **Review Failures**: Investigate new regression files in CI
3. **Document Known Issues**: Track expected failures in issue tracker

### Property Test Review Checklist

- [ ] Property is clearly documented
- [ ] Invariant is mathematically sound
- [ ] Generator produces valid inputs
- [ ] Test runs in <1s
- [ ] Shrinking produces minimal counterexamples
- [ ] Regression files committed

## References

- **CLAUDE.md**: Testing requirements and pmat/pforge pattern
- **Proptest Book**: https://altsysrq.github.io/proptest-book/intro.html
- **QuickCheck Paper**: "QuickCheck: A Lightweight Tool for Random Testing of Haskell Programs"
- **Hypothesis**: Python property testing inspiration
- **pmat Sprint 88**: Proven 80% coverage success pattern

---

**Document Status**: ✅ Complete
**Last Updated**: 2025-10-03
**Owner**: Ruchy Compiler Team
**Review Cycle**: Sprint retrospective
