# Next Sprint Priority Options - v3.64.0 Analysis

**Date**: 2025-10-01
**Current Version**: v3.64.0
**Last Completed**: DataFrame Core (4/7 tickets, 60% complete, production-ready)
**Context**: All external repos analyzed (ruchy-book, rosetta-ruchy, ruchy-repl-demos)

---

## 📊 Current Status Summary

### Completed Recently
- ✅ **v3.64.0**: DataFrame core (literals, builder, CSV/JSON, transforms, sort)
- ✅ **v3.63.0**: Actor system (31 tests, 250K msg/sec)
- ✅ **39 DataFrame tests** passing (100%)
- ✅ **3,422 total tests** passing (99.4% coverage)

### External Repo Signals

#### 📚 ruchy-book (INTEGRATION.md)
- **Total Examples**: 120
- **Passing**: 83/120 (69%)
- **One-liners**: 17/20 (85%)
- **Major Gaps**:
  - Chapter 18 (DataFrames): 0/4 working (0% - **needs integration!**)
  - Chapter 17 (Error Handling): 5/11 working (45%)
  - Chapter 5 (Control Flow): 11/17 working (65%)

#### 🧮 rosetta-ruchy (ALGORITHM_COVERAGE_STATUS.md)
- **Algorithm Coverage**: 22/22 (100% complete!) ✅
- **All algorithms** have 100% test coverage with TDD
- **Quality**: A+ average (0.85-1.00 quality scores)
- **Status**: Production-ready, no issues

#### 💻 ruchy-repl-demos (COMPLETION_STATUS.md)
- **REPL Demos**: 85 demos (100% quality gates passing)
- **One-liners**: 95 demos
- **Total**: 180 demos (120% of target)
- **Quality Score**: 90%
- **Status**: Excellent, comprehensive coverage

---

## 🎯 Four Priority Options (Ranked by Impact)

### **OPTION 1: DataFrame Book Integration Sprint** ⭐ HIGHEST PRIORITY

**Objective**: Update ruchy-book to test v3.64.0 DataFrame features
**Impact**: 🚀 **CRITICAL** - We shipped features the book can't test yet!
**Effort**: 1-2 days
**Value**: Immediate user validation + documentation sync

#### The Problem
We just completed DataFrame v3.64.0 with production-ready features BUT:
- Book shows: "Chapter 18 (DataFrames): **0/4 working (0%)**"
- Book expects: `.get()`, `.filter()`, `.group_by()`, `.agg()`, `.column()`, rolling stats
- We implemented: literals, builder, CSV/JSON, `.with_column()`, `.transform()`, `.sort_by()`

**Gap Analysis**:
| Book Expects | v3.64.0 Has | Status |
|--------------|-------------|--------|
| `DataFrame::new().column()` | ✅ Implemented | Ready |
| `DataFrame::from_csv_string()` | ✅ Implemented | Ready |
| `DataFrame::from_json()` | ✅ Implemented | Ready |
| `.rows()`, `.columns()` | ✅ Implemented | Ready |
| `.with_column(closure)` | ✅ Implemented (smart binding) | Ready |
| `.transform(closure)` | ✅ Implemented | Ready |
| `.sort_by(col, desc)` | ✅ Implemented | Ready |
| `.get(col, row)` | ❌ Not implemented | **Missing** |
| `.filter(closure)` | ⚠️ Different API (uses `col()`) | **Needs update** |
| `.group_by().agg()` | ❌ Not implemented (DF-005) | **Future work** |
| `.column().mean()/.std()` | ❌ Not implemented (DF-006) | **Future work** |
| `.rolling_mean(window)` | ❌ Not implemented (DF-006) | **Future work** |

#### Tickets
1. **DF-BOOK-001**: Implement `.get(column, row)` accessor (2 hours)
2. **DF-BOOK-002**: Update book test files with v3.64.0 syntax (3 hours)
3. **DF-BOOK-003**: Add note about unimplemented features (1 hour)
4. **DF-BOOK-004**: Run book CI and verify 4/4 passing (1 hour)

#### Success Metrics
- Chapter 18: 0/4 → 4/4 passing (100%)
- Book examples: 83/120 → 87/120 (73%)
- **First advertised feature** with 100% book compatibility

#### Why This Is Priority #1
1. **We already did the hard work** (v3.64.0 DataFrame core)
2. **Book is out of sync** with reality (shows 0% when we have 60%)
3. **Quick win** (1-2 days vs 5-7 days for full DataFrame completion)
4. **User validation** happens immediately
5. **Documentation debt** cleared before moving to next feature

---

### **OPTION 2: Complete DataFrame Sprint (DF-005, DF-006, DF-007)**

**Objective**: Finish remaining 3 DataFrame tickets (60% → 100%)
**Impact**: 🔧 **HIGH** - Complete advertised data science feature
**Effort**: 5-7 days
**Value**: Full-featured DataFrame with aggregations, statistics, Polars backend

#### Remaining Tickets
- **DF-005**: Advanced Aggregations (2 days)
  - `.group_by(column)` → GroupedDataFrame
  - `.agg(column, function)` - chained aggregations
  - Support: mean, sum, count, min, max, std

