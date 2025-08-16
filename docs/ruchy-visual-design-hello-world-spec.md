# Ruchy v3: Language Design Through Mechanical Transparency

## Core Philosophy: Observable Compilation

Every abstraction in Ruchy has **inspectable mechanics**. The compiler is not a black box but a glass box—every transformation is observable, measurable, and provable.

```bash
# Observe any compilation stage
$ ruchy show ast hello.ruchy      # Abstract Syntax Tree
$ ruchy show mir hello.ruchy      # Mid-level IR
$ ruchy show rust hello.ruchy     # Generated Rust
$ ruchy show asm hello.ruchy      # Assembly
$ ruchy show proof hello.ruchy    # Verification conditions
```

## 1. Progressive Disclosure Architecture

### Level 0: Pure Expression
```ruchy
println("Hello, World!")
```

### Level 1: Typed Function
```ruchy
fun greet(name: String) = println("Hello, {name}!")
```

### Level 2: Verified Program
```ruchy
#[ensures(result.len() > 0)]
fun greet(name: String) -> String {
    let message = "Hello, {name}!"
    println(message)
    message
}
```

### Level 3: Performance-Critical System
```ruchy
#[hot_path]
#[no_alloc]
fun process_packet(data: &[u8]) -> Result<Packet> {
    // Compiler enforces zero allocations
    let header = data[0..16].try_into()?
    let payload = &data[16..]
    Ok(Packet { header, payload })
}
```

## 2. Poly-Disassembly: Every Angle of Truth

### 2.1 Multi-Representation Analysis
```bash
$ ruchy disasm fibonacci.ruchy

Available representations:
  --ast        Abstract syntax tree (JSON)
  --mir        Mid-level IR (S-expressions)
  --rust       Generated Rust code
  --asm        Native assembly (Intel/AT&T)
  --llvm       LLVM IR
  --wasm       WebAssembly
  --metrics    Complexity metrics
  --dataflow   Flow analysis
  --proof      Verification conditions

$ ruchy disasm fibonacci.ruchy --metrics
┌─────────────────────────────────────┐
│ Function: fibonacci                 │
├─────────────────────────────────────┤
│ Cyclomatic Complexity: 3            │
│ Cognitive Complexity: 2             │
│ Halstead Volume: 34.87              │
│ Maintainability Index: 92.4         │
│ Big-O Complexity: O(n) [confidence: 0.95] │
│ Defect Probability: 0.03            │
│ Allocations: 0                      │
│ SATD Markers: None                  │
└─────────────────────────────────────┘
```

### 2.2 Dataflow Visualization
```bash
$ ruchy disasm process.ruchy --dataflow
╭─ process(data: Vec<Item>) ──────────╮
│                                      │
│  data ──┬──> filter ──> valid       │
│         │                 ↓          │
│         └──> errors ──> log         │
│                          ↓           │
│              valid ──> transform     │
│                          ↓           │
│                      result ←────────┤
╰──────────────────────────────────────╯
```

### 2.3 Proof Obligations
```bash
$ ruchy disasm sort.ruchy --proof
Verification Conditions:
  1. Precondition: input.len() > 0
  2. Loop invariant: 0 <= i < j <= input.len()
  3. Postcondition: ∀ i,j. i < j → result[i] <= result[j]
  
SMT solver: Z3 v4.12
Status: VERIFIED ✓ (12ms)
```

## 3. Type System: Bidirectional with Effects

### 3.1 Inference Flow Visualization
```ruchy
fun process(data) {          // data: τ₁
    data                     // τ₁
    |> validate()            // τ₁: Validatable ⇒ Result<τ₂>
    |> transform()           // τ₂: Transformable ⇒ Future<τ₃>
    |> save()                // τ₃: Saveable ⇒ Result<()>
}                            // Inferred: (τ₁: Validatable + Transformable) 
                            //          → Future<Result<()>>
```

### 3.2 Effect Tracking
```bash
$ ruchy show effects process.ruchy
╭─ Effect Analysis ────────────────────╮
│ Function: process                    │
│                                      │
│ Effects:                             │
│   • Async (from transform)          │
│   • Error (from validate, save)     │
│   • IO (from save)                  │
│                                      │
│ Await insertions:                   │
│   Line 3: After transform()         │
│   Line 4: Before save()             │
│                                      │
│ Error propagation:                  │
│   Lines 2,4: Implicit ? operator    │
╰──────────────────────────────────────╯
```

