# Ruchy Development Roadmap

**Single Source of Truth - Execution Focused**

## 📊 Current Status (2025-01-17)

### Build Status
- **`make lint`**: ❌ FAILING (134 clippy errors)
  - 18 duplicate match arms
  - 16 unwrap() on Result
  - 15 missing Error docs
  - 11 unnecessary pass-by-value
  - 10 expect() on Result
  - 9 unnecessary Result wrapping
  - 20+ misc violations
- **`make test`**: ❌ FAILING (203/229 passing - 88.6%)
- **`cargo build`**: ✅ PASSING

### Key Metrics
- **Version**: v0.3.1
- **Tests**: 203/229 passing (26 failures)
- **Lint Errors**: 134 clippy violations
- **Blocking Issues**: 3 core problems causing cascading failures
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

## 📚 Documentation Index

### Core Specifications (15 files)
- `docs/ruchy-lexer-spec.md` - Tokenization rules
- `docs/ruchy-repl-spec.md` - REPL behavior
- `docs/ruchy-binary-spec.md` - Binary compilation
- `docs/ruchy-transpiler-docs.md` - Transpiler architecture
- `docs/ruchy-edge-cases-spec.md` - Edge case handling
- `docs/ruchy-disassembly-spec.md` - Bytecode disassembly
- `docs/ruchy-cargo-integration.md` - Cargo integration
- `docs/ruchy-lsp-spec.md` - Language server protocol
- `docs/ruchy-oneliner-spec.md` - One-liner syntax
- `docs/ruchy-visual-design-hello-world-spec.md` - UI/UX design
- `docs/docker-spec.md` - Container deployment
- `docs/name-resolution-spec.md` - Name resolution rules
- `docs/ruchy-missing-specs.md` - Specification gaps
- `docs/ruchy-repl-testing-spec.md` - REPL test strategy
- `docs/ERROR_RECOVERY_IMPLEMENTATION.md` - Error recovery

### Architecture Documents (6 files)
- `docs/architecture/ruchy-design-architectur.md` - Overall design
- `docs/architecture/grammer.md` - Grammar specification
- `docs/architecture/message-passing-mcp.md` - MCP integration
- `docs/architecture/depyler-integration.md` - Python transpiler
- `docs/architecture/quality-proxy.md` - Quality gates
- `docs/architecture/script-capabilities.md` - Scripting features

### Project Management (8 files)
- `docs/ROADMAP.md` - **THIS FILE** (master plan)
- `docs/project-management/CLAUDE.md` - AI implementation guide
- `docs/project-management/CHANGELOG.md` - Version history
- `docs/project-management/QA_SUMMARY.md` - Quality summary
- `docs/project-management/QUICK_START.md` - Getting started
- `docs/project-management/PUBLISH_INSTRUCTIONS.md` - Publishing guide
- `docs/project-management/REPL_DEMONSTRATION.md` - REPL examples
- `docs/PROJECT-STATUS.md` - Overall status

### Completed Work (10+ files)
- `docs/done/session-2025-01-17-completed.md` - Today's work
- `docs/done/session-2025-01-16-completed.md` - Yesterday's work
- `docs/done/v0.2-completed-features.md` - v0.2 features
- `docs/done/implementation-status.md` - Implementation status
- `docs/done/completed-features.md` - Feature list
- `docs/done/lambda-feature-completed.yaml` - Lambda implementation
- `docs/done/coverage-improvements-completed.yaml` - Coverage work
- `docs/done/archived-todos/` - 20+ archived todo files

### Quality & Process (3 files)
- `docs/quality/QUALITY_REPORT.md` - Quality metrics
- `docs/process/RELEASE_PROCESS.md` - Release workflow
- `docs/releases/RELEASE_NOTES_v0.3.0.md` - v0.3 notes

### Guides & Reports (4 files)
- `docs/README.md` - Documentation overview
- `docs/REPL_GUIDE.md` - REPL user guide
- `docs/IMPLEMENTATION_REPORT.md` - Implementation details
- `docs/bugs/repl-qa-report.md` - Bug reports

### Total: 65+ documentation files organized across 8 categories

---
*Last Updated: 2025-01-17*
*Estimated Completion: 2025-01-22*
*Total Work: 36 hours*