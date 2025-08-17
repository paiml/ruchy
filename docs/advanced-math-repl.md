# Advanced Mathematical REPL Specification

## Executive Summary

This specification defines a data science REPL that achieves interpreted ergonomics with compiled performance. The architecture leverages Rust's zero-cost abstractions to eliminate the two-language problem while maintaining sub-10ms startup latency.

## Core Architecture

### Execution Model

```rust
// Tiered compilation with explicit thresholds
enum ExecutionMode {
    Interpret,           // First 2 evaluations
    JitCompile,         // 3+ evaluations (via Cranelift)
    AotTranspile,       // Persistent definitions
}

impl ExecutionStrategy {
    fn select(&self, expr: &Expr, heat: u32) -> ExecutionMode {
        match (expr.has_side_effects(), heat, expr.is_definition()) {
            (_, _, true) => ExecutionMode::AotTranspile,
            (false, 3.., false) => ExecutionMode::JitCompile,
            _ => ExecutionMode::Interpret,
        }
    }
}

### Memory Strategy

```rust
// Three-tier memory model
enum ValueStorage {
    Stack(StackValue),      // Primitives, small arrays (<256 bytes)
    Arena(ArenaRef),        // Session-scoped allocations
    Persistent(Arc<Value>), // Cross-session, reference-counted
}
```

## Mathematical Subsystems

### 1. DataFrame Engine (Priority: Critical)

**Backend**: Polars with zero-copy integration

```rust
// Transpilation directly to Polars' lazy API
df = read_csv("data.csv")
  |> filter(col("value") > threshold)
  |> groupby("category")
  |> agg([
      mean("x").alias("x_mean"),
      std("x").alias("x_std"),
      quantile("x", 0.95).alias("x_p95")
  ])
  |> sort("x_mean", descending=true)

// Transpiles to:
let df = LazyFrame::scan_csv("data.csv", Default::default())
    .filter(col("value").gt(threshold))
    .groupby([col("category")])
    .agg([
        col("x").mean().alias("x_mean"),
        col("x").std().alias("x_std"),
        col("x").quantile(0.95, QuantileInterpolOptions::Linear).alias("x_p95")
    ])
    .sort("x_mean", SortOptions { descending: true, ..Default::default() })
    .collect()?;
```

**Zero-copy guarantee**: DataFrame operations generate Polars LogicalPlan objects directly. No intermediate representations.

### 2. Linear Algebra Kernel

**Backend**: `ndarray` for data science, `nalgebra` for geometric operations

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

### 3. Symbolic Mathematics (Constrained Scope)

**Implementation Boundary**: 10K LOC maximum, no external CAS dependencies

```rust
// Core expression tree - simplified for maintainability
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

**Explicitly deferred (not in roadmap)**:
- Integration (requires Risch algorithm)
- Equation solving (requires Gröbner basis)
- Limit evaluation (requires series expansion)

### 4. Statistical Computing

**Backend**: Custom implementations for core statistics, `linfa` for ML

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

### 5. Visualization System

**Terminal**: Unicode plots via `textplots`
**Web**: Vega-Lite JSON generation

```rust
// Automatic backend selection
plot(sin, 0..2π)  // Unicode in terminal, SVG in notebook

// Grammar of graphics (ggplot2-inspired)
g = ggplot(df, aes(x="height", y="weight"))
  |> geom_point(alpha=0.5)
  |> geom_smooth(method="lm")
  |> facet_wrap("species")
  |> theme_minimal()

// Terminal rendering
┌────────────────────────────────┐
│     ●  ●●                      │
│   ●●  ●  ●●                    │
│  ● ● ●●    ●●                  │
│ ●   ●  ●●    ●●   ●            │
│●          ●●   ●●● ●●●         │
└────────────────────────────────┘
```

## Display Protocol

### Type-Directed Rendering

```rust
// Single trait, multiple representations
trait RichDisplay {
    fn capabilities(&self) -> &[MimeType];
    fn render(&self, target: MimeType) -> Result<Vec<u8>>;
}

// Terminal capability detection at startup
static RENDERER: Lazy<Renderer> = Lazy::new(|| {
    match term::capabilities() {
        Caps::Sixel => Renderer::Sixel,     // Full raster graphics
        Caps::Unicode => Renderer::Unicode,  // Box drawing, blocks
        Caps::Ansi => Renderer::Ansi,       // Colors only
        _ => Renderer::Ascii,               // Pure text
    }
});

// Automatic format negotiation
impl ReplDisplay for Value {
    fn display(&self, out: &mut dyn Write) -> Result<()> {
        let mime = self.negotiate_format(RENDERER.supported_types());
        out.write_all(&self.render(mime)?)
    }
}

## Performance Instrumentation

### Zero-Overhead Profiling

```rust
// Compile-time erasure in release builds
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

