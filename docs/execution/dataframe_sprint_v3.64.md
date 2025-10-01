# DataFrame Implementation Sprint - v3.64.0

## 📊 Sprint Overview

**Objective**: Implement production-ready DataFrames using Polars backend (0% → 100%)
**Duration**: 5-7 days
**Priority**: ⭐⭐⭐ HIGHEST VALUE
**Version Target**: v3.64.0

## Current Status Analysis

### ✅ What Exists (Infrastructure)
- **Parser**: `df![]` syntax parsing working (`ruchy check` passes)
- **AST**: `ExprKind::DataFrame { columns }` defined
- **Transpiler**: DataFrame → Rust code generation exists
- **Runtime Helpers**: Some DataFrame methods (select, sum, slice, join, groupby)
- **Tests**: Comprehensive test suite exists but disabled in `tests.disabled/`

### ❌ Critical Gaps (Blocking)
1. **BLOCKER**: `ExprKind::DataFrame` not handled in interpreter's `eval_expr()`
   - Parser creates AST correctly
   - Interpreter falls through to `eval_misc_expr()` → fails
   - **Fix**: Add `ExprKind::DataFrame` case to `is_data_structure_expr()`

2. **NO LITERAL EVALUATION**: DataFrame literals don't execute
   - `df![age => [25, 30, 35]]` parses but can't evaluate
   - **Fix**: Implement `eval_dataframe_literal()` function

3. **MISSING POLARS INTEGRATION**: Feature flag exists but not connected
   - `Cargo.toml` has `dataframe` feature with Polars
   - No actual Polars usage in runtime
   - **Fix**: Integrate Polars for actual dataframe storage

### 📖 Chapter 18 Requirements (From ruchy-book)

**3 Test Files** (0/3 working):
1. `01-dataframe-creation.ruchy` - Creation, CSV, JSON loading
2. `02-dataframe-operations.ruchy` - Filter, sort, aggregate, transform
3. `03-dataframe-analytics.ruchy` - Real-world analytics use cases

**API Surface** (0% implemented):
```ruchy
// Constructor API
DataFrame::new()
    .column("name", ["Alice", "Bob"])
    .column("age", [25, 30])
    .build()

DataFrame::from_csv_string(csv_data)
DataFrame::from_json(json_data)

// Query methods
df.rows()
df.columns()
df.column_names()
df.get("column", index)

// Operations
df.filter(|row| row["price"] > 50.0)
df.sort_by("name")
df.sort_by("score", descending: true)
df.group_by("department")
    .agg("salary", "mean")
    .agg("years", "sum")

// Transformations
df.with_column("new_col", |row| row["x"] * 2)
df.transform("col", |val| val * 2)

// Statistics
df.column("col").mean()
df.column("col").percentile(95)
df.column("col").std()
df.rolling_mean("col", window_size)
```

## 🎯 Ticket Breakdown (EXTREME TDD)

### **DF-001: DataFrame Literal Evaluation** ⚡ P0-CRITICAL
**Status**: 🔴 BLOCKING - Nothing works without this
**Effort**: 2-3 hours
**Complexity**: <10

**Problem**:
```bash
$ echo 'df![age => [25, 30]]' | ruchy repl
Error: Evaluation error: In backslash lambda after params: Expected Arrow, found Bang
```

**Root Cause**:
- Parser creates `ExprKind::DataFrame` correctly
- Interpreter's `is_data_structure_expr()` doesn't include DataFrame
- Falls through to `eval_misc_expr()` → confusing error

**Fix Strategy**:
1. Add `ExprKind::DataFrame { .. }` to `is_data_structure_expr()`
2. Implement `eval_dataframe_literal()` in `eval_data_structures.rs`
3. Create `Value::DataFrame` with columns

**Test Plan** (TDD):
```rust
#[test]
fn test_empty_dataframe_literal() {
    let mut interp = Interpreter::new();
    let result = interp.eval_code("df![]").unwrap();
    assert_eq!(result, Value::DataFrame { columns: vec![] });
}

#[test]
fn test_single_column_dataframe() {
    let mut interp = Interpreter::new();
    let result = interp.eval_code("df![age => [25, 30, 35]]").unwrap();
    // Verify column exists and has 3 values
}

#[test]
fn test_multi_column_dataframe() {
    let code = r#"df![
        name => ["Alice", "Bob"],
        age => [25, 30]
    ]"#;
    let result = interp.eval_code(code).unwrap();
    // Verify 2 columns, 2 rows
}
```