## 4. Mechanical Async: Deterministic Transformation

### 4.1 Await Insertion Rules
```ruchy
// INPUT: What you write
async fun fetch_and_process(url: String) {
    let data = fetch(url).json()
    let result = process(data)
    save(result)
}

// OUTPUT: What runs (observable via `ruchy show mir`)
async fun fetch_and_process(url: String) {
    let __future0 = fetch(url);        // Future<Response>
    let __response = __future0.await;   // Response
    let __future1 = __response.json();  // Future<Json>
    let data = __future1.await;         // Json
    let result = process(data);         // ProcessedData
    let __future2 = save(result);       // Future<()>
    __future2.await                     // ()
}
```

### 4.2 Effect Inference Trace
```bash
$ ruchy trace effects fetch_and_process.ruchy
┌─ Effect Inference Trace ─────────────┐
│ fetch(url)                          │
│   ↳ Returns: Future<Response>       │
│   ↳ Rule: Async function            │
│                                      │
│ .json()                              │
│   ↳ Returns: Future<Json>           │
│   ↳ Rule: Method on Future type     │
│                                      │
│ DECISION: Insert await at line 2    │
│   ↳ Reason: Binding site rule       │
│                                      │
│ process(data)                        │
│   ↳ Returns: ProcessedData          │
│   ↳ Rule: Pure function             │
│                                      │
│ save(result)                         │
│   ↳ Returns: Future<()>             │
│   ↳ Rule: IO function               │
│                                      │
│ DECISION: Insert await at line 4    │
│   ↳ Reason: Terminal position       │
└──────────────────────────────────────┘
```

## 5. Smart Defaults with Escape Hatches

### 5.1 Allocation Control
```ruchy
// DEFAULT: Allocating operations
fun process(data: Vec<Item>) -> Vec<Result> {
    data
    |> filter(valid)      // Allocates iterator
    |> map(transform)     // Allocates iterator
    |> collect()          // Allocates Vec
}

// OPTIMIZED: Zero-allocation version
#[no_alloc]
fun process(data: &[Item], out: &mut Vec<Result>) {
    data.iter()
        .filter(valid)
        .map(transform)
        .for_each(|r| out.push(r))  // Reuse existing allocation
}

// COMPILER FEEDBACK
$ ruchy analyze process.ruchy
Warning: 3 allocations in hot path
  Line 3: filter() allocates iterator
  Line 4: map() allocates iterator  
  Line 5: collect() allocates Vec

Suggestion: Use #[no_alloc] with pre-allocated buffer
Performance impact: ~2.3x speedup expected
```

### 5.2 String Strategy Selection
```ruchy
// Compiler chooses optimal representation
let s1 = "hello"              // &'static str (const context)
let s2 = "hello".to_string()  // String (explicit ownership)
let s3 = read_file("x.txt")   // String (from I/O)
let s4: &str = buffer.slice() // &str (zero-copy view)

// Observable via disassembly
$ ruchy show mir strings.ruchy --verbose
s1: ConstStr { data: 0x4000, len: 5 }      // Zero runtime cost
s2: HeapString { ptr: alloc(6), len: 5 }   // Heap allocated
s3: HeapString { ptr: io_buf, len: _ }     // From I/O buffer
s4: StrSlice { ptr: buffer+10, len: 20 }   // Borrowed view
```

## 6. Performance Observatory

### 6.1 Static + Dynamic Analysis
```bash
$ ruchy profile app.ruchy --release
Building with instrumentation...
Running with workload: synthetic_1k

┌─ Performance Report ──────────────────┐
│ Hot Paths (>5% runtime)              │
├───────────────────────────────────────┤
│ process_batch      43.2%  2.3M calls │
│ ├─ validate        18.7%  2.3M       │
│ ├─ transform       15.2%  1.9M       │
│ └─ serialize        9.3%  1.9M       │
│                                       │
│ handle_request     31.5%  45K calls  │
│ ├─ parse_header     8.2%  45K        │
│ ├─ route           11.3%  45K        │
│ └─ send_response   12.0%  45K        │
└───────────────────────────────────────┘

┌─ Allocation Sites ────────────────────┐
│ Confirmed Hot (>1K/sec)              │
├───────────────────────────────────────┤
│ serialize:27  Vec::with_capacity     │
│   → 2.3M allocations (46MB/sec)      │
│   → Consider: Object pool            │
│                                       │
│ transform:15  String::from_utf8      │
│   → 1.9M allocations (38MB/sec)      │
│   → Consider: Cow<str> or &str       │
└───────────────────────────────────────┘

Generate optimization patch? [Y/n]
```