// Sampling profiler integration (pprof-rs)
impl Profiler {
    fn report(&self) -> FlameGraph {
        // 1% sampling rate for <1% overhead
        // Automatic symbol demangling
        // Inline frame expansion
    }
}

// Custom allocator for precise tracking
#[global_allocator]
static ALLOC: TrackingAllocator = TrackingAllocator::new(System);
```

## Workspace Management

### Session Persistence

```rust
// Workspace operations
whos()              // List variables with types/sizes
clear!(r"temp_.*")  // Regex-based clearing
checkpoint()        // Save session state

// Serialization format (MessagePack)
struct Workspace {
    variables: HashMap<String, Value>,
    functions: HashMap<String, CompiledFn>,
    imports: Vec<ImportPath>,
    history: Vec<Command>,
}
```

### History Search

```rust
// Embedding-based semantic search
struct HistoryIndex {
    commands: Vec<Command>,
    embeddings: Array2<f32>,  // Sentence-BERT vectors
    index: HnswIndex,         // Hierarchical Navigable Small World graph
}

impl HistoryIndex {
    fn search_semantic(&self, query: &str, k: usize) -> Vec<&Command> {
        let query_vec = self.embed(query);
        let neighbors = self.index.search(&query_vec, k);
        neighbors.into_iter()
            .map(|idx| &self.commands[idx])
            .collect()
    }
    
    fn embed(&self, text: &str) -> Array1<f32> {
        // Pre-trained 384-dim model, quantized to 30MB
        // Runs locally, no network dependency
        EMBEDDING_MODEL.encode(text)
    }
}

## Implementation Phases

### Phase 1: Foundation (3 months)
- DataFrame operations (Polars integration)
- Basic plotting (Unicode terminal)
- Workspace management
- Performance instrumentation

### Phase 2: Mathematics (3 months)
- Matrix operations (ndarray)
- Statistical functions
- Formula parser
- Basic symbolic differentiation

### Phase 3: Interactivity (6 months)
- Reactive dependency tracking
- Web-based notebook frontend
- Interactive widgets
- Multi-modal display

### Phase 4: Advanced (6+ months)
- Full symbolic mathematics
- Distributed computation
- GPU acceleration
- Package ecosystem

## Technical Decisions

### Why Not Embed Python/R?
Embedding introduces:
- 200MB+ runtime overhead
- GIL/GVL synchronization complexity
- FFI marshalling costs
- Version management burden

Native implementations provide:
- 10-100x faster execution
- Predictable memory usage
- Seamless Rust integration
- Compile-time optimization

### Symbolic Math Scope Limitation
Full CAS implementation requires:
- Gröbner basis algorithms
- Risch integration
- Cylindrical algebraic decomposition
- ~500K LOC minimum

MVP focuses on:
- Polynomial manipulation
- Derivative computation
- Pattern-based simplification
- ~10K LOC achievable

### DataFrame Backend Selection
Polars chosen over alternatives:
- **vs pandas**: 5-10x faster, no Python overhead
- **vs DataFusion**: Better ergonomics, mature API
- **vs custom**: 5+ person-years saved

## Success Metrics

- **Startup time**: <10ms cold, <1ms warm
- **DataFrame operations**: Within 2x of native Polars
- **Plot rendering**: <50ms for 10K points
- **Memory overhead**: <50MB base footprint
- **Feature parity**: 80% of R/Julia workflows supported

## Appendix: Competition Analysis

| Feature | R | Julia | Wolfram | MATLAB | Ruchy |
|---------|---|-------|---------|--------|-------|
| Startup | 200ms | 150ms | 500ms | 2000ms | 10ms |
| DataFrame | tidyverse | DataFrames.jl | Dataset | table | Polars |
| Symbolic | Limited | SymbolicUtils | Full CAS | Symbolic Toolbox | Basic |
| Reactive | No | Pluto.jl | Dynamic | No | Yes |
| Compilation | No | JIT | No | JIT | AOT+JIT |
| Package Ecosystem | 20K+ | 8K+ | Built-in | 5K+ | Rust ecosystem |

## References

- Polars Architecture: https://pola.rs/internals
- Pluto.jl Reactivity: https://github.com/fonsp/Pluto.jl/wiki
- IPython Display Protocol: https://ipython.readthedocs.io/en/stable/
- R Formula Syntax: https://stat.ethz.ch/R-manual/R-devel/library/stats/html/formula.html