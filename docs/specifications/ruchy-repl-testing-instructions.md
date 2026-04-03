# Ruchy REPL Testing Infrastructure

## Overview

Build comprehensive test harness for Ruchy REPL using layered testing strategy: `assert_cmd` for non-interactive CLI tests, `rust-cli/rexpect` for PTY-based interaction tests, extended with zero-copy buffer management and sub-100us pattern matching for high-frequency test scenarios.

**Architecture Decision**: Hybrid approach using `assert_cmd` (non-interactive), `rexpect` (PTY-interactive), and optional `rexpect-extensions` (async/zero-copy performance layer).

## Sub-spec Index

| Sub-spec | Scope | Link |
|----------|-------|------|
| Architecture and Implementation Layers | Gap analysis, assert_cmd layer, rexpect layer, async extensions | [sub/repl-testing-architecture.md](sub/repl-testing-architecture.md) |
| Extreme TDD, Quality, and Workflows | TDD workflow, pmat mutation testing, property tests, test organization, dependencies, checklists | [sub/repl-testing-tdd-quality.md](sub/repl-testing-tdd-quality.md) |

## Key Design Decisions

- **Layer 1 (assert_cmd)**: Fast non-interactive CLI tests (<100ms per test)
- **Layer 2 (rexpect)**: PTY-based interactive tests for tab completion, signals, multiline
- **Layer 3 (rexpect-extensions)**: Build only if profiling shows rexpect is bottleneck

## Performance Targets

| Test Type | Count | Total Runtime | Feedback Loop |
|-----------|-------|---------------|---------------|
| CLI (assert_cmd) | 50+ | <2s | RED->GREEN: <10s |
| PTY (rexpect) | 20+ | <5s | RED->GREEN: <30s |
| Property | 10+ | <30s | Nightly CI |
| Mutation (pmat) | - | <5min | Pre-commit hook |

## Success Metrics

- **Test Coverage**: >85% line coverage (cargo-llvm-cov)
- **Mutation Score**: >75% on core REPL logic (pmat)
- **Test Runtime**: <10s for full suite
- **False Positives**: <5% flaky test rate
- **Property Tests**: >1000 cases per property

## Resources

- **assert_cmd docs**: https://docs.rs/assert_cmd
- **rexpect docs**: https://docs.rs/rexpect
- **proptest book**: https://proptest-rs.github.io/proptest/
