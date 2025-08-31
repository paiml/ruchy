# Ruchy REPL Completion & Help System - Complete Specification

## Executive Summary

This specification defines a comprehensive tab completion and help system for the Ruchy REPL that combines Deno's intelligent, high-performance completion with Python's discoverable help system. The implementation addresses critical challenges in building context-aware completions for partial syntax trees while maintaining sub-50ms response times.

## Problem Statement

### Core Challenges

**1. The Chicken-and-Egg Problem**
Traditional compilers fail when parsing incomplete or syntactically invalid expressions, yet REPL completion must provide intelligent suggestions precisely in these scenarios. When a user types `obj.meth<TAB>`, the expression is incomplete and unparseable by standard techniques.

**2. Performance vs. Accuracy Trade-off**  
Achieving Deno-level <50ms completion latency while providing Python-level contextual accuracy requires sophisticated caching and error recovery mechanisms. Cold cache scenarios can easily cause multi-second delays that destroy the interactive experience.

**3. Documentation Fragmentation**
Rust's ecosystem lacks a unified documentation query system comparable to Python's `help()` and `dir()` functions. Developers must constantly context-switch between REPL, docs.rs, and source code to understand APIs.

**4. Completion Ranking Complexity**
Creating intuitive completion ordering is a "black art" - users expect exact matches first, but also want fuzzy matching for typos, acronym matching for camelCase, and frequency-based learning from usage patterns.

## Architectural Overview

### Component Architecture

```rust
// Core completion orchestrator
pub struct RuchyCompleter {
    symbol_table: Arc<SymbolTable>,
    type_inference: Arc<TypeInference>,
    help_system: HelpSystem,
    completion_cache: CompletionCache,
    current_scope: ScopeId,
}
```

The system is built around five core components that work together to provide intelligent completions:

### 1. Context Analysis Engine

**Problem**: Determining user intent from cursor position in potentially malformed code.

**Solution**: Multi-stage context detection that gracefully degrades from full parsing to pattern matching.

```rust
pub enum CompletionContext {
    /// obj.method<TAB> - Type-specific method completions
    MethodAccess {
        receiver_type: Type,
        receiver_expr: String,
        partial_method: String,
    },
    /// std::collections::<TAB> - Module hierarchy navigation  
    ModulePath {
        segments: Vec<String>,
        partial_segment: String,
    },
    /// Variable/function completion in current scope
    FreeExpression {
        scope: ScopeId,
        partial_ident: String,
    },
    /// println(<TAB> - Parameter hints and signature help
    FunctionCall {
        function_name: String,
        current_param: usize,
    },
    /// help(object) - Documentation queries
    HelpQuery {
        query: String,
    },
}
```

### 2. Error-Tolerant Type Inference

**Problem**: Standard type checkers fail on incomplete expressions, yet we need type information for accurate method completion.

**Solution**: Four-stage fallback pipeline that maintains accuracy while providing graceful degradation.

```rust
/// Error-tolerant type inference for partial/broken expressions
/// Critical: The "chicken-and-egg" problem solver
fn infer_receiver_type_tolerant(&self, expr: &str) -> Result<Type, String> {
    // Stage 1: Try full parse + inference
    if let Ok(ty) = self.type_inference.infer_expression_type(expr, self.current_scope) {
        return Ok(ty);
    }

    // Stage 2: Pattern-based heuristics for common cases
    if let Some(ty) = self.pattern_based_type_inference(expr) {
        return Ok(ty);
    }

    // Stage 3: Incremental parsing with error recovery
    if let Some(ty) = self.incremental_type_recovery(expr) {
        return Ok(ty);
    }

    // Stage 4: Symbol table lookup for simple identifiers
    let simple_ident = expr.split_whitespace().last().unwrap_or(expr);
    if let Some(symbol) = self.symbol_table.lookup(simple_ident, self.current_scope) {
        return Ok(symbol.symbol_type.clone());
    }

    Err(format!("Cannot infer type for: {}", expr))
}
```

**Stage 2 Example - Pattern Recognition:**
```rust
fn pattern_based_type_inference(&self, expr: &str) -> Option<Type> {
    // Literal patterns - even with broken syntax
    if expr.starts_with('[') && expr.contains(']') {
        return Some(Type::List(Box::new(Type::Unknown)));
    }
    if expr.starts_with('"') || expr.starts_with('\'') {
        return Some(Type::String);
    }
    
    // Constructor patterns - "DataFrame::new(" -> DataFrame
    if expr.contains("DataFrame") || expr.contains("df") {
        return Some(Type::DataFrame);
    }
    
    None
}
```

