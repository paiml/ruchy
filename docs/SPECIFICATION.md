# Ruchy: Complete Language and System Specification

*Single source of truth - All 27 specification documents consolidated*

## Table of Contents

### Core Language Specifications
1. [Language Specification](#1-language-specification)
2. [Grammar Reference](#2-grammar-reference) 
3. [Implementation Specification](#3-implementation-specification)
4. [Parser Specification](#4-parser-specification)
5. [Lexer Specification](#5-lexer-specification)
6. [Script Capability Specification](#6-script-capability-specification)

### Architecture Specifications  
7. [MCP Message-Passing Architecture](#7-mcp-message-passing-architecture)
8. [LSP Specification](#8-lsp-specification)
9. [Critical Missing Components](#9-critical-missing-components)
10. [Binary Architecture](#10-binary-architecture)
11. [Edge Cases Specification](#11-edge-cases-specification)
12. [REPL Testing Specification](#12-repl-testing-specification)
13. [Docker Specification](#13-docker-specification)

### Integration Specifications
14. [Cargo Integration](#14-cargo-integration)
15. [Depyler Integration](#15-depyler-integration)
16. [Rust Cargo InterOp](#16-rust-cargo-interop)

### Execution Mode Specifications
17. [One-Liner and Script Execution](#17-one-liner-and-script-execution)
18. [Disassembly Specification](#18-disassembly-specification)
19. [Advanced Mathematical REPL](#19-advanced-mathematical-repl)

### Quality & Testing Specifications
20. [Quality Gates](#20-quality-gates)
21. [Provability](#21-provability)

### Project Management
22. [Master TODO](#22-master-todo)
23. [Project Status](#23-project-status)
24. [Deep Context](#24-deep-context)

### External Dependencies
25. [PMAT Integration](#25-pmat-integration)
26. [PDMT Integration](#26-pdmt-integration)
27. [External Tool Dependencies](#27-external-tool-dependencies)

---

## 1. Language Specification

### 1.1 Design Philosophy

- **Familiarity First**: Syntax borrowed from Swift, Kotlin, Elixir
- **Progressive Complexity**: Simple code looks simple
- **Zero Runtime Overhead**: All abstractions compile to efficient Rust
- **Type Inference**: Types required only at module boundaries
- **DataFrame-First**: Polars as primary collection type

### 1.2 Type System

```rust
// Primitive Types
i32, i64, f32, f64, bool, String, ()

// Composite Types
[T]                    // Lists
(T1, T2, ...)         // Tuples  
T1 -> T2              // Functions
Option<T>             // Nullable types
Result<T, E>          // Error handling

// Mathematical Types (first-class)
DataFrame             // Tabular data (Polars)
LazyFrame            // Lazy DataFrame
Series               // Column data
Matrix<T, R, C>      // Linear algebra
Vector<T, N>         // N-dimensional vector
Array<T, D>          // N-dimensional array
SymExpr              // Symbolic expression
Formula              // Statistical formula (y ~ x1 + x2)
Distribution<T>      // Probability distribution
```

### 1.3 Core Language Features

#### Functions
```rust
// Function definition with type inference
fun add(x: i32, y: i32) -> i32 {
    x + y
}

// Lambda expressions  
|x| x + 1
|x, y| x * y

// Mathematical functions
fun mean(numbers: [f64]) -> f64 {
    numbers.sum() / numbers.len() as f64
}

// Generic functions
fun id<T>(x: T) -> T { x }
```

#### Pattern Matching
```rust
match value {
    0 => "zero",
    1 | 2 => "small", 
    n if n > 10 => "large",
    _ => "other"
}

// List patterns
match list {
    [] => "empty",
    [x] => "single", 
    [x, y] => "pair",
    _ => "many"
}
```

#### Pipeline Operations
```rust
// Data processing pipelines
data |> filter(|x| x > 0) 
     |> map(|x| x * 2) 
     |> collect()

// DataFrame operations (Polars backend)
df |> filter(col("age") > 18)
   |> groupby("department")
   |> agg([mean("salary").alias("avg_salary")])
   |> sort("avg_salary", descending=true)
```

#### Actor System
```rust
actor Counter {
    count: i32,
    
    receive {
        Increment => self.count += 1,
        Get => self.count,
        Reset => self.count = 0
    }
}

// Message passing
counter ! Increment     // Send (fire-and-forget)
result = counter ? Get  // Ask (request-response)
```

## 2. Grammar Reference

### 2.1 Formal Grammar (EBNF)

```ebnf
program         = item*
item            = function | struct_def | enum_def | trait_def | impl_block
                | actor_def | module_def | import_stmt | type_alias

expression      = assignment
assignment      = pipeline ('=' assignment)?
pipeline        = logical_or ('|>' pipeline)*
logical_or      = logical_and ('||' logical_and)*
logical_and     = equality ('&&' equality)*
equality        = comparison (('==' | '!=') comparison)*
comparison      = term (('>' | '>=' | '<' | '<=') term)*
term            = factor (('+' | '-') factor)*
factor          = unary (('*' | '/' | '%' | '**' | '//') unary)*
unary           = ('!' | '-' | 'await')? postfix
postfix         = primary ('.' IDENTIFIER | '[' expression ']' | '(' arguments? ')' | '?' | '!')*

primary         = NUMBER | STRING | BOOLEAN | IDENTIFIER | '(' expression ')'
                | if_expr | match_expr | let_expr | lambda | array_expr
                | tuple_expr | record_expr | async_block | try_block
                | dataframe_literal

lambda          = '|' params? '|' (expr | block)
```

### 2.2 Operator Precedence Table

| Precedence | Operators | Associativity | Category |
|------------|-----------|---------------|----------|
| 1 | `.` `?.` `::` | Left | Member access |
| 2 | `()` `[]` | Left | Call, index |
| 3 | `!` `~` `-` (unary) | Right | Unary |
| 4 | `**` | Right | Power |
| 5 | `*` `/` `%` `//` | Left | Multiplicative |
| 6 | `+` `-` | Left | Additive |
| 7 | `<<` `>>` | Left | Shift |
| 8 | `&` | Left | Bitwise AND |
| 9 | `^` | Left | Bitwise XOR |
| 10 | `\|` | Left | Bitwise OR |
| 11 | `==` `!=` `<` `<=` `>` `>=` | Left | Comparison |
| 12 | `is` `in` | Left | Type/membership |
| 13 | `&&` | Left | Logical AND |
| 14 | `\|\|` | Left | Logical OR |
| 15 | `..` `...` `..=` | None | Range |
| 16 | `??` | Right | Null coalescing |
| 17 | `\|>` | Left | Pipeline |
| 18 | `=` `+=` `-=` etc. | Right | Assignment |

### 2.3 Keywords (31 total)

```
Reserved: let in if else match fun struct trait impl import async await try catch
          for while loop break continue return true false actor receive send ask
          df col mean std quantile filter groupby agg sort
```

## 3. Implementation Specification

### 3.1 DataFrame-First Design

Every collection defaults to Polars types:
- `DataFrame` for 2D data
- `Series` for 1D data  
- `LazyFrame` for lazy evaluation
- Explicit `Vec` and `HashMap` when needed

### 3.2 Execution Modes

```rust
enum ExecutionMode {
    Interpret,           // Tree-walk interpreter (REPL)
    JitCompile,         // Cranelift JIT (hot paths)
    AotTranspile,       // Rust transpilation (production)
}

// Tiered compilation strategy
impl ExecutionStrategy {
    fn select(&self, expr: &Expr, heat: u32) -> ExecutionMode {
        match (expr.has_side_effects(), heat, expr.is_definition()) {
            (_, _, true) => ExecutionMode::AotTranspile,
            (false, 3.., false) => ExecutionMode::JitCompile,
            _ => ExecutionMode::Interpret,
        }
    }
}
```

### 3.3 Compilation Pipeline

1. **Frontend**
   - Lexical Analysis - Tokenize source code
   - Parsing - Build AST with error recovery
   - Type Inference - Hindley-Milner with mathematical types
   - Semantic Analysis - Name resolution, mathematical operator binding

2. **Middle End**
   - MIR Generation - Medium-level IR for analysis
   - Mathematical Optimization - Algebraic simplification, loop fusion
   - Lowering - Prepare for backend

3. **Backend**
   - Transpilation to Rust - Generate idiomatic Rust code
   - Polars Integration - Direct LogicalPlan generation
   - JIT Compilation - Cranelift for hot paths
   - AOT Compilation - rustc for final executable

## 4. Parser Specification

### 4.1 Architecture
- Recursive descent with Pratt parsing for operators
- Single-pass, predictive with bounded lookahead(2)
- No backtracking, error recovery via synchronization tokens

### 4.2 Performance Characteristics
- **Throughput**: >50MB/s on modern hardware
- **Memory**: O(n) in AST nodes
- **Lookahead**: Maximum 2 tokens
- **Error recovery**: O(1) per error

### 4.3 AST Construction
```rust
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
    pub ty: Option<Type>,  // Present post-inference
}

pub enum ExprKind {
    Integer(i64),
    Float(f64),
    String { parts: Vec<StringPart> },
    Binary { op: BinaryOp, left: Box<Expr>, right: Box<Expr> },
    Call { func: Box<Expr>, args: Vec<Expr> },
    Lambda { params: Vec<Param>, body: Box<Expr> },
    DataFrame { columns: Vec<Column> },
    // ... all node types
}
```

## 5. Lexer Specification

### 5.1 Design Constraints
- Single pass with O(1) lookahead
- No parser feedback, no backtracking
- No heap allocation per token
- UTF-8 boundary handling

### 5.2 Performance Targets
- **Throughput**: >100MB/s target
- **Memory**: Token pool with arena allocation
- **Latency**: <1ms for typical files

### 5.3 Token Categories
- **Keywords**: 31 total (fun, let, if, match, actor, etc.)
- **Operators**: Arithmetic, comparison, logical, bitwise, pipeline
- **Delimiters**: Parentheses, brackets, braces, punctuation
- **Literals**: Numbers, strings, characters, booleans

## 6. Script Capability Specification

### 6.1 Execution Modes

```rust
enum ScriptMode {
    // Direct command-line execution
    OneLiner {
        code: String,
        auto_mode: AutoMode,  // -e, -p, -n, -f, -a flags
    },
    
    // Interactive REPL  
    Interactive {
        session: ReplSession,
        startup_time: Duration,  // <10ms target
    },
    
    // Script file execution
    Script {
        path: PathBuf,
        permissions: PermissionSet,  // Deno-style
    },
    
    // Compiled binary
    Binary {
        size: usize,  // <5MB target
        startup: Duration,  // <1ms target
    },
}
```

### 6.2 Permission System (Deno-style)

```bash
# Granular permissions
ruchy run --allow-read=/data --allow-net=api.example.com script.ruchy
ruchy run --allow-write=/tmp --allow-env script.ruchy

# Interactive prompts for missing permissions
ruchy run script.ruchy  # Prompts for each permission
```

### 6.3 URL Imports with Integrity

```rust
import "https://deno.land/std@0.100.0/fmt/colors.ruchy" as colors
import "https://cdn.skypack.dev/lodash@4.17.21" as _

// Integrity checking
import "https://example.com/lib.ruchy" 
    integrity="sha256-abc123..." 
    as lib
```

## 7. MCP Message-Passing Architecture

### 7.1 Core Design Principles
- **Protocol-First**: Actor behaviors derive from protocol specifications
- **Location Transparency**: Local and remote messages use identical syntax
- **Fault Tolerance**: Supervision trees provide automatic recovery
- **Type Safety**: Session types prove protocol compliance at compile time
- **Zero Overhead**: Message passing compiles to function calls when possible

### 7.2 Unified Message Runtime

```rust
pub struct MessageRuntime {
    // Per-core actor schedulers (M:N threading)
    schedulers: Vec<Scheduler>,
    
    // Global registry for actor/MCP endpoint discovery
    registry: DistributedRegistry,
    
    // Protocol bridges for external systems
    bridges: ProtocolBridges {
        mcp: MCPBridge,
        grpc: GrpcBridge,
        http: HttpBridge,
    },
    
    // NUMA-aware memory pools for zero-copy messaging
    message_pools: NumaAllocator,
}

// Location-transparent messaging
impl MessageRuntime {
    pub fn send<M: Message>(&self, target: ActorRef, msg: M) -> Result<()> {
        match self.registry.locate(target) {
            Location::Local(mailbox) => mailbox.enqueue(msg),     // 50ns latency
            Location::Remote(node) => self.remote_send(node, msg), // 50μs latency
            Location::MCP(endpoint) => self.mcp.call(endpoint, msg), // 100μs latency
        }
    }
}
```

### 7.3 Memory Layout Optimization

```rust
// Cache-line aligned message structure
#[repr(C, align(64))]
pub struct Message {
    // Header in first cache line (32 bytes)
    header: MessageHeader {
        msg_type: TypeId,        // 8 bytes
        correlation_id: u64,     // 8 bytes
        priority: Priority,      // 1 byte
        flags: MessageFlags,     // 1 byte
        padding: [u8; 14],       // Alignment
    },
    
    // Payload in subsequent cache lines
    payload: MessagePayload,     // Variable size
}

// Zero-copy payload for large messages
pub enum MessagePayload {
    Inline([u8; 496]),          // Small messages inline
    Arc(Arc<[u8]>),             // Shared large messages
    Mmap(MmapRegion),           // Memory-mapped for huge payloads
}
```

## 8. LSP Specification

### 8.1 Performance Requirements
- **Typing Latency**: <50ms for completions
- **Diagnostic Latency**: <200ms for type errors
- **Memory Budget**: <500MB for 50k LOC
- **CPU Usage**: <25% average on 4-core systems

### 8.2 Core Features

#### Phase 1: Foundation
- Syntax highlighting with semantic tokens
- Real-time diagnostics (parse, type, borrow checker)  
- Context-aware completions
- Hover information with type details
- Go-to-definition across modules
- Find references with usage context

#### Phase 2: Transpilation Preview
- Real-time Rust code preview
- Performance cost indicators
- Side-by-side comparison mode
- Optimization suggestions

#### Phase 3: Quality Enforcement  
- Property test generation
- Mutation testing integration
- Refinement type checking
- Quality gate dashboard

### 8.3 Progressive Disclosure

Information scales with user intent:
- **Beginner**: Basic types, simple suggestions
- **Intermediate**: Performance hints, idiom suggestions  
- **Expert**: Detailed AST, optimization details

## 9. Critical Missing Components

### 9.1 Error Diagnostics Architecture
- Elm-level error messages with helpful suggestions
- Structured error codes (L0001-R4999)
- Machine-applicable fix suggestions
- Terminal color rendering with proper fallbacks

### 9.2 Module System Details
- Separate compilation units with incremental checking
- Cross-module type checking without re-parsing
- Symbol visibility rules (pub, priv, internal)
- Incremental compilation cache invalidation

### 9.3 Memory Management
- Region inference algorithm for automatic lifetime management
- Escape analysis for stack vs heap allocation
- Arena allocation strategies for session-scoped data
- Reference counting with cycle detection

### 9.4 Borrow Checker Integration
- Lifetime inference compatible with Rust
- Ownership transfer rules in transpilation
- Borrowing patterns that map to Rust idioms
- Zero-cost Rust interop semantics

### 9.5 Effect System
- IO, Async, Unsafe, MCP effects tracking
- Effect polymorphism for generic functions
- Handler implementation for effect interpretation
- Runtime elimination of effect tracking

## 10. Binary Architecture

### 10.1 Single Binary Design (<5MB)

All tools embedded in one binary:
```rust
enum Command {
    Run(RunOpts),       // Execute scripts
    Repl(ReplOpts),     // Interactive shell
    Build(BuildOpts),   // Compile to binary
    Fmt(FmtOpts),       // Format code
    Lint(LintOpts),     // Lint with fixes
    Test(TestOpts),     // Run tests
    Check(CheckOpts),   // Type check only
    Serve(ServeOpts),   // MCP server mode
}
```

### 10.2 Shared Infrastructure
- Single AST parse shared by all tools
- DashMap concurrent cache with Blake3 hashing
- Lazy subsystem loading for fast startup
- Memory-mapped caches for persistence

### 10.3 Permission System (Deno-style)
- Secure by default execution
- Granular permissions (read, write, net, env)
- Interactive prompts for missing permissions
- Audit trail for security compliance

### 10.4 Exit Codes (Semantic)

```rust
enum ExitCode {
    Success = 0,
    ParseError = 1,
    TypeError = 2, 
    RuntimePanic = 3,
    LintViolations = 4,
    TestFailures = 5,
    CompilationError = 6,
    ConfigError = 7,
    NetworkError = 8,
}
```

## 11. Edge Cases Specification

### 11.1 Module Resolution Algorithm
- Path-based resolution with precedence rules
- URL import caching and integrity verification
- Circular dependency detection and handling
- Version compatibility checking

### 11.2 Ownership Transfer Rules
- Move semantics by default
- Explicit copy annotations
- Reference borrowing patterns
- Lifetime elision rules

### 11.3 Effect Polymorphism
- Effect inference across function boundaries
- Handler composition and nesting
- Effect masking and visibility
- Performance optimization for pure code

### 11.4 Trait Resolution
- Coherence rules preventing conflicts
- Orphan rules for external implementations
- Higher-ranked trait bounds
- Associated type projection

## 12. REPL Testing Specification

### 12.1 Testing Strategies (11 complementary approaches)

1. **Property-Based Testing**
   - QuickCheck-style for parser
   - Shrinking for minimal failing cases
   - Generator composition

2. **State Machine Testing** 
   - REPL session as state machine
   - Property testing of state transitions
   - Invariant verification

3. **Differential Testing**
   - Compare against reference implementations
   - Cross-validate with Rust behavior
   - Consistency across execution modes

4. **Incremental Compilation Fuzzing**
   - Random edit sequences
   - Cache invalidation testing
   - Performance regression detection

5. **Memory Safety Validation**
   - AddressSanitizer integration
   - Leak detection for long sessions
   - Stack overflow protection

### 12.2 Performance Testing
- Latency distribution analysis
- Memory usage profiling
- Scalability testing to large codebases
- Regression test automation

## 13. Docker Specification

### 13.1 Multi-Stage Build (6 stages)

```dockerfile
# Stage 1: Base dependencies
FROM rust:1.70-alpine AS base
RUN apk add --no-cache musl-dev

# Stage 2: Dependencies  
FROM base AS deps
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

# Stage 3: Build
FROM deps AS build
COPY src ./src
RUN cargo build --release

# Stage 4: WASM target
FROM build AS wasm
RUN cargo build --target wasm32-wasi --release

# Stage 5: Runtime
FROM alpine:3.18 AS runtime
COPY --from=build /target/release/ruchy /usr/local/bin/
RUN adduser -D ruchy

# Stage 6: WASM runtime
FROM scratch AS wasm-runtime
COPY --from=wasm /target/wasm32-wasi/release/ruchy.wasm /
```

### 13.2 Performance Targets
- **WASM bundle size**: <410KB
- **Container startup**: <100ms
- **Memory usage**: <10MB base
- **Security**: Rootless execution by default

## 14. Cargo Integration

### 14.1 Build Script Integration

```toml
# User's Cargo.toml
[build-dependencies]
ruchy = "1.0"

# build.rs - Auto-generated or minimal boilerplate
fn main() {
    ruchy::build::transpile_project().unwrap();
}
```

### 14.2 Module Resolution Strategy

```
project/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Standard entry point
│   ├── algo.ruchy       # Ruchy source
│   └── data/
│       ├── mod.ruchy    # Module definition
│       └── process.ruchy
└── target/
    └── ruchy-gen/       # Generated Rust (gitignored)
        ├── algo.rs
        └── data/
            ├── mod.rs   # Generated module root
            └── process.rs
```

### 14.3 IDE Integration

Build script generates virtual `Cargo.toml` for rust-analyzer:
```toml
# target/ruchy-gen/Cargo.toml (auto-generated)
[package]
name = "ruchy-gen"
version = "0.0.0"

[lib]
path = "lib.rs"  # Root module with #[path] attributes
```

## 15. Depyler Integration

### 15.1 Bidirectional Transpilation
- Python → Ruchy → Rust (migration path)
- Ruchy → Python (scripting interop)
- Property preservation validation
- Incremental migration support

### 15.2 Zero-Copy Bridging
- NumPy arrays to ndarray without copying
- Polars DataFrame direct sharing
- Automatic type inference from Python
- Memory layout compatibility

## 16. Rust Cargo InterOp

### 16.1 Polyglot Binary Masquerading

```rust
// Binary serves dual purposes based on invocation
match std::env::args().nth(0).unwrap() {
    path if path.ends_with("cargo-ruchy") => cargo_plugin_main(),
    path if path.ends_with("ruchy") => ruchy_main(),
    _ => detect_mode_and_run(),
}
```

### 16.2 Module Resolution Hijacking
- Intercept Rust module resolution
- Transparently compile .ruchy files
- Source map preservation
- Error message translation

## 17. One-Liner and Script Execution

### 17.1 Command-Line Interface

```bash
# One-liner execution modes
ruchy -e 'println("hello")'                    # Direct execution
ruchy -p 'it.to_uppercase()'                   # Process & print each line
ruchy -n 'sum += it.parse::<i32>()?'           # Process without printing
ruchy -f 'it.contains("error")'                # Filter lines
ruchy -a 'it.split(",")[0]'                    # Accumulate results

# Pipeline composition
cat data.csv | ruchy -p 'it.split(",")[2]'    # Extract third column
ls -la | ruchy -f 'it.contains(".rs")'         # Filter Rust files
seq 1 100 | ruchy -a 'it.parse::<i32>()? * 2' # Double and collect
```

### 17.2 Auto-Mode Detection Heuristics

```rust
enum AutoMode {
    None,           // -e: Execute as-is
    Print,          // -p: Auto-print after processing
    Process,        // -n: Process line-by-line, no print
    Filter,         // -f: Process and print if truthy
    Accumulate,     // -a: Collect results into array
}

impl AutoMode {
    fn detect_from_code(code: &str) -> Self {
        // Heuristic detection based on code patterns
        if code.contains("println!") || code.contains("print!") {
            AutoMode::None  // Already has explicit output
        } else if code.ends_with('?') || code.contains("if ") {
            AutoMode::Filter  // Likely a predicate
        } else if code.contains("+=") || code.contains("push") {
            AutoMode::Accumulate  // Building up state
        } else if code.contains("it.") {
            AutoMode::Print  // Transform current line
        } else {
            AutoMode::Process  // Generic processing
        }
    }
}
```

### 17.3 Implicit Variable Scoping Rules

```rust
pub struct ImplicitContext {
    // Line-scoped (reset each line)
    it: String,           // Current line from stdin
    fields: Vec<String>,   // it.split_whitespace()
    f1, f2, f3: String,    // First three fields
    captures: Vec<String>, // From last regex match
    
    // Session-scoped (persist across lines)
    sum: Dynamic,         // Running sum (auto-typed)
    count: usize,         // Line counter (1-indexed)
    acc: Vec<Dynamic>,    // Accumulator array
    
    // Context-scoped (available based on mode)
    filename: Option<String>,  // Available in file processing
    line_num: usize,          // 1-indexed line number
    
    // Regex capture groups (dynamic)
    captures: HashMap<String, String>,  // Named captures
}

impl ImplicitContext {
    fn update_line(&mut self, line: String) {
        self.it = line.clone();
        self.count += 1;
        self.line_num += 1;
        
        // Auto-split into fields
        self.fields = line.split_whitespace().map(String::from).collect();
        
        // Populate f1, f2, f3 convenience variables
        self.f1 = self.fields.get(0).cloned().unwrap_or_default();
        self.f2 = self.fields.get(1).cloned().unwrap_or_default();
        self.f3 = self.fields.get(2).cloned().unwrap_or_default();
    }
    
    fn apply_regex(&mut self, pattern: &Regex) {
        if let Some(caps) = pattern.captures(&self.it) {
            self.captures = caps.iter()
                .enumerate()
                .filter_map(|(i, m)| m.map(|m| (i.to_string(), m.as_str().to_string())))
                .collect();
        }
    }
}
```

### 17.4 Pipeline Composition Semantics

```rust
// Pipeline composition follows these rules:
// 1. Left-to-right evaluation
// 2. Implicit line iteration
// 3. Type preservation where possible
// 4. Error propagation via ?

impl PipelineComposer {
    fn compose_commands(commands: &[String]) -> Result<ComposedPipeline> {
        let mut pipeline = ComposedPipeline::new();
        
        for (i, cmd) in commands.iter().enumerate() {
            let stage = PipelineStage {
                command: cmd.clone(),
                input_type: if i == 0 { InputType::Stdin } else { InputType::Previous },
                output_type: OutputType::infer_from_command(cmd),
                error_handling: ErrorHandling::Propagate,
            };
            pipeline.add_stage(stage);
        }
        
        // Optimize pipeline for zero-copy where possible
        pipeline.optimize_transfers();
        Ok(pipeline)
    }
}

// Example optimizations:
// cat file.csv | ruchy -p 'parse_csv(it)' | ruchy -f 'it.len() > 3'
// Optimizes to: ruchy -e 'process_csv_filter("file.csv", |row| row.len() > 3)'
```

### 17.5 Automatic Imports

```rust
// Always available in one-liner mode
implicit_prelude! {
    use std::io::{self, BufRead, Write};
    use std::fs;
    use std::collections::{HashMap, HashSet};
    use std::path::{Path, PathBuf};
    use polars::prelude::*;
    use regex::Regex;
    use serde::{Serialize, Deserialize};
    use serde_json;
    
    // Mathematical functions
    use std::f64::consts::{PI, E, TAU};
    
    // Common utility functions
    fn parse_csv(line: &str) -> Vec<String> {
        line.split(',').map(|s| s.trim().to_string()).collect()
    }
    
    fn parse_tsv(line: &str) -> Vec<String> {
        line.split('\t').map(|s| s.trim().to_string()).collect()
    }
    
    // Regex shortcuts
    static COMMON_PATTERNS: Lazy<HashMap<&str, Regex>> = Lazy::new(|| {
        let mut map = HashMap::new();
        map.insert("email", Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap());
        map.insert("url", Regex::new(r"https?://[^\s]+").unwrap());
        map.insert("number", Regex::new(r"-?\d+\.?\d*").unwrap());
        map.insert("word", Regex::new(r"\b\w+\b").unwrap());
        map
    });
}

## 18. Disassembly Specification

### 18.1 Multi-Representation Output

```rust
pub trait Disassembler<Input> {
    type Output;
    fn disassemble(&self, input: &Input) -> Self::Output;
}

// Primary representations (canonical)
impl Disassembler<TypedAst> for JsonAstDisassembler { ... }
impl Disassembler<TypedAst> for MirDisassembler { ... }
impl Disassembler<TypedAst> for RustDisassembler { ... }
impl Disassembler<TypedAst> for BytecodeDisassembler { ... }
impl Disassembler<TypedAst> for SsaDisassembler { ... }

// Derived representations (computed from primary)
impl Disassembler<JsonAst> for AnnotatedAstDisassembler { ... }
impl Disassembler<SymbolTable> for MermaidDepsDisassembler { ... }
impl Disassembler<MirGraph> for OptimizationReportDisassembler { ... }
```

### 18.2 Bytecode Representation Details

```rust
// Stack-based bytecode for interpretation
#[derive(Debug, Clone)]
pub enum Instruction {
    // Literals
    LoadInt(i64),
    LoadFloat(f64),
    LoadString(StringId),
    LoadBool(bool),
    
    // Stack operations
    Dup,              // Duplicate top of stack
    Pop,              // Remove top of stack
    Swap,             // Swap top two stack items
    
    // Arithmetic
    Add, Sub, Mul, Div, Mod, Pow,
    
    // Comparison
    Eq, Ne, Lt, Le, Gt, Ge,
    
    // Logical
    And, Or, Not,
    
    // Control flow
    Jump(Label),
    JumpIf(Label),
    JumpIfNot(Label),
    Call(FunctionId, u8),  // function_id, arg_count
    Return,
    
    // Variables
    LoadLocal(LocalId),
    StoreLocal(LocalId),
    LoadGlobal(GlobalId),
    StoreGlobal(GlobalId),
    
    // Pattern matching
    MatchStart,
    MatchPattern(PatternId),
    MatchEnd,
    
    // DataFrame operations (specialized)
    DfFilter(ExprId),
    DfGroupBy(ColumnId),
    DfAgg(AggOp, ColumnId),
    DfSort(ColumnId, bool),  // column, descending
    
    // Actor operations
    ActorSend(ActorId),
    ActorAsk(ActorId),
    
    // Debug information
    DebugLine(u32),
    DebugColumn(u32),
}

// Bytecode function representation
#[derive(Debug)]
pub struct BytecodeFunction {
    name: String,
    params: Vec<LocalId>,
    locals: Vec<Type>,
    instructions: Vec<Instruction>,
    labels: HashMap<Label, usize>,  // label -> instruction_index
    debug_info: Vec<DebugInfo>,
}

// Bytecode optimization passes
impl BytecodeOptimizer {
    fn constant_fold(&mut self, function: &mut BytecodeFunction) {
        // Fold constant expressions at bytecode level
        // LoadInt(2), LoadInt(3), Add -> LoadInt(5)
    }
    
    fn dead_code_elimination(&mut self, function: &mut BytecodeFunction) {
        // Remove unreachable instructions
        // Remove unused local variables
    }
    
    fn peephole_optimize(&mut self, function: &mut BytecodeFunction) {
        // Pattern-based local optimizations
        // LoadLocal(x), LoadLocal(x) -> LoadLocal(x), Dup
    }
}
```

### 18.3 SSA Form Generation

```rust
// Static Single Assignment intermediate representation
#[derive(Debug, Clone)]
pub struct SsaFunction {
    name: String,
    params: Vec<SsaValue>,
    blocks: Vec<BasicBlock>,
    value_table: HashMap<SsaValue, SsaInstruction>,
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    id: BlockId,
    instructions: Vec<SsaInstruction>,
    terminator: Terminator,
    predecessors: Vec<BlockId>,
    successors: Vec<BlockId>,
}

#[derive(Debug, Clone)]
pub enum SsaInstruction {
    // Arithmetic
    Add(SsaValue, SsaValue),
    Sub(SsaValue, SsaValue),
    Mul(SsaValue, SsaValue),
    Div(SsaValue, SsaValue),
    
    // Memory
    Load(Address),
    Store(Address, SsaValue),
    
    // Phi functions (for control flow joins)
    Phi(Vec<(SsaValue, BlockId)>),
    
    // Function calls
    Call(FunctionId, Vec<SsaValue>),
    
    // DataFrame operations (high-level)
    DfOp(DataFrameOp, Vec<SsaValue>),
    
    // Type conversions
    Cast(SsaValue, Type),
    
    // Literals
    Constant(Literal),
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Return(Option<SsaValue>),
    Jump(BlockId),
    Branch(SsaValue, BlockId, BlockId),  // condition, true_block, false_block
    Switch(SsaValue, Vec<(Literal, BlockId)>, BlockId),  // value, cases, default
}

// SSA construction algorithm
impl SsaBuilder {
    fn build_ssa(&mut self, ast: &TypedAst) -> SsaFunction {
        // 1. Convert to basic blocks
        let blocks = self.build_basic_blocks(ast);
        
        // 2. Insert phi functions
        self.insert_phi_functions(&mut blocks);
        
        // 3. Rename variables to SSA form
        self.rename_variables(&mut blocks);
        
        SsaFunction { blocks, ..Default::default() }
    }
    
    fn insert_phi_functions(&mut self, blocks: &mut [BasicBlock]) {
        // Compute dominance frontier
        let dom_frontier = self.compute_dominance_frontier(blocks);
        
        // Insert phi functions for each variable at join points
        for (var, defs) in &self.variable_definitions {
            let mut work_list = defs.clone();
            let mut processed = HashSet::new();
            
            while let Some(block) = work_list.pop() {
                for frontier_block in &dom_frontier[&block] {
                    if !processed.contains(frontier_block) {
                        blocks[*frontier_block].insert_phi(*var);
                        processed.insert(*frontier_block);
                        work_list.push(*frontier_block);
                    }
                }
            }
        }
    }
}
```

### 18.4 Debug Symbol Preservation

```rust
// Debug information embedded in all representations
#[derive(Debug, Clone)]
pub struct DebugInfo {
    source_file: PathBuf,
    line_start: u32,
    line_end: u32,
    column_start: u32,
    column_end: u32,
    original_text: String,
    
    // Additional context
    function_name: Option<String>,
    variable_names: HashMap<LocalId, String>,
    type_annotations: HashMap<SsaValue, Type>,
}

// Source map generation for transpiled Rust
#[derive(Debug)]
pub struct SourceMap {
    version: u8,
    sources: Vec<PathBuf>,
    source_root: Option<PathBuf>,
    names: Vec<String>,
    mappings: String,  // VLQ-encoded mappings
}

impl SourceMapGenerator {
    fn generate(&self, ruchy_ast: &TypedAst, rust_tokens: &TokenStream) -> SourceMap {
        let mut mappings = Vec::new();
        
        // Map each Rust token back to original Ruchy source
        for (rust_span, ruchy_span) in self.span_mappings.iter() {
            mappings.push(SourceMapping {
                generated_line: rust_span.start().line,
                generated_column: rust_span.start().column,
                source_index: 0,  // Assuming single source file
                original_line: ruchy_span.start,
                original_column: ruchy_span.end,
                name_index: None,
            });
        }
        
        SourceMap {
            version: 3,
            sources: vec![self.source_file.clone()],
            source_root: None,
            names: self.symbol_names.clone(),
            mappings: self.encode_vlq(&mappings),
        }
    }
    
    fn encode_vlq(&self, mappings: &[SourceMapping]) -> String {
        // Variable-length quantity encoding for compact representation
        // Used by browser dev tools and IDEs
        let mut result = String::new();
        let mut prev_generated_column = 0;
        let mut prev_original_line = 0;
        let mut prev_original_column = 0;
        
        for mapping in mappings {
            // Encode relative offsets for compression
            result.push_str(&self.vlq_encode(mapping.generated_column as i32 - prev_generated_column));
            result.push_str(&self.vlq_encode(mapping.original_line as i32 - prev_original_line));
            result.push_str(&self.vlq_encode(mapping.original_column as i32 - prev_original_column));
            
            prev_generated_column = mapping.generated_column as i32;
            prev_original_line = mapping.original_line as i32;
            prev_original_column = mapping.original_column as i32;
        }
        
        result
    }
}

// Debug symbol table for runtime debugging
#[derive(Debug)]
pub struct SymbolTable {
    functions: HashMap<FunctionId, FunctionSymbol>,
    variables: HashMap<VariableId, VariableSymbol>,
    types: HashMap<TypeId, TypeSymbol>,
    source_locations: HashMap<InstructionId, SourceLocation>,
}

#[derive(Debug)]
pub struct FunctionSymbol {
    name: String,
    mangled_name: String,
    parameters: Vec<ParameterSymbol>,
    return_type: Type,
    source_location: SourceLocation,
    local_variables: Vec<VariableSymbol>,
}
```

### 18.5 Format Hierarchy

| Format | Purpose | Consumer | Stability |
|--------|---------|----------|-----------|
| `json-ast` | Canonical AST | MCP agents, tooling | Stable v1.0 |
| `symbol-table` | Entity index | Static analyzers | Stable v1.0 |
| `bytecode` | Interpretation | REPL, debugging | Stable v1.0 |
| `ssa` | Optimization analysis | Compiler developers | Internal |
| `mir` | Mid-level IR | Optimization passes | Internal |
| `rust` | Transpilation target | Build systems | Stable |
| `asm` | Performance verification | Systems programmers | Platform-specific |
| `source-map` | Debug mapping | IDEs, debuggers | Stable v1.0 |

### 18.6 Content-Based IDs

```typescript
interface AstNode {
    id: string;           // SHA256(kind + span + children_ids)[:8]
    kind: NodeKind;       // Discriminant
    span: [number, number];
    ty?: Type;           // Present on Expression nodes post-inference
    complexity?: {       // For function nodes
        cyclomatic: number;
        cognitive: number;
    };
    debug_info?: DebugInfo;  // Source location preservation
}

// ID generation algorithm ensures deterministic, content-based identifiers
// that remain stable across compilations of identical code
impl AstNode {
    fn compute_id(&self) -> String {
        let mut hasher = Sha256::new();
        
        // Hash structural content
        hasher.update(self.kind.discriminant().to_le_bytes());
        hasher.update(self.span.start.to_le_bytes());
        hasher.update(self.span.end.to_le_bytes());
        
        // Hash children IDs for compositional stability
        for child in &self.children() {
            hasher.update(child.id.as_bytes());
        }
        
        // Take first 8 hex characters for compact representation
        format!("{:08x}", u32::from_be_bytes(hasher.finalize()[..4].try_into().unwrap()))
    }
}
```

## 19. Advanced Mathematical REPL

### 19.1 Tiered Compilation Architecture

```rust
enum ExecutionMode {
    Interpret,           // First 2 evaluations
    JitCompile,         // 3+ evaluations (via Cranelift)
    AotTranspile,       // Persistent definitions
}

// Three-tier memory model
enum ValueStorage {
    Stack(StackValue),      // Primitives, small arrays (<256 bytes)
    Arena(ArenaRef),        // Session-scoped allocations
    Persistent(Arc<Value>), // Cross-session, reference-counted
}
```

### 19.2 DataFrame Engine (Polars Integration)

```rust
// Zero-copy transpilation to Polars LazyFrame
df = read_csv("data.csv")
  |> filter(col("value") > threshold)
  |> groupby("category")
  |> agg([
      mean("x").alias("x_mean"),
      std("x").alias("x_std"),
      quantile("x", 0.95).alias("x_p95")
  ])
  |> sort("x_mean", descending=true)

// Transpiles directly to:
LazyFrame::scan_csv("data.csv", Default::default())
    .filter(col("value").gt(threshold))
    .groupby([col("category")])
    .agg([
        col("x").mean().alias("x_mean"),
        col("x").std().alias("x_std"),
        col("x").quantile(0.95, QuantileInterpolOptions::Linear).alias("x_p95")
    ])
    .sort("x_mean", SortOptions { descending: true, ..Default::default() })
```

### 19.3 Linear Algebra Kernel

```rust
// Broadcasting semantics (NumPy-compatible)
A = rand(1000, 1000)
b = ones(1000, 1)
c = A @ b  // Matrix multiplication
d = A .* b  // Element-wise with broadcasting

// Solver syntax (MATLAB-compatible)
x = A \ b  // Solve Ax = b via LU decomposition
[U, S, V] = svd(A)  // Destructuring assignment

// Automatic backend selection
@geometric {
    // Uses nalgebra for better performance
    rotation = Rotation3::from_euler_angles(π/4, 0, π/2)
    transformed = rotation * point
}
```

### 19.4 Symbolic Mathematics (Constrained Scope)

```rust
// Core expression tree - simplified for maintainability (10K LOC max)
enum SymExpr {
    Var(Symbol),
    Num(f64),
    BinOp { op: Op, lhs: Box<SymExpr>, rhs: Box<SymExpr> },
    UnOp { op: UnaryOp, arg: Box<SymExpr> },
    Call { func: BuiltinFunc, args: Vec<SymExpr> },
}

// Pattern-based differentiation rules only
impl SymExpr {
    fn diff(&self, var: Symbol) -> SymExpr {
        match self {
            SymExpr::Var(s) if *s == var => SymExpr::Num(1.0),
            SymExpr::BinOp { op: Op::Add, lhs, rhs } => 
                lhs.diff(var) + rhs.diff(var),
            SymExpr::BinOp { op: Op::Mul, lhs, rhs } => 
                lhs.diff(var) * rhs.clone() + lhs.clone() * rhs.diff(var),
            // 20 core rules cover 95% of use cases
            _ => SymExpr::Num(0.0),
        }
    }
}
```

### 19.5 Statistical Computing

```rust
// Formula parser generates design matrices directly
struct Formula {
    response: Term,
    predictors: Vec<Term>,
    contrasts: ContrastScheme,
}

impl Formula {
    fn parse(input: &str) -> Result<Self> {
        // "y ~ x1 + x2 + x1:x2" -> 
        // X = [1, x1, x2, x1*x2] design matrix
        // Handles categorical encoding automatically
    }
    
    fn to_design_matrix(&self, data: &DataFrame) -> (Array2<f64>, Array1<f64>) {
        // Efficient construction via ndarray views
        // Zero-copy where possible
    }
}

// QR decomposition for numerical stability
fn lm(formula: Formula, data: DataFrame) -> LinearModel {
    let (X, y) = formula.to_design_matrix(&data);
    let qr = X.qr().unwrap();
    let coefficients = qr.solve(&y).unwrap();
    LinearModel::from_qr(qr, coefficients)
}
```

### 19.6 Visualization System

```rust
// Automatic backend selection
plot(sin, 0..2π)  // Unicode in terminal, SVG in notebook

// Grammar of graphics (ggplot2-inspired)
g = ggplot(df, aes(x="height", y="weight"))
  |> geom_point(alpha=0.5)
  |> geom_smooth(method="lm")
  |> facet_wrap("species")
  |> theme_minimal()

// Terminal rendering capabilities
// - Sixel: Full raster graphics
// - Unicode: Box drawing, blocks  
// - ANSI: Colors only
// - ASCII: Pure text
```

### 19.7 Performance Targets

| Operation | Target | Implementation |
|-----------|--------|----------------|
| Cold startup | <10ms | Bytecode cache |
| Warm startup | <2ms | Memory-mapped state |
| Expression eval | <1ms | Cranelift JIT |
| DataFrame filter | <10ms | Polars LazyFrame |
| Matrix multiply | <5ms | BLAS integration |
| Plot render | <50ms | Unicode/Sixel |
| Memory overhead | <50MB | Arena allocation |

## 20. Quality Gates

### 20.1 Code Quality Standards

- **Zero clippy warnings** allowed (`-D warnings`)
- **Maximum function complexity**: 10 (cognitive complexity)
- **Maximum file size**: 500 lines
- **Zero SATD** (TODO/FIXME) comments
- **100% test pass rate** required
- **Minimum 80% code coverage**

### 20.2 Toyota Way Quality Enforcement

```rust
struct QualityMetrics {
    cyclomatic_complexity: u32,      // Max 10
    cognitive_complexity: u32,       // Max 15  
    halstead_effort: f64,           // Max 5000
    maintainability_index: f64,     // Min 70
    test_coverage: f64,             // Min 80%
    satd_comments: u32,             // Exactly 0
    mutation_score: f64,            // Min 75%
}

impl QualityGate {
    fn enforce(&self, metrics: &QualityMetrics) -> Result<(), QualityViolation> {
        if metrics.satd_comments > 0 {
            return Err(QualityViolation::SATDDetected(metrics.satd_comments));
        }
        if metrics.cyclomatic_complexity > 10 {
            return Err(QualityViolation::ComplexityTooHigh(metrics.cyclomatic_complexity));
        }
        // ... all quality checks
        Ok(())
    }
}
```

### 20.3 PMAT Integration

Zero-overhead profiling with compile-time erasure:
```rust
#[cfg(debug_assertions)]
macro_rules! profile {
    ($expr:expr) => {{
        let _guard = PROFILER.start_scope(stringify!($expr));
        $expr
    }};
}

#[cfg(not(debug_assertions))]
macro_rules! profile {
    ($expr:expr) => { $expr };
}
```

## 21. Provability

### 21.1 SMT-Based Refinement Types

```rust
// Refinement type syntax
type PositiveInt = {x: i32 | x > 0}
type NonEmptyVec<T> = {xs: Vec<T> | xs.len() > 0}
type Probability = {p: f64 | 0.0 <= p && p <= 1.0}

// Function contracts
#[requires(x > 0)]
#[ensures(result > x)]
fn increment_positive(x: PositiveInt) -> PositiveInt {
    x + 1
}
```

### 21.2 Symbolic Execution Engine

```rust
struct SymbolicState {
    variables: HashMap<Symbol, SymbolicValue>,
    constraints: Vec<Constraint>,
    path_condition: BoolExpr,
}

impl SymbolicExecutor {
    fn execute_path(&mut self, program: &Program) -> PathResult {
        // Explore execution paths symbolically
        // Generate test cases for each path
        // Verify assertions and contracts
    }
}
```

### 21.3 Property Testing Integration

```rust
#[property]
fn test_sort_preserves_length(xs: Vec<i32>) {
    let sorted = sort(xs.clone());
    assert_eq!(xs.len(), sorted.len());
}

#[property]
fn test_reverse_is_involutive(xs: Vec<i32>) {
    let double_reversed = reverse(reverse(xs.clone()));
    assert_eq!(xs, double_reversed);
}
```

## 22. Master TODO

### 22.1 Critical Priority (Blocking)
1. **DataFrame Support with Polars** - Core feature for README examples
2. **Result Type with ? Operator** - Essential error handling
3. **Actor System Implementation** - Concurrency foundation

### 22.2 High Priority
4. **Async/Await Support** - Modern async programming
5. **Struct Definitions** - Custom types
6. **Impl Blocks** - Methods and associated functions

### 22.3 Medium Priority  
7. **Trait Definitions** - Polymorphism and interfaces
8. **Match Expressions** - Advanced pattern matching
9. **Enum Types** - Sum types and variants
10. **Module System** - Code organization

### 22.4 Future Roadmap
- JIT Compilation (Cranelift integration)
- Property Testing Framework
- SMT Solver for Refinement Types
- WebAssembly Target
- GPU Acceleration (CUDA/OpenCL)
- Package Manager and Registry

## 23. Project Status

### 23.1 Current Metrics (v0.2.1)

```
Quality Dashboard
┌─────────────────────────────────────────┐
│ Build:      ✅ Compiles clean            │
│ Lint:       ✅ 0 clippy errors          │
│ Tests:      🟡 221/229 (96.5%)         │
│ Coverage:   🟡 77.91% (target: 80%)    │
│ SATD:       ✅ 0 comments              │
│ Complexity: ✅ Max 10 (target: 10)     │
│ File Size:  ✅ Max 500 lines           │
└─────────────────────────────────────────┘
```

### 23.2 Performance Achievements
- **REPL startup**: <10ms ✅ (achieved)
- **Parse throughput**: >50MB/s ✅ (achieved)  
- **Type inference**: <15ms for typical programs ✅
- **Binary size**: 4.2MB ✅ (target: <5MB)

### 23.3 Velocity Tracking
- **Features/week**: ~5 (sustained)
- **Debt reduction**: -262 clippy warnings eliminated
- **Test growth**: +49 tests added in v0.2.1
- **Code quality**: Maintained zero SATD policy

## 24. Deep Context

### 24.1 Codebase Metrics

```
Complexity Analysis (Jan 2025)
┌─────────────────────────────────────────┐
│ Total Files:     89                     │
│ Total Lines:     15,847                 │
│ Test Coverage:   77.91%                 │
│ Max Complexity:  10 (target achieved)   │
│ Avg Complexity:  3.2                   │
│ Functions >10:   0 (zero violations)    │
│ SATD Comments:   0 (zero violations)    │
└─────────────────────────────────────────┘
```

### 24.2 Module Breakdown

| Module | Lines | Complexity | Coverage | Status |
|--------|-------|------------|----------|--------|
| Parser | 2,847 | 8.2 avg | 82% | ✅ Stable |
| Type Inference | 1,923 | 7.1 avg | 89% | ✅ Stable |
| Transpiler | 3,456 | 6.8 avg | 75% | 🟡 Needs work |
| REPL | 1,689 | 5.2 avg | 91% | ✅ Stable |
| AST | 1,234 | 3.1 avg | 95% | ✅ Stable |

### 24.3 Technical Debt Analysis
- **Zero SATD comments** maintained across entire codebase
- **Zero clippy warnings** with `-D warnings` enforcement
- **Cognitive complexity** under control (max 10)
- **File size discipline** enforced (max 500 lines)

## 25. PMAT Integration

### 25.1 Real-Time Quality Enforcement

```toml
# pmat.toml - MCP proxy blocks violations instantly
[thresholds]
cyclomatic_complexity = 10      # Blocks at write-time
cognitive_complexity = 15        # No mental overload
halstead_effort = 5000          # Computational limits
maintainability_index = 70      # Minimum maintainability
test_coverage = 80              # Coverage gate
satd_comments = 0               # Zero technical debt
mutation_score = 75             # Mutation testing gate
```

### 25.2 MCP Quality Proxy Tools

```bash
# PMAT exposes these MCP tools to Claude:
pmat_analyze_code       # Real-time complexity analysis
pmat_check_coverage     # Test coverage verification  
pmat_detect_smells      # Code smell detection
pmat_suggest_refactor   # Automated refactoring hints
pmat_mutation_test      # Mutation testing on-demand
pmat_quality_gate       # Full quality check
```

### 25.3 Live Quality Feedback

```rust
// As you type, PMAT MCP provides instant feedback:
fn process_data(data: &Data) -> Result<(), Error> {
    // PMAT: Complexity 3/10 ✅
    validate(data)?;
    
    if data.complex {  // PMAT: +1 complexity (4/10)
        for item in &data.items {  // PMAT: +2 (6/10)
            if item.check() {  // PMAT: +3 nested (9/10) ⚠️
                // PMAT WARNING: Approaching complexity limit
                process_item(item)?;
            }
        }
    }
    Ok(())
}
```

## 26. PDMT Integration

### 26.1 Deterministic Content Generation

YAML-based templates with quality enforcement:
```yaml
id: todo_list_generation
template_version: "1.0"
validation:
  quality_gates:
    max_complexity_per_task: 8
    require_time_estimates: true
    enforce_priority_ordering: true
output_format: "structured_markdown"
```

### 26.2 Project Scaffolding

```yaml
id: ruchy_project_scaffold
generates:
  - Cargo.toml with ruchy dependencies
  - src/lib.rs with proper imports
  - build.rs for transpilation
  - .gitignore with ruchy-gen/ exclusion
  - docs/ structure following CLAUDE.md
quality_enforcement:
  initial_coverage: 80%
  complexity_budget: 10
  satd_tolerance: 0
```

## 27. External Tool Dependencies

### 27.1 Core Dependencies

- **Rust Ecosystem**: Full access via transpilation
- **Polars**: DataFrame operations backend
- **Cranelift**: JIT compilation engine
- **syn**: Rust AST manipulation
- **proc-macro2**: Token stream processing

### 27.2 Quality Tools

- **PMAT**: Quality enforcement and metrics
- **PDMT**: Deterministic content generation
- **clippy**: Linting with `-D warnings`
- **cargo-tarpaulin**: Coverage measurement
- **proptest**: Property-based testing

### 27.3 Integration Tools

- **Depyler**: Python-to-Rust transpilation
- **pmcp**: MCP protocol implementation
- **Blake3**: Content hashing for caches
- **DashMap**: Concurrent caching
- **tokio**: Async runtime

---

## Summary

Ruchy represents a complete systems-oriented scripting language achieving Python ergonomics with Rust performance through mechanical transpilation. The design prioritizes:

1. **Zero-cost abstractions** - Every feature compiles to efficient Rust
2. **Progressive complexity** - Simple by default, powerful when needed  
3. **DataFrame-first** - Polars as the primary collection type
4. **Quality enforcement** - Built-in property testing and verification
5. **MCP-native** - First-class AI/LLM integration
6. **Mathematical computing** - Statistical and linear algebra capabilities
7. **Actor concurrency** - Erlang-style fault tolerance
8. **Performance predictability** - <10ms startup, deterministic latency

The implementation leverages proven technologies while introducing minimal novel concepts, ensuring predictable performance and maintainable code with comprehensive quality gates.

---

*Version: 3.0 | Last Updated: 2025-01-17*  
*Total Specifications: 27 | Lines of Specification: ~15,000+*  
*This document consolidates ALL Ruchy specifications into a single source of truth.*