# ADR-007: Standard Library Design Philosophy

## Status

Accepted

## Date

2024-04-15

## Context

Ruchy needs a standard library that:
- Provides common functionality out of the box
- Feels familiar to Python developers
- Leverages Rust's performance
- Supports both interpreted and transpiled modes

## Decision

We implement a **minimal, batteries-included** standard library with:

1. **Core modules**: Always available, no import needed
   - Basic types (int, float, string, bool)
   - Collections (list, dict, set)
   - I/O (print, input, file operations)

2. **Extended modules**: Import required
   - `math`: Mathematical functions
   - `json`: JSON parsing/serialization
   - `http`: HTTP client (async)
   - `csv`: CSV processing
   - `datetime`: Date/time handling

3. **Implementation strategy**:
   - Core: Implemented in Rust, exposed via trait
   - Extended: Mix of Rust and Ruchy

```rust
pub trait StdlibProvider {
    fn get_function(&self, name: &str) -> Option<NativeFunction>;
    fn get_module(&self, name: &str) -> Option<Module>;
}
```

## Consequences

### Positive

- Familiar API for Python developers
- Consistent behavior across execution modes
- Performance-critical code in Rust

### Negative

- Maintenance burden for stdlib
- Feature parity with Python impossible
- Documentation overhead

## References

- Python standard library documentation
- Rust std library design
- Lua minimal stdlib philosophy
