# Ruchy Cargo Integration Architecture

## Executive Summary

Ruchy transpiles to idiomatic Rust through `build.rs`, implementing all language features in v1.0. Runtime overhead: 2-5% for safety features, 5-10% for actor systems. Compile-time verification via SMT solvers ensures correctness.

## Core Design Constraints

1. **Zero Cargo Modification**: Stock `cargo` commands work unchanged
2. **Performance Bounds**: Runtime <=5% overhead (sequential), <=10% (concurrent)
3. **Verification Depth**: SMT-provable refinements, 80% mutation coverage
4. **Ecosystem Compatibility**: Any crates.io package directly usable

## Sub-spec Index

| Sub-spec | Description | Lines |
|----------|-------------|-------|
| [Architecture and Transpilation](sub/cargo-integration-architecture-transpilation.md) | Build pipeline, build script, module resolution, transpilation strategy, actor system translation, refinement type implementation, unsafe code verification, semantic equivalence testing | ~380 |
| [Quality, Showcase, and Roadmap](sub/cargo-integration-quality-showcase.md) | Mutation testing strategy, performance validation, complete showcase package, prior art synthesis, performance model, migration path, Rust interop, delivery roadmap, technical risks | ~378 |

## Summary

Ruchy delivers a complete systems scripting language via pragmatic transpilation to Rust. Key achievements:

1. **2-5% overhead** for safety features, **5-10%** for actors -- honest, achievable targets
2. **Compile-time SMT verification** for refinement types -- zero runtime cost when proven
3. **80% mutation coverage** enforced -- high but realistic quality bar
4. **Direct crates.io usage** -- no wrapper generation needed
5. **Semantic equivalence testing** via observational behavior, not AST comparison

The architecture prioritizes correctness over performance, debuggability over cleverness, and Rust idioms over novel constructs. This is systems programming with scripting ergonomics, not scripting with systems performance.
