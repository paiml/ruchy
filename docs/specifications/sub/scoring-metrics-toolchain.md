# Sub-spec: Ruchy Scoring — Metrics, CLI, and Extended Toolchain

**Parent:** [ruchy-scoring-spec.md](../ruchy-scoring-spec.md) Sections 6-12

---

## Metric Definitions

### Correctness (35%)

```rust
fn score_correctness(code: &Analysis) -> f64 {
    let mut score = 1.0;
    
    // Property test coverage (10% of correctness)
    let property_coverage = code.property_tests.coverage_ratio();
    score *= 0.9 + (property_coverage * 0.1);
    
    // Refinement type proofs (10%)
    let proof_ratio = code.proven_refinements / code.total_refinements.max(1);
    score *= 0.9 + (proof_ratio * 0.1);
    
    // Pattern match exhaustiveness (5%)
    let exhaustive_ratio = code.exhaustive_matches / code.total_matches.max(1);
    score *= 0.95 + (exhaustive_ratio * 0.05);
    
    // Mutation score (10%)
    if let Some(mutation_score) = code.mutation_score {
        score *= 0.9 + (mutation_score * 0.1);
    }
    
    score
}
```

### Performance (25%)

```rust
fn score_performance(code: &Analysis) -> f64 {
    // Allocation efficiency
    let alloc_score = 1.0 - (code.heap_allocs_per_op().min(0.3));
    
    // Complexity bounds
    let complexity_score = match code.max_cyclomatic {
        0..=10 => 1.0,
        11..=20 => 0.9 - ((code.max_cyclomatic - 10) as f64 * 0.01),
        _ => 0.7,
    };
    
    // Performance predictability
    let variance_score = 1.0 - (code.perf_coefficient_of_variation().min(0.3));
    
    (alloc_score * 0.4 + complexity_score * 0.4 + variance_score * 0.2)
}
```

### Maintainability (20%)

```rust
fn score_maintainability(code: &Analysis) -> f64 {
    // Afferent/efferent coupling
    let coupling_score = 1.0 / (1.0 + code.coupling_factor);
    
    // Module cohesion (LCOM4)
    let cohesion_score = code.average_cohesion;
    
    // Change amplification factor
    let amplification_score = 1.0 / code.change_amplification.max(1.0);
    
    (coupling_score * 0.3 + cohesion_score * 0.4 + amplification_score * 0.3)
}
```

### Safety (15%)

```rust
fn score_safety(code: &Analysis) -> f64 {
    let mut score = 1.0;
    
    // Unsafe block density
    let unsafe_density = code.unsafe_blocks as f64 / code.total_blocks as f64;
    score -= unsafe_density.min(0.5);
    
    // Error handling quality
    let unwrap_density = code.unwrap_count as f64 / code.result_types as f64;
    score -= (unwrap_density * 0.3).min(0.3);
    
    // Lifetime correctness proofs
    score += (code.lifetime_proofs as f64 * 0.01).min(0.2);
    
    score.max(0.0)
}
```

### Idiomaticity (5%)

```rust
fn score_idiomaticity(code: &Analysis) -> f64 {
    let iterator_score = code.iterator_usage_ratio();
    let error_score = 1.0 - code.panic_path_ratio();
    let pattern_score = code.pattern_match_ratio();
    
    (iterator_score * 0.3 + error_score * 0.5 + pattern_score * 0.2)
}
```

## Addressing Implementation Challenges

### 1. Mitigating "Garbage In, Garbage Out"

```rust
impl QualityScore {
    pub fn with_confidence(&self) -> (f64, f64) {
        // Return score with confidence interval
        let confidence = self.calculate_confidence();
        (self.value, confidence)
    }
    
    fn calculate_confidence(&self) -> f64 {
        // Confidence based on analysis completeness
        let coverage_confidence = self.test_coverage_known as u8 as f64;
        let static_confidence = self.static_analysis_depth as f64 / 3.0;
        let sample_confidence = (self.samples_analyzed as f64 / 1000.0).min(1.0);
        
        (coverage_confidence * 0.4 + static_confidence * 0.4 + sample_confidence * 0.2)
    }
}
```

