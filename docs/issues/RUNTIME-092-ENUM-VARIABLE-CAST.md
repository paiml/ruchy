# RUNTIME-092: Complete Enum Variable Cast Support (Issue #79)

**Status**: BLOCKED - Requires structural refactoring
**Priority**: HIGH - Blocks user codebases
**Estimated Effort**: 2-3 hours (large refactoring)
**Related**: GitHub Issue #79, v3.147.3 incomplete fix

---

## Problem Statement

**v3.147.3 PARTIAL FIX**: Only direct enum literal casts work, variable casts still fail

### What Works (v3.147.3)
```ruchy
enum LogLevel {
    Debug = 0,
    Info = 1,
}

fun main() {
    let val = LogLevel::Info as i32;  // ✅ WORKS (direct literal)
    println(val);  // Prints: 1
}
```

### What Still Fails (BLOCKER)
```ruchy
enum LogLevel {
    Debug = 0,
    Info = 1,
}

fun main() {
    let level = LogLevel::Debug;
    let val = level as i32;  // ❌ FAILS
    println(val);
}
```

**Error**: "Cannot cast enum variant Debug to integer: enum type information lost at runtime"

### Original Issue #79 Case (STILL BROKEN)
```ruchy
struct Logger {
    level: LogLevel,
}

impl Logger {
    fun test(&self) {
        let val = self.level as i32;  // ❌ FAILS
        println(val);
    }
}
```

---

## Root Cause Analysis (Five Whys)

**Why does enum variable cast fail?**
→ Runtime rejects `Value::EnumVariant` to integer casts (src/runtime/interpreter.rs:2332-2338)

**Why does runtime reject it?**
→ Code says "enum type information lost at runtime"

**Why is type information lost?**
→ `Value::EnumVariant` struct only stores `variant_name` and `data`, NOT `enum_name`

**Why doesn't it store `enum_name`?**
→ Original design didn't anticipate need for discriminant lookup from runtime values

**Why can't we add it now?**
→ We can! Requires structural refactoring across 10+ files

---

## Technical Solution

### Required Changes

**1. Modify `Value::EnumVariant` struct** (src/runtime/interpreter.rs:98-102)

```rust
// CURRENT (v3.147.3)
EnumVariant {
    variant_name: String,
    data: Option<Vec<Value>>,
}

// REQUIRED (v3.147.4)
EnumVariant {
    enum_name: String,       // NEW: The enum type (e.g., "LogLevel")
    variant_name: String,    // The variant (e.g., "Debug", "Info")
    data: Option<Vec<Value>>,
}
```

**2. Update ALL `EnumVariant` creation sites** (10+ locations)

Files requiring updates:
- `src/runtime/interpreter.rs` (5 locations)
  - Line 1099: `ExprKind::None`
  - Line 1103: `ExprKind::Some`
  - Line 1607: `eval_field_access` (enum variant construction)
  - Line 1865: `Option::None` hardcoded
  - Line 6531: Tuple variant constructor
- `src/runtime/eval_builtin.rs` (3 locations)
- `src/runtime/eval_method_dispatch.rs` (2 locations)
- `src/runtime/bytecode/vm.rs` (1 location)

**3. Update ALL pattern matches on `EnumVariant`**

Use `..` wildcard or add explicit `enum_name` field:
```rust
// OLD
Value::EnumVariant { variant_name, data } => { ... }

// NEW (Option 1 - wildcard)
Value::EnumVariant { variant_name, data, .. } => { ... }

// NEW (Option 2 - explicit)
Value::EnumVariant { enum_name, variant_name, data } => { ... }
```

**4. Fix type cast logic** (src/runtime/interpreter.rs:2332-2348)

```rust
// CURRENT (v3.147.3) - REJECTS variable casts
(Value::EnumVariant { variant_name, .. }, "i32" | "i64" | "isize") => {
    Err(InterpreterError::TypeError(format!(
        "Cannot cast enum variant {} to integer: enum type information lost at runtime",
        variant_name
    )))
}

// REQUIRED (v3.147.4) - PERFORMS discriminant lookup
(Value::EnumVariant { enum_name, variant_name, .. }, "i32" | "i64" | "isize") => {
    // Lookup enum definition in environment
    if let Some(Value::Object(enum_def)) = self.get_variable(&enum_name) {
        if let Some(Value::Object(variants)) = enum_def.get("__variants") {
            if let Some(Value::Object(variant_info)) = variants.get(&variant_name) {
                if let Some(Value::Integer(disc)) = variant_info.get("discriminant") {
                    return Ok(Value::Integer(*disc));
                }
            }
        }
    }
    Err(InterpreterError::TypeError(format!(
        "Cannot cast enum variant {}::{} to integer: enum definition not found",
        enum_name, variant_name
    )))
}
```

