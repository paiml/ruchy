# Ruchy: Unified Language Specification
## Rust Syntax with Python Simplicity - One Way to Do Things

### Core Principle: Rust Semantics, Simplified Syntax

Ruchy uses Rust's type system and ownership model with streamlined syntax. One keyword per concept, no redundancy. Every line compiles to safe, zero-cost Rust.

## Language Syntax

### Imports: Rust Style, Zero Ambiguity

```rust
// Single import syntax - Rust's use statement
use std::collections::{HashMap, BTreeMap};
use numpy as np;
use tokio::time::{sleep, timeout};

// Direct crate usage
use serde::{Serialize, Deserialize};
use rayon::prelude::*;

// Compiler enforces:
// - Prelude imports allowed (curated safe subsets)
// - Static resolution at compile time
// - Zero runtime cost
```

### Type System: Rust Types, Inferred Locals

```rust
// Function signatures require types
fun analyze(data: Vec<f64>) -> DataFrame {
    // Local inference within function body
    let result = data.iter().map(|x| x * 2.0).collect();
    DataFrame::from(result)
}

// Type aliases for domain modeling
type PositiveVec = Vec<f64> where |v| v.iter().all(|x| *x > 0.0);

fun log_transform(data: PositiveVec) -> Vec<f64> {
    // Compiler proves no domain errors
    data.iter().map(|x| x.ln()).collect()
}
```

### Functions: `fun` Keyword

```rust
// Functions use 'fun' exclusively
fun process(data: DataFrame, threshold: f64) -> Result<Stats, Error> {
    // Rust match syntax
    match data.validate() {
        Ok(valid) => Ok(calculate_stats(valid, threshold)),
        Err(e) => Err(ProcessError::validation(e)),
    }
}

// Async with simplified syntax
async fun fetch_data(url: &str) -> Result<DataFrame> {
    let response = http::get(url).await?;
    parse_csv(response.body())
}

// Closures remain Rust-style
let squared = |x: i32| -> i32 { x * x };
```

### Structs: Rust with Auto-Derive

```rust
// Structs with automatic common derives
struct Point {
    x: f64,
    y: f64,
}

// Compiler auto-generates: Debug, Clone, PartialEq
impl Point {
    fun distance(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

// Traits for composition
trait Comparable {
    fun compare(&self, other: &Self) -> Ordering;
}

// Actors for concurrency
#[actor]
struct DataProcessor {
    state: DataFrame,  // Owned by actor
}

impl DataProcessor {
    async fun process(&mut self, msg: Message) -> Response {
        self.state = transform(self.state, msg);
        Response::from(self.state.summary())
    }
}
```

### Ownership: Rust Rules, Clear Syntax

```rust
// Move semantics by default
fun transform(df: DataFrame) -> DataFrame {
    df.filter(|row| row.value > 0.0)
      .normalize()
}

// Borrowing explicit
fun preview(df: &DataFrame) -> &[Row] {
    &df.rows[..10]
}

// Mutable borrows
fun append_column(df: &mut DataFrame, col: Column) {
    df.columns.push(col);
}
```

### Pattern Matching: Rust Match

```rust
// Full Rust pattern matching
match value {
    Some(x) if x > 0 => process(x),
    Some(0) => handle_zero(),
    Some(x) => handle_negative(x),
    None => default_value(),
}

// Destructuring in let bindings
let Point { x, y } = calculate_center();

// If-let for simple cases
if let Some(data) = fetch_optional() {
    process(data);
}
```

### Error Handling: Result with ? Operator

```rust
fun read_and_process(path: &Path) -> Result<Stats> {
    let content = fs::read_to_string(path)?;
    let data = parse_csv(&content)?;
    Ok(analyze(data))
}

// Custom error types
#[derive(Error)]
enum ProcessError {
    Io(#[from] io::Error),
    Parse(#[from] ParseError),
    Validation(String),
}
```

### Collections: Rust Style with Comprehensions

```rust
// Rust iterators with method chaining
let squares: Vec<i32> = (0..100).map(|x| x * x).collect();
let filtered = data.iter().filter(|x| **x > threshold).collect();

// List comprehensions for data science workflows
let squares = [x * x for x in 0..100];
let filtered = [x for x in data if x > threshold];

// Set and map comprehensions
let unique = {x % 10 for x in data};
let word_lengths = {word: word.len() for word in text.split_whitespace()};
```

## Memory Safety Architecture

### Compile-Time Verification

```rust
// Every operation verified at compile time
fun safe_index<T>(data: &[T], index: usize) -> Option<&T> {
    if index < data.len() {
        Some(&data[index])  // Proven safe
    } else {
        None
    }
}

// Lifetime inference prevents use-after-free
fun get_max<'a>(data: &'a [f64]) -> &'a f64 {
    data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
}
```

