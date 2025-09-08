# Ruchy Development Roadmap

**Last Updated**: August 24, 2025  
**Current Version**: v1.9.1  
**Language Completeness**: 85-90% Core Features

## 🚨 Critical Path (Immediate Blockers)

### Priority 1: Format String Runtime Fix [BLOCKING EVERYTHING]
**Status**: 🔴 Critical - Blocking all projects  
**Impact**: rosetta-ruchy, ruchy-book, user scripts  
**Issue**: `println!("Result: {}", var)` validates but fails at runtime

```rust
// Must fix runtime compilation for:
println!("Result: {}", variable);
println!("Float: {:.2}", float_value);
println!("Multiple: {} and {}", a, b);
```

**Why Critical**:
- Every algorithm needs to display results
- Currently forced to use static strings only
- Blocks 81% of ruchy-book examples
- Makes language unusable for real work

**Solution Path**:
1. Fix transpiler format string generation
2. Ensure proper Rust macro generation
3. Add comprehensive format string tests
4. Validate against rosetta-ruchy algorithms

---

## 📊 Priority Matrix (Data-Driven from Sister Projects)

| Priority | Feature | Impact | Source | Status |
|----------|---------|--------|--------|--------|
| **P0** | Format Strings | 81% examples blocked | rosetta-ruchy | 🔴 Broken |
| **P1** | Math Functions | 30% examples need | ruchy-book | ⚠️ Missing |
| **P2** | File I/O | 25% examples need | ruchy-book | ⚠️ Partial |
| **P3** | HashMap/HashSet | 20% examples need | rosetta-ruchy | ⚠️ Missing |
| **P4** | Error Handling | 15% examples need | ruchy-book | ✅ Works |
| **P5** | Iterators | 10% need advanced | rosetta-ruchy | ⚠️ Basic only |

---

## 🎯 Sprint 15: Format String Emergency Fix (Next 2-4 hours)

### Objective: Unblock ALL Projects
**Success Metric**: `println!("n = {}", n)` works in REPL and transpiler

### Tasks:
- [ ] **FIX-001**: Debug why format strings fail at runtime
- [ ] **FIX-002**: Fix macro generation in transpiler
- [ ] **FIX-003**: Add format string evaluation to REPL
- [ ] **FIX-004**: Create comprehensive test suite
- [ ] **FIX-005**: Validate all rosetta-ruchy algorithms
- [ ] **FIX-006**: Release v1.9.2 immediately

### Test Cases Required:
```rust
// All must work:
println!("Simple: {}", 42);
println!("Float: {:.2}", 3.14159);
println!("Multiple: {} + {} = {}", 1, 2, 3);
println!("Mixed: {} and {}", "string", 123);
println!("Debug: {:?}", vec![1, 2, 3]);
```

---

## 📈 Sprint 16: Standard Library Essentials (After Format Fix)

### Math Functions (Most Requested)
```rust
// Core math needed by algorithms:
fn sqrt(x: f64) -> f64
fn pow(base: f64, exp: f64) -> f64
fn abs(x: T) -> T
fn min(a: T, b: T) -> T
fn max(a: T, b: T) -> T
fn floor(x: f64) -> f64
fn ceil(x: f64) -> f64
fn round(x: f64) -> f64
```

### Collection Utilities
```rust
// HashMap/HashSet for algorithms:
HashMap::new()
HashMap::insert(k, v)
HashMap::get(k)
HashMap::contains_key(k)
HashSet::new()
HashSet::insert(v)
HashSet::contains(v)
```

---

## 🚀 Phase 2: Integration Pattern Fixes (Week 2)

### Complex Patterns That Must Work
Based on ruchy-book failures:

1. **Nested Closures with Captures**
```rust
let outer = 10;
let f = |x| {
    let g = |y| x + y + outer;
    g(5)
};
```

2. **Method Chaining on Complex Types**
```rust
data.filter(|x| x > 0)
    .map(|x| x * 2)
    .collect::<Vec<_>>()
```

3. **Generic Type Inference**
```rust
fn identity<T>(x: T) -> T { x }
let result = identity(42);  // Should infer i32
```

---

## 🔄 Phase 3: WASM REPL Deployment (Week 3)

### Browser-Based REPL
**Status**: Specification complete, ready for implementation

- [ ] Implement core WASM module (<200KB)
- [ ] Progressive loading strategy
- [ ] GitHub Pages deployment
- [ ] Mobile browser support

---

## 📊 Success Metrics

### Immediate (Sprint 15)
- ✅ Format strings work: 100% of cases
- ✅ rosetta-ruchy: All algorithms display output
- ✅ ruchy-book: Jump from 19% → 40% compatibility

### Short-term (2 weeks)
- ✅ Standard library: 60% complete
- ✅ ruchy-book: 60% compatibility
- ✅ rosetta-ruchy: 100% algorithms working

### Medium-term (1 month)
- ✅ WASM REPL deployed
- ✅ ruchy-book: 80% compatibility
- ✅ Bootstrap Stage 0 complete

---

## 🚫 What We're NOT Doing

Based on analysis, these are already working or low priority:

### Already Working (Don't Touch)
- ✅ Pipeline operator
- ✅ Import/Export
- ✅ String methods
- ✅ Fat arrow syntax
- ✅ Async/await
- ✅ Generics
- ✅ Traits

### Low Priority (Later)
- Actor system (no examples need it)
- Advanced macros (not blocking anything)
- Custom operators (nice to have)
- IDE plugins (after core stable)

---

## 📝 Sprint Planning Template

```markdown
## Sprint N: [Feature Name] (Duration)
**Objective**: [Clear goal]
**Success Metric**: [Measurable outcome]
**Blocking**: [What this unblocks]

### Tasks:
- [ ] TASK-001: [Specific action]
- [ ] TASK-002: [Specific action]

### Validation:
- [ ] rosetta-ruchy: [Specific test]
- [ ] ruchy-book: [Specific test]
- [ ] Release: v1.x.x
```

---

## 🎯 The North Star

**Make Ruchy usable for real work by fixing what's actually broken, not adding new features.**

The data is clear:
1. Format strings are the #1 blocker
2. Standard library gaps are #2
3. Everything else is nice-to-have

---

**Remember**: We have 85-90% language completeness. We don't need more features. We need the existing features to work reliably in all contexts.