# ROADMAP.md - Ruchy Master Execution Plan

## 🎯 v0.3 STATUS: FEATURE COMPLETE ✅

**Test Results**: 297/301 tests passing (98.7% pass rate)  
**Core Features**: ✅ All P1/P2 features implemented  
**Ready for Release**: Only 4 IDE error recovery tests failing  

*See docs/execution/ for detailed task execution framework*

## Task Execution Protocol (NEW)

**All development now follows CLAUDE.md protocol:**

1. **LOCATE** specification section in SPECIFICATION.md
2. **FIND** task ID in docs/execution/roadmap.md  
3. **VERIFY** dependencies completed via DAG
4. **IMPLEMENT** with <10 complexity
5. **COMMIT** with task reference: `git commit -m "QA-P1-001: Fix let parsing"`

**Active Sprint**: Test Suite Stabilization (docs/execution/roadmap.md)

## 🔴 Current State (2025-01-17)

```
Actual Metrics (from deep_context.md)
┌─────────────────────────────────────────┐
│ Build:      ✅ Compiles                 │
│ Lint:       ✅ 0 clippy errors          │
│ Tests:      🔴 210/229 (91.7%)         │
│ Coverage:   🔴 65% (target: 80%)       │
│ SATD:       🔴 124 comments            │
│ Complexity: 🔴 37 max (target: 10)     │
└─────────────────────────────────────────┘
```

### Critical Violations
```
src/backend/transpiler.rs     917 complexity  (75K lines!)
src/frontend/parser/mod.rs     47 complexity
ruchy-cli/src/main.rs          37 complexity
src/frontend/parser/actors.rs  33 complexity
src/frontend/parser/collections.rs  32 complexity
```

## 🎯 Immediate Actions (Today)

### Hour 1: Split transpiler.rs
```bash
cd src/backend
mkdir transpiler
mv transpiler.rs transpiler/old.rs
# Create: mod.rs, expr.rs, stmt.rs, patterns.rs
```

### Hour 2: Eliminate SATD
```
Location                    Count  Action
src/runtime/repl.rs          93    Extract to GitHub issues
src/middleend/mir/lower.rs   12    Document or remove
src/runtime/repl_v2.rs       10    Document or remove
benches/repl_latency.rs       7    Convert to #[ignore]
```

### Hour 3: Fix failing tests
```
Priority order:
1. Transpiler tests (8) - Will fix with split
2. Type inference (4) - Lambda scoping
3. Parser edge cases (7) - Pattern matching
```

## 📋 Feature Implementation Status

### Language Core [70% Complete]
| Feature | Parse | Type | Transpile | Test | Spec |
|---------|-------|------|-----------|------|------|
| Literals | ✅ | ✅ | ✅ | ✅ | [spec](docs/architecture/grammar.md) |
| Functions | ✅ | ✅ | ✅ | ✅ | [spec](docs/architecture/grammar.md) |
| Lambdas | ✅ | 🟡 | ✅ | 🔴 | [spec](docs/done/lambda-feature-completed.yaml) |
| Pattern Match | ✅ | 🟡 | ✅ | 🔴 | [spec](docs/architecture/grammar.md) |
| Pipeline | ✅ | ✅ | ✅ | ✅ | [spec](docs/architecture/grammar.md) |
| Async/Await | ✅ | ❌ | 🟡 | 🔴 | [spec](docs/architecture/script-capabilities.md) |
| Try/Catch | ✅ | ❌ | 🔴 | 🔴 | - |
| Actors | ✅ | ❌ | 🔴 | 🔴 | [spec](docs/architecture/message-passing-mcp.md) |

### Pending Implementation [19 features]

#### P0: Blocking README Examples
- [ ] **Actor System** (5d effort)
    - Parse actor keyword ✅
    - Message passing (!)
    - Synchronous ask (?)
    - Supervision trees
    - [Architecture spec](docs/architecture/message-passing-mcp.md)

- [ ] **DataFrame Operations** (3d effort)
    - col() function
    - mean/std/alias
    - filter/groupby/agg
    - [No spec - needs creation]

#### P1: Core Language
- [x] **Impl Blocks** (3d) - Methods for structs ✅
- [x] **Trait System** (4d) - Full trait support ✅
- [x] **Pattern Guards** (2d) - if conditions in match ✅
- [x] **Break/Continue** (2d) - Loop control flow ✅

#### P2: Enhanced Features
- [x] **Property Testing** (3d) - #[property] attributes ✅
- [x] **List Comprehensions** (3d) - [x for x in list] ✅
- [x] **Generics** (5d) - Full type parameters ✅
- [x] **Object Literals** (2d) - {key: value} syntax ✅

#### P3: Future
- [ ] MCP Protocol Integration
- [ ] Refinement Types (SMT)
- [ ] JIT Compilation
- [ ] Row Polymorphism
- [ ] Package Manager
- [ ] LSP Implementation
- [ ] Debugger Support
- [ ] WebAssembly Target
- [ ] Incremental Compilation

## 📊 Quality Gates

| Metric | Current | Target | Blocker |
|--------|---------|--------|---------|
| Test Pass Rate | 98.7% | 100% | 🟡 |
| Coverage | 65% | 80% | Yes |
| SATD Comments | 124 | 0 | Yes |
| Max Complexity | 37 | 10 | Yes |
| Max File Size | 75K lines | 500 lines | Yes |
| Clippy Errors | 0 | 0 | ✅ |

## 🔗 Specification Documents

### Active Specs
- [Grammar Definition](docs/architecture/grammar.md)
- [MCP Integration](docs/architecture/message-passing-mcp.md)
- [Script Capabilities](docs/architecture/script-capabilities.md)
- [Quality Proxy](docs/architecture/quality-proxy.md)
- [Architecture](docs/architecture/ruchy-design-architecture.md)

### Implementation Records
- [Completed Features](docs/done/completed-features.md)
- [Lambda Implementation](docs/done/lambda-feature-completed.yaml)
- [v0.2 Features](docs/done/0.2-completed-features.yaml)

### Archived (Reference Only)
- `docs/done/archived-todos/` - Historical planning docs

## 📈 Velocity Tracking

```
Week of Jan 13-17:
├── Mon: Started with 262 clippy errors
├── Tue: Parser improvements
├── Wed: Type system work
├── Thu: REPL stabilization
├── Fri: Achieved 0 clippy errors ✅
└── Today: Fix SATD + split transpiler

Features/week: ~5
Debt reduction: -262 clippy, -124 SATD pending
```

## 🚦 Decision Matrix

```
Before ANY code change:
1. Check file complexity (max 10)
2. Check file size (max 500 lines)
3. Zero SATD tolerance
4. Run: make lint && make test

If modifying:
├── transpiler.rs → STOP, split first
├── parser/actors.rs → Reduce complexity first
├── Any REPL file → Remove SATD first
└── Other files → Proceed with caution
```

## 📅 Sprint Plan (Jan 17-24)

1. **Today**: Split transpiler, remove SATD, fix tests
2. **Mon-Tue**: Actor system implementation
3. **Wed-Thu**: DataFrame operations
4. **Fri**: Coverage to 80%

---
*Updated: 2025-01-17 | Next sync: After transpiler split*  
*Tracking: 124 SATD to remove, 19 tests to fix, 75K lines to split*