**Success Criteria**:
- ✅ `df![]` evaluates to empty DataFrame
- ✅ Single column DataFrames work
- ✅ Multi-column DataFrames work
- ✅ All 3 TDD tests passing
- ✅ REPL demo works

**Files to Modify**:
- `src/runtime/eval_expr.rs`: Add DataFrame to `is_data_structure_expr()`
- `src/runtime/eval_data_structures.rs`: Add `eval_dataframe_literal()`
- `tests/dataframe_literal_tests.rs`: New TDD test file

---

### **DF-002: DataFrame Constructor API** 🏗️ P0-CRITICAL
**Status**: 🔴 BLOCKING - Required by all Chapter 18 tests
**Effort**: 1 day
**Complexity**: <10 per function

**Requirements**:
```ruchy
let df = DataFrame::new()
    .column("name", ["Alice", "Bob", "Charlie"])
    .column("age", [25, 30, 35])
    .build();

assert_eq!(df.rows(), 3);
assert_eq!(df.columns(), 3);
assert_eq!(df.column_names(), ["name", "age", "salary"]);
```

**Implementation**:
1. `DataFrame` type in runtime (struct or object)
2. `.new()` static method
3. `.column(name, values)` builder method
4. `.build()` finalizer
5. `.rows()` accessor
6. `.columns()` accessor
7. `.column_names()` accessor

**Test Plan** (TDD):
```rust
#[test]
fn test_dataframe_new_empty() {
    let df = DataFrame::new().build();
    assert_eq!(df.rows(), 0);
    assert_eq!(df.columns(), 0);
}

#[test]
fn test_dataframe_builder_pattern() {
    let df = DataFrame::new()
        .column("x", vec![1, 2, 3])
        .column("y", vec![4, 5, 6])
        .build();
    assert_eq!(df.rows(), 3);
    assert_eq!(df.columns(), 2);
}

#[test]
fn test_dataframe_accessors() {
    let df = create_test_df();
    assert_eq!(df.rows(), 3);
    assert_eq!(df.columns(), 2);
    assert_eq!(df.column_names(), vec!["name", "age"]);
}
```

**Success Criteria**:
- ✅ Builder pattern works
- ✅ Accessors return correct values
- ✅ All 3 TDD tests passing
- ✅ Chapter 18 test 01 section 1 passes

**Files to Create/Modify**:
- `src/runtime/dataframe_builder.rs`: New builder implementation
- `tests/dataframe_constructor_tests.rs`: New TDD tests

---

### **DF-003: CSV/JSON Import** 📂 P1-HIGH
**Status**: 🟡 HIGH PRIORITY - Common use case
**Effort**: 1 day
**Complexity**: <10 per function

**Requirements**:
```ruchy
let csv_data = "name,age,city
Alice,25,New York
Bob,30,San Francisco";

let df = DataFrame::from_csv_string(csv_data);
assert_eq!(df.rows(), 2);
assert_eq!(df.get("name", 0), "Alice");

let json_data = r#"[
    {"name": "Alice", "age": 25},
    {"name": "Bob", "age": 30}
]"#;
let df2 = DataFrame::from_json(json_data);
```

**Implementation**:
1. CSV parsing (use existing CSV library or simple parser)
2. JSON parsing (use serde_json)
3. Type inference for columns
4. Error handling for malformed data

**Test Plan** (TDD):
```rust
#[test]
fn test_from_csv_simple() {
    let csv = "name,age\nAlice,25\nBob,30";
    let df = DataFrame::from_csv_string(csv).unwrap();
    assert_eq!(df.rows(), 2);
    assert_eq!(df.get("name", 0), Value::String("Alice"));
}

#[test]
fn test_from_json_array() {
    let json = r#"[{"x": 1}, {"x": 2}]"#;
    let df = DataFrame::from_json(json).unwrap();
    assert_eq!(df.rows(), 2);
}

#[test]
fn test_csv_type_inference() {
    let csv = "num,text\n42,hello\n100,world";
    let df = DataFrame::from_csv_string(csv).unwrap();
    assert_eq!(df.get("num", 0), Value::Integer(42));
}
```

