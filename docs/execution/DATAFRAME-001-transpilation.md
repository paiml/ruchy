# DATAFRAME-001: Fix DataFrame Transpilation (CRITICAL)

**Priority**: 🔴 CRITICAL
**Status**: PENDING
**Phase**: 1 (Immediate)
**Estimated Effort**: 12-16 hours
**Impact**: Enables data science use cases in compiled binaries
**Created**: 2025-10-13
**Source**: ruchy-book compatibility report (0/4 DataFrame examples working)

---

## Problem Statement

**Current State**: DataFrames work perfectly in interpreter mode but fail to compile to binaries.

**Error**:
```
error[E0433]: failed to resolve: use of unresolved module or unlinked crate 'polars'
```

**Root Cause**: Transpiled Rust code requires `polars` crate but `Cargo.toml` is not generated automatically.

**Impact**:
- Cannot create production binaries with DataFrame code
- Limits data science applications to scripts only
- Cannot deploy DataFrame apps as standalone executables
- **0/4 examples in Ch18 working** (100% failure rate)

---

## Specification

### Requirements

1. **Auto-generate Cargo.toml** during compilation
2. **Inject polars dependency** with correct version
3. **Support DataFrame operations** in compiled binaries
4. **Maintain interpreter compatibility** (don't break existing functionality)

### Expected Behavior

```bash
# Before (FAILS):
echo 'let df = df![{"x": [1, 2, 3]}]; println(df);' > test.ruchy
ruchy compile test.ruchy
./test  # Error: polars crate not found

# After (WORKS):
echo 'let df = df![{"x": [1, 2, 3]}]; println(df);' > test.ruchy
ruchy compile test.ruchy  # Auto-generates Cargo.toml with polars
./test  # Successfully displays DataFrame
```

### Auto-generated Cargo.toml Template

```toml
[package]
name = "ruchy_binary"
version = "0.1.0"
edition = "2021"

[dependencies]
polars = "0.35"  # Or latest version
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

## EXTREME TDD Implementation Plan

### RED Phase (Write Failing Tests First)

**Test File**: `tests/dataframe_001_transpilation_tdd.rs`

**Tests to Create** (10 unit tests):

1. `test_dataframe_001_basic_compilation`
   - Create DataFrame with df![] macro
   - Compile to binary
   - Execute binary
   - Verify DataFrame displays correctly

2. `test_dataframe_001_cargo_toml_generation`
   - Trigger compilation
   - Check that Cargo.toml is generated
   - Verify polars dependency present
   - Verify correct version

3. `test_dataframe_001_column_operations`
   - Create DataFrame with multiple columns
   - Perform column selection
   - Compile and execute
   - Verify output

4. `test_dataframe_001_filtering`
   - Create DataFrame
   - Apply filter operation
   - Compile and execute
   - Verify filtered results

5. `test_dataframe_001_multiple_dataframes`
   - Create multiple DataFrames
   - Perform operations on each
   - Compile and execute
   - Verify all DataFrames work

6. `test_dataframe_001_error_handling`
   - Invalid DataFrame syntax
   - Compilation should fail with clear error
   - Verify error message mentions DataFrame

7. `test_dataframe_001_large_dataframe`
   - Create DataFrame with 1000+ rows
   - Compile and execute
   - Verify performance acceptable

8. `test_dataframe_001_mixed_types`
   - DataFrame with int, float, string columns
   - Compile and execute
   - Verify all types preserved

9. `test_dataframe_001_cleanup`
   - Compilation creates temp files
   - Verify cleanup happens
   - No leftover files

10. `test_dataframe_001_interpreter_compatibility`
    - Same DataFrame code
    - Run in interpreter mode
    - Compile to binary
    - Both produce identical output

**Property Tests** (REFACTOR phase):
- Random DataFrame sizes
- Random column types
- Random operations
- Target: 10K+ test cases

---

## GREEN Phase (Minimal Implementation)

### Files to Modify

1. **`src/transpiler/mod.rs`** or **`src/compiler/mod.rs`**
   - Add Cargo.toml generation function
   - Detect DataFrame usage in AST
   - Inject polars dependency when needed

2. **`src/cli/commands/compile.rs`**
   - Call Cargo.toml generation before transpilation
   - Pass polars flag to transpiler

3. **`src/transpiler/dataframe.rs`** (new file)
   - Handle df![] macro transpilation
   - Generate Polars-compatible Rust code
   - Map Ruchy DataFrame syntax to Polars API

### Implementation Steps

#### Step 1: Detect DataFrame Usage

```rust
// src/transpiler/analyzer.rs
pub fn uses_dataframes(ast: &Program) -> bool {
    ast.statements.iter().any(|stmt| matches_dataframe_pattern(stmt))
}

fn matches_dataframe_pattern(stmt: &Statement) -> bool {
    // Check for df![] macro calls
    // Check for DataFrame methods
    matches!(stmt, Statement::Expr(Expr::Macro { name, .. }) if name == "df")
}
```

#### Step 2: Generate Cargo.toml

```rust
// src/compiler/cargo_generator.rs
pub fn generate_cargo_toml(binary_name: &str, uses_dataframes: bool) -> String {
    let mut deps = vec![
        ("serde", "{ version = \"1.0\", features = [\"derive\"] }"),
        ("serde_json", "\"1.0\""),
    ];
    
    if uses_dataframes {
        deps.push(("polars", "\"0.35\""));
    }
    
    format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
{}
"#,
        binary_name,
        deps.iter()
            .map(|(name, ver)| format!("{} = {}", name, ver))
            .collect::<Vec<_>>()
            .join("\n")
    )
}
```

#### Step 3: Transpile DataFrame Syntax

```rust
// src/transpiler/dataframe.rs
pub fn transpile_dataframe_macro(columns: &HashMap<String, Vec<Value>>) -> String {
    // df![{"x": [1, 2, 3]}]
    // ↓
    // polars::df! { "x" => &[1, 2, 3] }
    
    let columns_str = columns
        .iter()
        .map(|(name, values)| {
            format!("\"{}\" => &{:?}", name, values)
        })
        .collect::<Vec<_>>()
        .join(", ");
    
    format!("polars::df! {{ {} }}", columns_str)
}
```

#### Step 4: Update Compilation Pipeline

```rust
// src/cli/commands/compile.rs
pub fn compile_to_binary(source_path: &Path) -> Result<()> {
    // 1. Parse Ruchy code
    let ast = parse_file(source_path)?;
    
    // 2. Check if DataFrames are used
    let uses_dataframes = analyzer::uses_dataframes(&ast);
    
    // 3. Generate Cargo.toml
    let cargo_toml = cargo_generator::generate_cargo_toml("output_binary", uses_dataframes);
    std::fs::write("Cargo.toml", cargo_toml)?;
    
    // 4. Transpile to Rust
    let rust_code = transpile(&ast, uses_dataframes)?;
    std::fs::write("src/main.rs", rust_code)?;
    
    // 5. Compile with cargo
    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .status()?;
    
    Ok(())
}
```

---

## REFACTOR Phase

### Property Tests (10K+ iterations)

```rust
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn proptest_dataframe_any_size(rows in 1..1000usize) {
            let code = format!(
                "let df = df![{{\"x\": [{} .. {}]}}]; println(df);",
                0, rows
            );
            
            // Compile and execute
            let result = compile_and_run(&code);
            assert!(result.is_ok());
        }
        
        #[test]
        fn proptest_dataframe_operations(filter_val in 0..100i32) {
            let code = format!(
                "let df = df![{{\"x\": [1, 2, 3, 4, 5]}}]; \
                 let filtered = df.filter(x > {}); \
                 println(filtered);",
                filter_val
            );
            
            let result = compile_and_run(&code);
            assert!(result.is_ok());
        }
    }
}
```

### Mutation Testing

Target: ≥75% mutation coverage on:
- DataFrame detection logic
- Cargo.toml generation
- df![] macro transpilation
- Polars API mapping

---

## Acceptance Criteria

- [ ] All 4 Ch18 examples compile to binaries successfully
- [ ] Compiled binaries produce identical output to interpreter mode
- [ ] Cargo.toml auto-generated with polars dependency
- [ ] 10/10 unit tests passing
- [ ] 10K+ property test cases passing
- [ ] Mutation testing ≥75% coverage
- [ ] Complexity ≤10 for all new functions
- [ ] Documentation updated with DataFrame compilation examples

---

## Known Challenges

### Challenge 1: Polars Version Compatibility

**Issue**: Polars API may change between versions

**Solution**: Lock to specific version in generated Cargo.toml, document requirement

### Challenge 2: Large DataFrame Compilation Time

**Issue**: Polars is a large dependency, increases build time

**Solution**: Document expected build times, consider caching compiled dependencies

### Challenge 3: DataFrame Syntax Differences

**Issue**: Ruchy df![] syntax may not map 1:1 to Polars

**Solution**: Create comprehensive transpilation rules, add tests for edge cases

---

## Related Tickets

- RUNTIME-003: Classes (completed) - Similar pattern for reference types
- RUNTIME-004: Actors (pending) - Similar async/concurrency challenges
- STDLIB-001: String methods (pending) - Related stdlib work

---

## Success Metrics

**Before**:
- DataFrame examples in transpiled mode: 0/4 (0%)
- Binary compilation with DataFrames: FAILS
- ruchy-book compatibility: 84%

**After**:
- DataFrame examples in transpiled mode: 4/4 (100%)
- Binary compilation with DataFrames: WORKS
- ruchy-book compatibility: 87% (+3%)

**Impact**: Enables production deployment of data science applications

---

**Created**: 2025-10-13
**Sprint**: DATAFRAME-001
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR)
**Estimated Completion**: 2-3 days of focused work
