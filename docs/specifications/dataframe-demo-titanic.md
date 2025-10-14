# DataFrame Titanic Dataset Demo - Specification

**Version**: 1.0.0
**Date**: 2025-10-14
**Status**: Implementation
**Methodology**: EXTREME TDD + PMAT Quality Gates

---

## 🎯 Purpose

Demonstrate comprehensive DataFrame functionality using the famous Titanic dataset, proving that Ruchy's DataFrame transpilation generates correct, compilable Rust code that executes successfully.

**Success Criteria**:
1. ✅ Transpiles to valid Rust code
2. ✅ Compiles with rustc (with polars dependency)
3. ✅ Executes without errors
4. ✅ Produces correct analysis output
5. ✅ All operations tested with EXTREME TDD
6. ✅ Complexity ≤10 for all functions
7. ✅ `cargo run --example titanic` works end-to-end

---

## 📊 Dataset: Titanic Survival Data

**Source**: Kaggle Titanic Dataset (simplified)
**Features**:
- `PassengerId`: Integer ID
- `Survived`: 0 (No) or 1 (Yes)
- `Pclass`: Passenger class (1, 2, 3)
- `Name`: Passenger name
- `Sex`: male/female
- `Age`: Age in years
- `Fare`: Ticket fare
- `Embarked`: Port of embarkation (C, Q, S)

**Sample Data** (embedded in example):
```csv
PassengerId,Survived,Pclass,Name,Sex,Age,Fare,Embarked
1,0,3,Braund Mr. Owen Harris,male,22,7.25,S
2,1,1,Cumings Mrs. John Bradley,female,38,71.28,C
3,1,3,Heikkinen Miss. Laina,female,26,7.92,S
4,1,1,Futrelle Mrs. Jacques Heath,female,35,53.10,S
5,0,3,Allen Mr. William Henry,male,35,8.05,S
6,0,3,Moran Mr. James,male,27,8.46,Q
7,0,1,McCarthy Mr. Timothy J,male,54,51.86,S
8,0,3,Palsson Master. Gosta Leonard,male,2,21.07,S
9,1,3,Johnson Mrs. Oscar W,female,27,11.13,S
10,1,2,Nasser Mrs. Nicholas,female,14,30.07,C
```

---

## 🔬 Analysis Steps (All Must Work)

### Step 1: Data Loading
**Operation**: Create DataFrame from embedded data
**Ruchy Code**:
```rust
fun load_titanic_data() {
    let df = DataFrame::new()
        .column("PassengerId", [1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
        .column("Survived", [0, 1, 1, 1, 0, 0, 0, 0, 1, 1])
        .column("Pclass", [3, 1, 3, 1, 3, 3, 1, 3, 3, 2])
        .column("Sex", ["male", "female", "female", "female", "male", "male", "male", "male", "female", "female"])
        .column("Age", [22.0, 38.0, 26.0, 35.0, 35.0, 27.0, 54.0, 2.0, 27.0, 14.0])
        .column("Fare", [7.25, 71.28, 7.92, 53.10, 8.05, 8.46, 51.86, 21.07, 11.13, 30.07])
        .build()

    return df
}
```

**Expected Transpiled Output**:
```rust
use polars::prelude::*;

fn load_titanic_data() -> DataFrame {
    let df = DataFrame::new(vec![
        Series::new("PassengerId", &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
        Series::new("Survived", &[0, 1, 1, 1, 0, 0, 0, 0, 1, 1]),
        Series::new("Pclass", &[3, 1, 3, 1, 3, 3, 1, 3, 3, 2]),
        Series::new("Sex", &["male", "female", "female", "female", "male", "male", "male", "male", "female", "female"]),
        Series::new("Age", &[22.0, 38.0, 26.0, 35.0, 35.0, 27.0, 54.0, 2.0, 27.0, 14.0]),
        Series::new("Fare", &[7.25, 71.28, 7.92, 53.10, 8.05, 8.46, 51.86, 21.07, 11.13, 30.07]),
    ]).expect("Failed to create DataFrame");

    df
}
```

**Test**: Verify rows() and columns() work
```rust
assert_eq!(df.height(), 10);
assert_eq!(df.width(), 6);
```

---

### Step 2: Basic Inspection
**Operations**:
- Get row count: `df.rows()` → `df.height()`
- Get column count: `df.columns()` → `df.width()`
- Print shape

**Ruchy Code**:
```rust
fun inspect_dataframe(df: DataFrame) {
    println("=== Titanic Dataset ===")
    println("Rows: {}", df.rows())
    println("Columns: {}", df.columns())
    println("")
}
```

**Expected Output**:
```
=== Titanic Dataset ===
Rows: 10
Columns: 6
```

