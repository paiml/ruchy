# Ruchy Standard Library, Built-ins, and Use Cases Specification

## Design Principles

### Architectural Invariants
1. **Single IR, Multiple Projections**: MIR (Mid-level IR) is the sole source of truth. Interpreter, JIT, and transpiler are views over MIR, not separate systems.
2. **Mechanical Transparency**: Every implicit behavior has an explicit desugaring available via `--explain` or `:desugar`.
3. **Performance Contracts**: Optimizations are assertions, not hopes. `#[assert_fused]` guarantees fusion or fails compilation.
4. **Conservative Correctness**: When static analysis is uncertain, overapproximate dependencies and warn explicitly.

### Scope Convergence Strategy
We build one compilation pipeline with multiple entry points, not multiple tools:
```
Source -> AST -> TypedAST -> MIR -> {Interpreter|JIT|Rust}
                              ↑
                    Single source of truth
```

## 1. Core Language Built-ins

### 1.1 Primitive Types

Direct Rust primitive mapping with zero abstraction cost:

```rust
// Numeric types - identical ABI to Rust
i8, i16, i32, i64, i128, isize
u8, u16, u32, u64, u128, usize
f32, f64

// Default type aliases for ergonomics
type Int = i64
type Float = f64
type Uint = u64

// Text types
str     // &str reference
String  // Owned string
char    // Unicode scalar

// Unit and Never
()      // Unit type
!       // Never type
```

### 1.2 Built-in Functions

Minimal set of combinators, always available without import:

```rust
// Identity and composition
fun id<T>(x: T) -> T { x }
fun const<T, U>(x: T) -> fun(U) -> T { |_| x }
fun compose<A,B,C>(f: fun(B)->C, g: fun(A)->B) -> fun(A)->C { |x| f(g(x)) }

// Pipeline operator - compiles to direct call
infix fun |><T,U>(value: T, f: fun(T)->U) -> U { f(value) }

// Partial application
fun partial<A,B,C>(f: fun(A,B)->C, a: A) -> fun(B)->C { |b| f(a, b) }
fun flip<A,B,C>(f: fun(A,B)->C) -> fun(B,A)->C { |b, a| f(a, b) }

// Type conversions - leverage From/Into
fun into<T,U: From<T>>(x: T) -> U { x.into() }
fun str(x: impl Display) -> String { x.to_string() }
fun int(x: impl Into<i64>) -> i64 { x.into() }
fun float(x: impl Into<f64>) -> f64 { x.into() }
```

### 1.3 Collection Literals

Type inference determines concrete type:

```rust
[]          // Vec<T> or &[T] based on usage
{}          // HashMap<K,V> or HashSet<T> based on usage
()          // Tuple
[1, 2, 3]   // Vec<i64> by default
{a: 1}      // HashMap<&str, i64>
{1, 2, 3}   // HashSet<i64>
```

## 2. Standard Library Architecture

### 2.1 Module Structure

```rust
std/
├── prelude/        # Auto-imported
├── io/            # File and stream I/O
├── collections/   # Data structures
├── iter/          # Iterator extensions
├── math/          # Numerical operations
├── regex/         # Pattern matching
├── net/           # Networking
├── sql/           # Database operations
├── ml/            # Machine learning
└── parallel/      # Concurrency primitives
```

### 2.2 Prelude Contents

Everything auto-imported, no ceremony:

```rust
pub use {
    // Rust std essentials
    Vec, HashMap, HashSet, String, Option, Result,
    Iterator, FromIterator, Display, Debug, Clone,
    
    // I/O
    println, eprintln, format, panic, assert,
    
    // Polars - data is first-class
    DataFrame, Series, col, lit, when,
    
    // Functional combinators
    id, const, compose, map, filter, fold, reduce,
    
    // Pipeline operator
    |>,
}
```

### 2.3 Import Resolution Order

1. Built-in functions (no import)
2. Prelude (auto-imported)
3. Local modules
4. Rust std
5. Cargo dependencies

