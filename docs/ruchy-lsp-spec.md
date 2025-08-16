# Ruchy Language Server Protocol Specification
**Version 1.0** | Status: Draft

## Executive Summary

The Ruchy LSP transforms the IDE from a passive text editor into an active development partner that guides programmers through the script-to-systems journey. It provides real-time transpilation visibility, performance coaching, and correctness verification while maintaining sub-100ms response times.

## Core Design Principles

### 1. Mechanical Transparency
Every abstraction is observable. Developers see exactly how their Ruchy code becomes Rust code, with performance implications made explicit.

### 2. Progressive Disclosure
Information density scales with user intent. Script mode shows minimal annotations; production mode reveals full performance and correctness analysis.

### 3. Zero-Latency Interaction
All user-facing operations complete in <100ms. Heavy computations run asynchronously with intelligent caching and debouncing.

## Architecture

### Performance Requirements
- **Typing Latency**: <50ms for syntax highlighting and basic completions
- **Diagnostic Latency**: <200ms for type errors, <500ms for complex analysis
- **Memory Budget**: <500MB for typical 50k LOC project
- **CPU Usage**: <25% average on 4-core system during active editing

### Threading Model
```rust
struct LspServer {
    main_thread: RespondsToUser,        // <50ms SLA
    type_checker: BackgroundWorker,     // Incremental, cached
    transpiler: BackgroundWorker,       // Debounced 500ms
    analyzer: BackgroundWorker,         // Performance profiling
    solver: BackgroundWorker,           // SMT verification
}
```

## Feature Specifications

### Phase 1: Foundation (v0.1)
**Target**: Q1 2025 | **Status**: In Progress

#### 1.1 Basic Language Features
- **Syntax Highlighting**: TextMate grammar with semantic tokens
- **Diagnostics**: Parse errors, type errors, borrow checker violations
- **Completions**: Context-aware with type-based filtering
- **Hover**: Type information, documentation, quick info
- **Go-to-Definition**: Cross-module navigation
- **Find References**: Usage search with semantic understanding
- **Document Symbols**: Outline view with filtering
- **Workspace Symbols**: Project-wide symbol search

#### 1.2 Core Commands
```typescript
interface RuchyCommands {
  // View commands
  "ruchy.view.showType": void;           // Show inferred type
  "ruchy.view.showRust": void;           // Preview transpiled code
  "ruchy.view.showAst": void;            // Display AST structure
  
  // Action commands
  "ruchy.action.runInRepl": void;        // Execute selection
  "ruchy.action.benchmark": void;        // Run performance test
  "ruchy.action.generateTest": void;     // Create property test
  
  // Analysis commands
  "ruchy.analyze.explainError": void;    // Detailed error explanation
  "ruchy.analyze.performanceProfile": void; // Static performance analysis
  "ruchy.analyze.escapeAnalysis": void;  // Show escape analysis results
}
```

### Phase 2: Transpilation Preview (v0.2)
**Target**: Q2 2025 | **Priority**: CRITICAL

#### 2.1 Real-time Transpilation View
```typescript
interface TranspilationPreview {
  mode: "side-by-side" | "inline" | "hover";
  showCostIndicators: boolean;      // Performance annotations
  showDiff: boolean;                 // Highlight transformations
  debounceMs: number;                // Default: 500ms
}
```

#### 2.2 Performance Cost Indicators
```rust
// Inline annotations showing cost
let data = vec![1, 2, 3]  // 📦 Heap allocation (24 bytes)
    .map(|x| x * 2)        // ✅ Zero-cost iterator
    .collect()             // 📦 Allocation for result
```

#### 2.3 Transformation Explanations
- Closure capturing → Rust ownership
- Pipeline operators → Method chains
- Pattern matching → Match expressions
- Actor messages → Channel operations

### Phase 3: Performance Coach (v0.3)
**Target**: Q3 2025

#### 3.1 Script vs Compiled Mode Detection
```typescript
interface ModeDetection {
  indicators: {
    hasMainFunction: boolean;
    usesRepl: boolean;
    hasTestAnnotations: boolean;
    fileExtension: ".ruchy" | ".ruchyx";
  };
  suggestions: ModeSpecificSuggestion[];
}
```

#### 3.2 Progressive Performance Profiling
```typescript
interface PerformanceAnalysis {
  // Static analysis (always available)
  allocations: AllocationSite[];     // Heap allocations detected via escape analysis
  escapeAnalysis: EscapeResult[];    // Stack vs heap placement decisions
  inliningHints: InlineCandidate[];  // Function inlining opportunities
  
  // Dynamic profiling (when profile data available)
  hotPaths?: HotPath[];              // From `ruchy profile` execution traces
  cacheMetrics?: CachePerf[];        // Runtime cache hit/miss ratios
  branchPrediction?: BranchStats[];  // Misprediction frequency
}
```

**Note**: Phase 3 provides static performance analysis based on compiler heuristics and escape analysis. Dynamic profiling data from runtime execution (`ruchy profile`) will be integrated in a future phase, allowing correlation between predicted and actual performance characteristics.

#### 3.3 Gradual Typing Assistant
```rust
// Visual indicators for type boundaries
fn process(data) {  // ⚠️ Inferred as Dynamic
    data.filter(|x| x > 0)  // 🔄 Monomorphization point
}

// Quick fix: Add type annotation
fn process(data: Vec<i32>) {  // ✅ Fully typed
    data.filter(|x| x > 0)
}
```