**5. Update `eval_field_access`** (src/runtime/interpreter.rs:1598-1610)

Extract enum name from AST when creating variants:
```rust
// CURRENT
if let ExprKind::Identifier(enum_name) = &object.kind {
    // ... but enum_name is NOT stored in EnumVariant!
}

// REQUIRED
if let ExprKind::Identifier(ref enum_name) = object.kind {
    return Ok(Value::EnumVariant {
        enum_name: enum_name.clone(),  // STORE the enum name!
        variant_name: field.to_string(),
        data: None,
    });
}
```

---

## Test Plan (EXTREME TDD)

### RED Phase Tests (Already Added)

File: `tests/regression_079_enum_cast.rs`

**Test #6**: Enum variable cast
```rust
let level = LogLevel::Debug;
let val = level as i32;
println(val);  // Should print: 0
```

**Test #7**: Struct field enum cast (original Issue #79)
```rust
struct Logger { level: LogLevel }
impl Logger {
    fun test(&self) {
        let val = self.level as i32;
        println(val);  // Should print: 1
    }
}
```

**Test #8**: Multiple variable casts
```rust
let debug = LogLevel::Debug;
let info = LogLevel::Info;
let debug_val = debug as i32;
let info_val = info as i32;
println(debug_val);  // Should print: 0
println(info_val);   // Should print: 1
```

### GREEN Phase Validation

1. All 8 enum cast tests pass (tests 1-8 in regression_079_enum_cast.rs)
2. Full test suite passes (4028+ tests, zero regressions)
3. All test executions complete in <5s (no hangs)

### REFACTOR Phase

1. PMAT quality gates: Complexity ≤10, TDG A- minimum
2. No SATD comments introduced
3. All modified functions have doctests

---

## Implementation Checklist

### Phase 1: Struct Definition
- [ ] Modify `Value::EnumVariant` to add `enum_name` field
- [ ] Update `TypeId` implementation for `EnumVariant`
- [ ] Run `cargo build` to identify all compilation errors

### Phase 2: Fix Compilation Errors
- [ ] Update `eval_special_form` (ExprKind::None, ExprKind::Some)
- [ ] Update `eval_field_access` (enum variant construction)
- [ ] Update `lookup_variable` (Option::None hardcoded)
- [ ] Update `call_function` (tuple variant constructor)
- [ ] Update all files in compilation error list

### Phase 3: Pattern Match Updates
- [ ] Search codebase for `Value::EnumVariant` patterns
- [ ] Update each to use `..` wildcard or explicit fields
- [ ] Verify no pattern match exhaustiveness warnings

### Phase 4: Type Cast Logic
- [ ] Replace rejection error with discriminant lookup
- [ ] Handle "enum definition not found" error case
- [ ] Test with Option enum (built-in, no definition)

### Phase 5: Testing
- [ ] Run regression_079_enum_cast tests
- [ ] Verify all 8 tests pass
- [ ] Run full test suite (4028+ tests)
- [ ] Verify zero regressions

### Phase 6: Quality Gates
- [ ] PMAT quality gates pass (complexity ≤10)
- [ ] No SATD comments
- [ ] Doctests added for modified functions

---

## Estimated Complexity

**Cyclomatic Complexity Impact**:
- Type cast logic: +4 (nested if-let chains)
- Total function complexity stays ≤10 (acceptable)

**Lines of Code**:
- Modified: ~50 lines across 5+ files
- Added: ~20 lines (new enum_name handling)
- Removed: ~10 lines (rejection error)
- Net: +60 lines

**Files Modified**: 5-7 files minimum
- interpreter.rs
- eval_builtin.rs
- eval_method_dispatch.rs
- bytecode/vm.rs
- Additional files TBD based on compilation

---

## Risk Assessment

**LOW RISK** - Additive change, well-understood fix

**Mitigation**:
1. Comprehensive test coverage (8 enum cast scenarios)
2. Full test suite validation (4028+ tests)
3. PMAT quality gates enforce complexity limits
4. Property tests can verify enum behavior invariants

---

## Success Criteria

1. ✅ All 8 regression_079_enum_cast tests pass
2. ✅ Full test suite passes (zero regressions)
3. ✅ PMAT quality gates pass (complexity ≤10, TDG A-)
4. ✅ Logger/Common/Schema conversions unblocked (2/11 → 11/11 tests)
5. ✅ v3.147.4 published to crates.io

---

## References

- **GitHub Issue**: #79
- **Bug Report**: ../ubuntu-config-scripts/RUCHY-V3.147.3-TEST-RESULTS.md
- **Test File**: tests/regression_079_enum_cast.rs
- **v3.147.3 Fix**: src/runtime/interpreter.rs:2290-2312 (direct casts only)
- **Blocking Code**: src/runtime/interpreter.rs:2332-2338 (rejection error)

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