### 2. Preventing Gaming

```toml
# .ruchy/score.toml - Per-project configuration
[scoring]
# Teams can adjust weights within bounds
correctness_weight = 0.35    # (0.25-0.45)
performance_weight = 0.25     # (0.15-0.35)
maintainability_weight = 0.20 # (0.10-0.30)
safety_weight = 0.15          # (0.10-0.25)
idiomaticity_weight = 0.05    # (0.00-0.15)

# Anti-gaming rules
[scoring.rules]
min_property_tests_per_function = 3
max_trivial_test_ratio = 0.2
min_mutation_kill_ratio = 0.75
```

### 3. Performance Optimization

```rust
pub struct IncrementalScorer {
    ast_cache: DashMap<PathBuf, AstAnalysis>,
    type_cache: DashMap<ModuleId, TypeAnalysis>,
    score_cache: LruCache<FileHash, ComponentScores>,
    
    // Dependency tracking for incremental updates
    dep_graph: DependencyGraph,
    
    // Background analysis thread pool
    analyzer_pool: ThreadPool,
}

impl IncrementalScorer {
    pub fn watch_mode(&self, path: &Path) -> impl Stream<Item = QualityScore> {
        // Return score stream with progressive refinement
        stream::unfold(AnalysisDepth::Shallow, |depth| async {
            let score = self.score(path, depth);
            let next_depth = depth.next().filter(|_| score.confidence < 0.95);
            Some((score, next_depth?))
        })
    }
}
```

### 4. Dimension Preservation

```rust
impl QualityScore {
    pub fn explain_delta(&self, previous: &QualityScore) -> Explanation {
        let mut explanation = Explanation::new();
        
        for (component, weight) in self.components.iter() {
            let delta = component.value - previous.get(component.name);
            if delta.abs() > 0.01 {
                explanation.add_change(component.name, delta, weight);
            }
        }
        
        explanation.add_tradeoffs(self.identify_tradeoffs(previous));
        explanation
    }
    
    fn identify_tradeoffs(&self, previous: &QualityScore) -> Vec<Tradeoff> {
        // Detect when improvements in one dimension hurt another
        let mut tradeoffs = vec![];
        
        if self.components.performance > previous.components.performance &&
           self.components.maintainability < previous.components.maintainability {
            tradeoffs.push(Tradeoff::PerformanceVsMaintainability {
                perf_gain: self.components.performance - previous.components.performance,
                maint_loss: previous.components.maintainability - self.components.maintainability,
            });
        }
        
        tradeoffs
    }
}
```

## CLI Interface

```bash
# Fast feedback (AST-only, <100ms)
ruchy score src/main.ruchy --fast

# Standard analysis (default, <1s)
ruchy score src/

# Deep analysis for CI (complete, <30s)
ruchy score src/ --deep

# Watch mode with progressive refinement
ruchy score src/ --watch

# Explain score changes
ruchy score src/ --explain --baseline=main

# Project-specific configuration
ruchy score src/ --config=.ruchy/score.toml

# MCP integration
ruchy score src/ --json | mcp-client analyze
```

## Grade Boundaries

```rust
pub enum Grade {
    APlus,  // [0.97, 1.00] - Ship to production
    A,       // [0.93, 0.97) - Ship with confidence
    AMinus,  // [0.90, 0.93) - Ship with review
    BPlus,   // [0.87, 0.90) - Acceptable
    B,       // [0.83, 0.87) - Needs work
    BMinus,  // [0.80, 0.83) - Minimum viable
    CPlus,   // [0.77, 0.80) - Technical debt
    C,       // [0.73, 0.77) - Refactor advised
    CMinus,  // [0.70, 0.73) - Refactor required
    D,       // [0.60, 0.70) - Major issues
    F,       // [0.00, 0.60) - Fundamental problems
}
```

## Integration Points

### CI/CD Pipeline

