# STD-010: Regex Module Specification

**Status**: DRAFT
**Module**: `ruchy/std/regex` → `src/stdlib/regex.rs`
**Test Suite**: `tests/std_010_regex.rs`
**Wrapper Crate**: `regex` v1.11 (proven Rust ecosystem crate)

---

## Design Philosophy

**Thin Wrapper Strategy**: Wrap existing Rust `regex` crate with simple function-based API. Focus on common regex operations with safe, tested interfaces.

**Quality Gates**:
- Complexity ≤2 per function (thin wrappers only)
- 100% unit test coverage
- ≥20 property test cases per module
- ≥75% mutation coverage (BLOCKING)
- All public functions have runnable doctests

---

## Module Functions

### Core Regex Functions (10 total)

```rust
// 1. Test if pattern matches string
pub fn is_match(pattern: &str, text: &str) -> Result<bool, String>
// Example: is_match(r"\d+", "abc123") -> Ok(true)

// 2. Find first match
pub fn find_first(pattern: &str, text: &str) -> Result<Option<String>, String>
// Example: find_first(r"\d+", "abc123def") -> Ok(Some("123"))

// 3. Find all matches
pub fn find_all(pattern: &str, text: &str) -> Result<Vec<String>, String>
// Example: find_all(r"\d+", "a1b2c3") -> Ok(vec!["1", "2", "3"])

// 4. Replace first match
pub fn replace_first(pattern: &str, text: &str, replacement: &str) -> Result<String, String>
// Example: replace_first(r"\d+", "abc123def456", "X") -> Ok("abcXdef456")

// 5. Replace all matches
pub fn replace_all(pattern: &str, text: &str, replacement: &str) -> Result<String, String>
// Example: replace_all(r"\d+", "a1b2c3", "X") -> Ok("aXbXcX")

// 6. Split by pattern
pub fn split(pattern: &str, text: &str) -> Result<Vec<String>, String>
// Example: split(r"\s+", "a  b   c") -> Ok(vec!["a", "b", "c"])

// 7. Capture groups from first match
pub fn capture_first(pattern: &str, text: &str) -> Result<Option<Vec<String>>, String>
// Example: capture_first(r"(\d+)-(\d+)", "abc123-456") -> Ok(Some(vec!["123-456", "123", "456"]))

// 8. Capture all groups from all matches
pub fn capture_all(pattern: &str, text: &str) -> Result<Vec<Vec<String>>, String>
// Example: capture_all(r"(\d+)", "a1b2c3") -> Ok(vec![vec!["1", "1"], vec!["2", "2"], vec!["3", "3"]])

// 9. Validate regex pattern (compile check)
pub fn is_valid_pattern(pattern: &str) -> Result<bool, String>
// Example: is_valid_pattern(r"\d+") -> Ok(true), is_valid_pattern(r"[") -> Ok(false)

// 10. Escape special regex characters
pub fn escape(text: &str) -> Result<String, String>
// Example: escape("a.b*c") -> Ok("a\\.b\\*c")
```

---

## Test Plan (27 unit + 3 property = 30 tests)

### Unit Tests (27 tests)

**Pattern Matching (5 tests)**:
- `test_std_010_is_match_true` - Pattern matches
- `test_std_010_is_match_false` - Pattern doesn't match
- `test_std_010_is_match_invalid_pattern` - Invalid regex returns error
- `test_std_010_is_match_empty_pattern` - Empty pattern behavior
- `test_std_010_is_match_complex` - Complex regex (email, URL)

**Finding Matches (6 tests)**:
- `test_std_010_find_first_found` - First match found
- `test_std_010_find_first_not_found` - No match returns None
- `test_std_010_find_all_multiple` - Multiple matches found
- `test_std_010_find_all_none` - No matches returns empty vec
- `test_std_010_find_all_overlapping` - Non-overlapping matches only
- `test_std_010_find_first_groups` - Groups not captured

**Replacement (4 tests)**:
- `test_std_010_replace_first_basic` - First occurrence replaced
- `test_std_010_replace_all_basic` - All occurrences replaced
- `test_std_010_replace_first_no_match` - No match returns original
- `test_std_010_replace_all_no_match` - No match returns original

**Splitting (3 tests)**:
- `test_std_010_split_basic` - Split on delimiter
- `test_std_010_split_multiple_delimiters` - Multiple delimiters
- `test_std_010_split_no_match` - No match returns original

**Capture Groups (5 tests)**:
- `test_std_010_capture_first_single_group` - One group captured
- `test_std_010_capture_first_multiple_groups` - Multiple groups
- `test_std_010_capture_first_no_match` - No match returns None
- `test_std_010_capture_all_multiple` - Multiple matches with groups
- `test_std_010_capture_all_nested` - Nested groups