### 3. Performance-Optimized Caching

**Problem**: Cold cache scenarios cause unacceptable latency spikes that destroy the interactive experience.

**Solution**: Multi-tier caching with background indexing and performance monitoring.

```rust
pub struct CompletionCache {
    type_methods: HashMap<TypeId, Vec<MethodCompletion>>,
    scope_symbols: HashMap<ScopeId, Vec<SymbolCompletion>>,
    module_contents: HashMap<String, Vec<ModuleCompletion>>,
    
    // Background indexing to prevent cold cache spikes
    background_indexer: BackgroundIndexer,
    warm_cache_queue: VecDeque<CacheWarmupTask>,
    
    // Performance monitoring for operational visibility
    hit_count: u64,
    miss_count: u64,
    avg_lookup_time: std::time::Duration,
}
```

**Background Indexing Strategy:**
- **Proactive Warmup**: Pre-load common types (String, List, DataFrame) at startup
- **Standard Library Caching**: Keep `std::*` modules permanently cached
- **Usage Learning**: Priority-queue frequently accessed symbols
- **Performance Monitoring**: Alert on cache hit rates <70% or lookup times >50ms

### 4. Python-Style Help System

**Problem**: Rust ecosystem lacks unified documentation discovery comparable to Python's `help()` and `dir()`.

**Solution**: Integrated documentation system with multi-source aggregation.

```rust
pub struct HelpSystem {
    builtin_docs: HashMap<String, Documentation>,
    method_docs: HashMap<(TypeId, String), Documentation>,
    module_docs: HashMap<String, Documentation>,
    examples: HashMap<String, Vec<String>>,
}

pub struct Documentation {
    signature: String,
    description: String,
    parameters: Vec<Parameter>,
    return_type: Option<String>,
    examples: Vec<String>,
    see_also: Vec<String>,
}
```

**Multi-Source Documentation Strategy:**
1. **Builtin Functions**: Hand-curated docs for core language features
2. **Rustdoc Integration**: Extract from `cargo doc --message-format=json` output
3. **Proc-Macro Attributes**: Parse `#[doc = "..."]` attributes from AST
4. **docs.rs Integration**: Fetch external crate documentation via API
5. **Usage Examples**: Auto-generate from test files and doc tests

### 5. Intelligent Completion Ranking

**Problem**: Creating intuitive completion ordering requires balancing multiple competing factors.

**Solution**: Multi-criteria scoring system with user learning.

```rust
fn calculate_completion_score(&self, candidate: &str, query: &str) -> f64 {
    let mut score = 0.0;
    
    // Exact prefix match (highest priority)
    if candidate.starts_with(query) {
        score += 100.0;
        if candidate.len() == query.len() { score += 50.0; } // Exact match bonus
        score -= (candidate.len() - query.len()) as f64 * 0.1; // Length penalty
    } 
    // Fuzzy matching with Levenshtein distance
    else {
        let edit_distance = self.levenshtein_distance(candidate, query);
        let max_len = candidate.len().max(query.len());
        if max_len > 0 && edit_distance <= max_len / 2 {
            score += 50.0 * (1.0 - edit_distance as f64 / max_len as f64);
        }
    }
    
    // Word boundary matching - "HashMap" matches "HM"
    if self.matches_word_boundary(candidate, query) { score += 20.0; }
    
    // Usage frequency learning
    score += self.get_completion_frequency(candidate) * 5.0;
    
    // Contextual relevance
    if self.is_contextually_relevant(candidate) { score += 15.0; }
    
    score
}
```

## Type-Aware Method Completion

### List Methods
When a user types `[1,2,3].<TAB>`, the system provides contextually appropriate completions:

```rust
Type::List(_) => vec![
    MethodCompletion {
        name: "map".to_string(),
        signature: "map(f: T -> U) -> List<U>".to_string(),
        description: "Transform each element with function f".to_string(),
        return_type: "List<U>".to_string(),
        priority: 10,
    },
    MethodCompletion {
        name: "filter".to_string(),
        signature: "filter(f: T -> Bool) -> List<T>".to_string(),
        description: "Keep elements where f returns true".to_string(),
        return_type: "List<T>".to_string(),
        priority: 10,
    },
    // ... additional methods
],
```