### Zero-Cost Abstractions

```rust
// High-level operations compile to optimal machine code
fun process_matrix(m: Matrix<f64>) -> f64 {
    m.transpose()
     .multiply(&m)
     .eigenvalues()
     .sum()
    
    // Compiles to single SIMD loop with no allocations
}
```

## Quality Enforcement

### Compiler-Enforced Invariants

```rust
// Quality attributes as compiler directives
#[complexity(max = 10)]  // McCabe complexity limit
#[coverage(min = 95)]     // Test coverage requirement  
#[no_panic]              // No unwrap/panic in production
fun critical_function(data: DataFrame) -> Result<f64> {
    // Compiler rejects if constraints violated
    data.validate()?
        .analyze()
        .map(|r| r.confidence_interval())
}

// Property testing via attributes
#[property_test]
fun test_associative(a: f64, b: f64, c: f64) {
    // Generates 100K test cases automatically
    assert!((a + b) + c - (a + (b + c)).abs() < 1e-10);
}

// Mutation testing
#[mutation_score(min = 90)]
fun calculate_variance(data: &[f64]) -> f64 {
    let mean = data.iter().sum::<f64>() / data.len() as f64;
    data.iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>() / data.len() as f64
}
```

## Concurrency Model

### Async/Await with Structured Concurrency

```rust
// Rust async with automatic cancellation
async fun parallel_process(urls: Vec<String>) -> Vec<Result<Data>> {
    // Structured concurrency via scoped tasks
    urls.into_iter()
        .map(|url| tokio::spawn(fetch_and_process(url)))
        .collect::<Vec<_>>()
        .await_all()
}

// Actor supervision
#[supervisor(strategy = RestartOnFailure, max_restarts = 3)]
struct DataPipeline {
    workers: Vec<ActorHandle<DataProcessor>>,
}

impl DataPipeline {
    async fun handle_failure(&mut self, worker: ActorId, error: Error) {
        // Automatic recovery with exponential backoff
        self.restart_worker(worker).await;
    }
}
```

## Data Science Features

### Native DataFrame Support

```rust
// DataFrames as first-class types
let df = DataFrame::from_csv("data.csv")?;
let result = df
    .filter(col("age") > 18)
    .groupby("category")
    .agg([
        mean("value").alias("avg"),
        std("value").alias("stddev"),
        count().alias("n"),
    ]);

// SQL macro for compile-time query validation
let query_result = sql! {
    SELECT category, AVG(value) as avg
    FROM {df}
    WHERE age > 18
    GROUP BY category
};
```

### Scientific Computing

```rust
// Array operations with numpy-like ergonomics
let a = array![1.0, 2.0, 3.0];
let b = array![4.0, 5.0, 6.0];
let c = a.dot(&b);  // Dot product

// Broadcasting with compile-time dimension checking
let mut matrix = Matrix::<f64>::zeros(100, 100);
matrix.col_mut(0).assign(&linspace(0.0, 1.0, 100));

// Parallel by default
let large = Matrix::<f64>::random(10000, 10000);
let (eigenvals, eigenvecs) = large.eig();  // Uses all cores
```

### Visualization

```rust
// Type-safe plotting API
let plot = Plot::new()
    .scatter(df.col("age"), df.col("income"))
    .trendline(Method::Loess)
    .title("Age vs Income")
    .theme(Theme::Minimal);

plot.save("analysis.png", Dpi(300))?;
```

## Compilation Pipeline

### Single Transformation Path

```
Ruchy Source → AST → Type-Checked AST → Optimized MIR → Rust Code → Binary
                ↓                           ↓
           REPL/Debug                  Property Testing
```

### LLVM Optimization

```llvm
; Comprehensions compile to zero-cost loops
define @squares(%n: i64) -> %Vec {
    %result = alloca %Vec
    %i = alloca i64, 0
    br label %loop
    
loop:
    %val = load i64, %i
    %square = mul i64 %val, %val
    call @Vec.push(%result, %square)  ; Inlined
    %next = add i64 %val, 1
    %done = icmp eq i64 %next, %n
    br i1 %done, label %exit, label %loop
    
exit:
    ret %result
}
```

## Performance Guarantees

| Feature | Implementation | Overhead |
|---------|---------------|----------|
| Comprehensions | Iterator chains with fusion | 0% |
| Pattern matching | Jump tables | 0% |
| Async/await | State machines | 0% |
| Actors | M:N threading | <1% |
| Reference counting | Compile-time elision | 0% |
| Bounds checking | Eliminated when provable | <3% |
| Property testing | Compile-time only | 0% |

## Development Workflow

### REPL-Driven Development