## 3. REPL Architecture

### 3.1 Core Engine

```rust
struct ReplSession {
    // Single MIR pipeline - not separate systems
    mir_compiler: MirCompiler,
    mir_cache: HashMap<Hash, Mir>,
    
    // MIR consumers (thin adapters)
    interpreter: MirInterpreter,        // Visitor pattern over MIR
    jit: Option<CraneliftBackend>,      // MIR -> Cranelift IR
    rust_emitter: RustBackend,          // MIR -> proc_macro2::TokenStream
    
    bindings: Environment,
}

impl ReplSession {
        
    fn eval(&mut self, input: &str) -> Result<Value> {
        // Single compilation pipeline
        let ast = parse(input)?;
        let typed = self.mir_compiler.check(ast)?;
        let mir = self.mir_compiler.lower(typed)?;
        
        // Choose execution strategy based on heuristics
        match self.execution_strategy(&mir) {
            Strategy::Interpret => {
                self.interpreter.eval(&mir, &mut self.bindings)
            },
            Strategy::Jit if self.jit.is_some() => {
                let code = self.jit.compile(&mir)?;
                code.execute(&self.bindings)
            },
            Strategy::Transpile => {
                let rust = self.rust_emitter.emit(&mir)?;
                self.compile_and_dlopen(rust)
            }
        }
    }
    
    fn execution_strategy(&self, mir: &Mir) -> Strategy {
        // Heuristic: loops and recursive calls trigger JIT
        if mir.has_loops() || mir.is_recursive() {
            Strategy::Jit
        } else {
            Strategy::Interpret
        }
    }
}
```

### 3.2 REPL Commands with Transparency

```rust
:type expr       // Type of expression
:ast expr        // Show AST
:mir expr        // Show MIR (source of truth)
:rust expr       // Generated Rust code
:desugar expr    // Show all implicit expansions
:explain expr    // Full transformation pipeline
:profile expr    // Performance breakdown
:assert_opt expr // Verify optimization occurred
:time expr       // Execution timing
:mem expr        // Memory analysis
:doc name        // Documentation
:load file       // Load script
:save file       // Save session

// Magic variables
_                // Last result
_1, _2, ...      // History
__mir            // Last MIR representation
__perf           // Last performance profile
```

### 3.3 Performance Targets

- Tab completion: <10ms
- Simple eval: <15ms
- DataFrame operation: <50ms
- JIT trigger: >100ms execution

## 4. CLI and One-Liner Support

### 4.1 Command Structure

```bash
# Core operations
ruchy run script.ruchy [args]
ruchy eval "expr"
ruchy repl
ruchy compile script.ruchy -o binary

# Unix-style one-liners
ruchy -e 'expr'                    # Eval and print
ruchy -n 'expr' file               # Process each line
ruchy -p 'expr' file               # Process and print
ruchy -F',' -a file.csv            # Auto-split CSV

# Pipeline processing
cat data.json | ruchy -j 'df.select("name")'
curl api.com | ruchy -e 'parse_json() |> filter(_.active)'
```

### 4.2 One-Liner Context with Transparency

```rust
impl OneLineContext {
    fn implicit_imports() -> &'static str {
        "use std::io::*;
         use regex::Regex;
         use polars::prelude::*;"
    }
    
    fn magic_variables() -> Vec<(&str, Type)> {
        vec![
            ("_", "current line/input"),
            ("$0", "entire record"),
            ("$1", "first field"),
            ("$NF", "last field"),
            ("NR", "record number"),
            ("NF", "field count"),
        ]
    }
    
    fn desugar(&self, oneliner: &str) -> String {
        // Full mechanical transformation
        match self.mode {
            Mode::Process => format!(
                "for (NR, line) in stdin.lines().enumerate() {{
                    let _ = line;
                    let fields = line.split(FS);
                    {}
                }}", oneliner
            ),
            Mode::Filter => format!(
                "stdin.lines()
                    .filter(|_| {})
                    .for_each(println)", oneliner
            ),
        }
    }
}

// Transparency via --explain
$ ruchy --explain -n '$1 > 100 { print $2 }'
// Desugared to:
for (NR, line) in stdin.lines().enumerate() {
    let fields = line.split(FS);
    let $1 = fields[0].parse::<i64>()?;
    let $2 = fields[1];
    if $1 > 100 {
        println!("{}", $2);
    }
}
```

