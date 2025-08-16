# Ruchy Master TODO List

This is the **single source of truth** for all Ruchy development tasks.

## 📊 Summary
- **Completed**: 24 features (see `docs/done/completed-features.md`)
- **In Progress**: 0 features
- **Pending**: 24 features
- **Total Progress**: 50% complete

## 🚧 PENDING FEATURES

### Critical Priority (Blocking README Examples)

#### 1. DataFrame Support with Polars
- **Status**: Not Started
- **Priority**: CRITICAL
- **Description**: DataFrame literal syntax and operations shown in README
- **Tasks**:
  - [ ] Add polars to Cargo.toml dependencies
  - [ ] Implement DataFrame literal syntax `df![...]`
  - [ ] Create DataFrame AST node
  - [ ] Implement transpilation to Polars
  - [ ] Add column operations (col, mean, std, alias)
  - [ ] Implement filter, groupby, agg operations
  - [ ] Add comprehensive tests
- **Files to modify**: `src/frontend/lexer.rs`, `src/frontend/parser.rs`, `src/frontend/ast.rs`, `src/backend/transpiler.rs`

#### 2. Lambda Expressions
- **Status**: Not Started
- **Priority**: CRITICAL
- **Description**: Anonymous functions like `|x| x * 2`
- **Tasks**:
  - [ ] Parse lambda syntax with pipes
  - [ ] Handle closure capture analysis
  - [ ] Type inference for lambdas
  - [ ] Transpile to Rust closures
- **Blocked by**: None

#### 3. Result Type with ? Operator
- **Status**: Not Started
- **Priority**: CRITICAL
- **Description**: Error handling with Result<T, E>
- **Tasks**:
  - [ ] Parse ? operator
  - [ ] Implement Result type in type system
  - [ ] Add error propagation in transpiler
  - [ ] Support try/catch syntax (optional)

### High Priority (Core Language Features)

#### 4. Async/Await Support
- **Status**: Not Started
- **Priority**: HIGH
- **Description**: Asynchronous programming
- **Tasks**:
  - [ ] Parse async keyword for functions
  - [ ] Parse await expressions
  - [ ] Add Future type support
  - [ ] Transpile to Rust async/await

#### 5. Actor System
- **Status**: Not Started
- **Priority**: HIGH
- **Description**: Concurrent programming with actors
- **Tasks**:
  - [ ] Parse actor keyword
  - [ ] Implement message passing (!)
  - [ ] Implement synchronous ask (?)
  - [ ] Add supervision trees
  - [ ] Integrate with Bastion or custom implementation

#### 6. Struct Definitions
- **Status**: Not Started
- **Priority**: HIGH
- **Description**: Custom types
- **Tasks**:
  - [ ] Parse struct keyword
  - [ ] Support field definitions
  - [ ] Add visibility modifiers (pub)
  - [ ] Generate Rust structs

#### 7. Impl Blocks
- **Status**: Not Started
- **Priority**: HIGH
- **Description**: Method implementations
- **Tasks**:
  - [ ] Parse impl keyword
  - [ ] Support associated functions
  - [ ] Support methods with self
  - [ ] Handle trait implementations

#### 8. While Loops
- **Status**: Not Started
- **Priority**: HIGH
- **Description**: Basic while loop support
- **Tasks**:
  - [ ] Parse while keyword
  - [ ] Support break/continue
  - [ ] Type check loop conditions

### Medium Priority (Enhanced Features)

#### 9. Trait Definitions
- **Status**: Not Started
- **Priority**: MEDIUM
- **Description**: Trait system for polymorphism
- **Tasks**:
  - [ ] Parse trait keyword
  - [ ] Support associated types
  - [ ] Support default implementations
  - [ ] Trait bounds in generics

#### 10. Pattern Matching Guards
- **Status**: Not Started  
- **Priority**: MEDIUM
- **Description**: if conditions in match arms
- **Tasks**:
  - [ ] Parse if guards in patterns
  - [ ] Type check guard expressions
  - [ ] Transpile to Rust match guards

#### 11. List Comprehensions
- **Status**: Not Started
- **Priority**: MEDIUM
- **Description**: Python-style list comprehensions
- **Tasks**:
  - [ ] Parse [x for x in list] syntax
  - [ ] Support if filters
  - [ ] Transpile to iterator chains

#### 12. Property Testing Attributes
- **Status**: Not Started
- **Priority**: MEDIUM
- **Description**: #[property] for property-based tests
- **Tasks**:
  - [ ] Parse property attributes
  - [ ] Integrate with proptest
  - [ ] Generate property test code

