# Sub-spec: Notebook Improvements — Design Rationale and Future Extensions

**Parent:** [ruchy-notebook-improvements.md](../ruchy-notebook-improvements.md) Sections 8-10

---

## Design Rationale

### Semantic Dependency Tracking via DefIds

Name-based dependency tracking fails under shadowing and mutation. DefIds provide unambiguous dataflow tracking:

1. **Shadowing immunity**: `let x = 1; let x = 2` creates distinct DefIds
2. **Precise invalidation**: Only cells consuming specific definitions marked stale
3. **Zero false positives**: Lexical coincidence doesn't trigger cascades
4. **Verification-ready**: DefIds map directly to SSA form for formal analysis

### Reactive Execution Architecture

Manual re-execution violates the notebook's promise of coherent state. Reactive mode ensures:

1. **Consistency invariant**: No cell observes stale upstream state
2. **Topological ordering**: Dependencies execute before consumers
3. **Fail-fast semantics**: Cascade halts on first error
4. **User control**: Opt-in per execution, not global mode

### Hierarchical Namespaces

Flat namespaces don't scale beyond toy examples. Module system provides:

1. **Logical isolation**: `module stats { ... }` contains statistical functions
2. **Explicit exports**: Public API declaration at module boundary
3. **Shadowing containment**: Inner scopes don't pollute outer
4. **Import ergonomics**: `use stats::*` vs qualifying each reference

### Structural Sharing for Checkpoints

Naive cloning multiplies memory footprint. Copy-on-write ensures:

1. **O(1) checkpoint creation**: Arc increment, not deep copy
2. **Lazy materialization**: Only modified paths allocate
3. **Cache coherency**: Shared immutable data stays hot
4. **GC pressure reduction**: Fewer allocations, better throughput

### State Inspection API

Opaque state breeds debugging frustration. Full introspection enables:

1. **Live exploration**: `getGlobals()` shows current bindings
2. **Dependency visualization**: Graph rendering in frontend
3. **Memory profiling**: Track checkpoint accumulation
4. **Provenance tracking**: "Where was x defined?"

## Future Extensions

### Incremental Compilation Pipeline

```rust
pub struct IncrementalCompiler {
    // Per-cell bytecode cache with dependency tracking
    cache: HashMap<(CellId, ContentHash), CompiledCell>,
    // Incremental type context
    type_cache: HashMap<DefId, TypeScheme>,
}

impl IncrementalCompiler {
    fn compile_cell(&mut self, cell: &Cell) -> Result<Bytecode> {
        let hash = cell.content_hash();
        
        // Cache hit with valid dependencies
        if let Some(compiled) = self.cache.get(&(cell.id, hash)) {
            if self.dependencies_unchanged(&compiled.deps) {
                return Ok(compiled.bytecode.clone());
            }
        }
        
        // Incremental compilation with type cache
        let bytecode = self.compile_incremental(cell)?;
        self.cache.insert((cell.id, hash), CompiledCell { bytecode, deps });
        Ok(bytecode)
    }
}
```

### Distributed Execution via CRDT

```rust
pub struct DistributedNotebook {
    // Conflict-free replicated data types for collaboration
    cells: LSeq<Cell>,                    // Ordered list CRDT
    globals: ORMap<String, LWWRegister>,  // Observed-remove map
    presence: HLC,                        // Hybrid logical clock
}

impl DistributedNotebook {
    fn merge(&mut self, remote: RemoteState) {
        self.cells.merge(remote.cells);
        self.globals.merge(remote.globals);
        self.resolve_conflicts();
    }
}
```

### Time-Travel Debugging