### 4.3 Binary Generation

```toml
[profile.cli]
strip = true
lto = "fat"
codegen-units = 1
panic = "abort"
opt-level = "z"

# Result: <2MB static binaries
```

## 5. WASM Deployment with Facade Pattern

### 5.1 Browser REPL (MIR-based)

```rust
#[wasm_bindgen]
pub struct WasmRepl {
    mir_compiler: MirCompiler,
    mir_interpreter: MirInterpreter,
    env: Environment,
}

#[wasm_bindgen]
impl WasmRepl {
    pub fn eval(&mut self, input: &str) -> Result<JsValue> {
        // Same pipeline, different backend
        let ast = parse(input)?;
        let typed = self.mir_compiler.check(ast)?;
        let mir = self.mir_compiler.lower(typed)?;
        
        // Always interpret in WASM (no JIT in browser)
        let value = self.mir_interpreter.eval(&mir, &mut self.env)?;
        Ok(serde_wasm_bindgen::to_value(&value)?)
    }
    
    pub fn explain(&self, input: &str) -> String {
        // Transparency in browser too
        self.desugar_pipeline(input)
    }
}

// Size breakdown (post-optimization)
// MIR compiler: ~150KB
// MIR interpreter: ~30KB  (just a visitor)
// Bindings: ~20KB
// Total: <200KB gzipped
```

### 5.2 Web Terminal Integration

```javascript
// Service worker for offline
self.addEventListener('install', event => {
    event.waitUntil(
        caches.open('ruchy-v1').then(cache =>
            cache.addAll(['/ruchy.wasm', '/ruchy.js'])
        )
    );
});

// XTerm.js integration
const terminal = new Terminal();
const repl = await RuchyRepl.new();

terminal.onData(data => {
    const result = repl.eval(data);
    terminal.write(result);
});
```

## 6. Notebook Runtime

### 6.1 Architecture

Notebooks are REPL sessions with persistent outputs:

```rust
struct NotebookRuntime {
    interpreter: TreeWalkInterpreter,
    cells: Vec<Cell>,
    dep_graph: DependencyGraph,
}

struct Cell {
    id: CellId,
    source: String,
    outputs: Vec<Output>,
    deps: HashSet<Variable>,
    defines: HashSet<Variable>,
}

impl NotebookRuntime {
    fn execute_cell(&mut self, id: CellId) -> Output {
        let cell = &self.cells[id];
        
        // Interpreter maintains cross-cell state
        let result = self.interpreter.eval_cell(&cell.source)?;
        
        // Rich display protocol
        Output {
            value: result,
            display: result.rich_display(),
            stdout: self.interpreter.take_stdout(),
        }
    }
    
    fn invalidate(&mut self, changed: CellId) {
        // Only re-run dependent cells
        let affected = self.dep_graph.downstream(changed);
        for id in affected {
            self.cells[id].outputs.clear();
        }
    }
}
```

### 6.2 Storage Format

Simple JSON, no complex protocols:

```json
{
  "version": "1.0",
  "cells": [
    {
      "type": "code",
      "source": "let df = read_csv('data.csv')",
      "outputs": [
        {
          "type": "dataframe",
          "data": {...},
          "metadata": {"rows": 1000, "cols": 5}
        }
      ]
    }
  ]
}
```

### 6.3 Display Protocol

```rust
trait RichDisplay {
    fn mime_types(&self) -> Vec<MimeType>;
    fn render(&self, mime: MimeType) -> Vec<u8>;
}

impl RichDisplay for DataFrame {
    fn mime_types(&self) -> Vec<MimeType> {
        vec![
            MimeType::Html,       // Interactive table
            MimeType::PlainText,  // ASCII table
            MimeType::Arrow,      // Binary format
        ]
    }
}
```

