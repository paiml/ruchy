# Transpiler Defects - NO DEFECT OUT OF SCOPE

## Critical: These are DEFECTS, not "limitations"

**Toyota Way Principle**: Stop the line when defects are found. Document and fix systematically.

---

## DEFECT-001: String Type Annotations Don't Auto-Convert

**Severity**: HIGH
**Discovered**: 2025-10-07 (LANG-COMP-007 session)

### Problem
```ruchy
let name: String = "Alice"  // ❌ FAILS: "expected String, found &str"
```

### Expected Behavior
When a variable has a `String` type annotation, string literals should automatically be converted via `.to_string()`.

### Current Workaround
```ruchy
let name: String = "Alice".to_string()  // ✅ Manual conversion
let name = "Alice"  // ✅ Type inference (&str)
```

### Root Cause
Transpiler doesn't check type annotations when generating code for string literals.

### Fix Required
In transpiler, when emitting let binding with String type annotation:
1. Check if value is a string literal (&str)
2. If type annotation is String, wrap with `.to_string()`

---

## DEFECT-002: Integer Literal Type Suffixes Not Preserved

**Severity**: HIGH
**Discovered**: 2025-10-07 (LANG-COMP-008 session)

### Problem
```ruchy
let abs_val = (-5i32).abs()  // ❌ FAILS: Type suffix i32 lost in transpilation
```

### Expected Behavior
Integer literals with type suffixes (i32, i64, u32, etc.) should be preserved in generated Rust code.

### Current Workaround
```ruchy
let x: i32 = -5
let abs_val = x.abs()  // ✅ Use typed variable
```

### Root Cause
Transpiler strips or doesn't preserve type suffixes on integer literals.

### Fix Required
In transpiler literal handling:
1. Parse and preserve type suffix from source
2. Emit Rust code with same type suffix

---

## DEFECT-003: .to_string() Not Auto-Called on Method Context

**Severity**: MEDIUM
**Discovered**: 2025-10-07 (LANG-COMP-008 session)

### Problem
```ruchy
let as_string = num.to_string()  // ❌ Method call not generated
// Transpiles to: let as_string = num  // Just the variable!
```

### Expected Behavior
`.to_string()` method calls should be preserved and transpiled correctly.

### Current Workaround
This appears to work in some contexts but not others. Needs investigation.

### Root Cause
Method call transpilation may be context-dependent or have edge cases.

### Fix Required
Investigate method call transpilation logic and ensure `.to_string()` is always preserved.

---

## Testing Protocol for Fixes

**EXTREME TDD Required**:
1. **RED**: Write failing test demonstrating defect
2. **GREEN**: Fix transpiler to pass test
3. **REFACTOR**: Ensure fix doesn't break existing tests
4. **VALIDATE**: Run all LANG-COMP examples to verify no regression

**Validation Command**:
```bash
# Run all LANG-COMP examples
for file in examples/lang_comp/**/*.ruchy; do
    echo "Testing: $file"
    cargo run --bin ruchy -- run "$file" || echo "FAILED: $file"
done
```

---

## Priority Order

1. **DEFECT-001** (String auto-convert) - Blocks idiomatic String usage
2. **DEFECT-002** (Type suffixes) - Blocks integer method calls on literals
3. **DEFECT-003** (.to_string() call) - Needs investigation of scope

---

## Status

- [ ] DEFECT-001: Not fixed (documented 2025-10-07)
- [ ] DEFECT-002: Not fixed (documented 2025-10-07)
- [ ] DEFECT-003: Not fixed (documented 2025-10-07)

**Next Action**: Apply EXTREME TDD to fix DEFECT-001 first (highest impact).
