# Ruchy REPL QA Bug Report

**Date:** August 16, 2025  
**Version:** Ruchy v0.2.1  
**Platform:** Linux

## Executive Summary

The Ruchy REPL has multiple critical issues that prevent basic functionality from working correctly. The main problems are:
1. Variable declarations don't persist across REPL sessions
2. Functions can't be defined properly due to type inference issues
3. String operations fail due to Rust compilation errors
4. Arrays and structs don't display properly
5. Loop constructs produce compilation errors

## Test Categories & Results

### ✅ Working Features

#### Basic Arithmetic & Operators
- Simple arithmetic: `1 + 2`, `10 - 5`, `3 * 4`, `20 / 4`, `10 % 3` ✓
- Power operator: `2 ** 3` ✓
- Boolean operators: `true && false`, `true || false`, `!true` ✓
- Comparison operators: `1 == 1`, `1 != 2`, `5 > 3`, `3 < 5` ✓
- Conditional expressions: `if true { 1 } else { 2 }` ✓

#### REPL Commands
- `:help` - Shows available commands ✓
- `:clear` - Clears session ✓
- `:history` - Shows session history ✓
- `:quit` - Exits REPL ✓

### ❌ Critical Bugs

#### Bug #1: Variable Declaration Persistence
**Severity:** Critical  
**Description:** Variables declared with `let` don't persist across REPL lines  

**Test Case:**
```ruchy
let x = 10
x  // Error: cannot find value `x` in this scope
```

**Error:**
```
error: expected `;`, found `}`
 --> /tmp/ruchy_repl/ruchy_repl_1.rs:4:18
  |
4 |     let x = 10i64
  |                  ^ help: add `;` here

error[E0425]: cannot find value `x` in this scope
```

**Root Cause:** The transpiler doesn't add semicolons to `let` statements, and each REPL line is compiled in isolation without maintaining context.

---

#### Bug #2: Function Definition Type Issues
**Severity:** Critical  
**Description:** Functions can't be properly defined due to incorrect type inference

**Test Case:**
```ruchy
fun add(a, b) { a + b }
fun greet(name) { "Hello, {name}!" }
```

**Error:**
```
error[E0369]: cannot add `impl std::fmt::Display` to `impl std::fmt::Display`
error[E0277]: `()` doesn't implement `std::fmt::Display`
```

**Root Cause:** 
1. Functions default to `impl std::fmt::Display` parameters which can't be added
2. Functions return `()` instead of the expression value
3. String interpolation doesn't work in function bodies

---

#### Bug #3: String Operations
**Severity:** High  
**Description:** String concatenation and interpolation fail

**Test Case:**
```ruchy
"Hello" + " World"  // Error: cannot add `&str` to `&str`
let name = "Ruchy"
"Hello, {name}!"    // Error: cannot find value `name` in this scope
```

**Error:**
```
error[E0369]: cannot add `&str` to `&str`
help: create an owned `String` from a string reference
```

**Root Cause:** String concatenation generates invalid Rust code, and variable interpolation doesn't work across REPL lines.

---

#### Bug #4: Loop Constructs
**Severity:** High  
**Description:** For and while loops generate compilation errors

**Test Case:**
```ruchy
for i in 1..5 { i }    // Error: mismatched types, expected `()`, found `i64`
while false { "loop" }  // Error: mismatched types, expected `()`, found `&str`
```

**Error:**
```
error[E0308]: mismatched types
help: you might have meant to break the loop with this value
```

**Root Cause:** Loop bodies are expected to return `()` but expressions return values.

---

#### Bug #5: Array and Struct Display
**Severity:** Medium  
**Description:** Arrays and structs can't be displayed in REPL output

**Test Case:**
```ruchy
[1, 2, 3]  // Error: `Vec<i64>` doesn't implement `std::fmt::Display`
struct Point { x: 10, y: 20 }  // Error: Failed to parse input
```

**Error:**
```
error[E0277]: `Vec<i64>` doesn't implement `std::fmt::Display`
```

**Root Cause:** Collections need Debug trait, not Display trait for printing.

---

#### Bug #6: Struct Definition Syntax
**Severity:** Medium  
**Description:** Struct definitions aren't parsed correctly

**Test Case:**
```ruchy
struct Point { x: 10, y: 20 }  // Error: Failed to parse input
```

**Root Cause:** The parser doesn't support struct definitions in REPL mode.

---

### 🔧 Minor Issues

1. **`:exit` command not recognized** - Should be aliased to `:quit`
2. **Unnecessary parentheses warnings** in generated Rust code
3. **No multiline input support** for complex expressions
4. **No tab completion** for commands or variables
5. **Error messages expose internal Rust compilation details** instead of user-friendly Ruchy errors

## Recommendations

### Immediate Fixes Required
1. **Implement REPL context persistence** - Variables and functions should persist across lines
2. **Fix function type inference** - Use proper generic constraints or concrete types
3. **Fix string operations** - Convert to `String` for concatenation
4. **Use Debug trait for display** - Change `println!("{}", result)` to `println!("{:?}", result)`
5. **Add semicolon handling** - Automatically add semicolons to statements

### Enhancement Suggestions
1. Add multiline input support with `...` continuation prompt
2. Implement proper error messages that hide Rust compilation details
3. Add tab completion for better usability
4. Support struct and enum definitions
5. Add `:exit` as alias for `:quit`
6. Implement session save/load functionality

## Test Environment
- Ruchy version: 0.2.1
- Rust version: (inferred) 1.75+
- OS: Linux 6.11.0-26-generic
- Build: Release mode with optimizations

## Conclusion

The REPL is currently in an alpha state with fundamental issues that prevent basic programming constructs from working. The main problem is that each REPL input is compiled in isolation without maintaining state, making it impossible to build up programs interactively. A significant refactoring is needed to maintain compilation context across REPL sessions.