### Phase 4: Ecosystem Integration (v0.4)
**Target**: Q4 2025

#### 4.1 MCP Protocol Validation
```typescript
interface McpValidation {
  validateDecorators(): Diagnostic[];
  generateManifest(): ToolManifest;
  checkMessageSchemas(): SchemaError[];
  suggestProtocolPatterns(): Pattern[];
}
```

#### 4.2 Cargo Ecosystem Bridge
```typescript
interface CargoBridge {
  searchCrates(query: string): CrateInfo[];
  showCrateDocs(crate: string): Documentation;
  generateBindings(crate: string): RuchyBinding;
  checkCompatibility(crate: string): Warning[];
}
```

#### 4.3 Actor System Visualization
```typescript
interface ActorVisualization {
  supervisionTree: TreeNode[];
  messageFlows: Edge[];
  mailboxStates: QueueStatus[];
  deadlockAnalysis: PotentialDeadlock[];
}
```

### Phase 5: Verification Suite (v1.0)
**Target**: Q1 2026

#### 5.1 Property-Based Test Generation
```rust
// Automatic property generation from types
fn sort(list: Vec<i32>) -> Vec<i32> { ... }

// Generated properties:
#[property]
fn prop_sort_preserves_length(list: Vec<i32>) {
    assert_eq!(sort(list.clone()).len(), list.len())
}

#[property]
fn prop_sort_is_ordered(list: Vec<i32>) {
    let sorted = sort(list);
    sorted.windows(2).all(|w| w[0] <= w[1])
}
```

#### 5.2 Refinement Type Solver
```typescript
interface RefinementSolver {
  checkProof(type: RefinementType): ProofResult;
  showCounterexample(): Example;
  suggestWeakening(): RefinementType;
  explainProofObligation(): Explanation;
}
```

#### 5.3 Quality Gate Dashboard
```typescript
interface QualityMetrics {
  mutationCoverage: number;          // % mutants killed
  propertyCoverage: number;          // % properties tested
  complexityScore: number;           // Cyclomatic complexity
  technicalDebt: SatdItem[];         // Self-admitted debt
  performanceRegression: Delta[];    // vs baseline
}
```

## User Experience Design

### Information Hierarchy
1. **Always Visible**: Syntax errors, type errors
2. **On Hover**: Type info, performance hints, docs
3. **On Demand**: Transpilation preview, quality metrics
4. **In Panel**: Actor visualization, dashboards

### Visual Language
```
✅ Zero-cost abstraction     (green checkmark)
⚠️ Performance concern       (yellow warning)
❌ Error/unsupported         (red X)
📦 Heap allocation           (package emoji)
🔄 Type boundary             (cycle emoji)
⚡ Hot path                  (lightning emoji)
```


show_cost_indicators = true
show_type_boundaries = false
show_escape_analysis = false

# Quality
enable_property_generation = true
enable_mutation_hints = false
quality_gate_threshold = 0.8
```

## Implementation Strategy

### Technology Stack
- **Core**: Rust with tower-lsp framework
- **Parser**: Reuse ruchy-parser with incremental parsing
- **Type Checker**: Salsa for incremental computation
- **UI Integration**: VS Code extension (TypeScript)
- **Visualization**: D3.js for actor graphs

### Caching Strategy
```rust
struct Cache {
    ast_cache: HashMap<FileId, Ast>,           // Per-file
    type_cache: HashMap<ExprId, Type>,         // Per-expression
    transpilation_cache: HashMap<Hash, Rust>,  // Content-hash
    analysis_cache: TimedCache<Analysis>,      // 5-minute TTL
}
```

### Testing Requirements
- Unit tests for each LSP method
- Integration tests with VS Code
- Performance benchmarks (must meet SLA)
- Fuzzing for malformed input handling

## Success Metrics

### Performance KPIs
- P50 typing latency <30ms
- P95 diagnostic latency <200ms
- Memory usage <500MB for 50k LOC
- CPU usage <25% during active editing

### User Experience KPIs
- Time to first meaningful diagnostic <1s
- Transpilation preview update <500ms
- Zero false positives in error reporting
- 90% of errors have actionable quick fixes

## Appendix: Protocol Extensions

### Custom LSP Methods
```typescript
namespace RuchyProtocol {
  // Request transpiled Rust code
  interface TranspileRequest {
    method: "ruchy/transpile";
    params: { uri: DocumentUri };
  }
  
  // Get performance analysis
  interface AnalyzeRequest {
    method: "ruchy/analyze";
    params: { uri: DocumentUri; mode: "performance" | "quality" };
  }
  
  // Actor system state
  interface ActorStateRequest {
    method: "ruchy/actorState";
    params: { uri: DocumentUri };
  }
}
```

## References

- [LSP Specification 3.17](https://microsoft.github.io/language-server-protocol/)
- [Salsa Incremental Computation](https://github.com/salsa-rs/salsa)
- [Tower LSP Framework](https://github.com/ebkalderon/tower-lsp)
- [Rust Analyzer Architecture](https://github.com/rust-lang/rust-analyzer/blob/master/docs/dev/architecture.md)