# World-Class Formatter Specification v1.0

**Status**: ACTIVE - Specification for Perfect Formatter
**Created**: 2025-10-15
**Target**: v3.89.0 - v3.91.0 (3-release roadmap)
**Goal**: Make `ruchy fmt` PERFECT - industry-leading formatter

---

## Executive Summary

Create a **world-class formatter** for Ruchy that matches or exceeds the quality of industry leaders (rustfmt, Deno fmt, Ruff). The formatter must be **PERFECT** - preserving comments, doctests, annotations, and user intent while applying consistent formatting.

**Previous Status**: v3.88.0 - P0 corruption fixed, but formatter had P1 issues:
- Stripped ALL comments (documentation loss)
- Significant unwanted style changes
- Only 27/85 ExprKind variants implemented (~32%)
- Newline display issues

**Current Status**: v3.89.0 - Configuration + Ignore Directives COMPLETE (2025-10-15):
- Configuration system with TOML support (11 tests passing)
- Ignore directives fully functional (10/10 tests passing)
- Parser fixes: Line continuations + multiple comments (9 parser tests)
- 10 critical bugs fixed with Extreme TDD methodology
- Property tests: 6 tests with 10K+ random inputs
- Released to crates.io: https://crates.io/crates/ruchy/3.89.0

**Target Status**: v3.91.0 - Perfect formatter:
- 100% comment preservation
- 100% ExprKind coverage (85/85 variants)
- Minimal, intentional style changes only
- Configurable formatting options
- Round-trip validation
- Industry-leading quality

---

## Sub-spec Index

| Sub-spec | Sections | Lines | Description |
|----------|----------|-------|-------------|
| [Best Practices & Phase 1 Architecture](sub/formatter-best-practices-architecture.md) | Industry Best Practices, Defect Analysis, Requirements, Phase 1 Architecture | 347 | Lessons from rustfmt/Deno/Ruff, defect history, P0/P1/P2 requirements, comment preservation architecture |
| [TDD Tests & ExprKind Coverage](sub/formatter-tdd-exprkind-coverage.md) | Phase 1 TDD Tests, Phase 2 ExprKind Coverage | 389 | Extreme TDD test suite for comments, systematic ExprKind variant implementation |
| [Testing, Roadmap & Quality Gates](sub/formatter-testing-roadmap-gates.md) | Phase 3 Style/Config, Testing Strategy, Roadmap, Quality Gates, Success Metrics | 481 | Style preservation, configuration, test pyramid, implementation tickets, release gates |

---

## Toyota Way Principles Applied

### Jidoka (Built-in Quality)
- Comments preserved by design (not as afterthought)
- All variants implemented (no fallbacks)
- Tests written FIRST (TDD)
- Quality gates block bad releases

### Genchi Genbutsu (Go and See)
- Learned from external bug reports
- Studied industry leaders (rustfmt, Deno, Ruff)
- Tested with real-world code
- External validation required

### Poka-Yoke (Error Proofing)
- No fallback cases (panic if variant missing)
- Round-trip validation catches corruption
- Property tests catch edge cases
- Ignore directives give users control

### Kaizen (Continuous Improvement)
- Systematic 3-release roadmap
- Each release builds on previous
- Defect reports drive improvements
- Never satisfied with "good enough"

### Respect for People
- Preserve user's documentation (comments)
- Preserve user's style choices
- Give users control (configuration)
- Never lose user's work

---

## Appendix: Related Documents

- **CRITICAL-FMT-CODE-DESTRUCTION.md** - Operator mangling (FIXED v3.87.0)
- **CRITICAL-FMT-DEBUG-FALLBACK.md** - AST corruption (FIXED v3.88.0)
- **DEFECT-FMT-002-COMMENT-STRIPPING.md** - Comment loss (ACTIVE v3.88.0)
- **BUG_VERIFICATION_v3.88.0.md** - External validation report
- **15-tool-improvement-spec.md** - Tool quality standards
- **TICR-ANALYSIS.md** - Test complexity analysis

---

**Created**: 2025-10-15
**Target Completion**: v3.91.0 (3 sprints, ~8-11 days total)
**Status**: READY TO IMPLEMENT
**Goal**: Make `ruchy fmt` **PERFECT** - worthy of industry leaders
