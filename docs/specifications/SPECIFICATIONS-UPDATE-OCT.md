# Ruchy Language Specification v15.0

*Canonical reference for the Ruchy scripting language that transpiles to Rust*

## Table of Contents

1. Language Overview
2. Grammar Definition
3. Type System
4. Core Language Features
5. Transpilation Architecture
6. Interpreter Specification
7. REPL Specification
8. Standard Library
9. Tooling
10. Quality Requirements
11. Implementation Roadmap

## Sub-spec Index

| Sub-spec | Scope | Link |
|----------|-------|------|
| Language, Grammar, Types, and Transpilation | Sections 1-5: Design philosophy, EBNF grammar, type system, core features (functions, pattern matching, error handling, pipeline, actors, DataFrames), transpilation pipeline and MIR | [sub/specs-update-oct-language-transpilation.md](sub/specs-update-oct-language-transpilation.md) |
| Runtime, Tooling, and Roadmap | Sections 6-11: Tree-walk interpreter, REPL magic commands, completion engine, MCP integration, standard library, LSP, linter, formatter, quality gates, implementation roadmap | [sub/specs-update-oct-runtime-tooling.md](sub/specs-update-oct-runtime-tooling.md) |

## Design Philosophy

Ruchy achieves Python-like ergonomics through mechanical transformation to idiomatic Rust. Core principles:

- **Zero-cost abstractions**: All features compile to efficient Rust
- **DataFrame-first**: Collections default to Polars types
- **Progressive complexity**: Simple code remains simple
- **Type inference**: Explicit types only at module boundaries
- **Direct transpilation**: Source maps 1:1 to Rust constructs

## Execution Modes

```rust
pub enum ExecutionMode {
    Script,       // .ruchy files -> Rust transpilation
    Repl,         // Interactive -> tree-walk interpreter
    Compiled,     // AOT -> native binary via cargo
    OneLiner,     // -e flag -> immediate evaluation
}
```

## Performance Targets

| Metric | Target | Rationale |
|--------|--------|-----------|
| REPL startup | <10ms | Interactive responsiveness |
| REPL response | <15ms | Perceived instant |
| Transpile speed | 100K LoC/s | CI/CD viability |
| Runtime overhead | <5% | vs handwritten Rust |
| Binary size | <5MB | Minimal runtime |

## Implementation Roadmap Summary

- **Phase 0 (Weeks 0-4)**: Foundation -- eliminate SATD, complete parser, 80% coverage, complexity <=10
- **Phase 1 (Weeks 5-8)**: MVP -- AST->syn, type inference, DataFrame literals, function transpilation
- **Phase 2 (Weeks 9-12)**: Interactive -- interpreter, magic commands, pipeline operator
- **Phase 3 (Weeks 13-16)**: MIR -- MIR representation, DataFrame fusion, optimization passes
- **Phase 4 (Weeks 17-24)**: Production -- full type inference, pattern matching, actors, LSP

---

*This specification represents the complete, authoritative definition of the Ruchy language. All implementation must conform to these requirements.*
