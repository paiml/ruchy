# Ruchy Library Publishing Specification

## Abstract

This document specifies the publishing pipeline for Ruchy libraries to crates.io, enabling zero-friction consumption by standard Rust projects without Ruchy toolchain requirements.

The specification covers the dual-artifact strategy (Ruchy sources + pre-transpiled Rust), bidirectional transpilation architecture, Rust package import integration, and the complete publishing workflow including tooling, security, and ecosystem integration.

---

## Sub-spec Index

| Sub-spec | Sections | Description | Lines |
|----------|----------|-------------|-------|
| [Core Model and Pipeline](sub/publish-core-model-pipeline.md) | 1-6 | Dual-artifact strategy, build script architecture, manifest configuration, publishing pipeline, package optimization, bidirectional transpilation | 313 |
| [Rust Package Import and Integration](sub/publish-rust-import-integration.md) | 7-10 | Type system bridge, lifetime inference, macro transparency, trait implementation, zero-cost abstraction, API discovery, consumer integration patterns, version resolution, source maps, source synchronization | 443 |
| [Workflow, Tooling, and Ecosystem](sub/publish-workflow-tooling-ecosystem.md) | 11-18 | Publishing checklist, performance guarantees, security model, cargo-ruchy subcommand, migration path, pure-Rust contribution workflow, build infrastructure, troubleshooting, ecosystem integration summary, risks, experimental features | 480 |

---

## Key Architectural Decisions

1. **Dual-Artifact Strategy**: Published crates contain both `.ruchy` sources and pre-transpiled `.rs` code, enabling consumption without Ruchy toolchain
2. **Bidirectional Transpilation**: Canonical patterns enable forward (Ruchy to Rust) and reverse (Rust to Ruchy) transpilation with semantic preservation
3. **Zero-Cost Interop**: All Rust crate interop resolves at compile time with no runtime overhead
4. **Feature-Based Control**: `pregenerated` vs `from-source` features control build behavior

## Performance Invariants

- Transpilation overhead: <15% compile time
- Runtime overhead: 0% (zero-cost abstraction)
- Binary size increase: <5%
- Package size increase: <30%

## Compatibility Matrix

| Ruchy | Min Rust | Transpiler Features |
|-------|----------|-------------------|
| 1.0   | 1.70     | Basic, async/await |
| 1.1   | 1.75     | + Async traits |
| 1.2   | 1.75     | + Const generics |
| 1.3   | 1.80     | + Polonius |

## Quality Gates

Every published crate passes through:
1. Source synchronization verification (`ruchy verify`)
2. Bidirectional round-trip testing
3. API compatibility checking (`cargo semver-checks`)
4. Performance regression testing
5. Cross-version compatibility matrix

## Known Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Source/Generated Drift | High | `ruchy verify` in CI, pre-commit hooks |
| Transpiler Bugs | High | Differential testing, fuzzing |
| Package Bloat | Medium | Slim variants, compression |
| Version Matrix Explosion | Medium | Toolchain pinning |
