# CLAUDE.md - Ruchy Language Implementation Guide

## Project Context

You are implementing **Ruchy**, a systems-oriented scripting language that transpiles to Rust. This is a greenfield compiler project requiring deep understanding of type theory, parsing techniques, and Rust's ownership model.

## Critical Implementation Details

### Core Technical Challenges

1. **Type Inference Engine**: Implementing bidirectional type checking with Hindley-Milner inference extended with row polymorphism. The challenge is maintaining inference speed while supporting refinement types.

2. **Zero-Cost Transpilation**: Every Ruchy construct must map to idiomatic Rust without runtime overhead. Pipeline operators, pattern matching, and actor messages need careful lowering strategies.

3. **Incremental Compilation**: The REPL requires maintaining type environment across inputs while supporting hot-reload of modified modules.

## Current Implementation State

### What Exists
- Comprehensive specification documents (see attached)
- Grammar definition in EBNF
- Architecture decisions for MCP integration
- Quality gate requirements via PMAT

### What Needs Building
```rust
// Priority 1: Parser (Week 1-2)
src/frontend/
├── lexer.rs       // Token stream generation
├── parser.rs      // Recursive descent + Pratt
├── ast.rs         // AST definition
└── span.rs        // Source location tracking

// Priority 2: Type System (Week 3-4)  
src/middleend/
├── infer.rs       // Algorithm W implementation
├── types.rs       // Type representation
├── elaborate.rs   // Elaboration to typed AST
└── refinement.rs  // SMT integration for refinements

// Priority 3: Code Generation (Week 5-6)
src/backend/
├── rust_gen.rs    // AST to syn::Item conversion
├── optimize.rs    // Peephole optimizations
└── pretty.rs      // Code formatting
```

## Key Algorithms to Implement

### Parser: Pratt Parsing for Operators
```rust
fn parse_expr(&mut self, min_prec: i32) -> Expr {
    let mut left = self.parse_prefix()?;
    
    while let Some(op) = self.peek_binop() {
        if op.precedence() < min_prec { break; }
        self.advance();
        let right = self.parse_expr(op.precedence() + op.associativity())?;
        left = Expr::Binary(Box::new(left), op, Box::new(right));
    }
    left
}
```

### Type Inference: Level-Based Generalization
```rust
struct InferCtx {
    level: u32,
    subst: HashMap<TyVar, Type>,
}

fn generalize(&self, ty: Type) -> TypeScheme {
    let free_vars = ty.free_vars()
        .filter(|v| self.var_level(v) > self.level)
        .collect();
    TypeScheme::Poly(free_vars, ty)
}
```

### Escape Analysis for Memory Management
```rust
enum Lifetime {
    Stack(ScopeId),
    Heap(RefCount),
}

fn analyze_escape(&self, expr: &Expr) -> Lifetime {
    match expr {
        Expr::Lambda(_, body) if self.captures_escape(body) => 
            Lifetime::Heap(RefCount::Arc),
        Expr::Let(_, val, body) if !self.escapes_to(val, body) =>
            Lifetime::Stack(self.current_scope()),
        _ => self.conservative_heap()
    }
}
```

## Performance Requirements

