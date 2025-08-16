# Ruchy Poly-representational Disassembly Specification

## 1. Core Architecture

The disassembly system operates as a post-semantic-analysis compiler phase, extracting structured representations from the typed AST. Each representation serves a distinct purpose in the toolchain.

```rust
pub trait Disassembler<Input> {
    type Output;
    fn disassemble(&self, input: &Input) -> Self::Output;
}

// Primary disassemblers consume TypedAst
impl Disassembler<TypedAst> for JsonAstDisassembler { ... }
impl Disassembler<TypedAst> for MirDisassembler { ... }

// Derived disassemblers consume primary outputs
impl Disassembler<JsonAst> for AnnotatedAstDisassembler { ... }
impl Disassembler<SymbolTable> for MermaidDepsDisassembler { ... }
```

## 2. Representation Hierarchy

### 2.1 Primary Representations

Generated directly from compiler internals. These are canonical sources of truth.

| Format | Purpose | Consumer | Stability |
|--------|---------|----------|-----------|
| `json-ast` | Canonical AST representation | MCP agents, tooling | Stable v1.0 |
| `symbol-table` | Canonical entity index | Static analyzers | Stable v1.0 |
| `mir` | Optimization analysis | Compiler developers | Internal |
| `rust` | Transpilation target | Build systems | Stable |
| `asm` | Performance verification | Systems programmers | Platform-specific |

### 2.2 Derived Representations

Generated from primary representations. Optimized for specific use cases.

| Format | Purpose | Base | Update Frequency |
|--------|---------|------|------------------|
| `annotated-ast` | Human-readable summary | `json-ast` | Per-release |
| `mermaid-deps` | Dependency visualization | `symbol-table` | On-demand |
| `typed-ruchy` | Type annotation overlay | `json-ast` + source | Debug-only |
| `mermaid-cfg` | Control flow visualization | `mir` | On-demand |

## 3. JSON AST Schema

The canonical representation. All other formats derive from this.

```typescript
interface AstNode {
    id: string;           // Content-hash: SHA256(kind + span + children_ids)[:8]
    kind: NodeKind;       // Discriminant
    span: [number, number];
    ty?: Type;           // Present on all Expression nodes post-inference
    attrs?: Attributes;  // Metadata
}

// Type field presence rules:
// - ALWAYS present: Expression nodes (literals, calls, binops, etc.)
// - NEVER present: Structural nodes (blocks, match arms, modules)
// - CONDITIONALLY present: Statements (let bindings have it, imports don't)

interface FunctionNode extends AstNode {
    kind: "Function";
    name: string;
    params: ParamNode[];
    body: AstNode;
    ty: FunctionType;    // Always present post-inference
    complexity: {
        cyclomatic: number;
        cognitive: number;
    };
}

// ID Generation Algorithm:
// 1. Serialize node's structural content (kind, span, child IDs)
// 2. Compute SHA256 hash
// 3. Take first 8 hex characters
// This ensures deterministic, content-based IDs that remain stable
// across compilations of identical code
```

## 4. Disassembly Pipeline

```rust
impl Compiler {
    pub fn disassemble(&self, format: Format, source: &str) -> Result<Output> {
        // Shared frontend
        let ast = self.parse(source)?;
        let typed_ast = self.type_check(ast)?;
        
        // Format-specific backend
        match format {
            Format::JsonAst => JsonDisassembler.disassemble(&typed_ast),
            Format::Mir => self.lower_to_mir(&typed_ast),
            Format::Rust => self.transpile(&typed_ast),
            Format::Asm => self.codegen(&typed_ast).emit_asm(),
            _ => DerivedDisassembler::from(format).disassemble(&typed_ast),
        }
    }
}
```

## 5. MIR Specification

Mid-level representation exposes optimization decisions without platform specifics.

