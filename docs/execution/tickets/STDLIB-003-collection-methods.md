# STDLIB-003: Missing Collection Methods

## Summary
Implement missing array/vec methods that don't have direct Rust equivalents.

## Status
- **Priority**: P1 (Common operations needed by examples)
- **Complexity**: Medium
- **Estimated Effort**: 3 hours

## Current State
- Most methods work via Rust stdlib mapping
- Some methods need custom implementation

## Implementation Requirements

### Array Methods to Implement
```rust
// Array/Vec methods
.slice(start: Int, end: Int) -> Vec<T>
.concat(other: Vec<T>) -> Vec<T>  
.flatten() -> Vec<T>
.unique() -> Vec<T>
.join(separator: String) -> String  // For Vec<String>

// String method
.substring(start: Int, end: Int) -> String
```

### Implementation Strategy

#### slice(start, end)
```rust
// Transpiler mapping
vec[start..end].to_vec()
```

#### concat(other)
```rust
// Transpiler mapping
[vec, other].concat()
// or
vec.iter().chain(other.iter()).cloned().collect()
```

#### flatten()
```rust
// Transpiler mapping
vec.into_iter().flatten().collect()
```

#### unique()
```rust
// Transpiler mapping
vec.into_iter().collect::<HashSet<_>>().into_iter().collect()
```

#### join(separator) - for Vec<String>
```rust
// Transpiler mapping
vec.join(separator)
```

#### substring(start, end) - for String
```rust
// Transpiler mapping
s.chars().skip(start).take(end - start).collect::<String>()
```

## Test Cases
```rust
// slice
assert_eq!([1,2,3,4].slice(1,3), [2,3])

// concat  
assert_eq!([1,2].concat([3,4]), [1,2,3,4])

// flatten
assert_eq!([[1,2],[3]].flatten(), [1,2,3])

// unique
assert_eq!([1,2,1,3].unique(), [1,2,3])

// join
assert_eq!(["a","b","c"].join(","), "a,b,c")

// substring
assert_eq!("hello".substring(1,4), "ell")
```

## Files to Modify
- `src/backend/transpiler/expressions.rs` - Add method mappings
- `src/runtime/repl.rs` - Add interpreter implementations
- `tests/stdlib_collection_methods_test.rs` - New test file

## Success Criteria
- All collection methods work in both REPL and transpiled code
- Edge cases handled (empty collections, out of bounds)
- Performance acceptable for common use cases