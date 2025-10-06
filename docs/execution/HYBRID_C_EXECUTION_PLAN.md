# Hybrid C: User Value First - EXTREME TDD Execution Plan

**Date**: 2025-10-06
**Approach**: Maximum user value with EXTREME TDD, mutation testing, and property testing
**Duration**: 14-20 hours (estimated)
**Quality Standard**: TDG A-, 100% mutation coverage, property tests for all new functions

---

## Phase 1: Book Compatibility Gaps (8-12 hours)

### TICKET HYBRID-C-1: String Methods Implementation (2-3 hours)

**Goal**: Implement missing string methods discovered in compatibility testing

**Missing Methods**:
- `to_uppercase()` - Convert string to uppercase
- `to_lowercase()` - Convert string to lowercase  
- Fix `split()` output formatting (currently returns `[a, b, c]` instead of `["a", "b", "c"]`)

**EXTREME TDD Protocol**:
1. ✅ **Write failing test FIRST** (for each method)
2. ✅ **Property tests**: Random string inputs (10K cases per method)
3. ✅ **Mutation test**: Verify all code paths caught
4. ✅ **Doctests**: Every public function has runnable example
5. ✅ **Integration test**: End-to-end REPL validation

**Test Requirements**:
```rust
#[cfg(test)]
mod string_method_tests {
    // Unit tests
    #[test] fn test_to_uppercase() { ... }
    #[test] fn test_to_lowercase() { ... }
    #[test] fn test_split_formatting() { ... }
    
    // Property tests (10K cases each)
    proptest! {
        #[test]
        fn uppercase_idempotent(s: String) {
            let upper = to_upper(&s);
            assert_eq!(upper, to_upper(&upper));
        }
        
        #[test]
        fn lowercase_preserves_length(s: String) {
            assert_eq!(s.len(), to_lower(&s).len());
        }
    }
    
    // Doctests in function docs
    /// Converts string to uppercase
    /// 
    /// # Examples
    /// ```
    /// assert_eq!("HELLO", to_uppercase("hello"));
    /// ```
}
```

**Mutation Coverage Target**: 100% (all mutants caught)

**Files**:
- `src/runtime/eval_string_methods.rs` - Implementation
- `tests/string_methods_test.rs` - Comprehensive tests

---

### TICKET HYBRID-C-2: Try-Catch Parser (3-4 hours)

**Goal**: Complete try-catch syntax support for error handling (Chapter 17)

**Current Issue**: Parser incomplete
```
try { 10 / 0 } catch e { "error" }
→ Error: Expected RightBrace, found Let
```

**Required Grammar**:
```
TryExpr ::= "try" Block "catch" Identifier Block
```

**EXTREME TDD Protocol**:
1. ✅ **Failing test**: try-catch parsing
2. ✅ **Property test**: Random valid try-catch structures (10K cases)
3. ✅ **Mutation test**: Parser paths verified
4. ✅ **Integration test**: Runtime error handling works
5. ✅ **Book example**: Chapter 17 examples pass

**Test Requirements**:
```rust
#[test]
fn test_try_catch_parsing() {
    let code = "try { risky() } catch e { handle(e) }";
    let ast = parse(code).unwrap();
    assert!(matches!(ast, Expr::TryCatch { .. }));
}

#[test] 
fn test_try_catch_execution() {
    let code = "try { 10 / 0 } catch e { 42 }";
    assert_eq!(eval(code), Value::Int(42));
}

proptest! {
    #[test]
    fn try_catch_never_panics(body: String, handler: String) {
        let code = format!("try {{ {} }} catch e {{ {} }}", body, handler);
        let _ = parse(&code); // Should not panic
    }
}
```

**Mutation Coverage Target**: 100%

**Files**:
- `src/frontend/parser/expressions.rs` - Parse try-catch
- `src/runtime/eval_try_catch.rs` - Execute try-catch (already exists)
- `tests/try_catch_test.rs` - Comprehensive tests

---

### TICKET HYBRID-C-3: Output Formatting (2-3 hours)

**Goal**: Standardize output formatting for consistency

**Issues**:
1. `to_string()`: Returns `42` instead of `"42"`
2. Object literals: Different formatting than expected

**EXTREME TDD Protocol**:
1. ✅ **Failing tests**: Output format validation
2. ✅ **Property tests**: Format consistency (10K cases)
3. ✅ **Mutation test**: Format logic verified
4. ✅ **Regression tests**: Existing behavior preserved

**Test Requirements**:
```rust
#[test]
fn test_to_string_quotes() {
    let result = eval("42.to_string()");
    assert_eq!(result.to_string(), "\"42\""); // With quotes
}

#[test]
fn test_object_format() {
    let result = eval("{ name: \"Alice\", age: 30 }");
    assert!(result.to_string().contains("name: \"Alice\""));
}

proptest! {
    #[test]
    fn to_string_always_quoted(n: i64) {
        let code = format!("{}.to_string()", n);
        let result = eval(&code).to_string();
        assert!(result.starts_with('"') && result.ends_with('"'));
    }
}
```

**Mutation Coverage Target**: 100%

**Files**:
- `src/runtime/value.rs` - Value::fmt implementation
- `tests/output_format_test.rs` - Format tests

---

### TICKET HYBRID-C-4: Dataframe Parser (4-6 hours)

**Goal**: Implement basic dataframe literal parsing (Chapter 18)

**Current Issue**: Not implemented
```
df!["name" => ["Alice"], "age" => [30]]
→ Error: Unexpected token: FatArrow
```

**Required Grammar**:
```
DataFrame ::= "df" "!" "[" (StringLit "=>" ArrayLit ("," StringLit "=>" ArrayLit)*)? "]"
```

**EXTREME TDD Protocol**:
1. ✅ **Failing tests**: DataFrame parsing
2. ✅ **Property tests**: Valid dataframe structures (10K cases)
3. ✅ **Mutation test**: All parser paths verified
4. ✅ **Integration test**: DataFrame runtime support
5. ✅ **Book examples**: Chapter 18 examples pass

**Test Requirements**:
```rust
#[test]
fn test_dataframe_parsing() {
    let code = r#"df!["name" => ["Alice"], "age" => [30]]"#;
    let ast = parse(code).unwrap();
    assert!(matches!(ast, Expr::DataFrame { .. }));
}

