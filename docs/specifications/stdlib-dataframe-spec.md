# STD-007: DataFrame Module Specification

**Date**: 2025-10-10
**Status**: PLANNED
**Module**: `src/stdlib/dataframe.rs`
**Tests**: `tests/std_007_dataframe.rs`
**Ticket**: DF-IMPL (Phase 2, Month 2)

## Overview

Thin wrapper module around polars-rs for DataFrame operations in Ruchy.

**Design Philosophy**:
- Thin wrappers (complexity ≤3 per function) around polars DataFrame
- Behind feature flag: `#[cfg(feature = "dataframe")]`
- 100% unit test coverage
- Property tests (≥20 cases)
- Mutation tests (≥75% coverage)

## Core Functions

### 1. Creation & I/O

#### `from_columns`
```rust
pub fn from_columns(columns: Vec<(&str, Vec<i64>)>) -> Result<polars::prelude::DataFrame, String>
```
Create DataFrame from column name-value pairs.

**Example**:
```ruchy
let df = DataFrame::from_columns([
    ("age", [25, 30, 35]),
    ("score", [95, 87, 92])
]);
```

**Test Coverage**:
- Empty columns
- Single column
- Multiple columns with same length
- Mismatched column lengths (error case)

#### `read_csv`
```rust
pub fn read_csv(path: &str) -> Result<polars::prelude::DataFrame, String>
```
Read CSV file into DataFrame.

**Example**:
```ruchy
let df = DataFrame::read_csv("data.csv");
```

**Test Coverage**:
- Valid CSV file
- File not found (error case)
- Invalid CSV format (error case)
- Empty CSV
- CSV with headers

#### `write_csv`
```rust
pub fn write_csv(df: &polars::prelude::DataFrame, path: &str) -> Result<(), String>
```
Write DataFrame to CSV file.

**Example**:
```ruchy
DataFrame::write_csv(df, "output.csv");
```

**Test Coverage**:
- Write and read back (roundtrip)
- Invalid path (error case)
- Overwrite existing file

### 2. Selection & Filtering

#### `select`
```rust
pub fn select(df: &polars::prelude::DataFrame, columns: &[&str]) -> Result<polars::prelude::DataFrame, String>
```
Select specific columns from DataFrame.

**Example**:
```ruchy
let subset = DataFrame::select(df, ["age", "name"]);
```

**Test Coverage**:
- Select single column
- Select multiple columns
- Select non-existent column (error case)
- Select all columns
- Select empty columns list

#### `head`
```rust
pub fn head(df: &polars::prelude::DataFrame, n: usize) -> Result<polars::prelude::DataFrame, String>
```
Get first n rows.

**Example**:
```ruchy
let top5 = DataFrame::head(df, 5);
```

**Test Coverage**:
- n < total rows
- n > total rows (returns all)
- n = 0
- Empty DataFrame

#### `tail`
```rust
pub fn tail(df: &polars::prelude::DataFrame, n: usize) -> Result<polars::prelude::DataFrame, String>
```
Get last n rows.

**Example**:
```ruchy
let bottom5 = DataFrame::tail(df, 5);
```

**Test Coverage**:
- Same as head tests

### 3. Metadata

#### `shape`
```rust
pub fn shape(df: &polars::prelude::DataFrame) -> Result<(usize, usize), String>
```
Get (rows, columns) dimensions.

**Example**:
```ruchy
let (rows, cols) = DataFrame::shape(df);
```

**Test Coverage**:
- Non-empty DataFrame
- Empty DataFrame (0 rows, N columns)
- Single row, single column

#### `columns`
```rust
pub fn columns(df: &polars::prelude::DataFrame) -> Result<Vec<String>, String>
```
Get column names.

**Example**:
```ruchy
let names = DataFrame::columns(df);
```

**Test Coverage**:
- Multiple columns
- Single column
- Empty DataFrame

#### `row_count`
```rust
pub fn row_count(df: &polars::prelude::DataFrame) -> Result<usize, String>
```
Get number of rows.

**Example**:
```ruchy
let count = DataFrame::row_count(df);
```

