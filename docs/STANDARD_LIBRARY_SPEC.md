# Ruchy Standard Library Specification

## Overview

This document defines the standard library methods and functions that Ruchy must provide for production readiness. It serves as both a specification and validation checklist.

## Status Key
- ✅ Implemented in both interpreter AND transpiler
- 🟡 Implemented in interpreter ONLY (not available in transpiled code)
- ❌ Not implemented

---

## 1. String Methods

| Method | Signature | Status | Example | Notes |
|--------|-----------|--------|---------|-------|
| `len()` | `() -> i32` | 🟡 | `"hello".len() // 5` | Works in REPL only |
| `to_string()` | `() -> String` | 🟡 | `"hello".to_string()` | Identity for strings |
| `to_upper()` | `() -> String` | 🟡 | `"hello".to_upper() // "HELLO"` | |
| `to_lower()` | `() -> String` | 🟡 | `"HELLO".to_lower() // "hello"` | |
| `trim()` | `() -> String` | 🟡 | `"  hello  ".trim() // "hello"` | |
| `contains()` | `(String) -> bool` | 🟡 | `"hello".contains("ll") // true` | |
| `starts_with()` | `(String) -> bool` | 🟡 | `"hello".starts_with("he") // true` | |
| `ends_with()` | `(String) -> bool` | 🟡 | `"hello".ends_with("lo") // true` | |
| `split()` | `(String) -> [String]` | 🟡 | `"a,b,c".split(",") // ["a","b","c"]` | |
| `replace()` | `(String, String) -> String` | ❌ | `"hello".replace("l", "r")` | |
| `slice()` | `(i32, i32) -> String` | ❌ | `"hello".slice(1, 3) // "el"` | |
| `chars()` | `() -> [char]` | ❌ | `"hello".chars()` | |
| `repeat()` | `(i32) -> String` | ❌ | `"ab".repeat(3) // "ababab"` | |

## 2. Array/List Methods

| Method | Signature | Status | Example | Notes |
|--------|-----------|--------|---------|-------|
| `len()` | `() -> i32` | 🟡 | `[1,2,3].len() // 3` | |
| `push()` | `(T) -> [T]` | 🟡 | `[1,2].push(3) // [1,2,3]` | |
| `pop()` | `() -> [T]` | 🟡 | `[1,2,3].pop() // [1,2]` | |
| `first()` | `() -> Option<T>` | 🟡 | `[1,2,3].first() // 1` | |
| `last()` | `() -> Option<T>` | 🟡 | `[1,2,3].last() // 3` | |
| `get()` | `(i32) -> Option<T>` | 🟡 | `[1,2,3].get(1) // 2` | |
| `map()` | `(T -> U) -> [U]` | 🟡 | `[1,2].map(\|x\| x*2) // [2,4]` | |
| `filter()` | `(T -> bool) -> [T]` | 🟡 | `[1,2,3].filter(\|x\| x>1) // [2,3]` | |
| `reduce()` | `((T,T) -> T, T) -> T` | 🟡 | `[1,2,3].reduce(\|a,b\| a+b, 0) // 6` | |
| `find()` | `(T -> bool) -> Option<T>` | 🟡 | `[1,2,3].find(\|x\| x>1) // 2` | |
| `any()` | `(T -> bool) -> bool` | 🟡 | `[1,2,3].any(\|x\| x>2) // true` | |
| `all()` | `(T -> bool) -> bool` | 🟡 | `[1,2,3].all(\|x\| x>0) // true` | |
| `sort()` | `() -> [T]` | ❌ | `[3,1,2].sort() // [1,2,3]` | |
| `reverse()` | `() -> [T]` | ❌ | `[1,2,3].reverse() // [3,2,1]` | |
| `join()` | `(String) -> String` | ❌ | `["a","b"].join(",") // "a,b"` | |
| `slice()` | `(i32, i32) -> [T]` | ❌ | `[1,2,3,4].slice(1,3) // [2,3]` | |
| `concat()` | `([T]) -> [T]` | ❌ | `[1,2].concat([3,4]) // [1,2,3,4]` | |
| `flatten()` | `() -> [T]` | ❌ | `[[1,2],[3]].flatten() // [1,2,3]` | |
| `unique()` | `() -> [T]` | ❌ | `[1,2,1,3].unique() // [1,2,3]` | |

## 3. Number Methods

### Integer Methods
| Method | Signature | Status | Example | Notes |
|--------|-----------|--------|---------|-------|
| `to_string()` | `() -> String` | 🟡 | `42.to_string() // "42"` | |
| `abs()` | `() -> i32` | 🟡 | `(-5).abs() // 5` | |
| `pow()` | `(i32) -> i32` | 🟡 | `2.pow(3) // 8` | |
| `sqrt()` | `() -> f64` | 🟡 | `16.sqrt() // 4.0` | |
| `min()` | `(i32) -> i32` | 🟡 | `5.min(3) // 3` | |
| `max()` | `(i32) -> i32` | 🟡 | `5.max(3) // 5` | |

### Float Methods
| Method | Signature | Status | Example | Notes |
|--------|-----------|--------|---------|-------|
| `to_string()` | `() -> String` | 🟡 | `3.14.to_string()` | |
| `abs()` | `() -> f64` | 🟡 | `(-3.14).abs() // 3.14` | |
| `floor()` | `() -> f64` | 🟡 | `3.7.floor() // 3.0` | |
| `ceil()` | `() -> f64` | 🟡 | `3.2.ceil() // 4.0` | |
| `round()` | `() -> f64` | 🟡 | `3.5.round() // 4.0` | |
| `sqrt()` | `() -> f64` | 🟡 | `9.0.sqrt() // 3.0` | |

