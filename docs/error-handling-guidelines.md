# Error Handling Guidelines for Ruchy

## Overview
This document outlines best practices for error handling in the Ruchy compiler codebase. Following these guidelines ensures robust, maintainable code with clear error messages.

## Current Status
- **Total unwrap() calls**: ~553 (including tests)
- **Production code unwraps**: 314 (after cleanup)
- **Target**: <300 production unwraps

## Error Handling Patterns

### 1. Use the `?` Operator for Error Propagation
**Preferred** for functions that return `Result<T, E>`:
```rust
// Good
fn process_file(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)?;
    Ok(content)
}

// Avoid
fn process_file(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}
```

### 2. Use `.expect()` with Descriptive Messages
When unwrap is necessary (e.g., mutex locks), use `.expect()` with context:
```rust
// Good
let session = self.session.lock().expect("Failed to acquire session lock");

// Avoid
let session = self.session.lock().unwrap();
```

### 3. Use `.unwrap_or()` / `.unwrap_or_else()` for Defaults
For optional values with sensible defaults:
```rust
// Good
let duration = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_else(|_| Duration::from_secs(0));

// Avoid
let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
```

### 4. Use `.context()` from Anyhow for Better Errors
Add context to errors for better debugging:
```rust
use anyhow::{Context, Result};

// Good
let array = array.as_any()
    .downcast_ref::<Int64Array>()
    .context("Failed to downcast to Int64Array")?;

// Avoid
let array = array.as_any().downcast_ref::<Int64Array>().unwrap();
```

## When Unwrap is Acceptable

### 1. Test Code
Tests can use `.unwrap()` for simplicity:
```rust
#[test]
fn test_parser() {
    let result = parse("42").unwrap();
    assert_eq!(result, Expected::Value);
}
```

### 2. Infallible Operations
When the operation cannot fail by design:
```rust
// OK - regex is compile-time validated
static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d+$").unwrap());
```

### 3. Internal Invariants
When failure indicates a bug in our code:
```rust
// OK with comment explaining why
let token = tokens.get(index)
    .expect("Parser bug: token index out of bounds");
```

## Error Handling by Module

### Parser Module
- Use custom `ParseError` with source location
- Provide helpful error messages with context
- Never panic on invalid input

### Transpiler Module
- Use `anyhow::Result` for flexibility
- Add context at each transformation step
- Preserve source locations for error reporting

### Runtime Module
- Use `RuntimeError` for execution errors
- Distinguish between user errors and internal errors
- Provide stack traces when available

### WASM/Notebook Module
- Convert errors to `JsValue` at boundaries
- Use `.expect()` for lock operations with clear messages
- Handle memory limits gracefully

## Monitoring and Prevention

### Pre-commit Hook
Add to `.git/hooks/pre-commit`:
```bash
#!/bin/bash
./scripts/monitor_unwraps.sh || {
    echo "Error: Unwrap count increased. Please fix before committing."
    exit 1
}
```

### CI Integration
Run monitoring in CI pipeline:
```yaml
- name: Check unwrap usage
  run: ./scripts/monitor_unwraps.sh
```

### Regular Audits
Run quarterly audits to reduce unwrap usage:
```bash
python3 scripts/find_production_unwraps.py
```

## Migration Strategy

### Phase 1: Critical Path (Complete)
- ✅ Lock operations
- ✅ Parse operations
- ✅ Time operations
- ✅ Serialization

### Phase 2: Core Modules (In Progress)
- [ ] Parser error handling
- [ ] Transpiler error handling
- [ ] Runtime error handling

### Phase 3: Features (Future)
- [ ] DataFrame operations
- [ ] Actor system
- [ ] Observatory

## Tools and Scripts

### Monitor Unwraps
```bash
./scripts/monitor_unwraps.sh
```

### Find Production Unwraps
```bash
python3 scripts/find_production_unwraps.py
```

### Auto-fix Common Patterns
```bash
./scripts/fix_unwraps.sh
```

## Resources
- [Rust Error Handling Book](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [Anyhow Documentation](https://docs.rs/anyhow)
- [Error Handling Best Practices](https://nick.groenen.me/posts/rust-error-handling/)