# PMAT Bug Report: TDG Structural Score Remains 0.0/25 Despite Significant Refactoring

**Date**: 2025-10-03
**Reporter**: Ruchy Compiler Project
**PMAT Version**: 2.111.0
**Language**: Rust
**Severity**: Medium (Metric Accuracy Issue)

## Summary

TDG Structural Complexity score remains static at **0.0/25** despite systematic refactoring that reduced function complexity by 80-90% and extracted 24 helper functions, all meeting <10 cyclomatic complexity target.

## Environment

- **PMAT Version**: 2.111.0
- **OS**: Linux 6.8.0-83-generic
- **Language**: Rust
- **File**: `src/backend/wasm/mod.rs`
- **File Size**: 1,267 lines
- **Function Count**: ~45 functions (after refactoring)

## Issue Description

### Expected Behavior

TDG Structural Complexity score should **increase** when:
1. Large functions are decomposed into smaller functions
2. Each extracted function has complexity <10
3. Overall code maintainability improves significantly

### Actual Behavior

TDG Structural score remains **0.0/25** (unchanged) despite:
- Extracting 24 helper functions across 4 refactoring phases
- Reducing key function sizes by 80-90%
- All extracted functions verified <10 cyclomatic complexity
- Maintaining 100% test pass rate (26/26 tests)

### TDG Scores Throughout Refactoring

| Phase | Functions Extracted | TDG Overall | TDG Structural | TDG Duplication |
|-------|---------------------|-------------|----------------|-----------------|
| Baseline | 0 | 75.7/100 (B) | **0.0/25** | 15.7/20 |
| Phase 1 | 7 | 75.8/100 (B) | **0.0/25** | 15.7/20 |
| Phase 2 | 8 | 76.1/100 (B) | **0.0/25** | 16.1/20 |
| Phase 3 | 6 | 76.1/100 (B) | **0.0/25** | 16.1/20 |
| Phase 4 | 3 | 76.1/100 (B) | **0.0/25** | 16.1/20 |

**Note**: Duplication improved (15.7→16.1), but Structural unchanged despite dramatic refactoring.

## Evidence of Refactoring

### Function Size Reductions

**1. `emit()` function:**
- **Before**: 128 lines, estimated complexity 15-20
- **After**: 26 lines, complexity 4
- **Reduction**: 80%
- **Extracted**: 8 helper functions (all <10 complexity)

**2. `lower_expression()` function:**
- **Before**: ~240 lines, estimated complexity 55-60
- **After**: 24 lines, complexity 4
- **Reduction**: 90%
- **Extracted**: 13 helper functions (all <10 complexity)

**3. `infer_type()` function:**
- **Before**: Nested match, complexity ~12
- **After**: Simple dispatch, complexity 7
- **Extracted**: 3 helper functions (all <3 complexity)

### Sample Helper Functions (All <10 Complexity)

```rust
/// Complexity: 1 (Toyota Way: <10 ✓)
fn wasm_type_to_valtype(&self, ty: WasmType) -> wasm_encoder::ValType {
    match ty {
        WasmType::I32 => wasm_encoder::ValType::I32,
        WasmType::F32 => wasm_encoder::ValType::F32,
        WasmType::I64 => wasm_encoder::ValType::I64,
        WasmType::F64 => wasm_encoder::ValType::F64,
    }
}

/// Complexity: 2 (Toyota Way: <10 ✓)
fn emit_memory_section(&self, expr: &Expr) -> Option<MemorySection> {
    if self.needs_memory(expr) {
        let mut memories = MemorySection::new();
        memories.memory(MemoryType {
            minimum: 1,
            maximum: None,
            memory64: false,
            shared: false,
            page_size_log2: None,
        });
        Some(memories)
    } else {
        None
    }
}

/// Complexity: 4 (Toyota Way: <10 ✓)
fn lower_while(&self, condition: &Expr, body: &Expr) -> Result<Vec<Instruction<'static>>, String> {
    let mut instructions = vec![];
    instructions.push(Instruction::Loop(wasm_encoder::BlockType::Empty));
    instructions.extend(self.lower_expression(condition)?);
    instructions.push(Instruction::I32Eqz);
    instructions.push(Instruction::BrIf(1));
    instructions.extend(self.lower_expression(body)?);
    instructions.push(Instruction::Br(0));
    instructions.push(Instruction::End);
    Ok(instructions)
}
```

