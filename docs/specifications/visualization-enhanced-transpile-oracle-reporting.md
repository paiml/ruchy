# Visualization-Enhanced Transpile Oracle Reporting

**Status**: APPROVED - Text-Only Output
**References**: depyler, bashrs visualization systems
**Ticket**: VIS-001

## Executive Summary

This specification proposes 5 core and 3 additional visualization and reporting enhancements for ruchy's transpile oracle, drawing from proven patterns in depyler and bashrs. **All output is text-based** (terminal, JSON, SARIF, Markdown) - no HTML/web dashboards.

The design is grounded in Toyota Production System principles (Mieruka, Jidoka, Kaizen) and peer-reviewed SBFL research (Tarantula, Ochiai, D*).

## Sub-spec Index

| Sub-spec | Scope | Link |
|----------|-------|------|
| Academic Foundation and Top 5 Enhancements | SBFL citations, Toyota Way alignment, gap analysis, SBFL integration, rich ASCII reports, multi-format output, error clustering, convergence dashboard | [sub/vis-oracle-top5-enhancements.md](sub/vis-oracle-top5-enhancements.md) |
| Implementation, Additional Features, and References | Priority ordering, file structure, dependencies, acceptance criteria, delta debugging, semantic tagging, 5-phase corpus pipeline, full reference list | [sub/vis-oracle-additional-features.md](sub/vis-oracle-additional-features.md) |

## Verification Checklist

| # | Feature | Status |
|---|---------|--------|
| 1 | SBFL Fault Localization (Tarantula/Ochiai/D*) | APPROVED |
| 2 | Rich ASCII Reports (sparklines, grades, Andon) | APPROVED |
| 3 | Multi-Format Output (JSON, SARIF, Markdown) | APPROVED |
| 4 | Error Clustering + Pareto Analysis | APPROVED |
| 5 | Convergence Dashboard | APPROVED |
| 6 | Delta Debugging / Bisection Mode | APPROVED |
| 7 | Semantic Tagging & Corpus Filtering | APPROVED |
| 8 | 5-Phase Corpus Pipeline | APPROVED |

## Implementation Priority

Recommended order based on value/effort ratio:

1. **Rich ASCII Reports** (High value, low effort) - 1-2 days
2. **Multi-Format Output** (High value, medium effort) - 2-3 days
3. **Error Clustering + Pareto** (Medium value, medium effort) - 2 days
4. **SBFL Integration** (High value, medium effort) - 2-3 days
5. **Convergence Dashboard** (Medium value, high effort) - 3-4 days

**Total estimated effort**: 10-14 days

## Acceptance Criteria

- [ ] All features have unit tests (TDD per Extreme TDD Protocol)
- [ ] Property tests for SBFL algorithms (10K+ cases)
- [ ] Mutation test coverage >=75% (cargo-mutants)
- [ ] PMAT TDG grade >=A- for all new files
- [ ] Zero clippy warnings
- [ ] All output is text-based (no HTML)
