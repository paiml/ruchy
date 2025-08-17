# Ruchy Development Roadmap

**Single Source of Truth - Execution Focused**

## 📊 Current Status (2025-01-17)

- **Version**: v0.3.1
- **Tests**: 203/229 passing (88.6%)
- **Blocking Issues**: 3 core problems causing 26 test failures
- **Time to Resolution**: 36 hours focused work

## 🔴 Critical Path (36 Hours Total)

### Day 1: Parser Fixes (4 hours)

#### Struct Literal Disambiguation
```rust
// parser/expressions.rs - Add 3-token lookahead
fn peek_is_struct_literal(&self) -> bool {
    matches!(self.peek(), Token::Ident(_)) 
        && matches!(self.peek2(), Token::LBrace)
        && self.peek_nth_is_colon(3)  // Check for ':' after first field
}
```

#### Semicolon Insertion
- Fix: Multiple statements need automatic semicolon insertion
- Location: `parser/core.rs:58-67`
- Solution: Check for expression-terminating tokens

#### Operator Precedence
- Fix: Pipeline operator conflicts with bitwise OR
- Location: `parser/expressions.rs:121`
- Solution: Separate precedence table entry

### Day 2-3: Type System (16 hours)

#### Constraint Propagation
```rust
// middleend/infer.rs - Fix lambda inference
impl InferenceContext {
    fn infer_lambda(&mut self, params: &[Param], body: &Expr) -> Result<MonoType> {
        self.push_scope();
        let param_tys: Vec<TyVar> = params.iter()
            .map(|_| self.fresh_var())
            .collect();
        
        for (param, ty) in params.iter().zip(&param_tys) {
            self.env.bind(param.name.clone(), ty.clone());
        }
        
        let body_ty = self.infer_expr(body)?;
        self.pop_scope();
        
        Ok(MonoType::Function(param_tys, Box::new(body_ty)))
    }
}
```

#### Bidirectional Type Checking
- Implement check mode vs infer mode
- Required for: Lambda parameters, match arms
- Location: Create `middleend/bidir.rs`

### Day 4: Error Handling (8 hours)

#### Result Type Mapping
```rust
// backend/transpiler.rs - Fix try/catch
fn transpile_try_catch(&self, try_block: &Expr, catch_var: &str, catch_block: &Expr) -> TokenStream {
    quote! {
        (|| -> Result<_, Box<dyn std::error::Error>> {
            Ok(#try_block)
        })().unwrap_or_else(|#catch_var| #catch_block)
    }
}
```

#### Try Operator
- Map `expr?` to proper Result propagation
- Location: `backend/transpiler.rs:230`

### Day 5: Integration Testing (8 hours)

#### Test Suite Fixes
1. Fix spacing issues with `rustfmt` integration
2. Update assertions to match actual output
3. Add round-trip tests (parse → transpile → compile → run)

## ❌ Frozen Features (Do Not Touch)

These features remain incomplete until foundation is fixed:

1. **Actor System** - Message types need type inference
2. **DataFrame Operations** - Requires trait abstraction
3. **MCP Integration** - Depends on actor completion
4. **Package Manager** - Premature without stable core

## ✅ Completed This Session

### Parser
- [x] Let statements without `in`
- [x] Lambda parameter parsing `|x|`
- [x] Pipeline operator `|>`
- [x] Object literals with spread

### Transpiler
- [x] Async/await (both forms)
- [x] Import → use statements
- [x] Empty blocks → `()`

### Quality
- [x] 373 lint warnings fixed
- [x] Zero SATD policy

## 📋 Failing Test Analysis

### Parser Issues (6 tests)
- `test_compile_struct_literal` - Needs lookahead
- `test_compile_multiple_statements` - Semicolon insertion
- `test_compile_pattern_matching` - List patterns
- `test_compile_lambda` - Type inference
- `test_compile_empty_list` - Already works, test is wrong
- `test_compile_list` - Spacing issue only

### Type System Issues (4 tests)
- `test_infer_lambda` - Constraint propagation
- `test_compile_trait` - Trait bounds
- `test_transpile_async` - Async type inference
- `test_repl_v2_variable_persistence` - Type environment

### Transpiler Issues (16 tests)
- `test_transpile_try_catch` - Result mapping
- `test_transpile_try_operator` - Error propagation
- `test_transpile_range` - Range operators
- `test_transpile_send` - Actor messages
- `test_transpile_ask` - Actor responses
- `test_transpile_dataframe_operations` - Trait abstraction
- `test_transpile_col_function` - DataFrame column
- `test_transpile_list_comprehension` - Spacing only
- Plus 8 cascading failures from above

## 🎯 Implementation Order

1. **Hour 0-4**: Parser fixes (struct literal, semicolons, precedence)
2. **Hour 4-20**: Type system (inference, constraints, bidirectional)
3. **Hour 20-28**: Error handling (Result, try/catch, operator)
4. **Hour 28-36**: Test suite (spacing, assertions, round-trip)

## 📐 Technical Decisions

### No Visitor Pattern
Pattern matching with `Box` is more efficient than vtable dispatch.

### No Separate Semantic Phase
Type checking during lowering, like rustc.

### No Incremental Compilation (Yet)
REPL needs persistent definitions, not incremental parsing.

### DataFrame Trait Architecture
```rust
trait DataFrame: Sized {
    type Column;
    fn col(&self, name: &str) -> Self::Column;
}
```

## 📊 Success Metrics

- **Test Pass Rate**: 100% (currently 88.6%)
- **Code Coverage**: >80% weighted by complexity
- **Cyclomatic Complexity**: ≤10 per function
- **Build Time**: <30 seconds
- **REPL Startup**: <100ms

## 🚫 Not Doing

- Visitor pattern refactoring
- Separate semantic analysis phase  
- Actor system improvements
- DataFrame direct Polars coupling
- Package manager design
- LSP implementation

## 📅 Timeline

- **Day 1**: Parser fixes complete, 210/229 tests passing
- **Day 2-3**: Type system complete, 220/229 tests passing
- **Day 4**: Error handling complete, 226/229 tests passing
- **Day 5**: All tests passing, ready for v0.4.0

---
*Last Updated: 2025-01-17*
*Estimated Completion: 2025-01-22*
*Total Work: 36 hours*