## 4. Object/HashMap Methods

| Method | Signature | Status | Example | Notes |
|--------|-----------|--------|---------|-------|
| `keys()` | `() -> [String]` | ✅ | `{"a":1}.keys() // ["a"]` | Fixed in v1.20.1 |
| `values()` | `() -> [T]` | ✅ | `{"a":1}.values() // [1]` | Fixed in v1.20.1 |
| `items()` | `() -> [(String,T)]` | ✅ | `{"a":1}.items()` | Fixed in v1.20.1 |
| `len()` | `() -> i32` | 🟡 | `{"a":1,"b":2}.len() // 2` | |
| `get()` | `(String) -> Option<T>` | 🟡 | `{"a":1}.get("a") // 1` | |
| `contains_key()` | `(String) -> bool` | ❌ | `{"a":1}.contains_key("a")` | |
| `insert()` | `(String, T) -> Object` | ❌ | `{}.insert("a", 1)` | |
| `remove()` | `(String) -> Option<T>` | ❌ | `{"a":1}.remove("a")` | |
| `merge()` | `(Object) -> Object` | ❌ | `{"a":1}.merge({"b":2})` | |

## 5. File I/O (Not Implemented)

| Function | Signature | Status | Example |
|----------|-----------|--------|---------|
| `File::open()` | `(String) -> Result<File>` | ❌ | `File::open("data.txt")` |
| `File::create()` | `(String) -> Result<File>` | ❌ | `File::create("out.txt")` |
| `file.read()` | `() -> Result<String>` | ❌ | `file.read()` |
| `file.write()` | `(String) -> Result<()>` | ❌ | `file.write("data")` |
| `file.lines()` | `() -> [String]` | ❌ | `file.lines()` |

## 6. Global Functions

| Function | Signature | Status | Example |
|----------|-----------|--------|---------|
| `print()` | `(String) -> ()` | ✅ | `print("hello")` |
| `println()` | `(String) -> ()` | ✅ | `println("hello")` |
| `format()` | `(String, ...) -> String` | 🟡 | `format("{}:{}", a, b)` |
| `panic()` | `(String) -> !` | ✅ | `panic("error")` |
| `assert()` | `(bool, String) -> ()` | ✅ | `assert(x > 0, "msg")` |
| `type_of()` | `(T) -> String` | ❌ | `type_of(42) // "i32"` |
| `parse()` | `(String) -> Result<T>` | ❌ | `parse::<i32>("42")` |

## 7. Math Functions (Global)

| Function | Signature | Status | Example |
|----------|-----------|--------|---------|
| `sin()` | `(f64) -> f64` | ❌ | `sin(3.14159/2)` |
| `cos()` | `(f64) -> f64` | ❌ | `cos(0.0)` |
| `tan()` | `(f64) -> f64` | ❌ | `tan(3.14159/4)` |
| `log()` | `(f64) -> f64` | ❌ | `log(2.718)` |
| `log10()` | `(f64) -> f64` | ❌ | `log10(100)` |
| `random()` | `() -> f64` | ❌ | `random() // 0.0-1.0` |

## Summary Statistics

### Current Implementation Status
- **Total Methods Specified**: 71
- **Fully Implemented (✅)**: 7 (10%)
- **Interpreter Only (🟡)**: 35 (49%)
- **Not Implemented (❌)**: 29 (41%)

### Critical Gaps
1. **Transpiler Support**: 35 methods work in REPL but NOT in compiled code
2. **File I/O**: Completely missing
3. **String Operations**: Many common operations missing (replace, slice, chars)
4. **Array Operations**: Missing sort, reverse, join, slice, concat
5. **Math Functions**: No trigonometric or logarithmic functions

### Priority for Implementation
1. **P0 - Critical**: Make existing interpreter methods work in transpiler (35 methods)
2. **P1 - Important**: String operations (replace, slice, chars)
3. **P1 - Important**: Array operations (sort, reverse, join, slice)
4. **P2 - Nice to have**: File I/O
5. **P3 - Future**: Advanced math functions

## Testing Requirements

Every standard library method must have:
1. Unit test in `tests/stdlib_methods_test.rs`
2. Integration test showing REPL and transpiler consistency
3. Property test for mathematical invariants
4. Documentation with examples
5. Performance benchmark

## Implementation Strategy

### Phase 1: Transpiler Parity (Week 1)
Make all 35 interpreter-only methods work in transpiler

### Phase 2: Core String/Array (Week 2)
Implement missing essential string and array operations

### Phase 3: File I/O (Week 3)
Add basic file reading and writing capabilities

### Phase 4: Math Library (Week 4)
Add trigonometric and logarithmic functions

## Validation Test Suite

```rust
// Every method needs a test like this:
#[test]
fn validate_string_len() {
    // Test in interpreter
    assert_eq!(eval_interpreter(r#""hello".len()"#), "5");
    
    // Test in transpiler
    assert_eq!(eval_transpiled(r#""hello".len()"#), "5");
    
    // Test edge cases
    assert_eq!(eval_both(r#""".len()"#), "0");
    assert_eq!(eval_both(r#""🦀".len()"#), "4"); // UTF-8 bytes
}
```