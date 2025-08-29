# Object Inspection and Tab Completion Consistency Specification

## Executive Summary

REPLs require two orthogonal but complementary systems: object inspection for display and tab completion for discovery. Both must operate on the same type-directed namespace model to ensure consistency. This specification defines the minimal machinery for both systems.

## Part I: Object Inspection Protocol

### Core Abstraction

Object inspection differs from Debug/Display traits. It optimizes for human comprehension in interactive contexts with bounded resource consumption.

```rust
pub trait Inspect {
    fn inspect(&self, inspector: &mut Inspector) -> fmt::Result;
    
    fn inspect_depth(&self) -> usize {
        1 // Override for recursive structures
    }
}

pub struct Inspector {
    depth: usize,
    max_depth: usize,
    visited: VisitSet,
    budget: usize,          // Complexity bound
    style: InspectStyle,
}

struct VisitSet {
    // Optimized for <8 elements (common case)
    inline: [usize; 8],
    count: usize,
    overflow: Option<HashSet<usize>>,
}
```

### Display Classification

All values reduce to four canonical forms:

```rust
enum DisplayForm {
    Atomic(String),              // 42, true, "hello"
    Composite(CompositeForm),    // [1,2,3], {x: 10}
    Reference(usize, Box<DisplayForm>), // &value@0x7fff
    Opaque(OpaqueHandle),       // <fn>, <thread#42>
}

struct CompositeForm {
    opener: &'static str,
    elements: Vec<(Option<String>, DisplayForm)>,
    closer: &'static str,
    elided: Option<usize>,  // "...and 47 more"
}
```

### Automatic Derivation

```rust
#[proc_macro_derive(Inspect)]
pub fn derive_inspect(input: TokenStream) -> TokenStream {
    // Generate inspect impl with:
    // 1. Cycle detection via Inspector.visited
    // 2. Budget checking via Inspector.budget
    // 3. Depth limiting via Inspector.depth
}
```

### Performance Guarantees

- **Time**: O(min(size, budget))
- **Space**: O(min(depth, max_depth))
- **Allocation**: None for size < 128 bytes

### Implementation for Primitives

```rust
impl Inspect for i32 {
    fn inspect(&self, ins: &mut Inspector) -> fmt::Result {
        write!(ins, "{}", self)
    }
}

impl<T: Inspect> Inspect for Vec<T> {
    fn inspect(&self, ins: &mut Inspector) -> fmt::Result {
        if self.len() <= ins.style.max_elements {
            write!(ins, "[")?;
            for (i, item) in self.iter().enumerate() {
                if i > 0 { write!(ins, ", ")?; }
                item.inspect(ins)?;
            }
            write!(ins, "]")
        } else {
            // Elided display
            write!(ins, "[{} elements]", self.len())
        }
    }
}
```

## Part II: Tab Completion Engine

### Namespace Model

Tab completion operates on three distinct contexts:

```rust
#[derive(Debug, Clone)]
enum CompletionContext {
    FieldAccess { 
        receiver_type: Type,
        receiver_expr: String,
    },
    ModulePath {
        segments: Vec<String>,
    },
    FreeExpression {
        scope: ScopeId,
    },
}
```

### Context Analysis with Error Recovery

```rust
pub struct CompletionAnalyzer {
    parser: IncrementalParser,
    type_checker: TypeChecker,
    partial_cache: PartialTypeCache,
}

impl CompletionAnalyzer {
    pub fn analyze(&self, input: &str, cursor: usize) -> CompletionContext {
        let prefix = &input[..cursor];
        
        // Fast path: detect x.<TAB> pattern
        if let Some(dot_pos) = prefix.rfind('.') {
            let receiver = &prefix[..dot_pos];
            
            // Attempt full parse first
            match self.parser.parse_expression(receiver) {
                Ok(expr) => {
                    let ty = self.type_checker.infer_type(&expr);
                    return CompletionContext::FieldAccess {
                        receiver_type: ty,
                        receiver_expr: receiver.to_string(),
                    };
                }
                Err(_) => {
                    // Fallback: partial type recovery
                    if let Some(ty) = self.partial_cache.get(receiver) {
                        return CompletionContext::FieldAccess {
                            receiver_type: ty.clone(),
                            receiver_expr: receiver.to_string(),
                        };
                    }
                }
            }
        }
        
        // Check for module path
        if prefix.contains("::") {
            return self.analyze_module_path(prefix);
        }
        
        // Default: free expression
        CompletionContext::FreeExpression {
            scope: self.current_scope_id(),
        }
    }
}
```

### Trait and Extension Method Resolution

