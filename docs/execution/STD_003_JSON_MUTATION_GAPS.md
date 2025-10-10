# STD-003 JSON Mutation Testing - Test Gaps Analysis

## Summary

**Mutation Coverage: 80% (20/25 caught)**
- Status: ✅ PASSES ≥75% target
- 5 mutations MISSED
- Runtime: 8m 21s

## MISSED Mutations

### 1. `as_f64` function (4 MISSED mutations)

**Location**: `src/stdlib/json.rs:234:5`

**Mutations:**
1. Replace `as_f64 -> Option<f64>` with `Some(0.0)` - MISSED
2. Replace `as_f64 -> Option<f64>` with `Some(-1.0)` - MISSED
3. Replace `as_f64 -> Option<f64>` with `None` - MISSED
4. Replace `as_f64 -> Option<f64>` with `Some(1.0)` - MISSED

**Root Cause**: Tests call `as_f64()` but don't validate the actual float value returned.

**Current Test (Insufficient)**:
```rust
#[test]
fn test_std_003_as_f64_some() {
    let json_str = r#"{"value": 3.14}"#;
    let value = ruchy::stdlib::json::parse(json_str).unwrap();
    let result = ruchy::stdlib::json::as_f64(&value, "value");
    assert!(result.is_some(), "as_f64 should return Some for float");
    // ❌ MISSING: assert_eq!(result.unwrap(), 3.14, "Must validate actual value");
}
```

**Fix Required**:
```rust
#[test]
fn test_std_003_as_f64_some() {
    let json_str = r#"{"value": 3.14}"#;
    let value = ruchy::stdlib::json::parse(json_str).unwrap();
    let result = ruchy::stdlib::json::as_f64(&value, "value");
    assert!(result.is_some(), "as_f64 should return Some for float");
    assert_eq!(result.unwrap(), 3.14, "Must return actual float value");  // ✅ ADD THIS
}

#[test]
fn test_std_003_as_f64_none() {
    let json_str = r#"{"value": "not a number"}"#;
    let value = ruchy::stdlib::json::parse(json_str).unwrap();
    let result = ruchy::stdlib::json::as_f64(&value, "value");
    assert!(result.is_none(), "as_f64 should return None for string");  // ✅ ADD THIS
}
```

### 2. `as_bool` function (1 MISSED mutation)

**Location**: `src/stdlib/json.rs:257:5`

**Mutation:**
- Replace `as_bool -> Option<bool>` with `Some(true)` - MISSED

**Root Cause**: Test validates `is_some()` but not the actual boolean value.

**Current Test (Insufficient)**:
```rust
#[test]
fn test_std_003_as_bool_some() {
    let json_str = r#"{"flag": false}"#;
    let value = ruchy::stdlib::json::parse(json_str).unwrap();
    let result = ruchy::stdlib::json::as_bool(&value, "flag");
    assert!(result.is_some(), "as_bool should return Some for bool");
    // ❌ MISSING: assert_eq!(result.unwrap(), false, "Must validate actual bool value");
}
```

**Fix Required**:
```rust
#[test]
fn test_std_003_as_bool_true() {
    let json_str = r#"{"flag": true}"#;
    let value = ruchy::stdlib::json::parse(json_str).unwrap();
    let result = ruchy::stdlib::json::as_bool(&value, "flag");
    assert!(result.is_some(), "as_bool should return Some for bool");
    assert_eq!(result.unwrap(), true, "Must return actual bool value");  // ✅ ADD THIS
}

#[test]
fn test_std_003_as_bool_false() {
    let json_str = r#"{"flag": false}"#;
    let value = ruchy::stdlib::json::parse(json_str).unwrap();
    let result = ruchy::stdlib::json::as_bool(&value, "flag");
    assert!(result.is_some(), "as_bool should return Some for bool");
    assert_eq!(result.unwrap(), false, "Must return actual bool value");  // ✅ ADD THIS
}
```

## Impact

**Current Status**: Module COMPLETE (80% ≥ 75% target)
**Improvement Opportunity**: Add 3 targeted tests to achieve 100% coverage

## Recommendation

**OPTIONAL**: Add the 3 missing tests above to achieve 100% mutation coverage.
- Estimated time: 10 minutes
- Benefit: 80% → 100% coverage
- Priority: LOW (already exceeds target)

**Status**: ACCEPTABLE AS-IS (80% ≥ 75%)

## Lessons Learned

**Pattern**: Always validate actual returned VALUES, not just `is_some()`/`is_ok()`

**Before (Insufficient)**:
```rust
assert!(result.is_some());  // ❌ Only checks presence, not correctness
```

**After (Mutation-Resistant)**:
```rust
assert!(result.is_some());
assert_eq!(result.unwrap(), expected_value);  // ✅ Validates actual value
```

This pattern applies to ALL conversion functions (`as_i64`, `as_f64`, `as_bool`, `as_string`).