```rust
pub enum MirOp {
    Assign(Local, Rvalue),
    Call(Local, Fn, Vec<Operand>),
    Branch(BasicBlock, BasicBlock, Operand),
    Return(Operand),
}

pub struct MirFunction {
    locals: Vec<LocalDecl>,
    blocks: Vec<BasicBlock>,
    span_map: HashMap<MirOp, Span>,  // Source mapping
}
```

Example output:
```mir
fn analyze(_1: DataFrame) -> Result<Statistics> {
    bb0: {
        _2 = filter(move _1, const "value > 0");
        _3 = groupby(move _2, const "category");
        _4 = collect(move _3);
        return _4;
    }
}
```

## 6. Dependency Graph Generation

Dependency extraction runs as a single AST traversal, building both call and type dependency edges.

```rust
struct DependencyExtractor {
    graph: petgraph::Graph<Symbol, EdgeKind>,
    current_scope: ScopeId,
}

// Extensible edge taxonomy - designed for language growth
enum EdgeKind {
    // Current edges
    Calls,           // Function invocation
    Instantiates,    // Type construction
    Implements,      // Trait implementation
    Depends,         // Generic dependency
    
    // Future extension points (reserved)
    HasTraitBound,   // When trait bounds are added
    HasLifetime,     // When explicit lifetimes are added
    Awaits,          // Async dependency tracking
    Mutates,         // Mutation analysis
}

impl Visitor for DependencyExtractor {
    fn visit_call(&mut self, call: &Call) {
        let caller = self.current_scope.symbol();
        let callee = self.resolve(call.fn_name);
        self.graph.add_edge(caller, callee, EdgeKind::Calls);
    }
    
    fn visit_type_ref(&mut self, ty: &TypeRef) {
        let user = self.current_scope.symbol();
        let type_sym = self.resolve_type(ty);
        self.graph.add_edge(user, type_sym, EdgeKind::Instantiates);
    }
}

// The EdgeKind enum is versioned alongside the language.
// New variants can be added without breaking existing tools
// that only understand a subset of edge types.
```

## 7. Symbol Table Structure

Flat, indexed representation of all named entities.

```json
{
  "symbols": [
    {
      "id": "analyze_7a8f",
      "name": "analyze",
      "kind": "function",
      "ty": "(DataFrame) -> Result<Statistics>",
      "scope": "module",
      "span": [45, 180],
      "refs": ["call_92ab", "call_f3d1"]
    }
  ],
  "index": {
    "by_name": { "analyze": ["analyze_7a8f"] },
    "by_kind": { "function": ["analyze_7a8f", "process_8b2c"] }
  }
}
```

## 8. Assembly Annotation

Assembly output includes source mapping for correlation with high-level constructs.

```asm
; Function: analyze (DataFrame) -> Result<Statistics>
; Source: examples/analysis.ruchy:45-180
analyze_7a8f:
    push    rbp
    mov     rbp, rsp
    sub     rsp, 48
    ; Line 46: data |> filter(col("value") > 0)
    mov     rdi, qword ptr [rbp - 8]
    lea     rsi, [rip + .Lstr_value]
    call    filter_impl
    ; Line 47: |> groupby("category")
    mov     rdi, rax
    lea     rsi, [rip + .Lstr_category]
    call    groupby_impl
```

## 9. CLI Interface

```bash
# Primary disassembly
ruchy disassemble json-ast src/main.ruchy

# Pipeline composition
ruchy disassemble mir src/main.ruchy | ruchy optimize --level 3

# Batch processing
ruchy disassemble --all-formats --output-dir ./analysis src/**/*.ruchy

# Streaming for agents
ruchy disassemble --watch json-ast src/ | mcp-agent process
```

## 10. Performance Constraints

- JSON AST generation: <10ms per 1000 LOC
- MIR lowering: <50ms per 1000 LOC  
- Dependency extraction: O(n) single-pass
- Memory overhead: <2x source size

## 11. Versioning Strategy