**User Experience:**
```bash
>>> [1, 2, 3].
# TAB completion shows:
#   map(f: T -> U) -> List<U>     - Transform each element
#   filter(f: T -> Bool) -> List<T> - Keep matching elements  
#   sum() -> T                    - Sum all elements
#   len() -> Int                  - Number of elements
```

### DataFrame Methods
For data science workflows, DataFrame completions are essential:

```rust
Type::DataFrame => vec![
    MethodCompletion {
        name: "head".to_string(),
        signature: "head(n: Int = 5) -> DataFrame".to_string(),
        description: "Show first n rows".to_string(),
        return_type: "DataFrame".to_string(),
        priority: 10,
    },
    MethodCompletion {
        name: "select".to_string(),
        signature: "select(columns: List<String>) -> DataFrame".to_string(),
        description: "Select specific columns".to_string(),
        return_type: "DataFrame".to_string(),
        priority: 9,
    },
    // ... data analysis methods
],
```

## Auto-Import with Transparency

**Problem**: Automatic imports can be "magical" and confusing to users who don't understand where symbols came from.

**Solution**: Transparent auto-import with clear user notification.

```rust
pub fn execute_auto_import(&mut self, symbol: &str) -> Option<String> {
    let import_map = [
        ("HashMap", "use std::collections::HashMap;"),
        ("HashSet", "use std::collections::HashSet;"),
        ("read_to_string", "use std::fs::read_to_string;"),
    ];

    for (sym, import_stmt) in &import_map {
        if symbol == *sym {
            self.add_import_to_scope(import_stmt);
            
            // Transparent notification prevents confusion
            println!("ℹ️  Auto-imported: {}", import_stmt);
            
            return Some(import_stmt.to_string());
        }
    }
    None
}
```

**User Experience:**
```bash
>>> HashMap
ℹ️  Auto-imported: use std::collections::HashMap;
>>> let map = HashMap::new()
# Works immediately, user understands where HashMap came from
```

## Python-Style Help System

### Interactive Documentation

```bash
>>> help()
Ruchy Interactive Help System

Type help(object) for help on a specific object, function, or type.
Type dir(object) to list available methods and attributes.
Type type(object) to show the type of an object.

Quick reference:
  help(println)     - Help on println function  
  help(List)        - Help on List type
  dir([1,2,3])      - Show methods for lists
  type("hello")     - Shows String
  ?println          - Quick help (alias for help(println))

>>> help(println)
println(value: T) -> ()
=======================

Print a value to stdout followed by a newline.

Parameters:
  value: T - The value to print

Returns: ()

Examples:
  println("Hello, world!")
  println(42)
  println([1, 2, 3])

See also: print
```

### Discovery Functions

```bash
>>> dir([1, 2, 3])
['sum', 'map', 'filter', 'len', 'head', 'tail', 'reverse']

>>> type("hello")
String

>>> dir("hello")  
['len', 'upper', 'lower', 'trim', 'split', 'starts_with', 'ends_with']
```

## Performance Optimization Strategies

### Background Indexing
```rust
pub struct BackgroundIndexer {
    indexing_thread: Option<std::thread::JoinHandle<()>>,
    task_queue: Arc<Mutex<VecDeque<IndexingTask>>>,
    is_indexing: Arc<AtomicBool>,
}

pub enum IndexingTask {
    IndexModule { path: String, priority: u8 },
    IndexType { type_id: TypeId, priority: u8 },
    IndexLibrary { name: String, priority: u8 },
    RebuildCache { reason: String },
}
```

**Benefits:**
- **Eliminates UI Freezing**: Heavy parsing occurs in background thread
- **Priority Queuing**: Common types indexed first
- **Incremental Updates**: Only reindex changed modules
- **Resource Throttling**: Configurable indexing intervals

### Cache Performance Monitoring
```rust
fn invalidate(&mut self) {
    let hit_rate = self.hit_count as f64 / (self.hit_count + self.miss_count) as f64;
    
    if hit_rate < 0.7 {
        eprintln!("⚠️  Low completion cache hit rate: {:.1}%", hit_rate * 100.0);
        eprintln!("   Consider increasing cache size or warmup coverage");
    }
    
    // Smart selective invalidation - keep hot entries
    self.selective_invalidate();
}
```

## Integration with Rustyline

The completion system integrates seamlessly with the existing REPL infrastructure:

