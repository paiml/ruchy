# Interpreter Quality Refactoring Plan

## Critical Issue: evaluate_expr Complexity

Current state:
- **Cyclomatic Complexity**: 209 (limit: 50)
- **Lines**: 400+
- **Branches**: 30+ match arms

## Root Cause Analysis (Toyota Way)

The evaluate_expr function violates the Single Responsibility Principle by handling:
1. Literals and variables
2. Binary and unary operations
3. Control flow (if, match, for, while)
4. Functions and lambdas
5. Collections (lists, tuples, dataframes)
6. Method calls
7. Error handling (try/catch)
8. String interpolation
9. Pipeline operations

## Refactoring Strategy

### Phase 1: Extract Helper Functions (Target Complexity: <10 each)

```rust
// Current monolithic structure
fn evaluate_expr(&mut self, expr: &Expr, deadline: Instant, depth: usize) -> Result<Value> {
    match &expr.kind {
        // 30+ branches...
    }
}

// Refactored structure
fn evaluate_expr(&mut self, expr: &Expr, deadline: Instant, depth: usize) -> Result<Value> {
    self.check_bounds(deadline, depth)?;
    
    match &expr.kind {
        // Literals and simple values
        ExprKind::Literal(lit) => self.evaluate_literal(lit),
        ExprKind::Identifier(name) => self.lookup_variable(name),
        
        // Operations
        ExprKind::Binary { .. } => self.evaluate_binary_expr(expr, deadline, depth),
        ExprKind::Unary { .. } => self.evaluate_unary_expr(expr, deadline, depth),
        
        // Control flow
        ExprKind::If { .. } => self.evaluate_if_expr(expr, deadline, depth),
        ExprKind::Match { .. } => self.evaluate_match_expr(expr, deadline, depth),
        ExprKind::For { .. } => self.evaluate_for_loop(expr, deadline, depth),
        ExprKind::While { .. } => self.evaluate_while_loop(expr, deadline, depth),
        
        // Functions
        ExprKind::Function { .. } => self.evaluate_function_def(expr),
        ExprKind::Lambda { .. } => self.evaluate_lambda(expr),
        ExprKind::Call { .. } => self.evaluate_call_expr(expr, deadline, depth),
        
        // Collections
        ExprKind::List(..) => self.evaluate_list(expr, deadline, depth),
        ExprKind::Tuple(..) => self.evaluate_tuple(expr, deadline, depth),
        
        // Advanced features
        ExprKind::Pipeline { .. } => self.evaluate_pipeline_expr(expr, deadline, depth),
        ExprKind::MethodCall { .. } => self.evaluate_method_call(expr, deadline, depth),
        
        _ => self.evaluate_other_expr(expr, deadline, depth),
    }
}
```

### Phase 2: Extract Complex Method Calls

The MethodCall branch alone is 150+ lines. Break into:
- `evaluate_list_methods()` - map, filter, reduce
- `evaluate_string_methods()` - len, trim, split
- `evaluate_dataframe_methods()` - select, filter, groupby

### Phase 3: Simplify Control Flow

Extract loop body evaluation:
- `evaluate_for_iteration()`
- `evaluate_while_condition()`
- `restore_loop_bindings()`

## Quality Metrics Goals

| Function | Current | Target | Max |
|----------|---------|--------|-----|
| evaluate_expr | 209 | 20 | 50 |
| evaluate_method_call | N/A | 10 | 20 |
| evaluate_for_loop | N/A | 10 | 15 |
| evaluate_match_expr | N/A | 10 | 15 |
| Value::fmt | 66 | 15 | 30 |

## Testing Strategy

1. **Before Each Extraction**:
   - Run full test suite
   - Record performance baseline
   - Save coverage report

2. **After Each Extraction**:
   - Verify all tests pass
   - Check performance hasn't degraded
   - Ensure coverage maintained

3. **Final Validation**:
   - Run PMAT analysis
   - Verify all functions < 50 complexity
   - Run stress tests

## Implementation Order

1. Extract `check_bounds()` - Resource limit checking
2. Extract `lookup_variable()` - Variable resolution
3. Extract `evaluate_binary_expr()` - Binary operations
4. Extract `evaluate_control_flow()` - If/Match/Loops
5. Extract `evaluate_collections()` - Lists/Tuples
6. Extract `evaluate_functions()` - Function/Lambda/Call
7. Extract `evaluate_method_call()` - Method dispatch
8. Simplify main `evaluate_expr()` to dispatcher

## Success Criteria (Toyota Way)

- [ ] All functions < 50 cyclomatic complexity
- [ ] All functions < 100 lines
- [ ] 100% test coverage maintained
- [ ] No performance regression
- [ ] Zero new bugs introduced
- [ ] Code is more readable and maintainable