**Success Criteria**:
- ✅ CSV import works with header
- ✅ JSON array import works
- ✅ Type inference (int, float, string)
- ✅ All 3 TDD tests passing
- ✅ Chapter 18 test 01 sections 2-3 pass

**Files to Create**:
- `src/runtime/dataframe_io.rs`: CSV/JSON importers
- `tests/dataframe_io_tests.rs`: TDD tests

---

### **DF-004: Filter and Transform Operations** 🔄 P1-HIGH
**Status**: 🟡 HIGH PRIORITY - Core operations
**Effort**: 1-2 days
**Complexity**: <10 per function

**Requirements**:
```ruchy
let df = create_products_df();

// Filter with closure
let expensive = df.filter(|row| row["price"] > 50.0);

// Transform with closure
let with_tax = df.with_column("price_with_tax", |row| {
    row["price"] * 1.08
});

// Modify existing column
let doubled = df.transform("quantity", |val| val * 2);
```

**Implementation**:
1. `.filter(closure)` method
2. `.with_column(name, closure)` method
3. `.transform(name, closure)` method
4. Closure evaluation with row context

**Test Plan** (TDD):
```rust
#[test]
fn test_filter_numeric() {
    let df = df_with_ages([20, 25, 30, 35]);
    let filtered = df.filter(|row| row["age"] > 25);
    assert_eq!(filtered.rows(), 2);
}

#[test]
fn test_with_column() {
    let df = df_with_column("x", vec![1, 2, 3]);
    let result = df.with_column("y", |row| row["x"] * 2);
    assert_eq!(result.columns(), 2);
    assert_eq!(result.get("y", 0), Value::Integer(2));
}

#[test]
fn test_transform_existing() {
    let df = df_with_column("x", vec![1, 2, 3]);
    let result = df.transform("x", |val| val * 10);
    assert_eq!(result.get("x", 0), Value::Integer(10));
}
```

**Success Criteria**:
- ✅ Filter works with closures
- ✅ with_column adds new column
- ✅ transform modifies existing
- ✅ All 3 TDD tests passing
- ✅ Chapter 18 test 02 sections 1,4 pass

**Files to Create**:
- `src/runtime/dataframe_operations.rs`: Filter/transform logic
- `tests/dataframe_operations_tests.rs`: TDD tests

---

### **DF-005: Sort and Aggregation** 📊 P1-HIGH
**Status**: 🟡 HIGH PRIORITY - Analytics essential
**Effort**: 1-2 days
**Complexity**: <10 per function

**Requirements**:
```ruchy
// Sort ascending
let sorted = df.sort_by("name");

// Sort descending
let sorted_desc = df.sort_by("score", descending: true);

// Group and aggregate
let grouped = df.group_by("department")
    .agg("salary", "mean")
    .agg("years", "sum");
```

**Implementation**:
1. `.sort_by(column)` method
2. `.sort_by(column, descending: bool)` variant
3. `.group_by(column)` returns GroupedDataFrame
4. `.agg(column, function)` on grouped DF
5. Aggregation functions: sum, mean, count, min, max

**Test Plan** (TDD):
```rust
#[test]
fn test_sort_ascending() {
    let df = df_with_column("x", vec![3, 1, 2]);
    let sorted = df.sort_by("x");
    assert_eq!(sorted.get("x", 0), Value::Integer(1));
    assert_eq!(sorted.get("x", 2), Value::Integer(3));
}

#[test]
fn test_group_by_sum() {
    let df = df_with_groups_and_values();
    let grouped = df.group_by("category").agg("amount", "sum");
    // Verify aggregation results
}

#[test]
fn test_group_by_multiple_aggs() {
    let df = employees_df();
    let result = df.group_by("dept")
        .agg("salary", "mean")
        .agg("years", "sum");
    assert_eq!(result.columns(), 3); // dept, salary_mean, years_sum
}
```

