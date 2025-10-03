# WASM Backend Quality Analysis

## PMAT Analysis Results

### TDG Score: 75.7/100 (Grade: B)

**Component Breakdown:**
- ✅ Semantic Complexity: 20.0/20 (100%)
- ✅ Documentation: 10.0/10 (100%)  
- ✅ Consistency: 10.0/10 (100%)
- ✅ Coupling: 15.0/15 (100%)
- ⚠️ Duplication: 15.7/20 (78.5%)
- ❌ **Structural Complexity: 0.0/25 (0%)** ⚠️ **CRITICAL**

### Critical Finding

**Structural complexity scored 0.0/25** - This indicates severe complexity violations.

### File Metrics

- **LOC**: 1,181 lines
- **Functions**: 26 total
- **Max Complexity**: Not calculated (requires compiled WASM)

### Key Functions

Based on code review, likely high-complexity functions:

1. `lower_expression` (line 371)
   - Large match expression with ~15+ arms
   - Nested conditional logic
   - Type inference and conversion logic
   - **Estimated complexity**: 30-40 (HIGH)

2. `build_symbol_table` (line 236)
   - Recursive AST traversal
   - Multiple match arms
   - **Estimated complexity**: 15-20 (MEDIUM-HIGH)

3. `collect_functions_rec` (line 622)
   - Recursive function collection
   - Match expressions
   - **Estimated complexity**: 12-15 (MEDIUM)

4. `emit` (line 104)
   - Main entry point
   - Complex orchestration logic
   - **Estimated complexity**: 15-20 (MEDIUM-HIGH)

### Quality Issues

#### 1. Structural Complexity (CRITICAL - 0/25 points)

**Problem**: Functions exceed Toyota Way <10 complexity limit

**Evidence**: 
- `lower_expression`: Giant match with 15+ arms, nested conditions
- Multiple functions likely >10 complexity

**Impact**: 
- Hard to maintain
- High bug risk
- Difficult to test thoroughly

#### 2. Code Duplication (15.7/20 - Minor)

**Problem**: Some duplicated patterns

**Likely culprits**:
- Type conversion logic repeated
- Similar match arm patterns
- AST traversal patterns

### Recommendations (Priority Order)

#### P0 - CRITICAL: Refactor `lower_expression`

**Current state**: ~30-40 complexity
**Target**: <10 complexity per function

**Strategy**:
```rust
// BEFORE (simplified)
fn lower_expression(&self, expr: &Expr) -> Result<Vec<Instruction>, String> {
    match &expr.kind {
        ExprKind::Literal(lit) => { /* ... */ }
        ExprKind::Binary { op, left, right } => { /* 60 lines */ }
        ExprKind::Block(exprs) => { /* ... */ }
        ExprKind::If { ... } => { /* 30 lines */ }
        // ... many more arms
    }
}

// AFTER (recommended)
fn lower_expression(&self, expr: &Expr) -> Result<Vec<Instruction>, String> {
    match &expr.kind {
        ExprKind::Literal(lit) => self.lower_literal(lit),
        ExprKind::Binary { op, left, right } => self.lower_binary(op, left, right),
        ExprKind::Block(exprs) => self.lower_block(exprs),
        ExprKind::If { ... } => self.lower_if(condition, then_branch, else_branch),
        // Each helper <10 complexity
    }
}

fn lower_binary(&self, op: &BinaryOp, left: &Expr, right: &Expr) 
    -> Result<Vec<Instruction>, String> {
    let result_type = self.infer_binary_result_type(left, right);
    let instructions = self.emit_operands_with_conversion(left, right, result_type)?;
    let op_instr = self.binary_op_to_instruction(op, result_type)?;
    Ok([instructions, vec![op_instr]].concat())
}
```

#### P1 - HIGH: Extract Helper Functions

**Functions to extract**:
1. Type conversion logic → `emit_type_conversion(from, to)`
2. Binary operation mapping → `binary_op_to_instruction(op, ty)`
3. Type inference → Split into smaller helpers

#### P2 - MEDIUM: Reduce Duplication

**Areas**:
1. Type matching patterns
2. Instruction emission patterns
3. AST traversal patterns

### Success Metrics

**Before**:
- TDG Score: 75.7/100 (B)
- Structural: 0.0/25
- Est. max complexity: 30-40

**Target After Refactoring**:
- TDG Score: >85/100 (A-)
- Structural: >20/25
- Max complexity: <10 per function

**Verification**:
```bash
# After refactoring, verify:
pmat tdg src/backend/wasm/mod.rs --min-grade A-
pmat analyze complexity src/backend/wasm/mod.rs --max-cyclomatic 10
```

### Toyota Way Application

This is a textbook case where **quality must be built in, not bolted on**:

1. **Jidoka**: Automated quality gates detected the issue (TDG 0.0/25)
2. **Genchi Genbutsu**: Direct code inspection confirms complexity violations
3. **Kaizen**: Systematic refactoring to <10 complexity per function
4. **Respect for People**: Create maintainable code that's easy to understand

**HALT THE LINE**: No new features until structural complexity fixed.

### Timeline

**Immediate** (Sprint 1):
- Refactor `lower_expression` to <10 complexity
- Extract 3-5 helper functions
- **Goal**: TDG structural >15/25

**Next Sprint** (Sprint 2):  
- Refactor remaining high-complexity functions
- Reduce duplication
- **Goal**: TDG overall >85 (A-)

---

*Analysis conducted: 2025-10-03*
*PMAT Version: 2.111.0*
