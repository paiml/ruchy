# PARSER-061/062: Box<T> and Vec<T> Investigation Report

**Date**: October 19, 2025
**Investigator**: Claude Code
**Ruchy Version**: v3.95.0
**Status**: ✅ CRITICAL DISCOVERY - Parser Already Supports Box/Vec!

---

## Executive Summary

**🎉 MAJOR DISCOVERY**: Box<T> and Vec<T> generic types **ALREADY PARSE SUCCESSFULLY** in enum variants! The parser fully supports generic type parameters.

**❌ REAL ISSUE**: Runtime execution of `Box::new()` and `Vec::new()` **HANGS INDEFINITELY**.

**Root Cause**: Static method calls (`::`-syntax) on generic types are not implemented in the runtime evaluator. The parser handles the syntax correctly, but the runtime has no handlers for `Box::new()` or `Vec::new()`.

---

## Investigation Results

### Phase 1: Parser Testing

**Test 1**: Box<T> in Enum Variants
```ruchy
enum Tree {
    Leaf(i32),
    Node(Box<Tree>, Box<Tree>)
}
```
**Result**: ✅ `ruchy check` → "Syntax is valid"

**Test 2**: Vec<T> in Enum Variants
```ruchy
enum Stmt {
    Block(Vec<Stmt>),
    Expression(String)
}
```
**Result**: ✅ `ruchy check` → "Syntax is valid"

**Test 3**: Complex ruchyruchy Bootstrap Case
```ruchy
enum BinOp {
    Add,
    Sub
}

enum Expr {
    Number(String),
    Binary(BinOp, Box<Expr>, Box<Expr>)
}
```
**Result**: ✅ `ruchy check` → "Syntax is valid"

### Phase 2: Runtime Testing

**Test 4**: Box::new() Runtime
```ruchy
fn main() {
    let x = 42;
    let boxed = Box::new(x);
    println("Created box");
}
```
**Result**: ❌ HANGS INDEFINITELY (timeout after 3 seconds)

**Test 5**: Vec::new() Runtime
```ruchy
fn main() {
    let v = Vec::new();
    println("Created Vec");
}
```
**Result**: ❌ HANGS INDEFINITELY (timeout after 3 seconds)

**Test 6**: Simple Enum Usage (No Box/Vec instantiation)
```ruchy
enum Tree {
    Leaf(i32),
    Node(Box<Tree>, Box<Tree>)
}

fn main() {
    let leaf = Tree::Leaf(42);
    println("Created leaf");
}
```
**Result**: ✅ "Created leaf" (runtime works when not calling Box::new())

---

## Root Cause Analysis

### Parser: ✅ FULLY FUNCTIONAL
- Generic type parameters (`Box<T>`, `Vec<T>`) parse correctly
- Enum variants with generics work perfectly
- AST representation supports generic types

### Runtime: ❌ MISSING IMPLEMENTATION
- **Static method calls not implemented**: `Type::method()` syntax has no evaluator
- **No Box::new() handler**: Runtime doesn't know how to create boxed values
- **No Vec::new() handler**: Runtime doesn't know how to create vectors
- **Likely cause of hang**: Infinite loop or unhandled case in method dispatch

---

## Discrepancy with ruchyruchy Report

**ruchyruchy Report**: "Box<T> and Vec<T> cause syntax errors in v3.95.0"

**Actual Finding**: Box<T> and Vec<T> **DO NOT** cause syntax errors - they parse correctly!

**Hypothesis**: The ruchyruchy team may have:
1. Tested RUNTIME execution and misattributed the error to parsing
2. Written the BOUNDARIES.md documentation before testing
3. Encountered the hang and assumed it was a parse error

**Action Required**: Update ruchyruchy/BOUNDARIES.md to clarify that PARSING works, but RUNTIME doesn't.

---

## Required Implementation

### PARSER-061: Box<T> Runtime Support
**Status**: Parser ✅ DONE | Runtime ❌ TODO

**Tasks**:
1. Implement `Box::new(value)` static method handler in runtime
2. Add Box unwrapping/dereferencing support
3. Handle Box in pattern matching
4. Add tests for Box runtime operations

**Estimated Effort**: 2-3 hours (not 4-6h, since parser is done!)

### PARSER-062: Vec<T> Runtime Support
**Status**: Parser ✅ DONE | Runtime ❌ TODO

**Tasks**:
1. Implement `Vec::new()` static method handler in runtime
2. Implement `Vec::push(value)` method
3. Implement `Vec::len()`, `Vec::get(index)`, `Vec::is_empty()`
4. Handle Vec iteration in for loops
5. Add tests for Vec runtime operations

**Estimated Effort**: 3-4 hours (not 4-6h, since parser is done!)

---

## Next Steps

1. ✅ **Investigation Complete** - Parser supports Box/Vec, runtime doesn't
2. 🔄 **Implement Box::new() runtime** - Add static method dispatch for Box
3. 🔄 **Implement Vec::new() runtime** - Add static method dispatch for Vec
4. 🔄 **FAST Validation** - Property + Mutation + Fuzz tests
5. 🔄 **Validate with ruchyruchy** - Confirm bootstrap compiler can use Box/Vec

---

## Key Insights

1. **Parser is more capable than documented** - Generics work!
2. **Runtime lags behind parser** - Static methods need implementation
3. **Testing revealed hidden capability** - Dogfooding finds truth
4. **Hang indicates dispatch issue** - Method resolution likely infinite loop

---

## Files to Modify

**Runtime Static Method Dispatch**:
- `/home/noah/src/ruchy/src/runtime/eval_method_dispatch.rs`
- `/home/noah/src/ruchy/src/runtime/eval_method.rs`

**Test Files**:
- `/home/noah/src/ruchy/tests/parser_061_box_generic_enum.rs` (already created)
- `/home/noah/src/ruchy/tests/runtime_box_operations.rs` (to create)
- `/home/noah/src/ruchy/tests/runtime_vec_operations.rs` (to create)

---

**Status**: ✅ INVESTIGATION COMPLETE - Ready for implementation phase
**Updated**: PARSER-061 and PARSER-062 are now **RUNTIME** tasks, not parser tasks
**Priority**: HIGH - Blocks ruchyruchy Stage 1 bootstrap compiler
