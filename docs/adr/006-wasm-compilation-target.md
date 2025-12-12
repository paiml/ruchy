# ADR-006: WebAssembly as Compilation Target

## Status

Accepted

## Date

2024-04-01

## Context

Ruchy needs to run in web browsers for:
- Interactive documentation and playground
- Jupyter-style notebooks in browser
- Educational environments
- Serverless edge computing

Options considered:
1. JavaScript transpilation
2. WebAssembly (Wasm) compilation
3. Interpreter in JavaScript

## Decision

We target **WebAssembly** via Rust's wasm32-unknown-unknown target.

Architecture:
```
Ruchy Source → Parse → AST → Transpile → Rust → wasm-pack → WASM
```

Key design choices:
1. Use `wasm-bindgen` for JavaScript interop
2. Compile full interpreter to WASM (not just transpiled code)
3. Provide JavaScript API for REPL functionality
4. Support both sync and async execution models

## Consequences

### Positive

- Near-native performance in browser
- Single codebase for CLI and web
- Security sandbox (WASM isolation)
- Growing ecosystem support

### Negative

- Larger download size than pure JS
- WASM debugging tools still maturing
- Some Rust features unavailable (threads, filesystem)

### Neutral

- Requires wasm-pack build step

## References

- WebAssembly specification
- wasm-bindgen documentation
- Rust and WebAssembly book