### 6.2 Automatic Optimization Suggestions
```ruchy
// BEFORE: Original code
fun serialize(items: Vec<Item>) -> String {
    items.iter()
        .map(|item| format!("{:?}", item))  // Allocation per item
        .collect::<Vec<_>>()                // Intermediate Vec
        .join(", ")                         // Final String
}

// AFTER: Optimizer suggestion applied
fun serialize(items: Vec<Item>) -> String {
    let mut buffer = String::with_capacity(items.len() * 32);
    for (i, item) in items.iter().enumerate() {
        if i > 0 { buffer.push_str(", "); }
        write!(&mut buffer, "{:?}", item).unwrap();
    }
    buffer
}

// Performance: 3.7x faster, 85% fewer allocations
```

## 7. Prelude Architecture: Compositional Defaults

### 7.1 Hierarchical Prelude System
```yaml
# ruchy.yaml - Project configuration
project:
  name: web-service
  preludes:
    - core      # Always: Vec, HashMap, Result, Option
    - async     # When async detected: spawn, sleep, channel
    - web       # When web crate found: Request, Response
    - custom: ./src/prelude.ruchy

# Conditional activation
prelude_rules:
  - if: "uses async"
    include: ["tokio::spawn", "futures::join"]
  - if: "uses #[test]"
    include: ["proptest::*", "quickcheck::*"]
  - if: "target = wasm32"
    include: ["wasm_bindgen::*"]
```

### 7.2 Prelude Inspection
```bash
$ ruchy prelude inspect
┌─ Active Prelude Hierarchy ────────────┐
│ core (always)                         │
│ ├─ std::vec::Vec                     │
│ ├─ std::result::Result               │
│ └─ std::option::Option               │
│                                       │
│ async (detected: async keyword)      │
│ ├─ tokio::spawn                      │
│ └─ tokio::time::sleep                │
│                                       │
│ project (./src/prelude.ruchy)        │
│ ├─ crate::Error (AppError)           │
│ └─ crate::config::Config             │
└───────────────────────────────────────┘

$ ruchy prelude trace HashMap
HashMap comes from:
  core prelude (priority: 0)
  → std::collections::HashMap
  → Source: /rust/lib/std/collections/hash/map.rs:205
  → Used 47 times in current project
```

## 8. Compilation Pipeline: Glass Box Architecture

### 8.1 Observable Stages
```rust
Source → Tokens → AST → HIR → MIR → Rust → Assembly
           ↓       ↓      ↓     ↓     ↓       ↓
        Observe Observe Observe Observe Observe Observe
```

### 8.2 Stage Inspection
```bash
$ ruchy compile hello.ruchy --stop-after=mir
Compilation stopped after MIR generation
Output: hello.mir

$ cat hello.mir
fn main() -> () {
    bb0: {
        _1 = const "Hello, World!";     // &'static str
        _2 = println(move _1) -> bb1;   // Call println
    }
    bb1: {
        return;                          // Return unit
    }
}
```

### 8.3 Optimization Visibility
```bash
$ ruchy compile compute.ruchy --show-optimizations
┌─ Optimization Report ─────────────────┐
│ Function: compute                     │
├────────────────────────────────────────┤
│ Applied:                              │
│ ✓ Inline (3 call sites)              │
│ ✓ Loop unrolling (factor: 4)         │
│ ✓ Dead code elimination (7 lines)    │
│ ✓ Constant folding (12 expressions)  │
│                                       │
│ Not Applied:                         │
│ ✗ Vectorization (reason: irregular)  │
│ ✗ Tail call (reason: not recursive)  │
└────────────────────────────────────────┘
```

