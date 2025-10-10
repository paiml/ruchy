# STD-008: Time Module Specification

**Date**: 2025-10-10
**Status**: PLANNED
**Module**: `src/stdlib/time.rs`
**Tests**: `tests/std_008_time.rs`
**Ticket**: TIME-IMPL (Phase 2, Month 2)

## Overview

Thin wrapper module around Rust's `std::time` for time operations in Ruchy.

**Design Philosophy**:
- Thin wrappers (complexity ≤2 per function) around std::time
- No feature flags needed (standard library)
- 100% unit test coverage
- Property tests (≥20 cases)
- Mutation tests (≥75% coverage)

## Core Functions

### 1. Time Measurement

#### `now`
```rust
pub fn now() -> Result<u128, String>
```
Get current system time in milliseconds since Unix epoch.

**Example**:
```ruchy
let timestamp = Time::now();
```

**Test Coverage**:
- Returns positive integer
- Monotonically increasing (second call >= first call)
- Reasonable range (> 2020-01-01, < 2100-01-01)

#### `elapsed_millis`
```rust
pub fn elapsed_millis(start: u128) -> Result<u128, String>
```
Calculate elapsed milliseconds since start time.

**Example**:
```ruchy
let start = Time::now();
// ... do work ...
let elapsed = Time::elapsed_millis(start);
```

**Test Coverage**:
- Returns >= 0
- Elapsed time >= 0 for immediate calls
- Elapsed increases over time

### 2. Duration Operations

#### `sleep_millis`
```rust
pub fn sleep_millis(millis: u64) -> Result<(), String>
```
Sleep for specified milliseconds.

**Example**:
```ruchy
Time::sleep_millis(100);  // Sleep for 100ms
```

**Test Coverage**:
- Sleeps for approximately correct duration
- Does not panic on zero
- Does not panic on large values

#### `duration_secs`
```rust
pub fn duration_secs(millis: u128) -> Result<f64, String>
```
Convert milliseconds to seconds.

**Example**:
```ruchy
let secs = Time::duration_secs(1500);  // 1.5 seconds
```

**Test Coverage**:
- Correct conversion (1000ms = 1.0s)
- Handles zero
- Handles large values

### 3. Formatting

#### `format_duration`
```rust
pub fn format_duration(millis: u128) -> Result<String, String>
```
Format duration as human-readable string.

**Example**:
```ruchy
let formatted = Time::format_duration(90500);  // "1m 30s"
```

**Test Coverage**:
- Milliseconds: "500ms"
- Seconds: "5s"
- Minutes: "2m 30s"
- Hours: "1h 30m"
- Days: "2d 3h"

#### `parse_duration`
```rust
pub fn parse_duration(duration_str: &str) -> Result<u128, String>
```
Parse human-readable duration string to milliseconds.

**Example**:
```ruchy
let millis = Time::parse_duration("1h 30m");  // 5400000ms
```

**Test Coverage**:
- Parse "500ms"
- Parse "5s"
- Parse "2m 30s"
- Parse "1h 30m"
- Error on invalid format

## Quality Gates

### EXTREME TDD Requirements
1. **RED**: Write tests FIRST (all tests in place before implementation)
2. **GREEN**: Implement minimal code to pass tests
3. **REFACTOR**: Run FAST mutation testing (≥75% coverage)

### Mutation Testing Strategy
```bash
cargo mutants --file src/stdlib/time.rs -- --test std_008_time
```

**Target**: ≥75% mutation coverage
**Expected Runtime**: ~5-10 minutes (following FAST pattern)

### Property Tests (≥3 required)
1. **Monotonic**: `now()` never decreases
2. **Never Panics**: All functions return Result, no panics on any input
3. **Conversion Roundtrip**: `parse_duration(format_duration(x))` ≈ x (within tolerance)

### Test Organization
```
tests/std_008_time.rs
├── Time Measurement Tests (6)
│   ├── now (positive, reasonable range)
│   ├── elapsed_millis (zero, positive, increases)
├── Duration Tests (6)
│   ├── sleep_millis (approximately correct)
│   ├── duration_secs (conversion correct)
├── Formatting Tests (8)
│   ├── format_duration (ms, s, m, h, d)
│   ├── parse_duration (valid, invalid)
└── Property Tests (3)
    ├── Monotonic time
    ├── Never panics
    └── Format/parse roundtrip
```

**Total**: 23 tests (20 unit + 3 property)

## Dependencies

No new dependencies required - uses `std::time::SystemTime` and `std::thread::sleep`.

## Module Structure

```rust
//! Time Operations Module (ruchy/std/time)
//!
//! Thin wrappers around Rust's std::time for time measurement and duration operations.
//!
//! **Design**: Thin wrappers (complexity ≤2 per function) around std::time.
//! **Quality**: 100% unit test coverage, property tests, ≥75% mutation coverage.

use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::thread;

// Time Measurement
pub fn now() -> Result<u128, String> { ... }
pub fn elapsed_millis(start: u128) -> Result<u128, String> { ... }

// Duration Operations
pub fn sleep_millis(millis: u64) -> Result<(), String> { ... }
pub fn duration_secs(millis: u128) -> Result<f64, String> { ... }

// Formatting
pub fn format_duration(millis: u128) -> Result<String, String> { ... }
pub fn parse_duration(duration_str: &str) -> Result<u128, String> { ... }
```

## Success Criteria

- ✅ All 23 tests passing
- ✅ FAST mutation testing runtime <10 minutes
- ✅ ≥75% mutation coverage
- ✅ Complexity ≤2 per function
- ✅ 100% function coverage
- ✅ No panics on invalid input (all errors via Result)

## Timeline Estimate

**Based on STD-001 through STD-007 experience:**
- Tests (RED phase): 1.5 hours
- Implementation (GREEN phase): 1.5 hours
- Mutation testing (REFACTOR): 1 hour
- Documentation: 0.5 hour

**Total**: 4.5 hours (thin wrapper strategy)

## Out of Scope (Future Work)

- Time zones (requires `chrono` crate) - STD-011
- Date formatting (requires `chrono` crate) - STD-011
- High-precision timing (requires platform-specific APIs)
- Clock adjustments and leap seconds

These are deferred to future stdlib expansion after core time operations are validated.
