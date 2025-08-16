# Ruchy Master TODO List

This is the **single source of truth** for all Ruchy development tasks.

## 📊 Summary
- **Completed**: 45 features (see `docs/done/`)
- **In Progress**: 0 features  
- **Pending**: 17 features
- **Total Progress**: 72.6% complete
- **Coverage**: 78.25%

## ✅ RECENTLY COMPLETED (Latest Session)

1. **Property Testing Attributes** - #[property] syntax for property-based testing ✅
2. **List Comprehensions** - Python-style [x for x in list] syntax with filters ✅
3. **Async/Await Support** - Modern async programming ✅
4. **Vec Extension Methods** - sorted(), sum(), reversed(), unique(), min(), max() ✅
5. **Try/Catch Syntax** - Exception-style error handling ✅
6. **Coverage Command Fix** - Fixed make coverage to generate both HTML and LCOV ✅

*Property Testing and List Comprehensions completed with full parsing, type inference, transpilation, and comprehensive test coverage*

## 🚧 PENDING FEATURES

### Critical Priority (Core Language)

#### 1. Actor System
- **Status**: Not Started
- **Priority**: CRITICAL
- **Description**: Concurrent programming with actors
- **Tasks**:
  - [ ] Parse actor keyword
  - [ ] Implement message passing (!)
  - [ ] Implement synchronous ask (?)
  - [ ] Add supervision trees
  - [ ] Integrate with Bastion

#### 2. DataFrame Column Operations
- **Status**: Not Started (basic DataFrame support exists)
- **Priority**: CRITICAL
- **Description**: Complete DataFrame operations
- **Tasks**:
  - [ ] Implement col() function
  - [ ] Add mean, std, alias operations
  - [ ] Implement filter operation
  - [ ] Implement groupby operation
  - [ ] Add agg operations

### High Priority (Language Features)

#### 3. Impl Blocks
- **Status**: Not Started
- **Priority**: HIGH
- **Description**: Method implementations for structs
- **Tasks**:
  - [ ] Parse impl keyword
  - [ ] Support associated functions
  - [ ] Support methods with self
  - [ ] Handle trait implementations

#### 4. Trait Definitions
- **Status**: Not Started (basic parsing exists)
- **Priority**: HIGH
- **Description**: Full trait system
- **Tasks**:
  - [ ] Support associated types
  - [ ] Support default implementations
  - [ ] Trait bounds in generics
  - [ ] Trait objects

#### 5. Pattern Matching Guards
- **Status**: Not Started
- **Priority**: HIGH
- **Description**: if conditions in match arms
- **Tasks**:
  - [ ] Parse if/when guards in patterns
  - [ ] Type check guard expressions
  - [ ] Transpile to Rust match guards

#### 6. Break/Continue in Loops
- **Status**: Not Started
- **Priority**: HIGH
- **Description**: Loop control flow
- **Tasks**:
  - [ ] Parse break/continue keywords
  - [ ] Type check in loop context
  - [ ] Support labeled breaks
  - [ ] Transpile to Rust

### Medium Priority (Enhanced Features)

#### 7. Property Testing Attributes ✅ COMPLETED
- **Status**: COMPLETED
- **Priority**: MEDIUM
- **Description**: #[property] for property-based tests
- **Tasks**:
  - [x] Parse property attributes
  - [x] Integrate with proptest
  - [x] Generate property test code
  - [x] Support custom generators

#### 8. List Comprehensions ✅ COMPLETED
- **Status**: COMPLETED
- **Priority**: MEDIUM
- **Description**: Python-style list comprehensions
- **Tasks**:
  - [x] Parse [x for x in list] syntax
  - [x] Support if filters
  - [x] Transpile to iterator chains
  - [ ] Support nested comprehensions (future enhancement)

#### 9. Generic Type Parameters
- **Status**: Not Started
- **Priority**: MEDIUM
- **Description**: Full generics support
- **Tasks**:
  - [ ] Parse <T> syntax
  - [ ] Type parameter bounds
  - [ ] Generic inference
  - [ ] Associated type projections

#### 10. Object Literals
- **Status**: Not Started
- **Priority**: MEDIUM
- **Description**: JavaScript-style object literals
- **Tasks**:
  - [ ] Parse { key: value } syntax
  - [ ] Type inference for objects
  - [ ] Spread operator support
  - [ ] Transpile to Rust structs

### Low Priority (Future Enhancements)

#### 11. MCP Protocol Integration
- **Status**: Not Started
- **Priority**: LOW
- **Description**: AI/LLM tool support via MCP
- **Tasks**:
  - [ ] Parse MCP attributes
  - [ ] Generate MCP protocol bindings
  - [ ] Create MCP server implementation
  - [ ] Bridge actors to MCP tools