## 9. Error Diagnostics: Teaching Through Failure

### 9.1 Mechanical Explanations
```bash
error[E0308]: effect mismatch
  --> script.ruchy:5:10
   |
 5 |   process(fetch(url))
   |          ^^^^^^^^^^^ Future<Response> where Response expected
   |
   = Mechanical Rule Violated:
     Await insertion occurs ONLY at:
       • let bindings: let x = async_expr
       • pipeline stages: async |> next
       • block returns: { async_expr }
     
     NOT at function arguments (current location)
   
   = Why This Rule Exists:
     Function arguments evaluate eagerly in Rust.
     Inserting await here would change evaluation order.
   
   = Fix (apply with --fix):
     let response = fetch(url)  // Await inserted here
     process(response)
   
   = Learn More:
     ruchy explain await-rules
```

### 9.2 Performance Warnings
```bash
warning: allocation in hot loop
  --> compute.ruchy:8:5
   |
 8 |   let buffer = Vec::new()
   |   ^^^^^^^^^^^^^^^^^^^^^^^^ Allocates on each iteration
   |
   = Performance Impact:
     • 10,000 iterations = 10,000 allocations
     • Estimated overhead: 47ms (23% of runtime)
     • Memory fragmentation risk: HIGH
   
   = Suggested Fix:
 8 |   let mut buffer = Vec::with_capacity(expected_size)
   |   Move allocation outside loop, reuse via clear()
   
   = Verification:
     Run with --measure-allocs to confirm fix
```

## 10. Property-Based Quality Enforcement

### 10.1 Integrated Property Testing
```ruchy
#[property]
fun prop_sort_preserves_length(xs: Vec<i32>) {
    let original_len = xs.len()
    let sorted = sort(xs)
    assert_eq!(sorted.len(), original_len)
}

#[property(samples = 10000)]
fun prop_parse_print_roundtrip(ast: Ast) {
    let printed = pretty_print(ast)
    let parsed = parse(printed).unwrap()
    assert_eq!(ast, parsed)
}
```

### 10.2 Mutation Testing
```bash
$ ruchy mutate src/
Generating mutants...
Running test suite against mutants...

┌─ Mutation Testing Report ─────────────┐
│ Total Mutants: 127                    │
│ Killed: 119 (93.7%)                   │
│ Survived: 8 (6.3%)                    │
├────────────────────────────────────────┤
│ Surviving Mutants:                    │
│                                        │
│ sort.ruchy:15                         │
│   Original: if a <= b                 │
│   Mutant:   if a < b                  │
│   Impact: Unstable sort               │
│                                        │
│ [7 more...]                           │
└────────────────────────────────────────┘

Generate test to kill mutant? [Y/n]
```

## 11. Design Invariants (Enforced by Compiler)

### Invariant 1: Mechanical Transparency
```rust
trait Observable {
    fn show_ast(&self) -> Json;
    fn show_mir(&self) -> String;
    fn show_rust(&self) -> String;
    fn show_metrics(&self) -> Metrics;
}

// Every compilation artifact implements Observable
impl Observable for TypedAst { ... }
impl Observable for MIR { ... }
impl Observable for RustAst { ... }
```

### Invariant 2: Progressive Performance
```rust
#[test]
fn test_performance_progression() {
    let script = load("bench.ruchy");
    
    let interpreted_time = measure(|| interpret(script));
    let jit_time = measure(|| jit_execute(script));
    let native_time = measure(|| native_execute(script));
    
    assert!(jit_time < interpreted_time * 0.5);
    assert!(native_time < jit_time * 0.5);
}
```

### Invariant 3: Zero-Cost Abstractions
```rust
#[property]
fun prop_pipeline_zero_cost(data: Vec<i32>) {
    let pipeline_version = data
        |> filter(_ > 0)
        |> map(_ * 2)
        |> sum();
    
    let loop_version = {
        let mut sum = 0;
        for x in data {
            if x > 0 {
                sum += x * 2;
            }
        }
        sum
    };
    
    assert_eq!(pipeline_version, loop_version);
    assert_asm_equivalent!(pipeline_version, loop_version);
}
```