The `json-ast` format follows semantic versioning:
- v1.0: Initial stable schema
- v1.x: Additive changes only (new optional fields)
- v2.0: Breaking changes with migration tool

All formats include version metadata:
```json
{
  "format": "json-ast",
  "version": "1.0.0",
  "compiler": "ruchy-0.5.0",
  "timestamp": "2025-01-15T10:00:00Z",
  "data": { ... }
}
```

## 12. Agent Integration Protocol

MCP agents access disassembly via stdio protocol:

```json
{
  "method": "disassemble",
  "params": {
    "format": "json-ast",
    "source": "inline",
    "content": "fun compute(x: i32) -> i32 { x * 2 }"
  }
}
```

Response:
```json
{
  "result": {
    "format": "json-ast",
    "version": "1.0.0",
    "ast": { ... }
  }
}
```

## 14. Metrics Collection System

Supporting the rich annotations requires a comprehensive metrics collector that runs during semantic analysis:

```rust
pub struct MetricsCollector {
    complexity: ComplexityAnalyzer,
    halstead: HalsteadAnalyzer,
    big_o: BigOAnalyzer,
    provability: ProvabilityAnalyzer,
    coverage: CoverageEstimator,
    defect_predictor: DefectPredictor,
    satd_detector: SatdDetector,
}

pub struct FunctionMetrics {
    // McCabe Cyclomatic Complexity
    cyclomatic: u32,
    
    // Cognitive Complexity (Sonar methodology)
    cognitive: u32,
    
    // Big-O Complexity
    big_o: BigOComplexity,
    big_o_confidence: f32,
    
    // Halstead Metrics
    halstead: HalsteadMetrics {
        distinct_operators: u32,
        distinct_operands: u32,
        total_operators: u32,
        total_operands: u32,
        difficulty: f64,
        volume: f64,
        effort: f64,
        time: f64,
        bugs: f64,
    },
    
    // Quality Metrics
    maintainability_index: f32,  // 0-100 scale
    provability: f32,            // 0-100% formal verifiability
    coverage_estimate: f32,      // 0-100% estimated test coverage
    defect_probability: f32,     // 0-100% likelihood of bugs
    
    // Code Shape Metrics
    lines_of_code: u32,
    logical_lines: u32,
    parameters: u32,
    returns: u32,
    max_nesting: u32,
    branch_points: u32,
    error_paths: u32,
    
    // Dependencies
    called_functions: Vec<FunctionId>,
    used_types: Vec<TypeId>,
    external_crates: Vec<CrateId>,
    
    // Technical Debt
    satd_comments: Vec<SatdComment>,
    code_smells: Vec<CodeSmell>,
}

impl MetricsCollector {
    pub fn analyze_function(&self, func: &TypedFunction) -> FunctionMetrics {
        // Single-pass collection of all metrics
        let mut metrics = FunctionMetrics::default();
        
        // Walk AST once, collecting all metrics
        self.visit_function(func, &mut metrics);
        
        // Post-process for derived metrics
        metrics.maintainability_index = self.calculate_mi(&metrics);
        metrics.defect_probability = self.predict_defects(&metrics);
        
        metrics
    }
}

// Big-O Analysis with confidence scoring
enum BigOComplexity {
    Constant,           // O(1)
    Logarithmic,       // O(log n)
    Linear,            // O(n)
    Linearithmic,      // O(n log n)
    Quadratic,         // O(n²)
    Cubic,             // O(n³)
    Exponential,       // O(2^n)
    Factorial,         // O(n!)
    Unknown,           // O(?)
}

// SATD (Self-Admitted Technical Debt) detection
struct SatdComment {
    span: Span,
    category: SatdCategory,
    text: String,
}

enum SatdCategory {
    Todo,
    Fixme,
    Hack,
    Optimize,
    Refactor,
    TechnicalDebt,
}
```

These metrics are computed during type checking and cached in the AST nodes, making them available to all disassembly formats without recomputation.