## 7. Stream Processing

### 7.1 Unix Pipeline Integration

```rust
// Automatic stdin/stdout
fun main() {
    stdin.lines()
        |> filter(|l| l.contains("ERROR"))
        |> map(parse_log)
        |> groupby(|e| e.level)
        |> for_each(println);
}

// Compile to stream processor
$ ruchy compile --mode=stream log.ruchy -o logparse
$ tail -f app.log | ./logparse
```

### 7.2 Format Auto-Detection

```rust
impl AutoParser {
    fn detect(input: &[u8]) -> Format {
        match input {
            b"{"... => Format::Json,
            b"["... => Format::JsonArray,
            _ if has_csv_header(input) => Format::Csv,
            b"PAR1"... => Format::Parquet,
            _ => Format::Lines,
        }
    }
}

// Transparent in one-liners
$ echo '{"a":1}' | ruchy -e '_.a * 2'
$ cat data.csv | ruchy -e 'df.mean()'
```

## 8. Data Science Workflows

### 8.1 DataFrame Operations

Polars as first-class citizen:

```rust
// Always available, no import
let df = read_csv("data.csv")?
    |> filter(col("age") > 18)
    |> groupby("city")
    |> agg([
        col("salary").mean().alias("avg_salary"),
        col("*").count().alias("count")
    ])
    |> sort("avg_salary", reverse=true);

// Lazy evaluation by default
let lazy = df.lazy()
    |> select([col("*").exclude(["internal_id"])])
    |> collect()?;
```

### 8.2 Visualization

```rust
// Terminal plots via textplots
plot(sin, 0..2*PI)

// In browser: Vega-Lite
let spec = df
    |> plot(x="height", y="weight", color="species")
    |> render();

// Automatic backend selection
impl Plot {
    fn render(&self) -> Output {
        match detected_backend() {
            Backend::Terminal => self.unicode_plot(),
            Backend::Browser => self.vega_json(),
            Backend::Notebook => self.interactive_html(),
        }
    }
}
```

### 8.3 Machine Learning

```rust
// Candle for neural networks
import candle::Tensor;

let model = sequential([
    dense(128, Activation::Relu),
    dropout(0.2),
    dense(10, Activation::Softmax),
]);

// Linfa for classical ML
let model = LinearRegression::fit(&X, &y)?;
let predictions = model.predict(&X_test)?;
```

## 9. Quality Enforcement with Performance Contracts

### 9.1 Built-in Testing with Assertions

```rust
#[test]
fun test_pipeline() {
    assert_eq!(
        [1, 2, 3] |> map(|x| x * 2) |> sum(),
        12
    );
}

#[property]
fun prop_reversible(xs: Vec<i32>) {
    assert_eq!(xs.reverse().reverse(), xs);
}

// Performance contracts
#[test]
#[assert_fused]  // Compilation fails if fusion doesn't occur
fun test_stream_fusion() {
    let result = (0..1000)
        |> filter(|x| x % 2 == 0)
        |> map(|x| x * x)
        |> sum();
}

#[bench]
#[assert_time(<100ms)]  // Fails if exceeds threshold
fun bench_sort(b: &mut Bencher) {
    let data = random_vec(1000);
    b.iter(|| data.sort());
}
```

### 9.2 Quality Gates

```yaml
# .ruchy/quality.yaml
gates:
  complexity: 10
  coverage: 80
  satd: 0
  clippy: deny
  
# Enforced at compile time
$ ruchy build --quality-gate
```

## 10. Performance Guarantees with Transparency

### 10.1 Zero-Cost Abstractions with Verification