#### 13. Vec Extension Methods
- **Status**: Not Started
- **Priority**: MEDIUM
- **Description**: Methods like sorted(), sum()
- **Tasks**:
  - [ ] Implement sorted() method
  - [ ] Implement sum() method  
  - [ ] Implement is_sorted() method
  - [ ] Add to type inference

### Low Priority (Future Enhancements)

#### 14. MCP Protocol Integration
- **Status**: Not Started
- **Priority**: LOW
- **Description**: AI/LLM tool support
- **Tasks**:
  - [ ] Parse MCP attributes
  - [ ] Generate MCP protocol bindings
  - [ ] Create MCP server implementation

#### 15. Refinement Types
- **Status**: Not Started
- **Priority**: LOW
- **Description**: SMT verification
- **Tasks**:
  - [ ] Parse #[ensures] attributes
  - [ ] Integrate SMT solver
  - [ ] Verify refinement predicates

#### 16. JIT Compilation
- **Status**: Not Started
- **Priority**: LOW
- **Description**: <10ms REPL startup
- **Tasks**:
  - [ ] Integrate Cranelift or similar
  - [ ] Cache compiled code
  - [ ] Optimize startup time

#### 17. Object Literals
- **Status**: Not Started
- **Priority**: LOW
- **Description**: JavaScript-style object literals
- **Tasks**:
  - [ ] Parse { key: value } syntax
  - [ ] Type inference for objects
  - [ ] Transpile to Rust structs

#### 18. Try/Catch Syntax
- **Status**: Not Started
- **Priority**: LOW
- **Description**: Exception-style error handling
- **Tasks**:
  - [ ] Parse try/catch blocks
  - [ ] Convert to Result types
  - [ ] Handle finally blocks

#### 19. Generic Type Parameters
- **Status**: Not Started
- **Priority**: LOW
- **Description**: Full generics support
- **Tasks**:
  - [ ] Parse <T> syntax
  - [ ] Type parameter bounds
  - [ ] Generic inference

#### 20. Row Polymorphism
- **Status**: Not Started
- **Priority**: LOW
- **Description**: Extensible records
- **Tasks**:
  - [ ] Implement row types
  - [ ] Record extension syntax
  - [ ] Type inference for rows

#### 21. Package Manager
- **Status**: Not Started
- **Priority**: LOW
- **Description**: Ruchy package management
- **Tasks**:
  - [ ] Design package format
  - [ ] Create registry
  - [ ] Implement dependency resolution

#### 22. Language Server (LSP)
- **Status**: Not Started
- **Priority**: LOW
- **Description**: IDE support
- **Tasks**:
  - [ ] Implement LSP protocol
  - [ ] Add completion support
  - [ ] Add go-to-definition
  - [ ] Add refactoring support

#### 23. Debugger Support
- **Status**: Not Started
- **Priority**: LOW
- **Description**: Debugging capabilities
- **Tasks**:
  - [ ] Generate source maps
  - [ ] Integrate with lldb/gdb
  - [ ] Add breakpoint support

#### 24. WebAssembly Target
- **Status**: Not Started
- **Priority**: LOW
- **Description**: Compile to WASM
- **Tasks**:
  - [ ] Add WASM backend
  - [ ] Handle WASM-specific features
  - [ ] Create JavaScript bindings

## 📈 Progress Tracking

### By Priority
- **CRITICAL**: 0/3 complete (0%)
- **HIGH**: 0/5 complete (0%)
- **MEDIUM**: 0/5 complete (0%)
- **LOW**: 0/11 complete (0%)

### By Component
- **Parser**: 3 pending features
- **Type System**: 4 pending features
- **Transpiler**: 5 pending features
- **Runtime**: 3 pending features
- **Tooling**: 4 pending features
- **Experimental**: 5 pending features

## 🎯 Next Sprint (Recommended Order)

1. **DataFrame Support** - Most critical for README examples
2. **Lambda Expressions** - Enables functional programming
3. **Result Type** - Proper error handling
4. **Async/Await** - Modern async programming
5. **Structs & Impl** - Custom types

## 📝 Notes

- All completed features have been moved to `docs/done/completed-features.md`
- This file should be updated whenever a feature is started, completed, or reprioritized
- Each feature should maintain zero SATD policy
- Minimum 80% test coverage for new features
- All features must pass PMAT quality gates