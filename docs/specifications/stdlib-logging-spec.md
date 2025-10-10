# STD-009: Logging Module Specification

**Status**: DRAFT
**Module**: `ruchy/std/logging` → `src/stdlib/logging.rs`
**Test Suite**: `tests/std_009_logging.rs`
**Wrapper Crate**: `log` v0.4 + `env_logger` v0.11 (proven Rust ecosystem crates)

---

## Design Philosophy

**Thin Wrapper Strategy**: Wrap existing Rust `log` crate with simple function-based API. No macros (those come later in language features). Focus on basic logging operations that can be tested and validated.

**Quality Gates**:
- Complexity ≤2 per function (thin wrappers only)
- 100% unit test coverage
- ≥20 property test cases per module
- ≥75% mutation coverage (BLOCKING)
- All public functions have runnable doctests

---

## Module Functions

### Core Logging Functions (8 total)

```rust
// 1. Initialize logger with level
pub fn init_logger(level: &str) -> Result<(), String>
// Example: init_logger("info") sets global log level

// 2. Log info message
pub fn log_info(message: &str) -> Result<(), String>
// Example: log_info("Server started")

// 3. Log warning message
pub fn log_warn(message: &str) -> Result<(), String>
// Example: log_warn("Low memory")

// 4. Log error message
pub fn log_error(message: &str) -> Result<(), String>
// Example: log_error("Connection failed")

// 5. Log debug message
pub fn log_debug(message: &str) -> Result<(), String>
// Example: log_debug("Variable x = 42")

// 6. Log trace message
pub fn log_trace(message: &str) -> Result<(), String>
// Example: log_trace("Entering function foo")

// 7. Get current log level
pub fn get_level() -> Result<String, String>
// Returns: "trace", "debug", "info", "warn", "error", "off"

// 8. Check if level is enabled
pub fn is_level_enabled(level: &str) -> Result<bool, String>
// Example: is_level_enabled("debug") -> true/false
```

---

## Test Plan (21 unit + 3 property = 24 tests)

### Unit Tests (21 tests)

**Logger Initialization (4 tests)**:
- `test_std_009_init_logger_info` - Initialize with info level
- `test_std_009_init_logger_debug` - Initialize with debug level
- `test_std_009_init_logger_invalid` - Invalid level returns error
- `test_std_009_init_logger_off` - Initialize with off level

**Logging Functions (10 tests)**:
- `test_std_009_log_info_basic` - Log info message succeeds
- `test_std_009_log_warn_basic` - Log warning succeeds
- `test_std_009_log_error_basic` - Log error succeeds
- `test_std_009_log_debug_basic` - Log debug succeeds
- `test_std_009_log_trace_basic` - Log trace succeeds
- `test_std_009_log_info_empty` - Empty message works
- `test_std_009_log_info_long` - Long message works
- `test_std_009_log_info_special_chars` - Special characters work
- `test_std_009_log_info_unicode` - Unicode works
- `test_std_009_log_info_newlines` - Newlines work

**Level Checking (7 tests)**:
- `test_std_009_get_level_info` - Get level returns correct value
- `test_std_009_get_level_debug` - Get level after debug init
- `test_std_009_get_level_off` - Get level when off
- `test_std_009_is_level_enabled_true` - Check enabled level returns true
- `test_std_009_is_level_enabled_false` - Check disabled level returns false
- `test_std_009_is_level_enabled_invalid` - Invalid level returns error
- `test_std_009_is_level_enabled_trace` - Trace level check

### Property Tests (3 tests, 20 cases each)

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        #[test]
        fn test_std_009_logging_never_panics(message: String) {
            // Property: Logging functions never panic on any input
            let _ = log_info(&message);
            let _ = log_warn(&message);
            let _ = log_error(&message);
            let _ = log_debug(&message);
            let _ = log_trace(&message);
        }

        #[test]
        fn test_std_009_level_check_consistent(level in prop::sample::select(vec!["trace", "debug", "info", "warn", "error"])) {
            // Property: If level is enabled, logging at that level succeeds
            init_logger(&level).unwrap();
            let enabled = is_level_enabled(&level).unwrap();
            prop_assert!(enabled, "Level should be enabled after init");
        }

        #[test]
        fn test_std_009_invalid_level_fails(level: String) {
            // Property: Invalid levels always return errors
            let valid_levels = vec!["trace", "debug", "info", "warn", "error", "off"];
            if !valid_levels.contains(&level.as_str()) {
                prop_assert!(init_logger(&level).is_err(), "Invalid level should fail");
            }
        }
    }
}
```

---

## Quality Gates

### Mandatory Requirements

1. **Unit Tests**: 21 tests, 100% coverage
2. **Property Tests**: 3 tests with 20 cases each (60 total validations)
3. **Mutation Coverage**: ≥75% (BLOCKING - sprint incomplete without this)
4. **Complexity**: ≤2 per function (enforced by PMAT)
5. **Doctests**: Every public function has runnable examples

### PMAT Enforcement

```bash
# Pre-commit checks (BLOCKING)
pmat tdg src/stdlib/logging.rs --min-grade A- --fail-on-violation
pmat analyze complexity --max-cyclomatic 2 --file src/stdlib/logging.rs
pmat analyze satd --fail-on-violation --file src/stdlib/logging.rs
```

---

## Mutation Testing Strategy

### FAST Testing (5-10 minutes per module)

```bash
cargo mutants --file src/stdlib/logging.rs -- --test std_009_logging
```

**Expected**:
- Mutants: ~15-20 (simple wrapper functions)
- Runtime: ~7 minutes (based on STD-002 http pattern)
- Target: ≥75% caught (12+/15 minimum)

### Gap Analysis

If mutation coverage < 75%:
1. Document each MISSED mutation in `STD_009_LOGGING_MUTATION_GAPS.md`
2. Write targeted tests for specific mutations
3. Re-run mutation tests to validate fixes
4. Only acceptable if semantically equivalent (document why)

---

## Implementation Timeline

| Phase | Task | Time Estimate |
|-------|------|---------------|
| RED | Write 24 tests FIRST (EXTREME TDD) | 1.5h |
| GREEN | Implement 8 wrapper functions | 1h |
| REFACTOR | FAST mutation testing + gap fixes | 1.5h |
| DOCUMENT | Update roadmap, create gap analysis | 0.5h |
| **TOTAL** | | **4.5h** |

---

## Success Criteria

- ✅ All 24 tests passing (21 unit + 3 property)
- ✅ ≥75% mutation coverage achieved
- ✅ Complexity ≤2 per function (PMAT verified)
- ✅ TDG grade A- minimum (≥85 points)
- ✅ Zero SATD (no TODO/FIXME comments)
- ✅ Doctests in all 8 public functions
- ✅ Committed to git with `[STD-009]` tag
- ✅ Roadmap updated with completion status

---

## Notes

**Toyota Way Principles**:
- **Jidoka**: Stop the line if mutation coverage < 75%
- **Genchi Genbutsu**: Examine actual `log` crate usage patterns first
- **Kaizen**: Each mutation gap is an opportunity to improve tests

**Design Decisions**:
- **No macros**: Use simple functions, not `log!` macros (language feature for later)
- **Simple API**: Just message strings, no structured logging (keep complexity low)
- **Proven crate**: `log` is the standard Rust logging facade
- **Testability**: All functions return Result for consistent error handling
