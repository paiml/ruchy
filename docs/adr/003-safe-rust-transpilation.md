# ADR-003: Safe Rust Code Generation (Zero Unsafe Policy)

## Status

Accepted

## Date

2024-02-15

## Context

Ruchy transpiles to Rust for performance. The transpiler must decide how to handle:
- Mutable global state
- Interior mutability patterns
- Thread safety
- FFI boundaries

Using `unsafe` Rust would provide maximum flexibility but undermines the safety guarantees Ruchy promises.

## Decision

**Zero unsafe code policy**: The Ruchy transpiler MUST NEVER generate unsafe Rust code.

Specific patterns:

```rust
// FORBIDDEN - static mut
static mut COUNTER: i32 = 0;  // ❌

// REQUIRED - thread-safe alternative
use std::sync::LazyLock;
static COUNTER: LazyLock<Mutex<i32>> = LazyLock::new(|| Mutex::new(0));  // ✅

// FORBIDDEN - raw pointers
let ptr: *mut i32 = &mut x;  // ❌

// FORBIDDEN - unsafe blocks
unsafe { std::ptr::read(ptr) }  // ❌
```

Implementation strategy:
1. Mutable globals: `LazyLock<Mutex<T>>` or `LazyLock<RwLock<T>>`
2. Interior mutability: `RefCell<T>` (single-threaded) or `Mutex<T>` (multi-threaded)
3. Shared ownership: `Rc<T>` or `Arc<T>`
4. No FFI generation without explicit user annotation

## Consequences

### Positive

- **Memory safety**: Guaranteed by Rust's type system
- **Thread safety**: All generated code is `Send + Sync` safe
- **Auditability**: Generated code needs no unsafe review
- **User trust**: "Python syntax, Rust safety" promise kept

### Negative

- **Performance ceiling**: Some optimizations require unsafe
- **Flexibility**: Cannot generate optimal FFI bindings
- **Complexity**: Safe patterns sometimes more verbose

### Neutral

- Users wanting unsafe can write Rust directly

## Alternatives Considered

### Allow Limited Unsafe

Rejected:
- Breaks "safe by default" promise
- One unsafe block undermines all safety guarantees
- Audit burden unacceptable

### Runtime-Only Approach

Rejected:
- No transpilation means no Rust performance benefits
- Would make Ruchy "just another Python"

### Safe Subset + Unsafe Escape Hatch

Deferred:
- May add `@unsafe` annotation in future
- Requires explicit user opt-in and audit

## References

- Rust Nomicon: Safe and Unsafe Rust
- GitHub Issue #132: Transpiler generates invalid Rust code
- Cloudflare 2019 incident: unsafe code in production
