# Mutation Testing Specification for Ruchy

**Version**: 1.0.0
**Status**: Draft
**Based On**: pforge mutation testing methodology

## Executive Summary

Mutation testing is a quality assurance technique that evaluates the effectiveness of our test suite by introducing small, deliberate bugs (mutations) into the code and checking if our tests catch them. This specification defines how Ruchy will implement mutation testing to achieve **90%+ mutation kill rate**, ensuring our tests are truly effective at catching bugs.

For Ruchy as a compiler/interpreter, correctness is critical. Mutation testing ensures our parser, type checker, evaluator, and WASM backend tests are robust enough to catch subtle bugs.

## Sub-spec Index

| Sub-spec | Scope | Link |
|----------|-------|------|
| Categories, Priority Modules, and Execution | Why mutation testing, goals, cargo-mutants tool, 6 mutation categories (arithmetic, boolean, comparison, return values, match arms, empty functions), priority modules P0-P3, running commands, Makefile integration | [sub/mutation-testing-categories-execution.md](sub/mutation-testing-categories-execution.md) |
| Test Writing Guidelines, Quality Gates, and Roadmap | 6 test writing guidelines (exact assertions, all match arms, both branches, boundary conditions, non-default values, side effects), pre-commit hooks, CI/CD pipeline, 5-phase roadmap, metrics, reporting format, best practices | [sub/mutation-testing-guidelines-roadmap.md](sub/mutation-testing-guidelines-roadmap.md) |

## Goals

1. **Achieve 90%+ mutation kill rate** across all critical modules
2. **Identify weak tests** that pass even when code is broken
3. **Prevent regressions** by ensuring tests verify actual behavior
4. **Complement PMAT quality** with behavior-level validation

## Priority Module Targets

| Priority | Modules | Kill Rate Target |
|----------|---------|-----------------|
| P0 - Critical | Parser, Evaluator, Type System | 95%+ |
| P1 - High | WASM Backend, REPL | 90%+ |
| P2 - Medium | Standard Library, Error Handling | 85%+ |
| P3 - Lower | CLI | 80%+ |

## Conclusion

Mutation testing is a powerful complement to traditional coverage metrics and PMAT quality enforcement. By achieving 90%+ mutation kill rate, we ensure that our tests don't just execute code -- they actually verify correctness. This is critical for a compiler/interpreter where subtle bugs can have far-reaching consequences.

## References

- [cargo-mutants documentation](https://mutants.rs/)
- [Mutation Testing: A Comprehensive Survey](https://ieeexplore.ieee.org/document/5487526)
- [State of Mutation Testing at Google (2018)](https://research.google/pubs/pub46584/)