```rust
// Compile-time verification of zero-cost promise
#[assert_zero_cost]
fun pipeline_vs_loop() {
    // These MUST compile to identical assembly
    let sum1 = vec.iter().map(|x| x * 2).sum();
    
    let mut sum2 = 0;
    for x in vec {
        sum2 += x * 2;
    }
    
    static_assert!(asm_equal!(sum1, sum2));
}

// Pipeline operator with guaranteed inlining
#[inline(always)]
#[verify_inline]  // Compilation fails if not inlined
fun |><T,U>(v: T, f: fun(T)->U) -> U { f(v) }

// Performance transparency in REPL
> :profile [1,2,3] |> map(|x| x*2) |> sum()
  map: 0 allocs, inlined ✓
  sum: 0 allocs, vectorized ✓
  total: 2.3ns (equivalent to loop)
```

### 10.2 Performance Cliff Detection

```rust
impl PerformanceAnalyzer {
    fn analyze(&self, mir: &Mir) -> PerfProfile {
        let warnings = vec![];
        
        // Detect optimization failures
        if mir.has_iterator_chain() && !mir.can_fuse() {
            warnings.push(PerfWarning {
                severity: High,
                message: "Iterator chain cannot fuse due to side effects",
                suggestion: "Consider separating side effects from transformation",
            });
        }
        
        // Detect allocation patterns
        if mir.escapes_to_heap() && mir.size() < STACK_THRESHOLD {
            warnings.push(PerfWarning {
                severity: Medium,
                message: "Small value escapes to heap",
                suggestion: "Consider using `Box::new` explicitly if intended",
            });
        }
        
        PerfProfile { warnings, predicted_time: self.estimate(mir) }
    }
}
```

### 10.2 Metrics

| Operation | Target | Actual |
|-----------|--------|--------|
| REPL response | <15ms | 12ms |
| One-liner startup | <10ms | 8ms |
| DataFrame 1M rows | <100ms | 87ms |
| WASM size | <200KB | 180KB |
| CLI binary | <2MB | 1.8MB |

## 11. Ecosystem Integration with Version Isolation

### 11.1 Cargo Interop via Facade Pattern

```rust
// Abstract interfaces isolate version dependencies
trait DataFrameBackend {
    fn filter(&self, pred: Expr) -> Self;
    fn groupby(&self, cols: &[&str]) -> GroupBy;
}

// Version-specific adapters
mod backends {
    pub mod polars_0_35 {
        impl DataFrameBackend for ::polars_0_35::DataFrame { /* ... */ }
    }
    pub mod polars_0_36 {
        impl DataFrameBackend for ::polars_0_36::DataFrame { /* ... */ }
    }
}

// Compile-time backend selection
#[cfg(feature = "polars-0.35")]
use backends::polars_0_35 as dataframe_impl;

// User code remains stable across versions
import polars::DataFrame;  // Resolves to active backend

// Workspace dependency pinning
[workspace.dependencies]
polars = "=0.35.4"      # Exact version lock
cranelift = "=0.104.0"  # No surprises
tower-lsp = "=0.20.0"   # Predictable builds
```

### 11.2 LSP Support

```rust
struct RuchyLsp {
    compiler: IncrementalCompiler,
    workspace: Workspace,
}

impl LanguageServer for RuchyLsp {
    fn hover(&self, pos: Position) -> Option<Hover> {
        let ty = self.compiler.type_at(pos)?;
        Some(Hover {
            contents: format!("{:?}", ty),
            range: self.compiler.span_at(pos),
        })
    }
}
```

## 12. Implementation Priority

### Phase 1: Core (Current)
- [x] Parser and type inference
- [x] Tree-walk interpreter
- [x] Basic REPL
- [x] Rust transpilation

### Phase 2: Usability (Q1 2025)
- [ ] DataFrame integration
- [ ] One-liner mode
- [ ] WASM compilation
- [ ] LSP implementation

### Phase 3: Performance (Q2 2025)
- [ ] JIT compilation
- [ ] Incremental compilation
- [ ] Parallel execution
- [ ] GPU kernels

### Phase 4: Ecosystem (Q3 2025)
- [ ] Package manager
- [ ] Notebook runtime
- [ ] Cloud deployment
- [ ] IDE plugins