```rust
// Interactive with full type inference
> let df = DataFrame::from_csv("data.csv")?
> df.describe()
       age    income
mean   35.2   65000
std    12.4   25000
min    18.0   20000
max    67.0   180000

> df.filter(col("age") > 30).count()
4523
```

### Testing Framework

```rust
// Tests via #[test] attribute
#[test]
fun test_statistical_accuracy() {
    let data = Random::normal(0.0, 1.0, 10000);
    assert!(data.mean().abs() < 0.05);
    assert!((data.std() - 1.0).abs() < 0.05);
}

// Doctests in doc comments
/// Calculate population variance
/// 
/// ```
/// let data = vec![1.0, 2.0, 3.0];
/// assert_eq!(variance(&data), 0.6666666666666666);
/// ```
fun variance(data: &[f64]) -> f64 {
    let m = mean(data);
    data.iter()
        .map(|x| (x - m).powi(2))
        .sum::<f64>() / data.len() as f64
}
```

## Standard Library

### Core Modules

```rust
// Collections (persistent by default)
use std::collections::{Vec, HashMap, BTreeSet, VecDeque};

// Algorithms (parallel by default)
use std::algo::{sort, partition, binary_search};

// Numerical
use numpy::{array, matrix, linalg, random, fft};

// DataFrames
use pandas::{DataFrame, Series, read_csv, read_parquet};

// Machine Learning
use ml::{LinearRegression, RandomForest, CrossValidator};

// Plotting
use plot::{figure, scatter, histogram, heatmap};

// Async I/O
use tokio::{fs, time, net, sync};
```

## Ecosystem Integration

### Direct Crate Usage

```rust
// Cargo dependencies in source
#[crate("serde", "1.0")]
use serde::{Serialize, Deserialize};

#[crate("tokio", "1.0")]
use tokio::{spawn, select, time::sleep};

// Derive macros work seamlessly
#[derive(Serialize, Deserialize, Clone)]
struct Config {
    host: String,
    port: u16,
    timeout: Duration,
}
```

### FFI Support

```rust
// C interop with safe wrappers
#[link(name = "math")]
extern "C" {
    fun fast_fourier_transform(
        data: *const f64,
        size: usize,
        output: *mut Complex<f64>,
    ) -> c_int;
}

// Safe wrapper generated
fun fft(data: &[f64]) -> Result<Vec<Complex<f64>>> {
    let mut output = vec![Complex::zero(); data.len()];
    unsafe {
        let result = fast_fourier_transform(
            data.as_ptr(),
            data.len(),
            output.as_mut_ptr(),
        );
        if result != 0 {
            return Err(FftError::Code(result));
        }
    }
    Ok(output)
}
```

## Migration Path

### From Python

```python
# Python code
import pandas as pd
df = pd.read_csv("data.csv")
result = df.groupby("category").mean()
```

```rust
// Ruchy equivalent
use pandas as pd;
let df = pd::read_csv("data.csv")?;
let result = df.groupby("category").mean();
```

### From Rust

```rust
// Standard Rust
fn process(data: Vec<i32>) -> Result<Vec<i32>, Error> {
    data.iter()
        .map(|x| x.checked_mul(2).ok_or(Error::Overflow))
        .collect()
}
```

```rust
// Ruchy - cleaner with same safety
fun process(data: Vec<i32>) -> Result<Vec<i32>> {
    [x.checked_mul(2)? for x in data]
}
```

## Quality Metrics

All code enforced at compile time:
- **Cyclomatic Complexity**: ≤ 10
- **Test Coverage**: ≥ 95%  
- **Mutation Score**: ≥ 90%
- **Property Tests**: ≥ 100K iterations
- **Memory Safety**: 100% proven
- **Data Race Freedom**: 100% proven
- **Zero SATD**: No TODO/FIXME

## Performance Characteristics

| Feature | Implementation | Overhead |
|---------|---------------|----------|
| Comprehensions | Iterator fusion | 0% |
| Pattern matching | Jump tables | 0% |
| Async/await | State machines | 0% |
| Actors | Work stealing | <1% |
| Bounds checks | Eliminated when provable | <3% |
| Property testing | Compile-time only | 0% |
| Trait dispatch | Monomorphization | 0% |

---

**Technical Rationale**: This design uses Rust's syntax as the foundation with strategic simplifications. The `fun` keyword reduces cognitive load while maintaining Rust semantics. Python-style imports provide familiar ergonomics without sacrificing static resolution. Comprehensions compile to iterator chains via mechanical transformation. Type inference operates within function boundaries (Hindley-Milner W) for O(n log n) complexity. LLVM backend achieves 97%+ performance parity with C through aggressive inlining and bounds elision. Property-based testing with SMT verification ensures correctness. Single syntax per concept eliminates decision fatigue while maintaining full Rust safety guarantees.