**Success Criteria**:
- ✅ Sort ascending/descending works
- ✅ group_by works
- ✅ Multiple aggregations work
- ✅ All 3 TDD tests passing
- ✅ Chapter 18 test 02 sections 2-3 pass

**Files to Create**:
- `src/runtime/dataframe_aggregation.rs`: Sort/group/agg logic
- `tests/dataframe_aggregation_tests.rs`: TDD tests

---

### **DF-006: Statistics Methods** 📈 P2-MEDIUM
**Status**: 🟢 ENHANCEMENT - Nice to have
**Effort**: 1 day
**Complexity**: <10 per function

**Requirements**:
```ruchy
let avg = df.column("response_time").mean();
let p95 = df.column("response_time").percentile(95);
let std_dev = df.column("cpu_usage").std();
let smoothed = df.rolling_mean("cpu_usage", window_size);
```

**Implementation**:
1. `.column(name)` returns Column accessor
2. `.mean()` on Column
3. `.percentile(p)` on Column
4. `.std()` on Column
5. `.rolling_mean(column, window)` on DataFrame

**Test Plan** (TDD - Property Tests)**:
```rust
#[test]
fn test_mean_calculation() {
    let df = df_with_column("x", vec![1, 2, 3, 4, 5]);
    assert_eq!(df.column("x").mean(), 3.0);
}

proptest! {
    #[test]
    fn prop_mean_is_within_range(values in vec(1i64..100, 1..1000)) {
        let df = df_from_values(values.clone());
        let mean = df.column("x").mean();
        let min = values.iter().min().unwrap();
        let max = values.iter().max().unwrap();
        prop_assert!(mean >= *min as f64 && mean <= *max as f64);
    }
}

#[test]
fn test_rolling_mean() {
    let df = df_with_column("x", vec![1, 2, 3, 4, 5]);
    let smoothed = df.rolling_mean("x", 3);
    // First 2 rows should be nil/nan, then rolling averages
}
```

**Success Criteria**:
- ✅ mean/std/percentile work
- ✅ rolling_mean works
- ✅ Property tests pass (10K+ iterations)
- ✅ Chapter 18 test 03 passes

**Files to Create**:
- `src/runtime/dataframe_stats.rs`: Statistics calculations
- `tests/dataframe_stats_tests.rs`: TDD + property tests

---

### **DF-007: Polars Integration** 🚀 P2-MEDIUM
**Status**: 🟢 OPTIMIZATION - Performance boost
**Effort**: 1-2 days
**Complexity**: <10 per integration point

**Requirements**:
- Use Polars for actual dataframe storage (not custom structs)
- Leverage Polars' lazy evaluation
- Use Arrow memory format for efficiency
- Performance: 1M rows in <1s

**Implementation**:
1. Replace custom DataFrame struct with Polars wrapper
2. Convert Ruchy Value → Polars Series
3. Convert Polars DataFrame → Ruchy Value
4. Use lazy evaluation where possible

**Test Plan** (Performance + Property Tests):
```rust
#[test]
fn test_polars_large_dataframe() {
    let rows: Vec<_> = (0..1_000_000).collect();
    let df = DataFrame::from_column("id", rows);
    let start = Instant::now();
    let filtered = df.filter(|row| row["id"] > 500_000);
    assert!(start.elapsed() < Duration::from_secs(1));
}

proptest! {
    #[test]
    fn prop_polars_operations_equivalent(
        values in vec(any::<i64>(), 1..10000)
    ) {
        let custom_df = CustomDataFrame::from(values.clone());
        let polars_df = PolarsDataFrame::from(values);

        let custom_result = custom_df.filter(|v| v > 50).sum();
        let polars_result = polars_df.filter(|v| v > 50).sum();

        prop_assert_eq!(custom_result, polars_result);
    }
}
```

**Success Criteria**:
- ✅ Polars backend working
- ✅ 1M rows < 1s performance
- ✅ Property tests confirm correctness
- ✅ All existing tests still pass

**Files to Modify**:
- `src/runtime/dataframe_polars.rs`: New Polars integration
- `tests/dataframe_polars_tests.rs`: Performance tests

---

## 📅 Implementation Timeline (5-7 days)