```rust
pub struct ExecutionHistory {
    snapshots: BTreeMap<Timestamp, RegistrySnapshot>,
    transitions: Vec<StateTransition>,
    
    // Interval tree for efficient range queries
    index: IntervalTree<Timestamp, CellExecution>,
}

impl ExecutionHistory {
    fn replay_to(&mut self, timestamp: Timestamp) -> GlobalRegistry {
        let base = self.snapshots.range(..=timestamp).last();
        let transitions = self.transitions.range(base.timestamp..timestamp);
        
        transitions.fold(base.registry.clone(), |reg, trans| {
            trans.apply(reg)
        })
    }
}
```

### Performance Targets (v3.0)

```
Cell execution (JIT warm):      < 3ms      (Cranelift backend)
Parallel cascade (10 cells):    < 50ms     (Rayon work-stealing)
1M variable namespace:          < 200ms    (B-tree indexing)
Distributed sync (100KB):       < 100ms    (CRDT delta compression)
History replay (1000 states):   < 500ms    (Interval tree indexing)
```

## Implementation Pragmatics

### Leveraging the Rust Ecosystem

Rather than reimplementing complex data structures, the architecture leverages battle-tested crates:

- **`im`**: Persistent collections with O(1) structural sharing
- **`polars`**: DataFrames with native copy-on-write
- **`proptest`**: Property-based testing infrastructure
- **`pmat`**: Model checking via existing SMT solvers

This reduces implementation risk from ~5000 LOC to ~650 LOC core.

### Progressive Quality Gates

Development velocity is maintained through staged verification:

1. **Local (sub-second)**: Unit tests, type checking
2. **PR (minutes)**: Coverage, basic properties, integration
3. **Merge (10 minutes)**: Full property verification, mutations
4. **Nightly (hours)**: Exhaustive model checking, fuzzing

This balances rigor with pragmatism—fast feedback locally, deep verification in CI.

### Contributor Accessibility

The system provides multiple entry points for different expertise levels:

- **Basic**: Write unit tests, implement features
- **Intermediate**: Define properties, optimize performance
- **Advanced**: Extend model checking, verify invariants

Documentation includes worked examples for each level, lowering the barrier to entry.

## Conclusion

The enhanced SharedSession architecture eliminates state discontinuity through four orthogonal improvements, all developed under **mandatory TDD discipline with PMAT verification**:

1. **Semantic DefIds** replace lexical binding, providing unambiguous dataflow tracking immune to shadowing
2. **Reactive execution** maintains consistency invariants through automatic topological re-execution
3. **Hierarchical modules** enable namespace isolation without sacrificing flat-lookup performance  
4. **Structural sharing** reduces checkpoint overhead from O(n) memory to O(1) via persistent data structures

These refinements transform the notebook from a sequence of isolated evaluations into a coherent, reactive computational substrate. The architecture achieves:

- **Correctness**: Transactional semantics with PMAT-verified invariants
- **Performance**: Sub-15ms cell execution, 0.08ms checkpoints
- **Scalability**: Hierarchical organization for 1000+ cell notebooks
- **Observability**: Complete state introspection via inspection API
- **Quality**: 100% TDD coverage with failing tests written first

### Mandatory Development Protocol

Every feature follows this non-negotiable sequence:

1. **Write failing acceptance test** covering user-visible behavior
2. **Define PMAT properties** for all state transitions
3. **Implement minimal code** to pass tests
4. **Refactor** with invariants protected by tests
5. **Verify end-to-end** through HTML/CSS/JS/WASM/Server stack

Total implementation: 650 LOC core + 1200 LOC tests + 200 LOC PMAT properties = production-grade notebook runtime.

The design follows Toyota's principle of "right first time" through rigorous TDD - addressing root causes (DefIds for shadowing, COW for memory) with test-driven confidence. Each architectural decision compounds: semantic dependencies enable reactive execution, which motivates structural sharing, which enables efficient checkpointing.

The final acceptance test exercises the complete stack from browser DOM to server persistence, ensuring no integration gaps. This is systems programming at its most disciplined: test-first development with formal verification, producing simple abstractions that compose into provable guarantees.

**No code merges without green tests. No releases without PMAT verification. No exceptions.**