#### 12. Refinement Types
- **Status**: Not Started
- **Priority**: LOW
- **Description**: SMT verification
- **Tasks**:
  - [ ] Parse #[ensures] attributes
  - [ ] Integrate Z3 or similar SMT solver
  - [ ] Verify refinement predicates
  - [ ] Generate proof obligations

#### 13. JIT Compilation for REPL
- **Status**: Not Started
- **Priority**: LOW
- **Description**: <10ms REPL startup
- **Tasks**:
  - [ ] Integrate Cranelift or LLVM JIT
  - [ ] Cache compiled code
  - [ ] Incremental compilation
  - [ ] Hot code reload

#### 14. Row Polymorphism
- **Status**: Not Started
- **Priority**: LOW
- **Description**: Extensible records
- **Tasks**:
  - [ ] Implement row types
  - [ ] Record extension syntax
  - [ ] Type inference for rows
  - [ ] Transpile to Rust enums/structs

#### 15. Package Manager
- **Status**: Not Started
- **Priority**: LOW
- **Description**: Ruchy package management
- **Tasks**:
  - [ ] Design package format (.ruchy files)
  - [ ] Create package registry
  - [ ] Implement dependency resolution
  - [ ] Integration with crates.io

#### 16. Language Server (LSP)
- **Status**: Not Started
- **Priority**: LOW
- **Description**: IDE support
- **Tasks**:
  - [ ] Implement LSP protocol
  - [ ] Add completion support
  - [ ] Add go-to-definition
  - [ ] Add refactoring support

#### 17. Debugger Support
- **Status**: Not Started
- **Priority**: LOW
- **Description**: Debugging capabilities
- **Tasks**:
  - [ ] Generate source maps
  - [ ] Integrate with lldb/gdb
  - [ ] Add breakpoint support
  - [ ] Stack trace translation

#### 18. WebAssembly Target
- **Status**: Not Started
- **Priority**: LOW
- **Description**: Compile to WASM
- **Tasks**:
  - [ ] Add WASM backend
  - [ ] Handle WASM-specific features
  - [ ] Create JavaScript bindings
  - [ ] Optimize for size

#### 19. Incremental Compilation
- **Status**: Not Started
- **Priority**: LOW
- **Description**: Faster rebuilds
- **Tasks**:
  - [ ] Implement dependency tracking
  - [ ] Cache intermediate results
  - [ ] Partial recompilation
  - [ ] Module-level caching

## 📈 Progress Tracking

### By Priority
- **CRITICAL**: 0/2 complete (0%)
- **HIGH**: 2/4 partially complete (50%)
- **MEDIUM**: 2/4 complete (50%)
- **LOW**: 0/9 complete (0%)

### By Component
- **Parser**: 6 pending features
- **Type System**: 4 pending features
- **Transpiler**: 5 pending features
- **Runtime**: 2 pending features
- **Tooling**: 2 pending features

## 🎯 Next Sprint Priority Order

1. **Generic Type Parameters** - Essential for type system completeness
2. **Object Literals** - JavaScript-style object syntax
3. **Actor System** - Essential for concurrent programming model
4. **DataFrame Column Operations** - Complete the DataFrame support
5. **Impl Blocks** - Methods for structs

## 📝 Implementation Guidelines

- **Zero SATD Policy**: No TODO/FIXME/HACK comments in code
- **Test Coverage**: Minimum 80% for all new features
- **Complexity**: Cyclomatic complexity ≤10 per function
- **Documentation**: Every public API must have doctests
- **Quality Gate**: All features must pass PMAT validation

## 🗂️ Archived TODO Files

The following TODO files have been consolidated into this master file and should be archived:
- `docs/todo/v0.3-todo.yaml` - Merged, contains duplicate tasks
- `docs/todo/ruchy-disassembly-todo.yaml` - Low priority, keep for reference
- `docs/todo/repl-todo.yaml` - Merged into JIT Compilation task
- `docs/todo/ruchy-visual-design-hello-world-todo.yaml` - Documentation task, separate concern

## ⚡ Quick Command Reference

```bash
# Run all tests
make test

# Check coverage (now fixed!)
make coverage

# Run linting
make lint

# Run specific test
cargo test test_name

# Generate coverage report
cargo llvm-cov --html --output-dir target/coverage/html
```

---
*Last Updated: 2025-08-16 (Post List Comprehensions & Property Testing completion)*
*Next Review: When starting Generic Type Parameters implementation*