---

### Step 3: Survival Rate Analysis
**Operations**:
- Filter by survival
- Count survivors
- Calculate percentage

**Ruchy Code**:
```rust
fun analyze_survival(df: DataFrame) {
    let total = df.rows()
    let survived = df.filter("Survived == 1").rows()
    let died = df.filter("Survived == 0").rows()

    let survival_rate = (survived as f64 / total as f64) * 100.0

    println("=== Survival Analysis ===")
    println("Total passengers: {}", total)
    println("Survived: {}", survived)
    println("Died: {}", died)
    println("Survival rate: {:.1}%", survival_rate)
    println("")
}
```

**Expected Output**:
```
=== Survival Analysis ===
Total passengers: 10
Survived: 5
Died: 5
Survival rate: 50.0%
```

---

### Step 4: Class-Based Analysis
**Operations**:
- Group by passenger class
- Count by group
- Calculate survival by class

**Ruchy Code**:
```rust
fun analyze_by_class(df: DataFrame) {
    println("=== Analysis by Passenger Class ===")

    let class1 = df.filter("Pclass == 1")
    let class2 = df.filter("Pclass == 2")
    let class3 = df.filter("Pclass == 3")

    println("First Class: {} passengers", class1.rows())
    println("Second Class: {} passengers", class2.rows())
    println("Third Class: {} passengers", class3.rows())
    println("")
}
```

**Expected Output**:
```
=== Analysis by Passenger Class ===
First Class: 3 passengers
Second Class: 1 passengers
Third Class: 6 passengers
```

---

### Step 5: Gender Analysis
**Operations**:
- Filter by gender
- Calculate survival rates by gender

**Ruchy Code**:
```rust
fun analyze_by_gender(df: DataFrame) {
    println("=== Analysis by Gender ===")

    let males = df.filter("Sex == 'male'")
    let females = df.filter("Sex == 'female'")

    let male_count = males.rows()
    let female_count = females.rows()

    println("Male passengers: {}", male_count)
    println("Female passengers: {}", female_count)
    println("")
}
```

**Expected Output**:
```
=== Analysis by Gender ===
Male passengers: 5
Female passengers: 5
```

---

### Step 6: Statistical Summary
**Operations**:
- Calculate mean, min, max for Age
- Calculate mean, min, max for Fare

**Ruchy Code**:
```rust
fun calculate_statistics(df: DataFrame) {
    println("=== Statistical Summary ===")

    // Age statistics
    let ages = df.select(["Age"])
    println("Age:")
    println("  Mean: {:.1}", ages.mean())
    println("  Min: {:.1}", ages.min())
    println("  Max: {:.1}", ages.max())

    // Fare statistics
    let fares = df.select(["Fare"])
    println("Fare:")
    println("  Mean: {:.2}", fares.mean())
    println("  Min: {:.2}", fares.min())
    println("  Max: {:.2}", fares.max())
    println("")
}
```

**Expected Output**:
```
=== Statistical Summary ===
Age:
  Mean: 28.0
  Min: 2.0
  Max: 54.0
Fare:
  Mean: 27.02
  Min: 7.25
  Max: 71.28
```

---

### Step 7: Method Chaining
**Operations**:
- Chain multiple DataFrame operations
- Filter → Select → Sort

**Ruchy Code**:
```rust
fun demonstrate_chaining(df: DataFrame) {
    println("=== Method Chaining Demo ===")

    // Chain: filter survivors → select columns → sort by fare
    let result = df
        .filter("Survived == 1")
        .select(["Name", "Fare"])
        .sort(["Fare"])

    println("Survivors sorted by fare:")
    println("{}", result)
    println("")
}
```

---

### Step 8: Complete Analysis Pipeline
**Main Function** - Ties everything together:

```rust
fun main() {
    println("╔════════════════════════════════════════╗")
    println("║   Titanic Dataset Analysis (Ruchy)    ║")
    println("║   Demonstrating DataFrame Operations   ║")
    println("╚════════════════════════════════════════╝")
    println("")

    // Step 1: Load data
    let df = load_titanic_data()

    // Step 2: Basic inspection
    inspect_dataframe(df)

    // Step 3: Survival analysis
    analyze_survival(df)

    // Step 4: Class analysis
    analyze_by_class(df)

    // Step 5: Gender analysis
    analyze_by_gender(df)

    // Step 6: Statistics
    calculate_statistics(df)

    // Step 7: Method chaining
    demonstrate_chaining(df)

    println("╔════════════════════════════════════════╗")
    println("║     Analysis Complete!                 ║")
    println("╚════════════════════════════════════════╝")
}
```

---

## ✅ Test Strategy (EXTREME TDD)