#[test]
fn test_dataframe_execution() {
    let code = r#"df!["x" => [1, 2], "y" => [3, 4]]"#;
    let result = eval(code);
    assert!(matches!(result, Value::DataFrame(_)));
}

proptest! {
    #[test]
    fn dataframe_column_count_matches(cols: Vec<(String, Vec<i64>)>) {
        if cols.is_empty() { return Ok(()); }
        let code = generate_df_code(cols);
        let result = parse(&code);
        // Should parse successfully
        prop_assert!(result.is_ok());
    }
}
```

**Mutation Coverage Target**: 100%

**Files**:
- `src/frontend/parser/collections.rs` - DataFrame parsing
- `src/runtime/eval_dataframe.rs` - DataFrame execution
- `tests/dataframe_test.rs` - Comprehensive tests

---

## Phase 2: Strategic Features (6-10 hours)

### TICKET HYBRID-C-5: Strategic Feature Selection

**Candidates** (select 2-3 based on effort/impact):

#### Option A: Async/Await Syntax (3-4 hours)
- Parse `async fn` and `await` expressions
- Property tests: async composition (10K cases)
- Mutation coverage: 100%

#### Option B: Pattern Guards (2-3 hours)
- Implement `match x { n if n > 0 => ... }`
- Property tests: guard conditions (10K cases)
- Mutation coverage: 100%

#### Option C: Improved Error Messages (2-3 hours)
- Source location in errors
- "Did you mean?" suggestions
- Property tests: error clarity (10K cases)

#### Option D: REPL Multi-line (3-4 hours)
- Enable multi-line input
- Bracket/brace balancing
- Property tests: input parsing (10K cases)

**Selection Criteria**:
1. Maximum user value per hour
2. Enables book examples
3. Low risk of regression
4. Clear test criteria

---

## Quality Gates (MANDATORY - BLOCKING)

### Pre-Implementation (Every Ticket)
```bash
# Baseline check
pmat tdg . --min-grade A- --fail-on-violation
pmat quality-gate --fail-on-violation --checks=all
```

### During Implementation (After Each Function)
```bash
# Function-level quality
pmat tdg <file> --include-components --min-grade B+
pmat analyze complexity --max-cyclomatic 10 --max-cognitive 10
```

### Post-Implementation (Every Ticket)
```bash
# Mutation testing
cargo mutants --file <modified-file> --timeout 600

# Property testing  
cargo test --test properties -- --nocapture

# Coverage verification
cargo llvm-cov --html

# Final quality gate
pmat tdg . --min-grade A- --fail-on-violation
```

### Pre-Commit (BLOCKING)
```bash
# Comprehensive validation
cargo test --all
cargo clippy --all-targets -- -D warnings
pmat tdg . --min-grade A- --fail-on-violation
pmat quality-gate --fail-on-violation --format=detailed
```

---

## Success Criteria

### Phase 1 (Book Compatibility)
- ✅ Book compatibility: 82.6% → 90%+ 
- ✅ String methods: 100% working (to_uppercase, to_lowercase, split)
- ✅ Try-catch: Chapter 17 examples passing
- ✅ Output format: Consistent and correct
- ✅ Dataframes: Chapter 18 basic examples working

### Phase 2 (Strategic Features)
- ✅ 2-3 high-impact features implemented
- ✅ New capabilities enabled
- ✅ Documentation updated

### Quality (MANDATORY)
- ✅ TDG Grade: A- maintained across all files
- ✅ Mutation Coverage: 100% for all new code
- ✅ Property Tests: 10K cases per critical function
- ✅ Test Count: +30-50 comprehensive tests
- ✅ Zero Regressions: All existing tests pass
- ✅ Complexity: All functions ≤10 cyclomatic

---

## Risk Mitigation

### Parser Complexity (Medium Risk)
- **Mitigation**: Start with grammar design, validate with tests first
- **Fallback**: Defer complex features if exceeding effort estimates

### Mutation Testing Runtime (Low Risk)
- **Mitigation**: Use targeted mutation testing per file
- **Fallback**: Accept 80-90% coverage for test oracle limitations

### Scope Creep (Medium Risk)
- **Mitigation**: Strict ticket-based execution, time-box each task
- **Fallback**: Defer Phase 2 features if Phase 1 exceeds estimate

---

## Toyota Way Principles

1. **Jidoka** (自働化): Automated quality checks at every step
2. **Genchi Genbutsu** (現地現物): Test empirically, verify everything
3. **Kaizen** (改善): Continuous improvement via mutation/property tests
4. **Respect for People**: Build quality into process, not blame

---

## Execution Order

1. **HYBRID-C-1**: String methods (2-3h) - Quick win
2. **HYBRID-C-2**: Try-catch parser (3-4h) - High impact
3. **HYBRID-C-3**: Output formatting (2-3h) - Polish
4. **HYBRID-C-4**: Dataframe parser (4-6h) - Major feature
5. **HYBRID-C-5**: Strategic features (6-10h) - Growth

**Total**: 14-20 hours estimated

---

**Start**: HYBRID-C-1 (String Methods) with EXTREME TDD
**Quality**: TDG A-, 100% mutation, 10K property cases per function
**Goal**: Maximum user value with zero quality compromise
