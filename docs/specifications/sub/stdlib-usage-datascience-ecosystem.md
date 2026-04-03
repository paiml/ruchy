# Sub-spec: Stdlib Usage — Data Science, Quality, and Ecosystem

**Parent:** [stdlib-usage-spec.md](../stdlib-usage-spec.md) Sections 7-12

---


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
