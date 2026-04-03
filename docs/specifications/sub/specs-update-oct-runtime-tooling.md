# Sub-spec: Specifications Update Oct -- Runtime, Tooling, and Roadmap

**Parent:** [SPECIFICATIONS-UPDATE-OCT.md](../SPECIFICATIONS-UPDATE-OCT.md) Sections 6-11

---

## 6. Interpreter Specification

### 6.1 Tree-Walk Interpreter

For REPL and rapid prototyping:

```rust
pub struct Interpreter {
    globals: Environment,
    locals: Stack<Environment>,
    heap: Arena<Value>,
}

pub enum Value {
    // Stack allocated
    Int(i64),
    Float(f64),
    Bool(bool),
    
    // Heap allocated
    String(ArenaRef<String>),
    DataFrame(ArenaRef<DataFrame>),
    Function(ArenaRef<Closure>),
    
    // Special
    Null,
}
```

### 6.2 Execution Strategy

```rust
impl Interpreter {
    pub fn eval(&mut self, expr: &Expr) -> Result<Value> {
        self.heat_counter.track(expr);
        
        match self.heat_counter.get_heat(expr) {
            0..=2 => self.interpret(expr),      // Cold: interpret
            3..=10 => self.jit_compile(expr),   // Warm: JIT
            _ => self.aot_compile(expr),        // Hot: full compile
        }
    }
}
```

## 7. REPL Specification

### 7.1 Magic Commands

IPython/R/Julia compatible:

```rust
// Line magics
%time expr                  // Time expression
%load file.ruchy           // Load script
%pwd                       // Working directory
%who                       // List variables
%save session.rchy         // Save workspace

// Cell magics
%%time                     // Time cell
%%sql                      // SQL mode

// Shell
!cargo test               // Shell command

// Help
?DataFrame                // Quick help
??DataFrame              // Detailed help
```

### 7.2 Completion Engine

```rust
pub struct CompletionEngine {
    symbol_table: SymbolTable,
    type_info: TypeInference,
    
    pub fn complete(&self, partial: &str, cursor: usize) -> Vec<Completion> {
        match self.parse_context(partial, cursor) {
            Context::MemberAccess { receiver, partial } => {
                let receiver_type = self.type_info.infer(&receiver);
                self.get_members(receiver_type)
                    .filter(|m| m.starts_with(partial))
            }
            Context::Import { path } => self.complete_module(path),
            _ => self.global_completions(partial),
        }
    }
}
```

### 7.3 MCP Integration

Native Model Context Protocol support:

```rust
#[mcp_tool("analyze")]
fn analyze(df: DataFrame) -> Analysis {
    // Compile-time protocol generation
}

// In REPL
chat: "What patterns do you see in this data?"
// LLM receives full workspace context via MCP
```

## 8. Standard Library

### 8.1 Core Modules

```rust
// Automatic imports in REPL
use std::collections::{HashMap, HashSet};
use polars::prelude::*;

// Available modules
mod io;        // File I/O
mod net;       // Networking
mod math;      // Mathematical functions
mod stats;     // Statistical functions
mod plot;      // Visualization
```

### 8.2 DataFrame Operations

```rust
impl DataFrameOps for DataFrame {
    // All operations return LazyFrame for fusion
    fn filter(self, predicate: Expr) -> LazyFrame;
    fn select(self, columns: Vec<&str>) -> LazyFrame;
    fn groupby(self, keys: Vec<&str>) -> GroupBy;
    fn join(self, other: DataFrame, on: Vec<&str>) -> LazyFrame;
    fn sort(self, by: Vec<&str>) -> LazyFrame;
}
```

## 9. Tooling

### 9.1 Language Server Protocol

```rust
pub struct RuchyLsp {
    workspace: Workspace,
    analyzer: SemanticAnalyzer,
    
    capabilities: [
        TextDocumentSync::Full,
        Completion,
        Hover,
        GotoDefinition,
        References,
        Formatting,
        SemanticTokens,
    ]
}
```

### 9.2 Linter

```rust
pub enum LintLevel {
    Allow,
    Warn,
    Deny,
    Forbid,  // Cannot override
}

// Core lints
[lints]
correctness = "forbid"
satd_comments = "forbid"
complexity_over_10 = "forbid"

[lints.ruchy]
prefer_pipeline = "warn"     // x.f().g() -> x >> f >> g
prefer_dataframe = "warn"    // Vec<Vec<T>> -> DataFrame
actor_exhaustive = "deny"
```

### 9.3 Formatter

Rust-style formatting with DataFrame alignment:

```rust
// Before
let df=df!["name"=>["Alice","Bob"],"age"=>[30,25]]

// After  
let df = df![
    "name" => ["Alice", "Bob"],
    "age"  => [30, 25]
];
```

## 10. Quality Requirements

### 10.1 Phase 0 Gates (MANDATORY)

```rust
pub struct QualityGates {
    satd_count: usize,        // MUST be 0
    test_coverage: f64,       // MUST be >=80%
    max_complexity: u32,      // MUST be <=10
    parser_complete: bool,    // MUST be 100%
}

#[cfg(ci)]
compile_error_if!(SATD_COUNT > 0, "Zero SATD tolerance");
compile_error_if!(COVERAGE < 0.80, "Minimum 80% coverage");
compile_error_if!(COMPLEXITY > 10, "Maximum complexity 10");
```

### 10.2 Testing Requirements

```rust
// Property-based testing mandatory
#[proptest]
fn parser_roundtrip(source: String) {
    let ast = parse(&source);
    let printed = print(&ast);
    let reparsed = parse(&printed);
    prop_assert_eq!(ast, reparsed);
}

// Differential testing against Rust
#[test]
fn semantic_equivalence(ruchy_source: &str) {
    let rust_code = transpile(ruchy_source);
    assert_eq!(
        eval_ruchy(ruchy_source),
        eval_rust(&rust_code)
    );
}
```

## 11. Implementation Roadmap

### Phase 0: Foundation (Weeks 0-4) **BLOCKING**
- Eliminate all SATD comments
- Complete parser (30% remaining)
- Test coverage to 80%
- Reduce complexity to <=10
- CI quality gates

### Phase 1: MVP (Weeks 5-8)
- Direct AST -> syn generation
- Basic type inference
- DataFrame literals
- Function transpilation

### Phase 2: Interactive (Weeks 9-12)
- Tree-walk interpreter
- Magic commands
- Pipeline operator
- Syntax highlighting

### Phase 3: MIR (Weeks 13-16)
- MIR representation
- DataFrame fusion
- Optimization passes

### Phase 4: Production (Weeks 17-24)
- Full type inference
- Pattern matching
- Actor system
- LSP implementation

---

*This specification represents the complete, authoritative definition of the Ruchy language. All implementation must conform to these requirements.*
