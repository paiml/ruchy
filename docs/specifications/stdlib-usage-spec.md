# Ruchy Standard Library, Built-ins, and Use Cases Specification

## Design Principles

### Architectural Invariants
1. **Single IR, Multiple Projections**: MIR (Mid-level IR) is the sole source of truth. Interpreter, JIT, and transpiler are views over MIR, not separate systems.
2. **Mechanical Transparency**: Every implicit behavior has an explicit desugaring available via `--explain` or `:desugar`.
3. **Performance Contracts**: Optimizations are assertions, not hopes. `#[assert_fused]` guarantees fusion or fails compilation.
4. **Conservative Correctness**: When static analysis is uncertain, overapproximate dependencies and warn explicitly.

### Scope Convergence Strategy
We build one compilation pipeline with multiple entry points, not multiple tools:
```
Source -> AST -> TypedAST -> MIR -> {Interpreter|JIT|Rust}
                              ^
                    Single source of truth
```

## Sub-spec Index

| Sub-spec | Description | Lines |
|----------|-------------|-------|
| [Core Language, REPL, CLI, and WASM](sub/stdlib-usage-core-repl-wasm.md) | Core built-ins, primitive types, collection literals, stdlib architecture, prelude, REPL engine, CLI one-liners, WASM deployment, notebook runtime | ~415 |
| [Data Science, Quality, and Ecosystem](sub/stdlib-usage-datascience-ecosystem.md) | Stream processing, format auto-detection, DataFrame operations, visualization, machine learning, quality enforcement, performance guarantees, ecosystem integration, implementation priority | ~350 |

## Implementation Priority

### Phase 1: Core (Current)
- [x] Parser and type inference
- [x] Tree-walk interpreter
- [x] Basic REPL
- [x] Rust transpilation

### Phase 2: Usability (Q1 2025)
- [ ] DataFrame integration
- [ ] One-liner mode
- [ ] WASM compilation
- [ ] LSP implementation

### Phase 3: Performance (Q2 2025)
- [ ] JIT compilation
- [ ] Incremental compilation
- [ ] Parallel execution
- [ ] GPU kernels

### Phase 4: Ecosystem (Q3 2025)
- [ ] Package manager
- [ ] Notebook runtime
- [ ] Cloud deployment
- [ ] IDE plugins