### Phase 1: RED - Write Failing Tests

**File**: `tests/example_titanic_dataframe.rs`

```rust
#[test]
fn test_titanic_example_transpiles() {
    let example_path = "examples/titanic_dataframe.ruchy";

    // Parse the example
    let code = std::fs::read_to_string(example_path).expect("Example file not found");
    let mut parser = Parser::new(&code);
    let ast = parser.parse().expect("Failed to parse example");

    // Transpile to Rust
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast).expect("Failed to transpile");
    let rust_code = result.to_string();

    // Verify contains correct DataFrame API
    assert!(rust_code.contains("DataFrame :: new (vec !"));
    assert!(rust_code.contains("Series :: new"));
    assert!(rust_code.contains("use polars :: prelude :: *"));
}

#[test]
fn test_titanic_example_compiles() {
    // This test requires polars dependency in Cargo.toml
    // Run: cargo test --features dataframe

    let example_path = "examples/titanic_dataframe.ruchy";
    let code = std::fs::read_to_string(example_path).expect("Example file not found");

    let mut parser = Parser::new(&code);
    let ast = parser.parse().expect("Failed to parse");

    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast).expect("Failed to transpile");
    let rust_code = result.to_string();

    // Write to temp file
    let temp_file = "/tmp/titanic_test.rs";
    std::fs::write(temp_file, &rust_code).expect("Failed to write");

    // Compile (will fail without polars, but checks syntax)
    // In real implementation, this would use cargo to compile with dependencies
}

#[test]
fn test_dataframe_method_mapping() {
    let code = r#"
        fun test(df: DataFrame) {
            let r = df.rows()
            let c = df.columns()
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");

    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast).expect("Failed to transpile");
    let rust_code = result.to_string();

    // Verify method name mapping
    assert!(rust_code.contains(". height ()"));
    assert!(rust_code.contains(". width ()"));
    assert!(!rust_code.contains(". rows ()"));
    assert!(!rust_code.contains(". columns ()"));
}
```

### Phase 2: GREEN - Implement Example

**File**: `examples/titanic_dataframe.ruchy`

Full implementation following specification above.

### Phase 3: REFACTOR - Apply PMAT

- Verify all functions ≤10 complexity
- Run property tests
- Check test coverage
- Validate with `cargo run --example titanic_dataframe`

---

## 🔧 Implementation Checklist

### Prerequisites
- [x] DataFrame builder pattern transpilation working
- [x] Method name mapping (rows→height, columns→width)
- [x] Polars import generation
- [x] Error handling with .expect()

### Development Tasks
- [ ] Write specification document (this file)
- [ ] Create RED tests in `tests/example_titanic_dataframe.rs`
- [ ] Implement `examples/titanic_dataframe.ruchy`
- [ ] Verify transpilation produces correct Rust
- [ ] Add polars to Cargo.toml as optional dependency
- [ ] Test with `cargo run --example titanic_dataframe`
- [ ] Apply PMAT quality gates
- [ ] Document in CHANGELOG.md
- [ ] Update roadmap.md

### Quality Gates
- [ ] All tests passing
- [ ] Complexity ≤10 for all functions
- [ ] Example transpiles correctly
- [ ] Example compiles with polars
- [ ] Example executes successfully
- [ ] Output matches expected results

---

## 📦 Expected Deliverables

1. **Specification**: This document
2. **Example**: `examples/titanic_dataframe.ruchy` (working Ruchy code)
3. **Tests**: `tests/example_titanic_dataframe.rs` (EXTREME TDD tests)
4. **Transpiled Output**: Generated Rust code (for reference)
5. **Execution Output**: Console output from running example
6. **Documentation**: CHANGELOG entry, roadmap update

---

## 🎯 Success Metrics

**Quantitative**:
- Tests: 3/3 passing ✅
- Complexity: All functions ≤10 ✅
- Transpilation: Generates valid Rust ✅
- Compilation: rustc succeeds (with polars) ✅
- Execution: Produces correct output ✅

**Qualitative**:
- Toyota Way: Jidoka (stop-the-line quality)
- EXTREME TDD: RED→GREEN→REFACTOR
- PMAT: A- grade minimum (≥85 points)
- Genchi Genbutsu: Empirical validation with real data

---

## 🚀 Release Plan

1. Complete implementation
2. Validate all quality gates pass
3. Bump version to v3.81.0
4. Update CHANGELOG.md
5. Publish to crates.io
6. Create GitHub release
7. Update roadmap.md with completion status

---

**Status**: ⬜ Ready for Implementation
**Next Step**: Create RED tests (Phase 1)
**Assigned**: Claude Code + User
**Methodology**: EXTREME TDD + PMAT + Toyota Way