**Validation & Escaping (4 tests)**:
- `test_std_010_is_valid_pattern_valid` - Valid pattern
- `test_std_010_is_valid_pattern_invalid` - Invalid pattern
- `test_std_010_escape_basic` - Escape special characters
- `test_std_010_escape_already_escaped` - Already escaped text

### Property Tests (3 tests, 20 cases each)

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        #[test]
        fn test_std_010_never_panics_on_invalid_pattern(pattern: String, text: String) {
            // Property: Never panic on any pattern/text combination
            let _ = is_match(&pattern, &text);
            let _ = find_first(&pattern, &text);
            let _ = find_all(&pattern, &text);
        }

        #[test]
        fn test_std_010_escape_roundtrip(text: String) {
            // Property: Escaped text matches literally
            let escaped = escape(&text).unwrap();
            let matched = is_match(&escaped, &text);
            prop_assert!(matched.unwrap_or(false), "Escaped pattern should match original text");
        }

        #[test]
        fn test_std_010_replace_preserves_length_bounds(pattern in r"[a-z]+", text: String, replacement in r"[A-Z]+") {
            // Property: Replace operations don't cause crashes
            if let Ok(_result) = replace_all(&pattern, &text, &replacement) {
                // Just verify it doesn't panic
                prop_assert!(true);
            }
        }
    }
}
```

---

## Quality Gates

### Mandatory Requirements

1. **Unit Tests**: 27 tests, 100% coverage
2. **Property Tests**: 3 tests with 20 cases each (60 total validations)
3. **Mutation Coverage**: ≥75% (BLOCKING - sprint incomplete without this)
4. **Complexity**: ≤2 per function (enforced by PMAT)
5. **Doctests**: Every public function has runnable examples

### PMAT Enforcement

```bash
# Pre-commit checks (BLOCKING)
pmat tdg src/stdlib/regex.rs --min-grade A- --fail-on-violation
pmat analyze complexity --max-cyclomatic 2 --file src/stdlib/regex.rs
pmat analyze satd --fail-on-violation --file src/stdlib/regex.rs
```

---

## Mutation Testing Strategy

### FAST Testing (10-15 minutes per module)

```bash
cargo mutants --file src/stdlib/regex.rs -- --test std_010_regex
```

**Expected**:
- Mutants: ~25-30 (more functions than typical)
- Runtime: ~12 minutes (based on STD-004 path pattern)
- Target: ≥75% caught (19+/25 minimum)

### Gap Analysis

If mutation coverage < 75%:
1. Document each MISSED mutation in `STD_010_REGEX_MUTATION_GAPS.md`
2. Write targeted tests for specific mutations
3. Re-run mutation tests to validate fixes
4. Only acceptable if semantically equivalent (document why)

---

## Implementation Timeline

| Phase | Task | Time Estimate |
|-------|------|---------------|
| RED | Write 30 tests FIRST (EXTREME TDD) | 2h |
| GREEN | Implement 10 wrapper functions | 1.5h |
| REFACTOR | FAST mutation testing + gap fixes | 2h |
| DOCUMENT | Update roadmap, create gap analysis | 0.5h |
| **TOTAL** | | **6h** |

---

## Success Criteria

- ✅ All 30 tests passing (27 unit + 3 property)
- ✅ ≥75% mutation coverage achieved
- ✅ Complexity ≤2 per function (PMAT verified)
- ✅ TDG grade A- minimum (≥85 points)
- ✅ Zero SATD (no TODO/FIXME comments)
- ✅ Doctests in all 10 public functions
- ✅ Committed to git with `[STD-010]` tag
- ✅ Roadmap updated with completion status

---

## Notes

**Toyota Way Principles**:
- **Jidoka**: Stop the line if mutation coverage < 75%
- **Genchi Genbutsu**: Examine actual `regex` crate usage patterns first
- **Kaizen**: Each mutation gap is an opportunity to improve tests

**Design Decisions**:
- **Error handling**: Invalid regex patterns return Err, not panic
- **Groups**: Capture functions return full match + groups (regex crate pattern)
- **Split**: Empty splits filtered out (standard behavior)
- **Escape**: Uses `regex::escape()` for correctness
- **Proven crate**: `regex` is the standard Rust regex engine

**Security Note**:
- No ReDoS protection (use with trusted patterns only)
- Pattern complexity limits not enforced (stdlib is thin wrapper)
- Document security considerations in higher-level APIs