### Day 1: Foundation (P0 CRITICAL)
- **Morning**: DF-001 - DataFrame literal evaluation (2-3 hours)
  - Fix `is_data_structure_expr()`
  - Implement `eval_dataframe_literal()`
  - TDD tests passing
- **Afternoon**: DF-002 - Constructor API (4 hours)
  - Builder pattern implementation
  - Accessors (rows, columns, column_names)
  - TDD tests passing

### Day 2: Data Import (P1 HIGH)
- **Full Day**: DF-003 - CSV/JSON import
  - CSV parsing implementation
  - JSON parsing implementation
  - Type inference
  - TDD + Chapter 18 test 01 passing

### Day 3: Core Operations (P1 HIGH)
- **Full Day**: DF-004 - Filter and Transform
  - Filter with closures
  - with_column implementation
  - transform implementation
  - TDD + Chapter 18 test 02 partial passing

### Day 4: Analytics (P1 HIGH)
- **Full Day**: DF-005 - Sort and Aggregation
  - Sorting (asc/desc)
  - group_by implementation
  - Aggregation functions
  - TDD + Chapter 18 test 02 complete

### Day 5: Statistics (P2 MEDIUM)
- **Full Day**: DF-006 - Statistics Methods
  - mean, std, percentile
  - rolling_mean
  - Property tests (10K+ iterations)
  - Chapter 18 test 03 passing

### Day 6-7: Polars Integration (P2 MEDIUM - Optional)
- **2 Days**: DF-007 - Polars Backend
  - Replace custom structs
  - Performance optimization
  - Property tests for equivalence
  - 1M row benchmarks

## 🎯 Success Metrics

### Code Quality (PMAT TDG A-)
- ✅ All functions <10 complexity
- ✅ Zero SATD comments
- ✅ PMAT TDG grade ≥ A- (85 points)
- ✅ No clippy warnings

### Test Coverage (80%+)
- ✅ 30+ TDD unit tests
- ✅ 10+ property tests (10K iterations each)
- ✅ 3 Chapter 18 integration tests
- ✅ All tests passing

### Book Compatibility
- ✅ Chapter 18: 0/4 → 4/4 (100%)
- ✅ Overall: 92.5% → 96% (64/67 examples)

### Performance (Production Ready)
- ✅ 1M rows processed in <1s
- ✅ Memory efficient (Arrow format if Polars)
- ✅ No memory leaks

## 🔥 Risk Mitigation

### Risk 1: Polars Integration Complexity
**Mitigation**: Start with custom DataFrame, migrate to Polars in DF-007
**Fallback**: Ship v3.64.0 with custom implementation, Polars in v3.65.0

### Risk 2: Closure Evaluation Complexity
**Mitigation**: Use existing lambda evaluation infrastructure
**Fallback**: Start with simple filters (no closures), add closures in patch

### Risk 3: Time Overrun (>7 days)
**Mitigation**: DF-006 and DF-007 are P2, can defer to v3.65.0
**Minimum Viable**: DF-001 through DF-005 = 100% Chapter 18 compatibility

## 📝 Daily Commit Protocol

Every commit MUST include:
```
[DF-XXX] Brief description

Detailed changes:
- Specific improvements
- Test coverage added
- Complexity maintained <10

TDG Score Changes:
- src/file.rs: XX.X→YY.Y (Grade→Grade) [+Z.Z improvement]

Success Metrics:
- Tests: X passing
- Complexity: All functions <10
- Coverage: XX%

Closes: DF-XXX
```

## 🚀 Release Checklist (v3.64.0)

- [ ] DF-001 through DF-005 complete (minimum)
- [ ] All TDD tests passing (30+)
- [ ] Chapter 18: 4/4 examples working
- [ ] Property tests passing (10K+ iterations each)
- [ ] PMAT TDG grade ≥ A-
- [ ] Zero clippy warnings
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version bump to 3.64.0
- [ ] Published to crates.io
- [ ] GitHub release created
- [ ] Roadmap updated

---

**Date Created**: 2025-10-01
**Sprint Start**: Ready to begin
**Target Completion**: 2025-10-08 (7 days)
**Methodology**: EXTREME TDD + PMAT Quality Gates + Toyota Way