### Invariant 4: Allocation Accountability
```rust
#[no_alloc]
fun guaranteed_zero_alloc(data: &[u8]) -> Result<u32> {
    // Compiler enforces: no heap allocations allowed
    // Violating this is a compilation error
    let sum = data.iter().fold(0u32, |acc, &b| acc + b as u32);
    Ok(sum)
}
```

## 12. Implementation Strategy: Phased Excellence

### Phase 1: Core Compiler (Q1 2025)
- Parser with error recovery
- Bidirectional type inference
- Rust transpilation
- Basic REPL

### Phase 2: Observable Compilation (Q2 2025)
- AST/MIR/Rust inspection
- Static metrics computation
- Dataflow visualization
- Effect tracking

### Phase 3: Performance Observatory (Q3 2025)
- Profile-guided optimization
- Allocation tracking
- Automatic optimization suggestions
- Hot path analysis

### Phase 4: Quality Construction (Q4 2025)
- Property testing integration
- Mutation testing
- SMT verification
- Defect prediction

The modular architecture ensures each phase delivers immediate value while building toward the complete vision.

## 13. Rust vs Ruchy: Productivity Through Transparency

### Example 1: Error Handling
```rust
// RUST: Verbose ceremony
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::error::Error;

fn count_words(path: &str) -> Result<usize, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut count = 0;
    
    for line in reader.lines() {
        let line = line?;
        count += line.split_whitespace().count();
    }
    
    Ok(count)
}

fn main() -> Result<(), Box<dyn Error>> {
    let count = count_words("data.txt")?;
    println!("Words: {}", count);
    Ok(())
}
```

```ruchy
// RUCHY: Direct expression
fun count_words(path) = 
    read_file(path)?
    |> lines()
    |> flat_map(words)
    |> count()

fun main() {
    let count = count_words("data.txt")?
    println("Words: {count}")
}

// BONUS: Inspect the transformation
$ ruchy show rust count_words
// Generated Rust matches hand-written performance
```

### Example 2: Concurrent Processing
```rust
// RUST: Complex setup
use tokio;
use futures::future::join_all;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let urls = vec![
        "https://api1.example.com",
        "https://api2.example.com",
        "https://api3.example.com",
    ];
    
    let futures = urls.into_iter().map(|url| {
        tokio::spawn(async move {
            let response = reqwest::get(url).await?;
            response.text().await
        })
    });
    
    let results = join_all(futures).await;
    
    for result in results {
        match result {
            Ok(Ok(text)) => println!("Got: {}", text.len()),
            Ok(Err(e)) => eprintln!("Request failed: {}", e),
            Err(e) => eprintln!("Task failed: {}", e),
        }
    }
}
```

```ruchy
// RUCHY: Intent-focused
fun main() {
    let urls = [
        "https://api1.example.com",
        "https://api2.example.com", 
        "https://api3.example.com",
    ]
    
    urls
    |> map(fetch)           // Automatic async
    |> join_all()           // Automatic await
    |> each(|result| {
        match result {
            Ok(text) => println("Got: {text.len()}"),
            Err(e) => eprintln("Failed: {e}")
        }
    })
}

// BONUS: See the concurrency
$ ruchy show effects main
3 async operations spawned concurrently
Joined at line 9
Error handling: automatic Result<T> propagation
```

### Example 3: Data Processing Pipeline
```rust
// RUST: Manual optimization
use std::collections::HashMap;
use rayon::prelude::*;

struct Record {
    category: String,
    value: f64,
}

fn process_records(records: Vec<Record>) -> HashMap<String, f64> {
    let mut results = HashMap::new();
    
    records
        .into_par_iter()
        .filter(|r| r.value > 0.0)
        .map(|r| (r.category.clone(), r.value * 1.1))
        .collect::<Vec<_>>()
        .into_iter()
        .for_each(|(cat, val)| {
            *results.entry(cat).or_insert(0.0) += val;
        });
    
    results
}
```

```ruchy
// RUCHY: Clear intent
fun process_records(records: Vec<Record>) {
    records
    |> filter(_.value > 0)
    |> map(r => (r.category, r.value * 1.1))
    |> group_by(_.0)
    |> map_values(|group| group.map(_.1).sum())
}

// BONUS: Performance analysis
$ ruchy analyze process_records
Parallelization: Automatic (rayon)
Allocations: 1 HashMap, 0 intermediate
Complexity: O(n) time, O(k) space (k = categories)
Suggestion: Pre-size HashMap if category count known
```

