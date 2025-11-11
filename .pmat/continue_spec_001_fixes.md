# SPEC-001 Three-Mode Validation: Fix Remaining Failures

**Session Context**: Canonical YAML grammar framework completed and committed (b8c7af3d).

## 🚨 STOP THE LINE - 3 Critical Failures to Fix

Three features marked "implemented: true" in grammar.yaml **FAIL in production**:

### **1. SPEC-001-A: lambda_expr (CRITICAL)** ❌❌❌
**Status**: ALL THREE MODES BROKEN
- ❌ Interpreter: FAILS
- ❌ Transpile: FAILS
- ❌ Compile: FAILS

**Test**: `cargo test --test spec_001_three_mode_validation lambda_expr`

**Example**:
```ruchy
fun main() {
    let add = |x: i32, y: i32| -> i32 { x + y }
    let result = add(2, 3)
    println(result.to_string())
}
```

**Fix Required**:
- Add ExprKind::Lambda evaluation in `src/runtime/interpreter.rs`
- Add lambda transpilation to Rust closures in `src/backend/transpiler/expressions.rs`
- Create `tests/runtime_lambda.rs` with 5+ unit tests
- Create `tests/transpiler_lambda.rs` with 5+ transpile tests

---

### **2. SPEC-001-B: const_decl (HIGH)** ✅✅❌
**Status**: RUSTC COMPILATION FAILS
- ✅ Interpreter: WORKS
- ✅ Transpile: WORKS
- ❌ Compile: FAILS (rustc error)

**Test**: `cargo test --test spec_001_three_mode_validation const_decl`

**Example**:
```ruchy
const MAX_SIZE: i32 = 100
fun main() {
    println(MAX_SIZE.to_string())
}
```

**Debug Steps**:
```bash
ruchy transpile /tmp/test_const.ruchy -o /tmp/test_const.rs
cat /tmp/test_const.rs  # See what Rust is generated
rustc --crate-type lib /tmp/test_const.rs  # See exact rustc error
```

**Fix Required**:
- Fix const transpilation in `src/backend/transpiler/statements.rs`
- Ensure idiomatic Rust const syntax

---

### **3. SPEC-001-C: pipeline_expr (HIGH)** ❌✅❌
**Status**: INTERPRETER & RUSTC FAIL
- ❌ Interpreter: FAILS
- ✅ Transpile: WORKS
- ❌ Compile: FAILS (rustc error)

**Test**: `cargo test --test spec_001_three_mode_validation pipeline_expr`

**Example**:
```ruchy
fun double(x: i32) -> i32 { x * 2 }
fun add_one(x: i32) -> i32 { x + 1 }
fun main() {
    let result = 5 >> double >> add_one
    println(result.to_string())
}
```

**Fix Required**:
- Add ExprKind::Pipeline evaluation in `src/runtime/interpreter.rs`
- Fix pipeline transpilation for rustc in `src/backend/transpiler/expressions.rs`
- Create `tests/runtime_pipeline.rs` with pipeline tests

---

## 📋 EXTREME TDD Workflow (MANDATORY)

**For Each Ticket**:
```
🔴 RED:    Verify test FAILS (already done)
🟢 GREEN:  Minimal implementation to make test pass
🔵 REFACTOR: Apply PMAT quality gates (≤10 complexity, A- grade)
✅ VALIDATE: Rerun three-mode test - must PASS all three modes
```

**Validation Command**:
```bash
# After each fix, verify test passes
cargo test --test spec_001_three_mode_validation <feature>

# Final validation (must be 19/19)
cargo test --test spec_001_three_mode_validation

# Update grammar.yaml when feature works in ALL THREE MODES
```

---

## 🎯 Success Criteria

**Before marking ticket complete**:
- ✅ Test passes in ALL THREE MODES (interpreter, transpile, compile)
- ✅ PMAT quality gates pass (≤10 complexity, A- grade)
- ✅ Zero regressions in existing tests
- ✅ grammar.yaml updated with `implemented: true` and mode details

**Final Goal**: 19/19 tests passing (100%), Grade A

---

## 📊 Current Status

**Commit**: b8c7af3d  
**Test Results**: 16/19 passing (84.2%)  
**Failures**: lambda_expr, const_decl, pipeline_expr  
**Grade**: B (needs 100% for A)

**Files to Modify**:
- `src/runtime/interpreter.rs` (lambda, pipeline evaluation)
- `src/backend/transpiler/expressions.rs` (lambda, pipeline transpilation)
- `src/backend/transpiler/statements.rs` (const transpilation fix)
- `grammar/ruchy-grammar.yaml` (update when features work)

**New Test Files to Create**:
- `tests/runtime_lambda.rs`
- `tests/transpiler_lambda.rs`
- `tests/runtime_pipeline.rs`

---

**Start with SPEC-001-B (const_decl)** - likely simplest fix, builds confidence.

**Toyota Way**: Stop the Line → Fix Root Cause → Prevent Recurrence