## Steps to Reproduce

```bash
# Clone repository
git clone https://github.com/yourusername/ruchy
cd ruchy

# Check initial TDG score
pmat tdg src/backend/wasm/mod.rs --include-components

# View refactoring commits
git log --oneline --grep="WASM-REFACTOR"
# 162570b7 - Phase 1: Extract lower_* helper functions
# e3030a7f - Phase 2: Extract emit_* section helper functions
# e9834ce9 - Phase 3: Extract remaining lower_expression helpers
# ec0e784b - Phase 4: Extract infer_type helper functions

# Check each phase's TDG score (all show Structural: 0.0/25)
git checkout 162570b7 && pmat tdg src/backend/wasm/mod.rs --include-components
git checkout e3030a7f && pmat tdg src/backend/wasm/mod.rs --include-components
git checkout e9834ce9 && pmat tdg src/backend/wasm/mod.rs --include-components
git checkout ec0e784b && pmat tdg src/backend/wasm/mod.rs --include-components
```

## Hypotheses

### 1. File Size Threshold
**Hypothesis**: TDG may penalize files >1,000 lines regardless of internal complexity
- File remains ~1,267 lines after refactoring
- New functions added, but large test section (~300 lines) remains
- **Test**: Split into submodules and re-measure

### 2. Rust Analysis Limitation
**Hypothesis**: PMAT's Rust complexity analysis may not be fully functional
- `pmat analyze complexity` returned 0 functions for Rust files
- Tree-sitter parser may have issues with Rust
- **Test**: Compare with other Rust projects

### 3. Hidden Metrics
**Hypothesis**: Structural score may include metrics beyond cyclomatic complexity
- File-level metrics (module count, nesting depth)
- Absolute LOC thresholds
- **Test**: Review TDG scoring algorithm documentation

## Verification Commands

```bash
# TDG with all components
pmat tdg src/backend/wasm/mod.rs --include-components --format=table

# JSON output for detailed analysis
pmat tdg src/backend/wasm/mod.rs --format=json > tdg_output.json

# Complexity analysis (returns 0 functions for Rust)
pmat analyze complexity --path src/backend/wasm

# System diagnostics
pmat tdg diagnostics
```

## Expected Fix

One of:
1. **Bug fix**: TDG Structural should reflect function-level complexity improvements
2. **Documentation**: Clarify what Structural score measures and why file size dominates
3. **Feature request**: Add function-level complexity breakdown to TDG output

## Impact

**Low impact on development** (code quality objectively improved), but **high impact on metrics trust**:
- ✅ Code is dramatically more maintainable (verified by manual review)
- ✅ All functions meet <10 complexity target (Toyota Way compliant)
- ✅ Zero test regressions (26/26 passing throughout)
- ❌ TDG metric doesn't reflect objective improvements
- ❌ Reduces confidence in PMAT quality gates

## Additional Context

This refactoring was conducted following **Toyota Way** quality principles:
- **Jidoka**: TDG detected the issue (0.0/25 triggered refactoring)
- **Genchi Genbutsu**: Manual code inspection confirmed violations
- **Kaizen**: Systematic improvement through 4 refactoring phases
- **Hansei**: This bug report to improve the tool itself

The file includes comprehensive test coverage (300+ lines of tests), which may be skewing file-level metrics.

## Related Files

- Analysis document: `docs/execution/WASM_QUALITY_ANALYSIS.md`
- Source file: `src/backend/wasm/mod.rs`
- Test coverage: 26 integration tests passing

## Request

Please investigate why TDG Structural Complexity score remains static at 0.0/25 despite significant refactoring efforts. This affects confidence in PMAT's ability to measure Rust code quality accurately.

## Contact

GitHub: https://github.com/yourusername/ruchy
Issue: (to be filed)

---

**Automated Context**: This report documents systematic refactoring following Toyota Way principles. All claims are verifiable through git history and automated tests.