```yaml
- name: Quality Gate
  run: |
    # Fast check for PRs
    ruchy score src/ --fast --min=0.75
    
    # Deep check for main branch
    if [[ "$GITHUB_REF" == "refs/heads/main" ]]; then
      ruchy score src/ --deep --min=0.85
    fi
```

### MCP Tool

```rust
#[mcp::tool]
async fn analyze_code_quality(path: String, depth: Option<String>) -> QualityReport {
    let depth = depth.map(|d| d.parse()).transpose()?.unwrap_or_default();
    let score = ruchy::score(&path, depth).await?;
    
    QualityReport {
        score: score.value,
        confidence: score.confidence,
        grade: score.grade,
        components: score.components,
        recommendations: score.generate_recommendations(),
        tradeoffs: score.identify_tradeoffs(&baseline),
    }
}
```

## Success Metrics

- **Adoption**: 80% of Ruchy projects using score in CI
- **Accuracy**: <5% false positive rate on quality issues
- **Performance**: <100ms for incremental scoring
- **Trust**: 90% developer agreement with score assessments
- **Impact**: 25% reduction in production defects

## Future Enhancements

1. **Machine Learning Calibration**: Train weights on defect correlation data
2. **Team Profiles**: Learn team-specific quality preferences
3. **Historical Trending**: Score velocity and trajectory analysis
4. **Cross-Language Scoring**: Extend to transpiled Rust code
5. **Distributed Analysis**: Cluster-based deep analysis for large codebases

## Extended Toolchain Integration

### 1. Mechanical Sympathy Tuner (`ruchy optimize`)

Profile-guided optimization with hardware-aware cost modeling.

```rust
pub struct MechanicalSympathyTuner {
    cpu_model: CpuCostModel,        // Cycle-accurate predictions via llvm-mca
    profile_data: ProfileDatabase,   // PGO data from previous runs
    cache_simulator: CacheSimulator, // Abstract interpretation for cache behavior
}

impl MechanicalSympathyTuner {
    pub fn analyze_abstraction_cost(&self, feature: AbstractionType) -> CostReport {
        // Track monomorphization costs
        match feature {
            AbstractionType::Option(inner) => {
                let niche_cost = self.calculate_niche_optimization(inner);
                let branch_cost = self.cpu_model.predict_branch_overhead();
                CostReport::new(niche_cost + branch_cost)
            }
            AbstractionType::Iterator(chain) => {
                self.verify_zero_cost_iteration(chain)
            }
        }
    }
    
    pub fn hardware_lint(&self, ast: &AST) -> Vec<HardwareLint> {
        vec![
            self.detect_cache_line_contention(ast),
            self.find_missed_vectorization(ast),
            self.identify_branch_misprediction_hotspots(ast),
        ].into_iter().flatten().collect()
    }
}
```

**Integration with Score:**
```rust
// Hardware efficiency becomes a component of performance score
fn score_performance_with_sympathy(code: &Analysis, tuner: &MechanicalSympathyTuner) -> f64 {
    let base_score = score_performance(code);
    let hw_efficiency = tuner.calculate_hardware_efficiency(code);
    base_score * 0.8 + hw_efficiency * 0.2
}
```

### 2. Actor Observatory (`ruchy actor:observe`)

Live introspection of concurrent systems without performance degradation.

```rust
pub struct ActorObservatory {
    telemetry_buffer: LockFreeRingBuffer<ActorEvent>,
    shadow_states: DashMap<ActorId, ShadowState>,
    supervision_tree: Arc<RwLock<SupervisionTree>>,
}

#[cfg_attr(observe, instrument)]
trait ObservableActor: Actor {
    fn observe_state(&self) -> ShadowState {
        // Copy-on-observe semantics
        ShadowState {
            mailbox_depth: self.mailbox.len(),
            memory_usage: self.estimate_memory(),
            reduction_count: self.reductions,
            generation: self.state_generation.fetch_add(1, Ordering::SeqCst),
        }
    }
}

impl ActorObservatory {
    pub fn trace_messages(&self, filter: MessageFilter) -> impl Stream<Item = Message> {
        self.telemetry_buffer
            .stream()
            .filter(move |msg| filter.matches(msg))
    }
    
    pub fn render_tui(&self) -> Result<()> {
        // Terminal UI with real-time updates
        let ui = tui::Terminal::new()?;
        ui.draw(|f| {
            f.render_widget(ActorDashboard::new(&self.shadow_states), f.size());
        })
    }
}
```