```rust
pub struct MethodResolver {
    trait_impls: HashMap<(TypeId, TraitId), ImplId>,
    extension_methods: MultiMap<TypeId, MethodId>,
    trait_hierarchy: TraitGraph,
}

impl MethodResolver {
    pub fn resolve_methods(&self, ty: &Type, scope: &Scope) -> Vec<Method> {
        let mut methods = Vec::new();
        
        // 1. Inherent methods
        methods.extend(self.inherent_methods(ty));
        
        // 2. Trait methods in scope
        for trait_id in scope.visible_traits() {
            if let Some(impl_id) = self.trait_impls.get(&(ty.id(), trait_id)) {
                methods.extend(self.impl_methods(*impl_id));
            }
        }
        
        // 3. Extension methods
        methods.extend(self.extension_methods.get_vec(&ty.id()));
        
        // 4. Deduplication and ambiguity resolution
        self.resolve_ambiguities(methods)
    }
    
    fn resolve_ambiguities(&self, mut methods: Vec<Method>) -> Vec<Method> {
        // Sort by specificity: inherent > trait > extension
        methods.sort_by_key(|m| match m.source {
            MethodSource::Inherent => 0,
            MethodSource::Trait(_) => 1,
            MethodSource::Extension => 2,
        });
        
        // Remove duplicates, keeping most specific
        methods.dedup_by(|a, b| a.name == b.name);
        methods
    }
}
```

### Machine Learning-Enhanced Ranking

```rust
pub struct RankingModel {
    // Static weights (initial)
    base_weights: RankWeights,
    
    // Learned from usage patterns
    usage_stats: UsageDatabase,
    
    // Context-aware scoring
    context_scorer: ContextScorer,
}

impl RankingModel {
    pub fn rank(&self, completions: Vec<Completion>, context: &Context) -> Vec<Completion> {
        let mut scored = completions
            .into_iter()
            .map(|c| {
                let score = self.compute_score(&c, context);
                (score, c)
            })
            .collect::<Vec<_>>();
        
        // Stable sort preserves alphabetical within same score
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        
        scored.into_iter().map(|(_, c)| c).collect()
    }
    
    fn compute_score(&self, completion: &Completion, context: &Context) -> f64 {
        let mut score = 0.0;
        
        // Base priority (local > param > method > import > prelude)
        score += match completion.source {
            Source::Local => 1000.0,
            Source::Parameter => 900.0,
            Source::Method => 800.0,
            Source::Import => 600.0,
            Source::Prelude => 400.0,
        };
        
        // Usage frequency (logarithmic)
        let usage = self.usage_stats.frequency(&completion.name);
        score += (usage as f64 + 1.0).ln() * 100.0;
        
        // Contextual relevance
        score += self.context_scorer.score(completion, context) * 50.0;
        
        // Recency bonus (exponential decay)
        if let Some(last_use) = self.usage_stats.last_used(&completion.name) {
            let age_seconds = (Instant::now() - last_use).as_secs();
            score += 200.0 * (-age_seconds as f64 / 3600.0).exp();
        }
        
        score
    }
}
```

## Part III: Consistency Enforcement

### Shared Type Registry

Both systems must reference the same type information:

```rust
pub struct TypeRegistry {
    types: HashMap<TypeId, TypeInfo>,
    methods: HashMap<TypeId, Vec<MethodId>>,
    
    // Cache for inspection
    inspect_impls: HashMap<TypeId, InspectImpl>,
    
    // Cache for completion
    completion_cache: HashMap<TypeId, Vec<Completion>>,
}

impl TypeRegistry {
    pub fn register_type(&mut self, ty: TypeInfo) {
        let id = ty.id();
        self.types.insert(id, ty.clone());
        
        // Update both caches
        self.inspect_impls.insert(id, derive_inspect(&ty));
        self.completion_cache.insert(id, extract_methods(&ty));
    }
}
```

### Testing Infrastructure

```rust
#[test]
fn test_inspection_completion_consistency() {
    let mut repl = Repl::new();
    repl.eval("let x = vec![1, 2, 3]");
    
    // Inspection should show vector
    let output = repl.inspect("x");
    assert!(output.contains("[1, 2, 3]"));
    
    // Completion should show vector methods
    let completions = repl.complete("x.", 2);
    assert!(completions.contains(&"len".to_string()));
    assert!(completions.contains(&"push".to_string()));
    
    // Should NOT contain unrelated methods
    assert!(!completions.contains(&"sin".to_string()));
    assert!(!completions.contains(&"parse".to_string()));
}

#[test]
fn test_primitive_isolation() {
    let mut repl = Repl::new();
    repl.eval("let n = 42");
    
    let completions = repl.complete("n.", 2);
    
    // Only integer methods
    for completion in &completions {
        assert!(
            INTEGER_METHODS.contains(&completion.as_str()),
            "Unexpected completion: {}", completion
        );
    }
}
```

### Incremental Analysis Engine