### Example 4: Type-Safe Configuration
```rust
// RUST: Boilerplate-heavy
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    server: ServerConfig,
    database: DatabaseConfig,
}

#[derive(Debug, Deserialize, Serialize)]
struct ServerConfig {
    host: String,
    port: u16,
}

#[derive(Debug, Deserialize, Serialize)]
struct DatabaseConfig {
    url: String,
    pool_size: usize,
}

impl Config {
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config = toml::from_str(&contents)?;
        Ok(config)
    }
}
```

```ruchy
// RUCHY: Structure emerges from use
struct Config {
    server: { host: String, port: u16 },
    database: { url: String, pool_size: usize }
}

fun load_config(path) = 
    read_file(path)?
    |> parse_toml()

// BONUS: Automatic schema generation
$ ruchy show schema Config
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "server": {
      "type": "object",
      "properties": {
        "host": { "type": "string" },
        "port": { "type": "integer", "minimum": 0, "maximum": 65535 }
      }
    },
    "database": { ... }
  }
}
```

### Example 5: Performance-Critical Loop
```rust
// RUST: Manual optimization required
use std::simd::*;

fn sum_squares(data: &[f32]) -> f32 {
    let chunks = data.chunks_exact(4);
    let remainder = chunks.remainder();
    
    let mut vec_sum = f32x4::splat(0.0);
    
    for chunk in chunks {
        let vec = f32x4::from_slice(chunk);
        vec_sum += vec * vec;
    }
    
    let mut sum = vec_sum.horizontal_sum();
    
    for &x in remainder {
        sum += x * x;
    }
    
    sum
}
```

```ruchy
// RUCHY: Express intent, get optimization
#[vectorize]
fun sum_squares(data: &[f32]) -> f32 {
    data.iter().map(|x| x * x).sum()
}

// BONUS: Verify optimization
$ ruchy show asm sum_squares --intel
sum_squares:
    vmovups ymm0, [rdi]      ; Load 8 floats
    vmulps ymm0, ymm0, ymm0  ; Square them
    vaddps ymm1, ymm1, ymm0  ; Accumulate
    ; ... SIMD loop unrolled 4x
    vhaddps ymm0, ymm1, ymm1 ; Horizontal sum
    ret

$ ruchy benchmark sum_squares
Input size: 1M floats
Ruchy version: 1.24ms (auto-vectorized)
Rust version: 1.31ms (manual SIMD)
Speedup: 1.06x with 75% less code
```

### Productivity Metrics: Ruchy vs Rust

| Metric | Rust | Ruchy | Improvement |
|--------|------|-------|-------------|
| Lines of Code | 100 | 31 | 69% reduction |
| Type Annotations | 47 | 3 | 94% reduction |
| Error Handling Boilerplate | 23 lines | 2 operators | 91% reduction |
| Time to First Run | 3-5 min | 10 sec | 20x faster |
| Optimization Visibility | External tools | Built-in | ∞ |
| Performance | Baseline | 0-5% faster | Auto-optimization |

## 14. Summary: Lean Language Design

Ruchy achieves simplicity through **mechanical transparency**:

1. **Observable Compilation**: Every transformation inspectable
2. **Deterministic Optimization**: Performance predictable
3. **Progressive Disclosure**: Complexity reveals gradually
4. **Quality by Construction**: Correctness enforced at compile time

The result: Beginners write fast code by default. Experts have total control when needed.

```bash
# The Ruchy Promise: See Everything, Control Everything
$ ruchy --version
Ruchy v1.0.0 - Mechanical Transparency for Systems Scripting

$ ruchy compile hello.ruchy --show-all
[Token Stream]
[AST: 12 nodes]
[Type Inference: 3 constraints solved]
[MIR: 2 basic blocks]
[Rust: 5 lines generated]
[Assembly: 47 instructions]
[Binary: 1.2MB (stripped: 387KB)]

Compilation: 127ms
Zero technical debt detected ✓
```

This is lean engineering: Eliminate waste (boilerplate), preserve value (control), make work visible (observability).