### Benchmarks to Beat
- Parser: 100k LOC/sec (matching rustc's parser)
- Type checking: 50k LOC/sec  
- Transpilation: 200k LOC/sec
- REPL latency: <15ms for typical expressions
- Binary size: <5MB with minimal runtime

### Memory Budget
```rust
// Per-actor overhead
const ACTOR_STACK: usize = 2 * 1024 * 1024;  // 2MB
const MAILBOX_SIZE: usize = 64 * 1024;       // 64KB
const MESSAGE_INLINE: usize = 496;           // Bytes before boxing
```

## Testing Strategy

### Property Tests (Required)
```rust
#[quickcheck]
fn parse_print_roundtrip(ast: Ast) -> bool {
    let printed = pretty_print(&ast);
    let reparsed = parse(&printed).unwrap();
    ast == reparsed
}

#[quickcheck]
fn type_preservation(expr: TypedExpr) -> bool {
    let rust_code = transpile(&expr);
    let rust_type = infer_rust_type(&rust_code);
    expr.ty() == from_rust_type(rust_type)
}
```

### Fuzzing Targets
- Parser with arbitrary byte sequences
- Type inference with ill-typed programs  
- Actor mailbox with message floods

## Common Pitfalls to Avoid

1. **Don't reinvent Rust's type system** - Delegate to rustc for actual type checking
2. **Avoid intermediate representations** - Go directly from Ruchy AST to syn AST
3. **Don't implement custom memory management** - Use Rust's ownership with escape analysis hints
4. **Pipeline operators are syntax sugar** - Transform immediately to method chains

## Integration Points

### Cargo Integration
```toml
# In user's Cargo.toml
[build-dependencies]
ruchy = "1.0"

# build.rs
fn main() {
    ruchy::compile_glob("src/**/*.ruchy")
        .output_dir("target/ruchy_gen")
        .execute()?;
}
```

### MCP Protocol Bridge
```rust
// Actor automatically becomes MCP tool
#[actor(mcp_tool = "analyzer")]
impl ContextAnalyzer {
    #[mcp_handler]
    async fn analyze(&mut self, code: String) -> AnalysisResult {
        // Implementation
    }
}
```

## Quality Philosophy: The Toyota Way

### Core Principles
1. **Kaizen (Continuous Improvement)**: Every commit improves the codebase
2. **Jidoka (Built-in Quality)**: Quality is built in, not inspected in  
3. **Genchi Genbutsu (Go and See)**: Understand by direct observation and testing
4. **Respect for People**: Write code that respects future maintainers

### Zero Tolerance Policy
- **NO SATD (Self-Admitted Technical Debt)**: Zero TODO/FIXME/HACK/XXX comments
- **NO Broken Windows**: Fix issues immediately, don't accumulate debt
- **NO Shortcuts**: Do it right the first time
- If something needs doing later, track it in GitHub Issues, not code comments

### The Andon Cord Principle
Stop the line when quality issues are detected:
- Failing tests block all progress
- Linter warnings are errors
- Coverage drops are unacceptable
- Performance regressions trigger immediate fixes

## Quality Gates (via PMAT MCP Integration)

**MANDATORY**: All code must be developed using PMAT quality proxy via MCP (v2.4.0). Every commit must pass:

### MCP Server Configuration
PMAT MCP server is configured at `~/.config/claude/mcps/pmat.json` and provides:
- Real-time code quality analysis during development
- Automated quality gate enforcement
- Test coverage monitoring (minimum 80%)
- Complexity metrics tracking (cyclomatic complexity ≤10)
- Property test validation (>80% coverage)
- Zero SATD enforcement (no TODO/FIXME/HACK/XXX comments)

To verify PMAT MCP is running:
```bash
~/.local/bin/pmat serve --mode mcp --port 0
```

### PMAT Quality Requirements
- Cyclomatic complexity ≤10 per function
- Zero SATD comments (TODO/FIXME/HACK)
- Mutation score >75%
- Cognitive complexity ≤15 per function
- Halstead effort ≤5000 per function
- Maintainability index >70

### Test Coverage Requirements (80% Minimum)
- **Unit test coverage**: 80% minimum for all modules (measured with `cargo llvm-cov`)
- **Property test coverage**: Every public function must have property tests
- **Fuzz test coverage**: All parsers and serializers must have fuzz tests
- **Doctest coverage**: Every public API must have executable doctests
- **Integration tests**: Full end-to-end tests for all major workflows
- **Example coverage**: Every feature must have runnable examples via `cargo run --example`

### Code Coverage Measurement
**MANDATORY**: Use `cargo-llvm-cov` for all coverage measurements:
```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Generate coverage report
cargo llvm-cov --html --output-dir target/llvm-cov-html

# Check coverage percentage
cargo llvm-cov --summary-only

# Coverage with all test types
cargo llvm-cov --all-features --workspace --doctests
```

### Testing Implementation
```rust
// Every module must have:
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    // Unit tests
    #[test]
    fn test_basic_functionality() { /* ... */ }
    
    // Property tests
    proptest! {
        #[test]
        fn prop_invariant_holds(input: ArbitraryInput) {
            // Verify invariants
        }
    }
}

// Fuzz targets in fuzz/
#[cfg(fuzzing)]
pub fn fuzz_parser(data: &[u8]) {
    let _ = parse(data);
}

// Doctests in public APIs
/// Parse a Ruchy expression.
/// 
/// # Examples
/// ```
/// use ruchy::parse_expr;
/// let expr = parse_expr("x + 1")?;
/// assert_eq!(expr.node_type(), NodeType::Binary);
/// ```
pub fn parse_expr(input: &str) -> Result<Expr> { /* ... */ }
```

### Example Requirements
Every major feature must have a corresponding example in `examples/`:
```bash
cargo run --example parser_demo
cargo run --example type_inference
cargo run --example actor_system
cargo run --example mcp_integration
cargo run --example repl_session
```

### Continuous Quality Monitoring
- PMAT MCP tool runs on every file save during development
- Pre-commit hooks enforce all quality gates
- CI/CD pipeline blocks merges below 80% coverage
- Weekly quality reports generated via PMAT dashboard

## Architecture Decisions

### Why Not LLVM Backend Initially?
- Rust transpilation gives us borrow checking for free
- Debugging is easier with readable Rust output
- Cargo ecosystem works without modifications
- LLVM can be added later as optimization

### Actor Model Implementation
- Use Bastion for supervision trees
- Each actor owns its mailbox (no shared memory)
- Message passing via channels, not function calls
- Selective receive via pattern matching on message queue

## Next Steps

1. **Implement parser** - Start with expressions, add statements incrementally
2. **Build type inference** - Begin with simply-typed lambda calculus, add polymorphism
3. **Generate Rust code** - Use quote! and syn for AST construction
4. **Create REPL** - Use rustyline for readline, incremental compilation for state

## Code Style Guidelines

- Use `anyhow` for error handling in CLI tools
- Use `thiserror` for library error types
- Prefer `match` over `if let` chains
- Document invariants with debug_assert!
- Every public API needs doctests

## External Dependencies

```toml
[dependencies]
# Parsing
logos = "0.14"        # Lexer generator
chumsky = "0.9"       # Parser combinators (alternative to hand-written)

# Code generation  
syn = "2.0"
quote = "1.0"
proc-macro2 = "1.0"

# Type checking
ena = "0.14"          # Union-find for unification

# Runtime
bastion = "0.4"       # Actor supervision
crossbeam = "0.8"     # Lock-free data structures

# Quality
proptest = "1.0"
criterion = "0.5"
```

## Questions to Consider

1. Should we support gradual typing from day 1 or add it later?
2. How much syntax sugar is too much? (Current spec has ~15 desugarings)
3. Should actors compile to threads or async tasks?
4. Do we need our own package manager or piggyback on Cargo?

## File Structure for Implementation

```
ruchy/
├── Cargo.toml
├── src/
│   ├── main.rs           # CLI entry point
│   ├── lib.rs            # Library interface
│   ├── frontend/         # Parsing and AST
│   ├── middleend/        # Type checking and elaboration
│   ├── backend/          # Code generation
│   ├── runtime/          # REPL and interpreter
│   └── tests/            # Integration tests
├── examples/             # Example Ruchy programs
└── benches/             # Performance benchmarks
```

This implementation will require approximately 15-20k lines of Rust code for a production-ready v1.0.