```rust
pub struct IncrementalAnalyzer {
    // Salsa-style query engine for sub-millisecond updates
    db: salsa::Database,
    
    // Keystroke-level caching
    keystroke_cache: Arc<DashMap<String, TypeInfo>>,
    
    // Async runtime for external queries
    runtime: tokio::Handle,
}

impl IncrementalAnalyzer {
    pub async fn analyze_async(&self, input: &str, cursor: usize) -> CompletionContext {
        // Check keystroke cache first (sub-microsecond)
        let cache_key = format!("{}:{}", input, cursor);
        if let Some(cached) = self.keystroke_cache.get(&cache_key) {
            return cached.context.clone();
        }
        
        // Incremental recomputation via Salsa
        let ctx = self.db.completion_context(input, cursor).await;
        
        // Update cache with 100ms TTL
        self.keystroke_cache.insert(cache_key, ctx.clone());
        
        ctx
    }
}
```

## Part IV: Common Pitfalls and Solutions

### Problem 1: Global Namespace Pollution

**Symptom**: Integer variable shows string methods in completion.

**Root Cause**: Merging all available symbols without context.

**Solution**: Strict context-based filtering:
```rust
// WRONG
completions = all_symbols.filter(|s| s.starts_with(prefix))

// CORRECT
completions = symbols_for_type(inferred_type).filter(|s| s.starts_with(prefix))
```

### Problem 2: Infinite Recursion in Inspection

**Symptom**: REPL hangs on circular data structures.

**Root Cause**: No cycle detection.

**Solution**: Visit set with address tracking:
```rust
impl Inspector {
    fn enter<T>(&mut self, val: &T) -> bool {
        let addr = val as *const T as usize;
        self.visited.insert(addr)
    }
}
```

### Problem 3: Completion Performance Degradation

**Symptom**: Slow completion in large codebases.

**Root Cause**: Recomputing type information on every keystroke.

**Solution**: Incremental caching with invalidation:
```rust
struct CompletionCache {
    type_cache: HashMap<String, Type>,
    method_cache: HashMap<TypeId, Vec<Method>>,
    generation: u64,
}
```

## Performance Requirements

- Tab completion latency: <50ms for 10K symbols
- Object inspection: <10ms for structures up to 1MB
- Memory overhead: <5% of REPL heap
- Cache invalidation: O(1) amortized

## Security Considerations

### Inspection of Untrusted Data
Never dereference raw pointers during inspection:
```rust
impl Inspect for *const T {
    fn inspect(&self, ins: &mut Inspector) -> fmt::Result {
        write!(ins, "<ptr@{:p}>", self)
        // Do NOT dereference
    }
}
```

### Resource Exhaustion
Enforce strict bounds:
```rust
const MAX_INSPECT_DEPTH: usize = 10;
const MAX_INSPECT_ITEMS: usize = 1000;
const MAX_COMPLETION_RESULTS: usize = 100;
```

### Asynchronous LSP Integration

```rust
pub struct AsyncCompletionService {
    analyzer: Arc<IncrementalAnalyzer>,
    resolver: Arc<MethodResolver>,
    ranker: Arc<RankingModel>,
}

#[async_trait]
impl tower_lsp::LanguageServer for AsyncCompletionService {
    async fn completion(
        &self,
        params: CompletionParams,
    ) -> Result<CompletionResponse> {
        let doc = params.text_document_position.text_document;
        let pos = params.text_document_position.position;
        
        // Convert LSP position to byte offset
        let input = self.get_document(&doc.uri).await?;
        let cursor = self.position_to_offset(&input, pos);
        
        // Parallel analysis
        let (context, type_info) = tokio::join!(
            self.analyzer.analyze_async(&input, cursor),
            self.resolver.prefetch_types(&input)
        );
        
        // Generate candidates with timeout
        let candidates = tokio::time::timeout(
            Duration::from_millis(45), // Leave 5ms for ranking
            self.generate_candidates(context, type_info)
        ).await.unwrap_or_else(|_| {
            // Fallback to cached results on timeout
            self.cached_candidates()
        });
        
        // Rank and return
        let ranked = self.ranker.rank(candidates, &context);
        Ok(CompletionResponse::Array(ranked))
    }
}

impl AsyncCompletionService {
    async fn generate_candidates(
        &self,
        ctx: CompletionContext,
        types: TypeCache,
    ) -> Vec<Completion> {
        match ctx {
            CompletionContext::FieldAccess { receiver_type, .. } => {
                // May require async crate metadata lookup
                self.resolver.resolve_methods_async(&receiver_type).await
            }
            CompletionContext::ModulePath { segments } => {
                // May require filesystem access
                self.module_completions_async(segments).await
            }
            CompletionContext::FreeExpression { scope } => {
                // Usually synchronous
                self.scope_completions(scope)
            }
        }
    }
}

## References

- Rust Analyzer: Type-directed completion
- Julia REPL: Lazy inspection protocol
- Chrome DevTools: Object inspection budget
- Language Server Protocol: Completion context model