```rust
impl Completer for RuchyCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let context = self.analyze_context(line, pos);
        let completions = self.complete_context(context);
        
        // Find start of current word for replacement
        let start = line[..pos].rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
            .map(|i| i + 1)
            .unwrap_or(0);
        
        Ok((start, completions))
    }
}
```

## Testing Strategy

### Unit Tests for Context Analysis
```rust
#[test]
fn test_error_tolerant_context_analysis() {
    let completer = RuchyCompleter::new();
    
    // Test broken syntax - should still provide completions
    let context = completer.analyze_context("[1,2,3].", 8);
    assert!(matches!(context, CompletionContext::MethodAccess { .. }));
    
    // Test deeply nested broken expressions
    let context = completer.analyze_context("obj.method(broken.syntax.", 25);
    // Should still infer that we want method completions
}
```

### Performance Benchmarks
```rust
#[test]
fn test_completion_latency() {
    let completer = RuchyCompleter::new();
    let start = Instant::now();
    
    for _ in 0..1000 {
        completer.analyze_context("std::fs::", 9);
    }
    
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 50); // <50ms for 1000 completions
}
```

### Property-Based Testing
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn completion_never_panics(line in ".*", pos in 0..100usize) {
        let completer = RuchyCompleter::new();
        let safe_pos = pos.min(line.len());
        let _ = completer.analyze_context(&line, safe_pos); // Should never panic
    }
}
```

## Implementation Phases

### Phase 1: Core Infrastructure (Weeks 1-2)
- Error-tolerant context analysis
- Basic method completion for List/String/DataFrame
- Simple caching without background indexing
- Rustyline integration

### Phase 2: Help System (Week 3)
- Python-style `help()`, `dir()`, `type()` functions
- Documentation extraction from rustdoc
- Interactive help command integration

### Phase 3: Performance Optimization (Week 4)
- Background indexing implementation
- Cache performance monitoring
- Fuzzy matching and advanced ranking
- Auto-import with transparency

### Phase 4: Polish & Testing (Week 5)
- Comprehensive test suite
- Performance benchmarking
- Error message improvements
- Documentation completion

## Success Metrics

**Performance Targets:**
- <50ms completion latency (99th percentile)
- <70% cache hit rate triggers optimization warnings
- <10MB memory overhead for completion system

**User Experience Targets:**
- Zero-learning-curve for Deno/Python developers
- All language features discoverable via `help()` system
- Transparent auto-import prevents confusion
- Fuzzy matching handles common typos

**Quality Targets:**
- >95% uptime (no crashes from malformed input)
- 100% of public APIs documented in help system
- Property-based tests prevent regression

## Competitive Analysis

### Advantages over Existing Systems

**vs. Deno REPL:**
- **Better Type Awareness**: Rust's type system enables more precise completions
- **Compiled Performance**: No JavaScript runtime overhead
- **Richer Help System**: Integrated documentation discovery

**vs. Python REPL:**
- **Sub-50ms Latency**: Significantly faster than Python's completion
- **Static Type Benefits**: Compile-time error prevention
- **Memory Safety**: No runtime crashes from completion system

**vs. Other Rust REPLs:**
- **Production-Grade Architecture**: Background indexing and performance monitoring
- **User-Centric Design**: Optimized for developer productivity, not just correctness
- **Comprehensive Documentation**: Python-level API discoverability

## Risk Mitigation

### Technical Risks
**Risk**: Error-tolerant parsing complexity leads to incorrect completions  
**Mitigation**: Extensive property-based testing and graduated fallback stages

**Risk**: Background indexing consumes excessive resources  
**Mitigation**: Resource throttling and configurable indexing intervals

**Risk**: Cache invalidation bugs cause stale completions  
**Mitigation**: Generation-based cache invalidation with performance monitoring

### User Experience Risks
**Risk**: Auto-import magic confuses users about symbol origins  
**Mitigation**: Transparent notifications with clear import statements

**Risk**: Fuzzy matching prioritizes wrong completions  
**Mitigation**: Multi-criteria scoring with user feedback integration

## Conclusion

This specification represents a comprehensive solution to the fundamental challenges of building intelligent, high-performance completion for interactive programming environments. By combining error-tolerant parsing, sophisticated caching, and comprehensive documentation integration, the system provides a developer experience that matches the best aspects of Deno's performance with Python's discoverability.

The architecture is designed for incremental implementation while maintaining production-grade performance and reliability requirements. The result will be a completion system that significantly enhances developer productivity and reduces the cognitive load of learning new APIs and language features.