- **DF-006**: Statistics Methods (2 days)
  - Column-level: `.mean()`, `.std()`, `.percentile(p)`
  - Window functions: `.rolling_mean(window_size)`
  - Descriptive: `.min()`, `.max()`, `.median()`

- **DF-007**: Polars Integration (3 days)
  - Replace custom DataFrame with Polars wrapper
  - Arrow memory format for efficiency
  - Lazy evaluation support
  - Performance: 1M rows <1s

#### Success Metrics
- DataFrame: 60% → 100% complete
- Book Chapter 18: 0/4 → 4/4 passing (depends on OPTION 1 first)
- Performance: 10K rows <100ms with Polars
- 20+ additional tests for new features

#### Why Not Priority #1
- **Book integration** should happen first (validates what we built)
- **3x longer** than Option 1 (5-7 days vs 1-2 days)
- **Polars integration** is risky (big refactor)
- Can be **deferred** until after book sync

---

### **OPTION 3: Error Handling Sprint (45% → 90%)**

**Objective**: Complete Chapter 17 error handling features
**Impact**: 🛡️ **MEDIUM-HIGH** - Production reliability feature
**Effort**: 3-5 days
**Value**: Robust error handling + 6 more book examples passing

#### Current Status
- Book: 5/11 examples passing (45%)
- Missing features affect **production readiness**

#### Tickets
1. **ERROR-001**: Result<T, E> unwrap/expect methods (1 day)
2. **ERROR-002**: Error propagation with ? operator (1 day)
3. **ERROR-003**: Custom error types + impl Error (1 day)
4. **ERROR-004**: Error context and backtraces (1 day)
5. **ERROR-005**: try/catch syntax (1 day)

#### Success Metrics
- Chapter 17: 5/11 → 10/11 (90%)
- Book examples: 83/120 → 88/120 (73%)
- Zero panics in error handling tests
- 15+ TDD tests for error patterns

#### Why Not Priority #1
- **DataFrame book sync** is more urgent (feature we just shipped)
- **Error handling** is important but not blocking users
- Can be **scheduled** after DataFrame work complete

---

### **OPTION 4: Control Flow Completion Sprint (65% → 95%)**

**Objective**: Complete Chapter 5 control flow features
**Impact**: ⚡ **MEDIUM** - Fundamental language features
**Effort**: 2-4 days
**Value**: 6 more book examples + language completeness

#### Current Status
- Book: 11/17 examples passing (65%)
- Missing: loop labels, match guards, advanced patterns

#### Tickets
1. **CTRL-001**: Loop labels (`break 'outer`, `continue 'label`) (1 day)
2. **CTRL-002**: Match guards with complex expressions (1 day)
3. **CTRL-003**: Pattern matching enhancements (1 day)
4. **CTRL-004**: Advanced loop patterns (1 day)

#### Success Metrics
- Chapter 5: 11/17 → 16/17 (94%)
- Book examples: 83/120 → 88/120 (73%)
- 10+ TDD tests for control flow

#### Why Not Priority #1
- **Less urgent** than syncing DataFrame docs
- **Smaller impact** than completing DataFrame
- **Good follow-up** after DataFrame work

---

## 📋 Recommended Sprint Sequence

### **Immediate (This Week)**
1. **OPTION 1**: DataFrame Book Integration (1-2 days)
   - Quick win, validates v3.64.0 work
   - Clears documentation debt
   - Shows users DataFrames work

### **Next Sprint (Following Week)**
Choose based on strategy:
- **User-facing features**: OPTION 2 (Complete DataFrame)
- **Production reliability**: OPTION 3 (Error Handling)
- **Language fundamentals**: OPTION 4 (Control Flow)

### **Ideal Sprint Flow**
```
Week 1: DF-BOOK (Option 1) → Book: 83/120 → 87/120 (73%)
Week 2: DF-COMPLETE (Option 2) → DataFrame: 60% → 100%
Week 3: ERROR (Option 3) → Book: 87/120 → 93/120 (78%)
Week 4: CTRL (Option 4) → Book: 93/120 → 98/120 (82%)
```

**Result after 4 weeks**:
- DataFrames 100% complete
- Book compatibility: 69% → 82%
- Production-ready error handling
- Complete control flow

---

## 🎯 Final Recommendation

### **START WITH OPTION 1: DataFrame Book Integration**

**Reasoning**:
1. ✅ **Validates work already done** (v3.64.0 DataFrame core)
2. ✅ **Fastest ROI** (1-2 days to 4 more passing examples)
3. ✅ **Clears technical debt** (book out of sync with reality)
4. ✅ **User-facing** (shows DataFrames work in book)
5. ✅ **Low risk** (minimal code changes, mostly documentation)
6. ✅ **Enables decision** (see if users want DF-005/DF-006 or move on)

**After Option 1 completes**, reassess based on user feedback and strategic goals.

---

**Questions for Decision**:
1. Do users need full DataFrame features (aggregations, stats) immediately?
2. Is error handling blocking production use cases?
3. Are control flow gaps affecting real workflows?
4. Should we focus on **depth** (complete DataFrames) or **breadth** (more chapters)?