**Score Integration:**
```rust
// Actor health contributes to safety score
fn score_actor_safety(observatory: &ActorObservatory) -> f64 {
    let deadlock_risk = observatory.detect_potential_deadlocks();
    let mailbox_health = observatory.average_mailbox_depth() / MAX_HEALTHY_MAILBOX;
    1.0 - (deadlock_risk * 0.5 + mailbox_health.min(1.0) * 0.5)
}
```

### 3. Dataflow Debugger (`ruchy dataflow:debug`)

Interactive debugging for DataFrame pipelines with materialization on demand.

```rust
pub struct DataflowDebugger {
    pipeline_dag: DataflowGraph,
    breakpoints: HashSet<NodeId>,
    materialization_cache: LruCache<NodeId, DataFrame>,
}

enum DataflowNode {
    Lazy(Box<dyn Fn() -> DataFrame>),
    Materialized(Arc<DataFrame>), // Arc for zero-copy viewing
    Breakpoint(Box<DataflowNode>),
}

impl DataflowDebugger {
    pub fn step_into(&mut self, node_id: NodeId) -> DebugContext {
        let df = self.materialize_at(node_id);
        DebugContext {
            data: df.clone(),
            schema: df.schema(),
            prev_diff: self.compute_diff(node_id),
            query_plan: self.pipeline_dag.explain_at(node_id),
        }
    }
    
    pub fn set_breakpoint(&mut self, location: &str) -> Result<()> {
        let node_id = self.pipeline_dag.find_by_location(location)?;
        self.breakpoints.insert(node_id);
        Ok(())
    }
}
```

**Score Integration:**
```rust
// Pipeline complexity affects maintainability
fn score_dataflow_maintainability(debugger: &DataflowDebugger) -> f64 {
    let pipeline_depth = debugger.pipeline_dag.max_depth();
    let branch_factor = debugger.pipeline_dag.average_branching();
    1.0 / (1.0 + (pipeline_depth * branch_factor) / 100.0)
}
```

### 4. Component Toolkit (`ruchy wasm`)

Comprehensive WebAssembly component lifecycle management.

```rust
pub struct ComponentToolkit {
    wit_generator: WitGenerator,
    wasi_runtime: WasiRuntime,
    deployment_profiles: HashMap<Platform, DeploymentProfile>,
}

impl ComponentToolkit {
    pub fn generate_wit(&self, ruchy_type: &RuchyType) -> WitInterface {
        match ruchy_type {
            RuchyType::Actor(actor) => {
                WitInterface {
                    exports: actor.public_methods.map(|m| self.method_to_wit(m)),
                    imports: actor.dependencies.map(|d| self.dep_to_wit(d)),
                }
            }
            RuchyType::Refinement { base, predicate } => {
                let wit_base = self.generate_wit(base);
                self.inject_validation(wit_base, predicate)
            }
        }
    }
    
    pub fn deploy(&self, component: Component, platform: Platform) -> Result<DeploymentUrl> {
        let profile = self.deployment_profiles.get(&platform)?;
        let optimized = profile.optimize(component);
        let polyfilled = profile.polyfill_missing_features(optimized);
        profile.deploy(polyfilled).await
    }
}
```

**Score Integration:**
```rust
// Component portability affects idiomaticity score
fn score_wasm_idiomaticity(toolkit: &ComponentToolkit, component: &Component) -> f64 {
    let wit_compliance = toolkit.validate_wit_compliance(component);
    let platform_coverage = toolkit.count_supported_platforms(component) as f64 / 5.0;
    wit_compliance * 0.7 + platform_coverage * 0.3
}
```