**Test Coverage**:
- Non-empty DataFrame
- Empty DataFrame

## Quality Gates

### EXTREME TDD Requirements
1. **RED**: Write tests FIRST (all tests in place before implementation)
2. **GREEN**: Implement minimal code to pass tests
3. **REFACTOR**: Run FAST mutation testing (≥75% coverage)

### Mutation Testing Strategy
```bash
cargo mutants --file src/stdlib/dataframe.rs --features dataframe -- --test std_007_dataframe
```

**Target**: ≥75% mutation coverage
**Expected Runtime**: 10-15 minutes (following FAST pattern)

### Property Tests (≥3 required)
1. **Roundtrip**: `write_csv` → `read_csv` preserves data
2. **Never Panics**: All functions return Result, no panics on invalid input
3. **Shape Consistency**: `select` preserves row count, changes column count

### Test Organization
```
tests/std_007_dataframe.rs
├── Creation Tests (6)
│   ├── from_columns (empty, single, multiple, mismatched)
│   ├── read_csv (valid, not found, invalid)
├── Selection Tests (8)
│   ├── select (single, multiple, none, all, invalid)
│   ├── head/tail (various n values)
├── Metadata Tests (6)
│   ├── shape, columns, row_count
└── Property Tests (3)
    ├── CSV roundtrip
    ├── Never panics
    └── Shape consistency
```

**Total**: 23 tests (20 unit + 3 property)

## Dependencies

### Cargo.toml
```toml
[features]
dataframe = ["polars", "arrow", "arrow-array", "arrow-buffer", "arrow-schema"]

[dependencies]
polars = { version = "0.50", features = ["lazy"], optional = true }
arrow = { version = "54.0", optional = true }
arrow-array = { version = "54.0", optional = true }
arrow-buffer = { version = "54.0", optional = true }
arrow-schema = { version = "54.0", optional = true }
```

Already present in Cargo.toml ✅

## Module Structure

```rust
//! DataFrame Operations Module (ruchy/std/dataframe)
//!
//! Thin wrappers around Polars DataFrame for data manipulation.
//!
//! **Design**: Thin wrappers (complexity ≤3 per function) around polars crate.
//! **Quality**: 100% unit test coverage, property tests, ≥75% mutation coverage.
//! **Feature**: Only available when compiled with --features dataframe

#[cfg(feature = "dataframe")]
use polars::prelude::*;

// Creation & I/O
pub fn from_columns(...) -> Result<DataFrame, String> { ... }
pub fn read_csv(...) -> Result<DataFrame, String> { ... }
pub fn write_csv(...) -> Result<(), String> { ... }

// Selection & Filtering
pub fn select(...) -> Result<DataFrame, String> { ... }
pub fn head(...) -> Result<DataFrame, String> { ... }
pub fn tail(...) -> Result<DataFrame, String> { ... }

// Metadata
pub fn shape(...) -> Result<(usize, usize), String> { ... }
pub fn columns(...) -> Result<Vec<String>, String> { ... }
pub fn row_count(...) -> Result<usize, String> { ... }
```

## Success Criteria

- ✅ All 23 tests passing
- ✅ FAST mutation testing runtime <15 minutes
- ✅ ≥75% mutation coverage
- ✅ Complexity ≤3 per function
- ✅ 100% function coverage
- ✅ Compiles with and without `--features dataframe`
- ✅ No panics on invalid input (all errors via Result)

## Timeline Estimate

**Based on STD-001 through STD-006 experience:**
- Tests (RED phase): 2 hours
- Implementation (GREEN phase): 2 hours
- Mutation testing (REFACTOR): 1 hour
- Documentation: 1 hour

**Total**: 6 hours (vs 20 hours in original estimate - 70% savings via thin wrapper strategy!)

## Out of Scope (Future Work)

- Lambda/closure-based filtering (requires language support)
- Aggregations (groupby, sum, mean) - STD-008
- Joins (left, right, inner, outer) - STD-009
- Window functions - STD-010
- Polars lazy evaluation - STD-011

These are deferred to Phase 2 stdlib expansion after core DataFrame operations are validated.
