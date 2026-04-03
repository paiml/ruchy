# Ruchy Testing Strategy & Technical Specification

## Executive Summary

This document defines a systematic testing strategy to achieve >80% coverage across the Ruchy compiler through disciplined application of Toyota Way principles. We decompose the challenge into five orthogonal categories, each with enforced quality gates, TDD requirements, and complexity limits.

## Toyota Way Testing Philosophy

### Core Principles
- **Jidoka**: Build quality into each component through TDD
- **Andon**: Stop immediately when quality gates fail
- **Kaizen**: Continuous improvement through incremental coverage gains
- **Genchi Genbutsu**: Measure actual coverage, not estimates

### Quality Gate Requirements (Non-Negotiable)
```
TDD: Test written BEFORE implementation
Complexity: Cyclomatic complexity <=10 per function
PMAT Score: TDG grade >=A+ (95 points)
Coverage: >=80% per category
Zero Tolerance: No clippy warnings, no broken tests
```

## Five-Category Coverage Strategy

| Category | Target Coverage | Current | Gap | Sprint Priority |
|----------|----------------|---------|-----|-----------------|
| Frontend | 80% | 45% | 35% | 2 |
| Backend | 80% | 50% | 30% | 3 |
| Runtime | 80% | 40% | 40% | 4 |
| WASM | 80% | 15% | 65% | 5 |
| Quality | 80% | 60% | 20% | 1 |

## Sub-spec Index

| Sub-spec | Description | Lines |
|----------|-------------|-------|
| [Strategy and Language Architecture](sub/five-categories-strategy-architecture.md) | Sprint plans, Makefile implementation, TDD protocol, complexity enforcement, Five Whys analysis, language architecture, type system specification | ~441 |
| [Quality Enforcement and Tooling](sub/five-categories-quality-enforcement.md) | Quality enforcement framework, test development protocol, mutation testing, MCP integration, memory management, incremental compilation, tooling, benchmarking, error messages, sprint schedule, CI matrix, ADRs | ~399 |

## Contact & Contribution

- **Architecture**: Document changes as ADRs in `docs/architecture/`
- **Performance**: Include benchmarks proving no regression
- **API Changes**: Provide migration guide with semver impact
- **Test Coverage**: Maintain >80% per category or justify exception
