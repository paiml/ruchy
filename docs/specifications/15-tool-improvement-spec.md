# 33-Tool Improvement Specification v5.0

**Purpose**: Systematic analysis with complete testing pyramid (unit + property + mutation + **CLI contract**)
**Date**: 2025-10-15
**Status**: CLI contract testing for 32/33 tools (97% coverage)
**Methodology**: Genchi Genbutsu + Kaizen + AST generators + **Black-box CLI validation**

---

## Sub-spec Index

| Sub-spec | Sections | Description | Lines |
|----------|----------|-------------|-------|
| [Metrics, Pyramid, and CLI Testing Framework](sub/tool-improvement-metrics-cli-testing.md) | 1-4 | Executive summary, complete testing pyramid, per-tool test breakdown (all 33 tools), CLI expectation testing framework (assert_cmd + rexpect), non-interactive and interactive tool examples, per-tool CLI test specifications | 500 |
| [Testability Review, Shrinking, and Andon Cord](sub/tool-improvement-testability-andon.md) | 5-8 | TICR quantification, testability review gate criteria, complexity point examples, meta-testing shrinking mechanism, automated Andon cord issue creation (Toyota Way) | 314 |
| [Action Plan, Updated Metrics, and Critical Path](sub/tool-improvement-action-plan-path.md) | 9-12 | Revised action plan v4.0, sprint breakdown (5 sprints), updated metrics v4.0, current progress, definition of done for v1.0, critical path (22 days parallel), conclusion, fmt tool P0 bugs | 271 |

---

## Executive Summary

**Test Coverage**: 339+ CLI tests passing (97%+)
**CLI Contract Coverage**: 32/33 tools (97%)
**SATD Risk**: LOW -- Zero TODO/FIXME/unimplemented
**Vaporware Risk**: LOW -- 97% tools validated via CLI contract tests
**User-Facing Contract**: EXCELLENT -- 339+ CLI tests covering all major workflows

## Complete Testing Pyramid

```
              CLI Expectation (E2E)     <- assert_cmd/rexpect
             Mutation Testing           <- cargo-mutants
            Property Testing            <- AST generators
           Unit Testing                 <- cargo test
```

## Current Progress (v5.0)

```
Total Tests:              4,241+ (3,902 internal + 339 CLI)
Passing:                  4,210+ (99.3%)
CLI Contract Coverage:    97% tools (32/33)
Line Coverage:            85.3%
Mutation Score:            75.2%
```

## Conclusion

**Current State**: 58% complete (v4.0 assessment with complete testing pyramid)

**Complete Testing Pyramid Status**:
- Layer 1 (Unit): 12/16 tools (75%)
- Layer 2 (Property): 7/16 tools (44%) -- weak generators
- Layer 3 (Mutation): 1/16 tools (6%) -- critical gap
- Layer 4 (CLI): 12/16 tools (75%)

**Path to v1.0**: 42 days serial, 23 days parallel (4 engineers)

**Critical Insight**: Internal logic testing (unit + property + mutation) is necessary but insufficient. Public contract (CLI) must be validated separately.
